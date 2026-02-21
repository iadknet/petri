# Petri — V2 Technology Review

## Scope

This review reflects the active `v2/` implementation stack and immediate checkpoint needs.
Legacy stack details are historical reference and not the active target for this phase.

## V2 Runtime Stack (Current)

Rust workspace:
- `v2-core`
- `v2-server`
- `v2-cli`

Core dependency posture:
- `v2-core`: std-only runtime primitives at current checkpoint baseline.
- `v2-server`: `serde`, `serde_json`, `sha2`, `hex`, `v2-core`.
- `v2-cli`: `serde`, `serde_json`, `v2-core`.

## V2 Web Stack (Current)

- React 18
- TypeScript 5
- Vite 6
- Vitest 3 (unit/integration protocol tests)
- Playwright (CP-3 gate smoke coverage)

## Contract and Tooling Posture

- Server and CLI serialize explicit `v2alpha1` payloads.
- Web consumption is model/decoder-driven, fixture-backed.
- Verification gates are checkpoint-owned via active docs in `docs/plans/`
  (historical snapshots live in `docs/plans/archive/`).

## Rejected Assumptions for This Phase

- No requirement to maintain legacy API backward compatibility.
- No requirement for snapshot export/import in `v2alpha1`.
- No requirement for full-run deterministic replay as a product contract.

## Dependency Selection Rules

When adding dependencies in this phase, require:
1. clear checkpoint-scoped need;
2. explicit boundary ownership (`v2-core` vs `v2-server` vs `v2-cli` vs `v2-web`);
3. deterministic testability impact documented;
4. minimal overlap with existing stack.
