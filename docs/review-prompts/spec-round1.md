# Spec challenge, round 1 (design): {{FEATURE}}

{{RULES}}

## Target

- Feature: {{FEATURE}}, row in `{{TRACK}}`
- Worktree: `{{WORKTREE}}`
- Spec under review: `{{SPEC}}`
- Readings file: `{{READINGS}}`, which holds the spec owner's preparatory
  `$refactor-audit` table

## Inputs

The original requirement is the goal "Roadmap feature {{FEATURE}} is complete
on main" under `docs/workflow.md`'s per-feature contract, and the feature's
row. Check the spec against the requirement, not only against itself: a spec
can agree with itself and still miss the row. Read:

- `AGENTS.md`, the roadmap contract `docs/roadmaps/README.md` and the template
  `docs/specs/roadmap/_feature-template.md`.
- From `{{TRACK}}`: the {{FEATURE}} row, the rows it depends on, and the
  "Notes for AI Agents" entries naming {{FEATURE}} or one of those rows. Any
  design note the row cites, for its source-quoted requirements.
- From each dependency spec: Goal, Inputs and Invariants, and the Performance
  predeclaration.
- `docs/architecture.md`, the desired state.
- From `docs/workflow.md`: "Plan", "Environmental pressures in the standard
  baseline" and the "Verification plan" table under "Benchmark gate".
- The code the spec names, to check its claims about current code.

## Method

Apply `$plan-design-review` and run all six of its checks. Checks 3 and 5 run
against `docs/architecture.md`: report a leaky abstraction, coupling the file
forbids, a design decision that means the file should change, and a missing
edit to it. Check 6 tests the spec owner's audit table in the readings file;
it does not trust it.

## Output

The skill's full report: the coverage section with one line per check naming
what was examined, any `step-back:` lines, the assumptions and workaround
tables, and the `Could not verify:` list. Label each finding `blocking:` where the skill
would grade it P1 and `advisory:` where it would grade it P2 or P3; each cites
the spec section and the evidence, proposes a concrete change, and ends with
`[verified]` when you checked it against the code, the documents or stored
output, or `[plausible]` when it is reasoned but not confirmed. End with
exactly one line, which follows from the findings: `verdict: ready` when you
report no blocking finding, otherwise `verdict: not-ready`.
