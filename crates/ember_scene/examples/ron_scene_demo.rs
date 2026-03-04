//! RON Scene Demo
//!
//! Run: `cargo run --example ron_scene_demo -p ember_2d`

use ember_core::app::App;
use ember_core::plugin::Plugin;
use ember_core::query::Query;
use ember_core::system::ResMut;
use ember_render::camera::Camera2D;
use ember_render::clear_pass::{ClearColor, ClearPassNode};
use ember_render::context::{GpuStartupCallbacks, RenderContext, RenderPlugin};
use ember_render::graph::RenderGraph;
use ember_render::window::WindowPlugin;

use ember_2d::batch::{SpriteBatchNode, SpriteDrawQueue};
use ember_2d::sprite::{Sprite, Transform2D};
use ember_2d::texture::{Texture, TextureStore};
use ember_assets::server::{AssetChannel, AssetServer};
use ember_assets::storage::Assets;
use ember_scene::loader::SceneLoader;
use ember_scene::plugin::ScenePlugin;
use ember_scene::scene::Scene;

// We create a temporary RON file and then load it via AssetServer.
pub struct RonSceneDemoPlugin;

impl Plugin for RonSceneDemoPlugin {
    fn build(&self, app: &mut App) {
        // App resources
        app.insert_resource(Camera2D::new(800.0, 600.0));
        app.insert_resource(ClearColor(0.1, 0.1, 0.1, 1.0));

        let mut graph = RenderGraph::new();
        graph.add_node("clear", ClearPassNode);
        graph.add_node("sprites", SpriteBatchNode);
        graph.add_edge("clear", "sprites");
        app.insert_resource(graph);

        app.insert_resource(SpriteDrawQueue::new());
        app.insert_resource(Assets::<Scene>::new());

        let asset_server = AssetServer::new();
        let channel = AssetChannel::<Scene>::default();

        // Create sample RON file
        let ron_path = std::env::temp_dir().join("ember_test_scene.ron");
        let ron_data = r#"
        (
            roots: [
                (
                    name: Some("Root Entity"),
                    local_transform: None,
                    transform: Some((
                        position: (100.0, 100.0),
                        rotation: 0.0,
                        scale: (1.0, 1.0),
                        z_order: 0.0,
                    )),
                    sprite: Some((
                        texture_index: 0,
                        color: (0.1, 0.5, 0.9, 1.0),
                        custom_size: Some((80.0, 80.0)),
                        texture_rect: None,
                        texture_bound: false,
                    )),
                    children: [
                        (
                            name: Some("Child Entity"),
                            local_transform: Some(((
                                position: (50.0, 50.0),
                                rotation: 0.0,
                                scale: (0.5, 0.5),
                                z_order: 1.0,
                            ))),
                            transform: None,
                            sprite: Some((
                                texture_index: 0,
                                color: (0.9, 0.1, 0.1, 1.0),
                                custom_size: Some((80.0, 80.0)),
                                texture_rect: None,
                                texture_bound: false,
                            )),
                            children: [],
                        )
                    ]
                )
            ]
        )
        "#;
        std::fs::write(&ron_path, ron_data).unwrap();

        // Load scene
        let handle = asset_server.load(ron_path.to_str().unwrap(), SceneLoader, &channel);

        app.insert_resource(asset_server);
        app.insert_resource(channel);

        // Block until loaded
        loop {
            match app
                .world
                .resource::<AssetChannel<Scene>>()
                .unwrap()
                .receiver
                .try_recv()
            {
                Ok((_id, Ok(asset))) => {
                    app.world
                        .resource_mut::<Assets<Scene>>()
                        .unwrap()
                        .insert(&handle, asset);
                    break;
                }
                Ok((_id, Err(e))) => {
                    panic!("Failed to load scene: {}", e);
                }
                Err(_) => {
                    // Not ready yet, keep waiting
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }

        let scene = app
            .world
            .resource::<Assets<Scene>>()
            .unwrap()
            .get(&handle)
            .unwrap()
            .clone();
        scene.spawn(&mut app.world);

        // GPU Init
        let mut callbacks = GpuStartupCallbacks::new();
        callbacks.add(|app: &mut App| {
            let ctx = app.world.resource::<RenderContext>().unwrap();
            let mut tex_store = TextureStore::new(&ctx.device);
            // Fallback texture
            let fallback_rgba = vec![255, 255, 255, 255];
            let tx = Texture::from_rgba(
                &ctx.device,
                &ctx.queue,
                &fallback_rgba,
                1,
                1,
                Some("Fallback"),
            );
            tex_store.add(&ctx.device, tx);

            app.insert_resource(ember_2d::batch::SpriteBatchResources::new(
                &ctx.device,
                ctx.surface_format,
                &tex_store.bind_group_layout,
            ));
            app.insert_resource(tex_store);
        });
        app.insert_resource(callbacks);

        // Render system
        app.add_system::<fn(
            ResMut<'static, SpriteDrawQueue>,
            Query<'static, (&'static Sprite, &'static Transform2D)>,
        ), _>(queue_sprites_system);
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
        title: "Ember Engine — RON Scene Demo".to_string(),
        ..Default::default()
    });
    app.add_plugin(RenderPlugin);
    app.add_plugin(ScenePlugin);
    app.add_plugin(RonSceneDemoPlugin);
    app.run();
}
