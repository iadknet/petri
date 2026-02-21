# petri-server AGENTS.md

Local instructions for `crates/petri-server`.

## Scope

`petri-server` is a legacy maintenance surface.
For net-new architecture/contracts, treat V3 docs under `docs/strategy/` and
`docs/reference/` as canonical.

When touching this crate, keep transport and lifecycle orchestration around
`petri-core` here: REST endpoints, WebSocket frame streaming, startup/runtime
config patching, and simulation lifecycle management.

## Boundary Rules

- Keep simulation policy in `petri-core`; do not reimplement policy logic here.
- Keep wire/protocol serialization concerns in this crate.
- Avoid introducing frontend rendering concerns.

## Protocol Coordination

- When changing frame/API payloads, update Rust producer and TypeScript consumer together.
- Keep payload regression tests current (`crates/petri-server/tests/payload_regression.rs`).
- Preserve localhost defaults unless explicitly changed.

## Local Test Strategy

- Fast loop: `cargo test -p petri-server <test_name> -- --exact`
- Crate sweep: `cargo test -p petri-server`
- Contract changes require full workspace + frontend build verification.

## Related Canonical Docs

- Root policy: `AGENTS.md`
- Docs entrypoint: `docs/`
- Active strategy docs: `docs/strategy/`
- Active reference specs: `docs/reference/`
- Archived reference specs: `docs/reference/archive/`
- Web counterpart guidance: `web/AGENTS.md`
