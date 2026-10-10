# Vote-margin read: observation plan for the executed-but-silent reads (2026-10-09)

Date: 2026-10-09. Status: plan, not yet run; revised the same day after a
Codex adversarial review (Section "Review dispositions"). Question raised by
the user after the
[action-choice pressure note](action-choice-pressure-research-2026-10-09.md):
the focus is why solutions do not evolve. The one unowned lead in the record
is the diagnosis's "reads execute but never decide" finding, which the
follow-up's N3 row narrowed to three observation questions it could not
answer with the instrument it had. This plan answers those three questions
with instrument work only, no mechanism prototype, **on current production**.
It runs under the
[T22 exploration contract](../roadmaps/t22-capability-assays-and-evolvability-lab.md),
in the form of the [deletion-rate plan](deletion-rate-experiment-plan-2026-10-09.md).

## Problem and constraints

- **Problem.** On Confluence at 50,000 ticks, 34 of 35 executed target-family
  reads alter no vote on the observation panels (zero delta exactly), the
  few that move a vote raise a sink of a kind other than the committing one
  without any commit changing, and the two reads the funnel calls causal do
  so with unchanged votes ([follow-up](evolvability-diagnosis-run-2026-10-followup.md),
  N3 row and plan forward Section 3). The follow-up refuted dilution and saw
  no cancellation, then named three questions its stored statistics cannot
  settle:
  1. **Exposure against downstream silence.** A zero delta means either the
     panels never presented the channel nonzero where its consumer ran, or
     the consumer's output never reaches a decision. The `neighborhood-v1`
     battery presents zero for every target-family channel in all 80
     scenarios; only the 184 extension executions can register an ablation,
     and exposure when a consumer executes was not recorded.
  2. **Incumbent masking at the kind level.** The vote-moving reads move
     `StealEnergy(3)` and `Move(6)` sinks past their within-kind margins and,
     on some passes, past the baseline decision margin, with no commit
     changing. The stored statistics do not pair each delta with its pass's
     committing kind and effective votes, so masking stays a hypothesis.
  3. **The action-parameter route.** The two causal reads at 20,000 change
     complete action queues with zero vote delta, equal hop counts and equal
     pass-end reasons; the census did not record which decoded parameter
     field differed.
  Plus one aggregation gap: per-channel `ratio_median` is already a median of
  per-pass ratios, but the pooled median across channels' passes that the N3
  row predicted was never computed.
- **Why current production, not a replay of N3.** N3 launched from `main`
  `d4bc4a98`. Since then T03.F12 (gestation) changed birth timing and T20.F16
  (born-connected `Add`) added mutation draws, and N3's cohort was selected
  from the live population and never saved. Seed 33 on today's `main` is a
  different world. N3's readings are this plan's prior, not an identity
  target; the question is what blocks evolution now.
- **Constraints.**
  - Observation only: `instr:` commits on a branch, taken as file state from
    `worktree-output-wiring` (the census probe with `vote_deltas`,
    `sink_use`, the E6 block). No `proto:` arm. Policy unchanged, so the
    unchanged-policy requirement is an identity gate between two builds, not
    a control arm.
  - Nothing reaches `main` but the note.
  - The not-forcing checks apply to any mechanism the readings later
    suggest, not to this plan (measurement is exempt from the natural-analog
    rule, `docs/roadmap.md`).
  - One heavy job at a time on the host. The deletion-rate run was stopped
    on 2026-10-09, so the host is free unless another session is running.
  - No Fable in any role (section "Roles").
- **Decision criteria.**
  - For each executed target channel: was each of its consumers ever exposed
    to a nonzero value when it executed, and where do those consumers'
    outputs land locally?
  - For each vote-moving channel: did the read raise a kind that still lost
    (masked), lower a kind that was losing (suppressing), or neither?
  - For each action-changing channel with no vote change: which decoded
    parameter field carries the effect, and does the committed kind decode
    it?
  - Which track owns each class, so the stall is routed with counts, and
    which classes stay unresolved for T22 before any mechanism is proposed.

## Local evidence (inspected 2026-10-09)

**What the branch-only census already records** (`vote_delta.rs` on
`worktree-output-wiring`): per (parent, channel) `DeltaStats` with pass
counts, divergent and unmatched passes, `scenes_actions_differ`, max |Δ| and
its sink, signed sums per sink, the most-moved sink and kind, the
within-kind margin median and minimum, `ratio_median` and `ratio_max`,
`delta_reaches_margin_passes`, the decision margin median and minimum,
no-commit and guarded passes, and `committed_differs_passes`; per family the
additive reconstruction. Every delta is a raw-sink difference, **ablated
minus baseline**, and a total intervention effect (dispatch and earlier
commits included). Candidate channels are discovered from the panels
(`scenes.observe`) before ablation, and family membership derives from the
candidates. The consumer inventory stores target and liveness only. What it
does not record:

| Gap | What is missing | Where it would live |
| --- | --- | --- |
| 1 | per (parent, consumer, panel set): executions of that consumer, and executions with the channel's value nonzero when it ran; consumer identity (Graph compute node or VM node and program counter) kept, not deduplicated | a `consumer_exposure` list per channel; channel summaries derived from it |
| 1 | per Graph consumer: a **local endpoint annotation**, the sink kinds reachable from the channel's leaf within that node (`ActionVote`, `ActionParam`, or only `CustomOutput`, `WriteSlot`, `RouterGate`); cross-node, memory and control routes are not followed and stay unresolved; VM consumers get no static annotation (`dynamic`) | `consumers.rs` |
| 2 | per delta pass, in both arms: each kind's within-kind maximum, effective vote (maximum minus the kind's bar), the selected and committed kind, guard status; the moved sink and its kind | a per-pass `vote-delta-v2` record; the existing stats stay as they are |
| 3 | per execution whose committed queue differs: which decoded action-parameter fields differ, and whether the committed sink's kind decodes that field (`decode_commit`: `Eat` reads `EatFoodType`, `Reproduce` `ReproduceTransferFraction`, `StealEnergy` `StealEnergyAmount`, `Move` nothing) | `param_fields_differ`, `param_field_consumed` |
| 4 | the pooled median of per-pass |Δ| / within-kind margin across channels | the per-pass record above |

**Sign convention for this plan.** The census stores ablated minus baseline.
The v2 readings define the **read effect** as baseline minus ablated, per
raw sink and per kind's effective vote, so "the read raises a kind" means
the kind's effective vote is higher with the read than without it.

**The panels.** The census's 264 executions per parent: the 80
`neighborhood-v1` battery scenarios (zero on every target family) and 184
extension executions (32 recorded contexts at the checkpoint, their 4
recorded sequences of 32 ticks, 24 authored contexts over eight channel
groups). The authored groups' coverage of the target families was not read
in N3.

**Cost of the existing census.** N3 on Confluence to 50,000: 15,547.6 s of
simulation (15,611 s awake), censuses 7.2 s, vote-delta blocks 0.11 and
0.51 s. E2's run took 15,108 s. Today's `main` may differ (gestation slows
births; born-connected genomes may be larger), so V0 times it.

## Readings (predeclared)

Per executed target channel, one class, assigned in this order. The channel
population is the original panels' candidate set; the exposure contexts of
option C are read in their own block and never change it.

| # | Class | Rule | Routed to |
| --- | --- | --- | --- |
| 1 | `vote-causal` | the committed action queue differs in some execution and some aligned pass has a nonzero raw-sink read effect | the funnel's existing `causal` |
| 2 | `param-causal` | the committed queue differs, the raw-sink read effect is zero on every aligned pass, and `param_fields_differ` is nonempty with the field consumed by the committed kind | T11.F27 action-parameter surface: a second route to causality the funnel does not name |
| 3 | `action-unresolved` | the committed queue differs and neither rule above holds (empty or unconsumed parameter diff; the change sits in divergent or unmatched passes) | T22 instrument |
| — | *every class below requires `scenes_actions_differ` = 0* | | |
| 4 | `unexposed` | no consumer of the channel executed with the channel nonzero on the original panels | T22 instrument (the funnel's `causal` and retention stages read blind for this channel) and the world-exposure question for T20 |
| 5 | `vm-unresolved` | zero read effect, and at least one exposed consumer is a VM consumer | T22 instrument (VM read-to-write dependency attribution) |
| 6 | `silent-local-decision` | zero read effect; every exposed consumer is Graph and its local endpoints include `ActionVote` or `ActionParam` | T22 first (what downstream of the endpoint kills it: a zero weight, saturation, an unconsumed parameter), then T19/T11 |
| 7 | `silent-local-intermediate` | zero read effect; every exposed consumer is Graph and its local endpoints are only bus, memory or gate sinks | T22 first (cross-node path tracing), then T20 |
| 8 | `silent-mixed` | zero read effect; exposed Graph consumers with both endpoint kinds | reported with the split |
| 9 | `masked` | nonzero read effect; on the majority of delta passes that commit in both arms, the read raises the effective vote of a kind that is not the baseline committing kind | T19 (the pass-end rule and the incumbent), with the shortfall (the baseline gap to the winner) per pass |
| 10 | `suppressing` | nonzero read effect; on that majority the read lowers the effective vote of a kind that is not the committing kind in either arm | reported: inert suppression of a non-winner |
| 11 | `moving-residual` | nonzero read effect and none of the above (a raw sink moves while its kind's effective vote does not, mixed signs, or no-commit passes dominate) | reported, neutral |

No-commit, guarded and divergent passes are counted apart on every channel.
The family-level additive reconstruction and the pooled per-pass median are
reported beside, with the N3 labels (dilution, cancellation, subadditive,
neither, mixed) computed on the pooled statistic.

**Predictions, fixed before the run.** N3's pattern on the old engine is the
prior; each prediction fails as stated:
1. At the 50,000 checkpoint, at least half of the channels in classes 4 to 8
   are `unexposed` or `silent-local-intermediate`. Fails if fewer than half.
2. Among the vote-moving channels (classes 9 to 11) at each checkpoint,
   `masked` is a minority. Fails if `masked` is half or more. (N3's
   positive sink shares show sink suppression; they do not prove a kind was
   suppressed, so this is a prior, not a reading.)
3. Every action-changing channel is `param-causal` with exactly one
   differing field. Fails on any `vote-causal` or `action-unresolved`
   channel, or any `param-causal` channel with two or more fields.
If instead most of classes 4 to 8 are `silent-local-decision`, the stall
sits downstream of a decision endpoint and the routing changes accordingly.

## Options

| Option | What | For | Against |
| --- | --- | --- | --- |
| **A. Extend the census (`vote-delta-v2`) and run it on current production** | The four gaps as `instr:` on the output-wiring file state; Confluence seed 33 to 50,000 on today's `main`, checkpoints 20,000 and 50,000 | Answers the three questions on the substrate that matters now; one heavy run | A new baseline; N3's channel-level predictions become a prior |
| B. Replay N3 exactly on `d4bc4a98` | A branch on the old engine with the v2 instrument | Same parents, exact identity against N3's stored blocks | Answers the question for an engine that no longer exists; keeps a historical engine alive |
| C. Authored exposure contexts | Contexts that set each target family nonzero, with their own candidate discovery and their own `exposure_contexts` block | Makes `unexposed` a direct reading per consumer, separately from the original panels | A battery change; a consumer that does not execute in the altered context is `coverage-unresolved`, not exposed |
| D. Canyon beside A | The same census on Canyon seeds 1022 and 2022 to 20,000 | A second substrate | More runs; the deletion-rate run's `default` artifacts predate T11.F29's production change and are not reused |
| E. Record and stop | Write up N3's three questions as open | Zero cost | The lead stays unowned |

## Recommendation: A with C, D only if the host is free

A answers the questions where they matter. C is cheap if it is kept apart:
its own candidate discovery, its own block, and no effect on the original
candidate set, family membership or legacy aggregation. Predictions 1 to 3
apply to the original panels only; C's readings are reported beside.

### Gate V0.2: smoke and fixtures (first)

The follow-up's vote-delta smoke (`PETRI_E2_VOTE_DELTAS=1`, the 200 × 200
Canyon recipe, 400 founders, seed 33, checkpoints 300, 2,500 and 5,000) with
the v2 fields populated and the per-pass record written. Fixtures in
`make check` on the branch, each an authored genome on an authored context:
a `param-causal` read (a VM write to `EatFoodType` under an `Eat` commit), a
genuinely silent Graph read with a local vote endpoint at weight zero, a
`masked` read (raises a losing kind), a `suppressing` read (lowers a losing
kind), a two-consumer channel with one exposed dead-end consumer and one
unexposed voting consumer (must not class as `silent-local-decision`), and
an exposure-context consumer that does not execute (`coverage-unresolved`).

### Gate V0: identity between builds (second, short)

Two builds on today's `main` file state: the output-wiring census as it is
(v1) and the v2 census. Both on Confluence seed 33, full recipe, to 10,000
ticks with one checkpoint at 10,000 (extended to 20,000 if no sampled parent
carries an executed target channel at 10,000). Pass when:
- the simulation samples and the `input_use` and `e6` blocks hash-equal;
- a canonical whitelist projection of the v2 `vote_deltas` block (the v1
  schema's parent, channel, family and `DeltaStats` fields only) hash-equals
  the v1 block.
A mismatch stops the plan. The wall time of the v2 run sets the V1 budget.

### Row V1: one heavy run (third)

- **Design.** One execution of the v2 census: Confluence seed 33 on today's
  `main`, 50,000 ticks, checkpoints 20,000 and 50,000, eight threads, the
  original panels plus C's exposure contexts in their own block; awake kill
  at twice V0's projected time to 50,000, capped at 8 h. Optional D: Canyon
  seeds 1022 and 2022, 20,000 ticks, checkpoints 10,000 and 20,000, 3 h kill
  each.
- **Readings.** The class table per channel and checkpoint with counts; the
  per-consumer exposure and local endpoint for every channel; the per-pass
  effective-kind record summarized per channel; `param_fields_differ` and
  `param_field_consumed` per action-changing channel; the pooled per-pass
  median; the family labels; C's block beside, never merged.
- **Disposition.** Descriptive. The row succeeds when every executed target
  channel carries a class at every checkpoint reached and V0 passed. There
  is no validation threshold; the deliverable is the routed table and the
  three predictions' outcomes.
- **Stopping rule.** On a kill or an extinction, the checkpoints reached are
  kept, the row is `incomplete`, and the note closes on what was reached; no
  rerun unless the kill was a host failure, in which case one rerun under
  the same limit.
- **What each outcome means.**
  - Mostly `unexposed`: the funnel's `causal` stage has been reading blind
    for these families; T22 gets an instrument row (panel exposure per
    consumer) before any further `causal` reading is trusted, and the world
    exposure question goes to T20.
  - Mostly `silent-local-intermediate`: consumer outputs land on bus, memory
    or gates; T22 traces the cross-node path before T20 is handed a wiring
    question.
  - Mostly `silent-local-decision`: the read reaches a decision endpoint
    and dies downstream; T22 locates where (weight, saturation, unconsumed
    parameter) before T19/T11 gets an arithmetic question.
  - `masked` on the vote-moving channels: T19 receives the incumbent-masking
    hypothesis with per-pass shortfalls.
  - `param-causal` confirmed: the funnel gains a second causality route on
    the T11.F27 parameter surface, which vote-side readings have
    undercounted.

## Cost

- **Code.** The per-consumer exposure record, the local endpoint
  annotation, the per-pass effective-kind record, the parameter-field diff
  in the recording executor, C's contexts with their own discovery, and the
  six fixtures. About one session, no engine change.
- **Runs.** V0.2 minutes; V0 two runs to 10,000 (timed there; well under
  V1's cost); V1 one run of about 4.3 h awake if today's engine runs at
  N3's pace; D about 30 min.
- **Session.** One goal session of about 100 to 150 turns.

## Remaining uncertainty

- The local endpoint annotation stops at the node boundary. A
  `silent-local-intermediate` read may still decide through a later node;
  the class says where the trace stopped, not that no path exists.
- VM consumers get no static annotation; `vm-unresolved` is honest about
  that rather than forcing a class.
- A channel can be `unexposed` on the panels and exposed in the world. C
  settles the panel half only; the world half is a T20 reading.
- `masked` and `suppressing` are per-pass bookkeeping on effective kind
  votes, not causal claims about the pass-end rule.
- Today's Confluence may not carry executed target reads at 20,000 in the
  sampled cohort; V0's extension rule and the stopping rule cover that.

## Roles (this run's rule 8; no Fable)

Fable credits are exhausted (user, 2026-10-09). No Claude role in this run
is Fable. Rule 8 of the diagnosis plan says every role is Fable and that a
run stops rather than substitute Opus; that clause does not apply here, and
neither do the Fable roles in the output-wiring and born-connected run
notes. Nothing in them is a reason to stop or to wait for Fable.

- **Session.** Opus 5.5 (`claude-opus-5-5`) at effort `xhigh`. It is the
  implementer: it writes the instruments, runs the gates and rows, reads the
  results and writes the note. No subagent is needed; if one is spawned, it
  runs on Opus (`model: "opus"`), never Fable.
- **Advisor.** Opus, enabled with `/advisor opus` before the goal is pasted
  (the project settings default `advisorModel` to `fable`). Consulted through
  the `advisor` tool at every one of these points, each recorded with its
  dispositions in the note's Advice table:
  - before the first gate, on the approach to the whole run;
  - after each gate's or row's reading is drafted and before it is recorded;
  - whenever a gate fails, a row is `incomplete`, a result contradicts its
    prediction, or an approach has not converged after two attempts;
  - before the plan forward is drafted, and again after;
  - before closing.
  The advisor advises; it does not write code, run rows or edit the note. If
  the advisor tool is unavailable or returns a credit error, the run records
  it, sends the same brief to Codex as an advice job, and continues.
- **Reviews.** Codex `gpt-6.1-sol` at `high`, read-only, through
  `codex exec`: (a) the predeclaration before any heavy row runs; (b) the
  note's reading before closing. A Codex usage limit marks the review
  `pending`; the run continues, and the note recommends nothing until review
  (b) has completed.

## Review dispositions (Codex adversarial review, 2026-10-09)

| # | Finding | Disposition |
| --- | --- | --- |
| 1 | V0's identity against N3 conflicts with running on today's `main` (T03.F12, T20.F16 since `d4bc4a98`; N3's cohort never saved) | Adopted: current production with a new baseline; V0 is now identity between two builds; N3 is the prior (option B kept as the rejected alternative) |
| 2 | The ordered classes swallow `param-causal` | Adopted: action-changing classes first; every silent class requires no action change; `action-unresolved` added |
| 3 | Sink delta, kind delta, gap and masking conflated | Adopted: per-pass effective kind votes in both arms, an explicit sign convention, masked and suppressing on effective kinds, `moving-residual` neutral, prediction 2 fixed |
| 4 | Local sink endpoints cannot show "no decision path" | Adopted: renamed local endpoint annotation; intermediate and decision-endpoint silence both route to T22 before any mechanism question; parameter consumption recorded |
| 5 | A separate block does not protect candidate discovery; the stored summaries cannot hash the full old block | Adopted: C has its own discovery; V0 hashes a canonical whitelist projection between builds (N3's stored blocks are no longer the target) |
| 6 | Channel-level exposure cannot answer a consumer-level question | Adopted: exposure per identified consumer and panel set, joined to that consumer's endpoint; `silent-mixed`; `coverage-unresolved` for C |
| 7 | V0 before the run that supplies its evidence | Adopted: V0 is a short two-build identity run before V1; V1 is the one heavy run; stopping rule added |

## Goal command

Open a new session in the main checkout on a clean, current `main`, with no
other heavy job on the host. Set the model to Opus 5.5 at effort `xhigh`,
run `/advisor opus` (the project default is a Fable advisor, and Fable
credits are exhausted), and confirm the startup notice shows an Opus
advisor. Then paste:

```
/goal A vote-margin read run is complete on main. Read docs/strategy/vote-margin-read-experiment-plan-2026-10-09.md in full (its gaps table, sign convention, class table, predictions, gates V0.2 and V0, row V1, stopping rule and review dispositions are the contract's form), then docs/strategy/evolvability-diagnosis-run-2026-10-followup.md row N3 and plan forward Section 3, then docs/strategy/output-wiring-run-2026-10.md (gates, frozen-instrument pattern, Advice table). No Fable in any role: confirm the session is Opus 5.5 at effort xhigh with an Opus advisor and Codex for reviews, as the plan's Roles section states (it replaces rule 8; the Fable roles in the diagnosis plan and the output-wiring and born-connected notes do not apply, and are never a reason to stop). Confirm a clean main and no other heavy job on the host, and create the worktree with EnterWorktree named vote-margin. Take the census probe, vote_delta, sink_use and the diagnosis probe as file state from branch worktree-output-wiring (instr and lab commits only, no proto). Build vote-delta-v2 as instr exactly as the gaps table states, with option C's exposure contexts carrying their own candidate discovery and their own block, and the six fixtures of gate V0.2. Predeclare and commit V0.2, V0 and V1 before any of them runs: V0 two builds (v1 and v2) on Confluence seed 33 to 10,000 with the extension rule; V1 one v2 run on Confluence seed 33 to 50,000, checkpoints 20,000 and 50,000, eight threads, awake kill at twice V0's projection capped at 8 h; Canyon seeds 1022 and 2022 to 20,000 only if the host is free. Run V0.2, then V0, then V1. The class table and predictions are the plan's as written; the row is descriptive. Consult the Opus advisor at every point the Roles section lists and record it in the Advice table; send the predeclaration to Codex before V1 runs and the note's reading before closing. Write docs/strategy/vote-margin-read-run-2026-10.md with the gates, the row, the class counts per channel and checkpoint, the three predictions' outcomes, the Advice table and a plan forward that routes each class to its track with counts. Land docs only; keep instr on the branch. Done means the note is on main, make check-docs exited 0 on main, the worktree removed and its branch kept, main clean. Stop after 150 turns.
```
