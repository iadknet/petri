# Spec challenge, round 2 (readiness): {{FEATURE}}

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
row. Read the same inputs round 1 read: `AGENTS.md`,
`docs/roadmaps/README.md`, `docs/specs/roadmap/_feature-template.md`; from
`{{TRACK}}` the {{FEATURE}} row, its dependency rows and the "Notes for AI
Agents" entries naming them; any design note the row cites; each dependency
spec's Goal, Inputs and Invariants and Performance predeclaration;
`docs/architecture.md`; from `docs/workflow.md` "Plan", "Environmental
pressures in the standard baseline" and the "Verification plan" table; and
the code the spec names.

## Method

Apply `$spec-review`. Two jobs:

1. Judge each disposition in the readings file against the spec as it now
   stands. The dispositions and rebuttals are the spec owner's arguments for
   its own spec: accept one only when the spec, the row or the code supports
   it. Check that each fix is the smallest change that resolves its finding:
   a fix that adds more than the finding needed (a requirement, invariant,
   test, option or mechanism beyond it) is an `advisory:` over-fix.
2. Break confidence that the spec is ready to implement. Attack: contradiction
   with the row or the requirement; a source-quoted requirement the spec drops
   or covers on fewer routes than it names; scope widened or narrowed;
   acceptance criteria that cannot be tested; claims about current code the
   code does not support; missing invariants and edge cases; determinism; the
   natural-analog rule; an unrealistic Performance predeclaration; a
   Verification Plan the change does not fit; requirements that steer toward
   needless complexity.

## Output

First one line per disposition: `disposition <ref>: confirmed`, `disposition
<ref>: rejected — <why>` or `disposition <ref>: accepted rebuttal`. Then one
finding per line starting `blocking:` where the skill would grade it P1 or
`advisory:` where it would grade it P2 or P3, citing the spec section and the
evidence, proposing a concrete change, and ending with `[verified]` when you
checked it against the code, the documents or stored output, or `[plausible]`
when it is reasoned but not confirmed. Then a `Could not verify:` list. End
with exactly one line, which follows from the findings: `verdict: ready` when
you report no blocking finding and reject no disposition of a blocking one,
otherwise `verdict: not-ready`.
