use crate::hierarchy::Parent;
use ember_2d::sprite::{LocalTransform2D, Sprite, Transform2D};
use ember_core::world::World;
use serde::{Deserialize, Serialize};

/// Represents a serialized entity along with its children in a scene file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneEntity {
    pub name: Option<String>,
    pub local_transform: Option<LocalTransform2D>,
    pub transform: Option<Transform2D>,
    pub sprite: Option<Sprite>,
    #[serde(default)]
    pub children: Vec<SceneEntity>,
}

/// A serialized scene template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub roots: Vec<SceneEntity>,
}

impl Scene {
    /// Recursively spawns this template node and all children into the `World`.
    fn spawn_entity(
        world: &mut World,
        template: &SceneEntity,
        parent: Option<ember_core::entity::Entity>,
    ) -> ember_core::entity::Entity {
        let builder = world.spawn();
        let entity = builder.id();

        if let Some(ref local) = template.local_transform {
            world.insert_component(entity, local.clone());
        }

        if let Some(ref transform) = template.transform {
            world.insert_component(entity, transform.clone());
        } else if template.local_transform.is_some() && parent.is_some() {
            // Parent-child relationships need a global Transform2D target to write to
            world.insert_component(entity, Transform2D::default());
        }

        if let Some(ref sprite) = template.sprite {
            world.insert_component(entity, sprite.clone());
        }

        if let Some(p) = parent {
            world.insert_component(entity, Parent(p));
        }

        for child_template in &template.children {
            Self::spawn_entity(world, child_template, Some(entity));
        }

        entity
    }

    /// Extrudes the entire abstract scene graph into live ECS entities.
    pub fn spawn(&self, world: &mut World) {
        for root in &self.roots {
            Self::spawn_entity(world, root, None);
        }
    }
}
