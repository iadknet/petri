# Evolvability exploration, run 4: results (2026-10)

Date: 2026-10-02. Plan:
[evolvability-exploration-plan-2026-10-02-run4.md](evolvability-exploration-plan-2026-10-02-run4.md)
(committed on main at `4bb0c491` after five Codex rounds), continuing
[run 3](evolvability-exploration-2026-10-run3.md) under the
[T22 exploration contract](../roadmaps/t22-capability-assays-and-evolvability-lab.md).

**Evidence outcome: B (supported negative), for the declared effects only.**
**Execution status: complete** (all 88 campaigns, 72 masking probes and the
single analysis point). Closure: the note lands on `main` in the inherited
close order after its closing review.

None of run 3's six mechanism arms raises reach by 0.25 or more over an
independent mutation stream on either assay. None moves food's benefit-rung
stall by 0.25 or more. On wall, B rests on reach alone, because the A/A control
itself leaves the reference's stall rung in 18 of 32 eligible pairs.

## Question

Does any of run 3's six arms change reach or the reference's stall rung by more
than stream noise does? Run 4 measures that noise with an A/A control (the
reference on an independent mutation stream) and compares every arm with it on
48 fresh pairs per assay (plan rules 2–4).

## Run setup

- **Branch.** `worktree-evolvability-exploration-4`, from run 3's tip
  `12011c07` plus the plan.
- **Instruments.** `food-seeking-hunger` V2 (lifetime 200, standard fraction
  grid) and `wall-v1` (scale 2, lifetime 200).
- **Sizes.** Population 32, 100 generations, 8 replicates per seed batch,
  retention depth 2, `--quick` otherwise.
- **Fresh seeds.** 5–10 on both assays (row S0).
- **Builds** (sha256):
  - pinned `instr:` from run 3, `9849fd5e…`: the overlay arms, and every
    fresh combination's baseline (M1's invocation);
  - pinned `proto:` from run 3, `8edccef2…`: M5;
  - A/A build `2b69ee79` (`instr:`-only base), `10876c1e…`: the A/A control
    (row AA0).
- **Evidence.** Raw output is under `.bench-artifacts/lab/exploration/run4/`
  (copied to the main checkout at closing). Compact summaries are in
  [`evolvability-exploration-2026-10-run4/`](evolvability-exploration-2026-10-run4/):
  - [`analysis4.json`](evolvability-exploration-2026-10-run4/analysis4.json):
    the formal result;
  - [`analysis4-pooled-exploratory.json`](evolvability-exploration-2026-10-run4/analysis4-pooled-exploratory.json);
  - [`masking-verdicts.ndjson`](evolvability-exploration-2026-10-run4/masking-verdicts.ndjson);
  - [`seeds-calibration.ndjson`](evolvability-exploration-2026-10-run4/seeds-calibration.ndjson);
  - [`campaign-provenance.ndjson`](evolvability-exploration-2026-10-run4/campaign-provenance.ndjson):
    every campaign's summary and rows sha256;
  - the analysis code ([`analyze4.py`](evolvability-exploration-2026-10-run4/analyze4.py)
    with its synthetic tests).

## Ledger

| ID | Hypothesis | Arms, genomes and controls | Assay, scenes, sizes | Prediction / falsifier | Status | Result |
| --- | --- | --- | --- | --- | --- | --- |
| S0 | Seed selection (rule 3): the first six seeds from 5 upwards that calibrate on each assay | `--calibrate-only` on the pinned `instr:` build, holdouts skipped | Both assays, the frozen sizes | — | completed (seconds each) | Seeds 5–10 on both assays, none skipped. Food selects fraction 0.04 on seeds 5, 6, 8, 9 and 10 (thresholds 18.45–22.06), and 0.02 on seed 7 (threshold 10.65); wall selects scale 2 on all six |
| AA0 | Build: the A/A control (`instr:`) adds `native-aa`, the reference on mutation stream `tagged(replicate, "mutation-aa")`, only under `--aa-control`, and moves no other arm | Unit fixture: with the reference's own tag the control reproduces the reference's rows (arm name, role and name-seeded signature aside) and ladder exactly; with the salted tag it differs after the first breeding. End-to-end fixture: every other arm's rows are byte-identical with and without the flag | Lab tests | Both fixtures pass; on every run 4 combination the A/A build reproduces the baseline's reference rows and summary projection (rule 5) | completed; per-combination identity checks in row AA | Fixtures pass (`cargo test -p v3-lab`; Clippy clean). **Build deviation, corrected before any A/A data was used**: the first A/A build (`18a8819e` on the branch tip, sha256 `1a4c4b1a…`) also carried M5's default-off `proto:` field, so on food seed 3 its reference rows equalled B3's but the summary projection did not: the config digest serializes that field (`3870d05c…`, the same digest as run 3's M5 build, against `2d997590…`). With the digest removed, its projection and every built-in arm's rows equal B3's. That run is set aside (`aa-tipbuild-food-s3`). The A/A control now runs on an `instr:`-only build: the same change on `33ae32f9` (commit `2b69ee79`, branch `worktree-evolvability-exploration-4-aa`, sha256 `10876c1e…`), fixtures passing. **Rule 5 for M5, amended before M5 runs**: the pinned `proto:` build has the same digest difference by construction, so M5 is checked on its projection without `config_digest` and on every built-in arm's rows against M1's invocation of the same combination. The check returns equal on run 3's M5 and the set-aside A/A run against B3, and unequal across seeds |
| AA | The A/A control: stream noise on both effects | `native-aa` on the A/A build, with the built-in arms | Both assays; the 6 fresh seed batches and run 3's batches 3 and 4 | Descriptive: N's reach and rung departures against R, which calibrate every arm's excess | completed (16 of 16, 175–553 s each) | Rule 5 holds on all 16. **Fresh pairs**: N reaches in 0 of 48 pairs on either assay (R also 0). Food: E = 48 (every R replicate fails the benefit rung with adequacy), and N departs in 0. Wall: E = 32, and N departs in **18 of 32 (56 %)**, so under rule 4's scope gate the wall rung is out of scope. Pooled with run 3's batches (exploratory): N-only reaches 1 on food (seed batch 3) and 1 on wall (seed batch 4, replicate 5) |
| X1 | M1 (single-channel recruitment) exceeds stream noise | Run 3's M1 overlay on the `instr:` build | Both assays, 48 fresh pairs | Rule 4: A if b − c ≥ 12 of 48 for reach (or ⌈n/4⌉ for the rung) on both assays; B-eligible if U < 0.25. Predicted (run 3): no excess on either assay | completed (12 of 12, up to 390 s) | Prediction held. Food: reach b − c = 0 − 0 (U 0.074); rung 0 − 0 of 48 (U 0.074). Wall: reach 0 − 0 (U 0.074; the bound B needs); rung 8 − 9 of 32, out of scope, so its descriptive masked bound (0.297) does not enter B. Masking: 0 reach, 2 wall rung pairs. No A candidate; B on both assays |
| X2 | M2 (whole-family recruitment) exceeds stream noise | Run 3's M2 overlay | As X1 | As X1 | completed (12 of 12, up to 425 s) | Prediction held. Food: reach 0 − 0, rung 0 − 0 of 48. Wall: reach 0 − 0; rung 5 − 10 of 32 (out of scope). Masking: 4 wall rung pairs. B on both assays |
| X3 | Refinement exceeds stream noise | Run 3's refinement overlay | As X1 | As X1 | completed (12 of 12, up to 424 s) | Prediction held. Food: reach 0 − 0, rung 0 − 0 of 48. Wall: reach 0 − 0 (1 − 0 masked, U 0.111); rung 6 − 7 of 32 (out of scope). B on both assays |
| X4 | M1 + refinement exceeds stream noise | Run 3's M1 + refinement overlay | As X1 | As X1 | completed (12 of 12, up to 397 s) | Prediction held. Food: reach 0 − 0, rung 0 − 0 of 48. Wall: reach 0 − 0; rung 9 − 5 of 32 (out of scope). B on both assays |
| X5 | M4 (per-unit rate × 4) exceeds stream noise | Run 3's M4 overlay | As X1 | As X1 | completed (12 of 12, up to 410 s) | Prediction held. Food: reach 0 − 0, rung 0 − 0 of 48. Wall: reach 0 − 0; rung 4 − 8 of 32 (out of scope). B on both assays |
| X6 | M5 (H8 cross-node source) exceeds stream noise | Run 3's M5 overlay on the pinned `proto:` build; its built-in arms must reproduce the baseline | As X1 | As X1 | completed (12 of 12, up to 379 s) | Amended rule 5 holds on all 12: every built-in arm's rows and the projection without `config_digest` equal M1's invocation. Prediction held. Food: reach 0 − 0; rung 0 − 0 of 48 (lenient masked 1 − 0, U 0.111). Wall: reach 0 − 0; rung 8 − 7 of 32 (out of scope). B on both assays |
| MK | Masking (rule 4) for every arm and fresh pair where X's or R's final elite attempts reproduction on the development bank | Run 3's `r3_masking` probe, fixed (`7387e663`) | Each assay's development bank and the seed's validation scenes | Descriptive; feeds B only | completed (72 of 72, seconds each) | **Probe fix**: run 3 never ran this probe, and its summary field path was wrong (`validation_scenes` lives under `provenance.sizes`). The first queue failed on every combination before writing any output. Fixed, and the verdict now records **scene identity** (the founder's mean on the probe's validation scenes equals the gate's recorded founder mean): true in all 72. Pairs checked per arm: food 2–5, wall 38–42. Masked: 1 reach pair (refinement, wall) and 0–4 rung pairs per arm. No unresolved case |
| AN | Analysis tooling, frozen before the single analysis point | `analyze4.py` with `test_analyze4.py` (synthetic data only) under `.bench-artifacts/lab/exploration/run4/`. Codex code-only reviews (3 rounds, `ready`) read no campaign output | — | The script implements rule 4 as written. Decisions recorded here before any read: (1) formal inference needs six distinct fresh seeds, replicate and ladder IDs 0–7 in order in R, N and X, and 48 pairs per assay; (2) R's results and ladder must match between each arm's invocation and the A/A invocation, and every invocation must have logged a passing rule 5 check; (3) reach is tri-state, so an incomplete replicate's unknown reach never supplies X-only or N-only evidence; (4) any incomplete replicate blocks B (conservative); (5) E keeps run 3's convention that a reached reference has no stall rung (run 3 closing review 1, item 1) | frozen; run once at the analysis point | Formal inputs complete: 48 pairs per assay, no incomplete replicate, baseline log check passed. After masking, a mask file also needs scene identity to count as resolved (all 72 have it) |

## Frontier table

Descriptive only (run 1's plan): it ranks and retires nothing. Each cell shows,
for the fresh 48 pairs:
- **reach** b − c against the A/A control;
- **rung** b − c over E (strict coding);
- the final-best gap against the reference, in brackets.

| Arm | `food-seeking-hunger` V2 | barrier-navigation `wall-v1` |
| --- | --- | --- |
| reference R | reached 0/48; all 48 stall at `benefit: fail` with adequacy | reached 0/48; 33 stall with adequacy (17 at benefit, 16 at retention), 15 pass every rung; E = 32 (in one stalled pair N's read is inconclusive) |
| A/A control N | reached 0/48; departs 0 of 48 | reached 0/48; departs 18 of 32 |
| M1 recruitment, single channel | reach 0 − 0; rung 0 − 0 of 48; (−0.04) | reach 0 − 0; rung 8 − 9 of 32; (+0.17) |
| M2 recruitment, whole family | reach 0 − 0; rung 0 − 0 of 48; (+0.16) | reach 0 − 0; rung 5 − 10 of 32; (+0.17) |
| refinement | reach 0 − 0; rung 0 − 0 of 48; (+0.09) | reach 0 − 0; rung 6 − 7 of 32; (+0.38) |
| M1 + refinement | reach 0 − 0; rung 0 − 0 of 48; (+0.20) | reach 0 − 0; rung 9 − 5 of 32; (+0.01) |
| M4 rate × 4 | reach 0 − 0; rung 0 − 0 of 48; (+0.31) | reach 0 − 0; rung 4 − 8 of 32; (−0.11) |
| M5 cross-node source (`proto:`) | reach 0 − 0; rung 0 − 0 of 48; (−0.05) | reach 0 − 0; rung 8 − 7 of 32; (+0.33) |

## Findings

- **Independent mutation streams alone produce frequent wall rung
  departures.**
  - The A/A control departs from the reference's wall stall rung in 18 of 32
    eligible pairs.
  - The arms depart in 13 to 23, and their X-only counts sit beside their
    N-only counts (4–9 against 5–10).
  - These data do not establish an arm effect on the wall rung beyond that
    variation. Nor do they exclude one: the scope gate (predeclared) takes the
    wall rung out of B, which rests on reach alone there.
- **Food's benefit-rung stall is solid, and no mechanism moves it.**
  - Every one of the 48 reference replicates fails the benefit rung with
    adequacy. So do the A/A control and all six arms: 0 departures each.
  - The bound excludes a departure share of 0.25 or more (U = 0.074, 0.111 for
    M5 after masking).
- **Reach sits at the floor on fresh seeds.**
  - No reference, A/A or arm replicate reached the threshold on seeds 5–10 on
    either assay. That includes food seed 7 at fraction 0.02, threshold 10.65.
  - The observed baseline reach fraction is 0, so the reach bounds (0.074;
    0.111 for refinement's masked wall reach) exclude only a large rise.
  - Small gains, longer horizons and other sizes stay untested.
- **Pooled with run 3's batches (exploratory).**
  - On food, every reach comes from seed batch 3 (fraction 0.08). Arm-only
    reaches there are 1–3 per arm, and the A/A control also has one N-only
    reach.
  - On wall, the reference reaches once (seed 4, replicate 6) and the A/A
    control once (seed 4, replicate 5). No arm reaches.
  - The arms' run 3 food reaches are of the same order as the A/A control's.
    This is exploratory evidence, and it does not show that stream noise
    explains them.
- **Sterility.** Under the hunger regime, 2–5 food pairs per arm have an elite
  that attempts reproduction. None is masked for reach.
  - On wall, masking moves at most 4 rung pairs per arm and 1 reach pair
    (refinement).
  - Every bound that B requires stays below 0.25.
  - The descriptive masked wall rung bounds, which do not enter B, reach up to
    0.497 (M1 + refinement).
- **Scale and cost.** 88 campaigns took 3–9 minutes each (175–553 s for the
  A/A control) and ran sequentially, because one campaign saturates 8 cores.

## Evidence outcome

**B (supported negative), for the declared effects only.** On
`food-seeking-hunger` V2 and `wall-v1` at population 32 × 100 generations,
none of M1, M2, refinement, M1 + refinement, M4 and M5:
- raises the probability of reaching the calibrated threshold by 0.25 or more
  over an independent mutation stream, on either assay;
- makes the reference's food benefit-rung stall depart by 0.25 or more.

The wall rung was out of scope (A/A departures 56 %), so the wall claim is
reach-only. The reach contrast sits at a base rate of 0, so the result excludes
large effects only. It does not show that these mechanisms are useless in the
ecology, at longer horizons, or for smaller effects.

**Recommendation.** No roadmap feature from these six arms. Their prototypes
stay branch-only. Two lessons go into the next run's design:
- the quick-size wall rung is not a usable contrast;
- at these sizes, campaigns cannot see the benefit rung move.

[Run 5](evolvability-exploration-plan-2026-10-02-run5.md) (user direction,
2026-10-02) therefore measures mutation effects directly instead.

## Review dispositions

Plan reviews (five Codex rounds) are recorded in the plan's history. The
analysis-tooling reviews are in ledger row AN.

### Closing review 1 (Codex)

Outcome B confirmed. Codex independently recomputed every fresh contrast and
the frontier gaps, verified all 88 summary and row hashes, and checked the
baseline projections.

| # | Finding | Severity | Disposition |
| --- | --- | --- | --- |
| 1 | The wall finding claimed the movements "were" noise, which the out-of-scope contrast cannot establish | blocking | Accepted: reworded as frequent departures under independent streams, with no arm effect established or excluded |
| 2 | "Every masked bound stays below 0.25" is false for the descriptive wall rung bounds (up to 0.497) | blocking | Accepted: "every bound B requires"; X1's row separates its reach bound from its out-of-scope rung bound. B is unchanged |
| 3 | The pooled wall reaches are in seed 4, not seed 3. Pooled arm-only food reaches are 1–3 | blocking | Accepted: attribution corrected, and the wording kept exploratory |
| 4 | Closure prerequisites pending: dispositions, evidence copy, landing, worktree removal | blocking before landing | Accepted: this table; the close order follows (evidence copied before the worktree is removed) |
| 5 | A/A runtime 175–553 s; mention refinement's masked U 0.111; call 0 the observed base rate; the run 5 link | advisory | Accepted. The run 5 plan lands on `main` before this note, so the link resolves there |

## Branch and commits

- `18a8819e` (`instr:`): the A/A control arm. Superseded for the A/A build
  by `2b69ee79` on `worktree-evolvability-exploration-4-aa` (instr-only base).
- `7387e663` (`lab:`): the `r3_masking` field-path fix and the
  scene-identity check.
- Ledger commits `353a2c0b` … `6f99f48c` and this note.
- Only the note and its summaries land on `main`. The `instr:` and `lab:`
  probe commits depend on run 3's `instr:` commits and stay on the branch.
