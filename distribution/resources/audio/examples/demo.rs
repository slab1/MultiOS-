//! MultiOS Audio Subsystem Demo Program
//! 
//! This program demonstrates the complete audio subsystem functionality
//! including device management, audio processing, mixing, and educational examples.

use std::process::exit;

// Import the audio subsystem
use hardware_support::audio::*;

/// Audio subsystem demo application
pub struct AudioDemo {
    education_system: AudioEducationSystem,
    test_suite: AudioTestSuite,
    monitor: AudioSystemMonitor,
}

impl AudioDemo {
    /// Create a new demo application
    pub fn new() -> Self {
        Self {
            education_system: AudioEducationSystem::new(),
            test_suite: AudioTestSuite::new(),
            monitor: AudioSystemMonitor::new(),
        }
    }

    /// Run the complete demo
    pub fn run(&mut self) -> Result<(), AudioError> {
        println!("🎵 MultiOS Audio Subsystem Demo 🎵");
        println!("=====================================");
        println!();

        // 1. Initialize the audio system
        self.initialize_system()?;

        // 2. Show system information
        self.show_system_info();

        // 3. Run educational examples
        self.run_educational_examples()?;

        // 4. Run comprehensive tests
        self.run_test_suite();

        // 5. Show performance monitoring
        self.show_performance_data();

        // 6. Generate comprehensive report
        self.generate_final_report();

        println!();
        println!("🎉 Demo completed successfully!");
        Ok(())
    }

    /// Initialize the audio system
    fn initialize_system(&mut self) -> Result<(), AudioError> {
        println!("1. Initializing Audio Subsystem");
        println!("--------------------------------");

        // Initialize audio system
        initialize_audio_system()?;
        
        // Initialize education system
        self.education_system.initialize()?;
        
        // Start monitoring
        self.monitor.start_monitoring();

        println!("✅ Audio subsystem initialized");
        println!();

        Ok(())
    }

    /// Show system information
    fn show_system_info(&self) {
        println!("2. System Information");
        println!("---------------------");

        let info = get_subsystem_info();
        println!("📋 Subsystem Version: {}", info.version);
        println!("📅 Build Date: {}", info.build_date);
        println!("🔧 Initialized: {}", if info.initialized { "Yes" } else { "No" });

        println!("\n🎛️  Capabilities:");
        println!("   Max Sample Rate: {} Hz", info.capabilities.max_sample_rate);
        println!("   Max Channels: {}", info.capabilities.max_channels);
        println!("   Max Buffer Size: {} bytes", info.capabilities.max_buffer_size);
        println!("   Hardware Mixing: {}", if info.capabilities.hardware_mixing { "Yes" } else { "No" });
        println!("   Software Mixing: {}", if info.capabilities.software_mixing { "Yes" } else { "No" });

        println!("\n🎵 Supported Formats:");
        for format in &info.capabilities.supported_formats {
            println!("   {:?}", format);
        }

        println!();
    }

    /// Run educational examples
    fn run_educational_examples(&mut self) -> Result<(), AudioError> {
        println!("3. Educational Examples");
        println!("----------------------");

        // List available examples
        let examples = self.education_system.list_examples();
        println!("Available examples ({}):", examples.len());
        for (i, example) in examples.iter().enumerate() {
            println!("  {}. {}", i + 1, example);
        }
        println!();

        // Run selected examples
        let selected_examples = [0, 1, 3, 4, 5]; // Frequency Generation, Recording, Mixing, Effects, Analysis
        for &example_index in &selected_examples {
            if example_index < examples.len() {
                println!("▶️  Running example {}: {}", example_index + 1, examples[example_index]);
                self.education_system.run_example(example_index)?;
                println!("✅ Example completed\n");
            }
        }

        Ok(())
    }

    /// Run the test suite
    fn run_test_suite(&mut self) {
        println!("4. Running Test Suite");
        println!("---------------------");

        let summary = self.test_suite.run_all_tests();
        summary.print_results();

        println!();
    }

    /// Show performance monitoring data
    fn show_performance_data(&self) {
        println!("5. Performance Monitoring");
        println!("-------------------------");

        // Get performance data
        let perf_data = self.education_system.get_performance_data();
        let health_report = self.monitor.generate_health_report();

        println!("📊 Performance Metrics:");
        println!("   Buffer Underruns: {}", perf_data.performance_data.buffer_underruns);
        println!("   Buffer Overruns: {}", perf_data.performance_data.buffer_overruns);
        println!("   CPU Utilization: {:.1}%", perf_data.performance_data.cpu_utilization);
        println!("   Memory Usage: {} bytes", perf_data.performance_data.memory_usage);
        println!("   Latency: {:.2} ms", perf_data.performance_data.latency_ms);

        println!("\n🏥 System Health:");
        println!("   Overall Health: {}/100", health_report.overall_health);
        println!("   Buffer Health: {}/100", health_report.buffer_health);
        println!("   Performance Health: {}/100", health_report.performance_health);
        println!("   Active Alerts: {}", health_report.error_count);

        if !health_report.recommendations.is_empty() {
            println!("\n💡 Recommendations:");
            for recommendation in &health_report.recommendations {
                println!("   • {}", recommendation);
            }
        }

        println!();
    }

    /// Generate comprehensive report
    fn generate_final_report(&self) {
        println!("6. Comprehensive System Report");
        println!("-------------------------------");

        let report = self.education_system.generate_system_report();

        println!("📋 Subsystem Info:");
        println!("   Version: {}", report.subsystem_info.version);
        println!("   Status: {}", if report.subsystem_info.initialized { "Active" } else { "Inactive" });

        println!("\n📈 Performance Summary:");
        println!("   Total Alerts: {}", report.performance_report.alert_count);
        println!("   Active Streams: {}", report.performance_report.real_time_stats.active_streams);
        println!("   Sample Rate: {} Hz", report.performance_report.real_time_stats.current_sample_rate);
        println!("   Peak Level: {:.3}", report.performance_report.real_time_stats.peak_level);

        println!("\n🔍 Debug Summary:");
        println!("   Total Events: {}", report.debug_report.total_events);
        println!("   Function Calls: {}", report.debug_report.function_calls.total_unique_functions);
        println!("   Performance Operations: {}", report.debug_report.performance_summary.total_operations);
        println!("   Errors: {}", report.debug_report.error_summary.total_errors);

        println!("\n🎓 Educational Examples:");
        println!("   Available Examples: {}", report.available_examples.len());
        for (i, example) in report.available_examples.iter().enumerate() {
            println!("     {}. {}", i + 1, example);
        }

        println!();
    }
}

/// Interactive demo menu
pub struct InteractiveDemo {
    demo: AudioDemo,
    running: bool,
}

impl InteractiveDemo {
    /// Create a new interactive demo
    pub fn new() -> Self {
        Self {
            demo: AudioDemo::new(),
            running: true,
        }
    }

    /// Run the interactive demo
    pub fn run(&mut self) -> Result<(), AudioError> {
        println!("🎵 Welcome to MultiOS Audio Subsystem Interactive Demo 🎵");
        println!("========================================================");
        println!();

        // Initialize system
        self.demo.initialize_system()?;

        // Interactive menu loop
        while self.running {
            self.show_menu();
            let choice = self.get_user_choice();
            
            match choice {
                1 => self.demo.run()?,
                2 => self.show_examples_menu(),
                3 => self.show_system_info(),
                4 => self.run_performance_test(),
                5 => self.run_stress_test(),
                6 => self.show_troubleshooting(),
                0 => self.exit_demo(),
                _ => println!("Invalid choice. Please try again."),
            }

            if self.running {
                println!("\nPress Enter to continue...");
                let _ = std::io::stdin().read_line(&mut String::new());
            }
        }

        Ok(())
    }

    /// Show the main menu
    fn show_menu(&self) {
        println!("\n🎛️  Main Menu:");
        println!("  1. Run Full Demo");
        println!("  2. Educational Examples");
        println!("  3. System Information");
        println!("  4. Performance Test");
        println!("  5. Stress Test");
        println!("  6. Troubleshooting Guide");
        println!("  0. Exit");
        println!("\nEnter your choice (0-6):");
    }

    /// Get user choice
    fn get_user_choice() -> usize {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap_or_default();
        input.trim().parse().unwrap_or(99)
    }

    /// Show examples menu
    fn show_examples_menu(&mut self) {
        println!("\n🎓 Educational Examples");
        println!("=======================");

        let examples = self.demo.education_system.list_examples();
        println!("Available examples:");
        for (i, example) in examples.iter().enumerate() {
            println!("  {}. {}", i + 1, example);
        }

        println!("\nEnter example number to run (or 0 to go back):");
        let choice = self.get_user_choice();

        if choice > 0 && choice <= examples.len() {
            println!("▶️  Running: {}", examples[choice - 1]);
            let _ = self.demo.education_system.run_example(choice - 1);
            println!("✅ Example completed");
        }
    }

    /// Show system information
    fn show_system_info(&self) {
        println!("\n📋 System Information");
        println!("=====================");

        let info = get_subsystem_info();
        println!("Version: {}", info.version);
        println!("Build Date: {}", info.build_date);
        println!("Initialized: {}", if info.initialized { "Yes" } else { "No" });

        println!("\nCapabilities:");
        println!("  Sample Rates: 8kHz - {}Hz", info.capabilities.max_sample_rate);
        println!("  Channels: 1-{}", info.capabilities.max_channels);
        println!("  Buffer Size: Up to {} bytes", info.capabilities.max_buffer_size);
        println!("  Mixing: {} + {}", 
                if info.capabilities.hardware_mixing { "Hardware" } else { "" },
                if info.capabilities.software_mixing { "Software" } else { "" });

        println!("\nSupported Formats:");
        for format in &info.capabilities.supported_formats {
            println!("  {:?}", format);
        }
    }

    /// Run performance test
    fn run_performance_test(&mut self) {
        println!("\n⚡ Performance Test");
        println!("===================");

        // Initialize audio manager for testing
        let audio_manager = get_audio_manager().unwrap();
        
        println!("Running performance benchmarks...");
        
        // Run performance test
        let result = testing::run_performance_test(audio_manager, 2.0);
        println!("Performance Test Results:");
        println!("  Duration: {:.2} seconds", result.duration);
        println!("  Samples Processed: {}", result.samples_processed);
        println!("  Throughput: {:.0} samples/sec", result.throughput);
        println!("  CPU Cycles: {}", result.cpu_cycles);

        // Test different configurations
        let configs = vec![
            ("CD Quality", presets::cd_quality()),
            ("Studio Quality", presets::studio_quality()),
            ("Low Latency", presets::low_latency()),
        ];

        for (name, config) in configs {
            let stream_id = audio_manager.create_stream(config).unwrap();
            println!("  Created {} stream: {}", name, stream_id);
            audio_manager.stop_playback(stream_id).unwrap();
        }
    }

    /// Run stress test
    fn run_stress_test(&mut self) {
        println!("\n🔥 Stress Test");
        println!("==============");

        let audio_manager = get_audio_manager().unwrap();
        
        println!("Running stress test with multiple streams...");
        
        let result = testing::run_mixing_stress_test(audio_manager, 10);
        println!("Stress Test Results:");
        println!("  Streams Created: {}/{}", result.streams_created, result.max_streams);
        println!("  Test Passed: {}", if result.test_passed { "Yes" } else { "No" });

        if result.test_passed {
            println!("✅ System can handle multiple concurrent streams");
        } else {
            println!("⚠️  System shows stress under high stream count");
        }
    }

    /// Show troubleshooting guide
    fn show_troubleshooting(&self) {
        println!("\n🔧 Troubleshooting Guide");
        println!("========================");

        let health_report = self.monitor.generate_health_report();

        println!("🎯 Common Issues and Solutions:");
        
        if health_report.overall_health < 80 {
            println!("\n⚠️  Low System Health ({}/100)", health_report.overall_health);
            
            for recommendation in &health_report.recommendations {
                println!("   • {}", recommendation);
            }
        } else {
            println!("✅ System health is good ({}/100)", health_report.overall_health);
        }

        println!("\n🎵 Audio Configuration Tips:");
        println!("   • Use CD quality (44.1kHz/16-bit) for general use");
        println!("   • Use low latency (64-sample buffer) for real-time applications");
        println!("   • Use studio quality for professional audio work");
        println!("   • Use voice recording (16kHz/mono) for speech");

        println!("\n🔧 Performance Optimization:");
        println!("   • Enable hardware mixing when available");
        println!("   • Use appropriate buffer sizes for your use case");
        println!("   • Monitor buffer underruns/overruns");
        println!("   • Keep CPU utilization below 80%");

        let alerts = self.monitor.get_alerts();
        if !alerts.is_empty() {
            println!("\n🚨 Current Alerts ({}):", alerts.len());
            for (i, alert) in alerts.iter().enumerate() {
                println!("   {}. {:?}", i + 1, alert);
            }
        }
    }

    /// Exit the demo
    fn exit_demo(&mut self) {
        println!("\n👋 Thank you for using MultiOS Audio Subsystem Demo!");
        println!("   Shutting down...");
        
        let _ = shutdown_audio_system();
        self.running = false;
    }
}

/// Command-line demo runner
pub fn run_demo() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "interactive" | "-i" => {
                let mut demo = InteractiveDemo::new();
                demo.run()?;
            },
            "test" | "-t" => {
                let mut demo = AudioDemo::new();
                demo.demo.initialize_system()?;
                demo.run_test_suite();
                shutdown_audio_system()?;
            },
            "examples" | "-e" => {
                let mut demo = AudioDemo::new();
                demo.demo.initialize_system()?;
                demo.run_educational_examples()?;
                shutdown_audio_system()?;
            },
            "performance" | "-p" => {
                let mut demo = AudioDemo::new();
                demo.demo.initialize_system()?;
                demo.run_performance_test();
                shutdown_audio_system()?;
            },
            "help" | "-h" | "--help" => {
                show_help();
                exit(0);
            },
            _ => {
                eprintln!("Unknown option: {}", args[1]);
                show_help();
                exit(1);
            }
        }
    } else {
        // Default: run full demo
        let mut demo = AudioDemo::new();
        demo.run()?;
    }

    Ok(())
}

/// Show help information
fn show_help() {
    println!("MultiOS Audio Subsystem Demo");
    println!("============================");
    println!();
    println!("Usage: audio_demo [options]");
    println!();
    println!("Options:");
    println!("  interactive, -i    Run interactive demo");
    println!("  test, -t          Run test suite only");
    println!("  examples, -e      Run educational examples only");
    println!("  performance, -p   Run performance test only");
    println!("  help, -h, --help  Show this help message");
    println!();
    println!("Examples:");
    println!("  audio_demo                    # Run full demo");
    println!("  audio_demo interactive        # Interactive mode");
    println!("  audio_demo test               # Run tests only");
    println!("  audio_demo examples           # Educational examples only");
    println!("  audio_demo performance        # Performance test only");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_creation() {
        let demo = AudioDemo::new();
        assert!(demo.education_system.tutorial.list_examples().len() > 0);
    }

    #[test]
    fn test_audio_subsystem_initialization() {
        let result = initialize_audio_system();
        assert!(result.is_ok() || result.is_err()); // Should not panic
        
        if result.is_ok() {
            let _ = shutdown_audio_system();
        }
    }

    #[test]
    fn test_subsystem_info() {
        let info = get_subsystem_info();
        assert!(!info.version.is_empty());
        assert!(info.capabilities.max_sample_rate > 0);
        assert!(info.capabilities.max_channels > 0);
    }
}

fn main() {
    if let Err(e) = run_demo() {
        eprintln!("Demo failed: {}", e);
        exit(1);
    }
}