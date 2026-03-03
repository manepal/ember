---
description: How to debug and profile the Ember Engine
---

# Debugging & Profiling

Use this workflow when the engine panics, hangs, or struggles with performance.

// turbo-all

## 1. Engine Panics & Tracing

When a test fails or the engine panics, get the full backtrace:

1. Run the failing test or example with backtrace enabled:
   ```bash
   RUST_BACKTRACE=1 cargo test --workspace <test_name>
   RUST_BACKTRACE=1 cargo run --example <example_name>
   ```

2. Look for `unwrap()` or `expect()` calls in `ember_*` crates in the backtrace. If found inside library code, **refactor them immediately** to return `Result`s instead.

### Enabling Tracing

The engine uses the `tracing` crate. If you need to see what's happening internally, instruct the user to run the binary with `RUST_LOG`:

```bash
RUST_LOG=debug cargo run --example <example>
```

Add your own tracing to specific files if you are hunting a logic bug:
```rust
use tracing::{debug, info, warn, error};

fn some_system(time: Res<Time>) {
    debug!("Delta time: {}", time.delta_secs());
}
```

## 2. ECS Debugging

If systems are not executing or entities aren't updating:

1. **Check Plugin Registration**: Did you add the system to `app.add_system(...)` inside the plugin's `build()` function?
2. **Check Component Types**: Are you querying for `&Velocity` when the entity was spawned with `Velocity::default()`? If the entity lacks the component, the system simply skips it. It won't panic.
3. **Check System Order**: Does System A need to run before System B? Standard ECS architectures process queries randomly unless execution order is explicitly constrained. (Check API for `.before()` or `.after()` constraints if available).

## 3. Graphics & WGPU Debugging

If the screen is black, missing sprites, or crashing in WGPU:

1. Enable WGPU validation:
   ```bash
   WGPU_VALIDATION=1 cargo run --example <example>
   ```
2. For silent omissions (sprites aren't rendering):
   - Check the `Camera` transform vs the `Sprite` transform. Is the sprite off-screen?
   - Check the clear color pass in the Render Graph. Did the SpritePass get added to the graph?

## 4. Performance Profiling

If the engine is sluggish or dropping frames:

1. Run a release build:
   ```bash
   cargo run --release --example <example>
   ```
2. Profile the build process if compilation takes too long:
   ```bash
   cargo build --timings --workspace
   ```
   *View the generated HTML report.*

## 5. Memory & Lifetimes

If you encounter `borrow checker` issues:

- In Ember, components must remain plain data.
- Avoid placing `&'a T` references inside components. Use Entity IDs (`Entity`) or Asset Handles (`Handle<T>`) instead of raw references to avoid lifetime contagion.
