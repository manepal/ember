use ember_core::app::App;
use ember_core::plugin::Plugin;
use std::sync::Arc;
use winit::window::Window as WinitWindow;

/// Render settings that control GPU initialization preferences.
pub struct RenderSettings {
    pub power_preference: wgpu::PowerPreference,
    pub present_mode: wgpu::PresentMode,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            power_preference: wgpu::PowerPreference::HighPerformance,
            present_mode: wgpu::PresentMode::AutoVsync,
        }
    }
}

/// GPU context resource holding wgpu primitives.
pub struct RenderContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface_format: wgpu::TextureFormat,
}

/// Plugin that ensures RenderSettings are present (actual wgpu init happens in window.rs).
pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        if app.world.resource::<RenderSettings>().is_none() {
            app.insert_resource(RenderSettings::default());
        }
    }
}

/// Callbacks that run once after wgpu initialization completes.
/// Use this to create GPU resources that need the device (textures, pipelines).
///
/// # Usage
/// ```ignore
/// let mut callbacks = GpuStartupCallbacks::new();
/// callbacks.add(|app| {
///     let ctx = app.world.resource::<RenderContext>().unwrap();
///     // create textures, pipelines, etc.
/// });
/// app.insert_resource(callbacks);
/// ```
/// A startup callback that runs after GPU initialization.
pub type GpuStartupFn = Box<dyn FnOnce(&mut App) + Send + Sync>;

pub struct GpuStartupCallbacks {
    pub callbacks: Vec<GpuStartupFn>,
}

impl Default for GpuStartupCallbacks {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuStartupCallbacks {
    pub fn new() -> Self {
        Self {
            callbacks: Vec::new(),
        }
    }

    pub fn add<F: FnOnce(&mut App) + Send + Sync + 'static>(&mut self, f: F) {
        self.callbacks.push(Box::new(f));
    }
}

/// Called by the window handler after the window is created.
/// Initializes wgpu Instance, Adapter, Device, Queue, and Surface.
pub fn initialize_wgpu(app: &mut App, window: Arc<WinitWindow>) {
    let default_settings = RenderSettings::default();
    let settings = app
        .world
        .resource::<RenderSettings>()
        .unwrap_or(&default_settings);

    let power_preference = settings.power_preference;
    let present_mode = settings.present_mode;

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let surface = instance
        .create_surface(window.clone())
        .expect("Failed to create wgpu surface");

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .expect("Failed to find a suitable GPU adapter");

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("Ember GPU Device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::default(),
        },
        None,
    ))
    .expect("Failed to create GPU device");

    let size = window.inner_size();
    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps
        .formats
        .iter()
        .find(|f| f.is_srgb())
        .copied()
        .unwrap_or(surface_caps.formats[0]);

    let surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width.max(1),
        height: size.height.max(1),
        present_mode,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &surface_config);

    app.insert_resource(RenderContext {
        device,
        queue,
        surface,
        surface_config,
        surface_format,
    });
}

/// Called on window resize to reconfigure the surface.
pub fn handle_resize(app: &mut App, width: u32, height: u32) {
    if let Some(ctx) = app.world.resource_mut::<RenderContext>() {
        ctx.surface_config.width = width.max(1);
        ctx.surface_config.height = height.max(1);
        ctx.surface.configure(&ctx.device, &ctx.surface_config);
    }
}

/// Called each frame from the window event handler. Acquires a surface texture,
/// runs the render graph, and presents.
pub fn render_frame(app: &mut App) {
    // We need to gather data from the world, then execute the render graph
    let ctx = match app.world.resource::<RenderContext>() {
        Some(ctx) => ctx as *const RenderContext,
        None => return,
    };

    // Safety: we hold a shared reference; the render graph won't mutate RenderContext
    let ctx = unsafe { &*ctx };

    let output = match ctx.surface.get_current_texture() {
        Ok(output) => output,
        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
            // Reconfigure surface on lost/outdated
            ctx.surface.configure(&ctx.device, &ctx.surface_config);
            return;
        }
        Err(wgpu::SurfaceError::OutOfMemory) => {
            panic!("GPU out of memory");
        }
        Err(e) => {
            eprintln!("Surface error: {:?}", e);
            return;
        }
    };

    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    // Run the render graph if available
    if let Some(graph) = app.world.resource::<crate::graph::RenderGraph>() {
        let graph = graph as *const crate::graph::RenderGraph;
        let graph = unsafe { &*graph };
        graph.execute(ctx, &view, &app.world);
    }

    output.present();
}
