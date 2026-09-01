# Complementary Nutrition Budget — Master PRD

- Status: Complete
- Owner: Unassigned
- Created: 2026-09-01
- Review Status: APPROVED
- Review Count: 2

## Goal

Create one bounded, applied ecological pressure that makes a single-resource reflex
insufficient for reproductive success. Petri's production world will contain two
ordinary-food types with complementary effects: maintenance food restores metabolic
energy, while reproductive food builds a live reproductive reserve. A creature must
use both resources, and reproduction consumes the reserve.

The intended outcome is not to award complexity directly. It is to make resource
identity and internal state consequential to survival and reproduction, so selection
can favor conditional foraging behavior implemented by the existing graph/VM genome.

## Scope

- Replace the global ordinary-food energy reward with per-food-type applied yields:
  metabolic energy and reproductive reserve.
- Remove the obsolete untyped `apply_eat` application path and migrate its owned
  tests/callers to typed Eat so no type-erasing nutrition authority remains.
- Add one bounded reproductive-reserve value to each creature, initialized to zero,
  updated only by successful typed eating, and consumed only by successful reproduction.
- Add startup-owned reserve capacity and reproduction-cost configuration.
- Make insufficient reserve a distinct reproduction rejection and expose the live
  reserve to cognition through dynamic introspection.
- Change production defaults to two strictly complementary food types and update the
  canonical founder so it can sense, seek, and consume both before reproducing.
- Update mutation input-reference sampling so descendants can discover the reserve
  input as well as both typed-food inputs.
- Surface the applied state and food-yield metadata in existing runtime inspection and
  configuration paths.
- Synchronize affected durable specifications and verify the feature with TDD,
  behavior-focused tests, the viability-first gate, and `make check`.

## Non-Goals

- A general N-dimensional nutrient vector, geometric intake target, health system,
  digestion model, or species-specific metabolism.
- Seasonal, cyclic, stochastic, or climate-driven food availability.
- Communication, predation redesign, mating, sexual reproduction, trophic levels, or
  resource production by creatures.
- Direct rewards or penalties based on genome size, novelty, viability score,
  functional-complexity telemetry, or any other proxy fitness metric.
- A long-run claim that complex cognition or open-ended evolution has already emerged.
  This iteration establishes a necessary applied pressure; sustained evolutionary
  outcomes remain an empirical question.
- Backward-compatible aliases or migrations for `energy.costs.eat_reward_per_food` or
  old serialized startup payloads. Update owned callers, fixtures, and documentation.
- New aggregate complexity or diversity dashboards.

## Inputs and Existing-Code Interactions

### Problem and repository evidence

Research and repository inspection were performed on 2026-09-01.

- `crates/v3-core/src/config/simulation.rs` already defines typed ordinary food, but
  production defaults configure one type and every type receives the same global
  `energy.costs.eat_reward_per_food` value.
- `crates/v3-core/src/simulation/actions/mod.rs` applies typed consumption to the
  selected density plane, then converts the actual consumed amount into only scalar
  energy. Food identity therefore has no fitness consequence beyond spatial density.
- `crates/v3-core/src/creature/founder.rs` provides a small, effective founder that
  senses and eats food type 0, gates reproduction on energy/age, and has no reason to
  consider another resource. Alternate founder profiles also target type 0.
- `crates/v3-core/src/simulation/actions/reproduction.rs` gates reproduction on target,
  population, age, and scalar energy. No second acquired resource is required.
- `crates/v3-core/src/contracts/inputs.rs` and
  `crates/v3-core/src/runtime/inputs.rs` expose current energy live, while
  `crates/v3-core/src/mutation/sampling.rs` can already sample configured typed-food
  inputs. The cognition substrate needed for state-dependent food choice exists.
- Per-node and per-opcode runtime costs already charge computation. Adding direct
  complexity rewards would duplicate or oppose existing applied economics.
- `crates/v3-core/tests/viability.rs` establishes short-horizon survival,
  reproduction, and phenotype-divergence checks. It does not establish long-run
  ecological diversity or cognitive complexity.
- Durable goals GP-01 through GP-04 call for richer cognition, explicit boundaries,
  high-confidence behavior, and truthful observability. The target architecture keeps
  state in the creature/config owners and action semantics in the tick/application
  layer.
- Archive-only Git and retired planning evidence shows repeated work on typed food,
  reachability-aware mutation, plasticity, complexity costs, and collapse diagnostics.
  It also records seasonality as an old idea. This evidence explains the current
  affordances but is not current workflow guidance.

The collapse risk is therefore a selection-pressure problem more than a missing
representation problem: scalar energy and reproduction can be solved by a compact
type-0 forage/reproduce program, while additional computation continues to cost energy.

### External research

- Senior et al. (2015), [Evolving Nutritional Strategies in the Presence of
  Competition: A Geometric Agent-Based Model](https://doi.org/10.1371/journal.pcbi.1004111),
  evolves agents that switch among nutritionally imbalanced but complementary foods.
  Nutritional state determines breeding eligibility, and competition changes the
  evolved dietary strategy. This is the closest established design pattern.
- Corrales-Carvajal et al. (2016), [Internal states drive nutrient homeostasis by
  modulating exploration-exploitation trade-off](https://doi.org/10.7554/eLife.19920),
  shows that amino-acid and reproductive state alter food-patch decisions and global
  exploration. It supports exposing the live reserve to organism cognition.
- Liu et al. (2024), [Mating reconciles fitness and fecundity by switching diet
  preference in flies](https://doi.org/10.1038/s41467-024-54369-w), reports a
  state-dependent switch between foods that favor maintenance/lifespan and foods that
  favor fecundity. It supports separating maintenance and reproductive effects rather
  than treating all intake as interchangeable energy.
- Canino-Koning, Wiser, and Ofria (2019), [Fluctuating environments select for
  short-term phenotypic variation leading to long-term
  exploration](https://doi.org/10.1371/journal.pcbi.1006445), finds that cyclic
  environments can move Avida populations toward more evolvable mutational
  neighborhoods. This supports seasonality as a credible follow-up, not its inclusion
  in this iteration.
- Soros and Stanley (2019), [Open-Endedness for the Sake of
  Open-Endedness](https://doi.org/10.1162/artl_a_00289), argues that long-running or
  oscillating evolution alone is insufficient and highlights endogenous niches,
  nontrivial reproduction criteria, organism-controlled interaction, diversity, and
  complexity. This supports applied ecological criteria over proxy scoring.

These sources establish a mechanism pattern and evaluation cautions; they do not
predict Petri's long-run outcome or provide a reusable Rust component.

### Alternatives considered

| Alternative | Leverage | Material tradeoff | Decision |
| --- | --- | --- | --- |
| Complementary typed-food nutrition (selected) | Extends typed density planes, Eat metadata, sensors, mutation sampling, and reproduction. Makes both food identity and an internal state behaviorally necessary. | Adds one state variable and requires founder/default calibration. It creates pressure for conditional behavior but cannot guarantee evolutionary complexity. | Select for this iteration. |
| Type-specific seasonal growth or yield | Temporal change can favor plasticity, memory, and evolvability; external Avida evidence makes it credible. | A simple controller can follow currently abundant food without memory. Period, phase, amplitude, and startup bottlenecks add a larger calibration surface before food types have distinct organism-level value. | Strongest alternative; defer until complementary nutrition creates meaningful types. |
| Direct novelty/functional-complexity reproduction bonus | Fast, custom selection shaping and easy to measure. | Rewards a proxy that can be gamed by unused structure, couples telemetry to fitness, and risks preserving costly code rather than useful cognition. | Reject. |
| General geometric nutrient target | More extensible and closer to full nutritional-geometry models. | Adds vector state, target-distance semantics, excess/deficit policy, UI, and tuning that are not needed to test the two-resource hypothesis. | Reject for this bounded iteration. |

No external library is adopted. The selected mechanism is a few finite scalar state
transitions and belongs in Petri's existing config, creature, input, and action owners.

### Decision criteria

1. Selection pressure must arise from successfully applied world interactions and
   reproduction, never from a diagnostic score.
2. A controller that ignores either default food type must be unable to complete both
   maintenance and reproduction.
3. The existing genome can observe every state needed to solve the task, and mutation
   can discover those observations/actions.
4. The feature must fit one implementation iteration without a generalized metabolism
   framework or a new dependency.
5. Default founders must remain viable, and behavioral assertions must be reproducible
   only where a fixed seed is required.
6. Operator-visible values must be projections of live creature state or actual applied
   consumption.

### Research-first completion gate

- [x] The problem, constraints, and decision criteria are explicit.
- [x] Existing mechanisms and archive evidence were inspected before proposing custom work.
- [x] Current primary/authoritative sources were recorded with links and research date.
- [x] At least two credible alternatives and their material tradeoffs were compared.
- [x] The recommendation is traceable to evidence, and remaining uncertainty is named.

## Boundaries and Abstraction Layers

- Configuration owns finite nonnegative yield, capacity, and cost values and their
  startup normalization. World food planes continue to own only density/growth.
- `CreatureState` owns the live reproductive reserve. It is not cached in genome,
  telemetry, action logs, or UI stores as an alternate source of truth.
- Typed Eat application owns converting actual consumed density into both yields and
  clamping the resulting creature state.
- Reproduction application owns the reserve gate, rejection reason, and atomic reserve
  debit on successful spawning.
- Input resolution reads reserve directly from the acting creature at execution time.
  Mutation only makes that input reference discoverable; it does not manufacture state.
- Founder policy demonstrates solvability but does not receive privileged food or
  reserve state.
- Server/frontend surfaces project core config/state. They must not infer reserve from
  action intent, food density, or historical counters.

## Separation of Concerns and Decomposition

Stage 01 establishes the data/config/input contracts without changing action semantics.
Stage 02 applies those contracts to eating, reproduction, defaults, founder behavior,
and behavior tests. Stage 03 updates transport/UI projections and durable documentation,
then runs the repository-wide gates. Each stage has a coherent owner boundary and later
stages depend on the exact contracts established earlier.

## Tech Debt and Spaghetti-Code Implications

- Remove `EnergyCostsConfig::eat_reward_per_food`; retaining it alongside typed yields
  would create two reward authorities.
- Keep the mechanism concrete: one scalar reserve, not a generic nutrient trait or
  vector abstraction without a second implementation.
- Reproduction has an explicit ordered procedure and repeated rejection bookkeeping.
  Add the nutrition result exhaustively and, only if needed for this feature, extract a
  local rejection-recording helper without redesigning the action module.
- Update owned fixtures and examples rather than adding serde aliases/defaults whose
  only purpose is accepting the retired payload shape.
- Do not broaden this PRD into reconciliation of unrelated stale documented defaults;
  correct affected tables/examples where they are touched and file separate work for
  unrelated drift discovered during implementation.

## Documentation Impact and Synchronization

Synchronize the affected contracts in:

- `docs/reference/v3-runtime-config-spec.md`
- `docs/reference/v3-world-grid-spec.md`
- `docs/reference/v3-sensor-spec.md`
- `docs/reference/v3-reproduction-spec.md`
- `docs/reference/v3-startup-seeding-spec.md`
- `docs/reference/v3-tick-orchestration-spec.md`
- `docs/reference/v3-server-api-protocol-spec.md`

Update examples and generated/owned fixtures that show full configuration, food-type
metadata, reproduction results, or creature detail. Strategy and target-architecture
documents require no structural change: the feature implements their existing goals
and ownership boundaries.

## Stage Order and Links

1. [Stage 01 — Nutrition Contracts](stage-01-nutrition-contracts.md) — `Complete`
2. [Stage 02 — Applied Selection](stage-02-applied-selection.md) — `Complete`
3. [Stage 03 — Runtime Surfaces Verification](stage-03-runtime-surfaces-verification.md) — `Complete`

Stage 01 freezes normalized configuration, live state, and cognition contracts. Stage
02 makes them consequential in applied simulation behavior and production defaults.
Stage 03 projects only that applied truth, synchronizes docs, and closes verification.

## Cross-Stage Decisions

- `FoodTypeConfig` gains required `metabolic_energy_yield: f32` and
  `reproductive_reserve_yield: f32` fields. Both are finite, nonnegative, and applied
  per unit of density actually consumed.
- A required top-level `NutritionConfig` contains
  `reproductive_reserve_capacity: f32` and
  `reproductive_reserve_cost: f32`. Capacity must be finite and positive; cost must be
  finite, positive, and no greater than capacity. Both are startup-only.
- Delete global `energy.costs.eat_reward_per_food`. There is no compatibility alias or
  second precedence rule.
- Delete the public untyped `apply_eat` helper. Current production execution already
  uses typed Eat; tests and internal callers that still exercise the legacy helper must
  select an explicit `OrdinaryFoodTypeId` through `apply_typed_eat`.
- `CreatureState::reproductive_reserve` starts at `0.0` for founders and offspring and
  is clamped to `[0.0, capacity]` after successful Eat application.
- Add `DynamicIntrospectionKey::ReproductiveReserveCurrent`; resolve it from live state
  in raw reserve units. It must be available to mutation input-reference sampling.
- Add `ReproductionActionResult::RejectedNutritionConstraints`. Check reserve after
  target, population-cap, and age gates but before charging reproduce energy cost.
  Rejected attempts never debit reserve. After energy/transfer checks pass, debit the
  reserve exactly once in the same successful-spawn path.
- Production defaults start with two types:
  - `Maintenance Food`: coverage `0.27`, metabolic yield `10.0`, reserve yield `0.0`.
  - `Reproductive Food`: coverage `0.27`, metabolic yield `0.0`, reserve yield `1.0`.
  - Reserve capacity `8.0`; successful reproduction costs `4.0` reserve.
  These values keep expected initial metabolic capacity near the current single-food
  default while making the resources strictly complementary. Stage 02 may tune only
  these new/redistributed nutrition defaults if deterministic viability evidence shows
  a liveness failure; record the evidence and final values in tests and durable docs.
- The canonical founder must use ordinary typed-food sensors and the live reserve input.
  It receives no hidden lookup, free reserve, or founder-only exception.
- Runtime state/telemetry derives from applied state and actual consumed amount. No
  fitness decision reads UI projections or observability counters.
- Tests are written before behavior changes. Use exact assertions for enum/state
  transitions and tolerances for derived floating-point yields. Fixed seeds are used
  only for assertions that depend on reproducibility.
- Because production defaults, founder behavior, and tick-loop mechanics change, the
  implementation session must run `cargo test -p v3-core --test viability` before the
  first edit and again immediately after Stage 02 behavior lands. `make check` is the
  final completion gate.

## Implementation or Decision Tasks

- [x] Keep stage links, dependencies, status summaries, and verification evidence
  truthful throughout implementation.
- [x] Preserve the cross-stage contracts above; record any readiness-approved revision
  in this master and the owning stage before code changes.
- [x] Do not start implementation until a separate readiness review sets every PRD to
  `Ready` with `Review Status: APPROVED`.
- [x] After all stages are complete, request the separate final-code review and archive
  only after its findings and verification gates are resolved.

## Verification and Observable Success Criteria

- [x] Before implementation edits, run `cargo test -p v3-core --test viability` and
  record the baseline result in Stage 02.
- [x] A successful typed Eat changes energy/reserve by the configured yields multiplied
  by the amount actually removed from the selected food plane; empty/invalid eating
  creates no yield and reserve never exceeds capacity.
- [x] A creature lacking either energy or reserve cannot spawn; only a successful spawn
  debits reserve, and every child begins with zero reserve.
- [x] The canonical founder consumes both default food types and reproduces under fixed
  test seeds, while explicit one-resource ablations cannot satisfy both maintenance and
  reproduction contracts.
- [x] Cognition reads live reserve, and mutation sampling can produce the new input and
  both configured food-type references.
- [x] Runtime config, static food metadata, creature detail, action/reproduction
  outcomes, and UI inspection agree with core applied state.
- [x] Run `cargo test -p v3-core --test viability` immediately after Stage 02 and record
  the result.
- [x] Every stage's declared verification has passed.
- [x] Affected durable documentation is created, updated, or synchronized, or a no-change rationale is recorded.
- [x] `scripts/prd-index` and `scripts/prd-check` pass after status/evidence updates.
- [x] `make check` passes as the final implementation gate.
- [x] The final-code review gate has passed.

## Current Status

Complete. Stages 01–03 and their verification gates are complete. A separate Sol
final-code review confirmed that every L1/L2 finding is resolved; focused closure
tests, PRD validation, and the repository-wide completion gate pass. The implementation
is ready for archival.
