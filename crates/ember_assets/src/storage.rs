use std::any::Any;
use std::collections::HashMap;

use crate::handle::{AssetId, Handle};

/// Stores loaded assets of type `T`.
pub struct Assets<T: Any + Send + Sync + 'static> {
    data: HashMap<AssetId, T>,
}

impl<T: Any + Send + Sync + 'static> Default for Assets<T> {
    fn default() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl<T: Any + Send + Sync + 'static> Assets<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert an asset into the storage. This is typically called by the AssetServer
    /// once the asset has finished loading.
    pub fn insert(&mut self, handle: &Handle<T>, asset: T) {
        self.data.insert(handle.id, asset);
    }

    /// Retrieve a reference to the asset if it exists.
    pub fn get(&self, handle: &Handle<T>) -> Option<&T> {
        self.data.get(&handle.id)
    }

    /// Retrieve a mutable reference to the asset if it exists.
    pub fn get_mut(&mut self, handle: &Handle<T>) -> Option<&mut T> {
        self.data.get_mut(&handle.id)
    }

    /// Remove the asset from storage.
    pub fn remove(&mut self, handle: &Handle<T>) -> Option<T> {
        self.data.remove(&handle.id)
    }

    /// Iterate over all loaded assets.
    pub fn iter(&self) -> impl Iterator<Item = (&AssetId, &T)> {
        self.data.iter()
    }
}
