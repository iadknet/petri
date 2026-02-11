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

For frontend work under `web/`, follow `web/AGENTS.md`.

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
7. For Rust structural refactors (module splits, moving tests, boundary cleanup), include a short "Boundary Impact" note in the plan covering:
   - crate dependency direction (`petri-graph -> petri-core -> petri-server/petri-cli`) remains unchanged
   - public API/wire-format changes (expected none unless explicitly requested)
   - test migration approach (unit vs integration)

## Modularity Rules (Rust)

- Keep `src/lib.rs` export-focused (`mod` + `pub use` surface); avoid placing integration-style tests in `src/lib.rs`.
- Split oversized modules by responsibility using `module/mod.rs + submodules` when concerns are mixed.
- Prefer `pub(crate)` or private visibility by default; only expose `pub` items that are part of crate API.
- Keep transport concerns in `petri-server`, simulation policy in `petri-core`, and graph representation/eval/mutation in `petri-graph`.
- Separate behavioral refactors from behavior changes whenever practical (first move/split, then modify behavior in follow-up commits).

### Refactor Triggers

- If a production Rust file grows beyond ~400 lines, evaluate splitting it by concern.
- If a production Rust file exceeds ~600 lines, split is required unless documented with a concrete reason.
- If one module has multiple independent change reasons (for example lifecycle + serialization + transport mapping), split by concern before adding more logic.
- If integration-style tests dominate a source file, move them under `crates/<crate>/tests/`.

### Module Layout Pattern

- Use `mod.rs` for public surface + shared types.
- Put concern-specific logic in focused submodules (for example `tick.rs`, `snapshot.rs`, `perception.rs`).
- Keep low-level helpers in dedicated helper modules to avoid leaking utility details into API-facing modules.

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

## Rust Test Placement

- Unit tests for private logic should live near the code (`#[cfg(test)]` in module file or `<module>/tests.rs`).
- Integration tests should live in `crates/<crate>/tests/*.rs` and use public crate APIs.
- HTTP/transport, cross-module behavior, and payload-regression tests should be integration tests, not `src/lib.rs` tests.
- Keep test files grouped by behavior (for example health, CORS, lifecycle, payload regression) to avoid monolithic test modules.

## Rust Test Strategy

- Use the narrowest Rust test scope during iteration:
  - Single regression: `cargo test -p <crate> <test_name> -- --exact`
  - Touched crate sweep: `cargo test -p <crate>`
- Keep the full Rust gate before completion: `cargo test --workspace`.
- Run `cargo test --workspace` earlier when changes touch shared cross-crate behavior (core types, protocol contracts, workspace dependencies, or crate interfaces).
- Rust tests already run in parallel by default.
  - Do not use `--test-threads=1` unless debugging ordering/flaky behavior.
  - Only tune thread count when you have measured evidence it is faster on this machine.
  - Optional tuning command: `cargo test -p petri-server -- --test-threads=<n>`

## Completion Gate (Do Not Skip)

Before claiming completion, run and confirm all pass:

1. `cargo fmt --all --check`
2. `cargo test --workspace`
3. `cargo clippy --workspace --all-targets -- -D warnings`
4. `cd web && npm run build`
5. For Rust modularity refactors, include a brief size/boundary report in the final handoff:
   - largest production Rust files and whether they need splitting
   - confirmation that `src/lib.rs` files are export-focused
   - confirmation that integration-style tests primarily live under `crates/*/tests/`

## Git Hygiene

- Never revert unrelated user changes.
- Avoid destructive commands (`git reset --hard`, `git checkout -- .`) unless explicitly requested.
- Revert incidental lockfile churn if dependencies were not intentionally changed.
- After merge, clean up temporary worktrees/feature branches when requested.

## Documentation Hygiene

- Update `README.md` for user-visible behavior or command changes.
- Keep plan docs in `docs/plans/` for non-trivial work.
- Keep instructions concise and operational; avoid aspirational text with no enforcement value.
