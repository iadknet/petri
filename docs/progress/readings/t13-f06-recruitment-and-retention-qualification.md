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

**Retention at discovery + 16 and damage (prepared forms,
`summary.retention_outcomes` and `summary.damage`, arms 13 / 17 / 24 / 26):**

| Form | Policy | Useful / no longer useful / task-dead / deleted | Task-dead damage | Task-live loss |
| --- | --- | --- | --- | --- |
| graph_prepared | Selection | 16 / 1 / 0 / 0 | 0/3,072 | 148/3,072 |
| vm_prepared | Selection | 15 / 3 / 0 / 0 | 1/3,072 | 149/3,071 |
| graph_prepared | CostSelection | 17 / 0 / 0 / 0 | 0/3,072 | 166/3,072 |
| vm_prepared | CostSelection | 17 / 0 / 0 / 0 | 0/3,072 | 177/3,072 |

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
re-based 0.005 floor, exactly as F05's report read. Per the spec's
predeclaration the reading was escalated rather than assumed accepted; the
user accepted it for this feature on 2026-09-14 ("I approve the same floor
violation from previous features"), matching the T13.F03–F05 acceptances.
The floor and the goal-worlds epoch are unchanged.

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

## Fixture per-step `CostSelection` verdicts

Transcript of `cargo test -p v3-core --lib
recruitment_paths_qualified_cost_verdicts -- --nocapture` at the tested
commit (`QualifiedPath::cost_verdicts`; test
`recruitment_paths_qualified_cost_verdicts_retain_every_useful_last_step`,
1 passed). The maintained qualified-path fixture family (`*_blank`,
`*_copy`, `graph_split`, `*_unprepared`, `*_detour`) is the T13.F05 set,
not the 27 assay arms; the `*_prepared` arms have no fixture and the
`*_detour` fixtures have no arm. Each row is one path step judged as a
single-candidate `CostSelection` choice against its predecessor stage
(`QualifiedPath::cost_verdicts`): `retained` is the verdict, `score` is the
task score of the predecessor -> the step, `carrying` is the step's
`TaskSummary::carrying_sum` (the genome-carrying charge summed over the
task's scenes), `energy` is `TaskSummary::ending_energy_sum` (ending
creature energy summed over the scenes) of the predecessor -> the step. A
neutral step (score unchanged) is retained only when its summed ending
energy does not fall below the predecessor's; a higher score is retained
regardless of energy.

| Form | Step | Retained | Score | Carrying | Energy before -> after |
| --- | --- | --- | --- | --- | --- |
| graph_blank | cue_added | false | 4->4 | 0.0120 | 395.5886->395.5878 |
| graph_blank | slot_emits_move | true | 4->4 | 0.0120 | 395.5878->395.5878 |
| graph_blank | direction_node | false | 4->4 | 0.0136 | 395.5878->395.5862 |
| graph_blank | direction_doubled | false | 4->4 | 0.0144 | 395.5862->395.5854 |
| graph_blank | direction_read | false | 4->4 | 0.0160 | 395.5854->395.5838 |
| graph_blank | gate_edge_added | false | 4->4 | 0.0168 | 395.5838->395.5830 |
| graph_blank | activated | true | 4->8 | 0.0168 | 395.5830->394.9831 |
| graph_copy | gate_edge_added | false | 4->4 | 0.0184 | 395.5822->395.5814 |
| graph_copy | activated | true | 4->8 | 0.0184 | 395.5814->394.9814 |
| graph_split | gate_edge_added | false | 4->4 | 0.0200 | 395.5806->395.5798 |
| graph_split | activated | true | 4->8 | 0.0200 | 395.5798->394.9797 |
| vm_blank | cue_added | false | 4->4 | 0.0176 | 395.5832->395.5824 |
| vm_blank | read_cue | false | 4->4 | 0.0184 | 395.5824->395.5816 |
| vm_blank | skip_when_zero | false | 4->4 | 0.0192 | 395.5816->395.5808 |
| vm_blank | double_to_east | false | 4->4 | 0.0200 | 395.5808->395.5800 |
| vm_blank | write_direction | false | 4->4 | 0.0208 | 395.5800->395.5792 |
| vm_blank | push_move | false | 4->4 | 0.0216 | 395.5792->395.5784 |
| vm_blank | activated | true | 4->8 | 0.0216 | 395.5784->394.9783 |
| vm_copy | leading_halt_removed | true | 4->4 | 0.0264 | 395.5728->395.5736 |
| vm_copy | activated | true | 4->8 | 0.0264 | 395.5736->394.9736 |
| graph_unprepared | cue_swapped | true | 4->4 | 0.0200 | 394.9798->394.9798 |
| graph_unprepared | cue_edge_retargeted | true | 4->4 | 0.0200 | 394.9798->394.9798 |
| graph_unprepared | direction_node | false | 4->4 | 0.0216 | 394.9798->394.9782 |
| graph_unprepared | direction_doubled | false | 4->4 | 0.0224 | 394.9782->394.9774 |
| graph_unprepared | direction_read | true | 4->4 | 0.0224 | 394.9774->394.9774 |
| graph_unprepared | activated | true | 4->8 | 0.0224 | 394.9774->394.9773 |
| vm_unprepared | cue_swapped | true | 4->4 | 0.0264 | 394.9736->394.9736 |
| vm_unprepared | read_sub_idx_1 | true | 4->4 | 0.0264 | 394.9736->394.9736 |
| vm_unprepared | read_sub_idx_2 | true | 4->4 | 0.0264 | 394.9736->394.9736 |
| vm_unprepared | direction_half | true | 4->4 | 0.0264 | 394.9736->394.9736 |
| vm_unprepared | direction_east | true | 4->4 | 0.0264 | 394.9736->394.9736 |
| vm_unprepared | activated | true | 4->8 | 0.0264 | 394.9736->394.9736 |
| graph_detour | cue_added | false | 4->4 | 0.0120 | 395.5886->395.5878 |
| graph_detour | slot_emits_move | true | 4->4 | 0.0120 | 395.5878->395.5878 |
| graph_detour | direction_node | false | 4->4 | 0.0136 | 395.5878->395.5861 |
| graph_detour | direction_doubled | false | 4->4 | 0.0144 | 395.5861->395.5853 |
| graph_detour | direction_read | false | 4->4 | 0.0160 | 395.5853->395.5837 |
| graph_detour | gate_edge_added | true | 4->8 | 0.0168 | 395.5837->394.9829 |
| vm_detour | cue_added | false | 4->4 | 0.0176 | 395.5832->395.5824 |
| vm_detour | read_cue | false | 4->4 | 0.0184 | 395.5824->395.5816 |
| vm_detour | double_to_east | false | 4->4 | 0.0192 | 395.5816->395.5808 |
| vm_detour | write_direction | false | 4->4 | 0.0200 | 395.5808->395.5800 |
| vm_detour | skip_when_zero | false | 4->4 | 0.0208 | 395.5800->395.5792 |
| vm_detour | push_move | true | 4->8 | 0.0216 | 395.5792->394.9783 |

Every useful last step (score 4->8) is retained on both backends, which is
the test's assertion. The neutral-step barrier reading, by form and backend
(rejected / neutral steps; a neutral step whose ending energy falls is
rejected, one whose ending energy holds or rises is retained):

| Backend | Form | Neutral steps rejected |
| --- | --- | --- |
| Graph | graph_blank | 5 of 6 |
| Graph | graph_copy | 1 of 1 |
| Graph | graph_detour | 4 of 5 |
| Graph | graph_split | 1 of 1 |
| Graph | graph_unprepared | 2 of 5 |
| Graph | total | 13 of 18 |
| Vm | vm_blank | 6 of 6 |
| Vm | vm_copy | 0 of 1 |
| Vm | vm_detour | 5 of 5 |
| Vm | vm_unprepared | 0 of 5 |
| Vm | total | 11 of 17 |

Recorded, not scored: under a cost-visible single-candidate selection
every neutral step that lowers ending energy is rejected and only the
energy-neutral rewiring steps and the one energy-raising step
(`vm_copy` `leading_halt_removed`) survive, so 24 of the 35 neutral fixture
steps would not pass a cost-visible selection one at a time.

## Goal-worlds ecological reading

Drawn from existing report fields of the goal summary
(`docs/progress/features/t13-f06-recruitment-and-retention-qualification-goal.json`),
no new instrument. Two limits apply and neither side substitutes for the
other: the drift walk (`deterministic.goal_indicators.cases[i].drift_depth`)
has no energy and no selection, so its recruitment rungs are exposure and
retention under neutral drift only; the evolved population (`energy_flows`,
`mortality`, `reachable_structure_size_distribution`, the evolved
`mesh_summary`) has both energy and selection but no module attribution, so
nothing in it says which module paid or contributed. Every spec-named field
exists in the summary: 0 nulls, no owning-feature mapping needed. The only
missing values are censoring inside the drift walk — the depth-0 checkpoint
has no cohort (all rungs `Undefined`, medians `null`) and the depth-22
`retention` row measures from depth 0 (`Undefined`) — and they are recorded
as such below.

**Energy.** `energy_flows.genome_carrying` and `lifecycle_decay` are the
world totals over 2,000 ticks; `creature_ticks` is
`deterministic.per_seed[].creature_ticks`; mean carried units per
creature-tick is `genome_carrying / (1e-4 × creature_ticks)` per the spec
(the direct `energy_flows.genome_size_creature_ticks / creature_ticks`
agrees to six significant figures: 146.328121 / 160.226983 / 158.164621).
Deaths are `mortality.by_cause.genome_carrying` and `lifecycle_decay` over
`deaths_total`.

| World (seed) | `genome_carrying` | `lifecycle_decay` | carrying / decay | `creature_ticks` | Mean carried units per creature-tick | Deaths carrying / decay / total |
| --- | --- | --- | --- | --- | --- | --- |
| Orchards in grassland (11) | 301,103.797511 | 10,428,656.000163 | 0.028873 | 20,577,185 | 146.33 | 7,529 / 272,598 / 382,698 |
| Canyon country (22) | 309,198.433738 | 9,779,562.500030 | 0.031617 | 19,297,441 | 160.23 | 7,121 / 254,563 / 369,392 |
| Confluence (33) | 308,805.258702 | 9,888,246.000091 | 0.031230 | 19,524,180 | 158.17 | 7,301 / 245,011 / 360,001 |

Carrying is about 3% of lifecycle decay in every world, the mean carried
genome is 146–160 units, and ≈2% of deaths are attributed to the carrying
charge.

**Structure.** `reachable_structure_size_distribution` (final population)
and the evolved `mutational_neighborhood.evolved.per_seed[0].mesh_summary`
(12 sampled genomes, 960 executions):

| World (seed) | Reachable size min / p25 / median / mean / p75 / max | Evolved mesh executed / knockout / reachable / total |
| --- | --- | --- |
| Orchards in grassland (11) | 1 / 74 / 90 / 100.45 / 118 / 421 | 38 / 12 / 47 / 68 |
| Canyon country (22) | 24 / 70 / 84 / 101.35 / 117 / 379 | 35 / 11 / 49 / 56 |
| Confluence (33) | 4 / 75 / 98 / 106.84 / 129 / 319 | 33 / 10 / 45 / 57 |

**Drift walk recruitment rungs**
(`drift_depth.readings[].recruitment`, 50 lineages, cohort = modules created
after depth 0). The Orchards (11) and Confluence (33) `recruitment` blocks
are identical at every checkpoint, as observed; the same identity appears in
the two worlds' T13.F05 drift readings (0.0045 / 0.0035 at both). Cohort
rungs at each checkpoint (created / present / contributing, with the
contributing fraction), split by backend:

| World (seed) | Depth | Created | Present | Contributing (fraction) | Graph created / contributing | VM created / contributing |
| --- | --- | --- | --- | --- | --- | --- |
| Orchards (11) = Confluence (33) | 22 | 72 | 70 | 8 (0.114286) | 37 / 2 | 35 / 6 |
| Orchards (11) = Confluence (33) | 250 | 796 | 744 | 25 (0.033602) | 403 / 2 | 393 / 23 |
| Orchards (11) = Confluence (33) | 1,000 | 3,649 | 3,427 | 28 (0.008170) | 1,823 / 0 | 1,826 / 28 |
| Orchards (11) = Confluence (33) | 2,000 | 7,587 | 7,112 | 23 (0.003234) | 3,740 / 0 | 3,847 / 23 |
| Canyon country (22) | 22 | 77 | 75 | 9 (0.120000) | 41 / 2 | 36 / 7 |
| Canyon country (22) | 250 | 880 | 832 | 21 (0.025240) | 463 / 4 | 417 / 17 |
| Canyon country (22) | 1,000 | 3,831 | 3,600 | 32 (0.008889) | 1,895 / 2 | 1,936 / 30 |
| Canyon country (22) | 2,000 | 7,883 | 7,431 | 32 (0.004306) | 3,802 / 0 | 4,081 / 32 |

Retention between checkpoints (`recruitment.retention`: modules
contributing at the previous checkpoint that still contribute, are present
but no longer contribute, or are deleted) and time to first contribution
(`recruitment.time_to_first`, `fact = contribution`: reached / cohort,
median generations, censored present / deleted):

| World (seed) | Depth | Retention from previous checkpoint: still / present-not / deleted (fraction) | Time to first contribution: reached (fraction), median gen., censored present / deleted |
| --- | --- | --- | --- |
| Orchards (11) = Confluence (33) | 22 | from 0: 0 / 0 / 0 (Undefined, no cohort at depth 0) | 8 (0.111111), 14, 62 / 2 |
| Orchards (11) = Confluence (33) | 250 | from 22: 2 / 6 / 0 (0.250000) | 31 (0.038945), 132, 713 / 52 |
| Orchards (11) = Confluence (33) | 1,000 | from 250: 2 / 22 / 1 (0.080000) | 57 (0.015621), 227, 3,371 / 221 |
| Orchards (11) = Confluence (33) | 2,000 | from 1,000: 5 / 21 / 2 (0.178571) | 74 (0.009754), 308, 7,041 / 472 |
| Canyon country (22) | 22 | from 0: 0 / 0 / 0 (Undefined, no cohort at depth 0) | 9 (0.116883), 14, 66 / 2 |
| Canyon country (22) | 250 | from 22: 1 / 7 / 1 (0.111111) | 29 (0.032955), 107, 804 / 47 |
| Canyon country (22) | 1,000 | from 250: 3 / 17 / 1 (0.142857) | 58 (0.015140), 188, 3,544 / 229 |
| Canyon country (22) | 2,000 | from 1,000: 7 / 25 / 0 (0.218750) | 83 (0.010529), 240, 7,351 / 449 |

The earlier `time_to_first` facts at depth 2,000 (selection /
applicable_selection / internal_change / dispatch) reach 0.593 / 0.593 /
0.635 / 0.244 of the cohort in Orchards and Confluence and 0.563 / 0.563 /
0.611 / 0.226 in Canyon country; the contribution rung is censored for
about 99% of the cohort. Under drift, Graph cohort contribution is 0 at
depth 2,000 in all three worlds (0 from depth 1,000 in Orchards and
Confluence, 2 at depth 1,000 in Canyon country) while VM modules carry
every remaining contribution; between-checkpoint retention of a
contributing module is 8–25%. These are drift-walk exposure readings with
no energy and no selection; the assay's `CostSelection` retention (17/17
useful at discovery + 16 for both prepared forms) is a bound under
selection with production charges, and neither reading is an emergence or
cognition claim.

## Measured verdict summary

Gate: exit 0, `severe=false`, no wall/work flags against either reference,
six counters byte-identical to F05. Goal: exit 0, `severe=false`, no
wall/work flags against either reference (the wall-clock flag F05 recorded
did not recur here), six counters byte-identical to F05, all
byte-identical-to-F05 blocks confirmed identical, the eighteen F02 arms
byte-identical, the nine new `CostSelection` arms and 98 new pairs read as
predeclared. The depth-2,000 drift readings (0.0035/0.0045/0.0035) are
below the 0.005 floor, unchanged from F05, and the user accepted that miss
for this feature on 2026-09-14 with the floor and epoch unchanged. The
fixture `CostSelection` verdicts and the goal-worlds ecological reading
above complete the readings the spec names.
