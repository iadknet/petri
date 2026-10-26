# Spec challenge, round 3 (confirmation): {{FEATURE}}

{{RULES}}

## Target

- Feature: {{FEATURE}}, row in `{{TRACK}}`
- Worktree: `{{WORKTREE}}`
- Spec under review: `{{SPEC}}`
- Readings file: `{{READINGS}}`, which holds the earlier rounds' findings and
  the spec owner's dispositions and rebuttals
- Earlier rounds' reports:
{{PRIORS}}

## Inputs

The original requirement is the goal "Roadmap feature {{FEATURE}} is complete
on main" under `docs/workflow.md`'s per-feature contract, and the feature's
row in `{{TRACK}}`. Read the row, the spec, the readings file, the earlier
reports and `docs/architecture.md`, and whatever else a fix or a rebuttal
cites.

## Method

Apply `$spec-review`. Confirm the fixes made since the last round and judge
each rebuttal: the dispositions and rebuttals are the spec owner's arguments
for its own spec, so accept one only when the spec, the row or the code
supports it. Report only new blocking findings beyond those.

## Output

First one line per disposition: `disposition <ref>: confirmed`, `disposition
<ref>: rejected — <why>` or `disposition <ref>: accepted rebuttal`. Then any
new finding the skill would grade P1, one per line starting `blocking:`,
citing the spec section and the evidence, proposing a concrete change, and
ending with `[verified]` when you checked it against the code, the documents
or stored output, or `[plausible]` when it is reasoned but not confirmed. Then
a `Could not verify:` list. End with exactly one line, which follows from the
findings: `verdict: ready` when you report no blocking finding and reject no
disposition of a blocking one, otherwise `verdict: not-ready`.
