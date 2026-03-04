use glam::{Vec2, Vec4};

/// A 2D transform component for positioning, rotating, and scaling entities.
#[derive(Debug, Clone)]
pub struct Transform2D {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
    /// Z-order for draw sorting. Higher z = drawn later (on top).
    pub z_order: f32,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
            z_order: 0.0,
        }
    }
}

/// A rectangular region within a texture (in pixel coordinates).
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Convert pixel rect to UV coordinates given the texture dimensions.
    pub fn to_uv(&self, tex_width: f32, tex_height: f32) -> [f32; 4] {
        [
            self.x / tex_width,
            self.y / tex_height,
            (self.x + self.width) / tex_width,
            (self.y + self.height) / tex_height,
        ]
    }
}

/// A 2D sprite component.
#[derive(Debug, Clone)]
pub struct Sprite {
    /// Index into the TextureStore.
    pub texture_index: usize,
    /// Source rectangle within the texture (pixel coords). None = full texture.
    pub source_rect: Option<Rect>,
    /// Tint color (RGBA, 0.0–1.0).
    pub color: Vec4,
    /// Flip horizontally.
    pub flip_x: bool,
    /// Flip vertically.
    pub flip_y: bool,
    /// Custom size override. None = use source_rect or texture size.
    pub custom_size: Option<Vec2>,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            texture_index: 0,
            source_rect: None,
            color: Vec4::ONE, // White (no tint)
            flip_x: false,
            flip_y: false,
            custom_size: None,
        }
    }
}

/// Per-instance GPU data for batched sprite rendering.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl SpriteVertex {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SpriteVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // uv
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // color
                wgpu::VertexAttribute {
                    offset: 20,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// Generate 4 vertices and 6 indices for a single sprite quad.
pub fn build_sprite_quad(
    transform: &Transform2D,
    sprite: &Sprite,
    tex_width: f32,
    tex_height: f32,
) -> ([SpriteVertex; 4], [u32; 6], u32) {
    let (uv_min_x, uv_min_y, uv_max_x, uv_max_y) = if let Some(rect) = &sprite.source_rect {
        let uv = rect.to_uv(tex_width, tex_height);
        (uv[0], uv[1], uv[2], uv[3])
    } else {
        (0.0, 0.0, 1.0, 1.0)
    };

    // Apply flip
    let (u0, u1) = if sprite.flip_x {
        (uv_max_x, uv_min_x)
    } else {
        (uv_min_x, uv_max_x)
    };
    let (v0, v1) = if sprite.flip_y {
        (uv_max_y, uv_min_y)
    } else {
        (uv_min_y, uv_max_y)
    };

    // Determine sprite size
    let size = sprite.custom_size.unwrap_or_else(|| {
        if let Some(rect) = &sprite.source_rect {
            Vec2::new(rect.width, rect.height)
        } else {
            Vec2::new(tex_width, tex_height)
        }
    });

    let half = size * transform.scale * 0.5;
    let cos_r = transform.rotation.cos();
    let sin_r = transform.rotation.sin();

    let corners = [
        Vec2::new(-half.x, -half.y),
        Vec2::new(half.x, -half.y),
        Vec2::new(half.x, half.y),
        Vec2::new(-half.x, half.y),
    ];

    let color = sprite.color.to_array();
    let z = transform.z_order;
    let pos = transform.position;

    let mut vertices = [SpriteVertex {
        position: [0.0; 3],
        uv: [0.0; 2],
        color: [0.0; 4],
    }; 4];

    let uvs = [[u0, v1], [u1, v1], [u1, v0], [u0, v0]];

    for (i, corner) in corners.iter().enumerate() {
        let rotated = Vec2::new(
            corner.x * cos_r - corner.y * sin_r,
            corner.x * sin_r + corner.y * cos_r,
        );
        vertices[i] = SpriteVertex {
            position: [pos.x + rotated.x, pos.y + rotated.y, z],
            uv: uvs[i],
            color,
        };
    }

    let indices = [0, 1, 2, 0, 2, 3];
    (vertices, indices, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_to_uv_full_texture() {
        let rect = Rect::new(0.0, 0.0, 256.0, 256.0);
        let uv = rect.to_uv(256.0, 256.0);
        assert_eq!(uv, [0.0, 0.0, 1.0, 1.0]);
    }

    #[test]
    fn rect_to_uv_sub_region() {
        let rect = Rect::new(64.0, 64.0, 64.0, 64.0);
        let uv = rect.to_uv(256.0, 256.0);
        assert_eq!(uv, [0.25, 0.25, 0.5, 0.5]);
    }

    #[test]
    fn default_sprite_is_white() {
        let sprite = Sprite::default();
        assert_eq!(sprite.color, Vec4::ONE);
    }

    #[test]
    fn build_quad_produces_correct_vertex_count() {
        let transform = Transform2D::default();
        let sprite = Sprite {
            custom_size: Some(Vec2::new(32.0, 32.0)),
            ..Default::default()
        };
        let (vertices, indices, _) = build_sprite_quad(&transform, &sprite, 32.0, 32.0);
        assert_eq!(vertices.len(), 4);
        assert_eq!(indices.len(), 6);
    }
}
