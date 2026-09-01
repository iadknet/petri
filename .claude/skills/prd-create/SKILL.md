---
description: Create or materially revise a master-and-stage PRD set for repository work that needs researched choices, decomposition, boundaries, and observable verification.
license: MIT
metadata:
    github-path: skills/agent-project-scaffold/assets/project-skills/prd-create
    github-ref: 83ef0fb281a17c28c584a410b371d1afe7c6b37b
    github-repo: https://github.com/iadknet/new_app_scaffolding_skills
    github-tree-sha: 1de0efaf0e47f6ebc74aa38f93b6e18ee740e49c
name: prd-create
---
# Create a PRD Set

Turn an outcome into a lean, executable PRD set under
`docs/prds/active/<kebab-slug>/`.

## Workflow

1. Read `AGENTS.md`, relevant active PRDs, repository code, and durable decision
   records before proposing structure. Distinguish observed facts from assumptions.
2. Research credible existing solutions, standards, and current primary sources
   when a technology or design choice is involved. Record links, tradeoffs, and why
   rejected options do not fit; do not invent custom machinery without evidence.
3. Run `scripts/prd-new <slug> <stage-slug> [stage-slug ...]` for a new set.
   Revise existing files in place when the set already exists.
4. Keep the master at outcome and cross-stage level. Put concrete implementation
   and decision tasks in dependency-ordered stages.
5. For every master and stage, make scope, non-goals, existing-code interactions,
   boundaries, abstraction level, separation of concerns, technical-debt impact,
   documentation impact, tasks, and observable verification specific. Identify
   durable user, operator, developer, architecture, API, and generated-reference
   documentation to create, update, or synchronize. When none is affected, record
   a concrete no-change rationale. Write `None identified` when the debt analysis
   genuinely finds none.
6. Split a stage when it mixes independently testable components, unrelated
   abstraction levels, or tasks with different dependencies. Do not split merely
   to make files short.
7. Keep each PRD at or below 750 physical lines and all checkboxes truthful. New
   and materially revised PRDs start with status `Draft`, `Review Status: DRAFT`,
   and `Review Count: 0`; `$prd-review` owns subsequent review-state changes.
8. Run `scripts/prd-index` and `scripts/prd-check` before handing off.

Do not implement product code while using this skill unless the user separately
authorizes implementation.
