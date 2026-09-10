# T11.F18 — backend-neutral mesh node growth: measured readings

Measured evidence relocated on 2026-09-09 from
[`docs/specs/roadmap/t11-f18-backend-neutral-mesh-node-growth.md`](../../specs/roadmap/t11-f18-backend-neutral-mesh-node-growth.md),
unchanged in content. The spec keeps the predeclaration, the verdict, the
mutation survivor record, and every user decision. Machine-written reports:
[gate](../features/t11-f18-backend-neutral-mesh-node-growth.json), [goal](../features/t11-f18-backend-neutral-mesh-node-growth-goal.json).

**Gate report.** `make bench PROFILE=gate FEATURE=t11-f18-backend-neutral-mesh-node-growth` exited 0; `comparison.severe=false`. Stored `t11-f18-backend-neutral-mesh-node-growth.json`, measured code `b16f2820b15cf71d71e6d438a9b848bcfb9986dd`. Generated 2026-09-08T19:12:31Z; host `Isaacs-MacBook-Pro-2.local`, 8 threads.

| Reference | Measure | Current | Reference reading | Delta % | Result |
| --- | --- | --- | --- | --- | --- |
| remove-complementary-nutrition | mesh_hops | 2.028954 | 2.027261 | 0.083512 | ok |
| remove-complementary-nutrition | vm_steps | 22.425973 | 22.751316 | -1.429996 | ok |
| remove-complementary-nutrition | graph_relax_iters | 0.994920 | 0.995234 | -0.031550 | ok |
| remove-complementary-nutrition | plasticity_updates | 0.009880 | 0.011171 | -11.556709 | ok |
| remove-complementary-nutrition | actions_applied | 1.272860 | 1.275525 | -0.208934 | ok |
| remove-complementary-nutrition | births | 0.026902 | 0.026780 | 0.455564 | ok |
| remove-complementary-nutrition | wall ms/creature-tick | 0.0019781652 | 0.0015598310 | 26.819199 | flag |
| t03-f08-genome-size-maintenance-cost | mesh_hops | 2.028954 | 2.029348 | -0.019415 | ok |
| t03-f08-genome-size-maintenance-cost | vm_steps | 22.425973 | 22.443992 | -0.080284 | ok |
| t03-f08-genome-size-maintenance-cost | graph_relax_iters | 0.994920 | 0.995894 | -0.097802 | ok |
| t03-f08-genome-size-maintenance-cost | plasticity_updates | 0.009880 | 0.007563 | 30.635991 | flag |
| t03-f08-genome-size-maintenance-cost | actions_applied | 1.272860 | 1.271998 | 0.067767 | ok |
| t03-f08-genome-size-maintenance-cost | births | 0.026902 | 0.026895 | 0.026027 | ok |
| t03-f08-genome-size-maintenance-cost | wall ms/creature-tick | 0.0019781652 | 0.0013887669 | 42.440402 | host mismatch; raw only |

Observation times (ms): founder 52.399; evolved and drift unmeasured in gate; whole measured run 845.757. Caps remain founder 10 s, evolved 180 s, drift 30 s, whole investigation 900 s. Gate log `/tmp/t11-f18-gate.log`; goal log `/tmp/t11-f18-goal.log`.

**Goal report.** `make bench PROFILE=goal FEATURE=t11-f18-backend-neutral-mesh-node-growth` exited 0; `comparison.severe=false`. Stored `t11-f18-backend-neutral-mesh-node-growth-goal.json`, measured code `b16f2820b15cf71d71e6d438a9b848bcfb9986dd`. Generated 2026-09-08T19:22:34Z; host `Isaacs-MacBook-Pro-2.local`, 8 threads.

| Reference | Measure | Current | Reference reading | Delta % | Result |
| --- | --- | --- | --- | --- | --- |
| t11-f17-executed-biased-mutation-targeting-goal | mesh_hops | 2.068603 | 2.071219 | -0.126302 | ok |
| t11-f17-executed-biased-mutation-targeting-goal | vm_steps | 22.510047 | 146.184594 | -84.601628 | ok |
| t11-f17-executed-biased-mutation-targeting-goal | graph_relax_iters | 0.999660 | 1.002628 | -0.296022 | ok |
| t11-f17-executed-biased-mutation-targeting-goal | plasticity_updates | 0.028933 | 0.029247 | -1.073614 | ok |
| t11-f17-executed-biased-mutation-targeting-goal | actions_applied | 1.244276 | 1.400927 | -11.181953 | ok |
| t11-f17-executed-biased-mutation-targeting-goal | births | 0.015875 | 0.016457 | -3.536489 | ok |
| t11-f17-executed-biased-mutation-targeting-goal | wall ms/creature-tick | 0.0112561105 | 0.0065091316 | 72.927989 | flag |
| t03-f08-genome-size-maintenance-cost-goal | mesh_hops | 2.068603 | 2.074751 | -0.296325 | ok |
| t03-f08-genome-size-maintenance-cost-goal | vm_steps | 22.510047 | 22.556467 | -0.205795 | ok |
| t03-f08-genome-size-maintenance-cost-goal | graph_relax_iters | 0.999660 | 1.000711 | -0.105025 | ok |
| t03-f08-genome-size-maintenance-cost-goal | plasticity_updates | 0.028933 | 0.032645 | -11.370807 | ok |
| t03-f08-genome-size-maintenance-cost-goal | actions_applied | 1.244276 | 1.237238 | 0.568848 | ok |
| t03-f08-genome-size-maintenance-cost-goal | births | 0.015875 | 0.016080 | -1.274876 | ok |
| t03-f08-genome-size-maintenance-cost-goal | wall ms/creature-tick | 0.0112561105 | 0.0057962037 | 94.197978 | host mismatch; raw only |

Observation times (ms): founder 120.926; evolved total 645.460; drift 4388.340; whole measured run 561210.938. Caps remain founder 10 s, evolved 180 s, drift 30 s, whole investigation 900 s. Gate log `/tmp/t11-f18-gate.log`; goal log `/tmp/t11-f18-goal.log`.

**Founder and evolved neighborhoods.** Every founder VM/Graph/input-reference row is byte-identical to T03.F08. All founder topology rows also retain their complete previous tallies. Each detour operator remains 50 silent / 50 applied / 0 changed / 0 dead; paired characterization above explicitly covers both backends. Existing single-event founder silence is unchanged at 98/164 = 0.597561 (the historical reading, not a newly lowered floor).

| Founder operator | Silent / applied | Dead |
| --- | --- | --- |
| AlterGraphEdgeWeight | 35 / 50 | 0 |
| SwapGraphOperator | 7 / 50 | 0 |
| MutateGraphOperatorParam | 50 / 50 | 0 |
| MutateActionSlotBehavior | 50 / 50 | 0 |
| AddInternalGraphNode | 50 / 50 | 0 |
| RemoveInternalGraphNode | 0 / 50 | 0 |
| AddGraphEdge | 47 / 50 | 0 |
| RetargetGraphEdge | 3 / 50 | 0 |
| RemoveGraphEdge | 0 / 50 | 0 |
| GraphRawFieldMutation | 10 / 50 | 0 |
| CopyInternalNode | 50 / 50 | 0 |
| CopySubgraph | 50 / 50 | 0 |
| CopyEdgeBundle | 22 / 50 | 0 |
| EnableHebbian | 28 / 50 | 0 |
| DisableHebbian | 0 / 0 | 0 |
| MutateHebbianRule | 0 / 0 | 0 |
| MutateHebbianRate | 0 / 0 | 0 |
| ToggleHebbianLamarckian | 0 / 0 | 0 |
| EnableRewardModulation | 0 / 0 | 0 |
| DisableRewardModulation | 0 / 0 | 0 |
| MutateRewardSource | 0 / 0 | 0 |
| MutateTraceDecay | 0 / 0 | 0 |
| Add | 50 / 50 | 0 |
| Remove | 4 / 50 | 0 |
| Swap | 5 / 50 | 0 |
| RawFieldMutation | 18 / 50 | 0 |

| Subject | Birth bucket | T03.F08 S/C/D | T11.F18 S/C/D |
| --- | --- | --- | --- |
| founder | all / zero events | 500 / 292 | 500 / 292 |
| founder | any applied events | 113/95/0 (208 applied, 0 skipped, 208 trials) | 115/93/0 (208 applied, 0 skipped, 208 trials) |
| founder | 1 applied events | 98/66/0 (164 applied, 0 skipped, 164 trials) | 98/66/0 (164 applied, 0 skipped, 164 trials) |
| founder | 2 applied events | 12/21/0 (33 applied, 0 skipped, 33 trials) | 14/19/0 (33 applied, 0 skipped, 33 trials) |
| founder | 3 applied events | 3/7/0 (10 applied, 0 skipped, 10 trials) | 3/7/0 (10 applied, 0 skipped, 10 trials) |
| founder | 4 applied events | 0/1/0 (1 applied, 0 skipped, 1 trials) | 0/1/0 (1 applied, 0 skipped, 1 trials) |
| founder | requested-event counts | 0:292, 1:164, 2:33, 3:10, 4:1 | 0:292, 1:164, 2:33, 3:10, 4:1 |
| evolved seed 11 | all / zero events | 2400 / 1300 | 2400 / 1300 |
| evolved seed 11 | any applied events | 700/378/22 (1100 applied, 0 skipped, 1100 trials) | 719/380/1 (1100 applied, 0 skipped, 1100 trials) |
| evolved seed 11 | 1 applied events | 620/279/13 (912 applied, 0 skipped, 912 trials) | 636/275/1 (912 applied, 0 skipped, 912 trials) |
| evolved seed 11 | 2 applied events | 71/76/6 (153 applied, 0 skipped, 153 trials) | 70/83/0 (153 applied, 0 skipped, 153 trials) |
| evolved seed 11 | 3 applied events | 9/17/3 (29 applied, 0 skipped, 29 trials) | 12/17/0 (29 applied, 0 skipped, 29 trials) |
| evolved seed 11 | 4 applied events | 0/4/0 (4 applied, 0 skipped, 4 trials) | 1/3/0 (4 applied, 0 skipped, 4 trials) |
| evolved seed 11 | 5 applied events | 0/2/0 (2 applied, 0 skipped, 2 trials) | 0/2/0 (2 applied, 0 skipped, 2 trials) |
| evolved seed 11 | requested-event counts | 0:1300, 1:912, 2:153, 3:29, 4:4, 5:2 | 0:1300, 1:912, 2:153, 3:29, 4:4, 5:2 |
| evolved seed 22 | all / zero events | 2400 / 1300 | 2400 / 1300 |
| evolved seed 22 | any applied events | 734/340/26 (1100 applied, 0 skipped, 1100 trials) | 760/315/25 (1100 applied, 0 skipped, 1100 trials) |
| evolved seed 22 | 1 applied events | 645/248/19 (912 applied, 0 skipped, 912 trials) | 665/231/16 (912 applied, 0 skipped, 912 trials) |
| evolved seed 22 | 2 applied events | 78/70/5 (153 applied, 0 skipped, 153 trials) | 86/59/8 (153 applied, 0 skipped, 153 trials) |
| evolved seed 22 | 3 applied events | 10/18/1 (29 applied, 0 skipped, 29 trials) | 7/21/1 (29 applied, 0 skipped, 29 trials) |
| evolved seed 22 | 4 applied events | 1/2/1 (4 applied, 0 skipped, 4 trials) | 1/3/0 (4 applied, 0 skipped, 4 trials) |
| evolved seed 22 | 5 applied events | 0/2/0 (2 applied, 0 skipped, 2 trials) | 1/1/0 (2 applied, 0 skipped, 2 trials) |
| evolved seed 22 | requested-event counts | 0:1300, 1:912, 2:153, 3:29, 4:4, 5:2 | 0:1300, 1:912, 2:153, 3:29, 4:4, 5:2 |
| evolved seed 33 | all / zero events | 2400 / 1300 | 2400 / 1300 |
| evolved seed 33 | any applied events | 741/341/18 (1100 applied, 0 skipped, 1100 trials) | 778/322/0 (1100 applied, 0 skipped, 1100 trials) |
| evolved seed 33 | 1 applied events | 650/250/12 (912 applied, 0 skipped, 912 trials) | 683/229/0 (912 applied, 0 skipped, 912 trials) |
| evolved seed 33 | 2 applied events | 77/71/5 (153 applied, 0 skipped, 153 trials) | 83/70/0 (153 applied, 0 skipped, 153 trials) |
| evolved seed 33 | 3 applied events | 13/15/1 (29 applied, 0 skipped, 29 trials) | 10/19/0 (29 applied, 0 skipped, 29 trials) |
| evolved seed 33 | 4 applied events | 0/4/0 (4 applied, 0 skipped, 4 trials) | 1/3/0 (4 applied, 0 skipped, 4 trials) |
| evolved seed 33 | 5 applied events | 1/1/0 (2 applied, 0 skipped, 2 trials) | 1/1/0 (2 applied, 0 skipped, 2 trials) |
| evolved seed 33 | requested-event counts | 0:1300, 1:912, 2:153, 3:29, 4:4, 5:2 | 0:1300, 1:912, 2:153, 3:29, 4:4, 5:2 |

| Evolved seed | Changed topology row | T03.F08 S/C/D | T11.F18 S/C/D |
| --- | --- | --- | --- |
| 11 | RemoveNode | 144/9/7 (160 applied, 80 skipped, 240 trials) | 140/20/0 (160 applied, 80 skipped, 240 trials) |
| 11 | RetargetNodeTarget | 100/29/31 (160 applied, 80 skipped, 240 trials) | 147/13/0 (160 applied, 80 skipped, 240 trials) |
| 11 | AddRouteTarget | 180/0/0 (180 applied, 60 skipped, 240 trials) | 160/0/0 (160 applied, 80 skipped, 240 trials) |
| 11 | RemoveRouteTarget | 140/0/0 (140 applied, 100 skipped, 240 trials) | 160/0/0 (160 applied, 80 skipped, 240 trials) |
| 11 | ChangeEntryNode | 14/185/41 (240 applied, 0 skipped, 240 trials) | 13/191/36 (240 applied, 0 skipped, 240 trials) |
| 11 | CopyMeshBackwardSlice | 239/0/1 (240 applied, 0 skipped, 240 trials) | 240/0/0 (240 applied, 0 skipped, 240 trials) |
| 11 | CopyMeshForwardSlice | 236/4/0 (240 applied, 0 skipped, 240 trials) | 240/0/0 (240 applied, 0 skipped, 240 trials) |
| 11 | SwapRouteTargets | 77/16/47 (140 applied, 100 skipped, 240 trials) | 138/22/0 (160 applied, 80 skipped, 240 trials) |
| 11 | MutateGateBias | 212/9/19 (240 applied, 0 skipped, 240 trials) | 231/9/0 (240 applied, 0 skipped, 240 trials) |
| 22 | RemoveNode | 40/0/0 (40 applied, 200 skipped, 240 trials) | 100/0/0 (100 applied, 140 skipped, 240 trials) |
| 22 | RetargetNodeTarget | 57/0/23 (80 applied, 160 skipped, 240 trials) | 94/0/26 (120 applied, 120 skipped, 240 trials) |
| 22 | AddRouteTarget | 200/0/0 (200 applied, 40 skipped, 240 trials) | 180/0/0 (180 applied, 60 skipped, 240 trials) |
| 22 | RemoveRouteTarget | 60/0/0 (60 applied, 180 skipped, 240 trials) | 120/0/0 (120 applied, 120 skipped, 240 trials) |
| 22 | ChangeEntryNode | 8/213/19 (240 applied, 0 skipped, 240 trials) | 0/199/41 (240 applied, 0 skipped, 240 trials) |
| 22 | SwapRouteTargets | 20/0/40 (60 applied, 180 skipped, 240 trials) | 72/0/48 (120 applied, 120 skipped, 240 trials) |
| 22 | MutateGateBias | 218/0/22 (240 applied, 0 skipped, 240 trials) | 215/0/25 (240 applied, 0 skipped, 240 trials) |
| 33 | RemoveNode | 181/39/0 (220 applied, 20 skipped, 240 trials) | 122/58/0 (180 applied, 60 skipped, 240 trials) |
| 33 | RetargetNodeTarget | 180/29/11 (220 applied, 20 skipped, 240 trials) | 133/47/0 (180 applied, 60 skipped, 240 trials) |
| 33 | AddRouteTarget | 240/0/0 (240 applied, 0 skipped, 240 trials) | 220/0/0 (220 applied, 20 skipped, 240 trials) |
| 33 | RemoveRouteTarget | 180/0/0 (180 applied, 60 skipped, 240 trials) | 120/0/0 (120 applied, 120 skipped, 240 trials) |
| 33 | ChangeEntryNode | 1/214/25 (240 applied, 0 skipped, 240 trials) | 23/217/0 (240 applied, 0 skipped, 240 trials) |
| 33 | CopyMeshBackwardSlice | 238/2/0 (240 applied, 0 skipped, 240 trials) | 240/0/0 (240 applied, 0 skipped, 240 trials) |
| 33 | SwapRouteTargets | 78/49/53 (180 applied, 60 skipped, 240 trials) | 100/20/0 (120 applied, 120 skipped, 240 trials) |
| 33 | MutateGateBias | 201/23/16 (240 applied, 0 skipped, 240 trials) | 231/9/0 (240 applied, 0 skipped, 240 trials) |

**Backend extant counts.** Triples below are total / executed / action-contributing nodes. They are battery-specific extant readings, not creation counts or proof of full-state necessity. Historical reports lack this breakdown and remain unmeasured.

| Subject | Generation depth | Denominator | Graph T/E/C | VM T/E/C |
| --- | --- | --- | --- | --- |
| founder | 0 | 1 genome | 1 / 1 / 1 | 1 / 1 / 1 |
| evolved seed 11 | sample min/max 1/65; population median/max 34/70 | 12 genomes; 80 battery executions each | 20 / 14 / 12 | 20 / 15 / 12 |
| evolved seed 22 | sample min/max 2/51; population median/max 46/59 | 12 genomes; 80 battery executions each | 22 / 15 / 12 | 13 / 13 / 12 |
| evolved seed 33 | sample min/max 1/55; population median/max 41/58 | 12 genomes; 80 battery executions each | 31 / 20 / 11 | 26 / 14 / 13 |
| drift | 0 | 50 lineages; 4000 battery executions | 50 / 50 / 50 | 50 / 50 / 50 |
| drift | 22 | 50 lineages; 4000 battery executions | 96 / 58 / 40 | 90 / 61 / 49 |
| drift | 250 | 50 lineages; 4000 battery executions | 437 / 60 / 1 | 486 / 88 / 44 |
| drift | 1000 | 50 lineages; 4000 battery executions | 1737 / 71 / 1 | 1966 / 117 / 39 |
| drift | 2000 | 50 lineages; 4000 battery executions | 3489 / 100 / 1 | 4028 / 138 / 35 |

| Drift depth | Changed/all births old → new | Dead/all old → new | Mean executed old → new | Mean total | Route variation | Hop-cap hits |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 0.202500 → 0.201500 | 0.000500 → 0.000500 | 2.000000 → 2.000000 | 2.000000 | 0.000000 | 0 |
| 22 | 0.088000 → 0.095000 | 0.000000 → 0.000000 | 2.260000 → 2.380000 | 3.720000 | 0.140000 | 0 |
| 250 | 0.008500 → 0.011000 | 0.001500 → 0.002500 | 2.880000 → 2.960000 | 18.460000 | 0.020000 | 0 |
| 1000 | 0.001500 → 0.010000 | 0.002000 → 0.001000 | 4.280000 → 3.760000 | 74.060000 | 0.020000 | 0 |
| 2000 | 0.008000 → 0.005000 | 0.002000 → 0.001500 | 4.860000 → 4.760000 | 150.340000 | 0.020000 | 0 |

**Goal population and cognitive readings (2026-09-08).**

| Seed | Final population old → new | Minimum | Plateau | Extinction tick | Births | Clades / entropy nats | Memory-sensitive |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 11 | 2153 → 966 | 641 | 788.516000 | None | 257955 | 142 / 2.292697 | 0/966 (0.000000) |
| 22 | 1091 → 1676 | 1321 | 1726.620000 | None | 278669 | 128 / 2.403162 | 0/1676 (0.000000) |
| 33 | 986 → 862 | 821 | 1069.598000 | None | 254872 | 138 / 2.409312 | 0/862 (0.000000) |

Reachable structure distribution: {"max": 293, "mean": "87.371005", "median": 71, "min": 22, "p25": 65, "p75": 100}. Previous: {"max": 336, "mean": "88.999527", "median": 71, "min": 1, "p25": 65, "p75": 111}.

| Seed | Temporal substrate | Sensitive / living | Fraction |
| --- | --- | --- | --- |
| 11 | operator_state | 0/966 | 0.000000 |
| 11 | persisted_outputs | 53/966 | 0.054865 |
| 11 | previous_slots | 0/966 | 0.000000 |
| 22 | operator_state | 12/1676 | 0.007160 |
| 22 | persisted_outputs | 4/1676 | 0.002387 |
| 22 | previous_slots | 0/1676 | 0.000000 |
| 33 | operator_state | 0/862 | 0.000000 |
| 33 | persisted_outputs | 5/862 | 0.005800 |
| 33 | previous_slots | 0/862 | 0.000000 |

Nine deferred cognition indicators remain `Undefined`; no cognition improvement or equality of backend ecological costs is claimed.
