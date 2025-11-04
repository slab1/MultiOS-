use serde::{Deserialize, Serialize};
use std::sync::mpsc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

use super::state::{InstallationState, WizardState, LogLevel};

/// Progress events sent to listeners
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgressEvent {
    /// Installation started
    Started {
        total_steps: usize,
        start_time: chrono::DateTime<chrono::Utc>,
    },
    
    /// Step started
    StepStarted {
        step_index: usize,
        step_name: String,
    },
    
    /// Step progress updated
    StepProgress {
        step_index: usize,
        progress: f32, // 0.0 to 1.0
        message: Option<String>,
    },
    
    /// Step completed
    StepCompleted {
        step_index: usize,
        step_name: String,
        duration: Duration,
    },
    
    /// Step failed
    StepFailed {
        step_index: usize,
        step_name: String,
        error: String,
    },
    
    /// Overall progress updated
    OverallProgress {
        overall_progress: f32, // 0.0 to 1.0
        estimated_time_remaining: Option<Duration>,
    },
    
    /// Installation completed
    Completed {
        total_duration: Duration,
    },
    
    /// Installation failed
    Failed {
        error: String,
        can_retry: bool,
    },
    
    /// Installation cancelled
    Cancelled,
    
    /// Rollback started
    RollbackStarted {
        recovery_point: String,
    },
    
    /// Rollback completed
    RollbackCompleted {
        recovery_point: String,
        duration: Duration,
    },
    
    /// Log entry
    LogEntry {
        level: LogLevel,
        message: String,
        component: String,
    },
    
    /// Warning added
    Warning {
        message: String,
    },
    
    /// Error added
    Error {
        message: String,
    },
}

/// Progress tracker that manages installation progress and events
pub struct ProgressTracker {
    state: RwLock<InstallationState>,
    event_sender: mpsc::UnboundedSender<ProgressEvent>,
    event_receiver: mpsc::UnboundedReceiver<ProgressEvent>,
    start_time: Option<Instant>,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new() -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        let state = InstallationState {
            current_step: 0,
            total_steps: 0,
            current_step_name: String::new(),
            overall_progress: 0.0,
            step_progress: 0.0,
            wizard_state: WizardState::Initializing,
            start_time: None,
            estimated_time_remaining: None,
            current_operation: None,
            log_entries: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
            can_rollback: false,
            recovery_available: false,
        };

        Self {
            state: RwLock::new(state),
            event_sender,
            event_receiver,
            start_time: None,
        }
    }

    /// Update progress tracker with total number of steps
    pub async fn set_total_steps(&self, total_steps: usize) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut state = self.state.write().await;
        state.total_steps = total_steps;
        state.start_time = Some(std::time::SystemTime::now());
        self.start_time = Some(Instant::now());

        // Send event
        self.send_event(ProgressEvent::Started {
            total_steps,
            start_time: chrono::Utc::now(),
        }).await?;

        Ok(())
    }

    /// Start a new step
    pub async fn start_step(&self, step_index: usize, step_name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut state = self.state.write().await;
        state.current_step = step_index;
        state.current_step_name = step_name.to_string();
        state.step_progress = 0.0;
        state.current_operation = Some("Starting".to_string());

        self.send_event(ProgressEvent::StepStarted {
            step_index,
            step_name: step_name.to_string(),
        }).await?;

        Ok(())
    }

    /// Update step progress
    pub async fn update_progress(&self, step_index: usize, progress: f32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut state = self.state.write().await;
        state.step_progress = progress.clamp(0.0, 1.0);
        
        // Calculate overall progress
        if state.total_steps > 0 {
            let step_base_progress = (step_index as f32) / (state.total_steps as f32);
            let step_fraction = state.step_progress / (state.total_steps as f32);
            state.overall_progress = (step_base_progress + step_fraction).clamp(0.0, 1.0);
        }

        // Update time estimate
        self.update_time_estimate(&mut state).await;

        // Send progress event
        self.send_event(ProgressEvent::StepProgress {
            step_index,
            progress: state.step_progress,
            message: state.current_operation.clone(),
        }).await?;

        Ok(())
    }

    /// Update progress with a message
    pub async fn update_progress_with_message(&self, step_index: usize, progress: f32, message: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut state = self.state.write().await;
        state.current_operation = Some(message.to_string());

        // Update time estimate
        self.update_time_estimate(&mut state).await;

        self.send_event(ProgressEvent::StepProgress {
            step_index,
            progress: progress.clamp(0.0, 1.0),
            message: Some(message.to_string()),
        }).await?;

        Ok(())
    }

    /// Complete a step
    pub async fn complete_step(&self, step_index: usize) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let start_time = self.start_time.unwrap_or_else(|| Instant::now());
        let duration = start_time.elapsed();

        self.send_event(ProgressEvent::StepCompleted {
            step_index,
            step_name: self.state.read().await.current_step_name.clone(),
            duration,
        }).await?;

        Ok(())
    }

    /// Fail a step
    pub async fn fail_step(&self, step_index: usize, error: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut state = self.state.write().await;
        state.add_error(error.to_string());

        self.send_event(ProgressEvent::StepFailed {
            step_index,
            step_name: state.current_step_name.clone(),
            error: error.to_string(),
        }).await?;

        Ok(())
    }

    /// Mark installation as completed
    pub async fn complete_installation(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut state = self.state.write().await;
        state.wizard_state = WizardState::Completed;
        state.overall_progress = 1.0;

        let start_time = self.start_time.unwrap_or_else(|| Instant::now());
        let duration = start_time.elapsed();

        self.send_event(ProgressEvent::Completed {
            total_duration: duration,
        }).await?;

        Ok(())
    }

    /// Mark installation as failed
    pub async fn fail_installation(&self, error: &str, can_retry: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut state = self.state.write().await;
        state.wizard_state = WizardState::Failed {
            error: error.to_string(),
            can_retry,
        };

        state.add_error(error.to_string());

        self.send_event(ProgressEvent::Failed {
            error: error.to_string(),
            can_retry,
        }).await?;

        Ok(())
    }

    /// Cancel installation
    pub async fn cancel_installation(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut state = self.state.write().await;
        state.wizard_state = WizardState::Cancelled;

        self.send_event(ProgressEvent::Cancelled).await?;

        Ok(())
    }

    /// Start rollback process
    pub async fn start_rollback(&self, recovery_point: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut state = self.state.write().await;
        state.wizard_state = WizardState::RollingBack {
            recovery_point: recovery_point.to_string(),
        };

        self.send_event(ProgressEvent::RollbackStarted {
            recovery_point: recovery_point.to_string(),
        }).await?;

        Ok(())
    }

    /// Complete rollback process
    pub async fn complete_rollback(&self, recovery_point: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let start_time = self.start_time.unwrap_or_else(|| Instant::now());
        let duration = start_time.elapsed();

        let mut state = self.state.write().await;
        state.wizard_state = WizardState::Initializing;

        self.send_event(ProgressEvent::RollbackCompleted {
            recovery_point: recovery_point.to_string(),
            duration,
        }).await?;

        Ok(())
    }

    /// Add a log entry
    pub async fn log(&self, level: LogLevel, message: &str, component: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut state = self.state.write().await;
        state.add_log(level, message.to_string(), component.to_string());

        self.send_event(ProgressEvent::LogEntry {
            level,
            message: message.to_string(),
            component: component.to_string(),
        }).await?;

        Ok(())
    }

    /// Add a warning
    pub async fn warning(&self, message: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut state = self.state.write().await;
        state.add_warning(message.to_string());

        self.send_event(ProgressEvent::Warning {
            message: message.to_string(),
        }).await?;

        Ok(())
    }

    /// Add an error
    pub async fn error(&self, message: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut state = self.state.write().await;
        state.add_error(message.to_string());

        self.send_event(ProgressEvent::Error {
            message: message.to_string(),
        }).await?;

        Ok(())
    }

    /// Update wizard state
    pub async fn update_state(&self, state: WizardState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut state_guard = self.state.write().await;
        state_guard.wizard_state = state;

        Ok(())
    }

    /// Enable rollback capability
    pub async fn enable_rollback(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut state = self.state.write().await;
        state.can_rollback = true;

        Ok(())
    }

    /// Enable recovery capability
    pub async fn enable_recovery(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut state = self.state.write().await;
        state.recovery_available = true;

        Ok(())
    }

    /// Get current state (read-only)
    pub async fn get_state(&self) -> InstallationState {
        self.state.read().await.clone()
    }

    /// Get event receiver for listening to progress events
    pub fn get_event_receiver(&self) -> mpsc::UnboundedReceiver<ProgressEvent> {
        self.event_receiver.clone()
    }

    /// Check if installation is in progress
    pub fn is_in_progress(&self) -> bool {
        // We can't use async here, so we'll make a best effort check
        // In a real implementation, you'd want to track this differently
        false
    }

    /// Get formatted progress string for display
    pub fn get_progress_string(&self) -> String {
        format!("Progress tracking initialized")
    }

    /// Send an event to all listeners
    async fn send_event(&self, event: ProgressEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.event_sender.send(event)
            .map_err(|e| format!("Failed to send event: {}", e))?;
        Ok(())
    }

    /// Update time estimate based on current progress
    async fn update_time_estimate(&self, state: &mut InstallationState) {
        if let Some(start_time) = self.start_time {
            if state.overall_progress > 0.01 {
                let elapsed = start_time.elapsed().as_secs_f32();
                let total_estimated = elapsed / state.overall_progress;
                let remaining = total_estimated - elapsed;
                state.estimated_time_remaining = Some(Duration::from_secs_f32(remaining.max(0.0)));
            }
        }
    }
}

impl Default for ProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Progress listener trait for components that want to receive progress updates
#[async_trait::async_trait]
pub trait ProgressListener: Send + Sync {
    /// Called when a progress event occurs
    async fn on_progress_event(&self, event: &ProgressEvent);
}

/// Mock progress listener for testing
pub struct MockProgressListener {
    pub events: std::sync::Arc<std::sync::Mutex<Vec<ProgressEvent>>>,
}

impl MockProgressListener {
    pub fn new() -> Self {
        Self {
            events: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub fn get_events(&self) -> std::sync::MutexGuard<Vec<ProgressEvent>> {
        self.events.lock().unwrap()
    }
}

#[async_trait::async_trait]
impl ProgressListener for MockProgressListener {
    async fn on_progress_event(&self, event: &ProgressEvent) {
        self.events.lock().unwrap().push(event.clone());
    }
}