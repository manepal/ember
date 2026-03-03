# Milestone 1 — ECS + Plugin + App Lifecycle

**Crate:** `ember_core`
**Effort:** ~1 week
**Deliverable:** A working ECS with entities, components, typed queries, system scheduling, plugin registration, and a fixed-timestep game loop.

---

## Tasks

### 1.1 Entity & Generational Index

- [x] Define `Entity` struct with `id: u32` and `generation: u32`
- [x] Implement `EntityAllocator` — recycles IDs, increments generation on reuse
- [x] Implement `is_alive(entity)` check
- [x] Unit tests: create, destroy, reuse, generation mismatch detection

### 1.2 Component Storage (Archetypes)

- [x] Define `Component` trait (`'static + Send + Sync`)
- [x] Implement `Archetype` — a table of entities sharing the same component set
  - Stores columns as `Vec<u8>` with type-erased access
  - Tracks component layout (TypeId → column index)
- [x] Implement `ArchetypeId` and archetype graph (edges for add/remove component)
- [x] Implement archetype migration — moving entity between archetypes on component add/remove
- [x] Unit tests: insert, remove, migrate, type safety

### 1.3 World

- [x] Implement `World` struct combining `EntityAllocator` + archetype storage
- [x] `world.spawn()` → returns `EntityBuilder` for fluent component insertion
- [x] `world.despawn(entity)` → removes entity, frees slot
- [x] `world.insert_component(entity, component)` / `world.remove_component::<T>(entity)`
- [x] `world.get::<T>(entity)` / `world.get_mut::<T>(entity)`
- [x] Unit tests: full entity lifecycle, component access patterns

### 1.4 Resources

- [x] Implement `Resources` — type-map for singletons (`HashMap<TypeId, Box<dyn Any>>`)
- [x] `world.insert_resource(resource)` / `world.resource::<T>()` / `world.resource_mut::<T>()`
- [x] Unit tests: insert, overwrite, access, panic on missing

### 1.5 Query System

- [x] Implement `Query<T>` for iterating entities by component signature
  - Support single component: `Query<&Position>`
  - Support tuple: `Query<(&Position, &mut Velocity)>`
  - Support optional: `Query<(&Position, Option<&Sprite>)>`
- [x] Implement `With<T>` / `Without<T>` filters
- [x] Queries backed by archetype matching — cache matched archetypes
- [x] Unit tests: single, multi, optional, filter, empty results

### 1.6 System Trait & Scheduler

- [x] Define `System` trait (or use function pointers with type-erased params)
- [x] Implement `SystemParam` trait for `Query`, `Res<T>`, `ResMut<T>`, `EventReader<T>`, `EventWriter<T>`
- [x] Implement `Schedule` — ordered list of systems
- [x] Implement access declaration — systems declare read/write component types
- [x] Implement topological sort — detect conflicts, parallelize non-conflicting systems
- [x] Unit tests: ordering, conflict detection, parallel execution safety

### 1.7 Event Bus

- [x] Implement `Events<T>` — double-buffered ring buffer
- [x] `EventWriter<T>` — send events
- [x] `EventReader<T>` — read events from previous frame, tracks cursor
- [x] Unit tests: send/receive cycle, multi-reader, clear between frames

### 1.8 Time Management

- [x] Implement `Time` resource: delta, elapsed, fixed_delta, frame count
- [x] Implement fixed timestep accumulator logic
- [x] Unit tests: accumulator behavior, delta accuracy

### 1.9 Plugin & App Builder

- [x] Define `Plugin` trait: `fn build(&self, app: &mut App)`
- [x] Implement `App` struct:
  - `add_plugin(plugin)` — register a plugin
  - `add_system(system)` — add system to default schedule
  - `insert_resource(resource)` — add resource to world
  - `add_event::<T>()` — register event type
  - `run()` — enters the game loop (blocking)
- [x] `CorePlugin` — registers `Time`, event cleanup system
- [x] Unit tests: plugin ordering, double-add prevention, app lifecycle

---

## Exit Criteria

- [x] `cargo test -p ember_core` — all tests pass
- [x] `cargo clippy -p ember_core -- -D warnings` — clean
- [x] A simple main.rs demonstrates: spawn entities, add components, run systems that print to console

## Example

```rust
// examples/ecs_demo/main.rs
fn main() {
    App::new()
        .add_plugin(CorePlugin)
        .add_system(hello_system)
        .run();
}

fn hello_system(query: Query<&Name>, time: Res<Time>) {
    for name in query.iter() {
        println!("[{:.2}s] Hello, {}!", time.elapsed_secs(), name.0);
    }
}
```
