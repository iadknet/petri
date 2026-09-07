# T11.F16 — Drift-Depth Indicator

**Status**: In Progress
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

- [ ] Lift the bounded production drift walk into `neighborhood`, adding
      focused tests first and reusing existing mesh and per-birth evaluation.
- [ ] Add goal-only report assembly, metadata, historical defaults and
      separate timing; cover profile presence and deterministic serialization.
- [ ] Store fresh gate and single goal reports at
      `docs/progress/features/t11-f16-drift-depth-indicator.json` and
      `...-goal.json`, append both to `docs/progress/benchmark-series.json`,
      and add the closure row and concise depth definitions to
      `docs/progress.md`. Preserve historical reports and epoch baselines.

## Verification

- [ ] TDD fixtures independently replay a reduced seeded walk through the
      production engine and verify checkpoint mesh/birth results, exact depth
      progression, current reachability, and unconditional zero-event/dead
      offspring retention. A zero-mutation case retains founder measurements
      at every requested checkpoint; a nonzero case must exercise applied
      mutation so parity cannot pass solely on unchanged founders.
- [ ] Observation-isolation fixtures prove that adding an intermediate
      checkpoint or changing birth trial counts cannot change later lineage
      genomes/mesh readings; the source founder is unchanged. Verify first-20
      subset selection by identity/order and the exact walk/birth seed formulas
      with small fixtures; do not select lineages by measured behavior.
- [ ] Pure aggregation invariants have `proptest` coverage in `v3-core`:
      pooled counts equal the sum of input readings, count/subset bounds hold,
      and birth totals/buckets retain all denominators under regrouping.
      Reuse existing covered aggregation where possible; assertions must not
      depend on the cases drawn.
- [ ] CLI tests verify historical missing readings/timing, goal-only presence,
      one walk per report rather than per world seed, truthful reduced-size
      metadata, known fraction/mean denominators, and byte-identical reduced
      deterministic output across repeated runs and thread counts.
      Focused commands: `cargo test -p v3-core --lib neighborhood` and
      `cargo test -p v3-cli --lib bench::tests`; run
      `cargo check --workspace --all-targets` after coherent Rust edits.
- [ ] Complete the diff self-review for reuse, simplification and efficiency,
      then a fresh `MUTANTS_ITERATE=0 make rust-mutants`; record summary,
      output path, every missed/timed-out mutant and its resolution.
- [ ] Run `make bench PROFILE=gate FEATURE=t11-f16-drift-depth-indicator`
      and once `make bench PROFILE=goal FEATURE=t11-f16-drift-depth-indicator`.
      Record the full-depth measurement, 30-second cap result, existing
      observation budgets and compute comparisons. Compare every pre-existing
      gate/goal deterministic field with T11.F08 after excluding only the new
      drift field; they must be identical for this observation-only feature.
- [x] A second full goal run is not applicable: the 2026-09-05 user decision
      in `docs/workflow.md` requires one closure run; reproducibility tests
      in `make check` and the unchanged gate two-run check remain required.
- [ ] Run `make roadmap-check` on document edits; record independent review
      findings/resolutions and final closure-content `make check` evidence.

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
creature-tick against T11.F08 (previous closure) and the pinned T11.F04 epoch,
retaining graph-counter definition qualifications. Record dated existing
goal readings, their unchanged-field comparison, and all five drift rows
with mesh means, route/cap denominators, and birth counts/fractions. Natural
analog requirement is not applicable: this feature is observation only.

The measured executed-node means and changed/all-birth fractions at 1,000
and 2,000 become no-regression indicator components from this closure;
T11.F17 owns their first floors. No component is predeclared to move in this
feature. Compare the research note descriptively with its sample/seed/code
differences disclosed. Mutation-only depth is not evidence of ecological
persistence or cognition; unimplemented cognition indicators stay `Undefined`.

## Success Criteria

- [ ] The maintained goal report contains the complete reproducible depth
      baseline through 2,000 generations within the predeclared budget.
- [ ] Every prior deterministic result and production behavior is preserved;
      depth, executed structure, and birth denominators are explicit.
- [ ] Required checks, fresh mutation evidence, independent review and closure
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
- Closure cost record: advisor consultations, requirement corrections,
  acceptance exceptions, review findings, remediation passes, user
  interventions and task-specific usage will be recorded as available.
