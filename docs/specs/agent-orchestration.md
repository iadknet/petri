# Roadmap Execution Orchestration — Fable Orchestrator, Sonnet Executors, Advisor

**Status**: In progress
**Last updated**: 2026-09-03

## Goal

Run roadmap features with the configuration Anthropic currently documents as
best practice: a Fable 5.1 session plans, reviews, and integrates; Sonnet
subagents implement; each Sonnet subagent consults a stronger advisor at
decision points; an independent reviewer subagent checks the final diff. All of
it is built from Claude Code's shipped primitives plus one small project script.

## Non-Goals

- No custom advisor implementation. The advisor tool ships with Claude Code.
- No agent teams and no dynamic workflows for feature execution (see Inputs).
- No change to the roadmap contract, feature IDs, or `roadmap-check` scope.
- No throwaway baseline runs and no A/B campaign. The advisor setting is judged
  from per-feature cost records that accrue during ordinary execution.

## Inputs and Invariants

Research date: 2026-09-03. Claude Code installed: v2.1.259.

**Question answered: is there an official framework?** There is no packaged
"roadmap execution" framework. Anthropic ships the pieces and the guidance:

- Advisor tool (`/advisor`, `advisorModel`, `--advisor`): the main model
  consults a stronger model mid-task; the advisor sees the whole transcript.
  Subagents inherit the configured advisor and apply the pairing check against
  their own model. If the advisor is weaker than the main model it is simply
  not attached to the main model, while subagents that satisfy the pairing still
  use it. Requires the Anthropic API and feature-flag fetching. A Fable main
  accepts only a Fable 5.1 advisor; an Opus main accepts an Opus advisor.
  Source: <https://code.claude.com/docs/en/advisor>.
- Custom subagents: frontmatter supports `model`, `effort`, `isolation:
  worktree`, `permissionMode`, `hooks`, `maxTurns`, `memory`; a `Stop` hook in
  an agent file becomes `SubagentStop` and exit code 2 keeps the subagent
  working with feedback. Completed subagents resume via `SendMessage` with full
  context. Source: <https://code.claude.com/docs/en/sub-agents>,
  <https://code.claude.com/docs/en/hooks>.
- Subagent worktrees branch from the repository's default branch unless
  `worktree.baseRef` is `"head"`, in which case they branch from the current
  worktree's `HEAD`. In hooks, `${CLAUDE_PROJECT_DIR}` stays at the launch
  project root while the `cwd` field in the hook's stdin JSON follows the
  worktree. Source: <https://code.claude.com/docs/en/worktrees>.
- `/goal`: a Haiku evaluator re-checks a completion condition after every turn
  and keeps the session working; it does not change the permission mode. The
  docs point Fable users at it for outcomes larger than one sitting. Source:
  <https://code.claude.com/docs/en/goal>,
  <https://code.claude.com/docs/en/model-config> ("Work with Fable").
- Official orchestrator guidance: "Fable 5.1 as orchestrator with Sonnet 5
  workers" is the primary measured configuration. It wins when work fans out
  across independent pieces or exceeds one context window and loses on one
  dependent chain that fits in one context, where Fable alone at lower effort
  was 22 to 30 percent cheaper at equal accuracy. Source:
  <https://platform.claude.com/docs/en/about-claude/models/optimizing-for-cost-and-intelligence>.
- Official advisor results: Sonnet plus Opus advisor is +2.7 points on
  SWE-bench Multilingual at 11.9 percent lower cost per task. Same-tier
  pairings gain little. Sonnet 5 at low effort with a Fable advisor stopped
  consulting on SWE-bench Pro and scored below Sonnet alone, so executors must
  not run at low effort. Coding executors under-call the advisor without
  system-prompt steering. Sources: the page above,
  <https://claude.com/blog/the-advisor-strategy>,
  <https://platform.claude.com/docs/en/agents-and-tools/tool-use/advisor-tool>.

**Options considered and rejected.**

- Agent teams: experimental, no session resumption for in-process teammates,
  and when enabled any named subagent becomes a teammate, which breaks the
  resume-the-same-implementer contract. Roadmap features are one dependent
  chain each, which the official guidance says not to parallelize. Source:
  <https://code.claude.com/docs/en/agent-teams>.
- Dynamic workflows (`ultracode`, `.claude/workflows/`): built for fan-out of
  dozens to hundreds of agents with no mid-run user input. Feature execution is
  sequential and gated by user direction. Revisit only for a per-file review
  fan-out. Source: <https://code.claude.com/docs/en/workflows>.
- Third-party "fable-advisor" plugins and advisor-subagent workarounds
  (`github.com/dannymac180/fable-advisor`, destilabs blog): they predate or
  reimplement the shipped advisor tool with an extra subagent hop and no
  transcript access. Not adopted.
- Reddit thread `r/ClaudeAI/comments/1w5xqxd` (the trigger for this work) could
  not be fetched from this environment; the plan is grounded in the official
  sources above rather than the thread.

**Existing state in this repository.**

- `.claude/agents/roadmap-implementer.md`: Sonnet, `isolation: worktree`, no
  `Agent` tool, persistent across passes via `SendMessage`. Keep, but its
  worktree currently branches from `main`, so a second feature would not see
  the first feature's changes integrated on `roadmap/complete`.
- `docs/roadmaps/_goal-prompt-template-claude.md`: describes an Opus
  orchestrator plus Sonnet implementer and calls it the "advisor pattern". In
  Anthropic's vocabulary that is the orchestrator strategy; the advisor is a
  separate tool. Its note that subagent effort is not settable is stale
  (`effort` frontmatter exists), and its base-branch hedge ("if you require the
  strict named-feature-branch contract") leaves the problem above unresolved.
  Reviews are done in-session by the same model that planned, so there is no
  fresh-context review.
- No `.claude/settings.json`; no `advisorModel` or `worktree.baseRef` anywhere.

**Design decisions.**

- Orchestrator: Fable 5.1 (`/model fable`), effort `high`, running inside the
  `roadmap/complete` integration worktree so implementer worktrees branch from
  it. Owns planning, spec documents, readiness review, integration, final
  report. Writes no feature code.
- Executor: `roadmap-implementer` on Sonnet at `effort: medium`. Never `low`,
  per the SWE-bench Pro result. Reports its advisor consult count, since the
  orchestrator cannot see subagent transcripts.
- Advisor: `advisorModel: "opus"` in project settings. A Fable main rejects an
  Opus advisor, so the orchestrator runs with no advisor overhead while every
  Sonnet implementer gets the officially recommended Sonnet plus Opus pairing.
  Project scope makes the choice repo-owned and visible in review; it also
  means anyone cloning pays Opus advisor usage on Sonnet subagents. Promote to
  `"fable"` only if accumulated per-feature records show the implementer's
  advisor consults are the quality bottleneck.
- Revision 2026-09-03, after the T10.F10 shakeout: the executor is now
  `roadmap-implementer` on Opus 5 at `effort: high`, and `advisorModel` is
  `"fable"` so the executor keeps a cross-tier pairing (Opus plus Opus would
  gain little). The user chose this because the remaining first-slice
  features mostly change simulation behavior or introduce new measures, and
  the T10.F10 run showed Sonnet's weaknesses were discipline and code shape
  rather than reasoning. Cost: Opus is 2.5 times Sonnet per token and a Fable
  advisor is twice an Opus one per consult. Consequence of the global setting:
  the Fable orchestrator now also has the advisor attached and is instructed
  in the template not to consult it. Same evening, the user also moved
  `roadmap-reviewer` to Fable 5.1 at `effort: high`, so the final diff review
  is now a stronger model than the implementer as well as a fresh context;
  its body still forbids consulting the (same-tier) advisor. Cost: a Fable
  review is twice an Opus one per token, on a transcript of roughly 100k
  tokens per feature. Revisit with the cost records after
  T01.F13; the fallback is a second Sonnet implementer definition for harness
  and characterization features.
- Independent review: a new read-only `roadmap-reviewer` subagent on Opus at
  `effort: high` runs the final diff review from a fresh context. It inherits
  the Opus advisor (a same-tier pairing with little measured gain), so its
  body tells it not to consult. The orchestrator still decides P1 versus
  advisory.
- Long-running control: the orchestrator session sets `/goal` with the slice's
  completion condition and runs in auto mode, otherwise every unapproved
  command prompts and the goal cannot progress unattended.
- One deterministic gate: the implementer cannot report done while
  `make roadmap-check` fails in its worktree.
- Per-feature cost record: one line of telemetry per feature in that feature's
  spec Notes. This is state about the feature's execution, not orchestration
  policy, so it stays within the roadmap README's boundary.
- Agent teams stay off (`CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS` unset or `0`).

## Implementation Tasks

- [x] Add project settings at `.claude/settings.json`:
      `"advisorModel": "opus"` and `"worktree": {"baseRef": "head"}` (written
      2026-09-03). Notice confirmation 2026-09-03: on the next Fable session
      the user ran `/advisor opus`, which saved `advisorModel` to user settings
      and printed the notice that Opus is less capable than Fable so the
      advisor will not activate on the main model. Whether the project-scope
      key alone would have produced the startup notice was not observed; the
      template's launch section already covers the `/advisor opus` fallback.
- [x] Update `.claude/agents/roadmap-implementer.md`: add `effort: medium`;
      add an "Advisor" section to the body that says to consult the advisor
      before committing to an implementation approach, when the same test or
      build error recurs twice, and before reporting the feature done, and to
      follow its guidance unless file contents or a failed step contradict it.
      Extend the report-back to include the number of advisor consults and the
      decisive guidance from each.
- [x] Add `.claude/agents/roadmap-reviewer.md`: `model: opus`,
      `effort: high`, `permissionMode: plan`, `tools: Read, Grep, Glob, Bash`.
      Body: review exactly one feature diff against its flat spec and the
      roadmap contract; classify findings P1/P2/P3 using the template's
      definitions; check that every command and result recorded in the spec's
      Verification section is consistent with the diff and flag any that is
      implausible; do not run tests or builds and do not consult the advisor;
      report and stop. Never edits.
- [x] Add `scripts/implementer-gate` (POSIX `sh`, no `jq`): read the hook
      JSON from stdin, extract the `cwd` value with `sed`, resolve the worktree
      root with `git -C "$cwd" rev-parse --show-toplevel`, and if any file under
      `docs/roadmap.md`, `docs/roadmaps/`, or `docs/specs/roadmap/` differs from
      the merge base with `roadmap/complete` (override with
      `PETRI_GATE_BASE_REF`; falls back to `HEAD` when the branch is absent),
      run `scripts/roadmap-check.mjs --root <worktree>` there, which is what
      `make roadmap-check` invokes. On failure print the checker output and
      exit 2 so the implementer keeps working. Wired in the implementer's
      frontmatter as a `Stop` hook with command
      `${CLAUDE_PROJECT_DIR}/scripts/implementer-gate`.
      `scripts/implementer-gate.test.mjs` creates a temporary git repository
      with a base commit on `roadmap/complete` and a valid roadmap root, then
      covers: no roadmap change (0), uncommitted breakage (2) and restore (0),
      committed breakage on a feature branch (2), `cwd` in a subdirectory (2),
      and missing or unusable `cwd` (0). Runs via `make implementer-gate-test`,
      which `make policy-check` includes.
- [x] Rewrite `docs/roadmaps/_goal-prompt-template-claude.md`:
  - Rename the pattern to "orchestrator strategy with advisor" and describe the
    three roles (Fable orchestrator, Sonnet implementer with Opus advisor, Opus
    reviewer).
  - Launch section: create or enter the `roadmap/complete` integration
    worktree, start `claude --model fable` there in auto mode, `/effort high`,
    then `/goal` with a condition of the form "every feature in <list> is
    checked on `roadmap/complete` and `make check` passed on that branch in
    this conversation, or a concrete blocker has been reported; stop after N
    turns".
  - Reviews: readiness review stays with the orchestrator; final diff review
    goes to `roadmap-reviewer` with the worktree path, feature ID, and spec
    path; the orchestrator adjudicates findings.
  - Remove the stale effort note, the Opus-only routing notes, and the
    base-branch hedge; state that `worktree.baseRef` is `head` and that agent
    teams must stay disabled.
  - Add a "Per-feature cost record" line: after each feature closes, the
    orchestrator records `/usage` totals and the implementer's self-reported
    advisor consult count into that feature's spec Notes for AI Agents.
- [x] Link this spec from `docs/README.md`.
- [x] Run T10.F10 through the finished machinery as ordinary roadmap work under
      the rewritten goal prompt. This is the shakeout: record in this spec's
      Notes whether the orchestrator wrote no feature code, the implementer
      consulted the advisor and survived a `SendMessage` remediation pass, the
      gate hook fired on a roadmap document edit, and the reviewer returned
      findings from a fresh context. Record that feature's cost record in its
      own spec. The feature's closure gates are governed by its spec and the
      goal prompt, not by this document. Done 2026-09-03; see "Shakeout
      results" in Notes.

## Verification

- [x] `make roadmap-check` passes (this spec lives outside its scanned
      directories; the template edit must not break it). 2026-09-03:
      `roadmap-check: validation passed`.
- [x] `make quality-check` passes (ShellCheck on `scripts/implementer-gate`).
      2026-09-03: `Repository quality validation passed.`
- [x] `make roadmap-check-test` and `make implementer-gate-test` pass.
      2026-09-03: gate suite 5 tests, 5 pass, 0 fail.
- [x] `make check` passes before this spec is closed. 2026-09-03: exit 0 with
      the machinery in place, after `npm ci` in `frontend/` for this fresh
      worktree. Rerun at closure after task 7: 2026-09-03, exit 0 on
      `roadmap/complete` at `9dbf3fed` with T10.F10 integrated (1008 core
      unit tests, 7 bench tests, Clippy, frontend, dependency audit).
- [x] During T10.F10, `git merge-base roadmap/complete <implementer-worktree-HEAD>`
      equals the `roadmap/complete` tip at spawn time, proving the implementer
      branched from the integration branch. 2026-09-03: both were
      `47ede8ed`, checked from the orchestrator session while the implementer
      was running.
- [ ] Observed by the user in the agent panel during T10.F10: the session shows
      the advisor notices from the first task, and the implementer's transcript
      shows an `Advising` line with the Opus model name at least once. Partial
      2026-09-03: the `/advisor opus` output confirmed the not-attached notice
      for the main model; the implementer self-reported four consults with
      substantive Opus guidance, but the user has not yet confirmed seeing the
      `Advising` line in the agent panel.
- [ ] Benchmark report stored at `docs/progress/features/<id>.json`:
      Not applicable: process change, no simulation cost.

## Performance and Goal Impact

Not applicable: no simulation code changes. The per-feature cost records this
work introduces track orchestration spend, not creature-tick cost.

## Success Criteria

- [x] T10.F10 was executed through the Fable orchestrator, Sonnet implementer,
      and Opus reviewer, and this spec's Notes record whether each role engaged
      as designed, independent of whether the feature closed. 2026-09-03: see
      "Shakeout results" in Notes; the feature closed.
- [x] The first executed feature's spec carries a cost record naming usage
      totals and the implementer's reported advisor consults. 2026-09-03: the
      T10.F10 spec records advisor consults, reviewer finding counts, subagent
      token usage, and the session `/usage` totals the user pasted at closure.
- [x] The Claude goal-prompt template no longer describes the design as the
      "advisor pattern" and contains no stale routing or base-branch notes.

## Notes for AI Agents

- `advisorModel` is one global setting. Choosing `opus` deliberately uses the
  pairing check to give the advisor to Sonnet subagents only. Setting `fable`
  would also make the Fable orchestrator consult a second Fable on every hard
  decision, which the official measurements say buys almost nothing for a
  frontier executor.
- The `/goal` evaluator judges only what appears in the transcript, so the
  orchestrator must surface `make check` output and the checked roadmap rows in
  its own messages, not only inside subagent results.
- The user's settings pin `model: sonnet` and `effortLevel: medium`; the
  Fable orchestrator is selected per session, not by changing those defaults.
- Fable usage may bill to usage credits depending on plan; consent was already
  given in this environment.
- No solo baseline run is planned. T10.F10 is one dependent chain that fits in a
  single context window, the shape the official guidance says orchestration does
  not pay for, so a paired measurement there would be uninformative. The cost
  case is judged from wider features such as T01.F11 onward.
- Unverified assumptions to confirm during the first task: project-scope
  `advisorModel` is honored; plan-mode Bash permits `git diff` for the
  reviewer; feature-flag fetching is not disabled in this environment.
  Outcome 2026-09-03: project-scope honoring was not isolated because the
  user also ran `/advisor opus`; plan-mode Bash did permit `git diff` and
  `git show` for the reviewer; feature flags were evidently fetched, since
  the implementer reported Opus advisor guidance.

**Shakeout results (T10.F10, 2026-09-03).**

- Orchestrator wrote no feature code: yes. Its only edits were the T10.F10
  flat spec, this spec, and the merge onto `roadmap/complete`.
- Implementer consulted the advisor and survived `SendMessage` continuation:
  yes. Four self-reported consults; the first caught a gate horizon that
  was 2.5 times over the test-time budget before any artifact existed, the
  second caught an unreachable comparison level and unevidenced checked
  boxes, the third caught a report `git_revision` naming a commit without the
  harness. The same agent was resumed twice: once after an API spend-limit
  interruption mid-implementation, once for the post-review remediation
  pass. Context survived both.
- Gate hook fired on roadmap edits: yes. The implementer's transcript
  references `implementer-gate` seven times across its three runs, and no
  stop was allowed while `make roadmap-check` failed.
- Reviewer returned findings from a fresh context: yes. 3 P1 (all
  untruthful statements in the spec's Performance section), 5 P2, 7 P3, with
  file and line for each. The orchestrator adjudicated the P1 and P2 findings
  into one remediation pass and deferred the P3 findings into the feature
  spec's Notes.
- Friction observed: `/goal` and `/usage` are user commands, so the
  orchestrator asked the user to run them; the auto-mode classifier blocked
  one Bash call that combined read-only verification with `git merge`, and
  the merge succeeded when issued alone; a Sonnet monthly spend limit
  terminated the implementer once and the resume worked without changes.
- Integration: this session ran in the worktree that originally held the
  machinery branch and created `roadmap/complete` there with `git switch -c`,
  so no session restart was needed and `worktree.baseRef: head` still made
  the implementer branch from the integration tip. The feature branch was
  fast-forwarded onto `roadmap/complete`.
