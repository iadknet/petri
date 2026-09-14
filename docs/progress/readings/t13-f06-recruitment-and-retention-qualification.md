# T13.F06 — Recruitment and Retention Qualification readings

Benchmark-specialist tables and transcripts for the feature spec
[`docs/specs/roadmap/t13-f06-recruitment-and-retention-qualification.md`](../../specs/roadmap/t13-f06-recruitment-and-retention-qualification.md).
Tested commit `e57d6f67c878faeeed96d6aadf32b00e0409bcb8`. Both profiles run
from the worktree `.claude/worktrees/t13-f06`; raw artifacts land in the main
checkout's ignored `.bench-artifacts/t13-f06-recruitment-and-retention-qualification/`.

## Commands and exit status

| Profile | Command | Outer exit | CLI `cli_exit.code` | `severe` |
| --- | --- | --- | --- | --- |
| gate | `make bench PROFILE=gate FEATURE=t13-f06-recruitment-and-retention-qualification` | 0 (observed via `echo $?` on the wrapping shell) | 0 | false |
| goal | `make bench PROFILE=goal FEATURE=t13-f06-recruitment-and-retention-qualification` | 0 (observed via `echo $?` on the wrapping shell) | 0 | false |

Both `measurement_evidence.cli_exit.code` fields (in each committed summary)
also read 0. The goal summary's `measurement_evidence.dirty` is `true` (the
then-untracked gate summary was present in the worktree during the goal run,
same condition as T13.F05); the gate summary's `dirty` is `false`.

## Wall time and threshold verdicts

**Gate** — raw `.bench-artifacts/t13-f06-recruitment-and-retention-qualification/gate.json`
(94,090 bytes, sha256 `73349a7c9031292a78cc9a5b7b962d62b59a61194c1dc8a808300f46674f5691`),
summary `docs/progress/features/t13-f06-recruitment-and-retention-qualification.json`
(96,946 bytes, `generated_at` 2026-09-14T14:26:48Z). `wall_clock_ms_per_creature_tick`
= 0.001606536; `wall_clock_ms_total` = 686.44 ms. The outer shell wrapping
`make bench` was not timed with `/usr/bin/time` for the gate run (only the
goal run was); the CLI-recorded `wall_clock_ms_total` above is the only wall
source for the gate profile, and the outer exit code (0, via `echo $?`) is
the only outer-process status source.

| Reference | `level` | `delta%` |
| --- | --- | --- |
| epoch `remove-complementary-nutrition.json` | ok | +2.99% |
| previous `t13-f05-function-preserving-module-recruitment.json` | ok | −28.58% |

No wall or work flags on the gate profile (the six normalized counters below
are separately confirmed byte-identical to F05).

**Goal** — raw `.bench-artifacts/t13-f06-recruitment-and-retention-qualification/goal.json`
(511,723,596 bytes, sha256 `e33e5d25bdb982b731cf7c4fccd314ba08ae3d4e3a076ccd349c346ff578897b`),
summary `docs/progress/features/t13-f06-recruitment-and-retention-qualification-goal.json`
(6,708,955 bytes; F05's was 4,161,635 bytes, a growth of 2,547,320 bytes from
the nine new arms and 98 new pairs). `wall_clock_ms_per_creature_tick` =
0.008662445; `wall_clock_ms_total` = 514,538.86 ms (≈ 8.58 min); outer wall
via `/usr/bin/time -p`: real 550.69 s (≈ 9.18 min), user 644.30 s, sys 25.53 s.

| Reference | `level` | `delta%` |
| --- | --- | --- |
| epoch `t13-f03-mutation-target-applicability-goal.json` | ok | +14.27% |
| previous `t13-f05-function-preserving-module-recruitment-goal.json` | ok | −27.13% |

No wall or work flags on the goal profile. This differs from F05, which
recorded `level=flag` wall-clock rows against both references (host
conditions per that closure); this run's wall clock reads `ok` against both
the same references, i.e. the flag condition is new-vs-F05 in that it is
absent here, not a new flag.

**Wall caps** (`measurement_evidence.wall_caps`, all satisfied):

| Cap | Measured | Limit |
| --- | --- | --- |
| Founder observation | 94.18 ms | < 10 s |
| Evolved neighborhood, summed across 3 seeds | 510.94 ms | < 180 s |
| Drift walk (`drift_depth_wall_clock_ms`, total across 3 worlds) | 16,343.71 ms (≈ 5.4 s/world average; `environment` carries no per-world drift-wall field — checked all scalar and list/dict `environment` keys, including `phase_wall_clock_ms_per_seed`, none named for the drift battery specifically — so only the total is available, which already satisfies the cap regardless of per-world distribution) | < 30 s per world |
| Recruitment-paths experiment | 10,275.41 ms (10.275 s) | < 120 s (predeclared 12–14 s; this run measured faster) |
| Goal profile total | 514,538.86 ms (≈ 8.58 min); outer wall 550.69 s (≈ 9.18 min) | < 15 min |

## Six normalized simulation counters

Both profiles' `mesh_hops`, `vm_steps`, `graph_relax_iters`,
`plasticity_updates`, `actions_applied`, `births` compare `delta%=0.000000`,
`level=ok` against the F05 reference (gate and goal alike) — byte-identical
to T13.F05, as predeclared.

## Byte-identity of the eighteen F02 arms

Compared field-by-field between
`docs/progress/features/t13-f05-function-preserving-module-recruitment-goal.json`
and this closure's goal summary, `deterministic.goal_indicators.recruitment_paths.arms[0..18]`:
arm order/name/policy identical (`arms/0` is still `graph_blank`/`Drift`);
`outcome_counts`, `proposal_count`, `opportunities`, `lineages`,
`selected_inapplicable_backend_unresolved`,
`selected_inapplicable_by_backend_operator` are byte-identical for all 18
arms. `summary` and each of the 4 entries of `batches` (72 batch dicts
total across the 18 arms) were each projected separately: every
pre-existing field (`retained_discovery`, `retained_useful`,
`proposal_discovery`, `discovery_depth_range`,
`viable_retained_discovery`, and, in `batches`,
`retention_among_discoverers`) is byte-identical with no numerator,
denominator or Wilson interval moved; both `summary` and every `batches`
entry gained exactly the same five new keys (`checkpoint_cost`, `damage`,
`retention_outcomes`, `time_to_first_proposal`, `time_to_first_retained`)
and no others. Confirmed programmatically (Python diff over both JSON
files).

## Twenty-seven-arm table

`total_proposals` = 82,944 (27 arms × 3,072); 171 pairs (up from F05's 73,
+98 = 15 Task A + 12 Task B pairs added). Columns: proposal-discovery,
retained-discovery, retained-useful numerators over denominator 32.

| # | Form | Policy | Task | Proposal disc. | Retained disc. | Retained useful |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | graph_blank | Drift | A | 0/32 | 0/32 | 0/32 |
| 1 | graph_blank | Selection | A | 0/32 | 0/32 | 0/32 |
| 2 | graph_copy | Drift | A | 0/32 | 0/32 | 0/32 |
| 3 | graph_copy | Selection | A | 0/32 | 0/32 | 0/32 |
| 4 | graph_split | Drift | A | 0/32 | 0/32 | 0/32 |
| 5 | graph_split | Selection | A | 0/32 | 0/32 | 0/32 |
| 6 | vm_blank | Drift | A | 0/32 | 0/32 | 0/32 |
| 7 | vm_blank | Selection | A | 0/32 | 0/32 | 0/32 |
| 8 | vm_copy | Drift | A | 0/32 | 0/32 | 0/32 |
| 9 | vm_copy | Selection | A | 0/32 | 0/32 | 0/32 |
| 10 | graph_unprepared | Drift | B | 0/32 | 0/32 | 0/32 |
| 11 | graph_unprepared | Selection | B | 0/32 | 0/32 | 0/32 |
| 12 | graph_prepared | Drift | B | 17/32 | 13/32 | 2/32 |
| 13 | graph_prepared | Selection | B | 17/32 | 17/32 | 16/32 |
| 14 | vm_unprepared | Drift | B | 0/32 | 0/32 | 0/32 |
| 15 | vm_unprepared | Selection | B | 0/32 | 0/32 | 0/32 |
| 16 | vm_prepared | Drift | B | 18/32 | 14/32 | 1/32 |
| 17 | vm_prepared | Selection | B | 18/32 | 18/32 | 15/32 |
| 18 | graph_blank | CostSelection | A | 0/32 | 0/32 | 0/32 |
| 19 | graph_copy | CostSelection | A | 0/32 | 0/32 | 0/32 |
| 20 | graph_split | CostSelection | A | 0/32 | 0/32 | 0/32 |
| 21 | vm_blank | CostSelection | A | 0/32 | 0/32 | 0/32 |
| 22 | vm_copy | CostSelection | A | 0/32 | 0/32 | 0/32 |
| 23 | graph_unprepared | CostSelection | B | 0/32 | 0/32 | 0/32 |
| 24 | graph_prepared | CostSelection | B | 17/32 | 17/32 | 17/32 |
| 25 | vm_unprepared | CostSelection | B | 0/32 | 0/32 | 0/32 |
| 26 | vm_prepared | CostSelection | B | 17/32 | 17/32 | 17/32 |

Rows 0–17 equal the byte-identity check above. Rows 18–26 (`CostSelection`)
match the predeclaration: Task A and unprepared Task B arms are all 0/32;
`graph_prepared`/`vm_prepared` discover at 17/32 (at or near the Selection
arms' 17–18/32) with all 17 discoveries retained and useful — no floor was
predeclared for this row, reported as measured.

**Checkpoint-48 cost, `CostSelection` vs `Selection` (prepared forms only,
median over lineages):**

| Form | Policy | genome_size median | carrying_sum median |
| --- | --- | --- | --- |
| graph_prepared | Selection | 41.0 | 0.032806 |
| graph_prepared | CostSelection | 19.5 | 0.015594 |
| vm_prepared | Selection | 56.5 | 0.045212 |
| vm_prepared | CostSelection | 31.0 | 0.024811 |

`CostSelection` reads at or below `Selection` for both prepared forms on
both fields, as predeclared (no floor).

**Time to first contribution (median generation, `CostSelection` prepared
arms):** `graph_prepared` proposal 4.0 / retained 4.0; `vm_prepared`
proposal 4.0 / retained 4.0 (`Drift`/`Selection` values for the same forms:
`graph_prepared` Drift 4.0/5.0, Selection 4.0/4.0; `vm_prepared` Drift
5.0/5.5, Selection 5.0/5.0). All non-prepared arms are censored (`None`,
never discovered).

## Depth-2,000 drift readings, per world

`deterministic.goal_indicators.cases[i].drift_depth.readings[].changed_per_all_births`
at `depth` 1000 and 2000 (field read directly from the summary, not
derived):

| World (seed) | Depth 1,000 `changed_per_all_births` | Depth 2,000 `changed_per_all_births` |
| --- | --- | --- |
| Orchards in grassland (11) | 0.004500 | 0.003500 |
| Canyon country (22) | 0.008500 | 0.004500 |
| Confluence (33) | 0.004500 | 0.003500 |

(Equivalently 9/2,000, 17/2,000, 9/2,000 at depth 1,000 and 7/2,000,
9/2,000, 7/2,000 at depth 2,000, `births_total` = 2,000.)

The full `drift_depth` block (all checkpoints, all fields) is byte-identical
to T13.F05's goal summary for all three cases — confirmed programmatically.
Depth-2,000 readings (0.0035 / 0.0045 / 0.0035) are below the T11-track
re-based 0.005 floor, exactly as F05's report read. Per the spec's own
predeclaration text ("An identical reading is not a regression against the
previous closure; it is still below the floor... the reading is escalated
for the user's decision at closure, not assumed accepted"), **this floor
miss is escalated, not resolved by this run.**

## Byte-identical-to-F05 blocks (verified, no measured value beyond identity)

Compared field-by-field, goal summary vs F05 goal summary:

- Founder and evolved `mutational_neighborhood` (all 3 cases): identical.
- `reachable_structure_size_distribution` (all 3 cases): identical.
- Diversity/cognition indicator blocks (`lineage_diversity`, `strategy_count`,
  `strategy_causal_distinctness`, `adaptive_novelty`, `learning_dependence`,
  `memory_dependence`, `memory_sensitivity`, `temporal_memory_sensitivity`,
  `prediction_dependence`, `information_integration`,
  `reciprocal_interaction`, `structural_companions`,
  `evolutionary_activity`, `population_persistence`,
  `births_per_100_ticks`): identical.

## Recruitment-paths experiment wall time

`environment.recruitment_paths_wall_clock_ms` = 10,275.41 ms (10.275 s),
below the predeclared 12–14 s expectation range and well inside the 120 s
cap; not a regression. (F05's comparable single-experiment figure was
8,315 ms for 18 arms; this closure's 27-arm run at 10.275 s is a +23.6%
wall increase for a 50% arm-count increase — reported, no floor.)

## Series entries

Added `docs/progress/features/t13-f06-recruitment-and-retention-qualification.json`
to `gate.closed` and
`docs/progress/features/t13-f06-recruitment-and-retention-qualification-goal.json`
to `goal_worlds.closed` in `docs/progress/benchmark-series.json`, following
the same series used for T13.F05 (gate epoch
`remove-complementary-nutrition.json`; goal-worlds epoch
`t13-f03-mutation-target-applicability-goal.json`).

## Items not in this pass

Per the benchmark-specialist brief, the fixture per-step `CostSelection`
verdicts (`QualifiedPath::cost_verdicts`) and the ecological reading table
(spec lines 119–126: `energy_flows.genome_carrying` vs `lifecycle_decay`,
mean carried units per creature-tick, `mesh_summary`, drift-walk
`recruitment`/retention/`time_to_first` rows) were not run or compiled here;
they require a test run or a derivation this benchmark pass was not asked to
perform. They are owned by the implementer/orchestrator, not resolved in
this file.

## Measured verdict summary

Gate: exit 0, `severe=false`, no wall/work flags against either reference,
six counters byte-identical to F05. Goal: exit 0, `severe=false`, no
wall/work flags against either reference (the wall-clock flag F05 recorded
did not recur here), six counters byte-identical to F05, all
byte-identical-to-F05 blocks confirmed identical, the eighteen F02 arms
byte-identical, the nine new `CostSelection` arms and 98 new pairs read as
predeclared. One escalation carried forward unchanged from F05: the
depth-2,000 drift readings (0.0035/0.0045/0.0035) are below the 0.005 floor
and are, per the spec's own text, escalated rather than assumed accepted at
this closure too.
