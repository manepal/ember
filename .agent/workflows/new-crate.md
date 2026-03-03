---
description: How to add a new crate to the Ember Engine workspace
---

# Adding a New Crate

Follow these steps when a milestone requires creating a new `ember_*` crate.

// turbo-all

## 1. Check Architecture Docs

Use your file viewing tools to read `docs/architecture/overview.md`.

Find the crate in the **Workspace Layout** section. Note its dependencies from the **Dependency Graph**.

## 2. Scaffold the Crate

Use your file creation and modification tools (e.g. `write_to_file`) to create the following structure and files. Do not use bash commands like `cat >` as that is an anti-pattern for AI agents.

Create the directory structure: `crates/ember_<name>/src`

Create **`crates/ember_<name>/Cargo.toml`**:
```toml
[package]
name = "ember_<name>"
version = "0.1.0"
edition = "2021"
description = "<One-line description from architecture doc>"

[dependencies]
ember_core = { path = "../ember_core" }

[dev-dependencies]
# Test utilities if needed
```

Create **`crates/ember_<name>/src/lib.rs`**:
```rust
//! Ember <Name> — <description>

pub mod plugin;

pub use plugin::*;
```

Create **`crates/ember_<name>/src/plugin.rs`**:
```rust
use ember_core::prelude::*;

pub struct <Name>Plugin;

impl Plugin for <Name>Plugin {
    fn build(&self, app: &mut App) {
        // Register components, systems, resources, events
    }
}
```

## 3. Register in Workspace

Add the crate to the root `Cargo.toml`:

```toml
[workspace]
members = [
    "crates/ember_<name>",
    # ... existing members
]
```

## 4. Verify

```bash
cargo build -p ember_<name>
cargo test -p ember_<name>
cargo clippy -p ember_<name> -- -D warnings
```

## 5. Commit

```bash
git add crates/ember_<name>/ Cargo.toml
git commit -m "chore(ember_<name>): scaffold new crate"
```
