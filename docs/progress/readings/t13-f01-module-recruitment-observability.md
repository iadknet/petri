# T13.F01 — module recruitment observability: measured readings

Measured evidence for
[`docs/specs/roadmap/t13-f01-module-recruitment-observability.md`](../../specs/roadmap/t13-f01-module-recruitment-observability.md).
The spec keeps the predeclaration, the verdict, and the mutation survivor
record. Machine-written reports:
[gate](../features/t13-f01-module-recruitment-observability.json),
[goal](../features/t13-f01-module-recruitment-observability-goal.json).
Measured code `ab00bb07f9a556f5a345de9d8ce0890d1b6f4941` (the self-review
commit), host `Isaacs-MacBook-Pro-2.local`, Apple M1 Pro, 8 threads, release.

**What these numbers are and are not.** Every reading here is observation of
the mutation-only drift walk. Dispatch is not an effect, a *contribution* is
battery-signature sensitivity under the static-successor bypass only, and
usefulness is unmeasured. Nothing here is a cognition claim.

## Focused tests

Re-run at `ab00bb07` after the self-review commit, all green:

- `cargo test -p v3-core --test viability` — `ok, 24 passed; 0 failed`.
- `cargo test -p v3-core` — 1277 + 24 + 19 + 13 + 10 + 7 + 4 + 3 + 2 + 1 + 1
  passed, 0 failed, 3 ignored, across every target.
- `cargo test -p v3-cli` — 70 + 18 + 11 + 11 passed, 0 failed.
- `cargo clippy -p v3-core -p v3-cli --all-targets -- -D warnings` — clean.

The named tests these cover:
`a_selected_module_with_no_applicable_site_is_recorded_with_its_node_id`,
`an_event_with_no_eligible_node_records_no_target`,
`an_applied_event_records_the_node_the_genome_carried_before_it`,
`event_records_agree_with_the_totals_and_the_operator_funnels`,
`mesh_execution_sets_carry_the_ids_behind_the_unchanged_counts`,
`an_id_reused_after_deletion_is_two_modules`,
`copy_provenance_needs_both_a_copy_event_and_matching_content`,
`the_censoring_split_keeps_every_module_in_the_denominator`,
`a_selected_inapplicable_event_is_separated_from_a_missing_eligible_node`,
`retention_splits_the_previous_checkpoints_contributors`,
`cohort_readings_partition_every_module_they_count`,
`recruitment_readings_track_the_walk_without_changing_it`, and
`drift_checkpoint_reports_recruitment_and_opportunities_and_still_loads_older_reports`.

## Gate report

`make bench PROFILE=gate FEATURE=t13-f01-module-recruitment-observability`
exited 0; `comparison.severe=false`. Stored
`t13-f01-module-recruitment-observability.json`, generated 2026-09-10T14:40:24Z.
Log `/tmp/t13-f01-gate.log`. Run once.

| Reference | Measure | Current | Reference reading | Delta % | Result |
| --- | --- | --- | --- | --- | --- |
| remove-complementary-nutrition | mesh_hops | 2.028954 | 2.027261 | 0.083512 | ok |
| remove-complementary-nutrition | vm_steps | 22.425973 | 22.751316 | -1.429996 | ok |
| remove-complementary-nutrition | graph_relax_iters | 0.994920 | 0.995234 | -0.031550 | ok |
| remove-complementary-nutrition | plasticity_updates | 0.009880 | 0.011171 | -11.556709 | ok |
| remove-complementary-nutrition | actions_applied | 1.272860 | 1.275525 | -0.208934 | ok |
| remove-complementary-nutrition | births | 0.026902 | 0.026780 | 0.455564 | ok |
| remove-complementary-nutrition | wall ms/creature-tick | 0.0014120875 | 0.0015598310 | -9.471763 | ok |
| t11-f18-backend-neutral-mesh-node-growth | mesh_hops | 2.028954 | 2.028954 | 0.000000 | ok |
| t11-f18-backend-neutral-mesh-node-growth | vm_steps | 22.425973 | 22.425973 | 0.000000 | ok |
| t11-f18-backend-neutral-mesh-node-growth | graph_relax_iters | 0.994920 | 0.994920 | 0.000000 | ok |
| t11-f18-backend-neutral-mesh-node-growth | plasticity_updates | 0.009880 | 0.009880 | 0.000000 | ok |
| t11-f18-backend-neutral-mesh-node-growth | actions_applied | 1.272860 | 1.272860 | 0.000000 | ok |
| t11-f18-backend-neutral-mesh-node-growth | births | 0.026902 | 0.026902 | 0.000000 | ok |
| t11-f18-backend-neutral-mesh-node-growth | wall ms/creature-tick | 0.0014120875 | 0.0019781652 | -28.616300 | ok |

The harness compares the gate profile against the epoch and T11.F18, not
T12.F04. Hand-computed against T12.F04's gate report and carrying no harness
level: wall 0.0014120875 vs 0.0013002215 ms/creature-tick, +8.603613%.

Observation times (ms): founder 48.739; evolved and drift are not measured in
the gate profile (`drift_depth: "Undefined"`). Whole measured run 603.732 ms.
Caps unchanged: founder 10 s, evolved 180 s, drift 30 s, whole goal
investigation 900 s.

## Goal report

`make bench PROFILE=goal FEATURE=t13-f01-module-recruitment-observability`
exited 0; `comparison.severe=false`. Stored
`t13-f01-module-recruitment-observability-goal.json`, generated
2026-09-10T14:49:08Z. Log `/tmp/t13-f01-goal.log`. Run once, sequentially after
the gate run with nothing else running.

| Reference | Measure | Current | Reference reading | Delta % | Result |
| --- | --- | --- | --- | --- | --- |
| t12-f04-baseline-world-set-goal | mesh_hops | 2.247936 | 2.247936 | 0.000000 | ok |
| t12-f04-baseline-world-set-goal | vm_steps | 23.339759 | 23.339759 | 0.000000 | ok |
| t12-f04-baseline-world-set-goal | graph_relax_iters | 1.028540 | 1.028540 | 0.000000 | ok |
| t12-f04-baseline-world-set-goal | plasticity_updates | 0.046976 | 0.046976 | 0.000000 | ok |
| t12-f04-baseline-world-set-goal | actions_applied | 1.371176 | 1.371176 | 0.000000 | ok |
| t12-f04-baseline-world-set-goal | births | 0.018609 | 0.018609 | 0.000000 | ok |
| t12-f04-baseline-world-set-goal | wall ms/creature-tick | 0.0078817109 | 0.0074066599 | 6.413835 | ok |

Simulation wall 490.30 s total (184.69 / 124.52 / 181.09 s for seeds
11 / 22 / 33). End-to-end elapsed 518 s, measured as the interval between the
goal log's creation (07:40:30 local) and the report write (07:49:08 local),
against the 15-minute investigation threshold — under it, no investigation
triggered.

Observation times: founder 109.550 ms (cap 10 s); evolved 461.861 ms total,
168.288 / 160.340 / 133.233 ms per seed (cap 180 s); final-state observation
898.569 ms.

**Drift wall time.** The harness records a single accumulated
`drift_depth_wall_clock_ms` across the three worlds and no per-world split
exists in the report: 25,004.812 ms. Because the total is 25.00 s, no single
world can exceed 25.00 s, so every world is under the 30 s cap; the per-world
average is 8.33 s. T12.F04 recorded 11,179.458 ms for the same three worlds, so
the recruitment observation added 13.83 s in total, an average of +4.61 s per
world. **The predeclaration expected "under 2 s added per world"; the cap holds
but that expectation was missed.** Note, not an explanation and not acted on
here: the walk now clones each parent's `nodes` per birth and compares node
genomes for equality over every surviving node, across 100,000 births per
world.

## Structural comparison against T12.F04

Method: a recursive leaf diff over the whole `deterministic` block of each
report against T12.F04's, printing every value difference and every key present
on only one side, with no filtering; classification afterwards.

- **Gate**, against `t12-f04-baseline-world-set.json`: **0 difference lines.**
  Every deterministic field is equal. Note that the gate profile carries
  `drift_depth: "Undefined"`, so this comparison covers fewer fields than the
  goal one — no drift, recruitment, or opportunity field exists in it. Equal
  deterministic blocks mean the six gate counters, which are derived from the
  equal `per_creature_tick` and `totals` fields, equal T12.F04's exactly.
- **Goal**, against `t12-f04-baseline-world-set-goal.json`: **42 difference
  lines**, in four groups and nothing else:
  1. 15 × `goal_indicators.cases[i].drift_depth.readings[j].recruitment` —
     present only in the current report (the new recruitment block).
  2. 15 × `…readings[j].opportunities` — present only in the current report
     (the new opportunity block).
  3. 3 × `…drift_depth.version`: `"drift-depth-v3"` vs `"drift-depth-v2"`, one
     per world (an allowed exclusion).
  4. 9 lines — 3 worlds × `drift_depth.recruitment_version`,
     `drift_depth.module_identity`, `drift_depth.provenance_rule` — present
     only in the current report. These are new top-level `drift_depth`
     contract strings and therefore fall **outside** the three named
     exclusions; they are reported here as their own group rather than folded
     into the recruitment block.

**Result: no pre-existing deterministic field of either report differs from
T12.F04's.** Every difference is a field that did not exist before
`drift-depth-v3`, plus the version string itself.

## Drift floors

Per-world drift readings, identical to T12.F04's:

| World / seed | depth-1,000 changed/all births (floor 0.0015) | depth-2,000 changed/all births (floor 0.005) | dead/all births 1,000 / 2,000 | hop-cap hits |
| --- | --- | --- | --- | --- |
| Orchards in grassland / 11 | 0.004500 — meets | 0.006000 — meets | 0.000500 / 0.004000 | 0 |
| Canyon country / 22 | 0.010000 — meets | 0.005000 — meets, equal to the floor | 0.001000 / 0.001500 | 0 |
| Confluence / 33 | 0.004500 — meets | 0.006000 — meets | 0.000500 / 0.004000 | 0 |

No movement is claimed: the predeclaration said no indicator would move, and
the structural comparison confirms these fields are byte-identical to T12.F04.

## The selected-but-inapplicable split reads zero everywhere

Across all three worlds and all five checkpoints,
`selected_inapplicable_by_domain` is empty — **0 events out of 55,204 /
55,142 / 55,204 cumulative attempts per world** — and the `selected only` rung
of the ladder is **0 for every cohort, backend, world, and checkpoint**, so
`selection` and `applicable_selection` have identical time-to-first rows
everywhere. The only `NoApplicableTarget` skips recorded are
`no_eligible_node_by_domain`: `{Graph: 11}` in Orchards and Confluence and
`{Graph: 2}` in Canyon, 72 events in total across every checkpoint of every
world.

This follows from the engine behaviour the spec's deviations entry records: the
engine swap-removes an operator that reports `NoApplicableTarget` and retries
the rest of its domain, so a skip only reaches the summary when *every*
operator of the drawn domain failed, and the accumulated first pick is `None`
whenever no operator of that domain ever selected a node. The split is a real,
measured zero at this baseline, and it bears directly on T13.F03.

## Per-world, per-checkpoint readings

Checkpoints are depths 0, 22, 250, 1,000 and 2,000, over 50 lineages per world;
`created == deleted + present`, the six ladder rungs partition `present`, and
`reached + censored_deleted + censored_present == created` in every
time-to-first row (one instance of that identity is printed under each table).
Opportunity totals are cumulative from depth 0 to the checkpoint, pooled over
all 50 lineages.

### Orchards in grassland (seed 11) — drift_depth `drift-depth-v3`, 50 lineages

#### depth 0

Founder generation: no recruited module exists yet (cohort created 0), founders created 100, present 100, dispatched 100, contributing 100; opportunities all zero over 0 births. changed_per_all_births 0.206500, dead_per_all_births 0.000500.

#### depth 22
changed_per_all_births 0.092000, dead_per_all_births 0.000500

| Cohort | created | deleted | present | never sel | sel only | applied only | changed only | dispatched-not-contrib | contributing | present/created | dispatched/present | contributing/present |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cohort | 78 | 2 | 76 | 32 | 0 | 0 | 13 | 24 | 7 | 0.974359 | 0.407895 | 0.092105 |
| graph | 45 | 2 | 43 | 22 | 0 | 0 | 5 | 15 | 1 | 0.955556 | 0.372093 | 0.023256 |
| vm | 33 | 0 | 33 | 10 | 0 | 0 | 8 | 9 | 6 | 1.000000 | 0.454545 | 0.181818 |

Founder reference row: created 100, deleted 1, present 99, dispatched 85, contributing 80 (contributing/present 0.808081).

| Time to first | reached | reached/created | median gens | censored deleted | censored present |
| --- | --- | --- | --- | --- | --- |
| selection | 16 | 0.205128 | 5 | 0 | 62 |
| applicable_selection | 16 | 0.205128 | 5 | 0 | 62 |
| internal_change | 28 | 0.358974 | 5 | 1 | 49 |
| dispatch | 32 | 0.410256 | 5 | 2 | 44 |
| contribution | 7 | 0.089744 | 12 | 2 | 69 |

Denominator identity, selection row: 16 + 0 + 62 = 78 created.

Retention from depth 0: 80/100 still contributing (0.800000), 19 present not contributing, 1 deleted.

Opportunities (cumulative): births 1100, zero-event 597 (0.542727), attempted 643, applied 643 (1.000000), skipped 0; executed-target 550 (0.855365 of applied), reachable 544, unreachable 37.
attempted_by_domain {'Graph': 172, 'InputRef': 176, 'Topology': 131, 'Vm': 164}; applied_by_domain {'Graph': 172, 'InputRef': 176, 'Topology': 131, 'Vm': 164}; selected_inapplicable_by_domain {}; no_eligible_node_by_domain {}.

#### depth 250
changed_per_all_births 0.013000, dead_per_all_births 0.000500

| Cohort | created | deleted | present | never sel | sel only | applied only | changed only | dispatched-not-contrib | contributing | present/created | dispatched/present | contributing/present |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cohort | 918 | 45 | 873 | 317 | 0 | 4 | 444 | 85 | 23 | 0.950980 | 0.123711 | 0.026346 |
| graph | 445 | 28 | 417 | 149 | 0 | 1 | 213 | 53 | 1 | 0.937079 | 0.129496 | 0.002398 |
| vm | 473 | 17 | 456 | 168 | 0 | 3 | 231 | 32 | 22 | 0.964059 | 0.118421 | 0.048246 |

Founder reference row: created 100, deleted 11, present 89, dispatched 39, contributing 20 (contributing/present 0.224719).

| Time to first | reached | reached/created | median gens | censored deleted | censored present |
| --- | --- | --- | --- | --- | --- |
| selection | 487 | 0.530501 | 32 | 0 | 431 |
| applicable_selection | 487 | 0.530501 | 32 | 0 | 431 |
| internal_change | 558 | 0.607843 | 24 | 22 | 338 |
| dispatch | 249 | 0.271242 | 14 | 37 | 632 |
| contribution | 28 | 0.030501 | 170 | 45 | 845 |

Denominator identity, selection row: 487 + 0 + 431 = 918 created.

Retention from depth 22: 22/87 still contributing (0.252874), 59 present not contributing, 6 deleted.

Opportunities (cumulative): births 12500, zero-event 6940 (0.555200), attempted 7001, applied 6990 (0.998429), skipped 11; executed-target 5143 (0.735765 of applied), reachable 5308, unreachable 994.
attempted_by_domain {'Graph': 1882, 'InputRef': 1870, 'Topology': 1424, 'Vm': 1825}; applied_by_domain {'Graph': 1871, 'InputRef': 1870, 'Topology': 1424, 'Vm': 1825}; selected_inapplicable_by_domain {}; no_eligible_node_by_domain {'Graph': 11}.

#### depth 1000
changed_per_all_births 0.004500, dead_per_all_births 0.000500

| Cohort | created | deleted | present | never sel | sel only | applied only | changed only | dispatched-not-contrib | contributing | present/created | dispatched/present | contributing/present |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cohort | 4117 | 208 | 3909 | 1485 | 0 | 18 | 2212 | 162 | 32 | 0.949478 | 0.049629 | 0.008186 |
| graph | 1919 | 110 | 1809 | 613 | 0 | 8 | 1094 | 92 | 2 | 0.942678 | 0.051962 | 0.001106 |
| vm | 2198 | 98 | 2100 | 872 | 0 | 10 | 1118 | 70 | 30 | 0.955414 | 0.047619 | 0.014286 |

Founder reference row: created 100, deleted 20, present 80, dispatched 5, contributing 5 (contributing/present 0.062500).

| Time to first | reached | reached/created | median gens | censored deleted | censored present |
| --- | --- | --- | --- | --- | --- |
| selection | 2271 | 0.551615 | 86 | 0 | 1846 |
| applicable_selection | 2271 | 0.551615 | 86 | 0 | 1846 |
| internal_change | 2508 | 0.609181 | 68 | 89 | 1520 |
| dispatch | 936 | 0.227350 | 38 | 169 | 3012 |
| contribution | 54 | 0.013116 | 234 | 207 | 3856 |

Denominator identity, selection row: 2271 + 0 + 1846 = 4117 created.

Retention from depth 250: 8/43 still contributing (0.186047), 33 present not contributing, 2 deleted.

Opportunities (cumulative): births 50000, zero-event 27756 (0.555120), attempted 27874, applied 27863 (0.999605), skipped 11; executed-target 19251 (0.690916 of applied), reachable 19528, unreachable 5605.
attempted_by_domain {'Graph': 7497, 'InputRef': 7436, 'Topology': 5678, 'Vm': 7263}; applied_by_domain {'Graph': 7486, 'InputRef': 7436, 'Topology': 5678, 'Vm': 7263}; selected_inapplicable_by_domain {}; no_eligible_node_by_domain {'Graph': 11}.

#### depth 2000
changed_per_all_births 0.006000, dead_per_all_births 0.004000

| Cohort | created | deleted | present | never sel | sel only | applied only | changed only | dispatched-not-contrib | contributing | present/created | dispatched/present | contributing/present |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cohort | 8318 | 425 | 7893 | 2921 | 0 | 52 | 4715 | 176 | 29 | 0.948906 | 0.025972 | 0.003674 |
| graph | 3903 | 217 | 3686 | 1255 | 0 | 25 | 2304 | 100 | 2 | 0.944402 | 0.027672 | 0.000543 |
| vm | 4415 | 208 | 4207 | 1666 | 0 | 27 | 2411 | 76 | 27 | 0.952888 | 0.024483 | 0.006418 |

Founder reference row: created 100, deleted 26, present 74, dispatched 5, contributing 4 (contributing/present 0.054054).

| Time to first | reached | reached/created | median gens | censored deleted | censored present |
| --- | --- | --- | --- | --- | --- |
| selection | 4666 | 0.560952 | 158 | 0 | 3652 |
| applicable_selection | 4666 | 0.560952 | 158 | 0 | 3652 |
| internal_change | 5163 | 0.620702 | 124 | 159 | 2996 |
| dispatch | 1947 | 0.234071 | 68 | 333 | 6038 |
| contribution | 78 | 0.009377 | 430 | 421 | 7819 |

Denominator identity, selection row: 4666 + 0 + 3652 = 8318 created.

Retention from depth 1000: 4/37 still contributing (0.108108), 32 present not contributing, 1 deleted.

Opportunities (cumulative): births 100000, zero-event 55878 (0.558780), attempted 55204, applied 55193 (0.999801), skipped 11; executed-target 37949 (0.687569 of applied), reachable 38094, unreachable 11705.
attempted_by_domain {'Graph': 14729, 'InputRef': 14699, 'Topology': 11144, 'Vm': 14632}; applied_by_domain {'Graph': 14718, 'InputRef': 14699, 'Topology': 11144, 'Vm': 14632}; selected_inapplicable_by_domain {}; no_eligible_node_by_domain {'Graph': 11}.

### Canyon country (seed 22) — drift_depth `drift-depth-v3`, 50 lineages

#### depth 0

Founder generation: no recruited module exists yet (cohort created 0), founders created 100, present 100, dispatched 100, contributing 100; opportunities all zero over 0 births. changed_per_all_births 0.201500, dead_per_all_births 0.000500.

#### depth 22
changed_per_all_births 0.095000, dead_per_all_births 0.000000

| Cohort | created | deleted | present | never sel | sel only | applied only | changed only | dispatched-not-contrib | contributing | present/created | dispatched/present | contributing/present |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cohort | 88 | 1 | 87 | 44 | 0 | 0 | 11 | 26 | 6 | 0.988636 | 0.367816 | 0.068966 |
| graph | 48 | 1 | 47 | 27 | 0 | 0 | 4 | 15 | 1 | 0.979167 | 0.340426 | 0.021277 |
| vm | 40 | 0 | 40 | 17 | 0 | 0 | 7 | 11 | 5 | 1.000000 | 0.400000 | 0.125000 |

Founder reference row: created 100, deleted 1, present 99, dispatched 87, contributing 83 (contributing/present 0.838384).

| Time to first | reached | reached/created | median gens | censored deleted | censored present |
| --- | --- | --- | --- | --- | --- |
| selection | 15 | 0.170455 | 5 | 0 | 73 |
| applicable_selection | 15 | 0.170455 | 5 | 0 | 73 |
| internal_change | 25 | 0.284091 | 5 | 1 | 62 |
| dispatch | 33 | 0.375000 | 4 | 1 | 54 |
| contribution | 6 | 0.068182 | 12 | 1 | 81 |

Denominator identity, selection row: 15 + 0 + 73 = 88 created.

Retention from depth 0: 83/100 still contributing (0.830000), 16 present not contributing, 1 deleted.

Opportunities (cumulative): births 1100, zero-event 612 (0.556364), attempted 630, applied 630 (1.000000), skipped 0; executed-target 549 (0.871429 of applied), reachable 542, unreachable 28.
attempted_by_domain {'Graph': 172, 'InputRef': 164, 'Topology': 128, 'Vm': 166}; applied_by_domain {'Graph': 172, 'InputRef': 164, 'Topology': 128, 'Vm': 166}; selected_inapplicable_by_domain {}; no_eligible_node_by_domain {}.

#### depth 250
changed_per_all_births 0.011000, dead_per_all_births 0.002500

| Cohort | created | deleted | present | never sel | sel only | applied only | changed only | dispatched-not-contrib | contributing | present/created | dispatched/present | contributing/present |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cohort | 869 | 34 | 835 | 290 | 0 | 5 | 424 | 90 | 26 | 0.960875 | 0.138922 | 0.031138 |
| graph | 414 | 21 | 393 | 122 | 0 | 1 | 221 | 48 | 1 | 0.949275 | 0.124682 | 0.002545 |
| vm | 455 | 13 | 442 | 168 | 0 | 4 | 203 | 42 | 25 | 0.971429 | 0.151584 | 0.056561 |

Founder reference row: created 100, deleted 12, present 88, dispatched 32, contributing 19 (contributing/present 0.215909).

| Time to first | reached | reached/created | median gens | censored deleted | censored present |
| --- | --- | --- | --- | --- | --- |
| selection | 485 | 0.558113 | 33 | 0 | 384 |
| applicable_selection | 485 | 0.558113 | 33 | 0 | 384 |
| internal_change | 539 | 0.620253 | 26 | 19 | 311 |
| dispatch | 244 | 0.280783 | 18 | 28 | 597 |
| contribution | 30 | 0.034522 | 170 | 33 | 806 |

Denominator identity, selection row: 485 + 0 + 384 = 869 created.

Retention from depth 22: 21/89 still contributing (0.235955), 61 present not contributing, 7 deleted.

Opportunities (cumulative): births 12500, zero-event 6992 (0.559360), attempted 6837, applied 6835 (0.999707), skipped 2; executed-target 5029 (0.735772 of applied), reachable 5108, unreachable 1049.
attempted_by_domain {'Graph': 1826, 'InputRef': 1786, 'Topology': 1378, 'Vm': 1847}; applied_by_domain {'Graph': 1824, 'InputRef': 1786, 'Topology': 1378, 'Vm': 1847}; selected_inapplicable_by_domain {}; no_eligible_node_by_domain {'Graph': 2}.

#### depth 1000
changed_per_all_births 0.010000, dead_per_all_births 0.001000

| Cohort | created | deleted | present | never sel | sel only | applied only | changed only | dispatched-not-contrib | contributing | present/created | dispatched/present | contributing/present |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cohort | 3827 | 205 | 3622 | 1350 | 0 | 16 | 2075 | 145 | 36 | 0.946433 | 0.049972 | 0.009939 |
| graph | 1792 | 97 | 1695 | 594 | 0 | 9 | 1024 | 67 | 1 | 0.945871 | 0.040118 | 0.000590 |
| vm | 2035 | 108 | 1927 | 756 | 0 | 7 | 1051 | 78 | 35 | 0.946929 | 0.058640 | 0.018163 |

Founder reference row: created 100, deleted 19, present 81, dispatched 7, contributing 4 (contributing/present 0.049383).

| Time to first | reached | reached/created | median gens | censored deleted | censored present |
| --- | --- | --- | --- | --- | --- |
| selection | 2189 | 0.571989 | 83 | 0 | 1638 |
| applicable_selection | 2189 | 0.571989 | 83 | 0 | 1638 |
| internal_change | 2358 | 0.616148 | 68 | 86 | 1383 |
| dispatch | 962 | 0.251372 | 48 | 162 | 2703 |
| contribution | 62 | 0.016201 | 229 | 202 | 3563 |

Denominator identity, selection row: 2189 + 0 + 1638 = 3827 created.

Retention from depth 250: 5/45 still contributing (0.111111), 36 present not contributing, 4 deleted.

Opportunities (cumulative): births 50000, zero-event 27886 (0.557720), attempted 27593, applied 27591 (0.999928), skipped 2; executed-target 19235 (0.697148 of applied), reachable 19431, unreachable 5443.
attempted_by_domain {'Graph': 7326, 'InputRef': 7364, 'Topology': 5525, 'Vm': 7378}; applied_by_domain {'Graph': 7324, 'InputRef': 7364, 'Topology': 5525, 'Vm': 7378}; selected_inapplicable_by_domain {}; no_eligible_node_by_domain {'Graph': 2}.

#### depth 2000
changed_per_all_births 0.005000, dead_per_all_births 0.001500

| Cohort | created | deleted | present | never sel | sel only | applied only | changed only | dispatched-not-contrib | contributing | present/created | dispatched/present | contributing/present |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cohort | 7888 | 450 | 7438 | 2695 | 0 | 49 | 4458 | 202 | 34 | 0.942951 | 0.031729 | 0.004571 |
| graph | 3662 | 213 | 3449 | 1141 | 0 | 28 | 2180 | 99 | 1 | 0.941835 | 0.028994 | 0.000290 |
| vm | 4226 | 237 | 3989 | 1554 | 0 | 21 | 2278 | 103 | 33 | 0.943919 | 0.034094 | 0.008273 |

Founder reference row: created 100, deleted 21, present 79, dispatched 2, contributing 2 (contributing/present 0.025316).

| Time to first | reached | reached/created | median gens | censored deleted | censored present |
| --- | --- | --- | --- | --- | --- |
| selection | 4581 | 0.580756 | 152 | 0 | 3307 |
| applicable_selection | 4581 | 0.580756 | 152 | 0 | 3307 |
| internal_change | 4935 | 0.625634 | 125 | 189 | 2764 |
| dispatch | 1892 | 0.239858 | 66 | 354 | 5642 |
| contribution | 88 | 0.011156 | 456 | 444 | 7356 |

Denominator identity, selection row: 4581 + 0 + 3307 = 7888 created.

Retention from depth 1000: 7/40 still contributing (0.175000), 31 present not contributing, 2 deleted.

Opportunities (cumulative): births 100000, zero-event 55801 (0.558010), attempted 55142, applied 55140 (0.999964), skipped 2; executed-target 38026 (0.689626 of applied), reachable 38176, unreachable 11544.
attempted_by_domain {'Graph': 14723, 'InputRef': 14705, 'Topology': 10992, 'Vm': 14722}; applied_by_domain {'Graph': 14721, 'InputRef': 14705, 'Topology': 10992, 'Vm': 14722}; selected_inapplicable_by_domain {}; no_eligible_node_by_domain {'Graph': 2}.

### Confluence (seed 33) — drift_depth `drift-depth-v3`, 50 lineages

#### depth 0

Founder generation: no recruited module exists yet (cohort created 0), founders created 100, present 100, dispatched 100, contributing 100; opportunities all zero over 0 births. changed_per_all_births 0.206500, dead_per_all_births 0.000500.

#### depth 22
changed_per_all_births 0.092000, dead_per_all_births 0.000500

| Cohort | created | deleted | present | never sel | sel only | applied only | changed only | dispatched-not-contrib | contributing | present/created | dispatched/present | contributing/present |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cohort | 78 | 2 | 76 | 32 | 0 | 0 | 13 | 24 | 7 | 0.974359 | 0.407895 | 0.092105 |
| graph | 45 | 2 | 43 | 22 | 0 | 0 | 5 | 15 | 1 | 0.955556 | 0.372093 | 0.023256 |
| vm | 33 | 0 | 33 | 10 | 0 | 0 | 8 | 9 | 6 | 1.000000 | 0.454545 | 0.181818 |

Founder reference row: created 100, deleted 1, present 99, dispatched 85, contributing 80 (contributing/present 0.808081).

| Time to first | reached | reached/created | median gens | censored deleted | censored present |
| --- | --- | --- | --- | --- | --- |
| selection | 16 | 0.205128 | 5 | 0 | 62 |
| applicable_selection | 16 | 0.205128 | 5 | 0 | 62 |
| internal_change | 28 | 0.358974 | 5 | 1 | 49 |
| dispatch | 32 | 0.410256 | 5 | 2 | 44 |
| contribution | 7 | 0.089744 | 12 | 2 | 69 |

Denominator identity, selection row: 16 + 0 + 62 = 78 created.

Retention from depth 0: 80/100 still contributing (0.800000), 19 present not contributing, 1 deleted.

Opportunities (cumulative): births 1100, zero-event 597 (0.542727), attempted 643, applied 643 (1.000000), skipped 0; executed-target 550 (0.855365 of applied), reachable 544, unreachable 37.
attempted_by_domain {'Graph': 172, 'InputRef': 176, 'Topology': 131, 'Vm': 164}; applied_by_domain {'Graph': 172, 'InputRef': 176, 'Topology': 131, 'Vm': 164}; selected_inapplicable_by_domain {}; no_eligible_node_by_domain {}.

#### depth 250
changed_per_all_births 0.013000, dead_per_all_births 0.000500

| Cohort | created | deleted | present | never sel | sel only | applied only | changed only | dispatched-not-contrib | contributing | present/created | dispatched/present | contributing/present |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cohort | 918 | 45 | 873 | 317 | 0 | 4 | 444 | 85 | 23 | 0.950980 | 0.123711 | 0.026346 |
| graph | 445 | 28 | 417 | 149 | 0 | 1 | 213 | 53 | 1 | 0.937079 | 0.129496 | 0.002398 |
| vm | 473 | 17 | 456 | 168 | 0 | 3 | 231 | 32 | 22 | 0.964059 | 0.118421 | 0.048246 |

Founder reference row: created 100, deleted 11, present 89, dispatched 39, contributing 20 (contributing/present 0.224719).

| Time to first | reached | reached/created | median gens | censored deleted | censored present |
| --- | --- | --- | --- | --- | --- |
| selection | 487 | 0.530501 | 32 | 0 | 431 |
| applicable_selection | 487 | 0.530501 | 32 | 0 | 431 |
| internal_change | 558 | 0.607843 | 24 | 22 | 338 |
| dispatch | 249 | 0.271242 | 14 | 37 | 632 |
| contribution | 28 | 0.030501 | 170 | 45 | 845 |

Denominator identity, selection row: 487 + 0 + 431 = 918 created.

Retention from depth 22: 22/87 still contributing (0.252874), 59 present not contributing, 6 deleted.

Opportunities (cumulative): births 12500, zero-event 6940 (0.555200), attempted 7001, applied 6990 (0.998429), skipped 11; executed-target 5143 (0.735765 of applied), reachable 5308, unreachable 994.
attempted_by_domain {'Graph': 1882, 'InputRef': 1870, 'Topology': 1424, 'Vm': 1825}; applied_by_domain {'Graph': 1871, 'InputRef': 1870, 'Topology': 1424, 'Vm': 1825}; selected_inapplicable_by_domain {}; no_eligible_node_by_domain {'Graph': 11}.

#### depth 1000
changed_per_all_births 0.004500, dead_per_all_births 0.000500

| Cohort | created | deleted | present | never sel | sel only | applied only | changed only | dispatched-not-contrib | contributing | present/created | dispatched/present | contributing/present |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cohort | 4117 | 208 | 3909 | 1485 | 0 | 18 | 2212 | 162 | 32 | 0.949478 | 0.049629 | 0.008186 |
| graph | 1919 | 110 | 1809 | 613 | 0 | 8 | 1094 | 92 | 2 | 0.942678 | 0.051962 | 0.001106 |
| vm | 2198 | 98 | 2100 | 872 | 0 | 10 | 1118 | 70 | 30 | 0.955414 | 0.047619 | 0.014286 |

Founder reference row: created 100, deleted 20, present 80, dispatched 5, contributing 5 (contributing/present 0.062500).

| Time to first | reached | reached/created | median gens | censored deleted | censored present |
| --- | --- | --- | --- | --- | --- |
| selection | 2271 | 0.551615 | 86 | 0 | 1846 |
| applicable_selection | 2271 | 0.551615 | 86 | 0 | 1846 |
| internal_change | 2508 | 0.609181 | 68 | 89 | 1520 |
| dispatch | 936 | 0.227350 | 38 | 169 | 3012 |
| contribution | 54 | 0.013116 | 234 | 207 | 3856 |

Denominator identity, selection row: 2271 + 0 + 1846 = 4117 created.

Retention from depth 250: 8/43 still contributing (0.186047), 33 present not contributing, 2 deleted.

Opportunities (cumulative): births 50000, zero-event 27756 (0.555120), attempted 27874, applied 27863 (0.999605), skipped 11; executed-target 19251 (0.690916 of applied), reachable 19528, unreachable 5605.
attempted_by_domain {'Graph': 7497, 'InputRef': 7436, 'Topology': 5678, 'Vm': 7263}; applied_by_domain {'Graph': 7486, 'InputRef': 7436, 'Topology': 5678, 'Vm': 7263}; selected_inapplicable_by_domain {}; no_eligible_node_by_domain {'Graph': 11}.

#### depth 2000
changed_per_all_births 0.006000, dead_per_all_births 0.004000

| Cohort | created | deleted | present | never sel | sel only | applied only | changed only | dispatched-not-contrib | contributing | present/created | dispatched/present | contributing/present |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cohort | 8318 | 425 | 7893 | 2921 | 0 | 52 | 4715 | 176 | 29 | 0.948906 | 0.025972 | 0.003674 |
| graph | 3903 | 217 | 3686 | 1255 | 0 | 25 | 2304 | 100 | 2 | 0.944402 | 0.027672 | 0.000543 |
| vm | 4415 | 208 | 4207 | 1666 | 0 | 27 | 2411 | 76 | 27 | 0.952888 | 0.024483 | 0.006418 |

Founder reference row: created 100, deleted 26, present 74, dispatched 5, contributing 4 (contributing/present 0.054054).

| Time to first | reached | reached/created | median gens | censored deleted | censored present |
| --- | --- | --- | --- | --- | --- |
| selection | 4666 | 0.560952 | 158 | 0 | 3652 |
| applicable_selection | 4666 | 0.560952 | 158 | 0 | 3652 |
| internal_change | 5163 | 0.620702 | 124 | 159 | 2996 |
| dispatch | 1947 | 0.234071 | 68 | 333 | 6038 |
| contribution | 78 | 0.009377 | 430 | 421 | 7819 |

Denominator identity, selection row: 4666 + 0 + 3652 = 8318 created.

Retention from depth 1000: 4/37 still contributing (0.108108), 32 present not contributing, 1 deleted.

Opportunities (cumulative): births 100000, zero-event 55878 (0.558780), attempted 55204, applied 55193 (0.999801), skipped 11; executed-target 37949 (0.687569 of applied), reachable 38094, unreachable 11705.
attempted_by_domain {'Graph': 14729, 'InputRef': 14699, 'Topology': 11144, 'Vm': 14632}; applied_by_domain {'Graph': 14718, 'InputRef': 14699, 'Topology': 11144, 'Vm': 14632}; selected_inapplicable_by_domain {}; no_eligible_node_by_domain {'Graph': 11}.
