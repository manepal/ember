use glam::Vec2;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// ---------------------------------------------------------------------------
// WidgetId
// ---------------------------------------------------------------------------

/// A unique identifier for a widget, computed by hashing its label and parent chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetId(pub u64);

impl WidgetId {
    /// Create a widget ID from a label string and the current ID stack.
    pub fn new(label: &str, id_stack: &[u64]) -> Self {
        let mut hasher = DefaultHasher::new();
        for parent_id in id_stack {
            parent_id.hash(&mut hasher);
        }
        label.hash(&mut hasher);
        WidgetId(hasher.finish())
    }
}

// ---------------------------------------------------------------------------
// WidgetResponse
// ---------------------------------------------------------------------------

/// The response returned by every widget after processing, describing its
/// interaction state for the current frame.
#[derive(Debug, Clone, Copy, Default)]
pub struct WidgetResponse {
    /// The widget was clicked (mouse released over widget while it was active).
    pub clicked: bool,
    /// The mouse cursor is currently over the widget.
    pub hovered: bool,
    /// The widget is being pressed (mouse down on it).
    pub active: bool,
    /// The widget is being dragged (mouse moved while active).
    pub dragged: bool,
}

// ---------------------------------------------------------------------------
// DrawCommand
// ---------------------------------------------------------------------------

/// A color represented as RGBA floats in 0.0–1.0 range.
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);
    pub const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);
    pub const TRANSPARENT: Self = Self::rgba(0.0, 0.0, 0.0, 0.0);
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

/// A single draw command emitted by a widget during a frame.
#[derive(Debug, Clone)]
pub enum DrawCommand {
    /// Draw a filled rectangle.
    Rect {
        pos: Vec2,
        size: Vec2,
        color: Color,
        border_color: Color,
        border_width: f32,
        corner_radius: f32,
    },
    /// Draw a text string.
    Text {
        pos: Vec2,
        text: String,
        font_size: f32,
        color: Color,
    },
}

// ---------------------------------------------------------------------------
// GuiFrame — per-frame draw command collector
// ---------------------------------------------------------------------------

/// Collects all draw commands emitted during a single frame.
#[derive(Debug, Default)]
pub struct GuiFrame {
    pub commands: Vec<DrawCommand>,
}

impl GuiFrame {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn draw_rect(
        &mut self,
        pos: Vec2,
        size: Vec2,
        color: Color,
        border_color: Color,
        border_width: f32,
        corner_radius: f32,
    ) {
        self.commands.push(DrawCommand::Rect {
            pos,
            size,
            color,
            border_color,
            border_width,
            corner_radius,
        });
    }

    pub fn draw_text(&mut self, pos: Vec2, text: String, font_size: f32, color: Color) {
        self.commands.push(DrawCommand::Text {
            pos,
            text,
            font_size,
            color,
        });
    }
}

// ---------------------------------------------------------------------------
// GuiContext — the core state resource
// ---------------------------------------------------------------------------

/// The main GUI context resource, inserted into the ECS world.
/// Tracks interaction state (hot/active/focused widgets) and collects draw commands.
pub struct GuiContext {
    /// The widget currently under the mouse cursor.
    pub hot: Option<WidgetId>,
    /// The widget being interacted with (mouse button held down on it).
    pub active: Option<WidgetId>,
    /// The widget that has keyboard focus (for text input).
    pub focused: Option<WidgetId>,

    /// Current mouse cursor position in screen coordinates.
    pub cursor_pos: Vec2,
    /// Whether the primary mouse button is currently pressed.
    pub mouse_down: bool,
    /// Whether the primary mouse button was just pressed this frame.
    pub mouse_just_pressed: bool,
    /// Whether the primary mouse button was just released this frame.
    pub mouse_just_released: bool,

    /// Parent ID chain for nested widget scoping.
    pub id_stack: Vec<u64>,

    /// Draw commands collected this frame.
    pub frame: GuiFrame,
}

impl Default for GuiContext {
    fn default() -> Self {
        Self::new()
    }
}

impl GuiContext {
    pub fn new() -> Self {
        Self {
            hot: None,
            active: None,
            focused: None,
            cursor_pos: Vec2::ZERO,
            mouse_down: false,
            mouse_just_pressed: false,
            mouse_just_released: false,
            id_stack: Vec::new(),
            frame: GuiFrame::new(),
        }
    }

    /// Generate a `WidgetId` from a label, scoped by the current ID stack.
    pub fn make_id(&self, label: &str) -> WidgetId {
        WidgetId::new(label, &self.id_stack)
    }

    /// Push a parent scope onto the ID stack (for nesting widgets inside panels).
    pub fn push_id(&mut self, label: &str) {
        let id = self.make_id(label);
        self.id_stack.push(id.0);
    }

    /// Pop the last parent scope from the ID stack.
    pub fn pop_id(&mut self) {
        self.id_stack.pop();
    }

    /// Begin a new frame — clears transient state and draw commands.
    pub fn begin_frame(
        &mut self,
        cursor_pos: Vec2,
        mouse_down: bool,
        mouse_just_pressed: bool,
        mouse_just_released: bool,
    ) {
        self.cursor_pos = cursor_pos;
        self.mouse_down = mouse_down;
        self.mouse_just_pressed = mouse_just_pressed;
        self.mouse_just_released = mouse_just_released;
        self.hot = None;
        self.frame.clear();
    }

    /// Test whether a point is inside a rectangle.
    pub fn hit_test(&self, pos: Vec2, size: Vec2) -> bool {
        let cursor = self.cursor_pos;
        cursor.x >= pos.x
            && cursor.x <= pos.x + size.x
            && cursor.y >= pos.y
            && cursor.y <= pos.y + size.y
    }

    /// Process widget interaction logic.
    ///
    /// Call this in every widget to determine hover/active/click state.
    /// Returns the interaction response for this frame.
    pub fn widget_interaction(&mut self, id: WidgetId, pos: Vec2, size: Vec2) -> WidgetResponse {
        let hovered = self.hit_test(pos, size);
        let mut response = WidgetResponse::default();

        if hovered {
            self.hot = Some(id);
            response.hovered = true;
        }

        // Mouse just pressed: if hovering, make this widget active
        if self.mouse_just_pressed && hovered {
            self.active = Some(id);
        }

        // Widget is active (being held)
        if self.active == Some(id) {
            response.active = true;

            // If mouse moved while active, it's a drag
            if self.mouse_down && !self.mouse_just_pressed {
                response.dragged = true;
            }

            // Mouse released: if still hovering, it's a click
            if self.mouse_just_released {
                if hovered {
                    response.clicked = true;
                }
                self.active = None;
            }
        }

        response
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widget_id_same_label_same_id() {
        let stack: Vec<u64> = vec![];
        let id1 = WidgetId::new("button_ok", &stack);
        let id2 = WidgetId::new("button_ok", &stack);
        assert_eq!(id1, id2);
    }

    #[test]
    fn widget_id_different_label_different_id() {
        let stack: Vec<u64> = vec![];
        let id1 = WidgetId::new("button_ok", &stack);
        let id2 = WidgetId::new("button_cancel", &stack);
        assert_ne!(id1, id2);
    }

    #[test]
    fn widget_id_parent_scope_changes_id() {
        let id_no_parent = WidgetId::new("save", &[]);
        let id_with_parent = WidgetId::new("save", &[12345]);
        assert_ne!(id_no_parent, id_with_parent);
    }

    #[test]
    fn hit_test_inside() {
        let ctx = GuiContext {
            cursor_pos: Vec2::new(50.0, 50.0),
            ..GuiContext::new()
        };
        assert!(ctx.hit_test(Vec2::new(10.0, 10.0), Vec2::new(100.0, 100.0)));
    }

    #[test]
    fn hit_test_outside() {
        let ctx = GuiContext {
            cursor_pos: Vec2::new(200.0, 200.0),
            ..GuiContext::new()
        };
        assert!(!ctx.hit_test(Vec2::new(10.0, 10.0), Vec2::new(100.0, 100.0)));
    }

    #[test]
    fn hit_test_on_edge() {
        let ctx = GuiContext {
            cursor_pos: Vec2::new(10.0, 10.0),
            ..GuiContext::new()
        };
        assert!(ctx.hit_test(Vec2::new(10.0, 10.0), Vec2::new(100.0, 100.0)));
    }

    #[test]
    fn widget_interaction_hover() {
        let mut ctx = GuiContext::new();
        ctx.begin_frame(Vec2::new(50.0, 50.0), false, false, false);

        let id = ctx.make_id("test_button");
        let response = ctx.widget_interaction(id, Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));

        assert!(response.hovered);
        assert!(!response.clicked);
        assert!(!response.active);
        assert_eq!(ctx.hot, Some(id));
    }

    #[test]
    fn widget_interaction_click() {
        let mut ctx = GuiContext::new();
        let id = ctx.make_id("test_button");
        let pos = Vec2::new(0.0, 0.0);
        let size = Vec2::new(100.0, 100.0);

        // Frame 1: mouse just pressed while hovering → becomes active
        ctx.begin_frame(Vec2::new(50.0, 50.0), true, true, false);
        let response = ctx.widget_interaction(id, pos, size);
        assert!(response.hovered);
        assert!(response.active);
        assert!(!response.clicked);
        assert_eq!(ctx.active, Some(id));

        // Frame 2: mouse released while still hovering → clicked
        ctx.begin_frame(Vec2::new(50.0, 50.0), false, false, true);
        let response = ctx.widget_interaction(id, pos, size);
        assert!(response.clicked);
        assert!(response.hovered);
        assert_eq!(ctx.active, None); // released
    }

    #[test]
    fn widget_interaction_press_and_move_away() {
        let mut ctx = GuiContext::new();
        let id = ctx.make_id("test_button");
        let pos = Vec2::new(0.0, 0.0);
        let size = Vec2::new(100.0, 100.0);

        // Frame 1: press while hovering
        ctx.begin_frame(Vec2::new(50.0, 50.0), true, true, false);
        ctx.widget_interaction(id, pos, size);
        assert_eq!(ctx.active, Some(id));

        // Frame 2: release but cursor moved away → no click
        ctx.begin_frame(Vec2::new(200.0, 200.0), false, false, true);
        let response = ctx.widget_interaction(id, pos, size);
        assert!(!response.clicked);
        assert!(!response.hovered);
        assert_eq!(ctx.active, None);
    }

    #[test]
    fn push_pop_id_stack() {
        let mut ctx = GuiContext::new();
        assert!(ctx.id_stack.is_empty());

        ctx.push_id("panel_a");
        assert_eq!(ctx.id_stack.len(), 1);

        let id_inside = ctx.make_id("button");
        ctx.pop_id();

        let id_outside = ctx.make_id("button");
        assert_ne!(
            id_inside, id_outside,
            "Same label in different scopes should produce different IDs"
        );
    }

    #[test]
    fn gui_frame_collects_commands() {
        let mut frame = GuiFrame::new();
        assert!(frame.commands.is_empty());

        frame.draw_rect(
            Vec2::ZERO,
            Vec2::new(100.0, 50.0),
            Color::WHITE,
            Color::BLACK,
            1.0,
            0.0,
        );
        frame.draw_text(
            Vec2::new(10.0, 10.0),
            "Hello".to_string(),
            16.0,
            Color::WHITE,
        );

        assert_eq!(frame.commands.len(), 2);

        frame.clear();
        assert!(frame.commands.is_empty());
    }
}
