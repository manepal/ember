# Milestone 10 — Scripting (Lua)

**Crate:** `ember_scripting`
**Depends on:** Milestone 9 (NPC AI)
**Effort:** ~3-4 days
**Deliverable:** Lua scripting integration, scriptable components, and hot-reloadable game logic.

---

## Tasks

### 10.1 ScriptHost Trait

- [ ] Define `ScriptHost` trait in `ember_script`:
  - `fn load_script(&mut self, path: &Path) → Result<ScriptId>`
  - `fn unload_script(&mut self, id: ScriptId)`
  - `fn call_fn(&mut self, id: ScriptId, fn_name: &str, args: &[ScriptValue]) → Result<ScriptValue>`
  - `fn update(&mut self, world: &mut World, dt: f32)`
- [ ] Define `ScriptValue` enum: Nil, Bool, Int, Float, String, Vec2, Entity
- [ ] Define `ScriptId` — opaque handle to a loaded script

### 10.2 ScriptComponent

- [ ] Define `ScriptComponent`: backend type, script path, script id
- [ ] Implement `ScriptSystem` — for each entity with `ScriptComponent`:
  - Calls `on_update(entity, dt)` each frame
  - Optionally calls `on_start(entity)` on first frame
  - Calls `on_destroy(entity)` when entity is despawned

### 10.3 Lua Backend

- [ ] Implement `LuaScriptHost` via `mlua`:
  - Create Lua VM instance
  - Load and compile scripts
  - Execute function calls with ScriptValue marshaling
- [ ] Expose engine APIs to Lua:
  - `get_component(entity, "ComponentName")` → table
  - `set_component(entity, "ComponentName", table)`
  - `is_key_pressed(key_name)` → bool
  - `spawn_entity()` → entity id
  - `despawn_entity(entity)`
  - `get_position(entity)` → x, y
  - `set_position(entity, x, y)`
- [ ] Implement error handling — Lua errors → engine log, don't crash

### 10.4 Script Hot-Reload

- [ ] Watch script files via `ember_hot_reload`
- [ ] On change: unload old script, reload new version, call `on_start` again
- [ ] Preserve per-entity script state where possible (via blackboard/component data)
- [ ] Log reload events to console

### 10.5 ScriptPlugin

- [ ] Create `LuaScriptPlugin` that registers:
  - `LuaScriptHost` as a resource
  - `ScriptSystem` in the update schedule
  - Script file loaders in the asset server

---

## Exit Criteria

- [ ] `cargo test -p ember_script -p ember_script_lua` — all tests pass
- [ ] `examples/scripted_platformer` — player movement controlled by Lua script
- [ ] Modify script while game runs → behavior updates without restart

## Example Lua Script

```lua
-- scripts/player.lua

function on_start(entity)
    print("Player spawned: " .. entity)
end

function on_update(entity, dt)
    local speed = 200.0
    local vx, vy = 0, 0

    if is_key_pressed("ArrowRight") then vx = speed end
    if is_key_pressed("ArrowLeft")  then vx = -speed end
    if is_key_pressed("ArrowUp")    then vy = -speed end
    if is_key_pressed("ArrowDown")  then vy = speed end

    local x, y = get_position(entity)
    set_position(entity, x + vx * dt, y + vy * dt)
end
```
