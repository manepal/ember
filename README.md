# Ember Engine

A production-grade game engine built in Rust, starting 2D and designed to grow into 2.5D and 3D. Features a visual editor for real-time game building, plugin-first architecture, polyglot scripting, GPU-accelerated lighting and particles, and an ECS core.

---

## ✨ Key Features

- **Plugin-first architecture** — every subsystem is a plugin; the engine is just the sum of its plugins
- **Custom ECS** — archetypal storage, parallel scheduling, typed queries
- **Render graph** — dimension-agnostic pipeline over wgpu (Metal / Vulkan / DX12 / WebGPU)
- **2D lighting** — point, spot, ambient lights with compute-shader shadow casting
- **GPU particles** — compute shader pipeline supporting millions of particles
- **Animation** — sprite sheets, tweening with easing, animation state machines
- **Game AI** — behavior trees, FSMs, A* pathfinding, steering behaviors
- **Polyglot scripting** — Lua first, Python and JS planned
- **Visual editor** — native egui editor with viewport, inspector, asset browser, play/pause
- **Hot-reload** — scripts, assets, and game code reloaded without restart
- **Physics** — AABB/circle colliders, spatial hashing, rigid body dynamics
- **Audio** — sound effects and streaming music via rodio
- **MCP server** — AI agents (Claude, GPT, Copilot) can build and modify games via Model Context Protocol

---

## 📚 Documentation

### Architecture

| Document | Description |
|---|---|
| [Architecture Overview](docs/architecture/overview.md) | System design, crate layout, dependency graph, ECS, render pipeline, data flow |
| [Technology Decisions](docs/architecture/tech-decisions.md) | Why we chose each dependency, alternatives considered |

### Development Roadmap

| Document | Description |
|---|---|
| [Milestones Overview](docs/milestones.md) | 10-phase plan with effort estimates and dependency chain |

### Task Breakdowns (per milestone)

| Phase | Milestone | Effort | Tasks |
|---|---|---|---|
| 1 | [ECS + Plugin + App Lifecycle](docs/tasks/milestone-01-ecs-core.md) | ~1 week | Entity, Component, World, Query, System, Events, Plugin, App |
| 2 | [Window + Render Graph + wgpu](docs/tasks/milestone-02-render-graph.md) | ~1 week | winit window, wgpu init, render graph nodes, clear pass, camera |
| 3 | [2D Rendering + Animation](docs/tasks/milestone-03-2d-rendering.md) | ~1.5 weeks | Sprites, batching, atlas, sprite sheets, tweening, state machines |
| 4 | [Input + Assets + Scene](docs/tasks/milestone-04-input-assets-scene.md) | ~1 week | Keyboard/mouse, asset server, loaders, scene graph, RON serialization |
| 5 | [2D Lighting + Particles](docs/tasks/milestone-05-lighting-particles.md) | ~1.5 weeks | Point/spot/ambient lights, shadows, GPU particle emitters/affectors |
| 6 | [Audio + Physics](docs/tasks/milestone-06-audio-physics.md) | ~1 week | rodio playback, AABB/circle colliders, spatial hash, rigid bodies |
| 7 | [Game AI](docs/tasks/milestone-07-game-ai.md) | ~1 week | Behavior trees, FSMs, A* pathfinding, steering behaviors, blackboard |
| 8 | [Scripting (Lua)](docs/tasks/milestone-08-scripting.md) | ~3-4 days | ScriptHost trait, mlua integration, hot-reload, engine API exposure |
| 9 | [Visual Editor + Hot-Reload](docs/tasks/milestone-09-editor.md) | ~2-3 weeks | Viewport, gizmos, hierarchy, inspector, asset browser, play/pause, undo/redo |
| 10 | [Stretch Goals](docs/tasks/milestone-10-stretch.md) | Ongoing | 2.5D, 3D, skeletal animation, Python/JS scripting, MCP/AI agents |

**Total core estimate: ~11-13 weeks**

---

## 🏗️ Project Structure

```
ember-engine/
├── README.md                           ← you are here
├── docs/
│   ├── architecture/
│   │   ├── overview.md                 # System design & crate layout
│   │   └── tech-decisions.md           # Technology choices & rationale
│   ├── milestones.md                   # Phased development plan
│   └── tasks/
│       ├── milestone-01-ecs-core.md
│       ├── milestone-02-render-graph.md
│       ├── milestone-03-2d-rendering.md
│       ├── milestone-04-input-assets-scene.md
│       ├── milestone-05-lighting-particles.md
│       ├── milestone-06-audio-physics.md
│       ├── milestone-07-game-ai.md
│       ├── milestone-08-scripting.md
│       ├── milestone-09-editor.md
│       └── milestone-10-stretch.md
├── crates/                             # Engine crates (Cargo workspace)
│   ├── ember_core/                     # ECS, App, Plugin, Events, Time
│   ├── ember_render/                   # Render graph, wgpu backend
│   ├── ember_2d/                       # Sprites, atlas, tilemap, 2D pipeline
│   ├── ember_input/                    # Keyboard, mouse, gamepad
│   ├── ember_assets/                   # Asset server, handles, hot-reload
│   ├── ember_audio/                    # Sound playback (rodio)
│   ├── ember_animation/                # Sprite sheets, tweens, state machines
│   ├── ember_lighting/                 # 2D lights, shadows, light mask
│   ├── ember_particles/                # GPU compute particle system
│   ├── ember_physics/                  # Colliders, spatial hash, rigid body
│   ├── ember_ai/                       # Behavior trees, FSM, pathfinding
│   ├── ember_script/                   # ScriptHost trait
│   ├── ember_script_lua/               # Lua backend (mlua)
│   ├── ember_scene/                    # Scene graph, serialization
│   ├── ember_editor/                   # Visual editor (egui)
│   ├── ember_hot_reload/               # Dynamic lib + asset reload
│   └── ember_mcp/                      # MCP server — AI agent interface
├── examples/                           # Milestone demo apps
└── assets/                             # Shared dev assets
```

---

## 🛠️ Tech Stack

| Concern | Crate | Why |
|---|---|---|
| Windowing | `winit` | De-facto cross-platform standard |
| GPU | `wgpu` | Safe Rust GPU, Metal/Vulkan/DX12/WebGPU |
| Math | `glam` | SIMD-optimized, 2D & 3D |
| Audio | `rodio` | Simple, cross-platform |
| Scripting | `mlua` / `pyo3` / `deno_core` | Polyglot support |
| Editor UI | `egui` + `eframe` | Immediate-mode, embeddable |
| Serialization | `serde` + `ron` | Rust-native format for scenes |
| Logging | `tracing` | Structured, async-friendly |

See [Technology Decisions](docs/architecture/tech-decisions.md) for full rationale.

---

## License

MIT
