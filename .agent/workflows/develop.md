---
description: How to pick up and implement Ember Engine milestone tasks
---

# Developing Ember Engine

This workflow guides AI agents through the process of implementing features for the Ember Engine project.

// turbo-all

## 1. Understand the Project

Before making any changes, read the key documents:

- Read `README.md` for project overview and structure
- Read `docs/architecture/overview.md` for system design and crate dependencies
- Read `docs/architecture/tech-decisions.md` for technology choices
- Read `docs/milestones.md` for the phased development plan

## 2. Identify Current Milestone

Check which milestone is currently in progress:

1. Open `docs/milestones.md` to see the phase order
2. Look at crate directories under `crates/` — the last fully implemented crate tells you which phase was completed
3. Open the next milestone's task file in `docs/tasks/milestone-NN-*.md`
4. Identify the first unchecked `[ ]` task group to work on

## 3. Understand Dependencies Before Coding

Before implementing a task:

1. Check the crate's dependencies in the architecture doc (`docs/architecture/overview.md` → Dependency Graph)
2. Verify that dependent crates exist and have their public API implemented
3. Read existing crate code to understand established patterns:
   ```
   cargo doc --document-private-items -p ember_core --open
   ```

## 4. Create or Modify the Crate

### New Crate Setup

If the crate doesn't exist yet:

```bash
mkdir -p crates/ember_<name>/src
```

Create `crates/ember_<name>/Cargo.toml`:
```toml
[package]
name = "ember_<name>"
version = "0.1.0"
edition = "2021"

[dependencies]
ember_core = { path = "../ember_core" }
# Add other ember_* dependencies as needed
```

Add the crate to the workspace `Cargo.toml`:
```toml
[workspace]
members = [
    "crates/ember_<name>",
    # ... existing crates
]
```

Create `crates/ember_<name>/src/lib.rs` with the module structure.

### Coding Conventions

Follow these patterns throughout the project:

**Module organization:**
```rust
// lib.rs — re-export public API
pub mod component;
pub mod system;
pub mod plugin;

pub use component::*;
pub use system::*;
pub use plugin::*;
```

**Components** — plain structs, no methods (data only):
```rust
pub struct Transform2D {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}
```

**Systems** — free functions with typed parameters:
```rust
pub fn movement_system(query: Query<(&mut Transform2D, &Velocity)>, time: Res<Time>) {
    for (mut tf, vel) in query.iter_mut() {
        tf.position += vel.0 * time.delta_secs();
    }
}
```

**Plugins** — register components, systems, and resources:
```rust
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PhysicsConfig::default())
           .add_event::<CollisionEvent>()
           .add_system(broadphase_system)
           .add_system(narrowphase_system)
           .add_system(integration_system);
    }
}
```

**Error handling:**
- Use `thiserror` for library error types
- Use `anyhow` in examples/binaries only
- Never unwrap in library code — return `Result`

**Naming:**
- Crates: `ember_<subsystem>` (snake_case)
- Types: PascalCase (`SpriteAnimator`, `NavGrid`)
- Systems: `<subsystem>_<action>_system` (`physics_broadphase_system`)
- Plugins: `<Subsystem>Plugin` (`PhysicsPlugin`)

## 5. Write Tests

Every task group should include tests. Follow this pattern:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specific_behavior() {
        // Arrange
        let mut world = World::new();
        // ...

        // Act
        // ...

        // Assert
        assert_eq!(/* ... */);
    }
}
```

**Test categories:**
- **Unit tests** — in each module's `tests` block
- **Integration tests** — in `crates/ember_<name>/tests/`
- **Example programs** — in `examples/` (serve as both demo and manual test)

## 6. Verify Your Changes

After implementing a task group, run these checks:

```bash
# 1. Compile the workspace
cargo build --workspace

# 2. Run all tests
cargo test --workspace

# 3. Lint
cargo clippy --workspace -- -D warnings

# 4. Format
cargo fmt --check
```

All four must pass before marking a task as complete.

## 7. Update Task Tracking

**CRITICAL INSTRUCTION FOR AI AGENTS:** You MUST update the central project documentation immediately after completing any task. Do not rely solely on your internal artifact tracking; the project repository must remain the source of truth.

After completing a task or task group, perform these steps exactly:

1. Use the `view_file` tool to open the active milestone task file (e.g., `docs/tasks/milestone-NN-*.md`).
2. Use the `replace_file_content` tool to change the `[ ]` checkboxes to `[x]` for ALL completed tasks you just finished. **Do not use `sed` or `cat`.**
3. If all task groups in a milestone are done, check the **Exit Criteria** section.
4. Run the milestone's example app to verify end-to-end behavior.

## 8. Commit Conventions

Use conventional commits:

```
feat(ember_core): implement archetypal component storage

- Add Archetype struct with type-erased column storage
- Add archetype graph for component add/remove edges
- Add migration logic for moving entities between archetypes
- Add unit tests for insert, remove, migrate operations
```

Prefix categories:
- `feat` — new feature
- `fix` — bug fix
- `refactor` — code restructuring without behavior change
- `test` — adding or updating tests
- `docs` — documentation only
- `chore` — build, CI, dependencies

## 9. Common Pitfalls

- **Don't skip the ECS patterns.** Everything in Ember is an entity with components processed by systems. Don't create standalone managers or singletons — use Resources.
- **Don't add dependencies not in `tech-decisions.md`** without documenting the reason.
- **Don't break the public API** of an existing crate without updating all dependents.
- **Don't implement stretch goals** unless all core milestones (1-9) are complete.
- **Don't put game logic in engine crates.** Engine crates provide the framework; game-specific code goes in plugins or scripts.

## 10. Keep Documentation in Sync

Documentation is a living part of the project, not an afterthought. **Every implementation change must be reflected in docs.**

### What to update after completing a task:

| What changed | Update where |
|---|---|
| New public API in a crate | Add rustdoc comments on all public items |
| Task completed | Mark `[x]` in `docs/tasks/milestone-NN-*.md` |
| New demo or feature added | Register in Ember Showcase (see step 7b) |
| New pattern discovered | Add to `.agent/workflows/patterns.md` |
| Architecture decision changed | Update `docs/architecture/overview.md` and/or `tech-decisions.md` |
| New dependency added | Add to `docs/architecture/tech-decisions.md` with rationale |
| Milestone fully completed | Update `docs/milestones.md` with actual effort vs estimate |
| README claims outdated | Update `README.md` feature list or project structure |

### When to update docs proactively:

- **API changed from what the plan described?** Update the milestone task doc to reflect reality.
- **Found a better approach?** Document it in the patterns workflow and update the milestone.
- **Estimate was wrong?** Note the actual effort in `docs/milestones.md`.
- **New example needed?** Add it to the examples list in the milestone doc.

## 7b. Update the Ember Showcase

**IMPORTANT:** Whenever you complete a new demo example or add a significant visual feature, you must also update the **Ember Showcase** (`examples/ember_showcase.rs` or `crates/ember_showcase`). This is the single-window launcher app that lets users browse and run all engine demos from one place.

### When to update the Showcase:

- A new `*_demo` example is created in any crate
- An existing demo gets significantly reworked (new visuals, new interactions)
- A new subsystem is added that should be demonstrated

### How to update the Showcase:

1. Add a new `DemoEntry` implementation for the demo
2. Register it in the showcase's demo list with a name, description, and category
3. Verify the demo can be loaded and unloaded cleanly from the showcase menu
4. Test that ESC / Back returns to the menu without crashes

> **Note:** If the Showcase app does not exist yet (it is planned for Milestone 5), skip this step but leave a `TODO` comment in the demo's source file: `// TODO: Register in Ember Showcase once available`

## 11. Continuously Improve the Agentic Workflow

These workflow files are meant to **evolve with the project**. Every AI agent working on this project should actively improve them.

### When to update workflows:

1. **After completing a milestone** — reflect on what worked and what didn't:
   - Were the task breakdowns the right granularity?
   - Were there missing steps in the workflow?
   - Did you discover patterns not in `patterns.md`?
   - Did you encounter pitfalls not in the "Common Pitfalls" section?

2. **When you discover a new pattern** — add it to `patterns.md`:
   - New ECS patterns that emerged during implementation
   - Useful testing patterns
   - Performance patterns
   - Error handling patterns specific to a subsystem

3. **When a workflow step is confusing or incomplete** — fix it immediately:
   - Add missing context
   - Clarify ambiguous instructions
   - Add examples for abstract steps
   - Remove steps that turned out to be unnecessary

4. **When you find a faster way to do something** — update the workflow:
   - Better commands for verification
   - Shortcuts for common operations
   - Automations that save time

### How to update workflows:

```bash
# 1. Edit the workflow file directly
# 2. Keep the YAML frontmatter (--- description ---) accurate
# 3. Keep the // turbo-all annotation if present
# 4. Commit with:
git commit -m "docs(workflows): <what improved and why>"
```

### Improvement log

When making a significant workflow improvement, add a brief entry to `docs/workflow-changelog.md`:

```markdown
## YYYY-MM-DD — <What changed>
- **Why**: <What problem or friction prompted the change>
- **What**: <What was added/changed/removed>
- **Impact**: <How this improves the agent experience>
```

This log helps future agents understand why workflows look the way they do.

### Golden rule for workflow improvement:

> If you had to figure something out that wasn't in the workflows, **write it down** so the next agent doesn't have to figure it out again.
