//! Scene Demo — Demonstrates entity parent-child transforms
//!
//! Run: `cargo run --example scene_demo -p ember_2d`

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
use ember_2d::sprite::{LocalTransform2D, Sprite, Transform2D};
use ember_2d::texture::{Texture, TextureStore};
use ember_scene::hierarchy::Parent;
use ember_scene::plugin::ScenePlugin;
use glam::{Vec2, Vec4};

pub struct SceneDemoPlugin;

impl Plugin for SceneDemoPlugin {
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

        // Spawn Parent
        let parent_entity = app
            .world
            .spawn()
            .insert(Sprite {
                texture_index: 0,
                color: Vec4::new(0.8, 0.2, 0.2, 1.0),
                custom_size: Some(Vec2::new(100.0, 100.0)),
                ..Default::default()
            })
            // Parent orbits automatically via rotation system
            .insert(Transform2D {
                position: Vec2::ZERO,
                ..Default::default()
            })
            .id();

        // Spawn Child
        app.world
            .spawn()
            .insert(Sprite {
                texture_index: 0,
                color: Vec4::new(0.2, 0.8, 0.2, 1.0),
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..Default::default()
            })
            // Local transform offset
            .insert(LocalTransform2D(Transform2D {
                position: Vec2::new(100.0, 0.0),
                ..Default::default()
            }))
            .insert(Transform2D::default()) // The global transform is computed by ScenePlugin
            .insert(Parent(parent_entity))
            .id();

        // GPU Init
        let mut callbacks = GpuStartupCallbacks::new();
        callbacks.add(|app: &mut App| {
            let ctx = app.world.resource::<RenderContext>().unwrap();
            let mut tex_store = TextureStore::new(&ctx.device);

            // White fallback
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

        // Custom system to rotate the parent
        app.add_system::<fn(Query<'static, &'static mut Transform2D>), _>(rotate_parent_system);

        // Render system
        app.add_system::<fn(
            ResMut<'static, SpriteDrawQueue>,
            Query<'static, (&'static Sprite, &'static Transform2D)>,
        ), _>(queue_sprites_system);
    }
}

// Simple rotation to demonstrate that the child orbits the parent
fn rotate_parent_system(query: Query<&mut Transform2D>) {
    for transform in query.iter() {
        // In a real approach, we'd ensure only the parent gets grabbed,
        // but since the child has its Transform2D overwritten by the Scene system
        // every frame, adding to its rotation here gets clobbered!
        // This implicitly acts on the parent (and briefly child until overwritten).
        transform.rotation += 0.02;
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
        title: "Ember Engine — Scene Hierarchy Demo".to_string(),
        ..Default::default()
    });
    app.add_plugin(RenderPlugin);
    app.add_plugin(ScenePlugin); // Propagates local transforms
    app.add_plugin(SceneDemoPlugin);
    app.run();
}
