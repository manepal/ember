# Milestone 13 — Stretch Goals

**Crates:** `ember_25d`, `ember_3d`, `ember_script_python`, `ember_script_js`, and extensions to existing crates
**Depends on:** Milestone 12 (AI + MCP)
**Effort:** Ongoing
**Status:** These are picked up as the core engine stabilizes.

---

## 13A — 2.5D Rendering

**Crate:** `ember_25d`

- [ ] Implement `DepthLayer` component — Z-depth for sprite sorting
- [ ] Implement depth-sorted sprite pass (replace flat sprite pass with Z-aware version)
- [ ] Implement `ParallaxLayer` — multiple background layers scrolling at different rates
- [ ] Implement `IsometricTransform` — isometric projection math
- [ ] Implement isometric tile map rendering
- [ ] Example: isometric scene with parallax background

## 13B — 3D Rendering

**Crate:** `ember_3d`

- [ ] Implement `Mesh` component and vertex buffer management
- [ ] Implement `Material` component — base color, PBR metallic/roughness
- [ ] Implement `Camera3D` — perspective projection, orbit controls
- [ ] Implement basic 3D render pipeline (geometry pass + forward lighting)
- [ ] Implement `PointLight3D`, `DirectionalLight`, `SpotLight3D`
- [ ] Implement shadow mapping (directional light shadow maps)
- [ ] Implement glTF model loader
- [ ] Implement `Transform3D` using `glam::Affine3A`
- [ ] Example: loaded 3D model with lighting

## 13C — Skeletal Animation

**Crate:** `ember_animation` (extension)

- [ ] Implement `Skeleton2D` — bone hierarchy with parent indices
- [ ] Implement `Bone2D` — name, parent, local transform, length
- [ ] Implement `SkeletalClip` — keyframed bone transforms
- [ ] Implement skeletal animation playback — interpolate keyframes, compute world transforms
- [ ] Implement mesh skinning — vertex weights bound to bones
- [ ] Support blending between skeletal clips
- [ ] Import from Spine or Aseprite format (stretch)

## 13D — Python Scripting

**Crate:** `ember_script_python`

- [ ] Implement `PythonScriptHost` via `pyo3`
- [ ] Initialize Python interpreter embedded in engine
- [ ] Expose engine APIs: same surface as Lua (get/set components, input, spawn/despawn)
- [ ] Handle Python ↔ Rust type marshaling via `ScriptValue`
- [ ] Implement `PythonScriptPlugin`
- [ ] Test hot-reload of Python scripts

## 13E — JavaScript Scripting

**Crate:** `ember_script_js`

- [ ] Implement `JSScriptHost` via `deno_core`
- [ ] Initialize V8 runtime with `JsRuntime`
- [ ] Register engine APIs as "ops" via `deno_core::extension!`
- [ ] Handle JS ↔ Rust type marshaling via `ScriptValue`
- [ ] Wire event loop (`run_event_loop` via tokio)
- [ ] Implement `JSScriptPlugin`
- [ ] Support TypeScript (via deno's built-in transpiler)
- [ ] Test hot-reload of JS/TS scripts

## 13F — Networking / Multiplayer

**Crate:** `ember_net` (new)

- [ ] Define network architecture: client-server vs. peer-to-peer
- [ ] Implement `NetPlugin` with connection management
- [ ] Implement component replication — mark components as `#[replicated]`
- [ ] Implement snapshot interpolation for smooth remote entities
- [ ] Implement input prediction for responsive local play
- [ ] UDP transport via `laminar` or `quinn` (QUIC)

## 13G — Tilemap System

**Crate:** `ember_2d` (extension)

- [ ] Implement `TileMap` component — grid-based tile storage
- [ ] Implement `TileMapRenderer` — efficient chunked rendering
- [ ] Implement auto-tiling (rule-based tile selection)
- [ ] Load tile maps from Tiled (.tmx) format
- [ ] Collision shape generation from tile map layers

## 13H — Editor Enhancements

**Crate:** `ember_editor` (extension)

- [ ] Tile map editor mode — paint tiles visually
- [ ] Particle emitter editor — visual tuning with real-time preview
- [ ] Animation editor — timeline view for sprite/skeletal clips
- [ ] Behavior tree editor — visual node graph
- [ ] Build/export — package game as standalone binary
