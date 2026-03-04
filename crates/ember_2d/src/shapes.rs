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
