# Roadmap Execution Goal (Claude)

Claude-adapted companion to [`_goal-prompt-template.md`](_goal-prompt-template.md).
Use this complete prompt for a roadmap-owned Claude Code goal. Replace the values
in the first section, preserve the contract below, and keep every document and
status truthful. This variant assumes the **advisor pattern**: the goal session
runs on Opus as the orchestrator (planning and review), and delegates all
implementation to the `roadmap-implementer` subagent (Sonnet) defined in
`.claude/agents/roadmap-implementer.md`.

## Goal inputs

- Repository: `<local repository path>`
- Roadmap: optional master `docs/roadmap.md`, track roadmaps under
  `docs/roadmaps/tNN-<slug>.md`, flat specs under `docs/specs/roadmap/`
- Requested outcome: `<outcome>`

## Outcome and traversal

Traverse the track roadmaps in dependency order. Execute only unchecked
`TNN.FNN` features whose feature dependencies are checked. Do not invent
ecosystem content or bypass a dependency. Work sequentially by default; parallel
work requires an explicit reason and remains isolated per feature. Continue until
the roadmap reaches `roadmap/complete`, or stop with a concrete blocker and
request user direction. Create a flat spec just in time when a feature is
planned, and keep an unchecked feature unchecked until its complete spec exists.

## Roles and bounded review budget

- Orchestrator (you): Opus, launched at high or extra-high reasoning effort. Own
  the plan, the spec/roadmap documents, both reviews, integration, and the final
  report. Do NOT write feature implementation code yourself.
- Planning: run one bounded planning pass at your highest reasoning effort before
  any code, producing the plan and the just-in-time track/feature spec skeleton.
- Implementer: delegate ALL implementation and remediation to the
  `roadmap-implementer` subagent (Sonnet). Spawn exactly one implementer per
  feature and continue that SAME agent via SendMessage across every pass so it
  retains context — do not spawn a fresh implementer per pass or swap
  implementers. Give it a tight brief: the feature ID, the spec path, and the
  specific change requested.
- Reviews: run the readiness pass (plan and spec, before implementation) and the
  final diff review (after implementation) yourself, in-session. You are
  independent of the implementer because you are a different model — that is the
  review separation that matters. Optionally spawn a fresh Opus reviewer subagent
  for the final diff review when extra independence is warranted.
- Only a P1 finding blocks progress. P2 and P3 findings are advisory and must be
  recorded in the spec's "Deferred Review Findings" without expanding scope.
- Allow one readiness revision and one post-review remediation pass. Route any
  failed required verification back to the same implementer subagent until
  corrected or genuinely blocked; do not silently waive a required check.

## Branch, worktree, and integration contract

Run the entire goal on its own dedicated branch and worktree — never in the main
checkout. Create or resume the integration branch `roadmap/complete` and a
dedicated integration worktree as the durable roadmap execution goal. The
`roadmap-implementer` subagent is configured for `worktree` isolation, so each
feature is implemented in its own nested worktree; you never edit the files it is
editing. When the implementer reports a completed, verified feature, integrate
that change onto the integration branch in dependency order, then confirm the
integration worktree is clean. This prompt explicitly authorizes local branch,
worktree, and commit creation or modification within the repository scope. Do not
push, open or update a pull request, or merge into the user's `main` branch.

## Implementation and verification contract

The `roadmap-implementer` carries the detailed implementation rules ($rust-skills
for every Rust change, TDD for behavior changes and bug fixes, the viability
gate first — `cargo test -p v3-core --test viability` — when defaults, founders,
or tick-loop mechanics change, `make roadmap-check` on document edits, POSIX `sh`
compatibility, and dirty-worktree care). Hold it to them and verify its reported
commands and results rather than taking them on faith. Read the owning track and
flat feature spec before delegating. Make atomic, truthful spec/feature/track/
master status/date/commit updates that describe what actually exists. Ensure the
exact verification commands and results are recorded in the spec's Verification
section. Before closure, ensure `make check` passes on the clean integration
branch. A failed required verification goes back to the same implementer subagent
until corrected or genuinely blocked.

## Closure and final report

Close a feature only when its implementation, focused verification, review,
documentation, dependency, and checkbox gates pass. Update the owning track and
master only in the same truthful state transition; do not mark a rollup complete
early. On the clean integration branch, run the final `make check`. Close the
goal as `roadmap/complete` only when all tracks, features, final success
criteria, and post-review checks are complete.

Report the changed files, exact verification commands and results, review
findings and remediation, blockers, branch/worktree/commit state, integration
state, and whether `roadmap/complete` was reached. Preserve user changes and stop
for user direction on unresolved P1 findings, missing dependencies, an
unauthorized local mutation, or a material scope change.

## Model-routing notes

- Launch the goal on Opus (high or extra-high effort). That session is the
  orchestrator; planning and reviews inherit its effort.
- Implementation auto-routes to Sonnet via the `roadmap-implementer` agent file.
- Subagent reasoning effort is not settable per agent file — only the model is.
  The implementer therefore runs at the subagent default rather than a pinned
  "medium"; this is acceptable for spec-driven coding.
- `isolation: worktree` on the implementer controls worktree isolation, not
  branch naming or base commit. If you require the strict named-feature-branch
  contract, create the feature worktree explicitly at the orchestrator level
  instead of relying on the agent's isolation.
