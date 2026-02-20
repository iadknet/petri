# v2-cli AGENTS.md

Local instructions for `v2/crates/v2-cli`.

## Scope

`v2-cli` owns headless workflows over `v2-core`:
- run loop execution commands
- ablation workflows
- reproducible NDJSON reporting for tests/fixtures

## Boundary Rules

- Keep CLI output shaping in this crate.
- Do not add HTTP/WebSocket server concerns.
- Keep output schema independent from presentation/UI concerns.

## Contract Guardrails

- NDJSON events must keep required fields and stable ordering per checkpoint fixtures/tests.
- Prefer explicit event payloads over ad-hoc logging.
- Unknown-field and shape regressions should fail tests.
- Backward compatibility is not required; schema changes may assume restart.

## Test Expectations

- Maintain `run_output` and `ablation_output` contract tests.
- Add failing output-schema tests before behavior changes.
