# Stage 03 — Runtime Surfaces Verification

- Status: Complete
- Depends on: stage-02-applied-selection.md
- Master: [Master PRD](master-prd.md)

## Goal

Project the applied nutrition contract through existing server/frontend inspection and
configuration surfaces, synchronize durable documentation, and close the repository-wide
verification gates without creating a proxy fitness authority.

## Scope

- Include typed metabolic/reserve yields in static food-type metadata.
- Include live reserve and capacity in creature detail and the existing inspector
  vitals presentation.
- Represent the new config fields and removed global reward in startup/config UI types,
  controls, fixtures, and protocol examples.
- Map the nutrition rejection outcome through existing action/reproduction detail
  surfaces.
- Add cross-boundary tests proving projections equal core applied state.
- Complete the documentation audit, PRD evidence, `scripts/prd-index`,
  `scripts/prd-check`, and `make check`.

## Non-Goals

- A new dashboard, time-series store, aggregate population nutrition score, or alert.
- Recomputing reserve from food density, action history, or expected yield in the server
  or frontend.
- Runtime patching of food yields, reserve capacity, or reserve cost.
- A UI claim that reserve use measures intelligence, complexity, fitness, or novelty.
- Performance redesign of snapshot/projection infrastructure.

## Inputs and Existing-Code Interactions

- Stage 02 is the sole applied source of state and action outcomes.
- `crates/v3-server/src/state.rs`, `handlers/lifecycle.rs`, and
  `transport/protocol.rs` own world/static payloads and creature snapshots.
- `crates/v3-server/src/handlers/creature.rs` owns creature-detail responses; it must
  read current reserve/capacity from the simulation snapshot/config.
- Server query/cache/projection modules and tests contain complete snapshot fixtures
  that must remain coherent when food metadata or creature snapshots change.
- `frontend/src/types/config.ts`, `types/protocol.ts`, and
  `types/creature-detail.ts` mirror wire/config contracts.
- Existing config-panel food/energy sections, creature inspector/vitals, stores, and
  shared fixtures are the presentation owners. Remove the global Eat reward control and
  place yields with each food type; place reserve capacity/cost in a small nutrition
  section.

## Boundaries and Abstraction Layers

- Static food metadata may copy configured yield values because config is their core
  owner. Creature detail must copy current reserve, not derive it.
- The frontend renders values and sends startup configuration; it does not enforce
  action semantics or infer eligibility.
- Server validation rejects runtime patches to nutrition fields using the existing
  startup-only policy. It does not clamp already-live creatures after a patch.
- Action/reproduction results use the core stable key. Do not add an alternate UI-only
  classification.

## Separation of Concerns and Decomposition

This stage is a projection/documentation/verification slice. It depends on stable core
behavior and can be tested with known snapshots without changing simulation outcomes.
Splitting server and frontend would add another handoff without an independently useful
intermediate product.

## Tech Debt and Spaghetti-Code Implications

- Reuse existing payload, store, and inspector paths. Do not add a parallel nutrition
  cache or bespoke polling endpoint.
- Update shared fixture builders where possible rather than scattering new literal
  fields across tests; do not refactor unrelated fixtures.
- Keep food yields in static metadata rather than repeating them per cell/frame.
- Reserve is an applied vital, not a derived score. Labels/help text must describe its
  mechanics without anthropomorphic or intelligence claims.
- Apply the detailed Rust testing/documentation/exhaustive-match rules named in Stage
  01. Existing TypeScript project conventions govern frontend changes.

## Documentation Impact and Synchronization

Update `docs/reference/v3-server-api-protocol-spec.md` for food metadata, startup config,
creature detail, and nutrition rejection. Complete consistency checks across:

- `docs/reference/v3-runtime-config-spec.md`
- `docs/reference/v3-world-grid-spec.md`
- `docs/reference/v3-sensor-spec.md`
- `docs/reference/v3-reproduction-spec.md`
- `docs/reference/v3-startup-seeding-spec.md`
- `docs/reference/v3-tick-orchestration-spec.md`

No change is required to the CLI event contract unless implementation inspection finds
that it serializes one of the changed full payloads; if so, synchronize its owned
example and test without adding a new CLI metric.

## Implementation or Decision Tasks

- [x] Write failing server serialization/projection tests for both typed yields, live
  creature reserve/capacity, and `RejectedNutritionConstraints`.
- [x] Extend static food metadata and creature detail from core config/state; update
  query/cache/projection fixtures and exhaustive result mappings.
- [x] Write a failing server patch-policy test, then reject runtime changes to typed
  yields, reserve capacity, and reserve cost while accepting them at startup.
- [x] Update TypeScript config/protocol/detail types and shared fixture builders; remove
  every owned `eat_reward_per_food` use.
- [x] Write failing config-panel tests, then render per-type yield controls and the
  startup-only reserve capacity/cost controls with finite/nonnegative client affordances.
- [x] Write a failing creature-inspector test, then display current reserve against
  capacity as an applied vital with a mechanical label.
- [x] Map the nutrition rejection reason in existing action/reproduction inspection UI
  if that UI currently displays core rejection reasons; do not add a new surface solely
  for this result.
- [x] Run an `rg` audit for `eat_reward_per_food`, full food-type literals,
  introspection matches, reproduction result matches, and creature-detail fixtures;
  resolve all owned stale references.
- [x] Synchronize all listed durable documentation and record a no-change rationale for
  inspected but unaffected strategy/architecture/CLI documents.
- [x] Record focused and repository-wide verification evidence in the owning stages and
  master without marking unrun checks complete.

## Verification and Observable Success Criteria

- [x] Server tests prove food metadata equals configured per-type yields and creature
  detail reserve/capacity equals live core state/config.
- [x] Server tests prove nutrition fields are startup-only and the new rejection result
  keeps one stable core-to-wire classification.
- [x] Frontend tests prove startup controls serialize the new schema without the removed
  global reward and inspector vitals render the supplied live reserve/capacity.
- [x] An `rg -n "eat_reward_per_food" crates frontend docs/reference` audit returns no
  current-contract references; any intentionally retained historical occurrence is
  documented as archive evidence.
- [x] `cargo test -p v3-server` passes.
- [x] `cargo test -p v3-cli` passes if CLI-owned fixtures/examples changed.
- [x] The repository's documented frontend test command passes.
- [x] `cargo test -p v3-core --test viability` still passes after projection work.
- [x] `scripts/prd-index` passes.
- [x] `scripts/prd-check` passes.
- [x] `make check` passes as the final completion gate.
- [x] Affected durable documentation is created, updated, or synchronized, or a no-change rationale is recorded.

## Verification Evidence

Remediation evidence (2026-09-01): `cargo test -p v3-server` passed 36 library,
2 binary, and 79 integration tests, including direct configured-yield/live-reserve
projection and per-type-yield runtime-patch rejection tests; `cargo test -p v3-cli`
passed 7 tests;
`cargo test --workspace --doc` passed; and the post-projection viability gate passed
25 tests. Frontend verification passed 54 files and 272 tests, `npm run build`
passed, and `npm run lint` passed with three existing `noArrayIndexKey` warnings.
`scripts/prd-index`, `scripts/prd-check`, `cargo fmt --all -- --check`, and
`cargo clippy --workspace --all-targets -- -D warnings` passed. The full
`make check` gate passed under the required Node/npm/Cargo PATH; optional OSV,
skill-scanner, ShellCheck, and actionlint checks were skipped because their pinned
tools are not installed. Documentation was audited and synchronized across the
seven PRD-listed reference specifications.

## Current Status

Complete. Runtime projections, reserve-carrying mesh APIs, exact frontend fallback
defaults, stable nutrition rejection mapping, direct tests, and all listed durable
documentation are synchronized and verified. The master remains In Progress pending
the separate final-code review and archival gate.
