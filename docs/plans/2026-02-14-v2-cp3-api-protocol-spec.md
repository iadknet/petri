# Petri V2 CP-3 API and Protocol Spec

**Goal:** Define implementation-complete `v2-server`, `v2-cli`, and `v2-web` contracts for checkpoint `CP-3` so product-surface integration can proceed without schema churn.
**Goal IDs:** GP-01, GP-02, GP-03, GP-04
**Scope:** HTTP and WebSocket protocol contracts, CLI output contracts, fixture/test requirements, and versioning rules for `v2`; excludes legacy API compatibility.
**Docs Impact:** Adds CP-3 contract spec consumed by Stage 4 plan and test matrix.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-01`: exposes core runtime decisions/actions through stable interfaces.
- `GP-02`: enforces clear ownership between `v2-core` and integration layers.
- `GP-03`: locks payload schemas for reliable integration tests.
- `GP-04`: guarantees runtime status/action visibility to CLI/web.

## Boundary Impact

- `v2-server` is sole network surface and serializes `v2-core` runtime outputs.
- `v2-web` consumes contracts from this spec only.
- `v2-cli` reads directly from `v2-core` and emits deterministic report schema.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v2/crates/v2-server/src/*` | change | Must implement lifecycle and streaming endpoints using this schema. |
| `v2/web/src/*` | change | Must parse/render payloads exactly per protocol version. |
| legacy `crates/petri-server` + `web/` contracts | keep | No cross-compat requirement for initial `v2` release. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should `v2` payloads be backward compatible with legacy API? | No. | user+agent | resolved |
| Should status and frame data be separate endpoints? | Yes, status via HTTP and frames via WebSocket stream. | user+agent | resolved |
| Should CLI output be text-only? | No, use line-delimited JSON for stable parsing. | user+agent | resolved |
| Should lifecycle endpoints be strict or idempotent on repeated calls? | Idempotent for `start`/`pause`; state-restricted for `step`. | user+agent | resolved |
| Should status window counters share CP-2 telemetry semantics? | Yes; status counters use trailing `health_window_ticks` from CP-2 ecology config. | user+agent | resolved |
| Should snapshot import/export endpoints be required in initial `v2`? | No; snapshot endpoints are out of scope for `v2alpha1` unless re-planned. | user+agent | resolved |
| Should `config_digest` be deterministic and algorithm-defined? | Yes; `v2alpha1` uses lowercase hex SHA-256 over canonical startup JSON. | user+agent | resolved |
| Should CLI NDJSON event payload fields be explicitly fixed? | Yes; each event has required fields and canonical key ordering in this spec. | user+agent | resolved |

## Execution Decomposition

- CP-3 backend execution plan:
  - `docs/plans/2026-02-14-v2-cp3-backend-micro-implementation-plan.md`
- CP-3 frontend execution plan:
  - `docs/plans/2026-02-14-v2-cp3-frontend-micro-implementation-plan.md`

## Protocol Versioning

1. `protocol_version` is required in all top-level responses/events.
2. Initial value: `"v2alpha1"`.
3. Any breaking payload change requires protocol version bump and fixture refresh.

## HTTP API Contract (`v2-server`)

Base path: `/v2`

### `POST /v2/simulation/startup`

Request:
- `seed: u64`
- `world: { width: u16, height: u16, wrap: bool, sensor_radius: u16 }`
- `population: { initial_creatures: u32, max_creatures: u32 }`
- `runtime: { ticks_per_second: u16, max_tick_budget_ms: u16 }`

Response:
- `protocol_version: string`
- `state: "idle"`
- `config_digest: string`

Rules:
- `config_digest` must be deterministic for identical startup requests.
- `v2alpha1` definition: lowercase hex `sha256(canonical_json(startup_request))`.
- `canonical_json` uses lexicographically sorted object keys and no insignificant whitespace.
- `population.initial_creatures` is a requested target; actual seeded population is best-effort and bounded by occupancy and `max_creatures`.
- first `status`/`frame` payloads after startup may report population lower than requested `initial_creatures`.
- startup must enforce viability: if a request is below viable minimum bounds, server may normalize deterministically to a viable startup baseline; if viability still cannot be established, return a protocol-valid rejection (`422 validation_rejected`) instead of starting a dead-on-arrival run.

### `POST /v2/simulation/start`

Response:
- `protocol_version`
- `state: "running"`
- `tick: u64`

### `POST /v2/simulation/pause`

Response:
- `protocol_version`
- `state: "paused"`
- `tick: u64`

### `POST /v2/simulation/step`

Request:
- `steps: u16` (default `1`, max `1000`)

Rules:
- omitted `steps` defaults to `1`
- valid range is `1..=1000`
- invalid value returns `400` with error envelope
- endpoint requires `state="paused"`; other states return `409`

Response:
- `protocol_version`
- `state: "paused"`
- `tick: u64`

### `POST /v2/simulation/world/paint`

Request:
- `action: "stroke" | "clear_all"`
- `tool: "food" | "barrier" | "erase_food" | "erase_barrier"` (required when `action="stroke"`)
- `brush_half_extent: 0 | 1 | 2` (`stroke` only)
- `points: [{ x: u16, y: u16 }]` (`stroke` only, non-empty)

Rules:
- allowed only in editable states: `idle` and `paused`
- if current state is `running`: return `409` with `invalid_state_transition`
- `clear_all` ignores `tool`, `brush_half_extent`, and `points`
- `stroke` requires valid `tool`, `brush_half_extent`, and `points`
- malformed request shape returns `400 invalid_request`
- semantic validation failures (out of range, empty stroke points) return `422 validation_rejected`

Response:
- `protocol_version`
- `state: "idle" | "paused"`
- `tick: u64`
- `paint_result: { touched_cells: u32 }`

### `GET /v2/simulation/status`

Response:
- `protocol_version`
- `state: "idle" | "running" | "paused"`
- `tick: u64`
- `sensor_radius: u16`
- `health_window_ticks: u16`
- `population: u32`
- `mean_energy: f32`
- `births_last_window: u32`
- `deaths_last_window: u32`
- `last_action_counts: { move: u32, eat: u32, reproduce: u32, inventory_pickup: u32, inventory_put: u32, noop: u32 }`

Rules:
- `births_last_window` and `deaths_last_window` use CP-2 telemetry window semantics (`health_window_ticks`, trailing right-aligned window).

### `GET /v2/simulation/frame`

Response:
- `protocol_version`
- `tick: u64`
- `width: u16`
- `height: u16`
- `creatures: [{ id: u64, x: u16, y: u16, energy: f32, phenotype_rgb: [u8; 3] }]`
- `food: [{ x: u16, y: u16, density: u8 }]`
- `barriers: [{ x: u16, y: u16 }]`

Rules:
- Startup founders begin from one shared phenotype baseline; phenotype diversity is introduced by reproduction mutation, not startup randomization.

Snapshot endpoints policy (`v2alpha1`):
1. `GET /v2/simulation/snapshot` is out of scope.
2. `POST /v2/simulation/snapshot` is out of scope.
3. Snapshot contracts may be added in a follow-up checkpoint/plan if needed.

### HTTP error envelope (normative)

All non-2xx responses must use:
- `protocol_version: "v2alpha1"`
- `error: { code: string, message: string, details?: { endpoint?: string, field_errors?: [{ field: string, reason: string }], expected_state?: "idle" | "running" | "paused", current_state?: "idle" | "running" | "paused" } }`

Required error codes:
- `invalid_request` (`400`)
- `invalid_state_transition` (`409`)
- `validation_rejected` (`422`)
- `internal_error` (`500`)

### Lifecycle transition contract

1. `POST /simulation/startup`
- valid from any state
- resets simulation state to configured idle at `tick=0`

2. `POST /simulation/start`
- if `idle` (configured) or `paused`: transitions to `running`
- if already `running`: returns `200` with unchanged `running` state (idempotent)

3. `POST /simulation/pause`
- if `running`: transitions to `paused`
- if already `paused`: returns `200` with unchanged `paused` state (idempotent)
- if `idle`: returns `409` `invalid_state_transition`

4. `POST /simulation/step`
- valid only when `paused`
- advances exactly requested steps
- remains `paused` after completion

## WebSocket Stream Contract (`/v2/ws`)

Event envelope:
- `protocol_version: "v2alpha1"`
- `event: "status" | "frame" | "health"`
- `tick: u64`
- `payload: StatusPayload | FramePayload | HealthPayload`

Event payloads:
1. `status` payload uses `GET /simulation/status` schema.
2. `frame` payload uses `GET /simulation/frame` schema.
3. `health` payload:
- `population: u32`
- `genome_node_count_p50: u16`
- `genome_node_count_p90: u16`
- `mean_energy: f32`

Payload mapping rule:
1. `event="status"` must carry `StatusPayload`.
2. `event="frame"` must carry `FramePayload`.
3. `event="health"` must carry `HealthPayload`.
4. Any event/payload mismatch is protocol-invalid and must fail fixture/parser tests.

Ordering requirements:
1. `tick` is monotonic non-decreasing per connection.
2. For same tick, `status` event is emitted before `frame`.
3. If `health` is emitted for same tick, order is `status -> frame -> health`.

## CLI Contract (`v2-cli`)

Commands:
1. `v2-cli run --ticks <u64> --sample-every <u16> --seed <u64>`
2. `v2-cli ablation --ticks <u64> --seed <u64> --preset <name>`

Output format:
- stdout emits NDJSON records only.
- every line contains `protocol_version: "v2alpha1"` and `event_type`.
- records must contain all required fields for their `event_type` and no unknown top-level fields in fixture-based tests.
- field ordering in emitted JSON objects must follow the canonical ordering defined below (for deterministic fixtures).

Required `run` events:
1. `run_started`
2. periodic `tick_sample`
3. `run_completed`

Required `ablation` events:
1. `ablation_started`
2. per-preset `ablation_result`
3. `ablation_completed`

CLI event payload schemas (normative):

1. `run_started`:
- `protocol_version: "v2alpha1"`
- `event_type: "run_started"`
- `seed: u64`
- `ticks_requested: u64`
- `sample_every: u16`

2. `tick_sample`:
- `protocol_version: "v2alpha1"`
- `event_type: "tick_sample"`
- `tick: u64`
- `population: u32`
- `mean_energy: f32`
- `births_last_window: u32`
- `deaths_last_window: u32`
- `last_action_counts: { move: u32, eat: u32, reproduce: u32, inventory_pickup: u32, inventory_put: u32, noop: u32 }`

3. `run_completed`:
- `protocol_version: "v2alpha1"`
- `event_type: "run_completed"`
- `ticks_executed: u64`
- `final_population: u32`
- `final_mean_energy: f32`

4. `ablation_started`:
- `protocol_version: "v2alpha1"`
- `event_type: "ablation_started"`
- `seed: u64`
- `ticks_requested: u64`
- `presets: Vec<String>`

5. `ablation_result`:
- `protocol_version: "v2alpha1"`
- `event_type: "ablation_result"`
- `preset: String`
- `score: f32`
- `final_population: u32`
- `final_mean_energy: f32`

6. `ablation_completed`:
- `protocol_version: "v2alpha1"`
- `event_type: "ablation_completed"`
- `results_count: u16`
- `best_preset: String`

## Task List

### Task 1: Add failing contract tests and fixtures

Files:
- Create: `v2/crates/v2-server/tests/lifecycle.rs`
- Create: `v2/crates/v2-server/tests/payloads.rs`
- Create: `v2/crates/v2-server/tests/ws_stream.rs`
- Create: `v2/crates/v2-server/tests/world_paint.rs`
- Create: `v2/crates/v2-cli/tests/run_output.rs`
- Create: `v2/crates/v2-cli/tests/ablation_output.rs`
- Create: `v2/web/src/features/protocol/protocol.test.ts`
- Create: `v2/web/src/fixtures/protocol-v2alpha1/*.json`

Steps:
1. Add failing tests for every endpoint/event contract above.
2. Add fixture-based parser tests in web client.
3. Add CLI NDJSON schema tests for required fields, forbidden unknown top-level fields, and canonical field ordering.
4. Add failing tests for HTTP error envelope/status-code mapping and lifecycle transition rules.
5. Add failing tests for event/payload mismatch rejection and snapshot-endpoint absence in `v2alpha1`.
6. Add failing tests for paint stroke/clear behavior and phase restrictions.

### Task 2: Implement server lifecycle and streaming endpoints

Files:
- Modify: `v2/crates/v2-server/src/main.rs`
- Create: `v2/crates/v2-server/src/api.rs`
- Create: `v2/crates/v2-server/src/state.rs`
- Create: `v2/crates/v2-server/src/ws.rs`

Steps:
1. Implement all HTTP endpoints.
2. Implement WebSocket event stream with ordering guarantees.
3. Ensure all payloads include `protocol_version`.

### Task 3: Implement CLI contract-compliant output

Files:
- Modify: `v2/crates/v2-cli/src/main.rs`
- Create: `v2/crates/v2-cli/src/output.rs`
- Create: `v2/crates/v2-cli/src/ablation.rs`

Steps:
1. Implement `run` and `ablation` commands.
2. Emit NDJSON with stable field ordering and schema.
3. Keep output deterministic for fixture comparison tests.

### Task 4: Implement web protocol layer

Files:
- Create: `v2/web/src/features/protocol/client.ts`
- Create: `v2/web/src/features/protocol/models.ts`
- Create: `v2/web/src/features/protocol/decoders.ts`
- Modify: `v2/web/src/App.tsx`

Steps:
1. Implement runtime-safe decoders for server payloads.
2. Bind UI rendering to protocol models only.
3. Add guardrails for protocol-version mismatch.

## Verification Commands

1. `scripts/check-plan-harness.sh --mode strict`
2. Run the `CP-3` command gate from `docs/plans/2026-02-14-v2-implementation-test-matrix.md` (`## Command Gates by Checkpoint` -> `### CP-3 exit`).

## Risks and Rollback

- Risk: protocol churn can cascade across server, cli, and web simultaneously.
- Risk: weak fixtures can hide accidental schema drift.
- Rollback:
1. Revert CP-3 integration commits.
2. Reapply with fixture-first contract lock before implementation.
