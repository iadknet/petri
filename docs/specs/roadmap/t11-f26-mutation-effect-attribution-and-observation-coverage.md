# T11.F26 — Mutation-Effect Attribution and Observation Coverage

**Status**: In Progress
**Last updated**: 2026-09-24
**Feature**: T11.F26
**Track**: [T11 — Brain Genotype-Phenotype Map](../../roadmaps/t11-brain-genotype-phenotype-map.md)

## Goal

At every goal-profile closure, each world's report says, for the drift walk's
parents and the world's selected genomes, whether applied mutations reach
executed computation, change internal decisions, state or cost, or change
actions, with parents that are already actionless and edits that leave the
genome identical counted apart, and with the silence the battery cannot explain
kept as an explicit unresolved share. A separately versioned coverage panel
re-reads a fixed sample of battery-silent pairs on contexts and histories the
original battery lacks. The existing Petri progress report shows all of it from
the stored summaries. Observation only.

## Non-Goals

- No change to the mutation engine, operators, targeting, rates, supply, costs,
  runtime, founder, sensors, or any default; no production RNG consumed; no
  mutation, targeting or fitness decision reads these observations.
- No change to `neighborhood-v1`, its classifier, the drift walk, T14.F12's read,
  T11.F01's founder/evolved halves, or any existing report field, comparison
  key, floor, or chart.
- No floor, indicator component, or pass/fail threshold on the new readings; no
  claim that a changed action is useful behavior or cognition.
- Not owned here: parameter targeting (T11.F25), multistep neutral potential
  (T11.F12), selected mutation-policy comparisons (T11.F13), whole-module
  recruitment (T13), input-family/channel attribution and ecological
  opportunity (T20.F01). No second-step experiments, long ecological
  campaigns, generic tracing platform, new dashboard, or controller redesign.

## Inputs and Invariants

Sources: the track's F26 row, criteria lines for F26, and the six F26/F27
Notes entries; the [pilot](../../strategy/drift-silence-experiment-2026-09-24.md),
[follow-up](../../strategy/drift-silence-followup-2026-09-24.md) and
[ALife research](../../strategy/drift-silence-alife-research-2026-09-24.md)
notes (probes 1–2 are this feature's draft); T11.F16 and T14.F12 specs.

**Existing seams, verified in code.** `neighborhood::births::per_birth_result`
(own-size supply) and `per_birth_result_on_units` (the drift walk's pinned
`FOUNDER_GENOME_SIZE_UNITS` = 97) run births and fold `BirthResult`; a
zero-applied birth is counted, not evaluated. `Battery::signature` records
actions only; `Battery::mesh_execution_sets` yields executed and
knockout-contributing sets; `runtime::traced_mesh::execute_creature_mesh_traced`
returns `MeshOutput` (actions, `work_counters`, cost report, priority bid) with
hop and per-pass traces (`MeshPassTrace.votes`, `effective_votes`).
`MutationSummary.events` records each event's domain, operator and first
target. `drift::observe` holds the 20 birth-lineage genomes at each checkpoint;
`neighborhood_read_for_seed` samples `read_sample_ranks(n, 50, world_seed)`.
`simulation::tick::assemble_sensor_inputs` is the production assembler; it
zeroes extended perception for genomes that do not read it. The brain engine
never edits phenotype colour, so no kin-tag or phenotype edit enters a birth.

**Research decision.** Extend the neighborhood module and the goal report,
lifting the pilot/follow-up probes. The strongest alternative, a standalone
harness like the pilot's, reproduces nothing at closure and cannot feed the
page or T20.F01. No new dependency is needed. The ALife note's constraints are
adopted: separate all-birth, event-bearing and single-event denominators;
cluster by parent; keep the original metric and label extensions separately;
use positive controls; stop at the first causal separation.

**Placement and identity.** New per-case block
`cases[].mutation_effects: Indicator<MutationEffects>`, `#[serde(default)]`,
defined on the `goal-worlds-v1` profile only, `Undefined` elsewhere (gate
included), projected whole into the committed summary. Version strings:
`mutation-effects-v1` (strata and attribution, read on `neighborhood-v1`) and
`neighborhood-coverage-v1` (extension). Timing in
`environment.mutation_effects_wall_clock_ms_per_seed` and `_total`, outside
every existing timer. Existing blocks, including their key sets, stay
byte-identical.

**1. Exposure strata on the existing readings.** For each drift checkpoint
(0, 22, 250, 1,000, 2,000) and for the world's T14.F12 read, one row derived
from the executions and births those readings already perform: no extra
mutation call or battery execution for these rows.

| Field | Content |
| --- | --- |
| `panel` | `drift@<depth>` or `selected-read` |
| `supply` | `pinned 97 units (drift chart)` or `own genome_size (production)` |
| `parents`, `parents_all_noop`, `parents_one_queue` | sampled parents; all 80 battery executions `NoOp`; exactly one distinct action queue across the 80 |
| `distinct_queues_histogram` | parents by distinct-queue count, buckets 1, 2, 3–4, 5–8, 9+ |
| `births_total`, `zero_requested`, `requested_all_skipped`, `event_bearing` | partition of births |
| `requested_events_total`, `applied_events_total` | integer sums |
| `genome_identical` | event-bearing births whose child genome equals the parent under genome identity: equal `Debug` rendering, which distinguishes -0.0, ignores NaN payloads, and omits the birth-only `birth_weights` that `cgp.rs` excludes from identity |
| `from_actionless`, `from_acting` | silent/changed/dead/zero-applied births split by parent capability |

**2. Single-event attribution cohorts.** Diagnostic proposals, not production
births: each is a fresh parent copy through `apply_mutations_on_units` with
`units = 1` and `per_unit_rate = 1.0` (exactly one requested event), the
production operator mix and the parent's battery-executed set. Skipped
proposals stay in denominators; no retry.

| Parameter | Value |
| --- | --- |
| `founder` cohort | canonical `V3Alpha1`, 1 parent, reference only |
| `drift` cohort | the 20 birth lineages at depth 2,000 of this world's walk |
| `selected` cohort | `min(20, s)` of this world's `s` T14.F12 sample genomes, a uniform draw without replacement over sample positions with `SmallRng::seed_from_u64(24_000_000 + world_seed)`, ascending, so rows join the read's per-genome rows without favouring low creature ids |
| Proposals | 100 per parent |
| Seed | `20_000_000 + 1_000_000 × cohort (founder 0, drift 1, selected 2) + 1_000 × (parent_index + 1) + proposal_index`, where `parent_index` is the parent's zero-based ordinal within its cohort, not the stored `parents[].index` (a `selected` parent stores its sample position: Canyon's first selected parent stores index 6 and uses ordinal 0); the stored `proposal_seed_formula` string keeps the name `parent_index` |
| Recorded per parent | index, generation or depth, genome size, total/reachable/executed/contributing nodes, all-NoOp flag, distinct queues, and the partition below |

Each applied proposal is compared with its parent over all 80 battery
executions using traced execution, on two named records per execution:

| Record | Fields compared |
| --- | --- |
| Computation | per hop: node id, upstream slots, output slots, vote contribution, route decision (selected target index and id, and every gate's computed runtime and effective scores), and every applied output-sink, action-parameter or shared-memory write value the backend trace records; per pass: votes, effective votes, committed action. Genome content copied into traces (gate bias, input refs, VM instructions and constants, graph node kinds) is never compared. The VM trace records no action-parameter writes, so VM parameter values are not compared |
| State and cost | after each execution: shared memory, remaining energy, work counters, priority bid, and graph runtime state (`node_state`, `node_outputs`, `plasticity_weights`, `eligibility_traces`) aligned by node id, where a node present on one side only differs when it stores any scalar, zeros included (empty placeholders do not count) |

Categories partition applied proposals, in this precedence:

| Category | Rule |
| --- | --- |
| `action_changed` / `action_dead` | the existing classifier's Changed / Dead |
| `genome_identical` | child genome equal to parent under the genome identity below |
| `unexecuted_edit` | no node whose genome differs by node id (added, removed or edited; entry change counts as edited) is dispatched in any execution of parent or child |
| `masked_before_selection` | computation record differs somewhere (including a parameter value decoded to the same action), actions identical |
| `state_or_cost_only` | computation identical, state-and-cost record differs |
| `unresolved` | an edited node is dispatched and neither record differs |

Floats compare by bit pattern. Independently of the partition, count
action-silent applied proposals whose state-and-cost record differs
(`silent_with_state_or_cost`, whatever their category), and `unexecuted_edit`
proposals whose records differ anyway (`consistency_violations`, expected 0,
reported not asserted). Operator rows attribute each proposal to the event's
accepted operator; an event with no accepted operator (`operator: None`)
counts in a per-domain `domain_exhausted` row; operator rows sum to the
cohort's proposals and skips. Per cohort, the same counts appear per operator (rows only for
operators drawn) and per first-recorded-target class: knockout-contributing,
executed not contributing, reachable not executed, unreachable, no node
target. Target class is a stratum label, never a cause; knockout contribution
alone is never an explanation.

**3. Coverage extension `neighborhood-coverage-v1`.** Pair sample: per parent,
the first 10 proposals in proposal order that are applied, not
genome-identical, and action-silent on `neighborhood-v1`. Contexts, all from
fresh cognition state:

| Group | Content |
| --- | --- |
| `recorded` | up to 32 living creatures of this world's terminal population, sampled `SmallRng::seed_from_u64(23_000_000 + world_seed)` without replacement from the id-sorted population, in sample order; snapshot from the production assemblers with typed local food and extended perception both assembled unconditionally (the donor's genome never gates an input); the creature's own energy |
| `authored` | 24 contexts: context `i` copies `neighborhood-v1` snapshot `i` and sets channel group `i mod 8` (neighbor barriers, previous outcomes, area food and typed area food, area barrier, area occupancy, nearby core, nearby vitals, nearby identity; all zero in `neighborhood-v1`) to values drawn inside each field's production range from `SmallRng::seed_from_u64(25_000_000 + i)` (previous outcomes: EnergyDelta in [-1, 1], ActionSuccess in [0, 1], DamageDelta in [-1, 0], OffspringSuccess uniform over {0, 1}); labelled not guaranteed realizable |
| `sequences` | 4 sequences × 32 ticks; sequence `s` tick `t` uses recorded context `(8s + t) mod r` (`r` recorded contexts; authored contexts, same rule, when `r < 4`); production shared-memory bookkeeping, energy reset to the context's value each tick; ticks 1–4 and 5–32 reported apart |

Any change to these rules bumps the version; tests pin representative
contexts. Every cohort parent is also evaluated once on the extension. Per
cohort: parents evaluated, pairs sampled, pairs whose actions differ per
group, pairs with a state-and-cost difference only, and parents all-NoOp on
the original that act on the extension. Controls run in the same code on every report: a
barrier-dependent action edit and a slow integrator crossing its threshold
after tick 4 (both silent on `neighborhood-v1`, both must differ on the
extension), and a same-genome pair (no difference in any group); the block
stores each control's outcome.

**Short and failed samples.** An extinct world yields `Undefined` selected
cohort and recorded group with the reason; fewer parents or contexts than
requested are stored as requested and actual counts. Zero denominators are
`Undefined`, never 0.

**Invariants.** Observations run after the last tick on clones and consume no
simulation RNG. Every count is an integer folded in a fixed order and the block
is byte-identical across thread counts. Reconciliation holds exactly: each
exposure row's partitions sum to `births_total`, and its silent/changed/dead
sums equal the existing `any_events` tallies of the reading it describes; each
cohort's categories sum to its applied proposals. Storage: the committed goal
summary grows by at most 400 KB in total.

**4. Progress report.** Extend "4. Is the substrate still evolvable?" in
`docs/progress/index.html`, keeping every existing chart and value. Show per
world: exposure (requested/applied per birth, zero-applied and
genome-identical shares, supply label); parent capability (all-NoOp and
one-queue parents, births from acting parents); single-event attribution
category shares with counts, denominators and cohort identity (depth 2,000
versus selected mean generation and size); state/cost versus action effects
(`silent_with_state_or_cost` beside action effects);
original versus extended coverage with control status; and a per-cohort
detail table of operator and first-target-class rows with proposals,
skipped, applied and category counts. Label every count's
denominator and panel/version; never join drift depth and selected generation,
or pinned and own-size supply, as one trend. Reports lacking the block read "not
measured", zero denominators "undefined", short samples show actual/requested.
Link the closure reading in `docs/progress/readings/t11-f26.md` and state that an
action change is not evidence of useful behavior.

## Implementation Tasks

- [x] Core: exposure strata folded from the existing drift checkpoint and
      read executions, the attribution cohorts and partition, the coverage
      extension with its contexts and controls, all in `neighborhood`.
- [x] Bench: `mutation_effects` per case, timing fields, summary projection.
- [x] Progress report section and its fixtures.
- [x] Closure reading: the table below filled from the goal summary in
      `docs/progress/readings/t11-f26.md`, one row per candidate bottleneck
      per world, each row naming its evidence and owner. Supported: drift
      actionless parents (all worlds); unexecuted edits (Canyon drift,
      Orchards selected); masking, state or cost (Canyon selected).

| Candidate bottleneck | Denominator | Supported when (per cohort, drift and selected) | Owner if supported |
| --- | --- | --- | --- |
| Actionless parents | parents evaluated on both panels | ≥ 25% all-NoOp on both | T11.F12 / T11.F13 |
| Edits outside executed computation | applied proposals | `unexecuted_edit` ≥ 25%, and the plurality category in ≥ 25% of evaluated parents | T11.F13 |
| Masking before action selection | action-silent applied proposals | `masked_before_selection` ≥ 25%, and plurality in ≥ 25% of evaluated parents | T20 / T13 |
| State or cost without action | action-silent applied proposals | `silent_with_state_or_cost` ≥ 25% | T20 |
| Battery blind spot | coverage pairs sampled | ≥ 5% differ in actions on the extension | T20.F01 observation |
| Identical edits | applied proposals | `genome_identical` ≥ 10% | T11.F13 |

Each candidate reads "insufficient evidence" when its denominator is below 100
proposals, 20 coverage pairs, or 4 parents (or the cohort is `Undefined`);
otherwise "not observed" at a zero numerator, "supported" at its bar, and
"minor" below it. The `unresolved` share is stated per cohort, never
apportioned.

## Verification

- [x] Focused tests: reconciliation (strata vs existing tallies, category
      partition), the attribution precedence on constructed pairs (one per
      category, including a graph-state-only change and a Graph parameter
      change decoded to the same action; a pair whose computation and state both
      change with identical actions counts in `masked_before_selection` and
      in `silent_with_state_or_cost`; operator and `domain_exhausted` rows
      reconcile with proposals and skips), the three controls, unconditional
      recorded-context assembly (a donor without typed-food or perception
      references, a tested genome reading them), genome-identical and
      zero-denominator cases, byte identity across thread counts, and the
      pinned context sample and authored contexts; names and results in
      readings.
- [x] Existing blocks unchanged: gate `deterministic` equal to the T11.F27
      gate summary's; goal `deterministic` equal to the T11.F27 goal
      summary's after removing only `cases[].mutation_effects`; method and
      result in readings. Confirmed by the benchmark specialist (see
      readings) with `jq -S` diffs: identical in both cases.
- [x] Page: `node --test scripts/benchmark-artifacts.test.mjs` with new,
      historical (block absent), zero-denominator and short-sample fixtures,
      displayed counts and ratios checked against the fixture summary; a
      browser inspection of the served report with a screenshot path in
      readings. The fixture tests pass; the block-absent and measured views
      are inspected, the measured one spot-checked against the goal summary
      (see readings).
- [x] `cargo clippy --workspace --all-targets -- -D warnings` is clean and
      `make check` exits 0 (see readings).
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      every survivor resolved as killed, equivalent, or deferred.
- [x] Gate and goal summaries stored at
      `docs/progress/features/t11-f26-mutation-effect-attribution-and-observation-coverage.json`
      and `...-goal.json`, local raw hash/byte count and verification time
      checked (see readings), series entries added to
      `docs/progress/benchmark-series.json` (`gate.closed`,
      `goal_worlds.closed`), no new full report staged (raw artifacts stay
      under the ignored `.bench-artifacts/`); the second goal run is not
      required (workflow, 2026-09-05).

## Performance and Goal Impact

**Predeclaration — written before the run.** Observation only: no natural
analog is required and no simulation behavior changes.

| Item | Predeclared |
| --- | --- |
| References | gate: previous closure T11.F27 and gate epoch T11.F25; goal: previous closure and goal-worlds epoch T11.F27 (`docs/progress/benchmark-series.json`) |
| Deterministic counters, persistence, existing indicators | no change in either profile; any movement is a defect |
| Gate | untouched; `mutation_effects` absent |
| Goal observation cost | expected under 20 s summed across the three worlds (about 4,100 traced proposals and at most 410 coverage pairs per world); cap 60 s in `environment.mutation_effects_wall_clock_ms_total` |
| Existing caps | drift walk and read timers, which absorb the exposure strata, within 10% of their T11.F27 times (18.2 s, 2.1 s); founder 10 s, evolved 180 s, read 10 s unchanged |
| 15-minute goal investigation threshold | judged on the end-to-end process time the benchmark specialist records (T11.F27: 616 s of simulation plus about 41 s of timed observations); F26 adds only its own timer |
| Summary size | committed goal summary grows by at most 400 KB |
| Expected readings, sanity only | drift cohort: about half its parents all-NoOp, action effects near 1% of applied proposals (pilot 13/2,000); selected cohort: fewer actionless parents and more action effects (follow-up early survivors 42.9% changed of applied); a large `unresolved` share is a result, not a defect |
| Epoch | no re-pin expected or authorized |

Exceeding a cap is resolved before closure, never absorbed. Commands for the
benchmark specialist, sequential and after all benchmark-affecting work:

```sh
make bench PROFILE=gate FEATURE=t11-f26-mutation-effect-attribution-and-observation-coverage
make bench PROFILE=goal FEATURE=t11-f26-mutation-effect-attribution-and-observation-coverage
```

**Measured verdict.** Benchmark specialist run 2026-09-24/25 (worktree
`.claude/worktrees/t11-f26`, commit `01e9c277`).

| Item | Measured |
| --- | --- |
| Gate | `make bench PROFILE=gate FEATURE=t11-f26-mutation-effect-attribution-and-observation-coverage` exit 0; `severe=false` against both T11.F25 (gate epoch) and T11.F27 (previous closure); every defined counter delta 0.000000; `decided_passes` `percent_delta` is null (undefined, 0 → 0) against both references |
| Goal | `make bench PROFILE=goal FEATURE=t11-f26-mutation-effect-attribution-and-observation-coverage` exit 0; `severe=false` against T11.F27 (previous closure and goal-worlds epoch); all nine counter deltas 0.000000, `decided_passes` included |
| Deterministic counters, persistence, existing indicators | unchanged in both profiles: gate `deterministic` byte-for-byte identical to T11.F27's; goal `deterministic` identical to T11.F27's after `del(.cases[].mutation_effects)` (verified with `jq -S` diffs) |
| Goal observation cost | `environment.mutation_effects_wall_clock_ms_total` = 7,864.65 ms (7.86 s), under the 20 s expectation and the 60 s cap, across 3 worlds |
| Existing caps | drift walk 17,395.64 ms vs T11.F27's 18,242.30 ms (−4.6%, within 10%); read timer 1,622.90 ms vs T11.F27's 2,079.80 ms (**−22.0%, outside the declared ±10% band**, faster not slower); founder 324.96 ms (cap 10 s), evolved 5,288.63 ms total (cap 180 s), read 1,622.90 ms total (cap 10 s) — all hard caps met |
| 15-minute goal investigation threshold | end-to-end wall time measured 684 s (11.4 min) from command start to exit; simulation `environment.wall_clock_ms_total` = 630,602.69 ms (630.6 s) plus timed observations (drift 17.4 s + founder 0.32 s + evolved 5.29 s + read 1.62 s + mutation_effects 7.86 s + final_state 0.85 s ≈ 33.3 s) ≈ 664 s total; well under the 900 s (15 min) threshold |
| Summary size | goal summary grew from T11.F27's 7,810,874 bytes to 8,087,659 bytes: +276,785 bytes (≈270.3 KB), within the ≤400 KB cap |
| Expected readings, sanity only | attribution cohorts: drift 11/20 parents all-NoOp in every world ("about half"); drift `action_changed` 9, 5 and 9 of 2,000 applied proposals (0.45%, 0.25%, 0.45%; pilot 13/2,000, 0.65%), same order, on the low side; selected 2, 0 and 0 all-NoOp parents of 20 and `action_changed` 462/2,000, 433/1,755, 366/1,922 (19–25%), as expected. Exposure strata, over their own denominators: drift@2000 `from_acting.changed` 3, 4, 3 of 985 applied events; `selected-read` 18.2%, 20.3%, 17.0% |
| Epoch | not re-pinned; not authorized |

**Read-timer disposition (spec owner, 2026-09-25).** The read timer read
1,622.90 ms against T11.F27's 2,079.80 ms (−22.0%), outside the ±10% band.
Accepted as wall-clock variance; no action. The band guards against the
exposure strata adding cost to the drift and read timers that absorb them;
this run is faster although the read now carries that work, and every
deterministic counter is unchanged, so no observation or simulation work
was removed. Wall time is a secondary signal on this host, and every hard
cap is met with wide margin. The predeclaration stands unedited.

- Summaries: [gate](../../progress/features/t11-f26-mutation-effect-attribution-and-observation-coverage.json),
  [goal](../../progress/features/t11-f26-mutation-effect-attribution-and-observation-coverage-goal.json).
- Full readings: [`docs/progress/readings/t11-f26.md`](../../progress/readings/t11-f26.md).

## Success Criteria

- [ ] Each world's goal report carries exposure strata, attribution cohorts
      and the coverage panel, reconciled with the existing tallies, with every
      existing block unchanged.
- [ ] The progress report renders them with denominators, identities and
      missing-data states, original charts intact.
- [x] The closure reading classifies every candidate bottleneck as supported,
      minor, not observed, or insufficient evidence, with the unresolved
      share stated.
- [ ] Required checks, mutation evidence, independent review and closure
      records are complete; the row is checked and the spec Complete on main.

## Notes for AI Agents

- Decision: run substitutions for this feature (orchestrator brief, 2026-09-24): no Fable 5.1 model is used; the spec owner runs on Opus with high-effort intent, persistent and resumed by `SendMessage`; the roadmap-implementer does not consult the advisor tool and instead runs a fresh read-only Codex Astra (`gpt-6-astra`) `xhigh` task through the workflow's Codex channel at each consult point, brief written outside the worktree, reporting each consult and its decisive guidance; the orchestrator skipped the clean-main start check because main carries the pre-existing untracked `docs/specs/large-file-cleanup-2026-09-24.md`, left untouched.
