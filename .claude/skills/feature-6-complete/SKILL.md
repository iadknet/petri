---
name: feature-6-complete
description: Use when all implementation steps AND review gate checkmarks in an in-progress feature's master_plan.md are checked off. Do not invoke if any checkmark is unchecked.
---

# feature-6-complete

**Announce:** "Using feature-6-complete to run final gates and close out FEATURE-NAME."

## Overview

Three mandatory steps in order. Do not skip or reorder. See `completion-reference.md` in this skill directory for bash templates and the gate command list.

## Step 1: Verify Review Gates Were Completed

Read `master_plan.md` and verify ALL `Review Gate:` checkmarks are `- [x]` (checked off).

**If ANY review gate is unchecked:** STOP. Go back to `feature-4-implement` and complete them. Do not proceed.

**If no review gate checkmarks exist** (legacy plan): you must run them now before proceeding. Dispatch each as a subagent — inline prose review does not count.

1. **Code review:** Dispatch `superpowers:code-reviewer` subagent on full branch diff. Invoke domain skills (`rust-skills` for backend; `vercel-react-best-practices` + `vercel-composition-patterns` for frontend). Fix all findings. Re-dispatch until 0 findings.
2. **Architecture & decomposition review:** Dispatch subagent to review all changes for boundary violations, decomposition opportunities, separation of concerns. Re-read `docs/strategy/` and relevant `AGENTS.md` files. Fix easy issues, capture larger items in `docs/features/brainstorms/ideas.md`. Re-dispatch until 0 findings.
3. Check them off in `master_plan.md` and commit.

**Re-verification rule:** If any review in this step introduced code changes, you MUST re-run the full completion gate (Step 2) after those changes. A review that changes code invalidates any previously-passing gate results.

## Step 2: Completion Gate

Run ALL commands from the AGENTS.md `## Completion Gate` section. The canonical list lives there — always read it fresh rather than relying on a cached copy. See `completion-reference.md` for the current snapshot.

**Structural enforcement:**
- Run each command and capture its output.
- If ANY command fails: fix the issue, then re-run ALL commands from the beginning (not just the failing one).
- Do not proceed until every command produces a passing result in the same run.

**Frontend gate:** If any frontend files were touched, also run `cd frontend && npm run build`.

## Step 3: Finish Branch & Move to Completed

1. **Dispatch** `superpowers:finishing-a-development-branch` to handle merge, PR creation, and worktree cleanup.
2. After the worktree is merged and closed (you are now on main), run the move-to-completed commands from `completion-reference.md`.
