//! Sprite Demo — Demonstrates sprite rendering with procedurally generated textures
//!
//! This demo generates a simple character sprite sheet in code and renders
//! animated sprites on screen.
//!
//! Run: `cargo run --example sprite_demo -p ember_2d`

use ember_core::app::App;
use ember_core::plugin::Plugin;
use ember_render::camera::Camera2D;
use ember_render::clear_pass::{ClearColor, ClearPassNode};
use ember_render::context::RenderPlugin;
use ember_render::graph::RenderGraph;
use ember_render::window::WindowPlugin;

use ember_2d::atlas::TextureAtlas;
use ember_2d::batch::SpriteDrawQueue;
use ember_2d::sprite::{Sprite, Transform2D};

use glam::{Vec2, Vec4};

/// Generate a simple 128×32 sprite sheet with 4 frames (32×32 each).
/// Each frame is a different colored square with a simple pattern.
fn generate_sprite_sheet() -> (Vec<u8>, u32, u32) {
    let width: u32 = 128;
    let height: u32 = 32;
    let mut pixels = vec![0u8; (width * height * 4) as usize];

    let frame_colors: [(u8, u8, u8); 4] = [
        (220, 80, 80),  // Red frame
        (80, 200, 100), // Green frame
        (80, 120, 220), // Blue frame
        (220, 180, 60), // Yellow frame
    ];

    for (frame_idx, (r, g, b)) in frame_colors.iter().enumerate() {
        let x_offset = frame_idx as u32 * 32;

        for y in 0..32u32 {
            for x in 0..32u32 {
                let px = x_offset + x;
                let idx = ((y * width + px) * 4) as usize;

                // Create a bordered square with a face pattern
                let border = x < 2 || x >= 30 || y < 2 || y >= 30;
                let left_eye = (x >= 8 && x <= 12) && (y >= 8 && y <= 12);
                let right_eye = (x >= 20 && x <= 24) && (y >= 8 && y <= 12);
                let mouth = (x >= 10 && x <= 22)
                    && (y >= 20 && y <= 24)
                    && !(x >= 12 && x <= 20 && y >= 20 && y <= 22);

                if border {
                    pixels[idx] = r / 2;
                    pixels[idx + 1] = g / 2;
                    pixels[idx + 2] = b / 2;
                    pixels[idx + 3] = 255;
                } else if left_eye || right_eye {
                    pixels[idx] = 40;
                    pixels[idx + 1] = 40;
                    pixels[idx + 2] = 40;
                    pixels[idx + 3] = 255;
                } else if mouth {
                    pixels[idx] = 60;
                    pixels[idx + 1] = 30;
                    pixels[idx + 2] = 30;
                    pixels[idx + 3] = 255;
                } else {
                    pixels[idx] = *r;
                    pixels[idx + 1] = *g;
                    pixels[idx + 2] = *b;
                    pixels[idx + 3] = 255;
                }
            }
        }
    }

    (pixels, width, height)
}

/// Plugin that sets up the sprite demo scene.
struct SpriteDemoPlugin;

impl Plugin for SpriteDemoPlugin {
    fn build(&self, app: &mut App) {
        // Camera
        app.insert_resource(Camera2D::new(800.0, 600.0));
        app.insert_resource(ClearColor(0.08, 0.08, 0.12, 1.0));

        // Render graph
        let mut graph = RenderGraph::new();
        graph.add_node("clear", ClearPassNode);
        // SpriteBatchNode would go here in a full setup
        app.insert_resource(graph);

        // Create the atlas (will be used once rendering is fully wired)
        let atlas = TextureAtlas::from_grid(
            0,     // texture_index
            128.0, // texture width
            32.0,  // texture height
            32.0,  // tile width
            32.0,  // tile height
            4,     // columns
            1,     // rows
        );

        // Queue sprites for rendering
        let mut draw_queue = SpriteDrawQueue::new();

        // Place several sprites across the screen
        let positions = [
            (Vec2::new(-200.0, 100.0), 0),
            (Vec2::new(-50.0, 100.0), 1),
            (Vec2::new(100.0, 100.0), 2),
            (Vec2::new(250.0, 100.0), 3),
        ];

        for (pos, frame_idx) in &positions {
            let source_rect = atlas.get_rect(*frame_idx).cloned();
            let sprite = Sprite {
                texture_index: 0,
                source_rect,
                color: Vec4::ONE,
                custom_size: Some(Vec2::new(64.0, 64.0)), // Scale up from 32px
                ..Default::default()
            };
            let transform = Transform2D {
                position: *pos,
                scale: Vec2::ONE,
                ..Default::default()
            };
            draw_queue.draw(&sprite, &transform);
        }

        // Flipped sprite
        let flipped_sprite = Sprite {
            texture_index: 0,
            source_rect: atlas.get_rect(0).cloned(),
            color: Vec4::new(0.8, 0.8, 1.0, 1.0), // Light blue tint
            flip_x: true,
            custom_size: Some(Vec2::new(96.0, 96.0)),
            ..Default::default()
        };
        draw_queue.draw(
            &flipped_sprite,
            &Transform2D {
                position: Vec2::new(0.0, -100.0),
                ..Default::default()
            },
        );

        app.insert_resource(draw_queue);
        app.insert_resource(atlas);

        let (pixels, w, h) = generate_sprite_sheet();
        println!(
            "Sprite Demo — generated {}×{} sprite sheet ({} bytes)",
            w,
            h,
            pixels.len()
        );
        println!("Showing 4 character frames + 1 flipped/tinted sprite");
        println!("Close the window to exit.");
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugin(ember_core::plugin::CorePlugin);
    app.add_plugin(WindowPlugin::default());
    app.add_plugin(RenderPlugin);
    app.add_plugin(SpriteDemoPlugin);
    app.run();
}
