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
| P1 | 1 | Sterility shortcut: the plateau's food-seeking gain comes from silencing the reproduce readiness signal, not from foraging | Diagnostic genomes (plan rule 7, never selected or seeded): founder 2 × 2 = readiness silenced (node 0 `CustomOutput(1)` inputs cleared) × the plateau's upstream re-pointing (node 1 input ref 1 → slot 12, ref 2 → slot 9, as in `native-1`); decomposition: ref 1 only, ref 2 only; the plateau `native-1`; the plateau restored (node 0 `Multiply(CN0, CN1)` re-added as CN2 feeding `CustomOutput(1)`, node 1 ref 1 → slot 1); three more elites whose readiness path is intact (node 0 `CustomOutput(1)` fed by a live compute node and node 1 ref 1 reading slot 1): `native-0`, `native-5`, `shuffled-score-5` (run 1's E1 elites), each silenced and unsilenced | food-seeking scenes as run 1's E3 bank (32 scenes, seed `0xE3BA4C`, 64², food 0.04, 200 ticks, start energy 100); bank mean score, food eaten, moves attempted, penalty charged, deaths; wall budget 5 min | Predeclared: margin m = 0.5 bank-mean points (the margin the repaired version will use); gain G = plateau − founder. **Dominant** if silenced founder − founder ≥ 0.7 G and plateau − restored plateau ≥ 0.7 G; **contributor** if not dominant and any silencing contrast (silenced − unsilenced for the founder with or without re-pointing, for each elite, or plateau − restored) exceeds m; else **no shortcut**. Factors: silencing main effect, re-pointing main effect and their interaction from the 2 × 2. Predicted: dominant. Falsified if either 0.7 G condition fails | predeclared | |

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
