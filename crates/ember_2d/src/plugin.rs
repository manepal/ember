use ember_core::app::App;
use ember_core::plugin::Plugin;

/// Plugin that sets up the 2D rendering pipeline.
/// Must be added after `WindowPlugin` and `RenderPlugin`.
pub struct Render2DPlugin;

impl Plugin for Render2DPlugin {
    fn build(&self, _app: &mut App) {
        // SpriteBatchResources and TextureStore are initialized lazily
        // or by the user before app.run(), because they need the GPU device
        // which is only available after the window is created.
        //
        // Usage pattern:
        //   app.add_plugin(WindowPlugin::default());
        //   app.add_plugin(RenderPlugin);
        //   app.add_plugin(Render2DPlugin);
        //   // After app starts, the window handler initializes GPU,
        //   // then user code sets up TextureStore and SpriteBatchResources.
    }
}
