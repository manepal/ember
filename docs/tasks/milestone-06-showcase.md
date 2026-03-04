# Milestone 6 — Ember Showcase

**Crate:** `ember_showcase` (top-level example or standalone crate)
**Depends on:** Milestone 5 (GUI)
**Effort:** ~3-4 days
**Deliverable:** A single-window demo launcher that acts as a gallery for all engine demos. Dogfoods the `ember_gui` system and serves as the engine's showpiece.

---

## Tasks

### 6.1 Scaffold Showcase Application

- [ ] Create top-level `examples/ember_showcase.rs` (or `crates/ember_showcase`)
- [ ] Set up window, render pipeline, GUI plugin, and input plugin
- [ ] Define `DemoEntry` trait: `name()`, `description()`, `setup(app)`, `teardown(app)`

### 6.2 Demo Registry & Menu UI

- [ ] Implement sidebar/menu listing all available demos (Input, Animation, Assets, Scene, GUI, Shapes, etc.)
- [ ] Show a description panel for the selected demo before launching
- [ ] Each demo registers itself via the `DemoEntry` trait

### 6.3 Demo State Machine

- [ ] Implement demo load/unload state machine — clean scene transitions when switching
- [ ] Back button / ESC returns to the demo menu from any running demo
- [ ] Hot-key support (number keys 1–9 for quick demo switching)

### 6.4 Chrome & Polish

- [ ] Display engine version, FPS counter, and active demo name in a top bar
- [ ] Use `ember_gui` theming for the menu layout and demo descriptions
- [ ] Ensure smooth transitions between demos

---

## Exit Criteria

- [ ] `ember_showcase` opens a single window where all demos are browsable
- [ ] Selecting a demo loads and runs it within the same window
- [ ] ESC / back button returns to the demo list
- [ ] FPS and engine version displayed in the top bar
- [ ] At least 4 demos are registered and launchable
