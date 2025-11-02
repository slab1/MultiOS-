# MultiOS UI Testing Framework

A comprehensive, modern testing framework for MultiOS user interface components, featuring automated GUI testing, visual regression testing, accessibility validation, and performance benchmarking.

## Features

### 🚀 Core Testing Capabilities
- **GUI Automation**: Automated testing of UI interactions and workflows
- **Visual Regression**: Screenshot comparison with multiple algorithms
- **Accessibility Testing**: WCAG 2.1 AA compliance validation
- **Performance Benchmarking**: Frame rate, render time, and memory profiling
- **Widget Testing**: Comprehensive UI component validation
- **Cross-Platform**: Multi-OS and multi-resolution testing

### 🔧 Advanced Features
- **Real-time Debugging**: Live UI inspection and event logging
- **Usability Validation**: User experience and workflow testing
- **Platform Compatibility**: Multi-OS and hardware configuration testing
- **Comprehensive Reporting**: HTML, JSON, and JUnit XML report generation
- **Async/Await Support**: Concurrent test execution for performance
- **Configurable Thresholds**: Customizable similarity and performance thresholds

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
multios-ui-testing = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

### Basic Usage

```rust
use multios_ui_testing::*;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize automation engine
    let mut automation = AutomationEngine::new().await?;
    
    // Create a simple test scenario
    let scenario = TestScenario::new("login_test")
        .with_step(WidgetInteraction::find_by_id("username")?)
        .with_step(WidgetInteraction::type_text("test_user")?)
        .with_step(WidgetInteraction::find_by_id("password")?)
        .with_step(WidgetInteraction::type_text("password")?)
        .with_step(WidgetInteraction::find_by_id("login_button")?)
        .with_step(WidgetInteraction::click()?);
    
    // Execute the test
    let result = automation.execute_scenario(scenario).await?;
    println!("Test result: {:?}", result);
    
    automation.shutdown().await?;
    Ok(())
}
```

### Example Scenarios

The framework includes comprehensive examples in the `examples/` directory:

- **`basic_automation.rs`** - Simple UI automation workflow
- **`screenshot_comparison.rs`** - Visual regression testing
- **`accessibility_testing.rs`** - WCAG compliance validation
- **`performance_benchmark.rs`** - UI performance testing
- **`visual_regression.rs`** - Automated change detection
- **`comprehensive_test_suite.rs`** - Complete testing workflow

## API Overview

### Core Components

#### AutomationEngine
```rust
// GUI testing automation
let mut automation = AutomationEngine::new().await?;

// Create test scenarios
let scenario = TestScenario::new("test_name")
    .with_step(WidgetInteraction::find_by_id("button")?)
    .with_step(WidgetInteraction::click()?);

// Execute tests
let result = automation.execute_scenario(scenario).await?;
```

#### ComparisonEngine
```rust
// Screenshot comparison and visual regression
let mut comparison = ComparisonEngine::new().await?;

// Compare screenshots
let result = comparison.compare_ssim(&current, &baseline, 0.95).await?;

// Generate visual diffs
let diff = comparison.generate_diff(&current, &baseline, ComparisonMode::HighlightDifferences, 30).await?;
```

#### AccessibilityTester
```rust
// Accessibility and WCAG compliance testing
let mut accessibility = AccessibilityTester::new().await?;

// Run comprehensive accessibility audit
let audit = accessibility.run_full_audit("window_name", &[AccessibilityStandard::WCAG2AA]).await?;

// Test specific aspects
let keyboard_test = accessibility.test_keyboard_navigation().await?;
let contrast_check = accessibility.check_color_contrast().await?;
```

#### PerformanceBenchmark
```rust
// UI performance testing and profiling
let mut benchmark = PerformanceBenchmark::new().await?;

// Measure frame rates
let frame_stats = benchmark.measure_frame_rate("window_name", Duration::from_secs(10)).await?;

// Profile render times
let render_stats = benchmark.measure_render_time("component_name", 100).await?;
```

### Widget Interactions

```rust
// Find and interact with widgets
WidgetInteraction::find_by_id("element_id")?;
WidgetInteraction::find_by_class("element_class")?;
WidgetInteraction::find_by_text("Button Text")?;

// Click and type
WidgetInteraction::click()?;
WidgetInteraction::double_click()?;
WidgetInteraction::right_click()?;
WidgetInteraction::type_text("input text")?;

// Advanced interactions
WidgetInteraction::drag_and_drop(source_id, target_id)?;
WidgetInteraction::scroll(direction, amount)?;
WidgetInteraction::hover(element_id)?;
```

### Comparison Algorithms

```rust
// Multiple comparison methods available
let pixel_result = comparison.compare_pixel_perfect(&img1, &img2, 0.0)?;
let ssim_result = comparison.compare_ssim(&img1, &img2, 0.95)?;
let phash_result = comparison.compare_perceptual_hash(&img1, &img2, 10)?;
```

## Configuration

### Test Configuration

Create a `test_config.toml` file:

```toml
[general]
test_timeout = 300
screenshot_delay = 1000
retry_attempts = 3

[comparison]
default_threshold = 0.95
pixel_threshold = 0.02
ssim_threshold = 0.90

[performance]
target_fps = 60
memory_threshold_mb = 100
render_time_threshold_ms = 16.7

[accessibility]
wcag_level = "AA"
contrast_ratio_threshold = 4.5
```

### Test Scenarios

Define reusable test scenarios in `test_scenarios.toml`:

```toml
[login_scenario]
name = "User Login Workflow"

[[login_scenario.steps]]
action = "navigate"
target = "/login"

[[login_scenario.steps]]
action = "input_text"
target = "username_field"
value = "test_user"

# ... more steps
```

## Test Data Management

### Baseline Screenshots

Organize baseline screenshots in the `test_data/baselines/` directory:

```
test_data/
├── baselines/
│   ├── login_page.png
│   ├── dashboard.png
│   └── settings_dialog.png
├── screenshots/
│   └── current/
└── diffs/
    └── visual_differences/
```

### Report Generation

The framework generates reports in multiple formats:

- **HTML Reports**: Rich visual reports with charts and screenshots
- **JSON Reports**: Machine-readable test results
- **JUnit XML**: CI/CD integration format

```rust
// Generate comprehensive report
let report = unified_report.generate_html_report();
let json_report = unified_report.generate_json_report();
let xml_report = unified_report.generate_junit_xml_report();
```

## Integration with MultiOS

### Integration Points

The framework integrates with MultiOS UI components through:

1. **Direct UI API Access**: Native integration with MultiOS widget APIs
2. **Event System**: MultiOS event handling and propagation
3. **Rendering Pipeline**: Access to MultiOS rendering statistics
4. **Input System**: MultiOS input device simulation

### MultiOS-Specific Features

```rust
// MultiOS-specific widget testing
let multios_widget = MultiosWidget::find_by_id("multios_button")?;
multios_widget.test_multios_style_properties()?;

// MultiOS theme testing
let theme_tester = MultiosThemeTester::new()?;
theme_tester.validate_theme_consistency()?;

// MultiOS accessibility
let multios_accessibility = MultiosAccessibilityTester::new()?;
multios_accessibility.test_multios_screen_reader()?;
```

## Best Practices

### Test Organization

1. **Modular Tests**: Group related tests into logical suites
2. **Reusable Steps**: Create common interaction patterns
3. **Clear Assertions**: Use specific, meaningful assertions
4. **Proper Cleanup**: Always clean up test resources

### Performance Optimization

1. **Concurrent Execution**: Run independent tests in parallel
2. **Resource Management**: Monitor and limit resource usage
3. **Efficient Selectors**: Use optimized element selectors
4. **Proper Throttling**: Respect UI refresh rates

### Accessibility Compliance

1. **Early Testing**: Include accessibility checks from the start
2. **WCAG Guidelines**: Follow WCAG 2.1 AA standards
3. **Real User Testing**: Test with actual assistive technologies
4. **Continuous Monitoring**: Regular accessibility audits

## Troubleshooting

### Common Issues

**Tests timeout frequently**
- Increase timeout values in configuration
- Check for performance bottlenecks
- Verify UI responsiveness

**Visual comparisons fail unexpectedly**
- Check screenshot capture timing
- Verify threshold settings
- Consider platform-specific rendering differences

**Accessibility tests show false positives**
- Review dynamic content areas
- Adjust contrast thresholds
- Validate ARIA implementations

### Debugging

Enable debug logging:

```rust
env_logger::init();
let automation = AutomationEngine::new()
    .with_debug_mode(true)
    .await?;
```

Use the built-in debugging tools:

```rust
let debugger = UIDebugger::new().await?;
// Take debug screenshots
debugger.capture_debug_screenshot("test_failure")?;
// Inspect widget hierarchy
debugger.dump_widget_tree()?;
// Monitor event flow
debugger.start_event_logging()?;
```

## Contributing

### Development Setup

1. Clone the repository
2. Install Rust toolchain (latest stable)
3. Run `cargo test` to verify setup
4. Check examples with `cargo run --example basic_automation`

### Code Style

- Follow Rust standard formatting (`rustfmt`)
- Use comprehensive error handling
- Add documentation for public APIs
- Include integration tests

### Adding New Features

1. Create feature branch
2. Implement functionality with tests
3. Update documentation
4. Submit pull request with description

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For questions, issues, or contributions:

- **Documentation**: Check the `examples/` directory and `test_data/` for usage patterns
- **Issues**: Report bugs via the project issue tracker
- **Features**: Request new features via the project issue tracker
- **Community**: Join discussions in the project forums

---

**MultiOS UI Testing Framework** - Empowering reliable, accessible, and performant user interfaces.