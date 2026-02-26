# AGENTS.md

Project-level instructions for coding agents working in this repository.

## Mission

Build the Petri simulation incrementally while preserving high-confidence testability, clear crate boundaries, reproducible toolchains, and strict quality gates.

## Golden Rule

- Completion requires verified functional intent and integration behavior; contract/shape checks alone are never sufficient.

## Instruction Layering

Use the split-responsibility model:

- Skills own **HOW** execution is performed.
- Repository docs and `AGENTS.md` own **WHAT** project constraints must hold.

See canonical policy set: `docs/strategy/` and `docs/reference/`.

## Repository Map

- `v3/`: active implementation target
- `docs/`: canonical strategy/reference/plans docs

## Non-Negotiable Invariants

- Active architecture direction and contracts are defined under `docs/strategy/` and
  `docs/reference/` for V3.
- Runtime-facing telemetry/state values must be derived from applied simulation
  behavior (no synthetic placeholder metrics).
- Viability tests (`v3/crates/v3-core/tests/viability.rs`) must pass before any
  branch merge. See **Viability Test Policy** below.

## Determinism Scope (Canonical)

- Production runtime determinism is not a product requirement.
- Deterministic behavior is required in tests/harnesses when assertions depend on reproducibility.

## Viability Test Policy

The viability tests in `v3/crates/v3-core/tests/viability.rs` are a **merge gate**.
They verify that the simulation's production economics (energy, food, costs, runtime
limits) support a self-sustaining founder population. Any change to production
defaults, founder genome behavior, or tick-loop mechanics that breaks viability is
a real regression — not a test maintenance issue.

### Rules

1. **Viability tests must pass before merge.** No exceptions. If a config change,
   founder behavior change, or runtime mechanics change causes viability failures,
   the change must be revised — not the tests.

2. **Viability configs use production defaults.** The shared `viability_config()`
   function and individual test configs must use `SimulationConfig::default()` for
   all economic parameters (energy, costs, runtime limits, mutation rates). The only
   permitted overrides are:
   - **World size** — smaller for test speed (e.g. 32×32).
   - **Population count** — fewer creatures for test speed.
   - **Food coverage/density** — increased to compensate for the smaller world.
   - **Scenario-specific setup** — tests for specific behaviors (e.g. "creature
     eats when on food") may override parameters needed to set up that scenario
     (e.g. disabling food growth, placing creatures manually), but economic
     parameters (costs, decay, rewards, runtime limits) must remain at production
     defaults.

3. **Do not "fix" viability tests by weakening assertions or inflating energy.**
   If the population goes extinct in a viability test, the correct response is to
   fix the config or behavior change that caused extinction — not to give creatures
   more starting energy, reduce decay, or shorten the test horizon.

4. **When changing production defaults**, run `cargo test -p v3-core --test viability`
   as the first validation step, before any other testing. If viability breaks,
   reconsider the default change.

## Required Workflow

1. Read `README.md` and relevant local `AGENTS.md` before editing.
2. For multi-step changes, write or update a plan in `docs/plans/`.
3. Use isolated branches/worktrees with `codex/` branch prefix.
4. For behavior changes and bug fixes, use TDD (failing test first).
5. For feature/checkpoint completion claims, require intent-level + integration-level regression evidence (not only contract tests).
6. Keep commits focused and atomic.
7. For code review requests, run at least one substantive review pass using available review tooling (MCP reviewer, local static analysis, or manual diff review) and document which path was used.
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
2. **Architecture review**: Re-read active architecture and planning docs in `docs/strategy/` and `docs/plans/` (exclude `docs/plans/archive/` unless doing historical comparison), the relevant crate/module `AGENTS.md` files, and the Non-Negotiable Invariants in this file. Verify every proposed change is consistent with existing boundaries, dependency directions, and module responsibilities. Document any tensions found.
3. **Goal alignment review**: Re-read active strategy docs in `docs/strategy/` (including the goals catalog) and verify that the plan's Goal Alignment section accurately maps work items to goal IDs. Confirm no goal is undermined or ignored by the proposed changes.
4. **Revise** the plan to resolve any issues found in steps 2–3.
5. **Repeat** steps 2–4 until a clean pass (no architecture conflicts, no goal misalignment). Record the number of review cycles performed at the bottom of the plan in a `**Review cycles:** N` metadata line.

This cycle is mandatory — a plan that has not completed at least one clean architecture + goal alignment pass must not be used to drive implementation.

## Skills Policy

Agents must invoke the appropriate installed skills during architecture, planning, and implementation work. Skills are auto-discovered from `~/.agents/skills/` (Codex) and `~/.claude/skills/` (Claude Code).

### Rust crates (`v3/`)

- **`rust-skills`**: ALWAYS invoke when writing, reviewing, or refactoring Rust code in `v3/`. Covers ownership, error handling, async patterns, API design, memory optimization, performance, and testing.

### Frontend

- **`vercel-react-best-practices`**: Invoke when writing, reviewing, or refactoring React components. Covers performance patterns, data fetching, bundle optimization.
- **`vercel-composition-patterns`**: Invoke when designing component APIs, refactoring prop-heavy components, or building reusable component hierarchies.
- **`web-design-guidelines`**: Invoke when reviewing UI for accessibility, forms, animation, typography, and design quality.
- **`frontend-design`**: Invoke when creating new UI components that need high visual design quality. Guides bold aesthetic choices in typography, color, motion, and spatial composition.

### Architecture and planning

- Before designing crate APIs, module boundaries, or trait hierarchies: invoke `rust-skills`.
- Before designing component hierarchies or state management patterns: invoke `vercel-react-best-practices` and `vercel-composition-patterns`.
- Before writing implementation plans that touch frontend surfaces: review `web-design-guidelines` for accessibility and UX constraints.

## Completion Gate

Before claiming completion, run and confirm all pass:

1. `scripts/check-doc-harness.sh --mode warn` (through February 27, 2026)
2. `scripts/check-architecture-harness.sh --mode warn` (through February 27, 2026)
3. `scripts/check-doc-harness.sh --mode strict` (starting February 28, 2026)
4. `scripts/check-architecture-harness.sh --mode strict` (starting February 28, 2026)
5. `scripts/check-plan-harness.sh --mode strict` (effective immediately)
6. `cd v3 && cargo fmt --all -- --check`
7. `cd v3 && cargo test --workspace`
8. `cd v3 && cargo clippy --workspace --all-targets -- -D warnings`

## Doc Touch Policy

- For any non-trivial plan, follow metadata and section requirements in `docs/plans/` (template rules are maintained there).
- Use `Goal IDs` from the active goals catalog in `docs/strategy/` and include explicit `Goal Alignment`, `Existing Boundary Recheck`, and `Open Questions` sections.
- Include a `Docs Impact` section listing canonical docs touched and stale docs retired/superseded.
- Prefer references to canonical docs over repeating the same narrative in multiple files.
- Avoid conflicting invariant statements across docs; update all affected canonical docs together when invariants change.
- Keep root compatibility stubs (`petri-roadmap.md`, `petri-architecture.md`, `petri-technology-review.md`) short and pointing to canonical docs.

## Canonical Docs

- Docs entrypoint: `docs/`
- Active strategy docs: `docs/strategy/`
- Active reference specs: `docs/reference/`
- Archived reference specs: `docs/reference/archive/`
- Active plans: `docs/plans/`
- Archived plans: `docs/plans/archive/`

## Git Hygiene

- Never revert unrelated user changes.
- Avoid destructive commands (`git reset --hard`, `git checkout -- .`) unless explicitly requested.
- Revert incidental lockfile churn if dependencies were not intentionally changed.
