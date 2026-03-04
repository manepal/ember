use crate::loader::AssetLoader;

/// A raw image data container returned by the ImageLoader.
#[derive(Debug, Clone)]
pub struct Image {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Asset loader for PNG and JPEG image files.
pub struct ImageLoader;

impl AssetLoader for ImageLoader {
    type Asset = Image;

    fn load(&self, bytes: &[u8]) -> Result<Self::Asset, String> {
        let img =
            image::load_from_memory(bytes).map_err(|e| format!("Failed to decode image: {}", e))?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        Ok(Image {
            data: rgba.into_raw(),
            width,
            height,
        })
    }
}
