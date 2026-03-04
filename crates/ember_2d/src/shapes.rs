use glam::{Vec2, Vec4};

use crate::sprite::SpriteVertex;

/// A colored rectangle (no texture needed).
#[derive(Debug, Clone)]
pub struct ShapeRect {
    pub position: Vec2,
    pub size: Vec2,
    pub color: Vec4,
    pub rotation: f32,
    pub z_order: f32,
}

impl ShapeRect {
    pub fn new(position: Vec2, size: Vec2, color: Vec4) -> Self {
        Self {
            position,
            size,
            color,
            rotation: 0.0,
            z_order: 0.0,
        }
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_z_order(mut self, z: f32) -> Self {
        self.z_order = z;
        self
    }
}

/// A colored circle approximated by a polygon.
#[derive(Debug, Clone)]
pub struct ShapeCircle {
    pub center: Vec2,
    pub radius: f32,
    pub color: Vec4,
    pub segments: u32,
    pub z_order: f32,
}

impl ShapeCircle {
    pub fn new(center: Vec2, radius: f32, color: Vec4) -> Self {
        Self {
            center,
            radius,
            color,
            segments: 32,
            z_order: 0.0,
        }
    }

    pub fn with_segments(mut self, segments: u32) -> Self {
        self.segments = segments;
        self
    }
}

/// A colored line segment with thickness.
#[derive(Debug, Clone)]
pub struct ShapeLine {
    pub start: Vec2,
    pub end: Vec2,
    pub thickness: f32,
    pub color: Vec4,
    pub z_order: f32,
}

impl ShapeLine {
    pub fn new(start: Vec2, end: Vec2, thickness: f32, color: Vec4) -> Self {
        Self {
            start,
            end,
            thickness,
            color,
            z_order: 0.0,
        }
    }
}

/// Collects shape draw commands into vertex/index buffers.
pub struct ShapeDrawQueue {
    pub vertices: Vec<SpriteVertex>,
    pub indices: Vec<u32>,
}

impl Default for ShapeDrawQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl ShapeDrawQueue {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    /// Queue a filled rectangle.
    pub fn draw_rect(&mut self, rect: &ShapeRect) {
        let half = rect.size * 0.5;
        let cos_r = rect.rotation.cos();
        let sin_r = rect.rotation.sin();

        let corners = [
            Vec2::new(-half.x, -half.y),
            Vec2::new(half.x, -half.y),
            Vec2::new(half.x, half.y),
            Vec2::new(-half.x, half.y),
        ];

        let base = self.vertices.len() as u32;
        let color = rect.color.to_array();

        for corner in &corners {
            let rotated = Vec2::new(
                corner.x * cos_r - corner.y * sin_r,
                corner.x * sin_r + corner.y * cos_r,
            );
            self.vertices.push(SpriteVertex {
                position: [
                    rect.position.x + rotated.x,
                    rect.position.y + rotated.y,
                    rect.z_order,
                ],
                uv: [0.0, 0.0], // No texture
                color,
            });
        }

        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    /// Queue a filled circle (fan triangulation).
    pub fn draw_circle(&mut self, circle: &ShapeCircle) {
        let center_idx = self.vertices.len() as u32;
        let color = circle.color.to_array();

        // Center vertex
        self.vertices.push(SpriteVertex {
            position: [circle.center.x, circle.center.y, circle.z_order],
            uv: [0.5, 0.5],
            color,
        });

        let segments = circle.segments.max(3);
        for i in 0..=segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = circle.center.x + circle.radius * angle.cos();
            let y = circle.center.y + circle.radius * angle.sin();
            self.vertices.push(SpriteVertex {
                position: [x, y, circle.z_order],
                uv: [0.0, 0.0],
                color,
            });
        }

        for i in 0..segments {
            self.indices.push(center_idx);
            self.indices.push(center_idx + 1 + i);
            self.indices.push(center_idx + 2 + i);
        }
    }

    /// Queue a thick line segment (drawn as a rectangle).
    pub fn draw_line(&mut self, line: &ShapeLine) {
        let dir = (line.end - line.start).normalize_or_zero();
        let perp = Vec2::new(-dir.y, dir.x) * (line.thickness * 0.5);

        let base = self.vertices.len() as u32;
        let color = line.color.to_array();

        let corners = [
            line.start + perp,
            line.start - perp,
            line.end - perp,
            line.end + perp,
        ];

        for corner in &corners {
            self.vertices.push(SpriteVertex {
                position: [corner.x, corner.y, line.z_order],
                uv: [0.0, 0.0],
                color,
            });
        }

        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_rect_produces_quad() {
        let mut queue = ShapeDrawQueue::new();
        let rect = ShapeRect::new(Vec2::ZERO, Vec2::new(100.0, 50.0), Vec4::ONE);
        queue.draw_rect(&rect);
        assert_eq!(queue.vertices.len(), 4);
        assert_eq!(queue.indices.len(), 6);
    }

    #[test]
    fn draw_circle_produces_fan() {
        let mut queue = ShapeDrawQueue::new();
        let circle = ShapeCircle::new(Vec2::ZERO, 50.0, Vec4::ONE).with_segments(8);
        queue.draw_circle(&circle);
        // center + 9 perimeter vertices = 10
        assert_eq!(queue.vertices.len(), 10);
        assert_eq!(queue.indices.len(), 24); // 8 triangles × 3
    }

    #[test]
    fn draw_line_produces_quad() {
        let mut queue = ShapeDrawQueue::new();
        let line = ShapeLine::new(Vec2::ZERO, Vec2::new(100.0, 0.0), 5.0, Vec4::ONE);
        queue.draw_line(&line);
        assert_eq!(queue.vertices.len(), 4);
        assert_eq!(queue.indices.len(), 6);
    }

    #[test]
    fn clear_empties_queue() {
        let mut queue = ShapeDrawQueue::new();
        queue.draw_rect(&ShapeRect::new(Vec2::ZERO, Vec2::ONE, Vec4::ONE));
        assert!(!queue.vertices.is_empty());
        queue.clear();
        assert!(queue.vertices.is_empty());
        assert!(queue.indices.is_empty());
    }
}

// ─── Shape Rendering Pipeline ───

use ember_core::world::World;
use ember_render::camera::{Camera2D, CameraUniform};
use ember_render::graph::RenderNode;
use wgpu::util::DeviceExt;

/// WGSL shader for shapes — vertex color only, no texture sampling.
const SHAPE_SHADER: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#;

/// GPU resources for the shape rendering pipeline.
pub struct ShapeBatchResources {
    pub pipeline: wgpu::RenderPipeline,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
}

impl ShapeBatchResources {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        // Camera uniform bind group layout
        let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Shape Camera Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shape Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shape Camera Bind Group"),
            layout: &camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // Shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shape Shader"),
            source: wgpu::ShaderSource::Wgsl(SHAPE_SHADER.into()),
        });

        // Pipeline layout — single bind group for camera
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Shape Pipeline Layout"),
            bind_group_layouts: &[&camera_layout],
            push_constant_ranges: &[],
        });

        // Same vertex layout as SpriteVertex
        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SpriteVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 20,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Shape Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[vertex_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            camera_buffer,
            camera_bind_group,
        }
    }
}

/// Render graph node that draws all queued shapes using vertex colors.
pub struct ShapeBatchNode;

impl RenderNode for ShapeBatchNode {
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
        let draw_queue = match world.resource::<ShapeDrawQueue>() {
            Some(dq) => dq,
            None => return,
        };

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
            label: Some("Shape Vertex Buffer"),
            contents: bytemuck::cast_slice(&draw_queue.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Shape Index Buffer"),
            contents: bytemuck::cast_slice(&draw_queue.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Shape Batch Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Shape Batch Pass"),
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
