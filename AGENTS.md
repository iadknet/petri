# AGENTS.md

Project-level instructions for coding agents working in this repository.

## Mission

Build the Petri simulation incrementally while preserving deterministic testability, clear crate boundaries, reproducible toolchains, and strict quality gates.

## Golden Rule

- Completion requires verified functional intent and integration behavior; contract/shape checks alone are never sufficient.

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
- Runtime behavior telemetry must be truthfully simulation-derived: per-tick creature state, action counts, and energy/population metrics must come from applied world tick behavior, not synthetic placeholders.
- Frame food payload stays quantized byte density (`0..255`) unless explicitly changed.
- `initial_creatures` remains best-effort (bounded by occupancy and max creatures).
- `food_growth_rate` is honored directly (no hidden minimum floor).
- Localhost defaults (`127.0.0.1`) are intentional unless explicitly changed.

## Required Workflow

1. Read `README.md` and relevant local `AGENTS.md` before editing.
2. For multi-step changes, write or update a plan in `docs/plans/`.
3. Use isolated branches/worktrees with `codex/` branch prefix.
4. For behavior changes and bug fixes, use TDD (failing test first).
5. For feature/checkpoint completion claims, require intent-level + integration-level regression evidence (not only contract tests).
6. Keep commits focused and atomic.
7. For code review requests, run Gemini MCP (`gemini-analyze-code`) first; if unavailable, state that and run a local fallback review.
8. For Rust structural refactors, include a short "Boundary Impact" note in the plan covering dependency direction, public API/wire-format changes, and test migration approach.

## Plan Splitting Rule

Large plans **must** be split into multiple files:

- If a plan exceeds ~200 lines or covers more than one stage/checkpoint, split it.
- Use a **main plan file** that defines scope, goals, and a high-level task outline.
- Place detailed designs, specs, or per-stage breakdowns in **separate companion files** in the same directory.
- The main plan must include explicit `**See also:**` references linking to each companion file.
- Companion files must include a `**Parent plan:**` back-reference to the main plan.
- Each file (main and companion) must independently satisfy the required metadata and structure rules (Goal IDs, Goal Alignment, Boundary Impact, etc.).

## Architecture and Goal Alignment Review Cycle

After writing or substantially revising any plan, agents **must** run a review cycle before the plan is considered ready for implementation:

1. **Draft** the plan (main file + any companions).
2. **Architecture review**: Re-read the active architecture doc (currently `docs/plans/2026-02-14-v3-architecture-design.md`), the relevant crate/module `AGENTS.md` files, and the Non-Negotiable Invariants in this file. Verify every proposed change is consistent with existing boundaries, dependency directions, and module responsibilities. Document any tensions found.
3. **Goal alignment review**: Re-read `docs/strategy/goals.md` and verify that the plan's Goal Alignment section accurately maps work items to goal IDs. Confirm no goal is undermined or ignored by the proposed changes.
4. **Revise** the plan to resolve any issues found in steps 2–3.
5. **Repeat** steps 2–4 until a clean pass (no architecture conflicts, no goal misalignment). Record the number of review cycles performed at the bottom of the plan in a `**Review cycles:** N` metadata line.

This cycle is mandatory — a plan that has not completed at least one clean architecture + goal alignment pass must not be used to drive implementation.

## Skills Policy

Agents must invoke the appropriate installed skills during architecture, planning, and implementation work. Skills are auto-discovered from `~/.agents/skills/` (Codex) and `~/.claude/skills/` (Claude Code).

### Rust crates (`crates/`)

- **`rust-skills`**: Invoke when writing, reviewing, or refactoring any Rust code. Covers ownership, error handling, async patterns, API design, memory optimization, performance, and testing.
- **`cargo` MCP server** (Codex): Use for running clippy, check, test, fmt, and managing dependencies.

### Frontend (`web/`)

- **`vercel-react-best-practices`**: Invoke when writing, reviewing, or refactoring React components. Covers performance patterns, data fetching, bundle optimization.
- **`vercel-composition-patterns`**: Invoke when designing component APIs, refactoring prop-heavy components, or building reusable component hierarchies.
- **`web-design-guidelines`**: Invoke when reviewing UI for accessibility, forms, animation, typography, and design quality.
- **`frontend-design`**: Invoke when creating new UI components that need high visual design quality. Guides bold aesthetic choices in typography, color, motion, and spatial composition.

### Architecture and planning

- Before designing crate APIs, module boundaries, or trait hierarchies: invoke `rust-skills`.
- Before designing component hierarchies or state management patterns: invoke `vercel-react-best-practices` and `vercel-composition-patterns`.
- Before writing implementation plans that touch `web/`: review `web-design-guidelines` for accessibility and UX constraints.

## Completion Gate

Before claiming completion, run and confirm all pass:

1. `scripts/check-doc-harness.sh --mode warn` (through February 27, 2026)
2. `scripts/check-architecture-harness.sh --mode warn` (through February 27, 2026)
3. `scripts/check-doc-harness.sh --mode strict` (starting February 28, 2026)
4. `scripts/check-architecture-harness.sh --mode strict` (starting February 28, 2026)
5. `scripts/check-plan-harness.sh --mode strict` (effective immediately)
6. `cargo fmt --all --check`
7. `cargo test --workspace`
8. `cargo clippy --workspace --all-targets -- -D warnings`
9. `cd web && npm run build`

## Doc Touch Policy

- For any non-trivial plan, follow metadata and section requirements in `docs/plans/README.md`.
- Use `Goal IDs` from `docs/strategy/goals.md` and include explicit `Goal Alignment`, `Existing Boundary Recheck`, and `Open Questions` sections.
- Include a `Docs Impact` section listing canonical docs touched and stale docs retired/superseded.
- Prefer references to canonical docs over repeating the same narrative in multiple files.
- Avoid conflicting invariant statements across docs; update all affected canonical docs together when invariants change.
- Keep root compatibility stubs (`petri-roadmap.md`, `petri-architecture.md`, `petri-technology-review.md`) short and pointing to canonical docs.

## Canonical Docs

- Docs index: `docs/README.md`
- Strategy docs: `docs/strategy/goals.md`, `docs/strategy/roadmap.md`, `docs/strategy/architecture.md`, `docs/strategy/technology-review.md`
- Controller reference: `docs/reference/creature-controller-reference.md`
- V3 reference specs: `docs/reference/v3-vm-isa-spec.md`, `docs/reference/v3-graph-operator-spec.md`, `docs/reference/v3-genome-sensor-spec.md`, `docs/reference/v3-creature-lifecycle-spec.md`
- Operations and standards: `docs/operations/doc-hygiene.md`, `docs/standards/agent-instruction-layering.md`, `docs/standards/architecture-lint-policy.md`, `docs/standards/plan-quality-gate-policy.md`, `docs/standards/documentation-consistency-policy.md`, `docs/standards/intent-verification-policy.md`, `docs/standards/runtime-behavior-realism-policy.md`

## Git Hygiene

- Never revert unrelated user changes.
- Avoid destructive commands (`git reset --hard`, `git checkout -- .`) unless explicitly requested.
- Revert incidental lockfile churn if dependencies were not intentionally changed.
