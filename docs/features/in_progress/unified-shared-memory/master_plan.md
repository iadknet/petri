# Unified Shared Memory — Implementation Plan

**Parent refinement:** `refinement.md`

**Goal:** Replace the creature-level `[u8; 1024]` memory with a shared `[f32; 16]` memory bank accessible to both VM and graph backends, add temporal primitives (prev-tick snapshot, clear, decay), and add mutation motifs for memory circuits.

**Goal IDs:** GP-01, GP-02, GP-03, GP-04
**Scope:** v3-core creature state, VM ISA, graph backend, mutation engine, reproduction, tick orchestration, v3-server wire format, frontend creature inspector. Out of scope: multiple memory classes, inter-creature communication, frontend memory debugger/timeline.
**Docs Impact:** `docs/reference/v3-vm-isa-spec.md` (Section 8 rewrite, opcode table update), `docs/reference/v3-graph-backend-spec.md` (new node kinds), `docs/reference/v3-mutation-spec.md` (new motif operators)
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- **GP-01** (Evolve Richer Decision-Making): Shared memory gives all backends equal access to persistent state, enabling temporal reasoning. Mutation motifs lower the barrier for evolution to discover memory-using circuits.
- **GP-02** (Clean Architecture Boundaries): Shared memory is a creature-level concept on `CreatureState`; VM and graph access it through their existing execution interfaces without cross-boundary coupling. Graph staged writes follow the established deferred-effect pattern.
- **GP-03** (High-Confidence Iteration): TDD throughout. Viability tests run as merge gate. All existing tests must pass after migration.
- **GP-04** (Observable Behavior): Unified memory surface enables consistent "uses memory" metrics across both backends. Frontend shows shared slots directly.

## Boundary Impact

| Area | Change | Impact |
|------|--------|--------|
| `creature/state.rs` | Replace `memory: [u8; 1024]` with `shared_memory: [f32; 16]` + `prev_shared_memory: [f32; 16]` | Internal state change, ~1KB savings per creature |
| `creature/genome/mod.rs` | Replace 4 old `VmInstruction` memory variants with 6 new slot variants; add 4 `GraphNodeKind` variants | Genome schema change (breaking serde change — no migration, clean replace) |
| `config/simulation.rs` | Add `SharedMemoryConfig { decay_rate }` (derives `Copy`) as field on `SimulationConfig` with `#[serde(default)]` | Config extension |
| `runtime/vm.rs` + `traced_vm.rs` | Replace `memory: &mut [u8; 1024]` param with `shared_memory: &mut [f32; 16]` + `prev_shared_memory: &[f32; 16]` | VM executor signature change |
| `runtime/graph.rs` + `traced_graph.rs` | Pass shared memory through; staged write collection and post-convergence commit | Graph executor extension |
| `runtime/graph_effects.rs` | Commit staged slot writes alongside existing graph effects (Phase 1 and Phase 2 use `_ => {}` wildcards — compiler won't catch missing arms for `WriteSlot`/`ClearSlot`) | Effect system extension |
| `runtime/trace.rs` | Update `MemoryWrite` struct fields (`address: u16` + `u8` values → `slot_idx: u8` + `f32` values); rename `VmTrace.memory_writes` to `slot_writes`; update `kind_label` exhaustive match for new `GraphNodeKind` variants | Trace schema change |
| `runtime/mesh.rs` + `traced_mesh.rs` | Pass shared memory through to VM/graph executors | Call site update |
| `creature/genome/analysis.rs` | Update `vm_register_read_mask` and `vm_is_output_instruction` exhaustive matches for new slot opcodes; update `graph_is_output_node` to include `WriteSlot`/`ClearSlot` (uses `matches!()` — compiler won't catch missing arms) | Analysis function update |
| `mutation/vm/mod.rs` | Update random instruction generation; add 4 motif operators | Mutation engine extension |
| `mutation/graph/mod.rs` | Add slot node kinds to operator swap pool and random kind selection; update `is_parameterized` and `mutate_operator_param` (both use `matches!()`/`_ =>` wildcards — compiler won't catch missing arms) and `apply_graph_raw_field_mutation` for `slot_idx` fields | Mutation engine extension |
| `mutation/types.rs` | Add motif operators to `MutationOperator` enum | Type extension |
| `simulation/actions/reproduction.rs` | Copy `shared_memory` to child; zero `prev_shared_memory` | Reproduction semantics |
| `simulation/tick.rs` | Add snapshot + decay phase at tick start before cognition | Tick orchestration |
| `v3-server/handlers/creature.rs` | Serialize `shared_memory` as `number[]` instead of `memory` byte array | Wire format change |
| `frontend/inspector/MemoryHexView.tsx` | Replace with `SharedMemoryView` showing 16 f32 slots | UI replacement |

## Existing Boundary Recheck

| Area | Decision | Rationale |
|------|----------|-----------|
| `creature/state.rs` | change | Shared memory replaces VM-only byte memory; becomes creature-level shared state accessible by both backends |
| `runtime/` (vm, graph, mesh) | change | Execution functions gain shared memory parameters; graph uses established deferred-effect pattern for writes |
| `mutation/` | change | New motif operators follow existing `VmOperator` pattern with weights and complexity effects |
| `simulation/tick.rs` | change | Tick-start phase for snapshot + decay follows existing phase ordering convention |
| `v3-server` wire format | change | `memory` field replaced by `shared_memory`; `exclude` filter updated |
| Frontend inspector | change | `MemoryHexView` replaced entirely — 16 f32 slots need different visualization than 1024 u8 bytes |

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Slot count | 16 fixed (`const SHARED_MEMORY_SLOTS: usize = 16`) — small enough for easy 4-bit addressing; large enough for non-trivial state. Fixed array for performance. | Architect | Resolved |
| Temporal primitives | Prev-tick snapshot on CreatureState + `LoadSlotPrev`/`ReadSlotPrev` opcodes/nodes; latch is inherent; `ClearSlot` explicit. Simple, no per-slot metadata. | Architect | Resolved |
| Decay model | Global `SharedMemoryConfig.decay_rate` (default 0.0, normalized to `[0.0, 1.0]`) applied at tick start. Per-slot decay adds genome complexity; global is simpler and tunable. | Architect | Resolved |
| Energy costs | LoadSlot 0.12, StoreSlot 0.14, LoadSlotImm 0.10, StoreSlotImm 0.12, LoadSlotPrev 0.10, ClearSlot 0.12. Slightly cheaper than old u8 ops to encourage use. | Architect | Resolved |
| Graph WriteSlot commits | After each graph node's relaxation loop via `apply_graph_effects` extension. Consistent with existing deferred-effect pattern. | Architect | Resolved |
| Wire format | `shared_memory: number[]` on creature detail endpoint. Simple JSON array of f32 values. | Architect | Resolved |
| Motifs and reachability | Independent of reachability-aware mutation feature. Motifs operate within individual VM nodes; no cross-node targeting needed. | Architect | Resolved |
| Coordination with reachability-based-complexity | Both features touch `vm_is_output_instruction`. Whichever lands second updates references. Merge is clean since only match arms change. | Architect | Resolved |

## Design Details

### VM Opcodes (replacing LoadMem8/StoreMem8 family)

| Opcode | Operands | Semantics | Base Cost |
|--------|----------|-----------|-----------|
| `LoadSlot` | dst, slot_reg | `dst = shared_memory[reg[slot_reg] % 16]` | 0.12 |
| `StoreSlot` | slot_reg, src | `shared_memory[reg[slot_reg] % 16] = sanitize_f32(reg[src])` | 0.14 |
| `LoadSlotImm` | dst, slot_idx(u8) | `dst = shared_memory[slot_idx % 16]` | 0.10 |
| `StoreSlotImm` | slot_idx(u8), src | `shared_memory[slot_idx % 16] = sanitize_f32(reg[src])` | 0.12 |
| `LoadSlotPrev` | dst, slot_idx(u8) | `dst = prev_shared_memory[slot_idx % 16]` | 0.10 |
| `ClearSlot` | slot_idx(u8) | `shared_memory[slot_idx % 16] = 0.0` | 0.12 |

All slot writes pass through `sanitize_f32()` to prevent NaN/Infinity from persisting in shared memory across ticks. `ClearSlot` costs 0.12 (matching `StoreSlotImm` tier) — consistent with the pattern that write operations cost at least as much as the corresponding read.

VM slot write semantics: at 64 bytes, unconditionally copy `shared_memory` to a working buffer at VM entry (no `has_memory_ops()` conditional — the branch overhead exceeds the trivial copy cost). Commit working buffer on non-exhaustion exit; discard on energy exhaustion. Remove the `has_memory_ops()` fast-path entirely.

### Graph Node Kinds

| Kind | Semantics |
|------|-----------|
| `ReadSlot(u8)` | Output = `shared_memory[slot_idx % 16]` (reads snapshot from before graph eval) |
| `ReadSlotPrev(u8)` | Output = `prev_shared_memory[slot_idx % 16]` |
| `WriteSlot(u8)` | Stages `wsum` as pending write to slot; committed post-convergence |
| `ClearSlot(u8)` | Stages `0.0` as pending write to slot; committed post-convergence |

Graph `ReadSlot`/`ReadSlotPrev` read directly from the creature's live `shared_memory`/`prev_shared_memory` fields (passed as `&[f32; 16]` references via `EvalCtx`). This is safe because `WriteSlot`/`ClearSlot` do not modify `shared_memory` during relaxation — they only output `wsum`/`0.0` respectively (no side effects in `evaluate_kind`), matching the existing deferred-effect pattern used by `PushAction`/`CustomOutput`.

**Staging mechanism**: There is no separate staging buffer. The converged `curr_outputs[i]` array implicitly serves as the staged values — the same pattern used by `CustomOutput`, `RouterOutput`, and `WriteActionMeta`. Post-convergence, `apply_graph_effects` (extended with `shared_memory: &mut [f32; 16]` parameter) reads `curr_outputs[i]` for `WriteSlot`/`ClearSlot` nodes and commits the values. All graph slot writes pass through `sanitize_f32()`.

### Mutation Motifs (new VmOperator variants)

| Operator | Behavior | Weight | Complexity |
|----------|----------|--------|------------|
| `VmInsertReadStoreMotif` | Insert `ReadInput{dst=R, ..} + StoreSlotImm{slot=S, src=R}` pair at random position | 2 | Increasing |
| `VmInsertLoadCompareMotif` | Insert `LoadSlotImm{dst=R, slot=S} + CmpGt{dst=R2, a=R, b=R3}` pair at random position | 2 | Increasing |
| `VmMutateSlotAddress` | Find existing slot opcode, mutate its slot_idx field | 4 | Neutral |
| `VmMutatePairedSlotAddress` | Scan program for all `LoadSlotImm`/`StoreSlotImm` instructions, group by `slot_idx`. Pick a random group with at least one load AND one store. Mutate all instructions in the group to a new random `slot_idx`. Skip (`NoApplicableTarget`) if no such group exists. | 4 | Neutral |

### Tick-Start Memory Phase

Fold into the existing `run_phase_0` creature loop (which already iterates all creatures for aging and energy decay) to avoid an extra O(N) pass. Within the same loop body:
1. `prev_shared_memory = shared_memory`
2. If `decay_rate > 0.0`: `shared_memory[i] *= (1.0 - decay_rate)` for each slot

### Reproduction

- `shared_memory`: copied from parent to child (inherited). Pass as parameter to `CreatureState::new()` rather than post-construction field assignment.
- `prev_shared_memory`: zeroed in child via `CreatureState::new()` (not inherited — child has no "previous tick")

## Required Skills

- Rust/backend changes: invoke `rust-skills` BEFORE writing any Rust code and before each review
- Frontend changes: invoke `vercel-react-best-practices` and `vercel-composition-patterns`
  BEFORE writing any frontend code and before each review
- Frontend UI/design: invoke `web-design-guidelines` and `frontend-design` BEFORE writing any UI
  code; use `agent-browser` for screenshot-based design validation after each step

## TDD Policy

For all behavior changes and bug fixes: write a failing test FIRST, then implement.
A step is not complete until:
1. The failing test exists and is committed
2. The implementation makes it pass
3. No existing tests regress

Frontend: e2e tests using `agent-browser` MUST be written per user-facing step.
Frontend UI changes: use `agent-browser` screenshots + `web-design-guidelines` review after each step.
Repeat screenshot + review until clean (recursive).

## Code Review Policy

After completing each implementation step:
1. Run a thorough code review (backend: `rust-skills`; frontend: vercel skills)
2. Fix ALL findings
3. Run review AGAIN — repeat until no new findings (clean recursive pass)
4. Only after clean pass: commit the step

## Commit Policy

- Commits happen AFTER a clean code review pass, never before
- One commit per implementation step (focused, atomic)
- Do NOT advance to the next step until current step is committed and reviewed clean

## Implementation Steps

### Phase 1: Core Data Model

- [x] Step 1: Replace `memory: [u8; 1024]` with `shared_memory: [f32; 16]` and `prev_shared_memory: [f32; 16]` on `CreatureState`. Add `pub const SHARED_MEMORY_SLOTS: usize = 16` to `creature/state.rs`. Add a compile-time size assertion (`const _: () = assert!(size_of::<CreatureState>() <= ...);`) to lock in the size reduction and catch future bloat. Add `SharedMemoryConfig { decay_rate: f32 }` to `SimulationConfig` with `#[serde(default)]` on the field (derive `Copy, Debug, Clone, serde::Serialize, serde::Deserialize` + `#[serde(deny_unknown_fields)]` on the struct — matching all other config sub-structs; default `decay_rate = 0.0`). Add normalization in `SimulationConfig::normalize()` using the existing `normalize_f32_clamp` helper: clamp `decay_rate` to `[0.0, 1.0]` with fallback to `0.0` for NaN/non-finite values. Add `shared_memory: [f32; SHARED_MEMORY_SLOTS]` as a parameter to `CreatureState::new()`. Update all call sites: founders pass `[0.0; SHARED_MEMORY_SLOTS]`; reproduction temporarily passes `[0.0; SHARED_MEMORY_SLOTS]` (Step 3 will fix to use parent's memory). Fix all compilation errors (tests, seeding, reproduction, server handler). Update variant count test (`vm_instruction_all_39_variants_constructible` → 41). Run `cargo test --workspace`.

- [x] Step 2: Add tick-start memory phase. Fold into the existing `run_phase_0` creature loop in `simulation/tick.rs` (avoid extra O(N) pass): (a) copy `shared_memory` to `prev_shared_memory` for each creature, (b) apply decay if `config.shared_memory.decay_rate > 0.0`. TDD: test that prev snapshot matches pre-tick state; test that decay reduces non-zero slots.

- [x] Step 3: Update reproduction to pass parent's `shared_memory` to `CreatureState::new()` for the child. `prev_shared_memory` is zeroed by the constructor. Remove any post-construction field assignment patterns for memory. TDD: test inherited memory values and zeroed prev.

### Phase 2: VM Access

- [x] Step 4: Replace VM memory opcodes. Remove `LoadMem8`, `StoreMem8`, `LoadMem8Imm`, `StoreMem8Imm` from `VmInstruction` enum. Add `LoadSlot { dst, slot_reg }`, `StoreSlot { slot_reg, src }`, `LoadSlotImm { dst, slot_idx }`, `StoreSlotImm { slot_idx, src }`, `LoadSlotPrev { dst, slot_idx }`, `ClearSlot { slot_idx }`. Update `execute_vm_node` and `execute_vm_node_traced` signatures to take `&mut [f32; 16]` + `&[f32; 16]` instead of `&mut [u8; 1024]`. Implement opcode semantics: unconditionally copy `shared_memory` to working buffer at VM entry (no `has_memory_ops()` conditional — 64 bytes is trivial); commit on non-exhaustion exit; discard on exhaustion. Remove `has_memory_ops()` method entirely. All slot writes use `sanitize_f32()`. Update opcode cost table. Update `creature/genome/analysis.rs`: add new slot opcodes to `vm_register_read_mask` (note: `LoadSlot.slot_reg` is a register dependency, `LoadSlotImm.slot_idx` is not) and `vm_is_output_instruction` (`StoreSlot`, `StoreSlotImm`, `ClearSlot` are output/side-effecting instructions). Update `MemoryWrite` trace struct in `runtime/trace.rs` (change `address: u16` + `u8` values to `slot_idx: u8` + `f32` values; rename `VmTrace.memory_writes` to `slot_writes`). Update `traced_vm.rs`: trace recording logic for new slot opcodes AND `written_regs!` macro (add `LoadSlot { dst, .. }`, `LoadSlotImm { dst, .. }`, `LoadSlotPrev { dst, .. }` to register-writing arm). TDD: test each new opcode, test wrapping addressing, test exhaustion rollback, test sanitize_f32 on writes.

- [x] Step 5: Update mesh executor call sites. Pass `shared_memory`/`prev_shared_memory` through `mesh.rs` and `traced_mesh.rs` to VM executor. Fix all remaining compilation errors. Run `cargo test --workspace`.

- [x] Review Gate: Interim code review — review Steps 1-5 changes. Fix findings, re-review until clean.

### Phase 3: Graph Access

- [x] Step 6: Add `ReadSlot(u8)`, `ReadSlotPrev(u8)`, `WriteSlot(u8)`, `ClearSlot(u8)` to `GraphNodeKind`. For ReadSlot/ReadSlotPrev: implement in `evaluate_kind` reading from `&[f32; 16]` references passed via `EvalCtx`. For WriteSlot/ClearSlot: `evaluate_kind` must only return `wsum`/`0.0` respectively (no side effects — matching PushAction/CustomOutput deferred-effect pattern). Pass `shared_memory` and `prev_shared_memory` as `&[f32; 16]` fields on `EvalCtx`. Update variant count test (`graph_node_kinds_all_26_constructible` → 30). Update `kind_label` in `runtime/trace.rs` for new graph node kinds. Update `graph_is_output_node` in `creature/genome/analysis.rs` to include `WriteSlot` and `ClearSlot` in the `matches!()` pattern (uses implicit wildcard — compiler won't catch missing arms). TDD: test read returns correct slot value, test staged write does not affect reads during same evaluation, test WriteSlot evaluate_kind has no side effects.

- [x] Step 7: Implement staged slot write commit. Extend `apply_graph_effects` with an additional `shared_memory: &mut [f32; 16]` parameter. In the existing Phase 1 (staged-value writes) scan, add WriteSlot/ClearSlot handling: commit `sanitize_f32(curr_outputs[node_idx])` or `0.0` to the corresponding slot. This avoids a second `internal_nodes` iteration. Update `execute_graph_impl` to pass shared memory through to `apply_graph_effects`. Update `traced_graph.rs`. TDD: test that WriteSlot value appears in shared_memory after evaluation, test ClearSlot zeros the slot, test sanitize_f32 on commits.

### Phase 4: Mutation

- [x] Step 8: Update VM mutation for new opcodes. In `random_vm_instruction`: update range bound (39 → 41) and match arm indices for new slot opcodes. In `VmInstructionRawFieldMutation` (`mutate_instruction_raw_fields`): handle `slot_idx` fields on slot opcodes. In `remap_register_refs`: add new slot opcodes — `slot_reg` fields are register refs (must be remapped), but `slot_idx` fields are literals (must NOT be remapped). TDD: test that random VM mutations can produce slot opcodes; test that register remapping handles slot opcodes correctly.

- [x] Step 9: Update graph mutation for new node kinds. Add `ReadSlot`, `ReadSlotPrev`, `WriteSlot`, `ClearSlot` to `SwapGraphOperator` kind pool, `AddInternalGraphNode` random kind selection (`random_graph_node_kind` range 26 → 30), `is_parameterized` filter (all 4 new kinds carry a `u8` parameter), and `mutate_operator_param` match arms (wrapping `slot_idx` via `wrapping_add`/`wrapping_sub` with modulo 16, matching the `CustomOutput`/`WriteActionMeta`/`PushAction` u8 mutation pattern — without these, `is_parameterized` returning true will hit `unreachable!()`). Update `GraphRawFieldMutation` to handle `slot_idx` fields on slot node kinds. TDD: test that graph mutations can produce slot node kinds; test that `is_parameterized` returns true for slot kinds; test that `mutate_operator_param` does not panic on slot kinds.

- [x] Step 10: Add mutation motifs. Implement `VmInsertReadStoreMotif`, `VmInsertLoadCompareMotif`, `VmMutateSlotAddress`, `VmMutatePairedSlotAddress`. Add to `VmOperator::ALL` (update const array size and `TOTAL_WEIGHT`). Add corresponding entries to `MutationOperator` enum and all exhaustive match arms (`as_key`, `domain`, `semantic_category`, `complexity_effect`). Wire into engine dispatch. TDD: test each motif produces expected instruction patterns; test paired address mutation co-mutates both opcodes.

- [ ] Review Gate: Interim code review — review Steps 6-10 changes. Fix findings, re-review until clean.

### Phase 5: Wire Format & Frontend

- [ ] Step 11: Update v3-server creature handler. Replace `memory` byte array serialization with `shared_memory` f32 array. Update `exclude` field to accept `shared_memory` instead of `memory`. Update server tests.

- [ ] Step 12: Replace `MemoryHexView` with `SharedMemoryView`. Display 16 f32 slots in a compact grid: slot index, current value (formatted to 3 decimal places), non-zero highlighting. Update `CreatureInspector.tsx`, TypeScript types (`genome.ts` or relevant type file), and `useCreatureDetail.ts` to use `shared_memory` field.

### Phase 6: Specs & Verification

- [ ] Step 13: Update reference specs. `v3-vm-isa-spec.md`: rewrite Section 8 (Creature Memory Contract) for shared slots, update opcode table in Section 2 and cost table in Section 6, update opcode count. `v3-graph-backend-spec.md`: add slot node kinds to data model and evaluation sections. `v3-mutation-spec.md`: add motif operators to VM domain operator list.

- [ ] Step 14: Full verification. Run viability tests (`cargo test -p v3-core --test viability`). Run `cargo test --workspace`. Run `cargo clippy --workspace --all-targets -- -D warnings`. Run `cargo fmt --all -- --check`. Run `npm run build` in frontend.

- [ ] Review Gate: Code review — dispatch `superpowers:code-reviewer` subagent on full branch diff. Invoke domain skills (backend: `rust-skills`; frontend: `vercel-react-best-practices` + `vercel-composition-patterns`). Fix all findings. Re-review until clean pass.
- [ ] Review Gate: Architecture & decomposition review — review all changes for boundary violations, decomposition opportunities, separation of concerns. Re-read `docs/strategy/` and relevant `AGENTS.md` files. Fix easy issues, capture larger items in `docs/features/brainstorms/ideas.md`. Repeat until clean pass.
- [ ] Completion gate — run all checks from AGENTS.md Completion Gate section

**Review cycles:** 11
