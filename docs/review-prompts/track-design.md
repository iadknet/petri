# Roadmap design review

{{RULES}}

## Target

- Checkout: `{{WORKTREE}}`
- Documents under review, and their changes since `{{BASE}}`
  (`git diff {{BASE}} -- <document>`):
{{DOCS}}

{{INTENT}}

## Method

Apply `$plan-design-review` to the changed roadmap rows and to any design note
they cite, and run all six of its checks. `docs/architecture.md` is the desired
state the rows are checked against: report rows that change the desired
architecture without an edit to that file, and a known debt no row pays.

## Output

The skill's full report: the coverage section with one line per check naming
what was examined, any `step-back:` lines, the assumptions and workaround
tables, the findings and the `Could not verify:` list. When an intent export
is named above, add the Intent section it asks for.
