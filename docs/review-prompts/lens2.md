# Final diff review, lens 2 (design): {{FEATURE}}

{{RULES}}

## Target

- Feature: {{FEATURE}}
- Worktree: `{{WORKTREE}}`
- Spec: `{{SPEC}}`
- Diff: `git diff {{BASE}} -- . ':!docs'` ({{BASE}} is the merge base with
  `main`) and the untracked files outside `docs/` from `git status --porcelain`
- Later roadmap features that depend on this one, and so are expected to
  extend the touched code (derived from the roadmap rows):
{{EXTENDERS}}

## Method

Apply `$architecture-review`. The project checklist is
`.claude/agents/roadmap-reviewer.md`; ignore its front matter and apply its
general rules, "Lens 2: design and boundaries", "Severity" and "Report".
Two other fresh contexts cover correctness and code health; do not report what
belongs to them.

Read the spec's Goal, Inputs and Invariants, and Architecture sections, and
`docs/architecture.md` in the worktree. Start from the diff's code hunks. Do
not read the other touched files under `docs/`, the workflow documents, the
roadmap rows or the telemetry template: they are lens 1's inputs.

P1 only for coupling the diff introduces or widens that `AGENTS.md` or
`docs/architecture.md` forbids, or a new import cycle, unless the spec's
Architecture paragraph explains it and names the row that removes it.
Unchanged existing debt is not reported.

## Output

First the skill's coverage section, one line per check, clean ones included,
naming what was examined. Then any ungraded `step-back:` and `architecture:`
lines. Then one finding per line, most severe first:
`P1: path:line — issue — why it matters — smallest fix` (or `P2:`, `P3:`).
Then a `Could not verify:` list. Nothing else.
