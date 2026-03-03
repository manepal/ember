# Milestone 3 — 2D Rendering + Animation

**Crates:** `ember_2d`, `ember_animation`
**Depends on:** Milestone 2
**Effort:** ~1.5 weeks
**Deliverable:** Textured sprites render on screen with sprite sheet animation and tweening.

---

## Tasks

### 3.1 GPU Texture Management

- [ ] Implement `Texture` struct (wgpu texture, view, sampler)
- [ ] Implement `TextureLoader` — loads image files → GPU textures
- [ ] Implement texture bind group layout and creation
- [ ] Support common formats (PNG, JPEG via `image` crate)

### 3.2 Sprite Component & Rendering

- [ ] Define `Sprite` component (texture handle, source rect, tint color, flip x/y)
- [ ] Define `Transform2D` component (position, rotation, scale)
- [ ] Create sprite vertex/fragment shaders (WGSL)
  - Vertex: transform quad by view-projection × model matrix
  - Fragment: sample texture, apply tint
- [ ] Implement render pipeline for sprites

### 3.3 Batched Sprite Renderer

- [ ] Implement instanced quad rendering — single draw call per texture
- [ ] Create `SpriteInstance` struct (model matrix, UV rect, tint) → `bytemuck` buffer
- [ ] Sort sprites by texture to minimize draw calls, then by Z-order
- [ ] Implement `SpriteBatchNode` for the render graph
- [ ] Benchmark: target 10K+ sprites at 60fps

### 3.4 Texture Atlas

- [ ] Implement `TextureAtlas` — grid-based and free-form rect layouts
- [ ] `TextureAtlas::from_grid(texture, tile_size, columns, rows)`
- [ ] `TextureAtlas::from_rects(texture, rects: Vec<Rect>)`
- [ ] `atlas.get_rect(index) → Rect` for sprite UV mapping

### 3.5 Sprite Sheet Animation

- [ ] Define `SpriteAnimationClip` (name, frames, frame_duration, looping)
- [ ] Define `SpriteAnimator` component (clips map, current clip, timer, frame index, speed)
- [ ] Implement `SpriteAnimationSystem` — advances frame timer, updates sprite's atlas index
- [ ] Support play, pause, set_clip, set_speed

### 3.6 Tweening System

- [ ] Define `Tween<T>` component (target entity, from, to, duration, easing, on_complete)
- [ ] Implement easing functions: linear, ease_in, ease_out, ease_in_out, bounce, elastic, back
- [ ] Implement `TweenSystem` — ticks active tweens, applies interpolated value
- [ ] Support `TweenAction::Loop`, `PingPong`, `Remove`, `Callback`
- [ ] Support tween chaining (sequence) and parallel composition

### 3.7 Animation State Machine

- [ ] Define `AnimationStateMachine` component (states, transitions, current, parameters)
- [ ] Define `AnimTransition` (from, to, condition, blend_duration)
- [ ] Implement parameter types: `Bool`, `Float`, `Trigger`
- [ ] Implement `AnimStateMachineSystem` — evaluates conditions, switches clips
- [ ] Support cross-fade blending between clips

### 3.8 Animation Events

- [ ] Define `AnimationEvent` (frame index, callback)
- [ ] Fire ECS events when animation reaches specified frames
- [ ] Use case: footstep sounds, VFX spawns

---

## Exit Criteria

- [ ] `cargo test -p ember_2d -p ember_animation` — all tests pass
- [ ] `examples/sprite_demo` — shows animated sprite sheet, tweened properties, camera panning
