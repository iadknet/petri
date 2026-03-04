---
name: feature-4-implement
description: Use when a feature has a validated master_plan.md in ready_to_implement/ and you are ready to begin coding in a worktree. Moves feature to in_progress/ on main before creating worktree to avoid merge conflicts.
---

# feature-4-implement

**Announce:** "Using feature-4-implement to begin implementing FEATURE-NAME."

## Overview

Validates the plan (including review gate checkmarks), moves the feature to `in_progress/` on main (before worktree creation to avoid merge conflicts), sets up execution context, dispatches parallel agents where possible, and begins from the first unchecked step.

## Pre-Implementation Checklist

1. **Planning artifacts are committed** on the current branch (verify with `git status` — if `ready_to_implement/FEATURE-NAME/` has uncommitted changes, commit them first)
2. `scripts/check-plan-harness.sh --mode strict` passes against `master_plan.md`
3. `master_plan.md` has an `## Implementation Steps` section with `- [ ]` checkmark items
4. **Review gate checkmarks exist** — verify `master_plan.md` contains at least:
   - `- [ ] Review Gate: Code review` (or similar)
   - `- [ ] Review Gate: Architecture` (or similar)
   If missing, add them before proceeding. See `feature-3-plan` for the required format.

If any check fails, resolve it before proceeding.

## Move to In-Progress (on main, BEFORE worktree)

**IMPORTANT:** This move MUST happen on main before creating the worktree. If you move the directory on the worktree branch, it will cause merge conflicts when merging back to main (main still has the old path).

```bash
mv docs/features/ready_to_implement/FEATURE-NAME/ docs/features/in_progress/FEATURE-NAME/

# Maintain .gitkeep so empty directory stays tracked
[ -z "$(ls -A docs/features/ready_to_implement/ 2>/dev/null)" ] && touch docs/features/ready_to_implement/.gitkeep
git add docs/features/ready_to_implement/ docs/features/in_progress/FEATURE-NAME/
git commit -m "chore: move FEATURE-NAME to in_progress"
```

## Create Worktree

4. **REQUIRED SUB-SKILL:** Worktree created via `superpowers:using-git-worktrees`

The worktree branch will inherit the `in_progress/` location from main, so both branches agree on the directory path. No merge conflicts on completion.

## Per-Step Workflow

For each `- [ ]` item in `master_plan.md`:

1. **Implementation steps** — write code, following TDD and domain skill policies from the plan
2. **Review Gate steps** — these are explicit checkmarks, not optional. When you reach a `Review Gate:` checkmark:
   - **Code review gates:** Dispatch `superpowers:code-reviewer` subagent. Invoke domain skills. Fix ALL findings. Re-review until clean pass. An inline prose comment ("looks clean") is NOT a review.
   - **Architecture & decomposition gates:** Review all changes for boundary violations, decomposition opportunities, separation of concerns. Re-read `docs/strategy/` and relevant `AGENTS.md` files. Fix easy issues, capture larger items in `docs/features/brainstorms/ideas.md`. Repeat until clean pass.
3. **Check off** the item in `master_plan.md` (`- [ ]` → `- [x]`)
4. **Commit** the step's work together with the checkmark update

## Execution Policies

### TDD
For all behavior changes and bug fixes: write the failing test FIRST. Do not implement before the test exists and fails for the right reason.

### Frontend Design Validation
For any step that adds or changes UI:
1. Use `agent-browser` to take screenshots
2. Review against `web-design-guidelines` and `frontend-design` principles
3. Fix any issues; repeat until clean

### Domain Skills
Invoke the skills declared in `## Required Skills` of `master_plan.md` BEFORE writing code in each domain. Do not skip this even for "small" changes.

## Parallel Dispatch

Review `## Implementation Steps` for independence. If 2+ steps have `[parallel]` tag and no shared state or sequential dependencies:
- **REQUIRED SUB-SKILL:** Use `superpowers:dispatching-parallel-agents` to execute them in parallel
- For sequential independent groups: **REQUIRED SUB-SKILL:** Use `superpowers:subagent-driven-development`

## Finishing

**Do NOT invoke `feature-6-complete` or `finishing-a-development-branch` until ALL checkmarks in `master_plan.md` are checked off — including all Review Gate checkmarks.**

If you find yourself about to finish and review gates are still unchecked: STOP. Go back and complete them. They are not optional.

## Begin Execution

Start from the first unchecked `- [ ]` step in `master_plan.md`.
