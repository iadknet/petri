# Evolvability exploration: run plan

Date: 2026-10-01. Source: `a1258205`. Status: plan for one long exploration
run under the [T22 exploration contract](../roadmaps/t22-capability-assays-and-evolvability-lab.md);
no experiment run yet, no production source, default or founder changed.
The run writes its results to a separate note, named below. Reviewed by
Codex `gpt-6.1-sol` `high` on 2026-10-01 (verdict `not-ready`, 12 blocking,
6 advisory); round 2 confirmed 17 fixes and reopened baseline identity; round 3 confirmed that fix (verdict `ready`).

## Question

Why do newly added nodes and sensor reads rarely become useful under Petri's
own variation, and which general changes to the representation, operators,
costs or selection setting let them become useful? The run builds the
conditions under which a capability *can* evolve. It never makes creatures
use a particular sensor, node or action.

## Decisions (user, 2026-10-01)

1. Prototypes are allowed on the experiment branch only: production-crate
   mechanism changes, plus the lab harness-core changes needed to run and
   classify them. Every prototype arm is `policy-deviation`. The branch never
   merges into `main`. The T22 exploration contract records this exception.
2. One `/goal` run, capped at 150 turns.
3. Claims are reviewed by Codex `gpt-6.1-sol` at `high`, read-only, through
   `codex exec`. A fresh read-only `fable` agent stands in while Codex is
   rate-limited.

## Starting evidence

Read these in full before the first cycle. They are where the run starts,
not conclusions to defend. Each describes the revision it measured; check the
T11, T13, T17 and T20 tracks for defects repaired since.

- [T22.F04 readings](../progress/readings/t22-f04.md) and the stored
  [food-seeking summary](../progress/lab/t22-f02-food-seeking.json): of four
  completed native food-seeking replicates, two first fail at benefit and two
  pass it; retention is `inconclusive` in all four (quick runs select about
  4–6 improvements per replicate against a minimum of 15). Pooled, 13 of 321
  viable touching children improved. The lead is a low, uneven benefit rate,
  not a universal stall. Touching share is about 85 % because both founder
  nodes count as relevant sites, so the ladder does not isolate new modules.
  `ring-v1` is uncalibrated, and the T22 notes forbid tuning its geometry or
  margin to fix that.
- [Post-T19 sensor integration audit](post-t19-sensor-integration-audit-2026-09-23.md)
  and [mutation audit](post-t19-mutation-audit-2026-09-23.md), at `aef05f90`:
  every family was reachable by mutation; barrier readers were rare; the VM
  had a channel-zero creation bias; an unrepresented sensor needed
  preparation before it could connect. Some of these were repaired later
  (T11.F24, T11.F25 and others); the run re-measures before it assumes.
- [Drift-silence research](drift-silence-alife-research-2026-09-24.md) and
  [follow-up](drift-silence-followup-2026-09-24.md): deep-drift silence
  survives broader observation; early ecological genomes respond far more.
- [Live survey](live-survey-2026-09-16.md) and
  [motor-output encoding](motor-output-encoding-research-2026-09-16.md): an
  earlier "sensors don't evolve" case was traced largely to the scalar
  direction encoding (repaired by T11.F21); that note also says encoding was
  not the sole cause.
- [Mesh depth](mesh-depth-research-2026-09-07.md),
  [neutral module recruitment](neutral-module-recruitment-research-2026-09-08.md),
  [incremental recruitment](incremental-recruitment-research-2026-09-19.md),
  [sensorimotor ALife survey](alife-sensorimotor-learning-research-2026-09-23.md),
  [capability assay research](capability-assay-research-2026-09-28.md).
- Track notes for T11, T13, T17, T20, T21 and T22, so the run neither
  re-proposes a planned feature nor revives a rejected one, and the master
  rules in [`docs/roadmap.md`](../roadmap.md) (natural-analog rule; no feature
  optimizes against an indicator).

Opening hypotheses, to be confirmed, refined or killed:

| ID | Hypothesis | First test |
| --- | --- | --- |
| H1 | The low benefit rate is partly an instrument limit: few replicates, a scorer blind to partial progress, or a founder near the ceiling | More replicates; scorer sensitivity between founder, comparator and floor |
| H2 | Exposure: only food and `wall-v1` are calibrated, so most sensor families never matter in any scene; solo evaluation has no other creatures | List which families the current scenes can expose; new lab instrumentation for one more family at most in Stage 0 |
| H3 | Stepping stones: one mutation rarely completes a sense-to-vote path, and the partial steps are neutral or harmful | Path-length census on touching births; two-step neighbourhoods |
| H4 | Incumbent masking: the founder's food path wins the vote, so a new contribution cannot change the action without displacing it | Vote-margin readings on touching births; duplicate-and-diverge arm |
| H5 | Targeting: executed-biased targeting starves new, silent tissue of further edits | Targeting-variant arm against the reference arm |
| H6 | Cost: maintenance and compute costs remove new tissue before it can help | Cost-variant arms with matched controls; survival of touching lineages |
| H7 | Step size: new edges enter at weights too small to matter or large enough to be lethal | Initial-weight distribution arms |
| H8 | Node boundary: a Graph edge reads only its own node (`GraphSource` in `cgp.rs`); other nodes reach it only through the 24-slot upstream baton from the previous hop or the 16 shared-memory slots. A new node's output then needs extra steps to reach anything, and mesh, Graph and VM operators split the mutation supply | Extend the H3 census to count how many partial paths cross a node boundary. If the boundary is implicated, a prototype arm adds a cross-node edge source that reads another node's last value this tick (zero if that node has not run yet). Analog: axon growth forming a direct projection between existing regions. It changes evaluation semantics, so it needs matched controls |

H8 comes from a 2026-10-01 user question: why not make the mesh one graph,
with VM and Graph nodes as node kinds inside it? That question bundles two
changes. One address space is H8. Nesting graphs inside graphs (embedded CGP
modules, Koza's automatically defined functions) is not an arm. Miller's
review ([2020](https://link.springer.com/article/10.1007/s10710-019-09360-6), Section 3.2)
calls the module comparisons unfair because the modular runs got larger
genotypes, and leaves "whether module acquisition is beneficial" an open
question. The
[incremental-recruitment note](incremental-recruitment-research-2026-09-19.md)
already rules out replacing the mesh before the smaller contest.

## Options considered

| Option | Verdict | Why |
| --- | --- | --- |
| Stay inside the T22 contract (overlays, genome and arena files, lab code) | Rejected as the whole scope | Cannot test node architecture or operator changes, which H3–H8 need |
| Branch-only prototypes as labelled arms | **Adopted** | Tests representation and operator changes on the real engine; nothing reaches production without a roadmap feature |
| Run each change through the feature workflow | Rejected | T22 says lab exploration is not a feature; spec, mutation and benchmark gates buy nothing for throwaway arms |
| [karpathy/autoresearch](https://github.com/karpathy/autoresearch) (checked 2026-10-01): one editable file, fixed 5-minute runs, keep the commit if `val_bpb` improves, else reset; never stop to ask | Partly borrowed | Borrowed: fresh branch, fixed per-run wall budget, append-only log, crash → log and move on, no pausing to ask. **Not** its keep-if-score-improves rule: hill-climbing a lab score is the engineered-solution pattern the natural-analog rule forbids |
| [GEPA](https://gepa-ai.github.io/gepa/guides/) 0.1.4 (MIT, 2026-07-15; [Agrawal et al. 2025](https://arxiv.org/abs/2507.19457)): LLM reflection over traces evolves text artifacts against a metric, keeping a per-task Pareto frontier | Partly borrowed | Borrowed: a per-assay table that never collapses results into one number, kept descriptive. Not the optimizer: it climbs a metric by scoring candidates on minibatches over a configurable budget of many metric calls, which here is the forcing pattern, at a scale our build-and-campaign cycles cannot afford; and it adds a Python dependency |
| [AI Scientist-v2](https://arxiv.org/abs/2504.08066) (Sakana, 2025): an experiment manager runs staged agentic tree search and seeds each stage from the best node of the last | Partly borrowed | Borrowed: explicit stages with a review between them. Not its parallel tree search or paper writing; AGENTS.md forbids parallel workflow machinery, and cost |

## Rules of the run

### Where

A Claude worktree from a clean, current `main` (`EnterWorktree` named
`evolvability-exploration`). Record the launch revision. Before the first
build: `npm ci` in `frontend/`, `aqua policy allow` if prompted, at least
30 GiB free (`df -h .`). Never touch another session's worktree. One heavy job
(build, test suite, lab campaign, world run) at a time.

### What may change

- `lab:` commits: arenas, layout files, overlays, genome files, readings,
  scorers and assays in `crates/v3-lab/src` and `crates/v3-lab/tests`, not
  touching the harness core (loop, formats, calibration gate, controls).
- `docs:` commits: notes in `docs/strategy/`.
- `proto:` commits: mechanism prototypes in `v3-core` or other production
  crates, observation adapters, telemetry exports, and the harness-core
  changes needed to run and classify prototypes. Nothing else. The
  lab/production boundary holds on the branch too: no production crate
  depends on `v3-lab`, and no lab arena, scorer or selection code moves into
  a production crate.

A prototype sits behind a new config field whose default reproduces `main`,
and an arm selects it by overlay. The lab's automatic classification compares
only the `mutation` block (`classify` in `crates/v3-lab/src/arena.rs`), so the
first `proto:` commit extends it: any arm that enables a prototype field is
`policy-deviation` in rows, summaries and verdicts, with a fixture for a
non-mutation prototype.

**Baseline identity.** Before any edit, at the launch revision, run every
assay and settings combination the run will compare (seed, sizes, assay,
arena) and store two sha256 hashes under
`.bench-artifacts/lab/exploration/baseline/`:

- the reference arm's NDJSON rows, which are byte-identical for a seed by
  T22's own tests;
- a deterministic projection of the summary: the calibration block with its
  thresholds, the reference arm's results, fidelity and ladder verdicts, and
  the seeds and resolved config digest, without the `timing` block
  (`wall_seconds`, `per_creature_tick_ms`), the source revision, the dirty
  state and the other arms. The first cycle writes the projection as a
  `jq` filter stored beside the hashes and uses it unchanged after that.

Before a prototype arm is read, both hashes for the same combination must
match. A combination added later gets its baseline from a temporary checkout
of the launch revision. A new instrument (scene kind, scorer, assay) did not
exist at launch, so its baseline is taken with its reviewed `lab:` commits
applied to the launch revision and no `proto:` commit, before any prototype
arm uses it.

**Branch verification.** Use TDD for prototype behavior and `$rust-skills`
for Rust. When a prototype touches founder behavior or tick-loop mechanics,
run `cargo test -p v3-core --test viability` first. Before a prototype's
results count, `make check` on the branch must pass except for the v3-cli
recipe digest pin test, which a new `RuntimeConfig` field is known to break;
record that single named failure in the ledger. Any other failure makes the
evidence unusable until fixed.

### Telemetry

The T21 and T23 observability work is available wherever it helps. Use
`make telemetry-up` to start the local stack (Grafana on `127.0.0.1:3300`).
Lab and world runs export run records by default (`--telemetry on`).
Recorded creature windows (T23.F10) show what a creature sensed, chose and
got, tick by tick. If the stack is already up, use it as is. Never run
`make telemetry-clean` with `PROCEED=1`, and never delete its volume.
Dashboards are for looking. A number the results note relies on is cited
from a stored lab summary or report, as the T21 notes require.

### Not forcing evolution

An arm is admissible only if all hold:

1. Its change is a general rule over node kinds, inputs, operators, costs or
   placement. It names no sensor family, arena, action or scene. Arms that
   isolate a family to diagnose it are labelled `diagnostic` and are never
   recommended.
2. Its hypothesis names a natural analog and the analog's actual mechanism
   (for example gene duplication then divergence, connection cost,
   developmental bias, neutral drift).
3. It supplies no task-solving correspondence: no rule that pairs a source
   with the sink that solves a scene (such as input channel *i* to output
   *i*), and no copying of a competent incumbent into new modules.
4. Nothing it does depends on the assay: no proposal accepted, retried or
   weighted by an observed behavior or score, and no reward, subsidy,
   curriculum, authored wiring or pre-wired founder.
5. The ledger row states what ordinary variation still has to discover for
   the capability to appear.

Lab selection is the measuring instrument, not a proposal for the world.

**Matched controls.** An arm that changes evaluation semantics (costs,
execution, decoding) can raise reach by helping the unchanged founder, not by
improving discovery. For such an arm, also evaluate the fixed founder and the
comparator under the arm's semantics, and run its matched `mutation-off` and
`shuffled-score` arms, on the same scenes, horizon and thresholds. Report the
immediate effect (founder under the arm) apart from the evolutionary effect
(change in discovery beyond it).

**Holdout.** Before Stage 2 starts, reserve a held-out set (assay settings
and a seed bank) and record it in the note. A candidate is frozen, with its
ledger row committed, before the holdout is read. Once a holdout result has
been read, any further change or combination using it is exploratory and
needs a new holdout.

**What a result may claim.**

- *General assay improvement*: the reached fraction or stall rung moves
  against the reference arm, paired by seed, beyond the matched controls.
- *Recruitment*: additionally, the elite's newly acquired node or read is
  identified, ablating it (T22.F03 causal readings) removes the gain, the
  incumbent's competence is preserved, and discovery is reported apart from
  descendant retention.
- *Promising*: a recruitment claim that holds on the held-out set, on at
  least two assays, with fresh replicates. Six replicates per arm is the
  exploration minimum and supports leads only; a rung is read only where the
  replicate meets the ladder's adequacy minimum; arm contrasts are paired by
  seed, never inferred from overlapping Wilson intervals.

Keep and discard decisions are about findings; a higher score alone decides
nothing. Negative and null results stay in the ledger.

**Frontier table.** The results note keeps one row per arm and one column
per assay or sensor family. Each cell shows the paired difference from the
reference arm in reached fraction and stall rung, with direction (better,
worse, no detectable difference), `inconclusive` where adequacy fails and
`not run` where unmeasured. It is descriptive. It never ranks arms, never
retires or retains an arm by itself, and an arm is labelled better than
another on a cell only with a paired, supported difference. The next
experiment is chosen by the causal question it resolves, not by which cells
improved. Combining two arms is allowed only as a predeclared interaction
hypothesis with its own natural analog, never because their rows look
complementary.

**Natural-world check.** Before the final note recommends any roadmap
feature, run the prototype in a natural world (births, no scorer) next to
the default with paired starting state and seed and a common tick horizon,
using `v3-cli run` output and run telemetry; `input_use` and
`recruitment_paths` have no path for a custom world today, so either build an
observation-only `proto:` adapter or leave those readings out. Store a compact
reading for every number cited. One world, a declared tick cap, the 15-minute
rule below. Descriptive only: it shows ecological plausibility, the lab shows
reachability, and neither shows retention.

### Cycle

1. **Research.** Pick the next question from the ledger by the uncertainty
   it resolves. For a new mechanism, read its primary prior art in full and
   cite it. If a fetch fails, use the browser pane.
2. **Predeclare** in the ledger before running: hypothesis, natural analog,
   the not-forcing checks, arms and matched controls, assays, seeds and
   replicates, prediction, what would falsify it, wall budget.
3. **Build.** Lab additions or a prototype, under the verification rules
   above.
4. **Run.** The lab has no wall-cap flag, so run it under a 15-minute kill. A
   killed run is `incomplete`: it counts toward neither completed experiments
   nor reach. Raw output goes under `.bench-artifacts/lab/exploration/`;
   commit nothing raw.
5. **Read** the why-not rung, the reached fraction, what changed in the
   elites (T22.F03) and the controls, under the claim rules above.
6. **Record** the ledger row, update the results note and frontier table, and
   commit. The note is the checkpoint: a dead session loses at most one cycle.

Stages, which may step back: **0 Instrument** (pilot one full cycle and set
the cycle budget from its turns and wall time; record the baselines;
characterize the benefit rate with more replicates; list which families the
current scenes can expose), **1 Diagnose** (which rung, and why, per exposed
family), **2 Mechanism** (arms against H3–H8 and new hypotheses), **3
Confirm** (holdout, ablations, the natural-world check). Start Stage 3 by
turn 105 so confirmation is not squeezed out by more exploration.

H2 is bounded: layout files hold only empty cells, barriers, food and one
solo start, and the ladder knows food and barrier families. Another family
needs new lab instrumentation (scene kind, scorer, relevant-site predicate);
take on at most one in Stage 0, and record families that solo evaluation
cannot expose (social ones) as a finding, not a gap to paper over.

### Review

Send the results note, the ledger and `git diff <launch>...HEAD --stat` to a
fresh read-only Codex job:

```
codex exec --skip-git-repo-check -s read-only -m gpt-6.1-sol -c model_reasoning_effort=high -C <worktree> -o <out-file> - < <brief-file>
```

Review at the end of every stage, after every fourth cycle, and before the
note recommends anything. The brief asks for forcing violations against the
five checks, confounds, missing matched controls, holdout leaks, overclaimed
n, and alternative explanations. Each finding gets a disposition row in the
note. On "usage limit", use a fresh read-only general-purpose agent with
`model: "fable"` and the same brief.

### Stop and ask only when

Disk falls below 20 GiB, a step would change `main` outside the closing
steps, or the run needs a production default, founder or mutation-policy
change outside the branch. Otherwise keep going: log the problem, choose the
next question, continue.

## Deliverable and close

The results note `docs/strategy/evolvability-exploration-2026-10.md` holds:
the question, the launch revision, the ledger (one row per experiment, with
`incomplete` rows kept), the frontier table, findings by rung and family with
counts, review dispositions, the prototype branch name and commit hashes, and
a recommendation among roadmap feature (citing readings), no action, or more
exploration. Every cited number comes from a compact summary stored beside it
in `docs/strategy/evolvability-exploration-2026-10/` (within the tracked-size
limit, with provenance and content hashes). Richer evidence worth keeping
moves to `.bench-artifacts/lab/exploration/` in the main checkout, with the
note's paths updated, because removing the worktree deletes its ignored
files.

Start closing by turn 130, in this order:

1. Commit the final note and summaries on the branch; copy kept evidence to
   the main checkout's `.bench-artifacts/`.
2. `ExitWorktree` with keep.
3. Recheck `main`: if another session advanced or dirtied it since launch,
   follow the workflow's stop-and-report rule.
4. Cherry-pick the `docs:` commits and every `lab:` commit that builds
   without `proto:` changes; leave the rest on the branch.
5. Run `make check` if a `lab:` commit landed, else `make check-docs`.
6. Record the branch name from `git worktree list`, remove the worktree, and
   keep the branch.

## Goal command

Open a new session in the main checkout on a clean, current `main`: Opus 5.5,
effort `medium`, auto mode. Paste:

```
/goal An evolvability exploration run is complete on main. Read docs/strategy/evolvability-exploration-plan-2026-10-01.md first and follow it exactly; it is the run's contract, and the T22 exploration contract in docs/roadmaps/t22-capability-assays-and-evolvability-lab.md governs anything it does not cover. Confirm you are in the main checkout on a clean main, record the launch revision, create the worktree with EnterWorktree named evolvability-exploration, and read every starting-evidence document in full. Run research, predeclare, build, run, read, record cycles through the plan's stages without pausing to ask, except where the plan says to stop. Every arm must pass the plan's five not-forcing checks, matched controls and claim rules; the frontier table stays descriptive. Prototypes stay on the experiment branch as default-off proto commits classified policy-deviation and never merge. Commit the ledger and results note every cycle. Run the Codex reviews through codex exec as the plan says and record each finding's disposition. Begin closing by turn 130 in the plan's close order. Done means all of these are shown in this conversation, with command output and the note's ledger, frontier table and recommendation quoted, not only file paths: docs/strategy/evolvability-exploration-2026-10.md is on main with at least six completed (not incomplete) experiments, the frontier table, review dispositions and a recommendation; make check or make check-docs exited 0 on main; git worktree list no longer lists the worktree and git branch still lists its branch; git status on main is clean. If a concrete blocker stops the run, record it in the results note, land the note as above, report it, and stop. Stop after 150 turns.
```
