# Vote-margin read: observation plan for the executed-but-silent reads (2026-10-09)

Date: 2026-10-09. Status: plan, not yet run. Question raised by the user
after the [action-choice pressure note](action-choice-pressure-research-2026-10-09.md):
the focus is why solutions do not evolve. The one unowned lead in the record
is the diagnosis's "reads execute but never decide" finding, which the
follow-up's N3 row narrowed to three observation questions it could not
answer with the instrument it had. This plan answers those three questions
with instrument work only, no mechanism prototype. It runs under the
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
     the consumer's output never reaches a decision surface. The
     `neighborhood-v1` battery presents zero for every target-family channel
     in all 80 scenarios; only the 184 extension executions can register an
     ablation, and exposure when a consumer executes was not recorded.
  2. **Incumbent masking at the kind level.** The vote-moving reads raise
     `StealEnergy(3)` and `Move(6)` past their within-kind margins and, on
     some passes, past the baseline decision margin, with no commit changing.
     The stored statistics do not pair each delta with its pass's winning
     kind and gap, so masking stays a hypothesis.
  3. **The action-parameter route.** The two causal reads at 20,000 change
     complete action queues with zero vote delta, equal hop counts and equal
     pass-end reasons; the census did not record which decoded parameter
     field differed.
  Plus one aggregation deviation: the predeclared statistic (median over
  margined passes of |Δ| / within-kind margin) was never computed because
  per-pass ratios were not stored.
- **Constraints.**
  - Observation only: `instr:` commits on a branch, taken as file state from
    `worktree-output-wiring` (the census probe with `vote_deltas`,
    `sink_use`, the E6 block). No `proto:` arm. Policy unchanged, so the
    unchanged-policy requirement is an identity gate, not a control arm.
  - Nothing reaches `main` but the note (and any reading worth keeping inside
    `crates/v3-lab`, which this plan does not expect).
  - The not-forcing checks apply to any mechanism the readings later
    suggest, not to this plan.
  - One heavy job at a time on the host; the deletion-rate run holds that
    slot now, so this plan's census waits for it.
- **Decision criteria.**
  - For each executed target channel: was it ever exposed where its consumer
    ran, and does its consumer reach a decision surface?
  - For each vote-moving channel: is the moved kind losing to the committing
    kind by more than the delta (masked), or not?
  - For each causal-with-zero-delta channel: which action-parameter field
    carries the effect?
  - Which track owns each class, so the stall is routed by rung with counts,
    not re-diagnosed.

## Local evidence (inspected 2026-10-09)

**What the branch-only census already records** (`vote_delta.rs` on
`worktree-output-wiring`): per (parent, channel) `DeltaStats` with pass
counts, divergent and unmatched passes, `scenes_actions_differ`, max |Δ| and
its sink, signed sums per sink, the most-moved sink and kind, the
within-kind margin median and minimum, `ratio_median` and `ratio_max`,
`delta_reaches_margin_passes`, the decision margin median and minimum,
no-commit and guarded passes, and `committed_differs_passes`; per family the
additive reconstruction. What it does not record, each one a gap above:

| Gap | What is missing | Where it would live |
| --- | --- | --- |
| 1 | per (parent, channel): the number of panel executions in which the consumer executed with the channel's value nonzero | `ChannelVoteDelta.exposed_executions`, `consumer_executions` |
| 1 | per consumer: a static annotation of its downstream path (reaches an `ActionVote` sink, an `ActionParam` sink, only bus, memory or gate sinks, or nothing) | `consumers.rs`, read from the Graph edge table; VM consumers by their executed trace on the panels (the VM's push and vote writes) |
| 2 | per delta pass: the committing kind, its effective vote, the moved sink's kind and effective vote, the gap, whether the delta exceeds the gap, and the sign of the baseline read's effect on the moved kind (the ablation's positive share 1.0 on N3's four food-summary channels means the read *lowers* `StealEnergy(3)`) | a per-pass `vote-delta-v2` record, aggregated into the existing stats plus `masked_passes` and `suppressing_passes` counts |
| 3 | per execution whose committed queue differs: which decoded action-parameter fields differ, by field | `DeltaStats.param_fields_differ: BTreeMap<String, u32>` |
| 4 | per-pass ratios, so the predeclared median is computable | the per-pass record above |

**The panels.** N3's reading rests on the 184 extension executions: 32
recorded contexts at the checkpoint, their 4 recorded sequences of 32 ticks,
and 24 authored contexts over eight channel groups. The authored groups'
coverage of the target families was not read. E6 at 20,000 said food type 0
was visible to 94.2% of living creatures, so most recorded contexts present
a nonzero `AreaFoodSummary:0` `total_ratio`; `AreaFoodSummary:1` and
`NeighborBarrierRing` exposure is unknown (E1: the barrier edge exposed to
5.5 to 6.0% of sampled creatures).

**The parents.** N3's 20-parent selected cohorts at 20,000 and 50,000 on
Confluence seed 33 are reproducible: G2 proved the census trajectory
hash-equal to E2's. W1's `default` arm supplies the Canyon cohorts at
10,000 and 20,000 on seeds 1022 and 2022.

**Cost of the existing census.** Confluence to 50,000: 15,547 s of
simulation, censuses 7.2 s, vote-delta blocks 0.11 and 0.51 s. Canyon to
20,000: 11 to 15 min.

## Readings (predeclared)

Per executed target channel, one class from the readings, assigned in this
order:

| Class | Rule | Routed to |
| --- | --- | --- |
| `unexposed` | `exposed_executions` = 0 | T22 instrument (panel exposure; the funnel's `causal` and retention stages were reading blind for this channel), and the world-exposure question for T20 |
| `silent-no-path` | exposed > 0, zero delta, consumer path reaches no decision surface | T20/T11: the consumer's output lands on bus, memory or gate sinks only, the output-wiring question in a new form |
| `silent-with-path` | exposed > 0, zero delta, path reaches a vote or parameter sink | T19/T11 vote arithmetic: saturation, a zero weight, or an unvoted downstream node; the census records the path's last node kind |
| `path-unresolved` | exposed > 0, zero delta, VM consumer whose panel trace never reaches a decision surface in an exposed context (the annotation is dynamic for VM consumers, so the two silent classes cannot be told apart) | reported apart; not forced into either silent class |
| `masked` | delta passes > 0, no action change, and on the majority of delta passes the read *raises* a trailing kind by less than its gap to the committing kind | T19 (the pass-end rule and the incumbent) as a recorded hypothesis with counts; a mechanism question for a later row |
| `suppressing-loser` | delta passes > 0, no action change, and on the majority of delta passes the read *lowers* a kind that was already trailing | reported; inert suppression of a non-winner, not masking |
| `moving-unmasked` | delta passes > 0, no action change, not masked | reported; the delta is real but below the decision margin |
| `param-causal` | action change with zero vote delta; `param_fields_differ` nonempty | T11.F27 / T19 action-parameter surface: a second route to causality the funnel does not name |
| `vote-causal` | action change with a vote delta | the funnel's existing `causal` |

Family-level and the predeclared per-pass median are reported beside, with
the N3 labels (dilution, cancellation, subadditive, neither, mixed) computed
on the correct statistic this time.

**Predictions, stated to be falsified.** From N3's reading: at 50,000 at
least half of the 34 zero-delta channels are `unexposed` or
`silent-no-path`; the four vote-moving food-summary channels at 20,000 are
`suppressing-loser` (N3: the ablation raises `StealEnergy(3)`, so the read
lowers it) and the barrier-ring channel at 50,000 (the ablation raises
`Move(6)`) is `suppressing-loser` as well unless its per-pass sign says
otherwise, so `masked` is predicted for none of the five; the two causal channels at
20,000 are `param-causal` with a single differing field each. If instead
most zero-delta channels are `silent-with-path`, the stall is in the vote
arithmetic and the routing changes accordingly.

## Options

| Option | What | For | Against |
| --- | --- | --- | --- |
| **A. Extend the census (`vote-delta-v2`) and rerun on N3's parents** | The four gaps above as `instr:` on the output-wiring file state; Confluence seed 33 to 50,000 with checkpoints 20,000 and 50,000 | Resolves every N3 channel on the same parents; identity gate against N3's blocks is exact | 4.3 h awake; one substrate |
| B. A plus Canyon 20,000 on seeds 1022 and 2022 | The same census on W1's `default` cohorts | A second substrate at a quarter of the cost; comparable with the deletion run's seeds | More runs on a busy host |
| C. Authored exposure contexts | Add panel contexts that set each target family nonzero, in their own `exposure_contexts` stats block that the N3-equivalent stats never fold in | Settles `unexposed` for every channel directly; the N3-comparable numbers stay clean | A battery change; if its executions were folded into the existing stats, every old value (delta passes, ratio medians) would move and gate V0 could not hold |
| D. Record and stop | Write up N3's three questions as open | Zero cost | The lead stays unowned |

## Recommendation: A, with C as a labelled extension, B if the host is free

A answers the three questions on the parents where they were asked. C is
cheap and makes `unexposed` a direct reading rather than an inference, as
long as its executions are aggregated in their own `exposure_contexts`
block: the existing stats over the original 264 executions are then
unchanged in value, which is what gate V0 checks. B overlaps the
deletion-rate run, whose S1 `default` arm already runs the census with the
old vote-delta fields on Canyon seeds 1022 to 4022 at 10,000 and 20,000; B
adds only the v2 fields and reads the old ones from that run's artifacts if
they land first, otherwise it is dropped.

### Gate V0: identity

The extended census on Confluence seed 33: the `input_use`, `e6` and the
existing `vote_deltas` fields hash-equal to N3's at 20,000 and 50,000 in
value, which holds only because the new fields are additive and C's
executions live in their own block. On Canyon, the W1 `default` identity
summaries. A mismatch stops the row.

### Gate V0.2: smoke

The vote-delta smoke of the follow-up (`PETRI_E2_VOTE_DELTAS=1`, the 200 ×
200 Canyon recipe, 400 founders, seed 33, checkpoints 300, 2,500 and 5,000)
with the new fields populated and the per-pass record written; the
`masked_passes` and `exposed_executions` fixtures (authored contexts with a
known nonzero channel and a known committing kind) pass in `make check` on
the branch.

### Row V1

- **Design.** Confluence seed 33, 50,000 ticks, checkpoints 20,000 and
  50,000, eight threads, the extended census; 6 h awake kill as N3
  (predeclared because N3 took 15,611 s awake). Optional B: Canyon seeds
  1022 and 2022, 20,000 ticks, checkpoints 10,000 and 20,000, 3 h kill each.
- **Readings.** The class table above per channel, per checkpoint, with
  counts; the per-pass median; the family labels; `param_fields_differ` per
  causal channel; the consumer path annotation per channel (`static` for
  Graph consumers, `dynamic` for VM consumers); C's `exposure_contexts`
  block beside, never merged.
- **Disposition.** Descriptive. The row succeeds when every executed target
  channel carries a class at both checkpoints and V0 passed. There is no
  validation threshold, because nothing is being validated; the deliverable
  is the routed table.
- **What each outcome means.**
  - Mostly `unexposed`: the funnel's `causal` stage has been reading blind
    for these families; T22 gets an instrument row (panel exposure per
    channel) before any further `causal` reading is trusted, and the world
    exposure question (does the live world present these values where
    consumers run) goes to T20.
  - Mostly `silent-no-path`: the output-wiring question returns in its
    narrow form (consumers whose outputs land only on bus, memory or gate
    sinks); T20 owns it.
  - Mostly `silent-with-path`: a vote-arithmetic question for T19/T11, read
    from the recorded last-node kinds.
  - `masked` on the vote-moving channels: T19 receives the incumbent-masking
    hypothesis with per-pass counts; a mechanism row (the per-kind bar) is
    then a design question with evidence, not a guess.
  - `param-causal` confirmed: the funnel gains a second causality route; the
    T11.F27 action-parameter surface is a place where sensor reads already
    decide, which the vote-side readings have undercounted.

## Cost

- **Code.** Four additive fields and a per-pass record in `vote_delta.rs`;
  the consumer path annotation in `consumers.rs`; the parameter-field diff
  in the recording executor; authored exposure contexts (C). About one
  session, no engine change.
- **Runs.** V0.2 minutes; V1 about 4.3 h awake on Confluence, after the
  deletion-rate run releases the host; B adds about 30 min.
- **Session.** One goal session of about 100 to 150 turns.

## Remaining uncertainty

- The static path annotation for VM consumers is approximate: a VM read's
  route to a vote is dynamic. The plan records the executed trace on the
  panels and labels the annotation `dynamic` for VM consumers.
- A channel can be `unexposed` on the panels and exposed in the world. C
  settles the panel half only; the world half is a T20 reading.
- `masked` and `suppressing-loser` are per-pass bookkeeping classes, not
  causal claims about the pass-end rule; they say which way the read moved
  a trailing kind and by how much against its gap.
- The host is shared; the 6 h kill and the one-job rule are the only timing
  guards.

## Goal command

Open a new session in the main checkout on a clean `main`, model Opus 5.5,
with the advisor enabled on Opus 5.5 and Codex for reviews (the user's
choice on 2026-10-09), after the deletion-rate run has released the host.
Paste:

```
/goal A vote-margin read run is complete on main. Read docs/strategy/vote-margin-read-experiment-plan-2026-10-09.md in full (its gaps table, class table, gates V0 and V0.2 and row V1 are the contract's form), then docs/strategy/evolvability-diagnosis-run-2026-10-followup.md row N3 and plan forward Section 3, then docs/strategy/output-wiring-run-2026-10.md (gates, frozen-instrument pattern, Advice table). Confirm Opus 5.5 with the advisor enabled on Opus 5.5 and Codex for reviews, a clean main, no other heavy job on the host, and create the worktree with EnterWorktree named vote-margin. Take the census probe, vote_delta, sink_use and the diagnosis probe as file state from branch worktree-output-wiring (instr and lab commits only, no proto). Build vote-delta-v2 as instr: exposed_executions and consumer_executions per channel, the consumer path annotation (vote, param, no-decision-surface, none; dynamic for VM consumers), the per-pass record with committing kind, effective votes, gap and masked flag, param_fields_differ per causal channel, and the authored exposure contexts in their own exposure_contexts block that the existing stats never fold in. Predeclare and commit V0, V0.2 and V1 before any of them runs: Confluence seed 33, 50,000 ticks, checkpoints 20,000 and 50,000, eight threads, 6 h awake kill; Canyon seeds 1022 and 2022 at 20,000 only if the host is free. Run V0.2, then V0 against N3's blocks, then V1. The class table is the plan's table as written; the row is descriptive and has no validation threshold. Consult the advisor at every rule 8 point and record it; send the predeclaration to Codex before the row runs and the note's reading before closing. Write docs/strategy/vote-margin-read-run-2026-10.md with the gates, the row, the class counts per channel and checkpoint, the Advice table and a plan forward that routes each class to its track with counts. Land docs only; keep instr on the branch. Done means the note is on main, make check-docs exited 0 on main, the worktree removed and its branch kept, main clean. Stop after 150 turns.
```
