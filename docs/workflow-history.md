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

On 2026-09-12, the user changed the Codex orchestrator to Sol at `medium` and
all three Astra subagent roles to `xhigh`.

Later on 2026-09-12, the user isolated the two long-running mechanical closure
checks from the feature implementer and orchestrator. A Terra `high` benchmark
specialist now owns the gate and goal baseline runs and their records; a Sol
`medium` mutation specialist owns the fresh mutation gate, survivor triage, and
test-only remediation. They run sequentially and cannot weaken thresholds,
replace baselines, or edit production code. Production remediation remains with
the persistent Astra `xhigh` implementer and returns through fresh review before
the affected final checks run again.

The user then made the equivalent split for Claude. A Sonnet 5 benchmark
specialist owns the gate and goal baseline runs and their records, and a separate
Opus 5 `medium` mutation specialist owns the fresh mutation gate, survivor
triage, and test-only remediation. Production remediation remains with a fresh
Opus 5 `medium` roadmap implementer. The two mechanical specialists use narrow
briefs and run sequentially without competing workloads.

On 2026-09-13, the user moved the Claude orchestrator from Fable 5.1 to Opus 5
at `medium` and added a persistent Fable 5.1 `high` spec owner
(`.claude/agents/roadmap-spec-owner.md`), the Claude equivalent of the Codex
role adopted on 2026-09-06. The observation behind it: most orchestrator turns
are bookkeeping — briefs, verification of reports, commits, status edits, the
merge — and the frontier model was paying for context on every one of them,
while the work that needs it is the Plan step and the requirement decisions
during implementation. Those now live in the spec owner, spawned in the
foreground for the Plan step and resumed with `SendMessage` for escalations, so
the planning inputs (roughly 60–100k tokens of track, dependency, and code
reading in earlier sessions) no longer enter the orchestrator context at all.
The orchestrator's "do not consult the advisor" rule stands; the spec owner is
its Fable channel, so there is one escalation path rather than two. The fresh
Fable `high` reviewer is unchanged and is never the spec owner.

This was possible because `SendMessage` is exposed in the current desktop
client (it was absent on 2026-09-04, which is why the persistent implementer
rule was retired on 2026-09-09). The implementer stays fresh per pass on the
measured cost rationale — first launches carried ~300k tokens against 89–142k
for later ones — and the contract now says so instead of citing the tool. A
fallback to a fresh spec owner per escalation is written into the contract for
builds where the tool is missing again. Two figures from the 2026-09-07 research
temper the expectation: at API rates a Fable→Opus swap on a cache-read-dominated
session saves about 17% because Fable cache reads are priced at 0.025× base, so
the saving is mostly plan-side (Fable bills to credits) plus the removed Plan
context; it is a one-line reversal if the next measurement window does not show
it.

On 2026-09-22, the user moved every Opus 5 role (orchestrator, implementer,
mutation specialist) to Opus 5.5 at the same efforts. The agent files already
select `model: opus`, which resolves to the newest Opus, so only the contract
text changed.

Also on 2026-09-22, the user asked for Codex to challenge Claude specs
iteratively and adversarially before they are finalized, and to perform the
final code review. The Claude spec owner now runs up to three fresh Codex Astra
`xhigh` challenge rounds, answering each blocking finding with an edit or a
rebuttal, and the Fable reviewer is replaced by a fresh read-only Codex Astra
`xhigh` job that uses `.claude/agents/roadmap-reviewer.md` as its checklist.
The reason is a reviewer from a different model family, so the spec owner's
blind spots are not shared by its critic. Codex runs through the
`openai-codex` plugin's companion script as background `task` jobs; its review
slash commands were not used because they review git diffs with a generic
attack surface and cannot be model-invoked. Each round is a fresh thread
because `--resume-last` picks the latest job in the session and carries stale
context. A smoke test on 2026-09-22 confirmed that a job started with `--cwd`
on a `.claude/worktrees/` path is scoped to that worktree and that
`gpt-6-astra` passes through the plugin unchanged. Codex runs keep their single
readiness review, since their spec owner is already Astra.

On 2026-09-24, the user moved every Codex review from Astra `xhigh` to Astra
`high`: the Claude spec challenge rounds, the Claude final review, and the
Codex adapter's fresh-context reviewer. The Codex adapter's spec owner and
implementer stay at `xhigh`; they write and remediate rather than review.

Superseded material is historical and non-executable: the
[2026-09 orchestration design record](archive/agent-orchestration-2026-09.md),
the [archived PRDs](prds/archive/README.md), the
[historical V3 roadmap](archive/v3-program-roadmap.md), and the
[legacy workflow inventory](archive/legacy-workflow-inventory.md).
