# Milestone 5 — Immediate-Mode GUI

**Crate:** `ember_gui`
**Depends on:** Milestone 4 (Input + Assets)
**Effort:** ~1.5 weeks
**Deliverable:** In-game UI system with panels, buttons, text, sliders, and a layout engine — all rendered through `ember_2d`'s shape/sprite pipeline.

> [!NOTE]
> `ember_gui` is the engine's **in-game UI system** for game developers to build HUDs, menus, and overlays. The Phase 10 **Visual Editor** uses `egui`/`eframe` for its own developer-facing UI. These are intentionally separate — game UI needs to render through the engine's own pipeline and be themeable, while the editor uses a mature toolkit for rapid development.

---

## Tasks

### 5.1 Scaffold `ember_gui` Crate

- [ ] Create `crates/ember_gui` with dependencies on `ember_core`, `ember_2d`, `ember_input`, `ember_assets`
- [ ] Register in workspace `Cargo.toml`
- [ ] Create `GuiPlugin` that registers all GUI systems and resources

### 5.2 GUI Context & State

- [ ] Implement `GuiContext` resource — tracks hot/active widget IDs, focus, cursor position
- [ ] Implement `WidgetId` — unique identifier per widget (hash of label + parent chain)
- [ ] Implement hit-testing — point-in-rect for mouse interaction
- [ ] Track hover, press, click, drag states per widget

### 5.3 Layout Engine

- [ ] Implement `Layout` trait with `calculate(constraints) → Size` method
- [ ] Implement `VerticalLayout` — stacks children top-to-bottom with spacing
- [ ] Implement `HorizontalLayout` — stacks children left-to-right with spacing
- [ ] Implement `Padding` / `Margin` modifiers
- [ ] Implement `Anchor` positioning — screen-relative anchoring (top-left, center, etc.)
- [ ] Layout caching — only recalculate when inputs change

### 5.4 Core Widgets

- [ ] **Panel** — rectangular container with optional background color/texture, border, and corner radius
- [ ] **Label** — text rendering with font size, color, alignment
- [ ] **Button** — clickable rectangle with hover/pressed visual states, returns `bool` on click
- [ ] **ImageWidget** — displays a texture or sprite
- [ ] **Checkbox** — toggleable boolean with visual indicator
- [ ] **Slider** — horizontal/vertical drag for float values with range
- [ ] **TextInput** — editable single-line text field with cursor and selection
- [ ] **ProgressBar** — filled rectangle showing 0.0–1.0 progress

### 5.5 Text Rendering

- [ ] Implement `Font` asset type (bitmap font atlas or SDF font)
- [ ] Implement `FontLoader` (loads `.ttf`/`.otf` → rasterized glyph atlas via `fontdue` or `ab_glyph`)
- [ ] Implement `TextRenderer` — produces quads for each glyph, aligned and wrapped
- [ ] Support basic text layout: left/center/right alignment, word wrap, line height

### 5.6 Theming & Styling

- [ ] Implement `GuiTheme` resource — colors, spacing, font sizes, border widths
- [ ] Provide a default dark theme and a default light theme
- [ ] Allow per-widget style overrides via builder pattern
- [ ] Support color transitions on hover/press for interactive widgets

### 5.7 Rendering Integration

- [ ] Implement `GuiRenderNode` — collects all GUI draw commands and renders as an overlay
- [ ] Ensure GUI renders on top of the game world (high z-order / separate pass)
- [ ] Batch GUI quads efficiently (single draw call for all text, single for all rects)
- [ ] Support transparency and blending for overlays

### 5.8 Debug Overlay

A built-in debug overlay rendered via `ember_gui`, available for non-editor builds (toggled at runtime).

- [ ] **FPS Counter** — frames per second and frame time (ms)
- [ ] **Entity Stats** — total entities, archetypes, components
- [ ] **Render Stats** — draw calls, vertex count, texture binds
- [ ] **System Profiler** — per-system execution time (bar chart)
- [ ] **Wireframe Colliders** — optional wireframe rendering of physics colliders and bounding boxes
- [ ] Toggle via `F3` key or `DebugOverlay::toggle()` API
- [ ] Configurable via `DebugOverlayConfig` resource (choose which stats to show)

### 5.9 Demo & Testing

- [ ] Create `gui_demo` example — showcases all widgets: buttons, sliders, text, panels
- [ ] Unit tests for layout calculations
- [ ] Unit tests for hit-testing and widget state transitions
- [ ] Unit tests for text measurement and wrapping

---

## Exit Criteria

- [ ] `cargo test -p ember_gui` — all tests pass
- [ ] `gui_demo` opens a window with interactive UI elements
- [ ] Buttons respond to hover and click
- [ ] Sliders return correct float values when dragged
- [ ] Text renders correctly with word wrapping
- [ ] GUI renders as overlay on top of game content
- [ ] Debug overlay shows FPS and entity stats when toggled

