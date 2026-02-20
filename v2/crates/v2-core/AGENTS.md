# v2-core AGENTS.md

Local instructions for `v2/crates/v2-core`.

## Scope

`v2-core` owns simulation/runtime policy:
- mesh schema and execution semantics
- VM/graph backend behavior
- evolution/ecology policy
- runtime telemetry semantics needed by higher layers

## Boundary Rules

- Do not add HTTP/WebSocket/CLI/UI concerns.
- Do not pull in transport payload types from `v2-server`/`v2-web`.
- Keep policy logic in core modules; adapters belong elsewhere.

## Contract Guardrails

- Runtime ordering, energy charging, and validation semantics are contract surface.
- World tick semantics (action application, energy change, and depletion/removal paths where applicable) are contract surface.
- Backward compatibility is not required; runtime/schema changes may require world restart.
- Determinism scope is canonical in root `AGENTS.md`; deterministic behavior is required in tests/harnesses when assertions depend on reproducibility.
- Keep randomness seed-driven in tests that require reproducibility.

## Test Expectations

- Prefer focused integration tests in `v2/crates/v2-core/tests/`.
- Add regression tests for behavior/policy changes before implementation.
- Align required suite names with CP-1/CP-2 matrix gates.
- Ensure tests demonstrate intended behavior in integrated runtime flow, not only isolated contract conformance.
