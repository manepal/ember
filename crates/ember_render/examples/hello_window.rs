use ember_core::app::App;
use ember_core::plugin::CorePlugin;
use ember_render::clear_pass::{ClearColor, ClearPassNode};
use ember_render::context::RenderPlugin;
use ember_render::graph::RenderGraph;
use ember_render::window::WindowPlugin;

fn main() {
    let mut app = App::new();

    app.add_plugin(CorePlugin);
    app.add_plugin(WindowPlugin {
        title: "Ember Engine — Hello Window".to_string(),
        width: 1280,
        height: 720,
    });
    app.add_plugin(RenderPlugin);

    // Set the clear color (dark blue-grey)
    app.insert_resource(ClearColor(0.1, 0.1, 0.15, 1.0));

    // Build render graph: just a clear pass for now
    let mut graph = RenderGraph::new();
    graph.add_node("clear", ClearPassNode);
    app.insert_resource(graph);

    app.run();
}
