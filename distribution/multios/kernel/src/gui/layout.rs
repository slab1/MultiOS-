//! Layout management system for GUI toolkit
//! 
//! Provides different layout managers for organizing widgets
//! in containers: flow layout, grid layout, and absolute layout.

use alloc::vec::Vec;
use core::option::Option;

use super::{GUIResult, GUIError, WidgetId};
use super::graphics::{Rectangle, Size, Point};

/// Layout trait for all layout managers
pub trait Layout {
    /// Calculate and apply the layout for child widgets
    fn apply_layout(&self, container_rect: Rectangle, children: &mut [Box<dyn LayoutItem>]) -> GUIResult<()>;
    
    /// Get the preferred size for the layout
    fn get_preferred_size(&self, children: &[Box<dyn LayoutItem>]) -> Size;
    
    /// Get the minimum size for the layout
    fn get_minimum_size(&self, children: &[Box<dyn LayoutItem>]) -> Size;
}

/// Item in a layout (widget with layout constraints)
pub trait LayoutItem {
    /// Get the widget identifier
    fn get_widget_id(&self) -> WidgetId;
    
    /// Get layout constraints
    fn get_constraints(&self) -> &LayoutConstraints;
    
    /// Get the current position and size
    fn get_bounds(&self) -> Rectangle;
    
    /// Set the position and size
    fn set_bounds(&mut self, rect: Rectangle);
    
    /// Get the preferred size
    fn get_preferred_size(&self) -> Size;
    
    /// Check if the item is visible
    fn is_visible(&self) -> bool;
}

/// Layout constraints for positioning and sizing items
#[derive(Debug, Clone, Copy)]
pub struct LayoutConstraints {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub min_width: Option<u32>,
    pub min_height: Option<u32>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub weight_x: Option<f32>,
    pub weight_y: Option<f32>,
    pub anchor_left: bool,
    pub anchor_right: bool,
    pub anchor_top: bool,
    pub anchor_bottom: bool,
}

impl LayoutConstraints {
    pub fn new() -> Self {
        Self {
            x: None,
            y: None,
            width: None,
            height: None,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            weight_x: None,
            weight_y: None,
            anchor_left: false,
            anchor_right: false,
            anchor_top: false,
            anchor_bottom: false,
        }
    }

    /// Set position
    pub fn position(mut self, x: i32, y: i32) -> Self {
        self.x = Some(x);
        self.y = Some(y);
        self
    }

    /// Set size
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Set minimum size
    pub fn min_size(mut self, width: u32, height: u32) -> Self {
        self.min_width = Some(width);
        self.min_height = Some(height);
        self
    }

    /// Set maximum size
    pub fn max_size(mut self, width: u32, height: u32) -> Self {
        self.max_width = Some(width);
        self.max_height = Some(height);
        self
    }

    /// Set weights for flexible sizing
    pub fn weights(mut self, weight_x: f32, weight_y: f32) -> Self {
        self.weight_x = Some(weight_x);
        self.weight_y = Some(weight_y);
        self
    }

    /// Set anchors
    pub fn anchors(mut self, left: bool, right: bool, top: bool, bottom: bool) -> Self {
        self.anchor_left = left;
        self.anchor_right = right;
        self.anchor_top = top;
        self.anchor_bottom = bottom;
        self
    }

    /// Anchor to all edges
    pub fn fill(mut self) -> Self {
        self.anchor_left = true;
        self.anchor_right = true;
        self.anchor_top = true;
        self.anchor_bottom = true;
        self
    }
}

/// Layout manager trait
pub trait LayoutManager {
    /// Apply layout to a container
    fn layout(&self, container_rect: Rectangle, children: &mut [Box<dyn LayoutItem>]) -> GUIResult<()>;
    
    /// Get preferred size of the container
    fn preferred_size(&self, children: &[Box<dyn LayoutItem>]) -> Size;
}

/// Flow layout - arranges widgets in rows or columns
pub struct FlowLayout {
    pub direction: FlowDirection,
    pub spacing: u32,
    pub padding: u32,
    pub alignment: Alignment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Start,
    Center,
    End,
    Stretch,
}

impl FlowLayout {
    pub fn new() -> Self {
        Self {
            direction: FlowDirection::Horizontal,
            spacing: 8,
            padding: 8,
            alignment: Alignment::Start,
        }
    }

    pub fn horizontal() -> Self {
        Self::new().direction(FlowDirection::Horizontal)
    }

    pub fn vertical() -> Self {
        Self::new().direction(FlowDirection::Vertical)
    }

    pub fn direction(mut self, direction: FlowDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn padding(mut self, padding: u32) -> Self {
        self.padding = padding;
        self
    }

    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
}

impl Layout for FlowLayout {
    fn apply_layout(&self, container_rect: Rectangle, children: &mut [Box<dyn LayoutItem>]) -> GUIResult<()> {
        let mut x = container_rect.x + self.padding as i32;
        let mut y = container_rect.y + self.padding as i32;
        let mut max_width = 0u32;
        let mut current_line_height = 0u32;

        for child in children.iter_mut() {
            if !child.is_visible() {
                continue;
            }

            let child_size = child.get_preferred_size();
            
            // Check if we need to wrap to next line/column
            if self.direction == FlowDirection::Horizontal && 
               x + child_size.width as i32 > container_rect.x + container_rect.width as i32 - self.padding as i32 {
                // Wrap to next line
                x = container_rect.x + self.padding as i32;
                y += current_line_height + self.spacing as i32;
                current_line_height = 0;
            }

            // Apply layout based on alignment
            let mut layout_x = x;
            let mut layout_y = y;

            match self.alignment {
                Alignment::Start => {
                    // Use natural position
                }
                Alignment::Center => {
                    if self.direction == FlowDirection::Horizontal {
                        layout_y = container_rect.y + (container_rect.height as i32 - child_size.height as i32) / 2;
                    } else {
                        layout_x = container_rect.x + (container_rect.width as i32 - child_size.width as i32) / 2;
                    }
                }
                Alignment::End => {
                    if self.direction == FlowDirection::Horizontal {
                        layout_y = container_rect.y + container_rect.height as i32 - child_size.height as i32;
                    } else {
                        layout_x = container_rect.x + container_rect.width as i32 - child_size.width as i32;
                    }
                }
                Alignment::Stretch => {
                    // Will be handled by stretching logic below
                }
            }

            // Create the child rectangle
            let child_rect = Rectangle::new(layout_x, layout_y, child_size.width, child_size.height);

            // Stretch if needed
            let final_rect = if self.alignment == Alignment::Stretch {
                if self.direction == FlowDirection::Horizontal {
                    Rectangle::new(layout_x, layout_y, container_rect.width - (self.padding * 2) as u32, child_size.height)
                } else {
                    Rectangle::new(layout_x, layout_y, child_size.width, container_rect.height - (self.padding * 2) as u32)
                }
            } else {
                child_rect
            };

            child.set_bounds(final_rect);

            // Update position for next item
            if self.direction == FlowDirection::Horizontal {
                x = layout_x + child_size.width as i32 + self.spacing as i32;
                current_line_height = current_line_height.max(child_size.height);
                max_width = max_width.max(container_rect.x + (x - container_rect.x) as u32);
            } else {
                y = layout_y + child_size.height as i32 + self.spacing as i32;
                current_line_height = child_size.width; // For vertical, we track width differently
                max_width = max_width.max(child_size.width);
            }
        }

        Ok(())
    }

    fn get_preferred_size(&self, children: &[Box<dyn LayoutItem>]) -> Size {
        if children.is_empty() {
            return Size::new(self.padding * 2, self.padding * 2);
        }

        let mut total_width = self.padding * 2;
        let mut total_height = self.padding * 2;
        let mut current_line_width = self.padding * 2;
        let mut current_line_height = 0u32;

        for child in children.iter() {
            if !child.is_visible() {
                continue;
            }

            let child_size = child.get_preferred_size();
            
            if self.direction == FlowDirection::Horizontal {
                if current_line_width + child_size.width + self.spacing > total_width {
                    total_width = current_line_width;
                    total_height += current_line_height + self.spacing;
                    current_line_width = self.padding * 2;
                    current_line_height = 0;
                }
                current_line_width += child_size.width + self.spacing;
                current_line_height = current_line_height.max(child_size.height);
            } else {
                total_height += child_size.height + self.spacing;
                total_width = total_width.max(child_size.width + self.padding * 2);
            }
        }

        // Add the last line/column
        if self.direction == FlowDirection::Horizontal {
            total_width = total_width.max(current_line_width);
            total_height += current_line_height;
        } else {
            total_height += self.padding * 2;
        }

        Size::new(total_width, total_height)
    }

    fn get_minimum_size(&self, children: &[Box<dyn LayoutItem>]) -> Size {
        // For flow layout, minimum size is similar to preferred size but with minimum dimensions
        self.get_preferred_size(children)
    }
}

impl LayoutManager for FlowLayout {
    fn layout(&self, container_rect: Rectangle, children: &mut [Box<dyn LayoutItem>]) -> GUIResult<()> {
        self.apply_layout(container_rect, children)
    }

    fn preferred_size(&self, children: &[Box<dyn LayoutItem>]) -> Size {
        self.get_preferred_size(children)
    }
}

/// Grid layout - arranges widgets in a grid
pub struct GridLayout {
    pub rows: Option<u32>,
    pub columns: Option<u32>,
    pub spacing: u32,
    pub padding: u32,
    pub horizontal_alignment: Alignment,
    pub vertical_alignment: Alignment,
}

impl GridLayout {
    pub fn new() -> Self {
        Self {
            rows: None,
            columns: None,
            spacing: 8,
            padding: 8,
            horizontal_alignment: Alignment::Center,
            vertical_alignment: Alignment::Center,
        }
    }

    pub fn with_dimensions(rows: u32, columns: u32) -> Self {
        Self::new()
            .rows(Some(rows))
            .columns(Some(columns))
    }

    pub fn rows(mut self, rows: Option<u32>) -> Self {
        self.rows = rows;
        self
    }

    pub fn columns(mut self, columns: Option<u32>) -> Self {
        self.columns = columns;
        self
    }

    pub fn spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn padding(mut self, padding: u32) -> Self {
        self.padding = padding;
        self
    }

    pub fn alignment(mut self, horizontal: Alignment, vertical: Alignment) -> Self {
        self.horizontal_alignment = horizontal;
        self.vertical_alignment = vertical;
        self
    }
}

impl Layout for GridLayout {
    fn apply_layout(&self, container_rect: Rectangle, children: &mut [Box<dyn LayoutItem>]) -> GUIResult<()> {
        if children.is_empty() {
            return Ok(());
        }

        let mut visible_children: Vec<_> = children.iter_mut().filter(|c| c.is_visible()).collect();
        
        if visible_children.is_empty() {
            return Ok(());
        }

        // Calculate grid dimensions
        let (rows, columns) = self.calculate_grid_dimensions(visible_children.len());
        
        // Calculate cell sizes
        let inner_width = container_rect.width - (self.padding * 2) - (columns - 1) * self.spacing;
        let inner_height = container_rect.height - (self.padding * 2) - (rows - 1) * self.spacing;
        
        let cell_width = if columns > 0 { inner_width / columns } else { 0 };
        let cell_height = if rows > 0 { inner_height / rows } else { 0 };

        for (index, child) in visible_children.iter_mut().enumerate() {
            let row = index / columns as usize;
            let col = index % columns as usize;
            
            let cell_x = container_rect.x + self.padding as i32 + (col as u32 * (cell_width + self.spacing)) as i32;
            let cell_y = container_rect.y + self.padding as i32 + (row as u32 * (cell_height + self.spacing)) as i32;
            
            let child_size = child.get_preferred_size();
            let final_width = if child_size.width > cell_width { cell_width } else { child_size.width };
            let final_height = if child_size.height > cell_height { cell_height } else { child_size.height };
            
            // Apply alignment within the cell
            let final_x = match self.horizontal_alignment {
                Alignment::Start => cell_x,
                Alignment::Center => cell_x + ((cell_width - final_width) / 2) as i32,
                Alignment::End => cell_x + (cell_width - final_width) as i32,
                Alignment::Stretch => cell_x,
            };
            
            let final_y = match self.vertical_alignment {
                Alignment::Start => cell_y,
                Alignment::Center => cell_y + ((cell_height - final_height) / 2) as i32,
                Alignment::End => cell_y + (cell_height - final_height) as i32,
                Alignment::Stretch => cell_y,
            };

            let child_rect = if self.horizontal_alignment == Alignment::Stretch || self.vertical_alignment == Alignment::Stretch {
                Rectangle::new(final_x, final_y, cell_width, cell_height)
            } else {
                Rectangle::new(final_x, final_y, final_width, final_height)
            };

            child.set_bounds(child_rect);
        }

        Ok(())
    }

    fn get_preferred_size(&self, children: &[Box<dyn LayoutItem>]) -> Size {
        if children.is_empty() {
            return Size::new(self.padding * 2, self.padding * 2);
        }

        let visible_children: Vec<_> = children.iter().filter(|c| c.is_visible()).collect();
        
        if visible_children.is_empty() {
            return Size::new(self.padding * 2, self.padding * 2);
        }

        let (rows, columns) = self.calculate_grid_dimensions(visible_children.len());
        
        let mut max_child_width = 0u32;
        let mut max_child_height = 0u32;
        
        for child in visible_children.iter() {
            let size = child.get_preferred_size();
            max_child_width = max_child_width.max(size.width);
            max_child_height = max_child_height.max(size.height);
        }
        
        let width = self.padding * 2 + columns * max_child_width + (columns - 1) * self.spacing;
        let height = self.padding * 2 + rows * max_child_height + (rows - 1) * self.spacing;
        
        Size::new(width, height)
    }

    fn get_minimum_size(&self, children: &[Box<dyn LayoutItem>]) -> Size {
        // For grid layout, minimum size considers minimum required cell sizes
        let preferred = self.get_preferred_size(children);
        Size::new(preferred.width.max(100), preferred.height.max(100))
    }
}

impl GridLayout {
    /// Calculate optimal grid dimensions
    fn calculate_grid_dimensions(&self, child_count: usize) -> (u32, u32) {
        match (self.rows, self.columns) {
            (Some(rows), Some(columns)) => (rows, columns),
            (Some(rows), None) => {
                let columns = ((child_count as f32) / (rows as f32)).ceil() as u32;
                (rows, columns.max(1))
            }
            (None, Some(columns)) => {
                let rows = ((child_count as f32) / (columns as f32)).ceil() as u32;
                (rows.max(1), columns)
            }
            (None, None) => {
                // Auto-calculate based on aspect ratio
                let columns = ((child_count as f32).sqrt().ceil() as u32).max(1);
                let rows = ((child_count as f32) / (columns as f32)).ceil() as u32;
                (rows.max(1), columns)
            }
        }
    }
}

impl LayoutManager for GridLayout {
    fn layout(&self, container_rect: Rectangle, children: &mut [Box<dyn LayoutItem>]) -> GUIResult<()> {
        self.apply_layout(container_rect, children)
    }

    fn preferred_size(&self, children: &[Box<dyn LayoutItem>]) -> Size {
        self.get_preferred_size(children)
    }
}

/// Absolute layout - allows precise positioning of widgets
pub struct AbsoluteLayout {
    pub padding: u32,
}

impl AbsoluteLayout {
    pub fn new() -> Self {
        Self {
            padding: 8,
        }
    }

    pub fn padding(mut self, padding: u32) -> Self {
        self.padding = padding;
        self
    }
}

impl Layout for AbsoluteLayout {
    fn apply_layout(&self, container_rect: Rectangle, children: &mut [Box<dyn LayoutItem>]) -> GUIResult<()> {
        for child in children.iter_mut() {
            if !child.is_visible() {
                continue;
            }

            let constraints = child.get_constraints();
            let child_size = child.get_preferred_size();
            
            // Calculate position
            let x = constraints.x.unwrap_or(container_rect.x + self.padding as i32);
            let y = constraints.y.unwrap_or(container_rect.y + self.padding as i32);
            
            // Calculate size
            let width = constraints.width.unwrap_or(child_size.width);
            let height = constraints.height.unwrap_or(child_size.height);
            
            // Apply min/max constraints
            let final_width = constraints.min_width.map(|min| width.max(min)).unwrap_or(width);
            let final_height = constraints.min_height.map(|min| height.max(min)).unwrap_or(height);
            
            if let Some(max_w) = constraints.max_width {
                let final_width = final_width.min(max_w);
            }
            if let Some(max_h) = constraints.max_height {
                let final_height = final_height.min(max_h);
            }
            
            // Handle anchors
            let final_x = if constraints.anchor_left && constraints.anchor_right {
                container_rect.x
            } else if constraints.anchor_right {
                container_rect.x + container_rect.width as i32 - final_width as i32
            } else {
                x
            };
            
            let final_y = if constraints.anchor_top && constraints.anchor_bottom {
                container_rect.y
            } else if constraints.anchor_bottom {
                container_rect.y + container_rect.height as i32 - final_height as i32
            } else {
                y
            };
            
            let child_rect = Rectangle::new(final_x, final_y, final_width, final_height);
            child.set_bounds(child_rect);
        }
        
        Ok(())
    }

    fn get_preferred_size(&self, children: &[Box<dyn LayoutItem>]) -> Size {
        if children.is_empty() {
            return Size::new(self.padding * 2, self.padding * 2);
        }

        let mut max_x = 0i32;
        let mut max_y = 0i32;
        
        for child in children.iter() {
            if !child.is_visible() {
                continue;
            }

            let constraints = child.get_constraints();
            let child_size = child.get_preferred_size();
            
            let x = constraints.x.unwrap_or(0);
            let y = constraints.y.unwrap_or(0);
            
            max_x = max_x.max(x + child_size.width as i32);
            max_y = max_y.max(y + child_size.height as i32);
        }
        
        Size::new(
            max_x as u32 + self.padding * 2,
            max_y as u32 + self.padding * 2
        )
    }

    fn get_minimum_size(&self, children: &[Box<dyn LayoutItem>]) -> Size {
        self.get_preferred_size(children)
    }
}

impl LayoutManager for AbsoluteLayout {
    fn layout(&self, container_rect: Rectangle, children: &mut [Box<dyn LayoutItem>]) -> GUIResult<()> {
        self.apply_layout(container_rect, children)
    }

    fn preferred_size(&self, children: &[Box<dyn LayoutItem>]) -> Size {
        self.get_preferred_size(children)
    }
}

/// Border layout - arranges widgets around edges and center
pub struct BorderLayout {
    pub spacing: u32,
    pub padding: u32,
}

impl BorderLayout {
    pub fn new() -> Self {
        Self {
            spacing: 8,
            padding: 8,
        }
    }

    pub fn spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn padding(mut self, padding: u32) -> Self {
        self.padding = padding;
        self
    }
}

impl Layout for BorderLayout {
    fn apply_layout(&self, container_rect: Rectangle, children: &mut [Box<dyn LayoutItem>]) -> GUIResult<()> {
        // This is a simplified implementation
        // In a full implementation, you'd want to categorize children by their border position
        // and handle the center widget specially
        
        for child in children.iter_mut() {
            if !child.is_visible() {
                continue;
            }

            let child_size = child.get_preferred_size();
            let child_rect = Rectangle::new(
                container_rect.x + self.padding as i32,
                container_rect.y + self.padding as i32,
                child_size.width,
                child_size.height
            );
            
            child.set_bounds(child_rect);
        }
        
        Ok(())
    }

    fn get_preferred_size(&self, children: &[Box<dyn LayoutItem>]) -> Size {
        if children.is_empty() {
            return Size::new(self.padding * 2, self.padding * 2);
        }

        let mut max_width = self.padding * 2;
        let mut max_height = self.padding * 2;
        
        for child in children.iter() {
            if !child.is_visible() {
                continue;
            }

            let size = child.get_preferred_size();
            max_width = max_width.max(size.width + self.padding * 2);
            max_height = max_height.max(size.height + self.padding * 2);
        }
        
        Size::new(max_width, max_height)
    }

    fn get_minimum_size(&self, children: &[Box<dyn LayoutItem>]) -> Size {
        self.get_preferred_size(children)
    }
}

impl LayoutManager for BorderLayout {
    fn layout(&self, container_rect: Rectangle, children: &mut [Box<dyn LayoutItem>]) -> GUIResult<()> {
        self.apply_layout(container_rect, children)
    }

    fn preferred_size(&self, children: &[Box<dyn LayoutItem>]) -> Size {
        self.get_preferred_size(children)
    }
}