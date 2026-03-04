//! GUI Render Bridge — converts GUI DrawCommands into shape draw calls.
//!
//! This module bridges the immediate-mode GUI system with the `ember_2d` shape
//! rendering pipeline. Each frame, it clears the shape queue, reads the
//! `GuiContext`'s collected draw commands, and converts them into
//! `ShapeDrawQueue` entries for the GPU.
//!
//! Text is rendered as one colored rect per character (block style).

use crate::context::{DrawCommand, GuiContext};
use crate::font::GlyphAtlas;
use ember_2d::shapes::{ShapeDrawQueue, ShapeRect};
use ember_render::camera::Camera2D;
use glam::{Vec2, Vec4};

/// Convert a GUI `Color` to a `Vec4` for the shape pipeline.
fn color_to_vec4(c: &crate::context::Color) -> Vec4 {
    Vec4::new(c.r, c.g, c.b, c.a)
}

/// Convert screen-space coordinates (origin top-left, Y-down) to
/// world-space coordinates (origin center, Y-up) for the camera.
fn screen_to_world(screen_pos: Vec2, viewport_width: f32, viewport_height: f32) -> Vec2 {
    Vec2::new(
        screen_pos.x - viewport_width * 0.5,
        viewport_height * 0.5 - screen_pos.y,
    )
}

/// System that flushes all GUI draw commands into the shape draw queue.
///
/// This should run after all GUI widget code has executed for the frame,
/// but before the render graph executes.
pub fn gui_render_system(
    ctx: ember_core::system::Res<GuiContext>,
    camera: ember_core::system::Res<Camera2D>,
    mut shapes: ember_core::system::ResMut<ShapeDrawQueue>,
    mut font_atlas: ember_core::system::ResMut<GlyphAtlas>,
) {
    // Clear the draw queue each frame so vertices don't accumulate.
    shapes.clear();

    let vw = camera.viewport_width;
    let vh = camera.viewport_height;

    // Maximum vertices per frame to prevent GPU buffer overflow.
    // Each rect = 4 verts. 50,000 verts ≈ 12,500 rects ≈ 1.6 MB (well under 256 MB limit).
    const MAX_VERTICES: usize = 50_000;

    for cmd in &ctx.frame.commands {
        // Stop rendering if we're near the budget
        if shapes.vertices.len() >= MAX_VERTICES {
            break;
        }

        match cmd {
            DrawCommand::Rect {
                pos,
                size,
                color,
                border_color: _,
                border_width: _,
                corner_radius: _,
            } => {
                let center = screen_to_world(
                    Vec2::new(pos.x + size.x * 0.5, pos.y + size.y * 0.5),
                    vw,
                    vh,
                );
                shapes.draw_rect(&ShapeRect::new(center, *size, color_to_vec4(color)));
            }
            DrawCommand::Text {
                pos,
                text,
                font_size,
                color,
            } => {
                // Render text using the builtin pixel font.
                // Each "on" pixel in the 5×7 glyph becomes a small filled rect.
                let glyph_h = font_atlas.builtin_glyph_height() as f32;
                let scale = font_size / glyph_h;
                let pixel_size = scale;
                let text_color = color_to_vec4(color);

                let mut cursor_x = pos.x;

                for ch in text.chars() {
                    if shapes.vertices.len() >= MAX_VERTICES {
                        break;
                    }

                    if let Some(entry) = font_atlas.get_or_insert(ch, *font_size) {
                        let gw = entry.metrics.width;
                        let gh = entry.metrics.height;

                        // Draw each "on" pixel as a tiny rect
                        for row in 0..gh {
                            for col in 0..gw {
                                if shapes.vertices.len() >= MAX_VERTICES {
                                    break;
                                }
                                let idx =
                                    (entry.atlas_y + row) * font_atlas.width + entry.atlas_x + col;
                                if font_atlas.bitmap[idx as usize] > 127 {
                                    let px = cursor_x + col as f32 * pixel_size;
                                    let py = pos.y + row as f32 * pixel_size;
                                    let center = screen_to_world(
                                        Vec2::new(px + pixel_size * 0.5, py + pixel_size * 0.5),
                                        vw,
                                        vh,
                                    );
                                    shapes.draw_rect(&ShapeRect::new(
                                        center,
                                        Vec2::splat(pixel_size),
                                        text_color,
                                    ));
                                }
                            }
                        }

                        cursor_x += entry.metrics.advance_width * scale;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screen_to_world_center() {
        let pos = screen_to_world(Vec2::new(400.0, 300.0), 800.0, 600.0);
        assert!((pos.x - 0.0).abs() < 0.01);
        assert!((pos.y - 0.0).abs() < 0.01);
    }

    #[test]
    fn screen_to_world_top_left() {
        let pos = screen_to_world(Vec2::new(0.0, 0.0), 800.0, 600.0);
        assert!((pos.x - (-400.0)).abs() < 0.01);
        assert!((pos.y - 300.0).abs() < 0.01);
    }

    #[test]
    fn screen_to_world_bottom_right() {
        let pos = screen_to_world(Vec2::new(800.0, 600.0), 800.0, 600.0);
        assert!((pos.x - 400.0).abs() < 0.01);
        assert!((pos.y - (-300.0)).abs() < 0.01);
    }

    #[test]
    fn color_conversion() {
        let c = crate::context::Color::rgba(0.5, 0.6, 0.7, 0.8);
        let v = color_to_vec4(&c);
        assert!((v.x - 0.5).abs() < 0.001);
        assert!((v.y - 0.6).abs() < 0.001);
        assert!((v.z - 0.7).abs() < 0.001);
        assert!((v.w - 0.8).abs() < 0.001);
    }
}
