# AGENTS.md

Project-level instructions for coding agents working in this repository.

## Mission

Build the Petri simulation incrementally while preserving deterministic testability, clear crate boundaries, reproducible toolchains, and strict quality gates.

## Instruction Layering

Use the split-responsibility model:

- Skills own **HOW** execution is performed.
- Repository docs and `AGENTS.md` own **WHAT** project constraints must hold.

See canonical policy: `docs/standards/agent-instruction-layering.md`.

## Repository Map

- `crates/petri-core`: simulation world state and tick logic (`crates/petri-core/AGENTS.md`)
- `crates/petri-graph`: controller graph types/evaluation/mutation (`crates/petri-graph/AGENTS.md`)
- `crates/petri-server`: REST + WebSocket transport around core (`crates/petri-server/AGENTS.md`)
- `crates/petri-cli`: headless runner/ablation/benchmark entrypoints (`crates/petri-cli/AGENTS.md`)
- `web/`: React + TypeScript client (`web/AGENTS.md`)
- `docs/`: canonical strategy/reference/standards/operations docs (`docs/README.md`)

## Non-Negotiable Invariants

- Crate dependency direction remains `petri-graph -> petri-core -> petri-server/petri-cli`.
- Keep simulation policy in `petri-core`; keep transport policy in `petri-server`.
- Do not add web concerns to Rust core crates.
- Frame food payload stays quantized byte density (`0..255`) unless explicitly changed.
- `initial_creatures` remains best-effort (bounded by occupancy and max creatures).
- `food_growth_rate` is honored directly (no hidden minimum floor).
- Localhost defaults (`127.0.0.1`) are intentional unless explicitly changed.

## Required Workflow

1. Read `README.md` and relevant local `AGENTS.md` before editing.
2. For multi-step changes, write or update a plan in `docs/plans/`.
3. Use isolated branches/worktrees with `codex/` branch prefix.
4. For behavior changes and bug fixes, use TDD (failing test first).
5. Keep commits focused and atomic.
6. For code review requests, run Gemini MCP (`gemini-analyze-code`) first; if unavailable, state that and run a local fallback review.
7. For Rust structural refactors, include a short "Boundary Impact" note in the plan covering dependency direction, public API/wire-format changes, and test migration approach.

## Completion Gate

Before claiming completion, run and confirm all pass:

1. `scripts/check-doc-harness.sh --mode warn` (through February 27, 2026)
2. `scripts/check-architecture-harness.sh --mode warn` (through February 27, 2026)
3. `scripts/check-doc-harness.sh --mode strict` (starting February 28, 2026)
4. `scripts/check-architecture-harness.sh --mode strict` (starting February 28, 2026)
5. `cargo fmt --all --check`
6. `cargo test --workspace`
7. `cargo clippy --workspace --all-targets -- -D warnings`
8. `cd web && npm run build`

## Doc Touch Policy

- For any non-trivial plan, follow metadata and section requirements in `docs/plans/README.md`.
- Include a `Docs Impact` section listing canonical docs touched and stale docs retired/superseded.
- Keep root compatibility stubs (`petri-roadmap.md`, `petri-architecture.md`, `petri-technology-review.md`) short and pointing to canonical docs.

## Canonical Docs

- Docs index: `docs/README.md`
- Strategy docs: `docs/strategy/roadmap.md`, `docs/strategy/architecture.md`, `docs/strategy/technology-review.md`
- Controller reference: `docs/reference/creature-controller-reference.md`
- Operations and standards: `docs/operations/doc-hygiene.md`, `docs/standards/agent-instruction-layering.md`, `docs/standards/architecture-lint-policy.md`

## Git Hygiene

- Never revert unrelated user changes.
- Avoid destructive commands (`git reset --hard`, `git checkout -- .`) unless explicitly requested.
- Revert incidental lockfile churn if dependencies were not intentionally changed.
