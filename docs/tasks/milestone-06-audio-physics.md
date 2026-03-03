# Milestone 6 — Audio + Physics

**Crates:** `ember_audio`, `ember_physics`
**Depends on:** Milestone 5
**Effort:** ~1 week
**Deliverable:** Sound effects play on events; entities have AABB colliders and rigid body physics.

---

## Tasks

### 6.1 Audio Backend

- [ ] Create `AudioPlugin` wrapping `rodio`
- [ ] Implement `AudioOutput` resource (rodio `OutputStream` + `OutputStreamHandle`)
- [ ] Implement `AudioSource` asset type (loaded via asset server)
- [ ] Handle audio device initialization failures gracefully

### 6.2 Audio Playback API

- [ ] `audio.play(handle)` — one-shot sound effect
- [ ] `audio.play_looped(handle)` — background music
- [ ] `audio.stop(instance)` / `audio.pause(instance)` / `audio.resume(instance)`
- [ ] `audio.set_volume(instance, volume)`
- [ ] Return `AudioInstance` handle for controlling playback
- [ ] Implement `AudioEvent` for triggering sounds from systems

### 6.3 Spatial Audio (Stretch)

- [ ] Implement `AudioListener` component (attached to camera)
- [ ] Implement `AudioEmitter` component (attached to sound-producing entities)
- [ ] Pan and attenuate based on distance/position

### 6.4 Collider Components

- [ ] Define `Collider2D` enum: `AABB { half_extents }`, `Circle { radius }`
- [ ] Define `CollisionLayer` — bitmask for filtering what collides with what
- [ ] Implement collider-to-world transform (collider offset + entity transform)

### 6.5 Collision Detection

- [ ] Implement narrow-phase tests:
  - AABB vs AABB
  - Circle vs Circle
  - AABB vs Circle
- [ ] Return `CollisionManifold` (contact point, normal, penetration depth)
- [ ] Unit tests: overlapping, touching, separated cases for each pair

### 6.6 Spatial Hash Broadphase

- [ ] Implement `SpatialHashGrid` — cell-based broadphase acceleration
- [ ] Insert colliders into grid cells based on AABB bounds
- [ ] Query potential collision pairs from overlapping cells
- [ ] Skip pairs on different collision layers
- [ ] Benchmark: target 1000+ colliders at 60fps

### 6.7 Rigid Body Dynamics

- [ ] Define `RigidBody` component: mass, velocity, angular_velocity, damping, gravity_scale
- [ ] Define `RigidBodyType`: Dynamic, Kinematic, Static
- [ ] Implement integration system:
  - Apply gravity (`velocity += gravity * gravity_scale * dt`)
  - Apply damping (`velocity *= 1 - damping * dt`)
  - Update position (`position += velocity * dt`)
- [ ] Implement collision response — separate overlapping bodies, apply impulse

### 6.8 Collision Events

- [ ] Emit `CollisionEvent` on collision start/stay/end
- [ ] Emit `TriggerEvent` for sensor colliders (detect overlap without physics response)
- [ ] Wire into ECS event bus

---

## Exit Criteria

- [ ] `cargo test -p ember_audio -p ember_physics` — all tests pass
- [ ] `examples/physics_demo` — boxes fall with gravity, collide with ground, bounce, sound plays on impact
