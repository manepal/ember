use glam::Vec2;

// ---------------------------------------------------------------------------
// GuiRect
// ---------------------------------------------------------------------------

/// A rectangle defined by its top-left position and size.
#[derive(Debug, Clone, Copy)]
pub struct GuiRect {
    pub pos: Vec2,
    pub size: Vec2,
}

impl GuiRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            pos: Vec2::new(x, y),
            size: Vec2::new(width, height),
        }
    }

    pub fn from_pos_size(pos: Vec2, size: Vec2) -> Self {
        Self { pos, size }
    }

    /// Returns true if the point is inside this rectangle (inclusive edges).
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.pos.x
            && point.x <= self.pos.x + self.size.x
            && point.y >= self.pos.y
            && point.y <= self.pos.y + self.size.y
    }

    /// Shrink the rect by padding on all sides.
    pub fn shrink(&self, padding: f32) -> Self {
        Self {
            pos: self.pos + Vec2::splat(padding),
            size: self.size - Vec2::splat(padding * 2.0),
        }
    }
}

// ---------------------------------------------------------------------------
// Anchor
// ---------------------------------------------------------------------------

/// Screen-relative anchor positions for UI elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl Anchor {
    /// Compute the top-left position for a UI element of `element_size`
    /// anchored within a `screen_size` with the given `offset`.
    pub fn resolve(&self, screen_size: Vec2, element_size: Vec2, offset: Vec2) -> Vec2 {
        let base = match self {
            Anchor::TopLeft => Vec2::ZERO,
            Anchor::TopCenter => Vec2::new((screen_size.x - element_size.x) * 0.5, 0.0),
            Anchor::TopRight => Vec2::new(screen_size.x - element_size.x, 0.0),
            Anchor::CenterLeft => Vec2::new(0.0, (screen_size.y - element_size.y) * 0.5),
            Anchor::Center => (screen_size - element_size) * 0.5,
            Anchor::CenterRight => Vec2::new(
                screen_size.x - element_size.x,
                (screen_size.y - element_size.y) * 0.5,
            ),
            Anchor::BottomLeft => Vec2::new(0.0, screen_size.y - element_size.y),
            Anchor::BottomCenter => Vec2::new(
                (screen_size.x - element_size.x) * 0.5,
                screen_size.y - element_size.y,
            ),
            Anchor::BottomRight => screen_size - element_size,
        };
        base + offset
    }
}

// ---------------------------------------------------------------------------
// LayoutCursor
// ---------------------------------------------------------------------------

/// Direction in which a layout places its children.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutDirection {
    Vertical,
    Horizontal,
}

/// Tracks the current position for sequential widget placement within a layout
/// region. Used internally by `begin_vertical` / `begin_horizontal`.
#[derive(Debug, Clone)]
pub struct LayoutCursor {
    /// The bounding rectangle for this layout region.
    pub bounds: GuiRect,
    /// Current draw position (advances as widgets are placed).
    pub cursor: Vec2,
    /// Direction widgets are stacked.
    pub direction: LayoutDirection,
    /// Spacing between widgets in pixels.
    pub spacing: f32,
    /// Number of widgets placed so far.
    pub widget_count: usize,
}

impl LayoutCursor {
    pub fn new(bounds: GuiRect, direction: LayoutDirection, spacing: f32) -> Self {
        Self {
            bounds,
            cursor: bounds.pos,
            direction,
            spacing,
            widget_count: 0,
        }
    }

    /// Allocate space for a widget of the given size.
    /// Returns the position where the widget should be drawn.
    pub fn allocate(&mut self, size: Vec2) -> Vec2 {
        let pos = self.cursor;

        match self.direction {
            LayoutDirection::Vertical => {
                self.cursor.y += size.y + self.spacing;
            }
            LayoutDirection::Horizontal => {
                self.cursor.x += size.x + self.spacing;
            }
        }

        self.widget_count += 1;
        pos
    }

    /// Returns the remaining space available in the layout direction.
    pub fn remaining(&self) -> f32 {
        match self.direction {
            LayoutDirection::Vertical => (self.bounds.pos.y + self.bounds.size.y) - self.cursor.y,
            LayoutDirection::Horizontal => (self.bounds.pos.x + self.bounds.size.x) - self.cursor.x,
        }
    }
}

// ---------------------------------------------------------------------------
// LayoutStack — manages nested layouts
// ---------------------------------------------------------------------------

/// Manages a stack of nested layout cursors, enabling vertical-in-horizontal
/// and other nested layout patterns.
#[derive(Debug, Default)]
pub struct LayoutStack {
    stack: Vec<LayoutCursor>,
}

impl LayoutStack {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Begin a new vertical layout within the given bounds.
    pub fn begin_vertical(&mut self, bounds: GuiRect, spacing: f32) {
        self.stack.push(LayoutCursor::new(
            bounds,
            LayoutDirection::Vertical,
            spacing,
        ));
    }

    /// Begin a new horizontal layout within the given bounds.
    pub fn begin_horizontal(&mut self, bounds: GuiRect, spacing: f32) {
        self.stack.push(LayoutCursor::new(
            bounds,
            LayoutDirection::Horizontal,
            spacing,
        ));
    }

    /// End the current layout and pop it from the stack.
    pub fn end_layout(&mut self) -> Option<LayoutCursor> {
        self.stack.pop()
    }

    /// Allocate space for a widget in the current layout.
    /// Returns the position where the widget should be drawn, or `None` if no layout is active.
    pub fn allocate(&mut self, size: Vec2) -> Option<Vec2> {
        self.stack.last_mut().map(|cursor| cursor.allocate(size))
    }

    /// Returns a reference to the current active layout cursor, if any.
    pub fn current(&self) -> Option<&LayoutCursor> {
        self.stack.last()
    }

    /// Returns true if there are no active layouts.
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gui_rect_contains() {
        let rect = GuiRect::new(10.0, 10.0, 100.0, 50.0);
        assert!(rect.contains(Vec2::new(50.0, 30.0)));
        assert!(rect.contains(Vec2::new(10.0, 10.0))); // edge
        assert!(rect.contains(Vec2::new(110.0, 60.0))); // far edge
        assert!(!rect.contains(Vec2::new(5.0, 30.0))); // left of
        assert!(!rect.contains(Vec2::new(50.0, 70.0))); // below
    }

    #[test]
    fn gui_rect_shrink() {
        let rect = GuiRect::new(10.0, 10.0, 100.0, 50.0);
        let shrunk = rect.shrink(5.0);
        assert_eq!(shrunk.pos.x, 15.0);
        assert_eq!(shrunk.pos.y, 15.0);
        assert_eq!(shrunk.size.x, 90.0);
        assert_eq!(shrunk.size.y, 40.0);
    }

    #[test]
    fn anchor_top_left() {
        let pos =
            Anchor::TopLeft.resolve(Vec2::new(800.0, 600.0), Vec2::new(100.0, 50.0), Vec2::ZERO);
        assert_eq!(pos, Vec2::ZERO);
    }

    #[test]
    fn anchor_center() {
        let pos =
            Anchor::Center.resolve(Vec2::new(800.0, 600.0), Vec2::new(100.0, 50.0), Vec2::ZERO);
        assert_eq!(pos.x, 350.0);
        assert_eq!(pos.y, 275.0);
    }

    #[test]
    fn anchor_bottom_right_with_offset() {
        let pos = Anchor::BottomRight.resolve(
            Vec2::new(800.0, 600.0),
            Vec2::new(100.0, 50.0),
            Vec2::new(-10.0, -10.0),
        );
        assert_eq!(pos.x, 690.0);
        assert_eq!(pos.y, 540.0);
    }

    #[test]
    fn vertical_layout_allocates_top_to_bottom() {
        let bounds = GuiRect::new(10.0, 10.0, 200.0, 400.0);
        let mut cursor = LayoutCursor::new(bounds, LayoutDirection::Vertical, 5.0);

        let pos1 = cursor.allocate(Vec2::new(200.0, 30.0));
        assert_eq!(pos1, Vec2::new(10.0, 10.0));

        let pos2 = cursor.allocate(Vec2::new(200.0, 30.0));
        assert_eq!(pos2, Vec2::new(10.0, 45.0)); // 10 + 30 + 5(spacing)

        let pos3 = cursor.allocate(Vec2::new(200.0, 30.0));
        assert_eq!(pos3, Vec2::new(10.0, 80.0)); // 45 + 30 + 5
    }

    #[test]
    fn horizontal_layout_allocates_left_to_right() {
        let bounds = GuiRect::new(0.0, 0.0, 400.0, 50.0);
        let mut cursor = LayoutCursor::new(bounds, LayoutDirection::Horizontal, 10.0);

        let pos1 = cursor.allocate(Vec2::new(80.0, 50.0));
        assert_eq!(pos1, Vec2::new(0.0, 0.0));

        let pos2 = cursor.allocate(Vec2::new(80.0, 50.0));
        assert_eq!(pos2, Vec2::new(90.0, 0.0)); // 0 + 80 + 10

        assert_eq!(cursor.widget_count, 2);
    }

    #[test]
    fn layout_remaining_space() {
        let bounds = GuiRect::new(0.0, 0.0, 200.0, 400.0);
        let mut cursor = LayoutCursor::new(bounds, LayoutDirection::Vertical, 5.0);

        assert_eq!(cursor.remaining(), 400.0);

        cursor.allocate(Vec2::new(200.0, 100.0));
        assert_eq!(cursor.remaining(), 295.0); // 400 - 100 - 5

        cursor.allocate(Vec2::new(200.0, 50.0));
        assert_eq!(cursor.remaining(), 240.0); // 295 - 50 - 5
    }

    #[test]
    fn layout_stack_nested() {
        let mut stack = LayoutStack::new();
        assert!(stack.is_empty());

        // Start vertical layout
        stack.begin_vertical(GuiRect::new(0.0, 0.0, 200.0, 400.0), 5.0);
        assert!(!stack.is_empty());

        let pos1 = stack.allocate(Vec2::new(200.0, 30.0));
        assert_eq!(pos1, Some(Vec2::new(0.0, 0.0)));

        // Nest a horizontal layout inside
        stack.begin_horizontal(GuiRect::new(0.0, 35.0, 200.0, 30.0), 10.0);
        let h_pos = stack.allocate(Vec2::new(50.0, 30.0));
        assert_eq!(h_pos, Some(Vec2::new(0.0, 35.0)));

        stack.end_layout(); // pop horizontal

        // Continue in vertical
        let pos2 = stack.allocate(Vec2::new(200.0, 30.0));
        assert_eq!(pos2, Some(Vec2::new(0.0, 35.0))); // cursor continued from first vertical alloc

        stack.end_layout(); // pop vertical
        assert!(stack.is_empty());
    }
}
