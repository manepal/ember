use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::{Arc, Weak};

/// A unique identifier for an asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetId(pub u64);

/// Strong, reference-counted handle to an asset.
#[derive(Debug)]
pub struct Handle<T> {
    pub id: AssetId,
    ref_count: Arc<()>,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Handle<T> {
    // Used internally by AssetServer to create a new handle
    pub(crate) fn new(id: AssetId, ref_count: Arc<()>) -> Self {
        Self {
            id,
            ref_count,
            _marker: PhantomData,
        }
    }

    /// Convert to a weak handle that does not keep the asset alive.
    pub fn downgrade(&self) -> HandleWeak<T> {
        HandleWeak {
            id: self.id,
            weak_ref: Arc::downgrade(&self.ref_count),
            _marker: PhantomData,
        }
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            ref_count: self.ref_count.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for Handle<T> {}

impl<T> Hash for Handle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// A weak, non-owning handle to an asset.
#[derive(Debug)]
pub struct HandleWeak<T> {
    pub id: AssetId,
    weak_ref: Weak<()>,
    _marker: PhantomData<fn() -> T>,
}

impl<T> HandleWeak<T> {
    /// Upgrade to a strong handle if the asset is still alive.
    pub fn upgrade(&self) -> Option<Handle<T>> {
        self.weak_ref.upgrade().map(|ref_count| Handle {
            id: self.id,
            ref_count,
            _marker: PhantomData,
        })
    }
}

impl<T> Clone for HandleWeak<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            weak_ref: self.weak_ref.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T> PartialEq for HandleWeak<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for HandleWeak<T> {}

impl<T> Hash for HandleWeak<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// The current status of an asynchronously loaded asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoadState {
    NotLoaded,
    Loading,
    Loaded,
    Error,
}
