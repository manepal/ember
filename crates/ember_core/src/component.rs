use std::any::TypeId;

/// Marker trait representing any rust type that can be mapped to an Entity as a Component.
/// ECS components must be plain data elements to enable dense table-storage layout.
pub trait Component: 'static + Send + Sync {}

// Auto-implement the Component trait for all compatible types.
impl<T: 'static + Send + Sync> Component for T {}

/// Identifies a component's memory layout mapping inside of an Archetype.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentId(pub TypeId);

impl ComponentId {
    /// Helper to efficiently fetch the ComponentId representing type `T`.
    pub fn of<T: Component>() -> Self {
        Self(TypeId::of::<T>())
    }
}
