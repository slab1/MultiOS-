//! UI Debugging Module
//!
//! Provides comprehensive debugging tools for MultiOS UI components including
//! DOM inspection, event monitoring, state tracking, and performance analysis.

use super::{FrameworkResult, FrameworkError, UIFrameworkConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::time::{Duration, Instant};
use log::info;

/// Debug session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugSession {
    pub session_id: String,
    pub component_name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub debug_data: DebugData,
    pub events_captured: Vec<DebugEvent>,
    pub state_changes: Vec<StateChange>,
    pub performance_metrics: Vec<PerformanceMetric>,
    pub issues_detected: Vec<DebugIssue>,
    pub screenshots: Vec<String>,
    pub logs: Vec<DebugLog>,
}

/// Debug data collected during session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugData {
    pub dom_snapshot: DOMSnapshot,
    pub style_snapshot: StyleSnapshot,
    pub properties: HashMap<String, String>,
    pub layout_info: LayoutInfo,
    pub accessibility_info: AccessibilityDebugInfo,
    pub performance_info: PerformanceDebugInfo,
}

/// DOM snapshot for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DOMSnapshot {
    pub structure: String,
    pub element_count: u32,
    pub node_types: HashMap<String, u32>,
    pub depth_levels: Vec<u32>,
    pub attributes: HashMap<String, String>,
    pub content: String,
}

/// Style snapshot for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleSnapshot {
    pub css_rules: Vec<CSSRule>,
    pub computed_styles: HashMap<String, StyleProperty>,
    pub layout_properties: HashMap<String, String>,
    pub animation_properties: HashMap<String, String>,
    pub media_queries_applied: Vec<String>,
}

/// CSS rule debugging information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CSSRule {
    pub selector: String,
    pub properties: HashMap<String, String>,
    pub specificity: String,
    pub source: String,
    pub line_number: u32,
}

/// Style property debugging information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleProperty {
    pub name: String,
    pub value: String,
    pub priority: String,
    pub source: String,
    pub inherited: bool,
}

/// Layout debugging information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutInfo {
    pub bounding_box: BoundingBox,
    pub display_mode: String,
    pub position: String,
    pub size: Size,
    pub margin: BoxEdges,
    pub padding: BoxEdges,
    pub border: BoxEdges,
    pub z_index: i32,
    pub overflow: String,
}

/// Bounding box information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub top: f32,
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub width: f32,
    pub height: f32,
}

/// Size information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Size {
    pub width: f32,
    pub height: f32,
    pub min_width: f32,
    pub min_height: f32,
    pub max_width: f32,
    pub max_height: f32,
}

/// Box edges for margin, padding, border
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxEdges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

/// Accessibility debug information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityDebugInfo {
    pub aria_role: Option<String>,
    pub aria_label: Option<String>,
    pub aria_describedby: Vec<String>,
    pub aria_hidden: bool,
    pub tab_index: i32,
    pub focusable: bool,
    pub keyboard_accessible: bool,
    pub screen_reader_text: Option<String>,
}

/// Performance debug information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDebugInfo {
    pub render_time_ms: f64,
    pub paint_time_ms: f64,
    pub layout_time_ms: f64,
    pub script_time_ms: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub fps: f32,
    pub resource_usage: HashMap<String, f64>,
}

/// Debug event captured during session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugEvent {
    pub event_id: String,
    pub event_type: DebugEventType,
    pub timestamp: DateTime<Utc>,
    pub source_element: Option<String>,
    pub target_element: Option<String>,
    pub event_data: HashMap<String, String>,
    pub bubble_phase: bool,
    pub capture_phase: bool,
    pub target_phase: bool,
}

/// Types of debug events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugEventType {
    Click,
    MouseMove,
    MouseEnter,
    MouseLeave,
    KeyDown,
    KeyUp,
    Focus,
    Blur,
    Resize,
    Scroll,
    Load,
    Error,
    Custom(String),
}

/// State change debug information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    pub change_id: String,
    pub property_name: String,
    pub old_value: String,
    pub new_value: String,
    pub timestamp: DateTime<Utc>,
    pub trigger: StateChangeTrigger,
    pub component: String,
}

/// State change triggers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateChangeTrigger {
    UserInteraction,
    Programmatic,
    Timer,
    NetworkEvent,
    Lifecycle,
    External,
}

/// Performance metric debug information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    pub metric_name: String,
    pub value: f64,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
    pub component: String,
    pub measurement_type: PerformanceMeasurementType,
}

/// Performance measurement types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceMeasurementType {
    TimeToFirstPaint,
    TimeToInteractive,
    CumulativeLayoutShift,
    FirstInputDelay,
    LargestContentfulPaint,
    RenderTime,
    PaintTime,
    ScriptTime,
    MemoryUsage,
    CpuUsage,
    NetworkRequest,
    Custom(String),
}

/// Debug issue detected during analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugIssue {
    pub issue_id: String,
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub title: String,
    pub description: String,
    pub element: Option<String>,
    pub suggestion: String,
    pub auto_fixable: bool,
    pub related_metrics: Vec<String>,
}

/// Debug issue severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Debug issue categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueCategory {
    Performance,
    Accessibility,
    Layout,
    Styling,
    Functionality,
    Security,
    Usability,
    Compatibility,
}

/// Debug log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugLog {
    pub log_id: String,
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub component: String,
    pub message: String,
    pub data: HashMap<String, String>,
}

/// Log levels for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// Debug configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    pub enable_dom_inspection: bool,
    pub enable_event_monitoring: bool,
    pub enable_state_tracking: bool,
    pub enable_performance_monitoring: bool,
    pub enable_screenshot_capture: bool,
    pub enable_logging: bool,
    pub screenshot_interval_ms: u64,
    pub max_log_entries: usize,
    pub performance_sampling_rate: f64,
    pub capture_video: bool,
    pub capture_network_traffic: bool,
    pub enable_memory_profiling: bool,
    pub debug_level: LogLevel,
}

/// UI debugger for comprehensive debugging
pub struct UIDebugger {
    config: UIFrameworkConfig,
    debug_config: DebugConfig,
    active_sessions: HashMap<String, DebugSession>,
    session_counter: u32,
    event_listeners: HashMap<String, Vec<Box<dyn DebugEventListener + Send + Sync>>>,
}

/// Debug event listener interface
pub trait DebugEventListener {
    fn on_event(&self, event: &DebugEvent);
    fn on_state_change(&self, change: &StateChange);
    fn on_issue_detected(&self, issue: &DebugIssue);
    fn on_performance_metric(&self, metric: &PerformanceMetric);
}

/// UI debugger implementation
impl UIDebugger {
    /// Create a new UI debugger
    pub fn new(config: &UIFrameworkConfig) -> Self {
        let debug_config = DebugConfig {
            enable_dom_inspection: true,
            enable_event_monitoring: true,
            enable_state_tracking: true,
            enable_performance_monitoring: true,
            enable_screenshot_capture: true,
            enable_logging: true,
            screenshot_interval_ms: 1000,
            max_log_entries: 1000,
            performance_sampling_rate: 1.0,
            capture_video: false,
            capture_network_traffic: false,
            enable_memory_profiling: false,
            debug_level: LogLevel::Info,
        };

        Self {
            config: config.clone(),
            debug_config,
            active_sessions: HashMap::new(),
            session_counter: 0,
            event_listeners: HashMap::new(),
        }
    }

    /// Set custom debug configuration
    pub fn set_debug_config(&mut self, debug_config: DebugConfig) {
        self.debug_config = debug_config;
    }

    /// Start a debug session for a component
    pub fn start_debug_session(&mut self, component_name: String) -> FrameworkResult<String> {
        let session_id = format!("session_{}", self.session_counter);
        self.session_counter += 1;

        info!("Starting debug session: {} for component: {}", session_id, component_name);

        // Capture initial state
        let initial_data = self.capture_debug_data(&component_name).await?;

        let session = DebugSession {
            session_id: session_id.clone(),
            component_name: component_name.clone(),
            start_time: Utc::now(),
            end_time: None,
            debug_data: initial_data,
            events_captured: Vec::new(),
            state_changes: Vec::new(),
            performance_metrics: Vec::new(),
            issues_detected: Vec::new(),
            screenshots: Vec::new(),
            logs: Vec::new(),
        };

        self.active_sessions.insert(session_id.clone(), session);

        // Set up monitoring if enabled
        if self.debug_config.enable_event_monitoring {
            self.setup_event_monitoring(&session_id, &component_name).await?;
        }

        if self.debug_config.enable_performance_monitoring {
            self.start_performance_monitoring(&session_id).await?;
        }

        Ok(session_id)
    }

    /// End a debug session
    pub fn end_debug_session(&mut self, session_id: &str) -> FrameworkResult<DebugSession> {
        let session = self.active_sessions.remove(session_id)
            .ok_or_else(|| FrameworkError::Debugging(format!("Debug session not found: {}", session_id)))?;

        info!("Ending debug session: {}", session_id);

        // Run final analysis
        let mut final_session = session;
        final_session.end_time = Some(Utc::now());

        // Analyze collected data
        self.analyze_debug_data(&mut final_session).await?;

        Ok(final_session)
    }

    /// Capture debug data for a component
    async fn capture_debug_data(&self, component_name: &str) -> FrameworkResult<DebugData> {
        // Simulate DOM inspection
        let dom_snapshot = DOMSnapshot {
            structure: self.generate_dom_structure(component_name).await?,
            element_count: 25,
            node_types: {
                let mut types = HashMap::new();
                types.insert("div".to_string(), 10);
                types.insert("button".to_string(), 5);
                types.insert("input".to_string(), 3);
                types.insert("span".to_string(), 7);
                types
            },
            depth_levels: vec![1, 2, 3, 2, 1],
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert("class".to_string(), "ui-component".to_string());
                attrs.insert("id".to_string(), component_name.to_string());
                attrs
            },
            content: format!("Component content for {}", component_name),
        };

        // Simulate style inspection
        let style_snapshot = StyleSnapshot {
            css_rules: vec![
                CSSRule {
                    selector: ".ui-component".to_string(),
                    properties: {
                        let mut props = HashMap::new();
                        props.insert("color".to_string(), "#333".to_string());
                        props.insert("font-size".to_string(), "14px".to_string());
                        props
                    },
                    specificity: "0,1,0".to_string(),
                    source: "style.css".to_string(),
                    line_number: 42,
                }
            ],
            computed_styles: {
                let mut styles = HashMap::new();
                styles.insert("display".to_string(), StyleProperty {
                    name: "display".to_string(),
                    value: "block".to_string(),
                    priority: "author".to_string(),
                    source: "style.css:43".to_string(),
                    inherited: false,
                });
                styles
            },
            layout_properties: {
                let mut layout = HashMap::new();
                layout.insert("position".to_string(), "static".to_string());
                layout.insert("overflow".to_string(), "visible".to_string());
                layout
            },
            animation_properties: HashMap::new(),
            media_queries_applied: vec!["(min-width: 768px)".to_string()],
        };

        // Get accessibility info
        let accessibility_info = AccessibilityDebugInfo {
            aria_role: Some("button".to_string()),
            aria_label: Some(format!("{} button", component_name)),
            aria_describedby: Vec::new(),
            aria_hidden: false,
            tab_index: 0,
            focusable: true,
            keyboard_accessible: true,
            screen_reader_text: Some(format!("Interactive button: {}", component_name)),
        };

        // Get performance info
        let performance_info = PerformanceDebugInfo {
            render_time_ms: 15.5,
            paint_time_ms: 8.2,
            layout_time_ms: 4.1,
            script_time_ms: 3.2,
            memory_usage_mb: 2.5,
            cpu_usage_percent: 15.0,
            fps: 60.0,
            resource_usage: {
                let mut usage = HashMap::new();
                usage.insert("cpu".to_string(), 15.0);
                usage.insert("memory".to_string(), 2.5);
                usage.insert("gpu".to_string(), 5.0);
                usage
            },
        };

        // Get layout info
        let layout_info = LayoutInfo {
            bounding_box: BoundingBox {
                top: 10.0,
                left: 20.0,
                right: 120.0,
                bottom: 40.0,
                width: 100.0,
                height: 30.0,
            },
            display_mode: "block".to_string(),
            position: "static".to_string(),
            size: Size {
                width: 100.0,
                height: 30.0,
                min_width: 0.0,
                min_height: 0.0,
                max_width: f32::INFINITY,
                max_height: f32::INFINITY,
            },
            margin: BoxEdges {
                top: 0.0,
                right: 0.0,
                bottom: 8.0,
                left: 0.0,
            },
            padding: BoxEdges {
                top: 8.0,
                right: 16.0,
                bottom: 8.0,
                left: 16.0,
            },
            border: BoxEdges {
                top: 1.0,
                right: 1.0,
                bottom: 1.0,
                left: 1.0,
            },
            z_index: 0,
            overflow: "visible".to_string(),
        };

        Ok(DebugData {
            dom_snapshot,
            style_snapshot,
            properties: {
                let mut props = HashMap::new();
                props.insert("visible".to_string(), "true".to_string());
                props.insert("enabled".to_string(), "true".to_string());
                props.insert("focusable".to_string(), "true".to_string());
                props
            },
            layout_info,
            accessibility_info,
            performance_info,
        })
    }

    /// Generate DOM structure representation
    async fn generate_dom_structure(&self, component_name: &str) -> FrameworkResult<String> {
        // Simplified DOM structure generation
        Ok(format!(
            r#"<div class="ui-component" id="{}">
  <button class="btn-primary">Click Me</button>
  <input type="text" class="input-field" />
  <span class="label">Component: {}</span>
</div>"#,
            component_name, component_name
        ))
    }

    /// Set up event monitoring for a session
    async fn setup_event_monitoring(&mut self, session_id: &str, component_name: &str) -> FrameworkResult<()> {
        info!("Setting up event monitoring for session: {}", session_id);

        // Simulate event listener setup
        let events_to_monitor = vec![
            DebugEventType::Click,
            DebugEventType::MouseMove,
            DebugEventType::KeyDown,
            DebugEventType::Focus,
            DebugEventType::Resize,
        ];

        for event_type in events_to_monitor {
            let event = DebugEvent {
                event_id: Uuid::new_v4().to_string(),
                event_type,
                timestamp: Utc::now(),
                source_element: Some(component_name.to_string()),
                target_element: Some(component_name.to_string()),
                event_data: {
                    let mut data = HashMap::new();
                    data.insert("session_id".to_string(), session_id.to_string());
                    data.insert("component".to_string(), component_name.to_string());
                    data
                },
                bubble_phase: true,
                capture_phase: false,
                target_phase: true,
            };

            if let Some(session) = self.active_sessions.get_mut(session_id) {
                session.events_captured.push(event);
            }
        }

        Ok(())
    }

    /// Start performance monitoring
    async fn start_performance_monitoring(&mut self, session_id: &str) -> FrameworkResult<()> {
        info!("Starting performance monitoring for session: {}", session_id);

        let metrics = vec![
            PerformanceMetric {
                metric_name: "render_time".to_string(),
                value: 15.5,
                unit: "ms".to_string(),
                timestamp: Utc::now(),
                component: session_id.to_string(),
                measurement_type: PerformanceMeasurementType::RenderTime,
            },
            PerformanceMetric {
                metric_name: "memory_usage".to_string(),
                value: 2.5,
                unit: "MB".to_string(),
                timestamp: Utc::now(),
                component: session_id.to_string(),
                measurement_type: PerformanceMeasurementType::MemoryUsage,
            },
        ];

        if let Some(session) = self.active_sessions.get_mut(session_id) {
            session.performance_metrics.extend(metrics);
        }

        Ok(())
    }

    /// Capture a screenshot during debugging
    pub async fn capture_screenshot(&mut self, session_id: &str) -> FrameworkResult<String> {
        if let Some(session) = self.active_sessions.get_mut(session_id) {
            let screenshot_path = format!("debug_{}_{}.png", session_id, session.screenshots.len());
            
            // Simulate screenshot capture
            info!("Capturing screenshot for session: {}", session_id);
            
            session.screenshots.push(screenshot_path);
            Ok(screenshot_path)
        } else {
            Err(FrameworkError::Debugging(format!("Debug session not found: {}", session_id)))
        }
    }

    /// Log debug information
    pub fn log_debug(&mut self, session_id: &str, level: LogLevel, component: String, message: String, data: HashMap<String, String>) -> FrameworkResult<()> {
        if let Some(session) = self.active_sessions.get_mut(session_id) {
            let log_entry = DebugLog {
                log_id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                level,
                component,
                message,
                data,
            };

            session.logs.push(log_entry);
            
            // Maintain max log entries
            if session.logs.len() > self.debug_config.max_log_entries {
                session.logs.remove(0);
            }
        }

        Ok(())
    }

    /// Analyze collected debug data
    async fn analyze_debug_data(&self, session: &mut DebugSession) -> FrameworkResult<()> {
        info!("Analyzing debug data for session: {}", session.session_id);

        // Check for performance issues
        for metric in &session.performance_metrics {
            if metric.value > 100.0 && matches!(metric.measurement_type, PerformanceMeasurementType::RenderTime) {
                session.issues_detected.push(DebugIssue {
                    issue_id: Uuid::new_v4().to_string(),
                    severity: IssueSeverity::High,
                    category: IssueCategory::Performance,
                    title: "High render time detected".to_string(),
                    description: format!("Render time of {:.2}ms exceeds threshold", metric.value),
                    element: Some(session.component_name.clone()),
                    suggestion: "Optimize component rendering".to_string(),
                    auto_fixable: false,
                    related_metrics: vec![metric.metric_name.clone()],
                });
            }
        }

        // Check accessibility issues
        if let Some(accessibility_info) = &session.debug_data.accessibility_info {
            if !accessibility_info.focusable && accessibility_info.tab_index >= 0 {
                session.issues_detected.push(DebugIssue {
                    issue_id: Uuid::new_v4().to_string(),
                    severity: IssueSeverity::Medium,
                    category: IssueCategory::Accessibility,
                    title: "Accessibility issue detected".to_string(),
                    description: "Component should be focusable for keyboard navigation".to_string(),
                    element: Some(session.component_name.clone()),
                    suggestion: "Add appropriate accessibility attributes".to_string(),
                    auto_fixable: true,
                    related_metrics: Vec::new(),
                });
            }
        }

        // Check for layout issues
        let layout = &session.debug_data.layout_info;
        if layout.bounding_box.width <= 0.0 || layout.bounding_box.height <= 0.0 {
            session.issues_detected.push(DebugIssue {
                issue_id: Uuid::new_v4().to_string(),
                severity: IssueSeverity::High,
                category: IssueCategory::Layout,
                title: "Invalid element dimensions".to_string(),
                description: format!("Element has invalid dimensions: {}x{}", 
                                   layout.bounding_box.width, layout.bounding_box.height),
                element: Some(session.component_name.clone()),
                suggestion: "Check CSS styling and layout properties".to_string(),
                auto_fixable: false,
                related_metrics: Vec::new(),
            });
        }

        // Add debug logs summary
        session.logs.push(DebugLog {
            log_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            level: LogLevel::Info,
            component: "debugger".to_string(),
            message: format!("Debug session analysis completed. Found {} issues.", session.issues_detected.len()),
            data: {
                let mut data = HashMap::new();
                data.insert("issues_count".to_string(), session.issues_detected.len().to_string());
                data.insert("events_count".to_string(), session.events_captured.len().to_string());
                data.insert("metrics_count".to_string(), session.performance_metrics.len().to_string());
                data
            },
        });

        Ok(())
    }

    /// Get active debug session
    pub fn get_session(&self, session_id: &str) -> Option<&DebugSession> {
        self.active_sessions.get(session_id)
    }

    /// Get all active sessions
    pub fn get_all_sessions(&self) -> &HashMap<String, DebugSession> {
        &self.active_sessions
    }

    /// Generate debug report for a session
    pub fn generate_debug_report(&self, session_id: &str) -> FrameworkResult<DebugReport> {
        let session = self.get_session(session_id)
            .ok_or_else(|| FrameworkError::Debugging(format!("Debug session not found: {}", session_id)))?;

        let report = DebugReport {
            session_id: session.session_id.clone(),
            component_name: session.component_name.clone(),
            duration_ms: if let Some(end_time) = session.end_time {
                (end_time - session.start_time).num_milliseconds() as u64
            } else {
                0
            },
            summary: self.generate_session_summary(session),
            issues: session.issues_detected.clone(),
            performance_summary: self.generate_performance_summary(session),
            event_summary: self.generate_event_summary(session),
            recommendations: self.generate_recommendations(session),
            timestamp: Utc::now(),
        };

        Ok(report)
    }

    /// Generate session summary
    fn generate_session_summary(&self, session: &DebugSession) -> SessionSummary {
        SessionSummary {
            total_events: session.events_captured.len(),
            total_state_changes: session.state_changes.len(),
            total_metrics: session.performance_metrics.len(),
            total_screenshots: session.screenshots.len(),
            total_logs: session.logs.len(),
            total_issues: session.issues_detected.len(),
            critical_issues: session.issues_detected.iter()
                .filter(|i| matches!(i.severity, IssueSeverity::Critical))
                .count(),
            high_issues: session.issues_detected.iter()
                .filter(|i| matches!(i.severity, IssueSeverity::High))
                .count(),
        }
    }

    /// Generate performance summary
    fn generate_performance_summary(&self, session: &DebugSession) -> PerformanceSummary {
        let render_metrics: Vec<_> = session.performance_metrics.iter()
            .filter(|m| matches!(m.measurement_type, PerformanceMeasurementType::RenderTime))
            .collect();

        PerformanceSummary {
            avg_render_time_ms: if render_metrics.is_empty() {
                0.0
            } else {
                render_metrics.iter().map(|m| m.value).sum::<f64>() / render_metrics.len() as f64
            },
            max_render_time_ms: render_metrics.iter().map(|m| m.value).fold(0.0, f64::max),
            avg_memory_usage_mb: {
                let memory_metrics: Vec<_> = session.performance_metrics.iter()
                    .filter(|m| matches!(m.measurement_type, PerformanceMeasurementType::MemoryUsage))
                    .collect();
                if memory_metrics.is_empty() {
                    0.0
                } else {
                    memory_metrics.iter().map(|m| m.value).sum::<f64>() / memory_metrics.len() as f64
                }
            },
            performance_score: self.calculate_performance_score(session),
        }
    }

    /// Generate event summary
    fn generate_event_summary(&self, session: &DebugSession) -> EventSummary {
        let event_types: std::collections::HashMap<_, _> = session.events_captured.iter()
            .map(|e| (format!("{:?}", e.event_type), 1))
            .collect();

        EventSummary {
            event_type_counts: event_types,
            most_common_event: session.events_captured.first()
                .map(|e| format!("{:?}", e.event_type))
                .unwrap_or_default(),
            event_rate_per_minute: if session.end_time.is_some() {
                let duration_minutes = ((session.end_time.unwrap() - session.start_time).num_milliseconds() as f64) / 60000.0;
                if duration_minutes > 0.0 {
                    session.events_captured.len() as f64 / duration_minutes
                } else {
                    0.0
                }
            } else {
                0.0
            },
        }
    }

    /// Generate recommendations based on debug data
    fn generate_recommendations(&self, session: &DebugSession) -> Vec<DebugRecommendation> {
        let mut recommendations = Vec::new();

        // Performance recommendations
        if session.issues_detected.iter().any(|i| matches!(i.category, IssueCategory::Performance)) {
            recommendations.push(DebugRecommendation {
                id: Uuid::new_v4().to_string(),
                category: "Performance".to_string(),
                title: "Optimize Component Performance".to_string(),
                description: "Component shows performance issues that could impact user experience".to_string(),
                priority: "High".to_string(),
                steps: vec![
                    "Review render methods for efficiency".to_string(),
                    "Consider implementing virtualization for large lists".to_string(),
                    "Optimize CSS selectors and animations".to_string(),
                ],
            });
        }

        // Accessibility recommendations
        if session.issues_detected.iter().any(|i| matches!(i.category, IssueCategory::Accessibility)) {
            recommendations.push(DebugRecommendation {
                id: Uuid::new_v4().to_string(),
                category: "Accessibility".to_string(),
                title: "Improve Accessibility".to_string(),
                description: "Component has accessibility issues that should be addressed".to_string(),
                priority: "Medium".to_string(),
                steps: vec![
                    "Add proper ARIA labels and roles".to_string(),
                    "Ensure keyboard navigation works correctly".to_string(),
                    "Test with screen readers".to_string(),
                ],
            });
        }

        recommendations
    }

    /// Calculate performance score
    fn calculate_performance_score(&self, session: &DebugSession) -> f64 {
        if session.performance_metrics.is_empty() {
            return 100.0;
        }

        let mut score = 100.0;
        
        for metric in &session.performance_metrics {
            match metric.measurement_type {
                PerformanceMeasurementType::RenderTime => {
                    if metric.value > 16.0 {
                        score -= ((metric.value - 16.0) / 4.0).min(30.0);
                    }
                }
                PerformanceMeasurementType::MemoryUsage => {
                    if metric.value > 10.0 {
                        score -= ((metric.value - 10.0) / 2.0).min(20.0);
                    }
                }
                _ => {}
            }
        }

        score.max(0.0)
    }
}

/// Debug report for a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugReport {
    pub session_id: String,
    pub component_name: String,
    pub duration_ms: u64,
    pub summary: SessionSummary,
    pub issues: Vec<DebugIssue>,
    pub performance_summary: PerformanceSummary,
    pub event_summary: EventSummary,
    pub recommendations: Vec<DebugRecommendation>,
    pub timestamp: DateTime<Utc>,
}

/// Session summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub total_events: usize,
    pub total_state_changes: usize,
    pub total_metrics: usize,
    pub total_screenshots: usize,
    pub total_logs: usize,
    pub total_issues: usize,
    pub critical_issues: usize,
    pub high_issues: usize,
}

/// Performance summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub avg_render_time_ms: f64,
    pub max_render_time_ms: f64,
    pub avg_memory_usage_mb: f64,
    pub performance_score: f64,
}

/// Event summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSummary {
    pub event_type_counts: HashMap<String, usize>,
    pub most_common_event: String,
    pub event_rate_per_minute: f64,
}

/// Debug recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugRecommendation {
    pub id: String,
    pub category: String,
    pub title: String,
    pub description: String,
    pub priority: String,
    pub steps: Vec<String>,
}

/// Result of debug operation
pub type DebugResult = FrameworkResult<DebugReport>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_debug_session_creation() {
        let session = DebugSession {
            session_id: "test-session".to_string(),
            component_name: "test-component".to_string(),
            start_time: Utc::now(),
            end_time: None,
            debug_data: DebugData {
                dom_snapshot: DOMSnapshot {
                    structure: "<div></div>".to_string(),
                    element_count: 1,
                    node_types: HashMap::new(),
                    depth_levels: vec![],
                    attributes: HashMap::new(),
                    content: "test".to_string(),
                },
                style_snapshot: StyleSnapshot {
                    css_rules: vec![],
                    computed_styles: HashMap::new(),
                    layout_properties: HashMap::new(),
                    animation_properties: HashMap::new(),
                    media_queries_applied: vec![],
                },
                properties: HashMap::new(),
                layout_info: LayoutInfo {
                    bounding_box: BoundingBox {
                        top: 0.0,
                        left: 0.0,
                        right: 100.0,
                        bottom: 50.0,
                        width: 100.0,
                        height: 50.0,
                    },
                    display_mode: "block".to_string(),
                    position: "static".to_string(),
                    size: Size {
                        width: 100.0,
                        height: 50.0,
                        min_width: 0.0,
                        min_height: 0.0,
                        max_width: f32::INFINITY,
                        max_height: f32::INFINITY,
                    },
                    margin: BoxEdges {
                        top: 0.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: 0.0,
                    },
                    padding: BoxEdges {
                        top: 0.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: 0.0,
                    },
                    border: BoxEdges {
                        top: 0.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: 0.0,
                    },
                    z_index: 0,
                    overflow: "visible".to_string(),
                },
                accessibility_info: AccessibilityDebugInfo {
                    aria_role: None,
                    aria_label: None,
                    aria_describedby: vec![],
                    aria_hidden: false,
                    tab_index: 0,
                    focusable: false,
                    keyboard_accessible: false,
                    screen_reader_text: None,
                },
                performance_info: PerformanceDebugInfo {
                    render_time_ms: 0.0,
                    paint_time_ms: 0.0,
                    layout_time_ms: 0.0,
                    script_time_ms: 0.0,
                    memory_usage_mb: 0.0,
                    cpu_usage_percent: 0.0,
                    fps: 0.0,
                    resource_usage: HashMap::new(),
                },
            },
            events_captured: Vec::new(),
            state_changes: Vec::new(),
            performance_metrics: Vec::new(),
            issues_detected: Vec::new(),
            screenshots: Vec::new(),
            logs: Vec::new(),
        };
        
        assert_eq!(session.session_id, "test-session");
        assert_eq!(session.component_name, "test-component");
        assert!(session.events_captured.is_empty());
    }
    
    #[test]
    fn test_debug_event_types() {
        let click_event = DebugEventType::Click;
        let custom_event = DebugEventType::Custom("custom_event".to_string());
        
        assert!(matches!(click_event, DebugEventType::Click));
        assert!(matches!(custom_event, DebugEventType::Custom(_)));
    }
    
    #[test]
    fn test_issue_severity_ordering() {
        assert!(IssueSeverity::Critical > IssueSeverity::High);
        assert!(IssueSeverity::High > IssueSeverity::Medium);
        assert!(IssueSeverity::Medium > IssueSeverity::Low);
    }
    
    #[test]
    fn test_log_level_hierarchy() {
        assert!(LogLevel::Critical > LogLevel::Error);
        assert!(LogLevel::Error > LogLevel::Warning);
        assert!(LogLevel::Warning > LogLevel::Info);
        assert!(LogLevel::Info > LogLevel::Debug);
    }
}