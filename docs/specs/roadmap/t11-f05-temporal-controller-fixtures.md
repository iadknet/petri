# T11.F05 — Temporal Controller Fixtures

**Status**: In Progress
**Last updated**: 2026-09-05
**Feature**: T11.F05
**Track**: [T11 — Brain Genotype-Phenotype Map](../../roadmaps/t11-brain-genotype-phenotype-map.md)

## Goal

A maintained fixture suite of small constructed controllers, run through the
production tick path at production runtime settings, records for each
temporal building block what the brain is expected to do and what it observably
does: delayed cues with identical current observations across several delay
lengths, shared-memory retention, stateful-graph and backward-edge clocks under
a disconnected-node perturbation, and reward-trace timing under immediate,
delayed, and skipped-visit rewards. Every measured gap is recorded as a gap
assigned to T11.F06 or T11.F07, never as a pass. No production behavior
changes. T11.F01, T09.F08, T09.F01, and T11.F10 reuse the fixture genomes.

## Non-Goals

- No change to runtime semantics, production defaults, founders, mutation,
  the tick loop, or any reference spec. The graph relaxation clock, the
  eligibility-trace clock, and the reward gain rule are measured here and
  repaired by T11.F06 and T11.F07.
- No new indicator, no goal-profile wiring, no report-schema change; every
  `Undefined` indicator stays `Undefined`. The founder half of the T11.F01
  neighborhood indicator is not on `main` yet, so this feature records no
  indicator reading.
- No evolution, accessibility, or emergence claim (T11.F10); no versioned
  comparator or replay protocol (T09.F01); no state-intervention set
  (T09.F08); no frontend change.
- No repeated-dispatch-within-a-tick fixture: T11.F06 defines that semantics
  and owns its fixture.

## Inputs and Invariants

- Contract: the T11 track's T11.F05, T11.F06, and T11.F07 notes and its
  node-type contract (property 3: every piece of persistent state advances
  once per world tick); audit finding 7 in
  `docs/strategy/brain-evolvability-audit-2026-09-04.md` (stateful operators
  advance per relaxation pass, traces decay only on module execution while
  rewards apply every tick, the learning rate enters both the trace and the
  reward update, outputs are zeroed at dispatch so a backward edge reads the
  previous pass).
- Seams, all existing and public: `Simulation::new`, `run_tick`,
  `ActiveTrace` and `TickTrace` (`static_inputs`, `final_actions`, `hops`),
  `CreatureState` fields `shared_memory`, `prev_shared_memory`, `energy`, and
  `graph_runtime` (`node_state`, `plasticity_weights`, `eligibility_traces`),
  `SimStats` work counters, `execute_creature_mesh`, and the helpers in
  `crates/v3-core/tests/creature_workflow_e2e/support.rs` (`test_config`,
  `insert_creature`, `run_one_traced_tick`). Genome constructors:
  `CgpGraphBackendDef::new_with_fixed_outputs`, `OutputSinkKind::WriteSlot`,
  `GraphSource::SharedMemory { previous }`, `VmInstruction::{ReadInput,
  StoreSlotImm, LoadSlotImm, LoadSlotPrev, JumpIfZero, WriteRouteGate,
  PushAction, ExecuteActionQueue, Halt}`, `WorldInputKey::{FoodHere, AgeTicks}`.
- Production settings held fixed and never edited by a fixture:
  `RuntimeConfig::default()` (`max_graph_relax_iters` 15,
  `graph_convergence_epsilon` 1e-3, `graph_convergence_stable_passes` 2,
  `graph_node_base_cost` 1e-5, `plasticity_update_cost` 0,
  `reward_learning_cost` 0, `max_mesh_hops` 1024) and
  `shared_memory.decay_rate` 0. The only permitted deviations are the
  existing `test_config` precedent (tiny world, zero food coverage and
  growth, zero energy decay, zero mutation probability) and the one labeled
  decay treatment in fixture C2; the file's doc comment records them.
- Observed semantics the fixtures pin, from the code on 2026-09-05:
  `run_phase_0` copies `shared_memory` into `prev_shared_memory` then applies
  decay; `execute_graph_impl` zeroes `prev_outputs`/`curr_outputs` at every
  dispatch, advances `node_state` on every pass, runs at least
  `stable_passes + 1` passes and at most `max_graph_relax_iters`, and updates
  traces once per dispatch after convergence as
  `trace = decay * trace + eta * hebbian(pre, post, w)`;
  `run_reward_learning` applies `dw = eta * signal * trace` every tick to
  every creature with a reward-modulated node whether or not that node ran.
- Fixture rule: a fixture asserts the observed behavior exactly, so the suite
  is green on the current code and T11.F06/T11.F07 flip the assertion when
  they repair it (TDD: the repair's first step is inverting the fixture). The
  expected behavior lives in the fixture's doc comment and in the table below.
  A green fixture is not a capability pass; the table's status column is the
  record. No `#[ignore]`, no waived check.
- Invariants: telemetry derives from applied behavior; production code is
  never edited to kill a mutant; shell automation is POSIX `sh`; deterministic
  gate fields are unchanged; no stored baseline or threshold is edited.

### Fixture catalogue

Each row is one `#[test]` in `crates/v3-core/tests/temporal_fixtures.rs`
(submodules under `crates/v3-core/tests/temporal_fixtures/` are fine), named
`<id>_<slug>`. Each fixture runs whole ticks through `run_tick` on a
one-creature `Simulation` unless noted, reads the creature's state between
ticks, and asserts observed values with an explicit tolerance. "Expected" is
the node-type-contract behavior; "status" is filled in by the implementer.

| ID | Construction | Expected | Observed (to record) | Status |
| --- | --- | --- | --- | --- |
| A1 reactive control | VM: `ReadInput` FoodHere; if > 0 push `Move(E)`, else `NoOp`; `ExecuteActionQueue`. Food toggled on the cell by the test across 4 ticks. | Action tracks the current observation every tick; identical observations give identical actions. | | meets / gap |
| B1 shared-memory delayed cue | VM: on any tick with FoodHere > 0, `StoreSlotImm` slot 0 = 1.0; then `LoadSlotImm` slot 0, if > 0 push `Move(E)` else `NoOp`; never eats. Cue trial: food on the cell at tick 1 only (test removes it before tick 2). No-cue trial: never. Decision tick 1 + d for d in {1, 2, 4, 8, 16}. Assert `static_inputs` at the decision tick equal across trials, cue trial `Move(E)`, no-cue trial `NoOp`. Also assert A1 emits `NoOp` in both trials at the decision tick (matched reactive control). | Acts on the cue at every delay. | | |
| B2 previous-slot one-tick cue | VM: at tick 1 (AgeTicks == 1) `StoreSlotImm` slot 0 = 1.0; at tick 2 `ClearSlot` slot 0 before reading; every tick read slot 0 via `LoadSlotPrev` and push `Move(E)` if > 0 else `NoOp`. Assert the actions at ticks 2 and 3. | Tick 2 sees the tick-1 value (one-tick element); tick 3 sees 0. | | |
| C1 retention | VM stores 0.75 into slot 3 at tick 1 and never writes again. After d ticks for d in {1, 2, 4, 8, 16}: `shared_memory[3]` and, from tick 2, `prev_shared_memory[3]`. | 0.75 exactly at every delay at production decay 0. | | |
| C2 retention under decay (labeled treatment) | C1 with `shared_memory.decay_rate` = 0.1, the one non-production setting. | `0.75 * 0.9^d` within 1e-5 after d ticks. | | |
| C3 graph slot write and previous read | Graph: `Constant(0.6)` wired to `WriteSlot(0)`; a second node `Add` reading `SharedMemory { slot: 0, previous: true }` wired to `WriteSlot(1)`. | Tick 1: slot 0 = 0.6, slot 1 = 0; tick 2: slot 1 = 0.6. | | |
| D1 integrator clock | Graph: node 0 `Constant(1.0)`; node 1 `DecayIntegrator(0.5)` with one edge from node 0, weight 1. Read `node_state[..][1]` and per-tick `graph_relax_iters` after ticks 1..4. | Per-tick clock: 0.5, 0.75, 0.875, 0.9375. | Advances per pass: record the four states and the passes per tick. | |
| D2 disconnected-node perturbation | D1 plus node 2 `Oscillator(0.25)` with no inputs and no consumers. | Identical integrator trajectory and pass count to D1 (contract property 3). | Record trajectory and passes per tick (expected observation: 15 passes every tick and a different trajectory). | |
| D3 backward-edge recurrence | Graph: node 0 `Constant(1.0)`; node 1 `Add` with edges from node 0 and from itself (`ComputeNode(1)`), both weight 1; node 1 wired to `WriteSlot(0)`. Read slot 0 after ticks 1..3. | A one-tick memory element: 1, 2, 3. | Record the three values and passes per tick (expected observation: the same value every tick, set by the pass cap). | |
| E1 exact one-edge update, immediate reward | Graph: node 0 `Constant(1.0)`; node 1 `Add` with one edge from node 0, weight 1, `PlasticityConfig { rule: Classic, learning_rate: 0.5, weight_clamp: 10, modulation: Some { reward_source: <channel>, trace_decay: 0.5 } }`; node 1 wired to the execute gate and an `Eat` action slot so the controller acts every tick. Choose the outcome channel whose signal the test controls exactly (`ActionSuccess` with `Eat` on a food cell gives 1.0; `EnergyDelta` is read from `energy` before and after) and record it. Read `eligibility_traces` and `plasticity_weights` after tick 1. | One calibrated rule with the learning rate applied once: `dw = eta * signal * pre * post`. | Record the trace and the weight delta; state the effective gain (expected observation: `eta^2 * signal * pre * post`, the documented double application). | |
| E2 delayed reward, visited every tick | E1 with the signal held at 0 on ticks 2..d and nonzero at tick d + 1, for d in {1, 2, 4} (control the signal through the world, for example food present only on the reward tick). | Credit at tick d + 1 is discounted by elapsed world time: `eta * signal * decay^d * trace_1` plus the tick's own activity term. | Record whether it matches when the module runs every tick. | |
| E3 skipped module visits | Entry VM node routes to the E1 graph node only on ticks with FoodHere > 0 (`WriteRouteGate` toward the graph target, otherwise `NoOp` + `ExecuteActionQueue`). Visit at tick 1, skip ticks 2..3 with a nonzero signal on a skipped tick (energy moves by VM cost, or food placed for a reactive `Eat` in the VM branch). Read traces and weights after each tick. | Traces reflect elapsed ticks whether or not the module ran; a skipped tick carries no stale credit. | Record trace equality across skipped ticks and the weight movement on a skipped tick (expected observation: trace unchanged, weight moves by `eta * signal * stale trace`). | |

Property test (pure invariant, proptest): for any `a`, `b` in [0, 1], any
finite `state` and `input` in [-1e6, 1e6], one `DecayIntegrator(a)` step and
one `Momentum(b)` step through the public mesh path (or, if the implementer
adds a `pub` re-export of the evaluator for tests, through it) return a value
between `state` and `input` inclusive; assertions must not depend on which
cases are drawn. Commit any `proptest-regressions/` file that appears.

## Implementation Tasks

- [ ] Create `crates/v3-core/tests/temporal_fixtures.rs` with a doc header
      listing the fixture IDs, the settings it holds at production values,
      and the two permitted deviations. Share the small helpers with
      `creature_workflow_e2e` by moving `test_config`, `insert_creature`,
      `run_one_traced_tick`, and the founder constants into
      `crates/v3-core/tests/common/mod.rs` and referencing them from both
      files; do not duplicate them.
- [ ] Implement fixtures A1, B1, B2, C1, C2, C3, D1, D2, D3, E1, E2, E3 and
      the property test, each with a doc comment stating expected and
      observed behavior and the owning repair feature for any gap. Every
      fixture genome is a named constructor function (for example
      `delayed_cue_vm_genome(delay)`) so T11.F01 can lift it as a motif.
- [ ] Add `rust-test-temporal-fixtures` to the `Makefile` (`.PHONY` and the
      `rust-test-all` prerequisites) so `make check` runs the suite.
- [ ] Fill the catalogue's Observed and Status columns with the measured
      values, assign each gap to T11.F06 or T11.F07 in the table and in
      "Notes for AI Agents", and record the T11.F01 motif constructors.
- [ ] Generate the gate report with `make bench PROFILE=gate
      FEATURE=t11-f05-temporal-controller-fixtures`, append it to `closed` in
      `docs/progress/benchmark-series.json` (gate series only; no goal run),
      add the T11.F05 row to `docs/progress.md` in the existing format, and
      complete Performance and Goal Impact.

## Verification

- [ ] `cargo test -p v3-core --test temporal_fixtures` passes in a debug
      build in under 60 seconds; record the test count and time.
- [ ] `cargo test -p v3-core --test creature_workflow_e2e` still passes after
      the helper move; `cargo check --workspace --all-targets` and
      `make rust-clippy` pass.
- [ ] Viability: not run first, because no default, founder, or tick-loop
      mechanic changes; `make check` runs it in order.
- [ ] Fixture catalogue completed with observed values; every gap row names
      T11.F06 or T11.F07; no fixture is ignored or asserts a value it does
      not observe.
- [ ] `make rust-mutants` summary line, output path, and full survivor list,
      each survivor killed, equivalent, or deferred ("nothing to mutate" is a
      valid record for a test-only diff).
- [ ] `make roadmap-check` passes after the document edits.
- [ ] Benchmark report stored at
      `docs/progress/features/t11-f05-temporal-controller-fixtures.json` and
      appended to the gate `closed` list.
- [ ] `make check` passes (exit 0) at the closing commit.

## Performance and Goal Impact

Predeclared cost: none. The diff adds integration tests and one Make target
and touches no simulation code, so every deterministic gate counter must be
bit-identical to the T01.F12 report (the last closed gate reference) and to
the T10.F10 epoch baseline; any wall-clock delta is noise inside the 25
percent flag threshold. The epoch baseline is not re-pinned. No indicator is
added or wired; goal-v1 readings are unchanged and no goal run is made. This
is not a mechanism feature and has no natural analog; it observes the clocks
that T11.F06 (neural timescales) and T11.F07 (synaptic eligibility) will set.

Measured: to be recorded at closure with the table of six counters, both
references, and wall-clock per creature-tick.

## Success Criteria

- [ ] The twelve fixtures and the property test run in `make check` through
      the production tick path at production runtime settings.
- [ ] The catalogue records, per fixture, expected behavior, observed
      behavior with measured values, and a status; each gap is assigned to
      T11.F06 or T11.F07 with the observation that motivates it.
- [ ] No production file changed; the gate report's deterministic block
      equals both references.

## Notes for AI Agents

- Readiness review (orchestrator, 2026-09-05): one revision applied before
  commit. B2 was tightened to a single construction (store at tick 1, clear at
  tick 2, read the previous slot at ticks 2 and 3); E1 leaves the outcome
  channel to the implementer because the exact signal depends on action
  costs the test does not set. The repeated-dispatch fixture was moved to
  T11.F06 so this feature stays at twelve fixtures.
- The T11.F01 worktree is active concurrently. If `main` moves before this
  feature's fast-forward, stop and report per the workflow; do not rebase.
- Orchestration deviation: the `SendMessage` tool is absent from this
  session, so any remediation pass runs on a fresh implementer with a tight
  brief instead of continuing the first; record each pass here.
