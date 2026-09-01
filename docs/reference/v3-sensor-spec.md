# V3 Sensor Spec

Reference specification for sensor and introspection input mapping in V3 mesh
execution.

Status: Active

Related references:
- `v3-creature-identity-spec.md`
- `v3-genome-spec.md`
- `v3-mesh-execution-spec.md`
- `v3-tick-orchestration-spec.md`
- `v3-vm-isa-spec.md`
- `v3-graph-backend-spec.md`
- `v3-mutation-spec.md`
- `v3-reproduction-spec.md`
- `v3-world-grid-spec.md`
- `v3-runtime-config-spec.md`

---

## 1. Unified InputReference

```rust
pub enum InputReference {
    World(WorldInputKey),
    StaticIntrospection(StaticIntrospectionKey),
    DynamicIntrospection(DynamicIntrospectionKey),
    UpstreamSlot(usize),
    ActionQueue,
}
```

All node backends consume the same `input_refs` vector.

### Compound vs Scalar Inputs

Most `InputReference` variants are **scalar**: they resolve to a single `f32`.
Compound variants resolve to multiple sub-values addressed via `sub_idx`.

Compound inputs in v3alpha1:
- `ActionQueue`
- `World(NeighborFoodRing { type_idx })` — 8 sub-values (N, NE, E, SE, S, SW, W, NW)
- `World(NeighborBarrierRing)` — 8 sub-values
- `World(NeighborOccupiedRing)` — 8 sub-values
- `World(AreaFoodSummary { type_idx })` — 7 sub-values
- `World(AreaBarrierSummary)` — 7 sub-values
- `World(AreaOccupancySummary)` — 7 sub-values
- `World(NearbyCreatureCore)` — 16 sub-values
- `World(NearbyCreatureVitals)` — 8 sub-values
- `World(NearbyCreatureIdentity)` — 12 sub-values

For scalar inputs, `sub_idx` is ignored (the scalar value is returned regardless).
For compound inputs, `sub_idx` wraps via modular arithmetic (`sub_idx % width`).

---

## 2. Sensor Snapshot Boundary

World and static-introspection values are resolved during cognition from the
frozen post-Phase-0 snapshot described in `v3-tick-orchestration-spec.md`.

Conceptual runtime-facing sensor bundle:

```rust
pub struct SensorSnapshot {
    pub local: StaticInputs,
    pub perception: PerceptionSnapshot,
}
```

Boundary rules:
- `sensors/` owns snapshot assembly.
- `runtime/` reads already-frozen values only.
- no backend reads live world state directly.
- dynamic introspection and action queue remain live runtime reads.

---

## 3. Categories

### 3.1 World

Canonical conceptual world keys:

```rust
pub enum WorldInputKey {
    FoodHere { type_idx: OrdinaryFoodTypeId },
    NeighborFoodRing { type_idx: OrdinaryFoodTypeId },
    NeighborBarrierRing,
    NeighborOccupiedRing,
    AreaFoodSummary { type_idx: OrdinaryFoodTypeId },
    AreaBarrierSummary,
    AreaOccupancySummary,
    NearbyCreatureCore,
    NearbyCreatureVitals,
    NearbyCreatureIdentity,
}
```

#### Local scalar world sensors

Resolved once per acting-creature turn from the frozen local snapshot:
- `FoodHere { type_idx }`: `clamp(food_density[self_cell], 0.0, 1.0)` → `[0.0, 1.0]`
  for the selected ordinary-food type.

Typed food sensors are bound to the configured ordinary-food type catalog for
the run. Invalid typed accesses resolve softly to `0.0` rather than hard
failing the mesh.

#### Ring sensors (compound, 8 sub-values)

Ring sensors provide neighbor cell data as compound inputs with 8 sub-values,
one per direction indexed by `Direction::to_index()`:
N=0, NE=1, E=2, SE=3, S=4, SW=5, W=6, NW=7.

- `NeighborFoodRing { type_idx }[dir]`: `clamp(food_density[neighbor], 0.0, 1.0)` → `[0.0, 1.0]`
- `NeighborBarrierRing[dir]`: `1.0` if barrier, else `0.0`
- `NeighborOccupiedRing[dir]`: `1.0` if occupied by any creature, else `0.0`

World neighbor semantics (coordinate system, direction mapping, and edge-mode
resolution) are canonical in `v3-world-grid-spec.md`.

#### Extended perception world sensors

These are fixed-width compound inputs assembled from the frozen extended
perception snapshot for the acting creature.

Shared posture:
- all values are finite `f32`
- all invalid/missing compound accesses return `0.0`
- area summaries use one global `runtime.perception.vision_radius`
- visibility considers only cells visible under the contract in Section 4

### 3.2 Static Introspection

Resolved once per acting-creature turn from current creature state:

```rust
pub enum StaticIntrospectionKey {
    Generation,
    AgeTicks,
}
```

Resolved values are raw integer casts to `f32`.

### 3.3 Dynamic Introspection

Resolved live during mesh evaluation:

```rust
pub enum DynamicIntrospectionKey {
    EnergyCurrent,
    EnergyConsumedThisTick,
    ReproductiveReserveCurrent,
}
```

Resolved values:
- `EnergyCurrent`: raw current energy, bounded by lifecycle config
- `EnergyConsumedThisTick`: raw energy consumed since turn start
- `ReproductiveReserveCurrent`: raw live reproductive reserve, clamped to the
  startup-configured `nutrition.reproductive_reserve_capacity`; it is resolved
  from current creature state during node execution.

### 3.4 Upstream Output

`UpstreamSlot(slot)` reads routing-parent output slots.
If `slot >= 12`, value is `0.0`.

### 3.5 Action Queue (Compound)

`ActionQueue` exposes queue contents via two-level indexing:
- `sub_idx / 3` = queue slot
- `sub_idx % 3`: `0` = action_type, `1` = param0, `2` = param1

Total sub-values: `action_queue_cap * 3` (default `12`).

---

## 4. Extended Perception Visibility Contract

Canonical config owner for radius:
- `runtime.perception.vision_radius` in `v3-runtime-config-spec.md`
- default `5`
- valid range `1..=8`

Candidate window:
- all local offsets `(dx, dy)` where `dx in [-r, r]` and `dy in [-r, r]`
- self cell `(0, 0)` is included for food calculations
- self cell is excluded for non-self occupancy and nearby-creature ranking

Opacity rules:
- barriers are visible and opaque
- food is visible and non-opaque
- creatures are visible and non-opaque

LOS posture:
- visibility is computed in local-offset space
- `v3-world-grid-spec.md` owns `resolve_offset(origin, dx, dy, edge_mode)`
- sensors own opacity, strict-corner policy, and visible-cell contribution rules
- each local ray step is resolved through `resolve_offset`
- in wrap worlds, wrapping applies per step, not only at the final target

Strict-corner rule:
- for a diagonal step `(sx, sy)`, inspect the two orthogonal side cells
  `(x + sx, y)` and `(x, y + sy)`
- if both side cells are blocking barriers, diagonal advance is blocked
- if a side cell is unresolved in bounded mode, treat it as blocking for the
  corner test

Barrier visibility rule:
- if LOS reaches a barrier cell, that barrier cell is visible and contributes to
  barrier summaries
- cells beyond it on that ray are hidden

Hidden entities contribute nothing to area summaries or nearby-creature banks.

---

## 5. Extended Perception Compound Layout

### 5.1 `AreaFoodSummary`

`sub_value_count = 7`

`AreaFoodSummary { type_idx }` uses the same 7-field layout, but each summary
is computed for a single configured ordinary-food type.

| `sub_idx` | field |
| --- | --- |
| 0 | `total_ratio` |
| 1 | `gradient_x` |
| 2 | `gradient_y` |
| 3 | `nearest_dx` |
| 4 | `nearest_dy` |
| 5 | `nearest_dist` |
| 6 | `max_value` |

### 5.2 `AreaBarrierSummary`

`sub_value_count = 7`

| `sub_idx` | field |
| --- | --- |
| 0 | `density_ratio` |
| 1 | `blocked_adjacent_ratio` |
| 2 | `gradient_x` |
| 3 | `gradient_y` |
| 4 | `nearest_dx` |
| 5 | `nearest_dy` |
| 6 | `nearest_dist` |

### 5.3 `AreaOccupancySummary`

`sub_value_count = 7`

| `sub_idx` | field |
| --- | --- |
| 0 | `count_ratio` |
| 1 | `center_x` |
| 2 | `center_y` |
| 3 | `nearest_dx` |
| 4 | `nearest_dy` |
| 5 | `nearest_dist` |
| 6 | `crowding_ratio` |

### 5.4 `NearbyCreatureCore`

4 ranked slots × 4 fields = `16`

Per-slot order:
- `present`
- `rel_x`
- `rel_y`
- `dist`

### 5.5 `NearbyCreatureVitals`

4 ranked slots × 2 fields = `8`

Per-slot order:
- `energy_ratio`
- `reproduce_ready`

### 5.6 `NearbyCreatureIdentity`

4 ranked slots × 3 fields = `12`

Per-slot order:
- `kin_affinity`
- `lineage_match`
- `phenotype_similarity`

All nearby-creature banks share the same slot ordering.

Ranking order:
1. ascending Euclidean distance
2. ascending Chebyshev distance
3. ascending `(dy, dx)`
4. ascending `CreatureId`

Only visible non-self creatures participate.

---

## 6. Field Definitions and Normalization

Let:
- `r = runtime.perception.vision_radius as f32`
- `max_candidate_cells = (2r + 1)^2`
- `max_other_candidate_cells = max_candidate_cells - 1`
- `max_dist = sqrt(2 * r * r)`
- `dx_norm = dx as f32 / r`
- `dy_norm = dy as f32 / r`
- `dist_norm = euclidean_distance(dx, dy) / max_dist`

Area-summary denominator rule:
- density and gradient denominators use full candidate-window maxima, not
  currently visible cell count

### 6.1 Food

- `total_ratio = visible_food_sum / (max_candidate_cells * world.food.shared.max_density)`
- `gradient_x = sum(dx_norm * food_ratio) / max_candidate_cells`
- `gradient_y = sum(dy_norm * food_ratio) / max_candidate_cells`
- `max_value = max visible food ratio`
- nearest fields use the nearest visible cell with `food_ratio > 0.0`

### 6.2 Barriers

- `density_ratio = visible_barrier_count / max_candidate_cells`
- `blocked_adjacent_ratio = adjacent visible barrier count among 8 immediate neighbors / 8.0`
- gradients use barrier presence `0/1`
- nearest fields use the nearest visible barrier

### 6.3 Occupancy

- `count_ratio = visible non-self creature count / max_other_candidate_cells`
- `center_x`, `center_y` are means of normalized visible non-self offsets
- `crowding_ratio = sum(1.0 - dist_norm) / max_other_candidate_cells`
- nearest fields use the nearest visible non-self creature

### 6.4 Nearby Creature Vitals

- `energy_ratio = clamp(target.energy / energy.lifecycle.max_energy, 0.0, 1.0)`
- `reproduce_ready = 1.0` if
  `target.energy >= energy.lifecycle.min_reproduce_energy` and
  `target.age >= energy.lifecycle.min_reproduce_age`, else `0.0`

### 6.5 Nearby Creature Identity

- `lineage_match = 1.0` if lineage IDs match, else `0.0`
- `kin_affinity = 1.0 - (popcount(observer.kin_tag ^ target.kin_tag) / 32.0)`
- `phenotype_similarity = 1.0 - (mean_abs_channel_delta / 255.0)`

Clamp identity similarity values to `[0.0, 1.0]`.

Missing/absent conventions:
- out-of-range compound `sub_idx` returns `0.0`
- missing nearby-creature slots return `0.0` for all fields
- nearest-field absence convention is `nearest_dx = 0.0`,
  `nearest_dy = 0.0`, `nearest_dist = 0.0`

This absence convention is acceptable because:
- barriers cannot occupy the observer cell
- non-self creatures cannot occupy the observer cell
- food-on-self remains disambiguated by `FoodHere { type_idx }`

---

## 7. Resolution Timing

Per acting-creature turn, canonical timing is:
1. Build frozen local and extended perception snapshots from the post-Phase-0
   world state.
2. Execute runtime mesh evaluation against those frozen snapshots.
3. Resolve dynamic introspection and action queue live during evaluation.

Canonical phase order remains in `v3-tick-orchestration-spec.md`.

---

## 8. Soft Defaults

- Missing `input_refs` index: `0.0`
- Invalid upstream slot: `0.0`
- Compound input out-of-bounds `sub_idx`: wraps via `sub_idx % compound_width()`
- Scalar input with any `sub_idx`: returns the scalar value (sub_idx is ignored)
- Unknown/unsupported key variant at runtime boundary: `0.0`

Soft defaults are deliberate to support junk-DNA evolution without crashes.

---

## 9. Backend Access Paths and Energy Cost

- VM: `ReadInput { dst, ref_idx, sub_idx }`
- Graph: `GraphSource::InputLeaf { ref_idx, sub_idx }` (implicit edge source)

Backend posture:
- backends address the same compound layout using `(ref_idx, sub_idx)`
- Graph backend uses `GraphSource::InputLeaf { ref_idx, sub_idx }` as edge
  sources (implicit inputs, not physical nodes)
- no backend reads world state directly

Energy-cost decision for v1:
- extended perception uses the same `ReadInput` energy cost as existing world
  inputs
- there is no extra creature-energy debit for larger perception families

---

## 10. Resolution Context (ResolveCtx)

Input resolution uses a shared context containing the unified sensor snapshot:

```rust
pub struct ResolveCtx<'a> {
    pub sensors: &'a SensorSnapshot,
    pub upstream_slots: &'a [f32; 12],
    pub energy: f32,
    pub energy_consumed: f32,
    pub action_queue: &'a ActionQueue,
}
```

This struct is shared between graph evaluation and VM execution, ensuring
consistent resolution semantics across backends.
