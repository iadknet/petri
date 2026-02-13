# petri-core AGENTS.md

Local instructions for `crates/petri-core`.

## Scope

`petri-core` owns simulation policy: world state, tick semantics, energy accounting, spawning, perception, paint effects, and snapshot behavior.

## Boundary Rules

- Do not add transport concerns (`axum`, `tokio`, WebSocket/HTTP types) here.
- Do not import frontend/web concerns here.
- Keep crate API narrow; default to private or `pub(crate)` visibility.

## Determinism Requirements

- Favor deterministic seeds in tests.
- Treat snapshot round-trip and deterministic continuation as first-class checks.
- Preserve reproducible behavior for founder seeding and mutation-sensitive tests.

## Tick and Policy Guardrails

- Keep action/economics logic inside world/tick policy modules.
- When changing frame-relevant outputs, coordinate with `petri-server` and `web` consumers.
- If behavior changes, write regression tests in `crates/petri-core/src/world/tests.rs` or focused integration tests under `crates/petri-core/tests/`.

## Local Test Strategy

- Fast loop: `cargo test -p petri-core <test_name> -- --exact`
- Crate sweep: `cargo test -p petri-core`
- Cross-crate contracts touched: run workspace gate from root.

## Related Canonical Docs

- Root policy: `AGENTS.md`
- Reference behavior: `docs/reference/creature-controller-reference.md`
- Architecture context: `docs/strategy/architecture.md`
