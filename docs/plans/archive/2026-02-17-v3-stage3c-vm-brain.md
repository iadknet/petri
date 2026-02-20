# V3 Stage 3C: VM Brain

**Goal:** Replace the heuristic executor with a real VM execution engine. Creatures now think via a heritable VM program, making their cognition evolvable.

**Goal IDs:** GP-01, GP-02, GP-03, GP-04

**Scope:** Redesign `CreatureGenome` to a proper node-based VM schema, implement all 38 VM opcodes, introduce `creature/founders.rs` with a real VM-based founder, replace `execute_heuristic` with an energy-bound VM executor, update mutation to target node constants, update sensors/contracts for separate occupied/barrier fields. Excludes: graph nodes, multi-node routing (OutputDefinition), inventory actions, SensorFrame (beyond neighbor reads), phenotype evolution.

**See also:** `docs/reference/v3-vm-isa-spec.md`, `docs/reference/v3-genome-sensor-spec.md`

**Docs Impact:**
- Adds this plan only.
- No canonical strategy/reference docs changed in this slice.
- No stale docs retired.

**Supersedes:** none

**Superseded-By:** none

**Parent plan:** `docs/plans/2026-02-14-v3-architecture-design.md`

---

## Goal Alignment

- **GP-01:** VM brain replaces heuristic; creature cognition is now heritable and evolvable.
- **GP-02:** Genome types stay in `creature/genome.rs`; founders in `creature/founders.rs`; VM engine in `runtime/vm.rs`; no cross-boundary leakage.
- **GP-03:** TDD on all VM opcode paths; viability regressions preserved.
- **GP-04:** Energy consumed by VM execution is tracked and deducted from creature energy in real-time.

## Boundary Impact

- `v3-core/creature/genome.rs`: Replace minimal genome with full NodeId/VmInstruction/VmBackendDef/NodeGenome/CreatureGenome schema.
- `v3-core/creature/founders.rs` (new): Named founder registry (`simple`); defines the only founder with a real VM program.
- `v3-core/creature/mutation.rs`: Update to jitter constants within `VmBackendDef.constants` instead of the old flat `genome.constants`.
- `v3-core/contracts/inputs.rs`: Extend `NeighborSense` with separate `occupied: bool` and `barrier: bool` (replaces `passable: bool`).
- `v3-core/config/runtime/vm.rs` (new): `VmConfig { opcode_cost_multiplier: u32 }`.
- `v3-core/runtime/vm.rs` (new): VM execution engine (all 38 opcodes, sanitize_f32, output buffers, energy metering).
- `v3-core/runtime/executor.rs`: Replace `execute_heuristic` with `execute_vm_creature`.
- `v3-core/sensors/mod.rs`: Produce separate `occupied`/`barrier` fields in `NeighborSense`.
- `v3-core/tick/orchestrator.rs`: Phase 1 changes from `iter()` + heuristic to `iter_mut()` + VM executor.
- `v3-core/seed.rs`: Use `founders::get("simple")` instead of `CreatureGenome::simple_founder()`.
- Dependency direction unchanged (`v3-server → v3-core`). No transport API changes.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v3-core/creature/genome.rs` | change | Stage 3C breaks open the stub genome; this is the intended VM cutover. |
| `v3-core/runtime/executor.rs` | change | Stage 3B preserved heuristic to isolate genome/mutation risk. That risk is now resolved. |
| `v3-core/contracts/inputs.rs` | change | `NeighborSense.passable` conflates occupied and barrier; VM opcodes need separate flags per spec. |
| `v3-core/tick/orchestrator.rs` | change | VM executor requires `&mut creature.energy` during Phase 1; `iter_mut()` is required. |
| `v3-server/` | keep | No transport changes; server continues consuming `SimulationState::tick`. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should all 38 opcodes be implemented now or only a subset? | All 38, as per spec. Unimplemented variants should panic (invariant violation), not silently skip. | agent | resolved |
| Where does NodeGenome.input_refs live? | On `NodeGenome` (not VmBackendDef). `ReadInput(idx)` resolves against this list per spec §9. | agent | resolved |
| Should multi-node routing (OutputDefinition) be included? | No. `output_definitions` field exists but is always empty for Stage 3C. | agent | resolved |
| How are `EmitWorldAction` action types encoded? | 0=NoOp, 1=Eat, 2=Move, 3=Reproduce. Meta buffer: slot 0 = direction (0–7 float), slot 1 = energy_amount (Reproduce only). Direction: 0=N,1=NE,2=E,3=SE,4=S,5=SW,6=W,7=NW (Direction::ALL order). | agent | resolved |
| What does opcode_cost_multiplier default to? | 1. Opcode base costs (from ISA spec) are multiplied by this integer. Default 1 makes VM execution very cheap relative to food/decay values. This can be tuned without code changes. | agent | resolved |

---

## Implementation Checklist

### Task 1: Full Genome Schema + Founders Module

**Files:**
- Modify: `v3/crates/v3-core/src/creature/genome.rs`
- Create: `v3/crates/v3-core/src/creature/founders.rs`
- Modify: `v3/crates/v3-core/src/creature/mod.rs`

**Checklist:**
- [x] Add `NodeId(u32)` newtype with `PartialEq`, `Eq`, `Clone`, `Copy`, `Debug`.
- [x] Add `InputReference` enum: `World(WorldInputKey)`, `Introspection(IntrospectionInputKey)`, `NeighborCell { direction_idx: u8, field: NeighborCellField }`, `NeighborCreature { direction_idx: u8, field: NeighborCreatureField }`.
  - `WorldInputKey`: `FoodHere` only for Stage 3C (others are Stage 4+).
  - `IntrospectionInputKey`: `EnergyCurrent`, `Generation`, `AgeTicks`.
  - `NeighborCellField`: `FoodDensityNorm=0`, `BarrierFlag=1`, `OccupiedFlag=2`.
  - `NeighborCreatureField`: `PresentFlag=0`, `EnergyNorm=1` (others Stage 4+).
- [x] Add `VmInstruction` enum (all 38 opcodes with operand types from ISA spec). Use `u8` for register/const indices; `i16` for jump offsets.
- [x] Add `VmBackendDef { register_count: u8, program: Vec<VmInstruction>, constants: Vec<f32> }`.
- [x] Add `pub enum OutputDefinition {}` (empty enum — valid Rust, clearly signals Stage 4+ expansion point).
- [x] Add `NodeGenome { node_id: NodeId, input_refs: Vec<InputReference>, backend_def: VmBackendDef, output_definitions: Vec<OutputDefinition> }`.
  - `output_definitions` is always empty in Stage 3C (single-node VM emits via EmitWorldAction). Real variants added in Stage 4+.
- [x] Add `CreatureGenome { entry_node_id: NodeId, nodes: Vec<NodeGenome> }` with `PartialEq` derived.
- [x] Add `CreatureGenome::validate()` checking: nodes non-empty, all node IDs unique, entry_node_id exists.
- [x] Create `creature/founders.rs` with `get(name: &str) -> CreatureGenome` returning the `"simple"` founder.
  - `"simple"` founder: single VM node, 8 registers, 4 constants `[0.0, 300.0, 100.0, 2.0]`, 21-instruction program (eat→reproduce-N→move-N-if-food→move-E). `input_refs`: `[World(FoodHere), Introspection(EnergyCurrent)]`. See encoding table in this doc.
  - Panics with clear message on unknown name.
- [x] Expose `founders` module in `creature/mod.rs`.
- [x] Add tests: `CreatureGenome::validate()` passes for `simple` founder; `validate()` rejects empty nodes; unknown founder name panics.

### Task 2: VmConfig in Runtime Config

**Files:**
- Create: `v3/crates/v3-core/src/config/runtime/vm.rs`
- Modify: `v3/crates/v3-core/src/config/runtime/mod.rs`

**Checklist:**
- [x] Add `VmConfig { opcode_cost_multiplier: u32 }` with `Default` impl (multiplier = 1).
- [x] Add `RuntimeConfig.vm: VmConfig`.
- [x] Add contract test: `SimulationConfig::default().runtime.vm.opcode_cost_multiplier == 1`.

### Task 3: VM Execution Engine

**Files:**
- Create: `v3/crates/v3-core/src/runtime/vm.rs`
- Modify: `v3/crates/v3-core/src/runtime/mod.rs`

**Checklist:**
- [x] Add `sanitize_f32(v: f32) -> f32`: NaN→0.0, +Inf→1e9, -Inf→-1e9, magnitude>1e9 clamped.
- [x] Add `VmState { registers: Vec<f32>, pc: usize, world_action_meta: [f32; 8], internal_payload: [f32; 8] }`.
- [x] Implement `execute_vm_node(node: &NodeGenome, inputs: &CreatureInputs, memory: &mut [u8], energy: &mut Energy, vm_config: &VmConfig) -> Option<WorldAction>`.
  - Builds register file of `register_count` zeros.
  - Resolves `input_refs` to f32 values from `inputs` (slot-based lookup for `ReadInput`).
  - Executes instructions until halt, energy exhausted, or PC out of bounds.
  - Each opcode deducts `(base_cost * opcode_cost_multiplier) as u32` from energy via `energy.drain()`; if drain fails, halt immediately.
  - `EmitWorldAction(action_type)` decodes the action from the meta buffer and halts.
  - Returns `None` if no `EmitWorldAction` fired; caller defaults to `WorldAction::NoOp`.
- [x] Implement all 38 opcodes per ISA spec. Panic on out-of-bounds register/const/payload index (invariant violation).
- [x] Jump semantics: `PC += 1` after each instruction; `Jump`/`JumpIfZero` add offset to post-increment PC. Negative PC panics. PC >= program length halts normally.
- [x] Memory: `LoadMem8`/`StoreMem8` read address from `registers[addr_reg_idx]` as f32→usize, mod 1024. `LoadMem8Imm`/`StoreMem8Imm` use immediate `usize` address directly mod 1024.
- [x] `ReadInput(dst, slot_idx)`: if `slot_idx >= input_refs.len()` return 0.0 (soft default per spec, not a panic).
- [x] `ReadNeighborCell(dst, neighbor_idx, field_idx)`: neighbor_idx maps to `Direction::ALL[neighbor_idx]`; panic on invalid neighbor_idx (>= 8). field_idx: 0=FoodDensityNorm, 1=BarrierFlag, 2=OccupiedFlag; invalid field_idx returns 0.0 (soft default).
- [x] `ReadNeighborCreature(dst, neighbor_idx, field_idx)`: neighbor_idx panics if >= 8. field_idx 0=PresentFlag (= `neighbor.occupied as f32`); all other field_idx values return 0.0 (Stage 4+ expansion). This avoids the need for creature lookups in Stage 3C.
- [x] `ReadSensorCell/ReadSensorCreature/ReadSensorSummary`: return 0.0 for all inputs in Stage 3C regardless of indices (SensorFrame deferred to Stage 4+; no panic — soft default to allow mutated programs to call these opcodes).
- [x] Add unit tests: `sanitize_f32` table, jump offset semantics, memory wrap (addr mod 1024), `ReadInput` out-of-range returns 0.0, `EmitWorldAction` decodes correct `WorldAction`.

### Task 4: Update Contracts and Sensors

**Files:**
- Modify: `v3/crates/v3-core/src/contracts/inputs.rs`
- Modify: `v3/crates/v3-core/src/sensors/mod.rs`

**Checklist:**
- [x] Change `NeighborSense { food_density: u8, passable: bool }` to `NeighborSense { food_density: u8, occupied: bool, barrier: bool }`.
- [x] Add `NeighborSense::passable(&self) -> bool { !self.occupied && !self.barrier }` computed method.
- [x] Update `sensors/mod.rs` to produce `occupied`/`barrier` separately from `world.is_occupied()` and `world.is_barrier()`.
- [x] Update all existing callsites of `n.passable` to use `n.passable()` method.

### Task 5: Replace Heuristic Executor

**Files:**
- Modify: `v3/crates/v3-core/src/runtime/executor.rs`

**Checklist:**
- [x] Replace `execute_heuristic` with `execute_vm_creature(genome: &CreatureGenome, inputs: &CreatureInputs, memory: &mut [u8], energy: &mut Energy, config: &SimulationConfig) -> CreatureOutputs`.
  - Looks up `entry_node_id` in `genome.nodes` (panic if not found — invariant violation).
  - Calls `vm::execute_vm_node` on the entry node.
  - Returns `CreatureOutputs` with the emitted `WorldAction` (or `WorldAction::NoOp` if None).
- [x] Keep public API consistent: function name changes; `rng` parameter removed (VM is deterministic given state).
- [x] Add failing test for eat decision: a genome whose VM program emits Eat produces `WorldAction::Eat`.
- [x] Add failing test for NoOp default when program halts without EmitWorldAction.

### Task 6: Update Mutation for New Genome

**Files:**
- Modify: `v3/crates/v3-core/src/creature/mutation.rs`

**Checklist:**
- [x] Update `mutate_genome` to jitter constants in `genome.nodes[random_idx].backend_def.constants` instead of the old flat `genome.constants`.
- [x] If all nodes have empty constants pools, return early (no mutation possible).
- [x] Existing `MutationConfig` fields unchanged (`mutation_probability`, `per_birth_mutation_events_min/max`, `constant_jitter_magnitude`).
- [x] Add test: mutation on `simple` founder genome changes a constant value.

### Task 7: Update Orchestrator and All Callsites

**Files:**
- Modify: `v3/crates/v3-core/src/tick/orchestrator.rs`
- Modify: `v3/crates/v3-core/src/seed.rs`
- Modify: `v3/crates/v3-core/tests/creature_state_test.rs`
- Modify: `v3/crates/v3-core/tests/tick_actions_test.rs`
- Modify: `v3/crates/v3-core/tests/tick_integration_test.rs`
- Modify: `v3/crates/v3-core/tests/runtime_executor_test.rs`
- Modify: `v3/crates/v3-core/tests/contracts_test.rs`
- Modify: `v3/crates/v3-core/tests/seed_test.rs`
- Modify: `v3/crates/v3-core/tests/seed_viability.rs`
- Modify: `v3/crates/v3-core/tests/seed_viability_stage3.rs`

**Checklist:**
- [x] Update Phase 1 in orchestrator: use `iter_mut()`, call `execute_vm_creature` with `&creature.genome`, `&mut creature.memory`, `&mut creature.energy`; remove `rng` from executor call.
- [x] Update `seed.rs`: replace `CreatureGenome::simple_founder()` with `founders::get("simple")`.
- [x] Update all test callsites: replace `CreatureGenome { constants: vec![...] }` with `founders::get("simple")` (or a minimal inline `NodeGenome`/`CreatureGenome` for targeted unit tests).
- [x] Update `runtime_executor_test.rs` for new `execute_vm_creature` signature.
- [x] Confirm `seed_viability_stage3.rs` still passes (genome divergence test still valid with new mutation targeting node constants).
- [x] Note: `creature/reproduction.rs` compiles unchanged — `create_offspring` just clones `parent.genome: CreatureGenome`; the field name is the same in the new schema.

### Task 8: Verification

**Files:**
- Modify: `docs/plans/2026-02-17-v3-stage3c-vm-brain.md`

**Checklist:**
- [x] Mark completed items in this plan.
- [x] Run `cd v3 && cargo test --workspace`.
- [x] Run `cd v3 && cargo fmt --all --check`.
- [x] Run `cd v3 && cargo clippy --workspace --all-targets -- -D warnings`.

---

## Simple Founder VM Program (21 instructions)

**Constants:** `[0.0, 50.0, 100.0, 2.0]` (indices 0–3)
**input_refs:** `[World(FoodHere), Introspection(EnergyCurrent)]` (slots 0–1)
**Registers:** 8 (r0–r7), `register_count = 8`

Behavior: Eat if food at current cell → Reproduce N if energy > 50 → Move N if food at N → Move E (fallback).

Note: threshold 50.0 was corrected from 300.0 during implementation; 300.0 exceeded `max_energy` in all test configs making reproduction unreachable.

| idx | Instruction | Effect |
|-----|-------------|--------|
| 0 | `ReadInput r0, 0` | r0 = food_here_norm |
| 1 | `LoadConst r6, 0` | r6 = 0.0 |
| 2 | `CmpGt r1, r0, r6` | r1 = food > 0? |
| 3 | `JumpIfZero r1, +1` | no food → skip to 5 (PC=4+1=5) |
| 4 | `EmitWorldAction 1` | **Eat**, halt |
| 5 | `ReadInput r2, 1` | r2 = energy_current |
| 6 | `LoadConst r3, 1` | r3 = 50.0 |
| 7 | `CmpGt r4, r2, r3` | r4 = energy > 50? |
| 8 | `JumpIfZero r4, +4` | not enough → skip to 13 (PC=9+4=13) |
| 9 | `WriteWorldActionMeta 0, r6` | meta[0] = 0.0 (dir N = 0) |
| 10 | `LoadConst r5, 2` | r5 = 100.0 |
| 11 | `WriteWorldActionMeta 1, r5` | meta[1] = 100.0 (offspring energy) |
| 12 | `EmitWorldAction 3` | **Reproduce N**, halt |
| 13 | `ReadNeighborCell r0, 0, 0` | r0 = N food density norm |
| 14 | `CmpGt r1, r0, r6` | food at N? |
| 15 | `JumpIfZero r1, +2` | no food at N → skip to 18 (PC=16+2=18) |
| 16 | `WriteWorldActionMeta 0, r6` | meta[0] = 0.0 (dir N) |
| 17 | `EmitWorldAction 2` | **Move N**, halt |
| 18 | `LoadConst r0, 3` | r0 = 2.0 (dir E = constant idx 3) |
| 19 | `WriteWorldActionMeta 0, r0` | meta[0] = 2.0 (dir E) |
| 20 | `EmitWorldAction 2` | **Move E**, halt |

Jump offset rule: `JumpIfZero cond, offset` at index N — default PC = N+1; if jumping (cond falsy): PC = (N+1)+offset.

---

## Action Type and Direction Encoding

| action_type | WorldAction | Meta slot 0 | Meta slot 1 |
|-------------|-------------|-------------|-------------|
| 0 | NoOp | ignored | ignored |
| 1 | Eat | ignored | ignored |
| 2 | Move | direction (0–7) | ignored |
| 3 | Reproduce | direction (0–7) | energy_amount (f32 as u32) |

Direction encoding: 0=N, 1=NE, 2=E, 3=SE, 4=S, 5=SW, 6=W, 7=NW (Direction::ALL index). Round-to-nearest, clamp to 0–7.

---

## Verification Commands

- `cd v3 && cargo test -p v3-core --test contracts_test`
- `cd v3 && cargo test -p v3-core --test creature_state_test`
- `cd v3 && cargo test -p v3-core --test runtime_executor_test`
- `cd v3 && cargo test -p v3-core --test seed_viability`
- `cd v3 && cargo test -p v3-core --test seed_viability_stage3`
- `cd v3 && cargo test --workspace`
- `cd v3 && cargo fmt --all --check`
- `cd v3 && cargo clippy --workspace --all-targets -- -D warnings`

---

## Risks and Rollback

- Risk: VM founder program too simple to maintain viability under default energy tuning.
  - Mitigation: Viability test uses generous food config; simple eat/move loop is sufficient.
- Risk: NeighborSense API change cascades through more callsites than identified.
  - Mitigation: Compiler enforces `passable` removal; all sites caught at compile time. Use `passable()` method for backward compatibility.
- Risk: VM energy metering with `opcode_cost_multiplier=1` makes execution effectively free and could allow infinite loops.
  - Mitigation: `max_vm_program_len=512` bounds loop iterations; programs halt when `pc >= program.len()`. Infinite VM loops cannot exist with a bounded program and non-negative PC.
- Rollback: Revert Stage 3C files, restore Stage 3B genome schema, re-run Stage 3B tests.

**Review cycles:** 1

Cycle 1 (architecture + goal alignment): Found three issues:
1. `Vec<()>` placeholder for `output_definitions` — replaced with proper `pub enum OutputDefinition {}` (empty Rust enum; clear Stage 4+ expansion point).
2. Founder program had two unreachable Noop instructions — simplified to 21-instruction version with correct jump offsets.
3. `ReadNeighborCreature` field behavior for unsupported fields was unspecified — added explicit rule: field_idx 0=PresentFlag, all others return 0.0 (soft default). Also clarified `ReadSensorCell/Creature/Summary` return 0.0 in Stage 3C (SensorFrame deferred).
No architectural violations or goal misalignments found. Dependency direction unchanged. All module ownership correct.
