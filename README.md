# Petri

This repository contains the active Petri V3 evolutionary simulation.

## Active Direction

- Canonical architecture/planning/specs live under `docs/`.
- The active Rust workspace is at the repository root, with members under `crates/`, as defined by `docs/strategy/` and `docs/reference/`.

## Documentation Entrypoints

- `docs/`: canonical documentation root
- `docs/strategy/`: active architecture direction and goals
- `docs/reference/`: active contracts/specs
- `docs/features/`: active feature lifecycle and implementation plans
- `docs/plans/`: legacy planning records
- `docs/*/archive/`: historical docs

Compatibility note: `petri-roadmap.md`, `petri-architecture.md`, and `petri-technology-review.md` are root compatibility stubs.

## Project Policies

- [License](LICENSE)
- [Contributing](CONTRIBUTING.md)
- [Security](SECURITY.md)

## Development

Start backend + frontend together from repo root:

```bash
./scripts/dev.sh
```

Build or run the backend directly from the repository root:

```bash
cargo build
cargo run -p v3-server
```

## Verification

```bash
cargo test --workspace
```

## Frontend E2E (Local Only)

```bash
cd frontend && npm run test:e2e
```
