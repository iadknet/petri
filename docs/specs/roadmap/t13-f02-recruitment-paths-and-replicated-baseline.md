# T13.F02 — Recruitment Paths and Replicated Baseline

**Status**: In Progress
**Last updated**: 2026-09-12
**Feature**: T13.F02
**Track**: [T13 — Neutral Module Recruitment](../../roadmaps/t13-neutral-module-recruitment.md)

## Goal

Measure how often new Graph and VM mesh modules acquire useful reactive behavior
through viable intermediate steps, and whether that behavior persists. Separate
constructed possibility, production mutation discovery, dormant preparation,
selection, and variation between independent lineages in one bounded baseline.

## Non-Goals

- No mutation, targeting, growth, runtime, founder, reward, or production-default
  changes; T13.F03–F05 retain the repairs. No ecological selection, cognition
  claim, new environmental pressure, profile, campaign runner, or dashboard.
- No reinterpretation of historical floors, reports, or accepted exceptions.

## Inputs and Invariants

- The owning row and its F02 scope/comparison notes define acceptance.
  [T13.F01](t13-f01-module-recruitment-observability.md) supplies event records,
  opportunity funnels, bounded module identity and contribution observations;
  [T11.F05](t11-f05-temporal-controller-fixtures.md) supplies A1 and the applied
  tick/trace seams; [T11.F08](t11-f08-function-preserving-duplication-and-module-growth.md)
  supplies dormant VM copies, phase-faithful Graph copies/splits and mesh-copy
  activation. Dependencies are owned by the roadmap row.
- Extend `crates/v3-core/src/neighborhood/` and the existing
  `crates/v3-cli/src/bench.rs` goal-report assembly. Reuse
  `MutationEngine::apply_mutations_with_food_type_count`, `MutationSummary`,
  `ParentExecuted`, `RecruitmentTracker`, the production mesh/tick path and
  `static_successor_bypass`; reuse the fixture patterns in
  `tests/temporal_fixtures.rs`, `mutation/vm/f08_tests.rs`,
  `mutation/graph/tests/f08.rs` and `mutation/topology/f08_tests.rs`.
  Mutation/runtime/simulation never depend on observation types.
- Research decision, 2026-09-12: extend these seams with existing crates/`std`.
  A separate simulator or selection in historical drift would change the
  substrate or comparison. [Lenski et al. 2003](https://doi.org/10.1038/nature01568)
  motivates recording intermediates including damage; [Blount et al. 2008](https://pubmed.ncbi.nlm.nih.gov/18524956/)
  motivates matched-history replication. Neither supplies a discovery floor.

**Tasks and applied outcomes.** Task A is T11.F05's A1: `Move(E)` when
`FoodHere(type 0) > 0`, otherwise `NoOp`. Task B changes only the relevant cue
to `NeighborFood(E, type 0) > 0`. Each evaluation uses the eight combinations
of here/east/north food being 0 or 1; north is irrelevant to both answers.
Every scene starts a fresh creature at the centre of the same clear 12×12
world, energy 50, zeroed learned state, and runs one production tick. The
correct queue and applied eastward displacement (or `NoOp` and no displacement)
must agree; another action that happens to leave the creature still is wrong.
Score is correct scenes / 8. Retain actual survival, ending energy, execution
work and applied maintenance charge separately from correctness.

Use default runtime, mutation and lifecycle charges. Fixture-only world setup
has no founders except the subject, no food growth or spontaneous food, and
the stated food placements; runtime budgets and metabolic costs stay enabled.
Proposal mutation uses the default mutation config, independently of fixture
world setup. These are short task outcomes, not lifetime fitness or proof of
ecological viability. A parent is task-live when it survives all eight scenes;
also retain the established battery's distinct dead/silent/changed label at
recorded checkpoints.

**Constructed paths.** Provide one explicit path on each backend from an inert
mesh module through dormant preparation to a dispatched effect and useful Task
A contribution. Reuse the existing split/copy fixtures and demonstrate the
blank starting form too. Record every genome edit, its existing production
operator/helper, before/after values and the task/battery outcome; authored
edits and controlled seeds are labeled constructed. Every intermediate remains
task-live and loses at most 1/8 Task A score against the preceding parent.
Check unchanged incumbent behavior during inert/preparation steps using the
complete battery; track memory, queue payloads and routing effects separately
where those surfaces are used. Exact-copy activation must retain the existing
F08 guarantees. These paths prove possibility; they do not count as discovery.

**Starting forms and matching.** Task A discovery starts with the same
always-`NoOp` incumbent behavior (4/8) and one designated inert added module.
Use Graph blank, Graph dormant copy, Graph neutral split, VM blank and VM
dormant copy forms. Blank modules use the current constructor's definitions;
copy forms use the existing copy mechanism, and the Graph split form uses an
existing function-preserving edge split inside the inert module. Nonblank
forms may carry dormant sensor/action material but have no Task A contribution
at depth zero. Take these forms from the constructed paths' inert stages;
freeze their explicit genomes and creation edits before discovery measurement,
without selecting starts for observed discovery results. Verify
matched battery actions and task outcomes, and expose their sizes, maintenance
costs, dispatch and mutable sites. Do not claim equal mutational opportunity
from equal behavior. Constructed starts are not random constructor frequencies.

The changed-task comparison starts from a Task A-correct incumbent (8/8),
with a dormant copied module on each backend. Build one recorded common
history, fork before its dormant preparation edits, and replay the same
non-preparation edits into both descendants. The prepared arm keeps the silent
edits that read the east cue and prepare an eastward action; the unprepared
arm omits exactly those edits. Verify identical incumbent genome outside that
module, old-task outcomes and battery actions at each matched history point.
Both arms switch to Task B together at discovery generation zero. Store the
genomes/history and exact preparation difference. Task A tradeoffs after the
switch are reported and do not veto Task B adaptation. This treatment measures
the effect of constructed preparation, not its evolutionary discovery rate.

**Replicated experiment — fixed before measurement.** Run once per standard
goal report against the explicit default task configuration, outside its three
ecological world runs. It is not three independent observations of that same
configuration. Keep the existing per-world drift walk completely unchanged.

| Dimension | Predeclaration |
| --- | --- |
| Replication | Four independent batches, eight lineages per batch: 32 lineages per arm. Batch/lineage streams are disjoint; corresponding lineages are paired across treatment arms. |
| Main arms | Five starting forms × unconditional drift / bounded selection = 10 arms on Task A. |
| Changed-task arms | Two backends × prepared / unprepared × drift / selection = 8 arms on Task B. |
| Supply | Two sibling proposals per parent per generation, each made by one complete production mutation-engine call, including zero-event and skipped-event proposals. No forced event counts, operator weights, site choices or backend draws. |
| Horizon | 32 discovery generations followed by 16 further generations under the same treatment; 48 × 2 = 96 proposals per lineage. Total 55,296 proposals across the 18 arms. |
| Seeds | `SmallRng` per proposal, seeded by `13_020_000 + batch*1_000_000 + lineage*10_000 + generation*2 + sibling`; batch 0..3, lineage 0..7, generation 0..47, sibling 0..1. Arm comparisons reuse the corresponding seed, never a mutable RNG shared across arms. |
| Mutation context | Recompute reachability and the task-dispatched parent node indices before each sibling pair. Record that task execution, rather than the historical walk's 10-generation battery refresh, supplies `ParentExecuted`. |
| Drift | Retain sibling 0 unconditionally, including task-dead genomes; sibling 1 is observed but never selected. Continued mutation of task-dead parents is labeled genotype drift, not successful reproduction or ecological survival. |
| Selection | Among the parent and task-live offspring whose score is at least the parent's, retain the highest score. Ties choose sibling 0, then sibling 1, then parent, allowing neutral steps. If neither child qualifies, retain the parent. No task bonus, module-size reward, or hidden reseeding. |
| Practical loss | 1/8 absolute task score is the declared meaningful loss/benefit margin. A useful module loses at least 1/8 on bypass; a useful discovery also improves the active task by at least 1/8 over its starting parent. |
| Retention | For the first useful retained discovery by generation 32, inspect the same module 16 generations later, counting generations in which selection retained the parent. Classify task-dead first, otherwise deleted, otherwise present and useful / no longer useful. Lineages without discovery remain in the original 32-lineage denominator. |
| Observation wall budget | At most 120 s of additional release wall time for the entire experiment, measured separately; no wall-clock early stopping, sample reduction, or seed replacement. An overrun is investigated before closure. |

**Discovery and uncertainty.** A module is one mesh node, with T13.F01's
lineage/id/creation-depth identity. The designated starting scaffold and modules
created later are eligible; the incumbent is a separate reference. Retain the
created module's backend and its current backend if it changes. Dispatch,
output/state effect, battery contribution and task usefulness remain distinct.
Use the existing static-successor bypass on the same eight fresh scenes for
usefulness, recording dispatch and the paired score loss. Useful discovery
requires the unmodified subject to be task-live in all eight scenes. Score gain without
useful added tissue is controller adaptation, not module recruitment.

Store per-lineage results and both siblings' generation, parent live/dead
status, score, chosen/not-chosen status, module counts, costs, attempted/applied
events and target exposure, including selected-but-inapplicable discards by
backend/operator. Preserve zero-event births and non-recruiters. Report
proposal discovery separately from retained discovery and discovery reached
only through task-live, within-margin intermediates. At depths 0/32/48 retain
the full battery reading and T13.F01 cohort facts, with resolution/censoring
explicit. For each lineage's first observed successful path, store replayable
seeds, exact mutations and before/after affected values, including prior
silent edits; do not substitute an operator name for the mutation itself.
Detailed records are local-only under the storage exception below.

Report 95% Wilson intervals for the 32-lineage discovery/retention fractions,
the four separate batch results, paired lineage outcome differences, and
observed depth-to-discovery ranges. Siblings and reused seeds across arms are
not independent replicates. Retention among discoverers carries its own
denominator; zero discoverers makes that conditional estimate undefined.
There is no positive-discovery floor and no claim of treatment superiority
from pooled proposal counts. Nulls, wide intervals, low relevant exposure and
task-limited attribution are outcomes. Map measured applicability, activation
or path gaps to the already-owned F03/F04/F05 work without repairing them here.

**History/RNG confounds.** Observations consume no mutation RNG. Equal seeds
do not imply equal mutations after genomes or eligible sites diverge; record
the first such divergence and restrict paired-mutation claims to verified
matches. If an added draw is implicated, use the VM-only diagnostic below on
both original seeds and reports, never seek a passing replacement seed. When
there is no added-draw comparison, report this control as not applicable with
that reason. Any genotype/size/exposure difference remains a stated limitation.

| Conditional diagnostic | Fixed extent and interpretation |
| --- | --- |
| VM sham draw | Repeat the VM-blank Task A and VM-unprepared Task B arms, each under both policies, with one discarded `gen_bool(0.5)` before each proposal's engine call; same four batches/eight lineages/48 generations/two siblings, 12,288 additional proposals maximum. Only the initial backend is constrained to VM; subsequent production growth stays unchanged. Compare to the no-sham VM arms and label it RNG sensitivity, not a backend intervention. Include it within the 120 s budget. |

## Implementation Tasks

- [x] Add the constructed paths and paired task evaluation through existing
  runtime/tick seams, with behavior and invariant coverage.
- [x] Add the bounded replicated observation, per-sibling records, exact path
  replay and honest discovery/retention/uncertainty accounting.
- [x] Add `recruitment-paths-v1` to the standard goal report once per report,
  with explicit config/task identity and separate observation wall timing;
  keep the gate and existing drift/profile parameters unchanged. Historical
  absence is unmeasured, not zero. Register focused tests in existing gates.
- [ ] Complete verification and stored readings; update `docs/progress.md`,
  `docs/progress/benchmark-series.json` and the owning roadmap at closure.

## Verification

- [x] `cargo test -p v3-core recruitment_paths` and
  `cargo test -p v3-core --test recruitment_paths`: constructed paths, applied
  task results, controls, path replay and pure accounting/selection invariants;
  commands/results in [readings](../../progress/readings/t13-f02-recruitment-paths-and-replicated-baseline.md).
- [x] `cargo test -p v3-cli recruitment_paths`: goal-only report wiring,
  deterministic reduced-size results, explicit absence and truthful totals;
  readings as above. No stochastic discovery success is an invariant.
- [ ] `make check` and `make roadmap-check`; results in readings. Use TDD and
  proptest for pure invariants; preserve any generated regression files.
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: 94 survivors killed by
  tests; 12 equivalent; 0 deferred. Counts, output path and full disposition
  are in the [readings](../../progress/readings/t13-f02-recruitment-paths-and-replicated-baseline.md#mutation-gate).
- [x] Both benchmark commands generated the required outcomes, uncertainty,
  paths, controls, caps and verdicts; readings are linked above. The goal
  artifact is local-only by exception; retain the gate report for closure.
- [x] A second full goal run for determinism is not applicable: the shared
  workflow's 2026-09-05 decision requires one goal run; `make check` retains
  cross-process reproducibility and the gate's two-run check.

```sh
make bench PROFILE=gate FEATURE=t13-f02-recruitment-paths-and-replicated-baseline
make bench PROFILE=goal FEATURE=t13-f02-recruitment-paths-and-replicated-baseline
```

| Closure record | Value |
| --- | --- |
| Orchestrator and mutation specialist | `gpt-5.6-sol`, `medium` |
| Persistent spec owner/advisor and implementer; fresh reviewer | `gpt-6-astra`, `xhigh` |
| Benchmark specialist | `gpt-5.6-terra`, `high` |
| Advisor consultations | 7 |
| Initial review findings | P1=1, P2=2, P3=1 |
| Post-review remediation | One documentation-only pass; typed persisted event outcome remains deferred. |
| Requirement corrections / user interventions | 1 / 1: local-only raw goal evidence. |

## Performance and Goal Impact

**Predeclaration — written before the run.** Observation only; no natural analog
or recipe integration applies. All six normalized simulation counters and
existing deterministic observations must remain unchanged against T14.F03.
Only experiment fields and separate timing are additive: no existing indicator
movement or cognition claim. The new descriptive goal measure has no floor
and is unmeasured in gate.

The benchmark commands use the unchanged series index: gate epoch
`remove-complementary-nutrition.json` and previous
`t14-f03-applied-mortality-and-energy-accounting.json`; goal epoch
`t12-f04-baseline-world-set-goal.json` and previous
`t14-f03-applied-mortality-and-energy-accounting-goal.json`, all under
`docs/progress/features/`. Preserve +10%/+50% work and +25%/+100% wall flags,
record inherited epoch flags separately from change against the previous
closure, and investigate new flags. No severe compute allowance, threshold
change or epoch re-pin is predeclared. Simulation wall/creature-tick should
stay within its existing flag threshold; additional observation work is
bounded separately by 120 s and does not enter simulation work counters.
Founder observation remains under 10 s, evolved observation under 180 s summed
across seeds, and historical drift under 30 s per world. The complete goal
profile retains the 15-minute investigation threshold. Benchmark and mutation
specialists run sequentially without competing builds, tests or servers.

| Measured verdict | Evidence and limitation |
| --- | --- |
| Final-code profiles | Each ran once at `7f4c54fb`, exited 0 and has `comparison.severe=false`. No threshold, baseline or epoch changed. |
| Gate flag investigation | Simulation wall 570.409→745.293 ms versus T14.F03 (+30.659%, new and non-severe). Deterministic results and work counters match; recruitment observation is absent in gate. Cause remains unattributed, with no established host-noise attribution. |
| Goal work counters | Unchanged versus T14.F03; inherited epoch plasticity flag retained. |
| Individual observation caps | All pass; separate timings and verdicts are in readings. |
| Goal simulation timing | 515,641.938 ms, not full elapsed time. |
| Goal through completed observations and report writing | `<9m40s`, conservatively bounded by preceding gate completion `2026-09-13T04:06:19Z` and complete goal artifact mtime `2026-09-13T04:15:58Z`. Exact process-exit duration was not captured. The 15-minute investigation threshold is reviewed against this bound. |

- Report: [gate](../../progress/features/t13-f02-recruitment-paths-and-replicated-baseline.json).
- Full readings: [t13-f02-recruitment-paths-and-replicated-baseline.md](../../progress/readings/t13-f02-recruitment-paths-and-replicated-baseline.md).

**User storage exception, 2026-09-12.** Commit the gate report and readings,
not the raw goal report; do not compact, rerun or index the omitted goal file.
Detailed sibling/path evidence is local-only. Summary/storage implementation
belongs to Planned, unscheduled T15.F01.

| Local raw goal provenance | Value |
| --- | --- |
| Path | `/Users/istefanek/projects/petri/tmp/bench-artifacts/t13-f02/t13-f02-recruitment-paths-and-replicated-baseline-goal.json` |
| Bytes | `356934683` |
| SHA-256 | `bc7f039febdae9c6e32e9f5f83485d3705f0a1ff30181dbba32fa509fa6bb849` |
| Availability | Present and checksum-verified locally on 2026-09-12; neither portable nor promised to persist. |

## Success Criteria

- [ ] Both backends have an explicit viable inert/prepared/effect/useful path,
  and matched changed-task controls distinguish dormant preparation.
- [ ] The predeclared replicated baseline records all proposals, exposure,
  lineage variation, damage, costs, useful contribution and retention; nulls
  and capability gaps remain explicit without a cognition or repair claim.
- [ ] Required checks, review, mutation adjudication and benchmark records
  are complete and truthful; only then mark the roadmap row and spec Complete.

## Notes for AI Agents

- Decision: This is the pre-repair baseline. Applicability, direct Graph
  activation and recruitment-path repairs remain with T13.F03–F05.
- Exception: One user intervention/requirement correction on 2026-09-12:
  local-only goal evidence above; broader work moves to [T15](../../roadmaps/t15-benchmark-evidence-storage.md).
  Experiment parameters, thresholds, baselines and runtime behavior are unchanged.
- Deferred: `Event.outcome` is a debug-derived string; current accounting and
  replay use typed engine records and no parser exists. T15.F01 owns the future
  persisted representation/summary; T13.F03/F06 consumers must not parse this
  debug string.
- Cost: Task-specific usage unavailable.
