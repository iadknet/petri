---
name: capture-idea
description: Use when a new feature idea surfaces during conversation or implementation work and should be recorded for future consideration.
---

# features:capture-idea

**Announce:** "Using features:capture-idea to record this idea."

## Overview

Append a new entry to `docs/features/brainstorms/ideas.md`. No enforcement — intentionally loose.

## Steps

1. Determine the idea text (from user input or what emerged during current work).
2. Append to `docs/features/brainstorms/ideas.md` under the appropriate section (or at the end).
3. If the idea surfaced during another feature's implementation, add context: "(uncovered during FEATURE-NAME implementation)".
4. No schema required — freeform prose is fine.
5. Confirm the append to the user.

## Format

```markdown
### Idea Title

Brief description of the idea, including context if relevant.
(uncovered during FEATURE-NAME implementation, if applicable)
```
