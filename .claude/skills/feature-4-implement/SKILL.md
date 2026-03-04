---
name: feature-4-implement
description: Use when a feature has a validated master_plan.md in ready_to_implement/ and you are ready to begin coding in a worktree.
---

# feature-4-implement

**Announce:** "Using feature-4-implement to begin implementing FEATURE-NAME."

## Overview

Validates the plan, moves the feature to `in_progress/`, sets up execution context, dispatches parallel agents where possible, and begins from the first unchecked step.

## Pre-Implementation Checklist

1. `scripts/check-plan-harness.sh --mode strict` passes against `master_plan.md`
2. `master_plan.md` has an `## Implementation Steps` section with `- [ ]` checkmark items
3. **REQUIRED SUB-SKILL:** Worktree created via `superpowers:using-git-worktrees`

If any check fails, resolve it before proceeding.

## Move to In-Progress

```bash
mv docs/features/ready_to_implement/FEATURE-NAME/ docs/features/in_progress/FEATURE-NAME/
```

Commit this move on the feature branch.

## Execution Instructions (for implementing agent)

**MANDATORY policies:**

### Checkmarks
Update `master_plan.md` as each step completes: change `- [ ]` to `- [x]`. Commit the checkmark update together with the step's work.

### TDD
For all behavior changes and bug fixes: write the failing test FIRST. Do not implement before the test exists and fails for the right reason.

### Per-Step Code Review (recursive, mandatory)
After each step:
1. Run code review (backend: invoke `rust-skills`; frontend: invoke `vercel-react-best-practices` + `vercel-composition-patterns`)
2. Fix ALL findings
3. Run the review AGAIN
4. Repeat until clean pass (no new findings)
5. Only then commit the step

Do NOT advance to the next step until the current step is committed and reviewed clean.

### Frontend Design Validation
For any step that adds or changes UI:
1. Use `agent-browser` to take screenshots
2. Review against `web-design-guidelines` and `frontend-design` principles
3. Fix any issues
4. Repeat until clean

### Domain Skills
Invoke the skills declared in `## Required Skills` of `master_plan.md` BEFORE writing code in each domain. Do not skip this even for "small" changes.

## Parallel Dispatch

Review `## Implementation Steps` for independence. If 2+ steps have `[parallel]` tag and no shared state or sequential dependencies:
- **REQUIRED SUB-SKILL:** Use `superpowers:dispatching-parallel-agents` to execute them in parallel
- For sequential independent groups: **REQUIRED SUB-SKILL:** Use `superpowers:subagent-driven-development`

## Begin Execution

Start from the first unchecked `- [ ]` step in `master_plan.md`.
