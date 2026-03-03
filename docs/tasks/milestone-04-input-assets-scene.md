# Milestone 4 — Input + Assets + Scene

**Crates:** `ember_input`, `ember_assets`, `ember_scene`
**Depends on:** Milestone 3
**Effort:** ~1 week
**Deliverable:** Keyboard/mouse input drives on-screen sprites; assets load asynchronously; scenes save/load to RON.

---

## Tasks

### 4.1 Keyboard Input

- [ ] Implement `KeyboardState` resource: pressed, just_pressed, just_released per key
- [ ] Map `winit::event::KeyEvent` → engine key codes
- [ ] Implement state update system (runs at start of frame)
- [ ] Unit tests: press → hold → release state transitions

### 4.2 Mouse Input

- [ ] Implement `MouseState` resource: position, delta, buttons (pressed/just_pressed/just_released), scroll
- [ ] Convert screen coords → world coords using camera
- [ ] Unit tests: button states, coordinate conversion

### 4.3 Gamepad Input (Stretch)

- [ ] Implement `GamepadState` resource: buttons, axes, deadzones
- [ ] Support connect/disconnect events

### 4.4 InputPlugin

- [ ] Create `InputPlugin` that registers keyboard, mouse, and gamepad systems
- [ ] Expose `is_key_pressed(KeyCode)`, `is_mouse_button_pressed(MouseButton)` convenience methods

### 4.5 Asset Server

- [ ] Implement `AssetServer` resource:
  - `load::<T>(path)` → `Handle<T>` (typed, ref-counted)
  - Async I/O (background thread or async task)
  - Caching — same path returns same handle
- [ ] Implement `Handle<T>` (strong, ref-counted) and `HandleWeak<T>` (observer)
- [ ] Implement `AssetEvent<T>` — `Created`, `Modified`, `Removed`
- [ ] Handle loading states: `NotLoaded`, `Loading`, `Loaded`, `Error`

### 4.6 Asset Loaders

- [ ] Define `AssetLoader` trait: `fn load(bytes: &[u8]) -> Result<T>`
- [ ] Implement `ImageLoader` — loads PNG/JPEG → `Texture`
- [ ] Implement `AudioLoader` — loads WAV/OGG → `AudioSource`
- [ ] Implement `SceneLoader` — loads RON → `Scene`
- [ ] Register loaders by file extension
- [ ] Unit tests: loader dispatch, cache hit/miss, handle lifecycle

### 4.7 Scene Graph

- [ ] Implement `Parent` / `Children` components for entity hierarchy
- [ ] Implement transform propagation — child transforms inherit parent's
- [ ] Implement `SceneBundle` — spawn an entity tree from a scene definition

### 4.8 Scene Serialization

- [ ] Implement `Scene` struct — list of entity descriptors with component data
- [ ] Serialize to RON via `serde`
- [ ] Deserialize from RON → spawn entities in world
- [ ] Implement `Prefab` — reusable scene templates
- [ ] Support entity references within a scene (relative IDs)

---

## Exit Criteria

- [ ] `cargo test -p ember_input -p ember_assets -p ember_scene` — all tests pass
- [ ] Arrow keys move a sprite on screen
- [ ] Sprites load from asset files (not embedded)
- [ ] A scene saves to `.ron`, reloads identically
