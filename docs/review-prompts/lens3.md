# Final diff review, lens 3 (code health): {{FEATURE}}

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

Apply `$code-health-review`.

Read:

- The spec's Goal section and its `Decision:` and `Deferred:` notes.
- The diff's code hunks, the functions and types that enclose them, and
  `clippy.toml`. Read further into a touched code file only where a hunk
  extends existing code.
- Not the other files under `docs/` the diff touches (specs, readings,
  roadmaps), the workflow documents, the telemetry template, or the spec's
  Verification and Performance sections: they are lens 1's inputs.

Check, beyond the skill's smells:

- A function grown toward the `too_many_lines` threshold in `clippy.toml`, or
  given a new `#[allow(clippy::too_many_lines)]`.
- Hand-mirrored enums or tables with no test that keeps them in step.
- Compatibility shims: backward compatibility is not a default goal, so a shim
  for old genomes, configs or decoding that nothing uses and the spec does not
  require is cruft.
- Repeated validation or error paths that clap or serde could declare.
- Workarounds: a workaround the diff adds or keeps carries a
  `WORKAROUND(TNN.FNN): <intended design>` comment at the site and a
  `Deferred:` note in the spec naming the row or debt that pays it. Unmarked is
  P1; new code copying a marked workaround is P2.
- Pre-existing code: report it only where the diff builds on it or a later
  feature listed above will extend it, labeled `pre-existing:`, as P2 with the
  smallest refactor, and say whether it fits this feature's remediation pass or
  belongs in a `Deferred:` note. Other code in a touched file is out of scope.

P1 only for an unmarked workaround.

{{LENS}}

## Output

First the skill's coverage section, one line per check, clean ones included,
naming what was examined. Then one finding per line, most severe first:
`P1: path:line — issue — why it matters — smallest fix` (or `P2:`, `P3:`).
Then a `Could not verify:` list. Nothing else.
