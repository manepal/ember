# Architecture Overview

Ember Engine is built as a **Cargo workspace** of independent crates, each owning a single subsystem. Everything is a plugin — the engine is assembled at startup by composing plugins.

## Table of Contents

- [Design Principles](#design-principles)
- [Workspace Layout](#workspace-layout)
- [System Architecture Diagram](#system-architecture-diagram)
- [Core Systems](#core-systems)
- [Rendering Pipeline](#rendering-pipeline)
- [Data Flow](#data-flow)

---

## Design Principles

| Principle | Detail |
|---|---|
| **Plugin-first** | Every subsystem (renderer, input, audio, physics, scripting, AI) is a plugin. Users extend the engine the same way. |
| **Data-driven ECS** | Archetypal storage, parallel system scheduling, typed queries — composition over inheritance. |
| **Dimension-agnostic rendering** | Render graph over wgpu. 2D ships first; 2.5D and 3D plug into the same graph. |
| **Polyglot scripting** | `ScriptHost` trait with swappable backends (Lua, Python, JS). |
| **Hot-reloadable everything** | Assets, scripts, and game code — all hot-reloadable via file watcher + dynamic linking. |
| **Visual editor** | Native egui editor with viewport, inspector, and play-in-editor. |

---

## Workspace Layout

```
ember-engine/
├── crates/
│   ├── ember_core/          # ECS, App builder, Plugin trait, Events, Time, Scheduler
│   ├── ember_render/        # Render graph, wgpu backend, Camera
│   ├── ember_2d/            # Sprite, TextureAtlas, TileMap, 2D render pipeline
│   ├── ember_input/         # Keyboard, Mouse, Gamepad state
│   ├── ember_assets/        # AssetServer, Handle<T>, loaders, hot-reload
│   ├── ember_audio/         # Sound effects & music (rodio)
│   ├── ember_animation/     # Sprite sheets, tweening, animation state machines
│   ├── ember_lighting/      # 2D point/spot/ambient lights, shadow casting
│   ├── ember_particles/     # GPU compute shader particle system
│   ├── ember_physics/       # AABB/circle colliders, spatial hash, rigid body
│   ├── ember_ai/            # Behavior trees, FSM, pathfinding, steering
│   ├── ember_script/        # ScriptHost trait, ScriptComponent, ScriptValue
│   ├── ember_script_lua/    # Lua backend (mlua)
│   ├── ember_script_python/ # Python backend (pyo3) [stretch]
│   ├── ember_script_js/     # JS/TS backend (deno_core) [stretch]
│   ├── ember_scene/         # Scene graph, RON serialization, prefabs
│   ├── ember_editor/        # Visual editor (egui + eframe)
│   ├── ember_hot_reload/    # File watcher (notify) + dylib reloading
│   ├── ember_mcp/           # MCP server — AI agent interface [stretch]
│   ├── ember_25d/           # Depth layers, parallax, isometric [stretch]
│   └── ember_3d/            # Meshes, materials, PBR, glTF [stretch]
├── examples/
├── assets/
└── docs/
```

### Dependency Graph (crates)

```
ember_core ─────────────────────────────────────────────────┐
    │                                                        │
    ├── ember_render (depends on: core)                      │
    │       ├── ember_2d (depends on: render, core)          │
    │       ├── ember_lighting (depends on: render, core)    │
    │       └── ember_particles (depends on: render, core)   │
    │                                                        │
    ├── ember_input (depends on: core)                       │
    ├── ember_assets (depends on: core)                      │
    ├── ember_audio (depends on: core, assets)               │
    ├── ember_animation (depends on: core, 2d)               │
    ├── ember_physics (depends on: core)                     │
    ├── ember_ai (depends on: core, physics)                 │
    ├── ember_scene (depends on: core)                       │
    │                                                        │
    ├── ember_script (depends on: core)                      │
    │       ├── ember_script_lua (depends on: script)        │
    │       ├── ember_script_python (depends on: script)     │
    │       └── ember_script_js (depends on: script)         │
    │                                                        │
    ├── ember_hot_reload (depends on: core, assets, script)  │
    ├── ember_mcp (depends on: core, scene, script, editor)   │
    └── ember_editor (depends on: all above)  ───────────────┘
```

---

## System Architecture Diagram

```
┌─────────────┐
│  AI Agents  │ (Claude, GPT, Copilot, etc.)
│  via MCP    │
└──────┬──────┘
       │ JSON-RPC 2.0 (stdio / HTTP+SSE)
       │
┌──────▼──────────────────────────────────────────────────────────┐
│                    ember_mcp Server                             │
│  Tools: spawn, move, query, run_system, save_scene, ...        │
│  Resources: scene_tree, entity_info, asset_list, logs          │
└──────┬──────────────────────────────────────────────────────────┘
       │
┌──────▼──────────────────────────────────────────────────────────┐
│                        Ember Editor                             │
│  ┌──────────┐ ┌──────────────────┐ ┌───────────┐ ┌──────────┐  │
│  │Hierarchy │ │  Scene Viewport  │ │ Inspector │ │  Console │  │
│  │  Panel   │ │  (live wgpu)     │ │  Panel    │ │  Panel   │  │
│  └────┬─────┘ └────────┬─────────┘ └─────┬─────┘ └──────────┘  │
│       │                │                  │                     │
│  ┌────▼────────────────▼──────────────────▼─────────────────┐   │
│  │              Play / Pause / Step Controls                │   │
│  └──────────────────────────┬───────────────────────────────┘   │
└─────────────────────────────┼───────────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────────┐
│                         App Builder                             │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    Plugin Registry                       │   │
│  │  CorePlugin │ RenderPlugin │ InputPlugin │ GamePlugin …  │   │
│  └──────────────────────────┬───────────────────────────────┘   │
│                              │                                  │
│  ┌──────────────────────────▼───────────────────────────────┐   │
│  │                  System Scheduler                        │   │
│  │  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐          │   │
│  │  │Input │ │AI    │ │Phys  │ │Anim  │ │Audio │  ...      │   │
│  │  │System│ │System│ │System│ │System│ │System│           │   │
│  │  └──┬───┘ └──┬───┘ └──┬───┘ └──┬───┘ └──┬───┘          │   │
│  └──────┼───────┼────────┼────────┼────────┼───────────────┘   │
│         │       │        │        │        │                    │
│  ┌──────▼───────▼────────▼────────▼────────▼───────────────┐   │
│  │                     ECS World                           │   │
│  │  Entities │ Components (Archetypes) │ Resources │ Events │   │
│  └──────────────────────────┬──────────────────────────────┘   │
│                              │                                  │
│  ┌──────────────────────────▼───────────────────────────────┐   │
│  │                   Render Graph                           │   │
│  │  Clear → Sprites → Lighting → Particles → UI → Present  │   │
│  └──────────────────────────┬───────────────────────────────┘   │
│                              │                                  │
│  ┌──────────────────────────▼───────────────────────────────┐   │
│  │                    wgpu Backend                          │   │
│  │              Metal │ Vulkan │ DX12 │ WebGPU              │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Core Systems

### ECS (ember_core)

**Archetypal storage** — entities with the same component set share a contiguous table.

```rust
// Entity — generational index
pub struct Entity { id: u32, generation: u32 }

// World — entities, components (archetypes), resources, events
let player = world.spawn()
    .insert(Transform2D::new(100.0, 200.0))
    .insert(Sprite { texture: handle, .. })
    .insert(Velocity(Vec2::ZERO))
    .id();

// Systems — plain functions with typed parameters
fn movement(query: Query<(&mut Transform2D, &Velocity)>, time: Res<Time>) {
    for (mut tf, vel) in query.iter_mut() {
        tf.position += vel.0 * time.delta_secs();
    }
}
```

**Resources** — singletons stored in the World (e.g. `Time`, `InputState`, `AssetServer`).

**System scheduler** — topological sort by declared read/write access; non-conflicting systems run in parallel via rayon.

### Plugin System

Every subsystem is a `Plugin`:

```rust
pub trait Plugin {
    fn build(&self, app: &mut App);
}

App::new()
    .add_plugin(CorePlugin)
    .add_plugin(RenderPlugin)
    .add_plugin(Render2DPlugin)
    .add_plugin(InputPlugin)
    .add_plugin(PhysicsPlugin)
    .add_plugin(LuaScriptPlugin)
    .add_plugin(MyGamePlugin)   // user's game
    .run();
```

### Event Bus

Typed, double-buffered event channels. Write events in frame N, read in frame N+1:

```rust
// Send
events.send(CollisionEvent { a: entity_a, b: entity_b });

// Receive
fn on_collision(events: EventReader<CollisionEvent>) {
    for event in events.iter() { /* ... */ }
}
```

---

## Rendering Pipeline

### Render Graph

A DAG of self-contained render passes. Nodes declare input/output texture slots; the graph resolves execution order at startup.

```
┌───────────┐   ┌──────────────┐   ┌──────────────┐   ┌──────────┐   ┌─────────┐
│ Clear Pass│──▶│ Sprite Pass  │──▶│ Light Mask   │──▶│ Particle │──▶│ UI Pass │
│           │   │ (batched)    │   │ (compute)    │   │ Pass     │   │ (egui)  │
└───────────┘   └──────────────┘   └──────────────┘   └──────────┘   └─────────┘
```

Adding 2.5D or 3D = inserting new pass nodes into the graph. No core changes needed.

### 2D → 2.5D → 3D Growth Path

| Dimension | Adds | Render Graph Change |
|---|---|---|
| **2D** | Sprite, TextureAtlas, TileMap, Camera2D, batched quads | Sprite pass node |
| **2.5D** | DepthLayer, Parallax, IsometricTransform, Z-sorting | Depth-sorted sprite pass |
| **3D** | Mesh, Material, PointLight, Camera3D, glTF, depth buffer | Geometry + lighting pass nodes |

---

## Data Flow

### Game Loop (Fixed Timestep)

```
accumulator += frame_time
while accumulator >= FIXED_DT:
    run_fixed_update_systems(FIXED_DT)   // physics, AI, game logic
    accumulator -= FIXED_DT
alpha = accumulator / FIXED_DT
run_render_systems(alpha)                // interpolate for smooth visuals
```

### Hot-Reload Flow

```
File changed (notify) → Hot Reload Manager → Asset/Script reloaded
  .lua/.py/.js → ScriptHost.reload()
  .png/.wav    → AssetServer.reload()
  .rs (dylib)  → serialize state → unload old → cargo build → load new → deserialize
```

### Editor ↔ Engine Communication

```
Edit Mode:   Editor panels ←→ ECS World (direct read/write)
Play Mode:   Snapshot world → run game loop → Pause/Stop → restore snapshot
Step Mode:   Advance exactly one frame, then pause
```
