# Implementation Procedures

**Parent skill:** feature-4-implement/SKILL.md

## Move to In-Progress (on main, BEFORE worktree)

This move MUST happen on main before creating the worktree. If you move the directory on the worktree branch, it causes merge conflicts when merging back (main still has the old path).

```bash
mv docs/features/ready_to_implement/FEATURE-NAME/ docs/features/in_progress/FEATURE-NAME/

# Maintain .gitkeep so empty directory stays tracked
[ -z "$(ls -A docs/features/ready_to_implement/ 2>/dev/null)" ] && touch docs/features/ready_to_implement/.gitkeep
git add docs/features/ready_to_implement/ docs/features/in_progress/FEATURE-NAME/
git commit -m "chore: move FEATURE-NAME to in_progress"
```

The worktree branch inherits the `in_progress/` location from main, so both branches agree on the directory path.

## Parallel Dispatch

Review `## Implementation Steps` for independence. If 2+ steps have `[parallel]` tag and no shared state or sequential dependencies:

- **REQUIRED SUB-SKILL:** Use `superpowers:dispatching-parallel-agents` for parallel execution
- For sequential independent groups: **REQUIRED SUB-SKILL:** Use `superpowers:subagent-driven-development`
