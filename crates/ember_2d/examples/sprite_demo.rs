//! Sprite Demo — Demonstrates sprite rendering with procedurally generated textures
//!
//! This demo generates a simple character sprite sheet in code, uploads it to the
//! GPU, and renders sprites on screen using the full rendering pipeline.
//!
//! Run: `cargo run --example sprite_demo -p ember_2d`

use ember_core::app::App;
use ember_core::plugin::Plugin;
use ember_render::camera::Camera2D;
use ember_render::clear_pass::{ClearColor, ClearPassNode};
use ember_render::context::{GpuStartupCallbacks, RenderContext, RenderPlugin};
use ember_render::graph::RenderGraph;
use ember_render::window::WindowPlugin;

use ember_2d::atlas::TextureAtlas;
use ember_2d::batch::{SpriteBatchNode, SpriteBatchResources, SpriteDrawQueue};
use ember_2d::sprite::{Sprite, Transform2D};
use ember_2d::texture::{Texture, TextureStore};

use glam::{Vec2, Vec4};

/// Generate a simple 128×32 sprite sheet with 4 frames (32×32 each).
/// Each frame is a different colored square with a simple face pattern.
fn generate_sprite_sheet() -> (Vec<u8>, u32, u32) {
    let width: u32 = 128;
    let height: u32 = 32;
    let mut pixels = vec![0u8; (width * height * 4) as usize];

    let frame_colors: [(u8, u8, u8); 4] = [
        (220, 80, 80),  // Red
        (80, 200, 100), // Green
        (80, 120, 220), // Blue
        (220, 180, 60), // Yellow
    ];

    for (frame_idx, (r, g, b)) in frame_colors.iter().enumerate() {
        let x_offset = frame_idx as u32 * 32;
        for y in 0..32u32 {
            for x in 0..32u32 {
                let px = x_offset + x;
                let idx = ((y * width + px) * 4) as usize;

                let border = x < 2 || x >= 30 || y < 2 || y >= 30;
                let left_eye = (8..=12).contains(&x) && (8..=12).contains(&y);
                let right_eye = (20..=24).contains(&x) && (8..=12).contains(&y);
                let mouth = (10..=22).contains(&x)
                    && (20..=24).contains(&y)
                    && !((12..=20).contains(&x) && (20..=22).contains(&y));

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
        // Camera and background
        app.insert_resource(Camera2D::new(800.0, 600.0));
        app.insert_resource(ClearColor(0.08, 0.08, 0.12, 1.0));

        // Render graph: clear → sprites
        let mut graph = RenderGraph::new();
        graph.add_node("clear", ClearPassNode);
        graph.add_node("sprites", SpriteBatchNode);
        graph.add_edge("clear", "sprites");
        app.insert_resource(graph);

        // GPU startup callback — create textures and sprite pipeline after wgpu init
        let mut callbacks = GpuStartupCallbacks::new();
        callbacks.add(|app: &mut App| {
            let ctx = app.world.resource::<RenderContext>().unwrap();
            let device = &ctx.device;
            let queue = &ctx.queue;
            let format = ctx.surface_format;

            // Generate and upload sprite sheet texture
            let (pixels, w, h) = generate_sprite_sheet();
            let texture = Texture::from_rgba(device, queue, &pixels, w, h, Some("Sprite Sheet"));

            // Create texture store and add the texture
            let mut store = TextureStore::new(device);
            let _tex_idx = store.add(device, texture);

            // Create sprite batch resources with texture bind group layout
            let sprite_res = SpriteBatchResources::new(device, format, &store.bind_group_layout);

            // Create atlas for the 4×1 sprite sheet
            let atlas = TextureAtlas::from_grid(0, w as f32, h as f32, 32.0, 32.0, 4, 1);

            // Queue sprites for rendering
            let mut draw_queue = SpriteDrawQueue::new();

            // Row of character frames at different sizes
            let positions = [
                (Vec2::new(-250.0, 100.0), 0, 64.0),
                (Vec2::new(-80.0, 100.0), 1, 80.0),
                (Vec2::new(100.0, 100.0), 2, 96.0),
                (Vec2::new(280.0, 100.0), 3, 112.0),
            ];

            for (pos, frame_idx, size) in &positions {
                let source_rect = atlas.get_rect(*frame_idx).cloned();
                let sprite = Sprite {
                    texture_index: 0,
                    source_rect,
                    color: Vec4::ONE,
                    custom_size: Some(Vec2::new(*size, *size)),
                    ..Default::default()
                };
                let transform = Transform2D {
                    position: *pos,
                    scale: Vec2::ONE,
                    ..Default::default()
                };
                draw_queue.draw(&sprite, &transform);
            }

            // Flipped + tinted sprite
            let flipped = Sprite {
                texture_index: 0,
                source_rect: atlas.get_rect(0).cloned(),
                color: Vec4::new(0.7, 0.7, 1.0, 1.0),
                flip_x: true,
                custom_size: Some(Vec2::new(128.0, 128.0)),
                ..Default::default()
            };
            draw_queue.draw(
                &flipped,
                &Transform2D {
                    position: Vec2::new(0.0, -120.0),
                    ..Default::default()
                },
            );

            // Rotated sprite
            let rotated = Sprite {
                texture_index: 0,
                source_rect: atlas.get_rect(2).cloned(),
                color: Vec4::new(1.0, 0.8, 0.8, 1.0),
                custom_size: Some(Vec2::new(80.0, 80.0)),
                ..Default::default()
            };
            draw_queue.draw(
                &rotated,
                &Transform2D {
                    position: Vec2::new(-220.0, -150.0),
                    rotation: 0.4,
                    ..Default::default()
                },
            );

            app.insert_resource(store);
            app.insert_resource(sprite_res);
            app.insert_resource(draw_queue);

            println!("Sprite pipeline initialized — rendering {} sprites!", 6);
        });
        app.insert_resource(callbacks);

        println!("Sprite Demo — procedurally generated sprite sheet");
        println!("Close the window to exit.");
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugin(ember_core::plugin::CorePlugin);
    app.add_plugin(WindowPlugin {
        title: "Ember Engine — Sprite Demo".to_string(),
        ..Default::default()
    });
    app.add_plugin(RenderPlugin);
    app.add_plugin(SpriteDemoPlugin);
    app.run();
}
