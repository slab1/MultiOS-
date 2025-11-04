use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct SplashConfig {
    pub show_progress_bar: bool,
    pub show_phase_info: bool,
    pub show_time_estimate: bool,
    pub animation_style: AnimationStyle,
    pub color_scheme: ColorScheme,
    pub update_frequency: Duration,
}

#[derive(Clone, Debug)]
pub enum AnimationStyle {
    None,
    Spinner,
    ProgressBar,
    LoadingDots,
    Custom(String),
}

#[derive(Clone, Debug)]
pub struct ColorScheme {
    pub background: (u8, u8, u8),
    pub foreground: (u8, u8, u8),
    pub progress_bar: (u8, u8, u8),
    pub text: (u8, u8, u8),
    pub accent: (u8, u8, u8),
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            background: (0, 0, 0),
            foreground: (255, 255, 255),
            progress_bar: (0, 255, 0),
            text: (255, 255, 255),
            accent: (255, 255, 0),
        }
    }
}

impl Default for SplashConfig {
    fn default() -> Self {
        Self {
            show_progress_bar: true,
            show_phase_info: true,
            show_time_estimate: true,
            animation_style: AnimationStyle::ProgressBar,
            color_scheme: ColorScheme::default(),
            update_frequency: Duration::from_millis(100),
        }
    }
}

#[derive(Clone, Debug)]
pub struct BootPhase {
    pub name: String,
    pub description: String,
    pub estimated_duration: Duration,
    pub start_time: Option<Instant>,
    pub end_time: Option<Instant>,
    pub progress: f32, // 0.0 to 1.0
}

#[derive(Clone, Debug)]
pub struct BootProgress {
    pub current_phase: usize,
    pub total_phases: usize,
    pub overall_progress: f32,
    pub phase_progress: HashMap<String, f32>,
    pub estimated_time_remaining: Duration,
    pub elapsed_time: Duration,
}

pub struct BootSplashDisplay {
    config: SplashConfig,
    is_running: Arc<AtomicBool>,
    current_progress: Arc<Mutex<BootProgress>>,
    phases: Arc<Mutex<Vec<BootPhase>>>,
    update_thread: Option<thread::JoinHandle<()>>,
}

impl BootSplashDisplay {
    pub fn new(config: SplashConfig) -> Self {
        Self {
            config,
            is_running: Arc::new(AtomicBool::new(false)),
            current_progress: Arc::new(Mutex::new(BootProgress {
                current_phase: 0,
                total_phases: 0,
                overall_progress: 0.0,
                phase_progress: HashMap::new(),
                estimated_time_remaining: Duration::from_millis(0),
                elapsed_time: Duration::from_millis(0),
            })),
            phases: Arc::new(Mutex::new(Vec::new())),
            update_thread: None,
        }
    }

    pub fn start(&mut self) {
        if self.is_running.load(Ordering::SeqCst) {
            return; // Already running
        }

        self.is_running.store(true, Ordering::SeqCst);
        
        let is_running = self.is_running.clone();
        let current_progress = self.current_progress.clone();
        let phases = self.phases.clone();
        let config = self.config.clone();

        self.update_thread = Some(thread::spawn(move || {
            BootSplashDisplay::update_loop(is_running, current_progress, phases, config);
        }));
    }

    pub fn stop(&mut self) {
        self.is_running.store(false, Ordering::SeqCst);
        
        if let Some(thread) = self.update_thread.take() {
            thread.join().ok();
        }
    }

    fn update_loop(
        is_running: Arc<AtomicBool>,
        current_progress: Arc<Mutex<BootProgress>>,
        phases: Arc<Mutex<Vec<BootPhase>>>,
        config: SplashConfig,
    ) {
        let mut last_update = Instant::now();

        while is_running.load(Ordering::SeqCst) {
            let now = Instant::now();
            
            if now.duration_since(last_update) >= config.update_frequency {
                BootSplashDisplay::render_display(&current_progress, &phases, &config);
                last_update = now;
            }
            
            thread::sleep(Duration::from_millis(50));
        }
    }

    fn render_display(
        current_progress: &Arc<Mutex<BootProgress>>,
        phases: &Arc<Mutex<Vec<BootPhase>>>,
        config: &SplashConfig,
    ) {
        let progress = current_progress.lock().unwrap();
        let boot_phases = phases.lock().unwrap();

        // Clear screen (for terminal)
        print!("\x1B[2J\x1B[H");

        // Display header
        BootSplashDisplay::render_header();

        // Display overall progress
        if config.show_progress_bar {
            BootSplashDisplay::render_progress_bar(progress.overall_progress, 50);
        }

        // Display phase information
        if config.show_phase_info {
            BootSplashDisplay::render_phases(&boot_phases, &progress);
        }

        // Display time estimates
        if config.show_time_estimate {
            BootSplashDisplay::render_time_info(&progress);
        }

        // Display animation
        BootSplashDisplay::render_animation(&config.animation_style);

        // Flush output
        std::io::Write::flush(&mut std::io::stdout()).ok();
    }

    fn render_header() {
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║                    BOOT OPTIMIZATION SYSTEM                  ║");
        println!("║                     Target: < 2 seconds                      ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();
    }

    fn render_progress_bar(progress: f32, width: usize) {
        let filled = (progress * width as f32) as usize;
        let empty = width - filled;
        
        print!("Progress: [");
        for _ in 0..filled {
            print!("█");
        }
        for _ in 0..empty {
            print!("░");
        }
        println!("] {:>6.1}%", progress * 100.0);
        println!();
    }

    fn render_phases(phases: &[BootPhase], progress: &BootProgress) {
        println!("Boot Phases:");
        for (i, phase) in phases.iter().enumerate() {
            let status = if i < progress.current_phase {
                "✓"
            } else if i == progress.current_phase {
                "→"
            } else {
                "○"
            };
            
            let progress_bar = if i == progress.current_phase {
                let phase_progress = progress.phase_progress.get(&phase.name).unwrap_or(&0.0);
                BootSplashDisplay::create_phase_progress_bar(*phase_progress, 30)
            } else if i < progress.current_phase {
                "████████████████████████████".to_string()
            } else {
                "                              ".to_string()
            };
            
            println!("  {} {:<20} [{}] {}", 
                     status, 
                     phase.name, 
                     progress_bar,
                     phase.description);
        }
        println!();
    }

    fn create_phase_progress_bar(progress: f32, width: usize) -> String {
        let filled = (progress * width as f32) as usize;
        let empty = width - filled;
        
        let mut bar = String::new();
        for _ in 0..filled {
            bar.push_str("█");
        }
        for _ in 0..empty {
            bar.push_str("░");
        }
        bar
    }

    fn render_time_info(progress: &BootProgress) {
        let elapsed_secs = progress.elapsed_time.as_secs();
        let remaining_secs = progress.estimated_time_remaining.as_secs();
        
        println!("Time Information:");
        println!("  Elapsed: {:02}:{:02}  Remaining: {:02}:{:02}  Total: {:02}:{:02}", 
                 elapsed_secs / 60, elapsed_secs % 60,
                 remaining_secs / 60, remaining_secs % 60,
                 (elapsed_secs + remaining_secs) / 60, (elapsed_secs + remaining_secs) % 60);
        println!();
    }

    fn render_animation(style: &AnimationStyle) {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        
        let frame = COUNTER.fetch_add(1, Ordering::SeqCst) % 4;
        
        match style {
            AnimationStyle::Spinner => {
                let spinners = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
                print!("{}", spinners[frame]);
                print!("\r");
            },
            AnimationStyle::LoadingDots => {
                let dots = ["   ", ".  ", ".. ", "..."];
                print!("Loading{}", dots[frame]);
                print!("\r");
            },
            _ => {},
        }
    }

    pub fn update_phase(&self, phase_name: &str, progress: f32) {
        let mut phases = self.phases.lock().unwrap();
        for phase in phases.iter_mut() {
            if phase.name == phase_name {
                phase.progress = progress;
                break;
            }
        }

        // Update current progress
        let mut current_progress = self.current_progress.lock().unwrap();
        current_progress.phase_progress.insert(phase_name.to_string(), progress);
    }

    pub fn start_phase(&self, phase_name: &str, description: &str, estimated_duration: Duration) {
        let mut phases = self.phases.lock().unwrap();
        
        let phase = BootPhase {
            name: phase_name.to_string(),
            description: description.to_string(),
            estimated_duration,
            start_time: Some(Instant::now()),
            end_time: None,
            progress: 0.0,
        };
        
        phases.push(phase);

        // Update current progress
        let mut current_progress = self.current_progress.lock().unwrap();
        current_progress.total_phases = phases.len();
        current_progress.current_phase = phases.len() - 1;
    }

    pub fn end_phase(&self, phase_name: &str) {
        let mut phases = self.phases.lock().unwrap();
        
        for phase in phases.iter_mut() {
            if phase.name == phase_name {
                phase.end_time = Some(Instant::now());
                phase.progress = 1.0;
                break;
            }
        }

        // Update current progress
        let mut current_progress = self.current_progress.lock().unwrap();
        current_progress.current_phase = current_progress.current_phase.min(phases.len() - 1);
        
        // Recalculate overall progress
        let completed_phases = phases.iter().filter(|p| p.end_time.is_some()).count();
        current_progress.overall_progress = if phases.is_empty() {
            0.0
        } else {
            completed_phases as f32 / phases.len() as f32
        };
    }

    pub fn update_time_estimates(&self, elapsed_time: Duration, estimated_remaining: Duration) {
        let mut current_progress = self.current_progress.lock().unwrap();
        current_progress.elapsed_time = elapsed_time;
        current_progress.estimated_time_remaining = estimated_remaining;
    }

    pub fn set_boot_phases(&self, phases: Vec<BootPhase>) {
        let mut phases_lock = self.phases.lock().unwrap();
        *phases_lock = phases;
    }

    pub fn get_current_progress(&self) -> BootProgress {
        self.current_progress.lock().unwrap().clone()
    }
}

impl Drop for BootSplashDisplay {
    fn drop(&mut self) {
        self.stop();
    }
}

pub struct BootSplashSimulator {
    config: SplashConfig,
}

impl BootSplashSimulator {
    pub fn new() -> Self {
        Self {
            config: SplashConfig::default(),
        }
    }

    pub fn simulate_boot_sequence(&self) {
        let mut splash = BootSplashDisplay::new(self.config.clone());
        splash.start();

        // Define boot phases with realistic durations
        let phases = vec![
            ("firmware", "Initializing firmware and hardware", Duration::from_millis(200)),
            ("bootloader", "Loading bootloader and kernel", Duration::from_millis(150)),
            ("kernel_init", "Initializing kernel subsystems", Duration::from_millis(300)),
            ("device_init", "Enumerating and initializing devices", Duration::from_millis(400)),
            ("service_start", "Starting system services", Duration::from_millis(200)),
        ];

        let total_duration = phases.iter().map(|(_, _, d)| *d).sum::<Duration>();
        let start_time = Instant::now();

        for (i, (phase_name, description, estimated_duration)) in phases.iter().enumerate() {
            splash.start_phase(phase_name, description, *estimated_duration);
            
            // Simulate phase execution
            let phase_start = Instant::now();
            let mut progress = 0.0;
            
            while progress < 1.0 {
                progress += 0.1;
                if progress > 1.0 {
                    progress = 1.0;
                }
                
                splash.update_phase(phase_name, progress);
                
                // Update time estimates
                let elapsed = start_time.elapsed();
                let elapsed_for_phase = phase_start.elapsed();
                let remaining_for_phase = estimated_duration.saturating_sub(elapsed_for_phase);
                let total_remaining = total_duration.saturating_sub(elapsed);
                
                splash.update_time_estimates(elapsed, total_remaining);
                
                thread::sleep(Duration::from_millis(50));
            }
            
            splash.end_phase(phase_name);
        }

        // Final update
        splash.update_time_estimates(start_time.elapsed(), Duration::from_millis(0));
        
        thread::sleep(Duration::from_millis(500)); // Show completion briefly
        splash.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_splash_display_creation() {
        let config = SplashConfig::default();
        let splash = BootSplashDisplay::new(config);
        assert!(!splash.is_running.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn test_phase_management() {
        let config = SplashConfig::default();
        let splash = BootSplashDisplay::new(config);
        
        splash.start_phase("test", "Test phase", Duration::from_millis(100));
        splash.update_phase("test", 0.5);
        splash.end_phase("test");
        
        let progress = splash.get_current_progress();
        assert_eq!(progress.total_phases, 1);
    }

    #[test]
    fn test_color_scheme_default() {
        let scheme = ColorScheme::default();
        assert_eq!(scheme.background, (0, 0, 0));
        assert_eq!(scheme.foreground, (255, 255, 255));
    }

    #[test]
    fn test_splash_simulator() {
        let simulator = BootSplashSimulator::new();
        // This would run the actual simulation in a real scenario
        // For testing, we'll just verify it can be created
        assert!(simulator.config.show_progress_bar);
    }
}
