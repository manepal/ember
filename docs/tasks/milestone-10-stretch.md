# Milestone 10 — Stretch Goals

**Crates:** `ember_25d`, `ember_3d`, `ember_script_python`, `ember_script_js`, and extensions to existing crates
**Depends on:** Milestone 9
**Effort:** Ongoing
**Status:** These are picked up as the core engine stabilizes.

---

## 10A — 2.5D Rendering

**Crate:** `ember_25d`

- [ ] Implement `DepthLayer` component — Z-depth for sprite sorting
- [ ] Implement depth-sorted sprite pass (replace flat sprite pass with Z-aware version)
- [ ] Implement `ParallaxLayer` — multiple background layers scrolling at different rates
- [ ] Implement `IsometricTransform` — isometric projection math
- [ ] Implement isometric tile map rendering
- [ ] Example: isometric scene with parallax background

## 10B — 3D Rendering

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

## 10C — Skeletal Animation

**Crate:** `ember_animation` (extension)

- [ ] Implement `Skeleton2D` — bone hierarchy with parent indices
- [ ] Implement `Bone2D` — name, parent, local transform, length
- [ ] Implement `SkeletalClip` — keyframed bone transforms
- [ ] Implement skeletal animation playback — interpolate keyframes, compute world transforms
- [ ] Implement mesh skinning — vertex weights bound to bones
- [ ] Support blending between skeletal clips
- [ ] Import from Spine or Aseprite format (stretch)

## 10D — Python Scripting

**Crate:** `ember_script_python`

- [ ] Implement `PythonScriptHost` via `pyo3`
- [ ] Initialize Python interpreter embedded in engine
- [ ] Expose engine APIs: same surface as Lua (get/set components, input, spawn/despawn)
- [ ] Handle Python ↔ Rust type marshaling via `ScriptValue`
- [ ] Implement `PythonScriptPlugin`
- [ ] Test hot-reload of Python scripts

## 10E — JavaScript Scripting

**Crate:** `ember_script_js`

- [ ] Implement `JSScriptHost` via `deno_core`
- [ ] Initialize V8 runtime with `JsRuntime`
- [ ] Register engine APIs as "ops" via `deno_core::extension!`
- [ ] Handle JS ↔ Rust type marshaling via `ScriptValue`
- [ ] Wire event loop (`run_event_loop` via tokio)
- [ ] Implement `JSScriptPlugin`
- [ ] Support TypeScript (via deno's built-in transpiler)
- [ ] Test hot-reload of JS/TS scripts

## 10F — Networking / Multiplayer

**Crate:** `ember_net` (new)

- [ ] Define network architecture: client-server vs. peer-to-peer
- [ ] Implement `NetPlugin` with connection management
- [ ] Implement component replication — mark components as `#[replicated]`
- [ ] Implement snapshot interpolation for smooth remote entities
- [ ] Implement input prediction for responsive local play
- [ ] UDP transport via `laminar` or `quinn` (QUIC)

## 10G — Tilemap System

**Crate:** `ember_2d` (extension)

- [ ] Implement `TileMap` component — grid-based tile storage
- [ ] Implement `TileMapRenderer` — efficient chunked rendering
- [ ] Implement auto-tiling (rule-based tile selection)
- [ ] Load tile maps from Tiled (.tmx) format
- [ ] Collision shape generation from tile map layers

## 10H — Editor Enhancements

**Crate:** `ember_editor` (extension)

- [ ] Tile map editor mode — paint tiles visually
- [ ] Particle emitter editor — visual tuning with real-time preview
- [ ] Animation editor — timeline view for sprite/skeletal clips
- [ ] Behavior tree editor — visual node graph
- [ ] Build/export — package game as standalone binary

## 10I — MCP Server (AI Agent Interface)

**Crate:** `ember_mcp` (new)

An MCP (Model Context Protocol) server that exposes the engine to AI agents (Claude, GPT, Copilot, etc.), enabling AI-assisted game development.

### MCP Tools (Actions)

AI agents can invoke these to modify the game:

- [ ] `spawn_entity` — create an entity with specified components
- [ ] `despawn_entity` — remove an entity by ID
- [ ] `set_component` — add or modify a component on an entity
- [ ] `remove_component` — remove a component from an entity
- [ ] `move_entity` — set entity position/rotation/scale
- [ ] `query_entities` — find entities matching a component filter
- [ ] `execute_system` — run a named system once
- [ ] `save_scene` — serialize current scene to RON file
- [ ] `load_scene` — load a scene from RON file
- [ ] `create_script` — write a Lua/Python/JS script file
- [ ] `reload_asset` — force hot-reload of a specific asset
- [ ] `set_resource` — modify an engine resource (e.g. ClearColor, gravity)
- [ ] `play` / `pause` / `stop` — control editor play mode
- [ ] `take_screenshot` — capture viewport as image
- [ ] `get_viewport_screenshot` — return viewport image for visual feedback

### MCP Resources (Read-Only Data)

AI agents can read these to understand the game state:

- [ ] `scene://entities` — full entity hierarchy with component data
- [ ] `scene://entity/{id}` — detailed info for a specific entity
- [ ] `assets://list` — all loaded assets with types and paths
- [ ] `assets://info/{path}` — metadata for a specific asset
- [ ] `engine://systems` — registered systems and their schedules
- [ ] `engine://plugins` — active plugins
- [ ] `engine://logs` — recent engine log output
- [ ] `engine://stats` — FPS, entity count, draw calls, memory
- [ ] `scripts://list` — all loaded scripts with backends
- [ ] `physics://colliders` — all colliders and their state

### MCP Prompts (Guided Workflows)

- [ ] `create_game_object` — guided prompt for spawning a configured entity
- [ ] `setup_scene` — guided prompt for building a scene from description
- [ ] `debug_entity` — guided prompt for inspecting and fixing entity issues

### Transport & Architecture

- [ ] Implement MCP server using `rmcp` crate
- [ ] Support stdio transport (for local AI tool integration)
- [ ] Support HTTP+SSE transport (for remote/web AI clients)
- [ ] Implement JSON-RPC 2.0 message handling
- [ ] Bridge MCP tool calls → ECS world mutations (thread-safe channel)
- [ ] Bridge MCP resource reads → ECS world queries (read-only snapshot)
- [ ] Implement `McpPlugin` for registration into the App
- [ ] Config file for enabling/disabling MCP, port, allowed tools
- [ ] Security: sandboxing, rate limiting, tool whitelisting

### Example AI Interaction

```
AI Agent: "Create a player entity at position (100, 200) with a sprite and physics"
    ↓ MCP tool call: spawn_entity
    ↓ components: [Transform2D(100, 200), Sprite("player.png"), RigidBody, Collider2D::AABB(16, 16)]
    ↓ Response: { entity_id: 42, components: [...] }

AI Agent: "What entities are in the scene?"
    ↓ MCP resource read: scene://entities
    ↓ Response: [{ id: 42, name: "Player", components: [...] }, ...]

AI Agent: "Take a screenshot of the current viewport"
    ↓ MCP tool call: get_viewport_screenshot
    ↓ Response: { image: <base64 png> }
```
