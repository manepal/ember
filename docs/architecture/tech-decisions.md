# Technology Decisions

Key technology choices for Ember Engine and the reasoning behind each.

---

## Core Dependencies

| Concern | Crate | Version | Why This |
|---|---|---|---|
| **Windowing** | `winit` | 0.30 | De-facto standard for cross-platform window management in Rust. Mature, well-maintained. |
| **GPU** | `wgpu` | 24 | Modern, safe Rust GPU API. Abstracts Metal (macOS), Vulkan (Linux), DX12 (Windows), WebGPU (web). Future-proof via WebGPU standard. |
| **Math** | `glam` | 0.29 | SIMD-optimized, ergonomic. Supports 2D and 3D vectors, matrices, and quaternions. Used by Bevy. |
| **Image** | `image` | 0.25 | Broad format support (PNG, JPEG, etc.). Lightweight. |
| **Audio** | `rodio` | 0.20 | Simple cross-platform audio. Streaming playback, spatial audio support. |
| **Byte layout** | `bytemuck` | 1.x | Safe transmutation for GPU buffer ↔ Rust struct mapping. Zero-cost. |
| **Async runtime** | `pollster` | 0.4 | Minimal async executor for wgpu surface initialization. |

## Serialization

| Crate | Use |
|---|---|
| `serde` | Derive-based serialization framework |
| `ron` | Rusty Object Notation — human-readable, Rust-native format for scene files, asset manifests, config |

**Why RON over JSON/YAML?** RON understands Rust types (enums, tuples, structs), is more compact than JSON, and doesn't have YAML's ambiguity issues.

## Scripting

| Language | Crate | Priority | Notes |
|---|---|---|---|
| **Lua** | `mlua` 0.10 | Phase 8 (core) | Lightest FFI overhead, battle-tested in games (WoW, Roblox, Defold). Supports Lua 5.4 + Luau. |
| **Python** | `pyo3` 0.23 | Stretch | Rich ecosystem, familiar to many devs. Higher FFI overhead than Lua. |
| **JavaScript** | `deno_core` 0.31 | Stretch | V8-based, TypeScript support. Heaviest runtime but broadest language familiarity. |

**Architecture:** A `ScriptHost` trait in `ember_script` defines the interface. Each language backend implements the trait independently. Game code picks backends via feature flags.

## Editor & Hot-Reload

| Crate | Use |
|---|---|
| `egui` 0.30 + `eframe` | Immediate-mode GUI for editor panels (inspector, hierarchy, asset browser) |
| `notify` | Cross-platform file system watcher for hot-reload triggers |
| `libloading` | Dynamic library loading for Rust game code hot-reload |

## Pathfinding

| Crate | Use |
|---|---|
| `pathfinding` 4 | A*, Dijkstra, BFS algorithms for AI navigation |

## Animation

| Crate | Use |
|---|---|
| `interpolation` 0.3 | Easing functions (ease-in, ease-out, bounce, elastic, etc.) for tweening |

## MCP (AI Agent Interface)

| Crate | Use |
|---|---|
| `rmcp` | Rust MCP SDK — implements the Model Context Protocol for exposing tools/resources to AI agents |
| `tokio` | Async runtime for MCP server transport (stdio, HTTP+SSE) |

The MCP server exposes engine capabilities as **tools** (actions: spawn entity, modify component, run system) and **resources** (read-only data: scene tree, entity info, asset list). AI agents like Claude, GPT, or Copilot connect via JSON-RPC 2.0 and can build, modify, and debug games programmatically.

## Logging

| Crate | Use |
|---|---|
| `tracing` | Structured, async-friendly logging with span support. Integrates with editor console. |

---

## Alternatives Considered

### ECS

| Option | Pros | Cons | Decision |
|---|---|---|---|
| **Custom** | Maximum learning, full control, no external deps | Substantial effort (~2-3 weeks) | ✅ Chosen — core learning goal |
| `hecs` | Proven archetypal, minimal API | Less feature-rich, still external | Fallback option |
| `bevy_ecs` | Most featureful | Tightly coupled to Bevy | Rejected |

### GPU API

| Option | Pros | Cons | Decision |
|---|---|---|---|
| **`wgpu`** | Full GPU control, cross-platform, WebGPU-based | More boilerplate than high-level options | ✅ Chosen |
| `pixels` | Simple CPU framebuffer | No GPU acceleration, limited | Rejected |
| `macroquad` | Batteries-included | Opinionated, less extensible | Rejected |

### Editor UI

| Option | Pros | Cons | Decision |
|---|---|---|---|
| **`egui`** | Immediate-mode, lightweight, embeddable in wgpu | Less polished than native UI | ✅ Chosen |
| `iced` | Elm-like retained mode | Harder to embed in game viewport | Rejected |
| Web-based (Tauri) | Rich UI libraries | Latency, complexity, bundle size | Rejected |
