use crate::hierarchy::Parent;
use ember_2d::sprite::{LocalTransform2D, Transform2D};
use ember_core::entity::Entity;
use ember_core::query::Query;
use glam::Vec2;

/// A simple transform propagation system.
/// This reads `LocalTransform2D` and `Parent`, computes the new global `Transform2D`,
/// and applies it to the entity.
/// Note: This is a single-pass implementation. Deep hierarchies > 1 level may lag by a frame
/// unless sorted topologically.
pub fn propagate_transforms_system(
    query_children: Query<(Entity, &LocalTransform2D, &Parent)>,
    query_transforms: Query<&mut Transform2D>,
) {
    let mut updates = Vec::new();

    for (entity, local, parent) in query_children.iter() {
        if let Some(parent_transform) = query_children.get_component::<Transform2D>(parent.0) {
            let mut global = Transform2D::default();

            let cos_r = parent_transform.rotation.cos();
            let sin_r = parent_transform.rotation.sin();
            let scaled_local_x = local.0.position.x * parent_transform.scale.x;
            let scaled_local_y = local.0.position.y * parent_transform.scale.y;

            let rotated_x = scaled_local_x * cos_r - scaled_local_y * sin_r;
            let rotated_y = scaled_local_x * sin_r + scaled_local_y * cos_r;

            global.position = Vec2::new(
                parent_transform.position.x + rotated_x,
                parent_transform.position.y + rotated_y,
            );
            global.rotation = parent_transform.rotation + local.0.rotation;
            global.scale = parent_transform.scale * local.0.scale;
            global.z_order = parent_transform.z_order + local.0.z_order;

            updates.push((entity, global));
        }
    }

    for (entity, new_transform) in updates {
        if let Some(transform) = query_transforms.get_component_mut::<Transform2D>(entity) {
            *transform = new_transform;
        }
    }
}
