# V3 Viewport Transport Refactor Program

**Goal:** Deliver a viewport-driven server/frontend refactor that restores performance under large-world load while improving architectural boundaries, with benchmark gates before and after every phase.

**Goal IDs:** GP-01, GP-02, GP-03, GP-04

**Scope:** Included: server command/query/transport realignment, frontend viewport/world-view/render realignment, `v3alpha2` viewport transport, projection-backed reads, benchmark gates, and the core runtime optimization phases already identified by profiling. Excluded: new workspace crates, event sourcing/replay architecture, simulation semantic changes, and economy tuning.

**Docs Impact:**

| Doc | Action |
| --- | --- |
| `docs/plans/2026-03-02-v3-viewport-transport-plan.md` | Create main coordination plan |
| `docs/plans/2026-03-02-v3-viewport-transport-server-companion.md` | Create server architecture companion |
| `docs/plans/2026-03-02-v3-viewport-transport-frontend-companion.md` | Create frontend architecture companion |
| `docs/plans/2026-03-02-v3-viewport-transport-benchmarks-companion.md` | Create benchmarks/core-perf companion |
| `docs/plans/2026-03-02-v3-viewport-transport-verification-matrix.md` | Create shared verification matrix |
| `docs/reference/v3-server-query-projection-spec.md` | Create query/projection reference spec |
| `docs/strategy/architecture.md` | Update high-level ownership map |
| `docs/reference/v3-server-api-protocol-spec.md` | Update transport ownership boundaries |
| `docs/reference/v3-evolution-observability-spec.md` | Clarify energy-cost vs wall-clock perf semantics |
| `docs/reference/v3-cli-contract-spec.md` | Clarify independent CLI versioning |
| `docs/brainstorms/feature_ideas.md` | Record deferred architecture follow-ups |

**Supersedes:** none

**Superseded-By:** none

**See also:**
- `docs/plans/2026-03-02-v3-viewport-transport-server-companion.md`
- `docs/plans/2026-03-02-v3-viewport-transport-frontend-companion.md`
- `docs/plans/2026-03-02-v3-viewport-transport-benchmarks-companion.md`
- `docs/plans/2026-03-02-v3-viewport-transport-verification-matrix.md`
- `docs/reference/v3-server-query-projection-spec.md`

## Goal Alignment

- **GP-01:** Viewport-driven transport removes full-world steady-state payload waste so larger worlds and populations remain interactive.
- **GP-02:** The refactor makes the read-side boundary explicit instead of mixing transport concerns into simulation policy.
- **GP-03:** Every phase is benchmark-gated and tied to the shared verification matrix.
- **GP-04:** Projection-backed status, health, and performance telemetry remain behavior-backed and auditable.

## Boundary Impact

- `v3-core` remains the authoritative simulation domain.
- `v3-server` becomes explicitly layered into command, query, transport, and HTTP modules without introducing a new crate boundary.
- The frontend becomes explicitly layered into viewport state, world-view state, transport client, and renderer.
- Wire contracts and projection semantics move into canonical reference docs instead of living only in the plan.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v3/crates/v3-core` ownership split | keep | The core problem is not simulation ownership; it is the missing read/query boundary above core. |
| `v3/crates/v3-server` flat `state/handlers/ws/types` structure | change | Current transport, authority, and payload shaping responsibilities are too entangled. |
| `frontend/src/canvas` rendering ownership | keep | Drawing remains a frontend concern, but it should consume a render model rather than raw transport payloads. |
| `frontend/src/api` plus store coupling | change | Transport decoding and store mutation need a cleaner boundary so viewport subscriptions can evolve safely. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should the read/projection layer become a new crate now? | No. Keep it server-local for this refactor. | user+agent | resolved |
| Should running-state reads be projection-backed? | Yes. `/status`, `/snapshot`, and WS view emission use the published projection while running. | user+agent | resolved |
| How should the initial viewport bootstrap work? | `GET /v3/simulation/snapshot` without viewport params returns a bootstrap overview snapshot. | user+agent | resolved |
| Should numeric zoom thresholds live in the server contract? | No. The protocol carries semantic fidelity tiers only. | user+agent | resolved |
| How should overview-mode selection behave? | Selection is disabled in overview mode and only enabled in detail/inspect tiers. | user+agent | resolved |

## Execution Requirements

- Any Rust implementation or refactor work under `v3/` must explicitly load and apply the `rust-skills` skill before editing code or reviewing Rust changes.
- Any frontend implementation or refactor work under `frontend/` must explicitly load and apply `vercel-react-best-practices`, `vercel-composition-patterns`, and `frontend-design` before editing code.
- Any frontend smoke-flow or E2E validation added by this program must use `agent-browser` in addition to the scripted test suite.
- After every task that touches Rust code, run a recursive review pass guided by `rust-skills`; resolve every new finding introduced by that task and rerun the review until it returns no new findings.
- After every task that touches frontend code, run a recursive review pass guided by `vercel-react-best-practices` and `vercel-composition-patterns`; if the task changes UI behavior or presentation, include `frontend-design` in the same recursive review loop. Resolve every new finding introduced by that task and rerun until no new findings remain.
- Tasks that touch both Rust and frontend code must satisfy both recursive review gates before they can be marked complete.
- These skill requirements are execution gates for the companion plans and do not rely solely on repository-level `AGENTS.md`.

## Phase Order

1. Create the split plan package and reference-spec scaffolding.
2. Realign `v3-server` around command/query/transport boundaries and add projection publication.
3. Realign the frontend around viewport/world-view/render boundaries.
4. Add projection spatial indexing and bounded view assembly.
5. Cut over to `v3alpha2` viewport-driven HTTP/WS transport.
6. Add dirty-rect invalidation and session-targeted refresh.
7. Apply core metadata-cache optimizations.
8. Apply core allocation/perception optimizations.
9. Apply conditional final runtime cleanup if benchmark targets are still unmet.

## Go/Stop Checkpoints

- Do not start frontend protocol cutover until the server companion's projection publication and bootstrap contracts are implemented and tested.
- Do not start core runtime micro-optimization phases until the transport/projection waste is removed and re-profiled.
- Do not mark any task complete while the required recursive review gate still has new findings.
- Stop any phase that fails its benchmark gate or violates the shared verification matrix.

## Task List

### Task 1: Land the plan package and reference scaffolding

**Files:**
- Create: `docs/plans/2026-03-02-v3-viewport-transport-plan.md`
- Create: `docs/plans/2026-03-02-v3-viewport-transport-server-companion.md`
- Create: `docs/plans/2026-03-02-v3-viewport-transport-frontend-companion.md`
- Create: `docs/plans/2026-03-02-v3-viewport-transport-benchmarks-companion.md`
- Create: `docs/plans/2026-03-02-v3-viewport-transport-verification-matrix.md`
- Create: `docs/reference/v3-server-query-projection-spec.md`
- Modify: `docs/strategy/architecture.md`
- Modify: `docs/reference/v3-server-api-protocol-spec.md`
- Modify: `docs/reference/v3-evolution-observability-spec.md`
- Modify: `docs/reference/v3-cli-contract-spec.md`
- Modify: `docs/brainstorms/feature_ideas.md`

### Task 2: Execute the server companion

**Files:** See `docs/plans/2026-03-02-v3-viewport-transport-server-companion.md`.

### Task 3: Execute the frontend companion

**Files:** See `docs/plans/2026-03-02-v3-viewport-transport-frontend-companion.md`.

### Task 4: Execute the benchmarks/core-perf companion

**Files:** See `docs/plans/2026-03-02-v3-viewport-transport-benchmarks-companion.md`.

## Verification Commands

- Use `docs/plans/2026-03-02-v3-viewport-transport-verification-matrix.md` as the command source of truth.

## Risks and Rollback

- Highest-risk areas are projection publication semantics, session-aware websocket cutover, and concurrent frontend camera/render refactoring.
- Roll back protocol and frontend cutover independently from the server query/projection boundary if needed.
- Keep the benchmark and documentation scaffolding even if later phases need to pause or re-scope.

**Review cycles:** 1
