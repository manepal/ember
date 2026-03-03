# Milestone 9 — Visual Editor + Hot-Reload

**Crates:** `ember_editor`, `ember_hot_reload`
**Depends on:** Milestone 8
**Effort:** ~2-3 weeks
**Deliverable:** A native editor application where you build, test, and iterate on games in real-time.

---

## Tasks

### 9.1 Editor Shell

- [ ] Create `ember_editor` binary crate
- [ ] Set up `eframe` (egui native app framework) window
- [ ] Implement dockable panel layout system (left, center, right, bottom)
- [ ] Implement menu bar: File (new/open/save scene), Edit (undo/redo), View (toggle panels)
- [ ] Define `EditorState` resource: selected entity, active tool, editor mode (edit/play)

### 9.2 Scene Viewport

- [ ] Embed wgpu render output inside an egui panel
- [ ] Render the game world in real-time (shares render graph with the engine)
- [ ] Implement viewport controls:
  - Pan (middle-mouse drag or space + drag)
  - Zoom (scroll wheel)
  - Click to select entity (ray cast from screen to world)
- [ ] Draw selection outline around selected entity
- [ ] Draw grid overlay (toggleable)

### 9.3 Transform Gizmos

- [ ] Implement translate gizmo (drag arrows for X/Y axes)
- [ ] Implement rotate gizmo (arc handle)
- [ ] Implement scale gizmo (box handles)
- [ ] Toggle between gizmo modes via toolbar or keyboard (W/E/R)
- [ ] Snap to grid (configurable step size)
- [ ] Gizmo renders on top of scene (overlay pass)

### 9.4 Hierarchy Panel

- [ ] Display all entities in a tree view respecting parent-child hierarchy
- [ ] Click to select entity → updates inspector and viewport selection
- [ ] Drag to reorder / reparent entities
- [ ] Right-click context menu: rename, duplicate, delete, create child
- [ ] Search/filter bar

### 9.5 Inspector Panel

- [ ] Display all components on the selected entity
- [ ] Auto-generate property editors based on component type via reflection:
  - `f32` / `f64` → slider or drag field
  - `bool` → checkbox
  - `String` → text input
  - `Vec2` / `Vec3` → multi-field drag
  - `Color` → color picker
  - `Handle<T>` → asset picker (dropdown or drag-from-browser)
  - `Enum` → dropdown
- [ ] Implement `#[derive(Inspect)]` proc macro for automatic UI generation
- [ ] "Add Component" button with searchable component list
- [ ] "Remove Component" button (with confirmation)

### 9.6 Asset Browser

- [ ] Display project `assets/` folder as a file tree
- [ ] Show thumbnails for images
- [ ] Show file type icons for audio, scripts, scenes
- [ ] Double-click to open (scenes load in viewport, scripts open in external editor)
- [ ] Drag asset from browser → drop on viewport → spawn entity with appropriate components
- [ ] Refresh on file system changes

### 9.7 Console Panel

- [ ] Display structured logs from `tracing` subscriber
- [ ] Color-code by level: info (white), warn (yellow), error (red)
- [ ] Show script errors with file + line number
- [ ] Scrollable with search/filter
- [ ] Clear button
- [ ] Optional: command input line for engine commands

### 9.8 Play / Pause / Step

- [ ] **Play:** Snapshot world state → switch to play mode → run game loop normally
- [ ] **Pause:** Freeze game loop, continue rendering (can inspect entities)
- [ ] **Step:** Advance exactly one frame, then pause
- [ ] **Stop:** Restore world from snapshot → back to edit mode
- [ ] Visual indicator (toolbar color change) showing current mode
- [ ] Disable scene editing during play mode (or allow with warning)

### 9.9 Undo / Redo

- [ ] Implement command pattern: `EditorCommand` trait with `execute` and `undo`
- [ ] Commands for: move entity, change property, add/remove component, create/delete entity, reparent
- [ ] Maintain undo stack and redo stack
- [ ] Keyboard shortcuts: Cmd+Z / Cmd+Shift+Z
- [ ] Clear redo stack on new edit

### 9.10 Scene Save / Load

- [ ] Save current scene to RON file (via `ember_scene` serialization)
- [ ] Load scene from RON file → populate world
- [ ] Save dialog (file picker)
- [ ] Track "dirty" state — warn on unsaved changes before close/load
- [ ] Recent files list in File menu

### 9.11 Hot-Reload Infrastructure

- [ ] Implement `FileWatcher` using `notify` crate — watch `assets/` and `scripts/` recursively
- [ ] On file change → classify (asset vs. script vs. Rust code)
- [ ] Asset change → `AssetServer.reload(path)`
- [ ] Script change → `ScriptHost.reload(path)`
- [ ] Rust code change (stretch) → `cargo build --lib` → `libloading::Library::new()` → serialize state → swap → deserialize
- [ ] Debounce: wait ~200ms after last change before triggering reload
- [ ] Display "Reloaded: filename" toast in console

---

## Exit Criteria

- [ ] `cargo test -p ember_editor -p ember_hot_reload` — all tests pass
- [ ] Launch editor → create/place/move/delete entities in viewport
- [ ] Tweak component values in inspector → see changes in real-time
- [ ] Press Play → game runs in viewport → Pause → inspect state → Stop → restores
- [ ] Modify Lua script → saved → behavior updates live in play mode
- [ ] Save scene to RON → close → reopen → identical state
