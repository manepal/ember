//! Debug Overlay — built-in debug HUD rendered via `ember_gui`.
//!
//! Provides a toggleable overlay showing FPS, frame time, entity count,
//! and render stats. Can be toggled at runtime via `DebugOverlay::toggle()`.

use crate::context::{Color, GuiContext};
use crate::style::WidgetStyle;
use crate::widgets;
use ember_core::system::{Res, ResMut};
use ember_core::time::Time;
use glam::Vec2;

/// Configuration for the debug overlay.
pub struct DebugOverlayConfig {
    /// Whether the overlay is currently visible.
    pub visible: bool,
    /// Whether to show FPS and frame time.
    pub show_fps: bool,
    /// Whether to show frame count.
    pub show_frame_count: bool,
    /// Whether to show elapsed time.
    pub show_elapsed: bool,
    /// Whether to show mouse position.
    pub show_mouse: bool,
    /// Position (top-left corner) of the overlay.
    pub position: Vec2,
    /// Font size for overlay text.
    pub font_size: f32,
}

impl Default for DebugOverlayConfig {
    fn default() -> Self {
        Self {
            visible: true,
            show_fps: true,
            show_frame_count: true,
            show_elapsed: true,
            show_mouse: true,
            position: Vec2::new(5.0, 5.0),
            font_size: 12.0,
        }
    }
}

impl DebugOverlayConfig {
    /// Toggle overlay visibility on/off.
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }
}

/// Style used for the debug overlay (semi-transparent dark background, green text).
fn debug_style() -> WidgetStyle {
    WidgetStyle {
        bg_color: Color::rgba(0.0, 0.0, 0.0, 0.65),
        bg_color_hovered: Color::rgba(0.0, 0.0, 0.0, 0.65),
        bg_color_active: Color::rgba(0.0, 0.0, 0.0, 0.65),
        text_color: Color::rgba(0.0, 1.0, 0.4, 1.0),
        border_color: Color::rgba(0.0, 0.6, 0.3, 0.5),
        border_width: 1.0,
        corner_radius: 4.0,
        font_size: 12.0,
        padding: 4.0,
    }
}

/// System that draws the debug overlay.
///
/// Register this system in your app to get a live FPS/frame-time HUD.
/// Control visibility via the `DebugOverlayConfig` resource.
pub fn debug_overlay_system(
    mut ctx: ResMut<GuiContext>,
    time: Res<Time>,
    config: Res<DebugOverlayConfig>,
) {
    if !config.visible {
        return;
    }

    let style = debug_style();
    let x = config.position.x;
    let mut y = config.position.y;
    let line_h = config.font_size + 4.0;
    let panel_w = 180.0;

    // Calculate panel height based on what's shown
    let mut lines = 0;
    if config.show_fps {
        lines += 1;
    }
    if config.show_frame_count {
        lines += 1;
    }
    if config.show_elapsed {
        lines += 1;
    }
    if config.show_mouse {
        lines += 1;
    }
    let panel_h = lines as f32 * line_h + style.padding * 2.0;

    // Background panel
    widgets::panel(
        &mut ctx,
        Vec2::new(x, y),
        Vec2::new(panel_w, panel_h),
        &style,
    );

    y += style.padding;
    let text_x = x + style.padding;

    if config.show_fps {
        let dt = time.delta_seconds();
        let fps = if dt > 0.0 { 1.0 / dt } else { 0.0 };
        let text = format!("FPS: {:.0}  ({:.1}ms)", fps, dt * 1000.0);
        widgets::label(&mut ctx, Vec2::new(text_x, y), &text, &style);
        y += line_h;
    }

    if config.show_frame_count {
        let text = format!("Frame: {}", time.frame_count());
        widgets::label(&mut ctx, Vec2::new(text_x, y), &text, &style);
        y += line_h;
    }

    if config.show_elapsed {
        let text = format!("Elapsed: {:.1}s", time.elapsed_seconds());
        widgets::label(&mut ctx, Vec2::new(text_x, y), &text, &style);
        y += line_h;
    }

    if config.show_mouse {
        let text = format!("Mouse: ({:.0}, {:.0})", ctx.cursor_pos.x, ctx.cursor_pos.y);
        widgets::label(&mut ctx, Vec2::new(text_x, y), &text, &style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_visible() {
        let config = DebugOverlayConfig::default();
        assert!(config.visible);
        assert!(config.show_fps);
    }

    #[test]
    fn toggle_visibility() {
        let mut config = DebugOverlayConfig::default();
        assert!(config.visible);
        config.toggle();
        assert!(!config.visible);
        config.toggle();
        assert!(config.visible);
    }

    #[test]
    fn debug_style_has_green_text() {
        let style = debug_style();
        assert!(style.text_color.g > 0.8); // Green channel dominant
        assert!(style.text_color.r < 0.1);
    }
}
