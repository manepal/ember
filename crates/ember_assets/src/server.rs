use crossbeam_channel::{unbounded, Receiver, Sender};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::handle::{AssetId, Handle, LoadState};
use crate::loader::AssetLoader;
use crate::storage::Assets;
use ember_core::event::Events;
use ember_core::system::ResMut;

/// The resource used by the loader threads to send loaded assets back.
pub struct AssetChannel<T> {
    pub sender: Sender<(AssetId, Result<T, String>)>,
    pub receiver: Receiver<(AssetId, Result<T, String>)>,
}

impl<T> Default for AssetChannel<T> {
    fn default() -> Self {
        let (sender, receiver) = unbounded();
        Self { sender, receiver }
    }
}

/// AssetEvent is sent whenever an asset's state changes.
#[derive(Debug, Clone)]
pub enum AssetEvent<T> {
    Created {
        id: AssetId,
    },
    Modified {
        id: AssetId,
    },
    Removed {
        id: AssetId,
    },
    #[doc(hidden)]
    _Marker(std::marker::PhantomData<T>),
}

struct AssetInfo {
    state: LoadState,
    path: String,
}

/// The main AssetServer resource.
pub struct AssetServer {
    next_id: std::sync::atomic::AtomicU64,
    // Maps paths to already assigned AssetIds
    path_to_id: Arc<Mutex<HashMap<String, AssetId>>>,
    // Maps AssetIds to their current loading state and path
    id_to_info: Arc<Mutex<HashMap<AssetId, AssetInfo>>>,
}

impl Default for AssetServer {
    fn default() -> Self {
        Self {
            next_id: std::sync::atomic::AtomicU64::new(1),
            path_to_id: Arc::new(Mutex::new(HashMap::new())),
            id_to_info: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl AssetServer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieve the load state of a given asset by its handle.
    pub fn get_load_state<T>(&self, handle: &Handle<T>) -> LoadState {
        let info_map = self.id_to_info.lock().unwrap();
        info_map
            .get(&handle.id)
            .map(|info| info.state)
            .unwrap_or(LoadState::NotLoaded)
    }

    /// Load an asset from a file path using the given loader and channel.
    pub fn load<T: Send + Sync + 'static, L: AssetLoader<Asset = T>>(
        &self,
        path: &str,
        loader: L,
        channel: &AssetChannel<T>,
    ) -> Handle<T> {
        let mut path_map = self.path_to_id.lock().unwrap();

        // If it's already loading/loaded, just return a new handle with the same ID.
        if let Some(&id) = path_map.get(path) {
            return Handle::new(id, Arc::new(()));
        }

        // Allocate a new ID
        let id_raw = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let id = AssetId(id_raw);

        path_map.insert(path.to_string(), id);

        {
            let mut info_map = self.id_to_info.lock().unwrap();
            info_map.insert(
                id,
                AssetInfo {
                    state: LoadState::Loading,
                    path: path.to_string(),
                },
            );
        }

        let handle = Handle::new(id, Arc::new(()));

        // Spawn a background thread to do the actual loading
        let sender = channel.sender.clone();
        let path_clone = path.to_string();

        std::thread::spawn(move || {
            // Read file bytes
            match std::fs::read(&path_clone) {
                Ok(bytes) => {
                    // Call the appropriate loader
                    let result = loader.load(&bytes);
                    let _ = sender.send((id, result));
                }
                Err(e) => {
                    let _ = sender.send((id, Err(e.to_string())));
                }
            }
        });

        handle
    }
}

/// Generic system that processes results from the background threads for type `T`.
pub fn update_asset_state<T: Send + Sync + 'static>(
    mut assets: ResMut<Assets<T>>,
    mut server: ResMut<AssetServer>,
    channel: ResMut<AssetChannel<T>>,
    mut events: ResMut<Events<AssetEvent<T>>>,
) {
    while let Ok((id, result)) = channel.receiver.try_recv() {
        let mut info_map = server.id_to_info.lock().unwrap();

        if let Some(info) = info_map.get_mut(&id) {
            match result {
                Ok(asset) => {
                    info.state = LoadState::Loaded;
                    // We need a dummy handle just to insert, or we can add `insert_by_id` to `Assets`.
                    // Let's use `insert_by_id` equivalent since we have the ID directly.
                    // Wait, `Assets::insert` expects a `&Handle<T>`. We'll just forge one.
                    let dummy_handle = Handle::<T>::new(id, Arc::new(()));
                    assets.insert(&dummy_handle, asset);

                    events.send(AssetEvent::Created { id });
                }
                Err(e) => {
                    eprintln!("Failed to load asset '{}': {}", info.path, e);
                    info.state = LoadState::Error;
                }
            }
        }
    }
}
