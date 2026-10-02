# Evolvability exploration: results (2026-10)

Status: in progress. Contract: the [run plan](evolvability-exploration-plan-2026-10-01.md)
under the [T22 exploration contract](../roadmaps/t22-capability-assays-and-evolvability-lab.md).
Launch revision `ce40054dca9d68e0286755074a772a7cd510e5c1` (clean `main`, 2026-10-01).
Experiment branch `worktree-evolvability-exploration` (never merges). Compact
summaries cited below live in [`evolvability-exploration-2026-10/`](evolvability-exploration-2026-10/);
raw output under `.bench-artifacts/lab/exploration/` in the main checkout.

## Question

Why do newly added nodes and sensor reads rarely become useful under Petri's
own variation, and which general changes to the representation, operators,
costs or selection setting let them become useful? The run builds conditions
under which a capability can evolve; it never makes creatures use a
particular sensor, node or action.

## Ledger

One row per experiment, predeclared before its run; `incomplete` rows kept.

| ID | Stage | Hypothesis | Arms and controls | Assay, seeds, sizes | Prediction / falsifier | Status | Result |
| --- | --- | --- | --- | --- | --- | --- | --- |
| E1 | 0 | H1: low benefit rate is partly an instrument limit (few replicates) | Built-in arms only: native reference, founder-only, mutation-off, shuffled-score, comparator, random walk | food-seeking, sparse-food-v1, seed 1, `--quick --replicates 8 --food-fraction 0.04 --lifetime 200` (8 × 40 gen × 16); wall budget 15 min | Predicted: native benefit `pass` in most replicates, retention `inconclusive`; falsified if native stalls at benefit in ≥ 5 of 8 | completed (144 s) | See E1 below |
| E2 | 0 | H1, H2: same reading on the one calibrated non-food family (barrier) | As E1 | barrier-navigation, wall-v1, seed 1, `--quick --replicates 8 --scale 2 --lifetime 200`; 15 min | As E1 | completed (73 s) | See E2 below |
| E3 | 0 | H1: 4-scene truncation selection mostly tracks scene-sampling noise, not genotype benefit | Observation probe, no arms: 8 pools × 64 founder children (production engine, founder's last-scene record), each scored on two independent 4-scene batches and a 32-scene bank; truncation (top 16 of 64 on batch A) against a fixed pseudo-random 16 | food-seeking scenes (64², 0.04, 200 ticks); seeds in `crates/v3-lab/tests/exploration_probe.rs`; 15 min | Predicted: corr(4-scene Δ, bank Δ) < 0.3 and truncation's bank gain within noise of random. Falsified if corr ≥ 0.5 and truncation gain clearly above random in ≥ 6 of 8 pools | completed (9 s) | Falsified: corr 0.94; truncation above random in 8 of 8 pools |
| E4 | 1 | H3/H4/H8: gains stop accumulating because the elites' beneficial neighbourhood collapses after the first steps, and what improvements remain are tuning of the incumbent founder nodes, not added nodes | Observation probe: founder, E1's 8 native and 8 shuffled-score final elites; 96 production children each, scored against the parent on the E3 32-scene bank; improvers tallied by applied operator and target node (founder ids 0–1 vs added) | food-seeking scenes as E3; elites from E1 (seed 1); 15 min | Predicted: native elites' improved share of genome-changed children below half the founder's, and ≥ 80 % of improvers target founder nodes only. Falsified if native elites keep ≥ the founder's improved share (then horizon/retention, not neighbourhood, is the limit) | completed (≈ 40 s) | Prediction held: native elites 0 improvers of 196 changed children (founder 3 of 28); 12 of 13 improvers anywhere touch founder nodes only |

## Frontier table

Descriptive only: paired difference from the reference arm, by seed, in
reached fraction and stall rung. It never ranks or retires an arm.

| Arm | food-seeking (sparse-food-v1) | barrier-navigation (wall-v1) |
| --- | --- | --- |

## Findings by rung and family

**Baseline and Stage 0 (E1–E3).** Summaries: [`e1-food-seed1.json`](evolvability-exploration-2026-10/e1-food-seed1.json),
[`e2-wall-seed1.json`](evolvability-exploration-2026-10/e2-wall-seed1.json),
[`e3-selection-audit.json`](evolvability-exploration-2026-10/e3-selection-audit.json);
baseline hashes in [`baseline-hashes.json`](evolvability-exploration-2026-10/baseline-hashes.json).

- E1, food-seeking, 8 replicates. Reached: native 0/8, shuffled-score 2/8,
  founder-only and mutation-off 0/8 (threshold 15.956; founder 10.65,
  comparator 28.54, floor 3.38 at calibration). Native final bests on the
  training scenes 4.75–20.50 (mean 13.5) against founder-only 7.43–12.06.
  Native benefit rung: `pass` in 5, `fail` in 2, `inconclusive` in 1
  replicate; pooled 34 improved of 630 viable touching children (5.4 %).
  Shuffled-score's pooled rate is 102 of 684 (14.9 %); a likely but untested
  reason is that a randomly retained parent is often a weak one, so
  "improved over parent" is easier, and the rung is not comparable across
  selection regimes. The reach contrast (2/8 against 0/8) has two discordant
  pairs (exact paired p = 0.5): chance, drift through stepping stones and
  truncation's loss of diversity all remain open.
  Retention is `inconclusive` in 8 of 8 replicates in every evolving arm
  (2–10 selected improvements per replicate against the minimum of 15), so
  quick sizes cannot read retention at all. Arm verdict: native
  `inconclusive at retention`.
- E2, wall-v1, 8 replicates. Reached 0/8 in every evolving arm (threshold
  5.42; founder 0.61, comparator 10.42, floor 0.42). Native final bests
  −0.24–7.41 (mean 3.2) against founder-only −0.71–4.39; native benefit
  `pass` in 8 of 8 (66 of 804, 8.2 %); retention `inconclusive` 8 of 8.
  Shuffled-score's best fell to −0.73–1.59: on wall-v1 selection, not drift,
  carries the gains.
- E3, selection audit on 512 founder children. 351 identical (68.6 %), 161
  genome-changed, of which 96 score identically on all 32 bank scenes
  (silent on the assay) and 65 change it: 23 improve (14 by more than one
  point) and 42 get worse. A paired 4-scene delta predicts the 32-scene delta
  (Pearson 0.94; two independent 4-scene batches 0.86). Truncation on one
  batch gains +0.75 bank points over a fixed pseudo-random pick, ahead in 8 of
  8 pools. Founder means on single 4-scene batches range 5.1–14.0, so
  absolute scores are noisy but paired differences are not. Per pool (the
  replicate unit; children within a pool are clustered) the correlation over
  changed children is 0.86–0.99 and truncation beats the pool mean in 8 of 8
  pools (+0.08 to +1.07); pool 1 is the exception once losses below −1 are
  dropped (correlation −0.03, 1 of 5 batch-A gains confirmed on the bank).
  The audit is simplified: it freezes the parent's record on batch A and
  selects 16 of 64 siblings, where the campaign breeds 4 of 16 with elite
  carry-over from the previous generation's scenes. Read narrowly: **in the
  sampled founder neighbourhood, four-scene paired selection carries signal**;
  rankings among evolved, near-threshold lineages are not tested here. 96 of
  the 161 have a zero bank-mean delta (per-scene silence was not recorded),
  and 23 of 161 genome-changed children improve the bank mean (4.5 % of all
  births). The open question moves from
  "are improvements found" to "why do they not accumulate to the threshold"
  (H3 stepping stones, H4 masking, retention).
- E4, elite neighbourhoods
  ([`e4-elite-neighbourhoods.ndjson`](evolvability-exploration-2026-10/e4-elite-neighbourhoods.ndjson)).
  Six of E1's eight native elites score exactly 14.30 on the 32-scene bank
  (the founder 9.10; the two others 11.63 and 12.55): native selection
  converges on one phenotype. None of their 196 genome-changed children
  improves on it (one-sided 95 % upper bound about 1.5 %); most are
  bank-silent (175 of 196 zero-delta) and the rest worse. The founder's
  children improve in 3 of 28; the shuffled-score elites, spread from 1.25
  to 18.61, in 10 of 267. Of the 13 improvers, 12 touch founder nodes only
  and one an added node: **added nodes essentially never carry an
  improvement here.** What the plateau is: native-1 differs from the founder
  by deleting node 0's `Multiply` of the energy and age thresholds, whose
  output fed `CustomOutput 1`, the upstream slot that tells node 1 to vote
  `Reproduce`; node 1 also re-points two upstream references. The plateau
  creature never attempts reproduction. In the lab reproduction is always
  refused (`min_reproduce_energy` above `max_energy`) and each refused
  attempt forfeits a foraging tick and pays the failed-action penalty, so
  **the most accessible "improvement" in the food-seeking assay is deleting
  the reproduction drive**, which in the world would end the lineage. This is
  a benefit-rung instrument confound, not food seeking. Beyond that one step
  the founder architecture has no one-step improvement at all: sensor-guided
  movement is several coordinated edits away (H3).
- H2 exposure. The ladder's relevant families are `FoodHere`,
  `NeighborFoodRing` and `AreaFoodSummary` (food-seeking) plus the barrier
  ring and area summary (barrier navigation). Occupancy, the three
  nearby-creature banks and `ActionVotes`-style decision inputs are never
  graded: occupancy and nearby-creature families cannot be exposed by a solo
  evaluation (social families; a multi-creature scene kind is a T22 feature
  candidate, out of scope here because `eval.rs` credits simulation-wide
  counters to the evaluated genome). Energy and age introspection are read
  by the founder in every scene but not scored as a capability. No new
  family instrument was built in Stage 0: the E3 result redirected the
  budget to accumulation.

## Review dispositions

Review 1 (Stage 0, Codex `gpt-6.1-sol` high, read-only, brief and output under
`.bench-artifacts/lab/exploration/codex/review-1*`):

| # | Finding | Severity | Disposition |
| --- | --- | --- | --- |
| 1 | "H1's noise form is killed" exceeds a one-step founder audit | blocking | Accepted: narrowed to "four-scene paired selection carries signal in the sampled founder neighbourhood"; H1 stays open for evolved lineages |
| 2 | E3 freezes the record on the batch it scores, unlike the campaign | blocking | Accepted: E3 labelled a simplified audit; E4 freezes on a separate batch and scores on the bank |
| 3 | Pooled correlation dominated by losses; pools are the unit | blocking | Accepted: per-pool correlations, positive-tail agreement and loss-trimmed correlations added (pool 1 does not hold once losses are trimmed) |
| 4 | Stride subset is not a random draw | advisory | Accepted: truncation compared with the exact pool mean instead (8 of 8 pools ahead) |
| 5 | "Score identically" only checked the mean | blocking | Accepted: reworded to "zero bank-mean delta" |
| 6 | Shuffled-score benefit explanation untested; 2/8 vs 0/8 is p = 0.5 | advisory | Accepted: explanation qualified, exact paired p stated |
| 7 | Empty frontier; E3 provenance thin | advisory | Accepted: frontier populated after the Stage 2 arms; E3 provenance names commit c6796b04 and its seeds |

## Recommendation

Pending.
