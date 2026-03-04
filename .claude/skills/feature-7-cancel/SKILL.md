---
name: feature-7-cancel
description: Use when a feature at any lifecycle stage needs to be abandoned, deprioritized, or marked as no longer relevant.
---

# feature-7-cancel

**Announce:** "Using feature-7-cancel to cancel FEATURE-NAME."

## Overview

Works from any stage. Prompts for cancellation reason, adds a note to the file/directory, moves to `docs/features/cancelled/`.

## Steps

1. **Identify FEATURE-NAME and current stage:**
   - `brainstorms/ideas.md` entry
   - `needs_refinement/FEATURE-NAME.md`
   - `ready_to_implement/FEATURE-NAME/`
   - `in_progress/FEATURE-NAME/`

2. **Prompt for cancellation reason** (required).

3. **Add cancellation note:**
   - For a `.md` file: append at the top a YAML-style block or a `## Cancellation Note` section:
     ```markdown
     ## Cancellation Note
     **Status:** Cancelled
     **Reason:** [reason provided]
     **Date:** [today's date]
     ```
   - For a directory: add a `CANCELLED.md` file in the directory root with the same content.

4. **Move to cancelled:**
   - Single file stages (`needs_refinement`): move file to `docs/features/cancelled/FEATURE-NAME.md`
   - Directory stages (`ready_to_implement`, `in_progress`): move directory to `docs/features/cancelled/FEATURE-NAME/`
   - Ideas in `brainstorms/ideas.md`: add a strikethrough or "(cancelled)" annotation inline; no file to move.

5. **Commit** the move with message: `"chore: cancel FEATURE-NAME — [brief reason]"`

## For In-Progress Features

If the feature has an active worktree, close it first:
- **REQUIRED SUB-SKILL:** Use `superpowers:finishing-a-development-branch` with the "abandon" or "close worktree" path, then cancel.
- Or manually: close the worktree, then cancel.
