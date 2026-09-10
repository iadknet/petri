# T13.F01 — Module Recruitment Observability

**Status**: In Progress
**Last updated**: 2026-09-10
**Feature**: T13.F01
**Track**: [T13 — Neutral Module Recruitment](../../roadmaps/t13-neutral-module-recruitment.md)

This spec is the contract, not the record. Measured evidence lives in
`docs/progress/features/t13-f01-module-recruitment-observability{,-goal}.json`
and `docs/progress/readings/t13-f01-module-recruitment-observability.md`.

## Goal

Every goal closure shows, for the brain modules that appear along the existing
mutation-only drift walk, where recruitment advances or stalls: how many new
modules were created, whether mutation ever selected them, whether a selected
event found an applicable site, whether the module changed internally, whether
the fixed battery dispatched it, whether bypassing it changed the battery's
actions, and whether it was deleted or retained. The same walk reports, per
lineage, the mutation opportunities production actually offered by operator and
domain, so mutation starvation of the executed core is distinguishable from
silent scaffold before any substrate change.

## Non-Goals

- No change to mutation semantics, target applicability (T13.F03), Graph
  effect activation (T13.F04), recruitment paths (T13.F05), usefulness or
  retention under selection (T13.F02, T13.F06), or supply policy (T11.F13).
- No per-node output/state effect trace, no measure of usefulness, no named
  task, no ecological cohort tracking of the goal worlds' living populations,
  and no production genealogy: cohort identity lives in the drift harness only.
- No new dashboard, database, profile, campaign runner, CLI command, or
  `docs/progress/index.html` change. No frontend change.
- No change to the walk's seeds, sizes, checkpoints, refresh cadence, battery,
  knockout method, existing drift fields, or the standing depth floors.

## Inputs and Invariants

- Sources: the owning track's T13.F01 row, its "F01 scope" and "F01 semantics
  and limits" notes; [T11.F14](t11-f14-mesh-execution-observability.md)
  (`mesh-execution-v1`, `static-successor-bypass-v1`);
  [T11.F16](t11-f16-drift-depth-indicator.md) and T11.F17 (`drift-depth-v2`
  walk: 50 lineages, checkpoints 0/22/250/1,000/2,000, executed-set refresh
  every 10 generations, 20 birth lineages × 100 births); T11.F18's additive
  `MeshBackendCounts`; the [recruitment research note](../../strategy/neutral-module-recruitment-research-2026-09-08.md)
  (inspected `NoApplicableTarget` discard and zero-compute Graph early return).
  Dependencies live in the track row.
- Seams: `MutationEngine::apply_mutations_with_food_type_count` and its
  `MutationSummary` (already carries attempted/applied/skipped events, per
  operator funnels, skip reasons, domain tallies, reachable/executed target
  counts); `mutation::reachability::TargetSelector::select`;
  `neighborhood::drift::{observe, observe_checkpoint, refresh_executed_ids,
  MeshTotals, Checkpoint}`; `Battery::{executed_node_ids, mesh_execution}`;
  `DriftDepth`/`DriftDepthCheckpoint` and `drift_checkpoint` in
  `crates/v3-cli/src/bench.rs`; `docs/progress.md`. Runtime, mutation, and
  simulation must not depend on neighborhood or CLI types.
- Research decision, 2026-09-10: observe recruitment inside the existing drift
  walk rather than a new lineage tracer or an ecological genealogy. The walk
  already applies the production engine once per generation per lineage,
  refreshes each lineage's executed node ids every 10 generations, and runs the
  knockout battery at every checkpoint, so every cohort fact below is derivable
  from records the walk already produces or from the engine's own summary. The
  engine's summary discards which node each event selected; surfacing it is the
  one engine-side addition and it is the fact T13.F03 needs. The alternative of
  inferring selection from genome diffs cannot see a selected-but-inapplicable
  event at all.

**Engine event records (additive, deterministic, no RNG).** `MutationSummary`
gains `events: Vec<MutationEventRecord>`, one record per attempted event in
draw order: the `MutationDomain`, the `MutationOperator`, the `NodeId` of the
first node the event's `TargetSelector` returned (`None` when the operator
skipped before any selection, e.g. no node of the required backend exists), and
the outcome: applied with its `TargetReachability`, or skipped with its
`MutationSkipReason`. `TargetSelector` remembers its first pick; recording reads
it after the operator returns and translates the pre-event index to the id the
genome carried before that event. Recording consumes no RNG, changes no
selection, and leaves every existing summary field and production genome
byte-identical. `events.len() == attempted_events`, and the applied/skipped
records agree with `applied_events`/`skipped_events` and with each operator's
funnel. This runs on every production birth; its cost is one small vector per
birth that draws events.

**Opportunity accounting (per lineage, cumulative).** The walk pools each
lineage's per-generation summaries from depth 0 to each checkpoint:
births, zero-event births, attempted/applied/skipped events; per operator
attempted, applied, and skipped by reason; per domain (Topology/Vm/Graph/
InputRef) attempted and applied; reachable-, unreachable-, and executed-target
events; and, from the event records, per operator the split of skipped events
into *selected a target but found no applicable site* (record has a target and
`NoApplicableTarget`) and *no eligible node* (no target). Report pooled totals
across the 50 lineages per checkpoint, and per-lineage rows carrying births,
attempted, applied, skipped, selected-inapplicable, and applied by domain.
Nothing is inferred from a checkpoint's fresh births; those keep their existing
`BirthResult` role.

**Module identity and provenance.** A module is one mesh node in one lineage,
identified by `(lineage, NodeId, creation depth)`. Founder nodes are created at
depth 0 with provenance `founder`. After each generation's birth the harness
compares the lineage's node id set before and after: ids that vanished are
`deleted` at that depth; ids that appeared are new modules with the backend
they carry at creation and provenance `copy` when that birth applied at least
one `CopyNode` event and the new node's `backend_def` equals some pre-birth
node's, otherwise `new`. Because `next_node_id` reuses the lowest free id above
the current maximum, an id that vanishes and reappears is two distinct
modules; the id set diff is the whole identity rule, so an id present both
before and after a birth is the same module whatever its content did.
No cross-lineage identity exists. The harness stores only this table; it is
bounded by nodes created along the walk, never a production genealogy.

**Cohort facts (separate, never merged).** For each module the harness records
the depth of: creation; first *selection* (an event record naming its id, any
outcome); first *applicable selection* (an applied event naming its id); first
*internal change* (its `NodeGenome` differs from the previous generation's node
with the same id, including input refs, targets, and backend content); first
*dispatch* (its id is in the lineage's executed set at an executed-set refresh
or checkpoint reading, so resolution is the existing 10-generation cadence plus
checkpoints); first *battery contribution* (at a checkpoint, its id is executed
and its static-successor bypass changes the complete `Signature`, i.e. the
existing knockout loop's non-identical set, which `mesh_execution` now also
returns as node id sets); and deletion. Dispatch is not an effect, a
contribution is battery sensitivity only, and usefulness is unmeasured here;
the reading names those limits. Modules deleted before reaching a fact and
modules present at the end of observation without it are reported as separate
censoring counts, never dropped from the denominator.

**Cohort readings per checkpoint.** Over all modules with provenance `new` or
`copy` created at or before the checkpoint, split by creation backend
(Graph/VM) and pooled: created, deleted by now, present; among present, the
exclusive state ladder *never selected*, *selected only* (no applied event),
*applied only* (an applied event but no internal change, e.g. a route slot
write on another node counts for that node, not this one), *changed only*,
*dispatched not contributing* (dispatched at the latest executed reading at or
before this checkpoint), *contributing* (this checkpoint's knockout); a module
sits in the highest rung it has reached. Founder modules get the same present/
deleted/contributing counts as a reference row. Time-to-first readings: for
each fact, the count reached, the integer median generations from creation
among those (upper-middle rule for even counts), and the two censoring counts.
Retention between consecutive checkpoints: of modules contributing at the
earlier one, how many are contributing, present but not contributing, or
deleted at the later one. Per-lineage rows at each checkpoint carry created,
present, dispatched, and contributing counts for `new`+`copy` modules.
Invariants: created = present + deleted; the ladder partitions present;
contributing ⊆ dispatched ⊆ present; per-lineage sums equal pooled totals;
backend splits sum to totals; every count is an integer and the reading is
identical across thread counts and processes.

**Isolation.** Observation reads genomes and summaries the walk already
produces, consumes no walk or birth RNG, changes no lineage genome, adds no
battery execution beyond the existing refresh and checkpoint readings, and
leaves every existing `drift-depth-v2` field, checkpoint birth result, mesh
total, and the walk's genome sequence byte-identical. The version string
becomes `drift-depth-v3`; the new blocks are serde-defaulted so historical
reports load as `Undefined`/absent, never as zero. The walk's 30-second
release cap per goal world is unchanged and covers the new work.

## Implementation Tasks

- [ ] `TargetSelector` first-pick memory and `MutationSummary::events` with
      `MutationEventRecord`; tests for the count/funnel agreement, the
      selected-but-inapplicable case on an edgeless Graph module, and the
      no-eligible-node case; a property that recording leaves genome, RNG
      stream, and existing summary fields unchanged.
- [ ] `mesh_execution` exposes the executed and contributing node id sets
      beside the existing counts, with the counts unchanged.
- [ ] Drift harness: per-lineage opportunity pooling, the module table with
      the identity/provenance rule, cohort facts, per-checkpoint cohort and
      retention readings; property tests for the invariants above; fixtures
      for id reuse as two modules, copy provenance, the censoring split, and
      walk isolation (genomes/RNG identical with and without the readings).
- [ ] `bench.rs`: `drift-depth-v3` blocks under each checkpoint reading,
      serde-defaulted, six-decimal fractions from pooled integers; readings
      file and `docs/progress.md` row.

## Verification

- [ ] `cargo test -p v3-core --test viability` (engine birth path touched) and
      `make check` on the final code -> result and tested commit recorded here.
- [ ] Focused tests named above -> results in
      [`docs/progress/readings/t13-f01-module-recruitment-observability.md`](../../progress/readings/t13-f01-module-recruitment-observability.md).
- [ ] Every pre-existing deterministic field of the gate and goal reports
      equals T12.F04's (`t12-f04-baseline-world-set{,-goal}.json`), excluding
      the feature label, `drift_depth.version`, and the new blocks; checked by
      a recorded structural comparison, result in the readings file.
- [ ] Per goal world: drift wall time under the 30-second cap; the cohort,
      opportunity, and retention tables per checkpoint with denominators and
      censoring counts, in the readings file; depth-1,000 and 2,000
      changed/all-birth readings against the 0.0015 and 0.005 floors.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      and every survivor resolved as killed, equivalent, or deferred.
- [ ] Benchmark reports stored at
      `docs/progress/features/t13-f01-module-recruitment-observability.json`
      and `-goal.json`; the second goal run is not required (user decision
      2026-09-05, `docs/workflow.md`).

## Performance and Goal Impact

**Predeclaration — written before the run.** Observation-only feature; the
natural-analog requirement does not apply. Production births gain one small
event-record vector; no simulation semantics, RNG draw, or work counter
changes. Predeclared: all six normalized counters exactly unchanged in both
profiles against the previous closure T12.F04 (gate epoch
`remove-complementary-nutrition`; goal series `goal-worlds-v1` compares to
T12.F04 as its only reference); wall time per creature-tick within the
existing threshold, with any flag investigated before closure. The drift walk
gains per-generation id-set diffs and node comparisons for 50 lineages ×
2,000 generations and no battery work; expected under 2 s added per world
inside the unchanged 30 s cap. Founder 10 s, evolved 180 s, and the 15-minute
goal investigation thresholds are unchanged. No indicator is predeclared to
move; existing drift fields per world are predeclared byte-identical to
T12.F04. The new cohort and opportunity readings are descriptive baselines
with no floor; T13.F02 owns the replicated baseline. Cognition indicators
remain `Undefined`.

**Measured verdict.** To be recorded at closure.

- Reports: [gate](../../progress/features/t13-f01-module-recruitment-observability.json),
  [goal](../../progress/features/t13-f01-module-recruitment-observability-goal.json).
- Full readings: [`docs/progress/readings/t13-f01-module-recruitment-observability.md`](../../progress/readings/t13-f01-module-recruitment-observability.md).

## Success Criteria

- [ ] The goal report and `docs/progress.md` show, per world and checkpoint,
      new-module cohorts by backend across the state ladder with censoring and
      retention, and per-lineage mutation opportunities by operator with the
      selected-but-inapplicable split, all with denominators.
- [ ] Existing drift, birth, and mesh readings and every production
      trajectory are unchanged, and the observation stays inside its caps.

## Notes for AI Agents

- Readiness review 2026-09-10 (orchestrator, one pass): the first draft said
  a new module's first *selection* could be inferred from genome diffs; that
  cannot see a selected-but-inapplicable event, which is the fact T13.F03
  repairs, so the engine event record was added and the ladder's *selected
  only* rung defined from it. The copy-provenance rule was tightened to
  require a `CopyNode` event in the same birth so blank detours matching an
  existing blank node are not labeled copies.
- Unobserved facts stay unmeasured: effects and usefulness have no field here.
