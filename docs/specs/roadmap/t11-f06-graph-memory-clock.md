# T11.F06 — Graph Memory Clock

**Status**: In Progress
**Last updated**: 2026-09-06
**Feature**: T11.F06
**Track**: [T11 — Brain Genotype-Phenotype Map](../../roadmaps/t11-brain-genotype-phenotype-map.md)

## Goal

Graph memory has a world-tick clock: stateful operators take one temporal
step, and self/backward edges read the preceding tick's outputs. Extra
settling passes, repeated mesh visits, or disconnected computation cannot
accelerate that clock. Ordered combinational paths still compute in one visit.

## Non-Goals

- No new controller family, VM semantics, mesh routing, mutation supply,
  founder redesign, or learned-state inheritance changes.
- No eligibility decay or reward-gain repair (T11.F07), evolution assay
  (T11.F10), or full intervention framework (T09.F08).
- No threshold weakening, historical report replacement, second goal run,
  or persistence sweep campaign.

## Inputs and Invariants

- Sources: the T11.F06 track note and node-type contract;
  [T11.F01](t11-f01-mutational-neighborhood-indicator.md) for the fixed
  neighborhood battery; [T11.F05](t11-f05-temporal-controller-fixtures.md)
  fixtures D1–D3 for measured defects; the graph, mesh-execution, mutation,
  and runtime-config reference specs. Dependencies remain owned by the track.
- Extend existing `GraphRuntimeState`, `runtime/cgp/{execute,sources}.rs`,
  tick bookkeeping, temporal fixtures, and the benchmark's observation seam.
  No new dependency is needed. Production genomes are immutable during life;
  newborn graph temporal state starts at zero and is not inherited.
- Research checked 2026-09-06: [NEAT-Python's recurrent activation source](https://neat-python.readthedocs.io/en/latest/_modules/nn/recurrent.html)
  separates previous and next values with two buffers. Options considered:
  retain bounded relaxation with frozen temporal inputs, or replace relaxation
  with one ordered evaluation and persistent tick snapshots (selected).
  The latter removes pass-dependent timing and redundant work. Unlike NEAT's
  fully synchronous edge evaluation, Petri retains current-visit values for
  edges from lower indices, preserving its combinational paths. This is an
  extension of Petri's resolver, not adoption of a different controller.
- Clock contract: explicitly begin a world tick in production and in the
  neighborhood sequence path. Snapshot committed operator state and compute
  outputs once. During a visit, lower-index sources read current-visit
  outputs; self and higher-index sources read the frozen tick-start outputs.
  Evaluate every compute node exactly once. Stateful evaluation starts from
  the frozen tick-start operator state. Each successful visit replaces the
  pending/current committed result from that same base, so changed inputs
  can be recomputed on a repeated visit without taking a second temporal
  step. The last successful visit supplies next tick's state and outputs.
  Unvisited modules hold their values: no fabricated input, catch-up loop,
  autonomous update, or energy charge. Empty graphs do no work.
- Effects use the visit's computed outputs and current input/shared-memory
  values as before. Repeated visits retain their normal side effects and
  learning events; only graph temporal stepping changes here. The graph's
  old relaxation/convergence settings must no longer alter behavior or cost;
  document their disposition consistently with the existing config surface.
- Exhaustion is transactional for graph temporal state: if a visit exhausts
  energy before graph effects, preserve the prior successful operator state
  and persisted outputs, emit no graph effects, and retain the actual charge
  and entered-work count. Cover both evaluation and plasticity-cost exhaustion.
  Do not use this requirement to redesign reward or learned-weight rollback.
- Traced and ordinary execution share the same evaluator and clock. Counters
  and trace state describe applied execution. Retain `graph_relax_iters` as
  the wire counter for entered nonempty single-evaluation graph visits,
  including an unaffordable visit; record this changed definition explicitly
  in report/series metadata and progress documentation. It no longer counts
  convergence iterations. Never silently present its cross-definition delta
  as like-for-like work efficiency.
- Preserve the current-memory sensitivity component. Add a separately
  versioned `temporal_memory_sensitivity` component to the goal report with
  separate previous-slot, persisted-output, and operator-state interventions:
  zero and deterministic one-position rotation within each slot vector,
  changing only the named substrate and comparing full action queues with
  intact cloned state. Define the observation boundary explicitly so a
  tick-start snapshot cannot overwrite a perturbation and repeated-visit
  caching cannot make the probe inert. No live state or RNG mutation.
  Historical reports load with the new component `Undefined`. Reuse this
  bounded intervention vocabulary when T09.F08 is later planned.
- Keep neighborhood samples, seeds, batteries, and mutation probabilities
  fixed. The multi-tick path must invoke the actual graph clock. Add
  constructed positive controls for the new memory probe and clock, without
  replacing or silently enlarging the comparable neighborhood battery.

## Implementation Tasks

- [x] Flip D1–D3 to the intended per-tick trajectories and observe their
      failures before production edits; add repeated-visit, skipped-visit,
      combinational, traced-parity, newborn, and exhaustion regressions.
- [x] Implement tick snapshots and one ordered graph evaluation, using
      existing runtime storage/scratch patterns and shared clock bookkeeping.
- [x] Add property tests for one temporal step regardless of former pass
      settings and disconnected nodes, including integrator, momentum,
      oscillator, and adaptive gain; assert independently of drawn cases.
- [x] Add and test the versioned temporal-memory report component, substrate
      positive controls, observation non-mutation, and historical serde loading.
- [ ] Update affected reference/config/trace descriptions, T11.F05's current
      capability record (preserving its historical measurements), benchmark
      counter semantics, stored gate/goal reports, and progress row.
- [ ] Complete reuse/simplification/efficiency self-review, fresh mutation
      survivor triage, and final advisor consultation before review.

## Verification

- [x] Record TDD red/green commands and outcomes. After production semantics
      change, run `cargo test -p v3-core --test viability` first, then
      `cargo check --workspace --all-targets` after coherent Rust edits.
- [x] Run temporal fixtures, relevant runtime/observation/report tests, and
      property tests; commit any generated proptest regression files.
- [x] Run `make roadmap-check` on document edits and before reporting done.
- [x] Run fresh `MUTANTS_ITERATE=0 make rust-mutants` after self-review;
      record summary, output path, and the full missed/timeout list, resolving
      each as killed by tests, equivalent with reason, or explicitly deferred.
- [ ] Store `make bench PROFILE=gate FEATURE=t11-f06-graph-memory-clock`
      at `docs/progress/features/t11-f06-graph-memory-clock.json` and one
      `make bench PROFILE=goal FEATURE=t11-f06-graph-memory-clock`
      at `docs/progress/features/t11-f06-graph-memory-clock-goal.json`.
      Use the existing benchmark preflight and no competing local workload.
- [x] Second goal run: Not applicable by the 2026-09-05 workflow decision;
      cross-process reproducibility is covered by `make check`. The gate's
      two-run byte-identity test remains required.
- [ ] Independent orchestrator `make roadmap-check`, fresh final review,
      and `make check` exit 0 on final feature content; record the tested
      commit in this task after committing and checking for hook edits.

### Implementation evidence (in progress)

- TDD red: `cargo test -p v3-core --test temporal_fixtures d -- --nocapture`
  exited 101 before production edits; D1 observed [11,3,3,3] vs [1,1,1,1],
  D2 [15,15,15,15] vs [1,1,1,1], D3 15 vs 1.
  Log: `/private/tmp/t11-f06-tdd-red.log`.
- First verification after semantic edit: `cargo test -p v3-core --test viability`
  exited 0, 25 tests. Log: `/private/tmp/t11-f06-viability.log`.
- Green: `cargo test -p v3-core --test temporal_fixtures` exited 0, 13 tests;
  `/private/tmp/t11-f06-temporal.log`. Temporal substrate positive controls pass
  in `/private/tmp/t11-f06-observation.log`. Workspace/all-target compiler
  feedback was run explicitly after coherent edits.
- The first `make check` passed core/CLI tests but seven server WebSocket
  tests could not bind localhost in the sandbox. The same checks were
  rerun successfully with authorized local listener access; no tests were weakened.
- Reuse/simplification/efficiency self-review: kept the source resolver and
  allocation-reusing scratch buffers; committed candidate temporal state only
  after affordability; shared one intact observation across three substrates
  (seven graph observations rather than nine), used enum interventions and
  fixed arrays, removed obsolete internal convergence bookkeeping. Trace
  candidates carry `temporal_committed` and inspector labels match that state.
- Advisor consultations: 3, all accepted. First endorsed frozen bases,
  candidate commit, explicit outer clock and per-vector interventions. Second
  confirmed graph-only observation preparation from final committed state with
  sensors/current/previous slots fixed and no shared-memory snapshot/decay.
  No rejected advice or scope expansion. Consultation 3 is recorded in Notes below.

- Authorized local-listener rerun: `make check` exited 0 (exec session 36862),
  including 25 viability tests, 1,141 core unit tests, 13 temporal fixtures,
  reproducibility, CLI gate two-run byte identity, 80 server integration tests,
  Clippy, 54 frontend test files / 273 tests and frontend production build.
  Log: `/private/tmp/t11-f06-make-check.log`. A subsequently added focused
  inspector commit-status regression passes 2 tests in
  `/private/tmp/t11-f06-ui-test.log`; final orchestrator check remains required.
- `make roadmap-check` exited 0 in `/private/tmp/t11-f06-roadmap2.log`.

### Mutation survivor audit

Fresh run 1 (`MUTANTS_ITERATE=0 make rust-mutants`) exited 0:
`40 mutants tested in 4m: 8 missed, 19 caught, 13 unviable`; no timeouts.
Original output: `/Users/istefanek/.local/share/petri-tools/mutants/t11-f06/mutants.out`,
preserved before rerun at `/private/tmp/t11-f06-mutants-fresh1-output`.
Log: `/private/tmp/t11-f06-mutants-fresh1.log`.

Full initial survivor list (line numbers from that run):

| Survivor | Resolution |
| --- | --- |
| `crates/v3-cli/src/bench.rs:184:5` replace `legacy_graph_work_definition -> String` with `String::new()` | **Killed** by literal historical-metadata assertion; confirmed fresh run 2. |
| `crates/v3-cli/src/bench.rs:184:5` replace `legacy_graph_work_definition -> String` with `"xyzzy".into()` | **Killed** by the same literal assertion; confirmed fresh run 2. |
| `crates/v3-cli/src/bench.rs:1069:57` replace `==` with `!=` in `temporal_memory_sensitivity` | **Killed** by report-level positive control for per-substrate denominator/differences; confirmed fresh run 2. |
| `crates/v3-core/src/simulation/tick.rs:300:5` replace `observe_temporal_actions` with `vec![]` | **Killed** by requiring all three observations; confirmed fresh run 2. |
| `crates/v3-core/src/runtime/cgp/execute.rs:119:49` replace `*` with `/` | **Killed** by two-node actual energy assertion (100 to 99.5 at cost 0.25); confirmed fresh run 2. |
| `crates/v3-core/src/runtime/cgp/execute.rs:153:27` replace `&&` with `\|\|` | **Equivalent**: a plastic node implies `has_any_hebbian`; nonplastic nodes have empty learned slices (also under child inheritance) and the alternative collector uses the same resolver and genome weights via `effective_weight` fallback. Living genomes are immutable. |
| `crates/v3-core/src/runtime/cgp/execute.rs:210:26` replace `-` with `+` | **Killed** by trace tick-start delta assertions 1, 0.5, 0.25; confirmed fresh run 2. |
| `crates/v3-core/src/runtime/cgp/execute.rs:210:26` replace `-` with `/` | **Killed** by the same trace-delta assertions; confirmed fresh run 2. |

Only tests changed for survivor remediation. No exclusions or skip attributes
were added. Repeated self-review found no further production simplifications;
the strengthened assertions use explicit expected values and non-vacuous counts.
Targeted verification: workspace/all-target check exited 0 in
`/private/tmp/t11-f06-check-compile2.log`; clock tests passed in
`/private/tmp/t11-f06-clock-tests2.log`; all 30 CLI unit tests passed in
`/private/tmp/t11-f06-cli-tests2.log`.

Fresh closure run 2 (`MUTANTS_ITERATE=0 make rust-mutants`) exited 0:
`40 mutants tested in 4m: 1 missed, 26 caught, 13 unviable`; no timeouts.
Output: `/Users/istefanek/.local/share/petri-tools/mutants/t11-f06/mutants.out`
(`run-mode.txt` in its parent records `fresh`). Log:
`/private/tmp/t11-f06-mutants-fresh2.log`. Full final survivor list:

- **Equivalent**: `crates/v3-core/src/runtime/cgp/execute.rs:153:27`, replace
  `&&` with `||` in `execute_graph_impl`, for the empty-learned-slice fallback
  reason above. There are no deferred mutation findings.

Applied charge audit: fixtures assert actual paid energy independently of
entered-visit counters, including multiple nodes, repeated visits, and both
exhaustion boundaries. The benchmark measures work and wall time; it does not
add a graph-specific cumulative energy series or infer one from visit counts,
because compute-node counts vary. No baseline or threshold is changed.

## Performance and Goal Impact

Predeclared before implementation: natural analog is neural timescales,
expressed through remembered decisions and paid brain computation, with no new
sensor. Replace usually at least three graph passes with one per visit;
requested graph energy becomes `graph_node_base_cost * compute_node_count`
per entered nonempty visit. Repeated visits still pay for actual recomputation.
Persistent output and tick-start state storage adds linear memory and snapshot
copying for initialized graph modules. No severe increase in deterministic
work counters or host compute time is justified in advance. Measure actual
work and charged energy separately; reduced cognition cost can change ecology.

Compare gate per-creature-tick counters and wall time against both the previous
closed feature and epoch baseline (both T11.F04 at launch). Explain the
`graph_relax_iters` definition break in series metadata while retaining raw
historical comparisons and unchanged thresholds; do not re-pin a baseline to
hide a failure. Report persistence, births, structure, lineage diversity, old
and new memory sensitivity, and evolved neighborhood readings from the single
goal run, with T01.F11/T01.F12 baselines identified by profile rather than
conflating their different horizons.

Neighborhood expectation: VM-only behavior and neutral graph growth should
retain their readings. Stateful/recurrent graph variants can move from changed
or dead to silent or active as pass-cap behavior disappears; the reverse is
also permitted specifically where a previous within-tick recurrent computation
now requires multiple world ticks. Reduced graph energy can change dynamic
energy-sensitive decisions. Attribute every reduced component to these
predeclared mechanisms with concrete evidence; unrelated regressions block
closure. Compare founder denominators and every changed row against T11.F04;
evolved populations may differ and must be described as such. No floor is
loosened, and no operator family is disabled or reweighted.

## Success Criteria

- [x] D1 follows 0.5, 0.75, 0.875, 0.9375; D2 has that same trajectory;
      D3 produces 1, 2, 3 across ticks. Former pass settings cannot alter them.
- [x] Repeated visits, skipped visits, combinational propagation, and
      exhaustion obey the declared clock and energy contract in both paths.
- [x] Goal reports can detect previous-slot, persisted-output, and operator
      memory sensitivity in positive controls without mutating the world.
- [ ] Required reports, truthful reference specs, review, and checks pass;
      the feature is checked and this spec Complete at the tested commit on
      clean main, with its worktree and branch removed.

## Notes for AI Agents

- Planning readiness review (orchestrator, 2026-09-06): checked the template,
  dependencies, measured D1–D3 gaps, runtime source and observation seams,
  counter meaning, bounded scope, and closure requirements. One drafting
  revision made repeated visits recompute from a frozen base instead of
  freezing all combinational outputs; this preserves same-tick input use.
  Ready for implementation and its required approach consultation.
- Launch evidence: clean main at `adb8e203b357d62e6aea3df8b5ab55c05bdeba41`;
  worktree `/Users/istefanek/projects/petri/.worktrees/t11-f06`, branch
  `codex/t11-f06`. Session metadata verified Astra `gpt-6-astra`, `xhigh`.
- Cost record: pending closure; task usage unavailable; advisor consults 3 (all accepted),
  final reviewer not yet run.


- Advisor consultation 3 (pre-completion code/evidence audit): accepted.
  No correctness blocker; confirmed frozen snapshots, transactional temporal
  commit, production/sequence clock boundaries, graph-only probe preparation,
  substrate isolation, nonmutation evidence, and final survivor equivalence.
  Applied only a stale-comment correction in `runtime/plasticity/hebbian.rs`
  (ordered evaluation replaces relaxation terminology); learning code unchanged.
  Performance conclusions, final review, and orchestrator check remain pending.
- Concrete measurement blocker (2026-09-06): the required measured gate's
  `scripts/bench-wait` was held by PID 59652, `target/release/v3-server`,
  parent PID 1, cwd `/Users/istefanek/projects/petri`, started 00:14:03 local.
  No measured gate or goal report has been produced. The orchestrator asked
  the user whether to stop that exact server, which may hold their simulation;
  approval is pending and the implementer has not stopped it or bypassed the
  guard. Before pausing, only this task's waiting guard (PID 84202) was
  stopped; its make command exited 2 without starting measurement, while
  server PID 59652 was left running. Resume requires an explicit new guarded
  benchmark command after the server decision.
  Log: `/private/tmp/t11-f06-bench-gate.log`. The feature remains In
  Progress and unchecked; required benchmark evidence is not waived.
