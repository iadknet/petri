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
| Should status and frame data be separate endpoints? | Yes, status via HTTP and frames via WebSocket stream + optional HTTP snapshot endpoint. | user+agent | resolved |
| Should CLI output be text-only? | No, use line-delimited JSON for stable parsing. | user+agent | resolved |

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

Response:
- `protocol_version`
- `state: "paused"`
- `tick: u64`

### `GET /v2/simulation/status`

Response:
- `protocol_version`
- `state: "idle" | "running" | "paused"`
- `tick: u64`
- `sensor_radius: u16`
- `population: u32`
- `mean_energy: f32`
- `births_last_window: u32`
- `deaths_last_window: u32`
- `last_action_counts: { move: u32, eat: u32, reproduce: u32, inventory_pickup: u32, inventory_put: u32, noop: u32 }`

### `GET /v2/simulation/frame`

Response:
- `protocol_version`
- `tick: u64`
- `width: u16`
- `height: u16`
- `creatures: [{ id: u64, x: u16, y: u16, energy: f32, phenotype_rgb: [u8; 3] }]`
- `food: [{ x: u16, y: u16, density: u8 }]`
- `barriers: [{ x: u16, y: u16 }]`

### `GET /v2/simulation/snapshot`

Response:
- `protocol_version`
- `snapshot_format: "v2alpha1"`
- `tick: u64`
- `world_state: object`
- `creatures: [{ ..., memory_b64: string }]` (`memory_b64` encodes fixed `1024` bytes per creature)

### `POST /v2/simulation/snapshot`

Request:
- snapshot payload matching `GET` response

Response:
- `protocol_version`
- `accepted: bool`
- `state: "paused"`
- `tick: u64`

## WebSocket Stream Contract (`/v2/ws`)

Event envelope:
- `protocol_version: "v2alpha1"`
- `event: "status" | "frame" | "health"`
- `tick: u64`
- `payload: object`

Event payloads:
1. `status` payload uses `GET /simulation/status` schema.
2. `frame` payload uses `GET /simulation/frame` schema.
3. `health` payload:
- `population: u32`
- `genome_node_count_p50: u16`
- `genome_node_count_p90: u16`
- `mean_energy: f32`

Ordering requirements:
1. `tick` is monotonic non-decreasing per connection.
2. For same tick, `status` event is emitted before `frame`.

## CLI Contract (`v2-cli`)

Commands:
1. `v2-cli run --ticks <u64> --sample-every <u16> --seed <u64>`
2. `v2-cli ablation --ticks <u64> --seed <u64> --preset <name>`

Output format:
- stdout emits NDJSON records only.
- every line contains `protocol_version: "v2alpha1"` and `event_type`.

Required `run` events:
1. `run_started`
2. periodic `tick_sample`
3. `run_completed`

Required `ablation` events:
1. `ablation_started`
2. per-preset `ablation_result`
3. `ablation_completed`

## Task List

### Task 1: Add failing contract tests and fixtures

Files:
- Create: `v2/crates/v2-server/tests/lifecycle.rs`
- Create: `v2/crates/v2-server/tests/payloads.rs`
- Create: `v2/crates/v2-server/tests/ws_stream.rs`
- Create: `v2/crates/v2-cli/tests/run_output.rs`
- Create: `v2/crates/v2-cli/tests/ablation_output.rs`
- Create: `v2/web/src/protocol.test.ts`
- Create: `v2/web/src/fixtures/protocol-v2alpha1/*.json`

Steps:
1. Add failing tests for every endpoint/event contract above.
2. Add fixture-based parser tests in web client.
3. Add CLI NDJSON schema tests.

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
- Create: `v2/web/src/protocol.ts`
- Modify: `v2/web/src/App.tsx`

Steps:
1. Implement runtime-safe decoders for server payloads.
2. Bind UI rendering to protocol models only.
3. Add guardrails for protocol-version mismatch.

## Verification Commands

1. `scripts/check-plan-harness.sh --mode strict`
2. `cd v2 && cargo test -p v2-server`
3. `cd v2 && cargo test -p v2-cli`
4. `cd v2/web && npm run test`
5. `cd v2/web && npm run build`
6. `cd v2 && cargo test --workspace`

## Risks and Rollback

- Risk: protocol churn can cascade across server, cli, and web simultaneously.
- Risk: weak fixtures can hide accidental schema drift.
- Rollback:
1. Revert CP-3 integration commits.
2. Reapply with fixture-first contract lock before implementation.
