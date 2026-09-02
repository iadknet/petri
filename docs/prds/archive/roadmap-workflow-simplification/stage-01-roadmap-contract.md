# Stage 01 — Roadmap Contract

- Status: Complete
- Depends on: None
- Master: [Master PRD](master-prd.md)

## Goal

Provide a documented and mechanically checked contract from which a future
multi-track Petri roadmap and autonomous Codex goal can be created without
restoring repository-wide orchestration policy.

## Scope

- Roadmap, track, feature-spec, and goal-prompt templates plus author guidance.
- A dependency-free Node checker and regression suite.
- Makefile and pre-commit interfaces for the new checker.

## Non-Goals

- Removing the current PRD system or changing live repository instructions.
- Creating `docs/roadmap.md` or any ecosystem feature content.
- Adding product code, dependencies, or runtime interfaces.

## Inputs and Existing-Code Interactions

Use the accepted master/track/spec design from the user-approved plan and the
flat feature-spec pattern proven in the Kubernetes training reference project.
The checker runs in the existing Aqua-managed Node runtime and must use only
standard library modules. It integrates with `make` and the existing project
pre-commit script without changing application build or test behavior.

## Boundaries and Abstraction Layers

Markdown owns human intent and executable state. The checker owns syntactic and
graph consistency. The goal prompt owns planner/reviewer/implementer selection,
severity policy, worktree lifecycle, verification timing, integration, and
closure. No runtime crate or frontend boundary changes.

The normative Markdown grammar, lifecycle equations, CLI/error contract, goal
prompt clauses, and automation call graph are defined in the master PRD and are
inputs to this stage rather than implementation-time choices.

## Separation of Concerns and Decomposition

Templates and validation form one contract: fixtures prove the exact Markdown
shape the templates advertise. Splitting either would create an interval where
the documentation and validator disagree.

## Tech Debt and Spaghetti-Code Implications

Avoid a generic Markdown framework. Parse only the canonical lines and sections
defined by these templates, report all actionable errors in one invocation, and
keep the implementation dependency-free.

## Documentation Impact and Synchronization

Create `docs/roadmaps/README.md`, `_master-template.md`, `_track-template.md`,
`_goal-prompt-template.md`, and `docs/specs/roadmap/_feature-template.md`.

## Acceptance Criteria and Evidence

| ID | Acceptance criterion | Verification | Evidence |
| --- | --- | --- | --- |
| AC-1 | An absent live master passes only when no live tracks or specs exist. | Empty-scaffold checker fixture | `make roadmap-check-test`: 25 tests passed, including templates-only, present-empty-master, nested-live-without-master, and live-without-master cases. |
| AC-2 | IDs, canonical paths, links, unknown dependencies, duplicate ownership, and cycles are rejected. | Graph and path checker fixtures | `make roadmap-check-test`: canonical identity, dependency, later-listed cross-track, duplicate, orphan, noncanonical filename, malformed metadata/path, exact owning-track link, and cycle cases passed; checker emits deterministic diagnostics. |
| AC-3 | Checked features, complete specs, blocked specs, track rollups, and master completion stay synchronized. | Lifecycle checker fixtures | `make roadmap-check-test`: complete closure, blocked-spec, nested/ordinary unchecked tasks, premature completion, invalid Planning/Active states, and rollup-drift cases passed. |
| AC-4 | The goal prompt contains the requested P1-only, model-role, worktree, verification, integration, and authorization boundaries. | Diff-scoped documentation review | `docs/roadmaps/_goal-prompt-template.md` records the requested role models, P1-only policy, branch/worktree, focused checks, `make check`, closure, and authorization boundaries. |

The regression matrix is:

- Positive: absent master with templates only; active two-track roadmap with a
  cross-track dependency; complete feature/spec/track/master closure; blocked
  spec with a concrete blocker; staged no-op and staged add/delete/rename.
- Negative: live docs without a master; malformed IDs/paths/links/metadata;
  duplicate IDs; orphan track/spec; unknown dependency; track or feature cycle;
  prefix or owning-track-link mismatch; multiple specs for one feature;
  premature feature/spec completion; unchecked completed-spec tasks; blocked
  feature checked or missing blocker; stale track rollup; invalid Planning,
  Active, or Complete state.
- Repository contract: the real goal template contains every exact clause listed
  in the master's Automation Interfaces section.

## Implementation or Decision Tasks

- [x] Write the roadmap contract and four reusable templates without ecosystem content.
- [x] Implement the standard-library roadmap parser and validator.
- [x] Add fixtures for valid empty and multi-track states plus every required failure mode.
- [x] Expose the declared working-tree, fixture-root, and staged-index commands
  through the exact Makefile/pre-commit call graph.

## Verification and Observable Success Criteria

- [x] Run a focused check and replace `Pending` in the evidence table with the observable result.
- [x] Affected durable documentation is created, updated, or synchronized, or a no-change rationale is recorded.

## Current Status

Complete. The contract templates, dependency-free checker, regression suite,
Makefile interfaces, and staged pre-commit hook are implemented and their
focused verification evidence is recorded above.
