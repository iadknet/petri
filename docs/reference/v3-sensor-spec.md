# V3 Sensor Spec

Reference specification for sensor and introspection input mapping in V3 mesh
execution.

Status: Active

Related references:
- `v3-genome-spec.md`
- `v3-mesh-execution-spec.md`
- `v3-tick-orchestration-spec.md`
- `v3-vm-isa-spec.md`
- `v3-graph-backend-spec.md`
- `v3-mutation-spec.md`
- `v3-reproduction-spec.md`
- `v3-world-grid-spec.md`

---

## 1. Unified InputReference

```rust
pub enum InputReference {
    World(WorldInputKey),
    StaticIntrospection(StaticIntrospectionKey),
    DynamicIntrospection(DynamicIntrospectionKey),
    UpstreamSlot(usize),
}
```

All node backends consume the same `input_refs` vector.

---

## 2. Categories

### World

Resolved once at the start of each acting-creature turn from current world
state:

```rust
pub enum WorldInputKey {
    FoodHere,
    NeighborCellFood(Direction),
    NeighborCellBarrier(Direction),
    NeighborCellOccupied(Direction),
}
```

Resolved f32 values per key:
- `FoodHere`: `clamp(food_density[self_cell], 0.0, 1.0)` → `[0.0, 1.0]`
- `NeighborCellFood(dir)`: `clamp(food_density[neighbor], 0.0, 1.0)` → `[0.0, 1.0]`
- `NeighborCellBarrier(dir)`: `1.0` if barrier, else `0.0`
- `NeighborCellOccupied(dir)`: `1.0` if occupied by any creature, else `0.0`

Sensor rationale: food substrate is represented as continuous `f32` and sensor
contracts stay bounded to `[0.0, 1.0]` for stable cognition inputs. Eat action
reward mechanics still use the raw consumed food amount as defined in
`v3-runtime-config-spec.md` (`eat_reward_per_food`).

World neighbor semantics (coordinate system, direction mapping, and edge-mode
resolution) are canonical in `v3-world-grid-spec.md`.

### Static Introspection

Resolved once at the start of each acting-creature turn from current creature
state:

```rust
pub enum StaticIntrospectionKey {
    Generation,
    AgeTicks,
}
```

Resolved f32 values: raw integer cast to f32. Values are unbounded and
increase monotonically over the creature's lifetime.

### Dynamic Introspection

Resolved live during mesh evaluation:

```rust
pub enum DynamicIntrospectionKey {
    EnergyCurrent,
    EnergyConsumedThisTick,
}
```

Resolved f32 values: raw energy units (same scale as `energy.*` config fields
in `v3-runtime-config-spec.md`). `EnergyCurrent` is in `[0.0, max_energy]`;
`EnergyConsumedThisTick` accumulates action/cognition costs since turn start.

These values may change between node hops during the same tick.

### Upstream Output

`UpstreamSlot(slot)` reads routing-parent output slots.
If `slot >= 12`, value is `0.0`.

---

## 3. Resolution Timing

Per acting-creature turn, `tick/orchestrator` flow:
1. Read world/static-introspection values from current world/creature state at
   turn start (using world/grid semantics from `v3-world-grid-spec.md`).
2. Call runtime mesh executor.
3. Runtime resolves dynamic introspection and upstream slots per node evaluation.

This split keeps borrow boundaries explicit and easy to validate in tests.

Canonical turn timing and action-application order are specified in
`v3-tick-orchestration-spec.md`.

---

## 4. Soft Defaults

- Missing `input_refs` index: `0.0`.
- Invalid upstream slot: `0.0`.
- Unknown/unsupported key variant at runtime boundary: `0.0`.

Soft defaults are deliberate to support junk-DNA evolution without crashes.

---

## 5. Backend Access Paths

- VM: `ReadInput(dst, idx)` reads `input_refs[idx]`.
- Graph: `InputRef(idx)` reads `input_refs[idx]`.

No backend reads world state directly. All access is through `InputReference`
runtime dataflow.
