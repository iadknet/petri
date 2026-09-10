# Workflow history and rationale

Historical and non-executable. The live contract is
[`docs/workflow.md`](workflow.md) and its
[Codex adapter](workflow-codex.md); where this file and those disagree, they
win.

## Why this shape

Codex adaptation research, 2026-09-04; local CLI `0.153.0` and the current
desktop session's exposed tool schemas:

- **Extend the shared workflow with native subagents and Git (selected).**
  The desktop tools expose the exact requested models, explicit model/effort
  overrides with fresh context, and persistent follow-up messaging. Official
  [subagent guidance](https://learn.chatgpt.com/docs/agent-configuration/subagents)
  supports explicit delegation and model selection. The adapter keeps role
  instructions in this file instead of adding duplicated agent configurations.
- **Codex-managed worktrees (credible alternative).** Official
  [worktree guidance](https://learn.chatgpt.com/docs/environments/git-worktrees)
  describes app-owned worktrees and Handoff. Manual Git worktrees fit this
  project's requested main-rooted, merge-and-delete lifecycle and existing
  ignored directory; app-managed worktrees remain useful for other tasks.
- **Custom CLI runner or hook port (not selected).** This would add orchestration
  and maintenance to a repository that already has checks and a single workflow.
  The tradeoff is explicit orchestrator enforcement instead of Claude's stop
  hook. This is not claimed to provide equivalent automatic enforcement.

No cost or quality equivalence with Claude is established. A complete Codex
feature run remains the end-to-end validation: verify model selection, advisor
round trips, retained implementer context, checks, fast-forward, and cleanup.
The workflow edit itself does not run a feature or benchmark the models.

Research date 2026-09-04, Claude Code 2.1.260.

- Opus 5 executor with a Fable 5.1 advisor is the most accurate configuration
  Anthropic reports measuring, and the advisor is consulted at decision points
  rather than every turn.
  [Cost and intelligence](https://platform.claude.com/docs/en/about-claude/models/optimizing-for-cost-and-intelligence),
  [advisor](https://code.claude.com/docs/en/advisor).
- Trial, 2026-09-04 to 2026-09-06, at the user's direction: the implementer
  ran Sonnet 5 with an Opus 5 advisor, a deliberate deviation from the
  configuration the bullet above cites. On 2026-09-06 the user ended the trial
  and set the role table above to the cited configuration: Fable 5.1 `high`
  orchestrator, Opus 5 implementer, Fable 5.1 advisor with `medium` effort
  intended. The advisor tool exposes no effort setting, so the advisor runs at
  its default; the `medium` intent is documentation only. Feature specs closed
  during the trial keep their cost records for comparison.
- The same page says an orchestrator pays off only for fan-out or work beyond
  one context window, and a feature is one dependent chain. The orchestrator
  here is not a cost play: it buys context isolation, an implementer that
  survives remediation passes via `SendMessage`, a fresh-context reviewer, and a
  deterministic gate the implementer cannot talk its way past.
  [Subagents](https://code.claude.com/docs/en/sub-agents),
  [hooks](https://code.claude.com/docs/en/hooks).
- `/goal` re-evaluates a completion condition after every turn with a small
  model that reads only the transcript, which is why the condition names
  observable end states. [Goal](https://code.claude.com/docs/en/goal).
- Worktree creation, entry, isolation enforcement, and removal are Claude Code
  primitives. [Worktrees](https://code.claude.com/docs/en/worktrees).
- The three deterministic quality checks added on 2026-09-04: cargo-mutants is
  the only maintained Rust mutation tool, is absent from aqua's standard
  registry and ships no arm64 macOS binary, so `aqua-registry.yaml` builds it
  with `cargo install --locked` under `aqua-policy.yaml`. A feature diff yields
  tens to a few hundred mutants, so mutation generation is diff-scoped. The
  measured `mutants` build profile accelerates repeated tests; intermediate
  remediation can reuse results, and closure always requires a fresh run.
  Mutation score is a weak signal that agents game, so the check is survivor
  triage. PostToolUse hooks cannot block, so the compile hook reports (exit 2)
  rather than gates. proptest covers pure invariants where example tests only
  pin one case.

Rejected: agent teams (experimental; a named subagent becomes a teammate, which
breaks resuming the same implementer); dynamic workflows (built for large
fan-out with no mid-run user input); running the orchestrator from a custom
agent definition (its body would replace the Claude Code system prompt);
third-party advisor plugins (they predate the shipped advisor tool and lack
transcript access).

## History

On 2026-09-06, the user adopted the T11.F07 role allocation as the canonical
Codex workflow for every roadmap feature: a `medium` orchestrator with a
persistent `xhigh` spec owner. Historical feature specs retain their trial
records. Adoption does not establish measured cost savings or quality
equivalence with other role allocations.

Later that day, the user combined spec ownership and implementation advice
in the same persistent `xhigh` agent. Compared with retaining a separate
advisor, this preserves requirement context and reduces handoffs; the fresh
final reviewer remains separate to challenge shared blind spots. This follows
[OpenAI's guidance](https://openai.com/business/guides-and-resources/a-practical-guide-to-building-ai-agents/)
to justify additional agents against their coordination overhead. Net cost
savings remain unmeasured, since consultations now use `xhigh` rather than `high`.

On 2026-09-07, the user reduced the Codex orchestrator to `low`, the
persistent spec owner and advisor to `high`, and the fresh final reviewer
to `medium`. The implementer remains at `low`; all four roles use Astra.

On 2026-09-09, after measuring 48 Claude sessions with an offline transcript
analyzer, the user cut the context each reader carries. 88% of measured spend
was context re-sent every turn (51% cache read, 37% cache write) against 12%
generation, and the largest repeated payload was the feature spec: 26 specs
averaging 36 KB against a 1.8 KB template, of which the two evidence sections
were 54% of all spec bytes. Three changes followed. This file and
`docs/workflow-codex.md` were split out of `docs/workflow.md`, which had been
read whole about 25 times, so a Claude session no longer loads the Codex
adapter and a Codex session no longer loads the Claude-only rationale. Measured
evidence moved out of specs into `docs/progress/readings/`, leaving the
predeclaration, the verdict, and every user decision in the spec. Briefs now
name the spec sections a pass needs instead of only its path.

Two corrections were folded into that change. The Claude rule requiring one
persistent implementer continued with `SendMessage` was retired: `SendMessage`
is not exposed in the current desktop client, so remediation passes had already
been running on fresh agents, and the contract now states two staged briefs per
feature. The bullets above that cite `SendMessage` as a reason for the
orchestrator, and the rejection of agent teams for breaking implementer
resumption, are the original 2026-09-04 rationale and are retained as written
rather than edited. The Codex adapter is unaffected: it has `followup_task` and
`send_message` and keeps one persistent implementer. Separately, the
orchestrator effort set to `high` on 2026-09-06 (recorded above) was lowered to
`medium`, and the implementer from `high` to `medium`: sessions run at `xhigh`
or `max` were 16% of the sample and 40% of orchestrator spend, and the
implementer accounted for the largest single share of subagent cost. Both are
one-line reversals if the next measurement window does not support them.

The 2026-09-05 spec Verification cap and reviewer spot-check were recorded as
decisions at the time but never reached the contract; only the one-goal-run
rule from that policy landed. Both are written into the contract by this change.

Superseded material is historical and non-executable: the
[2026-09 orchestration design record](archive/agent-orchestration-2026-09.md),
the [archived PRDs](prds/archive/README.md), the
[historical V3 roadmap](archive/v3-program-roadmap.md), and the
[legacy workflow inventory](archive/legacy-workflow-inventory.md).
