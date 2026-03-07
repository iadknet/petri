# Reachability-Aware Mutation Pathways: Implementation Plan

**Parent refinement:** `refinement.md`

**Goal:** Add probabilistic reachability bias to mutation target selection so that mutation operators preferentially target reachable (functional) mesh nodes while preserving drift on unreachable structure.

**Goal IDs:** GP-01, GP-02, GP-03, GP-04
**Scope:**
- In: `ReachableBiasConfig` per-domain config, `cached_reachable_nodes` on `CreatureState`, biased node selection in VM/Graph/InputRef/Topology domain mutators, reachability telemetry on `MutationSummary`, reproduction/seeding wiring, stats aggregation, reference spec updates
- Out: New mutation operators (SplitLiveEdge, ActivateDormantSlice, etc.), two-lane mutation system, intra-node dead code biasing, frontend UI changes

**Docs Impact:**
- `docs/reference/v3-mutation-spec.md` — document reachability bias in engine contract and event pipeline
- `docs/reference/v3-runtime-config-spec.md` — add `ReachableBiasConfig` fields
- `docs/reference/v3-evolution-observability-spec.md` — add reachability telemetry fields

**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- **GP-01** (Evolve Richer Decision-Making): Biasing mutations toward reachable structure increases the rate at which active mesh improves, accelerating the evolution of richer cognition without reducing exploratory drift.
- **GP-02** (Maintain Clean Architecture Boundaries): New `reachability.rs` module sits within `mutation/` following existing domain boundaries. No cross-crate API changes. Engine→domain mutator orchestration boundary preserved.
- **GP-03** (Keep Iteration High-Confidence): TDD approach throughout. Bias=0.0 reproduces current uniform behavior exactly — backwards-compatible default allows incremental rollout. Viability tests validate economic stability.
- **GP-04** (Keep Behavior Observable): Reachability telemetry on `MutationSummary` lets operators see whether evolution is investing in live structure vs junk DNA. Cached reachable set enables future observability features.

## Boundary Impact

| Area | Change | Impact |
|------|--------|--------|
| `config/simulation.rs` | Add `ReachableBiasConfig` struct + field on `MutationConfig` | Config extension, serde change |
| `creature/state.rs` | Add `cached_reachable_nodes: Box<[usize]>` field | State extension, computed at birth |
| `mutation/types.rs` | Add `TargetReachability` enum, 3 telemetry fields on `MutationSummary` | Type extension |
| `mutation/reachability.rs` | New module: `biased_select_from()`, `classify_target()` | New internal module |
| `mutation/engine/mod.rs` | `apply_mutations` gains `parent_reachable_nodes: &[usize]` param; wrapper functions thread reachability | Signature change on public API |
| `mutation/vm/mod.rs` | `VmMutator::apply` gains reachability params, uses `biased_select_from` | Internal change |
| `mutation/graph/mod.rs` | `GraphMutator::apply` gains reachability params, uses `biased_select_from` | Internal change |
| `mutation/input_ref/mod.rs` | `InputRefMutator::apply` gains reachability params, uses `biased_select_from` | Internal change |
| `mutation/topology/mod.rs` | `TopologyMutator::apply` and per-operator functions gain reachability params | Internal change |
| `simulation/actions/reproduction.rs` | Pass `parent.cached_reachable_nodes` to `apply_mutations`, extend no-mutation fast path | Call site update |
| `simulation/seeding.rs` | `CreatureState::new()` now computes reachable set | Implicit (no code change needed) |
| `simulation/stats.rs` | Add reachability telemetry aggregation fields | Stats extension |

## Existing Boundary Recheck

| Area | Decision | Rationale |
|------|----------|-----------|
| `mutation/engine` orchestrates, domain mutators apply | keep | Reachability biasing is threaded through the existing orchestration→domain boundary. The engine passes the reachable set; domain mutators use it for target selection. No boundary change. |
| `creature/state.rs` owns cached genome-derived state | keep | `cached_reachable_nodes` follows the same pattern as `cached_complexity` — computed at birth, stored on state, genome-derived and immutable. |
| `creature/genome/analysis.rs` owns reachability analysis | keep | `mesh_reachable_nodes()` remains the authoritative BFS implementation. Mutation module consumes its output but does not duplicate the algorithm. |
| `mutation/types.rs` owns telemetry types | keep | `TargetReachability` and summary fields are mutation-domain concerns. |
| `config/simulation.rs` owns all config | keep | `ReachableBiasConfig` is a sub-struct of `MutationConfig`, following existing patterns. |

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Default `reachable_bias` values per domain? | 0.7 for topology/VM/graph, 0.5 for input_ref. Conservative starting point — viability tests validate. Can be tuned post-merge. | agent | resolved |
| Should "failure" branch select from all nodes or unreachable-only? | All eligible nodes (within the domain's backend filter) — preserves uniform behavior at bias=0.0. The eligible set IS the domain-filtered set, not all genome nodes. | user | resolved |
| Should parent's reachable set or offspring pre-mutation BFS be used? | Parent's cached set — available without extra BFS, valid since offspring starts as parent copy. | user | resolved |
| `ChangeEntryNode` interaction with reachability? | Exempt — it redefines reachability, so biasing toward "currently reachable" is meaningless. | user | resolved |
| `reachable_bias = 0.0` reproduces current behavior? | Yes — 0.0 means always select from all nodes uniformly. | - | resolved |
| Performance of `biased_select_from`? | Both `eligible` and `reachable` are sorted; use two-pointer merge for O(eligible + reachable) intersection. Acceptable — mutation is not a hot path (once per birth, up to 10 events). | agent | resolved |
| `Vec<usize>` vs `Box<[usize]>` for cached reachable set? | Use `Box<[usize]>` — immutable after creation, saves 8 bytes per creature (no capacity word), communicates immutability. Convert from `Vec` via `.into_boxed_slice()`. | agent | resolved |
| NaN handling in ReachableBiasConfig normalization? | Clamp with NaN fallback to 0.0 (matching existing `normalize_f32_clamp` pattern). `rng.gen_bool(NaN)` panics, so NaN must be rejected at normalization. | agent | resolved |
| `record_reachability` for skipped events? | Only record reachability for applied events. Skipped events (where no target was selected or parseability failed) don't record reachability. Reachability counters sum equals `applied_events`. | agent | resolved |
| Growing `CreatureState::new` parameter list? | Acknowledged. The `too_many_arguments` allow already exists. Defer builder/struct refactor — it would affect many call sites for no functional benefit. | agent | resolved |
| `#[non_exhaustive]` on `TargetReachability`? | Not needed — `TargetReachability` is crate-internal (`v3-core`), not exposed across crate boundaries. Adding variants is safe without it. | agent | resolved |
| Domain mutator `apply` methods are `pub` — signature change impact? | These `pub` methods are only called from within `v3-core` (engine wrappers and tests). All direct-call test sites need updating — pass `(&[], 0.0)` for backwards-compatible behavior. | agent | resolved |
| Serde derives on `TargetReachability`? | Omit for now — it is used only as a dispatch enum for `record_reachability()`, not as a HashMap key or serialized value. Add if needed later. | agent | resolved |
| RemoveNode biasing toward reachable — counterintuitive? | Intentional: biasing ALL mutation types (constructive and destructive) toward live structure increases evolutionary selection pressure. Good mutations on live code improve fitness; bad mutations (including removal) on live code reduce fitness and are selected against. This creates stronger evolutionary signal. Bias is soft (0.7, not 1.0) so unreachable pruning still occurs. | agent | resolved |
| `Copy` derive on `ReachableBiasConfig`? | Drop `Copy` — existing config structs use `Clone` only. The only `Copy` type in `config/simulation.rs` is the `ToroidalMode` enum. Follow convention. | agent | resolved |
| Pre-existing NaN gap in `mutation_probability`/`mesh_layer_probability` normalization? | Out of scope — existing f64 fields use bare `.clamp()` without NaN guard. The new `ReachableBiasConfig` uses the correct pattern. Fixing the pre-existing gap is a separate bug fix (capture in ideas.md if desired). | agent | resolved |

## Design Details

### Biased selection algorithm

```text
biased_select_from(eligible: &[usize], reachable: &[usize], bias: f64, rng) -> Option<(usize, TargetReachability)>

Precondition: both `eligible` and `reachable` are sorted ascending.

1. If eligible is empty → None
2. Roll rng against bias probability (clamped to [0.0, 1.0]):
   a. On success: compute intersection count via two-pointer merge (O(eligible + reachable))
      - If count > 0: pick random offset in [0, count), iterate two-pointer again to find
        the k-th intersection element → (idx, Reachable)
      - If count == 0: fall through to step 3
   b. On failure: fall through to step 3
3. Pick uniformly from eligible
4. Classify picked index: binary search in reachable → Reachable or Unreachable
5. Return (idx, classification)

Note on domain-specific filtering: `biased_select_from` returns the reachability
classification of the selected node. If a domain operator applies additional filtering
after selection (e.g., RemoveNode cannot remove the entry node), the caller should use
the classification from `biased_select_from` for the node that was actually selected,
not re-classify. If the selected node is rejected by domain filtering and a different
node is chosen via fallback, call `classify_target` on the final node instead.

Note: The two-pass approach (count then select) avoids allocating a temporary
collection for the intersection. Eligible sets are typically small (< 50 nodes).

Important: The parent's cached reachable set is used to bias mutations on the
offspring. After mutations, the offspring's actual reachable set may differ (e.g.,
AddRouteTarget can make unreachable nodes reachable). This is intentional — mutations
are biased toward what was functional in the parent. The offspring gets a fresh BFS
via CreatureState::new(). Document this in the module-level doc comment.
```

### Topology operator biasing matrix

| Operator | Biasing | Rationale |
|----------|---------|-----------|
| AddNode | exempt (NotApplicable) | Creates new disconnected node |
| ChangeEntryNode | exempt (NotApplicable) | Redefines reachability |
| RemoveNode | bias removable set | Bias toward reachable — increases evolutionary selection pressure (removal of functional structure reduces fitness, driving selection against harmful mutations) |
| RetargetNodeTarget | bias eligible set | Prefer retargeting reachable nodes |
| AddRouteTarget | bias all nodes | Prefer adding routes from reachable nodes |
| RemoveRouteTarget | bias eligible set | Prefer removing targets from reachable nodes |
| SwapNodeBackend | bias all nodes | Prefer swapping reachable backends |
| RewriteNodeId | bias all nodes | Prefer rewriting reachable IDs |
| CopyNode | bias source selection | Prefer copying functional structure |
| CopyMeshBackwardSlice | bias anchor selection | Prefer anchoring from reachable nodes |
| CopyMeshForwardSlice | bias seed selection | Prefer seeding from reachable nodes |
| SpliceNode | bias eligible set | Prefer splicing into reachable paths |
| SwapRouteTargets | bias eligible set | Prefer swapping within reachable nodes |

### Cross-boundary classification

A mutation event is classified as `CrossBoundary` when it structurally connects reachable and unreachable regions. Specifically:
- `RetargetNodeTarget` where the source node is reachable and the new target resolves to an unreachable node (or vice versa)
- `AddRouteTarget` where the source is reachable and the new target is unreachable
- `SpliceNode` where it inserts between reachable and unreachable nodes

For the initial implementation, `CrossBoundary` detection is deferred — all events are classified as `Reachable`, `Unreachable`, or `NotApplicable` based solely on the primary target node's reachability status. Cross-boundary analysis can be added as a follow-up refinement without changing the telemetry schema.

## Required Skills

- Rust/backend changes: invoke `rust-skills` BEFORE writing any Rust code and before each review

## TDD Policy

For all behavior changes and bug fixes: write a failing test FIRST, then implement.
A step is not complete until:
1. The failing test exists and is committed
2. The implementation makes it pass
3. No existing tests regress

## Code Review Policy

After completing each implementation step:
1. Run a thorough code review (backend: `rust-skills`)
2. Fix ALL findings
3. Run review AGAIN — repeat until no new findings (clean recursive pass)
4. Only after clean pass: commit the step

## Commit Policy

- Commits happen AFTER a clean code review pass, never before
- One commit per implementation step (focused, atomic)
- Do NOT advance to the next step until current step is committed and reviewed clean

## Call Site Audit

### `MutationEngine::apply_mutations` call sites

| File | Line | Context | Migration |
|------|------|---------|-----------|
| `simulation/actions/reproduction.rs` | 188 | Offspring mutation | Pass `&sim.creatures[parent_id].cached_reachable_nodes` |
| `mutation/engine/mod.rs` tests | various | Test helpers | Pass `&[]` (empty — no biasing in existing tests, bias=0.0 default) |

### Domain mutator `apply` call sites

| File | Caller | Migration |
|------|--------|-----------|
| `mutation/engine/mod.rs` | `apply_topology_event` | Pass `(reachable_nodes, config.reachable_bias.topology)` |
| `mutation/engine/mod.rs` | `apply_vm_event` | Pass `(reachable_nodes, config.reachable_bias.vm)` |
| `mutation/engine/mod.rs` | `apply_graph_event` | Pass `(reachable_nodes, config.reachable_bias.graph)` |
| `mutation/engine/mod.rs` | `apply_input_ref_event` | Pass `(reachable_nodes, config.reachable_bias.input_ref)` |

### `CreatureState::new` / `new_with_cached_complexity` call sites

| File | Line | Migration |
|------|------|-----------|
| `simulation/seeding.rs` | 67 | `new()` auto-computes `cached_reachable_nodes` — no change needed |
| `simulation/actions/reproduction.rs` | 260 | Rename to `new_with_cached_fields`, pass parent's `cached_reachable_nodes` (`Box<[usize]>` clone) |
| `simulation/actions/reproduction.rs` | 273 | `new()` auto-computes — no change needed |
| `creature/state.rs` tests | various | `new()` auto-computes — no code change, just verify field is populated |
| Various test fixtures | various | `new()` auto-computes — compile-time compatible |

## Implementation Steps

### Step 1: Add `ReachableBiasConfig` and telemetry types

**Files:** `v3/crates/v3-core/src/config/simulation.rs`, `v3/crates/v3-core/src/mutation/types.rs`, `v3/crates/v3-core/src/mutation/mod.rs`

1. Add `ReachableBiasConfig` struct to `config/simulation.rs`:
   - Fields: `topology: f64` (default 0.7), `vm: f64` (default 0.7), `graph: f64` (default 0.7), `input_ref: f64` (default 0.5)
   - Derive `Debug, Clone, Serialize, Deserialize` + `#[serde(deny_unknown_fields)]` (no `Copy` — follows existing config struct convention)
   - Add `pub reachable_bias: ReachableBiasConfig` field on `MutationConfig` with `#[serde(default)]`
   - Add normalization in `SimulationConfig::normalize()`: clamp each `f64` field to `[0.0, 1.0]` with NaN→0.0 fallback via inline `if v.is_finite() { v.clamp(0.0, 1.0) } else { 0.0 }` (existing normalization helpers are `f32`-only; this improves on existing `f64` normalization which lacks NaN guard)
2. Add `TargetReachability` enum to `mutation/types.rs`:
   - Variants: `Reachable`, `Unreachable`, `NotApplicable`
   - Derive `Debug, Clone, Copy, PartialEq, Eq, Hash`
3. Add 3 telemetry fields to `MutationSummary`:
   - `reachable_target_events: u32`
   - `unreachable_target_events: u32`
   - `not_applicable_events: u32`
4. Add `record_reachability(&mut self, target: TargetReachability)` method
5. Update `MutationSummary::zero()` to initialize new fields
6. Add re-export for `TargetReachability` in `mutation/mod.rs`
7. TDD: test config defaults, test serde round-trip, test normalization clamping (including NaN→0.0), test `MutationSummary::zero()` includes new fields, test `record_reachability` increments correct counters, test accounting invariant (`reachable + unreachable + not_applicable == applied_events`)

- [ ] Step 1: Add `ReachableBiasConfig` and telemetry types

### Step 2: Add `cached_reachable_nodes` to `CreatureState`

**Files:** `v3/crates/v3-core/src/creature/state.rs`

1. Add `pub cached_reachable_nodes: Box<[usize]>` field to `CreatureState`
2. In `CreatureState::new()`: compute `mesh_reachable_nodes(&genome).into_boxed_slice()` and store as `cached_reachable_nodes`
3. Rename `new_with_cached_complexity` to `new_with_cached_fields`, add `cached_reachable_nodes: Box<[usize]>` parameter
4. Update the single call site of `new_with_cached_complexity` in `reproduction.rs` to use new name and pass `parent.cached_reachable_nodes.clone()`
5. TDD: test that `new()` populates `cached_reachable_nodes` correctly for a genome with reachable and unreachable nodes; test that `new_with_cached_fields` preserves provided values

- [ ] Step 2: Add `cached_reachable_nodes` to `CreatureState`

### Step 3: Create reachability module with biased selection helper

**Files:** `v3/crates/v3-core/src/mutation/reachability.rs`, `v3/crates/v3-core/src/mutation/mod.rs`

1. Create `mutation/reachability.rs` with:
   - `#[must_use] pub fn biased_select_from(eligible: &[usize], reachable: &[usize], bias: f64, rng: &mut impl Rng) -> Option<(usize, TargetReachability)>` — implements the biased selection algorithm from Design Details
   - `#[must_use] pub fn classify_target(node_idx: usize, reachable: &[usize]) -> TargetReachability` — binary search classification
2. Add `pub mod reachability;` to `mutation/mod.rs`
3. TDD:
   - `bias=0.0` → uniform selection from eligible (no reachability preference)
   - `bias=1.0` with reachable eligible nodes → always selects reachable
   - `bias=1.0` with no reachable eligible nodes → falls back to uniform
   - Empty eligible → returns None
   - Classification: reachable index returns `Reachable`, unreachable returns `Unreachable`
   - Statistical test: `bias=0.7` with seeded RNG over N>=5000 iterations, verify reachable selection rate falls within [0.60, 0.80] when both reachable and unreachable nodes are eligible

- [ ] Step 3: Create reachability module with biased selection helper

### Step 4: Thread reachability through engine and bias VM/Graph/InputRef

**Files:** `v3/crates/v3-core/src/mutation/engine/mod.rs`, `v3/crates/v3-core/src/mutation/vm/mod.rs`, `v3/crates/v3-core/src/mutation/graph/mod.rs`, `v3/crates/v3-core/src/mutation/input_ref/mod.rs`

1. Change `MutationEngine::apply_mutations` signature: add `parent_reachable_nodes: &[usize]` parameter after `config`
2. Change `apply_vm_event`, `apply_graph_event`, `apply_input_ref_event` wrapper signatures: add `reachable_nodes: &[usize]` and `bias: f64`. Change return type from `Result<(), MutationSkipReason>` to `Result<TargetReachability, MutationSkipReason>`
3. Change `apply_topology_event` wrapper: same signature changes
4. In the engine event loop: pass `parent_reachable_nodes` and per-domain bias from `config.reachable_bias` to each wrapper. On `Ok(reachability)` from the wrapper (after parseability gate passes), record via `summary.record_reachability(reachability)`. Note: the wrapper snapshot/parseability gate may discard the domain mutator's `TargetReachability` on rollback — only record for events that fully apply.
5. Change `VmMutator::apply` signature: add `reachable_nodes: &[usize]`, `bias: f64`. Return `Result<TargetReachability, MutationSkipReason>`. Replace the current `vm_indices` collection + uniform pick with `biased_select_from(&vm_indices, reachable_nodes, bias, rng)`
6. Change `GraphMutator::apply` signature: same changes, same replacement
7. Change `InputRefMutator::apply` signature: add `reachable_nodes: &[usize]`, `bias: f64`. Return `Result<TargetReachability, MutationSkipReason>`. Unlike VM/Graph, InputRef does NOT select a node at the top of `apply` — each sub-operator (`apply_add`, `apply_remove`, `apply_swap`, `apply_raw_field_mutation`) selects nodes independently. Thread `reachable_nodes` and `bias` to each sub-operator and apply `biased_select_from` at each node selection point: `apply_add` biases `rng.gen_range(0..genome.nodes.len())` using all indices as eligible; `apply_remove` and `apply_swap` bias the `eligible` set of non-empty-input-refs nodes; `apply_raw_field_mutation` does a global cross-node scan — exempt from biasing, return `NotApplicable`. Since `InputRefMutator::apply` dispatches to exactly one sub-operator per call (via `match op`), each sub-operator's classification propagates directly as the return value — no aggregation needed
8. Update all engine tests: pass `&[]` as `parent_reachable_nodes` to preserve existing behavior (bias on empty reachable set is always uniform fallback)
9. TDD: test that `apply_mutations` with `&[]` reachable set matches previous behavior; test that with `bias=1.0` and a genome with mixed reachable/unreachable VM nodes, VM mutations only target reachable nodes

- [ ] Step 4: Thread reachability through engine and bias VM/Graph/InputRef

- [ ] Review Gate: Interim code review — review Steps 1-4 changes. Fix findings, re-review until clean.

### Step 5: Bias topology operators

**Files:** `v3/crates/v3-core/src/mutation/topology/mod.rs`

1. Change `TopologyMutator::apply` signature: add `reachable_nodes: &[usize]`, `bias: f64`. Return `Result<TargetReachability, MutationSkipReason>`
2. Only add `reachable_nodes: &[usize]` and `bias: f64` to per-operator functions that actually use biased selection (11 of 13). Exempt operators (`AddNode`, `ChangeEntryNode`) keep their current signatures — `TopologyMutator::apply` returns `Ok(NotApplicable)` directly for these without threading params.
3. Per-operator biasing details (each operator's specific selection point):
   - `RemoveNode`: bias the `removable` set (non-entry nodes) — replace `removable[rng.gen_range(...)]`
   - `RetargetNodeTarget`: bias nodes with non-empty targets — replace `eligible[rng.gen_range(...)]`
   - `AddRouteTarget`: bias `node_idx` selection from all nodes (first selection point); `target_id` is a random `NodeId`, not a node index — leave unbiased
   - `RemoveRouteTarget`: bias nodes with non-empty targets
   - `SwapNodeBackend`: bias all nodes
   - `RewriteNodeId`: bias all nodes
   - `CopyNode`: bias source selection from all nodes
   - `SpliceNode`: bias nodes with non-empty targets
   - `SwapRouteTargets`: bias nodes with ≥2 targets
   - `CopyMeshBackwardSlice`: bias anchor selection from all node indices via `biased_select_from` + `mesh_backward_slice(genome, anchor_idx, max_size)` (both functions are already `pub`)
   - `CopyMeshForwardSlice`: same pattern with `mesh_forward_slice` + biased seed selection
4. For exempt operators (`AddNode`, `ChangeEntryNode`): `TopologyMutator::apply` returns `Ok(TargetReachability::NotApplicable)` without calling the operator with reachability params
7. Update topology tests: pass `(&[], 0.0)` to preserve existing behavior
8. TDD: test that with `bias=1.0` and a genome with reachable and unreachable nodes, operators like `RemoveNode` and `CopyNode` prefer reachable targets; test exempt operators return `NotApplicable`

- [ ] Step 5: Bias topology operators

### Step 6: Wire reproduction, seeding, and stats aggregation

**Files:** `v3/crates/v3-core/src/simulation/actions/reproduction.rs`, `v3/crates/v3-core/src/simulation/stats.rs`

1. In `apply_reproduce`: clone parent's `cached_reachable_nodes` (`Box<[usize]>`) before calling `apply_mutations`, pass as `&parent_reachable_nodes` (deref coerces `Box<[usize]>` to `&[usize]`)
2. In the no-mutation fast path: pass the cloned `Box<[usize]>` directly to `new_with_cached_fields` (no extra allocation since we already cloned it in step 1)
3. Seeding: `CreatureState::new()` already computes `cached_reachable_nodes` — verify it works for founder genomes
4. Add reachability telemetry aggregation to `SimStats`:
   - `mutation_reachable_target_total: u64`
   - `mutation_unreachable_target_total: u64`
   - `mutation_not_applicable_target_total: u64`
5. In reproduction path: aggregate reachability telemetry from `MutationSummary` into `SimStats`
6. TDD: test that reproduction passes parent's reachable set; test that founder creatures have correct `cached_reachable_nodes`; test stats aggregation
7. Run viability tests: `cargo test -p v3-core --test viability`

- [ ] Step 6: Wire reproduction, seeding, and stats aggregation

### Step 7: Update reference specs

**Files:** `docs/reference/v3-mutation-spec.md`, `docs/reference/v3-runtime-config-spec.md`, `docs/reference/v3-evolution-observability-spec.md`

1. `v3-mutation-spec.md`:
   - Section 4.1: update engine contract signature to include `parent_reachable_nodes`; update `MutationSummary` minimum fields list to include `reachable_target_events`, `unreachable_target_events`, `not_applicable_events`
   - Section 4.2: add reachability bias step between domain selection and operator application
   - Add new Section 4.3 or subsection: "Reachability Bias" explaining the biased selection algorithm, per-domain config, exempt operators, and `TargetReachability` classification
2. `v3-runtime-config-spec.md`: add `ReachableBiasConfig` fields to the `MutationConfig` table
3. `v3-evolution-observability-spec.md`: add `reachable_target_events`, `unreachable_target_events`, `not_applicable_events` to the minimum telemetry fields

- [ ] Step 7: Update reference specs

### Step 8: Full verification

1. `cargo test -p v3-core --test viability` — merge gate
2. `cd v3 && cargo test --workspace`
3. `cd v3 && cargo clippy --workspace --all-targets -- -D warnings`
4. `cd v3 && cargo fmt --all -- --check`
5. `scripts/check-doc-harness.sh --mode strict`
6. `scripts/check-architecture-harness.sh --mode strict`
7. `scripts/check-plan-harness.sh --mode strict`

- [ ] Step 8: Full verification

- [ ] Review Gate: Code review — dispatch `superpowers:code-reviewer` subagent on full branch diff. Invoke domain skills (backend: `rust-skills`). Fix all findings. Re-review until clean pass.
- [ ] Review Gate: Architecture & decomposition review — review all changes for boundary violations, decomposition opportunities, separation of concerns. Re-read `docs/strategy/` and relevant `AGENTS.md` files. Fix easy issues, capture larger items in `docs/features/brainstorms/ideas.md`. Repeat until clean pass.
- [ ] Completion gate — run all checks from AGENTS.md Completion Gate section

**Review cycles:** 5 (Pass 1: 3 dispatches, Pass 2: 1 dispatch, Pass 3: 1 dispatch)
