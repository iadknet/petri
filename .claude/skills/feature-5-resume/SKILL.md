---
name: feature-5-resume
description: Use when returning to a feature after a session break, or when picking up an in-progress feature that was started in a previous conversation.
---

# feature-5-resume

**Announce:** "Using feature-5-resume to pick up where we left off on FEATURE-NAME."

## Overview

Reloads an in-progress feature's context, checks for uncommitted work, summarizes state, and hands off to continue from the next unchecked step.

## Steps

1. **Read** `docs/features/in_progress/FEATURE-NAME/master_plan.md`
2. **Check for uncommitted work:** Run `git status` and `git diff --stat`. If dirty state exists, summarize it for the user and ask how to proceed (commit, stash, or discard).
3. **Identify state:**
   - All `- [x]` (completed) steps
   - All `- [ ]` (unchecked) steps
   - The last completed step and the next unchecked step
4. **Summarize for the user:**
   - What has been completed
   - What the next step is
   - Any in-flight state to be aware of (partial work, open questions)
5. **Re-read all policy sections** from `master_plan.md` (TDD, Code Review, Commit, Required Skills, etc.). Remind the agent of all applicable policies before starting.
6. **Invoke domain skills** based on the next step's domain, following the same rules as `feature-4-implement` per-step workflow:
   - Rust/backend: `rust-skills`
   - Frontend: `vercel-react-best-practices` + `vercel-composition-patterns`
   - UI/design: also `web-design-guidelines` + `frontend-design`
7. **Hand off**: Continue from the first unchecked `- [ ]` step, following `feature-4-implement` per-step workflow (implementation, review gates, checkmark update, commit).
