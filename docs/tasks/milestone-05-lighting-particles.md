# Milestone 5 — 2D Lighting + Particle System

**Crates:** `ember_lighting`, `ember_particles`
**Depends on:** Milestone 4
**Effort:** ~1.5 weeks
**Deliverable:** Point/spot lights illuminate sprites with shadows; GPU particles render fire/smoke effects.

---

## Tasks

### 5.1 Light Components

- [ ] Define `Light2D` enum: `Point`, `Spot`, `Ambient` with position, color, radius, intensity, falloff
- [ ] Define `ShadowCaster2D` component (polygon vertices that block light)
- [ ] Implement `LightPlugin` — registers light systems and render nodes
- [ ] Lights are ECS components — attach to entities, move with transforms

### 5.2 Light Mask Render Pass

- [ ] Create off-screen render target for light accumulation (light mask texture)
- [ ] Implement WGSL compute shader for point light contribution
  - Radial falloff based on distance and falloff curve
- [ ] Implement WGSL compute shader for spot light contribution
  - Cone angle masking + radial falloff
- [ ] Accumulate all light contributions into the light mask (additive blending)

### 5.3 Shadow Casting

- [ ] Implement shadow ray compute shader:
  - For each light, cast rays against `ShadowCaster2D` polygon edges
  - Mark shadowed pixels in the light mask
- [ ] Optimize: only process shadow casters within light radius
- [ ] Support soft shadows (penumbra) as a stretch

### 5.4 Light Compositing

- [ ] Implement `CompositePassNode` for the render graph
  - Multiply scene color × light mask
  - Add ambient light contribution
- [ ] Insert into render graph: `Sprites → Light Mask → Composite → UI`

### 5.5 Particle Emitter Component

- [ ] Define `ParticleEmitter` component:
  - max_particles, spawn_rate, lifetime range
  - initial velocity, color, size ranges
  - texture, blend mode (Additive, Alpha), space (World/Local)
- [ ] Define `ParticleAffector` enum:
  - Gravity, Drag, ColorOverLifetime, SizeOverLifetime
  - Turbulence, Attractor, VortexField

### 5.6 GPU Particle Pipeline

- [ ] Design particle data struct (position, velocity, color, size, age, lifetime) as GPU buffer
- [ ] Implement WGSL compute shader: spawn stage
  - Emit N particles per frame based on spawn_rate × dt
  - Initialize from emitter ranges (random within range)
- [ ] Implement WGSL compute shader: update stage
  - Advance age, apply affector forces, update position/velocity/color/size
- [ ] Implement WGSL compute shader: compact stage
  - Remove dead particles (age > lifetime), defragment buffer
- [ ] Implement `ParticleRenderNode` — instanced quad draw with particle buffer as instance data

### 5.7 Particle Presets

- [ ] Define RON format for particle presets
- [ ] Create built-in presets: fire, smoke, sparks, rain, dust
- [ ] Load presets via asset server

---

## Exit Criteria

- [ ] `cargo test -p ember_lighting -p ember_particles` — all tests pass
- [ ] `examples/lighting_demo` — point/spot lights illuminate a scene, shadows cast from obstacles
- [ ] `examples/particle_demo` — fire and smoke particle effects at 60fps with 10K+ particles
