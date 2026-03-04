use crate::scene::Scene;
use ember_assets::loader::AssetLoader;

/// AssetLoader responsible for loading `.ron` files into `Scene` structs.
pub struct SceneLoader;

impl AssetLoader for SceneLoader {
    type Asset = Scene;

    fn load(&self, bytes: &[u8]) -> Result<Self::Asset, String> {
        let scene: Scene = ron::de::from_bytes(bytes).map_err(|e| e.to_string())?;
        Ok(scene)
    }
}
