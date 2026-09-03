# Roadmap Execution Goal (Claude)

Claude-adapted companion to [`_goal-prompt-template.md`](_goal-prompt-template.md).
Use this complete prompt for a roadmap-owned Claude Code goal. Replace the values
in the first section, preserve the contract below, and keep every document and
status truthful. This variant uses the **orchestrator strategy with advisor**
from Anthropic's model-routing guidance: a Fable 5.1 session orchestrates
(planning, review adjudication, integration), Sonnet `roadmap-implementer`
subagents execute with an Opus advisor attached, and a read-only Opus
`roadmap-reviewer` subagent reviews each final diff from a fresh context. Agent
definitions live in `.claude/agents/`; the advisor and worktree-base settings
live in `.claude/settings.json`; the design rationale is
[`docs/specs/agent-orchestration.md`](../specs/agent-orchestration.md).

## Goal inputs

- Repository: `<local repository path>`
- Roadmap: optional master `docs/roadmap.md`, track roadmaps under
  `docs/roadmaps/tNN-<slug>.md`, flat specs under `docs/specs/roadmap/`
- Requested outcome: `<outcome>`

## Launch

1. Create or enter the `roadmap/complete` integration worktree (see the branch
   contract below) and start the session from inside it. The project setting
   `worktree.baseRef: head` then makes every implementer worktree branch from
   the integration branch, so each feature sees the features integrated before
   it.
2. Start `claude --model fable` in auto mode, then `/effort high`. Auto mode is
   required: `/goal` does not change the permission mode, and an unattended
   goal cannot progress through permission prompts.
3. Confirm the startup notices: `Advisor Tool (experimental) is on`, and the
   notice that the advisor is not attached to the main model. Fable rejects an
   Opus advisor; Sonnet subagents still receive it. If the advisor notice is
   missing, run `/advisor opus` once to save the setting in user settings.
4. Set the goal, for example:
   `/goal every feature in <TNN.FNN list> is checked on roadmap/complete and make check passed on that branch in this conversation, or a concrete blocker has been reported to the user; stop after <N> turns`.
   The evaluator reads only this conversation, so surface `make check` output
   and the checked roadmap rows in your own messages, not only inside subagent
   results.
5. Keep agent teams disabled (`CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS` unset or
   `0`). A named subagent must stay a resumable subagent, not become a teammate.

## Outcome and traversal

Traverse the track roadmaps in dependency order. Execute only unchecked
`TNN.FNN` features whose feature dependencies are checked. Do not invent
ecosystem content or bypass a dependency. Work sequentially by default; parallel
work requires an explicit reason and remains isolated per feature. Continue until
the roadmap reaches `roadmap/complete`, or stop with a concrete blocker and
request user direction. Create a flat spec just in time when a feature is
planned, and keep an unchecked feature unchecked until its complete spec exists.

## Roles and bounded review budget

- Orchestrator (you): Fable 5.1 at high effort. Own the plan, the spec/roadmap
  documents, the readiness review, review adjudication, integration, and the
  final report. Do NOT write feature implementation code yourself.
- Planning: run one bounded planning pass before any code, producing the plan
  and the just-in-time track/feature spec skeleton.
- Implementer: delegate ALL implementation and remediation to the
  `roadmap-implementer` subagent (Sonnet, medium effort, Opus advisor). Spawn
  exactly one implementer per feature and continue that SAME agent via
  SendMessage across every pass so it retains context — do not spawn a fresh
  implementer per pass or swap implementers. Give it a tight brief: the feature
  ID, the spec path, and the specific change requested. Its report includes how
  often it consulted the advisor; keep that number for the cost record.
- Reviewer: delegate the final diff review to the `roadmap-reviewer` subagent
  (Opus, high effort, read-only, fresh context). Give it the implementer
  worktree path, the feature ID, and the spec path. It reports findings; you
  decide which are P1.
- Readiness review (plan and spec, before implementation) stays with you.
- Only a P1 finding blocks progress. P2 and P3 findings are advisory and must be
  recorded as deferred review findings in the spec's "Notes for AI Agents"
  without expanding scope.
- Allow one readiness revision and one post-review remediation pass. Route any
  failed required verification back to the same implementer subagent until
  corrected or genuinely blocked; do not silently waive a required check.

## Branch, worktree, and integration contract

Run the entire goal on its own dedicated branch and worktree — never in the main
checkout. Create or resume the integration branch `roadmap/complete` and a
dedicated integration worktree as the durable roadmap execution goal, and run
this session from inside that worktree. The `roadmap-implementer` subagent is
configured for `worktree` isolation and the project sets `worktree.baseRef` to
`head`, so each feature is implemented in its own nested worktree branched from
the integration branch; you never edit the files it is editing. When the
implementer reports a completed, verified feature, integrate that change onto
the integration branch in dependency order, then confirm the integration
worktree is clean. This prompt explicitly authorizes local branch, worktree, and
commit creation or modification within the repository scope. Do not push, open
or update a pull request, or merge into the user's `main` branch.

## Implementation and verification contract

The `roadmap-implementer` carries the detailed implementation rules ($rust-skills
for every Rust change, TDD for behavior changes and bug fixes, the viability
gate first — `cargo test -p v3-core --test viability` — when defaults, founders,
or tick-loop mechanics change, `make roadmap-check` on document edits, POSIX `sh`
compatibility, and dirty-worktree care). Hold it to them and verify its reported
commands and results rather than taking them on faith. A SubagentStop hook
(`scripts/implementer-gate`) prevents the implementer from reporting done while
roadmap documents in its worktree fail `make roadmap-check`; never work around
it. Read the owning track and flat feature spec before delegating. Make atomic,
truthful spec/feature/track/master status/date/commit updates that describe
what actually exists. Ensure the exact verification commands and results are
recorded in the spec's Verification section.

Once T10.F10 is checked on the integration branch, have the implementer run its
benchmark for each feature, store the report, and complete the spec's
Performance and Goal Impact section. A feature that introduces a diversity or
cognition measure must also wire its indicator into the report. The reviewer
checks that report as part of the final diff review: a severe compute regression
is a P1 finding that blocks the feature unless its spec predeclared and justified
the cost. Never accept a weakened threshold or an edited baseline as remediation.

Before closure, ensure `make check` passes on the clean integration branch. A
failed required verification goes back to the same implementer subagent until
corrected or genuinely blocked.

## Per-feature cost record

After each feature closes, add one line to that feature spec's "Notes for AI
Agents": the `/usage` totals at closure, the implementer's self-reported advisor
consult count, and the reviewer's finding counts by severity. This is telemetry
about the feature's execution, never a success criterion.

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
