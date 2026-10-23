# Roadmap Execution Workflow

This is the only live workflow for implementing roadmap features. One goal
implements one feature, in its own worktree, and merges it into `main`.
Claude and Codex share the per-feature contract below. Codex uses the
[Codex adapter](workflow-codex.md) for model, tool, and worktree differences;
Claude uses the original instructions. Do not mix their session controls.
Both may run concurrently on different eligible features. Claude's agent
definitions, hooks, settings, and goal command remain active.
Lab experiments on the capability lab (exploration and why-not diagnosis) are
not roadmap features and do not use this contract; the
[lab contract](lab.md) governs them.

## Roles

Claude:

| Role | Model | Where it is defined |
| --- | --- | --- |
| Orchestrator | Opus 5.5, effort `medium` | the session you paste the goal into |
| Spec owner and advisor | Fable 5.1, effort `high`, persistent | `.claude/agents/roadmap-spec-owner.md` |
| Implementer | Opus 5.5, effort `medium`, Fable 5.1 advisor | `.claude/agents/roadmap-implementer.md` |
| Benchmark specialist | Sonnet 5, default effort | `.claude/agents/roadmap-benchmark-specialist.md` |
| Mutation specialist | Opus 5.5, effort `medium` | `.claude/agents/roadmap-mutation-specialist.md` |
| Spec challenger | Codex Sol (`gpt-6.1-sol`), effort `high`, read-only, fresh thread per round | run by the spec owner; see "Codex channel" |
| Reviewers | three Codex Sol (`gpt-6.1-sol`) jobs, effort `high`, read-only, fresh threads, one lens skill each | run by the orchestrator; see "Review" |

The orchestrator delegates, verifies, and integrates; the thinking that needs
the frontier model — writing the spec and resolving requirement questions
during implementation — lives in one persistent Fable spec owner, resumed by
`SendMessage`. Independent judgment comes from a different model family:
Codex Sol challenges the spec adversarially before it is committed and
performs the final diff review through three lens skills.
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
registry) on the feature diff, and `make rust-mutants-ci` runs the same fresh
run sharded on GitHub Actions; both are deliberately outside `make check` and
the stop gate, because their output is a survivor list to triage, not a score.
The benchmark and mutation specialists run sequentially and never alongside
each other or competing builds, tests, servers, or measurements.

**Codex channel.** Both Codex roles run as read-only `codex exec` jobs on
CLI 0.159.1 or later, called directly: the `openai-codex` plugin's companion
script goes through a broker shared with other sessions that may run an older
CLI which rejects `gpt-6.1-sol`, and that broker must not be killed. Write each
brief to a file outside the worktree (the session scratchpad or `mktemp`) so it
never appears in the diff, and name an output file beside it. A Codex job can
outlast the 600-second Bash limit, so start it with the Bash tool's
`run_in_background` and read the output file when it completes:

```sh
codex exec --skip-git-repo-check -s read-only -m gpt-6.1-sol \
  -c model_reasoning_effort=high -C "$WORKTREE" -o "$OUT" - < "$BRIEF"
```

Report the output file's path, so another session can read the verdict. If a
job fails with a usage limit, mark the review pending and retry later.
Every brief states that the sandbox is read-only and that Codex must not edit
files or run `cargo`, `make`, or benchmarks. Every round is a fresh `codex exec`; never
resume a previous session, which carries a stale spec or diff forward. The plugin's optional
stop-time review gate is independent of this workflow; when enabled it adds a
Codex review at every orchestrator stop and can block the stop. Route any
finding it raises on feature code through the implementer or the spec owner as
this contract requires; the orchestrator still writes no feature code.

## Run the next feature

In Codex, use the [Codex launch instructions](workflow-codex.md#launch-and-goal-prompt) instead.

1. Ask any session in this repository: **"give me the goal command for the next
   roadmap feature."** It applies the next-feature rule below and returns one
   `/goal` command for Claude, or the Codex goal prompt when requested in Codex.
2. Open a new session in the **main checkout** (not a worktree), on a clean,
   current `main`. Select Opus 5.5, effort `medium`, and auto mode. Keep agent
   teams disabled (`CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS` unset or `0`).
3. Confirm the startup notice `Advisor Tool (experimental) is on`. If it is
   missing, run `/advisor fable` once (a Fable advisor bills to usage
   credits on plans where Fable usage does). Do not consult the advisor
   yourself; its value is on the Opus implementer, and the orchestrator's
   Fable channel is the spec owner.
4. Paste the goal command. Nothing else to paste: the goal tells the
   orchestrator to read this file.

**Next-feature rule.** Take the priority order in the "Notes for AI Agents"
section of `docs/roadmap.md`. The next feature is the first ID in that order
whose row is unchecked in its track roadmap and whose dependencies are all
checked. Compute it every time: the order carries no closure state, and this
file records no current answer.

**Plan a track or master change.** Before committing new, re-scoped or
reordered rows in a track roadmap or `docs/roadmap.md`, or a change to a
feature's dependencies or to the master's priority order, run one fresh read-only
Codex Sol `high` job through the Codex channel applying `$plan-design-review` to
the changed rows and any design note they cite, with all six of the skill's
checks and its coverage section, and with
[`docs/architecture.md`](architecture.md) named in the brief as the desired
state the rows are checked against. The track's Notes name the architecture
sections the rows serve and the A8 gaps each row closes; rows that change the
desired state edit that file in the same commit, and a gap no row closes is
proposed to the user, never added as a row silently. Record the coverage table, findings and
dispositions in `docs/progress/readings/<track>-design-review.md`, as T25 did. Upheld
`step-back:` and premise findings go to the user before the rows are committed.

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
/goal Roadmap feature <TNN.FNN> is complete on main. Read docs/workflow.md first and follow its per-feature contract exactly: confirm you are Opus 5.5 at effort medium in the main checkout on a clean main; create the feature worktree with EnterWorktree named <tnn-fnn>; delegate the flat spec and its Codex adversarial challenge rounds to roadmap-spec-owner, verify the final Codex verdict, and commit the spec there, and route requirement questions during implementation back to that same spec owner; delegate feature implementation and production-code remediation to roadmap-implementer, and, as the spec's Verification Plan declares them, the benchmark profile runs and their records to roadmap-benchmark-specialist and the mutation gate and test-only survivor remediation to roadmap-mutation-specialist; run the final diff review as three fresh read-only Codex Sol high jobs, one lens skill each, through the Codex channel; run the benchmark and mutation specialists sequentially and never alongside competing builds, tests, servers, or measurements; run make check in the worktree; ExitWorktree with keep, fast-forward main to the feature branch, then remove the worktree and its branch. Done means all of these are shown in this conversation: the <TNN.FNN> row is checked in its track roadmap on main and its spec is Complete; make check exited 0 on the feature code now on main and make check-docs exited 0 at the commit now on main; git worktree list no longer lists the feature worktree; git status on main is clean. If a concrete blocker stops the feature, record it in the spec, report it, and stop. Stop after 80 turns.
```

The `/goal` evaluator reads only this conversation and runs no commands, so
surface command output and the checked roadmap rows in your own messages, not
only inside subagent results. Run a heavyweight check once, redirect its output
to a log file, and read the log; never re-invoke `make check`, a benchmark, or
a mutation run just to filter its output differently.

**`make check` runs once per feature** (user decision, 2026-10-10): at Close,
on the final feature code, and nowhere else. No implementer, specialist, or
remediation brief asks for it. A pass verifies with
`cargo check --workspace --all-targets`, `cargo fmt --all --check`, the tests
of the crates it touched (the viability gate first where it applies),
`make rust-clippy` when Rust changed, and the frontend's own tests when it
touched the frontend. A fast-forward leaves `main`'s tree identical to the
tested commit, so `make check` is never rerun on `main`. Commits after the
tested commit that touch only documentation are checked with `make check-docs`.

## Per-feature contract

Normative. The orchestrator reads this at goal start and follows it, applying
the explicit substitutions in [`docs/workflow-codex.md`](workflow-codex.md)
when running in Codex.

### Start

Verify before doing anything else, and stop with the reason if any check fails:
you are Opus 5.5 at effort `medium`; `git rev-parse --show-toplevel` is the
main checkout, not a
path under `.claude/worktrees/`; `git status` is clean on `main`; the target
feature is unchecked and every dependency in its track row is checked. If the
session is already inside a worktree, stop and ask the user to relaunch from
the main checkout — a worktree session cannot merge into the main checkout.
Load `SendMessage` with `ToolSearch select:SendMessage`; the spec owner is
resumed through it. If the tool is absent in this build, continue, and apply the
fallback in "Plan" below.
Run the Codex channel's `setup --json` and confirm `"ready": true`. If Codex is
not installed or not logged in, stop and report it; both independent reviews
depend on it, and there is no Claude fallback.

Then call `EnterWorktree` with the feature name. The session moves to
`.claude/worktrees/<tnn-fnn>` on branch `worktree-<tnn-fnn>`, branched from
`main`. Run `cd frontend && npm ci` there; a worktree is a fresh checkout.
Prefix `PATH` with `~/.local/share/petri-tools/bin` and
`~/.local/share/aquaproj-aqua/bin` so pre-commit and Node resolve.

### Plan

Delegate the Plan step to `roadmap-spec-owner`, spawned in the foreground
(`run_in_background: false`) so nothing else touches the worktree while it
writes. Give it the feature ID, the worktree path, and the original requirement
from the goal; its agent definition names the sections it reads — the roadmap
contract, the feature's own track row, its dependency rows, the track Notes
entries that name them, and the Goal, Inputs and Invariants, and Performance
predeclaration of each dependency spec. Do not read those documents yourself to
pre-digest them: the point of the delegation is that the planning inputs never
enter the orchestrator's context.

The spec owner creates the flat feature spec at
`docs/specs/roadmap/tnn-fnn-<slug>.md` from the template and runs the Codex
challenge loop below, with its own `$spec-review` readiness review after the
design round and before the readiness rounds. The spec
declares its [Verification Plan](#verification-plan): which benchmark profiles
and whether the mutation gate run, each with its reason. It then sets
the spec to `In Progress`, promotes a `Planned` track and a `Planning` master,
and runs `make roadmap-check`. Verify its report against the files — the spec
exists, the statuses moved, the checker passed — read the final challenge
round's output file, and commit the plan
only when round 2 or later has run and its verdict is `ready` or every open
blocking finding carries a rebuttal that round accepted.

**Architecture.** [`docs/architecture.md`](architecture.md) is the desired
state of the system. The spec owner reads it and fills the spec's Architecture
section: the A-sections the feature works within, the A8 gaps (or the parts of
one) it closes, and the gaps owned by other rows it preserves. A feature that
changes the desired contract itself (A1–A7: a domain, an allowed dependency,
an owner or a boundary rule) edits `docs/architecture.md` in the plan commit,
with an A8 line naming the feature for the code it has not yet changed; work
the desired state already allows needs only the citations. A plan may preserve
a gap another row owns without taking on that row's scope; when the feature
cannot avoid adding a new use of that gap, the section declares it with the
owning row and the reason. Every challenge round's brief names that file and
states this rule, so a declared preservation is not a blocking finding.
`make roadmap-check` fails a spec without the section or citing a section the
file does not define. The orchestrator commits the plan only when the
Architecture section is filled and any such edit is in the commit.

**Preparatory refactoring.** While writing the spec, the spec owner applies
`$refactor-audit` in preparatory mode to the modules the feature will change,
so the feature fixes a deficiency instead of working around it. Each
`in-feature` refactor becomes a `Decision:` note, part of the feature's scope,
and an Implementation Task the implementer's brief carries. An `accept`
workaround gets a `Deferred:` note naming the row or reason that pays it, and
an Implementation Task to write its `WORKAROUND(TNN.FNN): <intended design>`
comment at the site. A `prerequisite` refactor or a `re-plan` finding is a
blocker: the spec owner reports it, and the user decides. When the user approves
a prerequisite row, the orchestrator adds it and the feature's dependency on it
to the track roadmap on main, runs `make roadmap-check`, commits that planning
change, and stops; the prerequisite then runs as its own feature before this
one resumes.

**Codex challenge loop.** The spec owner runs up to three rounds, each a fresh
Codex Sol `high` job through the Codex channel, whose brief names the spec
path, the original requirement, the roadmap contract, the feature's track row,
dependency rows, and matching Notes entries. Round 1 is the design round: it
applies `$plan-design-review` only and runs all six of its checks, namely code
decomposition, task decomposition, domain boundaries and separation of concerns,
outdated or never-correct premises, a larger design flaw (step back), and
workarounds that should be refactors. Checks 3 and 5 run against
`docs/architecture.md`: the round reports a leaky abstraction, drift from an
A7 rule or an A5 owner, a design decision that means a boundary or the desired
state should change, and a missing edit to that file. The brief supplies the spec owner's
`$refactor-audit` table for check 6 to test, not to trust, and requires the
skill's full report: the coverage section, with one line per check naming what
was examined, any `step-back:` lines, the assumptions and workaround tables, and
the `Could not verify:` list; the brief overrides only the severity labels
(`blocking:` and `advisory:`) and adds the verdict line. A round-1 output that
lacks a check's coverage line is incomplete: the spec owner replaces it with a
fresh round-1 job, which does not count as a round, rather than dispositioning
it.
Round 2 is the readiness round: it applies `$spec-review`, confirms the round-1
dispositions, and asks Codex to break confidence that the spec is ready to
implement. Its attack surface is contradiction with the roadmap row or
requirement, scope widened or narrowed, acceptance criteria that cannot be
tested, claims about current code that the code does not support, missing
invariants and edge cases, determinism, the natural-analog rule, an unrealistic
Performance predeclaration, a Verification Plan the change does not fit, and
requirements that steer toward needless complexity. Round 3 confirms round 2's
fixes. Each finding is `blocking` or `advisory`, cites the spec section and the
evidence, and proposes a concrete change. The brief dictates the output format:
one line per finding starting `blocking:` or `advisory:`, then a final line
`verdict: ready` or `verdict: not-ready`; round 1 adds the skill's assumptions
table.

The spec owner answers every blocking finding by editing the spec or writing a
one-sentence rebuttal grounded in the roadmap row, the requirement, or the
code, except for the user-owned findings below. Rounds 2 and 3 carry the
previous findings and their dispositions and ask Codex to confirm the fixes,
accept or reject each rebuttal, and, in round 3, report only new blocking
findings. The loop ends, from round 2 on, at a `ready` verdict or when no
blocking finding remains that Codex upholds. A blocking finding Codex
still upholds after round 3, or a new one first raised in round 3, is an
unresolved requirement: the spec owner reports it, and it follows the blocker
rule when only the user can settle it. Advisory findings are adopted or dropped
at the spec owner's judgment. Round 1's coverage section is copied into
`docs/progress/readings/<id>.md` as a six-row table (check, verdict, what was
examined, finding references or why it is clear). The findings and dispositions of every round go
in one table in `docs/progress/readings/<id>.md`, never in the spec; the spec
carries only the
resulting requirements. Quote the spec owner's agent name or ID in your own
message after the spawn so the resume address survives context compaction.

**User-owned findings.** In any design review of a spec or roadmap, by Codex
or by the spec owner's own `$plan-design-review`, two kinds of finding are the
user's to decide: a `step-back:` finding, which asks whether the work should
give way to a bigger change, and a blocking premise finding (`contradicted`,
`stale` or `unrecorded`) whose fix would change the scope, a constraint, a
boundary or an accepted tradeoff. A premise finding that only corrects a fact
about the code is the spec owner's to fix. A disputed classification goes to
the user. The spec owner reports user-owned findings under the blocker rule
and never rebuts them on its own authority.

**The spec owner stays on the feature.** It is the orchestrator's channel for
requirement questions for the rest of the run. Resume it by `SendMessage`,
never while an implementer or specialist is running, for: a proposed change to
requirements or acceptance criteria, conflicting technical advice, a request for
a verification exception, a benchmark or mutation result that does not fit the
predeclaration, or an integration conflict that changes behavior. Wait for its
answer before the dependent work proceeds. It resolves against the original
feature contract and records any spec revision as an explicit, checked edit; it
cannot expand user scope or waive a required check, and it is never reused as
the final reviewer. Send it the question and the evidence, not documents it
already read.

**Fallback.** If `SendMessage` is not exposed in the running build, each
escalation goes to a fresh `roadmap-spec-owner` whose brief carries the
committed spec path, the sections at issue, and the question; record the
deviation once in the spec's "Notes for AI Agents" as a `Decision:` bullet.

### Implement

Delegate feature implementation and production-code remediation to
`roadmap-implementer`. Mutation-gate test-only remediation belongs to
`roadmap-mutation-specialist`. Write no feature code yourself. Every implementer
pass starts a fresh agent and its brief carries what it needs. This is a cost
decision, not a tool limit: measured first launches carried ~300k tokens of
context against 89–142k for later ones, and a continued agent carries the large
context forward on every turn. `SendMessage` is available and is used for the
spec owner, whose context is small and worth keeping; it is deliberately not
used to keep an implementer alive. (The Codex adapter keeps one persistent
implementer; see `docs/workflow-codex.md`.)

**Brief 1 — build.** Give it the feature ID, the spec path, the change requested,
and the exact spec sections this pass needs — normally Goal, Inputs and
Invariants, Architecture, Telemetry, Implementation Tasks, and the Verification
checklist — and requires it to read the `docs/architecture.md` sections the
Architecture section cites and build toward them.
A spec whose Plan step ran before 2026-09-29 has no Telemetry section and
carries no telemetry obligation. Its scope is
implementation, tests (TDD, property tests for pure invariants), the viability
gate first where it applies, and a clean `cargo check --workspace --all-targets`.
It carries the detailed rules ($rust-skills, `make roadmap-check` on document
edits, POSIX `sh`). It reports the changed files, the exact commands it ran with
their results, and its advisor consult count, then stops.

**Brief 2 — self-review and spec (a fresh agent).** Give it the feature ID, the
spec path, brief 1's report, and the spec sections this pass needs —
Verification, plus the Performance predeclaration when the feature is subject to
it. Its scope is the `simplify` skill on the feature diff and the spec's
Verification update. It reports what the simplify pass changed, the verification
commands it reran with their results, and its advisor consult count.

**Self-review follows code, not passes.** Put the `simplify` skill in the brief
of the pass that follows a build, and in any remediation brief that adds code —
new code gets one self-review pass before it reaches the gate. A remediation
brief that only strengthens tests, or only edits documents, does not need it.
The implementer never decides this for itself; it does what its brief names.

**The mutation gate is not part of implementation.** When the spec's
Verification Plan declares it, it runs once, after review and any remediation,
on the code that ships. See "Mutation gate" below.

Splitting the work this way is a cost measure, not a correctness one: first
launches were the expensive ones in the sessions this rule came from. If it
proves worse in practice, collapse the two briefs back into one.

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
faith; if brief 2's report does not mention the simplify pass, send it back
before review.

**Order after implementation.** Review, then any post-review remediation, then
the declared benchmark profiles, then the mutation gate, then closure. Both
gates measure the code that ships, so neither runs before the review can still
change it.

**Mutation gate.** This is a closure gate, not an implementation step. It runs
only when the spec's [Verification Plan](#verification-plan) declares it, and
then **once, after the review and any post-review remediation are complete**, on
the final feature code. Running it earlier tests code that then changes and forces a
second full run; that ordering cost about two fresh runs per feature and up to
six on one of them. Delegate it to a fresh `roadmap-mutation-specialist` whose
only scope is this gate, its test-only triage, and the spec's Verification
record. Give it the worktree and spec paths, feature ID, final-review result,
and only the spec's Verification and Notes for AI Agents sections.

The gate is a fresh `make rust-mutants-ci` on the committed feature branch. It
requires a clean tree, pushes the branch to `origin`, and dispatches the
Offload workflow (`.github/workflows/offload.yml`) against the local merge
base with `main`, passed explicitly because `origin/main` may lag it. Four
macOS shards run `scripts/rust-mutants`, `scripts/rust-mutants-merge` refuses a
result unless every shard finished fresh on the same diff, and the target
downloads the merged output to `~/.local/share/petri-tools/mutants/<worktree>/`,
where a later `MUTANTS_ITERATE=1` pass reuses it. A run that fails or is
cancelled produces no result and is not a fresh run; dispatch it again. When
Actions is unavailable, the gate is a local fresh `make rust-mutants`
(`MUTANTS_ITERATE=0`, the default), recorded as such in the spec.
Either target diffs against the merge base with `main` (the local one also
includes uncommitted and untracked files), runs `cargo mutants --in-diff` with
the caps in `.cargo/mutants.toml`, and reports every survivor: mutants missed
by every test and mutants that timed out. The
diff limits where mutations are generated; each viable mutant invokes the
affected package's unfiltered test suite, including integration tests and
doctests. Test filtering and reducing property-test case counts are not part of
this target.
The `mutants` Cargo profile uses optimization level 1 with debug assertions and
overflow checks retained. The wrapper records `fresh` or `incremental` in
`run-mode.txt` beside `mutants.out` and identifies the mode in its output.
The mutation specialist records in the spec's Verification section the summary line, the
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
evidence, even if no survivors remain in that pass.

**One fresh run, two at most.** Killing survivors by adding or strengthening
tests does not require another fresh run: the contract already forbids editing
production code to kill a mutant, so the production content the first run
measured is unchanged. Run a second fresh `MUTANTS_ITERATE=0` pass only when
triage actually changed production code, test selection, or tool configuration,
or deleted or weakened a test — and record why in the spec. **A third fresh run
is not authorized:** stop, report the reason to the user, and let them decide.
Do not close a feature with unresolved survivors to stay under the limit; an
unresolved survivor is a blocker to report, not a number to hide.

### Review

Run the final diff review as three fresh Codex Sol `high` jobs through the
Codex channel, started together, one lens skill each. The skills live in
`~/.agents/skills/`, where Codex loads them by name; they are user-level, so a
host without them cannot run this step:

| Lens | Skill | The brief adds |
| --- | --- | --- |
| Correctness | `$adversarial-code-review` | lens 1; the spec sections Goal, Inputs and Invariants, Architecture, Telemetry, Verification, and the Performance predeclaration when the feature is subject to it; the Review section of this file |
| Design | `$architecture-review` | lens 2; the spec's Goal, Inputs and Invariants, and Architecture sections; `docs/architecture.md` sections A3, A5, A7, A8 and those the spec cites |
| Code health | `$code-health-review` | lens 3; the spec's Goal section and its `Decision:` and `Deferred:` notes |

Each brief also names `.claude/agents/roadmap-reviewer.md` as the project
checklist, ignoring its front matter, and its lens section there. The design
and code-health briefs name any later roadmap feature expected to extend the
touched code, since their P2 test depends on it and they do not read the
roadmap.

Every brief gives the worktree path, the feature ID and the spec path, and
asks for one finding per line starting `P1:`, `P2:`, or `P3:` with file, line,
issue, and smallest fix, followed by what it could not verify. The design and
code-health briefs also require their skill's coverage section, one line per
check with what was examined, listed first; the design brief also allows
ungraded `step-back:` and `architecture:` lines after it. A report missing a coverage line is
incomplete and that lens is rerun. The design and
code-health briefs tell Codex to start from the diff's code hunks and not to
read the touched `docs/` files other than `docs/architecture.md` (the design
lens reads its worktree copy), the workflow documents, the roadmap row or the
telemetry template: those are the correctness lens's inputs,
and reading them in every job multiplies the cost. No reviewer edits, runs
builds or tests, or consults the spec owner. Do not reuse a spec challenge
thread for a review. Quote each output file path, the findings and the
design and code-health coverage lines in your own message.

Only a P1 finding blocks progress. The design lens raises P1 only for a
breach the diff adds of an `AGENTS.md` rule or a `docs/architecture.md` A7
rule, a new import cycle between its A3 domains among them. Against an A8 gap: code that
already used it and that the diff only touches or moves is inherited, a
`pre-existing:` P2 naming the gap; a new use of it (a new import site, a new
caller, a wider type crossing the boundary) is added coupling and P1, unless
the spec's Architecture section declares it with the gap's owning row, which
makes it P2. The correctness lens raises P1 when the diff changes the desired
contract (A1–A7) without the matching `docs/architecture.md` edit, or the
spec's Architecture section misstates what the diff does. The
code-health lens raises P1 only for an unmarked workaround. A design-lens
`step-back:` finding, or an `architecture:` line proposing a change to the
desired state, never blocks closure and is never remediated in the feature:
record it as a `Deferred:` note and report it to the user at closure. P2 and P3 are advisory and are
recorded as deferred review findings in the spec's "Notes for AI Agents"
without expanding scope; a design or code-health P2 on code a later feature
will extend is worth one remediation pass, not scope creep. Refactors of code
the feature does not touch are never review findings: they come from an
on-demand `$refactor-audit` area-mode run over one crate area, whose
candidates become roadmap rows. Allow one post-review remediation pass, given
to a fresh implementer with a brief naming the finding, the files, and the spec
sections to update. Never silently waive a required check.

**Prose budget (enforced).** A spec records the state of the world at closure,
not the path taken to reach it. `scripts/roadmap-check.mjs` fails any spec over
**15 KB of non-table prose** — tables and fenced blocks are free — and rejects a
"Notes for AI Agents" line that is not a `Decision:`, `Exception:`, `Deferred:`,
or `Cost:` bullet. `scripts/implementer-gate` runs that checker on every changed
spec and blocks the implementer from reporting done, so the budget is a gate, not
an expectation. Specs written before 2026-09-10 are grandfathered in the checker
and leave that list when they are next rewritten under budget.

There is no readiness-review log, no implementation-deviation log, and no
pass-by-pass narrative. Fold each outcome into the section it changes, in the
present tense, and delete what a later pass superseded instead of annotating it.
A verification item names what is checked and where the result lives; it does not
prescribe test design in prose. Command transcripts and raw output go to
`docs/progress/readings/<id>.md`, which holds tables and transcripts, not
narrative.

**Telemetry section.** The reviewer checks that the spec's Telemetry section
is filled in or carries a `Not applicable` reason, that a counter or other
observation the diff adds is listed as a signal, and that a mechanism switch
or config value the diff adds is listed as recorded configuration. A spec whose
Plan step ran before 2026-09-29 has no Telemetry section, and the reviewer
does not require one.

**Spec claim spot-check.** The reviewer picks at least three claims from the
spec's Verification section — a command result, a file or test the diff adds,
and a specific number — and checks each against the diff or the stored
file. A claim it cannot check is reported as unverified, never assumed true. An
untruthful claim is P1 under the existing spec-truthfulness rule.

(Both were recorded as user decisions on 2026-09-05 alongside the one-goal-run
rule below, but never reached this contract until 2026-09-09.)

The reviewer audits neither the benchmark reports nor the mutation record: both
gates run after this review, so there is nothing final to audit yet. **The
orchestrator audits both at the closure gate instead**; the mutation record against the diff and, where readable, against the
`missed.txt` and `timeout.txt` in the recorded output path. A missing survivor
list, or a `#[mutants::skip]` or `exclude_re` without justification, is a waived
check and blocks closure. An unresolved survivor blocks closure until it is
killed, justified as equivalent, or explicitly deferred with the user's
agreement. A survivor "killed" by editing production code rather than a test is a
behavior change that must go back through review at the severity it warrants.

(Across the seventeen features closed before 2026-09-09, the reviewer's survivor
audit produced no P1. Moving it to the orchestrator keeps the check at the point
where the record is final, with one reader rather than two.)

### Benchmark gate

#### Verification plan

`make check`, the spec challenge loop and the Codex review run for every
feature. The expensive checks run only where their result could change a
decision (user decision, 2026-10-12). The spec owner declares them in the
spec's Verification Plan at Plan time, one line each with a one-sentence
reason, and picks the cheapest plan that answers the feature's question.

Start from that question: one sentence naming what the feature claims and
which reading would show it wrong. A check whose result could not change a
decision is not run. When a change spans several rows, the most demanding
row it touches wins. When two benchmark rows both seem to fit, take the
cheaper one unless the question names a reading only the costlier profile
takes. A feature that adds an environmental pressure declares at least
`gate + goal`, because the
[standard baseline contract](#environmental-pressures-in-the-standard-baseline)
reads it in the goal worlds.

| Benchmark profiles | Declare for | Cost |
| --- | --- | --- |
| `none` | a change that cannot alter a simulation trajectory or its cost: documentation, frontend, telemetry export, lab code, tooling | none |
| `gate` | a change to simulation cost or tick-loop code where no population reading is in question | seconds |
| `gate + goal` | a production behavior change answered by a 2,000-tick reading: a cost, a default, an operator, an early-dynamics effect | about 5 to 20 minutes |
| `gate + goal-long` | a question about long-run evolution: discovery, retention, memory, genome growth, persistence past the founding crash, and only on a feature the track plan names as one of its goal-long carriers (see below); add `goal` when the 2,000-tick series also matters | about an hour or more |

**Goal-long is rationed per track** (user decision, 2026-10-15): at most one
or two features per roadmap track declare it, and only where no cheaper
profile can answer the question. The track plan names those carriers; every
other feature declares at most `gate + goal` and defers its long-run reading
to the next carrier as a `Deferred:` bullet.

| Mutation gate | Declare for |
| --- | --- |
| `run` | new or changed production logic in `v3-core`, `v3-cli` or `v3-server` whose behavior tests must pin |
| `not run` | documentation, frontend, telemetry export, lab code, tooling, test-only changes, or a change confined to config values or constants that existing tests already pin |

| Slow tests (`make rust-test-slow`) | Declare for |
| --- | --- |
| `run` | a change to mutation operators or their draws, the sink or input catalogs, or recruitment code: the tier pins the recruitment-paths experiment's recorded readings, which such changes move; or a change to `crates/v3-lab`: the tier holds the lab's end-to-end runs |
| `not run` | everything else |

**Worked examples**: closed program 1 features, classified under these rules:

| Feature | What it changed | Benchmark | Mutation | Slow tests |
| --- | --- | --- | --- | --- |
| T23.F10 thinned creature window store | telemetry export and a stack job; no simulation behavior | `none` | `not run` | `not run` |
| T14.F11 progress page design | a static page in `docs/progress/` | `none` | `not run` | `not run` |
| T22.F02 barrier-navigation assay | lab code in `crates/v3-lab` only | `none` | `not run` | `run` (lab) |
| T24.F03 food growth phase timing | timing spans inside the tick loop; seeded output byte-identical | `gate` | `not run` | `not run` |
| T16.F02 effort-priced actions | action costs: early ecology moves | `gate + goal` | `run` | `not run` |
| Effort-cost defaults, 2026-10-10 | three cost values only | `gate + goal` | `not run` | `not run` |
| T02.F08 partial bites | a new action sink and its decoding | `gate + goal` | `run` | `run` (sink catalog) |
| T11.F31 ring projection motif | a mutation operator whose claim, avoidance evolving, shows only in long runs | `gate + goal-long` | `run` | `run` (draws) |

The slow tier holds `#[ignore]` tests marked `slow tier`, kept out of
`make check` because they cost minutes (user decision, 2026-10-12). When
declared it runs once at Close, beside `make check`, on the final code. A
cargo-mutants run does not execute it.

The reviewer checks the plan against the diff. A plan the diff contradicts,
such as `none` with a trajectory change or `not run` on new production logic,
is a P1. The user may set either line; the spec records that decision with its
date. A declared check that is skipped is a closure blocker, not a waiver.

A closure that runs no profile adds no benchmark summary and no series entry,
so later comparisons reference the last measured closure; the ordinary
benchmark tests inside `make check` still run. A closure that runs a profile
completes the spec's Performance and Goal Impact section, and a severe compute
regression without a predeclared, justified cost blocks closure until the user
accepts it or remediation removes it. Never weaken a
threshold or edit a stored baseline to make a feature pass.

**The long goal profile** (`make bench PROFILE=goal-long`) runs the three goal
worlds to 40,000 ticks as its own series, `goal_worlds_long` in the series
index, never compared with the 2,000-tick series. The first feature that
declares it finds no epoch baseline, so it runs the profile twice, the one
exception to the once-per-closure rule below. The benchmark specialist first
builds the feature's merge base with `main` in a detached scratch worktree and
runs `make bench PROFILE=goal-long FEATURE=<slug>-base` there, with
`SUMMARY_OUT` pointing into the feature worktree. The orchestrator then records
that summary as `goal_worlds_long.epoch_baseline`; this paragraph is the
standing authorization for that first pin, once per series, and no other
baseline edit. Then the feature's own run follows. A later goal-long reading
compares against the last one, which can be several closures old, so its
readings name every feature closed in between.

**Lab features.** A feature whose diff is confined to `crates/v3-lab`, the
workspace manifest and lock entries that register it, the `Makefile` entries
that build, test and run it, its tests and documentation declares `none`, and
the reviewer checks the scope claim against the diff. A lab feature that also
touches `v3-core`, `v3-cli`, `v3-server` or the frontend cannot declare `none`
on that ground, and neither can one whose manifest or lock change does more
than register the crate.

**Telemetry features.** A feature that exports or records observations
declares `none` and mutation `not run` on the ground that it changes no
simulation behavior, even when it wires export into `v3-cli`, `v3-server` and
observation seams in `v3-core`. That ground holds only while it changes no
simulation behavior: no
simulation default, no `SimulationConfig` or `RuntimeConfig` field, no
production RNG draw, no stored summary content and no deterministic output.
The process-level telemetry default that T21.F05 flips is not a simulation
default. The reviewer checks that claim against the diff, and a feature
that fails it loses both exemptions for that closure. An exempt closure adds
no benchmark summary and no series entry. `make check`, the spec challenge
loop and the Codex review still apply.

Three checks stand in for the skipped gates; the feature's spec states who
delivers each and what it must show.

1. The telemetry-neutrality test runs inside `make check` from T21.F01 on.
2. The parent comparison runs when the diff touches `v3-core` or code that
   runs inside the tick loop or a timed region. Build the feature's merge
   base with `main` in a scratch worktree, run the same short seeded workload
   on both builds with telemetry off, and require byte-identical output. It
   checks behavior only and takes seconds.
3. The overhead check runs when the feature adds work to a run, whether or
   not telemetry is on.

**One ceiling, one reference.** The ceiling in force is 25% until T21.F05
closes and the ceiling T21.F05 recorded after that. It applies to one ratio:
the changed build with telemetry on against the reference build of the same
commit, in which every telemetry addition is compiled out. Both sides hold the
same simulation code, so a change another track makes to simulation cost
moves both. Ratios recorded at earlier closures are attribution only and are
never multiplied into a current reading. The reviewer checks that every telemetry
addition to `v3-core` and to the binaries' run loops sits behind the build
switch the reference turns off.

**Method and time cap.** Timed runs are interleaved pairs on a short
workload and the reading is the median ratio. The spec predeclares the method
before any run: the workload, the pair count chosen from a short measurement
of the workload's run-to-run spread, and the result that counts as
inconclusive. The routine check is cheap by rule: all timed runs for one
closure, including the spread measurement and one permitted rerun, fit
inside 5 minutes of measured time. Every telemetry check runs on the
production `petri-telemetry` stack, started as `make telemetry-up` does when
it is down and never stopped by a check; the overhead check has one state,
that stack up for both sides of every pair. T21.F05's qualifying
measurement is the one longer allowance, 60 minutes, taken once. When a cap
is reached the result is inconclusive. An inconclusive result is reported to
the user and is never rounded to a pass or extended with more runs. Wrap
every timed run in `scripts/bench-wait`, and never run these checks alongside
another session's benchmark profile, mutation run or overhead check. The
orchestrator delegates them to the implementer's verification pass.

**Record.** The spec's Performance and Goal Impact section records
`Not applicable: telemetry feature` for the profiles, then each check
that ran with its method, the measured time used, the ratio against the
reference with the stack up and telemetry's self-timed cost per tick, or
`<check> not applicable: <reason>`.

From T15.F01 onward, each run writes a full local report under the main
checkout's ignored `.bench-artifacts/<feature>/<profile>.json` and a committed
summary under the calling checkout's `docs/progress/features/`. Gate uses
`<feature>.json`, goal `<feature>-goal.json`, goal-long
`<feature>-goal-long.json`, and sweep `<feature>-sweep.json`.
`OUT` changes raw output only; `SUMMARY_OUT` changes the summary. Both `make bench`
and direct CLI share these defaults. Commit summaries, concise readings and
series entries pointing to summaries; do not stage new full reports, even gate
reports. Existing historical reports remain valid references. The versioned
projection, provenance and deterministic conversion command are documented in
[`docs/benchmark-artifacts.md`](benchmark-artifacts.md).

**Telemetry commit rule (user decision, 2026-09-28).** This covers every
output a feature, assay or experiment produces, not only benchmark profiles:
commit the bare minimum summary needed to regenerate the report; keep richer
telemetry in the ignored `.bench-artifacts/` tree beneath a byte cap the
producing command enforces; never write per-tick traces or exhaustive
per-proposal records unless the spec predeclares them with a projected size.
A raw archive is evidence to cite by path, hash and bytes, not a second copy of
the summary, and a command that would exceed its cap stops and marks its
record incomplete rather than growing. Regenerating a report means rendering
its named consumers (the progress page, the closure comparison, an assay's
report), not reconstructing raw telemetry, so each producing command declares
a versioned keep-list for its summary, as the benchmark summary does. Writers
that exist on 2026-09-28 keep their behavior until they are next touched; the
rule binds every new command and every rewrite. Telemetry exported to the T21
local stack is never committed and never written into the repository tree: it
lives in the stack's Docker volumes until the user retires it with the track's
cleanup command, and a T21 spec that adds a trace predeclares its projected
size per run and its per-run cap. The stack's Compose file, configuration,
dashboard definitions and cleanup script are source.

After the review and any post-review remediation are complete, and before the
mutation gate, delegate the profiles the spec's Verification Plan declares to a fresh
`roadmap-benchmark-specialist`. Give it the worktree and spec paths, feature ID
and slug, exact commands, and only the spec's Verification and Performance and
Goal Impact sections. It owns the runs, local raw artifacts, committed summaries
and concise readings, and
the recorded CLI and observed outer-process exit statuses with their sources,
`severe` flag, threshold verdict, raw/summary byte counts and paths. It
does not change thresholds or baselines, remediate code, or interpret unexpected
results into a new requirement. Route those results through the orchestrator to
the spec owner and a fresh implementer as appropriate. If production
remediation after the run invalidates a report, return only the affected
final-code measurement to a fresh benchmark specialist before closure; this is
not a second determinism check. A production fix prompted by a benchmark result
goes back through review at the severity it warrants, as a production-code
mutation kill does.

From T11.F14 onward, the evolved-neighborhood observation cap is **180 seconds
per goal-profile run, summed across seeds**
(`environment.neighborhood_evolved_wall_clock_ms_total`). This permanent
budget adjustment was explicitly authorized by the user on 2026-09-06 and
supersedes T11.F01's 90-second cap for T11.F14 and future closures. The
founder-neighborhood cap remains 10 seconds of release wall time per profile
run, and the 15-minute total goal-profile investigation threshold remains for
the 2,000-tick profile. The long goal profile has no investigation threshold
until its first run records its time.
Profile parameters, sample/trial counts, mutation floors, normalized compute
thresholds, and stored baselines are unchanged. Preserve historical readings
and their original acceptance results; an exceeded current cap still requires
resolution before closure.

A declared goal or goal-long profile runs **once** per closure, except the
first goal-long run described above. A second run to re-check its
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

In the worktree: run `make check` once on the final feature code, plus
`make rust-test-slow` when the Verification Plan declares it, and record
that commit as the tested commit. Then set the spec to `Complete`, check the
feature row, delete the `docs/architecture.md` A8 lines the feature closed
and remove the parts of a line it closed in part, update the track and master rollups only if
their own criteria are now satisfied, run `make check-docs` (the closure edits are documentation, and
the full suite was just run on the same code), and commit. Then:

0. Confirm every check the spec's Verification Plan declares ran on the final
   code, and that each gap, or part of one, the spec's Architecture section
   closes is gone from `docs/architecture.md` A8 while the remainder stays. For a telemetry feature, confirm the spec records
   `Not applicable: telemetry feature`, the reviewer accepted the
   no-behavior-change claim, and the parent comparison and the overhead check
   are each recorded with a result or a `not applicable` reason.
   When benchmark profiles are declared, audit their records against the
   Performance and Goal Impact section: every declared profile ran on the final
   code, and every severe compute regression is predeclared and justified or
   accepted by the user.
   When the mutation gate is declared, audit its record: the
   summary line and output path in the spec's Verification section, every
   survivor resolved as killed, equivalent, or deferred, and `missed.txt` /
   `timeout.txt` at the recorded path agreeing with the list. A `#[mutants::skip]`
   or `exclude_re` without a written justification blocks closure. This replaces
   the reviewer's former survivor audit.
   Verify each new summary's raw hash/byte count against the available local
   file, its original measured identity and verification time, and its preserved
   comparison verdicts. Availability is local and time-stamped, not a download
   guarantee. Confirm series entries point to summaries and no new full
   benchmark artifact or raw telemetry of any kind is staged; historical
   migration is a separate feature.
1. `ExitWorktree` with `action: "keep"` — the session returns to the main
   checkout. While inside a worktree, Claude Code blocks every git command
   aimed at the main checkout, so the merge cannot happen before this step.
2. `git merge --ff-only worktree-<tnn-fnn>`. If it fails because `main` moved,
   rebase `worktree-<tnn-fnn>` onto `main` without asking, resolve the
   conflicts, and re-verify: `make check` on the rebased content only when the
   commits `main` gained touch code, `make check-docs` when they touch only
   documentation. When those commits change simulation behavior, re-run every
   profile the spec declares (gate, goal, goal-long) on the rebased tree and
   record those readings in place of the invalidated ones, keeping the
   superseded summaries' provenance in the readings file, so no reading omits
   a feature that merged first; this replaces invalidated verification and is
   not a second closure run. Record the new tested commit, then fast-forward. Report the
   rebase and its re-verification. Stop and report only when the conflicts
   cannot be resolved on the feature's own terms or the re-verification fails.
3. `git worktree remove .claude/worktrees/<tnn-fnn>` and
   `git branch -d worktree-<tnn-fnn>`; if the mutation gate pushed it, also
   `git push origin --delete worktree-<tnn-fnn>`.
4. Show `git status`, `git worktree list`, and the worktree's `make check`
   result with its tested commit in your own message, so the goal evaluator can
   see them. Do not rerun `make check` on `main`.

Executing a user-pasted goal authorizes local branch, worktree, and commit
creation, rebasing the feature branch onto `main`, the fast-forward of `main`,
and removal of the completed feature's worktree and branch. Merely reading or
editing this workflow grants no such authorization. It does **not** authorize
pushing, opening or updating a pull request, or any other remote mutation,
except that `make rust-mutants-ci` pushes the feature branch for the mutation
gate and closure deletes that remote branch.

### Cost record

After the feature closes, add one line to that feature spec's "Notes for AI
Agents": the `/usage` totals at closure (ask the user; `/usage` is a user
command), each implementer brief's self-reported advisor consult count and the
number of passes, the number of spec-owner resumes after the Plan step, the
number of Codex challenge rounds with the final verdict, and the Codex
reviewers' finding counts by lens and severity. Telemetry only, never a success
criterion.

## Codex, rationale, and history

Codex model, tool, and worktree substitutions for this contract:
[`docs/workflow-codex.md`](workflow-codex.md).
Why this workflow has this shape, and the dated record of changes to it:
[`docs/workflow-history.md`](workflow-history.md).
