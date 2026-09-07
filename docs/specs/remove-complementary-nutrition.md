# Remove Complementary Nutrition

**Status**: Complete
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
  benchmark results retain their original meaning and acceptance results, with
  the user's explicit exception for a supersession marker on the original
  complementary-nutrition feature record and its archive-index entry.

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
   rewrite completed T11 records and archives. Add a dated note to the original
   [complementary-nutrition record](../prds/archive/complementary-nutrition-budget/master-prd.md)
   and its archive index stating that the user reversed the design decision,
   linked here for implementation and integration status. Preserve the original
   completed history without claiming that removal is already integrated.

## Implementation Tasks

- [x] Add failing behavior tests for restored defaults, feeding/reproduction,
      founder priorities/queues/directions, and live reward updates; record red
      evidence before the corresponding production edits.
- [x] Remove nutrition across core config/state/contracts/actions, runtime
      inputs and callers, mutation sampling, founder construction, and tests.
- [x] Restore the shared live reward through server configuration and frontend;
      remove obsolete projections, schemas, controls, and fixtures while
      preserving configurable multiple ordinary types.
- [x] Update live references and directly affected planned specs; inspect the
      diff for accidental historical-file restoration and unrelated changes.
- [x] Complete self-review, fresh mutation-survivor triage, stored benchmarks,
      advisor checkpoint, independent review, and any permitted remediation.

## Verification

- [x] Baseline `cargo test -p v3-core --test viability`: exit 0, 25 passed on the
      starting commit; orchestrator log
      `/tmp/remove-complementary-nutrition-baseline-viability.log`.
- [x] Run `cargo test -p v3-core --test viability` first after changing defaults,
      founders, or tick mechanics, before compile/focused checks. Preserve the
      existing production-economics survival/reproduction gates; replace
      complementary-food ablations with a one-ordinary-food scenario that
      replenishes energy, survives, and produces offspring. Do not boost costs,
      rewards, runtime limits, or weaken viability assertions to pass.
- [x] In config and frontend startup tests, assert exact one-food defaults,
      empty-list fallback, and restart serialization; no nutrition/yield keys
      are serialized. Test shared reward fallback and nonnegative finite
      normalization with proptest over the pure invariant.
- [x] In core action/tick tests, consume equal positive densities of two
      explicitly configured ordinary types and observe equal energy gains,
      selected-plane consumption, other-plane preservation, cap-before-cost,
      and normal empty/invalid selection outcomes. Property-test the shared
      reward/type invariant over bounded finite inputs; no assertion depends
      on a randomly drawn case being present.
- [x] In reproduction tests, demonstrate successful spawning with ordinary food
      alone and no reserve preparation; assert parent/offspring transfer
      accounting, target/population/age rejection before cost, and
      minimum-energy/zero/nonfinite/unaffordable-transfer rejection after cost.
      Existing later mutation and inheritance regressions continue to pass.
- [x] Execute graph and VM founder tests, not only instruction-shape assertions:
      each profile's exact energy and age boundaries, priorities when food and
      reproduction are both available, transfer amounts, local Eat-then-Move,
      canonical Move-only fallback, forage-first Eat-then-Move fallback, correct
      typed metadata, and normal action-limit truncation (limit 1 keeps Eat).
      Cover all four unique cardinal maxima, ties including all-zero, and the
      historical adjacent-pair trap for movement and reproduction branches.
      Property-test cardinal selection over bounded finite food inputs against
      the N/E/S/W argmax oracle. Assert no founder second-food dependency and
      four unreferenced VM registers remain.
- [x] In server integration tests, change `energy.costs.eat_reward_per_food`
      through the live update endpoint without restarting, observe config
      readback, and verify subsequent applied typed Eats use the new reward for
      both types. Assert removed keys are absent from config, snapshot/detail,
      food metadata, and rejection surfaces; obsolete config is rejected by
      existing validation. Frontend tests change/save Eat Reward and assert
      nutrition/reserve/yield controls and inspector values are absent.
- [x] Run `cargo check --workspace --all-targets` after coherent Rust edits;
      focused core/server/frontend tests; `make roadmap-check` on document
      changes and before implementation handoff. Retain traced/untraced runtime,
      mutation, reproducibility, ecology, fertility, and typed-sensor coverage.
- [x] After self-review, run fresh `MUTANTS_ITERATE=0 make rust-mutants`. Record
      command summary, output path, `fresh` run-mode evidence, and the full
      missed/timeout survivor list with each killed/equivalent/deferred resolution.
      No production edit to kill a mutant and no undocumented exclusions.
- [x] Store `make bench PROFILE=gate FEATURE=remove-complementary-nutrition`
      at `docs/progress/features/remove-complementary-nutrition.json` and one
      closure `make bench PROFILE=goal FEATURE=remove-complementary-nutrition`
      at `docs/progress/features/remove-complementary-nutrition-goal.json`.
      Use the existing guarded commands without competing host work. Compare
      against T11.F08 and the pinned T11.F04 epoch before appending these paths
      to the corresponding existing `benchmark-series.json` closed arrays;
      `FEATURE` is the harness's report label, not a new roadmap ID.
- [x] Second goal-profile determinism run: Not applicable per the workflow's
      2026-09-05 decision; the ordinary reproducibility tests remain required.
- [x] Complete Performance and Goal Impact with dated readings, all threshold
      crossings and their resolutions, and actual population behavior under
      restored defaults from the goal report.
- [x] Fresh independent final review completed and findings recorded;
      orchestrator `make check` passed for the reviewed implementation.
      The parent owns the final closure-content check, local commit, hook-edit
      verification, and exact tested-commit evidence before integration.

Implementation verification (2026-09-07):

- Behavioral red before production: `cargo test -p v3-core --test viability
  ordinary_food_alone_replenishes_energy_and_reproduces` failed on zero births;
  `cargo test -p v3-core --lib energy_only_` failed all three default/queue/reserve
  cases. Server live reward test failed with HTTP 422 unknown field; frontend
  startup test failed on two-food defaults. Logs: `/tmp/remove-nutrition-red-core.log`,
  `/tmp/remove-nutrition-red-viability.log`, `/tmp/remove-nutrition-red-server.log`,
  `/tmp/remove-nutrition-red-frontend.log`.
- `cargo test -p v3-core --test viability`: 24 passed, log
  `/tmp/remove-nutrition-viability-final.log`. The two removed complementary-food
  ablations are replaced by ordinary-food energy/reproduction acceptance; the
  production-economics assertions remain unchanged.
- `cargo check --workspace --all-targets`: exit 0,
  `/tmp/remove-nutrition-check-final.log`; `cargo clippy --workspace --all-targets
  -- -D warnings`: exit 0, `/tmp/remove-nutrition-clippy-1.log`.
- `cargo test -p v3-core`: 1175 unit tests passed, one existing ignored test,
  all integration tests passed (including reproducibility, typed ecology,
  priority reachability, temporal fixtures and viability), log
  `/tmp/remove-nutrition-core-final.log`.
- `cargo test -p v3-server`: unit suite and all 80 integration tests passed,
  `/tmp/remove-nutrition-server-2.log`. The first run's seven websocket tests
  required socket permissions; the approved unsandboxed rerun passed.
- Frontend `npm run lint`, `npm run test`, `npm run build`: exit 0; 277 tests
  passed. Logs `/tmp/remove-nutrition-lint-final.log`,
  `/tmp/remove-nutrition-frontend-final.log`, `/tmp/remove-nutrition-build-final.log`.
  Lint retains three existing warnings; build retains its existing chunk warning.
- `make roadmap-check`: exit 0, `/tmp/remove-nutrition-roadmap-final.log`.
  All 18 JSON example fences in the affected server protocol reference parse.
- Diff self-review completed: reused the current cardinal builder across Move
  and Reproduce, collapsed reserve-only runtime wrappers, removed unused battery
  reserve state/RNG draws and the obsolete nutrition-only PATCH guard, retained
  live-versus-consumed energy semantics, and strengthened cap-before-cost and
  blank action-bank assertions. No whole historical file restoration, dependency,
  compatibility layer, exclusion, or unrelated performance change was introduced.
  `git diff --check` passed. Fresh mutation closure, stored measurements, advisor checks, and independent
  review are complete; the parent records final integration verification.
- Initial fresh mutation run: `MUTANTS_ITERATE=0 make rust-mutants`, exit 0,
  **71 mutants tested in 6m: 5 missed, 34 caught, 32 unviable**, no timeouts.
  Log `/tmp/remove-nutrition-mutants-1.log`; output
  `/Users/istefanek/.local/share/petri-tools/mutants/remove-complementary-nutrition/mutants.out`.
  Initial full missed list and final resolutions:
  - `config/simulation.rs:206:5`: `default_food_types` replaced with
    `vec![Default::default()]` — **equivalent**, exactly the existing single
    `FoodTypeConfig::default()` value.
  - `mutation/sampling.rs:39:9`: delete arm 7 — test-only remediation added
    controlled-RNG selection of both energy introspection keys; **caught** in final run.
  - `simulation/actions/mod.rs:72:13`: replace `>` with `>=` in `apply_typed_eat`
    — test-only remediation verifies empty Eat does not clamp existing energy
    after a live maximum reduction; **caught** in final run.
  - `simulation/actions/reproduction.rs:61:9`: replace
    `ReproductionActionResult::as_key` with `"xyzzy"` — test-only remediation adds
    the public result-key property; **caught** in final run.
  - Same function/location replaced with `""` — same property; **caught** in final run.
- Remediation self-review found no production changes or new abstractions;
  reused existing StepRng and proptest. `cargo check --workspace --all-targets`
  passed (`/tmp/remove-nutrition-check-remediation.log`); `cargo test -p v3-core
  --lib` passed 1178 tests, one existing ignored test
  (`/tmp/remove-nutrition-core-remediation.log`). The second **fresh** `MUTANTS_ITERATE=0 make rust-mutants` run exited 0:
  **71 tested in 6m: 1 missed, 38 caught, 32 unviable, 0 timeouts**. The full
  final survivor list is only the equivalent `default_food_types` constructor
  above. Log `/tmp/remove-nutrition-mutants-final.log`; the adjacent
  `run-mode.txt` records `fresh`. No production edit, exclusion, threshold
  change, or incremental result was used to close survivors.

- Orchestrator `make check` found a stale CLI-only coverage assertion expecting
  0.27 (`/tmp/remove-nutrition-orchestrator-check-1.log`). Test-only remediation
  now asserts the exact single `[0.54]` coverage and shared mirror. All-target
  check passed (`/tmp/remove-nutrition-check-cli-remediation.log`); CLI unit
  suites passed 34 + 9 tests. The first integration run passed 16 and failed
  only its unchanged severe-reference assertion before the authorized gate
  epoch update (`/tmp/remove-nutrition-cli-remediation.log`). Self-review found
  no production/report/threshold edit or weakened assertion. Final **fresh** `MUTANTS_ITERATE=0 make rust-mutants` rerun exited 0:
  **71 mutants tested in 7m: 1 missed, 38 caught, 32 unviable, 0 timeouts**
  (`/tmp/remove-nutrition-mutants-closure.log`). `run-mode.txt` is `fresh`;
  the full survivor list remains only the equivalent constructor above.
  `cargo test -p v3-cli` then exited 0 (`/tmp/remove-nutrition-cli-final.log`),
  including the unchanged severe-reference assertion and the two-run gate
  determinism check. `make roadmap-check` and `git diff --check` passed at
  implementer handoff; source/test/report writes are complete for review.

- Fresh independent Astra (`gpt-6-astra`) high review: **P1 0 / P2 0 / P3 0**;
  no deferred findings and **0 post-review remediation passes**. Pre-review
  test-only remediation comprised one mutation-coverage pass (four behavioral
  survivors) and one stale CLI coverage-fixture pass. No production edits were
  made to kill mutants. Orchestrator `make check` exited 0 for the reviewed
  implementation (`/tmp/remove-nutrition-orchestrator-check-2.log`). The parent
  will record the final closure-content check and exact committed-content
  evidence in its task; no future pass or commit hash is asserted here.

- Parent closure-content `make check` exited 0
  (`/tmp/remove-nutrition-orchestrator-check-final.log`); commit
  `02e90f29010eabb2c65949dd3522f2531b14c9a9` matched tested tree
  `15f2887a2b27365c1b2cad8642864650c445f4aa`. After the user advanced main with
  documentation-only effort settings, the parent cleanly rebased onto
  `daeabf60659c2c73cfe84696a1bd0cf45a7bb39c`, producing branch head
  `ae96957fbac0c9a211c814f47ad9a4e036e02cc5` before this telemetry update.
  No behavior changed. The user explicitly waived another full check for this documentation-only
  rebase (intervention 6). The earlier pass remains the evidence for the tested
  implementation tree; it is not a claim that the full rebased final commit
  was tested. The parent records the final commit and integration in its task. Historical benchmark revision fields retain the
  original planning commit and uncommitted-implementation context.

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

Gate measured 2026-09-07T16:18:13Z using the guarded command; report
`docs/progress/features/remove-complementary-nutrition.json`, log
`/tmp/remove-nutrition-bench-gate.log`. The harness exited 3 (`make` exited 2),
with severe flags retained. Against T11.F08 / T11.F04 respectively:

| Per-creature-tick counter | Current | Change vs F08 | Change vs F04 |
| --- | ---: | ---: | ---: |
| Mesh hops | 2.027261 | +1.36% | +1.39% |
| VM steps | 22.751316 | -18.87% | -18.87% |
| Graph visits | 0.995234 | -0.48% | -66.81% |
| Plasticity updates | 0.011171 | +1142.60% severe | +1139.84% severe |
| Applied actions | 1.275525 | +27.55% flag | +27.55% flag |
| Births | 0.026780 | +2086.12% severe | +2202.67% severe |

Wall time is 0.001559831 ms/creature-tick (-68.85% / -62.86%), but total
simulation time rose from F08's 306.480 ms to 665.338 ms. Creature-ticks grew
61,211 to 426,545 and births 75 to 11,423. All seeds survived the 75-tick
observation, with final populations 3,105 / 3,107 / 3,026. The restored queues
and energy-only reproduction explain increased action/birth work. Founders
have no plasticity; descendant mutations supply it, while updates per birth
fell from 0.733 to 0.417. These counters count rule applications, not necessarily
nonzero weight changes or useful learning. F04's graph counter predates the
recorded graph-work definition change and is not a like-for-like efficiency
comparison.

Advisor consultation 3 accepted this attribution and the single goal run but
**did not close the severe crossings** at that checkpoint. References and
thresholds were preserved pending a bounded user decision; no economics tuning
or self-reference rerun was proposed. Reports record planning commit
`a291797210d22828fe1dc96384eef2d0e001b2af` plus the uncommitted implementation.

**Post-observation gate-cost acceptance (2026-09-07).** The user stated,
"I have decided these are good changes and acceptable," quoting the gate
birth/plasticity increases and normalized wall-time reduction. This explicitly
accepts the measured gate readings: births `0.026780` per creature-tick
(+2086.12% versus F08 / +2202.67% versus F04) and plasticity rule applications
`0.011171` (+1142.60% / +1139.84%). It does not accept the goal-profile result
or future regressions. Under the workflow's existing accepted-cost re-pin
mechanism, the gate epoch now points to this gate report, also appended to the
gate closed array as prepared closure bookkeeping. The original F04 and F08
reports, this report's severe comparisons and CLI 3 / make 2 outcome, and all
thresholds remain unchanged. The existing CLI comparison test must pass using
these accepted future gate references; its assertion is not weakened. No
report is regenerated against itself. This acceptance is post-observation,
not an assertion that the original predeclaration authorized an unlimited cost.

Goal measured once, 2026-09-07T16:26:34Z, using the guarded command; report
`docs/progress/features/remove-complementary-nutrition-goal.json`, log
`/tmp/remove-nutrition-bench-goal.log`. Exact production-default profile:
1600x1600, 10,000 founders, seeds 11/22/33, 2,000 ticks. The harness exited 3
(`make` exited 2); its original severe birth comparisons are retained. No
second goal run was made. The separate acceptance and future goal-reference
update are recorded below.

| Goal counter per creature-tick | Current | Change vs F08 | Change vs F04 |
| --- | ---: | ---: | ---: |
| Mesh hops | 2.077344 | -5.09% | -36.68% |
| VM steps | 62.803936 | +46.67% flag | -94.93% |
| Graph visits | 0.997884 | -0.77% | -83.38% |
| Plasticity updates | 0.037150 | -59.49% | -73.46% |
| Applied actions | 1.261295 | +23.46% flag | +11.66% flag |
| Births | 0.016622 | +117.20% severe | +103.93% severe |

Goal simulation time: 328.841 seconds versus F08's 708.666 seconds;
0.006562312 ms/creature-tick (-7.89% / -8.29%). Founder observation took
55.189 ms (10-second cap), evolved observation 3,027.405 ms summed across
seeds (180-second cap), and final-state observation 481.264 ms. The full
measurement stayed below the 15-minute investigation threshold. Total births
were 832,937 versus F08's 761,230 (+9.4%), with 50,110,549 versus 99,469,439
creature-ticks (-49.6%): this lower denominator drives the normalized +117.2%
birth rise.
No unrelated parameters were tuned.

| Goal seed | Births | Minimum population | Final population | Generation median / max | Final mean energy |
| --- | ---: | ---: | ---: | ---: | ---: |
| 11 | 277,287 | 512 | 718 | 24 / 54 | 72.166 |
| 22 | 285,177 | 1,084 | 4,517 | 41 / 63 | 33.943 |
| 33 | 270,473 | 830 | 1,138 | 48 / 60 | 43.607 |

All seeds reached the 100,000 population cap at ticks 52/53/53, contracted,
and remained alive through tick 2,000; seed 22 recovered from its later trough.
Recorded plateau populations are 605.568 / 1,997.350 / 1,228.138. F08 final
populations were 11,627 / 12,402 / 12,171 and generation medians 22 throughout.
These observations demonstrate ordinary-food survival and reproduction over
the stated window, not long-run stability or a better population trajectory.

Surviving founder clades are 137/137/134, entropy 2.602116/1.086756/2.158890
nats, lower than F08's 211/191/207 and 4.374506/4.027032/4.365327. Reachable
structure size is min 44, median 81, mean 85.491919, max 283 (F08 median 106,
mean 126.924144, max 482). Shared-memory sensitivity and previous-slot temporal
sensitivity are zero in every seed. Operator-state sensitivity counts are
0/1/0; persisted-output sensitivity counts 3/8/8. These are final-state
counterfactual action observations, not evidence of useful learning. Undefined
cognition/evolutionary-activity indicators remain Undefined.

Founder mutation-neighborhood event-bearing births changed behavior in 94/208
trials (45.1923%), with no dead mutants; F08 was 112/208 (53.8462%). Evolved
pooled event-bearing trials changed 378/1,100, 320/1,100, 253/1,100
(34.3636% / 29.0909% / 23.0000%), with dead fractions 0.8182% / 0% / 3.0909%.
F08 changed fractions were 32.3636% / 33.4545% / 36.0000%. These bounded battery
observations retain the existing trial counts and do not establish adaptive
novelty. The obsolete reserve draw was removed from the battery, so individual
historical scenario trajectories are not expected to match.

Advisor consultation 4 found no additional correctness blocker or experiment.

**Post-observation goal-cost acceptance (2026-09-07).** After the separate goal
cost clarification, the user stated, "The birth rate result is acceptable too."
This accepts the measured goal birth rate of `0.016622` per creature-tick,
`+117.195871%` versus F08 and `+103.925899%` versus the F04 epoch. The total-birth
increase of 9.4% and creature-tick decline of 49.6% above remain part of its
interpretation; the accepted rate is not a claim of proportionally greater
total reproduction or better population stability. Following the existing
accepted-cost mechanism, the goal epoch now points to this goal report and its
path is appended to the goal closed array as prepared closure bookkeeping.
Historical report files, original severe comparisons, CLI 3 / make 2 outcomes,
and thresholds remain intact. No second goal run, self-comparison regeneration,
production tuning, or blanket acceptance of future costs is authorized by this
decision. Consultations 5 and 6 applied the separate gate and goal acceptances;
No cost-acceptance blocker remains. Independent review and the reviewed
implementation check passed; the parent owns final closure-content verification
and integration.

## Success Criteria

- [x] All feeding, reproduction, founder, live-update, and removal contracts
      above are demonstrated by passing acceptance tests and current surfaces.
- [x] Required verification, fresh survivor triage, benchmark/goal reports,
      advisory resolutions, and independent review are complete without waived
      checks or unresolved correctness blockers.

The parent task owns final integration evidence under the workflow: it must
show this Complete spec on main, `make check` exit 0 for the exact final
committed content now on main, and removal of the completed worktree and branch.
This local closure document does not assert that those later operations have
already occurred.

At closure preparation, main remained at
`3871ad1173f0197a3ca06d6f719ddef456c5dde0` with three unrelated, nonoverlapping
changes. The user explicitly authorized merging while preserving them:

- Modified `docs/README.md`.
- Untracked `docs/strategy/config-panel-runtime-apply-audit-2026-09-07.md`.
- Untracked `docs/strategy/config-panel-runtime-apply-audit-2026-09-07.results.json`.

For this integration only, the clean-main condition excludes these three
authorized changes. Preserve their contents and tracked/untracked state; do
not commit, stash, discard, or remove them as part of this maintenance. All
other verification, exact-tested-commit, fast-forward, and cleanup requirements
remain in force, with results reported in the parent task.

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
  That readiness review preceded implementation; the independent implementation
  review and measured evidence are recorded below. Planning/readiness does not count as an
  advisor consultation. Later consultations occur before approach selection,
  after a repeated failure twice, and before implementation handoff.
- Record consultation count and decisive guidance, review counts by severity,
  remediation passes, requirement corrections, user interventions, and total
  task-specific usage when available. Current usage unavailable. No blocker
  identified during initial historical/code inspection.
- User intervention 1 / requirement correction, 2026-09-07: the user requested
  a marker on the original reproductive-food feature recording that the design
  was reconsidered and rejected. This authorizes the narrow archive annotation
  above while preserving the original history. The master PRD and archive index
  now mark the decision reversed and link here for removal status. At the time
  of this annotation, removal remains in progress and is not integrated.

- User intervention 2 / gate-cost acceptance, 2026-09-07: the user's approval
  quoted only the gate findings. Consultation 5 applied that bounded acceptance
  through the existing gate epoch/closed-series mechanism without another
  permission request. Goal acceptance was not inferred at that checkpoint.
- User intervention 3 / goal-cost acceptance, 2026-09-07: the user separately
  accepted the goal birth-rate result. Consultation 6 recorded its exact measured
  rate and comparisons and applied only goal epoch/closed-series bookkeeping,
  preserving the original reports, thresholds, and population interpretation.
- User intervention 4 / integration contract correction, 2026-09-07: after the
  parent reported the three unrelated main-worktree changes listed above, the
  user stated, "That's fine, they are unrelated changes, you can just merge."
  Consultation 7 records authorization for fast-forward integration with those
  files preserved, waiving only their clean-main condition. No additional
  permission is required; this is not authority to alter unrelated work or
  waive tests, review, exact-commit verification, or cleanup.
- User intervention 5 / integration update, 2026-09-07: the user stated,
  "I just snuck in a small commit on main, but it should still be safe to just
  merge our changes." The parent completed the clean rebase described above.
  The original requested model/effort settings remain authoritative for this
  already-launched task; the new workflow defaults apply to future launches.
  No behavior conflict required another advisor consultation.
- User intervention 6 / verification override, 2026-09-07: the user stated,
  "We don't need to recheck eveything, just merge" and explicitly waived a
  further full check after the documentation-only rebase. The original final
  `make check` exit 0 remains recorded; no full rebased-commit test is claimed.
  Total user interventions: **6**; advisor consultations remain **7**.
- Advisor consultations: **7**. (1) Accepted owned PushAction
  metadata capture, one ExecuteActionQueue after Eat/Move, shared full-cardinal
  selection for reproduction, dedicated zero register, age-zero half-tick gate,
  and four spare registers. (2) Accepted moving a newly added ordinary test out
  of the existing proptest macro after the same test-authoring parse failure
  recurred twice; obtained behavioral red afterward. No acceptance change or
  optional scope expansion. (3) Accepted proceeding with one goal measurement
  against unchanged references after gate cost escalation; retain severe flags
  pending a bounded cost decision, distinguish lower normalized wall cost from
  higher total cost, and do not infer useful learning from plasticity counts.
  No epoch or closed-series edits were authorized at that checkpoint.
  (4) Pre-handoff advisor
  found the evidence sufficient, with no additional correctness blocker or
  experiment. Accepted explicit total-birth/creature-tick attribution and the
  bounded user decision described above; independent review and final checks
  remain required. (5) Recorded the user's post-observation gate acceptance and
  authorized gate-only reference bookkeeping; retained the unmodified CLI gate
  assertion, historical reports, and then-pending goal-cost decision.
  (6) Recorded the separate goal acceptance and goal-only reference update;
  no cost-acceptance blocker remains. (7) Applied the user's narrow integration
  override and assigned final merge/cleanup evidence to the parent task without
  claiming those operations had already happened. Required verification and
  integration evidence remain in force.
