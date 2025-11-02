//! Interactive TUI Module
//!
//! Provides a terminal-based user interface for real-time memory profiling
//! and system monitoring using ratatui and crossterm.

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Tabs},
    Frame, Terminal,
};
use std::io;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};

/// UI event types
#[derive(Debug, Clone)]
pub enum UIEvent {
    RefreshData,
    SwitchTab(usize),
    TogglePause,
    Exit,
    ShowDetails(String),
    ExportData,
}

/// Memory metrics for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub total_memory: u64,
    pub allocated_memory: u64,
    pub free_memory: u64,
    pub cache_memory: u64,
    pub buffer_memory: u64,
    pub swap_memory: u64,
    pub memory_pressure: f32,
    pub fragmentation: f32,
}

/// Cache metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetrics {
    pub l1_hit_ratio: f32,
    pub l2_hit_ratio: f32,
    pub l3_hit_ratio: f32,
    pub tlb_hit_ratio: f32,
    pub total_accesses: u64,
    pub average_latency: u32,
}

/// Stack metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackMetrics {
    pub thread_count: u32,
    pub max_stack_depth: u32,
    pub stack_overflows: u32,
    pub average_frame_size: usize,
    pub stack_usage: Vec<ThreadStackInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadStackInfo {
    pub thread_id: u32,
    pub current_usage: usize,
    pub peak_usage: usize,
    pub efficiency: f32,
}

/// NUMA metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NUMAMetrics {
    pub node_count: u8,
    pub local_access_ratio: f32,
    pub remote_access_ratio: f32,
    pub load_balance_score: f32,
    pub thermal_distribution: Vec<f32>,
    pub memory_distribution: Vec<f32>,
}

/// Leak information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeakInfo {
    pub total_leaks: u32,
    pub memory_waste: u64,
    pub top_leaks: Vec<IndividualLeak>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndividualLeak {
    pub address: u64,
    pub size: usize,
    pub suspicion_score: f32,
    pub age: u64,
    pub caller: u64,
}

/// Main interactive UI controller
pub struct InteractiveUI {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    app_state: AppState,
    current_tab: usize,
    is_paused: bool,
    last_update: Instant,
    data_sender: mpsc::UnboundedSender<UIEvent>,
    data_receiver: mpsc::UnboundedReceiver<UIEvent>,
}

impl InteractiveUI {
    /// Create new interactive UI
    pub fn new(config: AppConfig) -> Self {
        // Setup terminal
        enable_raw_mode().expect("Failed to enable raw mode");
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).expect("Failed to enter alternate screen");
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).expect("Failed to create terminal");
        
        // Setup channels
        let (data_sender, data_receiver) = mpsc::unbounded_channel();
        
        InteractiveUI {
            terminal,
            app_state: AppState {
                config,
                memory_metrics: MemoryMetrics::default(),
                cache_metrics: CacheMetrics::default(),
                stack_metrics: StackMetrics::default(),
                numa_metrics: NUMAMetrics::default(),
                leak_info: LeakInfo::default(),
            },
            current_tab: 0,
            is_paused: false,
            last_update: Instant::now(),
            data_sender,
            data_receiver,
        }
    }
    
    /// Set live updates enabled
    pub fn set_live_updates(&mut self, enabled: bool) {
        self.app_state.config.enable_real_time = enabled;
    }
    
    /// Set UI theme
    pub fn set_theme(&mut self, theme: &str) {
        // Theme configuration would be implemented here
    }
    
    /// Start the interactive UI
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.show_welcome_message().await;
        
        loop {
            // Handle events
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char(' ') => {
                            self.is_paused = !self.is_paused;
                            self.data_sender.send(UIEvent::TogglePause)?;
                        }
                        KeyCode::Char('r') => {
                            self.data_sender.send(UIEvent::RefreshData)?;
                        }
                        KeyCode::Tab => {
                            self.current_tab = (self.current_tab + 1) % 4;
                        }
                        KeyCode::Char('1') => self.current_tab = 0,
                        KeyCode::Char('2') => self.current_tab = 1,
                        KeyCode::Char('3') => self.current_tab = 2,
                        KeyCode::Char('4') => self.current_tab = 3,
                        KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }
            
            // Update data if not paused
            if !self.is_paused && self.app_state.config.enable_real_time {
                self.update_metrics().await;
            }
            
            // Draw UI
            self.draw()?;
            
            // Small delay to prevent excessive CPU usage
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        self.cleanup()?;
        Ok(())
    }
    
    /// Show welcome message
    async fn show_welcome_message(&self) {
        println!("Welcome to Memory Profiler Interactive Mode!");
        println!("Controls:");
        println!("  Tab/1-4: Switch tabs");
        println!("  Space: Pause/Resume");
        println!("  R: Refresh data");
        println!("  Q/Esc: Quit");
        println!();
    }
    
    /// Update metrics from profiling system
    async fn update_metrics(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_update) < Duration::from_millis(500) {
            return; // Rate limiting
        }
        self.last_update = now;
        
        // Simulate data updates (in reality would query kernel profiling)
        self.update_memory_metrics();
        self.update_cache_metrics();
        self.update_stack_metrics();
        self.update_numa_metrics();
        self.update_leak_info();
    }
    
    /// Update memory metrics
    fn update_memory_metrics(&mut self) {
        // Simulate real-time memory metrics
        let total_memory = 8 * 1024 * 1024 * 1024; // 8GB
        let allocated = (total_memory as f32 * 0.6 + 
                        (now().0 as f32 % 1000000.0) as f32 * 0.001) as u64;
        let free = total_memory - allocated;
        
        self.app_state.memory_metrics = MemoryMetrics {
            total_memory,
            allocated_memory: allocated,
            free_memory: free,
            cache_memory: 512 * 1024 * 1024, // 512MB
            buffer_memory: 256 * 1024 * 1024, // 256MB
            swap_memory: 0,
            memory_pressure: (allocated as f32 / total_memory as f32),
            fragmentation: 0.15 + (now().0 % 10) as f32 * 0.01,
        };
    }
    
    /// Update cache metrics
    fn update_cache_metrics(&mut self) {
        let time_factor = (now().0 % 100) as f32 / 100.0;
        
        self.app_state.cache_metrics = CacheMetrics {
            l1_hit_ratio: 0.90 + time_factor * 0.05,
            l2_hit_ratio: 0.95 + time_factor * 0.03,
            l3_hit_ratio: 0.98 + time_factor * 0.01,
            tlb_hit_ratio: 0.85 + time_factor * 0.10,
            total_accesses: 1000000 + now().0 as u64,
            average_latency: 25 + (time_factor * 10.0) as u32,
        };
    }
    
    /// Update stack metrics
    fn update_stack_metrics(&mut self) {
        let thread_count = 4;
        let mut stack_usage = Vec::new();
        
        for i in 0..thread_count {
            let current_usage = 256 * 1024 + (i * 64 * 1024);
            let peak_usage = current_usage + 128 * 1024;
            
            stack_usage.push(ThreadStackInfo {
                thread_id: i,
                current_usage,
                peak_usage,
                efficiency: 0.75 + (i as f32 * 0.05),
            });
        }
        
        self.app_state.stack_metrics = StackMetrics {
            thread_count,
            max_stack_depth: 15,
            stack_overflows: 0,
            average_frame_size: 128,
            stack_usage,
        };
    }
    
    /// Update NUMA metrics
    fn update_numa_metrics(&mut self) {
        let node_count = 4;
        
        self.app_state.numa_metrics = NUMAMetrics {
            node_count,
            local_access_ratio: 0.85,
            remote_access_ratio: 0.15,
            load_balance_score: 0.78,
            thermal_distribution: vec![45.0, 48.0, 42.0, 50.0],
            memory_distribution: vec![0.8, 0.7, 0.9, 0.6],
        };
    }
    
    /// Update leak information
    fn update_leak_info(&mut self) {
        let mut top_leaks = Vec::new();
        
        for i in 0..5 {
            top_leaks.push(IndividualLeak {
                address: 0x1000 + i * 0x1000,
                size: 1024 * (i + 1),
                suspicion_score: 0.5 + i as f32 * 0.1,
                age: 3600 + i * 600, // 1+ hours
                caller: 0x2000 + i * 0x1000,
            });
        }
        
        self.app_state.leak_info = LeakInfo {
            total_leaks: 27,
            memory_waste: 1024 * 1024 * 5, // 5MB
            top_leaks,
        };
    }
    
    /// Draw the main UI
    fn draw(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.terminal.draw(|f| {
            let size = f.size();
            
            // Create main layout
            let vertical = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(size);
            
            // Draw title bar
            self.draw_title_bar(f, vertical[0]);
            
            // Draw main content
            self.draw_main_content(f, vertical[1]);
            
            // Draw status bar
            self.draw_status_bar(f, vertical[2]);
        })?;
        
        Ok(())
    }
    
    /// Draw title bar
    fn draw_title_bar(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let title = if self.is_paused {
            "Memory Profiler - PAUSED"
        } else {
            "Memory Profiler - LIVE"
        };
        
        let title_block = Block::default()
            .borders(Borders::BOTTOM)
            .style(Style::default().bg(Color::Blue).fg(Color::White));
        
        let title_line = Line::from(vec![
            Span::styled(title, Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" | Press 'q' to quit, 'Space' to pause"),
        ]);
        
        let paragraph = Paragraph::new(title_line)
            .block(title_block)
            .style(Style::default().bg(Color::Blue).fg(Color::White));
        
        f.render_widget(paragraph, area);
    }
    
    /// Draw main content area
    fn draw_main_content(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        // Create tab layout
        let tab_titles = ["Memory", "Cache", "Stacks", "NUMA & Leaks"];
        
        let tabs = Tabs::new(tab_titles.to_vec())
            .block(Block::default().borders(Borders::ALL))
            .select(self.current_tab)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));
        
        f.render_widget(tabs, area);
        
        // Draw content based on current tab
        let content_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .margin(1)
            .split(area);
        
        self.draw_tab_content(f, content_area);
    }
    
    /// Draw content for current tab
    fn draw_tab_content(&self, f: &mut Frame, areas: [ratatui::layout::Rect; 2]) {
        match self.current_tab {
            0 => self.draw_memory_tab(f, areas),
            1 => self.draw_cache_tab(f, areas),
            2 => self.draw_stack_tab(f, areas),
            3 => self.draw_numa_leaks_tab(f, areas),
            _ => {}
        }
    }
    
    /// Draw memory tab
    fn draw_memory_tab(&self, f: &mut Frame, areas: [ratatui::layout::Rect; 2]) {
        let memory = &self.app_state.memory_metrics;
        
        // Draw memory usage overview
        let usage_ratio = memory.allocated_memory as f32 / memory.total_memory as f32;
        let usage_gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Memory Usage"))
            .gauge_style(Style::default().bg(Color::Red).fg(Color::White))
            .percent((usage_ratio * 100.0) as u16);
        
        f.render_widget(usage_gauge, areas[0]);
        
        // Draw detailed memory info
        let memory_details = vec![
            format!("Total Memory: {} MB", memory.total_memory / (1024 * 1024)),
            format!("Allocated: {} MB ({:.1}%)", 
                   memory.allocated_memory / (1024 * 1024), 
                   usage_ratio * 100.0),
            format!("Free Memory: {} MB", memory.free_memory / (1024 * 1024)),
            format!("Cache: {} MB", memory.cache_memory / (1024 * 1024)),
            format!("Buffers: {} MB", memory.buffer_memory / (1024 * 1024)),
            format!("Memory Pressure: {:.1}%", memory.memory_pressure * 100.0),
            format!("Fragmentation: {:.1}%", memory.fragmentation * 100.0),
        ];
        
        let details_list = List::new(memory_details.into_iter()
            .map(|s| ListItem::new(Line::from(Span::raw(s))))
            .collect())
            .block(Block::default().borders(Borders::ALL).title("Details"));
        
        f.render_widget(details_list, areas[1]);
    }
    
    /// Draw cache tab
    fn draw_cache_tab(&self, f: &mut Frame, areas: [ratatui::layout::Rect; 2]) {
        let cache = &self.app_state.cache_metrics;
        
        // Draw cache hit ratios
        let hit_ratios = vec![
            format!("L1 Cache Hit Ratio: {:.1}%", cache.l1_hit_ratio * 100.0),
            format!("L2 Cache Hit Ratio: {:.1}%", cache.l2_hit_ratio * 100.0),
            format!("L3 Cache Hit Ratio: {:.1}%", cache.l3_hit_ratio * 100.0),
            format!("TLB Hit Ratio: {:.1}%", cache.tlb_hit_ratio * 100.0),
            format!("Total Accesses: {}", cache.total_accesses),
            format!("Average Latency: {} ns", cache.average_latency),
        ];
        
        let cache_list = List::new(hit_ratios.into_iter()
            .map(|s| ListItem::new(Line::from(Span::raw(s))))
            .collect())
            .block(Block::default().borders(Borders::ALL).title("Cache Performance"));
        
        f.render_widget(cache_list, areas[0]);
        
        // Draw cache gauges
        let l1_gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("L1 Hit Ratio"))
            .gauge_style(Style::default().bg(Color::Green).fg(Color::White))
            .percent((cache.l1_hit_ratio * 100.0) as u16);
        
        f.render_widget(l1_gauge, Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50)])
            .split(areas[1])[0]);
        
        let l2_gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("L2 Hit Ratio"))
            .gauge_style(Style::default().bg(Color::Blue).fg(Color::White))
            .percent((cache.l2_hit_ratio * 100.0) as u16);
        
        f.render_widget(l2_gauge, Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50)])
            .split(areas[1])[1]);
    }
    
    /// Draw stack tab
    fn draw_stack_tab(&self, f: &mut Frame, areas: [ratatui::layout::Rect; 2]) {
        let stack = &self.app_state.stack_metrics;
        
        // Draw overall stack stats
        let stack_stats = vec![
            format!("Active Threads: {}", stack.thread_count),
            format!("Max Stack Depth: {}", stack.max_stack_depth),
            format!("Stack Overflows: {}", stack.stack_overflows),
            format!("Avg Frame Size: {} bytes", stack.average_frame_size),
        ];
        
        let stats_list = List::new(stack_stats.into_iter()
            .map(|s| ListItem::new(Line::from(Span::raw(s))))
            .collect())
            .block(Block::default().borders(Borders::ALL).title("Stack Statistics"));
        
        f.render_widget(stats_list, areas[0]);
        
        // Draw per-thread stack usage
        let mut thread_items = Vec::new();
        thread_items.push(ListItem::new(Line::from(Span::raw("Thread Stack Usage:"))));
        
        for thread_info in &stack.stack_usage {
            let usage_percent = (thread_info.current_usage as f32 / thread_info.peak_usage as f32) * 100.0;
            thread_items.push(ListItem::new(Line::from(vec![
                Span::raw(format!("  Thread {}: {} KB / {} KB ({:.1}%)", 
                                thread_info.thread_id,
                                thread_info.current_usage / 1024,
                                thread_info.peak_usage / 1024,
                                usage_percent)),
            ])));
        }
        
        let thread_list = List::new(thread_items)
            .block(Block::default().borders(Borders::ALL).title("Thread Details"));
        
        f.render_widget(thread_list, areas[1]);
    }
    
    /// Draw NUMA and leaks tab
    fn draw_numa_leaks_tab(&self, f: &mut Frame, areas: [ratatui::layout::Rect; 2]) {
        let numa = &self.app_state.numa_metrics;
        let leaks = &self.app_state.leak_info;
        
        // Draw NUMA info
        let numa_info = vec![
            format!("NUMA Nodes: {}", numa.node_count),
            format!("Local Access Ratio: {:.1}%", numa.local_access_ratio * 100.0),
            format!("Remote Access Ratio: {:.1}%", numa.remote_access_ratio * 100.0),
            format!("Load Balance Score: {:.1}", numa.load_balance_score),
            format!("Avg Temperature: {:.1}°C", 
                   numa.thermal_distribution.iter().sum::<f32>() / numa.thermal_distribution.len() as f32),
        ];
        
        let numa_list = List::new(numa_info.into_iter()
            .map(|s| ListItem::new(Line::from(Span::raw(s))))
            .collect())
            .block(Block::default().borders(Borders::ALL).title("NUMA Information"));
        
        f.render_widget(numa_list, areas[0]);
        
        // Draw leak information
        let mut leak_items = Vec::new();
        leak_items.push(ListItem::new(Line::from(vec![
            Span::styled("Memory Leaks:", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        ])));
        leak_items.push(ListItem::new(Line::from(Span::raw(format!("Total Leaks: {}", leaks.total_leaks)))));
        leak_items.push(ListItem::new(Line::from(Span::raw(format!("Memory Waste: {} MB", leaks.memory_waste / (1024 * 1024))))));
        
        leak_items.push(ListItem::new(Line::from(Span::raw("\nTop Leaks:"))));
        for (i, leak) in leaks.top_leaks.iter().enumerate() {
            leak_items.push(ListItem::new(Line::from(Span::raw(format!(
                "  {}: {} bytes (score: {:.2})", i + 1, leak.size, leak.suspicion_score
            )))));
        }
        
        let leak_list = List::new(leak_items)
            .block(Block::default().borders(Borders::ALL).title("Leak Detection"));
        
        f.render_widget(leak_list, areas[1]);
    }
    
    /// Draw status bar
    fn draw_status_bar(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let status = if self.is_paused {
            "PAUSED | Space: Resume | R: Refresh | Q: Quit"
        } else {
            "LIVE | Space: Pause | R: Refresh | Q: Quit"
        };
        
        let status_line = Line::from(Span::styled(
            status,
            Style::default().bg(Color::Gray).fg(Color::White)
        ));
        
        let status_block = Block::default().bg(Color::Gray);
        
        let paragraph = Paragraph::new(status_line)
            .block(status_block)
            .style(Style::default().bg(Color::Gray).fg(Color::White));
        
        f.render_widget(paragraph, area);
    }
    
    /// Cleanup and restore terminal
    fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

/// Application state for UI
#[derive(Debug, Clone)]
struct AppState {
    config: AppConfig,
    memory_metrics: MemoryMetrics,
    cache_metrics: CacheMetrics,
    stack_metrics: StackMetrics,
    numa_metrics: NUMAMetrics,
    leak_info: LeakInfo,
}

/// Get current time (simplified)
fn now() -> (u64, u64) {
    let instant = std::time::Instant::now();
    (instant.elapsed().as_millis() as u64, 0)
}

// Trait implementations for default values
impl Default for MemoryMetrics {
    fn default() -> Self {
        MemoryMetrics {
            total_memory: 8 * 1024 * 1024 * 1024,
            allocated_memory: 0,
            free_memory: 8 * 1024 * 1024 * 1024,
            cache_memory: 0,
            buffer_memory: 0,
            swap_memory: 0,
            memory_pressure: 0.0,
            fragmentation: 0.0,
        }
    }
}

impl Default for CacheMetrics {
    fn default() -> Self {
        CacheMetrics {
            l1_hit_ratio: 0.9,
            l2_hit_ratio: 0.95,
            l3_hit_ratio: 0.98,
            tlb_hit_ratio: 0.85,
            total_accesses: 0,
            average_latency: 25,
        }
    }
}

impl Default for StackMetrics {
    fn default() -> Self {
        StackMetrics {
            thread_count: 0,
            max_stack_depth: 0,
            stack_overflows: 0,
            average_frame_size: 128,
            stack_usage: Vec::new(),
        }
    }
}

impl Default for NUMAMetrics {
    fn default() -> Self {
        NUMAMetrics {
            node_count: 4,
            local_access_ratio: 0.8,
            remote_access_ratio: 0.2,
            load_balance_score: 0.75,
            thermal_distribution: Vec::new(),
            memory_distribution: Vec::new(),
        }
    }
}

impl Default for LeakInfo {
    fn default() -> Self {
        LeakInfo {
            total_leaks: 0,
            memory_waste: 0,
            top_leaks: Vec::new(),
        }
    }
}