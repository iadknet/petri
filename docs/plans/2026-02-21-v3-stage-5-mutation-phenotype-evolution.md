# V3 Stage 5: Mutation + Phenotype + Evolution Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` for implementation work and
> keep status checkmarks synchronized between this subplan and the master plan.

**Goal:** Replace the Stage 4 mutation engine stub with real genome mutation operators, wire
phenotype mutation into the reproduction flow, and add phenotype configuration — enabling genuine
evolutionary variation in offspring.

**Goal IDs:** GP-01, GP-02, GP-03

**Scope:** `mutation/` module in `v3/crates/v3-core/src/`; extend `config/simulation.rs`;
update `simulation/actions.rs`; new integration tests.

**Docs Impact:** `docs/plans/2026-02-21-v3-stage-5-mutation-phenotype-evolution.md` (this plan
created); `docs/plans/2026-02-21-v3-implementation-master-plan.md` Stage 5 row updated on
completion; archived evidence file removed.

**Supersedes:** none

**Superseded-By:** none

**Parent plan:** `docs/plans/2026-02-21-v3-implementation-master-plan.md`

---

## Context

Stages 1–4 built a fully runnable simulation with creatures that eat, move, reproduce, and survive
multiple ticks. The mutation engine is currently a stub that always returns a zero-event summary.
Stage 5 replaces the stub with real genome mutation operators, wires phenotype mutation into the
reproduction flow, and adds phenotype configuration fields. The result is a simulation capable of
producing genuine evolutionary variation in offspring.

---

## Goal Alignment

- **GP-01:** Richer creature decision-making becomes possible when offspring can carry mutated
  genomes that diverge from the founder template.
- **GP-02:** All new code lives inside the existing `mutation/` module; no cross-boundary leaks;
  dependency direction unchanged.
- **GP-03:** Every operator is unit-tested; E2E integration tests verify that offspring diverge
  under realistic mutation rates.

---

## Boundary Impact

- `config/simulation.rs`: `MutationConfig` gains a nested `PhenotypeConfig` sub-struct (4 new
  fields). `normalize()` extended. Existing serde/default tests updated.
- `mutation/engine.rs`: Stub replaced with real implementation. Imports new sub-modules.
- `mutation/topology.rs` (NEW): Topology domain mutators.
- `mutation/vm_mutator.rs` (NEW): VM domain mutators.
- `mutation/graph_mutator.rs` (NEW): Graph domain mutators.
- `mutation/phenotype.rs` (NEW): Phenotype mutation algorithm.
- `mutation/mod.rs`: Export new sub-modules.
- `simulation/actions.rs`: Steps 9–10 in `apply_reproduce` updated to use real summary and trigger
  phenotype mutation.
- No changes to `creature/`, `kernel/`, `sensors/`, `runtime/`, or `contracts/`.

---

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `mutation/engine.rs` | change | Replace stub; imports topology/vm/graph sub-modules |
| `mutation/types.rs` | keep | MutationSummary and MutationSkipReason already complete |
| `config/simulation.rs` | change | Add PhenotypeConfig, normalize phenotype fields |
| `creature/parseability.rs` | keep | ParseabilityGate already validates empty/duplicate-id |
| `simulation/actions.rs` | change | Wire phenotype trigger after real MutationEngine |
| `creature/genome.rs` | keep | All mutation targets already defined; no struct changes |
| `lib.rs` | keep | `pub mod mutation` already exported |

---

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Domain selection weighting | Equal probability for all 3 domains (1/3 each). Sub-operator selection also equal within domain. No config knobs for domain weights in v3alpha1. | agent | resolved |
| New node ID allocation in AddNode | `max(existing node_ids) + 1` (wrapping u32 arithmetic); genome is small so O(n) is fine | agent | resolved |
| Which operators to implement per domain | Topology: AddNode, RemoveNode, RetargetNodeTarget, AddRouteTarget, RemoveRouteTarget, ChangeEntryNode. VM: VmConstantMutation, VmInstructionMutation. Graph: AlterGraphEdgeWeight, SwapGraphOperator, MutateGraphOperatorParam, AddInternalGraphNode, RemoveInternalGraphNode. | agent | resolved |
| How to pick a random operator kind from VmInstruction enum | Use Noop as the safe replacement for instruction mutation (always parseable) | agent | resolved |
| PhenotypeConfig nesting | `MutationConfig.phenotype: PhenotypeConfig` — matches `runtime.mutation.phenotype.*` key path in spec | agent | resolved |

---

## Tasks

- [x] **Task 1: Add `PhenotypeConfig` to `MutationConfig` and extend `normalize()`**
  - Added `PhenotypeConfig` struct to `config/simulation.rs` with 4 fields per spec
  - Added `pub phenotype: PhenotypeConfig` to `MutationConfig`
  - Updated `MutationConfig::default()` with spec defaults
  - Extended `normalize()`: channel_step 0→2, clamp polarity_flip_chance, validate weight min/max
  - Added 4 `normalize_phenotype_*` tests + updated `default_config_matches_spec`
  - Re-exported `PhenotypeConfig` from `config/mod.rs`

- [x] **Task 2: Implement `mutation/phenotype.rs` and wire phenotype trigger in `actions.rs`**
  - Created `mutation/phenotype.rs` with `mutate_phenotype()` implementing spec Section 5 5-step algorithm
  - Weight sanitization + fallback to uniform channel selection when all weights ≤ ε
  - Exported from `mutation/mod.rs`
  - Updated `apply_reproduce` in `simulation/actions.rs`: real phenotype trigger when `summary.applied_events > 0`
  - 5 unit tests covering all algorithm branches

- [x] **Task 3: Implement `mutation/topology.rs` — TopologyMutator**
  - Created `TopologyOperator` enum (6 variants) and `TopologyMutator::apply()`
  - All 6 operators with proper pre-guards returning `NoApplicableTarget`
  - 7 unit tests including parseability gate validation after each operator

- [x] **Task 4: Implement `mutation/vm_mutator.rs` — VmMutator**
  - Created `VmOperator` enum (2 variants) and `VmMutator::apply()`
  - Pre-guard: must have at least one VM-backend node
  - VmConstantMutation: perturb or add constant; VmInstructionMutation: insert/replace/delete (never empties program)
  - 6 unit tests including parseability gate and graph-only-genome guard

- [x] **Task 5: Implement `mutation/graph_mutator.rs` — GraphMutator**
  - Created `GraphOperator` enum (5 variants) and `GraphMutator::apply()`
  - Pre-guard: must have at least one Graph-backend node
  - All 5 operators implemented with inner pre-guards where needed
  - 7 unit tests including parseability gate and vm-only-genome guard

- [x] **Task 6: Implement real `MutationEngine` replacing the stub**
  - Rewrote `mutation/engine.rs` with full probability gate, event count, domain selection
  - Each domain event: clone snapshot, apply operator, validate parseability, restore on failure
  - Replaced stub tests with 5 real tests (accounting invariant, probability gates, parseability preservation, 1000-round no-panic)

- [x] **Task 7: E2E integration test for mutation-driven phenotype divergence**
  - Added 4 new tests to `tests/viability.rs`:
    - `mutation_offspring_diverge_from_parent_over_time`: phenotype diverges within 30 ticks at 100% mutation rate
    - `mutation_accounting_invariant_in_viability`: 1000 calls to MutationEngine, all pass accounting invariant
    - `phenotype_inherits_unchanged_when_no_genome_mutation`: child inherits parent phenotype when mutation_probability=0.0
    - `viability_still_passes_with_real_mutations`: population survives + reproduces with real mutations

- [x] **Task 8: Quality gates, subplan file, master plan + evidence matrix**
  - cargo test --workspace: 238 unit + 14 integration = 252 tests pass; 4 pre-existing failures unchanged
  - cargo clippy --workspace --all-targets -- -D warnings: clean
  - cargo fmt --all -- --check: clean
  - scripts/check-plan-harness.sh --mode strict: violations=0, warnings=0
  - scripts/check-doc-harness.sh --mode warn: violations=2 (pre-existing CLAUDE.md warnings, not Stage 5)
  - scripts/check-architecture-harness.sh --mode warn: violations=0 (5 pre-existing petri-core warnings)
  - Subplan written; evidence matrix created; master plan Stage 5 updated

---

## Key Reference Files

- `v3/crates/v3-core/src/mutation/engine.rs` — real engine replacing stub
- `v3/crates/v3-core/src/mutation/types.rs` — MutationSummary, MutationSkipReason (unchanged)
- `v3/crates/v3-core/src/mutation/topology.rs` — NEW: TopologyMutator
- `v3/crates/v3-core/src/mutation/vm_mutator.rs` — NEW: VmMutator
- `v3/crates/v3-core/src/mutation/graph_mutator.rs` — NEW: GraphMutator
- `v3/crates/v3-core/src/mutation/phenotype.rs` — NEW: mutate_phenotype()
- `v3/crates/v3-core/src/config/simulation.rs` — PhenotypeConfig added
- `v3/crates/v3-core/src/simulation/actions.rs` — phenotype trigger wired
- `v3/crates/v3-core/tests/viability.rs` — 4 new E2E mutation tests

---

**Review cycles:** 1
