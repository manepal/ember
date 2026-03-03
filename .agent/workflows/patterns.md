---
description: Ember Engine coding patterns and architecture reference for AI agents
---

# Architecture Patterns

Quick reference for the patterns used throughout Ember Engine. Read this before writing any code.

## The Golden Rule

**Everything is an Entity with Components, processed by Systems, registered by Plugins.**

```
Plugin → registers → Systems + Components + Resources + Events
System → queries → Components from World
System → reads/writes → Resources
System → sends/receives → Events
```

## Pattern: Component (Data)

Components are plain data. No methods, no behavior. Derive standard traits.

```rust
use ember_core::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Debug, Clone)]
pub struct Velocity(pub Vec2);

// For serializable components (scene save/load):
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Sprite {
    pub texture: Handle<Texture>,
    pub color: Color,
    pub flip_x: bool,
    pub flip_y: bool,
}
```

## Pattern: System (Behavior)

Systems are free functions. Parameters are injected by the scheduler.

```rust
// Read-only query
fn debug_positions(query: Query<(&Transform2D, &Name)>) {
    for (tf, name) in query.iter() {
        println!("{}: {:?}", name.0, tf.position);
    }
}

// Mutable query with resource access
fn apply_gravity(
    mut query: Query<&mut Velocity, With<RigidBody>>,
    physics: Res<PhysicsConfig>,
    time: Res<Time>,
) {
    for mut vel in query.iter_mut() {
        vel.0.y += physics.gravity * time.delta_secs();
    }
}

// System that reads events
fn on_collision(
    events: EventReader<CollisionEvent>,
    mut audio: ResMut<AudioOutput>,
    assets: Res<AssetServer>,
) {
    for event in events.iter() {
        audio.play(assets.load("sounds/hit.wav"));
    }
}
```

**System parameter types:**
| Parameter | Use |
|---|---|
| `Query<&T>` | Read component T |
| `Query<&mut T>` | Write component T |
| `Query<(&A, &B), With<C>>` | Multiple components with filter |
| `Query<(&A, Option<&B>)>` | Optional component |
| `Res<T>` | Read resource |
| `ResMut<T>` | Write resource |
| `EventReader<T>` | Read events from last frame |
| `EventWriter<T>` | Send events |

## Pattern: Plugin (Registration)

Plugins wire everything together. One plugin per subsystem.

```rust
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        // 1. Resources (config, state)
        app.insert_resource(PhysicsConfig::default());
        app.insert_resource(SpatialHashGrid::new(64.0));

        // 2. Events
        app.add_event::<CollisionEvent>();
        app.add_event::<TriggerEvent>();

        // 3. Systems (in execution order)
        app.add_system(broadphase_system);
        app.add_system(narrowphase_system);
        app.add_system(collision_response_system);
        app.add_system(integration_system);
    }
}
```

## Pattern: Resource (Singleton)

Global state that isn't tied to an entity.

```rust
#[derive(Debug)]
pub struct PhysicsConfig {
    pub gravity: f32,
    pub substeps: u32,
    pub collision_layers: u32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: -9.81,
            substeps: 4,
            collision_layers: 0xFFFF_FFFF,
        }
    }
}
```

## Pattern: Event (Communication)

Typed, fire-and-forget messages between systems. Double-buffered.

```rust
pub struct CollisionEvent {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub normal: Vec2,
    pub penetration: f32,
}

// System A sends:
fn detect_collisions(mut events: EventWriter<CollisionEvent>) {
    events.send(CollisionEvent { /* ... */ });
}

// System B receives (next frame):
fn handle_collisions(events: EventReader<CollisionEvent>) {
    for event in events.iter() {
        // respond
    }
}
```

## Pattern: Asset Handle

Reference assets by handle, never by raw data.

```rust
// Load returns a handle immediately (async load in background)
let texture: Handle<Texture> = asset_server.load("sprites/player.png");

// Attach to component
world.spawn()
    .insert(Sprite { texture, ..default() })
    .insert(Transform2D::default());

// Handle is ref-counted — asset stays loaded while any handle exists
```

## Pattern: Render Graph Node

Render passes are nodes in a DAG.

```rust
pub struct SpritePassNode;

impl RenderNode for SpritePassNode {
    fn input_slots(&self) -> Vec<SlotDescriptor> {
        vec![SlotDescriptor::texture("scene_color")]
    }

    fn output_slots(&self) -> Vec<SlotDescriptor> {
        vec![SlotDescriptor::texture("scene_with_sprites")]
    }

    fn run(&self, ctx: &mut RenderContext, world: &World) {
        // Set up pipeline, bind textures, draw instanced quads
    }
}

// Register in plugin:
render_graph.add_node("sprites", SpritePassNode);
render_graph.add_edge("clear", "sprites");
render_graph.add_edge("sprites", "lighting");
```

## Anti-Patterns (Don't Do This)

❌ **Don't create singletons or managers** — use Resources instead
```rust
// BAD: static singleton
static AUDIO_MANAGER: Mutex<AudioManager> = ...;

// GOOD: Resource
app.insert_resource(AudioOutput::new());
```

❌ **Don't put logic in components** — keep them as pure data
```rust
// BAD: method on component
impl Health {
    fn take_damage(&mut self, amount: f32) { ... }
}

// GOOD: system
fn damage_system(mut query: Query<&mut Health>, events: EventReader<DamageEvent>) { ... }
```

❌ **Don't hardcode paths** — use the asset server
```rust
// BAD
let data = std::fs::read("assets/player.png")?;

// GOOD
let handle = asset_server.load("player.png");
```

❌ **Don't use `unwrap()` in library code**
```rust
// BAD
let entity = world.get::<Transform2D>(id).unwrap();

// GOOD
if let Some(transform) = world.get::<Transform2D>(id) { ... }
```
