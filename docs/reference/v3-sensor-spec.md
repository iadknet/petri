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
    ActionQueue,
}
```

All node backends consume the same `input_refs` vector.

### Compound vs Scalar Inputs

Most `InputReference` variants are **scalar**: they resolve to a single `f32`.
**Compound** variants (currently `ActionQueue`) resolve to multiple sub-values
addressed via a `sub_idx` parameter. For scalar inputs, `sub_idx > 0` returns
`0.0`. Backends use two-level indexing `(ref_idx, sub_idx)` to address
sub-values within a compound input.

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

### Action Queue (Compound)

`ActionQueue` is a compound input that exposes the creature's action queue
contents via two-level sub-value addressing:

- `sub_idx / 3` = queue slot index
- `sub_idx % 3`: `0` = action_type, `1` = param0, `2` = param1

Out-of-bounds queue indices return `0.0`. The total number of sub-values
is `action_queue_cap * 3` (default: `4 * 3 = 12`).

Resolution uses `ResolveCtx.action_queue`, which holds the live action
queue state during mesh evaluation.

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
- Compound input out-of-bounds sub_idx: `0.0`.
- Scalar input with `sub_idx > 0`: `0.0`.
- Unknown/unsupported key variant at runtime boundary: `0.0`.

Soft defaults are deliberate to support junk-DNA evolution without crashes.

---

## 5. Backend Access Paths

- VM: `ReadInput { dst, ref_idx, sub_idx }` reads `input_refs[ref_idx]`
  with sub-value index `sub_idx`.
- Graph: `InputRef { ref_idx, sub_idx }` reads `input_refs[ref_idx]`
  with sub-value index `sub_idx`.

No backend reads world state directly. All access is through `InputReference`
runtime dataflow.

---

## 6. Resolution Context (ResolveCtx)

Input resolution uses a shared `ResolveCtx` struct containing:

```rust
pub struct ResolveCtx<'a> {
    pub static_inputs: &'a StaticInputs,
    pub upstream_slots: &'a [f32; 12],
    pub energy: f32,
    pub energy_consumed: f32,
    pub action_queue: &'a ActionQueue,
}
```

This struct is shared between graph evaluation (via `EvalCtx`) and VM
execution, ensuring consistent resolution semantics across backends.
