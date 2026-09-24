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
- canonical world/grid configuration keys, defaults, normalization, and startup terrain;
- world coordinate and direction mapping;
- edge-mode behavior (`wrap` and `bounded`);
- arbitrary local-offset resolution primitives consumed by perception and action
  targeting;
- ordinary-food substrate growth/consume/seeding semantics;
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

General local-offset resolution uses the same coordinate system and edge-mode
contract:

```text
resolve_offset(origin, dx, dy, edge_mode) -> Option<Position>
```

Rules:
- resolves arbitrary signed local offsets, not only 8-neighbor directions;
- `wrap` applies modulo arithmetic per resolved step;
- `bounded` returns `None` when the resolved position falls outside world
  bounds;
- this is the only geometry primitive that converts local signed offsets into
  world positions for perception and other higher-level consumers.

---

## 4. World Config Contract (Canonical Owner)

This file is the canonical owner for world/grid config keys/defaults.

| Key | Type | Default | Constraint / normalization |
| --- | --- | --- | --- |
| `world.width` | `u16` | `1600` | Must be `>= 1`; invalid values fall back to `1600`. |
| `world.height` | `u16` | `1600` | Must be `>= 1`; invalid values fall back to `1600`. |
| `world.edge_mode` | `enum{wrap,bounded}` | `wrap` | Unknown/invalid values fall back to `wrap`. |
| `world.terrain` | `TerrainLayer[]` | `[]` | Ordered additive barrier layers; absent defaults to empty. |
| `world.world_seed` | `Option<u64>` | absent/null | Effective terrain/fertility seed is this value or the run seed. |
| `world.food.shared.growth_rate` | `f32` | `0.09` | Must be finite; clamp to `[0.0, 1.0]`; invalid falls back to `0.09`. |
| `world.food.shared.spread_threshold_ratio` | `f32` | `0.8` | Must be finite; clamp to `[0.0, 1.0]`; invalid falls back to `0.8`. |
| `world.food.shared.spread_density_ratio` | `f32` | `0.25` | Must be finite; clamp to `[0.0, 1.0]`; invalid falls back to `0.25`. Fraction of growth delta deposited to neighbor during spread. |
| `world.food.shared.recovery_spawn_rate` | `f32` | `0.01` | Must be finite; clamp to `[0.0, 1.0]`; invalid falls back to `0.01`. |
| `world.food.shared.recovery_floor_ratio` | `f32` | `0.01` | Must be finite; clamp to `[0.0, 1.0]`; invalid falls back to `0.01`. |
| `world.food.shared.max_density` | `f32` | `1.0` | Must be finite and `> 0.0`; invalid falls back to `1.0`. |
| `world.food.shared.occupancy_depletion.enabled` | `bool` | `true` | Enables the occupancy depletion mask that dampens food regrowth on occupied cells. |
| `world.food.shared.occupancy_depletion.deposit_per_occupied_tick` | `f32` | `0.08` | Must be finite; clamp to `[0.0, 1.0]`; invalid falls back to `0.08`. Amount of depletion deposited into each occupied passable cell per Phase 0 update. |
| `world.food.shared.grazing.enabled` | `bool` | `true` | Enables the grazing fertility modifier (T02.F04): one `[floor, 1.0]` value per food type per cell, multiplied into the mapped fertility at the local-growth source, the spread target, and the recovery spawn. Flipping it resets every modifier to `1.0`; disabled reads `1.0` and records no bite. |
| `world.food.shared.grazing.factor` | `f32` | `0.5` | Must be finite; clamp to `[0.0, 1.0]`; invalid falls back to `0.5`. Each consuming bite (a positive `consume_type`/`consume_any` removal) sets the type's modifier at that cell to `max(floor, modifier * factor)`. |
| `world.food.shared.grazing.floor` | `f32` | `0.05` | Must be finite; clamp to `[0.0, 1.0]`; invalid falls back to `0.05`. `floor <= factor` is not required. |
| `world.food.shared.grazing.recovery_ticks` | `u32` | `1000` | Clamped to `[1, 10000]`; zero or missing falls back to `1000`. At the start of every Phase 0 growth pass each non-barrier cell of every type advances `min(1.0, modifier + 1 / recovery_ticks)`, empty cells included; barrier cells stay `1.0`. |
| `world.food.types` | `FoodTypeConfig[]` | `[Primary Food]` | Ordered list of configured ordinary-food types. List position is the stable per-run `OrdinaryFoodTypeId`; each entry carries display metadata and startup seeding knobs. Empty lists normalize to one green Primary Food with density `1.0` and coverage `0.54`. |
| `world.food.types[].initial_density` | `f32` | `1.0` | Clamp to `[0.0, world.food.shared.max_density]`; invalid falls back to `max_density`. |
| `world.food.types[].initial_coverage` | `f32` | `0.54` | Must be finite; clamp to `[0.0, 1.0]`; invalid falls back to `0.54`. |
| `world.food.fertility.layers[].target` | `enum{AllFoods,SingleType{type_idx}}` | `AllFoods` | Fertility layer selector. Invalid/unknown targeted type indices normalize to `AllFoods` during config normalization. |

Terrain layers contain required `params: PatternParams` and optional/null `bounds:
PatternBounds` and `seed: u64`; unknown layer fields are rejected. The existing
`Maze`, `Spiral`, `Noise`, `ParallelLines`, and `Star` tagged parameter variants
and their normalization are reused. Absent bounds cover the whole world;
explicit `u16` rectangles intersect the world without wrapping, with saturating
subtraction from their origin before clipping width/height. Empty intersections
produce no barriers. Layers union through `set_barrier`; they never erase.
Startup terrain is not subject to the runtime HTTP pattern area's 1,000,000-cell
cap. Layer index `i` uses its explicit seed or effective map seed plus `i`,
wrapping in `u64`. Terrain and fertility derive independently and consume no
run-RNG draws. The map seed fixes barriers/fertility, while food placement,
founders, and runtime draws still depend on the run seed. Both new world fields
are restart-only, including null/empty PATCH values.

Food-type catalog posture:
- `world.food.types` is ordered, and the list index defines the stable per-run
  `OrdinaryFoodTypeId`.
- `FoodTypeConfig` carries display metadata (`name`, `color`), startup seeding fields
  (`initial_density`, `initial_coverage`, `initial_fertility_only`), and
  `growth_inhibitor`. Optional/null `energy_per_unit`, `growth_rate` and
  `recovery_spawn_rate` inherit `energy.costs.eat_reward_per_food`,
  `world.food.shared.growth_rate` and `world.food.shared.recovery_spawn_rate`.
  Explicit zero overrides. Finite reward clamps nonnegative; finite rates
  clamp to `[0,1]`; nonfinite overrides become inheritance. Save/load retains
  inheritance, so later shared runtime edits remain live. Typed Eat multiplies
  consumed density by the effective reward, then caps energy and charges the
  existing action cost. Growth/recovery resolve once per type and retain the
  existing local/spread/recovery formulas and suppression telemetry.
- Initial density and coverage live only on each `world.food.types[]` entry,
  and fertility and annealing only at `world.food.fertility` and
  `world.food.annealing`; `world.food.shared` carries no copy of them and
  rejects those keys as unknown fields.
- Default types are seeded independently: each type shuffles the passable-cell
  candidate list separately and takes its own rounded coverage target. A shared
  ordering is not reused across types, so equal coverages do not imply equal
  spatial placement.
- `initial_fertility_only` defaults false. When true, initial coverage counts
  only passable cells whose tick-zero effective fertility is strictly positive,
  including annealing. Disabled fertility admits all passable cells. No eligible
  cells means no food; this does not prohibit later growth. False preserves the
  original candidate order, shuffle and RNG draws.
- `FbmThreshold` terrain uses the existing raw fBm field with one pattern RNG
  u64 seed draw. The field is sampled at world coordinates
  (`bounds.x + x`, `bounds.y + y`), so a bounded layer equals the whole-world
  layer at the same seed cut to its bounds and nested same-seed layers grade one
  region. Values strictly above `threshold` become barriers, clipped to the
  coordinate space without wrapping. Parameters normalize: octaves
  `1..=32` (default 4), frequency `[0.000001,1]` (0.02), lacunarity `[1,4]`
  (2), persistence `[0,1]` (0.5), threshold `[-1,1]` (0); nonfinite floats use
  the listed defaults. Existing layer seeds and barrier union semantics apply.
- Type-targeted fertility layers are startup-only config; runtime patching does
  not mutate the catalog or layer targets.

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
densities. Occupancy does not directly block food growth; instead, the food
kernel maintains an internal occupancy depletion layer that suppresses
regrowth on currently occupied cells and then recovers over time.

For each non-barrier cell:
- `source = clamp(snapshot[cell], 0.0, max_density)`.
- Apply occupancy depletion recovery/deposit for the current tick using the
  current creature occupancy mask.
- Compute an occupancy multiplier from the depletion layer for the cell.
- `delta = source * effective_type_growth_rate`.
- Apply local growth:
  `food_density[cell] = clamp(food_density[cell] + delta * occupancy_multiplier, 0.0, max_density)`.
- If `source >= max_density * world.food.shared.spread_threshold_ratio` and
  `delta > 0.0`, pick one random valid cardinal neighbor (non-barrier) and add
  `delta * world.food.shared.spread_density_ratio * occupancy_multiplier_at_target`
  (clamped to `max_density`).

After local growth/spread pass:
- Compute `average_density_ratio = sum(snapshot_food) / (total_cells * max_density)`.
- If `average_density_ratio < world.food.shared.recovery_floor_ratio`, run
  `round(total_cells * effective_type_recovery_spawn_rate)` recovery attempts.
- Each attempt picks one random non-barrier cell and adds
  `max_density * effective_type_growth_rate * occupancy_multiplier` (clamped).

Barrier cells are excluded from growth/spread/recovery targets.
Creature occupancy does not block food growth/spread/recovery directly in
v3alpha1; it only modulates regrowth through the depletion layer.

### Consumption

`consume_food(cell)` semantics:
- returns the cell's current food amount;
- sets the cell food amount to `0.0`.

### Seeding

World initialization food seeding:
- clear prior food;
- enumerate non-barrier candidate cells;
- for each food type `t` in `world.food.types`, shuffle the candidates
  (restricted to fertile cells when `t.initial_fertility_only` is set and
  fertility is enabled) independently and sample exactly
  `round(t.initial_coverage * candidate_count)` unique cells;
- set that type's density on its sampled cells to
  `clamp(t.initial_density, 0.0, world.food.shared.max_density)`.

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
resolve_offset(position, dx, dy, edge_mode) -> Option<Position>
is_passable_cell(resolved_position, world_state_now) -> bool
is_valid_move_cell(resolved_position, world_state_now) -> bool
is_valid_spawn_cell(resolved_position, world_state_now) -> bool
```

Primitive responsibilities:
1. `resolve_neighbor` and `resolve_offset` are the only primitives that apply
   edge-mode behavior.
2. `resolve_offset` is the arbitrary-offset analogue used by perception and
   other local-space consumers.
3. `is_valid_move_cell` and `is_valid_spawn_cell` are cell-level checks and do
   not perform neighbor/edge resolution.
4. Cell-level checks require an in-bounds, resolved position.

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
- Perception and LOS consumers resolve each local ray step through
  `resolve_offset(...)` and then apply their own visibility policy.

Reproduction outcome reason mapping is owned by
`v3-reproduction-spec.md`.

---

## 8. Perception Geometry Ownership

This file owns geometry and edge-mode behavior only.

Owned here:
- coordinate system and direction mapping
- `resolve_neighbor(...)`
- `resolve_offset(...)`
- per-step wrap/bounded resolution semantics for local-space consumers

Not owned here:
- opacity policy
- strict-corner visibility rules
- which visible cell contents contribute to sensor summaries

Those perception-policy concerns are owned by `v3-sensor-spec.md`.

---

## 9. Cross-Spec Ownership Map

- Tick phase order/arbitration: `v3-tick-orchestration-spec.md`
- Sensor category semantics: `v3-sensor-spec.md`
- Reproduction outcomes and rejection enums: `v3-reproduction-spec.md`
- Runtime/mutation/energy config defaults: `v3-runtime-config-spec.md`
- Startup seeding and founder baseline policy: `v3-startup-seeding-spec.md`
- Server frame transport schemas: `v3-server-api-protocol-spec.md`

This file remains canonical for world/grid config and spatial validity
semantics consumed by those specs.

---

## 10. Policy References

- Project-level determinism scope is canonical in root `AGENTS.md`.
- V3 tick-order reproducibility controls are canonical in
  `v3-tick-orchestration-spec.md`.
