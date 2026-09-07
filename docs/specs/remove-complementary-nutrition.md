# Remove Complementary Nutrition

**Status**: In Progress
**Last updated**: 2026-09-07
**Scope**: Maintenance; no roadmap feature ID, dependency row, or closure changes

## Goal

Ordinary food alone supplies the energy for survival and reproduction. Restore
the previous shared feeding reward and founder policies against current main,
while retaining subsequent runtime and mutation fixes.

## Non-Goals

- Reverting whole historical files, reproducing historical trajectories, or
  tuning unrelated production parameters to match an old population curve.
- Removing configurable ordinary food types, typed actions/sensors, fertility
  maps, growth, spread, depletion, or inhibition.
- Compatibility reserves, nutrition migrations, new dependencies, or a new
  workflow. Archived documents, completed feature records, and stored historical
  benchmark results retain their original meaning and acceptance results.

## Inputs and Invariants

The user maintenance request is authoritative. Follow [the workflow](../workflow.md)
with the feature-template sections adapted here; roadmap ownership and checkbox
requirements do not apply. Work starts from
`3871ad1173f0197a3ca06d6f719ddef456c5dde0` in
`/Users/istefanek/projects/petri/.worktrees/remove-complementary-nutrition`, branch
`codex/remove-complementary-nutrition`.

Research on 2026-09-07 compared actual code and tests at nutrition commit
`a27a5d6be6828623c0daadd7a766d224e92a6f60`, parent `46866098`, and current main:

- `config/simulation.rs`, `simulation/actions/mod.rs`, and
  `simulation/actions/reproduction.rs` establish the removed shared reward,
  one-food defaults, and pre-existing energy charging sequence.
- `creature/founder.rs` and `creature/cgp_founder.rs`, including their historical
  executable tests, establish the priorities, thresholds, and Eat-then-Move
  queues below. Historical comments incorrectly described Move-then-Eat and
  the canonical direction heuristic did not select the full cardinal maximum.
- Current `creature/founder.rs` supplies a label-based VM builder, full cardinal
  argmax, stable N/E/S/W ties, and four spare VM registers introduced after the
  nutrition commit. Current `runtime/` and `mutation/` include later graph
  clocks, reward learning, mesh routing, and structural-mutation repairs.
- `frontend/src/stores/startupConfig.ts` and
  `components/config-panel/runtime/EnergyCostsSection.tsx` show the original
  backend/frontend agreement and the removed live Eat Reward field.

Selected approach: remove nutrition fields and restore energy-only semantics
inside the existing configuration, typed Eat action owner, founder builder,
runtime inputs, server projections, and UI. A whole-commit revert is the
strongest alternative but would overwrite later fixes and restore defective
canonical direction code; a compatibility layer would retain the mechanism
the user asked to remove. No external package choice is involved, so local
history is the decisive research evidence.

Required contracts:

1. **Defaults and feeding.** Backend defaults, empty-food-list normalization,
   frontend fallback/restart defaults, and their serialized config agree on one
   `Primary Food`, color `#22c55e`, initial density `1.0`, initial coverage `0.54`.
   Keep all other food ecology defaults. Restore
   `energy.costs.eat_reward_per_food = 5.0`, normalized by the existing finite
   nonnegative helper with fallback `5.0`, and its live runtime update path.
   Typed Eat consumes only the selected ordinary type and grants
   `consumed_density * eat_reward_per_food` equally for every configured type;
   cap energy at `max_energy` before subtracting the existing adjusted Eat cost.
   Empty/invalid selections keep normal failure and action-cost accounting.
   Remove per-type `metabolic_energy_yield` and `reproductive_reserve_yield`.

2. **Reproduction order.** Count the attempt; resolve/validate target; enforce
   population cap; enforce age; charge adjusted reproduction cost; enforce
   post-cost minimum energy; normalize requested transfer to zero for invalid,
   nonpositive, or nonfinite requests and otherwise cap at
   `default_offspring_energy`; reject zero/unaffordable transfers; deduct a
   successful transfer and continue the current offspring pipeline. Target,
   population, and age rejections precede the reproduction cost; minimum-energy
   and transfer rejections retain that charge. Keep tick-level failure penalties
   and subsequent inheritance/mutation behavior. Remove the reserve gate,
   reserve deduction, and `RejectedNutritionConstraints`/`NutritionConstraints`
   result variants from runtime and presentation.

3. **Founder policies.** Keep the two-node graph-to-VM mesh, normal mutable
   genome representation, action queue, runtime limits, and at least four
   unreferenced VM registers for subsequent mutation growth. The graph reads
   default-type food here, live energy, age, default-type cardinal neighbor food,
   and occupancy. Restore its three compute nodes (energy threshold, age
   threshold, their product) and six wired custom outputs (food here,
   can-reproduce, N/E/S/W food). No reserve or type-1 input remains. Energy
   thresholds are strict `>`; the existing age gate admits integer ages at
   `min_reproduce_age` (default 20), including configured age zero.

   | Profile | Energy threshold | Offspring transfer | Decision order |
   | --- | ---: | ---: | --- |
   | `V3Alpha1` | 30 | 20 | Reproduce; local forage; Move fallback |
   | `ForageFirstSparse` | 30 | 10 | Local forage; Reproduce; forage fallback |
   | `ForageFirstSparseConservative` | 60 | 10 | Local forage; Reproduce; forage fallback |
   | `ForageFirstSparseRichOffspring` | 40 | 20 | Local forage; Reproduce; forage fallback |
   | `ForageFirstSparseBalanced` | 50 | 15 | Local forage; Reproduce; forage fallback |

   Local forage means `food_here > 0`, then enqueue Eat(type 0) followed by
   Move and execute the queue. Forage-first fallback also queues Eat(type 0)
   followed by Move, even on an empty cell, preserving normal empty-Eat failure
   accounting. Canonical fallback queues Move alone. Reproduction queues one
   Reproduce with the profile transfer. Every movement and reproduction
   direction uses the full primary-food N/E/S/W maximum, encoding cardinal
   indices 0/2/4/6, with ties choosing the first in N/E/S/W order. Eat metadata
   must remain type 0 when direction metadata is written for the following Move.

4. **Intentional historical differences.** Preserve the corrected cardinal
   selection and its tie order for every profile and branch; do not restore the
   old canonical diagonal/wrong-axis results or forage-first tie preference.
   Retain the current spare-register affordance, current mesh single-visit and
   routing semantics, graph world-tick memory/reward clocks, reference repair,
   function-preserving duplication, mutation supply and sorted RNG candidates.
   Keep the current typed-food ecology/seeding implementation, including
   independent per-type placement; add explicit multiple ordinary types in
   tests that formerly depended on two defaults. Removed inputs and changed
   founders/defaults naturally change RNG draws and trajectories.

5. **Surfaces and documents.** Remove `nutrition` config, creature reserve,
   reserve capacity/cost, `ReproductiveReserveCurrent`, its mutation sampling,
   runtime argument plumbing, API/projection/inspector fields, UI nutrition and
   yield controls, and obsolete fixtures. Existing typed food IDs, sensors,
   metadata, config/startup APIs and transport paths remain in place. Use
   current strict config validation; do not accept obsolete fields via hidden
   compatibility storage. Restore the live **Eat Reward** control through the
   existing field definitions and update/save flow. Update the affected live
   reproduction, runtime-config, sensor, server-protocol, startup-seeding,
   tick-orchestration, world-grid, and mutation references. In the mutation
   split contract, remove the reserve key while retaining the distinction
   between live `EnergyCurrent` and unchanged `EnergyConsumedThisTick` across
   the plasticity deduction. Correct directly affected
   planned requirements in
   [T06.F02](roadmap/t06-f02-food-transport-and-caching.md) to use the shared
   energy reward and ordinary typed food; do not implement that feature or
   rewrite completed T11 records and archives.

## Implementation Tasks

- [ ] Add failing behavior tests for restored defaults, feeding/reproduction,
      founder priorities/queues/directions, and live reward updates; record red
      evidence before the corresponding production edits.
- [ ] Remove nutrition across core config/state/contracts/actions, runtime
      inputs and callers, mutation sampling, founder construction, and tests.
- [ ] Restore the shared live reward through server configuration and frontend;
      remove obsolete projections, schemas, controls, and fixtures while
      preserving configurable multiple ordinary types.
- [ ] Update live references and directly affected planned specs; inspect the
      diff for accidental historical-file restoration and unrelated changes.
- [ ] Complete self-review, fresh mutation-survivor triage, stored benchmarks,
      advisor checkpoint, independent review, and any permitted remediation.

## Verification

- [x] Baseline `cargo test -p v3-core --test viability`: exit 0, 25 passed on the
      starting commit; orchestrator log
      `/tmp/remove-complementary-nutrition-baseline-viability.log`.
- [ ] Run `cargo test -p v3-core --test viability` first after changing defaults,
      founders, or tick mechanics, before compile/focused checks. Preserve the
      existing production-economics survival/reproduction gates; replace
      complementary-food ablations with a one-ordinary-food scenario that
      replenishes energy, survives, and produces offspring. Do not boost costs,
      rewards, runtime limits, or weaken viability assertions to pass.
- [ ] In config and frontend startup tests, assert exact one-food defaults,
      empty-list fallback, and restart serialization; no nutrition/yield keys
      are serialized. Test shared reward fallback and nonnegative finite
      normalization with proptest over the pure invariant.
- [ ] In core action/tick tests, consume equal positive densities of two
      explicitly configured ordinary types and observe equal energy gains,
      selected-plane consumption, other-plane preservation, cap-before-cost,
      and normal empty/invalid selection outcomes. Property-test the shared
      reward/type invariant over bounded finite inputs; no assertion depends
      on a randomly drawn case being present.
- [ ] In reproduction tests, demonstrate successful spawning with ordinary food
      alone and no reserve preparation; assert parent/offspring transfer
      accounting, target/population/age rejection before cost, and
      minimum-energy/zero/nonfinite/unaffordable-transfer rejection after cost.
      Existing later mutation and inheritance regressions continue to pass.
- [ ] Execute graph and VM founder tests, not only instruction-shape assertions:
      each profile's exact energy and age boundaries, priorities when food and
      reproduction are both available, transfer amounts, local Eat-then-Move,
      canonical Move-only fallback, forage-first Eat-then-Move fallback, correct
      typed metadata, and normal action-limit truncation (limit 1 keeps Eat).
      Cover all four unique cardinal maxima, ties including all-zero, and the
      historical adjacent-pair trap for movement and reproduction branches.
      Property-test cardinal selection over bounded finite food inputs against
      the N/E/S/W argmax oracle. Assert no founder second-food dependency and
      four unreferenced VM registers remain.
- [ ] In server integration tests, change `energy.costs.eat_reward_per_food`
      through the live update endpoint without restarting, observe config
      readback, and verify subsequent applied typed Eats use the new reward for
      both types. Assert removed keys are absent from config, snapshot/detail,
      food metadata, and rejection surfaces; obsolete config is rejected by
      existing validation. Frontend tests change/save Eat Reward and assert
      nutrition/reserve/yield controls and inspector values are absent.
- [ ] Run `cargo check --workspace --all-targets` after coherent Rust edits;
      focused core/server/frontend tests; `make roadmap-check` on document
      changes and before implementation handoff. Retain traced/untraced runtime,
      mutation, reproducibility, ecology, fertility, and typed-sensor coverage.
- [ ] After self-review, run fresh `MUTANTS_ITERATE=0 make rust-mutants`. Record
      command summary, output path, `fresh` run-mode evidence, and the full
      missed/timeout survivor list with each killed/equivalent/deferred resolution.
      No production edit to kill a mutant and no undocumented exclusions.
- [ ] Store `make bench PROFILE=gate FEATURE=remove-complementary-nutrition`
      at `docs/progress/features/remove-complementary-nutrition.json` and one
      closure `make bench PROFILE=goal FEATURE=remove-complementary-nutrition`
      at `docs/progress/features/remove-complementary-nutrition-goal.json`.
      Use the existing guarded commands without competing host work. Compare
      against T11.F08 and the pinned T11.F04 epoch before appending these paths
      to the corresponding existing `benchmark-series.json` closed arrays;
      `FEATURE` is the harness's report label, not a new roadmap ID.
- [x] Second goal-profile determinism run: Not applicable per the workflow's
      2026-09-05 decision; the ordinary reproducibility tests remain required.
- [ ] Complete Performance and Goal Impact with dated readings, all threshold
      crossings and their resolutions, and actual population behavior under
      restored defaults from the goal report.
- [ ] Fresh independent final review completed and findings recorded. The
      orchestrator runs `make check` for final content, commits, verifies any
      hook edits, and records the exact tested commit before integration.

## Performance and Goal Impact

This maintenance restores ordinary food as the single energy source used for
body maintenance and reproduction. It reaches creatures through consumed food
and parent-to-offspring energy transfer; it adds no cognition indicator.

Predeclared effects: reducing two default food planes to one and removing
reserve evaluation should reduce world/runtime work. Restoring two queued
foraging actions can increase `actions_applied` per creature-tick from one
toward two; removing the reserve gate can increase births and subsequent
mutation/evolved-controller work. Those are expected behavioral changes, not
evidence of identical historical efficiency or trajectories. This is not an
unbounded compute exemption: any severe measured crossing requires concrete
attribution and resolution through the same advisor before closure. Preserve
all thresholds, historical baselines, and their results; do not alter them or
tune unrelated parameters to turn a failure into a pass.

Pending measurements: gate deterministic counters and wall time per
creature-tick versus T11.F08 and T11.F04; one dated goal run's population per
seed, births, extinction/persistence and generation readings, cognition/diversity
indicators, founder/evolved mutation-neighborhood outcomes, and runtime costs.
The goal profile already uses production food defaults (1600x1600, 10,000
founders, seeds 11/22/33, 2,000 ticks); report those exact observation limits.
Founder-neighborhood cap remains 10 seconds per profile, evolved-neighborhood
cap 180 seconds summed across seeds, and total goal investigation threshold
15 minutes. No second goal run for determinism; reruns require changed content
or a concrete failed measurement, not ritual confirmation.

## Success Criteria

- [ ] All feeding, reproduction, founder, live-update, and removal contracts
      above are demonstrated by passing acceptance tests and current surfaces.
- [ ] Required verification, fresh survivor triage, benchmark/goal reports,
      advisory resolutions, and independent review are complete without waived
      checks or unresolved correctness blockers.
- [ ] This spec is Complete on main; `make check` exited 0 for the exact final
      committed content now on main; main is clean; the completed worktree and
      branch are removed, with integration evidence in the parent task.

## Notes for AI Agents

- Planning uses research-first-planning, spec-writing, and spec-review. Required
  Codex roles: Astra (`gpt-6-astra`) medium orchestrator, this persistent xhigh
  spec owner/advisor, one persistent low implementer, fresh high final reviewer.
  The orchestrator verified its model/effort from session metadata. No separate
  advisor is created; serialize spec edits with implementation.
- Readiness self-review on 2026-09-07: **Ready** after one revision, P1 0 /
  P2 1 resolved / P3 0. The orchestrator identified the omitted live mutation
  reference in the document list; it now explicitly preserves the
  EnergyCurrent/EnergyConsumedThisTick split distinction while removing reserve.
  Checked user scope, actual historical/current
  contracts, template structure, acceptance paths, and maintenance boundaries.
  This is not independent implementation validation; runtime behavior is still
  to be implemented and measured. Planning/readiness does not count as an
  advisor consultation. Later consultations occur before approach selection,
  after a repeated failure twice, and before implementation handoff.
- Record consultation count and decisive guidance, review counts by severity,
  remediation passes, requirement corrections, user interventions, and total
  task-specific usage when available. Current usage unavailable. No blocker
  identified during initial historical/code inspection.
