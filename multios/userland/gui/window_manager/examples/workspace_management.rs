//! Workspace Management Example
//! 
//! This example demonstrates how to manage multiple virtual desktops/workspaces
//! using the MultiOS Window Management System.

use window_manager::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MultiOS Window Management System - Workspace Management ===\n");

    // Initialize the window manager
    let mut window_manager = WindowManager::new();
    println!("✓ Window manager initialized with default workspace");

    // Create some windows in the default workspace
    println!("\n--- Creating Windows in Default Workspace ---");
    
    let window1_id = window_manager.create_window(
        "Browser".to_string(),
        Rectangle::new(100, 100, 800, 600),
        WindowStyle::default(),
    )?;
    println!("✓ Created 'Browser' in default workspace");

    let window2_id = window_manager.create_window(
        "Text Editor".to_string(),
        Rectangle::new(150, 150, 600, 400),
        WindowStyle::default(),
    )?;
    println!("✓ Created 'Text Editor' in default workspace");

    let window3_id = window_manager.create_window(
        "Terminal".to_string(),
        Rectangle::new(200, 200, 500, 300),
        WindowStyle::default(),
    )?;
    println!("✓ Created 'Terminal' in default workspace");

    // Display initial workspace information
    println!("\n--- Initial Workspace Information ---");
    display_workspace_info(&window_manager, 0);

    // Create additional workspaces
    println!("\n--- Creating Additional Workspaces ---");
    
    let workspace1_id = window_manager.create_workspace("Development".to_string())?;
    println!("✓ Created workspace 1: 'Development' (ID: {})", workspace1_id);

    let workspace2_id = window_manager.create_workspace("Design".to_string())?;
    println!("✓ Created workspace 2: 'Design' (ID: {})", workspace2_id);

    let workspace3_id = window_manager.create_workspace("Gaming".to_string())?;
    println!("✓ Created workspace 3: 'Gaming' (ID: {})", workspace3_id);

    // Switch to Development workspace
    println!("\n--- Switching to Development Workspace ---");
    window_manager.switch_to_workspace(workspace1_id)?;
    println!("✓ Switched to workspace 1: 'Development'");
    
    // Create windows in Development workspace
    let window4_id = window_manager.create_window(
        "IDE".to_string(),
        Rectangle::new(100, 100, 1000, 700),
        WindowStyle::default(),
    )?;
    println!("✓ Created 'IDE' in Development workspace");

    let window5_id = window_manager.create_window(
        "Documentation".to_string(),
        Rectangle::new(120, 120, 600, 400),
        WindowStyle::default(),
    )?;
    println!("✓ Created 'Documentation' in Development workspace");

    display_workspace_info(&window_manager, workspace1_id);

    // Switch to Design workspace
    println!("\n--- Switching to Design Workspace ---");
    window_manager.switch_to_workspace(workspace2_id)?;
    println!("✓ Switched to workspace 2: 'Design'");

    let window6_id = window_manager.create_window(
        "Image Editor".to_string(),
        Rectangle::new(80, 80, 900, 600),
        WindowStyle::default(),
    )?;
    println!("✓ Created 'Image Editor' in Design workspace");

    let window7_id = window_manager.create_window(
        "Vector Tool".to_string(),
        Rectangle::new(100, 100, 700, 500),
        WindowStyle::default(),
    )?;
    println!("✓ Created 'Vector Tool' in Design workspace");

    display_workspace_info(&window_manager, workspace2_id);

    // Demonstrate workspace switching
    println!("\n--- Workspace Switching Demonstration ---");
    
    // Switch back to default workspace
    window_manager.switch_to_workspace(0)?;
    println!("✓ Returned to default workspace");
    display_workspace_info(&window_manager, 0);

    // Switch to Development
    window_manager.switch_to_workspace(workspace1_id)?;
    println!("✓ Switched to Development workspace");
    display_workspace_info(&window_manager, workspace1_id);

    // Switch to Gaming (should be empty)
    window_manager.switch_to_workspace(workspace3_id)?;
    println!("✓ Switched to Gaming workspace");
    display_workspace_info(&window_manager, workspace3_id);

    // Demonstrate window movement between workspaces
    println!("\n--- Moving Windows Between Workspaces ---");
    
    // Move IDE from Development to Gaming workspace
    let ide_window = window_manager.get_window(window4_id).unwrap().clone();
    println!("✓ Moving 'IDE' from Development to Gaming workspace");
    // Note: This functionality would be implemented in the actual window manager
    // For this example, we'll simulate the move

    // Switch back to Development to see the effect
    window_manager.switch_to_workspace(workspace1_id)?;
    println!("✓ Returned to Development workspace");
    display_workspace_info(&window_manager, workspace1_id);

    // Demonstrate workspace limits
    println!("\n--- Testing Workspace Limits ---");
    
    // Try to create more workspaces than allowed
    for i in 0..10 {
        match window_manager.create_workspace(format!("Workspace {}", i + 5)) {
            Ok(ws_id) => {
                println!("✓ Created workspace {}: 'Workspace {}'", ws_id, i + 5);
            }
            Err(WindowError::InvalidWorkspace) => {
                println!("✓ Reached workspace limit");
                break;
            }
            Err(e) => {
                println!("✗ Unexpected error: {:?}", e);
                break;
            }
        }
    }

    // Demonstrate workspace statistics
    println!("\n--- Workspace Statistics ---");
    for i in 0..window_manager.workspaces.len() {
        let window_count = window_manager.windows_in_workspace(i).len();
        println!("Workspace {}: {} windows", i, window_count);
    }

    // Process any pending events
    println!("\n--- Event Processing ---");
    let events = window_manager.process_events();
    println!("✓ Processed {} events", events.len());

    println!("\n✓ Workspace management demonstration completed!");

    Ok(())
}

fn display_workspace_info(window_manager: &WindowManager, workspace_id: usize) {
    let windows = window_manager.windows_in_workspace(workspace_id);
    println!("\nWorkspace {} Information:", workspace_id);
    println!("  Window count: {}", windows.len());
    
    if !windows.is_empty() {
        println!("  Windows:");
        for &window_id in &windows {
            if let Some(window) = window_manager.get_window(window_id) {
                println!("    - {} (ID: {:?})", window.title(), window.id());
            }
        }
    } else {
        println!("  (No windows in this workspace)");
    }
}
