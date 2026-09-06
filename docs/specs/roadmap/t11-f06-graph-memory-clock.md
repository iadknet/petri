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
- [x] Update affected reference/config/trace descriptions, T11.F05's current
      capability record (preserving its historical measurements), benchmark
      counter semantics, stored gate/goal reports, and progress row.
- [x] Complete reuse/simplification/efficiency self-review, fresh mutation
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
- [x] Store `make bench PROFILE=gate FEATURE=t11-f06-graph-memory-clock`
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
- `make roadmap-check` exited 0 after measured report/document updates in
  `/private/tmp/t11-f06-roadmap8.log`; `git diff --check` also exited 0.
- Guarded gate and single goal commands listed above both exited 0 against
  code revision `4c1b7727748665988acaebc88829b1dc2fe8320e`, with no competing
  local workloads during measurement. Logs: `/private/tmp/t11-f06-bench-gate2.log`
  and `/private/tmp/t11-f06-bench-goal1.log`. Both stored reports have
  `severe=false`; full comparisons and host-fingerprint limits follow below.

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

### Measured gate (T11.F06)

`make bench PROFILE=gate FEATURE=t11-f06-graph-memory-clock` exited 0 at
`4c1b7727748665988acaebc88829b1dc2fe8320e`; report
`docs/progress/features/t11-f06-graph-memory-clock.json`, log
`/private/tmp/t11-f06-bench-gate2.log`. The epoch baseline and previous closed
report are both T11.F04, so the harness deduplicates them into one reference.
No severe deterministic regression; thresholds and baselines are unchanged.

| Per creature-tick counter | T11.F04 | T11.F06 | Raw delta |
| --- | ---: | ---: | ---: |
| Mesh hops | 1.999541 | 1.999541 | 0% |
| VM steps | 28.044705 | 28.044705 | 0% |
| Graph counter | 2.998575 | 0.999541 | -66.666133% |
| Plasticity updates | 0.000901 | 0.000901 | 0% |
| Actions applied | 1.000000 | 1.000000 | 0% |
| Births | 0.001163 | 0.001163 | 0% |

The graph row crosses definitions (relaxation passes to entered nonempty
single-evaluation visits) and is not a like-for-like efficiency estimate.
Actual totals: 61,067 creature-ticks, 61,039 graph visits, 122,106 mesh hops,
1,712,606 VM steps, 55 plasticity updates, 61,067 applied actions, 71 births.
Applied graph charges are independently checked by the fixtures described
under Verification; the benchmark does not report cumulative graph energy.

Raw wall time is 0.004211063946 ms/creature-tick versus 0.004199411450 for
T11.F04 (about +0.277%). The harness correctly leaves its wall-clock comparison
null: stored hostnames differ (`Isaacs-MacBook-Pro-2.local` versus
`MacBookPro.lan`). CPU model (Apple M1 Pro), architecture and eight logical
cores agree, but identical-machine conditions are not verified. These raw
figures are descriptive across host fingerprints, not a controlled regression
finding. No metadata is rewritten to force comparability.

### Founder neighborhood comparison

The complete `neighborhood-v1` battery metadata, seeds, trial sizes, and
requested/applied birth denominators exactly match T11.F04. Every operator's
silent/changed/dead counts and fractions are unchanged, including all VM rows
and neutral graph growth. Add/copy compute node and input-reference Add retain
1.00 silence. Per birth: 500 total, 292 zero-event, 208 mutated; mutated
silent/changed/dead = 83/116/9 (0.399038/0.557692/0.043269), and single-event
164 with 72/86/6 (0.439024/0.524390/0.036585). Buckets 2–4 and their
classifications are unchanged. No floor is lowered or family reweighted.

Every changed founder companion component is listed here:

| Row/component | T11.F04 | T11.F06 | Attribution |
| --- | ---: | ---: | --- |
| CopyEdgeBundle, changed only in sequences | 0 | 5 | Recurrent copied edges now read prior-tick outputs, so five previously reactive differences require the sequence; D3 independently pins recurrence 1,2,3 rather than pass-capped 15. |
| CopyEdgeBundle, mean fraction differing | 0.075658 | 0.074013 | The same 38 changed subjects differ on fewer executions under that delayed recurrence; 12 remain silent, none dead. |
| Any-event births, mean fraction differing | 0.252100 | 0.252400 | Same 125 nonsilent subjects/208 applied births; only their execution-level signature differs under the clock and paid recomputation. |
| Single-event births, mean fraction differing | 0.225543 | 0.225951 | Same 92 nonsilent subjects/164 applied births; only execution-level signature differs. |

These are the predeclared recurrent-clock/energy-sensitive signature effects;
no silent/dead acceptance component decreases. D1/D2 and the separate
constructed controls validate the repaired clock without changing the
comparable neighborhood battery.

### Single measured goal and ecological impact

`make bench PROFILE=goal FEATURE=t11-f06-graph-memory-clock` exited 0 at
`4c1b7727748665988acaebc88829b1dc2fe8320e`; report
`docs/progress/features/t11-f06-graph-memory-clock-goal.json`, log
`/private/tmp/t11-f06-bench-goal1.log`. This is the only goal run. It retains
`goal-v1`: 1600×1600, 10,000 founders, default food coverage, seeds 11/22/33,
2,000 ticks. Epoch and previous report are both T11.F04. No severe regression.

| Per creature-tick counter | T11.F04 | T11.F06 | Raw delta |
| --- | ---: | ---: | ---: |
| mesh_hops | 3.280711 | 3.334148 | 1.628824% |
| vm_steps | 1237.540067 | 472.147369 | -61.847913% |
| graph_relax_iters | 6.002534 | 2.187397 | -63.558774% |
| plasticity_updates | 0.139997 | 0.153630 | 9.738066% |
| actions_applied | 1.129545 | 1.105299 | -2.146528% |
| births | 0.008151 | 0.008067 | -1.030548% |

The graph counter again crosses definitions. The other changes are actual
work under changed ecological trajectories: the largest difference is fewer
VM steps, while mesh hops and plasticity updates rise modestly. No baseline
or threshold is weakened. Raw wall time is 0.006681387433 ms/creature-tick
versus 0.007155270758 (-6.622857% descriptively), 664,219.982667 ms total.
As for gate, different stored host fingerprints make the controlled
wall-clock comparison null; these figures are not a same-host speedup claim.

All seeds remain alive through tick 2,000, with minimum population 10,000.
Cells below are T11.F04 → T11.F06; reductions remain visible.

| Seed | Final population | Births | Plateau population | Mean energy | Peak population (tick), T11.F06 |
| --- | ---: | ---: | ---: | ---: | --- |
| 11 | 11610 → 11024 | 267278 → 266793 | 12418.062000 → 11451.730000 | 69.611544 → 63.144385 | 32751 (140) |
| 22 | 10398 → 12745 | 268231 → 267557 | 11084.972000 → 12864.512000 | 62.908613 → 62.725554 | 32957 (146) |
| 33 | 11093 → 10499 | 261541 → 267595 | 11524.938000 → 10875.012000 | 65.286096 → 67.586979 | 32557 (132) |

Births per 100 world ticks rise 13,284.166667 → 13,365.750000 (+0.614140%),
even though births per creature-tick fall 1.030548%; the living population's
exposure denominator changed. Seed 11/33 final and plateau populations decline,
while seed 22 rises. This is the predeclared ecological consequence of changed
recurrent decisions and paid graph computation, not proof that memory improves
fitness. The report does not isolate the contribution of each mechanism.

Historical persistence context is profile-specific. T01.F11's **gate** has
75 ticks, 256 founders and 128×128 cells, so its final populations 284/260/264
and births 35/18/21 must not be compared as a 2,000-tick horizon. Its stored
**w1600 sweep** does match the goal's world/founders/seeds/horizon, and its
per-seed counts match the later T01.F12 **goal** baseline: final populations
5,291/10,997/8,130, births 134,117/179,125/166,410, no extinction. T11.F06
exceeds those historical long-horizon final populations and births on every
seed, but the direct feature comparison is T11.F04 above; intervening repairs
prevent attributing the full historical gain to this clock feature.

Structure min/p25/median/p75/max/mean is
1/96/100/118/531/115.767042, versus T11.F04's
1/96/100/118/523/115.430531 and T01.F12's
1/95/97/124/798/116.771071. Similar pooled medians do not imply identical
sampled controllers. Lineage observations are:

| Seed | Surviving founder clades, T11.F04 → T11.F06 | Shannon entropy nats, T11.F04 → T11.F06 |
| --- | ---: | ---: |
| 11 | 180 → 200 | 4.218733 → 4.262014 |
| 22 | 181 → 199 | 4.248631 → 4.119850 |
| 33 | 208 → 191 | 4.243092 → 4.126717 |

Seed 22/33 entropy declines, and seed 33 loses clades relative to T11.F04;
these are retained observations of the changed populations. T01.F12's matching
goal baseline had 142/144/140 clades and entropy 2.780730/2.947968/2.668014.

### Current and temporal memory observations

The preserved current-memory probe's either-intervention counts are
5/0/7 of 11,024/12,745/10,499 (fractions 0.000454/0.000000/0.000667), versus
T11.F04's 5/0/0 of 11,610/10,398/11,093 (0.000431/0/0). Zeroing changes
0/0/1 actions and rotation changes 5/0/7 in T11.F06. T01.F12's goal current
probe changed none of its 24,418 final creatures.

The separately versioned `temporal-memory-v1` reports the following full
population counts. Zero/rotation/either are per-substrate comparisons with the
same intact full action queue; either is a union, not a sum. Substrates may
overlap with one another and their counts must not be summed into a population
union. Historical temporal components load as `Undefined`, not measured zero.

| Seed | Substrate | Zero | Rotation | Either | Either fraction |
| --- | --- | ---: | ---: | ---: | ---: |
| 11 | previous_slots | 0 | 0 | 0 | 0.000000 |
| 11 | persisted_outputs | 5 | 11 | 15 | 0.001361 |
| 11 | operator_state | 12 | 12 | 12 | 0.001089 |
| 22 | previous_slots | 0 | 0 | 0 | 0.000000 |
| 22 | persisted_outputs | 9 | 16 | 22 | 0.001726 |
| 22 | operator_state | 14 | 14 | 14 | 0.001098 |
| 33 | previous_slots | 0 | 0 | 0 | 0.000000 |
| 33 | persisted_outputs | 14 | 13 | 27 | 0.002572 |
| 33 | operator_state | 33 | 33 | 33 | 0.003143 |

This establishes observable action sensitivity to graph memory in rare final
creatures; it does not establish useful memory, learning, or causal fitness
benefit. Previous-slot zero sensitivity is this observation's reading, not
proof that previous-slot capability is absent (its constructed control is
positive). Final sensors and current/previous slots are held fixed, cloned
graph snapshots are prepared before intervention, and no live world/RNG is
changed. Other deferred cognition indicators remain `Undefined`.

### Evolved neighborhood comparison: unpaired cohorts

Both reports sample 12 genomes per seed by fixed rank rule and run 20 trials
per operator and 200 births per genome. Every pooled birth denominator remains
2,400 total, 1,300 zero-event, 1,100 applied mutated births, with event buckets
1/2/3/4/5 containing 912/153/29/4/2. The battery and all seed/size metadata are
unchanged. Each pooled operator has 240 requested trials; skips depend on the
sampled genome and its eligible structure.

**These cohorts are unpaired.** Their final population sizes and sampled
ranks/genomes differ; rank position does not preserve genotype identity. Lower
aggregate silence therefore cannot be assigned to a same-genome clock effect
from these two reports alone. The predeclared recurrent/energy mechanisms can
change selection and the resulting samples, while unchanged founder tallies
provide the fixed-genome comparison. The tables below retain all declines,
including VM-only operator rows: unlike the fixed founder, those rows operate
on different evolved VM genomes. No claim of direct causal localization is
made, and no additional goal run is used to select a better result.

| Seed | Any-event silent/changed/dead, T11.F04 → T11.F06 | Single-event silent/changed/dead, T11.F04 → T11.F06 |
| --- | --- | --- |
| 11 | 0.600000/0.365455/0.034545 → 0.531818/0.429091/0.039091 | 0.635965/0.341009/0.023026 → 0.581140/0.391447/0.027412 |
| 22 | 0.618182/0.343636/0.038182 → 0.587273/0.380909/0.031818 | 0.645833/0.325658/0.028509 → 0.625000/0.350877/0.024123 |
| 33 | 0.612727/0.362727/0.024545 → 0.603636/0.358182/0.038182 | 0.657895/0.324561/0.017544 → 0.645833/0.326754/0.027412 |

Any-event and single-event silence decline on all three unpaired cohorts;
dead fractions rise on seeds 11/33 and fall on seed 22. These observations are
not hidden by the unchanged founder result. Concrete cohort differences:

| Seed | Reads/writes shared memory, T11.F04 → T11.F06 | Stateful/plastic samples, T11.F04 → T11.F06 | Sampled complexity vectors, T11.F04 → T11.F06 |
| --- | --- | --- | --- |
| 11 | 2/5 → 6/4 | 0/0 → 0/0 | 102,110,95,110,91,105,95,95,263,99,228,264 → 98,99,102,103,104,104,95,97,90,117,181,93 |
| 22 | 4/4 → 6/4 | 1/1 → 0/0 | 91,91,102,147,118,96,103,101,91,61,118,105 → 104,286,161,103,98,95,98,95,195,96,93,101 |
| 33 | 3/2 → 3/2 | 0/0 → 0/1 | 95,114,154,106,121,90,101,105,214,170,117,216 → 92,97,97,151,93,96,156,211,108,112,146,97 |

None of the 36 sampled genomes has a stateful operator, despite the positive
whole-population operator-state sensitivity counts. This is a bounded sampling
limit, not a contradiction or proof of absence. The sampled genomes are only a
small subset of the final population.

Every changed pooled evolved operator row is compared below (39/45/43 rows
for seeds 11/22/33). `A` is applied count; `S/C/D` are silent/changed/dead counts
so each fraction is recoverable against `A`; skipped count is `240-A`.
`Q` is changed-only-in-sequences; `M` is mean fraction differing. Unchanged rows
are omitted. All changed counts, denominators, sequence companions and means
are retained; full per-genome rows remain in the linked reports.

| Seed | Operator | A, before → after | S/C/D, before → after | Q, before → after | M, before → after |
| --- | --- | --- | --- | --- | --- |
| 11 | VmConstantMutation | 240 → 240 | 154/86/0 → 144/96/0 | 0 → 0 | 0.135320 → 0.139323 |
| 11 | VmInstructionMutation | 240 → 240 | 153/87/0 → 134/105/1 | 2 → 4 | 0.157759 → 0.203302 |
| 11 | VmDeleteInstruction | 228 → 240 | 119/109/0 → 92/148/0 | 3 → 10 | 0.173853 → 0.150845 |
| 11 | VmRegisterCountMutation | 194 → 220 | 194/0/0 → 220/0/0 | 0 → 0 | 0.000000 → 0.000000 |
| 11 | VmInstructionRawFieldMutation | 228 → 240 | 128/100/0 → 111/128/1 | 2 → 8 | 0.154375 → 0.139341 |
| 11 | VmCopyInstructionBlock | 240 → 240 | 136/103/1 → 110/128/2 | 3 → 2 | 0.339062 → 0.332885 |
| 11 | VmCopyInstructionBlockRemapped | 240 → 240 | 127/111/2 → 92/147/1 | 2 → 7 | 0.382412 → 0.301098 |
| 11 | VmCopyConstantBlock | 233 → 240 | 233/0/0 → 240/0/0 | 0 → 0 | 0.000000 → 0.000000 |
| 11 | VmCopyGeneBackwardSlice | 228 → 240 | 132/87/9 → 129/103/8 | 2 → 4 | 0.377344 → 0.394707 |
| 11 | VmCopyGeneForwardSlice | 228 → 240 | 144/84/0 → 133/107/0 | 1 → 3 | 0.352530 → 0.330841 |
| 11 | VmInsertReadStoreMotif | 228 → 240 | 200/28/0 → 204/36/0 | 1 → 1 | 0.150446 → 0.126042 |
| 11 | VmInsertReadBidMotif | 228 → 240 | 224/4/0 → 230/10/0 | 0 → 0 | 0.187500 → 0.128750 |
| 11 | VmInsertLoadCompareMotif | 240 → 240 | 216/24/0 → 203/37/0 | 2 → 2 | 0.196875 → 0.109459 |
| 11 | VmMutateSlotAddress | 86 → 148 | 86/0/0 → 148/0/0 | 0 → 0 | 0.000000 → 0.000000 |
| 11 | AlterGraphEdgeWeight | 215 → 240 | 184/31/0 → 192/48/0 | 8 → 11 | 0.037903 → 0.031510 |
| 11 | SwapGraphOperator | 215 → 240 | 92/123/0 → 74/166/0 | 1 → 6 | 0.162297 → 0.199172 |
| 11 | MutateGraphOperatorParam | 215 → 240 | 197/18/0 → 195/45/0 | 0 → 0 | 0.012500 → 0.020833 |
| 11 | AddInternalGraphNode | 230 → 240 | 230/0/0 → 240/0/0 | 0 → 0 | 0.000000 → 0.000000 |
| 11 | RemoveInternalGraphNode | 215 → 240 | 73/142/0 → 32/208/0 | 0 → 0 | 0.124208 → 0.133954 |
| 11 | AddGraphEdge | 240 → 240 | 221/19/0 → 213/27/0 | 0 → 1 | 0.162500 → 0.158333 |
| 11 | RetargetGraphEdge | 215 → 240 | 82/133/0 → 48/192/0 | 1 → 6 | 0.141071 → 0.145508 |
| 11 | RemoveGraphEdge | 215 → 240 | 80/135/0 → 46/194/0 | 1 → 6 | 0.141111 → 0.120361 |
| 11 | GraphRawFieldMutation | 214 → 240 | 87/127/0 → 62/178/0 | 3 → 8 | 0.142323 → 0.162711 |
| 11 | CopyInternalNode | 215 → 240 | 215/0/0 → 240/0/0 | 0 → 0 | 0.000000 → 0.000000 |
| 11 | CopySubgraph | 215 → 240 | 215/0/0 → 240/0/0 | 0 → 0 | 0.000000 → 0.000000 |
| 11 | CopyEdgeBundle | 213 → 240 | 132/81/0 → 110/130/0 | 1 → 17 | 0.103241 → 0.103173 |
| 11 | EnableHebbian | 215 → 240 | 154/61/0 → 173/67/0 | 61 → 67 | 0.027254 → 0.025000 |
| 11 | RemoveNode | 240 → 240 | 71/0/169 → 12/0/228 | 0 → 0 | 1.000000 → 0.993421 |
| 11 | RetargetNodeTarget | 240 → 240 | 144/7/89 → 136/2/102 | 0 → 0 | 0.943880 → 0.975000 |
| 11 | RemoveRouteTarget | 240 → 240 | 72/4/164 → 75/5/160 | 0 → 0 | 0.977083 → 0.963636 |
| 11 | ChangeEntryNode | 240 → 240 | 0/196/44 → 0/240/0 | 0 → 0 | 0.847812 → 0.840521 |
| 11 | SwapNodeBackend | 240 → 240 | 60/96/84 → 10/114/116 | 0 → 0 | 0.910069 → 0.929130 |
| 11 | CopyNode | 240 → 240 | 240/0/0 → 205/17/18 | 0 → 0 | 0.000000 → 0.946786 |
| 11 | SpliceNode | 240 → 240 | 230/10/0 → 228/12/0 | 0 → 1 | 0.238750 → 0.204167 |
| 11 | SwapRouteTargets | 80 → 80 | 36/10/34 → 60/20/0 | 0 → 0 | 0.784659 → 0.100000 |
| 11 | MutateGateBias | 240 → 240 | 221/5/14 → 234/6/0 | 0 → 0 | 0.746711 → 0.100000 |
| 11 | Remove | 240 → 240 | 79/161/0 → 66/174/0 | 3 → 1 | 0.157919 → 0.154741 |
| 11 | Swap | 240 → 240 | 80/160/0 → 60/180/0 | 2 → 1 | 0.194141 → 0.189722 |
| 11 | RawFieldMutation | 240 → 240 | 62/178/0 → 55/185/0 | 6 → 1 | 0.170857 → 0.182838 |
| 22 | VmConstantMutation | 240 → 240 | 146/94/0 → 145/95/0 | 0 → 0 | 0.161835 → 0.146579 |
| 22 | VmInstructionMutation | 240 → 240 | 145/95/0 → 125/113/2 | 2 → 1 | 0.231447 → 0.187826 |
| 22 | VmDeleteInstruction | 222 → 240 | 109/113/0 → 102/138/0 | 4 → 7 | 0.189712 → 0.152808 |
| 22 | VmRegisterCountMutation | 216 → 221 | 216/0/0 → 221/0/0 | 0 → 0 | 0.000000 → 0.000000 |
| 22 | VmInstructionRawFieldMutation | 221 → 240 | 111/110/0 → 106/134/0 | 2 → 6 | 0.171364 → 0.149534 |
| 22 | VmCopyInstructionBlock | 240 → 240 | 125/115/0 → 112/128/0 | 0 → 2 | 0.403370 → 0.340918 |
| 22 | VmCopyInstructionBlockRemapped | 240 → 240 | 118/121/1 → 104/135/1 | 2 → 2 | 0.349385 → 0.330790 |
| 22 | VmCopyConstantBlock | 221 → 240 | 221/0/0 → 240/0/0 | 0 → 0 | 0.000000 → 0.000000 |
| 22 | VmCopyGeneBackwardSlice | 221 → 240 | 120/93/8 → 129/100/11 | 1 → 4 | 0.385767 → 0.371284 |
| 22 | VmCopyGeneForwardSlice | 221 → 240 | 118/102/1 → 128/112/0 | 0 → 2 | 0.329369 → 0.326562 |
| 22 | VmInsertReadStoreMotif | 230 → 240 | 205/25/0 → 208/32/0 | 4 → 0 | 0.140500 → 0.145313 |
| 22 | VmInsertReadBidMotif | 230 → 240 | 221/9/0 → 235/5/0 | 0 → 0 | 0.179167 → 0.102500 |
| 22 | VmInsertLoadCompareMotif | 240 → 240 | 210/30/0 → 201/39/0 | 1 → 1 | 0.157083 → 0.126923 |
| 22 | VmMutateSlotAddress | 111 → 180 | 111/0/0 → 180/0/0 | 0 → 0 | 0.000000 → 0.000000 |
| 22 | AlterGraphEdgeWeight | 240 → 231 | 204/36/0 → 203/28/0 | 13 → 2 | 0.033333 → 0.040625 |
| 22 | SwapGraphOperator | 240 → 231 | 96/144/0 → 100/131/0 | 7 → 2 | 0.184809 → 0.172424 |
| 22 | MutateGraphOperatorParam | 240 → 222 | 217/23/0 → 205/17/0 | 0 → 0 | 0.017935 → 0.012500 |
| 22 | AddInternalGraphNode | 240 → 239 | 240/0/0 → 239/0/0 | 0 → 0 | 0.000000 → 0.000000 |
| 22 | RemoveInternalGraphNode | 240 → 231 | 80/160/0 → 71/160/0 | 0 → 0 | 0.126953 → 0.115156 |
| 22 | AddGraphEdge | 240 → 240 | 221/19/0 → 207/33/0 | 1 → 3 | 0.139474 → 0.140909 |
| 22 | RetargetGraphEdge | 240 → 231 | 99/141/0 → 89/142/0 | 4 → 4 | 0.164805 → 0.150264 |
| 22 | RemoveGraphEdge | 240 → 231 | 98/142/0 → 86/145/0 | 4 → 0 | 0.132570 → 0.115948 |
| 22 | GraphRawFieldMutation | 239 → 220 | 102/137/0 → 83/137/0 | 2 → 0 | 0.168157 → 0.159033 |
| 22 | CopyInternalNode | 240 → 231 | 240/0/0 → 231/0/0 | 0 → 0 | 0.000000 → 0.000000 |
| 22 | CopySubgraph | 240 → 213 | 240/0/0 → 213/0/0 | 0 → 0 | 0.000000 → 0.000000 |
| 22 | CopyEdgeBundle | 237 → 213 | 134/103/0 → 114/99/0 | 1 → 6 | 0.127427 → 0.104545 |
| 22 | EnableHebbian | 240 → 231 | 179/61/0 → 160/71/0 | 61 → 71 | 0.021926 → 0.025352 |
| 22 | DisableHebbian | 20 → 0 | 20/0/0 → 0/0/0 | 0 → 0 | 0.000000 → 0.000000 |
| 22 | MutateHebbianRule | 20 → 0 | 20/0/0 → 0/0/0 | 0 → 0 | 0.000000 → 0.000000 |
| 22 | MutateHebbianRate | 20 → 0 | 20/0/0 → 0/0/0 | 0 → 0 | 0.000000 → 0.000000 |
| 22 | ToggleHebbianLamarckian | 20 → 0 | 20/0/0 → 0/0/0 | 0 → 0 | 0.000000 → 0.000000 |
| 22 | EnableRewardModulation | 20 → 0 | 20/0/0 → 0/0/0 | 0 → 0 | 0.000000 → 0.000000 |
| 22 | RemoveNode | 240 → 240 | 38/0/202 → 67/0/173 | 0 → 0 | 1.000000 → 1.000000 |
| 22 | RetargetNodeTarget | 240 → 240 | 143/2/95 → 149/3/88 | 0 → 0 | 0.996649 → 0.970330 |
| 22 | RemoveRouteTarget | 240 → 240 | 63/0/177 → 84/1/155 | 0 → 0 | 1.000000 → 0.995192 |
| 22 | ChangeEntryNode | 240 → 240 | 19/210/11 → 25/200/15 | 0 → 0 | 0.853111 → 0.861802 |
| 22 | SwapNodeBackend | 240 → 240 | 32/99/109 → 63/95/82 | 0 → 0 | 0.922175 → 0.908121 |
| 22 | CopyNode | 240 → 240 | 203/11/26 → 220/6/14 | 0 → 0 | 0.974662 → 0.938750 |
| 22 | CopyMeshBackwardSlice | 240 → 240 | 238/0/2 → 239/0/1 | 0 → 0 | 1.000000 → 1.000000 |
| 22 | CopyMeshForwardSlice | 240 → 240 | 238/0/2 → 240/0/0 | 0 → 0 | 1.000000 → 0.000000 |
| 22 | SpliceNode | 240 → 240 | 229/11/0 → 229/11/0 | 1 → 0 | 0.319318 → 0.360227 |
| 22 | SwapRouteTargets | 40 → 60 | 40/0/0 → 38/2/20 | 0 → 0 | 0.000000 → 0.931818 |
| 22 | Remove | 240 → 240 | 89/151/0 → 70/170/0 | 1 → 1 | 0.175579 → 0.140515 |
| 22 | Swap | 240 → 240 | 86/154/0 → 76/164/0 | 2 → 0 | 0.212338 → 0.180640 |
| 22 | RawFieldMutation | 240 → 240 | 77/163/0 → 52/188/0 | 2 → 3 | 0.182745 → 0.177527 |
| 33 | VmConstantMutation | 240 → 240 | 156/84/0 → 134/106/0 | 0 → 0 | 0.124107 → 0.136321 |
| 33 | VmInstructionMutation | 240 → 240 | 150/90/0 → 153/87/0 | 6 → 2 | 0.179583 → 0.221839 |
| 33 | VmDeleteInstruction | 240 → 240 | 132/108/0 → 118/122/0 | 8 → 5 | 0.164815 → 0.156865 |
| 33 | VmRegisterCountMutation | 221 → 240 | 221/0/0 → 240/0/0 | 0 → 0 | 0.000000 → 0.000000 |
| 33 | VmInstructionRawFieldMutation | 240 → 240 | 141/99/0 → 127/113/0 | 5 → 2 | 0.138889 → 0.175553 |
| 33 | VmCopyInstructionBlock | 240 → 240 | 135/104/1 → 130/107/3 | 1 → 1 | 0.353095 → 0.375341 |
| 33 | VmCopyInstructionBlockRemapped | 240 → 240 | 131/107/2 → 123/116/1 | 3 → 1 | 0.364335 → 0.342842 |
| 33 | VmCopyGeneBackwardSlice | 240 → 240 | 146/87/7 → 147/88/5 | 1 → 2 | 0.317819 → 0.409812 |
| 33 | VmCopyGeneForwardSlice | 240 → 240 | 156/83/1 → 140/100/0 | 2 → 2 | 0.290327 → 0.326375 |
| 33 | VmInsertReadStoreMotif | 240 → 240 | 219/21/0 → 213/27/0 | 0 → 2 | 0.139881 → 0.127778 |
| 33 | VmInsertReadBidMotif | 240 → 240 | 233/7/0 → 234/6/0 | 0 → 0 | 0.100000 → 0.133333 |
| 33 | VmInsertLoadCompareMotif | 240 → 240 | 209/31/0 → 208/32/0 | 1 → 1 | 0.168548 → 0.126172 |
| 33 | AlterGraphEdgeWeight | 236 → 240 | 191/45/0 → 209/31/0 | 15 → 9 | 0.023333 → 0.032258 |
| 33 | SwapGraphOperator | 232 → 231 | 87/145/0 → 89/142/0 | 10 → 3 | 0.184310 → 0.219542 |
| 33 | MutateGraphOperatorParam | 220 → 215 | 190/30/0 → 178/37/0 | 0 → 0 | 0.013750 → 0.025676 |
| 33 | AddInternalGraphNode | 238 → 240 | 238/0/0 → 240/0/0 | 0 → 0 | 0.000000 → 0.000000 |
| 33 | RemoveInternalGraphNode | 232 → 231 | 35/197/0 → 63/168/0 | 0 → 0 | 0.130584 → 0.125446 |
| 33 | AddGraphEdge | 240 → 240 | 215/25/0 → 220/20/0 | 3 → 1 | 0.163000 → 0.188750 |
| 33 | RetargetGraphEdge | 236 → 240 | 78/158/0 → 85/155/0 | 1 → 2 | 0.131646 → 0.161210 |
| 33 | RemoveGraphEdge | 236 → 240 | 63/173/0 → 88/152/0 | 4 → 4 | 0.122688 → 0.136678 |
| 33 | GraphRawFieldMutation | 221 → 220 | 80/141/0 → 77/143/0 | 0 → 3 | 0.148050 → 0.171241 |
| 33 | CopyInternalNode | 232 → 231 | 232/0/0 → 231/0/0 | 0 → 0 | 0.000000 → 0.000000 |
| 33 | CopyEdgeBundle | 213 → 215 | 99/114/0 → 111/104/0 | 0 → 15 | 0.107675 → 0.098798 |
| 33 | EnableHebbian | 232 → 226 | 166/66/0 → 164/62/0 | 66 → 62 | 0.022348 → 0.027621 |
| 33 | DisableHebbian | 0 → 25 | 0/0/0 → 25/0/0 | 0 → 0 | 0.000000 → 0.000000 |
| 33 | MutateHebbianRule | 0 → 25 | 0/0/0 → 25/0/0 | 0 → 0 | 0.000000 → 0.000000 |
| 33 | MutateHebbianRate | 0 → 25 | 0/0/0 → 25/0/0 | 0 → 0 | 0.000000 → 0.000000 |
| 33 | ToggleHebbianLamarckian | 0 → 25 | 0/0/0 → 25/0/0 | 0 → 0 | 0.000000 → 0.000000 |
| 33 | EnableRewardModulation | 0 → 25 | 0/0/0 → 25/0/0 | 0 → 0 | 0.000000 → 0.000000 |
| 33 | RemoveNode | 240 → 240 | 56/0/184 → 50/0/190 | 0 → 0 | 1.000000 → 1.000000 |
| 33 | RetargetNodeTarget | 240 → 240 | 146/14/80 → 146/7/87 | 0 → 0 | 0.891090 → 0.944947 |
| 33 | RemoveRouteTarget | 240 → 240 | 78/0/162 → 82/0/158 | 0 → 0 | 1.000000 → 1.000000 |
| 33 | ChangeEntryNode | 240 → 240 | 5/205/30 → 21/207/12 | 0 → 0 | 0.873245 → 0.825342 |
| 33 | SwapNodeBackend | 240 → 240 | 45/101/94 → 69/87/84 | 0 → 0 | 0.860385 → 0.910673 |
| 33 | CopyNode | 240 → 240 | 237/0/3 → 218/7/15 | 0 → 0 | 1.000000 → 0.906250 |
| 33 | CopyMeshBackwardSlice | 240 → 240 | 240/0/0 → 235/2/3 | 0 → 0 | 0.000000 → 0.715000 |
| 33 | CopyMeshForwardSlice | 240 → 240 | 238/2/0 → 235/2/3 | 0 → 0 | 0.225000 → 0.715000 |
| 33 | SpliceNode | 240 → 240 | 224/16/0 → 231/9/0 | 0 → 1 | 0.224219 → 0.204167 |
| 33 | SwapRouteTargets | 80 → 60 | 80/0/0 → 40/0/20 | 0 → 0 | 0.000000 → 1.000000 |
| 33 | Add | 240 → 240 | 240/0/0 → 237/3/0 | 0 → 0 | 0.000000 → 0.179167 |
| 33 | Remove | 240 → 240 | 99/141/0 → 90/150/0 | 3 → 0 | 0.155408 → 0.143500 |
| 33 | Swap | 240 → 240 | 93/147/0 → 87/153/0 | 3 → 1 | 0.180867 → 0.179085 |
| 33 | RawFieldMutation | 240 → 240 | 87/153/0 → 88/152/0 | 3 → 1 | 0.181699 → 0.167352 |

Remaining evolved birth-bucket details (buckets 1 and any-event summarized
above; all requested/applied counts remain fixed):

| Seed | Events | Applied | S/C/D, before → after | Q, before → after | M, before → after |
| --- | ---: | ---: | --- | --- | --- |
| 11 | 2 | 153 | 75/66/12 → 49/91/13 | 2 → 1 | 0.336699 → 0.332572 |
| 11 | 3 | 29 | 4/20/5 → 5/19/5 | 0 → 1 | 0.427000 → 0.403125 |
| 11 | 4 | 4 | 0/4/0 → 1/3/0 | 1 → 0 | 0.209375 → 0.137500 |
| 11 | 5 | 2 | 1/1/0 → 0/2/0 | 0 → 0 | 0.037500 → 0.518750 |
| 22 | 2 | 153 | 75/67/11 → 66/77/10 | 0 → 0 | 0.378846 → 0.344540 |
| 22 | 3 | 29 | 14/11/4 → 9/17/3 | 0 → 0 | 0.517500 → 0.397500 |
| 22 | 4 | 4 | 1/3/0 → 0/4/0 | 0 → 1 | 0.220833 → 0.218750 |
| 22 | 5 | 2 | 1/0/1 → 1/1/0 | 0 → 0 | 1.000000 → 0.100000 |
| 33 | 2 | 153 | 70/75/8 → 64/75/14 | 0 → 3 | 0.293373 → 0.344944 |
| 33 | 3 | 29 | 2/24/3 → 8/18/3 | 0 → 0 | 0.292130 → 0.348214 |
| 33 | 4 | 4 | 2/2/0 → 2/2/0 | 0 → 0 | 0.375000 → 0.493750 |
| 33 | 5 | 2 | 0/2/0 → 1/1/0 | 0 → 0 | 0.106250 → 0.937500 |

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
  Performance conclusions were pending at that consultation; measured results are
  recorded above. Final review and orchestrator check remain pending.
- Concrete measurement blocker (2026-09-06): the required measured gate's
  `scripts/bench-wait` was held by PID 59652, `target/release/v3-server`,
  parent PID 1, cwd `/Users/istefanek/projects/petri`, started 00:14:03 local.
  At that pause no measured gate or goal report had been produced. The orchestrator asked
  the user whether to stop that exact server, which may hold their simulation;
  approval was pending at that pause and the implementer did not stop it or bypass the
  guard. Before pausing, only this task's waiting guard (PID 84202) was
  stopped; its make command exited 2 without starting measurement, while
  server PID 59652 was left running. Resumption subsequently used a new guarded
  benchmark command after the server decision, as recorded below.
  Log: `/private/tmp/t11-f06-bench-gate.log`. The feature remains In
  Progress and unchecked; required benchmark evidence is not waived.

- Measurement blocker resolved (2026-09-06): the user explicitly authorized
  stopping the identified main-checkout server. The orchestrator verified its
  identity, sent TERM, and confirmed PID 59652 absent. A new guarded gate ran
  successfully; the single goal run also completed successfully. The earlier paused attempt
  performed no measurement and is not counted as a goal run.
