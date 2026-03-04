//! Input Demo — Demonstrates keyboard and mouse input moving a sprite on screen
//!
//! Run: `cargo run --example input_demo -p ember_2d`

use ember_core::app::App;
use ember_core::plugin::Plugin;
use ember_core::query::Query;
use ember_core::system::{Res, ResMut};
use ember_render::camera::Camera2D;
use ember_render::clear_pass::{ClearColor, ClearPassNode};
use ember_render::context::{GpuStartupCallbacks, RenderContext, RenderPlugin};
use ember_render::graph::RenderGraph;
use ember_render::window::WindowPlugin;

use ember_2d::batch::{SpriteBatchNode, SpriteBatchResources, SpriteDrawQueue};
use ember_2d::sprite::{Sprite, Transform2D};
use ember_2d::texture::{Texture, TextureStore};

use ember_input::keyboard::{KeyCode, KeyboardState};
use ember_input::mouse::{MouseButton, MouseState};
use ember_input::plugin::InputPlugin;

use glam::{Vec2, Vec4};

/// Component attached to the player entity.
#[derive(Debug, Clone, Copy)]
pub struct Player {
    pub speed: f32,
}

/// Generate a simple 64x64 colored square texture
fn generate_player_texture() -> (Vec<u8>, u32, u32) {
    let width: u32 = 64;
    let height: u32 = 64;
    let mut pixels = vec![0u8; (width * height * 4) as usize];

    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            let border = x < 4 || x >= width - 4 || y < 4 || y >= height - 4;

            if border {
                pixels[idx] = 255;
                pixels[idx + 1] = 255;
                pixels[idx + 2] = 255;
                pixels[idx + 3] = 255;
            } else {
                pixels[idx] = 80;
                pixels[idx + 1] = 160;
                pixels[idx + 2] = 220;
                pixels[idx + 3] = 255;
            }
        }
    }

    (pixels, width, height)
}

/// System to handle player input and move the character.
fn player_movement_system(
    query: Query<(&'static Player, &'static mut Transform2D)>,
    keyboard: Res<KeyboardState>,
    mouse: Res<MouseState>,
) {
    let mut move_dir = Vec2::ZERO;

    if keyboard.is_pressed(KeyCode::KeyW) || keyboard.is_pressed(KeyCode::ArrowUp) {
        move_dir.y += 1.0;
    }
    if keyboard.is_pressed(KeyCode::KeyS) || keyboard.is_pressed(KeyCode::ArrowDown) {
        move_dir.y -= 1.0;
    }
    if keyboard.is_pressed(KeyCode::KeyA) || keyboard.is_pressed(KeyCode::ArrowLeft) {
        move_dir.x -= 1.0;
    }
    if keyboard.is_pressed(KeyCode::KeyD) || keyboard.is_pressed(KeyCode::ArrowRight) {
        move_dir.x += 1.0;
    }

    if move_dir.length_squared() > 0.0 {
        move_dir = move_dir.normalize();
    }

    for (player, transform) in query.iter() {
        // Move character (assuming 60fps fixed delta for demo, 1/60th of speed)
        transform.position += move_dir * player.speed * (1.0 / 60.0);
    }

    // Print mouse clicks to demonstrating reading mouse state
    if mouse.is_just_pressed(MouseButton::Left) {
        println!("Left clicked at position: {:?}", mouse.position);
    }
    if mouse.is_just_pressed(MouseButton::Right) {
        println!("Right clicked at position: {:?}", mouse.position);
    }
}

/// System to rebuild the sprite draw queue every frame.
fn queue_sprites_system(
    mut draw_queue: ResMut<SpriteDrawQueue>,
    query: Query<(&'static Sprite, &'static Transform2D)>,
) {
    draw_queue.clear();
    for (sprite, transform) in query.iter() {
        draw_queue.draw(sprite, transform);
    }
}

/// Plugin that sets up the input demo scene.
struct InputDemoPlugin;

impl Plugin for InputDemoPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Camera2D::new(800.0, 600.0));
        app.insert_resource(ClearColor(0.1, 0.1, 0.1, 1.0));

        let mut graph = RenderGraph::new();
        graph.add_node("clear", ClearPassNode);
        graph.add_node("sprites", SpriteBatchNode);
        graph.add_edge("clear", "sprites");
        app.insert_resource(graph);

        // Pre-create the DrawQueue
        app.insert_resource(SpriteDrawQueue::new());

        // Add updates systems
        app.add_system::<fn(
            Query<'static, (&'static Player, &'static mut Transform2D)>,
            Res<'static, KeyboardState>,
            Res<'static, MouseState>,
        ), _>(player_movement_system);
        app.add_system::<fn(
            ResMut<'static, SpriteDrawQueue>,
            Query<'static, (&'static Sprite, &'static Transform2D)>,
        ), _>(queue_sprites_system);

        let mut callbacks = GpuStartupCallbacks::new();
        callbacks.add(|app: &mut App| {
            let ctx = app.world.resource::<RenderContext>().unwrap();
            let device = &ctx.device;
            let queue = &ctx.queue;
            let format = ctx.surface_format;

            let (pixels, w, h) = generate_player_texture();
            let texture = Texture::from_rgba(device, queue, &pixels, w, h, Some("Player Texture"));

            let mut store = TextureStore::new(device);
            let tex_idx = store.add(device, texture);

            let sprite_res = SpriteBatchResources::new(device, format, &store.bind_group_layout);

            app.insert_resource(store);
            app.insert_resource(sprite_res);

            // Spawn player entity
            let player = Player { speed: 400.0 };
            let transform = Transform2D {
                position: Vec2::ZERO,
                scale: Vec2::ONE,
                rotation: 0.0,
                z_order: 0.0,
            };
            let sprite = Sprite {
                texture_index: tex_idx,
                source_rect: None,
                color: Vec4::ONE,
                flip_x: false,
                flip_y: false,
                custom_size: Some(Vec2::new(w as f32, h as f32)),
            };

            app.world
                .spawn()
                .insert(player)
                .insert(transform)
                .insert(sprite)
                .id();

            println!("Input Demo initialized!");
            println!("Controls:");
            println!("  W/A/S/D or Arrows to move");
            println!("  Left/Right Click for mouse coordinates in console");
        });
        app.insert_resource(callbacks);
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugin(ember_core::plugin::CorePlugin);
    app.add_plugin(WindowPlugin {
        title: "Ember Engine — Input Demo".to_string(),
        ..Default::default()
    });
    app.add_plugin(RenderPlugin);
    app.add_plugin(InputPlugin); // Crucial for input events!
    app.add_plugin(InputDemoPlugin);
    app.run();
}
