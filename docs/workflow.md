# Roadmap Execution Workflow

This is the only live workflow for implementing roadmap features. One goal
implements one feature, in its own worktree, and merges it into `main`.
Claude and Codex share the per-feature contract below. Codex uses the
[Codex adapter](workflow-codex.md) for model, tool, and worktree differences;
Claude uses the original instructions. Do not mix their session controls.
Both may run concurrently on different eligible features. Claude's agent
definitions, hooks, settings, and goal command remain active and unchanged.

## Roles

Claude:

| Role | Model | Where it is defined |
| --- | --- | --- |
| Orchestrator | Fable 5.1, effort `medium` | the session you paste the goal into |
| Implementer | Opus 5, effort `medium`, Fable 5.1 advisor | `.claude/agents/roadmap-implementer.md` |
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
when the build fails or exceeds 900 seconds. A passing check suppresses further
fires for `PETRI_COMPILE_CHECK_COOLDOWN` seconds (15); a failure never does. It
is feedback, not a gate: a breaking edit that ends a burst inside the cooldown
is caught by `make check`.
`make rust-mutants` runs cargo-mutants (installed through aqua's local
registry) on the feature diff; it is deliberately outside `make check` and the
stop gate, because its output is a survivor list to triage, not a score.

## Run the next feature

In Codex, use the [Codex launch instructions](workflow-codex.md#launch-and-goal-prompt) instead.

1. Ask any session in this repository: **"give me the goal command for the next
   roadmap feature."** It applies the next-feature rule below and returns one
   `/goal` command for Claude, or the Codex goal prompt when requested in Codex.
2. Open a new session in the **main checkout** (not a worktree), on a clean,
   current `main`. Select Fable 5.1, effort `medium`, and auto mode. Keep agent
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
/goal Roadmap feature <TNN.FNN> is complete on main. Read docs/workflow.md first and follow its per-feature contract exactly: confirm you are Fable 5.1 at effort medium in the main checkout on a clean main; create the feature worktree with EnterWorktree named <tnn-fnn>; plan and commit the flat spec there; delegate all implementation and remediation to the roadmap-implementer subagent and the final diff review to roadmap-reviewer; run make check in the worktree; ExitWorktree with keep, fast-forward main to the feature branch, then remove the worktree and its branch. Done means all of these are shown in this conversation: the <TNN.FNN> row is checked in its track roadmap on main and its spec is Complete; make check exited 0 on the feature code now on main and make check-docs exited 0 at the commit now on main; git worktree list no longer lists the feature worktree; git status on main is clean. If a concrete blocker stops the feature, record it in the spec, report it, and stop. Stop after 80 turns.
```

The `/goal` evaluator reads only this conversation and runs no commands, so
surface command output and the checked roadmap rows in your own messages, not
only inside subagent results. Run a heavyweight check once, redirect its output
to a log file, and read the log; never re-invoke `make check`, a benchmark, or
a mutation run just to filter its output differently.

## Per-feature contract

Normative. The orchestrator reads this at goal start and follows it, applying
the explicit substitutions in [`docs/workflow-codex.md`](workflow-codex.md)
when running in Codex.

### Start

Verify before doing anything else, and stop with the reason if any check fails:
you are Fable 5.1 at effort `medium`; `git rev-parse --show-toplevel` is the
main checkout, not a
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

Read the roadmap contract (`docs/roadmaps/README.md`). From the owning track read
only the feature's own row, the rows of its dependencies, and the entries in the
track's "Notes for AI Agents" that name this feature or one of its dependencies —
not the whole track, and not the whole Notes section, which is the bulk of a
track file. From each dependency spec read the Goal, Inputs and Invariants, and
the Performance predeclaration; open its readings file only when a specific
number is needed. Create the flat feature spec at
`docs/specs/roadmap/tnn-fnn-<slug>.md` from the template, run one readiness
review yourself, allow one revision, and commit it. Set the spec to
`In Progress` and promote a `Planned` track and a `Planning` master.

### Implement

Delegate **all** implementation and remediation to `roadmap-implementer`. Write
no feature code yourself. `SendMessage` is not exposed in the current Claude
desktop client, so an implementer cannot be kept alive across passes: every pass
starts a fresh agent and its brief carries what it needs. (This applies to Claude
only. The Codex adapter has `followup_task` and `send_message` and keeps one
persistent implementer; see `docs/workflow-codex.md`.)

**Brief 1 — build.** Give it the feature ID, the spec path, the change requested,
and the exact spec sections this pass needs — normally Goal, Inputs and
Invariants, Implementation Tasks, and the Verification checklist. Its scope is
implementation, tests (TDD, property tests for pure invariants), the viability
gate first where it applies, and a clean `cargo check --workspace --all-targets`.
It carries the detailed rules ($rust-skills, `make roadmap-check` on document
edits, POSIX `sh`). It reports the changed files, the exact commands it ran with
their results, and its advisor consult count, then stops.

**Brief 2 — self-review, mutants, spec (a fresh agent).** Give it the feature ID,
the spec path, brief 1's report, and the spec sections this pass needs —
Verification, plus the Performance predeclaration when the feature is subject to
it. Its scope is the `simplify` skill on the feature diff, one fresh
`MUTANTS_ITERATE=0 make rust-mutants` with full survivor triage, and the spec's
Verification update. It reports the summary line, the output path, every survivor
with its disposition, and its advisor consult count.

Splitting the work this way is a cost measure, not a correctness one: first
launches were the expensive ones in the sessions this rule came from. If it
proves worse in practice, collapse the two briefs back into one — the part that
cannot be restored is the `SendMessage` persistence, which the client does not
support.

**Name the sections, never just the path.** "Read the spec's Inputs and
Invariants and Implementation Tasks" costs a fraction of "read the spec at
`<path>`", and the difference is paid on every turn of that agent for the rest of
the pass.

**No spillover re-reads.** Read a document once per pass and work from what you
read. When a Bash result is too large and the harness spills it to a
`tool-results/*.txt` file, re-run the command narrowed — `grep`, a line range,
`--stat` — rather than reading the saved file, and never read the same spilled
file twice. Do not re-read a whole spec, track, or this workflow to reconfirm
something already in context, and do not hand a subagent a document you have
already summarized for it; quote the fact into the brief instead.

Verify each agent's reported commands and results rather than taking them on
faith; if brief 2's report does not mention the simplify pass or the mutation
run, send it back before review.

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
Brief 2's implementer records in the spec's Verification section the summary line, the
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
feature ID, the spec path, and the spec sections the review needs: Goal, Inputs
and Invariants, Verification including the survivor list, and Performance and
Goal Impact when the feature is subject to it. The reviewer reads the feature's
own track row and the matching track Notes entries, not the whole track, and the
no-spillover-re-read rule above applies to it as well.

Only a P1 finding blocks progress. P2 and P3 are advisory and are recorded as
deferred review findings in the spec's "Notes for AI Agents" without expanding
scope; a maintainability P2 on code a later feature will extend is worth one
remediation pass, not scope creep. Allow one post-review remediation pass, given
to a fresh implementer with a brief naming the finding, the files, and the spec
sections to update. Never silently waive a required check.

**Verification cap.** A spec's Verification section stays under about 3 KB
excluding the mutation survivor list. Pass-by-pass logs, command transcripts, and
raw test output go to `docs/progress/readings/<id>.md`, linked from the item that
produced them. A verification item names what is checked and where the result
lives; it does not prescribe test design in prose. This is a review expectation,
not a check — nothing in the repository measures it.

**Spec claim spot-check.** The reviewer picks at least three claims from the
spec's Verification and Performance sections — a command result, a stored-report
path, and a specific number — and checks each against the diff or the stored
file. A claim it cannot check is reported as unverified, never assumed true. An
untruthful claim is P1 under the existing spec-truthfulness rule.

(Both were recorded as user decisions on 2026-09-05 alongside the one-goal-run
rule below, but never reached this contract until 2026-09-09.)

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

### Environmental pressures in the standard baseline

From T12.F04 onward, every feature that adds an environmental pressure must
integrate it into all three standard goal environments: Orchards, Canyon,
and Confluence. Update the existing goal-profile recipes/configuration in the
same feature so the ordinary `make bench PROFILE=goal` run exercises the new
pressure once in each environment, three runs total. Retain previously added
pressures for subsequent closures; a separate opt-in sweep or an unchanged
world-set reading does not satisfy this requirement. Keep the short compute
gate unchanged unless its change is explicitly authorized.

The feature spec identifies the applied mechanism, its settings in each world,
and the evidence that it is enabled and can affect creatures there. For
creature-authored pressures, verify the action-to-world path with a focused
fixture and report actual occurrences in the goal runs, including zero; do not
force the behavior or claim it evolved merely because it is enabled. Record
per-environment persistence and pressure observations with effective config
identity in the closure report. Preserve each world's distinguishing habitat,
existing budgets, historical reports, and honest comparison boundaries when
inputs change. Disabled controls remain available for attribution, but do not
replace the three pressure-bearing standard cases. Unresolved incompatibility
with a world is a recorded blocker, not a silent omission or weakened check.

This applies to abiotic, biotic, and creature-authored pressures, including
T02.F01–F05, T04.F03/F04, T05.F01/F02/F06, T06.F01–F05,
T07.F01–F03/F05/F06, and future T12 pressure additions. Observation-only
features, recipe transport, and deferred characterization campaigns do not
invent new pressures to satisfy it. The final reviewer checks this integration
against the original roadmap goal before closure.

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
command), each implementer brief's self-reported advisor consult count and the
number of passes, and the reviewer's finding counts by severity. Telemetry only, never a success
criterion.

## Codex, rationale, and history

Codex model, tool, and worktree substitutions for this contract:
[`docs/workflow-codex.md`](workflow-codex.md).
Why this workflow has this shape, and the dated record of changes to it:
[`docs/workflow-history.md`](workflow-history.md).
