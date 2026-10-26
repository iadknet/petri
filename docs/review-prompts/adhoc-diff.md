# Diff review with `${{SKILL}}`

{{RULES}}

## Target

- Checkout: `{{WORKTREE}}`
- Diff: `git diff {{BASE}}` and the untracked files from
  `git status --porcelain`

{{FOCUS}}

## Method

Apply `${{SKILL}}`. `AGENTS.md` and `docs/architecture.md` hold the project's
rules and desired state.

## Output

The report the skill defines, its coverage section first when it has one.
