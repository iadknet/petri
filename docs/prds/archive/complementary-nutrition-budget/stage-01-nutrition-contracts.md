# Stage 01 — Nutrition Contracts

- Status: Complete
- Depends on: None
- Master: [Master PRD](master-prd.md)

## Goal

Establish one normalized configuration contract, one creature-owned reserve, and one
live cognition input for complementary nutrition, without yet making Eat or Reproduce
apply those values.

## Scope

- Add required per-type metabolic and reproductive yields to `FoodTypeConfig`.
- Add required startup-owned reserve capacity/cost configuration to
  `SimulationConfig`.
- Remove the global ordinary-food energy reward from `EnergyCostsConfig` and update
  owned config constructors/fixtures to the new authoritative fields.
- Add reproductive reserve to creature runtime state and initialize founders and
  offspring to zero.
- Add and resolve `ReproductiveReserveCurrent` as dynamic introspection.
- Make mutation input-reference sampling capable of selecting the new input.
- Add TDD coverage for validation, serialization, initialization, resolution, and
  mutation discoverability.

## Non-Goals

- Applying either yield during Eat.
- Enforcing or consuming reserve during Reproduce.
- Changing founder decisions or proving default population viability.
- Server/frontend projection work, aggregate metrics, or durable documentation outside
  the directly owned config/sensor contracts.
- A generic nutrient container, nutrient identifiers, target ratios, or digestion.

## Inputs and Existing-Code Interactions

- `crates/v3-core/src/config/simulation.rs` owns `FoodTypeConfig`,
  `EnergyCostsConfig`, `SimulationConfig::default`, normalization, and config tests.
  It must become the only authority for typed yields and reserve bounds.
- `crates/v3-core/src/creature/state.rs` owns live creature lifecycle state and every
  constructor path used by startup and reproduction.
- `crates/v3-core/src/contracts/inputs.rs` owns the serializable introspection enum;
  `crates/v3-core/src/runtime/inputs.rs` owns live value resolution.
- `crates/v3-core/src/mutation/sampling.rs` and
  `crates/v3-core/src/mutation/input_ref/` own random input discovery and
  food-type-aware mutation.
- Full-config literals and fixtures across `crates/v3-core`, `crates/v3-server`,
  `crates/v3-cli`, and `frontend` will fail until they use the required new schema.
  Update them mechanically in this stage, but defer behavior/UI assertions to their
  owning stages.

## Boundaries and Abstraction Layers

- Configuration values are normalized at the existing config boundary. Reject or
  replace NaN/infinity/negative values before runtime arithmetic.
- Reserve is a concrete `f32` adjacent to energy because capacity comes from runtime
  config. Do not add a generic nutrient trait or collection. The owning action paths
  will clamp it at mutation points in Stage 02.
- Dynamic introspection reads `CreatureState::reproductive_reserve` directly at node
  execution. It must not read a tick-start snapshot.
- Input mutation exposes the enum variant with the same reachability and configured
  food-count rules as existing inputs; it does not bias selection toward a desired
  controller.

## Separation of Concerns and Decomposition

This stage is the compile-time and data-contract foundation. It is separately testable
without changing tick semantics and allows Stage 02 to focus on action ordering and
ecological behavior. Further splitting would scatter one small state contract across
multiple incomplete builds.

## Tech Debt and Spaghetti-Code Implications

- Delete `eat_reward_per_food` rather than deprecating it. Two reward sources would
  create ambiguous precedence.
- Prefer existing config normalization helpers and explicit fields over a new framework.
- Update all exhaustive matches for the owned introspection enum. Do not hide the new
  variant behind wildcard arms.
- Review these Rust-skill rule files before implementation and apply them where relevant:
  - `.agents/skills/rust-skills/rules/type-newtype-validated.md`
  - `.agents/skills/rust-skills/rules/num-saturating-clamp.md`
  - `.agents/skills/rust-skills/rules/num-float-compare.md`
  - `.agents/skills/rust-skills/rules/pat-exhaustive-enum.md`
  - `.agents/skills/rust-skills/rules/own-borrow-over-clone.md`
  - `.agents/skills/rust-skills/rules/anti-over-abstraction.md`
  - `.agents/skills/rust-skills/rules/doc-all-public.md`
  - `.agents/skills/rust-skills/rules/test-descriptive-names.md`
  - `.agents/skills/rust-skills/rules/test-arrange-act-assert.md`

## Documentation Impact and Synchronization

Update `docs/reference/v3-runtime-config-spec.md` for the schema break, reserve
constraints, startup-only ownership, and removal of the global reward. Update
`docs/reference/v3-world-grid-spec.md` for per-food-type yields. Update
`docs/reference/v3-sensor-spec.md` for the live reserve input. Stage 03 performs the
final cross-document/example audit.

## Implementation or Decision Tasks

- [x] Before production edits, write failing config tests for required yield fields,
  finite/nonnegative normalization, positive capacity, positive cost, cost bounded by
  capacity, and the absence of `eat_reward_per_food` in the serialized schema.
- [x] Add `metabolic_energy_yield` and `reproductive_reserve_yield` to
  `FoodTypeConfig`; normalize every configured type without losing name/color/density
  semantics.
- [x] Add `NutritionConfig` with capacity/cost to `SimulationConfig`; make both fields
  startup-only and document all new public items.
- [x] Remove `EnergyCostsConfig::eat_reward_per_food` and update owned config literals,
  serde tests, server fixtures, CLI fixtures, and frontend TypeScript config shapes.
- [x] Write failing state-construction tests proving founders and children begin with
  exactly `0.0` reserve, then add `CreatureState::reproductive_reserve` to every
  construction path.
- [x] Write a failing runtime-input test proving post-construction reserve changes are
  observed live, then add `ReproductiveReserveCurrent` and resolve it without a cached
  snapshot.
- [x] Update mesh annotations, genome/input analysis, serialization, and every
  exhaustive owned-enum match for the new input.
- [x] Write a failing fixed-seed or bounded-sampling test, then include the reserve input
  in `random_input_reference_for_food_types` without regressing non-default typed-food
  sampling.
- [x] Synchronize the stage-owned durable specs and record final normalized defaults.

## Verification and Observable Success Criteria

- [x] Focused config tests prove invalid yields/capacity/cost cannot reach runtime math
  and serialized config has exactly one typed-yield authority.
- [x] Focused creature-state tests prove all current construction paths initialize
  reserve to zero.
- [x] Focused runtime-input tests prove `ReproductiveReserveCurrent` observes a live
  state change during execution rather than a tick-start copy.
- [x] Mutation/input-reference tests prove the new introspection key and both food-type
  references are discoverable; use deterministic seeds only for reproducible sampling
  assertions.
- [x] `cargo test -p v3-core config` passes.
- [x] `cargo test -p v3-core runtime::inputs` passes.
- [x] `cargo test -p v3-core mutation::input_ref` passes.
- [x] `cargo test -p v3-core mutation::sampling` passes.
- [x] `cargo test -p v3-core creature::state` passes.
- [x] Affected durable documentation is created, updated, or synchronized, or a no-change rationale is recorded.

## Current Status

Complete. Contract implementation, focused Rust verification, and owned durable-spec
synchronization are complete. Repository-wide and runtime-surface gates remain in
Stages 02–03.
