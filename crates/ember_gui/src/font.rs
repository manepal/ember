//! Font rasterization and glyph atlas for GUI text rendering.
//!
//! Uses `fontdue` for CPU-side glyph rasterization. Glyphs are cached in a
//! `GlyphAtlas` that can be uploaded to a GPU texture for rendering.
//!
//! For bootstrapping or environments without TTF files, a built-in
//! procedural pixel font is provided via `GlyphAtlas::with_builtin_font()`.

use std::collections::HashMap;

/// Metrics for a single rasterized glyph.
#[derive(Debug, Clone, Copy)]
pub struct GlyphMetrics {
    /// Width of the glyph bitmap in pixels.
    pub width: u32,
    /// Height of the glyph bitmap in pixels.
    pub height: u32,
    /// Horizontal offset from the cursor to the left edge of the glyph bitmap.
    pub xmin: f32,
    /// Vertical offset from the baseline to the top edge of the glyph bitmap.
    pub ymin: f32,
    /// How far the cursor advances after this glyph.
    pub advance_width: f32,
}

/// A cached glyph entry in the atlas, storing its metrics and position.
#[derive(Debug, Clone, Copy)]
pub struct GlyphEntry {
    pub metrics: GlyphMetrics,
    /// X position of this glyph's bitmap in the atlas texture (pixels).
    pub atlas_x: u32,
    /// Y position of this glyph's bitmap in the atlas texture (pixels).
    pub atlas_y: u32,
}

/// Key for looking up a cached glyph: (character, font_size_in_px as u16).
type GlyphKey = (char, u16);

/// A bitmap glyph atlas that caches rasterized glyphs for a font.
///
/// Glyphs are rasterized on-demand and packed into a greyscale bitmap atlas.
/// The atlas uses a simple row-based packing strategy.
pub struct GlyphAtlas {
    /// The underlying fontdue font (None for builtin pixel font).
    font: Option<fontdue::Font>,
    /// Atlas bitmap (single channel, greyscale). Row-major, width × height.
    pub bitmap: Vec<u8>,
    /// Atlas width in pixels.
    pub width: u32,
    /// Atlas height in pixels.
    pub height: u32,
    /// Cache of already-rasterized glyphs.
    glyphs: HashMap<GlyphKey, GlyphEntry>,
    /// Current packing cursor: next free X position in the current row.
    cursor_x: u32,
    /// Current packing cursor: Y position of the current row.
    cursor_y: u32,
    /// Height of the tallest glyph in the current row.
    row_height: u32,
    /// Tracks whether the bitmap has changed since the last GPU upload.
    pub dirty: bool,
}

impl GlyphAtlas {
    /// Create a new glyph atlas from raw font data (TTF/OTF bytes).
    ///
    /// `atlas_size` is the width and height of the atlas bitmap in pixels.
    /// A 512×512 atlas is typically sufficient for ASCII at multiple sizes.
    pub fn from_bytes(font_data: &[u8], atlas_size: u32) -> Result<Self, String> {
        let font = fontdue::Font::from_bytes(font_data, fontdue::FontSettings::default())
            .map_err(|e| format!("Failed to load font: {}", e))?;

        let total = (atlas_size * atlas_size) as usize;
        Ok(Self {
            font: Some(font),
            bitmap: vec![0u8; total],
            width: atlas_size,
            height: atlas_size,
            glyphs: HashMap::new(),
            cursor_x: 0,
            cursor_y: 0,
            row_height: 0,
            dirty: false,
        })
    }

    /// Load a font from a file path and create a glyph atlas.
    pub fn from_file(path: &str, atlas_size: u32) -> Result<Self, String> {
        let data = std::fs::read(path)
            .map_err(|e| format!("Failed to read font file '{}': {}", path, e))?;
        Self::from_bytes(&data, atlas_size)
    }

    /// Create a glyph atlas with a built-in procedural pixel font.
    ///
    /// This provides basic ASCII rendering (uppercase, lowercase, digits,
    /// punctuation) without requiring any external font files. Each glyph
    /// is a 5×7 pixel bitmap, suitable for debug overlays and demos.
    pub fn with_builtin_font() -> Self {
        let atlas_size = 256u32;
        let total = (atlas_size * atlas_size) as usize;
        let mut atlas = Self {
            font: None,
            bitmap: vec![0u8; total],
            width: atlas_size,
            height: atlas_size,
            glyphs: HashMap::new(),
            cursor_x: 0,
            cursor_y: 0,
            row_height: 0,
            dirty: false,
        };

        // Pre-populate printable ASCII (32..127) with pixel font glyphs
        for ch in ' '..='~' {
            let glyph_bitmap = builtin_glyph(ch);
            atlas.insert_builtin_glyph(ch, &glyph_bitmap, BUILTIN_GLYPH_W, BUILTIN_GLYPH_H);
        }

        atlas
    }

    /// Insert a pre-rendered glyph bitmap into the atlas.
    fn insert_builtin_glyph(&mut self, ch: char, pixels: &[u8], gw: u32, gh: u32) {
        // Use a fixed "size" key for the builtin font
        let key: GlyphKey = (ch, 0);

        // Check if we need to advance to the next row
        if self.cursor_x + gw + 1 > self.width {
            self.cursor_y += self.row_height + 1;
            self.cursor_x = 0;
            self.row_height = 0;
        }

        if self.cursor_y + gh > self.height {
            return; // Atlas full
        }

        let ax = self.cursor_x;
        let ay = self.cursor_y;

        // Copy into atlas bitmap
        for row in 0..gh {
            for col in 0..gw {
                let src = (row * gw + col) as usize;
                let dst = ((ay + row) * self.width + ax + col) as usize;
                if src < pixels.len() && dst < self.bitmap.len() {
                    self.bitmap[dst] = pixels[src];
                }
            }
        }

        let entry = GlyphEntry {
            metrics: GlyphMetrics {
                width: gw,
                height: gh,
                xmin: 0.0,
                ymin: 0.0,
                advance_width: (gw + 1) as f32, // +1 for spacing
            },
            atlas_x: ax,
            atlas_y: ay,
        };

        self.cursor_x += gw + 1;
        if gh + 1 > self.row_height {
            self.row_height = gh + 1;
        }

        self.glyphs.insert(key, entry);
        self.dirty = true;
    }

    /// Look up or rasterize a glyph, returning its atlas entry.
    ///
    /// If the glyph hasn't been rasterized at this size yet, it will be
    /// rasterized and packed into the atlas bitmap.
    pub fn get_or_insert(&mut self, ch: char, font_size_px: f32) -> Option<GlyphEntry> {
        // Builtin font: ignore font_size_px, use the key (char, 0)
        if self.font.is_none() {
            return self.glyphs.get(&(ch, 0)).copied();
        }

        let key: GlyphKey = (ch, font_size_px as u16);

        if let Some(&entry) = self.glyphs.get(&key) {
            return Some(entry);
        }

        // Rasterize the glyph using fontdue
        let font = self.font.as_ref().unwrap();
        let (metrics, raster) = font.rasterize(ch, font_size_px);

        let gw = metrics.width as u32;
        let gh = metrics.height as u32;

        // Skip zero-size glyphs (spaces, etc.) but still cache metrics
        if gw == 0 || gh == 0 {
            let entry = GlyphEntry {
                metrics: GlyphMetrics {
                    width: 0,
                    height: 0,
                    xmin: metrics.xmin as f32,
                    ymin: metrics.ymin as f32,
                    advance_width: metrics.advance_width,
                },
                atlas_x: 0,
                atlas_y: 0,
            };
            self.glyphs.insert(key, entry);
            return Some(entry);
        }

        // Check if we need to advance to the next row
        if self.cursor_x + gw + 1 > self.width {
            self.cursor_y += self.row_height + 1;
            self.cursor_x = 0;
            self.row_height = 0;
        }

        // Check if we've run out of vertical space
        if self.cursor_y + gh > self.height {
            return None; // Atlas is full
        }

        // Copy glyph bitmap into the atlas
        let ax = self.cursor_x;
        let ay = self.cursor_y;
        for row in 0..gh {
            let src_start = (row * gw) as usize;
            let dst_start = ((ay + row) * self.width + ax) as usize;
            let src_end = src_start + gw as usize;
            let dst_end = dst_start + gw as usize;
            if dst_end <= self.bitmap.len() && src_end <= raster.len() {
                self.bitmap[dst_start..dst_end].copy_from_slice(&raster[src_start..src_end]);
            }
        }

        let entry = GlyphEntry {
            metrics: GlyphMetrics {
                width: gw,
                height: gh,
                xmin: metrics.xmin as f32,
                ymin: metrics.ymin as f32,
                advance_width: metrics.advance_width,
            },
            atlas_x: ax,
            atlas_y: ay,
        };

        // Advance packing cursor
        self.cursor_x += gw + 1;
        if gh + 1 > self.row_height {
            self.row_height = gh + 1;
        }

        self.glyphs.insert(key, entry);
        self.dirty = true;

        Some(entry)
    }

    /// Measure the width of a string at the given font size (in pixels).
    pub fn measure_text(&mut self, text: &str, font_size_px: f32) -> f32 {
        let mut width = 0.0f32;
        for ch in text.chars() {
            if let Some(entry) = self.get_or_insert(ch, font_size_px) {
                width += entry.metrics.advance_width;
            }
        }
        width
    }

    /// Measure text with word wrapping, returning (max_line_width, total_height).
    pub fn measure_text_wrapped(
        &mut self,
        text: &str,
        font_size_px: f32,
        max_width: f32,
        line_height: f32,
    ) -> (f32, f32) {
        let mut x = 0.0f32;
        let mut lines = 1u32;
        let mut max_line_width = 0.0f32;

        for word in text.split_whitespace() {
            let word_width = self.measure_text(word, font_size_px);
            let space_width = self.measure_text(" ", font_size_px);

            if x + word_width > max_width && x > 0.0 {
                max_line_width = max_line_width.max(x - space_width);
                x = 0.0;
                lines += 1;
            }

            x += word_width + space_width;
        }

        max_line_width = max_line_width.max(x);
        (max_line_width, lines as f32 * line_height)
    }

    /// Returns the UV coordinates (u0, v0, u1, v1) for a glyph in the atlas.
    pub fn glyph_uv(&self, entry: &GlyphEntry) -> (f32, f32, f32, f32) {
        let u0 = entry.atlas_x as f32 / self.width as f32;
        let v0 = entry.atlas_y as f32 / self.height as f32;
        let u1 = (entry.atlas_x + entry.metrics.width) as f32 / self.width as f32;
        let v1 = (entry.atlas_y + entry.metrics.height) as f32 / self.height as f32;
        (u0, v0, u1, v1)
    }

    /// Get the effective glyph height for the builtin font.
    pub fn builtin_glyph_height(&self) -> u32 {
        BUILTIN_GLYPH_H
    }

    /// Get the effective glyph width for the builtin font.
    pub fn builtin_glyph_width(&self) -> u32 {
        BUILTIN_GLYPH_W
    }
}

// ---------------------------------------------------------------------------
// Built-in 5×7 pixel font
// ---------------------------------------------------------------------------

const BUILTIN_GLYPH_W: u32 = 5;
const BUILTIN_GLYPH_H: u32 = 7;

/// Returns a 5×7 pixel bitmap for printable ASCII characters.
/// Each byte is 0 (off) or 255 (on).
fn builtin_glyph(ch: char) -> Vec<u8> {
    let pattern: [u8; 7] = match ch {
        ' ' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
        '!' => [
            0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100, 0b00000,
        ],
        '"' => [
            0b01010, 0b01010, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
        '#' => [
            0b01010, 0b11111, 0b01010, 0b01010, 0b11111, 0b01010, 0b00000,
        ],
        '$' => [
            0b00100, 0b01111, 0b10100, 0b01110, 0b00101, 0b11110, 0b00100,
        ],
        '%' => [
            0b11001, 0b11010, 0b00100, 0b01000, 0b01011, 0b10011, 0b00000,
        ],
        '&' => [
            0b01100, 0b10010, 0b01100, 0b10101, 0b10010, 0b01101, 0b00000,
        ],
        '\'' => [
            0b00100, 0b00100, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
        '(' => [
            0b00010, 0b00100, 0b01000, 0b01000, 0b00100, 0b00010, 0b00000,
        ],
        ')' => [
            0b01000, 0b00100, 0b00010, 0b00010, 0b00100, 0b01000, 0b00000,
        ],
        '*' => [
            0b00000, 0b00100, 0b10101, 0b01110, 0b10101, 0b00100, 0b00000,
        ],
        '+' => [
            0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000,
        ],
        ',' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b00100, 0b01000,
        ],
        '-' => [
            0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
        ],
        '.' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b00000,
        ],
        '/' => [
            0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b00000, 0b00000,
        ],
        '0' => [
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111,
        ],
        '3' => [
            0b01110, 0b10001, 0b00001, 0b00110, 0b00001, 0b10001, 0b01110,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => [
            0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110,
        ],
        '6' => [
            0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100,
        ],
        ':' => [
            0b00000, 0b00100, 0b00000, 0b00000, 0b00100, 0b00000, 0b00000,
        ],
        ';' => [
            0b00000, 0b00100, 0b00000, 0b00000, 0b00100, 0b00100, 0b01000,
        ],
        '<' => [
            0b00010, 0b00100, 0b01000, 0b10000, 0b01000, 0b00100, 0b00010,
        ],
        '=' => [
            0b00000, 0b00000, 0b11111, 0b00000, 0b11111, 0b00000, 0b00000,
        ],
        '>' => [
            0b01000, 0b00100, 0b00010, 0b00001, 0b00010, 0b00100, 0b01000,
        ],
        '?' => [
            0b01110, 0b10001, 0b00010, 0b00100, 0b00000, 0b00100, 0b00000,
        ],
        '@' => [
            0b01110, 0b10001, 0b10111, 0b10101, 0b10111, 0b10000, 0b01110,
        ],
        'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'B' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
        'C' => [
            0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110,
        ],
        'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'F' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'G' => [
            0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111,
        ],
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'I' => [
            0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        'J' => [
            0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        'N' => [
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'Q' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        'S' => [
            0b01110, 0b10001, 0b10000, 0b01110, 0b00001, 0b10001, 0b01110,
        ],
        'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'V' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b01010, 0b00100,
        ],
        'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001,
        ],
        'X' => [
            0b10001, 0b01010, 0b00100, 0b00100, 0b01010, 0b10001, 0b00000,
        ],
        'Y' => [
            0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'Z' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
        ],
        '[' => [
            0b01110, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b01110,
        ],
        '\\' => [
            0b10000, 0b01000, 0b00100, 0b00010, 0b00001, 0b00000, 0b00000,
        ],
        ']' => [
            0b01110, 0b00010, 0b00010, 0b00010, 0b00010, 0b00010, 0b01110,
        ],
        '^' => [
            0b00100, 0b01010, 0b10001, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
        '_' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111,
        ],
        '`' => [
            0b01000, 0b00100, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
        'a' => [
            0b00000, 0b00000, 0b01110, 0b00001, 0b01111, 0b10001, 0b01111,
        ],
        'b' => [
            0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        'c' => [
            0b00000, 0b00000, 0b01110, 0b10000, 0b10000, 0b10001, 0b01110,
        ],
        'd' => [
            0b00001, 0b00001, 0b01111, 0b10001, 0b10001, 0b10001, 0b01111,
        ],
        'e' => [
            0b00000, 0b00000, 0b01110, 0b10001, 0b11111, 0b10000, 0b01110,
        ],
        'f' => [
            0b00110, 0b01001, 0b01000, 0b11100, 0b01000, 0b01000, 0b01000,
        ],
        'g' => [
            0b00000, 0b01111, 0b10001, 0b10001, 0b01111, 0b00001, 0b01110,
        ],
        'h' => [
            0b10000, 0b10000, 0b10110, 0b11001, 0b10001, 0b10001, 0b10001,
        ],
        'i' => [
            0b00100, 0b00000, 0b01100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        'j' => [
            0b00010, 0b00000, 0b00110, 0b00010, 0b00010, 0b10010, 0b01100,
        ],
        'k' => [
            0b10000, 0b10000, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010,
        ],
        'l' => [
            0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        'm' => [
            0b00000, 0b00000, 0b11010, 0b10101, 0b10101, 0b10001, 0b10001,
        ],
        'n' => [
            0b00000, 0b00000, 0b10110, 0b11001, 0b10001, 0b10001, 0b10001,
        ],
        'o' => [
            0b00000, 0b00000, 0b01110, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'p' => [
            0b00000, 0b00000, 0b11110, 0b10001, 0b11110, 0b10000, 0b10000,
        ],
        'q' => [
            0b00000, 0b00000, 0b01111, 0b10001, 0b01111, 0b00001, 0b00001,
        ],
        'r' => [
            0b00000, 0b00000, 0b10110, 0b11001, 0b10000, 0b10000, 0b10000,
        ],
        's' => [
            0b00000, 0b00000, 0b01111, 0b10000, 0b01110, 0b00001, 0b11110,
        ],
        't' => [
            0b01000, 0b01000, 0b11100, 0b01000, 0b01000, 0b01001, 0b00110,
        ],
        'u' => [
            0b00000, 0b00000, 0b10001, 0b10001, 0b10001, 0b10011, 0b01101,
        ],
        'v' => [
            0b00000, 0b00000, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
        ],
        'w' => [
            0b00000, 0b00000, 0b10001, 0b10001, 0b10101, 0b10101, 0b01010,
        ],
        'x' => [
            0b00000, 0b00000, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001,
        ],
        'y' => [
            0b00000, 0b00000, 0b10001, 0b10001, 0b01111, 0b00001, 0b01110,
        ],
        'z' => [
            0b00000, 0b00000, 0b11111, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        '{' => [
            0b00010, 0b00100, 0b00100, 0b01000, 0b00100, 0b00100, 0b00010,
        ],
        '|' => [
            0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        '}' => [
            0b01000, 0b00100, 0b00100, 0b00010, 0b00100, 0b00100, 0b01000,
        ],
        '~' => [
            0b00000, 0b00000, 0b01000, 0b10101, 0b00010, 0b00000, 0b00000,
        ],
        _ => [
            0b11111, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11111,
        ], // unknown
    };

    // Convert bit pattern to pixel bitmap (5 wide, 7 tall)
    let mut pixels = vec![0u8; (BUILTIN_GLYPH_W * BUILTIN_GLYPH_H) as usize];
    for row in 0..7u32 {
        for col in 0..5u32 {
            let bit = (pattern[row as usize] >> (4 - col)) & 1;
            pixels[(row * BUILTIN_GLYPH_W + col) as usize] = if bit == 1 { 255 } else { 0 };
        }
    }
    pixels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_atlas_creates_successfully() {
        let atlas = GlyphAtlas::with_builtin_font();
        assert!(atlas.width > 0);
        assert!(atlas.height > 0);
    }

    #[test]
    fn builtin_glyph_lookup() {
        let mut atlas = GlyphAtlas::with_builtin_font();
        let entry = atlas.get_or_insert('A', 16.0);
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.metrics.width, BUILTIN_GLYPH_W);
        assert_eq!(entry.metrics.height, BUILTIN_GLYPH_H);
    }

    #[test]
    fn builtin_space_has_advance() {
        let mut atlas = GlyphAtlas::with_builtin_font();
        let entry = atlas.get_or_insert(' ', 16.0).unwrap();
        assert!(entry.metrics.advance_width > 0.0);
    }

    #[test]
    fn builtin_measure_text() {
        let mut atlas = GlyphAtlas::with_builtin_font();
        let w = atlas.measure_text("Hello", 16.0);
        assert!(w > 0.0);
        // 5 chars × 6px advance each = 30
        assert!((w - 30.0).abs() < 1.0);
    }

    #[test]
    fn builtin_measure_empty() {
        let mut atlas = GlyphAtlas::with_builtin_font();
        assert_eq!(atlas.measure_text("", 16.0), 0.0);
    }

    #[test]
    fn builtin_all_ascii_present() {
        let mut atlas = GlyphAtlas::with_builtin_font();
        for ch in ' '..='~' {
            assert!(
                atlas.get_or_insert(ch, 16.0).is_some(),
                "Missing glyph for '{}'",
                ch
            );
        }
    }

    #[test]
    fn glyph_uv_within_bounds() {
        let mut atlas = GlyphAtlas::with_builtin_font();
        let entry = atlas.get_or_insert('X', 16.0).unwrap();
        let (u0, v0, u1, v1) = atlas.glyph_uv(&entry);
        assert!(u0 >= 0.0 && u0 <= 1.0);
        assert!(v0 >= 0.0 && v0 <= 1.0);
        assert!(u1 >= u0 && u1 <= 1.0);
        assert!(v1 >= v0 && v1 <= 1.0);
    }

    #[test]
    fn atlas_dirty_after_builtin_creation() {
        let atlas = GlyphAtlas::with_builtin_font();
        assert!(atlas.dirty);
    }
}
