# Milestones

Ember Engine is built in **10 phases**, each producing a working, testable deliverable. Phases 1–9 are the core engine. Phase 10 is an ongoing stretch track.

---

## Overview

| Phase | Name | Effort | Key Deliverable |
|---|---|---|---|
| [1](tasks/milestone-01-ecs-core.md) | ECS + Plugin + App Lifecycle | ~1 week | Entities, components, systems, plugin trait, app builder |
| [2](tasks/milestone-02-render-graph.md) | Window + Render Graph + wgpu | ~1 week | Window opens, clears to color via render graph |
| [3](tasks/milestone-03-2d-rendering.md) | 2D Rendering + Animation | ~1.5 weeks | Sprites, batching, camera, sprite sheets, tweening |
| [4](tasks/milestone-04-input-assets-scene.md) | Input + Assets + Scene | ~1 week | Keyboard/mouse input, async asset loading, scene serialization |
| [5](tasks/milestone-05-lighting-particles.md) | 2D Lighting + Particles | ~1.5 weeks | Point/spot lights, shadows, GPU particle system |
| [6](tasks/milestone-06-audio-physics.md) | Audio + Physics | ~1 week | Sound playback, AABB colliders, rigid body dynamics |
| [7](tasks/milestone-07-game-ai.md) | Game AI | ~1 week | Behavior trees, FSMs, A* pathfinding, steering |
| [8](tasks/milestone-08-scripting.md) | Scripting (Lua) | ~3-4 days | Lua scripting integration, scriptable components |
| [9](tasks/milestone-09-editor.md) | Visual Editor + Hot-Reload | ~2-3 weeks | Native editor with viewport, inspector, play/pause |
| [10](tasks/milestone-10-stretch.md) | Stretch Goals | Ongoing | 2.5D, 3D, skeletal animation, Python/JS scripting, MCP/AI agents |

**Total core estimate: ~11-13 weeks**

---

## Dependency Chain

```
Phase 1 (ECS)
    └── Phase 2 (Render)
            └── Phase 3 (2D + Animation)
                    ├── Phase 4 (Input + Assets + Scene)
                    │       └── Phase 5 (Lighting + Particles)
                    │               └── Phase 6 (Audio + Physics)
                    │                       └── Phase 7 (AI)
                    │                               └── Phase 8 (Scripting)
                    │                                       └── Phase 9 (Editor)
                    └── Phase 10 (Stretch — can start after Phase 3)
```

---

## Milestone Exit Criteria

Each milestone must pass before moving to the next:

1. All tasks within the milestone are complete
2. `cargo test --workspace` passes
3. `cargo clippy --workspace -- -D warnings` produces no warnings
4. The milestone's example app runs and demonstrates the expected behavior
5. Documentation for new APIs is written
