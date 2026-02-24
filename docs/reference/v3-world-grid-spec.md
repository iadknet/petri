# V3 World and Grid Spec

Reference specification for canonical world/grid state, geometry, and spatial
validity semantics in V3.

Status: Active

Related references:
- `v3-tick-orchestration-spec.md`
- `v3-sensor-spec.md`
- `v3-reproduction-spec.md`
- `v3-runtime-config-spec.md`
- `v3-creature-lifecycle-spec.md`
- `v3-startup-seeding-spec.md`
- `v3-server-api-protocol-spec.md`

---

## 1. Purpose and Scope

This document defines:
- canonical world/grid configuration keys, defaults, and normalization;
- world coordinate and direction mapping;
- edge-mode behavior (`wrap` and `bounded`);
- food substrate growth/consume/seeding semantics;
- occupancy and barrier invariants;
- action-time target-validity primitives used by move and reproduce flows.

This document does not define:
- cognition/runtime mesh behavior;
- mutation-domain behavior;
- top-level turn queue or action arbitration policy.
- startup viability/founder policy;
- transport/API payload schemas.

---

## 2. World State Model

Canonical conceptual model:

```rust
pub struct WorldState {
    pub width: u16,
    pub height: u16,
    pub edge_mode: WorldEdgeMode,
    food_density: Grid<f32>,
    barriers: Grid<bool>,
    creature_at: Grid<Option<CreatureId>>,
}
```

State semantics:
- `food_density` is per-cell scalar in `[0.0, max_density]`.
- v3alpha1 canonical operating scale is normalized `f32` in `[0.0, 1.0]`.
- `barriers` marks non-passable cells.
- `creature_at` is single-occupancy: at most one creature per cell.
- Food/barrier/occupancy values are independent stored state, but movement and
  spawn validity primitives enforce barrier and occupancy constraints.

Storage layout (flat vector, row-major, etc.) is implementation detail as long
as these semantics are preserved.

---

## 3. Coordinate and Direction Semantics

Coordinate system: `(0,0)` is top-left. X increases rightward, Y increases
downward (screen coordinates).

Coordinate domain:
- `x in [0, width - 1]`
- `y in [0, height - 1]`

Direction order is canonical:

```rust
Direction::ALL = [N, NE, E, SE, S, SW, W, NW]
```

Delta mapping is canonical:
- `N=(0,-1)`, `NE=(1,-1)`, `E=(1,0)`, `SE=(1,1)`,
- `S=(0,1)`, `SW=(-1,1)`, `W=(-1,0)`, `NW=(-1,-1)`.

Neighbor resolution:
- compute `nx = x + dx`, `ny = y + dy`;
- apply edge-mode behavior from Section 4.

---

## 4. World Config Contract (Canonical Owner)

This file is the canonical owner for world/grid config keys/defaults.

| Key | Type | Default | Constraint / normalization |
| --- | --- | --- | --- |
| `world.width` | `u16` | `400` | Must be `>= 1`; invalid values fall back to `400`. |
| `world.height` | `u16` | `400` | Must be `>= 1`; invalid values fall back to `400`. |
| `world.edge_mode` | `enum{wrap,bounded}` | `wrap` | Unknown/invalid values fall back to `wrap`. |
| `world.food.growth_rate` | `f32` | `0.25` | Must be finite; clamp to `[0.0, 1.0]`; invalid falls back to `0.25`. |
| `world.food.initial_density` | `f32` | `1.0` | Clamp to `[0.0, max_density]`; invalid falls back to `max_density`. |
| `world.food.initial_coverage` | `f32` | `0.15` | Must be finite; clamp to `[0.0, 1.0]`; invalid falls back to `0.15`. |
| `world.food.spread_threshold_ratio` | `f32` | `0.75` | Must be finite; clamp to `[0.0, 1.0]`; invalid falls back to `0.75`. |
| `world.food.recovery_spawn_rate` | `f32` | `0.02` | Must be finite; clamp to `[0.0, 1.0]`; invalid falls back to `0.02`. |
| `world.food.recovery_floor_ratio` | `f32` | `0.03` | Must be finite; clamp to `[0.0, 1.0]`; invalid falls back to `0.03`. |
| `world.food.max_density` | `f32` | `1.0` | Must be finite and `> 0.0`; invalid falls back to `1.0`. |

World edge-mode behavior:
- `wrap`: neighbor coordinates wrap with modulo arithmetic in both axes.
- `bounded`: neighbors outside world bounds are unresolved (`None`).

v3alpha1 transport posture:
- Server startup/config-patch transport MUST reject submitted world values that
  violate canonical constraints (`422 validation_rejected`); transport does not
  auto-normalize invalid wire values.
- The default/normalization rules in this section apply to effective world
  config derivation from defaults plus accepted values, and to non-server
  config-loading paths.

Runtime/mutation/energy config defaults remain canonical in
`v3-runtime-config-spec.md`.

---

## 5. Food Substrate Semantics

Per-cell food is represented as `f32` in `[0.0, max_density]`, with v3alpha1
canonical scale `[0.0, 1.0]`.

### Growth

At Phase 0 (tick start), growth uses a source snapshot of pre-growth food
densities.

For each non-barrier cell:
- `source = clamp(snapshot[cell], 0.0, max_density)`.
- `delta = source * world.food.growth_rate`.
- Apply local growth: `food_density[cell] = clamp(food_density[cell] + delta, 0.0, max_density)`.
- If `source >= max_density * world.food.spread_threshold_ratio` and
  `delta > 0.0`, pick one random valid cardinal neighbor (non-barrier) and add
  the same `delta` (clamped to `max_density`).

After local growth/spread pass:
- Compute `average_density_ratio = sum(snapshot_food) / (total_cells * max_density)`.
- If `average_density_ratio < world.food.recovery_floor_ratio`, run
  `round(total_cells * world.food.recovery_spawn_rate)` recovery attempts.
- Each attempt picks one random non-barrier cell and adds
  `max_density * world.food.growth_rate` (clamped).

Barrier cells are excluded from growth/spread/recovery targets.
Creature occupancy does not block food growth/spread/recovery in v3alpha1.

### Consumption

`consume_food(cell)` semantics:
- returns the cell's current food amount;
- sets the cell food amount to `0.0`.

### Seeding

World initialization food seeding:
- clear prior food;
- enumerate non-barrier candidate cells;
- sample exactly
  `round(world.food.initial_coverage * candidate_count)` unique cells;
- set sampled cells to
  `clamp(world.food.initial_density, 0.0, world.food.max_density)`.

Startup flow and founder-baseline policy consuming these world seeding semantics
are canonical in `v3-startup-seeding-spec.md`.

---

## 6. Occupancy and Barrier Invariants

Spatial invariants:
- Single occupancy per cell (`Option<CreatureId>` semantics).
- A valid movement/spawn destination must be barrier-free and unoccupied at
  action-application time.
- Occupancy mutations are immediate and visible to later turns in the same
  tick (first-processed-wins behavior is owned by
  `v3-tick-orchestration-spec.md`).

Primitive intent:
- `place_creature` and `remove_creature` mutate occupancy index state.
- Callers must respect target-validity primitives from Section 7 before placing
  or moving creatures.

---

## 7. Action-Time Validity Primitives

Canonical primitive shape:

```text
resolve_neighbor(position, direction, edge_mode) -> Option<Position>
is_passable_cell(resolved_position, world_state_now) -> bool
is_valid_move_cell(resolved_position, world_state_now) -> bool
is_valid_spawn_cell(resolved_position, world_state_now) -> bool
```

Primitive responsibilities:
1. `resolve_neighbor` is the only primitive that applies edge-mode behavior.
2. `is_valid_move_cell` and `is_valid_spawn_cell` are cell-level checks and do
   not perform neighbor/edge resolution.
3. Cell-level checks require an in-bounds, resolved position.

Canonical move/spawn gate algorithm:
1. Resolve target neighbor using Section 4 edge-mode behavior.
2. If unresolved in `bounded` mode, target is invalid.
3. If resolved cell is barrier-blocked, target is invalid.
4. If resolved cell is occupied, target is invalid.
5. Otherwise target is valid.

Usage expectations:
- Move action resolves a target neighbor, then applies cell-level validity
  before occupancy mutation.
- Reproduce action resolves a target neighbor, then applies cell-level validity
  before offspring drafting and mutation.

Reproduction outcome reason mapping is owned by
`v3-reproduction-spec.md`.

---

## 8. Cross-Spec Ownership Map

- Tick phase order/arbitration: `v3-tick-orchestration-spec.md`
- Sensor category semantics: `v3-sensor-spec.md`
- Reproduction outcomes and rejection enums: `v3-reproduction-spec.md`
- Runtime/mutation/energy config defaults: `v3-runtime-config-spec.md`
- Startup seeding and founder baseline policy: `v3-startup-seeding-spec.md`
- Server frame transport schemas: `v3-server-api-protocol-spec.md`

This file remains canonical for world/grid config and spatial validity
semantics consumed by those specs.

---

## 9. Policy References

- Project-level determinism scope is canonical in `AGENTS.md`
  (`Determinism Scope (Canonical)`).
- V3 tick-order reproducibility controls are canonical in
  `v3-tick-orchestration-spec.md`.
