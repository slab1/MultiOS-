//! Basic Window Operations Example
//! 
//! This example demonstrates how to create, manage, and manipulate windows
//! using the MultiOS Window Management System.

use window_manager::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MultiOS Window Management System - Basic Operations ===\n");

    // Initialize the window manager
    let mut window_manager = WindowManager::new();
    println!("✓ Window manager initialized");

    // Create multiple windows with different styles
    println!("\n--- Creating Windows ---");
    
    let window1_id = window_manager.create_window(
        "Main Application".to_string(),
        Rectangle::new(100, 100, 800, 600),
        WindowStyle::default(),
    )?;
    println!("✓ Created window 1: 'Main Application' (ID: {:?})", window1_id);

    let mut window2_style = WindowStyle::default();
    window2_style.always_on_top = true;
    window2_style.resizable = false;
    
    let window2_id = window_manager.create_window(
        "Floating Tool".to_string(),
        Rectangle::new(500, 200, 300, 200),
        window2_style,
    )?;
    println!("✓ Created window 2: 'Floating Tool' (ID: {:?})", window2_id);

    let mut window3_style = WindowStyle::default();
    window3_style.has_title_bar = false;
    window3_style.has_border = false;
    
    let window3_id = window_manager.create_window(
        "Borderless Window".to_string(),
        Rectangle::new(200, 300, 400, 300),
        window3_style,
    )?;
    println!("✓ Created window 3: 'Borderless Window' (ID: {:?})", window3_id);

    // Demonstrate window operations
    println!("\n--- Window Operations ---");

    // Move window 2
    window_manager.move_window(window2_id, Point::new(600, 150))?;
    println!("✓ Moved window 2 to position (600, 150)");

    // Resize window 1
    window_manager.resize_window(window1_id, Size::new(900, 700))?;
    println!("✓ Resized window 1 to 900x700");

    // Test window focus management
    println!("\n--- Focus Management ---");
    window_manager.set_focus(window1_id)?;
    println!("✓ Set focus to window 1");
    
    println!("✓ Focused window: {:?}", window_manager.focused_window());

    // Test z-order operations
    println!("\n--- Z-Order Management ---");
    window_manager.bring_to_front(window2_id)?;
    println!("✓ Brought window 2 to front");

    window_manager.send_to_back(window3_id)?;
    println!("✓ Sent window 3 to back");

    // Test window state changes
    println!("\n--- Window State Changes ---");
    window_manager.minimize_window(window1_id)?;
    println!("✓ Minimized window 1");

    window_manager.maximize_window(window2_id)?;
    println!("✓ Maximized window 2");

    window_manager.restore_window(window1_id)?;
    println!("✓ Restored window 1");

    // Display window information
    println!("\n--- Window Information ---");
    for window_id in window_manager.active_windows() {
        if let Some(window) = window_manager.get_window(window_id) {
            println!("Window ID: {:?}", window.id());
            println!("  Title: {}", window.title());
            println!("  Position: ({}, {})", window.bounds().position.x, window.bounds().position.y);
            println!("  Size: {}x{}", window.bounds().size.width, window.bounds().size.height);
            println!("  State: {:?}", window.state());
            println!("  Focused: {}", window.has_focus());
            println!("  Resizable: {}", window.is_resizable());
            println!("  Has Title Bar: {}", window.style.has_title_bar);
            println!();
        }
    }

    // Test hit testing
    println!("\n--- Hit Testing ---");
    let test_point = Point::new(150, 150);
    println!("Testing point ({}, {})", test_point.x, test_point.y);
    
    // This would typically be done with the actual window data
    println!("✓ Hit testing would be performed during rendering");

    // Process events
    println!("\n--- Event Processing ---");
    let events = window_manager.process_events();
    println!("✓ Processed {} events", events.len());

    // Test error handling
    println!("\n--- Error Handling ---");
    match window_manager.destroy_window(WindowId::new(999)) {
        Err(WindowError::WindowNotFound) => {
            println!("✓ Correctly handled invalid window ID");
        }
        Ok(_) => {
            println!("✗ Unexpected success for invalid window ID");
        }
        Err(e) => {
            println!("✗ Unexpected error: {:?}", e);
        }
    }

    // Clean up windows
    println!("\n--- Cleanup ---");
    window_manager.destroy_window(window1_id)?;
    println!("✓ Destroyed window 1");

    window_manager.destroy_window(window2_id)?;
    println!("✓ Destroyed window 2");

    window_manager.destroy_window(window3_id)?;
    println!("✓ Destroyed window 3");

    println!("\n✓ All operations completed successfully!");

    Ok(())
}
