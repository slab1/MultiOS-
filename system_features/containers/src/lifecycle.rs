//! Container Lifecycle Management
//! 
//! This module provides comprehensive container lifecycle management including
//! create, start, stop, pause, resume, restart, and delete operations.

use super::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};
use tokio::time::{timeout, Duration};

/// Lifecycle Manager - Handles all container lifecycle operations
pub struct LifecycleManager {
    active_operations: Arc<Mutex<HashMap<String, LifecycleOperation>>>,
    event_sender: mpsc::UnboundedSender<LifecycleEvent>,
    event_receiver: Arc<Mutex<mpsc::UnboundedReceiver<LifecycleEvent>>>,
}

impl LifecycleManager {
    /// Create a new lifecycle manager
    pub fn new() -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            active_operations: Arc::new(Mutex::new(HashMap::new())),
            event_sender,
            event_receiver: Arc::new(Mutex::new(event_receiver)),
        }
    }

    /// Start the lifecycle manager event loop
    pub async fn start_event_loop(&self) {
        let mut receiver = self.event_receiver.lock().unwrap();
        
        loop {
            if let Some(event) = receiver.recv().await {
                self.handle_lifecycle_event(event).await;
            }
        }
    }

    /// Create and initialize a container
    pub async fn create_container(&self, container_id: &str, config: &ContainerConfig) -> ContainerResult<LifecycleResult> {
        log::info!("Creating container {}", container_id);

        // Check if operation is already in progress
        {
            let active_operations = self.active_operations.lock().unwrap();
            if active_operations.contains_key(container_id) {
                return Err(ContainerError::InvalidConfig(
                    format!("Container {} is already being operated on", container_id)
                ));
            }
        }

        // Register operation
        let operation = LifecycleOperation {
            id: format!("create-{}", container_id),
            operation_type: OperationType::Create,
            status: OperationStatus::InProgress,
            start_time: SystemTime::now(),
            end_time: None,
            progress: 0.0,
            details: HashMap::new(),
        };

        {
            let mut active_operations = self.active_operations.lock().unwrap();
            active_operations.insert(container_id.to_string(), operation);
        }

        // Send create event
        let event = LifecycleEvent {
            container_id: container_id.to_string(),
            event_type: EventType::Create,
            timestamp: SystemTime::now(),
            data: serde_json::to_value(config).unwrap(),
        };

        if let Err(_) = self.event_sender.send(event) {
            return Err(ContainerError::System("Failed to send create event".to_string()));
        }

        // Wait for operation to complete (with timeout)
        let result = timeout(Duration::from_secs(300), self.wait_for_operation(container_id, OperationType::Create)).await;

        match result {
            Ok(Ok(result)) => {
                log::info!("Container {} created successfully", container_id);
                Ok(result)
            },
            Ok(Err(e)) => {
                log::error!("Failed to create container {}: {}", container_id, e);
                Err(e)
            },
            Err(_) => {
                log::error!("Timeout creating container {}", container_id);
                Err(ContainerError::System("Timeout creating container".to_string()))
            }
        }
    }

    /// Start a created container
    pub async fn start_container(&self, container_id: &str) -> ContainerResult<LifecycleResult> {
        log::info!("Starting container {}", container_id);

        // Validate container state
        // This would typically involve checking that the container exists and is in the correct state

        // Register operation
        let operation = LifecycleOperation {
            id: format!("start-{}", container_id),
            operation_type: OperationType::Start,
            status: OperationStatus::InProgress,
            start_time: SystemTime::now(),
            end_time: None,
            progress: 0.0,
            details: HashMap::new(),
        };

        {
            let mut active_operations = self.active_operations.lock().unwrap();
            active_operations.insert(container_id.to_string(), operation);
        }

        // Send start event
        let event = LifecycleEvent {
            container_id: container_id.to_string(),
            event_type: EventType::Start,
            timestamp: SystemTime::now(),
            data: serde_json::Value::Null,
        };

        if let Err(_) = self.event_sender.send(event) {
            return Err(ContainerError::System("Failed to send start event".to_string()));
        }

        // Wait for operation to complete
        let result = timeout(Duration::from_secs(120), self.wait_for_operation(container_id, OperationType::Start)).await;

        match result {
            Ok(Ok(result)) => {
                log::info!("Container {} started successfully", container_id);
                Ok(result)
            },
            Ok(Err(e)) => {
                log::error!("Failed to start container {}: {}", container_id, e);
                Err(e)
            },
            Err(_) => {
                log::error!("Timeout starting container {}", container_id);
                Err(ContainerError::System("Timeout starting container".to_string()))
            }
        }
    }

    /// Stop a running container
    pub async fn stop_container(&self, container_id: &str, timeout: Option<Duration>) -> ContainerResult<LifecycleResult> {
        log::info!("Stopping container {}", container_id);

        let stop_timeout = timeout.unwrap_or(Duration::from_secs(30));

        // Register operation
        let operation = LifecycleOperation {
            id: format!("stop-{}", container_id),
            operation_type: OperationType::Stop,
            status: OperationStatus::InProgress,
            start_time: SystemTime::now(),
            end_time: None,
            progress: 0.0,
            details: [("timeout".to_string(), stop_timeout.as_secs().to_string())].iter().cloned().collect(),
        };

        {
            let mut active_operations = self.active_operations.lock().unwrap();
            active_operations.insert(container_id.to_string(), operation);
        }

        // Send stop event
        let event = LifecycleEvent {
            container_id: container_id.to_string(),
            event_type: EventType::Stop,
            timestamp: SystemTime::now(),
            data: serde_json::to_value(stop_timeout).unwrap(),
        };

        if let Err(_) = self.event_sender.send(event) {
            return Err(ContainerError::System("Failed to send stop event".to_string()));
        }

        // Wait for operation to complete
        let result = timeout(Duration::from_secs(stop_timeout.as_secs() + 10), 
                            self.wait_for_operation(container_id, OperationType::Stop)).await;

        match result {
            Ok(Ok(result)) => {
                log::info!("Container {} stopped successfully", container_id);
                Ok(result)
            },
            Ok(Err(e)) => {
                log::error!("Failed to stop container {}: {}", container_id, e);
                Err(e)
            },
            Err(_) => {
                log::error!("Timeout stopping container {}", container_id);
                Err(ContainerError::System("Timeout stopping container".to_string()))
            }
        }
    }

    /// Pause a running container
    pub async fn pause_container(&self, container_id: &str) -> ContainerResult<LifecycleResult> {
        log::info!("Pausing container {}", container_id);

        let operation = LifecycleOperation {
            id: format!("pause-{}", container_id),
            operation_type: OperationType::Pause,
            status: OperationStatus::InProgress,
            start_time: SystemTime::now(),
            end_time: None,
            progress: 0.0,
            details: HashMap::new(),
        };

        {
            let mut active_operations = self.active_operations.lock().unwrap();
            active_operations.insert(container_id.to_string(), operation);
        }

        let event = LifecycleEvent {
            container_id: container_id.to_string(),
            event_type: EventType::Pause,
            timestamp: SystemTime::now(),
            data: serde_json::Value::Null,
        };

        if let Err(_) = self.event_sender.send(event) {
            return Err(ContainerError::System("Failed to send pause event".to_string()));
        }

        let result = timeout(Duration::from_secs(60), self.wait_for_operation(container_id, OperationType::Pause)).await;

        match result {
            Ok(Ok(result)) => {
                log::info!("Container {} paused successfully", container_id);
                Ok(result)
            },
            Ok(Err(e)) => {
                log::error!("Failed to pause container {}: {}", container_id, e);
                Err(e)
            },
            Err(_) => {
                log::error!("Timeout pausing container {}", container_id);
                Err(ContainerError::System("Timeout pausing container".to_string()))
            }
        }
    }

    /// Resume a paused container
    pub async fn resume_container(&self, container_id: &str) -> ContainerResult<LifecycleResult> {
        log::info!("Resuming container {}", container_id);

        let operation = LifecycleOperation {
            id: format!("resume-{}", container_id),
            operation_type: OperationType::Resume,
            status: OperationStatus::InProgress,
            start_time: SystemTime::now(),
            end_time: None,
            progress: 0.0,
            details: HashMap::new(),
        };

        {
            let mut active_operations = self.active_operations.lock().unwrap();
            active_operations.insert(container_id.to_string(), operation);
        }

        let event = LifecycleEvent {
            container_id: container_id.to_string(),
            event_type: EventType::Resume,
            timestamp: SystemTime::now(),
            data: serde_json::Value::Null,
        };

        if let Err(_) = self.event_sender.send(event) {
            return Err(ContainerError::System("Failed to send resume event".to_string()));
        }

        let result = timeout(Duration::from_secs(60), self.wait_for_operation(container_id, OperationType::Resume)).await;

        match result {
            Ok(Ok(result)) => {
                log::info!("Container {} resumed successfully", container_id);
                Ok(result)
            },
            Ok(Err(e)) => {
                log::error!("Failed to resume container {}: {}", container_id, e);
                Err(e)
            },
            Err(_) => {
                log::error!("Timeout resuming container {}", container_id);
                Err(ContainerError::System("Timeout resuming container".to_string()))
            }
        }
    }

    /// Restart a container
    pub async fn restart_container(&self, container_id: &str, timeout: Option<Duration>) -> ContainerResult<LifecycleResult> {
        log::info!("Restarting container {}", container_id);

        let restart_timeout = timeout.unwrap_or(Duration::from_secs(60));

        let operation = LifecycleOperation {
            id: format!("restart-{}", container_id),
            operation_type: OperationType::Restart,
            status: OperationStatus::InProgress,
            start_time: SystemTime::now(),
            end_time: None,
            progress: 0.0,
            details: [("timeout".to_string(), restart_timeout.as_secs().to_string())].iter().cloned().collect(),
        };

        {
            let mut active_operations = self.active_operations.lock().unwrap();
            active_operations.insert(container_id.to_string(), operation);
        }

        let event = LifecycleEvent {
            container_id: container_id.to_string(),
            event_type: EventType::Restart,
            timestamp: SystemTime::now(),
            data: serde_json::to_value(restart_timeout).unwrap(),
        };

        if let Err(_) = self.event_sender.send(event) {
            return Err(ContainerError::System("Failed to send restart event".to_string()));
        }

        let result = timeout(Duration::from_secs(restart_timeout.as_secs() + 10), 
                            self.wait_for_operation(container_id, OperationType::Restart)).await;

        match result {
            Ok(Ok(result)) => {
                log::info!("Container {} restarted successfully", container_id);
                Ok(result)
            },
            Ok(Err(e)) => {
                log::error!("Failed to restart container {}: {}", container_id, e);
                Err(e)
            },
            Err(_) => {
                log::error!("Timeout restarting container {}", container_id);
                Err(ContainerError::System("Timeout restarting container".to_string()))
            }
        }
    }

    /// Delete a container
    pub async fn delete_container(&self, container_id: &str, force: bool) -> ContainerResult<LifecycleResult> {
        log::info!("Deleting container {}", container_id);

        let operation = LifecycleOperation {
            id: format!("delete-{}", container_id),
            operation_type: OperationType::Delete,
            status: OperationStatus::InProgress,
            start_time: SystemTime::now(),
            end_time: None,
            progress: 0.0,
            details: [("force".to_string(), force.to_string())].iter().cloned().collect(),
        };

        {
            let mut active_operations = self.active_operations.lock().unwrap();
            active_operations.insert(container_id.to_string(), operation);
        }

        let event = LifecycleEvent {
            container_id: container_id.to_string(),
            event_type: EventType::Delete,
            timestamp: SystemTime::now(),
            data: serde_json::to_value(force).unwrap(),
        };

        if let Err(_) = self.event_sender.send(event) {
            return Err(ContainerError::System("Failed to send delete event".to_string()));
        }

        let result = timeout(Duration::from_secs(300), self.wait_for_operation(container_id, OperationType::Delete)).await;

        match result {
            Ok(Ok(result)) => {
                log::info!("Container {} deleted successfully", container_id);
                Ok(result)
            },
            Ok(Err(e)) => {
                log::error!("Failed to delete container {}: {}", container_id, e);
                Err(e)
            },
            Err(_) => {
                log::error!("Timeout deleting container {}", container_id);
                Err(ContainerError::System("Timeout deleting container".to_string()))
            }
        }
    }

    /// Get operation status for a container
    pub fn get_operation_status(&self, container_id: &str) -> Option<&LifecycleOperation> {
        let active_operations = self.active_operations.lock().unwrap();
        active_operations.get(container_id)
    }

    /// List all active operations
    pub fn list_active_operations(&self) -> Vec<LifecycleOperation> {
        let active_operations = self.active_operations.lock().unwrap();
        active_operations.values().cloned().collect()
    }

    // Private helper methods

    async fn handle_lifecycle_event(&self, event: LifecycleEvent) {
        log::debug!("Handling lifecycle event: {:?} for container {}", event.event_type, event.container_id);

        match event.event_type {
            EventType::Create => self.handle_create_event(event).await,
            EventType::Start => self.handle_start_event(event).await,
            EventType::Stop => self.handle_stop_event(event).await,
            EventType::Pause => self.handle_pause_event(event).await,
            EventType::Resume => self.handle_resume_event(event).await,
            EventType::Restart => self.handle_restart_event(event).await,
            EventType::Delete => self.handle_delete_event(event).await,
            EventType::StateChange => self.handle_state_change_event(event).await,
        }
    }

    async fn handle_create_event(&self, event: LifecycleEvent) {
        let container_id = &event.container_id;
        
        // Simulate creation process
        self.update_progress(container_id, 0.0).await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        self.update_progress(container_id, 0.3).await;

        // Additional setup steps would go here
        tokio::time::sleep(Duration::from_millis(100)).await;
        self.update_progress(container_id, 0.7).await;

        tokio::time::sleep(Duration::from_millis(100)).await;
        self.update_progress(container_id, 1.0).await;

        self.complete_operation(container_id, Ok(LifecycleResult {
            container_id: container_id.clone(),
            operation: OperationType::Create,
            success: true,
            message: "Container created successfully".to_string(),
            duration: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default(),
        }));
    }

    async fn handle_start_event(&self, event: LifecycleEvent) {
        let container_id = &event.container_id;
        
        self.update_progress(container_id, 0.0).await;
        
        // Start container process
        // This would involve the actual container startup logic
        
        tokio::time::sleep(Duration::from_millis(200)).await;
        self.update_progress(container_id, 0.5).await;

        tokio::time::sleep(Duration::from_millis(200)).await;
        self.update_progress(container_id, 1.0).await;

        self.complete_operation(container_id, Ok(LifecycleResult {
            container_id: container_id.clone(),
            operation: OperationType::Start,
            success: true,
            message: "Container started successfully".to_string(),
            duration: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default(),
        }));
    }

    async fn handle_stop_event(&self, event: LifecycleEvent) {
        let container_id = &event.container_id;
        let timeout: Duration = serde_json::from_value(event.data).unwrap_or(Duration::from_secs(30));
        
        self.update_progress(container_id, 0.0).await;
        
        // Send SIGTERM first
        // This would send the termination signal to the container process
        
        tokio::time::sleep(Duration::from_millis(100)).await;
        self.update_progress(container_id, 0.3).await;

        // Wait for graceful shutdown
        let wait_start = SystemTime::now();
        while SystemTime::now().duration_since(wait_start).unwrap_or_default() < timeout {
            tokio::time::sleep(Duration::from_millis(100)).await;
            self.update_progress(container_id, 0.5).await;
            
            // Check if process has terminated
            // This would involve checking if the container process is still running
        }

        // Force kill if still running
        // This would send SIGKILL if the process hasn't terminated gracefully
        
        tokio::time::sleep(Duration::from_millis(100)).await;
        self.update_progress(container_id, 0.8).await;

        tokio::time::sleep(Duration::from_millis(100)).await;
        self.update_progress(container_id, 1.0).await;

        self.complete_operation(container_id, Ok(LifecycleResult {
            container_id: container_id.clone(),
            operation: OperationType::Stop,
            success: true,
            message: "Container stopped successfully".to_string(),
            duration: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default(),
        }));
    }

    async fn handle_pause_event(&self, event: LifecycleEvent) {
        let container_id = &event.container_id;
        
        self.update_progress(container_id, 0.0).await;
        
        // Send SIGSTOP to container process
        // This would pause all processes in the container
        
        tokio::time::sleep(Duration::from_millis(200)).await;
        self.update_progress(container_id, 1.0).await;

        self.complete_operation(container_id, Ok(LifecycleResult {
            container_id: container_id.clone(),
            operation: OperationType::Pause,
            success: true,
            message: "Container paused successfully".to_string(),
            duration: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default(),
        }));
    }

    async fn handle_resume_event(&self, event: LifecycleEvent) {
        let container_id = &event.container_id;
        
        self.update_progress(container_id, 0.0).await;
        
        // Send SIGCONT to container process
        // This would resume all paused processes in the container
        
        tokio::time::sleep(Duration::from_millis(200)).await;
        self.update_progress(container_id, 1.0).await;

        self.complete_operation(container_id, Ok(LifecycleResult {
            container_id: container_id.clone(),
            operation: OperationType::Resume,
            success: true,
            message: "Container resumed successfully".to_string(),
            duration: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default(),
        }));
    }

    async fn handle_restart_event(&self, event: LifecycleEvent) {
        let container_id = &event.container_id;
        let timeout: Duration = serde_json::from_value(event.data).unwrap_or(Duration::from_secs(60));
        
        self.update_progress(container_id, 0.0).await;
        
        // Stop container first
        let stop_duration = self.stop_container_internal(container_id, timeout).await;
        
        tokio::time::sleep(Duration::from_millis(200)).await;
        self.update_progress(container_id, 0.5).await;

        // Start container again
        // This would restart the container process
        
        tokio::time::sleep(Duration::from_millis(200)).await;
        self.update_progress(container_id, 1.0).await;

        self.complete_operation(container_id, Ok(LifecycleResult {
            container_id: container_id.clone(),
            operation: OperationType::Restart,
            success: true,
            message: "Container restarted successfully".to_string(),
            duration: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default(),
        }));
    }

    async fn handle_delete_event(&self, event: LifecycleEvent) {
        let container_id = &event.container_id;
        let force: bool = serde_json::from_value(event.data).unwrap_or(false);
        
        self.update_progress(container_id, 0.0).await;
        
        // Stop container if running (unless force)
        if !force {
            // This would check container state and stop if necessary
            self.stop_container_internal(container_id, Duration::from_secs(10)).await;
        }
        
        tokio::time::sleep(Duration::from_millis(100)).await;
        self.update_progress(container_id, 0.3).await;

        // Cleanup resources
        // This would clean up namespaces, cgroups, network interfaces, etc.
        
        tokio::time::sleep(Duration::from_millis(200)).await;
        self.update_progress(container_id, 0.7).await;

        tokio::time::sleep(Duration::from_millis(100)).await;
        self.update_progress(container_id, 1.0).await;

        self.complete_operation(container_id, Ok(LifecycleResult {
            container_id: container_id.clone(),
            operation: OperationType::Delete,
            success: true,
            message: "Container deleted successfully".to_string(),
            duration: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default(),
        }));
    }

    async fn handle_state_change_event(&self, event: LifecycleEvent) {
        // Handle state change events from the container runtime
        log::info!("Container {} state changed", event.container_id);
        // This would update container state and notify listeners
    }

    async fn update_progress(&self, container_id: &str, progress: f32) {
        {
            let mut active_operations = self.active_operations.lock().unwrap();
            if let Some(operation) = active_operations.get_mut(container_id) {
                operation.progress = progress;
            }
        }
        
        // Send progress event if needed
        log::debug!("Container {} operation progress: {:.1}%", container_id, progress * 100.0);
    }

    fn complete_operation(&self, container_id: &str, result: ContainerResult<LifecycleResult>) {
        let operation_result = match result {
            Ok(ref success) => OperationStatus::Completed,
            Err(_) => OperationStatus::Failed,
        };

        {
            let mut active_operations = self.active_operations.lock().unwrap();
            if let Some(operation) = active_operations.get_mut(container_id) {
                operation.status = operation_result;
                operation.end_time = Some(SystemTime::now());
            }
        }

        log::info!("Container {} operation completed: {:?}", container_id, operation_result);
    }

    async fn wait_for_operation(&self, container_id: &str, operation_type: OperationType) -> ContainerResult<LifecycleResult> {
        // Wait for the operation to complete
        loop {
            {
                let active_operations = self.active_operations.lock().unwrap();
                if let Some(operation) = active_operations.get(container_id) {
                    if operation.operation_type == operation_type {
                        match operation.status {
                            OperationStatus::Completed => {
                                return Ok(LifecycleResult {
                                    container_id: container_id.to_string(),
                                    operation: operation_type,
                                    success: true,
                                    message: "Operation completed successfully".to_string(),
                                    duration: SystemTime::now().duration_since(operation.start_time).unwrap_or_default(),
                                });
                            },
                            OperationStatus::Failed => {
                                return Err(ContainerError::System("Operation failed".to_string()));
                            },
                            OperationStatus::InProgress => {
                                // Continue waiting
                            },
                            OperationStatus::Cancelled => {
                                return Err(ContainerError::System("Operation was cancelled".to_string()));
                            }
                        }
                    }
                }
            }
            
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    async fn stop_container_internal(&self, container_id: &str, timeout: Duration) {
        // Internal stop implementation
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// Lifecycle operation information
#[derive(Debug, Clone)]
pub struct LifecycleOperation {
    pub id: String,
    pub operation_type: OperationType,
    pub status: OperationStatus,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub progress: f32,
    pub details: HashMap<String, String>,
}

/// Operation types
#[derive(Debug, Clone)]
pub enum OperationType {
    Create,
    Start,
    Stop,
    Pause,
    Resume,
    Restart,
    Delete,
}

/// Operation status
#[derive(Debug, Clone, PartialEq)]
pub enum OperationStatus {
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Lifecycle event definition
#[derive(Debug, Clone)]
pub struct LifecycleEvent {
    pub container_id: String,
    pub event_type: EventType,
    pub timestamp: SystemTime,
    pub data: serde_json::Value,
}

/// Event types
#[derive(Debug, Clone)]
pub enum EventType {
    Create,
    Start,
    Stop,
    Pause,
    Resume,
    Restart,
    Delete,
    StateChange,
}

/// Lifecycle operation result
#[derive(Debug, Clone)]
pub struct LifecycleResult {
    pub container_id: String,
    pub operation: OperationType,
    pub success: bool,
    pub message: String,
    pub duration: Duration,
}

/// Container state transition
#[derive(Debug, Clone)]
pub struct StateTransition {
    pub from_state: ContainerState,
    pub to_state: ContainerState,
    pub operation: OperationType,
    pub allowed: bool,
    pub conditions: Vec<String>,
}

/// Lifecycle hook configuration
#[derive(Debug, Clone)]
pub struct LifecycleHook {
    pub event: EventType,
    pub command: Vec<String>,
    pub timeout: Duration,
    pub retry_count: usize,
    pub environment: HashMap<String, String>,
}

/// Container state machine
pub struct StateMachine {
    transitions: HashMap<(ContainerState, OperationType), StateTransition>,
}

impl StateMachine {
    pub fn new() -> Self {
        let mut transitions = HashMap::new();

        // Define allowed state transitions
        let transition_rules = [
            (ContainerState::Created, OperationType::Start, ContainerState::Running),
            (ContainerState::Running, OperationType::Stop, ContainerState::Stopped),
            (ContainerState::Running, OperationType::Pause, ContainerState::Paused),
            (ContainerState::Paused, OperationType::Resume, ContainerState::Running),
            (ContainerState::Stopped, OperationType::Start, ContainerState::Running),
            (ContainerState::Exited, OperationType::Start, ContainerState::Running),
            (ContainerState::Created, OperationType::Delete, ContainerState::Deleted),
            (ContainerState::Stopped, OperationType::Delete, ContainerState::Deleted),
            (ContainerState::Exited, OperationType::Delete, ContainerState::Deleted),
            (ContainerState::Running, OperationType::Restart, ContainerState::Running),
            (ContainerState::Paused, OperationType::Restart, ContainerState::Running),
        ];

        for (from_state, operation, to_state) in transition_rules {
            transitions.insert(
                (from_state.clone(), operation.clone()),
                StateTransition {
                    from_state,
                    to_state,
                    operation,
                    allowed: true,
                    conditions: vec![],
                }
            );
        }

        Self { transitions }
    }

    pub fn is_transition_allowed(&self, current_state: &ContainerState, operation: &OperationType) -> bool {
        self.transitions.get(&(current_state.clone(), operation.clone()))
            .map(|t| t.allowed)
            .unwrap_or(false)
    }
}