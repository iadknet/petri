# Final diff review, lens 3 (code health): {{FEATURE}}

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

Apply `$code-health-review`. The project checklist is
`.claude/agents/roadmap-reviewer.md`; ignore its front matter and apply its
general rules, "Lens 3: cruft and code health", "Severity" and "Report".
Two other fresh contexts cover correctness and design; do not report what
belongs to them.

Read the spec's Goal section and its `Decision:` and `Deferred:` notes. Start
from the diff's code hunks. Do not read the other touched files under `docs/`,
the workflow documents, the roadmap rows or the telemetry template: they are
lens 1's inputs.

P1 only for an unmarked workaround.

## Output

First the skill's coverage section, one line per check, clean ones included,
naming what was examined. Then one finding per line, most severe first:
`P1: path:line — issue — why it matters — smallest fix` (or `P2:`, `P3:`).
Then a `Could not verify:` list. Nothing else.
