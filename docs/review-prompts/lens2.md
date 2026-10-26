# Final diff review, lens 2 (design): {{FEATURE}}

{{RULES}}

## Target

- Feature: {{FEATURE}}
- Worktree: `{{WORKTREE}}`
- Spec: `{{SPEC}}`
- Diff: `git diff {{BASE}} -- . ':!docs'` ({{BASE}} is the merge base with
  `main`) and the untracked files outside `docs/` from `git status --porcelain`
- Later roadmap features whose rows depend directly on this one (derived from
  the roadmap rows; not exhaustive):
{{EXTENDERS}}
- Other later features that overlap the touched code: the "Notes for AI
  Agents" entries that `grep -n '{{FEATURE}}' docs/roadmaps/*.md` finds. Read
  only those entries, not the rows or the rest of the roadmaps.

## Method

Apply `$architecture-review`.

Read:

- The spec's Goal, Inputs and Invariants, and Architecture sections, and the
  crate layout in `AGENTS.md`.
- `docs/architecture.md` in the worktree, the desired state.
- The diff's code hunks and the items that enclose them. Follow a caller or an
  import only to settle a named boundary question, one search each.
- Not the other files under `docs/` the diff touches (specs, readings,
  roadmaps), the workflow documents, the telemetry template, or the spec's
  Verification and Performance sections: they are lens 1's inputs.

Check:

- The boundaries are `docs/architecture.md`'s rules, dependency order,
  prohibitions and owners. Check the coupling the diff introduces or widens
  against them, not against today's code: a leaky abstraction (a domain's
  internals read or built outside it), a second owner, a forbidden import, and
  a design decision in the diff that means the file should change (an
  `architecture:` line proposing the edit).
- Step back: run the skill's step-back challenge on the touched area, using
  `git log` on its paths for earlier features that patched it. A `step-back:`
  finding does not block this feature; the orchestrator reports it.

P1 only for coupling the diff introduces or widens that `AGENTS.md` or
`docs/architecture.md` forbids, or a new import cycle. Report it as P1 even
when the spec's Architecture paragraph explains it or names a row that removes
it: that is the author's argument, and the requester judges it after this
review. Unchanged existing debt is not reported, nor is a debt extended in its
present shape inside the modules `docs/architecture.md` names for it and
marked `WORKAROUND(<debt id>)` by a row ordered before the debt's payer.

{{LENS}}

## Output

First the skill's coverage section, one line per check, clean ones included,
naming what was examined. Then any ungraded `step-back:` and `architecture:`
lines. Then one finding per line, most severe first:
`P1: path:line — issue — why it matters — smallest fix` (or `P2:`, `P3:`).
Then a `Could not verify:` list. Nothing else.
