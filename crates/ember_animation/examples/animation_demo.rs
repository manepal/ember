//! Animation Demo — Visual animation with shapes moving, bouncing, and rotating
//!
//! Demonstrates the tweening and animation systems visually in an Ember window.
//!
//! Run: `cargo run --example animation_demo -p ember_2d`

use std::time::Instant;

use ember_core::app::App;
use ember_core::plugin::Plugin;
use ember_core::world::World;
use ember_render::camera::Camera2D;
use ember_render::clear_pass::{ClearColor, ClearPassNode};
use ember_render::context::{GpuStartupCallbacks, RenderContext, RenderPlugin};
use ember_render::graph::RenderGraph;
use ember_render::window::WindowPlugin;

use ember_2d::shapes::{ShapeBatchResources, ShapeCircle, ShapeDrawQueue, ShapeLine, ShapeRect};

use ember_render::camera::CameraUniform;
use ember_render::graph::RenderNode;
use wgpu::util::DeviceExt;

use glam::{Vec2, Vec4};

/// Resource that holds the animation start time.
pub struct AnimStartTime(pub Instant);

/// A render node that draws animated shapes each frame.
/// Computes positions from elapsed time for smooth, per-frame animation.
pub struct AnimatedShapesNode;

impl RenderNode for AnimatedShapesNode {
    fn run(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        world: &World,
    ) {
        let res = match world.resource::<ShapeBatchResources>() {
            Some(r) => r,
            None => return,
        };
        let start = match world.resource::<AnimStartTime>() {
            Some(s) => s,
            None => return,
        };

        let t = start.0.elapsed().as_secs_f32();

        // Build animated shapes
        let mut draw_queue = ShapeDrawQueue::new();

        // ── Bouncing circle ──
        let bounce_y = 80.0 * (t * 3.0).sin().abs(); // Bounce up and down
        draw_queue.draw_circle(&ShapeCircle::new(
            Vec2::new(-250.0, -100.0 + bounce_y),
            35.0,
            Vec4::new(1.0, 0.4, 0.3, 1.0), // Red-orange
        ));

        // ── Orbiting circles ──
        for i in 0..6 {
            let angle = t * 1.5 + (i as f32 * std::f32::consts::TAU / 6.0);
            let orbit_radius = 100.0;
            let x = orbit_radius * angle.cos();
            let y = orbit_radius * angle.sin();
            let hue = i as f32 / 6.0;
            let color = hsv_to_rgb(hue, 0.85, 1.0);
            draw_queue.draw_circle(
                &ShapeCircle::new(
                    Vec2::new(x, y),
                    18.0 + 5.0 * (t * 2.0 + i as f32).sin(),
                    color,
                )
                .with_segments(5 + i), // Different polygon shapes
            );
        }

        // ── Spinning rectangle ──
        draw_queue.draw_rect(
            &ShapeRect::new(
                Vec2::new(250.0, 100.0),
                Vec2::new(80.0, 80.0),
                Vec4::new(0.3, 0.8, 0.4, 1.0),
            )
            .with_rotation(t * 2.0),
        );

        // ── Pulsing rectangle ──
        let pulse = 1.0 + 0.3 * (t * 4.0).sin();
        let pulse_color_r = 0.5 + 0.5 * (t * 2.0).sin();
        let pulse_color_b = 0.5 + 0.5 * (t * 2.0).cos();
        draw_queue.draw_rect(&ShapeRect::new(
            Vec2::new(250.0, -100.0),
            Vec2::new(60.0 * pulse, 60.0 * pulse),
            Vec4::new(pulse_color_r, 0.3, pulse_color_b, 1.0),
        ));

        // ── Oscillating lines (wave) ──
        let segments = 20;
        for i in 0..segments {
            let x0 = -300.0 + (i as f32 / segments as f32) * 600.0;
            let x1 = -300.0 + ((i + 1) as f32 / segments as f32) * 600.0;
            let y0 = -220.0 + 30.0 * (t * 3.0 + x0 * 0.02).sin();
            let y1 = -220.0 + 30.0 * (t * 3.0 + x1 * 0.02).sin();
            let wave_hue = (i as f32 / segments as f32 + t * 0.2) % 1.0;
            draw_queue.draw_line(&ShapeLine::new(
                Vec2::new(x0, y0),
                Vec2::new(x1, y1),
                3.0,
                hsv_to_rgb(wave_hue, 0.7, 1.0),
            ));
        }

        // ── Pendulum triangle ──
        let swing = 0.8 * (t * 2.5).sin();
        draw_queue.draw_circle(
            &ShapeCircle::new(
                Vec2::new(-250.0, 180.0),
                30.0,
                Vec4::new(0.9, 0.7, 0.1, 1.0),
            )
            .with_segments(3), // Triangle
        );
        // Pendulum arm
        let pend_x = -250.0 + 80.0 * swing.sin();
        let pend_y = 180.0 - 80.0 * swing.cos();
        draw_queue.draw_line(&ShapeLine::new(
            Vec2::new(-250.0, 180.0),
            Vec2::new(pend_x, pend_y),
            2.0,
            Vec4::new(0.7, 0.7, 0.7, 0.8),
        ));
        draw_queue.draw_circle(&ShapeCircle::new(
            Vec2::new(pend_x, pend_y),
            12.0,
            Vec4::new(0.2, 0.5, 1.0, 1.0),
        ));

        // ── Title text hint (static row of dots spelling "EMBER") ──
        let title_y = 260.0;
        for i in 0..5 {
            let x = -40.0 + i as f32 * 20.0;
            let blink = ((t * 5.0 + i as f32 * 0.5).sin() + 1.0) * 0.5;
            draw_queue.draw_circle(
                &ShapeCircle::new(Vec2::new(x, title_y), 4.0, Vec4::new(1.0, 1.0, 1.0, blink))
                    .with_segments(8),
            );
        }

        if draw_queue.vertices.is_empty() {
            return;
        }

        // Update camera uniform
        let camera = world.resource::<Camera2D>().cloned().unwrap_or_default();
        let camera_uniform = CameraUniform::from_camera(&camera);
        queue.write_buffer(
            &res.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Anim Vertex Buffer"),
            contents: bytemuck::cast_slice(&draw_queue.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Anim Index Buffer"),
            contents: bytemuck::cast_slice(&draw_queue.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Anim Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Anim Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&res.pipeline);
            render_pass.set_bind_group(0, &res.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..draw_queue.indices.len() as u32, 0, 0..1);
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}

/// Convert HSV to RGBA Vec4 (for rainbow color cycling).
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Vec4 {
    let h = (h % 1.0) * 6.0;
    let f = h.fract();
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));

    let (r, g, b) = match h as u32 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };

    Vec4::new(r, g, b, 1.0)
}

struct AnimDemoPlugin;

impl Plugin for AnimDemoPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Camera2D::new(800.0, 600.0));
        app.insert_resource(ClearColor(0.06, 0.06, 0.1, 1.0));
        app.insert_resource(AnimStartTime(Instant::now()));

        let mut graph = RenderGraph::new();
        graph.add_node("clear", ClearPassNode);
        graph.add_node("animated_shapes", AnimatedShapesNode);
        graph.add_edge("clear", "animated_shapes");
        app.insert_resource(graph);

        let mut callbacks = GpuStartupCallbacks::new();
        callbacks.add(|app: &mut App| {
            let ctx = app.world.resource::<RenderContext>().unwrap();
            let res = ShapeBatchResources::new(&ctx.device, ctx.surface_format);
            app.insert_resource(res);
            println!("Animation demo pipeline initialized!");
        });
        app.insert_resource(callbacks);

        println!("Animation Demo — bouncing, orbiting, pulsing, waving shapes");
        println!("Close the window to exit.");
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugin(ember_core::plugin::CorePlugin);
    app.add_plugin(WindowPlugin {
        title: "Ember Engine — Animation Demo".to_string(),
        ..Default::default()
    });
    app.add_plugin(RenderPlugin);
    app.add_plugin(AnimDemoPlugin);
    app.run();
}
