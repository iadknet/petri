# Evolvability exploration, run 3: results (2026-10)

Status: complete. Evidence outcome **C (blocker, by exhaustion)**: every
phase 3 arm ran on both repaired assays, none met the minimum effect on both,
and every arm is `inconclusive` for outcome B. Reviewed by Codex (three
reviews; review 3 `ready`). Contract: the [run 3 plan](evolvability-exploration-plan-2026-10-02-run3.md),
which amends [run 2's](evolvability-exploration-plan-2026-10-02.md) and through
it [run 1's](evolvability-exploration-plan-2026-10-01.md), under the
[T22 exploration contract](../roadmaps/t22-capability-assays-and-evolvability-lab.md).
Launch revision `5ccd8a7fb6dd34feaa575bfecdaa3f54a736208d` (clean `main`,
2026-10-02). Experiment branch `worktree-evolvability-exploration-3` (never
merges). Compact summaries live in
[`evolvability-exploration-2026-10-run3/`](evolvability-exploration-2026-10-run3/);
raw output under `.bench-artifacts/lab/exploration/run3/` in the main checkout.
Previous runs: [run 1](evolvability-exploration-2026-10.md),
[run 2](evolvability-exploration-2026-10-run2.md).

## Question

Unchanged: why do newly added nodes and sensor reads rarely become useful
under Petri's own variation, and which general changes to the representation,
operators, costs or selection setting let them become useful?

## Run setup

- Worktree `.claude/worktrees/evolvability-exploration-3`; `HEAD` equal to the
  launch revision at creation; `npm ci` in `frontend/`; 103 GiB free.
- Run 2's `instr:` energy-ceiling commit cherry-picked as `e6c018fe`.
- Helpers copied from run 2 (`runlab.sh` with the awake-time kill, hash and
  keep scripts); run 1's E1 and E2 elites copied for the panels.
- Unattended (plan rule 10): choices off the stop-and-ask list are made by the
  run and recorded here. No Fable advisor or agent; Codex `gpt-6.1-sol`
  `high` read-only through `codex exec` for reviews and advice.
- Holdout banks still sealed: bank 1 = seeds 101 (food) and 102 (barrier),
  bank 2 = 111 and 112, bank 3 = 121 and 122.

## Ledger

One row per experiment, predeclared before its run; `incomplete` rows kept.

| ID | Phase | Hypothesis | Arms, genomes and controls | Assay, scenes, sizes | Prediction / falsifier | Status | Result |
| --- | --- | --- | --- | --- | --- | --- | --- |
| H0 | 1 | Build: `food-seeking-hunger` exists as plan rule 1 describes and leaves every existing assay byte-identical | `instr:` assay variant on top of `e6c018fe`; the instr check (run 1's food-seed1 and wall-seed1 combinations, three hashes each, against the launch-era hashes) | — | All six hashes unchanged; the new assay resolves start energy and ceiling to the production `initial_energy` (20) | completed (115 s, 70 s) | Held: all six hashes equal the launch-era values after `e6c018fe` and `33ae32f9`; the assay's tests pin start energy and ceiling at 20 |
| H1 | 1 | Freeze: the instrument's descriptor, scene banks and validation panel are fixed before any calibration or validation read | Descriptor committed with: the instr commit; food development bank (32 `sparse-food-v1` scenes, seed `0x3DF00D`) and sealed acceptance bank (32, seed `0x3AF00D`); wall development bank (run 2's `0xB2DE00`) and sealed wall bank (32 `wall-v1` scale-2 scenes, seed `0x3ABA44`); calibration seeds 3 and 4; every scene bank's content hash and every panel genome's hash | — | — | frozen at `c1da23d3` | Descriptor [`food-seeking-hunger-v1.json`](evolvability-exploration-2026-10-run3/food-seeking-hunger-v1.json); manifest [`h1-manifest.ndjson`](evolvability-exploration-2026-10-run3/h1-manifest.ndjson) (4 banks; 40 food and 42 wall panel genomes) |
| H2 | 1 | Requirement 2 (plan rule 2) holds for `food-seeking-hunger` on its development bank | The food panel: bases founder, comparator, run 2's restored plateau, `shuffled-score-5`; on the founder and the comparator every activating edit (gate threshold 0.08, 0.05; gate input weight 2.0) and every abandoning edit (silencing; age gate 0.5 and `Reproduce` votes cleared on activated copies; structure removed on the base and activated copies); silencing alone on the other two | Food development bank under the frozen instrument (harness evaluation path); seconds | Passes if score(a) − score(b) ≤ 0.5 for every abandoning variant a of every base b. Predicted: passes, with every abandoning variant equal to its base (the gate never opens at ≤ 20 energy) except structure removal, which saves a little carrying and compute cost (well under 0.5), and with the activated copies losing score | completed (57 s) | **Passes**: max abandoning gain 0.0. Every abandoning variant equals its base exactly (structure removal too), except vote clearing on activated copies, which is lower (founder −0.64 to −3.70; comparator −13.15 to −18.60: the gate still suppresses moves, so the creature idles). Activated copies lose 0.57–3.79 (founder) and 8.01–13.46 (comparator) |
| H3 | 1 | The frozen instrument calibrates | `v3-lab run --assay food-seeking-hunger --food-fraction 0.04 --lifetime 200 --calibrate-only` on seeds 3 and 4 | Lab gate, unchanged | Calibrated on both seeds (run 2's R4 calibrated on seeds 1 and 2). Falsified: `uncalibrated` on either seed, which makes this version `uncalibrated` for good | completed (seconds) | **Falsified: V1 is `uncalibrated` for good.** Seed 4 calibrates (threshold 20.63; founder 9.28, floor 1.61, comparator 39.65). Seed 3 fails the sensitivity check: comparator progress 0.536 is not above the floor's 0.540, although the comparator scores 38.60 against the floor's 1.29 and wins 16 of 16 scenes. H4 (V1's sealed bank) is not read |
| V2 | 1 | `food-seeking-hunger` version 2: V1 with the lab's standard food-fraction grid instead of the pinned 0.04 | As V1 except the calibration grid: no `--food-fraction`, so the gate takes the first passing point of the standard fractions (0.02, 0.04, 0.08) per seed; lifetime pinned at 200 (plan rule 1). Run decision (unattended): the standard grid is T22.F01's own predeclared rule, not a tuned margin or geometry; no other part changes. Fresh sealed food acceptance bank (32 scenes, seed `0x3BF00D`); V1's unread bank `0x3AF00D` is retired unread. Descriptor committed before its gate runs | Requirement 2 on the development bank (as H2), the gate on seeds 3 and 4, then the sealed bank once | Calibrated on both seeds and requirement 2 holding on both banks; else V2 is `uncalibrated` and, with both versions spent, the food instrument is blocked (outcome C) | completed (seconds; sealed bank 57 s) | **Accepted.** Development: H2's result (same bank and evaluation path). Gate: seed 3 selects food fraction 0.08 (threshold 29.34; founder 23.68, floor 3.38, comparator 55.30), seed 4 selects 0.04 (threshold 20.63; founder 9.28, floor 1.61, comparator 39.65). Sealed bank `0x3BF00D`: max abandoning gain 0.0. The two seed batches therefore run at different food densities |
| B1 | 1 | Baselines and the first pilot size (plan rules 5 and 8): reference campaigns on both repaired assays at quick sizes | Built-in arms only (reference, founder-only, mutation-off, shuffled-score, comparator, random walk); no `proto:` commit | `food-seeking-hunger --lifetime 200` and `barrier-navigation --scale 2 --lifetime 200`, `--quick --replicates 8`, seeds 3 and 4; 15 min each | Records the three hashes per combination as the run 3 baselines, each invocation's wall time, and the reference arm's stall rung and adequacy in each of the 16 replicates per assay. Predicted: food reads `stalls at benefit` with adequacy (as run 2's V1 did); wall reads `inconclusive at retention` (as run 1's E2 did), so quick sizes fail rule 8 on wall | completed (59–110 s per invocation) | Prediction held. **Food**: in 15 of 16 reference replicates the first non-pass rung is `benefit: fail` with adequacy (0–4 improved of 74–118 viable). The 16th (seed 3, replicate 5) reached the threshold (33.58 against 29.34; reference reach 1/8 at seed 3, 0/8 at seed 4). **Wall** (tally corrected after review 1): 9 of 16 replicates stop at `retention: inconclusive` (3–13 observations against 15), 5 at `benefit: fail`, 1 at `retention: fail` (15 observations), and 1 passes every rung. Quick sizes fail rule 8 on wall. Hashes (reference rows, summary projection, all arms): food s3 `87d20636…`, `918cbc87…`, `6d57ec8d…`; food s4 `5d76f6b7…`, `79bd72a8…`, `425da231…`; wall s3 `009b6fb0…`, `54dab2ee…`, `1a784e6a…`; wall s4 `246733b2…`, `9325be74…`, `fde7dd88…` |
| B2 | 1 | Pilot size 2 (population 16, 100 generations), plan rule 8 | As B1 at `--generations 100` | As B1 | Rule 8 passes at this size if every reference replicate on both assays either reaches or reads its first non-pass rung with adequacy, and each invocation stays under 900 s | completed (143–274 s) | **Fails on wall.** Food: 15 replicates `benefit: fail` with adequacy (0–7 improved of 158–305), 1 reached (seed 3, replicate 5). Wall: 7 `benefit: fail` or `retention: fail` with adequacy, 6 pass every rung, 3 `retention: inconclusive` (10, 14 and 10 observations against 15) |
| B3 | 1 | Pilot size 3 (population 32, 100 generations) | As B2 at `--population 32` | As B2 | As B2's criterion. A replicate that passes every rung counts as adequately read | completed (224–417 s) | **Passes; sizes frozen at population 32, 100 generations, retention depth 2.** Food: 14 replicates `benefit: fail` or `retention: fail` with adequacy (378–473 viable; retention 15). Seed 3 replicate 1 reached at generation 12 (`supply: inconclusive 77/288`), seed 3 replicate 6 reached at generation 41 after an adequate read. Wall: every replicate adequately read (`benefit: fail` 312–1,115 viable, `retention: fail` 26–65) or passing every rung; seed 4 replicate 6 reached at generation 98. Reference reach: food 2/8 and 0/8, wall 0/8 and 1/8. Baseline hashes (reference rows, summary projection, all arms): food s3 `e17bc8b7…`, `ba070c6b…`, `6bddc6cf…`; food s4 `dbac6e36…`, `8552a99c…`, `47434966…`; wall s3 `301620ed…`, `d61c4dfe…`, `6d4fa779…`; wall s4 `263dfea5…`, `afb23e14…`, `acaf0517…` |
| E1 | 1 | Run 1's E3, E4 and E7 on `food-seeking-hunger` (phase 1, last step) | E3 founder children; E4 on the founder and B1's seed-4 native and shuffled-score final elites (seed 4 calibrated at fraction 0.04, the density of the probes' scenes); E7 from the seed-4 native elite with the highest E3-bank mean (ties to the lowest index) | Run 1's probe banks (fraction 0.04) under the hunger setup; seconds | Descriptive: whether the founder's and elites' one-step neighbourhoods improve on the repaired instrument. Predicted, as run 2's P3: nearly flat | completed (seconds each) | Prediction held. E3: 4 of 161 genome-changed founder children improve the bank (43 change it; founder 5.94), identical to run 2's P3, because the scenes and energy rule are the same. E4: 0 improvers among the founder's 28 changed children and in 7 of 8 native parents; 7 of the 8 native elites score 6.05 on the bank. E7 from `native-7` (6.05): 2 of 512 grandchildren against 1 of 512 direct children improve (max +0.78), with penalty charged 0.0 for the founder and the parent. That run broke E1's tie-break; the declared parent `native-0` gives 0 of 512 grandchildren and 0 of 512 direct children improving ([`e1-e7-native0-two-step.ndjson`](evolvability-exploration-2026-10-run3/e1-e7-native0-two-step.ndjson)) |
| R3a | 1, 3 | Rule 3's per-elite readings on every campaign: the pilots B1 and B2, the frozen-size baselines B3, and every arm | `r3_elite_readings` on every final elite (not the comparator) | Each assay's development bank; seconds | Descriptive | completed (B1 and B2 added after closing review 2) | **Food, reference elites** (B1, B2, B3: 48 elites): none attempts reproduction, and the largest abandoning delta is 0.02. Shuffled-score elites do attempt (up to 1.07 per tick; one B1 elite gains 0.74 from silencing). **Food, arm elites**: 0–2 of 8 per batch attempt, at 0.001–0.040 per tick. Silencing gains at most 0.05, and clearing the `Reproduce` votes gains at most 0.21, except one M4 seed-4 elite at 0.82. **Wall**: the founder attempts 0.353 per tick (it starts at 100 energy). Reference elites attempt in 4–7 of 8 per batch, and silencing changes their scores by 0.0, except one B1 seed-3 elite (+2.39). Arm elites attempt in 3–7 of 8 per batch; silencing gains exceed m for three of them, in M1 seed 4 (0.85), refinement seed 4 (2.03; votes cleared 1.86) and M5 seed 3 (2.30), and an M2 seed-3 elite gains 0.25. The wall rung movements may therefore carry sterility. Files: `readings-{b1,p2,p3,m1,m2,refine,m1refine,m4,m5}-*.ndjson` |
| H4 | 1 | Requirement 2 holds on the sealed food acceptance bank, read once after H2 and H3 | As H2 | Food acceptance bank `0x3AF00D`, read once | As H2; a failure makes the version `uncalibrated` for good | predeclared | |
| W1 | 1 | Requirement 2 holds for `wall-v1` (plan rule 4) | The wall panel: bases founder, barrier comparator, run 1's E2 wall elites `native-0`, `native-1`, `shuffled-score-1`; every edit on the founder and the barrier comparator, silencing alone on the elites | `wall-v1` scale 2, base rule (start energy 100, no ceiling): the development bank `0xB2DE00`, then the sealed bank `0x3ABA44` read once | Passes as H2. Predicted: passes (run 2: silencing the founder changes its wall score by −0.06 and the barrier comparator's by +0.008). If it fails, the fallback instrument `barrier-navigation-hunger` is built under the full procedure | completed (2 s each) | **Passes**: max abandoning gain 0.008 on the development bank and 0.005 on the sealed bank. `wall-v1` is kept |

Phase 2 rows (predeclared now, run after phase 1 passes and its review):

| ID | Phase | Hypothesis | Arms, genomes and controls | Assay, scenes, sizes | Prediction / falsifier | Status | Result |
| --- | --- | --- | --- | --- | --- | --- | --- |
| C1 | 2 | Path census (H3, H4, H8): what separates the founder from a sensor-guided gain on the repaired instruments | Observation probe (`crates/v3-lab/tests/exploration_census.rs`): 8 lineages per assay replaying the lab's reference-arm rule (population 16, 4 survivors, 12 children per generation from uniformly drawn survivors through the production engine with the parent's frozen last-scene record, 4 fresh training scenes per generation, 40 generations). Every **selected improvement** is recorded: a child above its carried parent on the next generation's scenes and among that generation's survivors. Each record holds its applied operators; its target nodes (founder 0–1 or added); its structural diff (input refs added or changed and their kind; edges added, re-sourced or re-weighted, by source: a sensor or introspection read, a node-boundary read through an upstream slot or shared memory (H8), a compound input, or an internal compute node; vote and output sinks touched; nodes added; VM nodes changed); its delta on the development bank; and a decision-change reading (H4: the share of bank scenes in which the child's trajectory departs from the parent's, and the first departure tick). Descriptive only; never selection or discovery evidence | `food-seeking-hunger` on its development bank `0x3DF00D` and `wall-v1` on `0xB2DE00`; lineage seeds fixed in the probe; 15 min each | Predeclared M5 choice: **H8 cross-node source** if node-boundary reads are at least half of the input-changing edits among selected improvements on either assay, or the witness path (C2) needs a boundary crossing; **VM fresh reads** if instead at least half of the selected improvements change a VM node; if neither holds, H8. Predicted: few selected improvements on food; most re-weight or re-point the founder's node 1 | completed (8 s food, 11 s wall; run by the probe's author as its verification run, before phase 1's review: a recorded phase-order deviation; descriptive, no holdout or arm read) | Prediction held. **Food**: 13 selected improvements in 8 × 40 generations, 12 input-changing, 10 of them boundary reads (share 0.83, per edit 0.84). The dominant edit re-points node 1's `UpstreamSlot` references (`InputRefRawFieldMutation` 6, `InputRefSwap` 2), worth +0.22 on the bank. 1 of 13 touches an added node, no VM changes, 10 of 13 improve the bank (mean delta −0.29). **Wall**: 65 selected improvements, 41 input-changing, boundary share 0.44 (per edit 0.64), VM share 0.11, 16 of 65 touch `Reproduce` sinks. **M5 = H8** (the food boundary share is at least 0.5) |
| C2 | 2 | Witness path (H3): how many native-operator edits separate the founder from a sensor-guided mover above the threshold | Diagnostic genome sequence (plan rule 7): the founder edited one native-operator-applicable step at a time into the built-in comparator's structure (food: `area_food`; wall: `barrier_comparator`); the last step must equal the comparator genome. Every intermediate is scored on the development bank and labelled helpful, neutral (\|Δ\| ≤ 0.05) or harmful against the previous step. Never discovery evidence; its length is an upper bound on the shortest path | As C1; 2 min | Predicted: the food path is about 10 edits, mostly neutral until the edge that connects the gated vector to a move vote; the wall path adds one ring read and four inhibitory edges | completed (seconds; same deviation as C1) | Prediction nearly held; longer. **Food: 13 edits**, founder 8.19 → comparator 41.21: 9 neutral steps (one input reference, four compute nodes, four edges inside the new circuit), then 4 helpful steps, the four vote edges (+19.26, +7.71, +4.14, +1.93). A bootstrapped compute node comes with one input, so each further input is its own edge. The path reads one value across the node boundary (the gate's `UpstreamSlot(1)` input); removing it changes the score by 0.0, so the path does not need a crossing. **Wall: 34 edits** (4 helpful, 30 neutral): the food path plus a ring read, four ring edges, and 16 re-weights (a new edge's weight is drawn in [−1, 1] and a re-weight scales by at most 20 %, so −2.0 needs four). Every final genome equals its comparator exactly |

Phase 3 rows. Sizes frozen at B3: population 32, 100 generations, retention depth 2. Each arm runs in its own invocation per assay and seed batch, with the built-in arms. M3's four-arm factorial would exceed the 900 s kill in one invocation; the reference arm is byte-identical across invocations, which the reference-row hash checks every time, so pairing is preserved. This is a recorded deviation from "all in one invocation". Rows were predeclared before any arm ran; 16 paired replicates as seed batches 3 and 4 on each repaired assay; built-in controls in every invocation; every arm is a mutation-policy change, so `policy-deviation`; rule 3's readings and the masking count for every arm; per-batch results reported beside the pooled 16 because the food batches run at different densities):

| ID | Phase | Hypothesis | Arms, genomes and controls | Assay, scenes, sizes | Prediction / falsifier | Status | Result |
| --- | --- | --- | --- | --- | --- | --- | --- |
| M1 | 3 | Afferent growth (a new sensory synapse forms silent): one event that declares an input and wires it at weight 0 to a vote sink shortens C2's nine-neutral-step path to a useful read | Overlay `{"mutation": {"neutral_input_recruitment": "SingleChannel"}}` (T20.F04). Not forcing: a general rule over input families and vote sinks (family and channel uniform, sink uniform); no source-to-sink pairing; nothing depends on the assay. Still to discover by ordinary variation: which input, which vote sink, a non-zero weight (|w| ≤ 0.01 steps by ±0.1), and any gating | Both assays, frozen sizes | Minimum effect (run 2 rule 1): paired reach gain ≥ 4 of 16, or leaving the reference's stall rung in ≥ 4 of 16. Predicted: no detectable effect on reach; possibly benefit-rung movement on food | completed (379–598 s per invocation; reference rows equal B3's in all four) | **Food**: arm-only reaches 2, reference-only 1 (seed 3: 2 and 1; seed 4: 0 and 0); rung departures 0 of 14 usable pairs (corrected after closing review 2: the first count took a reach for a rung pass); final best +1.23 on average (seed 3 +1.69, seed 4 +0.78). **Wall**: arm-only 0, reference-only 1; departures 4 of 8 usable pairs (9 used, 1 unusable; the other 7 references pass every rung or reached); final best −0.38. The minimum effect is met on wall (rung) only, so M1 is no A candidate. 2 arm-only reaches give an exact one-sided 95 % bound of 0.34 > 0.25: M1 is `inconclusive` for B, which **rules outcome B out for this run** (run 2 rule 1). Pairs: [`m1-pairs.json`](evolvability-exploration-2026-10-run3/m1-pairs.json) (filter [`pairs.jq`](evolvability-exploration-2026-10-run3/pairs.jq)) |
| M2 | 3 | As M1, a whole receptive field per event | Overlay `{"mutation": {"neutral_input_recruitment": "WholeFamily"}}`. Not forcing as M1. Still to discover: as M1, per family | As M1 | As M1 | completed (322–689 s; reference rows equal B3's in all four) | **Food**: arm-only reaches 2, reference-only 1 (all in seed 3); departures 0 of 14; final best +1.33 (seed 3 +1.89, seed 4 +0.78). **Wall**: arm-only 0, reference-only 1; departures 4 of 8 (1 of 9 unusable); final best −0.56. As M1: the minimum effect is met on wall (rung) only, so M2 is no A candidate, and it is `inconclusive` for B. Pairs: [`m2-pairs.json`](evolvability-exploration-2026-10-run3/m2-pairs.json) |
| M3 | 3 | Interaction (predeclared): repeated circuit motifs varying together help only once access exists, so M1 + refinement exceeds M1 by more than refinement alone exceeds the reference | One invocation per assay and seed batch: reference, M1, refinement `{"mutation": {"structured_heritable_refinement": true}}` (T20.F05) and M1 + refinement. Refinement keeps T20.F05's existing groups and supplies no correspondence. Still to discover: as M1, plus which groups to co-vary | As M1 | Interaction falsified unless (M1+R − M1) − (R − reference) in paired reach exceeds 0 in more pairs than not; each arm also read against the minimum effect | completed (310–597 s per invocation; reference rows equal B3's in all eight) | **Interaction falsified**: (M1+R − M1) − (R − reference) is positive in 2 of 32 pairs (1 food, 1 wall) ([`m3-interaction.ndjson`](evolvability-exploration-2026-10-run3/m3-interaction.ndjson)). **M1 + R**: food arm-only reaches 3, reference-only 1 (all in seed 3); departures 0 of 14; final best +2.01 (seed 3 +3.88, seed 4 +0.14). Wall: arm-only 0, reference-only 1; departures 4 of 9; final best −0.32. No A candidate; `inconclusive` for B ([`m1refine-pairs.json`](evolvability-exploration-2026-10-run3/m1refine-pairs.json)). **Refinement (R)**: food arm-only reaches 3, reference-only 2 (all in seed 3); departures 0 of 14; final best +2.15 (seed 3 +3.63, seed 4 +0.68). Wall: arm-only 0, reference-only 1; departures 6 of 8 (1 of 9 unusable); final best −0.71. Minimum effect on wall (rung) only; no A candidate; `inconclusive` for B. Pairs: [`refine-pairs.json`](evolvability-exploration-2026-10-run3/refine-pairs.json) |
| M4 | 3 | A mutator background: four times the per-unit rate crosses a valley the reference cannot | Overlay `{"mutation": {"per_unit_rate": 0.02}}`. Not forcing: supply only. Still to discover: the whole path | As M1 | Predicted: no detectable effect (run 1's E8 gave 2/8 against 0/8 on the unrepaired food assay) | completed (315–592 s; reference rows equal B3's in all four) | Prediction held. **Food**: arm-only reaches 3, reference-only 2 (all in seed 3); departures 0 of 14; final best +0.73 (seed 3 +0.62, seed 4 +0.85). **Wall**: arm-only 0, reference-only 1; departures 3 of 8 (1 of 9 unusable); final best −0.64. The minimum effect is met on neither assay; `inconclusive` for B. Pairs: [`m4-pairs.json`](evolvability-exploration-2026-10-run3/m4-pairs.json) |
| M5 | 3 | H8, axon growth forming a direct projection between regions: a cross-node edge source removes the census's dominant boundary | `proto:` field `cross_node_edge_sources` (default off) and operator `AddCrossNodeEdge` (as `AddGraphEdge`, reading another node's last output this tick, zero before it runs), overlay `{"mutation": {"cross_node_edge_sources": true}}`. Not forcing: general over node kinds. Matched controls: the fixed founder and the comparator under the arm's semantics (no `NodeOutput` edge, so unchanged by construction; checked), with the reference's mutation-off and shuffled-score arms on the same scenes. Still to discover: which node, slot, consumer and weight | As M1 | Prior art (below) predicts no gain in two-node meshes, where the bus already carries the other node's output; predicted no detectable effect | completed on the `proto:` build `a78dc95b` (307–576 s; `AddCrossNodeEdge` applied 164 times in food seed 3 alone) | Prediction held. **Matched controls**: every built-in arm's rows (reference, founder-only, mutation-off, shuffled-score, comparator, random walk) are byte-identical to B3's on all four combinations, so the prototype build leaves the fixed founder, the comparator and the controls unchanged. **Food**: arm-only reaches 1, reference-only 2; departures 0 of 14; final best +0.52 (seed 3 +0.17, seed 4 +0.86). **Wall**: arm-only 0, reference-only 1; departures 3 of 8 (1 of 9 unusable); final best −0.54. The minimum effect is met on neither assay; `inconclusive` for B (1 arm-only reach gives a bound of 0.26). Pairs: [`m5-pairs.json`](evolvability-exploration-2026-10-run3/m5-pairs.json) |

## Frontier table

Descriptive only. Each cell is a paired difference from the reference arm over
16 pairs (seed batches 3 and 4), at population 32 and 100 generations. Reach is
given as arm-only − reference-only reaches, and the stall rung as the pairs in
which the arm leaves the reference's stall rung, out of the usable pairs. The
mean final-best gap is shown in parentheses. No cell carries a supported
difference: every count lies inside its exact one-sided 95 % bound. The table
never ranks or retires an arm.

| Arm | `food-seeking-hunger` V2 | barrier-navigation `wall-v1` |
| --- | --- | --- |
| reference | reached 2/16 (seed 3 at fraction 0.08: 2/8; seed 4 at 0.04: 0/8); every one of the 14 replicates that did not reach stalls at `benefit: fail` with adequacy | reached 1/16; 9 replicates stall with adequacy (5 at benefit, 4 at retention); 7 pass every rung, one of which reached |
| M1 recruitment, single channel | reach 2 − 1; rung 0 of 14; (+1.23); no detectable difference | reach 0 − 1; rung 4 of 8; (−0.38); no detectable difference |
| M2 recruitment, whole family | reach 2 − 1; rung 0 of 14; (+1.33); no detectable difference | reach 0 − 1; rung 4 of 8; (−0.56); no detectable difference |
| M3 refinement | reach 3 − 2; rung 0 of 14; (+2.15); no detectable difference | reach 0 − 1; rung 6 of 8; (−0.71); no detectable difference |
| M3 M1 + refinement (interaction falsified, 2 of 32 pairs positive) | reach 3 − 1; rung 0 of 14; (+2.01); no detectable difference | reach 0 − 1; rung 4 of 9; (−0.32); no detectable difference |
| M4 rate × 4 | reach 3 − 2; rung 0 of 14; (+0.73); no detectable difference | reach 0 − 1; rung 3 of 8; (−0.64); no detectable difference |
| M5 cross-node source (H8, `proto:`) | reach 1 − 2; rung 0 of 14; (+0.52); no detectable difference | reach 0 − 1; rung 3 of 8; (−0.54); no detectable difference |

Every food arm's mean final-best gap is positive in both batches. All arms are
compared with one shared reference realization, so a reference below its own
expectation would produce exactly that pattern. It is not a claim.

## Findings by phase

### Phase 1: the repaired instruments

Summaries: [`h1-manifest.ndjson`](evolvability-exploration-2026-10-run3/h1-manifest.ndjson),
[`h1-manifest-v2.ndjson`](evolvability-exploration-2026-10-run3/h1-manifest-v2.ndjson),
the two descriptors, and the requirement-2 readings listed under H2, V2 and W1.

- **`food-seeking-hunger` V1** (`e6c018fe` + `33ae32f9`, frozen at
  `c1da23d3`) met requirement 2 on its development bank exactly. Every
  abandoning variant of every base scores its base or less, while the
  activated copies (gate lowered or input weight raised) lose up to 13.46.
  But V1 failed its gate on seed 3. The gate's sensitivity check needs the
  comparator's interval progress above the random-walk floor's: 0.536
  against 0.540. That fails although the comparator scored 38.60 against
  1.29. Under the ceiling the floor dies early, part-way into an interval,
  so the progress check is fragile at fraction 0.04. V1 is `uncalibrated`
  for good; its sealed bank was retired unread.
- **V2** (frozen at `bf41f995`) changes only the calibration grid, back to
  T22.F01's standard food-fraction grid with the first passing point per
  seed. It calibrates on both seeds and passes the sealed bank exactly. The
  two seed batches run at different food densities: seed 3 at 0.08
  (founder 23.68 against a threshold of 29.34) and seed 4 at 0.04 (9.28
  against 20.63). Pairing is within a seed, so contrasts stay paired; pooled
  readings mix two densities.
- **`wall-v1`** passes requirement 2 on its development and sealed banks
  (largest abandoning gain 0.008).

### Phase 2: census, witness path and M5's prior art

Summaries: [`c1-food-census-summary.ndjson`](evolvability-exploration-2026-10-run3/c1-food-census-summary.ndjson),
[`c1-food-census-records.ndjson`](evolvability-exploration-2026-10-run3/c1-food-census-records.ndjson),
[`c1-wall-census-summary.ndjson`](evolvability-exploration-2026-10-run3/c1-wall-census-summary.ndjson),
[`c2-food-witness-steps.ndjson`](evolvability-exploration-2026-10-run3/c2-food-witness-steps.ndjson),
[`c2-wall-witness-summary.ndjson`](evolvability-exploration-2026-10-run3/c2-wall-witness-summary.ndjson).

- C1 is a separate reference-rule simulation, not an exact replay of the
  lab campaign (its RNG construction and tie keys differ), and its
  departure reading observes position, not vote margin, so it cannot show
  incumbent masking (H4). Its "boundary" class is structural: re-pointing
  an existing `UpstreamSlot` read counts, so the food share (0.83) records
  that selection mostly rewires the incumbent's reads of node 0, not that a
  node boundary blocks new tissue. Only 1 of 13 food improvements touches an
  added node.
- C2's lengths (13 food, 34 wall) are upper bounds for those constructions.
  The food path's nine neutral steps before any helpful one are the
  stepping-stone problem in concrete form: a sensor-guided vote needs a new
  reference, a gated vector and its wiring before the first edge that pays.
- **Prior art for M5** (primary texts read in full by a research subagent;
  extracts kept in the session scratchpad):
  - Stanley and Miikkulainen 2002 (NEAT, *Evolutionary Computation*
    10(2):99–127, doi:10.1162/106365602320169811): new connections take a
    random weight, usually lower fitness at first, and survive mainly under
    speciation (§2.3, §3.1–3.3).
  - Miller 2020 (*GPEM* 21:129–168, §3.2, §12): module comparisons were
    unfair because modular runs had larger genotypes, and extending what
    mutation can reach (EGGP, recurrent CGP) is what helped.
  - Walker and Miller 2008 (*IEEE TEC* 12(4)): modules helped only on some
    hard problems.
  - Kaiser and Hilgetag 2006 (*PLoS Comput Biol* 2(7):e95): long-range
    projections pay by cutting processing steps, and give no payoff where
    regions are already directly connected.
  - Clune, Mouret and Lipson 2013 (*Proc R Soc B* 280:20122863): a
    connection cost yields modularity and faster re-adaptation.

  In the two-node founder, node 1 already reads node 0's output through the
  bus, so a direct projection skips nothing. M5 is predicted null and is
  built only because the predeclared rule chose it.

## Review dispositions

Review 1 (end of phases 1 and 2, Codex `gpt-6.1-sol` `high`, read-only,
`.bench-artifacts/lab/exploration/codex/run3-review-1*`), verdict `not-ready`:

| # | Finding | Severity | Disposition |
| --- | --- | --- | --- |
| 1 | B2's "either reaches or" relaxes rule 8; B1's wall tally is wrong | blocking | Tally corrected (9 retention-inconclusive, 5 benefit-fail, 1 retention-fail, 1 all-pass). On rule 8, partly accepted, as a recorded run decision. Rule 8 exists so that outcome B's rung contrast is readable. Run 2 rule 1 reads rungs only "in every replicate the rung contrast uses", and a reference replicate that reached has no stall rung, so no arm can leave it and the rung contrast never uses that pair. Rule 8 therefore counts a reached reference replicate as resolved. At B3 this matters for one replicate (food seed 3, replicate 1); every other replicate on both assays is adequately read |
| 2 | Rule 3's per-elite readings and ancestry trace are missing for the baselines | blocking | Accepted: `r3_elite_readings` probe added (reproduce attempts per tick, penalty, deaths, food, both abandoning deltas) and run on every campaign: B1, B2, B3 and every arm (row R3a; corrected after closing review 2, which found B1's and B2's readings missing). The ancestry files `b1-*-ancestry.json` are a compact projection: per replicate, the count of generations whose best was charged a penalty and the largest charge. The per-generation traces are the raw campaign rows (`rows.ndjson`, `best_scenes[].penalty_charged`) under `.bench-artifacts/lab/exploration/run3/` |
| 3 | Requirement-2 readings are not tracked; no absolute differences | blocking | Accepted: [`h2-food-dev.ndjson`](evolvability-exploration-2026-10-run3/h2-food-dev.ndjson), [`v2-food-accept.ndjson`](evolvability-exploration-2026-10-run3/v2-food-accept.ndjson), [`w1-wall-dev.ndjson`](evolvability-exploration-2026-10-run3/w1-wall-dev.ndjson) and [`w1-wall-accept.ndjson`](evolvability-exploration-2026-10-run3/w1-wall-accept.ndjson) tracked with provenance and `abs_delta` |
| 4 | V2 is an admissible second version (outcome-informed, not tuning); the two densities make phase 3 an 8 + 8 mixture: report each batch beside the pooled result | advisory | Accepted: per-batch reporting predeclared in the phase 3 rows |
| 5 | Requirement 2's construction and directional test are correct; the passes certify the panel only | advisory | Accepted |
| 6 | C1 is not an exact campaign replay; its departure reading is not a vote margin; "boundary" is structural, not a bottleneck; C2's lengths are upper bounds and do not show H8 is generally unlikely to matter | advisory | Accepted; phase 2 findings worded accordingly |
| 7 | E1 broke its tie-break (seven native elites tie; the declared parent is `native-0`, E7 used `native-7`) | advisory | Accepted: the `native-7` run is kept as a deviation and E7 is rerun from `native-0` |
| 8 | No holdout leak; M1–M4 as declared with the full M3 factorial; H8 general, default-off, with matched controls; no witness or census wiring seeds an arm | advisory | Accepted; phase 3 rows above |

Review 2 (closing review, Codex `gpt-6.1-sol` `high`, read-only,
`.bench-artifacts/lab/exploration/codex/run3-review-2*`), verdict `not-ready`
(C supported):

| # | Finding | Severity | Disposition |
| --- | --- | --- | --- |
| 1 | `pairs.jq` counted a reaching arm as a rung departure and exempted reached arms from usability; the food departures were all reaches, and five wall denominators included an unusable pair | blocking | Accepted. Filter corrected: a departure needs an adequate `pass` at the reference's stall rung. Counts recomputed: food 0 of 14 for every arm; wall 4/8, 4/8, 6/8, 4/9, 3/8, 3/8 (M1, M2, R, M1+R, M4, M5). Ledger and frontier updated; C unchanged |
| 2 | B1 and B2 elite readings missing; ancestry files are a projection; rule 9's validation, food and death differences missing | blocking | Accepted: readings run on B1 and B2 (row R3a); ancestry described as a projection, with the traces in the raw rows; rule 9 readings added (`<arm>-rule9.json`) |
| 3 | Workspace Clippy not shown (`make -k` stops the recipe at the pin); the complete check ran after M5's results | blocking | Accepted: `cargo clippy --workspace --all-targets -- -D warnings` exits 0; the timing deviation is recorded under Branch and commits |
| 4 | The rule 8 decision is a documented amendment of the literal pilot criterion, not literal compliance | advisory | Accepted: so labelled here |
| 5 | Overclaiming phrases ("no arm moved reach", "not gains", "none produces") | advisory | Accepted; reworded as observed counts and absent qualifying evidence |
| 6 | R3a's numbers (M4 food 0.040 per tick and votes-cleared 0.82; M2 wall 0.25) | advisory | Accepted; corrected |
| 7 | State the density and seed-batch confound; the diagnostic banks characterise 0.04 only | advisory | Accepted; added to the phase 3 findings |
| 8 | The next-run claims overreach (stream divergence as the cause, 48 pairs, a necessary stepping stone) | advisory | Accepted; rewritten |

### Phase 3: mechanism arms

Pair files: `m1-pairs.json`, `m2-pairs.json`, `refine-pairs.json`,
`m1refine-pairs.json`, `m4-pairs.json`, `m5-pairs.json`, computed with
[`pairs.jq`](evolvability-exploration-2026-10-run3/pairs.jq). Compact run
summaries: `<arm>-<assay>-s<seed>.json`. Rule 3 readings: `readings-*.ndjson`.

- **Observed reach counts.** On food the arms' arm-only reaches (1–3) are
  close to the reference-only reaches (1–2), and every one of them falls in
  the seed-3 batch at food fraction 0.08. There the founder (23.68) starts
  close to the threshold (29.34). At fraction 0.04 (seed 4) neither the
  reference nor any arm reaches. On wall no arm reaches, and the reference
  reaches once. No qualifying reach evidence was found.
- **Food rung: no departures.** After the correction from closing review 2
  (a reach is not a rung pass), no arm leaves the reference's food stall
  rung in any of the 14 usable pairs.
- **Wall rung: departures with lower scores.** The arms leave the reference's
  wall stall rung in 3 to 6 of 8 or 9 usable pairs, which reaches the
  declared threshold of 4 for M1, M2, refinement and M1 + refinement. Their
  mean final bests are 0.3–0.7 lower, and rule 9's food eaten by the final
  best is lower too (−1.75 to −2.81 per pair). The benefit rung measures the
  share of viable touching children that improve on their parent. In the B1
  audit, shuffled-score selection moved it as well. That limits what the
  rung shows; the cause of these departures is not established.
- **Rule 9 descriptive readings** (`<arm>-rule9.json`). On food every arm's
  final best eats more on its training scenes than the reference's: +2.2
  (M5) to +8.8 (refinement) bites per pair, with equal deaths except one
  M1 + refinement pair (−1). The lab
  computes validation means only when a replicate's best crosses the
  training threshold, so the validation gap is undefined for most pairs.
- **Two confounds.** Density and seed batch are confounded: seed 3 runs at
  fraction 0.08 and seed 4 at 0.04. The fixed food diagnostic banks (the
  census, the rule 3 readings and E1) characterise 0.04, not the 0.08
  stratum. All arms also share one reference realization.
- **Recruitment and refinement: no qualifying reach difference.** M1 and
  M2 differ from each other only in children where recruitment applied
  (their food seed-4 final bests are identical in all 8 replicates). The
  predeclared M3 interaction is positive in 2 of 32 pairs.
- **H8 behaved as the prior art predicted**: 164 applied cross-node edges in
  one batch and no detectable difference, in founders whose second node
  already reads the first through the bus.
- **Sterility under the hunger instrument** (rule 3, `readings-*.ndjson`):
  no reference elite attempts reproduction on the food development bank,
  and the elite-level silencing deltas stay small (row R3a). The masking count
  for B was not computed, a recorded deviation: B was already ruled out by
  M1's arm-only reaches, and masking can only add counts. The elite
  counterfactual for A was not needed, because no arm produced an A
  candidate.

## Evidence outcome

**C. Blocker, by exhaustion.** Every phase 3 arm (M1, M2, M3 as refinement
and M1 + refinement, M4, and the M5 prototype) ran with 16 paired replicates
on both repaired assays. None met run 2's minimum effect on both assays, so
there is no confirmation candidate and phase 4 did not run. Every arm is
`inconclusive` for outcome B: food arm-only reaches of 1, 2 and 3 put the
exact one-sided 95 % bounds at about 0.264, 0.344 and 0.417, above 0.25; wall
rung departures of 3–6 do the same. B is therefore unavailable,
and no further predeclared arm remains. The plan allows a second prototype
only if the first is clearly negative. M5 is `inconclusive`, not clearly
negative, and the census found no VM tissue in the food incumbent path, so no
second prototype was built.

What this says: at the frozen sizes (population 32, 100 generations, two seed
batches), no tested general change showed qualifying evidence of a reach or
rung effect of a quarter of the replicates on both the repaired food
instrument and `wall-v1`. Effects of that size were not excluded either, and
smaller ones can neither be shown nor excluded with 16 pairs.

### Candidates

- **T22 feature candidate (instrument):** `food-seeking-hunger`, built from
  `instr:` commits `e6c018fe` (energy ceiling) and `33ae32f9` (the assay). It
  passes requirement 2 on development and sealed banks, and it calibrates on
  T22.F01's standard food-fraction grid. Its caveats are V1's progress-check
  fragility at fraction 0.04 under the ceiling, and that it cannot remove the
  shortcut from a descendant whose reproduction re-activates.
- **No roadmap mechanism feature.** The H8 prototype (`proto:` `a78dc95b`)
  is not qualified.

### What a next run could change (decided by the run, recorded for a run 4)

- **Sample size.** With 16 pairs, B's bound needs zero discordant pairs.
  Every arm here had 1–3 arm-only food reaches. Mutation streams diverge from
  the reference once the operator pool changes, which may contribute to
  such discordance; whether and how much was not measured. A run aiming at B needs more
  pairs per arm, which the 15-minute kill allows only as more seed batches.
  How many is not established here.
- **One construction with nine neutral edits.** C2 found a food path with
  nine neutral native edits before the first helpful one. It is one
  construction, an upper bound, not proof that evolution must take that
  path; a general mechanism that shortens such paths is an open question.

## Execution status

`complete`: outcome C reached and reviewed (run 3 reviews 1–3; review 3,
the confirmation of the closing review, verdict `ready`). Review 3 confirmed
every closing-review finding and raised three small wording fixes: an
unestablished cause in the next-run section, one M1 + refinement death gap
of −1, and M1's count of 7 excluded references. All three were applied
before landing. Holdout banks 1–3 were never read. The run finished well within its 400-turn
budget.

## Landing

In run 1's close order:

1. Final note and summaries committed on the branch. Raw evidence copied to
   the main checkout's `.bench-artifacts/lab/exploration/run3/`: campaign
   outputs, probe outputs, the pinned binaries, scripts and logs. Codex briefs
   and outputs go to `.bench-artifacts/lab/exploration/codex/run3-*`. Each
   summary's recorded raw sha256 is checked against the preserved copy
   before the worktree is removed.
2. `ExitWorktree` with keep; `main` rechecked against the launch revision.
3. Landed on `main`: this note and its summaries folder, as one docs commit.
   Every run 3 `lab:` commit (the probes) needs the `instr:` commit
   `e6c018fe`, so none builds on `main`; they stay on the branch with the
   `instr:` and `proto:` commits.
4. `make check-docs` on `main`, then the worktree is removed and the branch
   kept.

## Branch and commits

Branch `worktree-evolvability-exploration-3`, kept and never merged, from launch
`5ccd8a7f`. The commits by kind:

- **`instr:`** (stay on the branch; the T22 candidate above):
  - `e6c018fe`: the energy ceiling, cherry-picked from run 2's `0328335e`.
  - `33ae32f9`: the `food-seeking-hunger` assay.
- **`proto:`** (stays on the branch): `a78dc95b`, the M5/H8 cross-node edge
  source, default off.
- **`lab:`** (the probes and the ledger, all needing the `instr:` commits):
  - `d92b3e5a`: the panel probes.
  - `bf41f995`, `e61bdb2a`: H0–H3, V2, the hunger switch.
  - `4c40b640`: the census and witness-path probes.
  - `0d364d61`: the rule 3 elite readings.
  - `ddfcae9c`: the masking probe.
- **`docs:`**: the note and summaries, from `c1da23d3` to the closing record.

Verification on the branch, with the prototype included: `make -k check`
fails only the known v3-cli recipe digest pin
(`checked_in_goal_recipe_identities_are_unchanged_by_json_precision`). The 33
other Rust test suites, the viability suite, frontend tests and build,
dependency audit and skill check all pass. `make -k` does not reach the
recipe's workspace Clippy step after the pin fails, so `cargo clippy
--workspace --all-targets -- -D warnings` was run separately and exits 0
(`clippy-workspace.log`). Timing deviation: the complete check ran after
M5's campaigns and readings, not before its results were counted. The
prototype's subagent ran viability, the `v3-core` suite and workspace Clippy
before M5 started. The `instr:` check after both `instr:` commits
held: all six launch hashes are unchanged. Every arm invocation's reference
rows equal B3's, and M5's built-in arms equal B3's on every combination.
Logs: `.bench-artifacts/lab/exploration/run3/make-check-proto-k.log` and
`arms-overlay.log`, `m5.log`.
