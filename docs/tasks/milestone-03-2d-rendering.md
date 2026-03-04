# Milestone 3 — 2D Rendering + Animation

**Crates:** `ember_2d`, `ember_animation`
**Depends on:** Milestone 2
**Effort:** ~1.5 weeks
**Deliverable:** Textured sprites render on screen with sprite sheet animation and tweening.

---

## Tasks

### 3.1 GPU Texture Management

- [x] Implement `Texture` struct (wgpu texture, view, sampler)
- [x] Implement `TextureLoader` — loads image files → GPU textures
- [x] Implement texture bind group layout and creation
- [x] Support common formats (PNG, JPEG via `image` crate)

### 3.2 Sprite Component & Rendering

- [x] Define `Sprite` component (texture handle, source rect, tint color, flip x/y)
- [x] Define `Transform2D` component (position, rotation, scale)
- [x] Create sprite vertex/fragment shaders (WGSL)
  - Vertex: transform quad by view-projection × model matrix
  - Fragment: sample texture, apply tint
- [x] Implement render pipeline for sprites

### 3.3 Batched Sprite Renderer

- [x] Implement instanced quad rendering — single draw call per texture
- [x] Create `SpriteInstance` struct (model matrix, UV rect, tint) → `bytemuck` buffer
- [x] Sort sprites by texture to minimize draw calls, then by Z-order
- [x] Implement `SpriteBatchNode` for the render graph
- [x] Benchmark: target 10K+ sprites at 60fps

### 3.4 Texture Atlas

- [x] Implement `TextureAtlas` — grid-based and free-form rect layouts
- [x] `TextureAtlas::from_grid(texture, tile_size, columns, rows)`
- [x] `TextureAtlas::from_rects(texture, rects: Vec<Rect>)`
- [x] `atlas.get_rect(index) → Rect` for sprite UV mapping

### 3.5 Sprite Sheet Animation

- [x] Define `SpriteAnimationClip` (name, frames, frame_duration, looping)
- [x] Define `SpriteAnimator` component (clips map, current clip, timer, frame index, speed)
- [x] Implement `SpriteAnimationSystem` — advances frame timer, updates sprite's atlas index
- [x] Support play, pause, set_clip, set_speed

### 3.6 Tweening System

- [x] Define `Tween<T>` component (target entity, from, to, duration, easing, on_complete)
- [x] Implement easing functions: linear, ease_in, ease_out, ease_in_out, bounce, elastic, back
- [x] Implement `TweenSystem` — ticks active tweens, applies interpolated value
- [x] Support `TweenAction::Loop`, `PingPong`, `Remove`, `Callback`
- [x] Support tween chaining (sequence) and parallel composition

### 3.7 Animation State Machine

- [x] Define `AnimationStateMachine` component (states, transitions, current, parameters)
- [x] Define `AnimTransition` (from, to, condition, blend_duration)
- [x] Implement parameter types: `Bool`, `Float`, `Trigger`
- [x] Implement `AnimStateMachineSystem` — evaluates conditions, switches clips
- [x] Support cross-fade blending between clips

### 3.8 Animation Events

- [x] Define `AnimationEvent` (frame index, callback)
- [x] Fire ECS events when animation reaches specified frames
- [x] Use case: footstep sounds, VFX spawns

---

## Exit Criteria

- [x] `cargo test -p ember_2d -p ember_animation` — all tests pass
- [x] `examples/sprite_demo` — shows animated sprite sheet, tweened properties, camera panning
