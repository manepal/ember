use crate::server::AssetServer;
use ember_core::app::App;
use ember_core::plugin::Plugin;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AssetServer::new());
    }
}
