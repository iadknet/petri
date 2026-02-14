# V2 Boundaries

## Code Ownership

- `v2/crates/v2-core`: simulation/runtime policy.
- `v2/crates/v2-server`: transport/API for `v2` only.
- `v2/crates/v2-cli`: headless/ablation workflows for `v2`.
- `v2/web`: UI/protocol client for `v2-server`.

## Forbidden Coupling

- No direct dependency from any `v2` crate to `petri-core`, `petri-server`, `petri-cli`, or legacy `petri-graph`.
- No imports from root `web/` into `v2/web`.
- No edits to legacy runtime code while implementing `v2` stages.
