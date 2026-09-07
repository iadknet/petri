# T11.F16 — Drift-Depth Indicator

**Status**: Complete
**Last updated**: 2026-09-07
**Feature**: T11.F16
**Track**: [T11 — Brain Genotype-Phenotype Map](../../roadmaps/t11-brain-genotype-phenotype-map.md)

## Goal

At every goal-profile closure, report how the founder's descendants change
under production mutation alone through generation 2,000: what their meshes
execute and how often a subsequent birth changes behavior. Store the first
depth baseline for T11.F17 without changing the simulated world.

## Non-Goals

- No mutation delivery, operator, runtime, founder, learning, inheritance,
  selection, cost, size limit, pruning, or production configuration changes.
- No counterfactual policies, extended ecological run, whole-population
  census, new CLI/profile, UI, dependency, or generic observation framework.
- No new floor or cognition claim. Drift generations are mutation-only birth
  steps, not ecological survival, useful novelty, or the goal population's
  generation distribution.

## Inputs and Invariants

- Source intent: the owning track's T11.F16 row and detailed note;
  [T11.F14](t11-f14-mesh-execution-observability.md); the
  [depth research note](../../strategy/mesh-depth-research-2026-09-07.md)
  Section 3.5 and Appendix B's `probe_drift_depth_births` and
  `probe_drift_policies`. Dependencies remain in the owning track.
- Existing seams: `MutationEngine::apply_mutations_with_food_type_count`,
  `mesh_reachable_nodes`, `neighborhood::Battery::{signature,mesh_execution}`,
  `births::per_birth_result`, `BirthResult`/`Tally`, and the report conversion,
  integer pooling, timing, and `Indicator` patterns in
  `crates/v3-cli/src/bench.rs`. Keep core observations in `neighborhood`;
  mutation, runtime, and simulation must not depend back on observation types.
- Research decision, 2026-09-07: lift the existing production-only probe
  into the core neighborhood module and existing goal report. The strongest
  alternative is placing it in the gate: it would expose the same depth
  data but repeat a large mutation walk in ordinary debug gate checks.
  Goal placement gives one reading per closure at the existing once-per-goal
  observation boundary. Running a longer ecology is neither necessary for
  this drift question nor seconds-scale. Reuse the birth-only API instead of
  the draft's `evaluate_genome`, whose operator rows this feature does not
  consume. Existing mesh observations already include knockouts. No new
  package is needed. [Serde's field defaults](https://serde.rs/attr-default.html)
  support the required historical `Undefined` reading. The research note's
  seven-second result motivates the budget below, not an asserted current
  timing or a promised ecological prediction.

The reading is versioned `drift-depth-v1` and appears once per **goal report**,
outside its world-seed loop. Gate, sweep, and other profiles report
`Undefined`. The production experiment is fixed before implementation:

| Parameter | Value |
| --- | --- |
| Founder | Current production `V3Alpha1` founder |
| Lineages | 50, indexed 0 through 49 |
| Checkpoints | 0, 22, 250, 1,000, 2,000 birth steps |
| Walk seed | `90_000 + lineage_index`, one persistent `SmallRng` per lineage |
| Birth sample | First 20 lineage indices, fixed before any outcomes |
| Birth trials | 100 fresh offspring per sampled parent per checkpoint |
| Birth seed offset | `7_000_000 + 1_000 * (lineage_index + 1) + checkpoint` |
| Actual trial seed | Offset plus existing `BIRTH_SEED_BASE` (9,000) plus trial index |
| Battery | Unchanged `neighborhood-v1`: 48 snapshots and 8 four-tick sequences |
| Observation cap | 30 seconds release wall time for the complete walk and readings |

The lineage seeds and birth offsets retain the depth probe's formulas. The
20-lineage birth subset adopts the policy probe's explicit bounded subset;
the draft depth-birth function instead evaluates all 50. This is a new
versioned baseline, not a numerical reproduction of every table in the note.
Offsets can overlap between different checkpoints/lineages; they are fixed
paired sampling streams, not a claim of statistical independence. Checkpoint
observations use separate birth RNGs and must never consume walk RNG state.

Each generation applies the production engine exactly once per lineage with
the parent's current structural reachable set, current production mutation
configuration, and food type count. Retain the resulting genome
unconditionally, including zero-applied births, behaviorally dead offspring,
and later recovery; no rejection, retry, selection, restart, size stop, or
clamping. Every call advances depth by one irrespective of requested/applied
events. Observations never alter the lineage genome, its RNG, or any world
simulation. Walks carry genotypes only; battery evaluations start with the
existing fresh runtime state and retain state only within each four-tick
sequence. The reported depth does not change battery sensor values, including
the existing zero-valued generation sensor.

At each checkpoint, measure all 50 genomes through T11.F14's existing
`mesh-execution-v1` and `static-successor-bypass-v1` definitions. Store integer
totals and the lineage denominator, with means for total, reachable, executed,
and knockout mesh-node counts; route-varying lineage count out of 50; and
actual hop-cap hits out of 4,000 battery executions. Means/fractions use the
report's existing six-decimal formatting, computed from pooled integer counts.
No VM-instruction or graph-internal count stands in for mesh nodes.

For the first 20 checkpoint genomes, evaluate fresh production births using
the existing per-birth API and complete parent signature. Pool its full birth
result, preserving total births (2,000), zero-applied births, requested-event
counts, applied-event buckets, and silent/changed/dead tallies. Report each
class's fraction per all births alongside the reused conditional tally
fractions: `changed / all
births` is the exposure indicator, excludes the separately reported dead
class, and includes zero-applied births in its denominator. Identical
all-NoOp parents/children remain silent under the existing classifier;
zero-applied births are counted separately, not called mutated silent births.
Zero denominators remain `Undefined` according to the existing report rules.

Store metadata (version, founder, lineage/subset/trial counts, checkpoints,
seed formulas, battery/mesh/knockout versions and execution denominators) and
ordered checkpoint readings under serde-defaulted
`deterministic.goal_indicators.drift_depth`. Historical absent readings are
`Undefined`, never zero. Record its independent full elapsed time under
`environment.drift_depth_wall_clock_ms`, outside world tick, founder,
evolved-neighborhood, and final-population observation timers. Missing or
unrun historical timing must be distinguishable from a measured value.
Use the existing reduced-test-size pattern internally; no public tuning knob
or change to production profile identity or existing trial sizes is required.

## Implementation Tasks

- [x] Lift the bounded production drift walk into `neighborhood`, adding
      focused tests first and reusing existing mesh and per-birth evaluation.
- [x] Add goal-only report assembly, metadata, historical defaults and
      separate timing; cover profile presence and deterministic serialization.
- [x] Store fresh gate and single goal reports at
      `docs/progress/features/t11-f16-drift-depth-indicator.json` and
      `...-goal.json`, append both to `docs/progress/benchmark-series.json`,
      and add the closure row and concise depth definitions to
      `docs/progress.md`. Preserve historical reports and epoch baselines.

## Verification

- [x] TDD fixtures independently replay a reduced seeded walk through the
      production engine and verify checkpoint mesh/birth results, exact depth
      progression, current reachability, and unconditional zero-event/dead
      offspring retention. A zero-mutation case retains founder measurements
      at every requested checkpoint; a nonzero case must exercise applied
      mutation so parity cannot pass solely on unchanged founders.
- [x] Observation-isolation fixtures prove that adding an intermediate
      checkpoint or changing birth trial counts cannot change later lineage
      genomes/mesh readings; the source founder is unchanged. Verify first-20
      subset selection by identity/order and the exact walk/birth seed formulas
      with small fixtures; do not select lineages by measured behavior.
- [x] Pure aggregation invariants have `proptest` coverage in `v3-core`:
      pooled counts equal the sum of input readings, count/subset bounds hold,
      and birth totals/buckets retain all denominators under regrouping.
      Reuse existing covered aggregation where possible; assertions must not
      depend on the cases drawn.
- [x] CLI tests verify historical missing readings/timing, goal-only presence,
      one walk per report rather than per world seed, truthful reduced-size
      metadata, known fraction/mean denominators, and byte-identical reduced
      deterministic output across repeated runs and thread counts.
      Focused commands: `cargo test -p v3-core --lib neighborhood` and
      `cargo test -p v3-cli --lib bench::tests`; run
      `cargo check --workspace --all-targets` after coherent Rust edits.
- [x] Complete the diff self-review for reuse, simplification and efficiency,
      then a fresh `MUTANTS_ITERATE=0 make rust-mutants`; record summary,
      output path, every missed/timed-out mutant and its resolution.
- [x] Run `make bench PROFILE=gate FEATURE=t11-f16-drift-depth-indicator`
      and once `make bench PROFILE=goal FEATURE=t11-f16-drift-depth-indicator`.
      Record the full-depth measurement, 30-second cap result, existing
      observation budgets and compute comparisons. Compare every pre-existing
      gate/goal deterministic field with the corresponding
      `remove-complementary-nutrition` report after excluding only the new
      drift field; they must be identical for this observation-only feature.
- [x] A second full goal run is not applicable: the 2026-09-05 user decision
      in `docs/workflow.md` requires one closure run; reproducibility tests
      in `make check` and the unchanged gate two-run check remain required.
- [x] Run `make roadmap-check` on document edits; record independent review
      findings/resolutions and final closure-content `make check` evidence.

Implementation verification, 2026-09-07:

- TDD red runs: `cargo test -p v3-core --lib neighborhood::drift` first
  failed on the absent drift API; `cargo test -p v3-cli --lib bench::tests::drift`
  first failed on absent report fields. The checkpoint-isolation fixture
  likewise preceded extraction of the borrowed checkpoint reader.
- `cargo test -p v3-core --lib neighborhood`: 57 passed, 1 ignored;
  `cargo test -p v3-cli --lib bench::tests`: 37 passed. The fixtures cover
  nonzero applied mutations, zero-applied steps, applied mutations retained
  while behaviorally dead, exact seeded replay, the fixed ordered subset,
  unchanged genomes after checkpoint reads, and identical next offspring.
  Aggregation properties cover mixed requested/applied buckets and regrouping.
- `cargo check --workspace --all-targets` exited 0 after each coherent final
  Rust edit. `cargo clippy --workspace --all-targets -- -D warnings` exited 0.
  The first `make check` reached seven server tests blocked only by sandbox
  listener permissions. An escalated run passed all Rust tests, including
  those listeners, then found `run_deterministic` at 170/167 lines. Extracting
  the drift-only timing block fixed that finding; the compile check, 37 CLI
  tests, and Clippy passed afterward. These partial runs are not claimed as
  a full pass. The orchestrator independently ran `make check` with required
  local permissions on 2026-09-07: exit 0, log
  `/tmp/t11-f16-parent-check.log`. Closure checkbox/status updates are followed
  by a final full recheck; its result and the exact tested commit are surfaced
  in the parent task before integration.
- Diff self-review (repeated after the timing extraction): exact integer
  birth pooling moved from the CLI to `BirthResult::merge` and is reused by
  both observations; checkpoint reads borrow genomes and cannot consume walk
  RNGs. No new dependency, runtime/configuration behavior, observation
  framework, lint exemption, or redundant operator evaluation was added.
  Mesh totals remain integers until existing six-decimal report conversion.
- Fresh `MUTANTS_ITERATE=0 make rust-mutants` exited 0 after the last
  code/test edits and repeated self-review. Summary: `76 mutants tested in 6m:
  46 caught, 29 unviable, 1 timeouts`. Output:
  `/Users/istefanek/.local/share/petri-tools/mutants/t11-f16/mutants.out`;
  sibling `run-mode.txt` reads `fresh`. `missed.txt` is empty. Full survivor
  list (one): `crates/v3-core/src/neighborhood/drift.rs:100:19: replace +=
  with *= in observe` — **deferred**. Replacing `depth += 1` with `depth *= 1`
  holds depth at zero and makes any positive checkpoint nonterminating;
  the full suite reached the configured 120-second test timeout. This is
  observable, not equivalent. No production code, test selection, timeout,
  property case count, or mutation exclusion was changed to hide it.
- Document verification: `make roadmap-check` exited 0 after implementation
  evidence edits; only Aqua's nonblocking timestamp-cache warning appeared.

## Performance and Goal Impact

Predeclared cost: one mutation-only walk of 100,000 birth steps, 250 mesh
readings including their existing knockout batteries, and 10,000 sampled
birth trials. This is observation work, timed separately, with a **30-second
release cap**. Investigate an excess without reducing counts, omitting
checkpoints/knockouts, changing the battery or production semantics, or
silently raising the cap. The workflow's founder 10-second,
evolved-neighborhood 180-second, and total goal investigation 15-minute
thresholds remain unchanged.

No simulation work-counter change or severe compute regression is justified.
At closure, record gate deterministic-work and wall-time deltas per
creature-tick against both the previous closure and pinned epoch. Both are
the [nutrition-removal gate report](../../progress/features/remove-complementary-nutrition.json);
the goal series likewise uses the
[nutrition-removal goal report](../../progress/features/remove-complementary-nutrition-goal.json)
for both roles, as recorded in `docs/progress/benchmark-series.json` at the
planning base. No cross-definition graph-counter comparison is needed against
these references. Record dated existing
goal readings, their unchanged-field comparison, and all five drift rows
with mesh means, route/cap denominators, and birth counts/fractions. Natural
analog requirement is not applicable: this feature is observation only.

The measured executed-node means and changed/all-birth fractions at 1,000
and 2,000 become no-regression indicator components from this closure;
T11.F17 owns their first floors. No component is predeclared to move in this
feature. Compare the research note descriptively with its sample/seed/code
differences disclosed. Mutation-only depth is not evidence of ecological
persistence or cognition; unimplemented cognition indicators stay `Undefined`.

Measured baseline, 2026-09-07:

- Guarded `make bench PROFILE=gate FEATURE=t11-f16-drift-depth-indicator`
  and the single `make bench PROFILE=goal FEATURE=t11-f16-drift-depth-indicator`
  both exited 0, sequentially after mutation testing, with no competing
  builds/tests started during measurement. Stored reports are
  [gate](../../progress/features/t11-f16-drift-depth-indicator.json)
  (18:32:28Z) and [goal](../../progress/features/t11-f16-drift-depth-indicator-goal.json)
  (18:38:52Z); both are appended to the existing benchmark series.
- Python structural equality checks of each complete `deterministic` block,
  removing only `goal_indicators.drift_depth` from the new reports, passed
  against the corresponding `remove-complementary-nutrition` report. Those
  references serve as both preceding closure and pinned epoch, deduplicated
  by the harness. All six normalized counters are **0.000000%** changed in
  both profiles; no severe compute regression. Gate wall/creature-tick is
  **0.001427946932 ms**, **-8.455024%**; goal is **0.006461767113 ms**,
  **-1.532154%**, on the same Apple M1 Pro / 8-thread host. Gate total world
  time is 609.084 ms; goal total world time is 323,802.698 ms. These timings
  are secondary signals, not evidence that this observation speeds simulation.
- The complete drift walk/readings cost **2,189.799 ms**, passing the
  **30-second** cap without reducing any size. Founder observations cost
  46.348 ms (gate) / 59.705 ms (goal), below 10 seconds; summed evolved
  observations cost 2,826.039 ms, below 180 seconds; final-population
  observation is independently 479.147 ms. Goal log creation-to-final-write
  elapsed is 330.131 seconds (about 5.5 minutes), below the 15-minute
  investigation threshold. Gate drift is `Undefined`, with null elapsed time.
- Existing goal readings are exactly unchanged: final populations
  718/4,517/1,138 (seeds 11/22/33), no extinction; births/100 ticks
  13,882.283333; structure min/p25/median/p75/max/mean
  44/77/81/84/283/85.491919; surviving clades 137/137/134 with entropy
  2.602116/1.086756/2.158890. Population generation median/max is
  24/54, 41/63, 48/60. Memory and temporal readings, founder/evolved
  neighborhood tallies, and all deferred `Undefined` cognition readings
  are included in the exact equality check.

Each row below pools 50 mesh readings and exactly 2,000 births from the first
20 lineage indices. Class fractions use all births; zero-applied births stay
separate, and the JSON also retains every requested/applied bucket and the
conditional tally fractions. Route and cap fractions have explicit 50 and
4,000 denominators in the JSON.

| Birth depth | Mean total / reachable / executed / knockout nodes | Route-varying lineages | Cap hits / executions | Zero-applied births | Silent count / all-birth fraction | Changed count / all-birth fraction | Dead count / all-birth fraction |
| ---: | --- | ---: | ---: | ---: | --- | --- | --- |
| 0 | 2.000000 / 2.000000 / 2.000000 / 0.000000 | 0/50 | 0/4,000 | 1128 | 465 / 0.232500 | 406 / 0.203000 | 1 / 0.000500 |
| 22 | 3.500000 / 2.940000 / 2.220000 / 0.480000 | 4/50 | 0/4,000 | 1135 | 676 / 0.338000 | 185 / 0.092500 | 4 / 0.002000 |
| 250 | 16.180000 / 6.060000 / 2.500000 / 1.500000 | 0/50 | 0/4,000 | 1103 | 884 / 0.442000 | 9 / 0.004500 | 4 / 0.002000 |
| 1000 | 69.020000 / 11.280000 / 3.120000 / 2.360000 | 1/50 | 0/4,000 | 1132 | 864 / 0.432000 | 2 / 0.001000 | 2 / 0.001000 |
| 2000 | 136.180000 / 14.800000 / 3.000000 / 2.300000 | 1/50 | 0/4,000 | 1135 | 855 / 0.427500 | 9 / 0.004500 | 1 / 0.000500 |

At depth 1,000 the no-regression baseline components are **3.120000 executed
nodes** and **0.001000 changed/all births**; at depth 2,000 they are
**3.000000** and **0.004500**. T11.F17 owns the first floors. No cognition or
ecological-persistence improvement is claimed.

The research note's depth-2,000 table read 140.02 total, 16.84 reachable,
3.16 executed and 0.0026 changed/all births; this version reads 136.18,
14.80, 3.00 and 0.0045. Both show scaffold growth with few executed nodes
and low deep-birth exposure, but the values are not a paired improvement
comparison: this is the current production code, the fixed first-20 birth
subset replaces the appendix depth probe's all-50 subset, and the seed
formulas retain overlapping paired streams. The maintained version uses
only the production policy and birth-only API, with no counterfactual arms.
The historical seven-second probe timing is not a benchmark baseline.

## Success Criteria

- [x] The maintained goal report contains the complete reproducible depth
      baseline through 2,000 generations within the predeclared budget.
- [x] Every prior deterministic result and production behavior is preserved;
      depth, executed structure, and birth denominators are explicit.
- [x] Required checks, fresh mutation evidence, independent review and closure
      records are complete; the feature row is checked and spec Complete on main.

## Notes for AI Agents

- Planning base: `5921411bd3de8cfb0b502831da0c5b06648ef654`; worktree
  `/Users/istefanek/projects/petri/.worktrees/t11-f16`, branch `codex/t11-f16`.
  Track is already In Progress and master Active, so neither needs promotion.
- Models: `gpt-6-astra` low orchestrator, one persistent high spec owner and
  advisor, one persistent low implementer, fresh medium final reviewer.
  Planning self-review is not independent validation or an advisor consultation.
- Readiness self-review, 2026-09-07: 0 P1, 1 P2, 0 P3; Ready after one
  revision. The P2 was implicit absolute fraction output despite reusing a
  converter that exposes only conditional fractions; the birth paragraph
  now explicitly requires each class per all births. Reviewed template,
  roadmap intent, dependency definitions, sample/seed choices, production
  isolation, budget and verification. Runtime behavior is not yet verified;
  this author review is not the independent final review.
- Planning verification, 2026-09-07: `make roadmap-check` exited 0 with
  `roadmap-check: validation passed`; Aqua emitted non-blocking cache and
  timestamp permission warnings. The new spec's whitespace check passed.
  Only this flat spec is changed; the orchestrator verifies and commits it.
- Consultation 2 / requirement correction 1, 2026-09-07: the planning spec
  incorrectly named T11.F08 as the preceding closure and T11.F04 as the epoch.
  The benchmark series already placed `remove-complementary-nutrition` last
  and pinned its reports in both roles at starting commit `5921411b`.
  Its [Complete spec](../remove-complementary-nutrition.md) records the
  separately authorized gate/goal cost acceptances and baseline updates.
  Compare all prior deterministic fields and normalized compute against those
  actual references. This corrects a stale source assumption before measured
  profiles; it changes no required equality, threshold, budget, sample,
  simulation behavior, or historical report. No acceptance exception or user
  intervention is involved. Revision `make roadmap-check` exited 0 with
  `roadmap-check: validation passed`; `git diff --check` exited 0.
- Consultation 1 accepted the direct probe lift, shared exact birth pooling,
  optional elapsed timing, and all-birth fractions; no recommendations were
  rejected. Consultation 2's baseline correction was accepted.
- Deferred mutation finding (P2): the `drift.rs:100:19` depth-progress mutant
  above is detected only by the full-suite timeout, not a bounded test
  assertion. A watchdog/process harness would add timing-sensitive testing
  machinery for this loop; no such expansion is made in this observation
  feature. The original finite production loop and exact seeded depth
  progression are verified; the survivor remains explicitly deferred for
  independent review.
- Consultation 3 accepted explicit P2 deferral of the timeout survivor;
  advisor confirmed it is non-equivalent, the harness detected nontermination,
  and no watchdog machinery or production change is justified. No advice
  rejected.
- Consultation 4 accepted: advisor independently confirmed both full
  unchanged-field comparisons, work counters, metadata, all five depth rows,
  and observation budgets. Ready for the fresh independent reviewer; no
  code, measurement, or requirement change requested. All four consultations'
  recommendations were accepted; none rejected.
- Handoff cost record: 4 advisor consultations, 1 requirement correction,
  0 acceptance exceptions, 0 user interventions; one implementation lint
  remediation (timing extraction), 0 independent-review remediation passes
  at handoff. Independent review and closure verification are recorded below.
  Task-specific usage unavailable.
- Independent final review, 2026-09-07: fresh `gpt-6-astra` medium reviewer;
  0 P1, 1 P2, 0 P3. The sole P2 confirms the deferred depth-progress timeout
  above; no new finding or remediation requested. Reviewer independently
  audited fresh mutation artifacts, both prior-field equality comparisons,
  all five checkpoint rows and observation budgets. No post-review code edits.
- Closure cost record: 4 advisor consultations; 1 requirement correction;
  0 acceptance exceptions; 0 user interventions after launch; one implementation
  lint remediation and 0 post-review remediation passes. Roles were
  `gpt-6-astra` low orchestrator, persistent high spec owner/advisor, persistent
  low implementer, and fresh medium reviewer. Aggregate task-specific usage
  across all roles unavailable. Session metadata verified orchestrator
  model/effort; native spawn arguments explicitly set each delegated role.
  User-authorized local integration and cleanup only; no remote mutations.

- Closure verification sequencing: the checker rejected a provisional Complete
  status with the two still-pending checkboxes, so status was returned to
  In Progress until the independent full check passed. This was a document
  sequencing correction, not a requirement change or check exception.
