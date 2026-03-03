---
description: How to review the current project state and plan next steps
---

# Project Review & Planning

Use this workflow to understand the current state of the project and decide what to work on next.

## 1. Check Project Status

1. **What crates exist?**
   Use your directory listing tools (e.g. `list_dir`) to check the `crates/` directory.

2. **What tests pass?**
   Run the following command:
   ```bash
   cargo test --workspace 2>&1 | tail -20
   ```

3. **Any lint issues?**
   Run the following command:
   ```bash
   cargo clippy --workspace -- -D warnings 2>&1 | tail -20
   ```

## 2. Determine Current Phase

Read the milestone docs in order. For each milestone:

1. Open `docs/tasks/milestone-NN-*.md`
2. Count checked `[x]` vs unchecked `[ ]` tasks
3. The first milestone with unchecked tasks is the active one

**Quick summary command:**
```bash
for f in docs/tasks/milestone-*.md; do
    total=$(grep -c '\- \[' "$f" || echo 0)
    done=$(grep -c '\- \[x\]' "$f" || echo 0)
    echo "$f: $done/$total"
done
```

## 3. Pick Next Task

Within the active milestone file:

1. Find the first unchecked `[ ]` task group (e.g., "### 1.3 World")
2. Read all sub-tasks within that group
3. Check if any dependencies need to be built first (earlier task groups in the same milestone)
4. Start with the task group, implementing all sub-items

## 4. Plan the Implementation

Before coding, state your plan:

1. **What crate?** — Which `ember_*` crate will you modify or create?
2. **What files?** — List the files you'll create or modify
3. **What types?** — Components, Systems, Resources, Events you'll define
4. **What tests?** — How you'll verify the implementation
5. **What depends on this?** — Which future tasks need this to work

## 5. After Completion

1. Run `/test` workflow to verify everything passes
2. Mark completed tasks with `[x]` in the milestone doc
3. If the entire milestone is done:
   - Check exit criteria at the bottom of the milestone doc
   - Run the milestone's example app
   - Move to the next milestone

## 6. When Stuck

If you encounter a blocking issue:

1. **Missing dependency?** — Check if a previous milestone task needs to be done first
2. **API design unclear?** — Read `docs/architecture/overview.md` for the subsystem design
3. **Pattern unclear?** — Read `.agent/workflows/patterns.md` for code examples
4. **Technology question?** — Read `docs/architecture/tech-decisions.md` for rationale
5. **Still stuck?** — Ask the user for clarification rather than guessing

## 7. Update Docs and Workflows

Every review is an opportunity to improve the project documentation:

1. **Stale task docs?** — If tasks were implemented differently than planned, update the milestone doc to match reality
2. **Missing patterns?** — If you see code patterns not documented in `/patterns`, add them
3. **Outdated README?** — If new features exist that README doesn't mention, update it
4. **Workflow friction?** — If any workflow step was confusing or missing, fix it immediately
5. **Effort estimates off?** — Update `docs/milestones.md` with actual vs estimated effort

> **Principle**: Leave the docs and workflows better than you found them. Every agent session should improve the project's self-documentation.
