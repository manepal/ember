---
description: How to verify and test Ember Engine changes
---

# Testing & Verification

Run these checks after any code change. All must pass before a task is considered complete.

// turbo-all

## Quick Check (after every change)

```bash
cargo build --workspace
cargo test --workspace
```

## Full Verification (before marking milestone complete)

```bash
# 1. Build everything
cargo build --workspace

# 2. Run all tests
cargo test --workspace

# 3. Clippy lint — treat warnings as errors
cargo clippy --workspace -- -D warnings

# 4. Format check
cargo fmt --check

# 5. Run the milestone's example app (replace <example> with milestone name)
cargo run --example <example>
```

## Test Structure

### Unit Tests

Place in the same file as the code, at the bottom:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        // Arrange → Act → Assert
    }
}
```

### Integration Tests

Place in `crates/ember_<name>/tests/`:

```rust
// crates/ember_physics/tests/collision_tests.rs
use ember_physics::*;
use ember_core::prelude::*;

#[test]
fn aabb_vs_aabb_overlap() {
    // Test that uses the public API across modules
}
```

### Example Programs

Each milestone has a required example in `examples/`. These are both demos and manual verification:

| Milestone | Example | What to verify |
|---|---|---|
| 1-2 | `hello_window` | Window opens, clears to color, closes |
| 3 | `sprite_demo` | Sprites render, animation plays, tweens work |
| 4 | `input_demo` | Arrow keys move sprite, assets load from files |
| 5 | `lighting_demo` | Lights illuminate scene, shadows cast |
| 5 | `particle_demo` | Fire/smoke at 60fps |
| 6 | `physics_demo` | Objects fall, collide, bounce |
| 7 | `ai_demo` | Enemies patrol, chase, pathfind |
| 8 | `scripted_platformer` | Lua controls player, hot-reload works |
| 9 | `editor` | Full editor with viewport and play/pause |

## Debugging Tips

```bash
# Run a single crate's tests with output
cargo test -p ember_core -- --nocapture

# Run a specific test
cargo test -p ember_core test_entity_spawn -- --nocapture

# Check for unused dependencies
cargo machete

# View generated docs
cargo doc --document-private-items -p ember_core --open

# Profile compile times
cargo build --workspace --timings
```
