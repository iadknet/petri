# T13.F06 — Recruitment and Retention Qualification

**Status**: Complete
**Last updated**: 2026-09-14
**Feature**: T13.F06
**Track**: [T13 — Neutral Module Recruitment](../../roadmaps/t13-neutral-module-recruitment.md)

## Goal

On the integrated substrate (T13.F03–F05 repairs, T03.F08 carrying cost,
T03.F10 execution cost), the stored goal report records whether newly recruited
mesh modules are discovered and retained because they help, at the production
price of carrying and running them: the fixed T13.F02 comparison re-read
unchanged beside a third, cost-bearing selection policy on the same nine
starting forms, with probability and time to first useful contribution,
retention over the declared horizon, loss and censoring, damage, cost and
paired task performance on the unchanged task (A) and the changed task (B),
then the ordinary goal-profile ecological reading kept apart from assay
selection. It is observation only: nothing reaches creatures.

## Non-Goals

- No change to mutation rates, operator weights, targeting, activation,
  recruitment operators, runtime, founders, defaults or the T13.F02 task,
  scenes, seeds, arms, margins or horizon. A stochastic null is a measured
  limit; an implementation failure surfaced here is reported against the
  owning feature (T13.F03/F04/F05, T03.F08/F10), not repaired here.
- No energy-to-score conversion, task bonus, reward, multi-tick task, or
  authored fitness; no probabilistic selection.
- No ecological cohort tracking, genealogy or new instrument in the goal
  worlds; no dashboard, database, profile, campaign runner or sweep.
- No supply-policy comparison (T11.F13), memory-motif reading (T11.F10),
  cognition or emergence claim, and no reinterpretation of historical reports,
  floors or accepted exceptions.

## Inputs and Invariants

- Owning row: T13.F06; the track's **F06 scope** note defines acceptance.
  [T13.F05](t13-f05-function-preserving-module-recruitment.md) supplies the
  qualified paths and their maintained tests in
  `crates/v3-core/src/neighborhood/recruitment_paths/` (seven of nine forms in
  at most six events; `graph_blank`/`vm_blank` are seven-event growth gaps).
  [T03.F08](t03-f08-genome-size-maintenance-cost.md) supplies the carrying
  charge (`genome_carry_cost_per_unit` 1e-4 per `genome_size()` unit per
  tick, Phase 0, beside the 0.5 decay). T03.F10 supplies the ramped per-step
  execution charge. [T13.F02](t13-f02-recruitment-paths-and-replicated-baseline.md)
  supplies, unchanged, Task A and B, the eight one-tick scenes at energy 50,
  task-live, the 1/8 margin, usefulness by `static_successor_bypass`, the
  nine frozen starting forms, the seed formula, `Sizes::PRODUCTION`
  (4 × 8 lineages, 32 + 16 generations, two siblings), the drift and
  selection policies, the retention classification and the pairing rule.
- Code seams (read 2026-09-14): `recruitment_paths::observe` builds the arms
  (`experiment.rs`), `records::choose` is the selection rule,
  `TaskSummary.ending_energy_sum` / `carrying_sum` / `work` already carry
  every production charge of the eight scenes (`mod.rs::evaluate_with_config`
  reads `energy_flows.genome_carrying` and `lifecycle_decay` per scene), and
  `crates/v3-cli/src/bench/artifacts.rs` folds each arm's `summary`,
  `batches`, per-lineage discovery/retention rows and `outcome_counts` into
  the committed goal summary. `Sizes::proposals` multiplies by the arm count.
- Current readings the comparison is read against, from
  `docs/progress/features/t13-f05-function-preserving-module-recruitment-goal.json`
  (`deterministic.goal_indicators.recruitment_paths`): every Task A arm and
  both unprepared Task B arms have 0/32 proposal and retained discovery under
  drift and selection; `graph_prepared` reads 17/13/2 (proposal / retained /
  useful at +16) under drift and 17/17/16 under selection, `vm_prepared`
  18/14/1 and 18/18/15; experiment wall 8,315 ms. Starting forms carry 14–34
  `genome_size()` units, so the carrying charge is 0.0014–0.0034 per scene
  (0.3–0.7% of decay) and the summed ending energy is about 395.6 of 400.
- Research, local evidence only (the [recruitment research note](../../strategy/neutral-module-recruitment-research-2026-09-08.md),
  reviewed 2026-09-08, fixed the direction: extend the existing assay seams,
  no separate simulator or selection in historical drift). Options weighed
  for the cost-bearing reading: (1) a third selection policy inside
  `observe` that orders candidates by task score first and then by the
  summed ending energy the production charges already leave; (2) a multi-tick
  energy assay in which the mover eats — rejected: it replaces the fixed
  comparison and the Task A incumbent never eats, so energy would reward
  standing still; (3) a probabilistic selection coefficient from the charge —
  rejected: it needs an authored conversion and RNG variance the four-batch
  design cannot absorb. Option 1 wins: it uses only production charges, is
  deterministic, keeps the eighteen F02 arms byte-identical, and brackets the
  production case between cost-invisible (F02 selection) and cost-always-
  visible selection; the remaining tradeoff is that a strict tie-break is the
  infinite-population limit, so its readings are a bound, not an ecology
  estimate. Selection against slightly deleterious carried sequence is the
  deletional-bias reading T03.F08 cites (Mira, Ochman and Moran 2001).

**Cost-bearing policy (fixed before measurement).**

| Element | Definition |
| --- | --- |
| Name | `Policy::CostSelection`, third value beside `Drift` and `Selection`; run for all nine starting forms, 27 arms. |
| Candidates | The parent and each task-live child whose score on the arm's task is at least the parent's, exactly as `Selection`. |
| Order | Highest score wins. At equal score the candidate with the strictly higher `ending_energy_sum` (f64 sum of the eight scenes' ending energies, all starting at 50) wins. Remaining ties follow F02's order: sibling 0, then sibling 1, then parent. |
| Consequence | A neutral child that carries or runs more than its parent is not retained; a neutral child that sheds units is. A child whose score rises is retained whatever it costs, as long as it is task-live. |
| Charges seen | Everything the eight production ticks subtract: decay, carrying (T03.F08), ramped execution (T03.F10), action costs. Recorded as a limitation: at equal score, differing wrong-scene action patterns also move ending energy. |
| Seeds and RNG | Same proposal seed formula and generation index as the other two policies; the arm reuses each lineage's seeds and is paired with them. Observation consumes no mutation RNG. |
| Unchanged | Discovery, usefulness (bypass loss at least 1/8, active score at least 1/8 above the starting form, task-live), the retention classification at discovery + 16, checkpoints at 0/32/48, and the eighteen existing arms, whose per-proposal records, fingerprints, `outcome_counts` and every F02 estimate (numerators, denominators, intervals, depth range) equal the previous closure's; their `summary` and `batches` blocks gain only the new keys below, and `Selection` never reads energy. |

**Readings added to the report (per arm, in `Summary` and the committed
summary).**

| Reading | Definition |
| --- | --- |
| Time to first useful contribution | Sorted first retained-discovery generations of the 32 lineages, their median, and the count censored at generation 32 without discovery; the same for first proposal discovery. Existing `discovery_depth_range` stays. |
| Retention and loss | Counts of `Useful` / `NoLongerUseful` / `Deleted` / `TaskDead` at discovery + 16 among discoverers, beside the existing Wilson estimates; every non-discoverer stays in the 32-lineage denominator. Retention never censors: discovery is bounded by 32 and follow-up ends at 48. |
| Damage | Fraction of proposals that are task-dead, and fraction of task-live proposals that lose at least 1/8 against their parent. |
| Cost | Retained parent's `genome_size`, module count, `carrying_sum` and `ending_energy_sum` at checkpoints 0, 32 and 48 (min / median / max over lineages). |
| Paired performance | The existing pairing loop already pairs every same-task arm, so each start's cost arm is paired with its drift and selection arms with no new rule; pairs grow from 73 to 171 (15 Task A arms, 12 Task B arms), each keeping its 32 signed discovery / useful-retention differences and first divergence. The `pairs` block is copied whole into the committed summary and is the summary's growth source. |

**Fixture cost reading.** The maintained qualified-path tests gain, per path
and per step, the `CostSelection` verdict of that step against its predecessor
(retained / rejected) and the step's carrying charge and ending energy, and
assert that the last step (the useful one) is retained. That assertion is a
regression guard against energy ever overriding score, not evidence: a useful
step raises the score by definition. The count of neutral steps a
cost-visible selection rejects, by form and backend, is the barrier reading;
it is recorded, not scored.

**Ecological reading (no new instrument).** From the ordinary goal report per
world: `energy_flows.genome_carrying` against `lifecycle_decay`, mean carried
units per creature-tick derived as `genome_carrying / (1e-4 × creature_ticks)`,
`reachable_structure_size_distribution`, the evolved `mesh_summary`
(executed / knockout / reachable / total), and the drift walk's `recruitment`
rungs, retention and `time_to_first` rows. The drift walk has no energy and
no selection; the evolved population has both but no module attribution. The
readings file states both limits above the table.

**Invariants.** Mutation, runtime and simulation never depend on observation
types; the six normalized counters, founder neighborhood, evolved trajectories
and drift walk are byte-identical to T13.F05's; the experiment stays inside
T13.F02's 120 s wall cap; `Sizes::proposals` counts 27 arms (82,944 at
production; the test pinning 55,296 is re-pinned with that reason) and still
equals `total_proposals`; the conversion's fixed claim pointers keep meaning
(`arms/0` is still `graph_blank` / `Drift`); the summary file grows only by
the nine arms, the 98 added pairs and the new `summary` / `batches` keys on
all 27 arms. `CheckpointCost.modules` is `genome.nodes.len()`, as
`Proposal.modules` is; an even-count median is the mean of the two middle
values; the new `Summary` fields carry no `serde(default)`, since the
committed summaries are read untyped and older reports are not re-loaded
typed.

## Implementation Tasks

- [x] Add `Policy::CostSelection` and extend `choose` to take each
      candidate's `ending_energy_sum`; property tests (proptest) that the rule
      never retains a task-dead or lower-scoring child, never prefers a
      strictly lower ending energy at equal score, and reduces to F02's
      `Selection` choice when energies tie.
- [x] Run the third policy for every starting form in `observe`; keep the
      eighteen existing arms first in order; extend `Summary` with the
      time-to-first, retention-outcome, damage and checkpoint-cost readings;
      fold them through `bench/artifacts.rs` into the committed summary.
- [x] Extend the qualified-path tests with the per-step cost verdicts and the
      last-step assertion (`QualifiedPath::cost_verdicts`).
- [x] Add the readings file and the `docs/progress.md` row, and tick the
      track's qualification success criterion at closure. No reference doc
      describes this observation, so none changes.

## Verification

- [x] `cargo test -p v3-core recruitment_paths` and the property tests ->
      pass (32 passed in the crate plus 3 in the integration binaries,
      2026-09-14), including the pinned-baseline test
      `production_prepared_lineages_match_the_recorded_baseline_and_metadata`
      unchanged, the sibling pin
      `production_cost_selection_lineages_pin_the_first_reading`
      (`graph_prepared` and `vm_prepared` under `CostSelection` both
      17/17/17/17), the `Sizes::TEST` report tests at 27 arms and 171 pairs,
      and the committed
      `crates/v3-core/proptest-regressions/neighborhood/recruitment_paths/tests.txt`
      seed replayed.
- [x] `cargo test -p v3-core --test reproducibility` -> pass (3 passed;
      observation consumes no production RNG).
- [x] `cargo test -p v3-cli --test bench_artifacts` (20 passed, 1 ignored) and
      `cargo test -p v3-cli recruitment_paths` (2 passed) -> the whole
      `summary` block, new readings included, folds into the committed
      summary; `cargo fmt --all -- --check`, `cargo clippy --workspace
      --all-targets` and `cargo check --workspace --all-targets` -> clean.
- [x] `make check` -> pass (exit 0 on tested commit 3fe9a069).
- [x] Byte-identity of the eighteen F02 arms against the previous closure:
      per-arm `retained_discovery` / `retained_useful` numerators and
      `outcome_counts` equal the T13.F05 goal summary's, recorded in the
      readings file with the twenty-seven-arm table.
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants` (2026-09-14, one run):

```text
110 mutants tested in 16m: 19 missed, 82 caught, 8 unviable, 1 timeouts
run mode: fresh (run-mode.txt); diff against 52c7db79
output: ~/.local/share/petri-tools/mutants/t13-f06/mutants.out
        (fresh-run-2026-09-14.log beside it; a later MUTANTS_ITERATE=1 pass
        re-tested the 20 survivors: 19 caught, 1 timeout; not closure evidence)
```

| Survivor (`crates/v3-core/src/neighborhood/recruitment_paths/`) | Resolution |
| --- | --- |
| `records.rs:169` `TaskSummary::live` -> `true`; -> `false`; `==` -> `!=` | killed: `recruitment_paths_task_summary_is_live_only_with_all_eight_scenes` (tests.rs) |
| `records.rs:173` `TaskSummary::correct` -> `0`; -> `1` | killed: `recruitment_paths_task_summary_correct_reads_the_named_task_count` (tests.rs) |
| `records.rs:478-480` `RetentionOutcomes::record` `+=` -> `*=` (no_longer_useful), `-=`/`*=` (deleted), `-=`/`*=` (task_dead) | killed: proptest `recruitment_paths_retention_outcomes_count_each_outcome_and_total_them` (tests.rs) |
| `records.rs:485` `RetentionOutcomes::total` `+` -> `-` (x3), `+` -> `*` (x2) | killed: the same proptest asserts `total()` equals the recorded count |
| `records.rs:627` `choose` `<` -> `<=` | killed: `recruitment_paths_choose_gates_siblings_but_never_the_parent_by_liveness` (tests.rs) pins that liveness gates siblings only |
| `qualification.rs:193` `CostVerdict::neutral` -> `true` | killed: `recruitment_paths_cost_verdict_is_neutral_only_at_equal_scores` (tests.rs) |
| `experiment.rs:499` damage fold `+` -> `*`; `experiment.rs:505` `-` -> `+` | killed: `summary_damage_counts_task_death_over_all_and_loss_over_the_task_live` (experiment.rs `#[cfg(test)]` module, synthetic lineages; the reduced run draws no task-dead proposal) |
| `experiment.rs:307` `discovery + followup` -> `discovery * followup` (TIMEOUT, 120 s cap, both passes) | deferred: see Notes for AI Agents; detected by `recruitment_paths_reduced_run_has_complete_supply_and_replay` (`2 * 1` generations give 4 proposals, not the asserted 6) but the production-size pins in the same binary run 512 instead of 48 generations and exceed the cap first |

- [x] Benchmark summary stored at `docs/progress/features/<id>.json`, local raw
      hash/byte count and verification time checked, series entry points to the
      summary, and no new full report staged.

Transcripts, the twenty-seven-arm table, per-step fixture verdicts and the
ecological table go to
[`docs/progress/readings/t13-f06-recruitment-and-retention-qualification.md`](../../progress/readings/t13-f06-recruitment-and-retention-qualification.md).

## Performance and Goal Impact

**Predeclaration — written before the run.** Observation only: no natural
analog or environmental-pressure integration applies, and nothing reaches a
creature. Expected compute cost: none in simulation; the goal profile's
recruitment-paths observation grows from 18 to 27 arms, from 8,315 ms at
T13.F05 to an expected 12–14 s, inside T13.F02's 120 s cap; the gate profile
does not run the experiment. No severe allowance, threshold change or epoch
re-pin is predeclared. Both profiles compare against the series index at run
time: gate epoch `remove-complementary-nutrition.json` and previous
`t13-f05-function-preserving-module-recruitment.json`; goal-worlds epoch
`t13-f03-mutation-target-applicability-goal.json` and previous
`t13-f05-function-preserving-module-recruitment-goal.json`, all under
`docs/progress/features/`. The +10%/+50% work and +25%/+100% wall flags stand;
inherited epoch flags are recorded apart from change against the previous
closure; new flags are investigated. Caps: founder observation under 10 s,
evolved under 180 s summed across seeds, drift under 30 s per world, the
experiment under 120 s, goal profile under 15 minutes.

| Indicator | Predeclared direction |
| --- | --- |
| Six normalized simulation counters, both profiles | Byte-identical to T13.F05. |
| Founder and evolved neighborhood, drift-walk blocks, evolved trajectories | Byte-identical to T13.F05. Depth-2,000 drift therefore reads 7/9/7 per 2,000 = 0.0035/0.0045/0.0035 against the T11 track's re-based 0.005 floor (0.0015 at depth 1,000 holds). An identical reading is not a regression against the previous closure; it is still below the floor, and the T13.F05 acceptance was post-observation for that report only, so the reading is escalated for the user's decision at closure, not assumed accepted. |
| Eighteen F02 arms | Byte-identical: Task A and unprepared arms 0/32; `graph_prepared` 17/13/2 and 17/17/16; `vm_prepared` 18/14/1 and 18/18/15. |
| `CostSelection` arms, Task A and unprepared Task B | No positive floor. Expected 0/32 like the cost-free arms; a nonzero reading is reported with its path. |
| `CostSelection` arms, prepared Task B | Discovery expected at or near the selection arms' 17–18/32 (one connection edit raises the score, so cost cannot veto it); retention among discoverers reported with no direction. |
| Cost at checkpoint 48, `CostSelection` versus `Selection` | Median `genome_size` and `carrying_sum` at or below the selection arm's for the same form; reported, no floor. |
| Fixture per-step verdicts | Every qualified path's last step retained; neutral steps that add units rejected; counts reported per form and backend. |
| Wall/creature-tick, both profiles | No direction; the T13.F05 flags were host conditions and are re-read on the same host. |
| Summary byte count | Grows by the nine arms and 98 pairs of 32 rows; recorded beside T13.F05's 4,161,635 bytes, no cap. Shrinking the summary representation stays with T15.F01, never a per-feature exception. |
| Diversity and cognition indicators | Byte-identical to T13.F05. |

**Measured verdict.** Gate and goal: exit 0, `severe=false`, no wall/work
flags against either reference (F05's goal wall flag did not recur). Six
counters, founder/evolved neighborhood, drift-walk blocks, reachable-
structure distribution and every diversity/cognition indicator are byte-
identical to F05; the eighteen F02 arms are byte-identical; the nine new
`CostSelection` arms and 98 new pairs read as predeclared; the experiment
ran in 10,275 ms (cap 120 s). One escalation carries forward unchanged from
F05: depth-2,000 drift reads 0.0035/0.0045/0.0035, byte-identical to F05
and still below the 0.005 floor — escalated per the spec's own text, not
assumed accepted. Full tables and transcripts in the readings file.

- Summaries: [gate](../../progress/features/t13-f06-recruitment-and-retention-qualification.json),
  [goal](../../progress/features/t13-f06-recruitment-and-retention-qualification-goal.json).
- Full readings: [`docs/progress/readings/t13-f06-recruitment-and-retention-qualification.md`](../../progress/readings/t13-f06-recruitment-and-retention-qualification.md).

## Success Criteria

- [x] The stored goal report carries the fixed T13.F02 comparison unchanged
      and, for every starting form, a `CostSelection` arm with probability and
      time to first useful contribution, retention and loss at discovery + 16,
      damage, checkpoint cost and paired differences, on Task A and Task B.
- [x] The maintained fixtures show both backends' qualified useful paths with
      per-step cost verdicts, and the readings file separates assay selection
      from the goal worlds' ecological reading with both limits stated.
- [x] Nulls, censoring and gaps are explicit, mapped to their owning feature
      where they are implementation failures, and no cognition or universal
      emergence claim is made; T11.F10 and T11.F13 can cite the exposure and
      retention readings by summary path and arm name.
- [x] Required checks, review, mutation adjudication and benchmark records
      are complete and truthful; only then mark the roadmap row, the track's
      qualification criterion and this spec Complete.

## Notes for AI Agents

- Decision: The cost-bearing reading is a third selection policy on the
  existing assay, ordered by score then summed ending energy; it is a bound,
  not an ecology estimate, and no energy-to-score conversion is introduced.
- Decision: the user accepted the depth-2,000 drift reading
  0.0035/0.0045/0.0035 against the 0.005 floor for this feature on
  2026-09-14 ("I approve the same floor violation from previous features"),
  matching the T13.F03–F05 acceptances; the epoch is unchanged.
- Deferred: review P3s — `Policy::COUNT`/`STARTS` are hand-maintained
  (`Policy::ALL` would drop the `chain`); `checkpoint_cost` asserts inside a
  `map` and `filter_map`s a never-`None` `Spread::of`; the fixture-verdict
  test recomputes the retention rule the proptest already owns.
- Deferred: mutant `experiment.rs:307` times out instead of dying (see the
  Verification table); a kill needs a timeout or test-selection change the
  gate forbids. The user accepted this deferral on 2026-09-14.
