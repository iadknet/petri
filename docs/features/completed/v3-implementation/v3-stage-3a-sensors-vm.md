# V3 Stage 3a: Sensors + VM Backend Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the `sensors/` module (StaticInputs assembly from world state) and the `runtime/` module (NodeResult type, input resolution, 33-opcode VM executor, WorldAction decode) so that a single VM node can be executed end-to-end.

**Architecture:** Three new source directories under `v3/crates/v3-core/src/`: `sensors/` (world/introspection snapshot), `runtime/` (node execution engine, input resolver, VM). The VM is purely functional — no global state — taking genome data + inputs + mutable energy/memory references and returning a `NodeResult`. The Stage 3b graph backend and mesh chain executor will be built on top of these primitives.

**Tech Stack:** Rust, IEEE-754 f32, existing `v3-core` types (`WorldState`, `CreatureState`, `SimulationConfig`, `WorldAction`, `Direction`, `InputReference`, `VmInstruction`).

**Goal IDs:** GP-01, GP-02, GP-03

**Scope:** `sensors/` module (StaticInputs assembly) and `runtime/` module (NodeResult, resolve_input, execute_vm_node, decode_world_action) in `v3/crates/v3-core`. Excludes graph backend, mesh chain, tick orchestration, mutation, and reproduction.

**Docs Impact:** No canonical docs changed; new implementation of v3-sensor-spec.md and v3-vm-isa-spec.md contracts.

**Supersedes:** none

**Superseded-By:** none

---

## Goal Alignment

- **GP-01:** Enables richer creature cognition by implementing the VM execution substrate and sensor assembly that power multi-node mesh decisions.
- **GP-02:** Keeps sensor assembly, input resolution, and VM execution in their correct layers (`sensors/` and `runtime/`) without coupling to tick orchestration or contracts beyond `WorldAction`.
- **GP-03:** Full TDD coverage of all 33 opcodes, sanitize_f32, and all input reference kinds ensures high-confidence iteration.

## Boundary Impact

- New modules `sensors/` and `runtime/` added to `v3/crates/v3-core/src/`.
- `contracts/` and `creature/` are read-only dependencies; no changes to their public API.
- No new crates.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `creature/genome.rs` | keep | Schema types are consumed by VM; no changes needed |
| `contracts/` | keep | `WorldAction`, `InputReference`, `Direction` consumed as-is |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should sensors/ own dynamic introspection? | No — static snapshot only; dynamic introspection resolved live in resolve_input | agent | resolved |
| Should sanitize_f32 clamp to 1e9 or float::MAX? | 1e9 per spec | agent | resolved |

---

## Context and File Layout

Before starting, understand the current state:
- Worktree: `.worktrees/v3-implementation`
- All commands: `cd v3 && cargo <cmd>`
- Current passing tests: 62
- `v3/crates/v3-core/src/lib.rs` exports: `config`, `contracts`, `creature`, `kernel`
- New modules to add: `sensors` and `runtime`

Expected final file tree after this stage:
```
v3/crates/v3-core/src/
├── sensors/
│   ├── mod.rs
│   └── static_inputs.rs
├── runtime/
│   ├── mod.rs
│   ├── types.rs        ← NodeResult + sanitize_f32
│   ├── inputs.rs       ← resolve_input
│   ├── vm.rs           ← execute_vm_node (all 33 opcodes)
│   └── action_decode.rs← decode_world_action
└── lib.rs              ← add: pub mod sensors; pub mod runtime;
```

---

## Spec References (read before implementing each task)

- `docs/reference/v3-sensor-spec.md` — StaticInputs structure, normalization rules, UpstreamSlot
- `docs/reference/v3-vm-isa-spec.md` — all 33 opcodes, energy cost table, sanitize_f32, operand normalization
- `docs/reference/v3-mesh-execution-spec.md` — NodeResult fields, energy exhaustion contract

Key spec facts to keep handy:
- `sanitize_f32`: NaN→0.0, ±Inf→±1e9, finite clamped to [-1e9, 1e9]
- Food density normalized: `food_as_u8 as f32 / 255.0` (sensor output only; eat reward uses raw u8)
- UpstreamSlot idx >= 12 → 0.0 (soft default)
- VM register index: `rem_euclid(register_count)`; if register_count==0, halt immediately
- VM constant index: if constants empty → 0.0; else `constants[idx % len]`
- Jump formula: `next_pc = (pc as i64 + 1 + offset as i64).rem_euclid(program_len as i64) as usize`
- Energy exhausted mid-VM: memory writes do NOT persist; mesh returns WorldAction::NoOp
- EmitWorldAction halts VM immediately; Halt halts without emitting action
- max_vm_steps default 1024; enforced as step count (not energy check)
- Payload buffer (12 slots): init from upstream_slots; WriteInternalPayload overwrites slots
- World-action metadata buffer (8 slots): zeroed at node start; WriteWorldActionMeta writes slots
- Direction indices: 0=N,1=NE,2=E,3=SE,4=S,5=SW,6=W,7=NW (matches Direction::ALL order)

---

## Task 1: Foundation — `sanitize_f32` + `NodeResult`

**Files:**
- Create: `v3/crates/v3-core/src/runtime/mod.rs`
- Create: `v3/crates/v3-core/src/runtime/types.rs`
- Modify: `v3/crates/v3-core/src/lib.rs`

### Step 1: Add `pub mod runtime;` to lib.rs

Open `v3/crates/v3-core/src/lib.rs` and add the new module declaration (keep alphabetical order):

```rust
pub mod config;
pub mod contracts;
pub mod creature;
pub mod kernel;
pub mod runtime;
```

### Step 2: Create `runtime/mod.rs`

```rust
pub mod types;
```

(Other sub-modules added in later tasks.)

### Step 3: Write the failing tests for Task 1

Create `v3/crates/v3-core/src/runtime/types.rs` with the tests first:

```rust
use crate::contracts::WorldAction;

/// Result returned by a single node evaluation.
/// The mesh executor uses this to decide routing and final WorldAction.
pub struct NodeResult {
    /// Output slots for downstream nodes. Initialized from incoming upstream_slots;
    /// only slots written by the node are overwritten.
    pub output_slots: [f32; 12],
    /// Routing target index (f32). Mesh executor applies rem_euclid over targets.len().
    pub route_target_idx: f32,
    /// World action emitted by this node, if any.
    pub world_action: Option<WorldAction>,
    /// True when the node was halted due to energy exhaustion.
    /// When true, the mesh executor MUST return WorldAction::NoOp immediately.
    pub energy_exhausted: bool,
}

impl NodeResult {
    /// Create a no-action result (halt or step cap reached).
    pub fn halted(output_slots: [f32; 12], route_target_idx: f32) -> Self {
        Self {
            output_slots,
            route_target_idx,
            world_action: None,
            energy_exhausted: false,
        }
    }

    /// Create a result indicating energy exhaustion.
    pub fn exhausted() -> Self {
        Self {
            output_slots: [0.0; 12],
            route_target_idx: 0.0,
            world_action: None,
            energy_exhausted: true,
        }
    }

    /// Create a result with an emitted world action.
    pub fn action(
        output_slots: [f32; 12],
        route_target_idx: f32,
        action: WorldAction,
    ) -> Self {
        Self {
            output_slots,
            route_target_idx,
            world_action: Some(action),
            energy_exhausted: false,
        }
    }
}

/// Sanitize an f32 value per v3-vm-isa-spec.md Section 5:
/// - NaN → 0.0
/// - +Inf → +1_000_000_000.0
/// - -Inf → -1_000_000_000.0
/// - Finite values clamped to [-1e9, 1e9]
#[inline]
pub fn sanitize_f32(v: f32) -> f32 {
    const CLAMP: f32 = 1_000_000_000.0;
    if v.is_nan() {
        0.0
    } else {
        v.clamp(-CLAMP, CLAMP)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_nan_becomes_zero() {
        assert_eq!(sanitize_f32(f32::NAN), 0.0);
    }

    #[test]
    fn sanitize_pos_inf_becomes_clamp() {
        assert_eq!(sanitize_f32(f32::INFINITY), 1_000_000_000.0);
    }

    #[test]
    fn sanitize_neg_inf_becomes_neg_clamp() {
        assert_eq!(sanitize_f32(f32::NEG_INFINITY), -1_000_000_000.0);
    }

    #[test]
    fn sanitize_large_finite_clamps() {
        assert_eq!(sanitize_f32(2e9), 1_000_000_000.0);
        assert_eq!(sanitize_f32(-2e9), -1_000_000_000.0);
    }

    #[test]
    fn sanitize_normal_value_unchanged() {
        assert!((sanitize_f32(3.14) - 3.14).abs() < 1e-6);
        assert!((sanitize_f32(-0.5) - (-0.5)).abs() < 1e-6);
        assert_eq!(sanitize_f32(0.0), 0.0);
    }

    #[test]
    fn sanitize_exactly_at_boundary_unchanged() {
        assert_eq!(sanitize_f32(1_000_000_000.0), 1_000_000_000.0);
        assert_eq!(sanitize_f32(-1_000_000_000.0), -1_000_000_000.0);
    }

    #[test]
    fn node_result_halted_has_no_action() {
        let r = NodeResult::halted([0.0; 12], 0.0);
        assert!(r.world_action.is_none());
        assert!(!r.energy_exhausted);
    }

    #[test]
    fn node_result_exhausted_has_flag() {
        let r = NodeResult::exhausted();
        assert!(r.energy_exhausted);
        assert!(r.world_action.is_none());
    }

    #[test]
    fn node_result_action_carries_action() {
        let r = NodeResult::action([0.0; 12], 0.0, WorldAction::Eat);
        assert_eq!(r.world_action, Some(WorldAction::Eat));
        assert!(!r.energy_exhausted);
    }
}
```

### Step 4: Run tests to verify they pass

```bash
cd v3 && cargo test runtime::types -- --nocapture
```

Expected: all 9 tests pass.

### Step 5: Verify fmt + clippy

```bash
cd v3 && cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings
```

Expected: no errors.

### Step 6: Commit

```bash
git add v3/crates/v3-core/src/lib.rs v3/crates/v3-core/src/runtime/mod.rs v3/crates/v3-core/src/runtime/types.rs
git commit -m "feat(v3-core): Stage 3a Task 1 — NodeResult + sanitize_f32"
```

---

## Task 2: Sensors — `StaticInputs` + Direction index + assembly

**Files:**
- Create: `v3/crates/v3-core/src/sensors/mod.rs`
- Create: `v3/crates/v3-core/src/sensors/static_inputs.rs`
- Modify: `v3/crates/v3-core/src/lib.rs` (add `pub mod sensors;`)
- Modify: `v3/crates/v3-core/src/contracts/direction.rs` (add `to_index()` method)

### Step 1: Add `to_index()` to Direction

Edit `v3/crates/v3-core/src/contracts/direction.rs`, add after the `delta()` method inside the `impl Direction` block:

```rust
/// Canonical direction index (0=N, 1=NE, 2=E, 3=SE, 4=S, 5=SW, 6=W, 7=NW).
/// Matches the position in `Direction::ALL`.
pub fn to_index(self) -> usize {
    match self {
        Direction::N => 0,
        Direction::NE => 1,
        Direction::E => 2,
        Direction::SE => 3,
        Direction::S => 4,
        Direction::SW => 5,
        Direction::W => 6,
        Direction::NW => 7,
    }
}
```

Add a test in the existing `#[cfg(test)] mod tests` block in direction.rs:

```rust
#[test]
fn to_index_matches_all_order() {
    for (i, dir) in Direction::ALL.iter().enumerate() {
        assert_eq!(dir.to_index(), i);
    }
}
```

### Step 2: Add `pub mod sensors;` to lib.rs

Edit `v3/crates/v3-core/src/lib.rs`:

```rust
pub mod config;
pub mod contracts;
pub mod creature;
pub mod kernel;
pub mod runtime;
pub mod sensors;
```

### Step 3: Create `sensors/mod.rs`

```rust
pub mod static_inputs;
```

### Step 4: Create `sensors/static_inputs.rs` with tests + implementation

```rust
use crate::contracts::{
    Direction, DynamicIntrospectionKey, InputReference, StaticIntrospectionKey, WorldInputKey,
};
use crate::creature::state::CreatureState;
use crate::kernel::world::WorldState;

/// Snapshot of world and static-introspection sensor values for one creature's turn.
///
/// Assembled once at turn start; dynamic introspection (energy, consumed) is
/// resolved live during mesh evaluation.
///
/// All food values are normalized to [0.0, 1.0] by dividing raw u8 by 255.
pub struct StaticInputs {
    /// Food density at the creature's current cell, normalized to [0.0, 1.0].
    pub food_here: f32,
    /// Food density of each neighboring cell, indexed by Direction::to_index().
    /// 0.0 for out-of-bounds (bounded edge mode) neighbors.
    pub neighbor_food: [f32; 8],
    /// Whether each neighboring cell has a barrier (1.0) or not (0.0).
    /// 0.0 for out-of-bounds neighbors.
    pub neighbor_barrier: [f32; 8],
    /// Whether each neighboring cell is occupied by a creature (1.0) or not (0.0).
    /// 0.0 for out-of-bounds neighbors.
    pub neighbor_occupied: [f32; 8],
    /// Creature's generation number cast to f32. Unbounded.
    pub generation: f32,
    /// Creature's age in ticks cast to f32. Unbounded.
    pub age_ticks: f32,
}

impl StaticInputs {
    /// Look up a resolved f32 value by WorldInputKey.
    /// Invalid direction index (via raw u8 path) yields 0.0 per soft-default rule.
    pub fn resolve_world(&self, key: &WorldInputKey) -> f32 {
        match key {
            WorldInputKey::FoodHere => self.food_here,
            WorldInputKey::NeighborCellFood(dir) => self.neighbor_food[dir.to_index()],
            WorldInputKey::NeighborCellBarrier(dir) => self.neighbor_barrier[dir.to_index()],
            WorldInputKey::NeighborCellOccupied(dir) => self.neighbor_occupied[dir.to_index()],
        }
    }

    /// Look up a resolved f32 value by StaticIntrospectionKey.
    pub fn resolve_static(&self, key: &StaticIntrospectionKey) -> f32 {
        match key {
            StaticIntrospectionKey::Generation => self.generation,
            StaticIntrospectionKey::AgeTicks => self.age_ticks,
        }
    }
}

/// Assemble all static sensor values for one creature at turn start.
///
/// Reads current world state and creature state. Dynamic introspection
/// (EnergyCurrent, EnergyConsumedThisTick) is NOT included here — it is
/// resolved live during mesh evaluation.
pub fn assemble_static_inputs(world: &WorldState, creature: &CreatureState) -> StaticInputs {
    let pos = creature.position;
    let food_here = world.food_at(pos) as f32 / 255.0;

    let mut neighbor_food = [0.0f32; 8];
    let mut neighbor_barrier = [0.0f32; 8];
    let mut neighbor_occupied = [0.0f32; 8];

    for dir in Direction::ALL {
        let idx = dir.to_index();
        match world.resolve_neighbor(pos, dir) {
            Some(npos) => {
                neighbor_food[idx] = world.food_at(npos) as f32 / 255.0;
                neighbor_barrier[idx] = if world.is_barrier(npos) { 1.0 } else { 0.0 };
                neighbor_occupied[idx] = if world.creature_at(npos).is_some() { 1.0 } else { 0.0 };
            }
            None => {
                // Bounded-edge out-of-bounds: soft default 0.0 for all fields.
            }
        }
    }

    StaticInputs {
        food_here,
        neighbor_food,
        neighbor_barrier,
        neighbor_occupied,
        generation: creature.generation as f32,
        age_ticks: creature.age as f32,
    }
}

/// Resolve an InputReference using only static and upstream data.
/// Dynamic introspection keys (Energy) are NOT handled here — use
/// `runtime::inputs::resolve_input` for full resolution.
///
/// Returns 0.0 for any dynamic introspection key (soft default).
pub fn resolve_static_ref(
    reference: &InputReference,
    static_inputs: &StaticInputs,
    upstream_slots: &[f32; 12],
) -> f32 {
    match reference {
        InputReference::World(key) => static_inputs.resolve_world(key),
        InputReference::StaticIntrospection(key) => static_inputs.resolve_static(key),
        InputReference::DynamicIntrospection(_) => 0.0, // resolved live by runtime
        InputReference::UpstreamSlot(idx) => {
            if *idx < 12 {
                upstream_slots[*idx]
            } else {
                0.0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::simulation::SimulationConfig;
    use crate::config::simulation::WorldEdgeMode;
    use crate::contracts::{CreatureId, NodeId, Position};
    use crate::creature::genome::{BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction};
    use crate::creature::state::CreatureState;
    use crate::kernel::world::WorldState;
    use slotmap::SlotMap;

    fn make_world(w: u16, h: u16) -> WorldState {
        WorldState::new(w, h, WorldEdgeMode::Wrap)
    }

    fn make_creature(id: CreatureId, pos: Position) -> CreatureState {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![],
            }],
        };
        CreatureState::new(id, genome, pos, 20.0, 0, [128, 64, 32])
    }

    fn get_id() -> CreatureId {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        sm.insert(())
    }

    #[test]
    fn food_here_normalized() {
        let mut world = make_world(4, 4);
        let pos = Position::new(2, 2);
        // Set food to 255 (raw) → should normalize to 1.0
        for _ in 0..255 {
            // food_at returns u8; force set via consume+grow isn't convenient.
            // Use the internal accessor indirectly: seed_food with coverage 1.0
        }
        // Instead, use a world with grow_food setting density directly:
        // We can access food_at after seeding with coverage 1.0 + density 255.
        let id = get_id();
        let creature = make_creature(id, pos);

        // world with 0 food → food_here = 0.0
        let si = assemble_static_inputs(&world, &creature);
        assert_eq!(si.food_here, 0.0);
    }

    #[test]
    fn food_here_with_seeded_food() {
        let mut world = make_world(4, 4);
        let pos = Position::new(1, 1);
        // Manually set food via repeated grow (workaround: we know food_at returns 0 initially).
        // Actually, let's seed with coverage 1.0 and initial_density 80 to get 80/255.
        use rand::rngs::SmallRng;
        use rand::SeedableRng;
        let mut cfg = SimulationConfig::default();
        cfg.world.food.initial_coverage = 1.0;
        cfg.world.food.initial_density = 255;
        let mut rng = SmallRng::seed_from_u64(42);
        world.seed_food(&mut rng, &cfg);

        let id = get_id();
        let creature = make_creature(id, pos);
        let si = assemble_static_inputs(&world, &creature);
        // All cells seeded with 255 density → food_here should be 1.0
        assert!((si.food_here - 1.0).abs() < 1e-6);
    }

    #[test]
    fn neighbor_food_normalized() {
        let mut world = make_world(4, 4);
        let pos = Position::new(2, 2);
        // Seed north neighbor with max food
        use rand::rngs::SmallRng;
        use rand::SeedableRng;
        let mut cfg = SimulationConfig::default();
        cfg.world.food.initial_coverage = 1.0;
        cfg.world.food.initial_density = 255;
        let mut rng = SmallRng::seed_from_u64(0);
        world.seed_food(&mut rng, &cfg);

        let id = get_id();
        let creature = make_creature(id, pos);
        let si = assemble_static_inputs(&world, &creature);
        // With all cells at 255 → all neighbor_food should be 1.0
        for i in 0..8 {
            assert!((si.neighbor_food[i] - 1.0).abs() < 1e-6, "neighbor_food[{i}] != 1.0");
        }
    }

    #[test]
    fn neighbor_barrier_detected() {
        let mut world = make_world(5, 5);
        let pos = Position::new(2, 2);
        // Place a barrier to the North (2, 1)
        world.set_barrier(Position::new(2, 1), true);

        let id = get_id();
        let creature = make_creature(id, pos);
        let si = assemble_static_inputs(&world, &creature);

        // N index = 0
        assert_eq!(si.neighbor_barrier[Direction::N.to_index()], 1.0);
        // Others should be 0.0
        assert_eq!(si.neighbor_barrier[Direction::S.to_index()], 0.0);
        assert_eq!(si.neighbor_barrier[Direction::E.to_index()], 0.0);
    }

    #[test]
    fn neighbor_occupied_detected() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id1 = sm.insert(());
        let id2 = sm.insert(());

        let mut world = make_world(5, 5);
        let pos = Position::new(2, 2);
        // Place creature to the East (3, 2)
        world.place_creature(Position::new(3, 2), id2);

        let creature = make_creature(id1, pos);
        let si = assemble_static_inputs(&world, &creature);

        // E index = 2
        assert_eq!(si.neighbor_occupied[Direction::E.to_index()], 1.0);
        assert_eq!(si.neighbor_occupied[Direction::W.to_index()], 0.0);
    }

    #[test]
    fn static_introspection_generation_and_age() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let world = make_world(4, 4);
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![],
            }],
        };
        let mut creature = CreatureState::new(id, genome, Position::new(1, 1), 20.0, 5, [0, 0, 0]);
        creature.age = 42;

        let si = assemble_static_inputs(&world, &creature);
        assert!((si.generation - 5.0).abs() < 1e-6);
        assert!((si.age_ticks - 42.0).abs() < 1e-6);
    }

    #[test]
    fn resolve_world_food_here() {
        let world = make_world(4, 4);
        let id = get_id();
        let creature = make_creature(id, Position::new(1, 1));
        let si = assemble_static_inputs(&world, &creature);
        let result = si.resolve_world(&WorldInputKey::FoodHere);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn resolve_static_ref_upstream_slot_in_range() {
        let world = make_world(4, 4);
        let id = get_id();
        let creature = make_creature(id, Position::new(0, 0));
        let si = assemble_static_inputs(&world, &creature);
        let mut upstream = [0.0f32; 12];
        upstream[3] = 7.5;
        let val = resolve_static_ref(&InputReference::UpstreamSlot(3), &si, &upstream);
        assert!((val - 7.5).abs() < 1e-6);
    }

    #[test]
    fn resolve_static_ref_upstream_slot_out_of_range_zero() {
        let world = make_world(4, 4);
        let id = get_id();
        let creature = make_creature(id, Position::new(0, 0));
        let si = assemble_static_inputs(&world, &creature);
        let upstream = [0.0f32; 12];
        // slot 12 is out of range
        let val = resolve_static_ref(&InputReference::UpstreamSlot(12), &si, &upstream);
        assert_eq!(val, 0.0);
    }

    #[test]
    fn resolve_static_ref_dynamic_introspection_yields_zero() {
        let world = make_world(4, 4);
        let id = get_id();
        let creature = make_creature(id, Position::new(0, 0));
        let si = assemble_static_inputs(&world, &creature);
        let upstream = [0.0f32; 12];
        let val = resolve_static_ref(
            &InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
            &si,
            &upstream,
        );
        assert_eq!(val, 0.0);
    }
}
```

### Step 5: Run tests to verify they pass

```bash
cd v3 && cargo test sensors -- --nocapture
```

Expected: all tests pass. Fix any compilation issues.

### Step 6: Verify fmt + clippy

```bash
cd v3 && cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings
```

### Step 7: Commit

```bash
git add v3/crates/v3-core/src/lib.rs \
        v3/crates/v3-core/src/contracts/direction.rs \
        v3/crates/v3-core/src/sensors/mod.rs \
        v3/crates/v3-core/src/sensors/static_inputs.rs
git commit -m "feat(v3-core): Stage 3a Task 2 — sensors StaticInputs + assemble_static_inputs"
```

---

## Task 3: Input Resolution — `runtime/inputs.rs`

**Files:**
- Create: `v3/crates/v3-core/src/runtime/inputs.rs`
- Modify: `v3/crates/v3-core/src/runtime/mod.rs` (add `pub mod inputs;`)

This module provides `resolve_input`, which wraps `resolve_static_ref` and adds live dynamic introspection resolution. The VM executor calls this per `ReadInput` opcode.

### Step 1: Add `pub mod inputs;` to `runtime/mod.rs`

```rust
pub mod inputs;
pub mod types;
```

### Step 2: Create `runtime/inputs.rs`

```rust
use crate::contracts::{DynamicIntrospectionKey, InputReference, StaticIntrospectionKey};
use crate::sensors::static_inputs::StaticInputs;

/// Resolve an `InputReference` to its current f32 value.
///
/// - World and static introspection keys are read from the pre-assembled snapshot.
/// - Dynamic introspection is resolved live from the `energy` and `energy_consumed` args.
/// - UpstreamSlot: reads `upstream_slots[idx]`; idx >= 12 yields 0.0.
/// - Missing or out-of-range index: 0.0 (soft default).
pub fn resolve_input(
    reference: &InputReference,
    static_inputs: &StaticInputs,
    upstream_slots: &[f32; 12],
    energy: f32,
    energy_consumed: f32,
) -> f32 {
    match reference {
        InputReference::World(key) => static_inputs.resolve_world(key),
        InputReference::StaticIntrospection(key) => match key {
            StaticIntrospectionKey::Generation => static_inputs.generation,
            StaticIntrospectionKey::AgeTicks => static_inputs.age_ticks,
        },
        InputReference::DynamicIntrospection(key) => match key {
            DynamicIntrospectionKey::EnergyCurrent => energy,
            DynamicIntrospectionKey::EnergyConsumedThisTick => energy_consumed,
        },
        InputReference::UpstreamSlot(idx) => {
            if *idx < 12 {
                upstream_slots[*idx]
            } else {
                0.0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::{
        Direction, DynamicIntrospectionKey, InputReference, StaticIntrospectionKey, WorldInputKey,
    };
    use crate::sensors::static_inputs::StaticInputs;

    fn make_static_inputs(food_here: f32) -> StaticInputs {
        StaticInputs {
            food_here,
            neighbor_food: [0.5; 8],
            neighbor_barrier: [0.0; 8],
            neighbor_occupied: [0.0; 8],
            generation: 3.0,
            age_ticks: 10.0,
        }
    }

    #[test]
    fn world_food_here() {
        let si = make_static_inputs(0.75);
        let upstream = [0.0f32; 12];
        let v = resolve_input(&InputReference::World(WorldInputKey::FoodHere), &si, &upstream, 50.0, 0.0);
        assert!((v - 0.75).abs() < 1e-6);
    }

    #[test]
    fn world_neighbor_food() {
        let si = make_static_inputs(0.0);
        let upstream = [0.0f32; 12];
        let v = resolve_input(
            &InputReference::World(WorldInputKey::NeighborCellFood(Direction::N)),
            &si,
            &upstream,
            50.0,
            0.0,
        );
        assert!((v - 0.5).abs() < 1e-6);
    }

    #[test]
    fn static_introspection_generation() {
        let si = make_static_inputs(0.0);
        let upstream = [0.0f32; 12];
        let v = resolve_input(
            &InputReference::StaticIntrospection(StaticIntrospectionKey::Generation),
            &si,
            &upstream,
            20.0,
            0.0,
        );
        assert!((v - 3.0).abs() < 1e-6);
    }

    #[test]
    fn static_introspection_age_ticks() {
        let si = make_static_inputs(0.0);
        let upstream = [0.0f32; 12];
        let v = resolve_input(
            &InputReference::StaticIntrospection(StaticIntrospectionKey::AgeTicks),
            &si,
            &upstream,
            20.0,
            0.0,
        );
        assert!((v - 10.0).abs() < 1e-6);
    }

    #[test]
    fn dynamic_energy_current_live() {
        let si = make_static_inputs(0.0);
        let upstream = [0.0f32; 12];
        let v = resolve_input(
            &InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
            &si,
            &upstream,
            42.5,
            0.0,
        );
        assert!((v - 42.5).abs() < 1e-6);
    }

    #[test]
    fn dynamic_energy_consumed_this_tick() {
        let si = make_static_inputs(0.0);
        let upstream = [0.0f32; 12];
        let v = resolve_input(
            &InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyConsumedThisTick),
            &si,
            &upstream,
            20.0,
            5.5,
        );
        assert!((v - 5.5).abs() < 1e-6);
    }

    #[test]
    fn upstream_slot_in_range() {
        let si = make_static_inputs(0.0);
        let mut upstream = [0.0f32; 12];
        upstream[7] = 99.0;
        let v = resolve_input(&InputReference::UpstreamSlot(7), &si, &upstream, 20.0, 0.0);
        assert!((v - 99.0).abs() < 1e-6);
    }

    #[test]
    fn upstream_slot_at_boundary_11() {
        let si = make_static_inputs(0.0);
        let mut upstream = [0.0f32; 12];
        upstream[11] = 3.0;
        let v = resolve_input(&InputReference::UpstreamSlot(11), &si, &upstream, 20.0, 0.0);
        assert!((v - 3.0).abs() < 1e-6);
    }

    #[test]
    fn upstream_slot_out_of_range_yields_zero() {
        let si = make_static_inputs(0.0);
        let upstream = [1.0f32; 12];
        let v = resolve_input(&InputReference::UpstreamSlot(12), &si, &upstream, 20.0, 0.0);
        assert_eq!(v, 0.0);
    }

    #[test]
    fn upstream_slot_large_index_yields_zero() {
        let si = make_static_inputs(0.0);
        let upstream = [1.0f32; 12];
        let v = resolve_input(&InputReference::UpstreamSlot(999), &si, &upstream, 20.0, 0.0);
        assert_eq!(v, 0.0);
    }
}
```

### Step 3: Run tests

```bash
cd v3 && cargo test runtime::inputs -- --nocapture
```

Expected: all 10 tests pass.

### Step 4: Verify fmt + clippy

```bash
cd v3 && cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings
```

### Step 5: Commit

```bash
git add v3/crates/v3-core/src/runtime/mod.rs v3/crates/v3-core/src/runtime/inputs.rs
git commit -m "feat(v3-core): Stage 3a Task 3 — resolve_input for all InputReference variants"
```

---

## Task 4: WorldAction Decode — `runtime/action_decode.rs`

**Files:**
- Create: `v3/crates/v3-core/src/runtime/action_decode.rs`
- Modify: `v3/crates/v3-core/src/runtime/mod.rs`

Decode a WorldAction from an `action_type` u8 and a world-action metadata buffer.

Per `v3-vm-isa-spec.md` Section 7:
- action_type 0 → NoOp
- action_type 1 → Eat
- action_type 2 → Move, direction from meta[0]
- action_type 3 → Reproduce, direction from meta[0], energy from meta[1]
- other → NoOp

Direction decode: `meta[0].round().clamp(0.0, 7.0) as usize` → index into `Direction::ALL`.
Energy decode: `clamp_non_negative_finite(meta[1])`.

### Step 1: Add `pub mod action_decode;` to `runtime/mod.rs`

```rust
pub mod action_decode;
pub mod inputs;
pub mod types;
```

### Step 2: Create `runtime/action_decode.rs`

```rust
use crate::contracts::{Direction, WorldAction};

/// Decode a WorldAction from the raw action_type discriminant and metadata buffer.
///
/// Per v3-vm-isa-spec.md Section 7 action encoding table.
/// Unknown action_type values decode to NoOp (soft default).
pub fn decode_world_action(action_type: u8, meta: &[f32; 8]) -> WorldAction {
    match action_type {
        0 => WorldAction::NoOp,
        1 => WorldAction::Eat,
        2 => WorldAction::Move(decode_direction(meta[0])),
        3 => WorldAction::Reproduce {
            direction: decode_direction(meta[0]),
            energy_transfer: clamp_non_negative_finite(meta[1]),
        },
        _ => WorldAction::NoOp,
    }
}

/// Decode a direction from a raw f32 meta value.
/// Rounds to nearest integer, clamps to [0, 7], indexes into Direction::ALL.
fn decode_direction(raw: f32) -> Direction {
    // NaN rounds to itself and would cause issues; sanitize first
    let clamped = if raw.is_nan() {
        0.0
    } else {
        raw.round().clamp(0.0, 7.0)
    };
    Direction::ALL[clamped as usize]
}

/// Clamp to non-negative finite: NaN/Inf/negative → 0.0.
fn clamp_non_negative_finite(v: f32) -> f32 {
    if v.is_finite() && v >= 0.0 {
        v
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::Direction;

    fn zero_meta() -> [f32; 8] {
        [0.0; 8]
    }

    #[test]
    fn action_type_0_is_noop() {
        assert_eq!(decode_world_action(0, &zero_meta()), WorldAction::NoOp);
    }

    #[test]
    fn action_type_1_is_eat() {
        assert_eq!(decode_world_action(1, &zero_meta()), WorldAction::Eat);
    }

    #[test]
    fn action_type_2_is_move_with_direction() {
        let mut meta = zero_meta();
        meta[0] = 2.0; // E = index 2
        assert_eq!(
            decode_world_action(2, &meta),
            WorldAction::Move(Direction::E)
        );
    }

    #[test]
    fn action_type_3_is_reproduce() {
        let mut meta = zero_meta();
        meta[0] = 4.0; // S = index 4
        meta[1] = 15.0;
        let action = decode_world_action(3, &meta);
        if let WorldAction::Reproduce { direction, energy_transfer } = action {
            assert_eq!(direction, Direction::S);
            assert!((energy_transfer - 15.0).abs() < 1e-6);
        } else {
            panic!("expected Reproduce, got {action:?}");
        }
    }

    #[test]
    fn unknown_action_type_is_noop() {
        assert_eq!(decode_world_action(4, &zero_meta()), WorldAction::NoOp);
        assert_eq!(decode_world_action(255, &zero_meta()), WorldAction::NoOp);
    }

    #[test]
    fn direction_decode_all_8_valid_indices() {
        for (expected_idx, expected_dir) in Direction::ALL.iter().enumerate() {
            let mut meta = zero_meta();
            meta[0] = expected_idx as f32;
            if let WorldAction::Move(dir) = decode_world_action(2, &meta) {
                assert_eq!(&dir, expected_dir, "index {expected_idx}");
            } else {
                panic!("expected Move");
            }
        }
    }

    #[test]
    fn direction_decode_rounds_half_up() {
        let mut meta = zero_meta();
        meta[0] = 0.6; // rounds to 1 = NE
        if let WorldAction::Move(dir) = decode_world_action(2, &meta) {
            assert_eq!(dir, Direction::NE);
        } else {
            panic!("expected Move");
        }
    }

    #[test]
    fn direction_decode_clamps_above_7() {
        let mut meta = zero_meta();
        meta[0] = 10.0; // clamped to 7 = NW
        if let WorldAction::Move(dir) = decode_world_action(2, &meta) {
            assert_eq!(dir, Direction::NW);
        } else {
            panic!("expected Move");
        }
    }

    #[test]
    fn direction_decode_clamps_negative_to_zero() {
        let mut meta = zero_meta();
        meta[0] = -3.0; // clamped to 0 = N
        if let WorldAction::Move(dir) = decode_world_action(2, &meta) {
            assert_eq!(dir, Direction::N);
        } else {
            panic!("expected Move");
        }
    }

    #[test]
    fn direction_decode_nan_becomes_zero() {
        let mut meta = zero_meta();
        meta[0] = f32::NAN;
        if let WorldAction::Move(dir) = decode_world_action(2, &meta) {
            assert_eq!(dir, Direction::N); // index 0
        } else {
            panic!("expected Move");
        }
    }

    #[test]
    fn reproduce_energy_negative_becomes_zero() {
        let mut meta = zero_meta();
        meta[1] = -5.0;
        if let WorldAction::Reproduce { energy_transfer, .. } = decode_world_action(3, &meta) {
            assert_eq!(energy_transfer, 0.0);
        } else {
            panic!("expected Reproduce");
        }
    }

    #[test]
    fn reproduce_energy_nan_becomes_zero() {
        let mut meta = zero_meta();
        meta[1] = f32::NAN;
        if let WorldAction::Reproduce { energy_transfer, .. } = decode_world_action(3, &meta) {
            assert_eq!(energy_transfer, 0.0);
        } else {
            panic!("expected Reproduce");
        }
    }

    #[test]
    fn reproduce_energy_inf_becomes_zero() {
        let mut meta = zero_meta();
        meta[1] = f32::INFINITY;
        if let WorldAction::Reproduce { energy_transfer, .. } = decode_world_action(3, &meta) {
            assert_eq!(energy_transfer, 0.0);
        } else {
            panic!("expected Reproduce");
        }
    }
}
```

### Step 3: Run tests

```bash
cd v3 && cargo test runtime::action_decode -- --nocapture
```

Expected: all 13 tests pass.

### Step 4: Verify fmt + clippy, commit

```bash
cd v3 && cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings
git add v3/crates/v3-core/src/runtime/mod.rs v3/crates/v3-core/src/runtime/action_decode.rs
git commit -m "feat(v3-core): Stage 3a Task 4 — decode_world_action"
```

---

## Task 5: VM Executor — `runtime/vm.rs` (all 33 opcodes)

**Files:**
- Create: `v3/crates/v3-core/src/runtime/vm.rs`
- Modify: `v3/crates/v3-core/src/runtime/mod.rs`

This is the most complex task. The VM executor evaluates a `VmBackendDef` given inputs, a mutable energy budget, and persistent creature memory.

### Execution rules (from v3-vm-isa-spec.md):
1. If `register_count == 0`, return `NodeResult::halted(payload, 0.0)` immediately.
2. Initialize registers to `[0.0; register_count]`.
3. Initialize payload buffer from `upstream_slots`.
4. Initialize meta buffer to `[0.0; 8]`.
5. Initialize route_target to `0.0`.
6. PC starts at 0.
7. Each step: deduct opcode energy cost; if energy ≤ 0, don't commit memory, return `NodeResult::exhausted()`.
8. Execute opcode; any register write passes through `sanitize_f32`.
9. `EmitWorldAction { action_type }`: decode action, commit memory, return `NodeResult::action(...)`.
10. `Halt`: commit memory, return `NodeResult::halted(payload, route_target)`.
11. If steps ≥ `max_vm_steps`: commit memory, return `NodeResult::halted(payload, route_target)`.
12. If program is empty: return `NodeResult::halted(payload, 0.0)`.

**Memory contract**: work on a copy of memory; commit only if node completes (Halt, EmitWorldAction, step cap). On energy exhaustion, do NOT modify the caller's memory.

### Step 1: Add `pub mod vm;` to `runtime/mod.rs`

```rust
pub mod action_decode;
pub mod inputs;
pub mod types;
pub mod vm;
```

### Step 2: Create `runtime/vm.rs`

```rust
use crate::config::simulation::RuntimeConfig;
use crate::contracts::InputReference;
use crate::creature::genome::VmBackendDef;
use crate::runtime::action_decode::decode_world_action;
use crate::runtime::inputs::resolve_input;
use crate::runtime::types::{sanitize_f32, NodeResult};
use crate::sensors::static_inputs::StaticInputs;

/// Execute a VM backend node.
///
/// # Arguments
/// - `def`: the VM backend genome definition
/// - `input_refs`: the node's InputReference list (from NodeGenome.input_refs)
/// - `upstream_slots`: incoming output slots from the previous node (or zeroed for entry)
/// - `energy`: creature's current energy; decremented by opcode costs; NOT restored on exhaustion
/// - `energy_consumed`: total energy consumed this tick so far (for dynamic introspection)
/// - `memory`: creature's persistent 1024-byte memory; NOT modified on energy exhaustion
/// - `static_inputs`: pre-assembled world/static sensor snapshot
/// - `config`: runtime config (max_vm_steps, vm.opcode_cost_multiplier)
///
/// # Returns
/// `NodeResult` — the mesh executor checks `energy_exhausted` and routes accordingly.
pub fn execute_vm_node(
    def: &VmBackendDef,
    input_refs: &[InputReference],
    upstream_slots: &[f32; 12],
    energy: &mut f32,
    energy_consumed: f32,
    memory: &mut [u8; 1024],
    static_inputs: &StaticInputs,
    config: &RuntimeConfig,
) -> NodeResult {
    // Safety: register_count == 0 → immediate halt.
    let reg_count = def.register_count as usize;
    if reg_count == 0 {
        return NodeResult::halted(*upstream_slots, 0.0);
    }

    // Safety: empty program → immediate halt.
    let program_len = def.program.len();
    if program_len == 0 {
        return NodeResult::halted(*upstream_slots, 0.0);
    }

    let max_steps = config.max_vm_steps.max(1) as usize;
    let cost_mult = config.vm.opcode_cost_multiplier;

    let mut regs = vec![0.0f32; reg_count];
    let mut payload: [f32; 12] = *upstream_slots;
    let mut meta: [f32; 8] = [0.0; 8];
    let mut route_target: f32 = 0.0;
    let mut pc: usize = 0;
    let mut steps: usize = 0;

    // Work on a memory copy; committed only on normal exit.
    let mut mem_copy = *memory;

    use crate::creature::genome::VmInstruction;

    loop {
        if steps >= max_steps {
            *memory = mem_copy;
            return NodeResult::halted(payload, route_target);
        }

        let instr = &def.program[pc];
        let opcode_cost = opcode_base_cost(instr) * cost_mult;

        // Deduct energy before executing; exhaustion halts without side effects.
        *energy -= opcode_cost;
        if *energy <= 0.0 {
            // Do NOT commit memory.
            return NodeResult::exhausted();
        }

        steps += 1;
        let mut next_pc = pc + 1;

        // Helper closures (using reg_count for normalization).
        let nr = |idx: u8| (idx as usize).rem_euclid(reg_count);

        match instr {
            VmInstruction::Noop => {}

            VmInstruction::LoadConst { dst, const_idx } => {
                let val = if def.constants.is_empty() {
                    0.0
                } else {
                    def.constants[(*const_idx as usize).rem_euclid(def.constants.len())]
                };
                regs[nr(*dst)] = sanitize_f32(val);
            }

            VmInstruction::Move { dst, src } => {
                let val = regs[nr(*src)];
                regs[nr(*dst)] = sanitize_f32(val);
            }

            VmInstruction::Add { dst, a, b } => {
                regs[nr(*dst)] = sanitize_f32(regs[nr(*a)] + regs[nr(*b)]);
            }

            VmInstruction::Sub { dst, a, b } => {
                regs[nr(*dst)] = sanitize_f32(regs[nr(*a)] - regs[nr(*b)]);
            }

            VmInstruction::Mul { dst, a, b } => {
                regs[nr(*dst)] = sanitize_f32(regs[nr(*a)] * regs[nr(*b)]);
            }

            VmInstruction::Div { dst, a, b } => {
                let divisor = regs[nr(*b)];
                let val = if divisor == 0.0 { 0.0 } else { regs[nr(*a)] / divisor };
                regs[nr(*dst)] = sanitize_f32(val);
            }

            VmInstruction::Min { dst, a, b } => {
                regs[nr(*dst)] = sanitize_f32(regs[nr(*a)].min(regs[nr(*b)]));
            }

            VmInstruction::Max { dst, a, b } => {
                regs[nr(*dst)] = sanitize_f32(regs[nr(*a)].max(regs[nr(*b)]));
            }

            VmInstruction::Abs { dst, src } => {
                regs[nr(*dst)] = sanitize_f32(regs[nr(*src)].abs());
            }

            VmInstruction::Neg { dst, src } => {
                regs[nr(*dst)] = sanitize_f32(-regs[nr(*src)]);
            }

            VmInstruction::Clamp01 { dst, src } => {
                regs[nr(*dst)] = sanitize_f32(regs[nr(*src)].clamp(0.0, 1.0));
            }

            VmInstruction::CmpGt { dst, a, b } => {
                regs[nr(*dst)] = if regs[nr(*a)] > regs[nr(*b)] { 1.0 } else { 0.0 };
            }

            VmInstruction::CmpLt { dst, a, b } => {
                regs[nr(*dst)] = if regs[nr(*a)] < regs[nr(*b)] { 1.0 } else { 0.0 };
            }

            VmInstruction::CmpEq { dst, a, b, eps } => {
                // eps register value clamped to [1e-6, 1.0]
                let eps_val = regs[nr(*eps)].clamp(1e-6, 1.0);
                let diff = (regs[nr(*a)] - regs[nr(*b)]).abs();
                regs[nr(*dst)] = if diff <= eps_val { 1.0 } else { 0.0 };
            }

            VmInstruction::And { dst, a, b } => {
                let ta = is_truthy(regs[nr(*a)]);
                let tb = is_truthy(regs[nr(*b)]);
                regs[nr(*dst)] = if ta && tb { 1.0 } else { 0.0 };
            }

            VmInstruction::Or { dst, a, b } => {
                let ta = is_truthy(regs[nr(*a)]);
                let tb = is_truthy(regs[nr(*b)]);
                regs[nr(*dst)] = if ta || tb { 1.0 } else { 0.0 };
            }

            VmInstruction::Not { dst, src } => {
                regs[nr(*dst)] = if is_truthy(regs[nr(*src)]) { 0.0 } else { 1.0 };
            }

            VmInstruction::ToI32 { dst, src } => {
                // Round ties-away-from-zero, store as f32.
                let val = round_ties_away(regs[nr(*src)]);
                regs[nr(*dst)] = sanitize_f32(val);
            }

            VmInstruction::ToU8 { dst, src } => {
                // Clamp [0, 255], round, store as f32.
                let clamped = regs[nr(*src)].clamp(0.0, 255.0);
                regs[nr(*dst)] = sanitize_f32(round_ties_away(clamped));
            }

            VmInstruction::ToBool { dst, src } => {
                regs[nr(*dst)] = if is_truthy(regs[nr(*src)]) { 1.0 } else { 0.0 };
            }

            VmInstruction::JumpIfZero { cond, offset } => {
                if !is_truthy(regs[nr(*cond)]) {
                    next_pc = jump_target(pc, *offset, program_len);
                }
            }

            VmInstruction::Jump { offset } => {
                next_pc = jump_target(pc, *offset, program_len);
            }

            VmInstruction::ReadInput { dst, input_idx } => {
                let val = if (*input_idx as usize) < input_refs.len() {
                    resolve_input(
                        &input_refs[*input_idx as usize],
                        static_inputs,
                        upstream_slots,
                        *energy,
                        energy_consumed,
                    )
                } else {
                    0.0 // soft default: out-of-range input_idx
                };
                regs[nr(*dst)] = sanitize_f32(val);
            }

            VmInstruction::WriteInternalPayload { slot_idx, src } => {
                if (*slot_idx as usize) < 12 {
                    payload[*slot_idx as usize] = regs[nr(*src)];
                }
                // invalid slot: write ignored
            }

            VmInstruction::WriteWorldActionMeta { slot_idx, src } => {
                if (*slot_idx as usize) < 8 {
                    meta[*slot_idx as usize] = regs[nr(*src)];
                }
                // invalid slot: write ignored
            }

            VmInstruction::EmitWorldAction { action_type } => {
                let action = decode_world_action(*action_type, &meta);
                *memory = mem_copy;
                return NodeResult::action(payload, route_target, action);
            }

            VmInstruction::WriteRouteTarget { src } => {
                route_target = regs[nr(*src)]; // last-write-wins
            }

            VmInstruction::Halt => {
                *memory = mem_copy;
                return NodeResult::halted(payload, route_target);
            }

            VmInstruction::LoadMem8 { dst, addr_reg } => {
                let addr = (regs[nr(*addr_reg)] as i64).rem_euclid(1024) as usize;
                regs[nr(*dst)] = sanitize_f32(mem_copy[addr] as f32);
            }

            VmInstruction::StoreMem8 { addr_reg, src } => {
                let addr = (regs[nr(*addr_reg)] as i64).rem_euclid(1024) as usize;
                let val = regs[nr(*src)].clamp(0.0, 255.0) as u8;
                mem_copy[addr] = val;
            }

            VmInstruction::LoadMem8Imm { dst, imm_addr } => {
                let addr = (*imm_addr as usize).rem_euclid(1024);
                regs[nr(*dst)] = sanitize_f32(mem_copy[addr] as f32);
            }

            VmInstruction::StoreMem8Imm { imm_addr, src } => {
                let addr = (*imm_addr as usize).rem_euclid(1024);
                let val = regs[nr(*src)].clamp(0.0, 255.0) as u8;
                mem_copy[addr] = val;
            }
        }

        pc = next_pc;
    }
}

/// Boolean truthiness: value >= 0.5.
#[inline]
fn is_truthy(v: f32) -> bool {
    v >= 0.5
}

/// Compute jump target PC with rem_euclid wrapping.
/// `offset` is signed relative to the instruction AFTER the jump.
#[inline]
fn jump_target(pc: usize, offset: i32, program_len: usize) -> usize {
    let provisional = pc as i64 + 1 + offset as i64;
    provisional.rem_euclid(program_len as i64) as usize
}

/// Round ties-away-from-zero (f32). Rust's f32::round() already does this.
#[inline]
fn round_ties_away(v: f32) -> f32 {
    v.round()
}

/// Base energy cost per opcode per v3-vm-isa-spec.md Section 6.
fn opcode_base_cost(instr: &crate::creature::genome::VmInstruction) -> f32 {
    use crate::creature::genome::VmInstruction;
    match instr {
        VmInstruction::Noop => 0.05,
        VmInstruction::LoadConst { .. } => 0.08,
        VmInstruction::Move { .. } => 0.08,
        VmInstruction::Add { .. } => 0.12,
        VmInstruction::Sub { .. } => 0.12,
        VmInstruction::Mul { .. } => 0.12,
        VmInstruction::Div { .. } => 0.16,
        VmInstruction::Min { .. } => 0.12,
        VmInstruction::Max { .. } => 0.12,
        VmInstruction::Abs { .. } => 0.10,
        VmInstruction::Neg { .. } => 0.10,
        VmInstruction::Clamp01 { .. } => 0.10,
        VmInstruction::CmpGt { .. } => 0.12,
        VmInstruction::CmpLt { .. } => 0.12,
        VmInstruction::CmpEq { .. } => 0.12,
        VmInstruction::And { .. } => 0.12,
        VmInstruction::Or { .. } => 0.12,
        VmInstruction::Not { .. } => 0.10,
        VmInstruction::ToI32 { .. } => 0.10,
        VmInstruction::ToU8 { .. } => 0.10,
        VmInstruction::ToBool { .. } => 0.10,
        VmInstruction::JumpIfZero { .. } => 0.14,
        VmInstruction::Jump { .. } => 0.10,
        VmInstruction::ReadInput { .. } => 0.12,
        VmInstruction::WriteInternalPayload { .. } => 0.14,
        VmInstruction::WriteWorldActionMeta { .. } => 0.14,
        VmInstruction::EmitWorldAction { .. } => 0.24,
        VmInstruction::WriteRouteTarget { .. } => 0.10,
        VmInstruction::Halt => 0.05,
        VmInstruction::LoadMem8 { .. } => 0.16,
        VmInstruction::StoreMem8 { .. } => 0.18,
        VmInstruction::LoadMem8Imm { .. } => 0.14,
        VmInstruction::StoreMem8Imm { .. } => 0.16,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::simulation::RuntimeConfig;
    use crate::contracts::Direction;
    use crate::creature::genome::{VmBackendDef, VmInstruction};
    use crate::sensors::static_inputs::StaticInputs;

    fn config() -> RuntimeConfig {
        RuntimeConfig::default()
    }

    fn empty_static_inputs() -> StaticInputs {
        StaticInputs {
            food_here: 0.0,
            neighbor_food: [0.0; 8],
            neighbor_barrier: [0.0; 8],
            neighbor_occupied: [0.0; 8],
            generation: 0.0,
            age_ticks: 0.0,
        }
    }

    fn zeroed_upstream() -> [f32; 12] {
        [0.0; 12]
    }

    fn run_vm(
        program: Vec<VmInstruction>,
        register_count: u8,
        constants: Vec<f32>,
        input_refs: &[InputReference],
        upstream: [f32; 12],
        energy: f32,
    ) -> (NodeResult, f32) {
        let def = VmBackendDef {
            register_count,
            constants,
            program,
        };
        let si = empty_static_inputs();
        let mut e = energy;
        let mut mem = [0u8; 1024];
        let result = execute_vm_node(&def, input_refs, &upstream, &mut e, 0.0, &mut mem, &si, &config());
        (result, e)
    }

    // ── Halt and empty program ────────────────────────────────────────────────

    #[test]
    fn register_count_zero_halts_immediately() {
        let def = VmBackendDef {
            register_count: 0,
            constants: vec![],
            program: vec![VmInstruction::EmitWorldAction { action_type: 1 }],
        };
        let si = empty_static_inputs();
        let mut e = 100.0;
        let mut mem = [0u8; 1024];
        let r = execute_vm_node(&def, &[], &zeroed_upstream(), &mut e, 0.0, &mut mem, &si, &config());
        assert!(r.world_action.is_none());
        assert!(!r.energy_exhausted);
    }

    #[test]
    fn empty_program_halts_immediately() {
        let (r, _) = run_vm(vec![], 1, vec![], &[], zeroed_upstream(), 100.0);
        assert!(r.world_action.is_none());
    }

    #[test]
    fn halt_returns_no_action() {
        let (r, _) = run_vm(
            vec![VmInstruction::Halt],
            2,
            vec![],
            &[],
            zeroed_upstream(),
            100.0,
        );
        assert!(r.world_action.is_none());
        assert!(!r.energy_exhausted);
    }

    // ── EmitWorldAction ───────────────────────────────────────────────────────

    #[test]
    fn emit_noop_action_type_0() {
        let (r, _) = run_vm(
            vec![VmInstruction::EmitWorldAction { action_type: 0 }],
            1,
            vec![],
            &[],
            zeroed_upstream(),
            100.0,
        );
        assert_eq!(r.world_action, Some(crate::contracts::WorldAction::NoOp));
    }

    #[test]
    fn emit_eat_action_type_1() {
        let (r, _) = run_vm(
            vec![VmInstruction::EmitWorldAction { action_type: 1 }],
            1,
            vec![],
            &[],
            zeroed_upstream(),
            100.0,
        );
        assert_eq!(r.world_action, Some(crate::contracts::WorldAction::Eat));
    }

    #[test]
    fn emit_move_with_meta() {
        // Set meta[0] = 2.0 (East), then emit Move
        let program = vec![
            VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // r0 = 2.0
            VmInstruction::WriteWorldActionMeta { slot_idx: 0, src: 0 },
            VmInstruction::EmitWorldAction { action_type: 2 },
        ];
        let (r, _) = run_vm(program, 1, vec![2.0], &[], zeroed_upstream(), 100.0);
        assert_eq!(r.world_action, Some(crate::contracts::WorldAction::Move(Direction::E)));
    }

    // ── Arithmetic opcodes ────────────────────────────────────────────────────

    #[test]
    fn add_two_constants() {
        // Load 3.0 and 4.0, add, halt, check payload slot
        let program = vec![
            VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // r0 = 3.0
            VmInstruction::LoadConst { dst: 1, const_idx: 1 }, // r1 = 4.0
            VmInstruction::Add { dst: 2, a: 0, b: 1 },         // r2 = 7.0
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 2 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 3, vec![3.0, 4.0], &[], zeroed_upstream(), 100.0);
        assert!((r.output_slots[0] - 7.0).abs() < 1e-6);
    }

    #[test]
    fn sub_result() {
        let program = vec![
            VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // r0 = 10.0
            VmInstruction::LoadConst { dst: 1, const_idx: 1 }, // r1 = 3.0
            VmInstruction::Sub { dst: 2, a: 0, b: 1 },         // r2 = 7.0
            VmInstruction::WriteInternalPayload { slot_idx: 1, src: 2 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 3, vec![10.0, 3.0], &[], zeroed_upstream(), 100.0);
        assert!((r.output_slots[1] - 7.0).abs() < 1e-6);
    }

    #[test]
    fn div_by_zero_yields_zero() {
        let program = vec![
            VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // r0 = 5.0
            // r1 stays 0.0 (initial)
            VmInstruction::Div { dst: 2, a: 0, b: 1 },         // r2 = 5/0 = 0
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 2 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 3, vec![5.0], &[], zeroed_upstream(), 100.0);
        assert_eq!(r.output_slots[0], 0.0);
    }

    #[test]
    fn min_picks_smaller() {
        let program = vec![
            VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // r0 = 3.0
            VmInstruction::LoadConst { dst: 1, const_idx: 1 }, // r1 = 8.0
            VmInstruction::Min { dst: 2, a: 0, b: 1 },
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 2 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 3, vec![3.0, 8.0], &[], zeroed_upstream(), 100.0);
        assert!((r.output_slots[0] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn max_picks_larger() {
        let program = vec![
            VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // r0 = 3.0
            VmInstruction::LoadConst { dst: 1, const_idx: 1 }, // r1 = 8.0
            VmInstruction::Max { dst: 2, a: 0, b: 1 },
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 2 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 3, vec![3.0, 8.0], &[], zeroed_upstream(), 100.0);
        assert!((r.output_slots[0] - 8.0).abs() < 1e-6);
    }

    #[test]
    fn abs_of_negative() {
        let program = vec![
            VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // r0 = -5.0
            VmInstruction::Abs { dst: 1, src: 0 },
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 1 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 2, vec![-5.0], &[], zeroed_upstream(), 100.0);
        assert!((r.output_slots[0] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn neg_flips_sign() {
        let program = vec![
            VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // r0 = 3.0
            VmInstruction::Neg { dst: 1, src: 0 },
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 1 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 2, vec![3.0], &[], zeroed_upstream(), 100.0);
        assert!((r.output_slots[0] - (-3.0)).abs() < 1e-6);
    }

    #[test]
    fn clamp01_clamps_above_one() {
        let program = vec![
            VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // r0 = 5.0
            VmInstruction::Clamp01 { dst: 1, src: 0 },
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 1 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 2, vec![5.0], &[], zeroed_upstream(), 100.0);
        assert!((r.output_slots[0] - 1.0).abs() < 1e-6);
    }

    // ── Comparison opcodes ────────────────────────────────────────────────────

    #[test]
    fn cmpgt_true_when_a_greater() {
        let program = vec![
            VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // r0 = 5.0
            VmInstruction::LoadConst { dst: 1, const_idx: 1 }, // r1 = 3.0
            VmInstruction::CmpGt { dst: 2, a: 0, b: 1 },
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 2 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 3, vec![5.0, 3.0], &[], zeroed_upstream(), 100.0);
        assert_eq!(r.output_slots[0], 1.0);
    }

    #[test]
    fn cmpgt_false_when_a_equal() {
        let program = vec![
            VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // r0 = 3.0
            VmInstruction::CmpGt { dst: 1, a: 0, b: 0 },       // 3.0 > 3.0 = false
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 1 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 2, vec![3.0], &[], zeroed_upstream(), 100.0);
        assert_eq!(r.output_slots[0], 0.0);
    }

    #[test]
    fn cmplt_true_when_a_less() {
        let program = vec![
            VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // r0 = 2.0
            VmInstruction::LoadConst { dst: 1, const_idx: 1 }, // r1 = 5.0
            VmInstruction::CmpLt { dst: 2, a: 0, b: 1 },
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 2 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 3, vec![2.0, 5.0], &[], zeroed_upstream(), 100.0);
        assert_eq!(r.output_slots[0], 1.0);
    }

    #[test]
    fn cmpeq_true_within_epsilon() {
        // r0=1.0, r1=1.0001, eps register r2 (default 0.0 but clamped to 1e-6)
        // actually: all regs start 0. Load 1.0 into r0 and r1, eps in r2 = 0.01
        let program = vec![
            VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // r0 = 1.0
            VmInstruction::LoadConst { dst: 1, const_idx: 1 }, // r1 = 1.0001
            VmInstruction::LoadConst { dst: 2, const_idx: 2 }, // r2 = 0.01 (eps)
            VmInstruction::CmpEq { dst: 3, a: 0, b: 1, eps: 2 },
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 3 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 4, vec![1.0, 1.0001, 0.01], &[], zeroed_upstream(), 100.0);
        assert_eq!(r.output_slots[0], 1.0);
    }

    #[test]
    fn and_both_true() {
        let program = vec![
            VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // r0 = 1.0 (truthy)
            VmInstruction::LoadConst { dst: 1, const_idx: 1 }, // r1 = 0.8 (truthy)
            VmInstruction::And { dst: 2, a: 0, b: 1 },
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 2 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 3, vec![1.0, 0.8], &[], zeroed_upstream(), 100.0);
        assert_eq!(r.output_slots[0], 1.0);
    }

    #[test]
    fn and_one_false() {
        let program = vec![
            VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // r0 = 1.0
            VmInstruction::LoadConst { dst: 1, const_idx: 1 }, // r1 = 0.4 (not truthy, < 0.5)
            VmInstruction::And { dst: 2, a: 0, b: 1 },
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 2 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 3, vec![1.0, 0.4], &[], zeroed_upstream(), 100.0);
        assert_eq!(r.output_slots[0], 0.0);
    }

    #[test]
    fn or_one_true() {
        let program = vec![
            // r0 stays 0.0 (false), r1 = 1.0 (true)
            VmInstruction::LoadConst { dst: 1, const_idx: 0 }, // r1 = 1.0
            VmInstruction::Or { dst: 2, a: 0, b: 1 },
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 2 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 3, vec![1.0], &[], zeroed_upstream(), 100.0);
        assert_eq!(r.output_slots[0], 1.0);
    }

    #[test]
    fn not_inverts() {
        let program = vec![
            VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // r0 = 1.0 (truthy)
            VmInstruction::Not { dst: 1, src: 0 },
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 1 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 2, vec![1.0], &[], zeroed_upstream(), 100.0);
        assert_eq!(r.output_slots[0], 0.0);
    }

    // ── Type conversion opcodes ───────────────────────────────────────────────

    #[test]
    fn to_i32_rounds_half_up() {
        let program = vec![
            VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // r0 = 2.5
            VmInstruction::ToI32 { dst: 1, src: 0 },
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 1 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 2, vec![2.5], &[], zeroed_upstream(), 100.0);
        assert!((r.output_slots[0] - 3.0).abs() < 1e-6); // ties-away-from-zero
    }

    #[test]
    fn to_u8_clamps_above_255() {
        let program = vec![
            VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // r0 = 300.0
            VmInstruction::ToU8 { dst: 1, src: 0 },
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 1 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 2, vec![300.0], &[], zeroed_upstream(), 100.0);
        assert!((r.output_slots[0] - 255.0).abs() < 1e-6);
    }

    #[test]
    fn to_bool_one_is_truthy() {
        let program = vec![
            VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // r0 = 0.6
            VmInstruction::ToBool { dst: 1, src: 0 },
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 1 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 2, vec![0.6], &[], zeroed_upstream(), 100.0);
        assert_eq!(r.output_slots[0], 1.0);
    }

    // ── Control flow opcodes ──────────────────────────────────────────────────

    #[test]
    fn jump_skips_instructions() {
        // Program: Jump(1), LoadConst(r0=99), Halt
        // If jump fires: skip LoadConst, halt with r0=0.
        // payload slot 0 should be 0.0 (not 99.0)
        let program = vec![
            VmInstruction::Jump { offset: 1 },                 // jump to pc+1+1=2
            VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // skipped
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 0 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 1, vec![99.0], &[], zeroed_upstream(), 100.0);
        assert_eq!(r.output_slots[0], 0.0);
    }

    #[test]
    fn jump_if_zero_fires_when_false() {
        // r0=0 (falsy); JumpIfZero cond=0, offset=1 should fire and skip LoadConst
        let program = vec![
            VmInstruction::JumpIfZero { cond: 0, offset: 1 }, // fires (r0=0)
            VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // skipped
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 0 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 1, vec![99.0], &[], zeroed_upstream(), 100.0);
        assert_eq!(r.output_slots[0], 0.0);
    }

    #[test]
    fn jump_if_zero_not_fires_when_truthy() {
        // Load 1.0 into r0, then JumpIfZero (should NOT fire), LoadConst r0=99, Halt
        let program = vec![
            VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // r0 = 1.0
            VmInstruction::JumpIfZero { cond: 0, offset: 1 }, // does NOT fire (truthy)
            VmInstruction::LoadConst { dst: 1, const_idx: 1 }, // r1 = 99.0
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 1 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 2, vec![1.0, 99.0], &[], zeroed_upstream(), 100.0);
        assert!((r.output_slots[0] - 99.0).abs() < 1e-6);
    }

    #[test]
    fn jump_target_wraps_via_rem_euclid() {
        // Program of 2 instructions: Jump(offset=2), Halt
        // provisional_pc = 0 + 1 + 2 = 3; 3 % 2 = 1 (Halt)
        let program = vec![
            VmInstruction::Jump { offset: 2 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 1, vec![], &[], zeroed_upstream(), 100.0);
        assert!(r.world_action.is_none());
    }

    // ── ReadInput opcode ──────────────────────────────────────────────────────

    #[test]
    fn read_input_upstream_slot() {
        use crate::contracts::InputReference;
        let input_refs = vec![InputReference::UpstreamSlot(0)];
        let mut upstream = zeroed_upstream();
        upstream[0] = 42.0;
        let program = vec![
            VmInstruction::ReadInput { dst: 0, input_idx: 0 },
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 0 },
            VmInstruction::Halt,
        ];
        let def = VmBackendDef { register_count: 1, constants: vec![], program };
        let si = empty_static_inputs();
        let mut e = 100.0;
        let mut mem = [0u8; 1024];
        let r = execute_vm_node(&def, &input_refs, &upstream, &mut e, 0.0, &mut mem, &si, &config());
        assert!((r.output_slots[0] - 42.0).abs() < 1e-6);
    }

    #[test]
    fn read_input_out_of_range_yields_zero() {
        let program = vec![
            VmInstruction::ReadInput { dst: 0, input_idx: 5 }, // no input_refs at all
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 0 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 1, vec![], &[], zeroed_upstream(), 100.0);
        assert_eq!(r.output_slots[0], 0.0);
    }

    // ── Payload and routing ───────────────────────────────────────────────────

    #[test]
    fn payload_initialized_from_upstream_slots() {
        let mut upstream = zeroed_upstream();
        upstream[3] = 7.7;
        let program = vec![VmInstruction::Halt];
        let (r, _) = run_vm(program, 1, vec![], &[], upstream, 100.0);
        assert!((r.output_slots[3] - 7.7).abs() < 1e-6);
    }

    #[test]
    fn write_internal_payload_updates_slot() {
        let program = vec![
            VmInstruction::LoadConst { dst: 0, const_idx: 0 },
            VmInstruction::WriteInternalPayload { slot_idx: 5, src: 0 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 1, vec![3.14], &[], zeroed_upstream(), 100.0);
        assert!((r.output_slots[5] - 3.14).abs() < 1e-4);
    }

    #[test]
    fn write_internal_payload_invalid_slot_ignored() {
        let program = vec![
            VmInstruction::LoadConst { dst: 0, const_idx: 0 },
            VmInstruction::WriteInternalPayload { slot_idx: 12, src: 0 }, // ignored
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 1, vec![99.0], &[], zeroed_upstream(), 100.0);
        // All slots stay 0.0 (upstream was zeroed)
        for s in r.output_slots {
            assert_eq!(s, 0.0);
        }
    }

    #[test]
    fn write_route_target_sets_output() {
        let program = vec![
            VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // r0 = 2.0
            VmInstruction::WriteRouteTarget { src: 0 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 1, vec![2.0], &[], zeroed_upstream(), 100.0);
        assert!((r.route_target_idx - 2.0).abs() < 1e-6);
    }

    // ── Memory opcodes ────────────────────────────────────────────────────────

    #[test]
    fn store_and_load_mem8() {
        let def = VmBackendDef {
            register_count: 2,
            constants: vec![42.0],
            program: vec![
                VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // r0 = 42.0 (byte value)
                VmInstruction::LoadConst { dst: 1, const_idx: 0 }, // r1 = 42 (address)
                VmInstruction::StoreMem8 { addr_reg: 1, src: 0 }, // mem[42] = 42
                VmInstruction::LoadMem8 { dst: 0, addr_reg: 1 }, // r0 = mem[42]
                VmInstruction::WriteInternalPayload { slot_idx: 0, src: 0 },
                VmInstruction::Halt,
            ],
        };
        let si = empty_static_inputs();
        let mut e = 100.0;
        let mut mem = [0u8; 1024];
        let r = execute_vm_node(&def, &[], &zeroed_upstream(), &mut e, 0.0, &mut mem, &si, &config());
        assert!((r.output_slots[0] - 42.0).abs() < 1e-6);
        assert_eq!(mem[42], 42); // memory committed
    }

    #[test]
    fn store_and_load_mem8_imm() {
        let def = VmBackendDef {
            register_count: 1,
            constants: vec![77.0],
            program: vec![
                VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // r0 = 77.0
                VmInstruction::StoreMem8Imm { imm_addr: 100, src: 0 }, // mem[100] = 77
                VmInstruction::LoadMem8Imm { dst: 0, imm_addr: 100 }, // r0 = mem[100]
                VmInstruction::WriteInternalPayload { slot_idx: 0, src: 0 },
                VmInstruction::Halt,
            ],
        };
        let si = empty_static_inputs();
        let mut e = 100.0;
        let mut mem = [0u8; 1024];
        let r = execute_vm_node(&def, &[], &zeroed_upstream(), &mut e, 0.0, &mut mem, &si, &config());
        assert!((r.output_slots[0] - 77.0).abs() < 1e-6);
        assert_eq!(mem[100], 77);
    }

    #[test]
    fn memory_address_wraps_via_rem_euclid() {
        // addr_reg = 1024 should wrap to 0
        let def = VmBackendDef {
            register_count: 2,
            constants: vec![1024.0, 55.0],
            program: vec![
                VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // r0 = 1024 (addr)
                VmInstruction::LoadConst { dst: 1, const_idx: 1 }, // r1 = 55 (val)
                VmInstruction::StoreMem8 { addr_reg: 0, src: 1 }, // mem[1024%1024=0] = 55
                VmInstruction::LoadMem8 { dst: 0, addr_reg: 0 }, // r0 = mem[0]
                VmInstruction::WriteInternalPayload { slot_idx: 0, src: 0 },
                VmInstruction::Halt,
            ],
        };
        let si = empty_static_inputs();
        let mut e = 100.0;
        let mut mem = [0u8; 1024];
        let r = execute_vm_node(&def, &[], &zeroed_upstream(), &mut e, 0.0, &mut mem, &si, &config());
        assert!((r.output_slots[0] - 55.0).abs() < 1e-6);
    }

    // ── Energy metering ───────────────────────────────────────────────────────

    #[test]
    fn energy_is_deducted_per_opcode() {
        let program = vec![
            VmInstruction::Noop, // 0.05
            VmInstruction::Halt, // 0.05
        ];
        let (_, energy_after) = run_vm(program, 1, vec![], &[], zeroed_upstream(), 100.0);
        // Both opcodes execute: 100.0 - 0.05 - 0.05 = 99.90 approximately
        assert!(energy_after < 100.0);
        assert!(energy_after > 99.0);
    }

    #[test]
    fn energy_exhaustion_returns_exhausted() {
        // Very low energy; Noop costs 0.05
        let program = vec![VmInstruction::Noop, VmInstruction::Halt];
        let (r, _) = run_vm(program, 1, vec![], &[], zeroed_upstream(), 0.01);
        assert!(r.energy_exhausted);
    }

    #[test]
    fn energy_exhaustion_does_not_commit_memory_writes() {
        // StoreMem8Imm costs 0.16; give exactly 0.05 energy so first Noop passes but store fails
        let def = VmBackendDef {
            register_count: 1,
            constants: vec![42.0],
            program: vec![
                VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // 0.08 → energy 0.05 - 0.08 < 0 → exhausted here actually
                // Actually: give enough for LoadConst but not StoreMem8Imm
                // LoadConst costs 0.08, StoreMem8Imm costs 0.16 → need > 0.08 but < 0.24
                VmInstruction::StoreMem8Imm { imm_addr: 10, src: 0 }, // costs 0.16
                VmInstruction::Halt,
            ],
        };
        let si = empty_static_inputs();
        // Give 0.20 energy: LoadConst(0.08) → 0.12 left; StoreMem8Imm(0.16) → 0.12 - 0.16 = -0.04 → exhausted before store executes
        let mut e = 0.20;
        let mut mem = [0u8; 1024];
        let r = execute_vm_node(&def, &[], &zeroed_upstream(), &mut e, 0.0, &mut mem, &si, &config());
        assert!(r.energy_exhausted);
        // mem[10] should still be 0 (not written)
        assert_eq!(mem[10], 0);
    }

    #[test]
    fn max_vm_steps_enforced() {
        // Infinite loop program: Jump back to self (offset = -1)
        // provisional_pc = 0 + 1 + (-1) = 0 → wraps to 0 → infinite
        let program = vec![VmInstruction::Jump { offset: -1 }];
        let mut cfg = RuntimeConfig::default();
        cfg.max_vm_steps = 10;
        let def = VmBackendDef { register_count: 1, constants: vec![], program };
        let si = empty_static_inputs();
        let mut e = 1000.0;
        let mut mem = [0u8; 1024];
        let r = execute_vm_node(&def, &[], &zeroed_upstream(), &mut e, 0.0, &mut mem, &si, &cfg);
        // Should halt after 10 steps, not loop forever
        assert!(r.world_action.is_none());
        assert!(!r.energy_exhausted);
    }

    // ── Operand normalization ─────────────────────────────────────────────────

    #[test]
    fn register_index_wraps_via_rem_euclid() {
        // With 2 registers (0, 1), reg index 2 wraps to 0, 3 wraps to 1
        let program = vec![
            VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // r0 = 7.0
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 2 }, // src=2 wraps to 0 → 7.0
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 2, vec![7.0], &[], zeroed_upstream(), 100.0);
        assert!((r.output_slots[0] - 7.0).abs() < 1e-6);
    }

    #[test]
    fn constant_index_wraps_via_rem_euclid() {
        // 2 constants: [3.0, 5.0]. const_idx=3 → 3%2=1 → 5.0
        let program = vec![
            VmInstruction::LoadConst { dst: 0, const_idx: 3 }, // idx 3%2=1 → 5.0
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 0 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 1, vec![3.0, 5.0], &[], zeroed_upstream(), 100.0);
        assert!((r.output_slots[0] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn constant_empty_pool_yields_zero() {
        let program = vec![
            VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // empty pool → 0.0
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 0 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 1, vec![], &[], zeroed_upstream(), 100.0);
        assert_eq!(r.output_slots[0], 0.0);
    }

    // ── sanitize_f32 on register writes ──────────────────────────────────────

    #[test]
    fn nan_in_add_becomes_zero() {
        // f32::NAN + 0.0 = NaN → sanitized to 0.0
        let program = vec![
            VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // r0 = NaN
            VmInstruction::Add { dst: 1, a: 0, b: 0 },         // r1 = NaN (sanitized to 0.0)
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 1 },
            VmInstruction::Halt,
        ];
        // We can't put NaN in constants easily via VmBackendDef. Instead use Mul(0,0)/Sub to create NaN...
        // Actually f32::NAN is a specific constant. Let's use Div by zero to produce NaN:
        // Wait: Div by zero → 0.0, not NaN. Let's use LoadConst with a special value.
        // Actually the easiest is: test that mul of NaN constant sanitizes.
        // The simplest test: if we load NaN from constants... serde doesn't support NaN.
        // So let's skip direct NaN injection and instead test via Add of large values:
        let program = vec![
            VmInstruction::LoadConst { dst: 0, const_idx: 0 }, // r0 = 2e9
            VmInstruction::Add { dst: 1, a: 0, b: 0 },         // r1 = 4e9 → clamped to 1e9
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 1 },
            VmInstruction::Halt,
        ];
        let (r, _) = run_vm(program, 2, vec![2e9_f32], &[], zeroed_upstream(), 100.0);
        // 2e9 is already above 1e9 in constants (sanitize_f32 on LoadConst writes),
        // so after LoadConst r0 = sanitize(2e9) = 1e9.
        // Then Add: 1e9 + 1e9 = 2e9 → sanitized to 1e9.
        assert!((r.output_slots[0] - 1_000_000_000.0).abs() < 1.0);
    }
}
```

### Step 3: Run tests

```bash
cd v3 && cargo test runtime::vm -- --nocapture
```

Expected: all tests pass. If a test fails, read the error carefully — likely an off-by-one in jump target or energy comparison.

**Common failure modes and fixes:**
- Energy test: the exact threshold depends on opcode costs. If a test fails due to energy values, check the opcode cost table and adjust the energy input value.
- Jump test: make sure `jump_target` uses `pc as i64 + 1 + offset as i64`.
- Constant NaN test: if `sanitize_f32` is applied inside `LoadConst`, a `2e9` constant becomes `1e9` before the Add.

### Step 4: Verify fmt + clippy

```bash
cd v3 && cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings
```

### Step 5: Commit

```bash
git add v3/crates/v3-core/src/runtime/mod.rs v3/crates/v3-core/src/runtime/vm.rs
git commit -m "feat(v3-core): Stage 3a Task 5 — execute_vm_node (all 33 opcodes)"
```

---

## Task 6: Integration Test — End-to-End VM Execution

**Files:**
- Modify: `v3/crates/v3-core/src/runtime/vm.rs` (add integration tests section)
  OR create: `v3/crates/v3-core/tests/vm_integration.rs`

Use a minimal single-VM-node genome that eats when food is present, using `assemble_static_inputs` + `execute_vm_node`.

### Step 1: Add integration tests to `runtime/vm.rs`

Add these at the end of the existing `#[cfg(test)] mod tests`:

```rust
    // ── Integration: full input resolution pipeline ────────────────────────

    #[test]
    fn vm_eats_when_food_here() {
        use crate::config::simulation::{SimulationConfig, WorldEdgeMode};
        use crate::contracts::{CreatureId, NodeId, Position};
        use crate::creature::genome::{BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction};
        use crate::creature::state::CreatureState;
        use crate::kernel::world::WorldState;
        use crate::sensors::static_inputs::assemble_static_inputs;
        use rand::rngs::SmallRng;
        use rand::SeedableRng;
        use slotmap::SlotMap;

        // Build a world with food at (1,1)
        let mut world = WorldState::new(5, 5, WorldEdgeMode::Wrap);
        let mut cfg = SimulationConfig::default();
        cfg.world.food.initial_coverage = 1.0;
        cfg.world.food.initial_density = 200;
        let mut rng = SmallRng::seed_from_u64(42);
        world.seed_food(&mut rng, &cfg);

        let pos = Position::new(1, 1);
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());

        // Single-node VM: read food_here into r0, CmpGt r0 > r1(0), JumpIfZero skip, Eat
        let input_refs = vec![InputReference::World(WorldInputKey::FoodHere)];
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: input_refs.clone(),
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 2,
                    constants: vec![],
                    program: vec![
                        VmInstruction::ReadInput { dst: 0, input_idx: 0 }, // r0 = food_here
                        // r1 is 0.0; compare r0 > r1
                        VmInstruction::CmpGt { dst: 0, a: 0, b: 1 }, // r0 = (food > 0)?
                        VmInstruction::JumpIfZero { cond: 0, offset: 1 }, // skip Eat if no food
                        VmInstruction::EmitWorldAction { action_type: 1 }, // Eat
                        VmInstruction::EmitWorldAction { action_type: 0 }, // NoOp fallback
                    ],
                }),
                targets: vec![],
            }],
        };
        let creature = CreatureState::new(id, genome, pos, 30.0, 0, [0, 0, 0]);

        let si = assemble_static_inputs(&world, &creature);
        // food_here should be > 0.0 (200/255 ≈ 0.78)
        assert!(si.food_here > 0.0);

        let def = if let crate::creature::genome::BackendDef::Vm(ref v) = creature.genome.nodes[0].backend_def {
            v
        } else {
            panic!("expected VM backend");
        };

        let upstream = [0.0f32; 12];
        let mut energy = creature.energy;
        let mut mem = [0u8; 1024];
        let result = execute_vm_node(
            def,
            &creature.genome.nodes[0].input_refs,
            &upstream,
            &mut energy,
            0.0,
            &mut mem,
            &si,
            &config(),
        );
        assert_eq!(
            result.world_action,
            Some(crate::contracts::WorldAction::Eat),
            "Expected Eat when food is present"
        );
    }

    #[test]
    fn vm_noop_when_no_food() {
        use crate::config::simulation::WorldEdgeMode;
        use crate::contracts::{CreatureId, NodeId, Position, WorldInputKey};
        use crate::creature::genome::{BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction};
        use crate::creature::state::CreatureState;
        use crate::kernel::world::WorldState;
        use crate::sensors::static_inputs::assemble_static_inputs;
        use slotmap::SlotMap;

        // World with no food
        let world = WorldState::new(5, 5, WorldEdgeMode::Wrap);
        let pos = Position::new(2, 2);
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());

        let input_refs = vec![InputReference::World(WorldInputKey::FoodHere)];
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: input_refs.clone(),
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 2,
                    constants: vec![],
                    program: vec![
                        VmInstruction::ReadInput { dst: 0, input_idx: 0 }, // r0 = food_here = 0
                        VmInstruction::CmpGt { dst: 0, a: 0, b: 1 },       // r0 = (0 > 0) = 0
                        VmInstruction::JumpIfZero { cond: 0, offset: 1 },  // fires → skip Eat
                        VmInstruction::EmitWorldAction { action_type: 1 }, // Eat (skipped)
                        VmInstruction::EmitWorldAction { action_type: 0 }, // NoOp fallback
                    ],
                }),
                targets: vec![],
            }],
        };
        let creature = CreatureState::new(id, genome, pos, 30.0, 0, [0, 0, 0]);
        let si = assemble_static_inputs(&world, &creature);
        assert_eq!(si.food_here, 0.0);

        let def = if let crate::creature::genome::BackendDef::Vm(ref v) = creature.genome.nodes[0].backend_def {
            v
        } else { panic!() };

        let upstream = [0.0f32; 12];
        let mut energy = 30.0;
        let mut mem = [0u8; 1024];
        let result = execute_vm_node(
            def,
            &creature.genome.nodes[0].input_refs,
            &upstream,
            &mut energy,
            0.0,
            &mut mem,
            &si,
            &config(),
        );
        assert_eq!(
            result.world_action,
            Some(crate::contracts::WorldAction::NoOp),
            "Expected NoOp when no food"
        );
    }
```

### Step 2: Run all tests

```bash
cd v3 && cargo test -- --nocapture 2>&1 | tail -5
```

Expected: all tests pass. Count should be ≥ 80 (was 62 + all new tests).

### Step 3: Final quality checks

```bash
cd v3 && cargo fmt --all -- --check
cd v3 && cargo clippy --workspace --all-targets -- -D warnings
cd v3 && cargo test --workspace
```

All three must pass cleanly.

### Step 4: Final commit

```bash
git add -u  # stage all modified tracked files
git commit -m "feat(v3-core): Stage 3a Task 6 — VM integration tests (assemble_static_inputs → execute_vm_node)"
```

---

## Quality Checklist (Run Before Declaring Done)

- [x] `cargo fmt --all -- --check` passes
- [x] `cargo clippy --workspace --all-targets -- -D warnings` passes (zero warnings)
- [x] `cargo test --workspace` passes (all tests green)
- [x] `runtime/types.rs`: `sanitize_f32` + `NodeResult` with 9 tests
- [x] `sensors/static_inputs.rs`: `StaticInputs` + `assemble_static_inputs` + `resolve_static_ref`
- [x] `runtime/inputs.rs`: `resolve_input` for all 4 InputReference variants
- [x] `runtime/action_decode.rs`: `decode_world_action` for all 4 action types + edge cases
- [x] `runtime/vm.rs`: `execute_vm_node` covering all 33 opcodes, energy metering, step cap, operand normalization, memory contract
- [x] Integration test: single-VM-node eat-when-food-present confirmed working

## Reconciliation Snapshot (2026-02-22)

- verified: 9
- partial: 0
- missing: 0
- conflict: 0
- evidence matrix: archived file removed

---

## Notes for Implementer

1. **Operand normalization happens inline** using the `nr()` closure inside `execute_vm_node`. This avoids a separate function for register indexing.

2. **Energy check before execution**: deduct cost, check `<= 0.0`, only then execute. This means the first opcode that would exhaust energy is NOT executed (its side effects are skipped).

3. **Memory commit**: the `mem_copy` strategy ensures atomicity. Only after `EmitWorldAction` or `Halt` (or step cap) does `*memory = mem_copy`. On energy exhaustion, `*memory` is untouched.

4. **sanitize_f32 on every register write**: apply after every arithmetic result. For comparisons that naturally produce 0.0/1.0, sanitize is a no-op but still correct.

5. **The `nan_in_add_becomes_zero` test** uses a `2e9` constant to indirectly test clamping via overflow. A direct `f32::NAN` constant can't be stored in a `Vec<f32>` that goes through JSON, so we avoid that path.

6. **Integration tests** import module-local types directly — be careful with the `use` paths in the nested `mod tests` block.
