use crate::context::GuiContext;
use crate::font::GlyphAtlas;
use crate::overlay::DebugOverlayConfig;
use crate::style::GuiTheme;
use ember_core::app::App;
use ember_core::plugin::Plugin;
use ember_core::system::{Res, ResMut};
use ember_input::MouseState;
use ember_render::camera::Camera2D;
use ember_render::window::EmberWindow;

/// System that syncs input state from `ember_input` into the `GuiContext`.
pub fn gui_input_sync_system(
    mouse: Res<MouseState>,
    camera: Res<Camera2D>,
    window: Res<EmberWindow>,
    mut ctx: ResMut<GuiContext>,
) {
    // Map window physical coordinates to camera logical coordinates
    // (e.g. retina 1600x1200 physical -> 800x600 logical)
    let scale_x = if window.width > 0 {
        camera.viewport_width / window.width as f32
    } else {
        1.0
    };
    let scale_y = if window.height > 0 {
        camera.viewport_height / window.height as f32
    } else {
        1.0
    };

    let cursor_pos = glam::Vec2::new(mouse.position.x * scale_x, mouse.position.y * scale_y);
    let mouse_down = mouse.is_pressed(ember_input::mouse::MouseButton::Left);
    let just_pressed = mouse.is_just_pressed(ember_input::mouse::MouseButton::Left);
    let just_released = mouse.is_just_released(ember_input::mouse::MouseButton::Left);

    ctx.begin_frame(cursor_pos, mouse_down, just_pressed, just_released);
}

/// The GUI plugin. Registers the `GuiContext`, `GuiTheme`, `GlyphAtlas`,
/// `DebugOverlayConfig` resources, and the input synchronization system.
pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GuiContext::new());
        app.insert_resource(GuiTheme::default());
        app.insert_resource(GlyphAtlas::with_builtin_font());
        app.insert_resource(DebugOverlayConfig::default());
        app.add_system::<fn(
            Res<'static, MouseState>,
            Res<'static, Camera2D>,
            Res<'static, EmberWindow>,
            ResMut<'static, GuiContext>,
        ), _>(gui_input_sync_system);
    }
}
