# AGENTS.md

Project-level instructions for coding agents working in this repository.

## Mission

Build the Petri simulation incrementally while preserving:
- deterministic testability
- clear crate boundaries
- reproducible toolchains
- strict quality gates

## Repository Layout

- `crates/petri-core`: simulation world state and tick logic
- `crates/petri-graph`: controller graph types/evaluation/mutation
- `crates/petri-server`: REST + WebSocket transport around `petri-core`
- `crates/petri-cli`: headless simulation and ablation runner
- `web/`: React + TypeScript canvas client
- `docs/plans/`: implementation plans and execution notes

## Required Workflow

1. Read `README.md` and relevant crate/module before editing.
2. For multi-step changes, write a plan first in `docs/plans/YYYY-MM-DD-<topic>.md`.
3. Prefer isolated worktrees under `.worktrees/` and branch names with `codex/` prefix.
4. Use TDD for behavior changes and bug fixes:
   - add/adjust failing test first
   - implement minimal fix
   - keep tests green while refactoring
5. Keep commits focused and atomic.
6. If asked for code review:
   - run Gemini MCP review first via `gemini-analyze-code`
   - if unavailable, explicitly say it is unavailable and run a local fallback review

## Toolchains

- Rust is pinned via `rust-toolchain.toml` (`1.93.0`) and should be used through `rustup`.
- Web toolchain is pinned in `web/package.json` via Volta (`node 25.6.0`, `npm 11.8.0`).

## Common Commands

- Format check: `cargo fmt --all --check`
- Tests: `cargo test --workspace`
- Lint (required): `cargo clippy --workspace --all-targets -- -D warnings`
- Run server: `cargo run -p petri-server`
- Run CLI: `cargo run -p petri-cli -- run --ticks 1000 --sample-every 25`
- Run ablation: `cargo run -p petri-cli -- ablation --ticks 500`
- Web build: `cd web && npm run build`

## Architecture Guardrails

- Keep simulation policy in `petri-core`; keep transport policy in `petri-server`.
- Do not add web concerns to Rust core crates.
- Do not add server transport concerns to `petri-core`.
- Keep `petri-graph` focused on controller representation/eval/mutation.

When changing frame/protocol types:
- update Rust producer and TypeScript consumer together
- keep wire-format changes covered by tests

## Project Invariants

- `initial_creatures` is best-effort (bounded by occupancy and max creatures).
- `food_growth_rate` must be honored directly (no hidden minimum floor).
- Localhost defaults (`127.0.0.1`) are intentional unless explicitly changed.
- Frame food payload is quantized bytes (`0..255`) for transport efficiency.

## Testing Expectations

- Every bug fix gets a regression test near the affected module.
- Prefer deterministic seeds for simulation tests.
- Add payload/perf regression checks when changing frame shape/size.
- Favor targeted test runs during iteration; run full workspace suite before completion.

## Completion Gate (Do Not Skip)

Before claiming completion, run and confirm all pass:

1. `cargo fmt --all --check`
2. `cargo test --workspace`
3. `cargo clippy --workspace --all-targets -- -D warnings`
4. `cd web && npm run build`

## Git Hygiene

- Never revert unrelated user changes.
- Avoid destructive commands (`git reset --hard`, `git checkout -- .`) unless explicitly requested.
- Revert incidental lockfile churn if dependencies were not intentionally changed.
- After merge, clean up temporary worktrees/feature branches when requested.

## Documentation Hygiene

- Update `README.md` for user-visible behavior or command changes.
- Keep plan docs in `docs/plans/` for non-trivial work.
- Keep instructions concise and operational; avoid aspirational text with no enforcement value.
