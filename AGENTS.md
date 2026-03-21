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
- `docs/`: canonical strategy/reference/features docs

## Non-Negotiable Invariants

- Active architecture direction and contracts are defined under `docs/strategy/` and
  `docs/reference/` for V3.
- Runtime-facing telemetry/state values must be derived from applied simulation
  behavior (no synthetic placeholder metrics).
- Backward compatibility is NOT a project goal. Breaking changes are acceptable by
  default, and agents should not add migration/compatibility work unless a task
  explicitly asks for it.
- Viability tests (`v3/crates/v3-core/tests/viability.rs`) must pass before any
  branch merge. See **Viability Test Policy** below.
- **Implementation plans MUST include review gate checkmarks.** Every
  `master_plan.md` must contain explicit `- [ ] Review Gate:` checkmark items
  within `## Implementation Steps`. These are not optional polish — they are
  mandatory steps that must be completed (checked off) before invoking
  `feature-6-complete` or `finishing-a-development-branch`. See **Review Gate
  Policy** below.

## Review Gate Policy

Every `master_plan.md` must include **at minimum** these review gate checkmarks
as items in `## Implementation Steps`:

1. **Code review gate** — recursive code review using domain skills
   (`rust-skills`, `vercel-react-best-practices`, `vercel-composition-patterns`
   as applicable). Dispatch `superpowers:code-reviewer` subagent. Fix all
   findings, re-review until clean pass.
2. **Architecture & decomposition review gate** — review for boundary violations,
   decomposition opportunities, separation of concerns, and consistency with
   `docs/strategy/` architecture docs and relevant `AGENTS.md` files.

### Rules

1. Review gates are checkmark items (`- [ ] Review Gate: ...`), not prose
   policies. Agents follow checkmarks — if it is not a checkmark, it will be
   skipped.
2. Review gates must appear AFTER the implementation steps they cover, not in a
   separate section.
3. `feature-3-plan` must generate these checkmarks in every plan. The plan
   harness (`scripts/check-plan-harness.sh`) validates their presence.
4. Agents must NOT invoke `feature-6-complete` or `finishing-a-development-branch`
   until all review gate checkmarks are checked off in `master_plan.md`.
5. An inline prose comment ("looks clean") is NOT a review. A review means
   dispatching the `superpowers:code-reviewer` subagent or running domain skills
   and documenting findings.
6. **Re-verification after review-introduced changes.** If a code review finding
   leads to a code change, any previously-passed verification steps that cover
   the changed code MUST be re-run. A verification step checked off before the
   change does not count as verification of the post-change code. This includes
   build checks, test suites, and type checking.
7. **Gemini is prohibited for code review workflows in this repository.** Use
   other available review paths (e.g., `superpowers:code-reviewer`,
   Codex review, local/manual review).

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
2. For multi-step changes, use the `features:` skill workflow (see `docs/features/`).
3. Use isolated branches/worktrees with `codex/` branch prefix.
4. For behavior changes and bug fixes, use TDD (failing test first).
5. For feature/checkpoint completion claims, require intent-level + integration-level regression evidence (not only contract tests).
6. Keep commits focused and atomic.
7. For code review requests, run at least one substantive review pass using available review tooling (MCP reviewer, local static analysis, or manual diff review) and document which path was used.
8. For Rust structural refactors, include a short "Boundary Impact" note in the plan covering dependency direction, public API/wire-format changes, and test migration approach.

## Plan Splitting Rule

Large plans **must** be split into multiple files:

- If a plan exceeds ~500 lines or covers more than one major area, split it.
- Use a **main plan file** (`master_plan.md`) that defines scope, goals, and a high-level task outline.
- Place detailed designs, specs, or per-stage breakdowns in **separate companion files** in the same `FEATURE-NAME/` directory.
- The main plan must include explicit `**See also:**` references linking to each companion file.
- Companion files must include a `**Parent plan:**` back-reference to the main plan.
- Each file (main and companion) must independently satisfy the required metadata and structure rules (Goal IDs, Goal Alignment, Boundary Impact, etc.).

## Architecture and Goal Alignment Review Cycle

After writing or substantially revising any plan, agents **must** run a review cycle before the plan is considered ready for implementation:

1. **Draft** the plan (main file + any companions).
2. **Architecture review**: Re-read active architecture and planning docs in `docs/strategy/` and `docs/features/in_progress/` (exclude `docs/features/completed/` and `docs/features/cancelled/` unless doing historical comparison), the relevant crate/module `AGENTS.md` files, and the Non-Negotiable Invariants in this file. Verify every proposed change is consistent with existing boundaries, dependency directions, and module responsibilities. Document any tensions found.
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

1. `scripts/check-doc-harness.sh --mode strict`
2. `scripts/check-architecture-harness.sh --mode strict`
3. `scripts/check-plan-harness.sh --mode strict`
4. `cd v3 && cargo fmt --all -- --check`
5. `cd v3 && cargo test --workspace`
6. `cd v3 && cargo clippy --workspace --all-targets -- -D warnings`
7. `cd frontend && npm run build` (if any frontend files were touched)

## Doc Touch Policy

- For any non-trivial plan, follow the `feature-*` skill workflow and metadata/section requirements enforced by `feature-3-plan`.
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
- Feature ideas: `docs/features/brainstorms/ideas.md`
- Features in refinement: `docs/features/needs_refinement/`
- Features ready to implement: `docs/features/ready_to_implement/`
- Features in progress: `docs/features/in_progress/`
- Completed features: `docs/features/completed/`
- Cancelled features: `docs/features/cancelled/`

## Git Hygiene

- Never revert unrelated user changes.
- Avoid destructive commands (`git reset --hard`, `git checkout -- .`) unless explicitly requested.
- Revert incidental lockfile churn if dependencies were not intentionally changed.
