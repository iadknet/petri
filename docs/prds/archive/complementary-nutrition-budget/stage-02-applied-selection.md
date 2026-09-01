# Stage 02 — Applied Selection

- Status: Complete
- Depends on: stage-01-nutrition-contracts.md
- Master: [Master PRD](master-prd.md)

## Goal

Make complementary nutrition consequential in applied simulation behavior: actual typed
consumption changes both creature budgets, successful reproduction consumes reserve,
and the production founder can meet both requirements using only ordinary cognition.

## Scope

- Apply per-food-type metabolic and reserve yields from the density actually consumed.
- Remove the obsolete untyped `apply_eat` helper and migrate its owned tests/callers to
  explicit typed Eat.
- Clamp energy and reserve to their configured capacities and preserve action-cost
  ordering.
- Add an explicit insufficient-reserve reproduction outcome and atomic successful debit.
- Establish the two complementary production-default food types and reserve defaults.
- Update the canonical founder and mutation/action metadata paths to support both foods
  and live reserve state.
- Add unit, integration, ablation, and viability tests that prove behavioral pressure and
  founder solvability.

## Non-Goals

- Rewarding genome size, functional complexity, mutation viability score, plasticity,
  memory use, or any chosen implementation strategy.
- Requiring a creature to alternate on a prescribed schedule; any behavior that
  acquires both applied resources remains valid.
- Long-run evolutionary claims, benchmark thresholds for evolved complexity, or
  automatic parameter optimization.
- Seasonal resource changes or more than two production-default nutritional roles.
- Runtime protocol/UI work, except core result/state types required to compile.

## Inputs and Existing-Code Interactions

- Stage 01 supplies normalized yields/capacity/cost, creature reserve, and the live
  introspection key.
- `crates/v3-core/src/simulation/actions/mod.rs` owns `apply_typed_eat`; its return shape
  and tick caller must preserve the actual consumed amount already recorded by applied
  action logging.
- `crates/v3-core/src/simulation/actions/reproduction.rs` owns the canonical ordered
  reproduction sequence, rejection counters, mutation, state inheritance, and spawn.
- `crates/v3-core/src/simulation/tick.rs` owns action ordering, failure penalties,
  action-log records, and post-action plasticity reward derived from energy delta and
  applied outcomes.
- `crates/v3-core/src/creature/founder.rs` owns canonical founder inputs and action
  priorities. It must use typed food-0/food-1 sensors plus live energy/reserve, with no
  privileged state.
- `crates/v3-core/tests/viability.rs` is the mandatory first test when defaults, founder,
  or tick-loop behavior changes and the post-change liveness gate.

## Boundaries and Abstraction Layers

- World consumption remains a density-plane operation. Action application looks up the
  selected `FoodTypeConfig` and translates only the returned consumed amount into
  creature state.
- There is no type-erasing Eat fallback: callers must supply an
  `OrdinaryFoodTypeId`. Removing the legacy helper is an intentional internal API break.
- Missing/out-of-range types and empty cells yield no nutrition. Existing Eat cost and
  failed-action semantics still apply exactly once.
- Reproduction ordering is:
  1. resolve/validate target;
  2. enforce population cap;
  3. enforce minimum age;
  4. enforce reserve availability;
  5. charge the existing adjusted reproduce energy cost;
  6. enforce energy minimum and valid transfer;
  7. debit transfer and reserve once;
  8. construct/mutate/place the zero-reserve child and record success.
- Reserve is unchanged on every rejection, including an energy rejection after the
  energy action cost. Existing failed-action penalties remain tick-owned.
- The founder is an ordinary genome and must solve the same sensor/action contract as
  descendants.

## Separation of Concerns and Decomposition

Typed Eat and Reproduce are the two mutation points of the new state and belong in one
behavior stage so tests can prove the full acquire-and-spend loop. Default/founder
changes are included because a production pressure that the baseline population cannot
solve is not shippable. Transport projection remains separately owned by Stage 03.

## Tech Debt and Spaghetti-Code Implications

- Preserve a single path for typed Eat; do not duplicate nutritional arithmetic in the
  tick loop, founder, stats, or server.
- Exhaustively add `RejectedNutritionConstraints` to action-result matches and stable
  reason mappings. A local helper for repeated rejection counters is acceptable only if
  it reduces this feature's duplication without redesigning unrelated actions.
- Avoid special-casing food indices outside the production-default builder and founder
  policy. Action application must work for any configured type count.
- Do not add an evolutionary test that passes by reading a complexity metric. Behavioral
  ablation tests are the deterministic proof of selection pressure.
- Apply the detailed Rust rules named in Stage 01, especially float tolerances, clamping,
  exhaustive enum matching, descriptive AAA tests, and avoiding speculative generic
  nutrient abstractions.

## Documentation Impact and Synchronization

Update `docs/reference/v3-tick-orchestration-spec.md` for typed Eat yields and ordered
reproduction application. Update `docs/reference/v3-reproduction-spec.md` for the reserve
gate/debit and zero-reserve offspring. Update
`docs/reference/v3-startup-seeding-spec.md` for two default food roles and the ordinary
founder policy. Cross-check the world/config/sensor specs from Stage 01.

## Implementation or Decision Tasks

- [x] As the first implementation command before edits, run
  `cargo test -p v3-core --test viability` and record the baseline result here.

Baseline evidence (2026-09-01): `cargo test -p v3-core --test viability` passed
21 tests with 0 failures before production edits.

Post-contract evidence (2026-09-01): `cargo test -p v3-core --test viability --
--test-threads=1` passed 25 tests with 0 failures after typed nutrition,
reproduction gating, defaults, and founder updates. The remediation rerun also
passed `cargo test -p v3-core --lib --quiet` (993 tests) and `cargo check -p
v3-core --all-targets`. Direct regressions cover the exact fractional reserve
predecessor gate, independent typed seeding, deficient-resource founder choice,
live-reserve mesh execution, invalid typed Eat, reserve atomicity for every
rejection and repeated successful debits, applied typed Eat/spawn behavior with
mutations disabled, and maintenance-only/reproductive-only ablations.
- [x] Write failing typed-Eat tests for metabolic-only food, reserve-only food,
  fractional actual consumption, empty cells, invalid type IDs, and energy/reserve
  capacity clamping.
- [x] Apply yields in the existing typed-Eat owner from the actual consumed amount; do
  not infer yield from action intent or pre-consumption density.
- [x] Delete `apply_eat` and migrate every owned test/caller to `apply_typed_eat` with an
  explicit type ID; do not silently map untyped eating to type 0.
- [x] Write failing reproduction tests for insufficient reserve, every rejection's
  reserve atomicity, successful exact debit, repeated births, and zero-reserve child
  initialization.
- [x] Add `RejectedNutritionConstraints` and implement the ordered gate/debit contract;
  update rejection counters, action-log outcomes, tick failure handling, and all
  exhaustive result mappings.
- [x] Write failing default-config tests, then configure maintenance food and
  reproductive food plus capacity/cost exactly as recorded in the master.
- [x] Write failing founder tests showing it reads both typed-food families and the live
  reserve input, seeks whichever budget is deficient, and attempts reproduction only
  when both gates can succeed.
- [x] Update the canonical founder with ordinary graph/VM inputs and typed Eat actions;
  do not provide hidden state or a non-genomic behavioral branch.
- [x] Add a deterministic behavior harness with mutations disabled that records actual
  typed Eat and spawn outcomes for the canonical founder.
- [x] Add two explicit one-resource ablations: maintenance-only may acquire energy but
  never reserve/spawn; reproductive-only may acquire reserve but cannot replenish
  energy or satisfy the energy/spawn contract.
- [x] Use a small documented fixed-seed set for founder/default assertions. If the
  initial numeric defaults fail liveness, tune only the redistributed coverages, typed
  yields, capacity, or reserve cost; keep strict complementarity, preserve approximate
  prior metabolic capacity, and record before/after evidence.
- [x] Synchronize the stage-owned durable behavior and startup specifications.

## Verification and Observable Success Criteria

- [x] Baseline `cargo test -p v3-core --test viability` result is recorded before edits.
- [x] Focused action tests prove state delta equals `actual_consumed * typed_yield`
  within a documented float tolerance, including fractional and clamped cases.
- [x] Focused reproduction tests prove reserve gates spawning, all rejections preserve
  reserve, success debits exactly one configured cost, and children start at zero.
- [x] Tick/action-log tests prove recorded type, amount, and result describe the action
  actually applied; cognition/plasticity continues to use live post-action state.
- [x] With mutations disabled and fixed seeds, the canonical production founder
  consumes both types and produces at least one child within the test horizon.
- [x] Under the same harness, maintenance-only produces zero children because reserve
  remains zero; reproductive-only produces zero children because metabolic energy is
  never replenished. Assertions inspect applied creature state/actions, not a proxy
  score.
- [x] Immediately after behavior/default/founder changes,
  `cargo test -p v3-core --test viability` passes and the result is recorded.
- [x] `cargo test -p v3-core simulation::actions` passes.
- [x] `cargo test -p v3-core simulation::tick` passes.
- [x] `cargo test -p v3-core creature::founder` passes.
- [x] Affected durable documentation is created, updated, or synchronized, or a no-change rationale is recorded.

Behavior evidence (2026-09-01): canonical fixed-seed founder, maintenance-only,
and reproductive-only scenarios pass with applied state/action assertions. Full
`cargo test -p v3-core --lib --quiet` passes 997 tests; the viability gate passes
25 tests after Stage 02 behavior/default/founder changes and remediation. The
genome/VM founder tests cover full-cardinal argmax for both typed rings, including
stable north-first ties, and the tick test
`tick_action_log_records_applied_food_type_amount_and_result` proves applied type,
amount, and result.

## Current Status

Complete. Sol review findings for the founder reserve boundary, independent typed
seeding/deficiency seeking, and direct applied-behavior evidence were corrected and
verified. Stage 03 may proceed with the corrected applied behavior.
