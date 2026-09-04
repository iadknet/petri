# Roadmap Execution Workflow

This is the only live workflow for implementing roadmap features. One goal
implements one feature, in its own worktree, and merges it into `main`.

## Roles

| Role | Model | Where it is defined |
| --- | --- | --- |
| Orchestrator | Fable 5.1, effort `high` | the session you paste the goal into |
| Implementer | Opus 5, effort `high`, Fable advisor | `.claude/agents/roadmap-implementer.md` |
| Reviewer | Fable 5.1, effort `high`, read-only | `.claude/agents/roadmap-reviewer.md` |

`.claude/settings.json` sets `advisorModel: fable` and `worktree.baseRef: head`.
`scripts/implementer-gate` is a SubagentStop hook that blocks the implementer
from reporting done while `make roadmap-check` fails in its worktree.
`scripts/implementer-compile-check` is a PostToolUse hook on the implementer's
Edit and Write calls: after a `.rs` edit it runs
`cargo check --workspace --all-targets` and returns the last 30 lines of output
when the build fails or exceeds 300 seconds. It is feedback, not a gate.
`make rust-mutants` runs cargo-mutants (installed through aqua's local
registry) on the feature diff; it is deliberately outside `make check` and the
stop gate, because its output is a survivor list to triage, not a score.

## Run the next feature

1. Ask any session in this repository: **"give me the goal command for the next
   roadmap feature."** It applies the next-feature rule below and returns one
   `/goal` command.
2. Open a new session in the **main checkout** (not a worktree), on a clean,
   current `main`. Select Fable 5.1, effort `high`, and auto mode. Keep agent
   teams disabled (`CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS` unset or `0`).
3. Confirm the startup notice `Advisor Tool (experimental) is on`. If it is
   missing, run `/advisor fable` once. Do not consult the advisor yourself; a
   same-tier consult buys little. Its value is on the Opus implementer.
4. Paste the goal command. Nothing else to paste: the goal tells the
   orchestrator to read this file.

**Next-feature rule.** Take the first-slice order in the "Notes for AI Agents"
section of `docs/roadmap.md`. The next feature is the first ID in that order
whose row is unchecked in its track roadmap and whose dependencies are all
checked. As of 2026-09-04 that is T01.F11.

## Goal command template

Substitute `<TNN.FNN>` and the lowercase `<tnn-fnn>` worktree name.

```
/goal Roadmap feature <TNN.FNN> is complete on main. Read docs/workflow.md first and follow its per-feature contract exactly: confirm you are Fable 5.1 in the main checkout on a clean main; create the feature worktree with EnterWorktree named <tnn-fnn>; plan and commit the flat spec there; delegate all implementation and remediation to the roadmap-implementer subagent and the final diff review to roadmap-reviewer; run make check in the worktree; ExitWorktree with keep, fast-forward main to the feature branch, then remove the worktree and its branch. Done means all of these are shown in this conversation: the <TNN.FNN> row is checked in its track roadmap on main and its spec is Complete; make check exited 0 at the commit now on main; git worktree list no longer lists the feature worktree; git status on main is clean. If a concrete blocker stops the feature, record it in the spec, report it, and stop. Stop after 80 turns.
```

The `/goal` evaluator reads only this conversation and runs no commands, so
surface command output and the checked roadmap rows in your own messages, not
only inside subagent results.

## Per-feature contract

Normative. The orchestrator reads this at goal start and follows it.

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
implementer runs `make rust-mutants` once. The target diffs the worktree
(committed, uncommitted, and untracked) against its merge base with `main`,
runs `cargo mutants --in-diff` with the caps in `.cargo/mutants.toml`, writes
its output under `~/.local/share/petri-tools/mutants/<worktree>/`, and prints
every survivor: mutants missed by every test and mutants that timed out. The
implementer records in the spec's Verification section the summary line, the
output path, and the full survivor list, each survivor resolved as **killed**
(a test added or strengthened, then the target rerun), **equivalent** (one
sentence on why it cannot change observable behavior), or **deferred** (a
deferred finding in "Notes for AI Agents"). Production code is never edited to
kill a mutant, and any `#[mutants::skip]` or `exclude_re` entry carries a
written justification. "No survivors" or "nothing to mutate" are valid records
when that is what the target printed.

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

### Close and integrate

In the worktree: set the spec to `Complete`, check the feature row, update the
track and master rollups only if their own criteria are now satisfied, run
`make check`, and commit. Then:

1. `ExitWorktree` with `action: "keep"` — the session returns to the main
   checkout. While inside a worktree, Claude Code blocks every git command
   aimed at the main checkout, so the merge cannot happen before this step.
2. `git merge --ff-only worktree-<tnn-fnn>`. If it fails because `main` moved,
   stop and report; do not merge or rebase without asking.
3. `git worktree remove .claude/worktrees/<tnn-fnn>` and
   `git branch -d worktree-<tnn-fnn>`.
4. Show `git status`, `git worktree list`, and the `make check` result in your
   own message so the goal evaluator can see them.

This workflow authorizes local branch, worktree, and commit creation and the
fast-forward of `main`. It does **not** authorize pushing, opening or updating a
pull request, or any other remote mutation.

### Cost record

After the feature closes, add one line to that feature spec's "Notes for AI
Agents": the `/usage` totals at closure (ask the user; `/usage` is a user
command), the implementer's self-reported advisor consult count, and the
reviewer's finding counts by severity. Telemetry only, never a success
criterion.

## Why this shape

Research date 2026-09-04, Claude Code 2.1.260.

- Opus 5 executor with a Fable 5.1 advisor is the most accurate configuration
  Anthropic reports measuring, and the advisor is consulted at decision points
  rather than every turn.
  [Cost and intelligence](https://platform.claude.com/docs/en/about-claude/models/optimizing-for-cost-and-intelligence),
  [advisor](https://code.claude.com/docs/en/advisor).
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
  tens to a few hundred mutants at roughly 25 seconds each for v3-cli and
  longer for v3-core at two jobs, so the run is diff-scoped and one-shot.
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

Superseded material is historical and non-executable: the
[2026-09 orchestration design record](archive/agent-orchestration-2026-09.md),
the [archived PRDs](prds/archive/README.md), the
[historical V3 roadmap](archive/v3-program-roadmap.md), and the
[legacy workflow inventory](archive/legacy-workflow-inventory.md).
