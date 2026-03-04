use crate::hierarchy::Parent;
use crate::transform::propagate_transforms_system;
use ember_2d::sprite::{LocalTransform2D, Transform2D};
use ember_core::app::App;
use ember_core::entity::Entity;
use ember_core::plugin::Plugin;
use ember_core::query::Query;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_system::<fn(
            Query<'static, (Entity, &'static LocalTransform2D, &'static Parent)>,
            Query<'static, &'static mut Transform2D>,
        ), _>(propagate_transforms_system);
    }
}
