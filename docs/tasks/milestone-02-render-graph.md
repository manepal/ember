# Milestone 2 — Window + Render Graph + wgpu

**Crates:** `ember_render`
**Depends on:** Milestone 1
**Effort:** ~1 week
**Deliverable:** A window opens, clears to a color via a render graph backed by wgpu.

---

## Tasks

### 2.1 Window Management

- [ ] Create `WindowPlugin` wrapping `winit`
- [ ] Implement `Window` resource (handle, size, title, vsync, fullscreen flag)
- [ ] Bridge `winit` event loop with the engine's `App::run()` loop
- [ ] Handle resize events — update surface configuration
- [ ] Handle close event — clean shutdown

### 2.2 wgpu Initialization

- [ ] Create `RenderPlugin` that initializes wgpu on startup
- [ ] Implement `RenderContext` resource:
  - `Instance`, `Adapter`, `Device`, `Queue`
  - `Surface`, `SurfaceConfiguration`
- [ ] Handle device loss and recovery
- [ ] Implement `RenderSettings` resource (present mode, power preference, sample count)

### 2.3 Render Graph Framework

- [ ] Define `RenderGraph` struct — DAG of `RenderNode`s
- [ ] Define `RenderNode` trait:
  - `fn input_slots() -> Vec<SlotDescriptor>`
  - `fn output_slots() -> Vec<SlotDescriptor>`
  - `fn run(&self, context: &mut RenderContext, world: &World)`
- [ ] Implement slot types: `TextureSlot`, `BufferSlot`
- [ ] Implement graph builder: `graph.add_node("name", node)`, `graph.add_edge("from", "to")`
- [ ] Implement graph resolution — topological sort, validate no cycles, resolve slot connections
- [ ] Implement graph executor — runs nodes in order, passes resources between them

### 2.4 Clear Pass Node

- [ ] Implement `ClearPassNode` — clears the swap chain texture to a configurable color
- [ ] Register as the first node in the default render graph
- [ ] Add `ClearColor` resource to control the background color

### 2.5 Frame Presentation

- [ ] Implement frame lifecycle: acquire surface texture → run render graph → present
- [ ] Implement `RenderSystem` that executes the render graph each frame
- [ ] Wire into the app's game loop (runs after all update systems)

### 2.6 Camera Foundation

- [ ] Implement `Camera2D` component: position, zoom, viewport rect
- [ ] Implement orthographic projection matrix computation
- [ ] Create camera uniform buffer (view-projection matrix)
- [ ] Upload camera uniforms to GPU each frame

---

## Exit Criteria

- [ ] `cargo test -p ember_render` — all tests pass
- [ ] `cargo clippy -p ember_render -- -D warnings` — clean
- [ ] `examples/hello_window` — window opens, shows solid color, resizes correctly, closes cleanly

## Example

```rust
// examples/hello_window/main.rs
fn main() {
    App::new()
        .add_plugin(CorePlugin)
        .add_plugin(WindowPlugin { title: "Ember Engine", width: 1280, height: 720 })
        .add_plugin(RenderPlugin)
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.15)))
        .run();
}
```
