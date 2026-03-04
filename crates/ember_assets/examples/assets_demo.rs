//! Assets Demo — Demonstrates async asset loading with ImageLoader
//!
//! Run: `cargo run --example assets_demo -p ember_2d`

use ember_assets::handle::Handle;
use ember_assets::image_loader::{Image, ImageLoader};
use ember_assets::server::{update_asset_state, AssetChannel, AssetServer};
use ember_assets::storage::Assets;
use ember_core::app::App;
use ember_core::plugin::Plugin;
use ember_core::query::Query;
use ember_core::system::{Res, ResMut};
use ember_render::camera::Camera2D;
use ember_render::clear_pass::{ClearColor, ClearPassNode};
use ember_render::context::{GpuStartupCallbacks, RenderContext, RenderPlugin};
use ember_render::graph::RenderGraph;
use ember_render::window::WindowPlugin;

use ember_2d::batch::{SpriteBatchNode, SpriteDrawQueue};
use ember_2d::sprite::{Sprite, Transform2D};
use ember_2d::texture::{Texture, TextureStore};
use glam::Vec2;

/// A marker component for our dynamically loaded sprite.
#[derive(Debug, Clone)]
struct DynamicSprite {
    image_handle: Handle<Image>,
    texture_bound: bool,
}

pub struct AssetsDemoPlugin;

impl Plugin for AssetsDemoPlugin {
    fn build(&self, app: &mut App) {
        // Core resources
        app.insert_resource(Camera2D::new(800.0, 600.0));
        app.insert_resource(ClearColor(0.2, 0.2, 0.25, 1.0));

        let mut graph = RenderGraph::new();
        graph.add_node("clear", ClearPassNode);
        graph.add_node("sprites", SpriteBatchNode);
        graph.add_edge("clear", "sprites");
        app.insert_resource(graph);

        app.insert_resource(SpriteDrawQueue::new());

        // Asset Pipeline
        let asset_server = AssetServer::new();
        let image_channel = AssetChannel::<Image>::default();

        // Let's create a temporary 32x32 image on disk just for the demo to load dynamically
        let cache_dir = std::env::temp_dir().join("ember_demo_assets");
        std::fs::create_dir_all(&cache_dir).unwrap();
        let test_img_path = cache_dir.join("test_sprite.png");
        {
            let mut img = image::ImageBuffer::new(64, 64);
            for (x, y, pixel) in img.enumerate_pixels_mut() {
                let r = (x as f32 / 64.0 * 255.0) as u8;
                let g = (y as f32 / 64.0 * 255.0) as u8;
                *pixel = image::Rgba([r, g, 128, 255]);
            }
            img.save(&test_img_path).unwrap();
        }

        // Load it!
        let handle =
            asset_server.load(test_img_path.to_str().unwrap(), ImageLoader, &image_channel);

        app.insert_resource(Assets::<Image>::new());
        app.insert_resource(asset_server);
        app.insert_resource(image_channel);
        app.add_event::<ember_assets::server::AssetEvent<Image>>();

        // Spawn entity
        app.world
            .spawn()
            .insert(DynamicSprite {
                image_handle: handle,
                texture_bound: false,
            })
            // Temporary sprite to be replaced with real bounds once loaded
            .insert(Sprite {
                texture_index: 0,
                ..Default::default()
            })
            .insert(Transform2D {
                position: Vec2::ZERO,
                ..Default::default()
            })
            .id();

        // GPU Init
        let mut callbacks = GpuStartupCallbacks::new();
        callbacks.add(|app: &mut App| {
            let ctx = app.world.resource::<RenderContext>().unwrap();
            let mut tex_store = TextureStore::new(&ctx.device);

            // Add a fallback texture at ID 0
            let fallback_rgba = vec![255, 0, 255, 255]; // Magenta fallback
            let tx = Texture::from_rgba(
                &ctx.device,
                &ctx.queue,
                &fallback_rgba,
                1,
                1,
                Some("Fallback"),
            );
            tex_store.add(&ctx.device, tx);

            let sprite_res = ember_2d::batch::SpriteBatchResources::new(
                &ctx.device,
                ctx.surface_format,
                &tex_store.bind_group_layout,
            );
            app.insert_resource(sprite_res);
            app.insert_resource(tex_store);
        });
        app.insert_resource(callbacks);

        // Systems
        // First empty the channel
        app.add_system::<fn(
            ResMut<'static, Assets<Image>>,
            ResMut<'static, AssetServer>,
            ResMut<'static, AssetChannel<Image>>,
            ResMut<'static, ember_core::event::Events<ember_assets::server::AssetEvent<Image>>>,
        ), _>(update_asset_state::<Image>);
        app.add_system::<fn(
            Query<'static, (&'static mut DynamicSprite, &'static mut Sprite)>,
            Res<'static, Assets<Image>>,
            ResMut<'static, TextureStore>,
            Res<'static, RenderContext>,
        ), _>(bind_loaded_texture_system);
        app.add_system::<fn(
            ResMut<'static, SpriteDrawQueue>,
            Query<'static, (&'static Sprite, &'static Transform2D)>,
        ), _>(queue_sprites_system);
    }
}

fn bind_loaded_texture_system(
    query: Query<(&mut DynamicSprite, &mut Sprite)>,
    images: Res<Assets<Image>>,
    mut tex_store: ResMut<TextureStore>,
    ctx: Res<RenderContext>,
) {
    for (dyn_sprite, sprite) in query.iter() {
        if !dyn_sprite.texture_bound {
            if let Some(image) = images.get(&dyn_sprite.image_handle) {
                // Asset is loaded! Let's convert it to a GPU Texture.
                println!("Image loaded dynamically! Uploading to GPU...");
                let tex = Texture::from_image(&ctx.device, &ctx.queue, image, Some("DynTexture"));
                let texture_id = tex_store.add(&ctx.device, tex) as u32;

                sprite.texture_index = texture_id as usize;
                sprite.custom_size = Some(Vec2::new(200.0, 200.0));
                dyn_sprite.texture_bound = true;
            }
        }
    }
}

fn queue_sprites_system(
    mut draw_queue: ResMut<SpriteDrawQueue>,
    query: Query<(&Sprite, &Transform2D)>,
) {
    draw_queue.clear();
    for (sprite, transform) in query.iter() {
        draw_queue.draw(sprite, transform);
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugin(ember_core::plugin::CorePlugin);
    app.add_plugin(WindowPlugin {
        title: "Ember Engine — Async Assets Demo".to_string(),
        ..Default::default()
    });
    app.add_plugin(RenderPlugin);
    app.add_plugin(AssetsDemoPlugin);
    app.run();
}
