# v2-server AGENTS.md

Local instructions for `v2/crates/v2-server`.

## Scope

`v2-server` owns `v2` transport and lifecycle orchestration:
- HTTP endpoint contracts
- WebSocket stream contracts
- phase/editability guards and error-envelope mapping

## Boundary Rules

- Keep simulation policy decisions in `v2-core`.
- Validate and map requests/responses here; do not reimplement core policy logic.
- Keep protocol schema explicit and fixture-locked for the current checkpoint.
- Do not synthesize creature/world tick outcomes or telemetry values that are available from core runtime state.

## Contract Guardrails

- Non-2xx responses must follow the CP-3 error envelope.
- Lifecycle transitions must stay consistent with CP-3 rules.
- Event ordering for same tick must remain stable where tests/fixtures and protocol assertions require reproducibility.
- Backward compatibility is not required; schema changes may assume restart.
- Runtime-facing payload fields (`status`, `frame`, `health`, and equivalent future surfaces) must remain behavior-truthful per `docs/standards/runtime-behavior-realism-policy.md`.

## Test Expectations

- Add/maintain endpoint + payload + ws tests in `v2/crates/v2-server/tests/`.
- For contract changes, update server tests and web fixtures in the same slice.
- Keep intent/integration regressions in place so endpoint-contract tests are not the only completion evidence.
