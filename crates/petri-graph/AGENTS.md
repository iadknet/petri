# petri-graph AGENTS.md

Local instructions for `crates/petri-graph`.

## Scope

`petri-graph` owns controller representation, graph evaluation, presets, and mutation behavior.

## Purity Rules

- Keep this crate pure computation.
- No transport/runtime dependencies (`tokio`, `axum`) in graph logic.
- Inputs/outputs should remain explicit and testable.

## Mutation and Eval Guardrails

- Preserve deterministic evaluation behavior under fixed seeds.
- Keep mutation operators bounded and covered by tests.
- When adding node semantics, update palette/evaluation tests and corresponding docs.

## Local Test Strategy

- Fast loop: `cargo test -p petri-graph <test_name> -- --exact`
- Crate sweep: `cargo test -p petri-graph`
- If behavior changes affect runtime interpretation, coordinate with `petri-core` tests.

## Related Canonical Docs

- Root policy: `AGENTS.md`
- Docs index: `docs/README.md`
- Active strategy docs: `docs/strategy/`
- Active reference specs: `docs/reference/`
- Archived reference specs: `docs/reference/archive/`
