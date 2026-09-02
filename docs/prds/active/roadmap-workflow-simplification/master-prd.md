# Roadmap Workflow Simplification — Master PRD

- Status: Ready
- Owner: Codex
- Created: 2026-09-01
- Review Status: APPROVED
- Review Count: 1
- Execution Mode: Deep
- Deep Mode Authorization: User explicitly authorized one Deep migration in the 2026-09-01 roadmap-workflow planning session and implementation request.
- Affected-File Budget: 60
- Actual Affected Files: Pending

## Goal

Replace Petri's active master-and-stage PRD governance with a small, durable
roadmap execution contract: stable repository invariants in `AGENTS.md`,
multi-track roadmap templates, one flat technical spec per executable feature,
a state-and-link validator, and a reusable Codex goal prompt.

## Scope

- Add the roadmap, track, feature-spec, and long-running goal prompt templates.
- Add dependency-free structural and lifecycle validation for future roadmaps.
- Retire the active PRD skills, templates, scripts, checks, and live guidance.
- Preserve completed PRDs and the V3 roadmap as non-executable history.
- Synchronize contribution, CI, pre-commit, provenance, and compatibility docs.

## Non-Goals

- Authoring the ecosystem-and-cognition roadmap, its tracks, or its features.
- Changing simulation, server, frontend, persistence, or public runtime behavior.
- Adding a general workflow skill or retaining master-and-stage feature specs.
- Pushing, opening a pull request, or merging the topic branch into `main`.

## Inputs and Existing-Code Interactions

Petri currently repeats orchestration policy across root `AGENTS.md`, four
repository skills, two templates, six lifecycle scripts, Makefile/CI/pre-commit
wiring, and contribution documentation. The reference project at
`/Users/istefanek/projects/kubernetes_training_for_ecs_engineers` demonstrates
the selected separation: stable repository rules, a roadmap-owned execution
queue, one flat feature spec, and orchestration supplied by the goal prompt.

Official OpenAI documentation says Codex loads layered `AGENTS.md` guidance
before work and that long-running goals should state their outcome, constraints,
and review criteria in the prompt:

- https://learn.chatgpt.com/docs/agent-configuration/agents-md
- https://learn.chatgpt.com/docs/long-running-work
- https://learn.chatgpt.com/docs/environments/git-worktrees

Alternatives considered were patching only the P2 conflict and retaining a
smaller PRD validator. The user selected full retirement because either option
would preserve duplicated orchestration state and the same drift risk.

## Boundaries and Abstraction Layers

Stage 01 owns the new documentation schema, goal-prompt contract, validator, and
tests. Stage 02 owns repository governance cleanup and all removal/migration
wiring. The checker validates document identity and truthful state; it must not
encode model choice, review recursion, severity, or worktree orchestration.
Those execution policies live only in the reusable goal prompt.

## Separation of Concerns and Decomposition

The replacement contract must exist and pass focused tests before the current
workflow is removed. Keeping retirement separate makes the cutover reviewable
and ensures the final `make check` has a valid policy gate throughout.

## Tech Debt and Spaghetti-Code Implications

The current PRD system is the debt being retired: it duplicates process policy
across prose, skills, templates, and shell validators. The replacement adds one
small parser because dependency and completion consistency are objective state
that review alone cannot reliably keep synchronized. It uses only Node standard
library APIs already available in the repository toolchain.

## Documentation Impact and Synchronization

Add roadmap authoring/execution guidance and templates. Update `AGENTS.md`,
`CONTRIBUTING.md`, the PR template, documentation indexes, strategy goals and
architecture, and root compatibility stubs. Move the prior V3 roadmap to the
archive. Preserve completed PRDs with an explicit historical-only notice.

## Stage Order and Links

1. [Stage 01 — Roadmap Contract](stage-01-roadmap-contract.md) — `Ready`
2. [Stage 02 — Workflow Retirement](stage-02-workflow-retirement.md) — `Ready`

Stage 01 establishes the replacement contract and validation. Stage 02 switches
all live guidance and automated gates to that contract, archives this migration,
and removes the replaced machinery.

## Cross-Stage Decisions

- `docs/roadmap.md` is the optional live master; its absence is valid only when
  no live track roadmap or feature spec exists.
- Master rollups use `TNN`; executable features live only in track documents and
  use `TNN.FNN`; feature specs use normalized `tnn-fnn-<slug>.md` paths.
- A feature has at most one flat spec, created just in time by its planning
  phase. Oversized work is split into dependency-linked features rather than
  master-and-stage specifications.
- The roadmap checker enforces paths, links, IDs, dependency graphs, and
  completion truth, but not agent orchestration policy.
- Future roadmap execution uses P1-only blocking. This migration retains the
  current P1/P2 blocking gate until the cutover is complete.
- Readiness budget: one independent review, one bounded revision, and one final
  readiness check. Implementation budget: one pre-retirement review and one
  final cutover checkpoint, each with at most one bounded P1/P2 remediation pass
  and one closure review.

## Normative Roadmap Schema

Live documents and templates use these exact interfaces:

- Optional master: `docs/roadmap.md`; template:
  `docs/roadmaps/_master-template.md`.
- Live track: `docs/roadmaps/tNN-<kebab-slug>.md`; template:
  `docs/roadmaps/_track-template.md`.
- Live feature spec: `docs/specs/roadmap/tNN-fNN-<kebab-slug>.md`; template:
  `docs/specs/roadmap/_feature-template.md`.
- Goal prompt template: `docs/roadmaps/_goal-prompt-template.md`.
- Files beginning with `_` are templates and never live execution state.

The master requires `**Status**: Planning | Active | Complete`,
`**Last updated**: YYYY-MM-DD`, and the headings `Success Definition`, `Track
Roadmaps`, `Final Success Criteria`, and `Notes for AI Agents`. Each track row is
exactly:

`- [ ] **T01 — <title>** — [Roadmap](roadmaps/t01-<slug>.md) — Depends on: None`

or uses a comma-separated list of `TNN` IDs after `Depends on:`. Track files
require status `Planned | In Progress | Complete`, the same date field,
`**Master**: [Program Roadmap](../roadmap.md)`, and the headings `Goal`, `Track
Success Criteria`, `Executable Features`, and `Notes for AI Agents`. Each feature
row is exactly:

`- [ ] **T01.F01 — <title>** — Depends on: None`

or uses a comma-separated list of `TNN.FNN` IDs. The track path ID must match its
rows' prefix. Track and feature dependency graphs are independently acyclic.

Feature specs require status `Planned | In Progress | Blocked | Complete`, the
date field, `**Feature**: T01.F01`, the exact owning-track metadata line
`**Track**: [T01 — <title>](../../roadmaps/t01-<slug>.md)`, and the headings
`Overview`, `Goal`, `Non-Goals`, `Inputs and Invariants`, `Implementation Tasks`,
`Verification`, `Success Criteria`, `Blocker`, `Deferred Review Findings`, and
`Notes for AI Agents`.

Lifecycle equations are exact:

- Without `docs/roadmap.md`, no live track or feature-spec file may exist.
- Every live track is linked exactly once by the master. An unchecked feature
  may have no spec until its planning phase, but any live spec maps bijectively
  to exactly one feature through the canonical Track link; no feature may have
  more than one spec. Duplicate IDs or normalized ID prefixes are invalid.
- A checked feature has exactly one `Complete` spec and all its dependencies are
  checked. A `Complete` spec requires its feature checked and no unchecked box
  in its implementation, verification, or success sections.
- A spec in `In Progress`, `Blocked`, or `Complete` requires all dependencies
  checked. A `Blocked` spec remains unchecked and its `Blocker` section is
  neither empty nor `None.`.
- A `Planned` master or track has no checked rollup/feature. A `Complete` track
  has at least one feature, all features and track success criteria checked, and
  its master rollup checked. The rollup is checked if and only if the track is
  `Complete`.
- An `Active` master has at least one track and is not complete. A `Complete`
  master has at least one track and all rollups and final success criteria
  checked.

## Automation Interfaces

- `scripts/roadmap-check.mjs [--root <repository>] [--staged]` validates the
  working tree by default, an explicit fixture root with `--root`, or a temporary
  materialization of the Git index with `--staged`. Staged mode covers additions,
  deletions, and renames; it exits successfully without materializing when no
  roadmap/spec path is staged. The options are mutually exclusive.
- The checker accumulates deterministic diagnostics, writes each as
  `roadmap-check: <message>` to stderr, exits 1 for validation failure, exits 2
  for usage errors, and prints one success/no-op line on exit 0.
- `scripts/roadmap-check.test.mjs` uses Node's standard `node:test` and temporary
  fixture roots. `make roadmap-check` runs the checker; `make
  roadmap-check-test` runs the suite through the Aqua-managed Node runtime.
- `scripts/project-precommit` runs staged roadmap validation before repository
  policy, audits, quality, and secrets. `make policy-check` runs the working-tree
  checker, its tests, residual-policy tests, then `scripts/policy-check`. The CI
  policy job runs only `make policy-check quality-check`.
- `scripts/residual-cruft-check [--root <repository>]` rejects live references to
  `project-workflow`, the three `prd-*` skills, `docs/prds/active`,
  `docs/prds/templates`, `scripts/prd-*`, readiness review-count metadata, and
  Lean/Deep PRD modes. It additively preserves the current Superpowers and
  retired feature-lifecycle denylist. Only `docs/prds/archive/` and
  `docs/archive/` are excluded content roots. The scanner skips its two named
  implementation/harness files, `scripts/residual-cruft-check` and
  `scripts/residual-cruft-check-test`, solely to avoid matching their embedded
  denylist literals. The test proves live failures, both archive exclusions,
  safe scanner self-treatment, and preservation of the earlier denylist.
- The real goal prompt template is tested for the exact model/role assignments
  `gpt-5.6-sol` xhigh planner, independent `gpt-5.6-sol` high readiness and diff
  reviewers, and persistent `gpt-5.6-luna` high implementer; P1-only blocking;
  one readiness revision; one remediation pass; `roadmap/complete`; per-feature
  branches/worktrees; focused checks; post-review `make check`; truthful closure;
  explicit local branch/worktree/commit authorization; and explicit push, PR,
  and user-main-merge prohibition.

## Affected Path Manifest

Create the five contract documents named above plus
`scripts/roadmap-check.mjs`, `scripts/roadmap-check.test.mjs`, and
`scripts/residual-cruft-check-test`.

Modify `AGENTS.md`, `CONTRIBUTING.md`, `.github/pull_request_template.md`,
`docs/README.md`, `docs/prds/archive/README.md`, strategy goals/architecture,
both root roadmap compatibility stubs, `Makefile`, `.github/workflows/ci.yml`,
`scripts/project-precommit`, `scripts/policy-check`,
`scripts/residual-cruft-check`, `scripts/generate-skills-lock.mjs`,
`scripts/skill-provenance-check`, and `.agents/skills.lock.json`.

Delete the four workflow skill directories and matching `.claude/skills/`
symlinks, `docs/prds/active/`, `docs/prds/templates/`, and all six `scripts/prd-*`
files after this record moves to the archive.

Preserve the former `docs/strategy/roadmap.md` content as
`docs/archive/v3-program-roadmap.md`, replace its old path with a pointer to the
roadmap contract, and move this PRD set to
`docs/prds/archive/roadmap-workflow-simplification/` during policy handoff.

The final root `AGENTS.md` retains these current invariants semantically:
runtime-facing state derives from applied simulation behavior; backward
compatibility is not a default; TDD covers behavior changes and bug fixes;
determinism is required only for reproducibility-dependent assertions;
`$rust-skills` applies to every Rust change; viability runs first for defaults,
founders, or tick mechanics; shell automation is POSIX `sh`; user changes are
preserved; `make check` is the completion gate; and commits, remotes, PRs, and
other external state require explicit authorization. All model, readiness,
severity, review-loop, and worktree orchestration clauses move only to the goal
prompt. The root also retains the explicit prohibition on Superpowers and the
older retired feature lifecycle; historical material remains non-executable.

## One-Time Bootstrap and Policy Handoff

The user explicitly authorizes this self-retirement exception to the current
archive-helper requirement. It is bounded to this migration and does not become
future workflow guidance.

1. Old policy governs implementation and a pre-retirement review while the PRD
   tools still exist. Run Stage 01 tests, `scripts/prd-check`, policy/quality
   checks, and review the contract plus cutover manifest. One review, one bounded
   fix pass, and one closure review are allowed. Unresolved P1/P2 stops work.
2. After that pass, apply the complete cutover, including old-tool removal.
   Manually move this still-`In Progress` PRD set to its archive path because its
   archive helper is retired. That temporary status is allowed only inside the
   uncommitted cutover diff. If cutover fails, apply a compensating patch back to
   the prior topic-branch commit; never reset or alter `main`.
3. Run focused new-policy checks and full `make check`, then review the entire
   base-to-worktree diff. One review, one bounded fix pass, and one closure review
   are allowed. After the first review and any fixes, record evidence, affected
   files, completed statuses, and the review gate in the archived PRD before the
   closure review. Unresolved P1/P2 stops work.
4. Run final `make check` and `git diff --check` on the exact closure tree, create
   the final local commit, and leave branch/worktree intact. After a committed
   handoff, rollback uses a normal revert commit.

Maximum readiness reviews: 2. Maximum implementation review invocations: 4.
Maximum remediation passes: 2 total, one per checkpoint.

## Acceptance Criteria and Evidence

| ID | Acceptance criterion | Verification | Evidence |
| --- | --- | --- | --- |
| AC-1 | The roadmap contract and regression fixtures validate all required identity, dependency, and completion invariants. | `make roadmap-check roadmap-check-test` | Pending |
| AC-2 | Live repository guidance and automation no longer require the retired PRD workflow. | `scripts/residual-cruft-check` and targeted `rg` audit | Pending |
| AC-3 | Curated skill provenance remains internally consistent after workflow-skill removal. | `scripts/skill-provenance-check` | Pending |
| AC-4 | All project completion gates pass with no runtime behavior changes. | `git diff --check`, `make quality-check`, and `make check` | Pending |

## Implementation or Decision Tasks

- [ ] Keep stage links, statuses, affected-file count, and evidence current.
- [ ] Complete the independent readiness review before implementation.
- [ ] Complete the pre-retirement and final cutover reviews within the declared budget.
- [ ] Remove the old lifecycle tooling and move this still-In-Progress record to
  the archive in the same authorized cutover; mark it Complete only after the
  full-diff review and evidence update.

## Verification and Observable Success Criteria

- [ ] Run focused checks during implementation and record their results in the evidence table.
- [ ] Run `make check` before the final cutover review and again on the exact post-archive closure tree.
- [ ] Every stage's declared verification has passed.
- [ ] Affected durable documentation is created, updated, or synchronized, or a no-change rationale is recorded.
- [ ] The final-code review gate has passed.

## Current Status

Ready. The user authorized one new bounded correction/review cycle after the
initial readiness budget was exhausted. Its independent final check found no
P1/P2/P3 findings, so this PRD is approved for implementation. Local
worktree/branch/commit authorization remains in scope; no remote or `main`
mutation is authorized.
