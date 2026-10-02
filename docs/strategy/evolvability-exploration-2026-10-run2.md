# Evolvability exploration, run 2: results (2026-10)

Status: complete; evidence outcome **C (blocker)**: no admissible, validated
food-seeking instrument exists for the required two-assay mechanism
contrasts. Reviewed by Codex (four reviews, one advice round; review 4 verdict `ready`). Contract: the [run 2 plan](evolvability-exploration-plan-2026-10-02.md),
which amends the [run 1 plan](evolvability-exploration-plan-2026-10-01.md), under the
[T22 exploration contract](../roadmaps/t22-capability-assays-and-evolvability-lab.md).
Launch revision `eefca5db369cc1c90ffdc943845340518af5efaa` (clean `main`, 2026-10-01).
Experiment branch `worktree-evolvability-exploration-2` (never merges). Compact
summaries cited below live in [`evolvability-exploration-2026-10-run2/`](evolvability-exploration-2026-10-run2/);
raw output under `.bench-artifacts/lab/exploration/run2/` in the main checkout.
Run 1's results: [evolvability-exploration-2026-10.md](evolvability-exploration-2026-10.md).

## Question

Unchanged from run 1: why do newly added nodes and sensor reads rarely become
useful under Petri's own variation, and which general changes to the
representation, operators, costs or selection setting let them become useful?

## Run setup

- Worktree `.claude/worktrees/evolvability-exploration-2`, `HEAD` equal to the
  launch revision at creation; `npm ci` in `frontend/`; 95 GiB free.
- Helpers copied from run 1 into the worktree's `.bench-artifacts/`. `runlab.sh`
  replaced (plan rule 6): `caffeinate -i`, and a kill that counts awake time
  only (5 s polls, at most 10 s credited per poll), so a host sleep does not
  count toward 900 s; a run is `incomplete` when its summary's
  `timing.wall_seconds` exceeds 900 s.
- No Fable advisor, agent or reviewer; Codex `gpt-6.1-sol` `high` read-only
  through `codex exec` for reviews and advice. Code-reading subagents run on
  the session model (Opus).
- Holdout banks reserved (plan rule 5), all sealed: bank 1 = seeds 101 (food)
  and 102 (barrier); bank 2 = 111 and 112; bank 3 = 121 and 122.

## Ledger

One row per experiment, predeclared before its run; `incomplete` rows kept.

| ID | Phase | Hypothesis | Arms, genomes and controls | Assay, scenes, sizes | Prediction / falsifier | Status | Result |
| --- | --- | --- | --- | --- | --- | --- | --- |
| B0 | 0 | Baseline identity: run 1's two stored combinations reproduce at this launch revision | None (built-in arms) | food-seed1 and wall-seed1 commands from run 1's `baseline-hashes.json` | All three hashes equal run 1's for both | completed (193 s, 126 s) | Held: all six hashes equal run 1's (`48fcfe2c…`, `797814a3…`, `1c7d1195…`; `5b0b7948…`, `8560ac5f…`, `412ce1e5…`) |
| P1 | 1 | Sterility shortcut: the plateau's food-seeking gain comes from silencing the reproduce readiness signal, not from foraging | Diagnostic genomes (plan rule 7, never selected or seeded): founder 2 × 2 = readiness silenced (node 0 `CustomOutput(1)` inputs cleared) × the plateau's upstream re-pointing (node 1 input ref 1 → slot 12, ref 2 → slot 9, as in `native-1`); decomposition: ref 1 only, ref 2 only; the plateau `native-1`; the plateau restored (node 0 `Multiply(CN0, CN1)` re-added as CN2 feeding `CustomOutput(1)`, node 1 ref 1 → slot 1); three more elites whose readiness path is intact (node 0 `CustomOutput(1)` fed by a live compute node and node 1 ref 1 reading slot 1): `native-0`, `native-5`, `shuffled-score-5` (run 1's E1 elites), each silenced and unsilenced | food-seeking scenes as run 1's E3 bank (32 scenes, seed `0xE3BA4C`, 64², food 0.04, 200 ticks, start energy 100); bank mean score, food eaten, moves attempted, penalty charged, deaths; wall budget 5 min | Predeclared: margin m = 0.5 bank-mean points (the margin the repaired version will use); gain G = plateau − founder. **Dominant** if silenced founder − founder ≥ 0.7 G and plateau − restored plateau ≥ 0.7 G; **contributor** if not dominant and any silencing contrast (silenced − unsilenced for the founder with or without re-pointing, for each elite, or plateau − restored) exceeds m; else **no shortcut**. Factors: silencing main effect, re-pointing main effect and their interaction from the 2 × 2. Predicted: dominant. Falsified if either 0.7 G condition fails | completed (1 s) | **Dominant**: G = 5.20; silencing alone +4.77 (92 % of G), restoration −5.10 (98 %). See P1 below |
| P1b | 1 | The same shortcut on `wall-v1` | Diagnostic genomes: founder 2 × 2 as P1 (P1's silencing and `native-1` re-pointing); three wall elites from run 1's E2 (`base-wall-s1`) with an intact readiness path, the first two native and the first shuffled-score by index: `native-0`, `native-1`, `shuffled-score-1`, each silenced and unsilenced | barrier-navigation, `wall-v1` scale 2, 32 dev scenes (seed `0xB2DE00`), 200 ticks, start energy 100, barrier scoring; 5 min | **Contributor** if any silencing contrast exceeds m = 0.5, else no shortcut (no wall plateau is defined, so dominance is not read). Predicted: contributor (the founder stands still when ready on `wall-v1` too) | completed (as P2's base-rule wall rows, ≈ 60 s) | Prediction failed: **no shortcut** (max contrast 0.056: founder 1.17 → 1.11 silenced). See P1b below |
| P2 | 1 | Repair: a scene rule under which abandoning reproduction cannot raise the score | Candidate rules, in predeclared order of preference (least semantic change first): R1 start energy 20 (production `initial_energy`), nothing else; R2 (plan design 3) refused-reproduce penalty refunded and reproduce-attempt ticks paused (not counted toward the 200-tick lifetime; world-tick cap 600); R3 (plan designs 1 + 2) reproduction accepted at the production `min_reproduce_energy`, offspring removed at birth, reproduce-attempt ticks paused (cap 600); R4 energy ceiling 20 (start 20, energy capped at 20 after every tick; costs and starvation real); R5 energy held at 20 (start 20, reset to 20 after every tick; no starvation). Validation pairs: P1's seven silencing pairs (food) and P1b's five (wall) | Development banks only: food = P1's bank, wall = P1b's bank; probe evaluation loop in `exploration_probe.rs`; 5 min | Validation: \|silenced − unsilenced\| ≤ m = 0.5 for every pair on both banks. Adopt the first rule in order that passes on both banks and whose built version passes the unchanged calibration gate on both assays; freeze it as version 1 of each repaired assay, then read the sealed acceptance banks once (food seed `0xACCEF00D`, wall `0xACCEBA44`, 32 scenes each, same pairs). Predicted: R1–R3 fail (the founder still stands when ready, or gives energy to offspring); R4 and R5 pass exactly (energy never reaches a tested gate, the lowest being 0.106 × 200 = 21.2); R4 may fail calibration (early starvation) | completed (≈ 60 s) | Food: R1 passes (max \|Δ\| 0.01, but every tested genome dies in 32 of 32 scenes), R2 fails (8.79), R3 fails (6.65), R4 and R5 pass exactly (0). Wall: the base rule already passes (max 0.056), so **P1b: no shortcut on `wall-v1`**; R1, R4 pass (0), R5 passes (0.13), R2 and R3 fail (1.41, 2.22). See P2 below |
| P2b | 1 | R1's pass may hold only because creatures die before reaching the readiness gate; a competent forager could still gain by abandoning reproduction under R1 | Added before any version is frozen, stricter than P2 (development use only): the built-in food comparator (`area_food` of the founder, lab-authored, it keeps the founder's readiness path) silenced vs unsilenced under R1, R4 and R5 on P2's food bank | As P2; 2 min | R1 fails development validation if the comparator pair exceeds m = 0.5; then the adoption order moves to R4, then R5, each also needing this pair within m. Predicted: R1 fails (the comparator eats enough to pass the gate), R4 and R5 pass exactly | completed (65 s) | Prediction held: comparator silencing gain +16.41 under the base rule, +7.71 under R1 (fails), 0 under R4 and R5. Adoption moves to R4 |
| P2c | 1 | P2b's stricter standard applied to `wall-v1`: the barrier comparator (it keeps the founder's readiness path) may gain by abandoning reproduction even though P1b's genomes do not | The built-in barrier comparator silenced vs unsilenced under the base rule, R4 and R5 on P1b's wall bank | As P1b; 2 min | If the base-rule pair exceeds m = 0.5, `wall-v1` is treated as shortcut-affected and needs a repaired version (R4 first, then R5, each needing every P1b pair and this pair within m, then calibration); otherwise `wall-v1` is kept unchanged. Predicted: exceeds m (a competent navigator also stands still when ready) | completed (63 s) | Prediction failed: base-rule delta +0.008 (10.386 → 10.394), R4 and R5 0. **`wall-v1` kept unchanged** |
| V1 | 1 | Food-seeking repaired version 1 = R4 | `food-seeking`, `sparse-food-v1`, 64², food 0.04, lifetime 200, `--start-energy 20 --energy-ceiling 20` (instr `0328335e`), built-in `area_food` comparator, calibration margin 1.0, grid rule unchanged; development set = P2's food bank plus the gate's calibration scenes | Before freezing: the instr check (both stored combinations' three hashes equal the launch baseline), the harness's R4 equals the probe's R4 on the development bank, and the calibration gate passes on seeds 1 and 2. Then frozen and the sealed acceptance bank (`0xACCEF00D`, 32 scenes) read once with every P2 food pair and the P2b comparator pair | Accepted if every pair is within m = 0.5 on the acceptance bank; otherwise `uncalibrated` for good and version 2 = R5 with a fresh sealed bank | frozen (descriptor [`v1-version.json`](evolvability-exploration-2026-10-run2/v1-version.json), sha256 `81e155d3ea8f7553…`); acceptance read once (65 s) | **Accepted**: every pair, the comparator included, Δ = 0.0 on the sealed bank (founder 8.80, comparator 40.81). Pre-freeze checks: instr hash check held (all six hashes equal the launch baseline); harness R4 = probe R4 (max diff 0.0); calibrated on seed 1 (threshold 22.34; founder 8.71, floor 1.33, comparator 43.35) and seed 2 (19.93; 6.52, 1.33, 38.53) |
| P2d | 1 | After review 1: an admissible repair must meet the requirement on reproduction-active genomes, not only on the inactive ones P2 used | Reproduction-active diagnostic genomes (plan rule 7), each against its silenced copy: the founder; the founder with its energy gate threshold lowered to 0.08 and to 0.05 (active below the ceiling of R4); the comparator; the comparator at 0.05; `shuffled-score-5`; the restored plateau. Rules: the base rule, R2, R3, R4, R5 and two lifetime-preserving versions of the plan's designs: R6 (designs 2 + 3) refused-reproduce penalty refunded, the 200-tick lifetime kept, score scaled by lifetime over the ticks without a reproduce attempt; R7 (designs 1 + 2) reproduction accepted at the production `min_reproduce_energy`, offspring removed at birth, scored as R6 | P2's food bank, start energy as each rule; 3 min | A rule meets the requirement on active genomes if every pair is within m = 0.5. Admissible under review 1: R6 and R7 (they change only reproduction acceptance or charging and the scorer; start state and lifetime unchanged). Predicted: every rule fails on at least one active pair, R4 on the gate-lowered founders; if R6 and R7 both fail, no admissible food repair exists within the plan and the run records outcome C | completed (≈ 70 s) | **Every rule fails on active genomes**: base 26.32, R2 39.53, R3 15.77, R4 11.47 (comparator with its gate at 0.05: 27.94 → 39.41 silenced), R5 40.82, **R6 4.38** with the sign reversed (active founder 15.16 against 13.87 silenced; gate at 0.05: 18.25), **R7 10.68** (founder 7.38 → 13.87). See P2d below |
| P2e | 1 | Advice 1's caveats on P2d: R6 refunds after a fatal penalty has removed the creature, rescales terminal progress, and R7's loss is not broken down; the plateau pair mixed restoration with silencing | The lifetime-preserving rules on the same active genomes: R6 as P2d; **R6b** every failed-action penalty set to zero in the config (exact zero charging for genomes whose only failed actions are reproduce attempts) with R6's scaling; **R6c** R6b with only food scaled per opportunity; R7 with births and refusals counted; reproduce-attempt death ticks excluded from opportunities; the plateau pair is now the restored plateau against its silenced copy | As P2d; 3 min | If R6b or R6c keeps every active pair within m = 0.5, it becomes the candidate for food version 2 (new assay name, frozen before calibration, fresh sealed bank); otherwise the food repair is blocked (outcome C, worded as advice 1 proposes). Predicted: both fail | completed (≈ 60 s) | **Both fail**: R6b 4.38 (identical to R6: no attempt was fatal on this bank), R6c 2.79 in both directions (active founder at 0.05 +2.79; `shuffled-score-5` silenced +1.51), R7 10.68 (the founder's 56 attempts were all accepted births, 0 refused). **Food repair blocked → outcome C** |
| P3 | 1 | Does run 1's plateau persist on the repaired food version (plan phase 1, last step)? | V1 baseline campaign (built-in arms; it is also V1's baseline for phase 3, with the instr commit and no proto); then run 1's E3 (founder children), E4 (founder plus the campaign's 8 native and 8 shuffled-score final elites) and E7 (two-step from the campaign's native elite with the highest E3-bank mean, ties to the lowest index) under V1 | `food-seeking`, V1, seed 1, `--quick --replicates 8 --food-fraction 0.04 --lifetime 200`; probes on run 1's 32-scene bank; 15 min each | Predicted: native elites no longer converge on one phenotype, and E4's native elites keep improving children (at least one improver in ≥ 3 of 8 parents). Falsified (plateau persists) if 0 improvers in ≥ 6 of 8 native parents, as in run 1 | completed (campaign 74 s; probes 5–31 s) | **Falsified, the sampled plateau persists** (on V1, later withdrawn): E4 0 improvers in 7 of 8 native parents and in the founder's 28 changed children; native reach 0/8, ladder `stalls at benefit` (fail 8 of 8, adequacy met); E3 4 of 161 changed children improve the bank; E7 grandchildren 11 of 512 improve against direct children 7 of 512. See P3 below. V1 baseline hashes (seed 1, quick 8): `aa5a5f2f…`, `8aaeb981…`, `5e987139…` |
| P4 | 2 | Path census (H3, H4, H8): what separates the founder from a sensor-guided gain on the repaired instruments | Observation probe: 8 lineages per assay replaying the lab's reference arm rule (population 16, 4 survivors, 12 children per generation from uniformly drawn survivors through the production engine with the parent's frozen last-scene record, 4 fresh training scenes per generation, 40 generations); every **selected improvement** (a child above its parent on the generation's scenes and among the next survivors) is recorded with its applied operators, target nodes (founder 0–1 or added), its structural diff (input refs added or changed and their family; edges added or re-sourced and their source: a sensor or introspection read, a node-boundary read through an upstream slot or shared memory (H8), or an internal compute node; vote and output sinks touched; nodes added), its delta on the 32-scene development bank, and a decision-change reading (H4: share of bank scenes in which the child's trajectory departs from the parent's, and the first departure tick). Descriptive only; never selection or discovery evidence | food V1 (P2's food bank) and `wall-v1` (P1b's bank), seeds fixed in the probe; 15 min each | Predeclared M5 choice: **H8 cross-node source** if node-boundary reads make up at least half of the input-changing edits among selected improvements on either assay, or the witness path (P5) needs a boundary crossing; **VM fresh reads** if instead at least half of selected improvements change a VM node; if neither holds, H8 (the founder's two Graph nodes talk only through the baton). Predicted: few selected improvements touch added tissue; most re-weight or re-point the founder's node 1 | not run | Phase 1 blocked (P2e): no repaired food instrument exists for the census |
| P5 | 2 | Witness path (H3): how many native-operator edits separate the founder from a sensor-guided mover that reaches the threshold | Diagnostic genome sequence (plan rule 7): the founder edited step by step into the built-in comparator's structure (food: `area_food`; wall: `barrier_comparator`), one native-operator-applicable edit per step (add an input reference, add a compute node, add or re-weight an edge); every intermediate scored on the development bank and labelled helpful, neutral or harmful against the previous step (\|Δ\| ≤ 0.05 neutral). Never discovery evidence; its length is an upper bound on the shortest path | As P4; 2 min | Predicted: the food path is about 10 edits, mostly neutral until the last edge that connects the gated vector to a move vote; the wall path adds one ring read and four inhibitory edges, which may help one at a time | not run | Phase 1 blocked (P2e) |

## Frontier table

Descriptive only: paired difference from the reference arm, by seed, in
reached fraction and stall rung; never ranks or retires an arm. No phase 3
arm ran: the plan forbids phase 2 and 3 readings on a shortcut-affected food
instrument until a validated version exists, and none does. The cells say
`not evaluated` (prerequisite failed), which is neither a negative nor an
inapplicability finding.

| Arm | food-seeking (repaired) | barrier-navigation (`wall-v1`, kept) |
| --- | --- | --- |
| native (reference) | no validated instrument (V1 withdrawn) | not run in phase 3 (run 1 E2 at quick sizes: reached 0/8, `inconclusive at retention`) |
| M1 `neutral_input_recruitment: SingleChannel` | not evaluated (prerequisite failed) | not evaluated (prerequisite failed) |
| M2 `neutral_input_recruitment: WholeFamily` | not evaluated (prerequisite failed) | not evaluated (prerequisite failed) |
| M3 factorial with `structured_heritable_refinement` | not evaluated (prerequisite failed) | not evaluated (prerequisite failed) |
| M4 `per_unit_rate: 0.02` | not evaluated (prerequisite failed) | not evaluated (prerequisite failed) |
| M5 prototype (chosen by phase 2) | not built (phase 2 not run) | not built (phase 2 not run) |

## Findings by phase

### Phase 1: the sterility shortcut

Summaries: [`p1-sterility.ndjson`](evolvability-exploration-2026-10-run2/p1-sterility.ndjson),
[`p2-repair-candidates.ndjson`](evolvability-exploration-2026-10-run2/p2-repair-candidates.ndjson),
[`p2b-comparator.ndjson`](evolvability-exploration-2026-10-run2/p2b-comparator.ndjson).

- **Mechanism (from source).** The lab's founder is `V3Alpha1` (the
  production default). Once older than 20 ticks with energy above
  0.16 × 200 = 32, its decision node votes only `Reproduce` (Move and Eat
  carry −2 per unit of the readiness signal), so the tick commits one
  `Reproduce` and nothing else. The lab refuses every attempt
  (`min_reproduce_energy` above `max_energy`) and charges the failed-action
  penalty, 1.0 rising to 10 with the age cost after age 100. In the world
  the same attempt succeeds and drops the parent's energy below the gate
  after one tick; in the lab the creature stands still, paying, until basal
  decay and penalties take it back under 32.
- **P1, food (dominant).** On run 1's 32-scene bank: founder 9.10, plateau
  `native-1` 14.30 (G = 5.20). Readiness silenced (node 0 `CustomOutput(1)`
  cleared) 13.87 (+4.77, 92 % of G); the plateau with readiness restored
  9.20 (−5.10, 98 %). The plateau's node 1 re-points input ref 1 (the
  readiness read) to an unwritten slot, which is itself a silencing: ref 1
  alone gives 13.87, identical to silencing. Ref 2 (north food ring → an
  unwritten slot) gives +0.10 alone and +0.43 under silencing; the other
  plateau edits contribute 0. 2 × 2: silencing main effect +2.39,
  re-pointing +2.81, interaction −4.77; the factors are not independent
  (re-pointing ref 1 is itself a silencing), and the restoration adds one
  compute node (a small compute cost). P1 does not separate the refused
  penalty, the lost foraging ticks and survival.
  Penalty charged: founder 1,432, every silenced or re-pointed genome 0.
  `native-0` and `native-5` gain nothing from silencing because neither
  attempts reproduction any more (penalty 0, their readiness path silenced
  elsewhere); `shuffled-score-5` gains +4.69. In run 1's E1 elites, node 1's
  ref 1 points at an unwritten slot in 5 of 8 native elites (a sixth,
  `native-2`, has a dead node 0 readiness output) and 6 of 8 shuffled-score
  elites (structural tally, not a behavioural reading).
- **P1b, wall-v1 (no shortcut by the predeclared criterion).** Founder 1.17,
  silenced 1.11 (−0.06); `native-0`, `native-1`, `shuffled-score-1` within
  0.06. On `wall-v1` the re-pointing carries a gain (+0.94, founder 1.17 →
  2.11) through the lost north food-ring read, not through silencing.
- **P2, repair candidates (development banks).** Food: R1 (start 20) passes
  at 0.01 but every tested genome dies in 32 of 32 scenes; R2 (penalty
  refund, paused clock) fails at 8.79 because the paused founder still ages
  and drains while it waits; R3 (accepted reproduction, offspring removed)
  fails at 6.65 because energy given to offspring is lost to survival; R4
  (ceiling 20) and R5 (held at 20) pass at exactly 0. The tested
  implementations of the plan's designs (R2, R3: paused clocks that extend
  the world horizon) fail on these pairs.
- **P2b.** The built-in food comparator keeps the founder's readiness path:
  silencing it raises its bank score by +16.41 under the T22.F01 rule (the
  calibration comparator is itself handicapped by refused reproduction) and
  by +7.71 under R1, so R1 fails; R4 and R5 give 0.
- **P2c and the wall decision.** The barrier comparator gains +0.008 from
  silencing on `wall-v1` (10.386 → 10.394): the barrier score saturates at
  the food block and penalises blocked moves, so standing still when ready
  costs almost nothing there. `wall-v1` is kept unchanged: no contributing
  shortcut was detected in the tested pairs (P1b's criterion and P2b's
  stricter one); that is not immunity, since the saturating block score may
  mask lost time and survival.
- **V1 (food-seeking, R4) frozen and accepted, then withdrawn after review
  1** (inadmissible: it changes the start state and writes live energy;
  its validation genomes were all reproduction-inactive under it; it was
  calibrated before the freeze and kept the existing names). The record
  below stands as history, not evidence. `--start-energy 20
  --energy-ceiling 20` on `sparse-food-v1` (instr `0328335e`; descriptor
  [`v1-version.json`](evolvability-exploration-2026-10-run2/v1-version.json)).
  The instr check held (all six launch hashes unchanged), the harness equals
  the probe exactly, the gate calibrates on seeds 1 and 2 (thresholds 22.34
  and 19.93; founder 8.71 and 6.52; comparator 43.35 and 38.53; floor 1.33),
  and the sealed acceptance bank gives Δ = 0.0 on every pair, the comparator
  included ([`v1-acceptance.ndjson`](evolvability-exploration-2026-10-run2/v1-acceptance.ndjson)).
  What R4 does: energy never exceeds 20, below every readiness gate of the
  tested inactive genomes (the lowest 0.106 × 200 = 21.2), so for those
  genomes the reproduce drive never fires and abandoning it changes nothing;
  costs, ageing and starvation stay real (every tested genome still dies in
  most scenes). What it does not do: a descendant whose gate fell below 0.1
  of `max_energy` is reproduction-active under R4, and silencing it then
  raises its score (P2d: +1.18 for the founder with its gate at 0.05,
  +11.47 for the comparator), so R4 does reward abandoning reproduction for
  such genomes. It also removes energy reserves above 20 as a resource. The
  descriptor names the implementation commit `0328335e` as `frozen_at_commit`;
  the descriptor itself was frozen at `dba17051`.
- **P3, the plateau on V1** (a reading on the withdrawn V1, descriptive
  only; universal early starvation at energy ≤ 20 is an alternative
  explanation for its flat neighbourhoods) ([`p3-v1-food-seed1.json`](evolvability-exploration-2026-10-run2/p3-v1-food-seed1.json),
  [`p3-e3-selection-audit.ndjson`](evolvability-exploration-2026-10-run2/p3-e3-selection-audit.ndjson),
  [`p3-e4-elite-neighbourhoods.ndjson`](evolvability-exploration-2026-10-run2/p3-e4-elite-neighbourhoods.ndjson),
  [`p3-e7-two-step.ndjson`](evolvability-exploration-2026-10-run2/p3-e7-two-step.ndjson)).
  With the founder's own readiness gate never reached under R4 (descendants
  that lower their gates are not covered; the shortcut is not removed in
  general, see P2d), native selection finds almost nothing: final
  bests 1.25–8.88 against founder-only 1.25–8.88 (equal in 5 of 8
  replicates, within 0.25 in all 8), reach 0/8 against a threshold of
  22.34, and the ladder
  `stalls at benefit` in 8 of 8 replicates with adequacy met (0–3 improved of
  71–106 viable touching children). The founder's sampled neighbourhood is nearly
  flat: E3 finds 4 bank improvers among 161 genome-changed children (118
  zero-delta, 39 worse), E4 none among the founder's 28. Six of the eight
  native elites converge on a bank mean of 6.05 against the founder's 5.94.
  From the best native elite (`native-5`, 7.38), E7's two-step neighbourhood
  gives 11 improvers of 512 grandchildren (3 above one point) against 7 of
  512 direct children (1 above one point; the grandchildren share 32
  intermediate parents): the H3 criterion (≥ 5 with ≤ 1 direct) is not met.
  The comparator, which steers by `AreaFoodSummary`, scores about 40 on the
  same scenes, so a large sensor-guided gain exists that these samples of
  one- and two-event neighbourhoods did not reach.
- **P2d, reproduction-active genomes** ([`p2d-active-genomes.ndjson`](evolvability-exploration-2026-10-run2/p2d-active-genomes.ndjson)).
  Lowering the founder's energy gate (one `Threshold` parameter change, a
  native operator) makes reproduction active under any energy rule. On
  those genomes every rule fails the requirement. The two admissible ones
  fail differently: under R6 (penalty refunded, score per foraging
  opportunity) an active genome scores above its silenced copy (founder
  +1.29, gate at 0.05 +4.38) while `shuffled-score-5` still gains 0.94 from
  silencing, so R6 fails in both directions; R7 (births accepted, offspring
  discarded) still lets silencing pay (+6.49 for the founder). R4 fails as
  review 1 predicted: with the gate at 0.05 the comparator scores 27.94 and
  its silenced copy 39.41.
- **P2e, the caveats of advice 1** ([`p2e-exact-zero-charge.ndjson`](evolvability-exploration-2026-10-run2/p2e-exact-zero-charge.ndjson)).
  Exact zero charging (R6b: every failed-action penalty zero in the config)
  gives results identical to R6 (max 4.38), so the refund ordering did not
  matter on this bank. Scaling only food per opportunity (R6c) narrows the
  failure to 2.79 but keeps it in both directions: active founder at 0.05
  16.66 against 13.87 silenced; `shuffled-score-5` 11.07 against 12.58. The
  founder's R7 loss (7.38 against 13.87) comes with 56 accepted births and 0
  refusals, which rules out refused-attempt penalties for that pair; it does
  not separate the parental transfer from the reproduction charge, the
  actions spent on reproducing and survival. The gate-lowered genomes add
  refused attempts (230 and 326) because their low-energy births fail the
  litter floor.
- **What phase 1 shows.** A reproduction-active genome's own decision spends
  ticks (refused attempts) or energy (accepted births) on reproduction. In a
  solo scene with a fixed lifetime and unchanged start state, the tested
  scorers either charge that spending (silencing pays: base rule, R7) or
  rescale for it (R6, R6b, R6c; attempting reproduction pays for some
  genomes, silencing for others), and none keeps every active pair within
  0.5. The designs that remove the incentive for the tested inactive genomes
  (R4, R5) change the start state and write live energy, and still fail on
  lowered gates. This is an instrumentation blocker for this run, not
  evidence against any mechanism, and not proof that no repair can exist
  (action-local or time-stratified scorers were not tried; advice 1).

## Review dispositions

Review 1 (end of phase 1, Codex `gpt-6.1-sol` `high`, read-only; brief and
output under `.bench-artifacts/lab/exploration/codex/run2-review-1*`), verdict
`not-ready`:

| # | Finding | Severity | Disposition |
| --- | --- | --- | --- |
| 1 | R4 does not establish the requirement: it keeps every validation genome below its readiness gate, so the pairs are vacuous; a lowered gate (one threshold mutation) makes reproduction active under R4, and silencing it would then raise the score | blocking | Accepted. V1's validation is vacuous on reproduction-active genomes; P2d tests every rule on reproduction-active diagnostic genomes |
| 2 | R4 changes the evaluation start state (start energy 100 → 20) and writes live energy every tick; the run 2 amendment authorises reproduction acceptance and charging changes with start and lifetime preserved, not this | blocking | Accepted. R4 and R5 are inadmissible; so are R2 and R3 as built (they extend the world horizon to 600 ticks). Admissible candidates are reproduction acceptance or charging changes plus a per-opportunity scorer within the fixed lifetime (R6, R7 in P2d) |
| 3 | V1's procedure: calibration ran before the freeze, new behaviour used the existing assay names, the descriptor named the implementation commit as the freeze; the five candidates need reconciling with "any change is a new version" and the two-version cap | blocking | Accepted. **V1 is withdrawn** (not admissible, and its procedure broke the freeze-before-calibration and new-name rules); its acceptance read is recorded but is not evidence. Version accounting (corrected after review 2): under the rule that any change to any part is a new version, the candidates R1–R7 evaluated on development scenes were versions too, which exceeds the two-version cap. This is a recorded procedural deviation; the run claims no remaining version allowance, and any further food-seeking instrument needs new authorization (a run 3 plan). The descriptor's freeze was `dba17051`, not `0328335e` |
| 4 | P1's dominant verdict follows, but re-pointing is itself a silencing (not an independent factor), restoration adds a compute node (compute cost), and penalty, lost ticks and survival are not separated; two elites were already inactive | advisory | Accepted; P1's wording narrowed below |
| 5 | Validation passed fixed pairs, not the general requirement; pairs duplicate; bank means do not show per-scene equality | advisory | Accepted; P2d adds active genomes, and P2d's verdict is read per pair |
| 6 | Keeping `wall-v1` follows the declared rule, but means no shortcut detected in these pairs, not immunity (the score saturates at the block) | advisory | Accepted; wording narrowed |
| 7 | P3 overstates: E3 found 4 improving founder children, so "one-step local optimum" is wrong; E7's 512 grandchildren share 32 parents; universal starvation under R4 is an alternative explanation | advisory | Accepted. P3 is a reading on the withdrawn V1 and carries no weight beyond it; the wording is corrected below |
| 8 | R2/R3/R5 breach the lifetime or no-subsidy rules if adopted; their failures do not cover every implementation of the plan's scorers; no `make check` recorded for the instr commit | advisory | Accepted; P2d tests lifetime-preserving implementations (R6, R7); `make check` on the branch is run and recorded before any instr commit's results are used again |

Advice 1 (not a review; Codex `gpt-6.1-sol` `high`, read-only,
`.bench-artifacts/lab/exploration/codex/run2-advice-1*`), `advice: C now`:

| # | Point | Disposition |
| --- | --- | --- |
| a | No credible untried admissible design is expected to keep every active pair within 0.5 and stay a useful calibrated assay; charging versus rescaling is not exhaustive (action-local, nonlinear or time-stratified scorers remain), so do not spend the second version on a speculative scorer | Accepted: no further version is frozen (and, per review 2, none is claimed to remain); the impossibility claim is not made |
| b | C is the correct outcome as an instrumentation blocker; the plan does not require spending both versions or running wall-only phases first; M1–M5 are "not evaluated because the prerequisite failed" | Accepted: wording adopted in the outcome and the frontier table |
| c | R6 refunds after a fatal penalty may have removed the creature; reproduce-attempt death ticks were counted as opportunities | Accepted: P2e's R6b zeroes every failed-action penalty in the config and excludes those death ticks; results identical to R6 |
| d | R6 rescales terminal progress with food | Accepted: P2e's R6c scales food only; still fails (2.79) |
| e | R6's sign is not uniformly reversed (`shuffled-score-5` +0.94 from silencing) | Accepted: P2d wording corrected |
| f | R7's loss is not broken into births and refusals | Accepted: P2e counts them (founder 56 births, 0 refusals) |
| g | The restored-plateau pair mixed restoration with silencing | Accepted: P2e pairs the restored plateau with its silenced copy |
| h | `reproduce_ticks` reads 0 for rules that never count it | Accepted: P2e reports attempt ticks only for the lifetime-preserving rules |

Review 2 (closing review of the C note, Codex `gpt-6.1-sol` `high`,
read-only, `.bench-artifacts/lab/exploration/codex/run2-review-2*`), verdict
`not-ready` (C supported; fixes to wording, accounting and preservation):

| # | Finding | Severity | Disposition |
| --- | --- | --- | --- |
| 1 | The R4 contradiction remains ("V1 cannot reward abandoning reproduction", P3's "with the shortcut removed"), and R4 removes the shortcut only for the tested inactive genomes; allowing the energy intervention alone does not unblock the plan | blocking | Accepted: V1 and P3 wording corrected; the R4 option now says the requirement would also have to change or the confound be accepted |
| 2 | The version-cap objection is unresolved; annotate the descriptor's freeze commit | blocking | Accepted: the candidates count as versions under the rule, a recorded deviation; no remaining allowance is claimed; the freeze commit `dba17051` is recorded |
| 3 | "The plan's three repair designs fail" is too strong | blocking | Accepted: "the tested repair implementations fail", with the untested designs named |
| 4 | Evidence preservation is claimed but not done | blocking | Accepted: raw outputs, Codex briefs and outputs and the `make check` logs are copied to the main checkout's `.bench-artifacts/lab/exploration/run2/` and checked against the summaries' recorded raw hashes before the worktree is removed (Landing) |
| 5 | P3's improved range is 0–3, not 1–3 | advisory | Accepted; corrected |
| 6 | The causal attribution is too narrow (the controller chooses reproduction; suppression prevents the birth; R7 does not isolate parental transfer) | advisory | Accepted; the T22 finding and outcome item 1 reworded |
| 7 | "Every food-seeking reach threshold computed so far" is too broad | advisory | Accepted; limited to the unmodified instrument |
| 8 | The lineage-scene option is unvalidated and changes the scoring and counter boundary; the action-local scorer is an untested possibility and need not change a scoring rule; say "no production mechanism candidate" | advisory | Accepted; options and candidates reworded |
| 9 | Closing metadata: the final commit hash, the missing review 2 record | advisory | Accepted |

Review 3 (confirmation of the review 2 fixes, Codex `gpt-6.1-sol` `high`,
read-only, `.bench-artifacts/lab/exploration/codex/run2-review-3*`), verdict
`not-ready` on two open items; it confirmed review 2's items 2–5 and 7–9,
verified the preserved raw hashes independently, and found the Landing
method follows run 1's close order:

| # | Finding | Disposition |
| --- | --- | --- |
| 1 | P3's "the founder lineage's readiness kept below its gate" still generalises beyond the fixed genomes | Accepted: reworded to the founder's own gate under R4, descendants not covered |
| 2 | P2e still says the founder's R7 loss "is the energy handed to discarded offspring, not penalties" | Accepted: reworded; zero refusals rules out refused penalties only |
| 3 | The closing-record commit is still "(next)" | Accepted: `1c28feba` recorded |

Review 4 (confirmation of these three fixes, Codex `gpt-6.1-sol` `high`, read-only, `.bench-artifacts/lab/exploration/codex/run2-review-4*`): all three confirmed, no new inconsistency, verdict `ready`.

## Evidence outcome

**C. Blocker.** Phase 1 is blocked: the food-seeking instrument the run adopted
(V1, rule R4) is withdrawn, and neither tested lifetime-preserving repair (R6,
R6b, R6c; R7) satisfies the requirement on reproduction-active genomes
(P2d, P2e). No admissible, validated food instrument is available for the
required two-assay mechanism contrasts, so outcomes A and B, which both need
the two repaired assays, are unavailable with the current instruments. This
is an instrumentation blocker: it is not evidence against M1–M5 and not proof
that every possible repair fails.

What the run did establish:

1. **The sterility shortcut is real and dominant on food-seeking** (P1): run 1's
   plateau gain is 92 % readiness silencing. The founder's own controller
   chooses `Reproduce` whenever it is ready; the T22.F01 lab's suppression
   prevents the birth that would drop its energy below the gate, and the
   penalty adds cost, so a one-tick event in the world becomes a long,
   charged stall in the lab. The calibration comparator of the unmodified
   food-seeking instrument is handicapped by the same mechanism (+16.41 when
   silenced, P2b), so the reach thresholds computed on that instrument
   (run 1 and the T22 readings) include it.
2. **`wall-v1` shows no detected shortcut in the tested pairs** (P1b, P2c):
   kept unchanged.
3. **The tested repair implementations fail** on reproduction-active genomes
   (P2, P2d, P2e): paused clocks (R2, R3), per-opportunity rescaling with
   refunded, zeroed or ordinary-acceptance charging (R6, R6b, R6c, R7), and
   the energy rules (R1, R4, R5). The plan's per-opportunity design with
   ordinary charging alone, and action-local or time-stratified scorers,
   were not evaluated.

### What would unblock a run 3 (user decisions)

The first three options need a decision this run could not make on its own;
the fourth stays within the scoring rules but needs a new instrument version,
which this run cannot claim (see review 1, item 3):

- **Accept a hunger-regime food instrument as a T22 amendment, and accept or
  change the requirement it still fails**: start energy at the production
  seeding energy (20) with an energy ceiling of 20 (R4, instr `0328335e` on
  the branch). On the tested inactive genomes it gives Δ = 0.0 on
  development and sealed scenes and calibrates (thresholds 22.34 and 19.93),
  but it changes the evaluation start state, writes live energy, and still
  rewards abandoning reproduction for a descendant whose gate drops below
  the ceiling (P2d: +11.47 for the comparator at 0.05, +1.18 for the
  founder). Allowing the energy intervention alone does not unblock the
  plan; the behavioural requirement would also have to change, or this
  confound be accepted and measured. It needs a new assay name, a freeze
  before calibration and a fresh sealed bank.
- **Value births in a lineage scene**: a multi-creature scene kind in which
  accepted offspring forage and the score credits the lineage, so
  reproduction becomes part of the scored behaviour. Unvalidated as a
  repair. It extends run 1's multi-creature candidate (which proposed
  focal-only attribution) by changing the scoring and counter boundary to
  the lineage, and needs its own calibration and active-genome validation.
- **Relax the two-assay requirement**: run phases 2–3 on `wall-v1` alone,
  with claims limited to one assay (a plan amendment). `wall-v1`'s result
  is no shortcut detected in the tested pairs, not immunity.
- **Try an action-local or time-stratified scorer** (advice 1): an untested
  possibility, not an expected repair; predeclared with its own
  calibration and active-genome validation, under a new authorization.

### Hypotheses at close

| ID | Status after run 2 |
| --- | --- |
| H1 instrument limit | Confirmed for food-seeking: the sterility shortcut is dominant (P1); the calibration comparator carries it too (P2b). `wall-v1`: no shortcut detected |
| H2 exposure | Unchanged from run 1 |
| H3 stepping stones | Not tested on an admissible instrument (P4, P5 not run). On the withdrawn V1 only: E7's two-step criterion unmet (11 of 512 against 7 of 512) |
| H4–H8, M1–M5 | Not evaluated: the food-instrument prerequisite failed |

### Candidates

- **T22 feature candidate (instrument), not admissible under the run 2 rules:**
  the energy ceiling (`--energy-ceiling`, instr `0328335e`) as a hunger-regime
  food-seeking assay, with the caveats above (it fails on lowered gates).
- **T22 finding:** in the T22.F01 lab the founder's controller votes
  `Reproduce` whenever it is ready, reproduction suppression prevents the
  birth that would end that state, and the retained penalty adds cost, so
  the founder stands still while ready; reach thresholds and readings on
  the unmodified food-seeking instrument (whose comparator shares the
  founder's readiness path) mix sterility with foraging. P1 does not
  separate the penalty, the lost foraging ticks and survival; P2e does not
  separate parental transfer from the reproduction charge, lost actions and
  survival.
- **T22 feature candidate (instrument):** a multi-creature or lineage scene
  kind (run 1's multi-creature candidate, plus lineage scoring as described
  above; unvalidated).
- No `proto:` commit was made, so no production mechanism (roadmap feature)
  candidate.

## Execution status

`complete`: outcome C reached and reviewed (reviews 1–4, advice 1; review 4 `ready`); the close
steps are recorded under Landing. Holdout banks 1–3 (seeds 101/102,
111/112, 121/122) were never read. No further instrument version is claimed
(review 1, item 3). Used about 180 of 400 turns.

## Branch and commits

Branch `worktree-evolvability-exploration-2` (kept, never merges), from launch
`eefca5db`:

| Commit | Kind | What |
| --- | --- | --- |
| `b0aa9733` | lab | P1 probe; ledger B0, P1 |
| `086e767a` | lab | P1 result; P1b, P2 probe |
| `1ef87f26` | lab | P2 result; P2b probe |
| `0328335e` | instr | optional energy ceiling (`--energy-ceiling`), default off; stays on the branch |
| `4ac9248f` | lab | P2c, V1 and P3 probes (needs `0328335e`) |
| `dba17051` | docs | P2c result; V1 frozen |
| `2b59fb8a` | docs | phase 1 results; P4, P5 predeclared |
| `4d2f978e` | lab | review 1 dispositions, V1 withdrawn; P2d probe (needs `0328335e`) |
| `ec9c81a4` | lab | P2d result; P2e probe (needs `0328335e`) |
| `2a5c98fb` | lab | P2e result, outcome C; the probe made to build without `0328335e` (the V1-only probes stay at `ec9c81a4`) |
| `1c28feba` | docs | review 2 dispositions and closing record |
| `faf71668` | docs | review 3 dispositions |
| (closing) | docs | review 4 verdict recorded |

## Landing

In run 1's close order:

1. Final note and summaries committed on the branch. Kept evidence copied to
   the main checkout: `.bench-artifacts/lab/exploration/run2/` (raw probe
   outputs, the lab runs B0, I1, V1 and the R4 calibrations, the helper
   scripts, both `make check` logs; 34 MB) and
   `.bench-artifacts/lab/exploration/codex/run2-*` (review and advice
   briefs, logs and outputs). Checked before the worktree was removed: all
   10 summaries' recorded raw sha256 values match the preserved copies.
2. `ExitWorktree` with keep; `main` rechecked against the launch revision.
3. Landed on `main`: the `lab:` commits that build without `instr:` changes
   (`b0aa9733`, `086e767a`, `1ef87f26`) by cherry-pick; the later `docs:`
   and `lab:` content (this note, its summaries and the final probe, which
   builds without `0328335e`) as one landing commit, because the
   intermediate `lab:` commits `4ac9248f`, `4d2f978e` and `ec9c81a4` need the
   `instr:` commit and cannot land as they are. `0328335e` stays on the
   branch (listed above as a T22 feature candidate).
4. `make check` on `main` after the landing, then the worktree removed and
   the branch kept.

Verification on the branch: `make check` passed after the final probe change
(log `.bench-artifacts/lab/exploration/run2/make-check-2.log`; an earlier run
failed only because it compiled a half-edited probe file). The instr check
after `0328335e` held: both stored combinations' three hashes equal the
launch baseline.
