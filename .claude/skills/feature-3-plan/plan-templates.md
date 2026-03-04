# Plan Reference & Templates

**Parent skill:** feature-3-plan/SKILL.md

## Required Sections in master_plan.md

| Section | Content |
|---------|---------|
| Metadata | Goal, Goal IDs, Scope, Docs Impact, Supersedes/Superseded-By |
| `## Goal Alignment` | Maps each Goal ID to specific work items |
| `## Boundary Impact` | What crate/module boundaries change |
| `## Existing Boundary Recheck` | Table: `| Area | Decision | Rationale |` (≥2 rows) |
| `## Open Questions` | Table: `| Question | Decision | Owner | Status |` (≥1 row, all resolved) |
| `## Required Skills` | Domain skills to invoke (template below) |
| `## TDD Policy` | Template below |
| `## Code Review Policy` | Template below |
| `## Commit Policy` | Template below |
| `## Implementation Steps` | `- [ ]` checkmarks with **mandatory review gate checkmarks** |

## Required Metadata

```markdown
**Goal:** [one-line goal statement]
**Goal IDs:** GP-XX, GO-XX (from docs/strategy/goals.md)
**Scope:** [what's in/out]
**Docs Impact:** [canonical docs touched]
**Supersedes:** [prior plans this replaces, or "none"]
**Superseded-By:** [none]
```

## Required Skills

```markdown
## Required Skills
- Rust/backend changes: invoke `rust-skills` BEFORE writing any Rust code and before each review
- Frontend changes: invoke `vercel-react-best-practices` and `vercel-composition-patterns`
  BEFORE writing any frontend code and before each review
- Frontend UI/design: invoke `web-design-guidelines` and `frontend-design` BEFORE writing any UI
  code; use `agent-browser` for screenshot-based design validation after each step
```

## TDD Policy

```markdown
## TDD Policy
For all behavior changes and bug fixes: write a failing test FIRST, then implement.
A step is not complete until:
1. The failing test exists and is committed
2. The implementation makes it pass
3. No existing tests regress

Frontend: e2e tests using `agent-browser` MUST be written per user-facing step.
Frontend UI changes: use `agent-browser` screenshots + `web-design-guidelines` review after each step.
Repeat screenshot + review until clean (recursive).
```

## Code Review Policy

```markdown
## Code Review Policy
After completing each implementation step:
1. Run a thorough code review (backend: `rust-skills`; frontend: vercel skills)
2. Fix ALL findings
3. Run review AGAIN — repeat until no new findings (clean recursive pass)
4. Only after clean pass: commit the step
```

## Commit Policy

```markdown
## Commit Policy
- Commits happen AFTER a clean code review pass, never before
- One commit per implementation step (focused, atomic)
- Do NOT advance to the next step until current step is committed and reviewed clean
```

## Implementation Steps — Standard

```markdown
## Implementation Steps
- [ ] Step 1: ...
- [ ] Step 2: ...
- [ ] Step N: ... (last implementation step)
- [ ] Review Gate: Code review — dispatch `superpowers:code-reviewer` subagent on full branch diff. Invoke domain skills (backend: `rust-skills`; frontend: `vercel-react-best-practices` + `vercel-composition-patterns`). Fix all findings. Re-review until clean pass.
- [ ] Review Gate: Architecture & decomposition review — review all changes for boundary violations, decomposition opportunities, separation of concerns. Re-read `docs/strategy/` and relevant `AGENTS.md` files. Fix easy issues, capture larger items in `docs/features/brainstorms/ideas.md`. Repeat until clean pass.
- [ ] Completion gate — run all checks from AGENTS.md Completion Gate section
```

## Implementation Steps — With Interim Gates (6+ steps)

```markdown
- [ ] Step 1: ...
- [ ] Step 2: ...
- [ ] Step 3: ...
- [ ] Review Gate: Interim code review — review Steps 1-3 changes. Fix findings, re-review until clean.
- [ ] Step 4: ...
- [ ] Step 5: ...
- [ ] Step 6: ...
- [ ] Review Gate: Code review — full branch diff review (see above)
- [ ] Review Gate: Architecture & decomposition review (see above)
- [ ] Completion gate — run all AGENTS.md completion checks
```

Every plan MUST include review gate checkmarks as `- [ ]` items. The plan harness validates their presence. For 6+ step features, add interim review gates every 3-4 steps.

## Plan Splitting Rule

If `master_plan.md` exceeds ~500 lines or covers more than one major area, split into companion files:
- Companion naming: `FEATURE-NAME-TOPIC.md` (e.g., `storage-slots-server.md`)
- `master_plan.md` must include `**See also:**` links to each companion
- Each companion must include a `**Parent plan:**` back-reference to `master_plan.md`
- Each file (main and companion) must independently satisfy required metadata and structure rules

## Pre-Commit Checklist

Before committing the plan, verify:
- [ ] **All 3 review passes dispatched to subagents and converged to 0 findings**
- [ ] **`review_log.md` exists with all 3 passes recorded**
- [ ] `rust-skills` reference if any Rust/backend changes in scope
- [ ] `vercel-react-best-practices` / `vercel-composition-patterns` if any frontend in scope
- [ ] `frontend-design` + `web-design-guidelines` if any UI/design changes in scope
- [ ] `agent-browser` for e2e tests per frontend step (not as afterthought)
- [ ] `superpowers:using-git-worktrees` mentioned in implementation setup
- [ ] `## TDD Policy` section present
- [ ] `## Required Skills` section present
- [ ] `## Code Review Policy` with recursive gate
- [ ] `## Commit Policy`
- [ ] Implementation Steps use `- [ ]` format
- [ ] **Review Gate checkmarks present in Implementation Steps** (code review + architecture review)
- [ ] Interim review gates added for features with 6+ steps
- [ ] `**Review cycles:** N` at bottom (must be ≥ 3 — one per pass minimum)

## Final Action — Commit All Planning Artifacts

1. Archive the refinement doc into the plan directory:

```bash
mv docs/features/needs_refinement/FEATURE-NAME.md docs/features/ready_to_implement/FEATURE-NAME/refinement.md

# Maintain .gitkeep so empty directory stays tracked
[ -z "$(ls -A docs/features/needs_refinement/ 2>/dev/null)" ] && touch docs/features/needs_refinement/.gitkeep
```

2. Commit ALL artifacts:

```bash
git add docs/features/ready_to_implement/FEATURE-NAME/  # includes master_plan.md, refinement.md, review_log.md
git add docs/features/needs_refinement/
git add docs/features/brainstorms/ideas.md  # if modified
git commit -m "plan: FEATURE-NAME — architectural review and implementation plan"
```

This commit is mandatory. `feature-4-implement` creates a worktree from HEAD — uncommitted artifacts won't exist in the worktree.
