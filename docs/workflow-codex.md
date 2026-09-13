# Codex adapter for the roadmap execution workflow

Not a parallel workflow. The live contract is
[`docs/workflow.md`](workflow.md); this file records only Codex's model, tool,
and worktree substitutions for it. Read the shared Plan, Implement, Review, and
Close contract there first.

## Codex adapter

Use the shared Plan, Implement, Review, and Close contract above with these
substitutions. This adapter uses native subagents and Git; it adds no runner,
plugin, separate roadmap, or global model settings.

Select `gpt-5.6-sol` with `medium` reasoning effort when launching any feature.
Verify the active session model and effort; this workflow does not set them.

| Role | Model | Effort | Responsibility |
| --- | --- | --- | --- |
| Orchestrator | `gpt-5.6-sol` | `medium` | Delegate, verify, integrate |
| Spec owner and advisor | `gpt-6-astra` | `xhigh` | Write spec, review readiness, advise implementation, resolve escalations; same agent throughout |
| Implementer | `gpt-6-astra` | `xhigh` | Feature code and production-code remediation; same agent across passes |
| Benchmark specialist | `gpt-5.6-terra` | `high` | Run the gate and goal baseline profiles and record their verdicts |
| Mutation specialist | `gpt-5.6-sol` | `medium` | Run the mutation gate, perform test-only survivor remediation, and record its verdict |
| Reviewer | `gpt-6-astra` | `xhigh` | Fresh-context final review |

The reviewer uses a fresh Astra context at `xhigh` effort for independent
review. The spec owner also provides implementation advice in the same
persistent context; do not spawn a separate advisor. Advice is read-only with
respect to feature code. The benchmark and mutation specialists have narrow,
separate contexts and run sequentially, never concurrently with each other or
with competing builds, tests, servers, or measurements. Native subagents inherit
the session's sandbox; this adapter does not claim a separate permission boundary.

### Planning and spec ownership

Every Codex roadmap feature uses the roles above. The separate spec owner
handles the shared Plan step; required checks, review severity rules, and
integration conditions still follow the shared contract.

After worktree setup, delegate the shared Plan step to `roadmap_spec_owner`
with `model: gpt-6-astra`, `reasoning_effort: xhigh`, and `fork_turns: none`.
Give it the original user requirements, absolute repository/worktree paths,
feature ID, and instructions to read `AGENTS.md`, the roadmap contract and
template, and the sections it needs rather than whole files: from
`docs/workflow.md`, "Plan", "Review", and "Environmental pressures in the
standard baseline"; from this adapter, "Planning and spec ownership". From the
owning track it reads the feature's row, its dependency rows, and the "Notes for
AI Agents" entries naming them; from each dependency spec the Goal, Inputs and
Invariants, and Performance predeclaration, opening a readings file only for a
specific number. It reads relevant code directly. It uses the research-first-planning, spec-writing, and spec-review
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
follow the existing blocker rule. All production-code remediation remains with
the same implementer; the mutation specialist owns only test additions or
strengthening needed for survivor triage. Serialize document edits between agents.

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
   Select **Sol**, effort **medium**.
   The task stays rooted there while all
   feature edits and checks use the feature worktree's absolute path.
3. Paste the substituted prompt below. The requested goal is explicit; use
   Codex's native goal tool if exposed. Do not assume Claude's `/goal` syntax,
   transcript evaluator, or 80-turn limit exists in Codex. Do not invent a
   token budget. Use the native tool's rules for goal status and blockers.

```text
Set a goal: roadmap feature <TNN.FNN> is complete on main. Read docs/workflow.md and docs/workflow-codex.md, and follow the shared per-feature contract with the adapter's substitutions. Use Sol (gpt-5.6-sol, medium) as orchestrator. Delegate spec writing, readiness review, and implementation advice to one separate persistent Astra (gpt-6-astra, xhigh) spec owner and advisor. Return to that same agent at the workflow's advisor checkpoints and for requirement corrections, conflicting technical advice, verification exceptions, and integration decisions that change behavior; do not spawn a separate advisor. Use one persistent Astra (gpt-6-astra, xhigh) subagent for feature implementation and production-code remediation, a separate Terra (gpt-5.6-terra, high) benchmark specialist for the gate and goal baseline runs and their records, a fresh Astra (gpt-6-astra, xhigh) subagent for final review, and a separate Sol (gpt-5.6-sol, medium) mutation specialist for the mutation gate, test-only survivor remediation, and its record. Run the benchmark and mutation specialists sequentially and never alongside competing builds, tests, servers, or measurements. Start in the main checkout on clean main. I authorize creating .worktrees/<tnn-fnn> on codex/<tnn-fnn>, local planning and implementation commits, rebasing codex/<tnn-fnn> onto main and rerunning the required checks when main moves, fast-forwarding main after all required checks pass, and removing that completed worktree and branch. Do not push or mutate remotes. Done means the feature row is checked and its spec is Complete on main; make check exited 0 for the final feature content and its tested commit (the rebased one when a rebase was needed) is now main; the feature worktree and branch are removed; and main is clean. Show the evidence in this task and record the workflow's required model/effort settings, advisor consultations, review findings, remediation passes, requirement corrections, user interventions, and total usage when available. If a concrete blocker prevents completion, record it in the spec when one exists, report it, and preserve the worktree; never report the goal complete with required work remaining.
```

### Start and worktree

Replace Claude's Start model check with `gpt-5.6-sol`. Verify model and effort
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
`reasoning_effort: xhigh`, and `fork_turns: none`. Supply a self-contained brief:
absolute repository and worktree paths, feature ID, spec path, assigned role,
and instructions to read `AGENTS.md` and the sections it needs, not whole files:
from `docs/workflow.md`, "Start", "Implement", and "Environmental pressures in
the standard baseline"; from this adapter, "Start and worktree" and "Delegation
and advice". Name the spec sections too, as the Implement step requires. The
planning, launch, review, and close sections belong to the
orchestrator, not to the implementer. A full-history
fork cannot accept model overrides in the current desktop tools.

Keep that agent for every pass: use `followup_task` to resume an idle agent and
`send_message` to steer a running one. Do not replace it to discard context.
The orchestrator can inspect requirements and verification evidence while
the implementer owns code edits; avoid simultaneous writes to the same documents.

The implementer requests advice through the orchestrator at the three existing
advisor checkpoints: before choosing an approach, after the same failure
recurs twice, and before reporting done. The orchestrator returns to the
existing `roadmap_spec_owner` at `xhigh`, using `followup_task` when idle or
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
property tests, viability first, and truthful spec updates. Replace the
Claude-only `simplify` skill requirement with an explicit
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

### Benchmark specialist

After the implementer has finished benchmark-affecting work and before final
review, spawn `roadmap_benchmark_specialist` with `model: gpt-5.6-terra`,
`reasoning_effort: high`, and `fork_turns: none`. Give it the absolute worktree
and spec paths, feature ID and slug, the exact gate and goal commands required by
the spec, and only the spec's Verification and Performance and Goal Impact
sections. Instruct it to read the shared "Benchmark gate" section and, when applicable,
"Environmental pressures in the standard baseline." Its scope is to run each
required profile once, retain full reports in the main checkout's ignored
`.bench-artifacts/<feature>/`, and store generated summaries and concise readings
in the feature worktree. Record CLI and observed outer-process exit statuses
with their sources, `severe`, threshold verdict, both paths, hashes and byte
counts. `OUT` overrides raw only; `SUMMARY_OUT` overrides the summary. Follow
[`docs/benchmark-artifacts.md`](benchmark-artifacts.md); new series entries name
summaries, and no new full report is committed. It
does not change thresholds or stored baselines and does not remediate code or
unexpected results; it reports them to the orchestrator. The orchestrator routes
a regression or malformed report to the spec owner and persistent implementer as
appropriate. If later production remediation invalidates a report, return to the
same specialist for only the affected final-code measurement before closure;
this is not a second determinism check.

Codex's filesystem sandbox does not permit the `/bin/ps` process inspection used
by `scripts/bench-wait`. Run each measured `make bench` command through
`exec_command` with `sandbox_permissions: "require_escalated"` and a concise
justification that the benchmark preflight requires process inspection. Request
that permission on the first attempt; do not first run the command in the
sandbox, and do not bypass `scripts/bench-wait` when permission is unavailable.

### Final review

For final review, spawn `roadmap_reviewer` using `gpt-6-astra`, `xhigh`, and
`fork_turns: none`. Provide the worktree/spec paths and feature ID; instruct it
to read the shared Review contract and `.claude/agents/roadmap-reviewer.md`
as a checklist, ignoring its Claude model/tool front matter. It must not edit,
run tests/builds, or consult the spec owner. Do not reuse the spec owner as reviewer. Apply
the existing severity rules and route remediation to the same implementer agent.

### Mutation specialist

After final review and any post-review remediation, spawn
`roadmap_mutation_specialist` with `model: gpt-5.6-sol`,
`reasoning_effort: medium`, and `fork_turns: none`. Give it the absolute
worktree and spec paths, feature ID, the final-review result, and only the spec's
Verification and Notes for AI Agents sections. Instruct it to read the shared
"Mutation gate" contract; this replaces the shared contract's fresh implementer
for Codex. Its scope is the fresh gate, survivor triage,
test-only remediation, permitted iterative feedback, and the final Verification
record. It may add or strengthen tests, using `$rust-skills` for Rust changes,
but it must not edit production code, test-filtering or selection configuration,
mutation configuration, thresholds, or exclusions. An equivalent survivor needs
the contract's written reason; a deferred survivor still needs the user's agreement.

When a survivor exposes a production defect or requires any forbidden change,
the mutation specialist stops and returns the evidence to the orchestrator. The
same persistent Astra implementer owns that remediation, including self-review;
the fresh-review requirement applies again before returning to the same mutation
specialist for the required final-code run. The orchestrator, not either
specialist, audits both records and decides whether closure may proceed.

Never run the benchmark and mutation specialists concurrently or while another
agent is running builds, tests, servers, or measurements in the worktree. Keep
their handoffs concise: command, exit status, report or output path, verdict,
and actionable exceptions rather than raw command output.

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
Inspect summary provenance against available local raw files, preserve measured
revision/time separately from converter identity, and confirm no new full
benchmark artifact is staged. The local artifact root remains in main when the
feature worktree is removed.

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
