use ember_core::world::World;
use ember_render::camera::{Camera2D, CameraUniform};
use ember_render::graph::RenderNode;
use wgpu::util::DeviceExt;

use crate::pipeline;
use crate::sprite::{build_sprite_quad, Sprite, SpriteVertex, Transform2D};
use crate::texture::TextureStore;

/// GPU resources for the sprite batching system.
pub struct SpriteBatchResources {
    pub pipeline: wgpu::RenderPipeline,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_bind_group: wgpu::BindGroup,
}

impl SpriteBatchResources {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let camera_bind_group_layout = pipeline::create_camera_bind_group_layout(device);

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Uniform Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group =
            pipeline::create_camera_bind_group(device, &camera_bind_group_layout, &camera_buffer);

        let render_pipeline = pipeline::create_sprite_pipeline(
            device,
            surface_format,
            &camera_bind_group_layout,
            texture_bind_group_layout,
        );

        Self {
            pipeline: render_pipeline,
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
        }
    }
}

/// Holds the sprite render data collected by the user or a system before rendering.
/// This is inserted as a resource and consumed by SpriteBatchNode each frame.
pub struct SpriteDrawQueue {
    pub entries: Vec<SpriteDrawEntry>,
}

impl Default for SpriteDrawQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl SpriteDrawQueue {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Queue a sprite for drawing.
    pub fn draw(&mut self, sprite: &Sprite, transform: &Transform2D) {
        self.entries.push(SpriteDrawEntry {
            texture_index: sprite.texture_index,
            sprite: sprite.clone(),
            transform: transform.clone(),
        });
    }

    /// Clear the draw queue (called after rendering).
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// A single sprite draw entry.
pub struct SpriteDrawEntry {
    pub texture_index: usize,
    pub sprite: Sprite,
    pub transform: Transform2D,
}

/// A render graph node that draws all queued sprites in batches grouped by texture.
pub struct SpriteBatchNode;

impl RenderNode for SpriteBatchNode {
    fn run(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        world: &World,
    ) {
        let batch_res = match world.resource::<SpriteBatchResources>() {
            Some(r) => r,
            None => return,
        };
        let texture_store = match world.resource::<TextureStore>() {
            Some(ts) => ts,
            None => return,
        };
        let draw_queue = match world.resource::<SpriteDrawQueue>() {
            Some(dq) => dq,
            None => return,
        };

        if draw_queue.entries.is_empty() {
            return;
        }

        // Update camera uniform
        let camera = world.resource::<Camera2D>().cloned().unwrap_or_default();
        let camera_uniform = CameraUniform::from_camera(&camera);
        queue.write_buffer(
            &batch_res.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );

        // Sort by texture_index, then by z_order
        let mut sorted: Vec<&SpriteDrawEntry> = draw_queue.entries.iter().collect();
        sorted.sort_by(|a, b| {
            a.texture_index.cmp(&b.texture_index).then_with(|| {
                a.transform
                    .z_order
                    .partial_cmp(&b.transform.z_order)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        });

        // Build vertex/index buffers grouped by texture
        let mut all_vertices: Vec<SpriteVertex> = Vec::new();
        let mut all_indices: Vec<u32> = Vec::new();
        let mut batches: Vec<(usize, u32, u32)> = Vec::new();
        let mut current_texture = usize::MAX;
        let mut batch_start = 0u32;

        for entry in &sorted {
            if entry.texture_index != current_texture {
                if current_texture != usize::MAX {
                    let count = all_indices.len() as u32 - batch_start;
                    batches.push((current_texture, batch_start, count));
                }
                current_texture = entry.texture_index;
                batch_start = all_indices.len() as u32;
            }

            let tex_width = texture_store
                .textures
                .get(entry.texture_index)
                .map(|t| t.width as f32)
                .unwrap_or(1.0);
            let tex_height = texture_store
                .textures
                .get(entry.texture_index)
                .map(|t| t.height as f32)
                .unwrap_or(1.0);

            let base_vertex = all_vertices.len() as u32;
            let (vertices, indices, _) =
                build_sprite_quad(&entry.transform, &entry.sprite, tex_width, tex_height);
            all_vertices.extend_from_slice(&vertices);
            for idx in &indices {
                all_indices.push(base_vertex + idx);
            }
        }

        // Push final batch
        if current_texture != usize::MAX {
            let count = all_indices.len() as u32 - batch_start;
            batches.push((current_texture, batch_start, count));
        }

        if all_vertices.is_empty() {
            return;
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite Vertex Buffer"),
            contents: bytemuck::cast_slice(&all_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite Index Buffer"),
            contents: bytemuck::cast_slice(&all_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Sprite Batch Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Sprite Batch Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Don't clear — ClearPass already did
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&batch_res.pipeline);
            render_pass.set_bind_group(0, &batch_res.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            for (texture_idx, start, count) in &batches {
                if let Some(bind_group) = texture_store.bind_groups.get(*texture_idx) {
                    render_pass.set_bind_group(1, bind_group, &[]);
                    render_pass.draw_indexed(*start..(*start + *count), 0, 0..1);
                }
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}
