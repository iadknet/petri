# Evolvability exploration, run 6: results (2026-10)

Date: 2026-10-04. Plan:
[evolvability-exploration-plan-2026-10-04-run6.md](evolvability-exploration-plan-2026-10-04-run6.md)
(committed on main at `18c796ae` after three Codex rounds), following
[run 5](evolvability-exploration-2026-10-run5.md) under the
[T22 exploration contract](../roadmaps/t22-capability-assays-and-evolvability-lab.md).
Status: in progress.

## Question

Over 1,000 generations, does M4 (per-unit rate × 4) or an exuberant
silent-afferent start (ALT) reach the calibrated threshold more often than an
A/A control? And does M4 change persistence in the production goal worlds?

## Run setup

- **Branch.** `worktree-evolvability-exploration-6`, from run 5's tip `8fb85fb7`
  plus the plan.
- **Campaign binary.** Pinned `v3-lab-aa-instr`, sha256 `10876c1e…`.
- **Production binary.** `v3-cli` from `main` `18c796ae`, sha256 `e902a94e…`.
- **Seeds.** 11–23 calibrate on both assays (none skipped). Seed 11 is the
  pilot on each assay.
  - Food selects fraction 0.02 on most seeds, 0.04 on seeds 18 and 19, and
    0.08 on seeds 16 and 22.
  - Wall selects scale 2 on all of them.
- **Evidence.** `.bench-artifacts/lab/exploration/run6/`, copied to the main
  checkout at closing.

## Ledger

| ID | Hypothesis | Arms, genomes and controls | Assay, scenes, sizes | Prediction / falsifier | Status | Result |
| --- | --- | --- | --- | --- | --- | --- |
| F0 | Build: ALT draws come from the production `SingleChannel` sampler, K = 24 weight-0 edges, frozen and unconditional; `r3_masking` takes a reference arm | `lab:` commit `b07ceb66`: probe `r6_alt` and 9 unit tests; `PETRI_R3_REF`. Codex review 1 `ready` | Lab tests | Fixtures pass; every draw loads in the pinned binary | completed | The production `recruit` runs through `GraphMutator::apply_with_food_type_count(RecruitNeutralInput)`, so it is called, not reproduced, and no distribution test is needed. **Construction choice, stated explicitly**: every edge targets node 1 (the vote node) through the production `TargetSelector` with executed set {1}. Both founder nodes carry vote sinks, so the estimand concerns these node-1 starts. One `SmallRng` per draw seeded `hash("run6-alt", assay, seed)`; a skip panics, so no draw is rejected or replaced |
| F1 | Freeze: seeds, ALT draws (lab hashes), binaries, sizing inputs | `freeze6.sh`; ALT parity reported (never gating) | Development banks | — | frozen before any campaign | 30 draws: seeds 11–23 per assay, plus 101 and 102 per assay ([`alt-draws.ndjson`](evolvability-exploration-2026-10-run6/alt-draws.ndjson)). All are in the sampler domain. All loaded through the pinned binary in a calibrate-only run at seed 1 (no run 6 seed, no holdout), with matching start and arm hashes. Genome size goes from 97 to 139–153 units (12–20 new references each), about 1.7 times the plan's estimate. **Parity** (reported): votes and actions are identical on every shared tick in all scenes. ALT dies about one tick earlier in most scenes because of the carrying cost, so food scores differ in 54 of 480 scene readings; wall scores are identical |
| PL | Pilot (seed 11 per assay; excluded from inference) and sizing (rule 2) | One full invocation per assay at 1,000 generations | Both assays | Joint B power ≥ 0.8 within 60 h, or the best affordable pair recorded as accepted C risk | predeclared | |
| L-M4 | M4 reaches more often than N by 1,000 generations | M4 against N by replicate on the frozen main batches | Both assays | A candidate if b − c ≥ ⌈n / 4⌉ on both assays; B if U < 0.25 on both (masked). Predicted: no detectable excess, since run 4 saw none at 100 generations | predeclared | |
| L-ALT | ALT reaches more often than N by 1,000 generations | ALT against N, as L-M4 | Both assays | As L-M4. Predicted: no detectable excess | predeclared | |
| CT | ALT matched controls (descriptive) | `--genome alt` invocation per assay at 100 generations on the pilot seed | Both assays | — | predeclared | |
| MK | Masking (B only) with N as the reference | `r3_masking`, `PETRI_R3_REF=native-aa` | Development bank and validation scenes | Scene identity must hold | predeclared | |
| P | Production check: M4 against the default | `v3-cli run`, 20,000 ticks, 3 goal worlds × 2 fresh seeds | Production | Descriptive. Harm guard for an M4 A. Complete when all 12 runs finish and every manipulation check holds | predeclared | |
