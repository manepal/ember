use ember_core::app::App;
use ember_core::plugin::Plugin;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window as WinitWindow, WindowId};

/// Configuration for the window, set before app launch.
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Ember Engine".to_string(),
            width: 1280,
            height: 720,
            vsync: true,
        }
    }
}

/// Resource that holds a reference to the winit window after creation.
pub struct EmberWindow {
    pub window: Arc<WinitWindow>,
    pub width: u32,
    pub height: u32,
}

/// Plugin that creates the window and sets up the winit event loop runner.
pub struct WindowPlugin {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

impl Default for WindowPlugin {
    fn default() -> Self {
        Self {
            title: "Ember Engine".to_string(),
            width: 1280,
            height: 720,
        }
    }
}

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WindowConfig {
            title: self.title.clone(),
            width: self.width,
            height: self.height,
            vsync: true,
        });

        app.set_runner(winit_runner);
    }
}

/// The winit runner replaces the default game loop.
/// It creates the event loop and hands control to winit.
fn winit_runner(app: App) {
    let event_loop = EventLoop::new().expect("Failed to create winit event loop");
    let mut handler = EmberAppHandler {
        app,
        initialized: false,
    };
    event_loop.run_app(&mut handler).expect("Event loop failed");
}

struct EmberAppHandler {
    app: App,
    initialized: bool,
}

impl ApplicationHandler for EmberAppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.initialized {
            return;
        }
        self.initialized = true;

        let config = self
            .app
            .world
            .resource::<WindowConfig>()
            .expect("WindowConfig resource missing");

        let title = config.title.clone();
        let width = config.width;
        let height = config.height;

        let window_attrs = WinitWindow::default_attributes()
            .with_title(&title)
            .with_inner_size(PhysicalSize::new(width, height));

        let window = Arc::new(
            event_loop
                .create_window(window_attrs)
                .expect("Failed to create window"),
        );

        self.app.insert_resource(EmberWindow {
            window: window.clone(),
            width,
            height,
        });

        // Initialize wgpu if RenderPlugin was added
        crate::context::initialize_wgpu(&mut self.app, window.clone());
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if new_size.width > 0 && new_size.height > 0 {
                    // Update the window resource
                    if let Some(win) = self.app.world.resource_mut::<EmberWindow>() {
                        win.width = new_size.width;
                        win.height = new_size.height;
                    }
                    // Reconfigure the surface
                    crate::context::handle_resize(&mut self.app, new_size.width, new_size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                // Run one ECS tick
                self.app.update();

                // Execute the render graph and present
                crate::context::render_frame(&mut self.app);

                // Request the next frame
                if let Some(ember_win) = self.app.world.resource::<EmberWindow>() {
                    ember_win.window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Request a redraw to keep the loop going
        if let Some(ember_win) = self.app.world.resource::<EmberWindow>() {
            ember_win.window.request_redraw();
        }
    }
}
