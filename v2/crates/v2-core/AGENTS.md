# v2-core AGENTS.md

Local instructions for `v2/crates/v2-core`.

## Scope

`v2-core` owns simulation/runtime policy:
- mesh schema and execution semantics
- VM/graph backend behavior
- evolution/ecology policy
- deterministic runtime telemetry needed by higher layers

## Boundary Rules

- Do not add HTTP/WebSocket/CLI/UI concerns.
- Do not pull in transport payload types from `v2-server`/`v2-web`.
- Keep policy logic in core modules; adapters belong elsewhere.

## Contract Guardrails

- Runtime ordering, energy charging, and validation semantics are contract surface.
- Backward compatibility is not required; runtime/schema changes may require world restart.
- Keep randomness seed-driven and deterministic in tests.

## Test Expectations

- Prefer focused integration tests in `v2/crates/v2-core/tests/`.
- Add regression tests for behavior/policy changes before implementation.
- Align required suite names with CP-1/CP-2 matrix gates.
