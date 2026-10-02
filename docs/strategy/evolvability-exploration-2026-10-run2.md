# Evolvability exploration, run 2: results (2026-10)

Status: in progress. Contract: the [run 2 plan](evolvability-exploration-plan-2026-10-02.md),
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
| P1b | 1 | The same shortcut on `wall-v1` | Diagnostic genomes: founder 2 × 2 as P1 (P1's silencing and `native-1` re-pointing); three wall elites from run 1's E2 (`base-wall-s1`) with an intact readiness path, the first two native and the first shuffled-score by index: `native-0`, `native-1`, `shuffled-score-1`, each silenced and unsilenced | barrier-navigation, `wall-v1` scale 2, 32 dev scenes (seed `0xB2DE00`), 200 ticks, start energy 100, barrier scoring; 5 min | **Contributor** if any silencing contrast exceeds m = 0.5, else no shortcut (no wall plateau is defined, so dominance is not read). Predicted: contributor (the founder stands still when ready on `wall-v1` too) | predeclared | |
| P2 | 1 | Repair: a scene rule under which abandoning reproduction cannot raise the score | Candidate rules, in predeclared order of preference (least semantic change first): R1 start energy 20 (production `initial_energy`), nothing else; R2 (plan design 3) refused-reproduce penalty refunded and reproduce-attempt ticks paused (not counted toward the 200-tick lifetime; world-tick cap 600); R3 (plan designs 1 + 2) reproduction accepted at the production `min_reproduce_energy`, offspring removed at birth, reproduce-attempt ticks paused (cap 600); R4 energy ceiling 20 (start 20, energy capped at 20 after every tick; costs and starvation real); R5 energy held at 20 (start 20, reset to 20 after every tick; no starvation). Validation pairs: P1's seven silencing pairs (food) and P1b's five (wall) | Development banks only: food = P1's bank, wall = P1b's bank; probe evaluation loop in `exploration_probe.rs`; 5 min | Validation: \|silenced − unsilenced\| ≤ m = 0.5 for every pair on both banks. Adopt the first rule in order that passes on both banks and whose built version passes the unchanged calibration gate on both assays; freeze it as version 1 of each repaired assay, then read the sealed acceptance banks once (food seed `0xACCEF00D`, wall `0xACCEBA44`, 32 scenes each, same pairs). Predicted: R1–R3 fail (the founder still stands when ready, or gives energy to offspring); R4 and R5 pass exactly (energy never reaches a tested gate, the lowest being 0.106 × 200 = 21.2); R4 may fail calibration (early starvation) | completed (≈ 60 s) | Food: R1 passes (max \|Δ\| 0.01, but every tested genome dies in 32 of 32 scenes), R2 fails (8.79), R3 fails (6.65), R4 and R5 pass exactly (0). Wall: the base rule already passes (max 0.056), so **P1b: no shortcut on `wall-v1`**; R1, R4 pass (0), R5 passes (0.13), R2 and R3 fail (1.41, 2.22). See P2 below |
| P2b | 1 | R1's pass may hold only because creatures die before reaching the readiness gate; a competent forager could still gain by abandoning reproduction under R1 | Added before any version is frozen, stricter than P2 (development use only): the built-in food comparator (`area_food` of the founder, lab-authored, it keeps the founder's readiness path) silenced vs unsilenced under R1, R4 and R5 on P2's food bank | As P2; 2 min | R1 fails development validation if the comparator pair exceeds m = 0.5; then the adoption order moves to R4, then R5, each also needing this pair within m. Predicted: R1 fails (the comparator eats enough to pass the gate), R4 and R5 pass exactly | predeclared | |

## Frontier table

Descriptive only: paired difference from the reference arm, by seed, in
reached fraction and stall rung; never ranks or retires an arm. Filled once
phase 3 arms run on both repaired assays.

## Findings by phase

None yet.

## Review dispositions

None yet.

## Evidence outcome

Not yet reached.

## Execution status

In progress.
