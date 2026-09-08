# Roadmap Execution Workflow

This is the only live workflow for implementing roadmap features. One goal
implements one feature, in its own worktree, and merges it into `main`.
Claude and Codex share the per-feature contract below. Codex uses the
[Codex adapter](#codex-adapter) for model, tool, and worktree differences;
Claude uses the original instructions. Do not mix their session controls.
Both may run concurrently on different eligible features. Claude's agent
definitions, hooks, settings, and goal command remain active and unchanged.

## Roles

Claude:

| Role | Model | Where it is defined |
| --- | --- | --- |
| Orchestrator | Fable 5.1, effort `high` | the session you paste the goal into |
| Implementer | Opus 5, effort `high`, Fable 5.1 advisor | `.claude/agents/roadmap-implementer.md` |
| Reviewer | Fable 5.1, effort `high`, read-only | `.claude/agents/roadmap-reviewer.md` |

`.claude/settings.json` sets `advisorModel: fable` and `worktree.baseRef: head`.
The intended advisor effort is `medium`, but neither Claude Code nor the API
exposes an advisor effort setting, so that intent is recorded here and not
enforced.
`scripts/implementer-gate` is a SubagentStop hook that blocks the implementer
from reporting done while `make roadmap-check` fails in its worktree.
`scripts/implementer-compile-check` is a PostToolUse hook on the implementer's
Edit and Write calls: after a `.rs` edit it runs
`cargo check --workspace --all-targets` and returns the last 30 lines of output
when the build fails or exceeds 900 seconds. It is feedback, not a gate.
`make rust-mutants` runs cargo-mutants (installed through aqua's local
registry) on the feature diff; it is deliberately outside `make check` and the
stop gate, because its output is a survivor list to triage, not a score.

## Run the next feature

In Codex, use the [Codex launch instructions](#codex-adapter) instead.

1. Ask any session in this repository: **"give me the goal command for the next
   roadmap feature."** It applies the next-feature rule below and returns one
   `/goal` command for Claude, or the Codex goal prompt when requested in Codex.
2. Open a new session in the **main checkout** (not a worktree), on a clean,
   current `main`. Select Fable 5.1, effort `high`, and auto mode. Keep agent
   teams disabled (`CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS` unset or `0`).
3. Confirm the startup notice `Advisor Tool (experimental) is on`. If it is
   missing, run `/advisor fable` once (a Fable advisor bills to usage
   credits on plans where Fable usage does). Do not consult the advisor
   yourself; its value is on the Opus implementer.
4. Paste the goal command. Nothing else to paste: the goal tells the
   orchestrator to read this file.

**Next-feature rule.** Take the priority order in the "Notes for AI Agents"
section of `docs/roadmap.md`. The next feature is the first ID in that order
whose row is unchecked in its track roadmap and whose dependencies are all
checked. Compute it every time: the order carries no closure state, and this
file records no current answer.

**Concurrent sessions.** Before launching, inspect `git worktree list` for
existing Claude and Codex feature worktrees. An unchecked feature may already
be in progress; do not assign it twice. If the next feature is already active,
report that fact and identify the next dependency-ready, unclaimed feature
for the user to select rather than silently changing the next-feature rule.
Recheck the target at execution start. Keep Claude work in
`.claude/worktrees/` and Codex work in `.worktrees/`, with their distinct
branch names. Never edit, remove, or take over another session's worktree.
Integration is sequential: if another session advances `main`, the later
session follows the existing stop-and-report rule before any reconciliation.

## Goal command template

This template is for Claude. Codex uses its own template below, with the same
completion conditions. Generating either template does not execute it.

Substitute `<TNN.FNN>` and the lowercase `<tnn-fnn>` worktree name.

```
/goal Roadmap feature <TNN.FNN> is complete on main. Read docs/workflow.md first and follow its per-feature contract exactly: confirm you are Fable 5.1 in the main checkout on a clean main; create the feature worktree with EnterWorktree named <tnn-fnn>; plan and commit the flat spec there; delegate all implementation and remediation to the roadmap-implementer subagent and the final diff review to roadmap-reviewer; run make check in the worktree; ExitWorktree with keep, fast-forward main to the feature branch, then remove the worktree and its branch. Done means all of these are shown in this conversation: the <TNN.FNN> row is checked in its track roadmap on main and its spec is Complete; make check exited 0 at the commit now on main; git worktree list no longer lists the feature worktree; git status on main is clean. If a concrete blocker stops the feature, record it in the spec, report it, and stop. Stop after 80 turns.
```

The `/goal` evaluator reads only this conversation and runs no commands, so
surface command output and the checked roadmap rows in your own messages, not
only inside subagent results. Run a heavyweight check once, redirect its output
to a log file, and read the log; never re-invoke `make check`, a benchmark, or
a mutation run just to filter its output differently.

## Per-feature contract

Normative. The orchestrator reads this at goal start and follows it, applying
the Codex adapter's explicit substitutions when running in Codex.

### Start

Verify before doing anything else, and stop with the reason if any check fails:
you are Fable 5.1; `git rev-parse --show-toplevel` is the main checkout, not a
path under `.claude/worktrees/`; `git status` is clean on `main`; the target
feature is unchecked and every dependency in its track row is checked. If the
session is already inside a worktree, stop and ask the user to relaunch from
the main checkout — a worktree session cannot merge into the main checkout.

Then call `EnterWorktree` with the feature name. The session moves to
`.claude/worktrees/<tnn-fnn>` on branch `worktree-<tnn-fnn>`, branched from
`main`. Run `cd frontend && npm ci` there; a worktree is a fresh checkout.
Prefix `PATH` with `~/.local/share/petri-tools/bin` and
`~/.local/share/aquaproj-aqua/bin` so pre-commit and Node resolve.

### Plan

Read the roadmap contract (`docs/roadmaps/README.md`), the owning track, and any
dependency specs. Create the flat feature spec at
`docs/specs/roadmap/tnn-fnn-<slug>.md` from the template, run one readiness
review yourself, allow one revision, and commit it. Set the spec to
`In Progress` and promote a `Planned` track and a `Planning` master.

### Implement

Delegate **all** implementation and remediation to `roadmap-implementer`. Write
no feature code yourself. Spawn exactly one implementer per feature and continue
that same agent with `SendMessage` across every pass, so it keeps its context;
never spawn a fresh implementer per pass. Give it a tight brief: the feature ID,
the spec path, and the specific change requested. It carries the detailed rules
($rust-skills, TDD, property tests for pure invariants, viability gate first,
`make roadmap-check` on document edits, POSIX `sh`), runs the `simplify` skill
on its own diff before reporting done, and reports its advisor consult count.
Verify its reported commands and results rather than taking them on faith; if
its report does not mention the simplify pass or the mutation run, send it back
before review.

**Mutation survivors.** After the simplify pass and before reporting done, the
implementer runs a fresh `make rust-mutants` (`MUTANTS_ITERATE=0`, the default).
The target diffs the worktree (committed, uncommitted, and untracked) against
its merge base with `main`,
runs `cargo mutants --in-diff` with the caps in `.cargo/mutants.toml`, writes
its output under `~/.local/share/petri-tools/mutants/<worktree>/`, and prints
every survivor: mutants missed by every test and mutants that timed out. The
diff limits where mutations are generated; each viable mutant invokes the
affected package's unfiltered test suite, including integration tests and
doctests. Test filtering and reducing property-test case counts are not part of
this target.
The `mutants` Cargo profile uses optimization level 1 with debug assertions and
overflow checks retained. The wrapper records `fresh` or `incremental` in
`run-mode.txt` beside `mutants.out` and identifies the mode in its output.
The implementer records in the spec's Verification section the summary line, the
output path, and the full survivor list, each survivor resolved as **killed**
(a test added or strengthened, then the target rerun), **equivalent** (one
sentence on why it cannot change observable behavior), or **deferred** (a
deferred finding in "Notes for AI Agents"). Production code is never edited to
kill a mutant, and any `#[mutants::skip]` or `exclude_re` entry carries a
written justification. "No survivors" or "nothing to mutate" are valid records
when that is what the target printed.

While adding or strengthening tests to resolve survivors on unchanged production
content, use `MUTANTS_ITERATE=1 make rust-mutants` for intermediate feedback.
This passes cargo-mutants' `--iterate`: prior caught/unviable results in the
same output directory may be reused, with accumulated entries in
`mutants.out/previously_caught.txt`. Matching is heuristic and does not prove
that coverage survived other changes. Incremental output is never closure
evidence, even if no survivors remain in that pass. After remediation, run
`MUTANTS_ITERATE=0 make rust-mutants` and record its fresh survivor list. Use a
fresh run immediately after production, test-selection, or tool-configuration
changes, or after deleting or weakening tests.

### Review

Delegate the final diff review to `roadmap-reviewer` with the worktree path, the
feature ID, and the spec path. Only a P1 finding blocks progress. P2 and P3 are
advisory and are recorded as deferred review findings in the spec's "Notes for
AI Agents" without expanding scope; a maintainability P2 on code a later feature
will extend is worth one remediation pass, not scope creep. Allow one
post-review remediation pass, routed back to the same implementer. Never
silently waive a required check.

The reviewer audits the mutation survivor record against the diff and, where
readable, against the `missed.txt` and `timeout.txt` in the recorded output
path. A missing survivor list, or a `#[mutants::skip]` or `exclude_re` without
justification, is a waived check and P1. An unresolved survivor is P2. A
survivor "killed" by editing production code rather than a test is a finding at
the severity of the behavior change it made.

Every feature closed after T10.F10 stores a benchmark report and completes the
spec's Performance and Goal Impact section. A severe compute regression without
a predeclared, justified cost is a P1. Never weaken a threshold or edit a stored
baseline to make a feature pass.

From T11.F14 onward, the evolved-neighborhood observation cap is **180 seconds
per goal-profile run, summed across seeds**
(`environment.neighborhood_evolved_wall_clock_ms_total`). This permanent
budget adjustment was explicitly authorized by the user on 2026-09-06 and
supersedes T11.F01's 90-second cap for T11.F14 and future closures. The
founder-neighborhood cap remains 10 seconds of release wall time per profile
run, and the 15-minute total goal-profile investigation threshold remains.
Profile parameters, sample/trial counts, mutation floors, normalized compute
thresholds, and stored baselines are unchanged. Preserve historical readings
and their original acceptance results; an exceeded current cap still requires
resolution before closure.

The goal profile runs **once** per closure. A second goal run to re-check its
`deterministic` block is not required and must not be reinstated as a lost
safeguard: the run costs about eleven minutes, and cross-process
reproducibility is already covered by
`crates/v3-core/tests/reproducibility.rs` inside `make check` (user decision,
2026-09-05, taken to cut per-feature closure cost). A spec's Verification item
for a second goal run is closed with that reason recorded against it, in
either the `Not applicable` or the stored-reports-only form, and is never
left unchecked. The gate profile's
two-run byte-identical check is unaffected: it is seconds-scale and runs
inside `make check` as an ordinary test.

Use `make bench` for measured gate, goal, and sweep reports. Its
`scripts/bench-wait` preflight waits for other `v3-server`, `v3-cli`, and
`v3-core` executables, their `v3_*` unit-test binaries, and `cargo-mutants`
anywhere on the host, including other checkouts. It prints blocking PIDs,
polls every five seconds, and fails without starting the benchmark after
one hour (override with `BENCH_WAIT_TIMEOUT=<seconds>`; `0` checks once).
Process-inspection failure also blocks the benchmark. It never stops another
process; an orphaned server needs deliberate cleanup before retrying.
This is a preflight, not host isolation: do not start competing benchmarks,
servers, builds, or test runs during measurement. Direct CLI invocations bypass
the guard; wrap profiling commands with `scripts/bench-wait <command> ...`.
The ordinary `make check` tests remain unguarded: their gates use deterministic
counters, not host wall-clock timings.

`make rust-mutants` runs the same preflight restricted to `cargo-mutants`
(`BENCH_WAIT_BLOCKERS=mutants`), so a second mutation run never starts while
one is in progress anywhere on the host; a running server or benchmark does
not hold it up. The per-mutant caps in `.cargo/mutants.toml` are deliberately
generous (five times the baseline test time with a two minute floor, eight
times for builds): a mutant that times out is reported as a survivor to
triage, so a tight cap on a loaded host manufactures survivors and a retriage
loop rather than catching anything.

### Close and integrate

In the worktree: run `make check` once on the final feature code and record
that commit as the tested commit. Then set the spec to `Complete`, check the
feature row, update the track and master rollups only if their own criteria are
now satisfied, run `make check-docs` (the closure edits are documentation, and
the full suite was just run on the same code), and commit. Then:

1. `ExitWorktree` with `action: "keep"` — the session returns to the main
   checkout. While inside a worktree, Claude Code blocks every git command
   aimed at the main checkout, so the merge cannot happen before this step.
2. `git merge --ff-only worktree-<tnn-fnn>`. If it fails because `main` moved,
   rebase `worktree-<tnn-fnn>` onto `main` without asking, resolve the
   conflicts, rerun `make check` on the rebased content, record the new tested
   commit, and then fast-forward. Report the rebase and its re-verification.
   Stop and report only when the conflicts cannot be resolved on the feature's
   own terms or the rebased content fails `make check`.
3. `git worktree remove .claude/worktrees/<tnn-fnn>` and
   `git branch -d worktree-<tnn-fnn>`.
4. Show `git status`, `git worktree list`, and the `make check` result in your
   own message so the goal evaluator can see them.

Executing a user-pasted goal authorizes local branch, worktree, and commit
creation, rebasing the feature branch onto `main`, the fast-forward of `main`,
and removal of the completed feature's worktree and branch. Merely reading or
editing this workflow grants no such authorization. It does **not** authorize
pushing, opening or updating a pull request, or any other remote mutation.

### Cost record

After the feature closes, add one line to that feature spec's "Notes for AI
Agents": the `/usage` totals at closure (ask the user; `/usage` is a user
command), the implementer's self-reported advisor consult count, and the
reviewer's finding counts by severity. Telemetry only, never a success
criterion.

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
Set a goal: roadmap feature <TNN.FNN> is complete on main. Read docs/workflow.md and follow its shared per-feature contract and Codex adapter. Use Astra (gpt-6-astra, low) as orchestrator. Delegate spec writing, readiness review, and implementation advice to one separate persistent Astra (gpt-6-astra, high) spec owner and advisor. Return to that same agent at the workflow's advisor checkpoints and for requirement corrections, conflicting technical advice, verification exceptions, and integration decisions that change behavior; do not spawn a separate advisor. Use one persistent Astra (gpt-6-astra, low) subagent for all implementation and remediation and a fresh Astra (gpt-6-astra, medium) subagent for final review. Follow the adapter's waiting and context discipline: prefer 25-minute event-driven waits within tool and higher-priority session limits, do not duplicate healthy workers' work, and never interrupt or replace a worker solely because a wait timed out. Start in the main checkout on clean main. I authorize creating .worktrees/<tnn-fnn> on codex/<tnn-fnn>, local planning and implementation commits, rebasing codex/<tnn-fnn> onto main and rerunning the required checks when main moves, fast-forwarding main after all required checks pass, and removing that completed worktree and branch. Do not push or mutate remotes. Done means the feature row is checked and its spec is Complete on main; make check exited 0 for the final feature content and its tested commit (the rebased one when a rebase was needed) is now main; the feature worktree and branch are removed; and main is clean. Show the evidence in this task and record the workflow's required model/effort settings, advisor consultations, review findings, remediation passes, requirement corrections, user interventions, and total usage when available. If a concrete blocker prevents completion, record it in the spec when one exists, report it, and preserve the worktree; never report the goal complete with required work remaining.
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
and instructions to read `AGENTS.md` and this entire workflow. A full-history
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

Complete the shared closure document updates and run `make check` in the
feature worktree. Commit the final content, inspect any pre-commit changes,
and rerun checks if the committed content differs from what passed. Record
the tested commit hash. Wait for all agents to finish writing before merging.

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

Superseded material is historical and non-executable: the
[2026-09 orchestration design record](archive/agent-orchestration-2026-09.md),
the [archived PRDs](prds/archive/README.md), the
[historical V3 roadmap](archive/v3-program-roadmap.md), and the
[legacy workflow inventory](archive/legacy-workflow-inventory.md).
