# Petri

This repository contains legacy simulation stacks and an active V3 architecture/refactor target.

## Active Direction

- Canonical architecture/planning/specs live under `docs/`.
- The active target is V3 (`v3/`) as defined by `docs/strategy/` and `docs/reference/`.
- Legacy stacks under `crates/`, `v2/`, and root `web/` are retained for maintenance and historical traceability.

## Documentation Entrypoints

- `docs/`: canonical documentation root
- `docs/strategy/`: active architecture direction and goals
- `docs/reference/`: active contracts/specs
- `docs/plans/`: active plans
- `docs/standards/`: policy and quality gates
- `docs/operations/`: doc operations/harness guidance
- `docs/*/archive/`: historical docs

Compatibility note: `petri-roadmap.md`, `petri-architecture.md`, and `petri-technology-review.md` are root compatibility stubs.

## Legacy Runtime Quickstart (Optional)

Run legacy server:

```bash
cargo run -p petri-server
```

Run legacy web client:

```bash
cd web
npm install
npm run dev
```

Run legacy CLI:

```bash
cargo run -p petri-cli -- run --ticks 1000 --sample-every 25
```

## Verification

```bash
cargo test --workspace
cd web && npm run build
```
