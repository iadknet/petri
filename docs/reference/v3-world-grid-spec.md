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
    food_density: Grid<u8>,
    barriers: Grid<bool>,
    creature_at: Grid<Option<CreatureId>>,
}
```

State semantics:
- `food_density` is per-cell scalar in `[0, 255]`.
- `barriers` marks non-passable cells.
- `creature_at` is single-occupancy: at most one creature per cell.
- Food/barrier/occupancy values are independent stored state, but movement and
  spawn validity primitives enforce barrier and occupancy constraints.

Storage layout (flat vector, row-major, etc.) is implementation detail as long
as these semantics are preserved.

---

## 3. Coordinate and Direction Semantics

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
| `world.food.growth_rate` | `f32` | `0.02` | Must be finite; clamp to `[0.0, 1.0]`; invalid falls back to `0.02`. |
| `world.food.initial_density` | `u8` | `80` | Clamp to `[0, 255]`; invalid falls back to `80`. |
| `world.food.initial_coverage` | `f32` | `0.3` | Must be finite; clamp to `[0.0, 1.0]`; invalid falls back to `0.3`. |

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

Per-cell food is quantized as `u8` in `[0, 255]`.

### Growth

At Phase 0 (tick start), each non-barrier cell independently rolls a Bernoulli
trial with `world.food.growth_rate`.

If successful:
- `food_density[cell] = saturating_add(food_density[cell], 1)`.

Barrier cells do not grow food in baseline behavior.

### Consumption

`consume_food(cell)` semantics:
- returns the cell's current food amount;
- sets the cell food amount to `0`.

### Seeding

World initialization food seeding:
- each non-barrier cell independently rolls with
  `world.food.initial_coverage`;
- on success, set cell food to `world.food.initial_density`.

No diffusion/spread mechanic is defined in this baseline.
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
