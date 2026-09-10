# T13.F01 — module recruitment observability: measured readings

Measured evidence for
[`docs/specs/roadmap/t13-f01-module-recruitment-observability.md`](../../specs/roadmap/t13-f01-module-recruitment-observability.md).
The spec keeps the predeclaration, the verdict, and the mutation survivor
record. Machine-written reports:
[gate](../features/t13-f01-module-recruitment-observability.json),
[goal](../features/t13-f01-module-recruitment-observability-goal.json).
Measured code `ea0044d5f4390ada662b0506caf020191fea9252` (the last commit of
the 2026-09-10 review remediation), host `Isaacs-MacBook-Pro-2.local`, Apple M1 Pro,
8 threads, release.

**Supersedes the 2026-09-10 remediation measurement (`c1cfee5a`).** These
09:15–09:23 runs replace that pair, which is superseded because its code counted
founder modules in the cohort retention row and reported 0 selected-inapplicable
per lineage. Every cohort ladder, founder row, time-to-first row and discard
total below is unchanged from it; the retention lines and the per-lineage
opportunity rows are not. Both reports were overwritten in place.

**Supersedes the first measurement.** The 07:40–07:49 runs at `ab00bb07` are
superseded because that code dropped each discarded operator, so its
selected-but-inapplicable split was empty and its drift walk took 25,004.812 ms
accumulated over the three worlds. Both reports were overwritten in place. An
intermediate pair at `afa6ae5c` (08:19–08:28, drift 19,186.559 ms) was
overwritten in turn by the `c1cfee5a` runs after the copy-provenance fix; its
`deterministic` block is byte-identical to the `c1cfee5a` one, since provenance
never reaches the report.

**What these numbers are and are not.** Every reading here is observation of
the mutation-only drift walk. Dispatch is not an effect, a *contribution* is
battery-signature sensitivity under the static-successor bypass only, and
usefulness is unmeasured. Nothing here is a cognition claim.

## Focused tests

Re-run at `ea0044d5` after the 2026-09-10 review remediation, all green:

- `cargo test -p v3-core --test viability` — `ok, 24 passed; 0 failed`
  (the engine birth path is touched, so this ran first).
- `cargo test -p v3-core` — 1281 + 24 + 19 + 13 + 10 + 7 + 4 + 3 + 2 + 1 + 0
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
`drift_checkpoint_reports_recruitment_and_opportunities_and_still_loads_older_reports`,
and the four added by the `afa6ae5c`..`c1cfee5a` remediation:
`a_discarded_operator_stays_visible_when_a_later_operator_applied`,
`a_module_named_only_by_a_discarded_operator_reaches_the_selected_only_rung`,
`a_discarded_operator_that_selected_nothing_counts_as_no_eligible_node`, and
`copy_provenance_compares_only_against_the_pre_birth_nodes`.

## Gate report

`make bench PROFILE=gate FEATURE=t13-f01-module-recruitment-observability`
exited 0; `comparison.severe=false`. Stored
`t13-f01-module-recruitment-observability.json`, generated 2026-09-10T16:15:23Z.
Log `/tmp/t13-f01-gate4.log`. Run once, before the goal run, nothing else
running.

| Reference | Measure | Current | Reference reading | Delta % | Result |
| --- | --- | --- | --- | --- | --- |
| remove-complementary-nutrition | mesh_hops | 2.028954 | 2.027261 | 0.083512 | ok |
| remove-complementary-nutrition | vm_steps | 22.425973 | 22.751316 | -1.429996 | ok |
| remove-complementary-nutrition | graph_relax_iters | 0.994920 | 0.995234 | -0.031550 | ok |
| remove-complementary-nutrition | plasticity_updates | 0.009880 | 0.011171 | -11.556709 | ok |
| remove-complementary-nutrition | actions_applied | 1.272860 | 1.275525 | -0.208934 | ok |
| remove-complementary-nutrition | births | 0.026902 | 0.026780 | 0.455564 | ok |
| remove-complementary-nutrition | wall ms/creature-tick | 0.0013768476 | 0.0015598310 | -11.730980 | ok |
| t11-f18-backend-neutral-mesh-node-growth | mesh_hops | 2.028954 | 2.028954 | 0.000000 | ok |
| t11-f18-backend-neutral-mesh-node-growth | vm_steps | 22.425973 | 22.425973 | 0.000000 | ok |
| t11-f18-backend-neutral-mesh-node-growth | graph_relax_iters | 0.994920 | 0.994920 | 0.000000 | ok |
| t11-f18-backend-neutral-mesh-node-growth | plasticity_updates | 0.009880 | 0.009880 | 0.000000 | ok |
| t11-f18-backend-neutral-mesh-node-growth | actions_applied | 1.272860 | 1.272860 | 0.000000 | ok |
| t11-f18-backend-neutral-mesh-node-growth | births | 0.026902 | 0.026902 | 0.000000 | ok |
| t11-f18-backend-neutral-mesh-node-growth | wall ms/creature-tick | 0.0013768476 | 0.0019781652 | -30.397747 | ok |

The harness compares the gate profile against the epoch and T11.F18, not
T12.F04. Hand-computed against T12.F04's gate report and carrying no harness
level: wall 0.0013768476 vs 0.0013002215 ms/creature-tick, +5.893308%.

Observation times (ms): founder 60.624; evolved and drift are not measured in
the gate profile (`drift_depth: "Undefined"`). Whole measured run 588.666 ms.
Caps unchanged: founder 10 s, evolved 180 s, drift 30 s, whole goal
investigation 900 s.

## Goal report

`make bench PROFILE=goal FEATURE=t13-f01-module-recruitment-observability`
exited 0; `comparison.severe=false`. Stored
`t13-f01-module-recruitment-observability-goal.json`, generated
2026-09-10T16:23:50Z. Log `/tmp/t13-f01-goal4.log`. Run once, sequentially after
the gate run with nothing else running.

| Reference | Measure | Current | Reference reading | Delta % | Result |
| --- | --- | --- | --- | --- | --- |
| t12-f04-baseline-world-set-goal | mesh_hops | 2.247936 | 2.247936 | 0.000000 | ok |
| t12-f04-baseline-world-set-goal | vm_steps | 23.339759 | 23.339759 | 0.000000 | ok |
| t12-f04-baseline-world-set-goal | graph_relax_iters | 1.028540 | 1.028540 | 0.000000 | ok |
| t12-f04-baseline-world-set-goal | plasticity_updates | 0.046976 | 0.046976 | 0.000000 | ok |
| t12-f04-baseline-world-set-goal | actions_applied | 1.371176 | 1.371176 | 0.000000 | ok |
| t12-f04-baseline-world-set-goal | births | 0.018609 | 0.018609 | 0.000000 | ok |
| t12-f04-baseline-world-set-goal | wall ms/creature-tick | 0.0078041359 | 0.0074066599 | 5.366468 | ok |

Simulation wall 485.47 s total (182.21 / 123.59 / 179.68 s for seeds
11 / 22 / 33). End-to-end elapsed 507 s, measured as the interval between the
goal log's creation (09:15:23 local) and the report write (09:23:50 local),
against the 15-minute investigation threshold — under it, no investigation
triggered.

Observation times: founder 100.379 ms (cap 10 s); evolved 466.108 ms total,
164.142 / 161.590 / 140.377 ms per seed (cap 180 s); final-state observation
875.871 ms.

**Drift wall time.** The harness records a single accumulated
`drift_depth_wall_clock_ms` across the three worlds and no per-world split
exists in the report: 18,956.800 ms, against 18,966.423 ms at `c1cfee5a` and
25,004.812 ms in the first measurement. Because the total is 18.96 s, no single
world can exceed 18.96 s, so every world is under the 30 s cap; the per-world
average is 6.32 s.
T12.F04 recorded 11,179.458 ms for the same three worlds, so the recruitment
observation adds 7.78 s in total, an average of **+2.59 s per world**. The walk
no longer clones each parent's `nodes` per birth: each lineage keeps one node
snapshot that the birth is diffed against in place, and only the nodes a birth
changed or added are cloned into it. **The predeclaration expected "under 2 s
added per world"; the cap holds with room to spare and the remediation cut the
overshoot from +4.61 s to +2.59 s, but the predeclared bound is still
missed.** What remains is the per-generation equality comparison over every
surviving node of 50 lineages across 2,000 generations, which allocates nothing
but still reads every node; no further work is done here.

## Structural comparison against T12.F04

Method: a recursive leaf diff over the whole `deterministic` block of each
report against T12.F04's, printing every value difference and every key present
on only one side, with no filtering; classification afterwards.

- **Gate**, against `t12-f04-baseline-world-set.json`: **0 difference lines**
  (re-run against the 2026-09-10 review-remediation gate report).
  Every deterministic field is equal. Note that the gate profile carries
  `drift_depth: "Undefined"`, so this comparison covers fewer fields than the
  goal one — no drift, recruitment, or opportunity field exists in it. Equal
  deterministic blocks mean the six gate counters, which are derived from the
  equal `per_creature_tick` and `totals` fields, equal T12.F04's exactly.
- **Goal**, against `t12-f04-baseline-world-set-goal.json`: **42 difference
  lines** (re-run against the review-remediation goal report; the same count and
  the same four groups as both superseded measurements), and nothing else:
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

## The selected-but-inapplicable split, per discarded operator

The engine swap-removes an operator that reports `NoApplicableTarget` and
retries another operator of the same domain, so an *event-level* skip is
recorded only when every operator of the drawn domain failed. Measured that way
the event-level split is almost empty: across all three worlds and all five
checkpoints `selected_inapplicable_by_domain` is `{}` everywhere, and
`no_eligible_node_by_domain` is `{Graph: 11}` in Orchards and Confluence and
`{Graph: 2}` in Canyon.

Each event record now also lists the operators the event discarded with the
node each had selected, and those are where the fact lives. Cumulative to
depth 2,000, pooled over 50 lineages per world:

| World | attempted events | discards that selected a target with no applicable site | discards with no eligible node | selected-only modules present |
| --- | --- | --- | --- | --- |
| Orchards in grassland (seed 11) | 55,204 | 27,096 | 356 | 609 |
| Canyon country (seed 22) | 55,142 | 28,380 | 139 | 621 |
| Confluence (seed 33) | 55,204 | 27,096 | 356 | 609 |

So roughly one discarded operator per two attempted events selected a module
and found no site on it. The count is dominated by the Graph domain's
per-feature operators on modules that carry no such feature — at depth 2,000 in
Orchards the largest are `Graph.MutateTraceDecay` 2,746,
`Graph.MutateHebbianRate` 2,512, `Graph.MutateGraphOperatorParam` 2,257,
`Vm.MutatePairedSlotAddress` 1,702, `Graph.MutateRewardSource` 1,584 and
`Graph.ToggleHebbianLamarckian` 1,425 — and the `no eligible node` discards are
a flat per-operator tail (11 per Graph operator in Orchards and Confluence,
2 in Canyon, plus the Topology route operators).

The ladder's *selected only* rung is now reachable and occupied: 609 / 621 /
609 modules present at depth 2,000 have been named by some operator and never
by one that applied. The rung was 0 everywhere in the superseded measurement
purely because the discard was dropped. Every per-operator figure here is one
operator's discard, not an event: an event that discards three operators and
then applies contributes three discards and one applied event.

**Per lineage.** Each row of `opportunities.lineages[]` carries
`discarded_selected_inapplicable`, the sum of that lineage's
`discarded_selected_inapplicable_by_operator`, and the 50 rows sum exactly to
the pooled total at every checkpoint of every world (checked for all fifteen
readings). At depth 2,000 the per-lineage counts run 283–722 in Orchards and
Confluence and 285–759 in Canyon; at depth 22, 46 of 50 lineages in Orchards
and Confluence and 47 of 50 in Canyon already carry at least one and the rest
carry none. In both superseded measurements every one of these rows read 0 at
every checkpoint, because the row summed the event-level per-domain split,
which is empty by construction.

## Per-world, per-checkpoint readings

Checkpoints are depths 0, 22, 250, 1,000 and 2,000, over 50 lineages per world;
`created == deleted + present`, the six ladder rungs partition `present`, and
`reached + censored_deleted + censored_present == created` in every
time-to-first row (one instance of that identity is printed under each table).
Opportunity totals are cumulative from depth 0 to the checkpoint, pooled over
all 50 lineages. Retention is a cohort reading: its denominator is the
`new`+`copy` modules contributing at the earlier checkpoint, so it equals that
checkpoint's cohort `contributing` count and founder modules — which the
reference row reports separately — never enter it.

### Orchards in grassland (seed 11) — drift_depth `drift-depth-v3`, 50 lineages

#### depth 0

Founder generation: no recruited module exists yet (cohort created 0), founders created 100, present 100, dispatched 100, contributing 100; opportunities all zero over 0 births. changed_per_all_births 0.206500, dead_per_all_births 0.000500.

#### depth 22
changed_per_all_births 0.092000, dead_per_all_births 0.000500

| Cohort | created | deleted | present | never sel | sel only | applied only | changed only | dispatched-not-contrib | contributing | present/created | dispatched/present | contributing/present |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cohort | 78 | 2 | 76 | 31 | 1 | 0 | 13 | 24 | 7 | 0.974359 | 0.407895 | 0.092105 |
| graph | 45 | 2 | 43 | 21 | 1 | 0 | 5 | 15 | 1 | 0.955556 | 0.372093 | 0.023256 |
| vm | 33 | 0 | 33 | 10 | 0 | 0 | 8 | 9 | 6 | 1.000000 | 0.454545 | 0.181818 |

Founder reference row: created 100, deleted 1, present 99, dispatched 85, contributing 80 (contributing/present 0.808081).

| Time to first | reached | reached/created | median gens | censored deleted | censored present |
| --- | --- | --- | --- | --- | --- |
| selection | 23 | 0.294872 | 6 | 0 | 55 |
| applicable_selection | 16 | 0.205128 | 5 | 0 | 62 |
| internal_change | 28 | 0.358974 | 5 | 1 | 49 |
| dispatch | 32 | 0.410256 | 5 | 2 | 44 |
| contribution | 7 | 0.089744 | 12 | 2 | 69 |

Denominator identity, selection row: 23 + 0 + 55 = 78 created.

Retention from depth 0: the depth-0 cohort is empty, so `contributing_before` is 0 and the fraction is `Undefined` (founder modules are a separate reference row and never enter retention).

Opportunities (cumulative): births 1100, zero-event 597 (0.542727), attempted 643, applied 643 (1.000000), skipped 0; executed-target 550 (0.855365 of applied), reachable 544, unreachable 37.
attempted_by_domain {'Graph': 172, 'InputRef': 176, 'Topology': 131, 'Vm': 164}; applied_by_domain {'Graph': 172, 'InputRef': 176, 'Topology': 131, 'Vm': 164}; selected_inapplicable_by_domain {}; no_eligible_node_by_domain {}.
Discarded operators — selected but no applicable site {'Graph.AddInternalGraphNode': 1, 'Graph.AlterGraphEdgeWeight': 6, 'Graph.CopyEdgeBundle': 3, 'Graph.CopyInternalNode': 1, 'Graph.CopySubgraph': 2, 'Graph.DisableHebbian': 4, 'Graph.DisableRewardModulation': 5, 'Graph.EnableHebbian': 2, 'Graph.EnableRewardModulation': 5, 'Graph.GraphRawFieldMutation': 1, 'Graph.MutateGraphOperatorParam': 4, 'Graph.MutateHebbianRate': 29, 'Graph.MutateHebbianRule': 8, 'Graph.MutateRewardSource': 11, 'Graph.MutateTraceDecay': 33, 'Graph.RemoveGraphEdge': 3, 'Graph.RetargetGraphEdge': 3, 'Graph.ToggleHebbianLamarckian': 16, 'Vm.InsertReadStoreMotif': 1, 'Vm.MutatePairedSlotAddress': 31, 'Vm.MutateSlotAddress': 21, 'Vm.VmRegisterCountMutation': 1} (total 191); no eligible node {'Topology.RemoveNode': 5, 'Topology.RemoveRouteTarget': 11, 'Topology.RetargetNodeTarget': 5, 'Topology.SwapRouteTargets': 16} (total 37).

#### depth 250
changed_per_all_births 0.013000, dead_per_all_births 0.000500

| Cohort | created | deleted | present | never sel | sel only | applied only | changed only | dispatched-not-contrib | contributing | present/created | dispatched/present | contributing/present |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cohort | 918 | 45 | 873 | 259 | 58 | 4 | 444 | 85 | 23 | 0.950980 | 0.123711 | 0.026346 |
| graph | 445 | 28 | 417 | 103 | 46 | 1 | 213 | 53 | 1 | 0.937079 | 0.129496 | 0.002398 |
| vm | 473 | 17 | 456 | 156 | 12 | 3 | 231 | 32 | 22 | 0.964059 | 0.118421 | 0.048246 |

Founder reference row: created 100, deleted 11, present 89, dispatched 39, contributing 20 (contributing/present 0.224719).

| Time to first | reached | reached/created | median gens | censored deleted | censored present |
| --- | --- | --- | --- | --- | --- |
| selection | 566 | 0.616558 | 24 | 0 | 352 |
| applicable_selection | 487 | 0.530501 | 32 | 0 | 431 |
| internal_change | 558 | 0.607843 | 24 | 22 | 338 |
| dispatch | 249 | 0.271242 | 14 | 37 | 632 |
| contribution | 28 | 0.030501 | 170 | 45 | 845 |

Denominator identity, selection row: 566 + 0 + 352 = 918 created.

Retention from depth 22: 2/7 cohort modules still contributing (0.285714), 5 present not contributing, 0 deleted.

Opportunities (cumulative): births 12500, zero-event 6940 (0.555200), attempted 7001, applied 6990 (0.998429), skipped 11; executed-target 5143 (0.735765 of applied), reachable 5308, unreachable 994.
attempted_by_domain {'Graph': 1882, 'InputRef': 1870, 'Topology': 1424, 'Vm': 1825}; applied_by_domain {'Graph': 1871, 'InputRef': 1870, 'Topology': 1424, 'Vm': 1825}; selected_inapplicable_by_domain {}; no_eligible_node_by_domain {'Graph': 11}.
Discarded operators — selected but no applicable site {'Graph.AddInternalGraphNode': 6, 'Graph.AlterGraphEdgeWeight': 99, 'Graph.CopyEdgeBundle': 95, 'Graph.CopyInternalNode': 36, 'Graph.CopySubgraph': 50, 'Graph.DisableHebbian': 79, 'Graph.DisableRewardModulation': 87, 'Graph.EnableHebbian': 33, 'Graph.EnableRewardModulation': 79, 'Graph.GraphRawFieldMutation': 100, 'Graph.MutateGraphOperatorParam': 176, 'Graph.MutateHebbianRate': 273, 'Graph.MutateHebbianRule': 159, 'Graph.MutateRewardSource': 151, 'Graph.MutateTraceDecay': 336, 'Graph.RemoveGraphEdge': 58, 'Graph.RemoveInternalGraphNode': 52, 'Graph.RetargetGraphEdge': 54, 'Graph.SwapGraphOperator': 68, 'Graph.ToggleHebbianLamarckian': 156, 'Vm.CopyConstantBlock': 9, 'Vm.CopyGeneBackwardSlice': 9, 'Vm.CopyGeneForwardSlice': 7, 'Vm.InsertReadBidMotif': 22, 'Vm.InsertReadStoreMotif': 34, 'Vm.MutatePairedSlotAddress': 233, 'Vm.MutateSlotAddress': 98, 'Vm.VmDeleteInstruction': 11, 'Vm.VmInstructionRawFieldMutation': 29, 'Vm.VmRegisterCountMutation': 37} (total 2636); no eligible node {'Graph.AddGraphEdge': 11, 'Graph.AddInternalGraphNode': 11, 'Graph.AlterGraphEdgeWeight': 11, 'Graph.CopyEdgeBundle': 11, 'Graph.CopyInternalNode': 11, 'Graph.CopySubgraph': 11, 'Graph.DisableHebbian': 11, 'Graph.DisableRewardModulation': 11, 'Graph.EnableHebbian': 11, 'Graph.EnableRewardModulation': 11, 'Graph.GraphRawFieldMutation': 11, 'Graph.MutateActionSlotBehavior': 11, 'Graph.MutateGraphOperatorParam': 11, 'Graph.MutateHebbianRate': 11, 'Graph.MutateHebbianRule': 11, 'Graph.MutateRewardSource': 11, 'Graph.MutateTraceDecay': 11, 'Graph.RemoveGraphEdge': 11, 'Graph.RemoveInternalGraphNode': 11, 'Graph.RetargetGraphEdge': 11, 'Graph.SwapGraphOperator': 11, 'Graph.ToggleHebbianLamarckian': 11, 'Topology.AddNode': 2, 'Topology.AddRouteTarget': 5, 'Topology.MutateGateBias': 2, 'Topology.RemoveNode': 9, 'Topology.RemoveRouteTarget': 31, 'Topology.RetargetNodeTarget': 15, 'Topology.SpliceNode': 1, 'Topology.SwapRouteTargets': 45} (total 352).

#### depth 1000
changed_per_all_births 0.004500, dead_per_all_births 0.000500

| Cohort | created | deleted | present | never sel | sel only | applied only | changed only | dispatched-not-contrib | contributing | present/created | dispatched/present | contributing/present |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cohort | 4117 | 208 | 3909 | 1183 | 302 | 18 | 2212 | 162 | 32 | 0.949478 | 0.049629 | 0.008186 |
| graph | 1919 | 110 | 1809 | 383 | 230 | 8 | 1094 | 92 | 2 | 0.942678 | 0.051962 | 0.001106 |
| vm | 2198 | 98 | 2100 | 800 | 72 | 10 | 1118 | 70 | 30 | 0.955414 | 0.047619 | 0.014286 |

Founder reference row: created 100, deleted 20, present 80, dispatched 5, contributing 5 (contributing/present 0.062500).

| Time to first | reached | reached/created | median gens | censored deleted | censored present |
| --- | --- | --- | --- | --- | --- |
| selection | 2670 | 0.648530 | 63 | 0 | 1447 |
| applicable_selection | 2271 | 0.551615 | 86 | 0 | 1846 |
| internal_change | 2508 | 0.609181 | 68 | 89 | 1520 |
| dispatch | 936 | 0.227350 | 38 | 169 | 3012 |
| contribution | 54 | 0.013116 | 234 | 207 | 3856 |

Denominator identity, selection row: 2670 + 0 + 1447 = 4117 created.

Retention from depth 250: 5/23 cohort modules still contributing (0.217391), 17 present not contributing, 1 deleted.

Opportunities (cumulative): births 50000, zero-event 27756 (0.555120), attempted 27874, applied 27863 (0.999605), skipped 11; executed-target 19251 (0.690916 of applied), reachable 19528, unreachable 5605.
attempted_by_domain {'Graph': 7497, 'InputRef': 7436, 'Topology': 5678, 'Vm': 7263}; applied_by_domain {'Graph': 7486, 'InputRef': 7436, 'Topology': 5678, 'Vm': 7263}; selected_inapplicable_by_domain {}; no_eligible_node_by_domain {'Graph': 11}.
Discarded operators — selected but no applicable site {'Graph.AddInternalGraphNode': 46, 'Graph.AlterGraphEdgeWeight': 483, 'Graph.CopyEdgeBundle': 594, 'Graph.CopyInternalNode': 221, 'Graph.CopySubgraph': 312, 'Graph.DisableHebbian': 368, 'Graph.DisableRewardModulation': 404, 'Graph.EnableHebbian': 268, 'Graph.EnableRewardModulation': 356, 'Graph.GraphRawFieldMutation': 531, 'Graph.MutateGraphOperatorParam': 1048, 'Graph.MutateHebbianRate': 1238, 'Graph.MutateHebbianRule': 695, 'Graph.MutateRewardSource': 760, 'Graph.MutateTraceDecay': 1334, 'Graph.RemoveGraphEdge': 308, 'Graph.RemoveInternalGraphNode': 262, 'Graph.RetargetGraphEdge': 290, 'Graph.SwapGraphOperator': 434, 'Graph.ToggleHebbianLamarckian': 681, 'Vm.CopyConstantBlock': 48, 'Vm.CopyGeneBackwardSlice': 61, 'Vm.CopyGeneForwardSlice': 46, 'Vm.InsertReadBidMotif': 177, 'Vm.InsertReadStoreMotif': 185, 'Vm.MutatePairedSlotAddress': 837, 'Vm.MutateSlotAddress': 327, 'Vm.VmDeleteInstruction': 49, 'Vm.VmInstructionRawFieldMutation': 182, 'Vm.VmRegisterCountMutation': 189} (total 12734); no eligible node {'Graph.AddGraphEdge': 11, 'Graph.AddInternalGraphNode': 11, 'Graph.AlterGraphEdgeWeight': 11, 'Graph.CopyEdgeBundle': 11, 'Graph.CopyInternalNode': 11, 'Graph.CopySubgraph': 11, 'Graph.DisableHebbian': 11, 'Graph.DisableRewardModulation': 11, 'Graph.EnableHebbian': 11, 'Graph.EnableRewardModulation': 11, 'Graph.GraphRawFieldMutation': 11, 'Graph.MutateActionSlotBehavior': 11, 'Graph.MutateGraphOperatorParam': 11, 'Graph.MutateHebbianRate': 11, 'Graph.MutateHebbianRule': 11, 'Graph.MutateRewardSource': 11, 'Graph.MutateTraceDecay': 11, 'Graph.RemoveGraphEdge': 11, 'Graph.RemoveInternalGraphNode': 11, 'Graph.RetargetGraphEdge': 11, 'Graph.SwapGraphOperator': 11, 'Graph.ToggleHebbianLamarckian': 11, 'Topology.AddNode': 2, 'Topology.AddRouteTarget': 7, 'Topology.MutateGateBias': 2, 'Topology.RemoveNode': 9, 'Topology.RemoveRouteTarget': 31, 'Topology.RetargetNodeTarget': 15, 'Topology.SpliceNode': 1, 'Topology.SwapRouteTargets': 47} (total 356).

#### depth 2000
changed_per_all_births 0.006000, dead_per_all_births 0.004000

| Cohort | created | deleted | present | never sel | sel only | applied only | changed only | dispatched-not-contrib | contributing | present/created | dispatched/present | contributing/present |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cohort | 8318 | 425 | 7893 | 2312 | 609 | 52 | 4715 | 176 | 29 | 0.948906 | 0.025972 | 0.003674 |
| graph | 3903 | 217 | 3686 | 804 | 451 | 25 | 2304 | 100 | 2 | 0.944402 | 0.027672 | 0.000543 |
| vm | 4415 | 208 | 4207 | 1508 | 158 | 27 | 2411 | 76 | 27 | 0.952888 | 0.024483 | 0.006418 |

Founder reference row: created 100, deleted 26, present 74, dispatched 5, contributing 4 (contributing/present 0.054054).

| Time to first | reached | reached/created | median gens | censored deleted | censored present |
| --- | --- | --- | --- | --- | --- |
| selection | 5474 | 0.658091 | 118 | 0 | 2844 |
| applicable_selection | 4666 | 0.560952 | 158 | 0 | 3652 |
| internal_change | 5163 | 0.620702 | 124 | 159 | 2996 |
| dispatch | 1947 | 0.234071 | 68 | 333 | 6038 |
| contribution | 78 | 0.009377 | 430 | 421 | 7819 |

Denominator identity, selection row: 5474 + 0 + 2844 = 8318 created.

Retention from depth 1000: 2/32 cohort modules still contributing (0.062500), 29 present not contributing, 1 deleted.

Opportunities (cumulative): births 100000, zero-event 55878 (0.558780), attempted 55204, applied 55193 (0.999801), skipped 11; executed-target 37949 (0.687569 of applied), reachable 38094, unreachable 11705.
attempted_by_domain {'Graph': 14729, 'InputRef': 14699, 'Topology': 11144, 'Vm': 14632}; applied_by_domain {'Graph': 14718, 'InputRef': 14699, 'Topology': 11144, 'Vm': 14632}; selected_inapplicable_by_domain {}; no_eligible_node_by_domain {'Graph': 11}.
Discarded operators — selected but no applicable site {'Graph.AddInternalGraphNode': 120, 'Graph.AlterGraphEdgeWeight': 1068, 'Graph.CopyEdgeBundle': 1286, 'Graph.CopyInternalNode': 491, 'Graph.CopySubgraph': 657, 'Graph.DisableHebbian': 732, 'Graph.DisableRewardModulation': 850, 'Graph.EnableHebbian': 612, 'Graph.EnableRewardModulation': 739, 'Graph.GraphRawFieldMutation': 1185, 'Graph.MutateGraphOperatorParam': 2257, 'Graph.MutateHebbianRate': 2512, 'Graph.MutateHebbianRule': 1465, 'Graph.MutateRewardSource': 1584, 'Graph.MutateTraceDecay': 2746, 'Graph.RemoveGraphEdge': 636, 'Graph.RemoveInternalGraphNode': 553, 'Graph.RetargetGraphEdge': 601, 'Graph.SwapGraphOperator': 919, 'Graph.ToggleHebbianLamarckian': 1425, 'Vm.CopyConstantBlock': 131, 'Vm.CopyGeneBackwardSlice': 154, 'Vm.CopyGeneForwardSlice': 133, 'Vm.InsertReadBidMotif': 481, 'Vm.InsertReadStoreMotif': 465, 'Vm.MutatePairedSlotAddress': 1702, 'Vm.MutateSlotAddress': 682, 'Vm.VmDeleteInstruction': 99, 'Vm.VmInstructionRawFieldMutation': 402, 'Vm.VmRegisterCountMutation': 409} (total 27096); no eligible node {'Graph.AddGraphEdge': 11, 'Graph.AddInternalGraphNode': 11, 'Graph.AlterGraphEdgeWeight': 11, 'Graph.CopyEdgeBundle': 11, 'Graph.CopyInternalNode': 11, 'Graph.CopySubgraph': 11, 'Graph.DisableHebbian': 11, 'Graph.DisableRewardModulation': 11, 'Graph.EnableHebbian': 11, 'Graph.EnableRewardModulation': 11, 'Graph.GraphRawFieldMutation': 11, 'Graph.MutateActionSlotBehavior': 11, 'Graph.MutateGraphOperatorParam': 11, 'Graph.MutateHebbianRate': 11, 'Graph.MutateHebbianRule': 11, 'Graph.MutateRewardSource': 11, 'Graph.MutateTraceDecay': 11, 'Graph.RemoveGraphEdge': 11, 'Graph.RemoveInternalGraphNode': 11, 'Graph.RetargetGraphEdge': 11, 'Graph.SwapGraphOperator': 11, 'Graph.ToggleHebbianLamarckian': 11, 'Topology.AddNode': 2, 'Topology.AddRouteTarget': 7, 'Topology.MutateGateBias': 2, 'Topology.RemoveNode': 9, 'Topology.RemoveRouteTarget': 31, 'Topology.RetargetNodeTarget': 15, 'Topology.SpliceNode': 1, 'Topology.SwapRouteTargets': 47} (total 356).

### Canyon country (seed 22) — drift_depth `drift-depth-v3`, 50 lineages

#### depth 0

Founder generation: no recruited module exists yet (cohort created 0), founders created 100, present 100, dispatched 100, contributing 100; opportunities all zero over 0 births. changed_per_all_births 0.201500, dead_per_all_births 0.000500.

#### depth 22
changed_per_all_births 0.095000, dead_per_all_births 0.000000

| Cohort | created | deleted | present | never sel | sel only | applied only | changed only | dispatched-not-contrib | contributing | present/created | dispatched/present | contributing/present |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cohort | 88 | 1 | 87 | 43 | 1 | 0 | 11 | 26 | 6 | 0.988636 | 0.367816 | 0.068966 |
| graph | 48 | 1 | 47 | 27 | 0 | 0 | 4 | 15 | 1 | 0.979167 | 0.340426 | 0.021277 |
| vm | 40 | 0 | 40 | 16 | 1 | 0 | 7 | 11 | 5 | 1.000000 | 0.400000 | 0.125000 |

Founder reference row: created 100, deleted 1, present 99, dispatched 87, contributing 83 (contributing/present 0.838384).

| Time to first | reached | reached/created | median gens | censored deleted | censored present |
| --- | --- | --- | --- | --- | --- |
| selection | 21 | 0.238636 | 6 | 0 | 67 |
| applicable_selection | 15 | 0.170455 | 5 | 0 | 73 |
| internal_change | 25 | 0.284091 | 5 | 1 | 62 |
| dispatch | 33 | 0.375000 | 4 | 1 | 54 |
| contribution | 6 | 0.068182 | 12 | 1 | 81 |

Denominator identity, selection row: 21 + 0 + 67 = 88 created.

Retention from depth 0: the depth-0 cohort is empty, so `contributing_before` is 0 and the fraction is `Undefined` (founder modules are a separate reference row and never enter retention).

Opportunities (cumulative): births 1100, zero-event 612 (0.556364), attempted 630, applied 630 (1.000000), skipped 0; executed-target 549 (0.871429 of applied), reachable 542, unreachable 28.
attempted_by_domain {'Graph': 172, 'InputRef': 164, 'Topology': 128, 'Vm': 166}; applied_by_domain {'Graph': 172, 'InputRef': 164, 'Topology': 128, 'Vm': 166}; selected_inapplicable_by_domain {}; no_eligible_node_by_domain {}.
Discarded operators — selected but no applicable site {'Graph.AddInternalGraphNode': 1, 'Graph.AlterGraphEdgeWeight': 4, 'Graph.CopyEdgeBundle': 3, 'Graph.CopyInternalNode': 2, 'Graph.CopySubgraph': 2, 'Graph.DisableHebbian': 4, 'Graph.DisableRewardModulation': 7, 'Graph.EnableRewardModulation': 2, 'Graph.GraphRawFieldMutation': 1, 'Graph.MutateGraphOperatorParam': 5, 'Graph.MutateHebbianRate': 29, 'Graph.MutateHebbianRule': 10, 'Graph.MutateRewardSource': 9, 'Graph.MutateTraceDecay': 33, 'Graph.RemoveGraphEdge': 2, 'Graph.RetargetGraphEdge': 3, 'Graph.SwapGraphOperator': 1, 'Graph.ToggleHebbianLamarckian': 18, 'Vm.InsertReadStoreMotif': 1, 'Vm.MutatePairedSlotAddress': 31, 'Vm.MutateSlotAddress': 24, 'Vm.VmRegisterCountMutation': 1} (total 193); no eligible node {'Topology.RemoveNode': 5, 'Topology.RemoveRouteTarget': 11, 'Topology.RetargetNodeTarget': 5, 'Topology.SwapRouteTargets': 15} (total 36).

#### depth 250
changed_per_all_births 0.011000, dead_per_all_births 0.002500

| Cohort | created | deleted | present | never sel | sel only | applied only | changed only | dispatched-not-contrib | contributing | present/created | dispatched/present | contributing/present |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cohort | 869 | 34 | 835 | 234 | 56 | 5 | 424 | 90 | 26 | 0.960875 | 0.138922 | 0.031138 |
| graph | 414 | 21 | 393 | 82 | 40 | 1 | 221 | 48 | 1 | 0.949275 | 0.124682 | 0.002545 |
| vm | 455 | 13 | 442 | 152 | 16 | 4 | 203 | 42 | 25 | 0.971429 | 0.151584 | 0.056561 |

Founder reference row: created 100, deleted 12, present 88, dispatched 32, contributing 19 (contributing/present 0.215909).

| Time to first | reached | reached/created | median gens | censored deleted | censored present |
| --- | --- | --- | --- | --- | --- |
| selection | 562 | 0.646720 | 26 | 0 | 307 |
| applicable_selection | 485 | 0.558113 | 33 | 0 | 384 |
| internal_change | 539 | 0.620253 | 26 | 19 | 311 |
| dispatch | 244 | 0.280783 | 18 | 28 | 597 |
| contribution | 30 | 0.034522 | 170 | 33 | 806 |

Denominator identity, selection row: 562 + 0 + 307 = 869 created.

Retention from depth 22: 2/6 cohort modules still contributing (0.333333), 3 present not contributing, 1 deleted.

Opportunities (cumulative): births 12500, zero-event 6992 (0.559360), attempted 6837, applied 6835 (0.999707), skipped 2; executed-target 5029 (0.735772 of applied), reachable 5108, unreachable 1049.
attempted_by_domain {'Graph': 1826, 'InputRef': 1786, 'Topology': 1378, 'Vm': 1847}; applied_by_domain {'Graph': 1824, 'InputRef': 1786, 'Topology': 1378, 'Vm': 1847}; selected_inapplicable_by_domain {}; no_eligible_node_by_domain {'Graph': 2}.
Discarded operators — selected but no applicable site {'Graph.AddInternalGraphNode': 12, 'Graph.AlterGraphEdgeWeight': 110, 'Graph.CopyEdgeBundle': 102, 'Graph.CopyInternalNode': 41, 'Graph.CopySubgraph': 54, 'Graph.DisableHebbian': 86, 'Graph.DisableRewardModulation': 87, 'Graph.EnableHebbian': 43, 'Graph.EnableRewardModulation': 82, 'Graph.GraphRawFieldMutation': 101, 'Graph.MutateGraphOperatorParam': 175, 'Graph.MutateHebbianRate': 286, 'Graph.MutateHebbianRule': 167, 'Graph.MutateRewardSource': 166, 'Graph.MutateTraceDecay': 330, 'Graph.RemoveGraphEdge': 56, 'Graph.RemoveInternalGraphNode': 45, 'Graph.RetargetGraphEdge': 64, 'Graph.SwapGraphOperator': 102, 'Graph.ToggleHebbianLamarckian': 176, 'Vm.CopyConstantBlock': 17, 'Vm.CopyGeneBackwardSlice': 8, 'Vm.CopyGeneForwardSlice': 10, 'Vm.InsertReadBidMotif': 34, 'Vm.InsertReadStoreMotif': 36, 'Vm.MutatePairedSlotAddress': 236, 'Vm.MutateSlotAddress': 96, 'Vm.VmDeleteInstruction': 13, 'Vm.VmInstructionRawFieldMutation': 30, 'Vm.VmRegisterCountMutation': 35} (total 2800); no eligible node {'Graph.AddGraphEdge': 2, 'Graph.AddInternalGraphNode': 2, 'Graph.AlterGraphEdgeWeight': 2, 'Graph.CopyEdgeBundle': 2, 'Graph.CopyInternalNode': 2, 'Graph.CopySubgraph': 2, 'Graph.DisableHebbian': 2, 'Graph.DisableRewardModulation': 2, 'Graph.EnableHebbian': 2, 'Graph.EnableRewardModulation': 2, 'Graph.GraphRawFieldMutation': 2, 'Graph.MutateActionSlotBehavior': 2, 'Graph.MutateGraphOperatorParam': 2, 'Graph.MutateHebbianRate': 2, 'Graph.MutateHebbianRule': 2, 'Graph.MutateRewardSource': 2, 'Graph.MutateTraceDecay': 2, 'Graph.RemoveGraphEdge': 2, 'Graph.RemoveInternalGraphNode': 2, 'Graph.RetargetGraphEdge': 2, 'Graph.SwapGraphOperator': 2, 'Graph.ToggleHebbianLamarckian': 2, 'Topology.AddRouteTarget': 4, 'Topology.CopyNode': 1, 'Topology.MutateGateBias': 2, 'Topology.RemoveNode': 7, 'Topology.RemoveRouteTarget': 27, 'Topology.RetargetNodeTarget': 11, 'Topology.SwapRouteTargets': 41} (total 137).

#### depth 1000
changed_per_all_births 0.010000, dead_per_all_births 0.001000

| Cohort | created | deleted | present | never sel | sel only | applied only | changed only | dispatched-not-contrib | contributing | present/created | dispatched/present | contributing/present |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cohort | 3827 | 205 | 3622 | 1044 | 306 | 16 | 2075 | 145 | 36 | 0.946433 | 0.049972 | 0.009939 |
| graph | 1792 | 97 | 1695 | 349 | 245 | 9 | 1024 | 67 | 1 | 0.945871 | 0.040118 | 0.000590 |
| vm | 2035 | 108 | 1927 | 695 | 61 | 7 | 1051 | 78 | 35 | 0.946929 | 0.058640 | 0.018163 |

Founder reference row: created 100, deleted 19, present 81, dispatched 7, contributing 4 (contributing/present 0.049383).

| Time to first | reached | reached/created | median gens | censored deleted | censored present |
| --- | --- | --- | --- | --- | --- |
| selection | 2574 | 0.672589 | 63 | 0 | 1253 |
| applicable_selection | 2189 | 0.571989 | 83 | 0 | 1638 |
| internal_change | 2358 | 0.616148 | 68 | 86 | 1383 |
| dispatch | 962 | 0.251372 | 48 | 162 | 2703 |
| contribution | 62 | 0.016201 | 229 | 202 | 3563 |

Denominator identity, selection row: 2574 + 0 + 1253 = 3827 created.

Retention from depth 250: 3/26 cohort modules still contributing (0.115385), 21 present not contributing, 2 deleted.

Opportunities (cumulative): births 50000, zero-event 27886 (0.557720), attempted 27593, applied 27591 (0.999928), skipped 2; executed-target 19235 (0.697148 of applied), reachable 19431, unreachable 5443.
attempted_by_domain {'Graph': 7326, 'InputRef': 7364, 'Topology': 5525, 'Vm': 7378}; applied_by_domain {'Graph': 7324, 'InputRef': 7364, 'Topology': 5525, 'Vm': 7378}; selected_inapplicable_by_domain {}; no_eligible_node_by_domain {'Graph': 2}.
Discarded operators — selected but no applicable site {'Graph.AddInternalGraphNode': 70, 'Graph.AlterGraphEdgeWeight': 556, 'Graph.CopyEdgeBundle': 614, 'Graph.CopyInternalNode': 257, 'Graph.CopySubgraph': 310, 'Graph.DisableHebbian': 399, 'Graph.DisableRewardModulation': 418, 'Graph.EnableHebbian': 301, 'Graph.EnableRewardModulation': 382, 'Graph.GraphRawFieldMutation': 577, 'Graph.MutateGraphOperatorParam': 1092, 'Graph.MutateHebbianRate': 1368, 'Graph.MutateHebbianRule': 766, 'Graph.MutateRewardSource': 765, 'Graph.MutateTraceDecay': 1417, 'Graph.RemoveGraphEdge': 319, 'Graph.RemoveInternalGraphNode': 240, 'Graph.RetargetGraphEdge': 306, 'Graph.SwapGraphOperator': 503, 'Graph.ToggleHebbianLamarckian': 736, 'Vm.CopyConstantBlock': 58, 'Vm.CopyGeneBackwardSlice': 64, 'Vm.CopyGeneForwardSlice': 60, 'Vm.InsertReadBidMotif': 215, 'Vm.InsertReadStoreMotif': 215, 'Vm.MutatePairedSlotAddress': 850, 'Vm.MutateSlotAddress': 326, 'Vm.VmDeleteInstruction': 58, 'Vm.VmInstructionRawFieldMutation': 190, 'Vm.VmRegisterCountMutation': 178} (total 13610); no eligible node {'Graph.AddGraphEdge': 2, 'Graph.AddInternalGraphNode': 2, 'Graph.AlterGraphEdgeWeight': 2, 'Graph.CopyEdgeBundle': 2, 'Graph.CopyInternalNode': 2, 'Graph.CopySubgraph': 2, 'Graph.DisableHebbian': 2, 'Graph.DisableRewardModulation': 2, 'Graph.EnableHebbian': 2, 'Graph.EnableRewardModulation': 2, 'Graph.GraphRawFieldMutation': 2, 'Graph.MutateActionSlotBehavior': 2, 'Graph.MutateGraphOperatorParam': 2, 'Graph.MutateHebbianRate': 2, 'Graph.MutateHebbianRule': 2, 'Graph.MutateRewardSource': 2, 'Graph.MutateTraceDecay': 2, 'Graph.RemoveGraphEdge': 2, 'Graph.RemoveInternalGraphNode': 2, 'Graph.RetargetGraphEdge': 2, 'Graph.SwapGraphOperator': 2, 'Graph.ToggleHebbianLamarckian': 2, 'Topology.AddRouteTarget': 4, 'Topology.CopyNode': 1, 'Topology.MutateGateBias': 2, 'Topology.RemoveNode': 7, 'Topology.RemoveRouteTarget': 27, 'Topology.RetargetNodeTarget': 11, 'Topology.SwapRouteTargets': 43} (total 139).

#### depth 2000
changed_per_all_births 0.005000, dead_per_all_births 0.001500

| Cohort | created | deleted | present | never sel | sel only | applied only | changed only | dispatched-not-contrib | contributing | present/created | dispatched/present | contributing/present |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cohort | 7888 | 450 | 7438 | 2074 | 621 | 49 | 4458 | 202 | 34 | 0.942951 | 0.031729 | 0.004571 |
| graph | 3662 | 213 | 3449 | 677 | 464 | 28 | 2180 | 99 | 1 | 0.941835 | 0.028994 | 0.000290 |
| vm | 4226 | 237 | 3989 | 1397 | 157 | 21 | 2278 | 103 | 33 | 0.943919 | 0.034094 | 0.008273 |

Founder reference row: created 100, deleted 21, present 79, dispatched 2, contributing 2 (contributing/present 0.025316).

| Time to first | reached | reached/created | median gens | censored deleted | censored present |
| --- | --- | --- | --- | --- | --- |
| selection | 5383 | 0.682429 | 109 | 0 | 2505 |
| applicable_selection | 4581 | 0.580756 | 152 | 0 | 3307 |
| internal_change | 4935 | 0.625634 | 125 | 189 | 2764 |
| dispatch | 1892 | 0.239858 | 66 | 354 | 5642 |
| contribution | 88 | 0.011156 | 456 | 444 | 7356 |

Denominator identity, selection row: 5383 + 0 + 2505 = 7888 created.

Retention from depth 1000: 6/36 cohort modules still contributing (0.166667), 28 present not contributing, 2 deleted.

Opportunities (cumulative): births 100000, zero-event 55801 (0.558010), attempted 55142, applied 55140 (0.999964), skipped 2; executed-target 38026 (0.689626 of applied), reachable 38176, unreachable 11544.
attempted_by_domain {'Graph': 14723, 'InputRef': 14705, 'Topology': 10992, 'Vm': 14722}; applied_by_domain {'Graph': 14721, 'InputRef': 14705, 'Topology': 10992, 'Vm': 14722}; selected_inapplicable_by_domain {}; no_eligible_node_by_domain {'Graph': 2}.
Discarded operators — selected but no applicable site {'Graph.AddInternalGraphNode': 121, 'Graph.AlterGraphEdgeWeight': 1215, 'Graph.CopyEdgeBundle': 1314, 'Graph.CopyInternalNode': 542, 'Graph.CopySubgraph': 633, 'Graph.DisableHebbian': 825, 'Graph.DisableRewardModulation': 880, 'Graph.EnableHebbian': 646, 'Graph.EnableRewardModulation': 807, 'Graph.GraphRawFieldMutation': 1265, 'Graph.MutateGraphOperatorParam': 2386, 'Graph.MutateHebbianRate': 2750, 'Graph.MutateHebbianRule': 1578, 'Graph.MutateRewardSource': 1603, 'Graph.MutateTraceDecay': 2937, 'Graph.RemoveGraphEdge': 668, 'Graph.RemoveInternalGraphNode': 499, 'Graph.RetargetGraphEdge': 622, 'Graph.SwapGraphOperator': 1043, 'Graph.ToggleHebbianLamarckian': 1512, 'Vm.CopyConstantBlock': 129, 'Vm.CopyGeneBackwardSlice': 149, 'Vm.CopyGeneForwardSlice': 115, 'Vm.InsertReadBidMotif': 489, 'Vm.InsertReadStoreMotif': 501, 'Vm.MutatePairedSlotAddress': 1651, 'Vm.MutateSlotAddress': 646, 'Vm.VmDeleteInstruction': 91, 'Vm.VmInstructionRawFieldMutation': 372, 'Vm.VmRegisterCountMutation': 391} (total 28380); no eligible node {'Graph.AddGraphEdge': 2, 'Graph.AddInternalGraphNode': 2, 'Graph.AlterGraphEdgeWeight': 2, 'Graph.CopyEdgeBundle': 2, 'Graph.CopyInternalNode': 2, 'Graph.CopySubgraph': 2, 'Graph.DisableHebbian': 2, 'Graph.DisableRewardModulation': 2, 'Graph.EnableHebbian': 2, 'Graph.EnableRewardModulation': 2, 'Graph.GraphRawFieldMutation': 2, 'Graph.MutateActionSlotBehavior': 2, 'Graph.MutateGraphOperatorParam': 2, 'Graph.MutateHebbianRate': 2, 'Graph.MutateHebbianRule': 2, 'Graph.MutateRewardSource': 2, 'Graph.MutateTraceDecay': 2, 'Graph.RemoveGraphEdge': 2, 'Graph.RemoveInternalGraphNode': 2, 'Graph.RetargetGraphEdge': 2, 'Graph.SwapGraphOperator': 2, 'Graph.ToggleHebbianLamarckian': 2, 'Topology.AddRouteTarget': 4, 'Topology.CopyNode': 1, 'Topology.MutateGateBias': 2, 'Topology.RemoveNode': 7, 'Topology.RemoveRouteTarget': 27, 'Topology.RetargetNodeTarget': 11, 'Topology.SwapRouteTargets': 43} (total 139).

### Confluence (seed 33) — drift_depth `drift-depth-v3`, 50 lineages

#### depth 0

Founder generation: no recruited module exists yet (cohort created 0), founders created 100, present 100, dispatched 100, contributing 100; opportunities all zero over 0 births. changed_per_all_births 0.206500, dead_per_all_births 0.000500.

#### depth 22
changed_per_all_births 0.092000, dead_per_all_births 0.000500

| Cohort | created | deleted | present | never sel | sel only | applied only | changed only | dispatched-not-contrib | contributing | present/created | dispatched/present | contributing/present |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cohort | 78 | 2 | 76 | 31 | 1 | 0 | 13 | 24 | 7 | 0.974359 | 0.407895 | 0.092105 |
| graph | 45 | 2 | 43 | 21 | 1 | 0 | 5 | 15 | 1 | 0.955556 | 0.372093 | 0.023256 |
| vm | 33 | 0 | 33 | 10 | 0 | 0 | 8 | 9 | 6 | 1.000000 | 0.454545 | 0.181818 |

Founder reference row: created 100, deleted 1, present 99, dispatched 85, contributing 80 (contributing/present 0.808081).

| Time to first | reached | reached/created | median gens | censored deleted | censored present |
| --- | --- | --- | --- | --- | --- |
| selection | 23 | 0.294872 | 6 | 0 | 55 |
| applicable_selection | 16 | 0.205128 | 5 | 0 | 62 |
| internal_change | 28 | 0.358974 | 5 | 1 | 49 |
| dispatch | 32 | 0.410256 | 5 | 2 | 44 |
| contribution | 7 | 0.089744 | 12 | 2 | 69 |

Denominator identity, selection row: 23 + 0 + 55 = 78 created.

Retention from depth 0: the depth-0 cohort is empty, so `contributing_before` is 0 and the fraction is `Undefined` (founder modules are a separate reference row and never enter retention).

Opportunities (cumulative): births 1100, zero-event 597 (0.542727), attempted 643, applied 643 (1.000000), skipped 0; executed-target 550 (0.855365 of applied), reachable 544, unreachable 37.
attempted_by_domain {'Graph': 172, 'InputRef': 176, 'Topology': 131, 'Vm': 164}; applied_by_domain {'Graph': 172, 'InputRef': 176, 'Topology': 131, 'Vm': 164}; selected_inapplicable_by_domain {}; no_eligible_node_by_domain {}.
Discarded operators — selected but no applicable site {'Graph.AddInternalGraphNode': 1, 'Graph.AlterGraphEdgeWeight': 6, 'Graph.CopyEdgeBundle': 3, 'Graph.CopyInternalNode': 1, 'Graph.CopySubgraph': 2, 'Graph.DisableHebbian': 4, 'Graph.DisableRewardModulation': 5, 'Graph.EnableHebbian': 2, 'Graph.EnableRewardModulation': 5, 'Graph.GraphRawFieldMutation': 1, 'Graph.MutateGraphOperatorParam': 4, 'Graph.MutateHebbianRate': 29, 'Graph.MutateHebbianRule': 8, 'Graph.MutateRewardSource': 11, 'Graph.MutateTraceDecay': 33, 'Graph.RemoveGraphEdge': 3, 'Graph.RetargetGraphEdge': 3, 'Graph.ToggleHebbianLamarckian': 16, 'Vm.InsertReadStoreMotif': 1, 'Vm.MutatePairedSlotAddress': 31, 'Vm.MutateSlotAddress': 21, 'Vm.VmRegisterCountMutation': 1} (total 191); no eligible node {'Topology.RemoveNode': 5, 'Topology.RemoveRouteTarget': 11, 'Topology.RetargetNodeTarget': 5, 'Topology.SwapRouteTargets': 16} (total 37).

#### depth 250
changed_per_all_births 0.013000, dead_per_all_births 0.000500

| Cohort | created | deleted | present | never sel | sel only | applied only | changed only | dispatched-not-contrib | contributing | present/created | dispatched/present | contributing/present |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cohort | 918 | 45 | 873 | 259 | 58 | 4 | 444 | 85 | 23 | 0.950980 | 0.123711 | 0.026346 |
| graph | 445 | 28 | 417 | 103 | 46 | 1 | 213 | 53 | 1 | 0.937079 | 0.129496 | 0.002398 |
| vm | 473 | 17 | 456 | 156 | 12 | 3 | 231 | 32 | 22 | 0.964059 | 0.118421 | 0.048246 |

Founder reference row: created 100, deleted 11, present 89, dispatched 39, contributing 20 (contributing/present 0.224719).

| Time to first | reached | reached/created | median gens | censored deleted | censored present |
| --- | --- | --- | --- | --- | --- |
| selection | 566 | 0.616558 | 24 | 0 | 352 |
| applicable_selection | 487 | 0.530501 | 32 | 0 | 431 |
| internal_change | 558 | 0.607843 | 24 | 22 | 338 |
| dispatch | 249 | 0.271242 | 14 | 37 | 632 |
| contribution | 28 | 0.030501 | 170 | 45 | 845 |

Denominator identity, selection row: 566 + 0 + 352 = 918 created.

Retention from depth 22: 2/7 cohort modules still contributing (0.285714), 5 present not contributing, 0 deleted.

Opportunities (cumulative): births 12500, zero-event 6940 (0.555200), attempted 7001, applied 6990 (0.998429), skipped 11; executed-target 5143 (0.735765 of applied), reachable 5308, unreachable 994.
attempted_by_domain {'Graph': 1882, 'InputRef': 1870, 'Topology': 1424, 'Vm': 1825}; applied_by_domain {'Graph': 1871, 'InputRef': 1870, 'Topology': 1424, 'Vm': 1825}; selected_inapplicable_by_domain {}; no_eligible_node_by_domain {'Graph': 11}.
Discarded operators — selected but no applicable site {'Graph.AddInternalGraphNode': 6, 'Graph.AlterGraphEdgeWeight': 99, 'Graph.CopyEdgeBundle': 95, 'Graph.CopyInternalNode': 36, 'Graph.CopySubgraph': 50, 'Graph.DisableHebbian': 79, 'Graph.DisableRewardModulation': 87, 'Graph.EnableHebbian': 33, 'Graph.EnableRewardModulation': 79, 'Graph.GraphRawFieldMutation': 100, 'Graph.MutateGraphOperatorParam': 176, 'Graph.MutateHebbianRate': 273, 'Graph.MutateHebbianRule': 159, 'Graph.MutateRewardSource': 151, 'Graph.MutateTraceDecay': 336, 'Graph.RemoveGraphEdge': 58, 'Graph.RemoveInternalGraphNode': 52, 'Graph.RetargetGraphEdge': 54, 'Graph.SwapGraphOperator': 68, 'Graph.ToggleHebbianLamarckian': 156, 'Vm.CopyConstantBlock': 9, 'Vm.CopyGeneBackwardSlice': 9, 'Vm.CopyGeneForwardSlice': 7, 'Vm.InsertReadBidMotif': 22, 'Vm.InsertReadStoreMotif': 34, 'Vm.MutatePairedSlotAddress': 233, 'Vm.MutateSlotAddress': 98, 'Vm.VmDeleteInstruction': 11, 'Vm.VmInstructionRawFieldMutation': 29, 'Vm.VmRegisterCountMutation': 37} (total 2636); no eligible node {'Graph.AddGraphEdge': 11, 'Graph.AddInternalGraphNode': 11, 'Graph.AlterGraphEdgeWeight': 11, 'Graph.CopyEdgeBundle': 11, 'Graph.CopyInternalNode': 11, 'Graph.CopySubgraph': 11, 'Graph.DisableHebbian': 11, 'Graph.DisableRewardModulation': 11, 'Graph.EnableHebbian': 11, 'Graph.EnableRewardModulation': 11, 'Graph.GraphRawFieldMutation': 11, 'Graph.MutateActionSlotBehavior': 11, 'Graph.MutateGraphOperatorParam': 11, 'Graph.MutateHebbianRate': 11, 'Graph.MutateHebbianRule': 11, 'Graph.MutateRewardSource': 11, 'Graph.MutateTraceDecay': 11, 'Graph.RemoveGraphEdge': 11, 'Graph.RemoveInternalGraphNode': 11, 'Graph.RetargetGraphEdge': 11, 'Graph.SwapGraphOperator': 11, 'Graph.ToggleHebbianLamarckian': 11, 'Topology.AddNode': 2, 'Topology.AddRouteTarget': 5, 'Topology.MutateGateBias': 2, 'Topology.RemoveNode': 9, 'Topology.RemoveRouteTarget': 31, 'Topology.RetargetNodeTarget': 15, 'Topology.SpliceNode': 1, 'Topology.SwapRouteTargets': 45} (total 352).

#### depth 1000
changed_per_all_births 0.004500, dead_per_all_births 0.000500

| Cohort | created | deleted | present | never sel | sel only | applied only | changed only | dispatched-not-contrib | contributing | present/created | dispatched/present | contributing/present |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cohort | 4117 | 208 | 3909 | 1183 | 302 | 18 | 2212 | 162 | 32 | 0.949478 | 0.049629 | 0.008186 |
| graph | 1919 | 110 | 1809 | 383 | 230 | 8 | 1094 | 92 | 2 | 0.942678 | 0.051962 | 0.001106 |
| vm | 2198 | 98 | 2100 | 800 | 72 | 10 | 1118 | 70 | 30 | 0.955414 | 0.047619 | 0.014286 |

Founder reference row: created 100, deleted 20, present 80, dispatched 5, contributing 5 (contributing/present 0.062500).

| Time to first | reached | reached/created | median gens | censored deleted | censored present |
| --- | --- | --- | --- | --- | --- |
| selection | 2670 | 0.648530 | 63 | 0 | 1447 |
| applicable_selection | 2271 | 0.551615 | 86 | 0 | 1846 |
| internal_change | 2508 | 0.609181 | 68 | 89 | 1520 |
| dispatch | 936 | 0.227350 | 38 | 169 | 3012 |
| contribution | 54 | 0.013116 | 234 | 207 | 3856 |

Denominator identity, selection row: 2670 + 0 + 1447 = 4117 created.

Retention from depth 250: 5/23 cohort modules still contributing (0.217391), 17 present not contributing, 1 deleted.

Opportunities (cumulative): births 50000, zero-event 27756 (0.555120), attempted 27874, applied 27863 (0.999605), skipped 11; executed-target 19251 (0.690916 of applied), reachable 19528, unreachable 5605.
attempted_by_domain {'Graph': 7497, 'InputRef': 7436, 'Topology': 5678, 'Vm': 7263}; applied_by_domain {'Graph': 7486, 'InputRef': 7436, 'Topology': 5678, 'Vm': 7263}; selected_inapplicable_by_domain {}; no_eligible_node_by_domain {'Graph': 11}.
Discarded operators — selected but no applicable site {'Graph.AddInternalGraphNode': 46, 'Graph.AlterGraphEdgeWeight': 483, 'Graph.CopyEdgeBundle': 594, 'Graph.CopyInternalNode': 221, 'Graph.CopySubgraph': 312, 'Graph.DisableHebbian': 368, 'Graph.DisableRewardModulation': 404, 'Graph.EnableHebbian': 268, 'Graph.EnableRewardModulation': 356, 'Graph.GraphRawFieldMutation': 531, 'Graph.MutateGraphOperatorParam': 1048, 'Graph.MutateHebbianRate': 1238, 'Graph.MutateHebbianRule': 695, 'Graph.MutateRewardSource': 760, 'Graph.MutateTraceDecay': 1334, 'Graph.RemoveGraphEdge': 308, 'Graph.RemoveInternalGraphNode': 262, 'Graph.RetargetGraphEdge': 290, 'Graph.SwapGraphOperator': 434, 'Graph.ToggleHebbianLamarckian': 681, 'Vm.CopyConstantBlock': 48, 'Vm.CopyGeneBackwardSlice': 61, 'Vm.CopyGeneForwardSlice': 46, 'Vm.InsertReadBidMotif': 177, 'Vm.InsertReadStoreMotif': 185, 'Vm.MutatePairedSlotAddress': 837, 'Vm.MutateSlotAddress': 327, 'Vm.VmDeleteInstruction': 49, 'Vm.VmInstructionRawFieldMutation': 182, 'Vm.VmRegisterCountMutation': 189} (total 12734); no eligible node {'Graph.AddGraphEdge': 11, 'Graph.AddInternalGraphNode': 11, 'Graph.AlterGraphEdgeWeight': 11, 'Graph.CopyEdgeBundle': 11, 'Graph.CopyInternalNode': 11, 'Graph.CopySubgraph': 11, 'Graph.DisableHebbian': 11, 'Graph.DisableRewardModulation': 11, 'Graph.EnableHebbian': 11, 'Graph.EnableRewardModulation': 11, 'Graph.GraphRawFieldMutation': 11, 'Graph.MutateActionSlotBehavior': 11, 'Graph.MutateGraphOperatorParam': 11, 'Graph.MutateHebbianRate': 11, 'Graph.MutateHebbianRule': 11, 'Graph.MutateRewardSource': 11, 'Graph.MutateTraceDecay': 11, 'Graph.RemoveGraphEdge': 11, 'Graph.RemoveInternalGraphNode': 11, 'Graph.RetargetGraphEdge': 11, 'Graph.SwapGraphOperator': 11, 'Graph.ToggleHebbianLamarckian': 11, 'Topology.AddNode': 2, 'Topology.AddRouteTarget': 7, 'Topology.MutateGateBias': 2, 'Topology.RemoveNode': 9, 'Topology.RemoveRouteTarget': 31, 'Topology.RetargetNodeTarget': 15, 'Topology.SpliceNode': 1, 'Topology.SwapRouteTargets': 47} (total 356).

#### depth 2000
changed_per_all_births 0.006000, dead_per_all_births 0.004000

| Cohort | created | deleted | present | never sel | sel only | applied only | changed only | dispatched-not-contrib | contributing | present/created | dispatched/present | contributing/present |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cohort | 8318 | 425 | 7893 | 2312 | 609 | 52 | 4715 | 176 | 29 | 0.948906 | 0.025972 | 0.003674 |
| graph | 3903 | 217 | 3686 | 804 | 451 | 25 | 2304 | 100 | 2 | 0.944402 | 0.027672 | 0.000543 |
| vm | 4415 | 208 | 4207 | 1508 | 158 | 27 | 2411 | 76 | 27 | 0.952888 | 0.024483 | 0.006418 |

Founder reference row: created 100, deleted 26, present 74, dispatched 5, contributing 4 (contributing/present 0.054054).

| Time to first | reached | reached/created | median gens | censored deleted | censored present |
| --- | --- | --- | --- | --- | --- |
| selection | 5474 | 0.658091 | 118 | 0 | 2844 |
| applicable_selection | 4666 | 0.560952 | 158 | 0 | 3652 |
| internal_change | 5163 | 0.620702 | 124 | 159 | 2996 |
| dispatch | 1947 | 0.234071 | 68 | 333 | 6038 |
| contribution | 78 | 0.009377 | 430 | 421 | 7819 |

Denominator identity, selection row: 5474 + 0 + 2844 = 8318 created.

Retention from depth 1000: 2/32 cohort modules still contributing (0.062500), 29 present not contributing, 1 deleted.

Opportunities (cumulative): births 100000, zero-event 55878 (0.558780), attempted 55204, applied 55193 (0.999801), skipped 11; executed-target 37949 (0.687569 of applied), reachable 38094, unreachable 11705.
attempted_by_domain {'Graph': 14729, 'InputRef': 14699, 'Topology': 11144, 'Vm': 14632}; applied_by_domain {'Graph': 14718, 'InputRef': 14699, 'Topology': 11144, 'Vm': 14632}; selected_inapplicable_by_domain {}; no_eligible_node_by_domain {'Graph': 11}.
Discarded operators — selected but no applicable site {'Graph.AddInternalGraphNode': 120, 'Graph.AlterGraphEdgeWeight': 1068, 'Graph.CopyEdgeBundle': 1286, 'Graph.CopyInternalNode': 491, 'Graph.CopySubgraph': 657, 'Graph.DisableHebbian': 732, 'Graph.DisableRewardModulation': 850, 'Graph.EnableHebbian': 612, 'Graph.EnableRewardModulation': 739, 'Graph.GraphRawFieldMutation': 1185, 'Graph.MutateGraphOperatorParam': 2257, 'Graph.MutateHebbianRate': 2512, 'Graph.MutateHebbianRule': 1465, 'Graph.MutateRewardSource': 1584, 'Graph.MutateTraceDecay': 2746, 'Graph.RemoveGraphEdge': 636, 'Graph.RemoveInternalGraphNode': 553, 'Graph.RetargetGraphEdge': 601, 'Graph.SwapGraphOperator': 919, 'Graph.ToggleHebbianLamarckian': 1425, 'Vm.CopyConstantBlock': 131, 'Vm.CopyGeneBackwardSlice': 154, 'Vm.CopyGeneForwardSlice': 133, 'Vm.InsertReadBidMotif': 481, 'Vm.InsertReadStoreMotif': 465, 'Vm.MutatePairedSlotAddress': 1702, 'Vm.MutateSlotAddress': 682, 'Vm.VmDeleteInstruction': 99, 'Vm.VmInstructionRawFieldMutation': 402, 'Vm.VmRegisterCountMutation': 409} (total 27096); no eligible node {'Graph.AddGraphEdge': 11, 'Graph.AddInternalGraphNode': 11, 'Graph.AlterGraphEdgeWeight': 11, 'Graph.CopyEdgeBundle': 11, 'Graph.CopyInternalNode': 11, 'Graph.CopySubgraph': 11, 'Graph.DisableHebbian': 11, 'Graph.DisableRewardModulation': 11, 'Graph.EnableHebbian': 11, 'Graph.EnableRewardModulation': 11, 'Graph.GraphRawFieldMutation': 11, 'Graph.MutateActionSlotBehavior': 11, 'Graph.MutateGraphOperatorParam': 11, 'Graph.MutateHebbianRate': 11, 'Graph.MutateHebbianRule': 11, 'Graph.MutateRewardSource': 11, 'Graph.MutateTraceDecay': 11, 'Graph.RemoveGraphEdge': 11, 'Graph.RemoveInternalGraphNode': 11, 'Graph.RetargetGraphEdge': 11, 'Graph.SwapGraphOperator': 11, 'Graph.ToggleHebbianLamarckian': 11, 'Topology.AddNode': 2, 'Topology.AddRouteTarget': 7, 'Topology.MutateGateBias': 2, 'Topology.RemoveNode': 9, 'Topology.RemoveRouteTarget': 31, 'Topology.RetargetNodeTarget': 15, 'Topology.SpliceNode': 1, 'Topology.SwapRouteTargets': 47} (total 356).
