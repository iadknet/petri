# Design review of documents

{{RULES}}

## Target

- Checkout: `{{WORKTREE}}`
- Documents under review, and their changes since `{{BASE}}`
  (`git diff {{BASE}} -- <document>`):
{{DOCS}}

{{FOCUS}}

## Method

Apply `$plan-design-review` and run all six of its checks.
`docs/architecture.md` is the desired state the documents are checked against.

## Output

The skill's full report: the coverage section with one line per check naming
what was examined, any `step-back:` lines, the assumptions and workaround
tables, the findings and the `Could not verify:` list.
