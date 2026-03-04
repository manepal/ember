# Milestone 9 — NPC AI (In-Game Behavior)

**Crate:** `ember_npc_ai`
**Depends on:** Milestone 8 (Audio + Physics)
**Effort:** ~1 week
**Deliverable:** Behavior trees, finite state machines, A* pathfinding, and steering behaviors for NPC entities. *(Note: This is about deterministic game AI for NPCs — LLM-based AI integration is in Phase 12.)*

---

## Tasks

### 9.1 Blackboard

- [ ] Implement `Blackboard` component — `HashMap<String, BlackboardValue>`
- [ ] Define `BlackboardValue` enum: Bool, Int, Float, Vec2, Entity, String
- [ ] Implement typed get/set convenience methods
- [ ] Unit tests: insert, update, type mismatch handling

### 9.2 Behavior Trees — Data Structures

- [ ] Define `BehaviorNode` enum:
  - **Composite:** `Sequence`, `Selector`, `Parallel`
  - **Decorator:** `Inverter`, `Repeater(n)`, `Succeeder`, `UntilFail`
  - **Leaf:** `Condition(fn)`, `Action(fn)`, `Wait(duration)`
- [ ] Define `BehaviorStatus`: Running, Success, Failure
- [ ] Define `BehaviorTree` component wrapping root node + tick state

### 9.3 Behavior Trees — Execution

- [ ] Implement tree traversal (depth-first tick)
- [ ] Implement `Sequence` — run children in order, fail on first failure
- [ ] Implement `Selector` — try children in order, succeed on first success
- [ ] Implement `Parallel` — run all children, configurable success/failure policy
- [ ] Implement decorator nodes (Inverter, Repeater, etc.)
- [ ] Implement `BehaviorTreeSystem` — ticks all entities with `BehaviorTree` each frame
- [ ] Support resuming `Running` nodes across frames
- [ ] Unit tests: each node type, composed trees, running/suspend/resume

### 9.4 Behavior Trees — Data-Driven

- [ ] Define RON format for behavior tree definitions
- [ ] Implement `BehaviorTreeLoader` for the asset server
- [ ] Register action/condition functions by string name
- [ ] Load tree from RON → resolve function references → instantiate

### 9.5 Finite State Machines

- [ ] Define `State` trait: `on_enter`, `on_update`, `on_exit`
- [ ] Define `Transition` struct: from_state, to_state, condition
- [ ] Implement `StateMachine` component: current state, transitions list
- [ ] Implement `StateMachineSystem` — evaluates transitions, manages state lifecycle
- [ ] Unit tests: enter/update/exit hooks, transition chaining

### 9.6 Pathfinding — NavGrid

- [ ] Implement `NavGrid` resource: width, height, walkable bitmap, cost map
- [ ] Implement A* on grid (`find_path(from, to) → Option<Vec<IVec2>>`)
- [ ] Implement Dijkstra fallback for exploration
- [ ] Support diagonal movement (configurable)
- [ ] Implement dynamic obstacle updates (re-mark cells)
- [ ] Unit tests: known grids with known shortest paths, unreachable targets

### 9.7 Pathfinding — Path Following

- [ ] Implement `PathFollower` component — stores current path, waypoint index
- [ ] Implement `PathFollowSystem` — steers entity toward next waypoint
- [ ] Request new paths via `PathRequest` event
- [ ] Handle path invalidation (obstacle appeared on path)

### 9.8 Steering Behaviors

- [ ] Implement steering behavior functions → return `Vec2` force:
  - `seek(target)`, `flee(target)`, `arrive(target, decel_radius)`
  - `wander(radius, jitter)`
  - `separation(neighbors, radius)`, `alignment(neighbors, radius)`, `cohesion(neighbors, radius)`
- [ ] Implement `SteeringAgent` component — list of active behaviors with weights
- [ ] Implement `SteeringSystem` — computes weighted sum of forces, applies to velocity
- [ ] Unit tests: force direction and magnitude for each behavior

---

## Exit Criteria

- [ ] `cargo test -p ember_npc_ai` — all tests pass
- [ ] `examples/npc_ai_demo` — enemies patrol waypoints, detect player, chase with A* pathfinding, flock with steering
