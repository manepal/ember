use glam::{Mat4, Vec2};

/// 2D Camera component that defines the view into the world.
#[derive(Debug, Clone)]
pub struct Camera2D {
    /// Camera position in world space.
    pub position: Vec2,
    /// Zoom level (1.0 = default, >1 = zoomed in, <1 = zoomed out).
    pub zoom: f32,
    /// Viewport dimensions in pixels.
    pub viewport_width: f32,
    pub viewport_height: f32,
}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
            viewport_width: 1280.0,
            viewport_height: 720.0,
        }
    }
}

impl Camera2D {
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            viewport_width,
            viewport_height,
            ..Default::default()
        }
    }

    /// Compute the orthographic view-projection matrix.
    /// This maps world coordinates to clip space (-1..1).
    pub fn view_projection(&self) -> Mat4 {
        let half_w = (self.viewport_width / 2.0) / self.zoom;
        let half_h = (self.viewport_height / 2.0) / self.zoom;

        let left = self.position.x - half_w;
        let right = self.position.x + half_w;
        let bottom = self.position.y - half_h;
        let top = self.position.y + half_h;

        Mat4::orthographic_rh(left, right, bottom, top, -1.0, 1.0)
    }
}

/// GPU-ready camera uniform data.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn from_camera(camera: &Camera2D) -> Self {
        Self {
            view_proj: camera.view_projection().to_cols_array_2d(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_camera_produces_valid_matrix() {
        let camera = Camera2D::default();
        let vp = camera.view_projection();
        // The matrix should not be zero
        let cols = vp.to_cols_array();
        assert!(cols.iter().any(|&v| v != 0.0));
    }

    #[test]
    fn camera_zoom_changes_projection() {
        let mut camera = Camera2D::new(800.0, 600.0);
        let vp1 = camera.view_projection();

        camera.zoom = 2.0;
        let vp2 = camera.view_projection();

        // Zooming in should change the projection
        assert_ne!(vp1, vp2);
    }

    #[test]
    fn camera_uniform_round_trip() {
        let camera = Camera2D::new(1280.0, 720.0);
        let uniform = CameraUniform::from_camera(&camera);

        // Verify the uniform data matches the camera's matrix
        let expected = camera.view_projection().to_cols_array_2d();
        assert_eq!(uniform.view_proj, expected);
    }
}
