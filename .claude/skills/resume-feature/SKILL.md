---
name: resume-feature
description: Use when returning to a feature after a session break, or when picking up an in-progress feature that was started in a previous conversation.
---

# features:resume-feature

**Announce:** "Using features:resume-feature to pick up where we left off on FEATURE-NAME."

## Overview

Loads an in-progress feature's `master_plan.md`, checks for uncommitted work, summarizes state, re-invokes applicable domain skills, and hands off to continue from the next unchecked step.

## Steps

1. **Read** `docs/features/in_progress/FEATURE-NAME/master_plan.md`
2. **Check for uncommitted work:** Run `git status` and `git diff --stat` to detect any partial/uncommitted changes from the previous session. If dirty state exists, summarize it for the user and ask how to proceed (commit, stash, or discard).
3. **Identify state:**
   - All `- [x]` (completed) steps
   - All `- [ ]` (unchecked) steps
   - The last completed step and the next unchecked step
4. **Summarize for the user:**
   - What has been completed
   - What the next step is
   - Any in-flight state to be aware of (partial work, open questions, etc.)
5. **Re-read policies** from master_plan.md:
   - `## TDD Policy`
   - `## Code Review Policy`
   - `## Commit Policy`
   - `## Required Skills`
   Remind the agent of all applicable policies before starting.
6. **Invoke domain skills** based on the next step's domain:
   - **REQUIRED SUB-SKILL:** Rust/backend → invoke `rust-skills`
   - **REQUIRED SUB-SKILL:** Frontend → invoke `vercel-react-best-practices` + `vercel-composition-patterns`
   - **REQUIRED SUB-SKILL:** UI/design → also invoke `web-design-guidelines` + `frontend-design`
7. **Hand off**: Continue from the first unchecked `- [ ]` step, following all policies in `master_plan.md`.

## Reminder: Policies Always Apply

- TDD is mandatory for behavior changes/bug fixes (test first)
- Per-step code review is recursive (run until clean pass, then commit)
- Checkmarks must be updated in master_plan.md as each step completes
- One commit per step, only after clean review
