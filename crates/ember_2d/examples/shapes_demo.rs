//! Shapes Demo — Demonstrates basic shape rendering (rectangles, circles, lines)
//!
//! Run: `cargo run --example shapes_demo -p ember_2d`

use ember_core::app::App;
use ember_core::plugin::Plugin;
use ember_render::camera::Camera2D;
use ember_render::clear_pass::{ClearColor, ClearPassNode};
use ember_render::context::{GpuStartupCallbacks, RenderContext, RenderPlugin};
use ember_render::graph::RenderGraph;
use ember_render::window::WindowPlugin;

use ember_2d::shapes::{
    ShapeBatchNode, ShapeBatchResources, ShapeCircle, ShapeDrawQueue, ShapeLine, ShapeRect,
};

use glam::{Vec2, Vec4};

/// Plugin that sets up the shapes demo scene.
struct ShapesDemoPlugin;

impl Plugin for ShapesDemoPlugin {
    fn build(&self, app: &mut App) {
        // Camera
        app.insert_resource(Camera2D::new(800.0, 600.0));
        app.insert_resource(ClearColor(0.1, 0.1, 0.15, 1.0));

        // Render graph: clear → shapes
        let mut graph = RenderGraph::new();
        graph.add_node("clear", ClearPassNode);
        graph.add_node("shapes", ShapeBatchNode);
        graph.add_edge("clear", "shapes");
        app.insert_resource(graph);

        // Queue shapes to draw
        let mut shapes = ShapeDrawQueue::new();

        // Colorful rectangles
        shapes.draw_rect(&ShapeRect::new(
            Vec2::new(-200.0, 150.0),
            Vec2::new(120.0, 80.0),
            Vec4::new(0.9, 0.2, 0.3, 1.0),
        ));
        shapes.draw_rect(
            &ShapeRect::new(
                Vec2::new(0.0, 150.0),
                Vec2::new(100.0, 100.0),
                Vec4::new(0.2, 0.7, 0.3, 1.0),
            )
            .with_rotation(0.3),
        );
        shapes.draw_rect(&ShapeRect::new(
            Vec2::new(200.0, 150.0),
            Vec2::new(80.0, 120.0),
            Vec4::new(0.2, 0.4, 0.9, 1.0),
        ));

        // Circles
        shapes.draw_circle(&ShapeCircle::new(
            Vec2::new(-200.0, -50.0),
            60.0,
            Vec4::new(1.0, 0.8, 0.1, 1.0),
        ));
        shapes.draw_circle(
            &ShapeCircle::new(Vec2::new(0.0, -50.0), 45.0, Vec4::new(0.8, 0.3, 0.9, 1.0))
                .with_segments(6), // Hexagon
        );
        shapes.draw_circle(
            &ShapeCircle::new(Vec2::new(200.0, -50.0), 50.0, Vec4::new(0.1, 0.8, 0.8, 1.0))
                .with_segments(3), // Triangle
        );

        // Lines
        shapes.draw_line(&ShapeLine::new(
            Vec2::new(-300.0, -200.0),
            Vec2::new(300.0, -200.0),
            4.0,
            Vec4::new(1.0, 1.0, 1.0, 0.8),
        ));
        shapes.draw_line(&ShapeLine::new(
            Vec2::new(-50.0, -150.0),
            Vec2::new(50.0, -250.0),
            3.0,
            Vec4::new(1.0, 0.5, 0.2, 1.0),
        ));
        shapes.draw_line(&ShapeLine::new(
            Vec2::new(50.0, -150.0),
            Vec2::new(-50.0, -250.0),
            3.0,
            Vec4::new(1.0, 0.5, 0.2, 1.0),
        ));

        app.insert_resource(shapes);

        // GPU startup callback — create shape pipeline after wgpu init
        let mut callbacks = GpuStartupCallbacks::new();
        callbacks.add(|app: &mut App| {
            let ctx = app.world.resource::<RenderContext>().unwrap();
            let format = ctx.surface_format;
            let res = ShapeBatchResources::new(&ctx.device, format);
            app.insert_resource(res);
            println!("Shape pipeline initialized — rendering shapes!");
        });
        app.insert_resource(callbacks);

        println!("Shapes Demo — rectangles, circles, and lines");
        println!("Close the window to exit.");
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugin(ember_core::plugin::CorePlugin);
    app.add_plugin(WindowPlugin {
        title: "Ember Engine — Shapes Demo".to_string(),
        ..Default::default()
    });
    app.add_plugin(RenderPlugin);
    app.add_plugin(ShapesDemoPlugin);
    app.run();
}
