pub trait AssetLoader: Send + Sync + 'static {
    type Asset;
    /// Convert bytes into the specific asset type.
    fn load(&self, bytes: &[u8]) -> Result<Self::Asset, String>;
}
