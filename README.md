# Petri

This repository contains the active V3 simulation architecture/refactor target.

## Active Direction

- Canonical architecture/planning/specs live under `docs/`.
- The active target is V3 (`v3/`) as defined by `docs/strategy/` and `docs/reference/`.

## Documentation Entrypoints

- `docs/`: canonical documentation root
- `docs/strategy/`: active architecture direction and goals
- `docs/reference/`: active contracts/specs
- `docs/plans/`: active plans (new work lands here)
- `docs/plans/archive/`: archived completed plans
- `docs/*/archive/`: historical docs

Compatibility note: `petri-roadmap.md`, `petri-architecture.md`, and `petri-technology-review.md` are root compatibility stubs.

## Development

Start backend + frontend together from repo root:

```bash
./scripts/dev.sh
```

## Verification

```bash
cd v3 && cargo test --workspace
```
