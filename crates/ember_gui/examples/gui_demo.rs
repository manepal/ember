//! GUI Demo — Demonstrates the immediate-mode GUI system
//!
//! Run: `cargo run --example gui_demo -p ember_gui`
//!
//! Shows interactive panels, buttons, sliders, checkboxes, and a progress bar.
//! Move the mouse to hover over widgets, click buttons, drag the slider.

use ember_core::app::App;
use ember_core::plugin::Plugin;
use ember_core::system::{Res, ResMut};
use ember_core::time::Time;
use ember_render::camera::Camera2D;
use ember_render::clear_pass::{ClearColor, ClearPassNode};
use ember_render::context::{GpuStartupCallbacks, RenderContext, RenderPlugin};
use ember_render::graph::RenderGraph;
use ember_render::window::WindowPlugin;

use ember_2d::shapes::{ShapeBatchNode, ShapeBatchResources, ShapeDrawQueue};
use ember_input::InputPlugin;

use ember_gui::context::GuiContext;
use ember_gui::font::GlyphAtlas;
use ember_gui::overlay::{debug_overlay_system, DebugOverlayConfig};
use ember_gui::plugin::GuiPlugin;
use ember_gui::render::gui_render_system;
use ember_gui::style::{GuiTheme, WidgetStyle};
use ember_gui::widgets;

use glam::Vec2;

// ---------------------------------------------------------------------------
// Demo State — tracks mutable widget values across frames
// ---------------------------------------------------------------------------

struct GuiDemoState {
    slider_value: f32,
    checkbox_a: bool,
    checkbox_b: bool,
    click_count: u32,
}

impl Default for GuiDemoState {
    fn default() -> Self {
        Self {
            slider_value: 0.5,
            checkbox_a: false,
            checkbox_b: true,
            click_count: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// GUI System — builds the UI each frame (immediate-mode style)
// ---------------------------------------------------------------------------

fn gui_demo_system(
    mut ctx: ResMut<GuiContext>,
    theme: Res<GuiTheme>,
    mut state: ResMut<GuiDemoState>,
    time: Res<Time>,
) {
    let panel_x = 20.0;
    let panel_y = 20.0;
    let panel_w = 320.0;
    let panel_h = 400.0;

    // --- Main Panel Background ---
    widgets::panel(
        &mut ctx,
        Vec2::new(panel_x, panel_y),
        Vec2::new(panel_w, panel_h),
        &theme.panel,
    );

    // --- Title ---
    let title_style = WidgetStyle {
        font_size: 20.0,
        ..theme.label.clone()
    };
    widgets::label(
        &mut ctx,
        Vec2::new(panel_x + 15.0, panel_y + 15.0),
        "Ember GUI Demo",
        &title_style,
    );

    // --- Separator line (thin rect) ---
    let separator_style = WidgetStyle {
        bg_color: ember_gui::context::Color::rgba(0.3, 0.3, 0.35, 1.0),
        ..Default::default()
    };
    widgets::panel(
        &mut ctx,
        Vec2::new(panel_x + 10.0, panel_y + 45.0),
        Vec2::new(panel_w - 20.0, 2.0),
        &separator_style,
    );

    // --- Buttons ---
    let button_y = panel_y + 60.0;

    let response = widgets::button(
        &mut ctx,
        "Click Me!",
        Vec2::new(panel_x + 15.0, button_y),
        Vec2::new(140.0, 35.0),
        &theme.button,
    );
    if response.clicked {
        state.click_count += 1;
    }

    // Click counter label
    let count_text = format!("Clicks: {}", state.click_count);
    widgets::label(
        &mut ctx,
        Vec2::new(panel_x + 170.0, button_y + 8.0),
        &count_text,
        &theme.label,
    );

    let _reset_response = widgets::button(
        &mut ctx,
        "Reset",
        Vec2::new(panel_x + 15.0, button_y + 45.0),
        Vec2::new(100.0, 30.0),
        &theme.button,
    );
    if _reset_response.clicked {
        state.click_count = 0;
        state.slider_value = 0.5;
    }

    // --- Slider ---
    let slider_y = button_y + 95.0;
    widgets::label(
        &mut ctx,
        Vec2::new(panel_x + 15.0, slider_y),
        "Volume",
        &theme.label,
    );

    let slider_result = widgets::slider(
        &mut ctx,
        "volume_slider",
        state.slider_value,
        0.0,
        1.0,
        Vec2::new(panel_x + 15.0, slider_y + 22.0),
        panel_w - 30.0,
        &theme.button,
    );
    state.slider_value = slider_result.value;

    // Slider value display
    let value_text = format!("{:.0}%", state.slider_value * 100.0);
    widgets::label(
        &mut ctx,
        Vec2::new(panel_x + 15.0, slider_y + 55.0),
        &value_text,
        &theme.label,
    );

    // --- Checkboxes ---
    let checkbox_y = slider_y + 80.0;
    widgets::label(
        &mut ctx,
        Vec2::new(panel_x + 15.0, checkbox_y),
        "Options",
        &theme.label,
    );

    let cb_a = widgets::checkbox(
        &mut ctx,
        "Enable shadows",
        state.checkbox_a,
        Vec2::new(panel_x + 15.0, checkbox_y + 25.0),
        &theme.button,
    );
    if cb_a.clicked {
        state.checkbox_a = !state.checkbox_a;
    }

    let cb_b = widgets::checkbox(
        &mut ctx,
        "Show FPS",
        state.checkbox_b,
        Vec2::new(panel_x + 15.0, checkbox_y + 55.0),
        &theme.button,
    );
    if cb_b.clicked {
        state.checkbox_b = !state.checkbox_b;
    }

    // --- Progress Bar ---
    let progress_y = checkbox_y + 95.0;
    widgets::label(
        &mut ctx,
        Vec2::new(panel_x + 15.0, progress_y),
        "Loading...",
        &theme.label,
    );

    // Animate progress bar using elapsed time
    let progress = (time.elapsed_seconds() * 0.15).fract();
    widgets::progress_bar(
        &mut ctx,
        progress,
        Vec2::new(panel_x + 15.0, progress_y + 22.0),
        panel_w - 30.0,
        &theme.button,
    );

    // --- Second Panel (right side) ---
    let right_x = 370.0;
    let right_y = 20.0;
    let right_w = 280.0;
    let right_h = 200.0;

    widgets::panel(
        &mut ctx,
        Vec2::new(right_x, right_y),
        Vec2::new(right_w, right_h),
        &theme.panel,
    );

    let status_style = WidgetStyle {
        font_size: 18.0,
        ..theme.label.clone()
    };
    widgets::label(
        &mut ctx,
        Vec2::new(right_x + 15.0, right_y + 15.0),
        "Status",
        &status_style,
    );

    widgets::panel(
        &mut ctx,
        Vec2::new(right_x + 10.0, right_y + 45.0),
        Vec2::new(right_w - 20.0, 2.0),
        &separator_style,
    );

    let fps_text = format!("Frame: {}", time.frame_count());
    widgets::label(
        &mut ctx,
        Vec2::new(right_x + 15.0, right_y + 60.0),
        &fps_text,
        &theme.label,
    );

    let dt_text = format!("DT: {:.1}ms", time.delta_seconds() * 1000.0);
    widgets::label(
        &mut ctx,
        Vec2::new(right_x + 15.0, right_y + 80.0),
        &dt_text,
        &theme.label,
    );

    let elapsed_text = format!("Elapsed: {:.1}s", time.elapsed_seconds());
    widgets::label(
        &mut ctx,
        Vec2::new(right_x + 15.0, right_y + 100.0),
        &elapsed_text,
        &theme.label,
    );

    // Show mouse position
    let mouse_text = format!("Mouse: ({:.0}, {:.0})", ctx.cursor_pos.x, ctx.cursor_pos.y);
    widgets::label(
        &mut ctx,
        Vec2::new(right_x + 15.0, right_y + 130.0),
        &mouse_text,
        &theme.label,
    );

    // Show active widget info
    let active_text = if ctx.active.is_some() {
        "Active: YES"
    } else if ctx.hot.is_some() {
        "Active: HOT"
    } else {
        "Active: NONE"
    };
    widgets::label(
        &mut ctx,
        Vec2::new(right_x + 15.0, right_y + 150.0),
        active_text,
        &theme.label,
    );
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

struct GuiDemoPlugin;

impl Plugin for GuiDemoPlugin {
    fn build(&self, app: &mut App) {
        // Camera & clear — dark background for contrast with light theme
        app.insert_resource(Camera2D::new(800.0, 600.0));
        app.insert_resource(ClearColor(0.12, 0.12, 0.15, 1.0));

        // Render graph: clear → shapes (which includes GUI rects)
        let mut graph = RenderGraph::new();
        graph.add_node("clear", ClearPassNode);
        graph.add_node("shapes", ShapeBatchNode);
        graph.add_edge("clear", "shapes");
        app.insert_resource(graph);

        // Shape draw queue (shared between GUI and regular shapes)
        app.insert_resource(ShapeDrawQueue::new());

        // Override theme to light for better visibility on dark background
        app.insert_resource(GuiTheme::light());

        // Demo state
        app.insert_resource(GuiDemoState::default());

        // GPU startup callback
        let mut callbacks = GpuStartupCallbacks::new();
        callbacks.add(|app: &mut App| {
            let ctx = app.world.resource::<RenderContext>().unwrap();
            let format = ctx.surface_format;
            let res = ShapeBatchResources::new(&ctx.device, format);
            app.insert_resource(res);
            println!("GUI demo pipeline initialized!");
        });
        app.insert_resource(callbacks);

        // Systems: debug overlay → GUI widget logic → render bridge
        // The overlay and demo systems MUST run before the render bridge
        // so their draw commands are in frame.commands when the bridge reads them.

        // Main demo GUI (adds widget draw commands)
        app.add_system::<fn(
            ResMut<'static, GuiContext>,
            Res<'static, GuiTheme>,
            ResMut<'static, GuiDemoState>,
            Res<'static, Time>,
        ), _>(gui_demo_system);

        // Debug overlay (adds stats draw commands)
        app.add_system::<fn(
            ResMut<'static, GuiContext>,
            Res<'static, Time>,
            Res<'static, DebugOverlayConfig>,
        ), _>(debug_overlay_system);

        // Render bridge (reads ALL draw commands, converts to shapes)
        app.add_system::<fn(
            Res<'static, GuiContext>,
            Res<'static, Camera2D>,
            ResMut<'static, ShapeDrawQueue>,
            ResMut<'static, GlyphAtlas>,
        ), _>(gui_render_system);

        println!("Ember GUI Demo — Interactive panels, buttons, sliders, checkboxes");
        println!("Move mouse to interact. Click buttons, drag the slider.");
        println!("Close the window to exit.");
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let mut app = App::new();
    app.add_plugin(ember_core::plugin::CorePlugin);
    app.add_plugin(WindowPlugin {
        title: "Ember Engine — GUI Demo".to_string(),
        ..Default::default()
    });
    app.add_plugin(RenderPlugin);
    app.add_plugin(InputPlugin);
    app.add_plugin(GuiPlugin);
    app.add_plugin(GuiDemoPlugin);
    app.run();
}
