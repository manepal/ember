use ember_core::entity::Entity;

/// Component indicating that an entity is a child of another entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parent(pub Entity);

/// Component listing all child entities of this entity.
#[derive(Debug, Clone, Default)]
pub struct Children(pub Vec<Entity>);
