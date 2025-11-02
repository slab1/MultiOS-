//! Window Decorations Example
//! 
//! This example demonstrates how to create and customize window decorations
//! using the MultiOS Window Management System.

use window_manager::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MultiOS Window Management System - Window Decorations ===\n");

    // Create decoration renderer with different themes
    println!("\n--- Decorations with Different Themes ---");

    // Default theme
    let mut default_style = DecorationStyle::default();
    let default_renderer = DecorationRenderer::new(default_style);
    println!("✓ Created default decoration renderer");

    // Dark theme
    let dark_style = DecorationStyle::new(DecorationTheme::Dark)
        .with_button_size(28)
        .with_border_width(3);
    let dark_renderer = DecorationRenderer::new(dark_style);
    println!("✓ Created dark theme decoration renderer");

    // macOS style
    let macos_style = DecorationStyle::new(DecorationTheme::MacOS)
        .with_corner_radius(8)
        .with_title_bar_height(28);
    let macos_renderer = DecorationRenderer::new(macos_style);
    println!("✓ Created macOS-style decoration renderer");

    // Windows style
    let windows_style = DecorationStyle::new(DecorationTheme::Windows)
        .with_title_bar_height(32)
        .with_button_size(32);
    let windows_renderer = DecorationRenderer::new(windows_style);
    println!("✓ Created Windows-style decoration renderer");

    // Initialize window manager
    let mut window_manager = WindowManager::new();

    // Create windows with different decoration configurations
    println!("\n--- Creating Windows with Different Decorations ---");

    // Standard window with default decorations
    let window1_id = window_manager.create_window(
        "Standard Window".to_string(),
        Rectangle::new(100, 100, 600, 400),
        WindowStyle {
            resizable: true,
            maximizable: true,
            minimizable: true,
            closable: true,
            has_title_bar: true,
            has_border: true,
            always_on_top: false,
        },
    )?;
    println!("✓ Created standard window with full decorations");

    // Resizable-only window
    let mut style2 = WindowStyle::default();
    style2.closable = false;
    style2.maximizable = false;
    style2.minimizable = false;
    
    let window2_id = window_manager.create_window(
        "Resize Only Window".to_string(),
        Rectangle::new(200, 200, 500, 300),
        style2,
    )?;
    println!("✓ Created window with only resizing enabled");

    // Borderless window
    let mut style3 = WindowStyle::default();
    style3.has_title_bar = false;
    style3.has_border = false;
    style3.resizable = false;
    style3.maximizable = false;
    style3.minimizable = false;
    style3.closable = false;
    
    let window3_id = window_manager.create_window(
        "Borderless Tool".to_string(),
        Rectangle::new(300, 300, 400, 250),
        style3,
    )?;
    println!("✓ Created borderless tool window");

    // Always-on-top window
    let mut style4 = WindowStyle::default();
    style4.always_on_top = true;
    
    let window4_id = window_manager.create_window(
        "Always on Top".to_string(),
        Rectangle::new(400, 250, 300, 200),
        style4,
    )?;
    println!("✓ Created always-on-top window");

    // Test button hit detection
    println!("\n--- Testing Button Hit Detection ---");

    // Test close button hit detection
    if let Some(window) = window_manager.get_window(window1_id) {
        if let Some(close_bounds) = window.close_button_bounds() {
            println!("✓ Close button bounds: ({}, {}) size {}x{}",
                close_bounds.position.x,
                close_bounds.position.y,
                close_bounds.size.width,
                close_bounds.size.height
            );
        }

        if let Some(min_bounds) = window.minimize_button_bounds() {
            println!("✓ Minimize button bounds: ({}, {}) size {}x{}",
                min_bounds.position.x,
                min_bounds.position.y,
                min_bounds.size.width,
                min_bounds.size.height
            );
        }

        if let Some(max_bounds) = window.maximize_button_bounds() {
            println!("✓ Maximize button bounds: ({}, {}) size {}x{}",
                max_bounds.position.x,
                max_bounds.position.y,
                max_bounds.size.width,
                max_bounds.size.height
            );
        }
    }

    // Test title bar hit detection
    println!("\n--- Testing Title Bar Hit Detection ---");

    if let Some(window) = window_manager.get_window(window1_id) {
        let title_bar_center = Point::new(
            window.bounds().position.x + (window.bounds().size.width as i32) / 2,
            window.bounds().position.y + 15, // Middle of title bar
        );

        let title_bar_test = window.contains_title_bar(title_bar_center);
        println!("✓ Title bar contains point ({}, {}): {}",
            title_bar_center.x, title_bar_center.y, title_bar_test);
    }

    // Test window hit testing
    println!("\n--- Testing Window Hit Testing ---");

    if let Some(window) = window_manager.get_window(window1_id) {
        let bounds = window.bounds();
        
        // Test various points
        let test_points = [
            Point::new(bounds.position.x + 10, bounds.position.y + 10), // Inside window
            Point::new(bounds.position.x - 10, bounds.position.y - 10), // Outside window
            Point::new(bounds.position.x + 50, bounds.position.y - 5), // Above window
        ];

        for (i, point) in test_points.iter().enumerate() {
            let hit_test = hit_test_window(window, *point);
            println!("✓ Hit test {}: Point ({}, {}) -> {:?}",
                i + 1, point.x, point.y, hit_test);
        }
    }

    // Demonstrate decoration styles
    println!("\n--- Decoration Style Information ---");

    let themes = [
        ("Default", DecorationTheme::Default),
        ("Dark", DecorationTheme::Dark),
        ("Rounded", DecorationTheme::Rounded),
        ("macOS", DecorationTheme::MacOS),
        ("Windows", DecorationTheme::Windows),
        ("Linux", DecorationTheme::Linux),
    ];

    for (name, theme) in &themes {
        let style = DecorationStyle::new(*theme);
        let colors = DecorationColors::for_theme(*theme);
        
        println!("{} Theme:", name);
        println!("  Title bar height: {}", style.title_bar_height);
        println!("  Button size: {}", style.button_size);
        println!("  Border width: {}", style.border_width);
        println!("  Corner radius: {}", style.corner_radius);
        println!("  Title bar color: RGB({}, {}, {})",
            match colors.title_bar { Color::RGB(r, g, b) => (r, g, b), _ => (0, 0, 0) }.0,
            match colors.title_bar { Color::RGB(r, g, b) => (r, g, b), _ => (0, 0, 0) }.1,
            match colors.title_bar { Color::RGB(r, g, b) => (r, g, b), _ => (0, 0, 0) }.2
        );
        println!();
    }

    // Simulate decoration rendering
    println!("\n--- Simulating Decoration Rendering ---");

    if let Some(window) = window_manager.get_window(window1_id) {
        let mut render_context = RenderContext::new();
        
        // This would normally be connected to the actual graphics system
        println!("✓ Would render decorations for window: {}", window.title());
        println!("  Theme: {:?}", default_renderer.style().theme);
        println!("  Style: {:?}", default_renderer.style());
    }

    // Process events
    println!("\n--- Event Processing ---");
    let events = window_manager.process_events();
    println!("✓ Processed {} events", events.len());

    // Clean up
    println!("\n--- Cleanup ---");
    window_manager.destroy_window(window1_id)?;
    window_manager.destroy_window(window2_id)?;
    window_manager.destroy_window(window3_id)?;
    window_manager.destroy_window(window4_id)?;
    println!("✓ All windows destroyed");

    println!("\n✓ Window decorations demonstration completed!");

    Ok(())
}
