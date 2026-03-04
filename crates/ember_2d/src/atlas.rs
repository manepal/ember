use crate::sprite::Rect;

/// A texture atlas maps indices to rectangular regions within a texture.
/// Supports both grid-based and freeform layouts.
pub struct TextureAtlas {
    pub texture_index: usize,
    pub rects: Vec<Rect>,
    pub texture_width: f32,
    pub texture_height: f32,
}

impl TextureAtlas {
    /// Create an atlas from a grid of equally-sized tiles.
    pub fn from_grid(
        texture_index: usize,
        texture_width: f32,
        texture_height: f32,
        tile_width: f32,
        tile_height: f32,
        columns: usize,
        rows: usize,
    ) -> Self {
        let mut rects = Vec::with_capacity(columns * rows);
        for row in 0..rows {
            for col in 0..columns {
                rects.push(Rect::new(
                    col as f32 * tile_width,
                    row as f32 * tile_height,
                    tile_width,
                    tile_height,
                ));
            }
        }
        Self {
            texture_index,
            rects,
            texture_width,
            texture_height,
        }
    }

    /// Create an atlas from arbitrary rectangles.
    pub fn from_rects(
        texture_index: usize,
        texture_width: f32,
        texture_height: f32,
        rects: Vec<Rect>,
    ) -> Self {
        Self {
            texture_index,
            rects,
            texture_width,
            texture_height,
        }
    }

    /// Get the rectangle for a given frame/tile index.
    pub fn get_rect(&self, index: usize) -> Option<&Rect> {
        self.rects.get(index)
    }

    /// Number of frames/tiles in this atlas.
    pub fn len(&self) -> usize {
        self.rects.len()
    }

    /// Whether this atlas has no frames.
    pub fn is_empty(&self) -> bool {
        self.rects.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_atlas_correct_count() {
        let atlas = TextureAtlas::from_grid(0, 256.0, 256.0, 32.0, 32.0, 8, 8);
        assert_eq!(atlas.len(), 64);
    }

    #[test]
    fn grid_atlas_first_rect() {
        let atlas = TextureAtlas::from_grid(0, 128.0, 128.0, 32.0, 32.0, 4, 4);
        let r = atlas.get_rect(0).unwrap();
        assert_eq!(r.x, 0.0);
        assert_eq!(r.y, 0.0);
        assert_eq!(r.width, 32.0);
        assert_eq!(r.height, 32.0);
    }

    #[test]
    fn grid_atlas_second_row_first_col() {
        let atlas = TextureAtlas::from_grid(0, 128.0, 128.0, 32.0, 32.0, 4, 4);
        let r = atlas.get_rect(4).unwrap(); // row 1, col 0
        assert_eq!(r.x, 0.0);
        assert_eq!(r.y, 32.0);
    }

    #[test]
    fn freeform_atlas() {
        let rects = vec![
            Rect::new(0.0, 0.0, 50.0, 50.0),
            Rect::new(50.0, 0.0, 30.0, 40.0),
        ];
        let atlas = TextureAtlas::from_rects(0, 256.0, 256.0, rects);
        assert_eq!(atlas.len(), 2);
        assert_eq!(atlas.get_rect(1).unwrap().width, 30.0);
    }
}
