# Evolvability exploration, run 3: plan

Date: 2026-10-02. Source: `5a9b279f`. Status: plan for the continuation of
[run 2](evolvability-exploration-2026-10-run2.md) (outcome C), under the
[T22 exploration contract](../roadmaps/t22-capability-assays-and-evolvability-lab.md).
No experiment run yet. Reviewed by Codex `gpt-6.1-sol` `high` on 2026-10-02
in five rounds:

- Round 1: `not-ready`, 5 blocking and 3 advisory findings.
- Round 2: confirmed 7 fixes; attribution stayed open and a new blocker
  was raised on rung recomputation.
- Rounds 3 and 4: the masking and ancestry rules.
- Round 5: verdict `ready`.

Briefs and outputs are under `.bench-artifacts/lab/exploration/codex/run3-plan-review-*`.

## Why a third run

Run 2 found that run 1's food-seeking plateau was mostly a sterility
shortcut: the founder votes only `Reproduce` once its energy passes 32, the
lab refuses and charges every attempt, and the founder stands still. Silencing
the readiness signal recovers 92 % of the plateau's gain. No tested repair
kept the score unchanged for genomes that still try to reproduce, so run 2
closed with outcome C. Of run 2's unblock options the user chose the
hunger-regime food instrument (2026-10-02) and asked the run to make obvious
choices itself and run unattended.

## Question

Unchanged from runs 1 and 2: why do newly added nodes and sensor reads rarely
become useful under Petri's own variation, and which general changes to the
representation, operators, costs or selection setting let them become useful?

## Rules

[Run 2's plan](evolvability-exploration-plan-2026-10-02.md) governs everything
this section does not change, and through it [run 1's](evolvability-exploration-plan-2026-10-01.md).
Changes for run 3:

1. **Hunger-regime food instrument** (user decision, 2026-10-02). A new assay
   name, `food-seeking-hunger`: `sparse-food-v1` scenes and geometry, the
   `BITES_AND_PROGRESS` scorer, the built-in `area_food` comparator, and the
   unchanged calibration gate, margin and grid rule. Every evaluated actor
   (genomes, the comparator and the scripted instruments) starts at the
   production `initial_energy` (20) and has its energy capped at 20 at every
   living tick boundary. For this instrument only, this amends run 1's
   "evaluation start unchanged" rule and run 2's live-state boundary. The
   mutation engine, lab selection, start tick, lifetime (200 ticks) and
   focal-only counters stay unchanged. It is built from `instr:` commits
   (run 2's `0328335e` plus the assay variant), and its frozen descriptor is
   committed before its calibration gate first runs. It draws a fresh
   development set and a fresh sealed acceptance set; run 2's `0xACCEF00D`
   bank is spent. It is a new instrument with its own cap of two versions.
2. **Changed behavioural requirement** (user decision, 2026-10-02). Abandoning
   reproduction is never better than keeping the founder's reproduction
   circuit. It is tested on a fixed panel of diagnostic genomes (run 2 rule
   7), and the panel certifies the instrument only; whether evolving lineages
   gain by sterility is rule 3's question.
   - *A base genome* `b` keeps its reproduction circuit as it evolved or
     was built: node 0's readiness output into node 1's readiness read and
     its `Reproduce` votes.
   - *An activating edit* makes reproduction fire under the ceiling: node
     0's energy-gate threshold (compute node 0, a `Threshold` on the
     `EnergyCurrent` leaf, as in the founder) lowered to 0.08 or 0.05, or
     that gate's input weight raised to 2.0 (the gate then opens at 16
     energy). It applies only to a base whose node 0 carries that gate.
   - *An abandoning edit* removes reproduction:
     - silencing (run 2's: node 0 `CustomOutput(1)` inputs cleared);
     - the age gate raised above the lifetime (threshold 0.5);
     - node 1's `Reproduce` vote sinks cleared;
     - the reproduction structure removed (node 0's two gates and its
       `Multiply`, and the `CustomOutput(1)` edge).

     The age-gate and vote edits are applied to an activated copy, the other
     two to the base and to activated copies.
   - The requirement is directional: for every base `b` and every genome `a`
     made from `b` by abandoning edits (with or without activating edits
     first), score(`a`) − score(`b`) ≤ m, with m = 0.5 bank-mean points.
     Abandoning is never better than keeping the base's own circuit.
     Absolute differences are reported beside it, and so are the activated
     copies' own losses.
   - It must hold first on the development set, then on the sealed
     acceptance set after the freeze. A version failing either is
     `uncalibrated`.

   The food bases are the founder, the comparator, run 2's restored plateau
   and `shuffled-score-5`. Every edit is applied to the founder and the
   comparator, and silencing alone to the other two.
3. **Residual confound: a sterility-matched contrast.** The hunger
   instrument does not remove the shortcut from a descendant whose
   reproduction becomes active again (one edit can lower the gate below 0.1
   of `max_energy`); it makes that activation cost score.
   - Every campaign on either repaired assay reports, per pair and arm, for
     the final elites on the instrument's development bank: reproduce
     attempts per tick, penalty charged, deaths, food eaten, and the
     elite's silencing delta. The rows' per-generation best-scene penalties
     are reported as an ancestry trace.
   - The sterility counterfactual is elite-level. For a reach or score
     contrast that would carry an A candidate, the paired elites' scores are
     recomputed with two abandoning copies of each: silencing (when node 0
     carries a readiness output into node 1's readiness read), and every
     node's `Reproduce` vote sinks cleared. Each contrast is recomputed on
     both copies, and the gain must survive the smaller of the two. A gain
     that falls below the minimum effect is attributed to reproduction
     differences, not recruitment, and cannot support A.
   - An elite that still attempts reproduction on the development bank
     after both copies, or whose reproduction wiring fits neither copy, is
     `attribution-unresolved`; its gain cannot support A until phase 4's
     ablations resolve it. Phase 4 repeats the counterfactual on the holdout
     bank.
   - Ladder rungs come from campaign-wide counts, which elite rescoring
     cannot reconstruct. Rung contrasts are therefore reported with this
     rule's descriptive readings only. A rung movement that would carry an A
     candidate stays `inconclusive` for attribution unless that candidate's
     elites pass the counterfactual and phase 4's ablations.
   - *Masking, for outcome B.* Reproduction costs can hide a gain. For every
     pair in which either final elite attempts reproduction on the
     development bank, both elites' validation means are recomputed on both
     abandoning copies, taking each elite's higher copy.
     - If the arm's recomputed value reaches the threshold and the
       reference's does not, the pair counts as an arm-only reach in B's
       bound.
     - If the arm's recomputed gain over its own elite exceeds the
       reference's by more than m, the pair also counts as the arm leaving
       the reference's stall rung, because rungs cannot be recomputed.

     Both counts are conservative.
   - *Ancestry is outside claim A.* Run 1's recruitment claim is about the
     final elite: its acquired node or read is identified, ablating it
     removes the gain, and incumbent competence is preserved. Run 3's A
     claims add the elite-level counterfactual above and make no claim about
     a lineage's history, which may include reproduction-related steps. Each
     A claim says its history was not attributed. The rows' per-generation
     best-scene penalties are reported as a descriptive ancestry trace.
   - These readings never reject, retry or reweight an offspring.
4. **`wall-v1` kept, requirement 2 checked on it.**
   - The wall bases are the founder, the barrier comparator, and run 1's E2
     wall elites `native-0`, `native-1` and `shuffled-score-1`. Every edit is
     applied to the founder and the barrier comparator, and silencing alone
     to the elites.
   - It is checked first on run 2's wall development bank (`0xB2DE00`),
     then on a fresh sealed wall bank, read once.
   - If it fails, the fallback is `barrier-navigation-hunger`: `wall-v1`
     geometry at scale 2, barrier scoring and the barrier comparator
     unchanged, with only rule 1's energy regime. It goes through the full
     procedure before phase 2: frozen descriptor, development validation,
     calibration, sealed acceptance and baseline. Passing requirement 2
     alone does not qualify it. If it fails, the run records outcome C.
5. **The two repaired assays** of run 2's outcomes A and B are
   `food-seeking-hunger` and `wall-v1` (or `barrier-navigation-hunger`).
   Their baselines are taken under run 1's new-instrument rules: the `lab:`
   and `instr:` commits applied to the launch revision, with no `proto:`
   commit.
6. **Holdout banks.** Run 2's banks are still sealed and are reused
   unchanged: bank 1 is seeds 101 (food) and 102 (barrier), bank 2 is 111
   and 112, and bank 3 is 121 and 122.
7. **Seeds and scene identity.** The lab derives calibration and validation
   scenes from the campaign seed, so seeds 1 and 2 would repeat geometry run
   2 has already read. Run 3's paired campaigns use seed batches 3 and 4.
   The descriptor records every scene bank's seed, count and content hash,
   each validation genome's hash, the energy semantics and the freeze
   commit. Every development variant evaluated counts as a version under
   the two-version cap.
8. **Pilots set sizes by adequacy, not only runtime.**
   - Candidate sizes, tried in order: quick (population 16, 40
     generations); 16 and 100; 32 and 100; the campaign default (64 and
     100); retention depth 2 throughout.
   - The first size at which the reference arm reads its stall rung with
     adequacy in all 16 replicates on both assays, with every invocation
     under 900 s (arms split across invocations as needed), is frozen
     before any arm contrast is read.
   - Rung movement means: at the reference replicate's stall rung, the
     reference reads `fail` with adequacy and the paired arm replicate reads
     `pass` with adequacy. A pair with an inconclusive read at that rung is
     not usable.
   - If no candidate size reads adequately, the run reports reach-only
     contrasts and records outcome B as unavailable.
9. **Descriptive readings beside every contrast.** Paired differences in
   final best, validation mean, food eaten and deaths are reported
   descriptively. Outcome B excludes only the predeclared effects at the
   frozen sizes; it is not a claim that a mechanism does not help.
   Thresholds are never changed after arm results are read.
10. **Unattended run** (user decision). The run starts in the session that
   wrote this plan, after the plan's Codex review, and makes every choice
   that is not on the stop-and-ask list itself, recording each one in the
   ledger. The budget is 400 turns from the run's launch, and closing
   starts by turn 360.

## Phases

1. **Instrument.**
   - Build `food-seeking-hunger` and freeze its descriptor.
   - Check requirement 2 on its development set, then run its calibration
     gate on seeds 3 and 4, then read its sealed acceptance set.
   - Check requirement 2 on `wall-v1` (rule 4).
   - Take the baselines.
   - Rerun run 1's E3, E4 and E7 on the new instrument.
2. **Path census**, as in run 2's plan, on the two repaired assays. Its
   predeclared rule picks M5, and one witness path is built per assay.
3. **Mechanism arms M1–M5**, as in run 2's plan. A pilot sets the sizes
   (rule 8). Each arm then runs 16 paired replicates (seed batches 3 and 4)
   on both assays, with rule 3's readings and rule 9's descriptive
   readings. Each arm's ledger row names the sensing, connection and action
   changes still left to ordinary variation, and no witness path or
   diagnostic genome seeds or constrains an arm.
4. **Confirm**, as in run 2's plan.

## Deliverable

The results note `docs/strategy/evolvability-exploration-2026-10-run3.md`
follows run 2's form. Its compact summaries go in
`docs/strategy/evolvability-exploration-2026-10-run3/`. The note ends with the
evidence outcome (A, B or C), the execution status, and a resume handoff if
the run is incomplete.
