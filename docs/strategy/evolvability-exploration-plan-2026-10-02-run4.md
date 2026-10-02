# Evolvability exploration, run 4: plan

Date: 2026-10-02. Source: `a0559bbd`. Status: plan for the continuation of
[run 3](evolvability-exploration-2026-10-run3.md) (outcome C by exhaustion),
under the [T22 exploration contract](../roadmaps/t22-capability-assays-and-evolvability-lab.md).
User direction (2026-10-02): keep going unattended.

## Why a fourth run

Run 3 ran every arm with 16 paired replicates on both repaired assays. Every
arm was `inconclusive` for outcome B, and none met the minimum effect on both
assays. Two things in run 3's data shape run 4.

- **The absolute bound cannot separate an effect from mutation-stream
  noise.** Every arm uses the reference's mutation and selection seeds, but
  any change to the operator pool changes how each child's mutations are
  drawn. From then on the arm's lineages follow a different random path.
  On `wall-v1` every arm, including M5, which the prior art predicted to be
  null, leaves the reference's stall rung in 3–6 of 8 usable pairs. On
  food every arm has 1–3 arm-only reaches. Run 2's B asks for an exact
  bound below 0.25 on those raw shares, which a pure difference in random
  path may already exceed. The refinement arm's 6 of 8 wall departures
  alone make that B unreachable at any affordable sample size.
- **Sixteen pairs are too few** for any bound to be tight.

Run 4 therefore measures the noise directly with a stream-only control and
compares every arm with it, on three times as many pairs.

## Rules

[Run 3's plan](evolvability-exploration-plan-2026-10-02-run3.md) governs
everything this section does not change, and through it runs 2 and 1. Changes
for run 4:

1. **Same instruments, sizes, arms and builds.**
   - Instruments: `food-seeking-hunger` version 2 as frozen in run 3, and
     `wall-v1`.
   - Sizes: population 32, 100 generations, retention depth 2.
   - Arms: run 3's six (M1, M2, refinement, M1 + refinement, M4, M5) with
     the same overlays.
   - Builds: the overlay arms keep run 3's pinned `instr:` binary
     (`v3-lab-instr`, sha256 `9849fd5e…`) and M5 the pinned `proto:` binary
     (`v3-lab-proto`, `8edccef2…`).
   - The branch starts from run 3's tip (`12011c07`) plus this plan.
2. **The A/A control** (an `instr:` change, a new built-in control). The
   reference configuration, with every replicate's mutation stream salted to
   an independent seed (`tagged(replicate, "mutation-aa")` instead of
   `tagged(replicate, "mutation")`). Its selection stream, scenes and
   everything else are unchanged.
   - It differs from the reference only in its random path, so its
     disagreements with the reference measure stream noise.
   - It runs through a new flag, in its own invocations, on its own build
     (the A/A build), which is checked to reproduce the pinned `instr:`
     build's reference rows and summary projection byte for byte.
   - It is labelled `control`, policy `native`.
   - A fixture test shows that with the reference's own tag the control
     reproduces the reference's rows and ladder exactly.
   - E below is the excess over this particular control. The control shares
     its selection stream and scenes with R, so it is an independent
     mutation-path control, not an independent campaign.
3. **48 fresh pairs per arm and assay.**
   - Every arm and the A/A control run six new 8-replicate seed batches. On
     each assay these are the first six seeds from 5 upwards whose
     calibration gate passes on that assay, chosen by `--calibrate-only`
     before any arm runs.
   - Skipped seeds are recorded, and the holdout seeds (101, 102, 111, 112,
     121, 122) are never used.
   - Formal inference (rule 4) uses the 48 fresh pairs only. Run 3's seed
     batches 3 and 4 are also run for the A/A control. The pooled 64 pairs
     are reported as exploratory, because the continuation was chosen after
     run 3's outcome.
   - Results are reported per seed batch and per food density beside the
     fresh 48. There is one analysis point, after every arm has completed all
     its batches.
4. **Noise-calibrated outcomes** (they replace run 2's rule 1 counts for run
   4, declared before any new data).

   Paired effect counts:
   - A *pair* is replicate i of one fresh seed batch; it holds R's, N's and
     X's runs for that replicate. For each arm X, assay and effect, *X-only*
     and *N-only* are the pairs in which X, respectively the A/A control N,
     shows the effect against the reference R and the other does not. Here
     *b* counts X-only pairs and *c* N-only pairs, over the effect's pair
     set of size n.
   - *Reach effect*: reached while R did not. Its pair set is all 48 fresh
     pairs.
   - *Rung effect*: at R's stall rung, R reads `fail` with adequacy and the
     run reads `pass` with adequacy. Its pair set is the *eligible set* E:
     the pairs where R reads `fail` with adequacy at its stall rung and N
     reads `pass` or `fail` with adequacy there. E depends on R and N
     alone, so it is the same for every arm.
   - On E, X counts as departed in two codings. The *strict* coding (used
     for A) needs X's `pass` with adequacy. The *lenient* coding (used for
     B) also counts any X read that is not `fail` with adequacy as a
     departure. That makes an unreadable X count against B.

   Bound:
   - **Excess**: E_X = (b − c) / n, the estimate of δ = P̄(X-only) −
     P̄(N-only), the mean over the n pairs of each pair's probabilities.
   - **Upper bound on δ**: U = CP⁺(b, n) − CP⁻(c, n). CP⁺ and CP⁻ are the
     exact one-sided 97.5 % Clopper–Pearson upper and lower bounds on a
     proportion, with two conventions: CP⁻ = 0 when c ≤ 1, and CP⁺ = 1 when
     b ≥ n − 1. By Bonferroni, U covers δ with probability at least 95 %
     whenever each bound covers its own mean at 97.5 %. Dependence between b
     and c does not matter.
   - **Heterogeneity**: the pairs are independent but not identically
     distributed (seeds, densities, masking). b and c are therefore
     Poisson-binomial. Hoeffding (1956, *Ann. Math. Statist.* 27:713–721,
     Theorem 4) bounds a Poisson-binomial tail by the binomial tail at the
     same mean for P(S ≤ k) with k ≤ np̄ − 1, and for P(S ≥ k) with
     k ≥ np̄ + 1. The two conventions keep every rejection region of both
     bounds inside those ranges. This holds because a binomial reaches its
     mean, and stays at or below it, each with probability above 1/4 when
     1/n < p̄ < 1 − 1/n (Greenberg and Mohri 2014, *Stat. Probab. Lett.*
     86:91–98), which a 2.5 % tail cannot contain. The two conventions cover
     the remaining extremes.
   - **Numeric check** (`cp_hetero_check.py` under the run's
     `.bench-artifacts/lab/exploration/run4/`, saved with the note):
     - Zero condition failures for n = 16 … 48 over a 1/20,000 grid of p̄.
     - Worst exact miss probability 0.02484 over two- and three-point
       heterogeneous designs.
     - Review 3's counterexample has a miss probability of 0.

   Outcomes, on the fresh pairs:
   - **A candidate**: on both assays, for the same effect, b − c ≥ ⌈n / 4⌉
     with the strict coding and the unmasked counts. For reach this is
     12 of 48. A rung candidate also needs the rung in scope on both
     assays (below), so n ≥ 16. It goes to phase 4 under run 2's rules.
   - **Empty set**: when an effect's pair set is empty, its excess and bound
     are not computed and the table shows `n = 0`.
   - **Rung scope** (a prospective scope restriction, chosen before data):
     the rung enters B on an assay only when |E| ≥ 16 and N departs in at
     most 0.25 |E| of E's pairs. Both conditions depend on R and N alone,
     never on an arm. Otherwise the rung is reported descriptively, and any
     B on that assay is described as reach-only.
   - **B (supported negative)**: for every arm, U < 0.25 with the masked
     counts, for reach on both assays and, with the lenient coding, for the
     rung on every assay in rung scope.
   - **Inconclusive**: any unresolved masking case (below) makes the arm
     `inconclusive` for B, which rules B out.
   - **Raw counts**: run 2's raw counts (arm-only reaches, departures) are
     reported beside every excess, with the strict and lenient rung codings
     and the unmasked and masked bounds.
   - **Masking** (per pair and effect, B only, before the bounds):
     - Rule 3's masking recomputation decides every fresh pair in which X's
       or R's final elite attempts reproduction on the development bank.
     - If X's higher abandoning copy reaches the threshold and R's does not,
       the pair becomes X-only for reach. Any N-only contribution of that
       pair is removed first, so b and c are recomputed once per pair.
     - If X's recomputed gain over its own elite exceeds R's by more than
       0.5 on a pair in E, the pair becomes X-only for the rung, in the same
       way. Pairs outside E carry no rung contrast for any arm.
     - A recomputation that fails to run, or an abandoning copy that cannot
       be built, is an unresolved masking case.
5. **Baseline identity.**
   - The pinned `instr:` build's first invocation in each new combination
     supplies its baseline: the reference rows and the summary projection.
   - Every later invocation of that combination must reproduce both:
     M5's (with all built-in arms) and the A/A build's.
6. **Confirmation (phase 4).** Run 2's rules, with bank 1 (seeds 101 and 102)
   as fresh 8-replicate campaigns of the candidate, the reference and the A/A
   control. The candidate is frozen, with its ledger row committed, before
   the bank is read.
   - A candidate confirms only when its frozen effect gives b − c ≥ 2 of 8
     against the A/A control on both assays. This is recomputed after the
     sterility counterfactuals. For a rung effect, all eight pairs must also
     be usable, or the confirmation is `inconclusive`.
   - The recruitment, competence and natural-world checks must then pass.
   - Failed or uncalibrated banks are spent and never reopened.
   - This overrides run 2's 16-replicate rule for confirmation only.
7. **Budget.** 400 turns from launch; closing starts by turn 360. Each
   invocation runs under the awake-time 15-minute kill. Probes and analysis
   tools may be built from the frozen sources; the campaign binaries do not
   change.

## Deliverable

`docs/strategy/evolvability-exploration-2026-10-run4.md` with summaries in
`docs/strategy/evolvability-exploration-2026-10-run4/`, in run 3's form.
