# Evolvability exploration, run 2: plan

Date: 2026-10-02. Source: `6d2efeb5`. Status: plan for the continuation of
the [first exploration run](evolvability-exploration-2026-10.md), under the
[T22 exploration contract](../roadmaps/t22-capability-assays-and-evolvability-lab.md).
No experiment run yet; no production source, default or founder changed.
Reviewed by Codex `gpt-6.1-sol` `high` on 2026-10-01 (round 1 `not-ready`,
5 blocking and 4 advisory, all applied below; round 2 confirmed 7 and
reopened outcome B's rung bound and the shortcut gate; round 3 confirmed
both, verdict `ready`).

## Why a second run

Run 1 ([plan](evolvability-exploration-plan-2026-10-01.md),
[results](evolvability-exploration-2026-10.md)) closed after eight
diagnostic experiments with "more exploration" and no mechanism evidence.
The user expected the run to keep researching, testing and analysing until
it had concrete evidence for or against brain-structure and mutation
changes. Run 1 stopped because its done criterion accepted any
recommendation after six experiments, its 150-turn cap left about 80 turns
for experiments after a full re-read of the starting evidence, and it never
tested a mechanism that could matter for a two-node founder. Its main
finding also undercuts the food-seeking assay: native lineages climb in one
or two edits to a plateau that appears to abandon the reproduction drive
(refused attempts cost a foraging tick and the failed-action penalty in the
lab), and no sampled child of that plateau improved.

Run 2 keeps run 1's question and rules, repairs the instrument first, then
tests mechanisms on it until it has a supported answer.

## Question

Unchanged from run 1: why do newly added nodes and sensor reads rarely become
useful under Petri's own variation, and which general changes to the
representation, operators, costs or selection setting let them become useful?

## Rules

Run 1's plan governs everything this section does not change: where the run
happens, what may change (`lab:`, `docs:`, `proto:`, `instr:` commits), the
new-instrument rules, baseline identity, branch verification, telemetry, the
five not-forcing checks, matched controls, holdout, the claim rules, the
descriptive frontier table, the natural-world check, the cycle, the Codex
review and advice rules (Codex `gpt-6.1-sol` `high`, read-only `codex exec`;
no Fable advisor, agent or reviewer), the stop-and-ask list, and the close
order. Changes for run 2:

1. **Evidence outcome.** The run ends with exactly one of:
   - **A. Feature evidence.** At least one candidate reached run 1's
     *promising* claim: a recruitment claim (the elite's acquired node or
     read identified, its ablation removes the gain, incumbent competence
     preserved, discovery reported apart from retention) that holds on a
     sealed holdout bank, on both repaired assays, with fresh replicates,
     followed by the natural-world check and a completed Codex review of the
     recommendation. The note then recommends the roadmap feature.
   - **B. Supported negative.** The minimum effect of interest is
     predeclared now: a paired reach gain of 0.25 (4 of 16 replicates) over
     the reference, or the arm's verdict leaving the reference's stall rung
     in at least 4 of 16 replicates. For every arm in phase 3 on each
     repaired assay: 16 paired replicates completed; the arm's and the
     reference's stall rungs read with adequacy met in every replicate the
     rung contrast uses; the exact one-sided 95 % upper bound on the paired
     reach gain below 0.25 (with 0 arm-only reaches in 16 pairs, the bound
     is about 0.17); and the exact one-sided 95 % upper bound on the share of
     pairs in which the arm leaves the reference's stall rung (and the
     reference does not) also below 0.25. Both declared effects must be
     excluded. An arm
     that cannot meet this is `inconclusive`, and any `inconclusive` arm
     rules B out. The note relates arm results to the phase 2 census
     descriptively; it does not claim the census caused them.
   - **C. Blocker.** A concrete blocker, recorded and landed as run 1's plan
     describes. Exhaustion counts as a blocker only when every phase 3 arm
     has either run or been shown inapplicable with the reason recorded.

   Separately, the note records an execution status: `complete`,
   `incomplete` (turn cap reached before an outcome) or `unreviewed` (Codex
   unavailable at close). "More exploration" is never an outcome. An
   `incomplete` note ends with a **resume handoff**: the next arm or step,
   its exact commands, the branch and commit to resume from, and which
   holdout banks are still sealed.
2. **Budget.** Cap 400 turns; start closing by turn 360. Pilot one complete
   16-replicate configuration of each phase 3 arm kind first and set the
   generation horizon from it; a 16-replicate arm may run as two
   predeclared 8-replicate seed batches (seeds 1 and 2) when one invocation
   would exceed the 15-minute kill. Codex reviews at the end of each phase
   and before any recommendation; the every-fourth-cycle review may be
   merged into the next phase review.
3. **Starting evidence.** Read in full: run 1's results note, its compact
   summaries under `docs/strategy/evolvability-exploration-2026-10/`, this
   plan, run 1's plan and the T22 track. Run 1's other starting-evidence
   documents are consulted on demand, preferably through an `Explore` or
   `general-purpose` subagent run with `model: "opus"` or `"sonnet"` (never
   Fable) that returns only the passages needed.
4. **Replicates.** Phase 3 and phase 4 contrasts use 16 paired replicates per
   arm; 8 is allowed only for phase 1 calibration and pilots.
5. **Holdout banks.** Confirmation banks are reserved now and opened in
   order, one bank per frozen confirmation batch: bank 1 = seeds 101 (food)
   and 102 (barrier), sealed and unread since run 1; bank 2 = 111 and 112;
   bank 3 = 121 and 122. A batch freezes every candidate it will confirm,
   with its ledger row committed, before its bank is read. After a bank is
   read, any changed or combined candidate needs the next bank. Each new
   instrument version draws its own development and sealed acceptance sets
   under run 1's new-instrument rules; those are not confirmation banks.
6. **Host.** The host is a laptop that sleeps on battery. Wrap every heavy
   job in `caffeinate -i`. Run 1's `runlab.sh` kills on elapsed time, which
   counts sleep: replace that with a kill that also checks the run's own
   clock, and judge `incomplete` by `timing.wall_seconds` over 900 s.
7. **Diagnostic genomes.** Hand-edited genomes (phase 1's silencing and
   restoration variants, phase 2's witness path) are allowed as isolated
   diagnostics: they are evaluated on fixed scenes and never start a
   lineage, enter selection, seed an arm, serve as a comparator or count as
   discovery evidence. This narrows, for diagnostics only, run 1's rule that
   no authored wiring enters a genome.

Run 1's helpers live under `.bench-artifacts/lab/exploration/` in the main
checkout (`runlab.sh`, `codexrun.sh`, `compact.jq`, the baseline projections
and overlays); copy them into the worktree's `.bench-artifacts/`. Run 1's
probe is `crates/v3-lab/tests/exploration_probe.rs` (E3, E4, E7); extend it.
The worktree tool may branch from `origin/main`: confirm the worktree's
`HEAD` equals the launch revision and reset the fresh branch if it does not.
In the worktree, the session's shell guard refuses compound commands that
compute paths at run time (`$(...)`, heredocs mixed with other commands);
write scripts to files and run them.

## Phases

### 1. Repair the instrument (sterility shortcut)

Run 1 showed an association (E7: penalty charged 1,432 for the founder, 0.0
for the plateau; food eaten 283 against 447). Test it with diagnostic
genomes on run 1's 32-scene bank, as a 2 × 2 on the founder (reproduce
signal silenced or not × the plateau's upstream re-pointing applied or not)
plus the plateau with the readiness signal restored, and the same silencing
applied to three more of run 1's native and shuffled elites. Predeclared:
silencing is the **dominant explanation** when silencing alone recovers at
least 70 % of the founder-to-plateau bank gain and restoration removes at
least 70 % of it; it is a **contributor** when silencing, alone or through
its interaction with the re-pointing, raises any genome's bank score by more
than the margin the repaired version will use; otherwise there is **no
shortcut**. The note records which factor or interaction carries the gain.

When silencing is dominant or a contributor, build a new food-seeking instrument version
under the new-instrument rules with one behavioural requirement: abandoning
reproduction cannot raise the score. Validation: every silencing variant
above scores within the version's predeclared margin of its unsilenced
genome on the development set, and again on the sealed acceptance set
after the version is frozen; a version failing either is `uncalibrated`.
Candidate designs, none adopted until it passes that validation:

- reproduction accepted with its real cost as in the world, the offspring
  removed at birth. Silencing still saves the cost and the action, so this
  likely fails the requirement on its own; it needs the next design too;
- the score graded per foraging opportunity, so ticks spent on any
  reproduce attempt (refused or accepted) neither add nor subtract;
- the failed-action penalty for a refused reproduce attempt set to zero and
  the score graded per foraging opportunity.

A design that changes reproduction acceptance or charging amends T22.F01's
reproduction suppression for this branch: it is an `instr:` commit, keeps
the native mutation engine, lab selection, the focal creature's evaluation
start and lifetime and focal-only counters unchanged, and is recorded in
the note as that amendment. Repeat the diagnostics, and if needed the
repair, on `wall-v1`. Then rerun run 1's E3, E4 and E7 on the repaired food
version to see whether the plateau persists.

No phase 2 or phase 3 reading runs on a food or barrier instrument that a
contributing shortcut affects until a version passing the validation above
exists. Only when the diagnostics show no shortcut are the existing
instruments kept, with the factor that carries the plateau's gain recorded.

### 2. Path census

On the repaired instruments, describe what separates the founder from a
sensor-guided gain worth the reach threshold:

- the selected improvements in reference-arm lineages, each with its
  applied operators, its target nodes, whether a new or changed edge reads
  another node's output through the upstream baton or shared memory (a
  node-boundary crossing, H8), and its vote margin against the incumbent
  (H4);
- one witness path: a diagnostic genome sequence (rule 7) from the founder
  to a sensor-guided mover that reaches the threshold, decomposed into
  edits each of which a native operator can apply, with each intermediate
  scored as neutral, harmful or helpful. Its length is an upper bound on the
  shortest path, not a minimum, and it is never discovery evidence (H3).

The census picks the phase 3 prototype; it does not explain arm results
causally.

### 3. Mechanism arms

Every arm passes the five not-forcing checks, names its analog, runs on both
repaired assays with 16 paired replicates against the unchanged reference,
and carries matched controls when it changes evaluation semantics. Overlay
arms first, because they are cheap:

| Arm | Overlay or prototype | Analog | Prediction |
| --- | --- | --- | --- |
| M1 | `neutral_input_recruitment: SingleChannel` (T20.F04, opt-in, exists) | afferent growth: a new sensory synapse forms initially silent | one-event declare-and-connect shortens the path to a useful read |
| M2 | `neutral_input_recruitment: WholeFamily` | as M1, a whole receptive field | as M1, wider access per event |
| M3 | factorial: reference, M1, `structured_heritable_refinement: true` (T20.F05, opt-in, exists) and M1 + refinement, all in one invocation | access then repeated circuit motifs varying together | interaction hypothesis, predeclared: refinement helps only once access exists, so M1 + refinement exceeds M1 by more than refinement alone exceeds the reference; refinement keeps T20.F05's existing groups and draws no task-solving correspondence |
| M4 | `per_unit_rate: 0.02` | mutator background | more variation per birth crosses a valley run 1 could not |
| M5 | one prototype chosen by phase 2: a cross-node edge source reading another node's last output this tick, zero if that node has not run (H8, if boundary crossings dominate the census), or VM fresh reads drawn over every channel and declared reference (S1/M2, if VM tissue carries the incumbent path) | axon growth between regions / receptor diversity | the census's dominant barrier, removed by a general rule |

Copying a competent action module into new tissue (contextual duplication)
is not an arm here: run 1's not-forcing check 3 forbids copying a competent
incumbent, and it belongs to T13.F08. M5 is built only after phase 2, and a
second prototype only if the first is clearly negative. A prototype sits
behind a default-off config field, is classified `policy-deviation` by the
first `proto:` commit's classification change, passes `make check` on the
branch (except the known recipe digest pin), and runs `cargo test -p v3-core
--test viability` first when it touches founder behaviour or tick-loop
mechanics. M5 changes evaluation semantics, so its matched controls run.

### 4. Confirm

For any arm whose paired contrast meets the minimum effect on both assays:
freeze the confirmation batch, open the next sealed bank once, identify the
acquired node or read in the elites and ablate it (T22.F03 readings), check
incumbent competence, then run the natural-world check from run 1's plan.
Only then may the note recommend a feature. A candidate that fails is
recorded and the run returns to phase 3; a later candidate uses the next
bank.

## Deliverable

The results note `docs/strategy/evolvability-exploration-2026-10-run2.md`,
with compact summaries in `docs/strategy/evolvability-exploration-2026-10-run2/`,
in run 1's form: question, launch revision, ledger, frontier table (both
repaired assays as columns), findings by rung and family, review
dispositions, branch and commit hashes, the evidence outcome (A, B or C),
the execution status, and a resume handoff when incomplete. It lists any
`instr:` commit worth keeping (the repaired instrument especially) as a T22
feature candidate and any qualified prototype as a roadmap feature
candidate in its owning track.

## Goal command

Open a new session in the main checkout on a clean, current `main`: Opus 5.5,
effort `medium`, auto mode, no Fable advisor. Paste:

```
/goal An evolvability exploration run 2 is complete on main. Read docs/strategy/evolvability-exploration-plan-2026-10-02.md first and follow it exactly; it amends docs/strategy/evolvability-exploration-plan-2026-10-01.md, which governs everything it does not change, under the T22 exploration contract. Confirm you are in the main checkout on a clean main, record the launch revision, create the worktree with EnterWorktree named evolvability-exploration-2, confirm its HEAD is the launch revision, and read the starting evidence the plan lists. Run the plan's phases in order with research, predeclare, build, run, read, record cycles, without pausing to ask except where run 1's plan says to stop. Keep exploring until one of the plan's evidence outcomes holds: A, feature evidence that reached the promising claim on both repaired assays with a sealed holdout bank, ablation and the natural-world check; B, a supported negative meeting the plan's predeclared effect bound for every phase 3 arm on both repaired assays; or C, a concrete blocker. "More exploration" is never an outcome. Every arm passes the five not-forcing checks, matched controls and claim rules; the frontier table stays descriptive. Prototypes are default-off proto commits classified policy-deviation and instrument harness changes are instr commits; neither merges. Commit the ledger and results note every cycle. Run the Codex reviews and advice through codex exec as the plans say, use no Fable advisor or agent, and record each finding's disposition. Begin closing by turn 360 in run 1's close order. Done means all of these are shown in this conversation, with command output and the note's ledger, frontier table, evidence outcome and execution status quoted, not only file paths: docs/strategy/evolvability-exploration-2026-10-run2.md is on main; make check or make check-docs exited 0 on main; git worktree list no longer lists the worktree and git branch still lists its branch; git status on main is clean. If the turn cap arrives before an outcome holds, label the note incomplete with its resume handoff, land it as above and report it. Stop after 400 turns.
```
