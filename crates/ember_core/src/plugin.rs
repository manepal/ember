use crate::app::App;

/// A plugin configures the `App` instance by inserting resources, adding systems,
/// and registering other core functionality prior to application startup.
pub trait Plugin {
    fn build(&self, app: &mut App);
}

/// The foundational plugin for the Ember Engine. It initializes structural resources like `Time`.
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(crate::time::Time::new());
    }
}
