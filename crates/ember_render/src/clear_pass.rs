use ember_core::world::World;

use crate::graph::RenderNode;

/// Resource that controls the background clear color (RGBA, 0.0–1.0).
pub struct ClearColor(pub f64, pub f64, pub f64, pub f64);

impl Default for ClearColor {
    fn default() -> Self {
        Self(0.1, 0.1, 0.15, 1.0) // Dark blue-grey
    }
}

/// Render node that clears the swap chain texture to the configured `ClearColor`.
pub struct ClearPassNode;

impl RenderNode for ClearPassNode {
    fn run(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        world: &World,
    ) {
        let color = world
            .resource::<ClearColor>()
            .map(|c| wgpu::Color {
                r: c.0,
                g: c.1,
                b: c.2,
                a: c.3,
            })
            .unwrap_or(wgpu::Color {
                r: 0.1,
                g: 0.1,
                b: 0.15,
                a: 1.0,
            });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Clear Pass Encoder"),
        });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            // Render pass drops here, finishing the pass
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}
