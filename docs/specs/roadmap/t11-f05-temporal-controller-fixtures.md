# T11.F05 — Temporal Controller Fixtures

**Status**: Complete
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
  PushAction, ExecuteActionQueue, Halt}`, `WorldInputKey::FoodHere`, and
  `StaticIntrospectionKey::AgeTicks`.
- Production settings held fixed and never edited by a fixture:
  `RuntimeConfig::default()` (`max_graph_relax_iters` 15,
  `graph_convergence_epsilon` 1e-3, `graph_convergence_stable_passes` 2,
  `graph_node_base_cost` 1e-5, `plasticity_update_cost` 0,
  `reward_learning_cost` 0, `max_mesh_hops` 1024) and
  `shared_memory.decay_rate` 0. The only permitted deviations are the
  existing `test_config` precedent (tiny world, zero food coverage and
  growth, zero energy decay, zero mutation probability), the one labeled
  decay treatment in fixture C2, and `max_graph_relax_iters` 1 in the
  property test so one dispatch is one operator step; the file's doc
  comment records them.
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
| A1 reactive control | VM: `ReadInput` FoodHere; if > 0 push `Move(E)`, else `NoOp`; `ExecuteActionQueue`. Food toggled on the cell by the test across 4 ticks. | Action tracks the current observation every tick; identical observations give identical actions. | Actions were `Move(E), NoOp, Move(E), NoOp` for `food = 1, 0, 1, 0` across ticks 1-4 (the east neighbor is barriered so a successful move never relocates the creature off the observed cell). No persistent state to desynchronize from the tick clock. | meets |
| B1 shared-memory delayed cue | VM: on any tick with FoodHere > 0, `StoreSlotImm` slot 0 = 1.0; then `LoadSlotImm` slot 0, if > 0 push `Move(E)` else `NoOp`; never eats. Cue trial: food on the cell at tick 1 only (test removes it before tick 2). No-cue trial: never. Decision tick 1 + d for d in {1, 2, 4, 8, 16}. Assert `static_inputs` at the decision tick equal across trials, cue trial `Move(E)`, no-cue trial `NoOp`. Also assert A1 emits `NoOp` in both trials at the decision tick (matched reactive control). | Acts on the cue at every delay. | Cue trial emitted `Move(E)` and no-cue trial `NoOp` at the decision tick for every d in {1,2,4,8,16} (10 trial runs); the matched A1 control emitted `NoOp` in both trials at every delay; `static_inputs` (food_here, neighbor rings, generation, age_ticks) at the decision tick were identical field-by-field across the cue and no-cue trials. The east neighbor is barriered in both trials so the cue trial's own `Move(E)` after the cue tick cannot relocate the creature and desync the comparison. | meets |
| B2 previous-slot one-tick cue | VM: at tick 1 (AgeTicks == 1) `StoreSlotImm` slot 0 = 1.0; at tick 2 `ClearSlot` slot 0 before reading; every tick read slot 0 via `LoadSlotPrev` and push `Move(E)` if > 0 else `NoOp`. Assert the actions at ticks 2 and 3. | Tick 2 sees the tick-1 value (one-tick element); tick 3 sees 0. | Tick1 `NoOp` (no prior cue), tick2 `Move(E)` (sees tick1's latch via `prev_shared_memory`), tick3 `NoOp` (latch cleared at tick2). Matches the idealized one-tick memory element exactly. | meets |
| C1 retention | VM stores 0.75 into slot 3 at tick 1 and never writes again. After d ticks for d in {1, 2, 4, 8, 16}: `shared_memory[3]` and, from tick 2, `prev_shared_memory[3]`. | 0.75 exactly at every delay at production decay 0. | `shared_memory[3] == 0.75` and `prev_shared_memory[3] == 0.75` (both within 1e-6) at every tested delay; zero decay leaves the latched value bit-stable. | meets |
| C2 retention under decay (labeled treatment) | C1 with `shared_memory.decay_rate` = 0.1, the one non-production setting. | `0.75 * 0.9^d` within 1e-5 after d ticks. | Measured `shared_memory[3]` matched `0.75 * 0.9^d` within 1e-5 for every d in {1,2,4,8,16} (e.g. d=1: 0.675; d=16: 0.75\*0.9^16 ≈ 0.138977); decay is applied exactly as documented (snapshot to `prev_shared_memory` happens before the multiplicative decay each tick). | meets |
| C3 graph slot write and previous read | Graph: `Constant(0.6)` wired to `WriteSlot(0)`; a second node `Add` reading `SharedMemory { slot: 0, previous: true }` wired to `WriteSlot(1)`. | Tick 1: slot 0 = 0.6, slot 1 = 0; tick 2: slot 1 = 0.6. | Observed exactly: tick1 slot0=0.6, slot1=0.0; tick2 slot1=0.6. `previous: true` reads the tick-start snapshot regardless of same-tick `WriteSlot` ordering. | meets |
| D1 integrator clock | Graph: node 0 `Constant(1.0)`; node 1 `DecayIntegrator(0.5)` with one edge from node 0, weight 1. Read `node_state[..][1]` and per-tick `graph_relax_iters` after ticks 1..4. | Per-tick clock: 0.5, 0.75, 0.875, 0.9375. | Observed passes per tick: 11, 3, 3, 3 (not 1 every tick). Observed states: 0.99951172, 0.99993896, 0.99999237, 0.99999905 (not 0.5, 0.75, 0.875, 0.9375). Tick 1 alone runs 11 relaxation passes because `prev_outputs` resets to zero each tick while `node_state` persists, so the convergence check compares the persisted state against an artificial zero baseline; the integrator races to its fixed point within the first tick instead of advancing one step per tick. **Gap — assigned to T11.F06**: the graph relaxation clock is not a one-tick-per-relaxation clock. | gap (T11.F06) |
| D2 disconnected-node perturbation | D1 plus node 2 `Oscillator(0.25)` with no inputs and no consumers. | Identical integrator trajectory and pass count to D1 (contract property 3). | Observed passes per tick: 15, 15, 15, 15 (every tick hits the `max_graph_relax_iters` cap, vs. D1's 11, 3, 3, 3). Observed integrator states: 0.99996948, 1.0, 1.0, 1.0 (vs. D1's 0.99951172, 0.99993896, 0.99999237, 0.99999905 — numerically different from tick 1 onward). The disconnected, unconsumed `Oscillator` node cycles output 1, 0, -1, 0, ... every pass and never satisfies the convergence epsilon, forcing every tick to the pass cap regardless of the integrator's own convergence. **Gap — assigned to T11.F06**: a structurally disconnected, unconsumed node changes both the settling-pass count and (through it) another node's trajectory, violating node-type contract property 3. | gap (T11.F06) |
| D3 backward-edge recurrence | Graph: node 0 `Constant(1.0)`; node 1 `Add` with edges from node 0 and from itself (`ComputeNode(1)`), both weight 1; node 1 wired to `WriteSlot(0)`. Read slot 0 after ticks 1..3. | A one-tick memory element: 1, 2, 3. | Observed: slot0 = 15.0, 15.0, 15.0 across ticks 1-3 (not 1, 2, 3). The self-loop reads `prev_outputs` (reset to zero every tick, not the previous tick's converged output — `Add` has no persistent `state`), so it accumulates by 1.0 per relaxation pass and is capped at `max_graph_relax_iters` = 15 every tick, giving the same value every tick instead of a genuinely advancing one-tick memory element. **Gap — assigned to T11.F06**: a backward edge reads the previous relaxation pass, not the previous world tick. | gap (T11.F06) |
| E1 exact one-edge update, immediate reward | Graph: node 0 `Constant(1.0)`; node 1 `Add` with one edge from node 0, weight 1, `PlasticityConfig { rule: Classic, learning_rate: 0.5, weight_clamp: 10, modulation: Some { reward_source: <channel>, trace_decay: 0.5 } }`; node 1 wired to the execute gate and an `Eat` action slot so the controller acts every tick. Choose the outcome channel whose signal the test controls exactly (`ActionSuccess` with `Eat` on a food cell gives 1.0; `EnergyDelta` is read from `energy` before and after) and record it. Read `eligibility_traces` and `plasticity_weights` after tick 1. | One calibrated rule with the learning rate applied once: `dw = eta * signal * pre * post`. | Chosen channel: `ActionSuccess` (the wired `Eat` succeeds every tick with food present, giving signal = 1.0 exactly — the only attempted action). Observed after tick 1: `eligibility_traces[0][1][0] = 0.5` (= eta·pre·post, matches `decay·0 + eta·pre·post`); `plasticity_weights[0][1][0] = 1.25` (genome weight 1.0 + 0.25). The realized gain is 0.25 = eta² · signal · pre · post, not eta · signal · pre · post (0.5) — the learning rate is folded into the trace and applied again at the reward update. **Gap — assigned to T11.F07**: one calibrated rule, not a double application of eta. | gap (T11.F07) |
| E2 delayed reward, visited every tick | E1 with the signal held at 0 on ticks 2..d and nonzero at tick d + 1, for d in {1, 2, 4} (control the signal through the world, for example food present only on the reward tick). | Credit at tick d + 1 is discounted by elapsed world time: `eta * signal * decay^d * trace_1` plus the tick's own activity term. | For d in {1,2,4} (reward tick d+1 = 2, 3, 5), the observed trace at the reward tick matched the elapsed-tick recurrence `trace_n = decay·trace_{n-1} + eta·pre·post` computed over `n = d+1` ticks (e.g. d=1: trace=0.75, weight=1.375; d=4: trace≈0.96875, weight≈1.484375) within 1e-5. **Timing: meets** — because the module runs every tick, decay correctly tracks elapsed world time (contrast with E3). **Gain: inherits E1's gap** — the weight update still folds eta into the trace and again into the reward update (`expected_weight = 1.0 + eta·signal·expected_trace` where `expected_trace` already carries one factor of eta), so this fixture does not clear T11.F07 on gain, only on timing. | meets (timing) / gap (T11.F07, gain) |
| E3 skipped module visits | Entry VM node routes to the E1 graph node only on ticks with FoodHere > 0 (`WriteRouteGate` toward the graph target, otherwise `NoOp` + `ExecuteActionQueue`). Visit at tick 1, skip ticks 2..3. Read traces and weights after each tick. | Traces reflect elapsed ticks whether or not the module ran; a skipped tick carries no stale credit. | Reward channel: `EnergyDelta` (measured directly from `creature.energy` around each tick, and asserted nonzero on every tick so the weight-moved claim below is not vacuous), with no action bank wired on the graph node. The signal is not "ordinary VM/graph opcode cost": on the visit tick the graph node has no route targets of its own, so the mesh soft-defaults to `NoOp`; on a skipped tick the entry VM explicitly pushes `NoOp`; both paths land on Phase 2's `noop_cost` (0.05, `crates/v3-core/src/config/simulation.rs:390`), which dominates the VM/graph opcode costs it sits on top of (1e-6 to 1e-5 scale). Measured signal ≈ -0.05 per tick (signal_1 ≈ -0.050079, skipped signal ≈ -0.050018). Visit tick 1: trace = 0.5 (eta·pre·post), weight = 1.0 + eta·signal_1·trace_1. Skipped ticks 2 and 3: **trace stayed frozen at the tick-1 value (within 1e-9)** — it does not reflect the two elapsed ticks — while **the weight still moved** each skipped tick by `eta·signal·trace_1` ≈ -0.0125 (the stale, frozen trace), because `run_reward_learning` applies to every creature with a reward-modulated node every tick regardless of whether that node's mesh hop executed. **Gap — assigned to T11.F07**: traces do not advance with elapsed ticks when the module is skipped, and reward learning still spends stale credit on skipped ticks rather than withholding it. | gap (T11.F07) |

Property test (pure invariant, proptest): for any `a`, `b` in [0, 1], any
finite `state` and `input` in [-1e6, 1e6], one `DecayIntegrator(a)` step and
one `Momentum(b)` step through the public mesh path (or, if the implementer
adds a `pub` re-export of the evaluator for tests, through it) return a value
between `state` and `input` inclusive; assertions must not depend on which
cases are drawn. Commit any `proptest-regressions/` file that appears.

## Implementation Tasks

- [x] Create `crates/v3-core/tests/temporal_fixtures.rs` with a doc header
      listing the fixture IDs, the settings it holds at production values,
      and the two permitted deviations. Share the small helpers with
      `creature_workflow_e2e` by moving `test_config`, `insert_creature`,
      `run_one_traced_tick`, and the founder constants into
      `crates/v3-core/tests/common/mod.rs` and referencing them from both
      files; do not duplicate them.
- [x] Implement fixtures A1, B1, B2, C1, C2, C3, D1, D2, D3, E1, E2, E3 and
      the property test, each with a doc comment stating expected and
      observed behavior and the owning repair feature for any gap. Every
      fixture genome is a named constructor function (for example
      `delayed_cue_vm_genome(delay)`) so T11.F01 can lift it as a motif.
- [x] Add `rust-test-temporal-fixtures` to the `Makefile` (`.PHONY` and the
      `rust-test-all` prerequisites) so `make check` runs the suite.
- [x] Fill the catalogue's Observed and Status columns with the measured
      values, assign each gap to T11.F06 or T11.F07 in the table and in
      "Notes for AI Agents", and record the T11.F01 motif constructors.
- [x] Generate the gate report with `make bench PROFILE=gate
      FEATURE=t11-f05-temporal-controller-fixtures`, append it to `closed` in
      `docs/progress/benchmark-series.json` (gate series only; no goal run),
      add the T11.F05 row to `docs/progress.md` in the existing format, and
      complete Performance and Goal Impact.

## Verification

- [x] `cargo test -p v3-core --test temporal_fixtures` passes in a debug
      build in under 60 seconds; record the test count and time. **13 tests
      (12 fixtures + 1 proptest), 0.01–0.02s wall clock**, e.g.:
      `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered
      out; finished in 0.02s`.
- [x] `cargo test -p v3-core --test creature_workflow_e2e` still passes after
      the helper move (7 tests, ok); `cargo check --workspace --all-targets`
      and `make rust-clippy` pass (both exit 0, no warnings).
- [x] Viability: not run first, because no default, founder, or tick-loop
      mechanic changes; `make check` runs it in order (confirmed passing as
      part of `make check` below).
- [x] Fixture catalogue completed with observed values; every gap row names
      T11.F06 or T11.F07; no fixture is ignored or asserts a value it does
      not observe.
- [x] `make rust-mutants` summary line, output path, and full survivor list,
      each survivor killed, equivalent, or deferred ("nothing to mutate" is a
      valid record for a test-only diff). Output:
      ```
      rust-mutants: diff against ae012a87427ae262bc3355d166aa95957ac3187f, output in /Users/istefanek/.local/share/petri-tools/mutants/t11-f05/mutants.out
       INFO No mutants to filter
      rust-mutants: no survivors
      ```
      This diff touches only `tests/*.rs` files (no `src/` production code),
      so `cargo mutants --in-diff` found no production lines to mutate.
      Record: no survivors — valid for a test-only diff. The stated output
      path's `mutants.out` directory was not created; only `in-diff.patch`
      exists at `/Users/istefanek/.local/share/petri-tools/mutants/t11-f05/`,
      because cargo-mutants found no mutants in the diff at all (`INFO No
      mutants to filter`) and so had no mutant run to write a report
      directory for. "no survivors" is the tool's report for that empty
      mutant set, not evidence of a completed-and-clean mutant run.
      Second run (remediation pass, 2026-09-05, after the simplify pass
      below): identical result —
      ```
      rust-mutants: diff against ae012a87427ae262bc3355d166aa95957ac3187f, output in /Users/istefanek/.local/share/petri-tools/mutants/t11-f05/mutants.out
       INFO No mutants to filter
      rust-mutants: no survivors
      ```
      `mutants.out` still not created (only `in-diff.patch`), for the same
      reason as the first run: the diff still touches only `tests/*.rs`.
- [x] Remediation pass (2026-09-05, one P2 and five P3 review findings)
      applied and re-verified: `cargo fmt --all -- --check` (exit 0),
      `cargo test -p v3-core --test temporal_fixtures` (13 passed),
      `cargo test -p v3-core --test creature_workflow_e2e` (7 passed),
      `make rust-clippy` (exit 0, no warnings), `make roadmap-check`
      (`validation passed`), a single-pass `simplify` skill run (Agent tool
      unavailable; one fix applied — see Notes for AI Agents), and a second
      `make rust-mutants` run (recorded above). `git diff --stat
      main...HEAD -- crates/*/src/` remained empty (production code still
      untouched).
- [x] `make roadmap-check` passes after the document edits (`roadmap-check:
      validation passed`).
- [x] Benchmark report stored at
      `docs/progress/features/t11-f05-temporal-controller-fixtures.json` and
      appended to the gate `closed` list.
- [x] `make check` passes (exit 0). Ran to completion (exit 0) covering
      commit `ccf50afa` (policy-check, quality-check, rust-check —
      format/viability/all Rust test subsets including
      `rust-test-temporal-fixtures`/clippy, frontend-check, dependency-audit,
      skill-check all green; `temporal_fixtures`: 13 passed;
      `creature_workflow_e2e`: 7 passed). The commits after that
      (`ea4dd270`, `2bacadc1`, and this remediation pass's commit) are
      spec-only (this document's prose), each re-verified with
      `make roadmap-check` (passed) and
      `git diff --stat main...HEAD -- crates/*/src/` (empty) rather than a
      second full `make check`; the orchestrator records this remediation
      pass's own `make check` result at closure.

## Performance and Goal Impact

Predeclared cost: none. The diff adds integration tests and one Make target
and touches no simulation code, so every deterministic gate counter must be
bit-identical to the T01.F12 report (the last closed gate reference) and to
the T10.F10 epoch baseline; any wall-clock delta is noise inside the 25
percent flag threshold. The epoch baseline is not re-pinned. No indicator is
added or wired; goal-v1 readings are unchanged and no goal run is made. This
is not a mechanism feature and has no natural analog; it observes the clocks
that T11.F06 (neural timescales) and T11.F07 (synaptic eligibility) will set.

Measured: `make bench PROFILE=gate FEATURE=t11-f05-temporal-controller-fixtures`
against both references, `severe=false` for each:

| Counter (per creature-tick) | Current | T10.F10 epoch | T01.F12 previous | delta% (both) |
| --- | --- | --- | --- | --- |
| mesh_hops | 1.998362 | 1.998362 | 1.998362 | 0.000000 |
| vm_steps | 28.028231 | 28.028231 | 28.028231 | 0.000000 |
| graph_relax_iters | 2.998624 | 2.998624 | 2.998624 | 0.000000 |
| plasticity_updates | 0.000000 | 0.000000 | 0.000000 | both-zero (null) |
| actions_applied | 1.000000 | 1.000000 | 1.000000 | 0.000000 |
| births | 0.001212 | 0.001212 | 0.001212 | 0.000000 |

All six counters are bit-identical to both references, as predeclared.
Wall-clock: 0.004259825 ms/creature-tick, vs. T10.F10 epoch 0.005447724
(-21.805417%, noise) and T01.F12 previous 0.004590127 (-7.195926%, noise);
both inside the 25 percent flag threshold. Report:
[t11-f05-temporal-controller-fixtures.json](../../progress/features/t11-f05-temporal-controller-fixtures.json).

## Success Criteria

- [x] The twelve fixtures and the property test run in `make check` through
      the production tick path at production runtime settings.
- [x] The catalogue records, per fixture, expected behavior, observed
      behavior with measured values, and a status; each gap is assigned to
      T11.F06 or T11.F07 with the observation that motivates it.
- [x] No production file changed; the gate report's deterministic block
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
- Gaps by repair feature, for T11.F06 and T11.F07 to read directly instead of
  re-deriving from the fixtures: **T11.F06 (graph memory clock)** — D1 (a
  `DecayIntegrator` advances per relaxation pass, not once per tick, because
  `prev_outputs` resets to zero every tick while `node_state` persists,
  forcing a large first-tick settling burst); D2 (a structurally
  disconnected, unconsumed `Oscillator` node changes another node's settling
  pass count and trajectory, violating node-type contract property 3); D3 (a
  backward edge reads the previous relaxation pass via `prev_outputs`, not
  the previous world tick, and is capped by `max_graph_relax_iters` rather
  than acting as a one-tick memory element). **T11.F07 (reward trace
  clock)** — E1 (the learning rate is applied twice, once folded into the
  eligibility trace and again at the reward update, giving `eta^2` gain
  instead of one calibrated `eta`); E2 inherits E1's gain gap even though its
  elapsed-time discounting is correct; E3 (eligibility traces do not advance
  with elapsed ticks when the module is skipped — they freeze at the last
  visited tick's value — yet the reward-learning pass still spends that
  stale, frozen trace on every skipped tick because it does not check
  whether the node's mesh hop executed; the signal driving this is not
  ordinary VM/graph opcode cost as the fixture's genome comment originally
  claimed — the graph node has no route targets, so the mesh soft-defaults
  to `NoOp` on the visit tick and the entry VM explicitly pushes `NoOp` on a
  skipped tick, and both land on Phase 2's `noop_cost` (0.05), which
  dominates the 1e-6 to 1e-5-scale opcode costs; measured signal ≈ -0.05 per
  tick and weight movement ≈ -0.0125 per skipped tick, both now asserted
  nonzero in the fixture so the claim cannot pass vacuously).
- Motif constructors T11.F01 (and T09.F08, T09.F01, T11.F10) can lift
  directly from `crates/v3-core/tests/temporal_fixtures.rs`: reactive control
  — `reactive_control_vm_genome()`; shared-memory delayed cue —
  `delayed_cue_vm_genome()`; previous-slot one-tick cue —
  `previous_slot_cue_vm_genome()`; single-write retention —
  `slot_write_once_vm_genome(slot, value)`; graph slot write/previous-read —
  `slot_write_and_previous_read_graph_genome()`; integrator clock —
  `decay_integrator_graph_genome()` and, for the disconnected-node
  perturbation, `decay_integrator_with_disconnected_oscillator_graph_genome()`;
  backward-edge recurrence — `backward_edge_recurrence_graph_genome()`;
  reward-modulated node — `reward_modulated_node_genome(reward_source)` (with
  a wired `Eat` action) and `reward_modulated_node_genome_inert(reward_source)`
  (bare compute nodes, no action wiring, for signal channels like
  `EnergyDelta` that should not depend on food); conditional module routing —
  `conditional_route_vm_node(node_id, graph_target, alt_target)`. All are
  `pub(crate)`-free module-private functions in the same file; a caller
  outside that test binary needs to either duplicate the small construction
  or the two crates need a shared, more visible location — left to T11.F01
  to decide since this feature does not touch production visibility.
- Advisor consulted twice. **Deviation from the standing three-checkpoint
  rule**: consult 1 happened after the twelve fixtures and the proptest were
  already written and passing, not before committing to the implementation
  approach as the standing instructions direct — recorded here per the
  `SendMessage`-deviation precedent above. Consult 1 caught that B1's
  cue-trial creature physically walked off its start position after the cue
  latched (the same genome keeps emitting `Move(E)` every tick once the cue
  is seen), making the fixture's food-removal step vacuous and its
  decision-tick comparison meaningless; directed a fix mirroring A1's
  barrier-blocking plus a full `static_inputs` field comparison instead of
  `food_here` alone. It also directed: qualifying E2's "meets" status as
  inheriting E1's `eta^2` gain gap; dumping and transcribing D1/D2/D3's
  actual measured values into the catalogue rather than relying solely on
  the in-test helper-vs-observed equality; three simplify-pass items (inline
  the `ActionSlotBehavior` shim, replace the function-pointer identity check
  with an explicit tuple flag, move `graph_hop` into `common/mod.rs`); and
  two documentation notes (the `WorldInputKey`/`AgeTicks` discrepancy above,
  and E3's `EnergyDelta` measurement depending on `energy_decay_per_tick ==
  0.0` and `reward_learning_cost == 0.0` staying at their current defaults).
  Consult 2 (this one, before reporting done) verified the fixes were
  correctly applied and flagged two closing gaps: confirm `make check`'s
  actual exit status on the closing commit rather than an in-flight run, and
  confirm mechanically (not just by assertion) that no production file
  changed (`git diff --stat main...HEAD -- crates/*/src/` is empty).
- Remediation pass (fresh implementer, 2026-09-05; the `SendMessage`
  deviation above applies again). Applied a reviewed one-P2/five-P3 finding
  set: E3's `EnergyDelta` signal now asserted nonzero on the visit tick and
  every skipped tick (probed and confirmed: signal_1 ≈ -0.050079, skipped
  signal ≈ -0.050018, both dominated by `noop_cost` 0.05, not by opcode
  cost); E3's two doc comments and its catalogue row and Notes entry
  reworded to attribute the signal to the mesh's soft-default `NoOp` (visit
  tick, no route targets) and the entry VM's explicit `NoOp` (skipped
  ticks), not to "ordinary VM/graph opcode costs"; the D1/D2 test bodies now
  assert against the literal `D1_PASSES`/`D1_STATES`/`D2_PASSES`/`D2_STATES`
  constants recorded from the catalogue instead of a duplicated
  `simulate_decay_integrator_clock` helper (deleted, along with its
  never-converging variant, since nothing else called them); the proptest's
  `max_graph_relax_iters = 1` setting is now the file header's third labeled
  deviation, with a discrepancy note above recording that the Inputs and
  Invariants section's deviation count predates it; the redundant
  `#[path = "common/mod.rs"]` attributes were removed from both test files
  (a plain `mod common;` already resolves `tests/common/mod.rs`); the
  Verification section's stale "one commit after that" sentence was
  corrected to name both spec-only commits after `ccf50afa`
  (`ea4dd270`, `2bacadc1`) and this pass's own commit, without implying a
  fresh full `make check` ran during the pass (the orchestrator records
  that at closure); and a note was added recording that `mutants.out` was
  never created because cargo-mutants found no mutants at all, not because
  a mutant run completed clean. Two advisor consults. Consult 1, before
  starting the edits: confirmed the probed E3 signal values against the
  finding's description (noop_cost, not move_cost or failed_action_penalty),
  flagged that both E3 doc comments (the genome constructor's and the
  test's) carry the same error and both needed rewording, identified three
  more stale spots the D1/D2 helper deletion touches beyond the assertions
  themselves (the "── D helpers" section comment, D1's doc-comment
  cross-reference to the deleted helper, and D2's second call to the deleted
  helper for its `d1_states`/`d1_passes` comparison), and recommended
  verifying the `git log` range and the mutants output directory's actual
  contents before wording items 2 and 5 rather than transcribing the
  brief's text unverified. Consult 2, before reporting done: caught that the
  rewritten "D helpers and literals" comment (written per consult 1's
  guidance) had overcorrected into a false claim — it said the literals
  "were derived by, and match to float precision, the documented capped
  Gauss-Seidel recurrence" and gave pseudocode with no term for the
  disconnected oscillator, so a reader running that pseudocode would get
  D1's numbers, not D2's; directed rewording it to state the literals are
  measured from the production tick path with the recurrence explained only
  as the mechanism (why), not a re-derivable formula. It also caught a
  forward reference in the E3 doc comment ("see the assertions below for
  the exact recorded value") pointing at assertions that only check
  `> 1e-3`, no exact value; replaced with the measured numbers directly.
  Both fixes are applied in the file. The single-pass `simplify` skill run
  (Agent tool unavailable
  in this session, so it ran as one inline pass rather than the usual
  4-agent fan-out) found one fix: `assert_eq!`/`assert_ne!` comparisons were
  calling `.to_vec()` on the `D1_PASSES`/`D2_PASSES` array constants before
  comparing against `Vec<u32>`, an unneeded allocation — `Vec<T>` already
  implements `PartialEq<[T; N]>`, so the comparisons now compare the array
  directly. No reuse, altitude, or other efficiency findings; the doc
  comments and constant layout were left as sized for a 4-tick fixture, not
  worth compressing further.
- Closure (orchestrator, 2026-09-05): the two spec-text discrepancies the
  implementer recorded (`AgeTicks` is a `StaticIntrospectionKey`, and the
  property test's `max_graph_relax_iters` 1 is a third labeled deviation)
  were corrected in the Inputs and Invariants section at closure and their
  notes removed. Review 2026-09-05 (roadmap-reviewer, fresh context): 0 P1,
  1 P2, 5 P3; all six were fixed in the one remediation pass at `13c6b69c`.
  No deferred review findings. `make check` exited 0 on the closing content
  (the closing commit cannot embed its own hash; the goal transcript shows
  the hash and the exit code).
- Cost record at closure: `/usage` totals not collected (autonomous session;
  `/usage` is a user command). Implementer advisor consults: 2 in the
  implementation pass, 2 in the remediation pass. Reviewer findings: P1 0,
  P2 1, P3 5.
- Integration blocker (orchestrator, 2026-09-05): after `ExitWorktree`,
  `git merge --ff-only worktree-t11-f05` on `main` (still at `ae012a87`)
  refused with "Your local changes to the following files would be
  overwritten by merge: Makefile". The main checkout carried uncommitted
  work that is not part of this feature (a `scripts/bench-wait` benchmark
  preflight: modified `Makefile` and `docs/workflow.md`, untracked
  `scripts/bench-wait` and `scripts/bench-wait-test`, written 08:59 to 09:00
  the same day). The orchestrator did not stash, commit, or discard it. The
  feature branch is complete at `2544dd42`, where `make check` exited 0; this
  note is a documentation-only commit verified with `make roadmap-check`. To
  integrate: commit or set aside the bench-wait work in the main checkout,
  then run `git merge --ff-only worktree-t11-f05` (if `main` has moved by
  then, a regular merge is needed; the branch's `Makefile` edits are the
  `.PHONY` and `rust-test-all` lines and do not overlap the `bench` recipe),
  then `git worktree remove .claude/worktrees/t11-f05` and
  `git branch -d worktree-t11-f05`.
