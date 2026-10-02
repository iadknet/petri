# Evolvability exploration: results (2026-10)

Status: complete; reviewed by Codex (two reviews, two advice rounds). Contract: the [run plan](evolvability-exploration-plan-2026-10-01.md)
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

| E5 | 2 | H4 (duplicate-and-diverge; analog: gene duplication then divergence) and H5 (targeting; analog: transcription-associated mutagenesis, here removed) let lineages leave the plateau | Two mutation-policy arms, both `policy-deviation`: `copy100` = `large_copy_weight_percent` 100 (production 25), `bias0` = `executed_bias` 0 (production 0.9); built-in reference and controls in the same invocation; pure mutation-policy changes, so evaluation semantics and the founder/comparator are unchanged | food-seeking as E1 (seed 1, 8 reps); reference rows must hash to the E1 baseline; 15 min | Predicted: no detectable paired difference in reach (0/8 each) and the same plateau. Falsified if either arm reaches ≥ 3/8 or its replicates' final bests exceed the reference's in ≥ 6 of 8 seeds. Still to discover by ordinary variation: a sensor-to-move contribution worth ≥ 1.7 bank points | completed (218 s) | Prediction held: 0/8 in both arms; final bests equal the reference's in 7 of 8 seeds (`copy100`, higher in 1) and 8 of 8 (`bias0`) |
| E6 | 2 | H3 / retention: given three times the generations, drift among silent plateau variants reaches a stepping stone | Reference and built-in controls only, `--generations 120` | food-seeking, seed 1, 8 reps, otherwise as E1; new combination, baseline is itself (no `proto:` or `instr:` commit exists); 15 min | Predicted: native still 0/8 and its training-scene bests no higher than at 40 generations in paired seeds. Falsified if native reaches ≥ 2/8 | completed (438.7 s by the run's own clock; the host slept during it, so the helper's elapsed 7,418 s is not run time) | Prediction held at the margin: native 1/8 (replicate 4, generation 50); native verdict now `stalls at benefit` (fail 6 of 6) |
| E7 | 1 | H3: the plateau is escaped through neutral stepping stones two edits away, not one; plus a check that the plateau is the sterility shortcut | Observation probe on E1's `native-1` plateau elite: ≤ 32 bank-silent (every scene zero-delta) genome-changed children × 16 grandchildren, against as many further direct children; founder and plateau bank totals of food eaten, moves and penalty charged | E3 bank (32 food-seeking scenes); 15 min | Predicted: grandchildren and direct children both ≈ 0 improvers (escape needs more than two edits), and the plateau's penalty charged ≈ 0 where the founder's is large. H3 supported if grandchildren improve in ≥ 5 while direct children ≤ 1; sterility diagnosis falsified if penalty charged is similar | completed (90.9 s) | Prediction held: grandchildren 1 of 512 improve (one by +6.40) against direct children 1 of 512 (+0.23); penalty charged founder 1,432 vs plateau 0.0 |
| E8 | 2 | Supply (analog: a mutator background, more blind variation per birth so edits can combine before selection removes intermediates; valley crossing, Weissman et al. 2009): a ×4 per-unit rate lets lineages leave the plateau | One mutation-policy arm, `rate4x` = `per_unit_rate` 0.02 (production 0.005; the founder's 0.485 requested events per birth become 1.94), `policy-deviation`; operators and targeting unchanged; built-in controls in the same invocation | food-seeking as E1 (seed 1, 8 reps); reference rows must hash to E1's; 15 min | Predicted: no detectable reach difference (0/8). Falsified if `rate4x` reaches ≥ 3/8. Ordinary variation still has to find the whole sensor-to-move path; raised supply is the intervention, not better per-edit evolvability | completed (210 s) | Prediction held: 2/8 vs 0/8 (two discordant pairs, exact paired p = 0.5); final best higher in 4, lower in 2, equal in 2 pairs; `stalls at benefit` (fail 3 of 3) |

## Frontier table

Descriptive only: paired difference from the reference arm, by seed, in
reached fraction and stall rung. It never ranks or retires an arm.

| Arm | food-seeking (sparse-food-v1), seed 1, 8 paired replicates | barrier-navigation (wall-v1), seed 1, 8 paired replicates |
| --- | --- | --- |
| native (reference) | reached 0/8; `inconclusive at retention` (fail 0 / inconclusive 5 of 5) | reached 0/8; `inconclusive at retention` (0 / 8 of 8) |
| shuffled-score (control) | 2/8 vs 0/8: two discordant pairs, no detectable difference (exact paired p = 0.5); `inconclusive at retention` | 0/8 vs 0/8; no detectable difference |
| `copy100` (H4) | 0/8 vs 0/8; no detectable difference (final best equal in 7 of 8 pairs, higher in 1); `inconclusive at retention` | not run |
| `bias0` (H5) | 0/8 vs 0/8; no detectable difference (final bests equal in 8 of 8); `inconclusive at retention` | not run |
| `rate4x` (supply ×4) | 2/8 vs 0/8; no detectable difference (exact paired p = 0.5); final best higher in 4, lower in 2, equal in 2 pairs; `stalls at benefit` (fail 3 of 3) | not run |
| native at 120 generations (E6; a horizon, not an arm) | 1/8; `stalls at benefit` (fail 6 of 6) | not run |

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
  genome-changed, of which 96 have a zero bank-mean delta and 65 do not: 23 improve (14 by more than one
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
  converges on one phenotype. No bank-mean improvement was observed among the 196
  sampled genome-changed children of the eight native elites (0 in each of
  the eight parents; children cluster within parents and reuse seeds across
  them, so no pooled bound is given); most are
  bank-silent (175 of 196 zero-delta) and the rest worse. The founder's
  children improve in 3 of 28; the shuffled-score elites, spread from 1.25
  to 18.61, in 10 of 267. Of the 13 improvers, 12 had every applied event
  targeted at a founder node and one at an added node. Target identity is
  descriptive: which tissue causally carries a gain was not ablated, so
  recruitment is unestablished either way. What the plateau is: native-1 differs from the founder
  by deleting node 0's `Multiply` of the energy and age thresholds, whose
  output fed `CustomOutput 1`, the upstream slot that tells node 1 to vote
  `Reproduce`; node 1 also re-points two upstream references. The plateau
  creature never attempts reproduction. In the lab reproduction is always
  refused (`min_reproduce_energy` above `max_energy`) and each refused
  attempt forfeits a foraging tick and pays the failed-action penalty, so
  the plateau is a **suspected sterility shortcut**: the most accessible
  gain in the food-seeking assay appears to be abandoning the reproduction
  drive, which in the world would end the lineage. E7 supports this by
  association only. Path length to a sensor-guided gain is unresolved.
- E5, E6, E8: no mechanism arm moved reach. Summaries:
  [`e5-arms-food-seed1.json`](evolvability-exploration-2026-10/e5-arms-food-seed1.json),
  [`e6-horizon-food-seed1.json`](evolvability-exploration-2026-10/e6-horizon-food-seed1.json),
  [`e8-rate4x-food-seed1.json`](evolvability-exploration-2026-10/e8-rate4x-food-seed1.json);
  overlays [`copy100.json`](evolvability-exploration-2026-10/copy100.json),
  [`bias0.json`](evolvability-exploration-2026-10/bias0.json),
  [`rate4x.json`](evolvability-exploration-2026-10/rate4x.json). In E5 and E8
  the reference arm's rows and summary projection hash to the E1 baseline
  (`48fcfe2c…`, `797814a3…`), so the reference is unchanged by adding arms.
  `bias0` changes almost nothing because on a two- or three-node genome every
  node executes, so executed-biased targeting has nothing to bias; H5 cannot
  be read at this genome size. `copy100` differs from the reference in one
  replicate. Four times the supply (E8) and three times the generations (E6)
  each reach 1–2 of 8, all within chance of the reference's 0/8, and turn the
  ladder verdict from `inconclusive at retention` into `stalls at benefit`:
  the longer a lineage sits on the plateau the more its touching children
  fail to improve (E6 native benefit `fail` in 6 of 8 replicates). That is
  the E4 plateau seen through the ladder.
- E7, two-step and sterility
  ([`e7-two-step-sterility.ndjson`](evolvability-exploration-2026-10/e7-two-step-sterility.ndjson)).
  On the 32 bank scenes the founder is charged 1,432 penalty energy and the
  plateau elite 0.0; the plateau eats 447 food against 283 and attempts 5,868
  moves against 3,124 (both die in most scenes, 32 and 27 of 32). Every
  refused `Reproduce` vote costs the founder a foraging tick and a penalty;
  the plateau casts none. This is association, not an isolated cause: the
  elite also re-points two upstream references, and no diagnostic genome
  with only the reproduce vote silenced was built. From the plateau, 32
  independent silent intermediates (zero delta on every bank scene) gave 512
  grandchildren, of which 1 improved, by +6.40 bank points, enough to clear
  the threshold; 512 matched direct children gave 1 improvement of +0.23.
  The prespecified H3 criterion (≥ 5 grandchild improvers with ≤ 1 direct)
  was not met. The counts are equal but the magnitudes are not (+6.40 against
  +0.23): the single large two-step gain is an unresolved stepping-stone
  lead whose ancestry was not analysed and which was not validated on fresh
  scenes, so it is not threshold reach.
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

## Hypotheses at close

| ID | Status after this run |
| --- | --- |
| H1 instrument limit | Split. Scene-sampling noise in four-scene paired selection: not the bottleneck in the sampled founder neighbourhood (E3). Instrument confound: a suspected sterility shortcut, supported by association (E4, E7), not isolated causally |
| H2 exposure | Food and barrier families only; social families need a multi-creature scene kind (finding, T22 candidate) |
| H3 stepping stones | Prespecified two-step criterion unmet (E7: 1 of 512 vs 1 of 512); one +6.40 two-step gain left as an unresolved lead |
| H4 incumbent masking / duplication | `copy100` no detectable difference (E5); 1 of 13 improvers had an event targeted at an added node (E4, descriptive) |
| H5 targeting | Not readable at two-to-three-node genome sizes (E5 `bias0` identical in 8 of 8) |
| H6 cost | Not run: lab lifetimes of 200 ticks make carrying cost negligible and replication cost is unused in a lab that refuses reproduction (Codex advice 0) |
| H7 step size | Not run: cut on Codex advice until edge contributions are measured |
| H8 node boundary | Unresolved: too few improvers on added tissue to test a boundary barrier; no prototype built |

Review 2 (Stages 1–2 and the draft recommendation, Codex `gpt-6.1-sol` high,
read-only, `.bench-artifacts/lab/exploration/codex/review-2*`):

| # | Finding | Severity | Disposition |
| --- | --- | --- | --- |
| 1 | Pooled 1.5 % bound on E4's 0/196 treats clustered children as independent | blocking | Accepted: bound removed, per-parent zeros reported |
| 2 | "No one-step improvement at all" overstates a sample of mutation applications | blocking | Accepted: reworded to "no improvement observed among the sampled children"; path length left unresolved |
| 3 | E7 "H3 not supported" hides the +6.40 vs +0.23 magnitudes | blocking | Accepted: criterion reported as unmet, the large gain kept as an unresolved lead |
| 4 | Sterility shortcut stated as established and dominant | blocking | Accepted: "suspected sterility shortcut", association only; silencing-only, reference-only and restoration diagnostics named as the next check |
| 5 | Score has no direct reproduction term, so "scored neutrally" is not a repair | blocking | Accepted: the candidate is stated as a behavioural requirement with unvalidated designs |
| 6 | Leftover "score identically" in E3 | blocking | Accepted: replaced with "zero bank-mean delta" |
| 7 | Target identity does not show which tissue carries a gain; H8 not excluded | advisory | Accepted: descriptive wording; H8 unresolved |
| 8 | Arms pass the five checks; built-in controls suffice | advisory | Accepted, no change |
| 9 | Holdouts sealed; frontier descriptive | advisory | Accepted; "no detectable difference" kept distinct from equivalence |
| 10 | "More exploration, no mechanism feature" follows | advisory | Accepted; instrument candidate presented as needing validation |

Advice rounds (not reviews): advice 0 redirected Stage 0 to the fixed-pool
selection audit (E3) and cut H6/H7; advice 2 replaced a second duplication
cycle with the supply arm (E8) and asked for the sterility counts (E7).
Review 1 and review 2 cover the end of every stage the run reached; review 2
was also the "after the fourth cycle" review, run after cycle 8 because of
the turn budget (a deviation from the plan's cadence, recorded here).

## Recommendation

**More exploration, after one T22 instrument repair; no roadmap mechanism
feature.** No arm moved reach beyond chance and no recruitment was seen, so
nothing qualifies as a mechanism feature and the natural-world check was not
run. The run's main result is about the instrument: the food-seeking assay's
cheapest selectable gain is to stop attempting reproduction, which the lab
always refuses at a cost (E4, E7), and no bank-mean improvement was observed among the 196 sampled children of
the eight native elites (E4). Until the
assay cannot be improved by abandoning reproduction, food-seeking reach and
benefit readings mix sterility with foraging.

1. **T22 feature candidate (instrument), unvalidated:** a food-seeking
   instrument version whose behavioural requirement is that abandoning
   reproduction cannot raise the score. The current score has no direct
   reproduction term; the effect is indirect (lost foraging ticks, penalty
   energy, survival), so candidate designs (for example a refused attempt
   that costs neither the tick nor the penalty) must be checked against
   that requirement with diagnostic genomes that silence only the reproduce
   vote, only re-point the references, or restore the signal, kept outside
   evolving lineages. Built
   under the new-instrument rules (new assay or scorer name, frozen version,
   development set and sealed acceptance set, lab-authored comparator only if
   needed) and calibrated before any arm runs on it. Its first reading
   should repeat E3, E4 and E7 on the founder. The same check applies to
   `wall-v1` (E2's native gains were not decomposed here).
2. **Then more exploration, in this order:** the elite-neighbourhood audit
   (E4) on the repaired instrument; a multi-step path census from the
   founder to a sensor-to-move contribution worth the threshold (how many
   coordinated edits, through which operators), which is what H3, H4 and H8
   need; the `rate4x` and horizon arms repeated with 16 replicates if the
   plateau persists.
3. **T22 feature candidate (instrument):** a multi-creature scene kind, so
   social sensor families can be exposed without crediting a companion's
   counters to the focal genome.

No `instr:` or `proto:` commit was made, so none is listed as a candidate.
The holdout seeds (101 food, 102 wall) were reserved and never read: no
candidate was frozen.
