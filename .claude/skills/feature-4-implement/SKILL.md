---
name: feature-4-implement
description: Use when a feature has a validated master_plan.md in ready_to_implement/ and you are ready to begin coding.
---

# feature-4-implement

**Announce:** "Using feature-4-implement to begin implementing FEATURE-NAME."

## Overview

Validates the plan, moves feature to `in_progress/`, sets up worktree, then executes each `- [ ]` step in `master_plan.md` with mandatory review gates. See `implementation-procedures.md` in this skill directory for bash procedures and parallel dispatch instructions.

## Pre-Implementation Checklist

1. Planning artifacts are committed (verify with `git status`)
2. `scripts/check-plan-harness.sh --mode strict` passes against `master_plan.md`
3. `master_plan.md` has `## Implementation Steps` with `- [ ]` checkmarks
4. Review gate checkmarks exist (at least Code review + Architecture). If missing, add them per `feature-3-plan`.

If any check fails, resolve it before proceeding.

## Setup

1. **Move to in_progress/** on main BEFORE creating worktree (see `implementation-procedures.md` for bash commands). This avoids merge conflicts.
2. **REQUIRED SUB-SKILL:** Create worktree via `superpowers:using-git-worktrees`.

## Per-Step Workflow

For each `- [ ]` item in `master_plan.md`:

### Implementation Steps

1. Invoke domain skills declared in `## Required Skills` of `master_plan.md` BEFORE writing code
2. Write code following TDD: failing test FIRST for all behavior changes and bug fixes
3. For UI changes: screenshot with `agent-browser`, review against `web-design-guidelines` + `frontend-design`, fix and repeat until clean

### Review Gate Steps

When you reach a `- [ ] Review Gate:` checkmark, dispatch the appropriate subagent. An inline comment ("looks clean") is NOT a review.

- **Code review gates:** Dispatch `superpowers:code-reviewer` subagent. Invoke domain skills. Fix ALL findings. Re-dispatch until 0 findings.
- **Architecture gates:** Review changes for boundary violations, decomposition, separation of concerns. Re-read `docs/strategy/` and relevant `AGENTS.md`. Fix issues, capture larger items in `docs/features/brainstorms/ideas.md`. Repeat until clean.

### Re-Verification Rule (Critical)

**If a review gate introduces ANY code changes**, all previously-passed verification steps (tests, clippy, fmt, harnesses) MUST be re-run before checking off the gate. Do not assume a "small fix" from review is safe. This is the most common cause of broken builds at merge time.

### After Each Step

1. Check off the item: `- [ ]` to `- [x]`
2. Commit the step's work together with the checkmark update

## Parallel Dispatch

If `## Implementation Steps` contains `[parallel]`-tagged steps with no shared state, see `implementation-procedures.md` for dispatch instructions.

## Finishing

**Do NOT invoke `feature-6-complete` until ALL checkmarks in `master_plan.md` are `- [x]` — including ALL Review Gate checkmarks.** If any are unchecked: STOP and complete them.

## Begin

Start from the first unchecked `- [ ]` step in `master_plan.md`.
