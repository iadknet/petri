# Codex adapter for the roadmap execution workflow

Not a parallel workflow. The live contract is
[`docs/workflow.md`](workflow.md); this file records only Codex's model, tool,
and worktree substitutions for it. Read the shared Plan, Implement, Review, and
Close contract there first.

## Codex adapter

Use the shared Plan, Implement, Review, and Close contract above with these
substitutions. This adapter uses native subagents and Git; it adds no runner,
plugin, separate roadmap, or global model settings.

Select `gpt-6-astra` with `low` reasoning effort when launching any feature.
Verify the active session model and effort; this workflow does not set them.
The implementer uses `low` reasoning effort ("Astra light").

| Role | Model | Effort | Responsibility |
| --- | --- | --- | --- |
| Orchestrator | `gpt-6-astra` | `low` | Delegate, verify, integrate |
| Spec owner and advisor | `gpt-6-astra` | `high` | Write spec, review readiness, advise implementation, resolve escalations; same agent throughout |
| Implementer | `gpt-6-astra` | `low` | All feature code and remediation; same agent across passes |
| Reviewer | `gpt-6-astra` | `medium` | Fresh-context final review |

The reviewer uses a fresh Astra context at `medium` effort for independent
review. The spec owner also provides implementation advice in the same
persistent context; do not spawn a separate advisor. Advice is read-only with
respect to feature code. Native subagents inherit the session's sandbox;
this adapter does not claim a separate permission boundary.

### Planning and spec ownership

Every Codex roadmap feature uses the roles above. The separate spec owner
handles the shared Plan step; required checks, review severity rules, and
integration conditions still follow the shared contract.

After worktree setup, delegate the shared Plan step to `roadmap_spec_owner`
with `model: gpt-6-astra`, `reasoning_effort: high`, and `fork_turns: none`.
Give it the original user requirements, absolute repository/worktree paths,
feature ID, and instructions to read `AGENTS.md`, this workflow, the roadmap
contract and template, owning track, dependency specs, and relevant code
directly. It uses the research-first-planning, spec-writing, and spec-review
skills to write the flat spec, perform one readiness review, and allow one
revision. Its self-review is not independent validation. It handles the
shared Plan status updates and runs `make roadmap-check`; the orchestrator
verifies the result and commits the plan before implementation starts.

Keep the spec owner available through follow-up messages for contradictions
or mistaken assumptions discovered during implementation. Route proposed
changes to requirements or acceptance criteria, conflicting technical advice,
requests for verification exceptions, and integration conflicts that change
behavior to it before proceeding with the dependent work. It resolves these
against the original feature contract and records necessary spec revisions;
it cannot expand user scope or waive required checks. Unresolvable decisions
follow the existing blocker rule. All code remediation remains with the same
implementer. Serialize document edits between agents.

The spec owner's advice must challenge mistaken assumptions against the
original requirements, not defend the spec merely because it wrote it.
Spec revisions are explicit document changes, serialized with the
implementer's edits and checked with `make roadmap-check` before dependent
implementation proceeds. The combined role's self-review and advice are not
independent validation. The fresh final reviewer checks the original roadmap
intent as well as the spec and diff; do not reuse the spec owner for that review.

### Launch and goal prompt

1. Ask **"give me the Codex goal prompt for the next roadmap feature."** Apply
   the shared next-feature rule using the current roadmap, not its dated example.
2. Start a new **Local** Codex task in the main checkout on clean `main`.
   Select **Astra**, effort **low**.
   The task stays rooted there while all
   feature edits and checks use the feature worktree's absolute path.
3. Paste the substituted prompt below. The requested goal is explicit; use
   Codex's native goal tool if exposed. Do not assume Claude's `/goal` syntax,
   transcript evaluator, or 80-turn limit exists in Codex. Do not invent a
   token budget. Use the native tool's rules for goal status and blockers.

```text
Set a goal: roadmap feature <TNN.FNN> is complete on main. Read docs/workflow.md and docs/workflow-codex.md, and follow the shared per-feature contract with the adapter's substitutions. Use Astra (gpt-6-astra, low) as orchestrator. Delegate spec writing, readiness review, and implementation advice to one separate persistent Astra (gpt-6-astra, high) spec owner and advisor. Return to that same agent at the workflow's advisor checkpoints and for requirement corrections, conflicting technical advice, verification exceptions, and integration decisions that change behavior; do not spawn a separate advisor. Use one persistent Astra (gpt-6-astra, low) subagent for all implementation and remediation and a fresh Astra (gpt-6-astra, medium) subagent for final review. Follow the adapter's waiting and context discipline: prefer 25-minute event-driven waits within tool and higher-priority session limits, do not duplicate healthy workers' work, and never interrupt or replace a worker solely because a wait timed out. Start in the main checkout on clean main. I authorize creating .worktrees/<tnn-fnn> on codex/<tnn-fnn>, local planning and implementation commits, rebasing codex/<tnn-fnn> onto main and rerunning the required checks when main moves, fast-forwarding main after all required checks pass, and removing that completed worktree and branch. Do not push or mutate remotes. Done means the feature row is checked and its spec is Complete on main; make check exited 0 for the final feature content and its tested commit (the rebased one when a rebase was needed) is now main; the feature worktree and branch are removed; and main is clean. Show the evidence in this task and record the workflow's required model/effort settings, advisor consultations, review findings, remediation passes, requirement corrections, user interventions, and total usage when available. If a concrete blocker prevents completion, record it in the spec when one exists, report it, and preserve the worktree; never report the goal complete with required work remaining.
```

### Start and worktree

Replace Claude's Start model check with `gpt-6-astra`. Verify model and effort
from available session metadata, not from an agent's self-description. Verify
that native subagent tools expose explicit model and effort selection before
starting; if unavailable, report the limitation instead of silently using a
different model. Resolve the main checkout from the first entry in
`git worktree list --porcelain` and compare it with the current root. Preserve
the clean-main and feature-dependency checks in Start.

Use `$repo-local-worktrees`. Verify `/.worktrees/` is ignored, the destination
is unused, and the branch does not exist, then run from the main checkout:

```sh
git worktree add -b codex/<tnn-fnn> .worktrees/<tnn-fnn> main
```

Record main's starting commit and the absolute worktree path; verify the new
worktree's branch and HEAD. Apply the shared PATH setup and run `npm ci` in
the worktree's `frontend/`. Use an explicit working directory for every command
and absolute edit paths. Subagents share the filesystem and do not acquire an
isolated checkout merely by being spawned. No `EnterWorktree`, `ExitWorktree`,
app Handoff, or separate user-owned task is needed.

### Delegation and advice

The orchestrator spawns `roadmap_implementer` with `model: gpt-6-astra`,
`reasoning_effort: low`, and `fork_turns: none`. Supply a self-contained brief:
absolute repository and worktree paths, feature ID, spec path, assigned role,
and instructions to read `AGENTS.md`, `docs/workflow.md`, and this adapter. A full-history
fork cannot accept model overrides in the current desktop tools.

Keep that agent for every pass: use `followup_task` to resume an idle agent and
`send_message` to steer a running one. Do not replace it to discard context.
The orchestrator can inspect requirements and verification evidence while
the implementer owns code edits; avoid simultaneous writes to the same documents.

The implementer requests advice through the orchestrator at the three existing
advisor checkpoints: before choosing an approach, after the same failure
recurs twice, and before reporting done. The orchestrator returns to the
existing `roadmap_spec_owner` at `high`, using `followup_task` when idle or
`send_message` when running. Supply the implementer's question, current
evidence, and worktree/spec paths; relay each answer back to the implementer.
During consultations the spec owner reads and advises; it does not edit
feature code, run builds or tests, or integrate. Necessary spec revisions
follow the planning rules above. The implementer must receive the relevant
answer before proceeding with the dependent decision; independent inspection
can continue in the meantime. Count these consultations in the existing
advisor count and record decisive guidance; planning/readiness review alone
does not count as an advisor consultation. The implementer spawns no additional agents.

Include this constraint in the spec owner's brief at every consultation:
recommend the smallest change that satisfies the spec, preferring existing code and
dependencies. Do not propose abstractions, configuration, extension points,
or adjacent refactors for hypothetical future needs. Tie each recommendation
to a concrete spec requirement or observed failure, and distinguish correctness
blockers from optional improvements. If the current approach is sufficient,
say so; a consultation need not produce changes.

The advisor's advice is input, not an instruction to implement. The implementer evaluates it
against the spec and code, reports which recommendations it accepts or rejects
and why, and does not implement optional improvements merely because the advisor
suggested them. The orchestrator rejects advice that expands feature scope; required
verification and the shared review severity rules still apply.

The implementer follows all shared implementation rules, including Rust skills, TDD,
property tests, viability first, truthful spec updates, and mutation survivor
triage. Replace the Claude-only `simplify` skill requirement with an explicit
self-review of the feature diff for reuse, simplification, and efficiency,
using the same enum/framework/existing-dependency criteria in the Claude
implementer instructions. Apply fixes and rerun affected verification; report
what changed. Repeat after remediation that adds code.

Claude's hooks are not installed for Codex. After a coherent Rust edit, the implementer
runs `cargo check --workspace --all-targets` explicitly; when the viability
rule applies, run that gate first. Run `make roadmap-check` on document edits
and before reporting done. The orchestrator independently runs `make roadmap-check`
before accepting the implementer's report and `make check` before integration. These
are required workflow checks, not an automatic SubagentStop gate.

For final review, spawn `roadmap_reviewer` using `gpt-6-astra`, `medium`, and
`fork_turns: none`. Provide the worktree/spec paths and feature ID; instruct it
to read the shared Review contract and `.claude/agents/roadmap-reviewer.md`
as a checklist, ignoring its Claude model/tool front matter. It must not edit,
run tests/builds, or consult the spec owner. Do not reuse the spec owner as reviewer. Apply
the existing severity rules and route remediation to the same implementer agent.

### Waiting and context discipline

Apply these rules while the spec owner, implementer, or reviewer is running:

- When there is no concrete independent orchestrator work, call `wait_agent`
  with `timeout_ms: 1500000` (25 minutes), subject to the tool's supported
  range and higher-priority session instructions. Events and user input can
  end the wait early. If session instructions impose a shorter blocking-wait
  or communication limit, use the longest permitted interval instead; for
  example, a 60-second limit means `timeout_ms: 60000`. State that limitation
  once. Do not silently fall back to the 30-second default or build a custom
  polling runner to bypass the limit.
- A timeout alone is not evidence of a stalled worker. With no actionable
  update, wait again. Do not add status probes, reread transcripts or diffs,
  request "still working?" reports, or interrupt, replace, or restart a worker
  solely because time elapsed. Intervene for a concrete error, an explicit
  blocker or advice request, evidence of incorrect work, or user steering.
- Do independent work only when it advances a named workflow obligation.
  Do not duplicate the worker's investigation, implementation, or checks to
  fill waiting time. Required orchestrator verification still runs at the
  acceptance and integration gates after the worker hands off its result.
- Keep briefs and replies scoped to the decision: paths, requirements,
  changed facts, relevant evidence, and the question or result. Reuse the
  persistent agents and their retained context; do not resend unchanged
  documents or full histories. Request updates at handoffs, required advice
  checkpoints, and blockers rather than on a timer. Follow higher-priority
  user-update requirements without launching extra inspections merely to
  manufacture progress to report.

This adopts the behavioral mitigations proposed in the user's linked
[Reddit comment](https://www.reddit.com/r/codex/comments/1wa9c9d/comment/p8gggw9/).
The 25-minute interval is an operational preference, not a verified Codex
prompt-cache lifetime or quota guarantee. Official
[Astra API guidance](https://developers.openai.com/api/docs/guides/latest-model)
documents API cache settings; it does not establish this desktop session's
cache policy or subscription accounting. Savings and the same failure mode
in Petri remain unmeasured.

### Close and integrate in Codex

Before the final commit, record advisor count and reviewer finding counts in
the spec, along with model/effort settings, remediation-pass count,
requirement corrections, and user interventions. Record total task-specific
usage across the orchestrator and subagents only when actually available;
otherwise write `usage unavailable`. Account-wide rate limits are not task
costs. This replaces the post-closure Claude `/usage` request and stays
non-blocking.

Run `make check` once on the final feature code and record that commit as
the tested commit, then complete the shared closure document updates and run
`make check-docs` in the feature worktree. Commit the final content, inspect
any pre-commit changes, and rerun the relevant check if the committed content
differs from what passed. Wait for all agents to finish writing before merging.

From the main checkout, recheck that main is clean, still on `main`, and still
at its recorded starting commit. If it moved, rebase `codex/<tnn-fnn>` onto
`main` and re-verify as in the shared contract, then continue; the recorded
tested commit becomes the rebased, rechecked one. Verify the exact feature
worktree is clean and on the recorded feature branch, then run:

```sh
git merge --ff-only codex/<tnn-fnn>
git worktree remove .worktrees/<tnn-fnn>
git branch -d codex/<tnn-fnn>
```

Perform these sequentially, stopping on failure. Never force removal. Confirm
main's HEAD equals the tested commit, the checked roadmap row and Complete
spec exist there, main is clean, and neither the worktree nor branch remains.
Report command results and verification evidence in the parent task before
marking the goal complete. Preserve the worktree on a blocker.
