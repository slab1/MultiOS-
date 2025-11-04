# MultiOS Window Management System

A comprehensive, lightweight window management system designed for the MultiOS operating system. This system provides all essential window management functionality including window creation, manipulation, decoration, focus management, and multi-desktop support.

## Features

### Core Window Management
- ✅ Window creation and destruction
- ✅ Window movement and resizing
- ✅ Window state management (normal, minimized, maximized)
- ✅ Window focus management
- ✅ Z-order (stacking order) management

### Window Decorations
- ✅ Customizable title bars
- ✅ Window control buttons (close, minimize, maximize)
- ✅ Border rendering with different themes
- ✅ Hit testing for UI elements
- ✅ Multiple decoration themes (Default, Dark, macOS, Windows, Linux, Rounded)

### Multi-Desktop Support
- ✅ Multiple virtual workspaces
- ✅ Workspace switching
- ✅ Window assignment to workspaces
- ✅ Workspace-specific window management

### Event System
- ✅ Event-driven architecture
- ✅ Mouse and keyboard event handling
- ✅ Window lifecycle events
- ✅ Focus change events
- ✅ Z-order change events

### Advanced Features
- ✅ Always-on-top windows
- ✅ Borderless windows
- ✅ Window history management
- ✅ Window grouping by workspace
- ✅ Comprehensive error handling

## Architecture

The window management system is built on a modular architecture with clear separation of concerns:

```
WindowManager (main orchestrator)
├── Window (individual window representation)
├── ZOrderManager (stacking order management)
├── FocusManager (keyboard focus handling)
├── WorkspaceManager (multi-desktop support)
├── EventManager (event processing)
└── DecorationRenderer (UI decoration rendering)
```

### Core Components

#### WindowManager
The central orchestrator that coordinates all window operations:
- Creates and destroys windows
- Manages window states and positions
- Coordinates between different subsystems
- Provides a unified API for applications

#### Window
Represents a single application window:
- Stores window properties (title, bounds, style, state)
- Handles window-specific operations
- Manages decoration layouts
- Provides hit testing functionality

#### ZOrderManager
Manages window stacking order:
- Tracks window z-order per workspace
- Supports bring-to-front and send-to-back operations
- Enables window layering for overlapping windows
- Provides hit testing for window ordering

#### FocusManager
Handles keyboard focus management:
- Tracks currently focused window
- Maintains focus history for cycling
- Supports different activation modes
- Coordinates keyboard input routing

#### WorkspaceManager
Manages multiple virtual desktops:
- Creates and manages workspaces
- Handles workspace switching
- Assigns windows to specific workspaces
- Provides workspace statistics

#### EventManager
Processes window-related events:
- Queues and dispatches events
- Supports custom event handlers
- Handles mouse and keyboard events
- Manages window lifecycle events

#### DecorationRenderer
Handles visual window decorations:
- Renders title bars, buttons, and borders
- Supports multiple visual themes
- Provides customizable decoration styles
- Handles button and border hit testing

## Quick Start

### Basic Usage

```rust
use window_manager::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the window manager
    let mut window_manager = WindowManager::new();
    
    // Create a window
    let window_id = window_manager.create_window(
        "My Application".to_string(),
        Rectangle::new(100, 100, 800, 600),
        WindowStyle::default(),
    )?;
    
    // Move the window
    window_manager.move_window(window_id, Point::new(200, 200))?;
    
    // Set focus
    window_manager.set_focus(window_id)?;
    
    // Minimize the window
    window_manager.minimize_window(window_id)?;
    
    // Restore and maximize
    window_manager.restore_window(window_id)?;
    window_manager.maximize_window(window_id)?;
    
    // Clean up
    window_manager.destroy_window(window_id)?;
    
    Ok(())
}
```

### Advanced Usage

```rust
use window_manager::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut window_manager = WindowManager::new();
    
    // Create multiple windows
    let window1 = window_manager.create_window(
        "Browser".to_string(),
        Rectangle::new(100, 100, 1024, 768),
        WindowStyle::default(),
    )?;
    
    let window2 = window_manager.create_window(
        "Terminal".to_string(),
        Rectangle::new(150, 150, 800, 600),
        WindowStyle::default(),
    )?;
    
    // Manage window ordering
    window_manager.bring_to_front(window1)?;
    window_manager.send_to_back(window2)?;
    
    // Create and switch workspaces
    let dev_workspace = window_manager.create_workspace("Development".to_string())?;
    let design_workspace = window_manager.create_workspace("Design".to_string())?;
    
    window_manager.switch_to_workspace(dev_workspace)?;
    
    // Process events
    let events = window_manager.process_events();
    for event in events {
        println!("Event: {:?}", event);
    }
    
    Ok(())
}
```

### Custom Window Styles

```rust
use window_manager::*;

// Create a borderless tool window
let mut style = WindowStyle::default();
style.has_title_bar = false;
style.has_border = false;
style.resizable = false;
style.always_on_top = true;

let window_id = window_manager.create_window(
    "Tool Window".to_string(),
    Rectangle::new(500, 300, 200, 100),
    style,
)?;
```

### Theme Customization

```rust
use window_manager::*;

// Use dark theme
let dark_style = DecorationStyle::new(DecorationTheme::Dark)
    .with_button_size(28)
    .with_border_width(3);

let renderer = DecorationRenderer::new(dark_style);

// Render decorations for a window
let mut render_context = RenderContext::new();
renderer.render_decorations(window, &mut render_context)?;
```

## API Reference

### WindowManager

#### Core Operations
- `new()` - Create a new window manager instance
- `create_window(title, bounds, style)` - Create a new window
- `destroy_window(id)` - Destroy an existing window
- `get_window(id)` - Get window reference by ID

#### Window Manipulation
- `move_window(id, position)` - Move window to position
- `resize_window(id, size)` - Resize window
- `bring_to_front(id)` - Bring window to front
- `send_to_back(id)` - Send window to back

#### State Management
- `minimize_window(id)` - Minimize window
- `maximize_window(id)` - Maximize window
- `restore_window(id)` - Restore window from minimized/maximized

#### Focus Management
- `set_focus(id)` - Set keyboard focus to window
- `clear_focus()` - Clear keyboard focus
- `focused_window()` - Get currently focused window

#### Workspace Management
- `create_workspace(name)` - Create new workspace
- `switch_to_workspace(id)` - Switch to workspace
- `active_workspace()` - Get current workspace ID
- `windows_in_workspace(id)` - Get windows in workspace

### Window

#### Properties
- `id()` - Get window ID
- `title()` - Get window title
- `bounds()` - Get window bounds (position and size)
- `state()` - Get window state
- `has_focus()` - Check if window has focus

#### Operations
- `set_title(title)` - Set window title
- `set_position(position)` - Set window position
- `set_size(size)` - Set window size
- `set_state(state)` - Set window state

#### Hit Testing
- `contains_point(point)` - Check if point is inside window
- `contains_title_bar(point)` - Check if point is in title bar
- `close_button_bounds()` - Get close button bounds
- `minimize_button_bounds()` - Get minimize button bounds
- `maximize_button_bounds()` - Get maximize button bounds

### WindowStyle

```rust
struct WindowStyle {
    pub resizable: bool,      // Window can be resized
    pub maximizable: bool,    // Window can be maximized
    pub minimizable: bool,    // Window can be minimized
    pub closable: bool,       // Window can be closed
    pub has_title_bar: bool,  // Show title bar
    pub has_border: bool,     // Show window border
    pub always_on_top: bool,  // Always stay on top
}
```

### WindowState

- `Normal` - Normal window state
- `Minimized` - Window is minimized to taskbar/dock
- `Maximized` - Window is maximized to fill screen

### Rectangle

```rust
struct Rectangle {
    pub position: Point,  // Window position (x, y)
    pub size: Size,       // Window size (width, height)
}
```

### Point and Size

```rust
struct Point {
    pub x: i32,
    pub y: i32,
}

struct Size {
    pub width: u32,
    pub height: u32,
}
```

## Examples

The project includes comprehensive examples demonstrating all features:

- **`basic_window_operations.rs`** - Basic window creation and manipulation
- **`workspace_management.rs`** - Multi-desktop workspace management
- **`window_decorations.rs`** - Custom window decorations and themes

Run examples with:
```bash
cargo run --example basic_window_operations
cargo run --example workspace_management
cargo run --example window_decorations
```

## Testing

The window manager includes comprehensive unit tests covering:

- Window creation and destruction
- Window movement and resizing
- State management operations
- Focus management
- Z-order operations
- Workspace management
- Event processing
- Error handling
- Hit testing

Run tests with:
```bash
cargo test
```

## Error Handling

The system uses a comprehensive error handling system with distinct error types:

- `WindowError::WindowNotFound` - Window ID not found
- `WindowError::InvalidBounds` - Invalid window bounds
- `WindowError::InvalidWorkspace` - Invalid workspace ID
- `ZOrderError::AlreadyAtTop` - Window already at top of z-order
- `ZOrderError::AlreadyAtBottom` - Window already at bottom of z-order
- `WorkspaceError::MaxWorkspacesReached` - Maximum workspace limit reached

All operations return `Result<T, E>` types for proper error handling.

## Performance Considerations

- **Lightweight**: Minimal external dependencies
- **Efficient Data Structures**: Uses appropriate collections for performance
- **Event-Driven**: Non-blocking event processing
- **Lazy Updates**: Decorations updated only when needed
- **Memory Management**: Proper cleanup of window resources

## Platform Support

The window manager is designed to be platform-agnostic and can be adapted for:

- Desktop environments
- Embedded systems
- Touch interfaces
- Mobile devices
- VR/AR environments

## Integration

The window manager integrates seamlessly with:
- MultiOS kernel services
- Graphics rendering systems
- Input handling systems
- Process management
- IPC mechanisms

## Contributing

Contributions are welcome! Please ensure:
- All tests pass
- Code follows Rust best practices
- Documentation is updated
- Examples are included for new features

## License

MIT License - see LICENSE file for details.

## Architecture Decisions

### Design Principles
1. **Modularity**: Clear separation of concerns
2. **Extensibility**: Easy to add new features
3. **Performance**: Minimal overhead
4. **Reliability**: Comprehensive error handling
5. **Portability**: Platform-agnostic design

### Key Design Choices
- **Event-Driven**: Allows asynchronous operations
- **Immutable where possible**: Reduces side effects
- **Ownership-based**: Proper resource management
- **Error handling**: Comprehensive error types
- **Thread safety**: Support for concurrent operations

## Future Enhancements

Potential future improvements:
- Window transparency/opacity support
- Advanced window animations
- Virtual desktop thumbnails
- Window snapping and alignment
- Custom window shapes
- Enhanced theme system
- Window grouping and tabbing
- Multi-monitor support
- Accessibility features
- Performance optimizations

## Support

For questions, issues, or contributions:
- Check the examples for usage patterns
- Review the test cases for expected behavior
- Consult the API documentation
- Submit issues for bugs or feature requests
