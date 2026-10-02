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
  Shuffled-score's pooled rate is 102 of 684 (14.9 %), because a randomly
  retained parent is often a weak one, so "improved over parent" is easier; the
  benefit rung is therefore not comparable across selection regimes.
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
  absolute scores are noisy but paired differences are not. **H1's noise
  form is killed**: selection on 4 shared scenes is not the bottleneck, and
  the founder's one-step beneficial neighbourhood is not sparse (4.5 % of
  all births, 14 % of genome-changed ones). The open question moves from
  "are improvements found" to "why do they not accumulate to the threshold"
  (H3 stepping stones, H4 masking, retention).
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

## Recommendation

Pending.
