# Final diff review, lens 1 (correctness): {{FEATURE}}

{{RULES}}

## Target

- Feature: {{FEATURE}}, row in `{{TRACK}}`
- Worktree: `{{WORKTREE}}`
- Spec: `{{SPEC}}`
- Diff: `git diff {{BASE}}` ({{BASE}} is the merge base with `main`) and the
  untracked files from `git status --porcelain`

## Method

Apply `$adversarial-code-review`. The project checklist is
`.claude/agents/roadmap-reviewer.md`; ignore its front matter and apply its
general rules, "Lens 1: correctness and contract", "Severity" and "Report".
Two other fresh contexts cover design and code health; do not report what
belongs to them.

Read from the spec: Goal, Inputs and Invariants, Architecture, Telemetry,
Verification, Verification Plan, and the Performance predeclaration when the
feature is subject to it. Read the "Review" section of `docs/workflow.md`
(from `### Review` to `### Benchmark gate`) and apply its Telemetry section
check, its spec claim spot-check and its Verification Plan check. The
benchmark profiles run after this review; do not audit them.

## Output

One finding per line, most severe first:
`P1: path:line — issue — the input or code path that triggers it, and why it
matters — smallest fix` (or `P2:`, `P3:`).
Then a `Could not verify:` list. Nothing else.
