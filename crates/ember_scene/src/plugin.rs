use ember_core::app::App;
use ember_core::plugin::Plugin;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, _app: &mut App) {
        // Register components, systems, resources, events
    }
}
