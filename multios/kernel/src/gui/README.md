# MultiOS GUI Toolkit

A comprehensive GUI toolkit for the MultiOS educational operating system, providing a complete foundation for graphical user interfaces.

## Features

- **Complete Widget Set**: Buttons, text fields, labels, menus, dialogs, list boxes, progress bars
- **Advanced Event System**: Mouse, keyboard, window, and focus events with safe dispatching
- **Flexible Layout Management**: Flow, grid, absolute, and border layout managers
- **Comprehensive Styling**: Theme-based styling with multiple built-in themes
- **Window Management**: Multi-window support with z-order management
- **Memory Safe**: Built with Rust for guaranteed memory safety
- **Extensible Architecture**: Easy to add custom widgets and extend functionality

## Quick Start

```rust
use multios_kernel::gui::{init, manager::GUIManager};

// Initialize the GUI system
init()?;

// Create GUI manager
let mut gui_manager = GUIManager::new(1024, 768);
gui_manager.initialize()?;

// Create a window
let app_id = gui_manager.create_application("My App");
let window_id = gui_manager.create_window_for_app(app_id, "Hello MultiOS", 400, 300)?;

// Create some widgets
use multios_kernel::gui::widgets::{Button, Label, Container};

let mut container = Container::new("main");
container.add_child(Box::new(Label::new("Hello, MultiOS!")))?;
container.add_child(Box::new(Button::new("Click Me!")))?;

// Add to window
if let Some(window) = gui_manager.get_window_manager_mut().get_window_mut(window_id) {
    window.set_content(Box::new(container));
    window.show();
}

// Render the GUI
gui_manager.render()?;
```

## Widgets

### Basic Widgets

- **Container**: Base widget for holding other widgets
- **Button**: Interactive button with click handling
- **Label**: Static text display widget
- **TextField**: Editable text input widget

### Advanced Widgets

- **Menu**: Dropdown menu with hierarchical items
- **Dialog**: Modal and modeless dialog windows
- **ListBox**: Scrollable list with selection
- **ProgressBar**: Progress indication widget

## Layout Managers

- **FlowLayout**: Arranges widgets in horizontal or vertical flow
- **GridLayout**: Places widgets in a grid structure
- **AbsoluteLayout**: Precise positioning with constraints
- **BorderLayout**: Edge-based arrangement

## Styling

The GUI toolkit supports comprehensive styling through the style system:

```rust
use multios_kernel::gui::style::{Style, Color, Border};

// Create custom style
let style = Style::new("custom")
    .background_color(Color::BLUE)
    .foreground_color(Color::WHITE)
    .padding(12)
    .border(Border::new(2, Color::DARK_BLUE, BorderStyle::Solid));

widget.set_style(style);
```

### Built-in Themes

- **Default**: Standard appearance
- **Light**: Light color scheme
- **Dark**: Dark color scheme
- **High Contrast**: Accessibility-focused theme

## Event Handling

The event system provides comprehensive event handling:

```rust
impl EventHandler for MyWidget {
    fn handle_event(&self, event: &mut Event) -> bool {
        match event.event_type {
            EventType::MouseDown => {
                if let EventData::Mouse(mouse_data) = event.get_data() {
                    // Handle mouse click
                    event.consume();
                    return true;
                }
            }
            _ => {}
        }
        false
    }
    
    fn can_handle(&self, event_type: EventType) -> bool {
        matches!(event_type, EventType::MouseDown | EventType::MouseUp)
    }
}
```

## Window Management

The window manager provides full window management capabilities:

```rust
// Create and manage windows
let window_id = gui_manager.create_window("My Window", 800, 600)?;

// Window operations
if let Some(window) = gui_manager.get_window_manager_mut().get_window_mut(window_id) {
    window.resize(1024, 768);
    window.move_to(100, 100);
    window.bring_to_front();
    window.close();
}
```

## Architecture

The GUI toolkit is built with a modular architecture:

- **Core**: Main module with exports and initialization
- **Graphics**: Rendering and drawing primitives
- **Events**: Event handling and dispatching system
- **Style**: Styling and theming system
- **Layout**: Layout management algorithms
- **Widgets**: Base widgets and implementations
- **Manager**: Window and application management

## Memory Management

The GUI toolkit uses Rust's ownership system for safe memory management:

- No manual memory management required
- Automatic cleanup through Drop implementations
- No memory leaks possible
- Thread-safe operations

## Integration

The GUI toolkit is fully integrated into the MultiOS kernel:

```rust
// In kernel initialization
fn kernel_main(arch: ArchType, boot_info: &BootInfo) -> KernelResult<()> {
    // ... other initialization ...
    
    // Initialize GUI system
    gui::init()?;
    
    // ... rest of initialization ...
}
```

## Examples

See the `examples.rs` file for comprehensive examples including:

- Basic widget usage
- Layout management examples
- Event handling patterns
- Custom widget creation
- Application frameworks

## Testing

The GUI toolkit includes comprehensive tests:

```bash
# Run GUI-specific tests
cargo test gui

# Run all kernel tests
cargo test
```

## Performance

The GUI toolkit is designed for performance:

- Efficient framebuffer operations
- Minimal redraw through dirty flag tracking
- Optimized event dispatching
- Memory-efficient widget storage

## Future Enhancements

Planned improvements include:

- Complete text rendering system
- Animation framework
- Hardware acceleration support
- Multi-monitor support
- Accessibility features
- Touch input support
- File system integration for icons and resources

## Contributing

The GUI toolkit is designed to be extensible. To add new widgets:

1. Implement the `Widget` trait
2. Add event handling if needed
3. Provide rendering implementation
4. Add to module exports

## License

This GUI toolkit is part of the MultiOS project and follows the same licensing terms.