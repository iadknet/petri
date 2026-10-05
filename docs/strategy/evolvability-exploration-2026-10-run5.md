# Evolvability exploration, run 5: results (2026-10)

Date: 2026-10-02. Plan:
[evolvability-exploration-plan-2026-10-02-run5.md](evolvability-exploration-plan-2026-10-02-run5.md)
(committed on main at `2d1b6946` after three Codex rounds), following
[run 4](evolvability-exploration-2026-10-run4.md) (outcome B) under the
[T22 exploration contract](../roadmaps/t22-capability-assays-and-evolvability-lab.md).
**Evidence outcome: B (supported negative), for the declared effect only.**
**Execution status: complete** (screen, A/A, controls and comparison set, and
the single analysis point). Closure: the note lands on `main` in the inherited
close order after its closing review.

Among natural births where a jump fired, pooled over the frozen parents, this J
configuration is not twice as often confirmed-helpful as the native ±20 % step
on either assay; the declared doubling is excluded.
- On food, fired J births are helpful 0.40 times as often as their native twins
  (66 against 165) and harmful about twice as often (904 against 434). The
  discordance ratio is 29 against 128.
- On wall the two are close (175 against 180 helpful). Wall's θ_U of 1.59 still
  allows a smaller benefit.

## Question

Does a heavy-tailed weight jump (J) make a natural birth confirmed-helpful
more often than the native ±20 % step does, on the founder and on evolved
reference elites of both repaired assays? This is read directly as a
mutation-effect screen, not through campaigns (plan rules 2–7).

## Run setup

- **Branch.** `worktree-evolvability-exploration-5`, from run 4's tip
  `04b31ef4`, plus:
  - the plan (`08b6833b`);
  - prototype J (`020b85d3`, `9ab66a60`), cherry-picked from `run5-j-proto`;
  - the screen probe (`17752bb9`, `05de2486`, `95644d1c`), cherry-picked from
    `run5-screen`.
- **One build** serves the screen and any campaigns.
- **Evidence.** Raw output is under `.bench-artifacts/lab/exploration/run5/`,
  copied to the main checkout at closing. Compact summaries are in
  [`evolvability-exploration-2026-10-run5/`](evolvability-exploration-2026-10-run5/):
  - [`analysis5.json`](evolvability-exploration-2026-10-run5/analysis5.json):
    the gated verdict;
  - [`run5-projection.json`](evolvability-exploration-2026-10-run5/run5-projection.json):
    versioned compact counts behind every number here, with raw byte totals;
  - [`comparison-set.json`](evolvability-exploration-2026-10-run5/comparison-set.json);
  - the manifest and the freeze summaries;
  - [`aa-equivalence.ndjson`](evolvability-exploration-2026-10-run5/aa-equivalence.ndjson);
  - [`range-provenance.ndjson`](evolvability-exploration-2026-10-run5/range-provenance.ndjson):
    sha256 of all 501 range files;
  - the analysis code.

## Ledger

| ID | Hypothesis | Arms, genomes and controls | Assay, scenes, sizes | Prediction / falsifier | Status | Result |
| --- | --- | --- | --- | --- | --- | --- |
| F0 | Build: J is default-off and leaves native births byte-identical; the screen probe is deterministic | J fixtures 1–7 plus the recipe vectors and rollback case (`v3-core`); the screen's unit tests and its smoke: whole vs split ranges, one thread, a fired birth, an arm-only confirmed pair. Reviews: Codex J reviews 1–2 (`ready`), screen reviews 1–5 (`ready`, analysis gate included) | Unit fixtures; 2-scene smoke banks | All pass; `make check` passes apart from the known recipe-digest pin | completed | All pass: `make check` stops only at the known v3-cli recipe-digest pin (`checked_in_goal_recipe_identities_are_unchanged_by_json_precision`); workspace Clippy clean; `v3-core` mutation tests 497 passed; `v3-lab` tests pass; the smoke passes on the pinned release binary `screen-bin` (sha256 `210f2e9e…`) |
| F1 | Freeze (rule 3): 16 strata per assay; banks A and B; frozen records; sterility baselines; PC-J and PC-F | `r5_freeze` on run 4's reference elites (seeds 5–10, from M1's invocations), with run 3's B3 elites as fallback | Food bank A `0x3DF00D`, B `0x5BF00D`; wall A `0xB2DE00`, B `0x5BBA44` | — | frozen before any screen read | Food: 16 strata (founder plus 15 distinct run 4 elites from seeds 5–10; none skipped), bank A `0eacb7dd…`, bank B `23d6fb11…`, parent list sha256 `b89c5987…`; PC-F built (bank A 8.18, B 6.97). Wall: 16 strata, bank A `bc6b60c1…`, bank B `61d20165…`, parent list `e970d970…`. **PC-J**: candidate 1 (ring edge at −1.0) fails its precheck (no damage: Δ 0.0 on both banks); candidate 2 (the same edge sign-flipped to +2.0) passes: bank A 10.39 → 4.99, restoring −2.0 gains +5.40 (A) and +4.95 (B), not sterility-attributable |
| P0 | Pilot and size (rule 5) | J against native, tag `run5-pilot`, 20,000 twins per stratum | Both assays | N per assay = clamp(⌈20,000 × 150 / max(m₀, 1)⌉, 20,000, 1,000,000), committed in the manifest before the main draw | completed; manifest committed before the main draw | Food: b₀ = 3, c₀ = 20 (m₀ 23), so N = 130,435 per stratum. Wall: b₀ = 9, c₀ = 6 (m₀ 15), so N = 200,000. Expected m = 150 per assay; sign-test power at θ = 2 is 0.98. Throughput 42–340 pairs/s by stratum, so range sizes are set per stratum to finish within 540 s ([`ranges.txt`](evolvability-exploration-2026-10-run5/ranges.txt)). Estimated runtime: J main draw 14.1 h, founder reruns 0.6 h. Manifest [`manifest.json`](evolvability-exploration-2026-10-run5/manifest.json) (sha256 `efc16b8f…`) |
| PJ | PC-J pilot and power (rule 6) | J on the PC-J genome, tag `run5-pcj-pilot`, 100,000 twins | Wall | `powered` if the exact Poisson power of the sign test at N = 1,000,000 is at least 0.8 | completed | b₀ = 3, c₀ = 15 in 100,000 twins: on its own repair genome the jump is helpful less often than the native step (the step that shrinks the wrong-sign +2.0 weight helps too). Power at N = 1,000,000 is numerically negligible (about 4 × 10⁻³¹), so PC-J is `underpowered`: reported, not gating, and its main draw is not run (rule 6) |
| J | J's screen (rules 5–7) | J against native, tag `run5`, N per stratum, 32 strata | Both assays | Pass: pooled one-sided sign p ≤ 0.025, b/c ≥ 1.5, no assay reversal, leave-largest-stratum-out p ≤ 0.05. Supported negative: on each assay θ_U < 2 or δ_U < 2.5 × 10⁻⁴, PC-J holds unless it is not constructible or underpowered. Predicted (prior art): food no gain at the founder, more harm; wall gain if anywhere at elites and PC-J | completed (123 ranges, about 14 h) | **Supported negative.** Food: b = 29, c = 128 over 2,086,960 twins (discordance ratio 0.23; θ_U = 0.39). Wall: b = 50, c = 55 over 3,200,000 twins (θ_U = 1.59). Pooled 79 against 183, one-sided p ≈ 1.0; both assays reverse (b < c with m ≥ 10). The prediction held for food. On wall's 15 elite strata the cells are 35 against 34 (descriptive; the doubling exclusion applies to the full frozen-parent pool, not to the elites alone) |
| V | Validity (rule 6) | A/A: native against native on independent seeds (`run5-aa`), same N; founder reruns on both assays; twin identity; exact 99.9 % fire-rate range; fixtures | Both assays | Void (every J label `inconclusive`) unless all hold; enforced by `analyze5.py`'s gate | completed | **Valid.** Manifest complete; all 32 strata tile 0..N with consistent provenance; twin-identity violations 0; jumps fired on 15,074 of 151,157 applied weight events (9.97 %, inside the exact 99.9 % range); founder reruns byte-identical on both assays; A/A b = 34,918, c = 34,695, two-sided p = 0.40, with record equivalence on both samples (row V-A2) |
| V-A | Implementation of rule 6's A/A check, fixed before any A/A data | The probe's `aa` mode would re-evaluate the main draw's native twins (about 28 h). Instead: native children on the `run5-aa` seeds (arm `native`, tag `run5-aa`, whose arm side is a copy), paired by child index with the main draw's native-side records (arm `j`, tag `run5`). b = run5-aa-only counted, c = run5-only counted: the same statistic. Guards: every record must sit in its own range's summary file, and the counted records per range must equal `native.counted`. **Equivalence before use**: the probe's own `aa` mode on food stratum 0 and wall stratum 7 over 0..20,000 must give identical record sets on both sides and identical cells (non-empty), or the A/A stage stops. Codex reviews 1 (`not-ready`: equivalence not enforced, orphan records) and 2 (`ready`) | Both assays | Valid when every guard holds and the two-sided p is at least 0.01 | predeclared | |
| V-A2 | Deviation: the predeclared wall equivalence sample was vacuous | Food stratum 0 matched: identical record sets on both sides (238 run5 and 227 run5-aa records) and identical cells. Wall stratum 7, the fastest stratum, picked for cost, has 0 counted records on both sides in 0..20,000: the sets and cells agree, but the non-empty requirement fails, and `aa.sh` stopped as designed. That would void the screen through a sample choice that says nothing about the pipeline. **Replacement, fixed by rule before any A/A result is read**: the wall founder, stratum 0, mirroring the food sample. The same checks apply (identical non-empty record sets on both sides, identical cells) | Wall | Equivalence holds on the replacement sample | completed | Holds: identical record sets (219 run5 and 224 run5-aa records) and identical cells (b 222, c 217). The A/A result was computed only after this |
| PCJ | PC-J holds (rule 6), if powered | J on the PC-J genome, tag `run5`, N = 1,000,000 | Wall | Holds when the twin sign test gives p ≤ 0.025 | not run (PC-J `underpowered`, row PJ) | — |
| PCF | PC-F (rule 6, descriptive) | Native on the C2 food genome after 9 edits, 4,096 children | Food | The pipeline sees a known helpful step | completed | 52 of 4,096 children confirmed helpful (1,310 changed), the largest Δ_A +0.39. The known +19.26 vote edge, one specific addition, does not appear in 4,096 natural births |
| CS | Comparison set (rule 5, descriptive, never deciding) | One step (twin sign test, N = 4,096 per stratum): refinement, M4, M5. Two steps (64 changed-neutral children × 16 grandchildren per stratum, against native): M1, M2, M1 + refinement | Both assays | Cross-table beside run 4's campaign verdicts; agreement neither validates nor refutes the screen | completed (96 one-step and 128 two-step invocations, all complete) | See the cross-table below |
| BC | Deviation: byte cap (rule 10 / T22 telemetry rule) | The screen probe and its run wrappers enforce no cumulative byte cap | — | — | recorded at closing review 1 | Screen outputs total 307,079,114 bytes, and the whole run 5 evidence tree (with logs, scripts and the pinned binary) 311,930,626 bytes at closing, measured just before the projection file itself was rewritten (`run5-projection.json`: `screen_output_bytes`, `evidence_tree_bytes`), all under the ignored `.bench-artifacts/lab/`. Records were written only for confirmed-helpful children (no per-proposal records). Reusing the instrument requires cap enforcement first. The verdict is unaffected |
| AN | Analysis at the single analysis point | `analyze5.py main` over the manifest, main, A/A, rerun and equivalence directories; `cs5.py` for the comparison set | — | — | completed | **Tooling fix at the analysis point**: the first run stalled in the exact sign-test tail, because the A/A's discordance is in the tens of thousands (independent seeds rarely draw the same helpful children). For m > 2,000 the tail now uses the log-space binomial sum, which matches the exact integer sum to within 3 × 10⁻¹³ at m = 2,000. No rule changed |

## Births where a jump fired

Descriptive: each J birth in which a jump fired, against its native twin. The
twin differs only by the jumped weight values.

| Assay | Fired twins | J: helpful / harmful / inert | Native twin: helpful / harmful / inert |
| --- | --- | --- | --- |
| `food-seeking-hunger` V2 | 4,209 | 66 (1.6 %) / 904 (21.5 %) / 54 | 165 (3.9 %) / 434 (10.3 %) / 56 |
| `wall-v1` | 10,835 | 175 (1.6 %) / 2,175 (20.1 %) / 1,114 | 180 (1.7 %) / 1,472 (13.6 %) / 1,123 |

Weight events are 2.0 % of food births and 3.4 % of wall births, so at
p = 0.1 a jump fires in about 0.2–0.3 % of births. Summed Δ_B over all counted
helpful children is nearly identical on both sides: food 1,612 (J) against
1,632 (native), wall 23,096 against 23,102.

## Comparison set (descriptive)

One step: twin cells over 65,536 seed-paired births per assay. Two steps:
positive units (a child with a confirmed-helpful grandchild) out of 1,024
changed-neutral children per side, with a stratified exact one-sided p. Run 4's
campaign verdict is shown beside each arm. Agreement neither validates nor
refutes the screen.

| Arm | Food | Wall | Run 4 campaign |
| --- | --- | --- | --- |
| refinement (one step) | b 26, c 37 (two-sided p 0.21) | b 62, c 57 (0.71) | B (no excess) |
| M4 rate × 4 (one step) | **b 487, c 38**; per changed child 1.2 % against 0.9 % | **b 1,446, c 75**; per changed child 3.3 % against 1.8 % | B (no excess) |
| M5 cross-node source (one step) | b 15, c 27 (0.09) | b 31, c 43 (0.20) | B (no excess) |
| M1 (two steps) | 37 against 37 (p 0.55) | 108 against 106 (0.46) | B (no excess) |
| M2 (two steps) | 37 against 37 (0.55) | 110 against 106 (0.39) | B (no excess) |
| M1 + refinement (two steps) | 42 against 37 (0.32) | 103 against 106 (0.64) | B (no excess) |

## Findings

- **This J configuration does not double the helpful rate. On food it is
  helpful less often and harmful more often than the step.**
  - Food: in births where a jump fired, J's child is confirmed-helpful 66
    times against the native twin's 165 (ratio 0.40), and harmful 904 times
    against 434.
  - Wall: helpful 175 against 180, harmful 2,175 against 1,472.
  - The wall witness path's 16 small re-weights (run 3, C2) did not show up as
    a large per-event advantage for jumps at these parents. A smaller benefit
    is not excluded on wall.
  - PC-J, the comparator with one ring edge sign-flipped, points the same way
    in its pilot. J was the helpful one in 3 discordant pairs, the step in
    15: a step that shrinks the wrong-sign weight already helps. With these
    rates its power at 1,000,000 twins is numerically negligible
    (about 4 × 10⁻³¹).
- **M4 supplies more helpful one-step children, yet run 4's campaigns saw no
  gain.**
  - Per drawn child, M4 gives about three times as many confirmed-helpful
    children: 675 against 226 on food, 1,986 against 615 on wall.
  - Per changed child it gives about 1.3–1.8 times as many.
  - Its twin discordance is 487 against 38 (food) and 1,446 against 75 (wall).
  - Run 4 excluded its declared large campaign effects for M4 (reach and food's
    benefit-rung stall at quick sizes).
  - How the screen's supply increase relates to campaign outcomes is
    unresolved: the two are separate measurements. This reading is
    descriptive.
- **No difference detected for recruitment, refinement or cross-node edges at
  the screen.** Their one-step and two-step counts are close to native's, and
  their run 4 campaign readings were B.
- **The screen instrument works.**
  - Twin identity held with 0 violations across 5.3 million pairs.
  - The A/A check is balanced (p 0.40).
  - It detects large differences in both directions: J's food deficit and M4's
    supply excess.
  - It costs about 1 hour per 400,000 twins.

## Evidence outcome

**B (supported negative), for the declared effect only.**
- On `food-seeking-hunger` V2 and `wall-v1`, at the founder and 15 evolved
  reference elites per assay, J (p = 0.1, Laplace(0, 1) jumps clamped to
  ±max(5, |w|)) is not twice as often confirmed-helpful as the native step,
  among natural births where a jump fired, pooled over the frozen parents.
- The upper bounds on the discordance odds are 0.39 (food) and 1.59 (wall).
  Each is built from two one-sided 97.5 % Clopper–Pearson bounds, so its joint
  coverage is at least 95 % per assay (plan rule 5).
- PC-J was underpowered and did not gate (plan rule 6).
- The claim does not cover other jump laws or probabilities, other parents or
  assays, or ecological selection.

**Recommendation.** No roadmap feature for heavy-tailed weight jumps. J stays
branch-only.

The mutation-effect screen is a useful, cheap instrument. It is a candidate
T22 feature: a natural-birth twin screen with an A/A check. Its first finding
is that supply-side arms (M4) raise one-step helpful children where campaigns
showed no gain. The gap between the screen and the campaigns is the open
question. The side chat's other options (longer horizons, the T18 founder,
production ecology) remain unrun.

## Review dispositions

Plan reviews: three Codex rounds (round 3 `ready`). Code reviews: J 1–2, screen
1–5 (analysis gate included) and record-based A/A 1–2, recorded in the ledger
rows.

### Closing review 1 (Codex)

B confirmed. Codex independently recomputed b, c, n and both bounds per assay,
the pass rules, PC-J's power, every validity check and all 501 provenance
hashes.

| # | Finding | Severity | Disposition |
| --- | --- | --- | --- |
| 1 | "A quarter as often" is the discordance ratio (29/128); the fired-birth helpful ratio is 0.40. "Do not help" and "no gain at elites" overstate a doubling exclusion | blocking | Accepted: the opening, the J row and the findings state the doubling exclusion for this configuration, pooled over the frozen parents. They report 0.40 and 0.23 separately, and wall's elite cells (35/34) with the smaller benefit left open |
| 2 | θ_U called a "97.5 % bound" | blocking | Accepted: described as built from two one-sided 97.5 % bounds, with joint coverage of at least 95 % per assay |
| 3 | Compact summaries cannot regenerate the note's fired-birth, M4, PC-F and per-stratum figures | blocking | Accepted: versioned [`run5-projection.json`](evolvability-exploration-2026-10-run5/run5-projection.json) with its script and raw byte totals |
| 4 | Closing dispositions empty; evidence not yet in the main checkout | blocking before closure | Accepted: this table; the close order follows (evidence copied and verified before the worktree is removed; `make check-docs` on `main`) |
| 5 | No cumulative byte cap in the probe or wrappers | advisory | Accepted: ledger row BC (screen outputs 307 MB, evidence tree 312 MB; cap required before reuse) |
| 6 | Causal M4 wording; "flat"; PC-J power "0.0" | advisory | Accepted: "no difference detected"; the M4 relationship is unresolved; PC-J power is about 4 × 10⁻³¹ |

## Branch and commits

- `020b85d3`, `9ab66a60` (`proto:`): J, from `run5-j-proto`.
- `17752bb9`, `05de2486`, `95644d1c` (`lab:`): the screen probe, from
  `run5-screen`.
- The ledger commits and this note.
- Only the note and its summaries land on `main`. The `proto:` commits, and
  the `lab:` probe, which depends on run 3's `instr:` commits, stay on the
  branch.
