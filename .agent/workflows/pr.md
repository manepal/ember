---
description: How to version control and submit pull requests
---

# Version Control & PRs

Use this workflow to finalize a milestone and stage the code for review.

## 1. Verify Before Committing

Ensure all checks pass. You must run these yourself:

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

## 2. Branching

Before starting a milestone, create a branch. Do not work on `main`.

1. Read `docs/milestones.md` to identify the current milestone name.
2. Draft a branch name focusing on the feature crate, e.g. `feat/ember-physics` or `fix/render-graph-bug`.
3. Check out the branch:
   ```bash
   git checkout -b <branch-name>
   ```

## 3. Committing Changes

Commit using Conventional Commits. Commits must be highly localized to the crate modified.

**Valid Formats:**
- `feat(ember_core): implement plugin system`
- `fix(ember_render): correct clear color pass`
- `test(ember_physics): add overlap integration tests`
- `docs(workflows): add debug workflow instructions`

## 4. Drafting the Pull Request

Once the milestone is completed, prepare a final PR description using your code modification tools to create a `pr_description.md` file in the project root or provide the summary directly to the user.

A good PR description includes:
1. **Milestone Summary:** A recap of what Phase/Milestone this completes.
2. **Changed Crates:** A list of `ember_*` crates touched.
3. **Public API additions:** A quick list of core types (`System`, `Component`, `Plugin`) exposed.
4. **Verification Log:** An explicit statement that `test`, `clippy`, and `fmt` passed.

**Example PR File:**
```markdown
# Complete Phase 3: 2D Rendering

## Overview
This PR satisfies Milestone 3 by bringing `ember_2d` to life. It introduces the core `Sprite` component, the `Transform2D` component, and adds them to the WGPU `RenderGraph` via a new Sprite Pass. 

## Crates modified
- **ember_2d**: Added core rendering types and batching logic.
- **ember_render**: Updated render graph to expose the main scene buffer.

## API Additions
- `Sprite` component
- `Transform2D` component
- `SpritePassNode` graph node

## Verification
- [x] Workspaces tests passing
- [x] Clippy warnings resolved
- [x] Formatted using `rustfmt`
```
