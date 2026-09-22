# T19.F05 — Decision-State Inputs

**Status**: In Progress
**Last updated**: 2026-09-22
**Feature**: T19.F05
**Track**: [T19 — Mesh Action Selection and Live State](../../roadmaps/t19-mesh-action-selection-and-live-state.md)

## Goal

Corollary discharge: a brain senses what it has just decided and how tired it
is. Five self inputs become drawable input references: the current pass's vote
vector (`ActionVotes`), the previous pass's final vote vector
(`PreviousPassVotes`), the per-kind bars (`CommitCounts`), the hops dispatched
this tick (`HopsThisTick`), and the previous tick's four outcome channels
(`PreviousOutcome`). The first four are live within the tick; the fifth is
frozen at tick start like every other perception. No founder changes; every
mutated birth moves because the draw enumeration grows from 22 to 27.

## Non-Goals

- A derived queued-displacement input (declined by the user); re-resolving
  perception at an imagined position; world feedback between passes.
- T19.F06's retirements and inspector redesign: the new references get the
  minimal label, color, and read-class rendering that keeps the inspector
  truthful, nothing more.
- Changing how the outcome channels are computed for reward learning, the
  `NoOp`-counts-as-success rule, the reward pass itself, or plasticity.
- Any `RuntimeConfig` or `MutationConfig` field; any change to the existing
  references' resolution, widths, kinds, or soft defaults; any founder edit.
- A trace or protocol field for the new reads beyond what compiles
  (`PROTOCOL_VERSION` unchanged: nothing is removed).

## Inputs and Invariants

Sources of truth: the track row, its "Decisions recorded" and "Scope,
T19.F05" notes, and the "Epochs" and "Readings that become historical"
notes; the [mesh action-selection review](../../strategy/mesh-action-selection-review-2026-09-20.md)
Section 2.7 (the five inputs, as the user settled them) and Section 7.4
(urgency signals); T19.F04 invariants 1, 5, 7, and 8; T11.F22's kind rule
(`mutation/input_ref/mod.rs`); the code named in each invariant. No external
research is repeated: the note's Section 7 read the primary texts, and the
input set is a user decision.

Options settled here, all internal to the codebase:

| Option | Disposition |
| --- | --- |
| Five `InputReference` variants vs. one `DecisionState` compound with 59 sub-values | Five variants: each has its own width and kind, so T11.F22's within-kind swap stays meaningful and the census counts each separately; the scope names five |
| Counts as raw `f32` vs. unit-scaled | Raw for `CommitCounts`, `HopsThisTick`, and `OffspringSuccess`, like today's queue reads (`action_type_at`, `ReadActionQueueLength`); energy quantities as fractions of `max_energy` like `EnergyCurrent` (T17.F02); `ResolveCtx` carries no config, and a ramp-relative hop scale divides by zero at `hop_ramp_allowance = 0` |
| `HopsThisTick` as a new variant vs. a `DynamicIntrospectionKey` | Key, as the scope says: it joins the introspection scalars' swap kind |
| Read class for the in-tick decision inputs | New `MeshReadClass::Decision`; classing them `Introspection` would let a `Swap` turn a 4-wide `CommitCounts` into a 4-wide `PreviousOutcome`, a different modality |
| Where the previous outcome lives | On `CreatureState`, written once per tick for every creature in Phase 2.5 from the same signal bank the reward pass reads; the accumulator is per tick and is dropped today |
| Out-of-range `sub_idx` on the new compounds | `0.0`, as `ActionQueue` and the mesh spec's matrix; the graph draw already bounds `sub_idx` by width |

1. **Catalog.** `InputReference` gains `ActionVotes`, `PreviousPassVotes`,
   `CommitCounts`, and `PreviousOutcome`; `DynamicIntrospectionKey` gains
   `HopsThisTick`. Widths (`mutation/compound.rs` and the census universe)
   are constants of the vote catalog, never config:

| Reference | Width | Sub-value `i` | Value |
| --- | --- | --- | --- |
| `ActionVotes` | `VOTE_SINK_COUNT` (27) | `VoteSink::from_index(i)` | `side_outputs.votes[i]`: the sanitized sum of every contribution committed so far this pass; the dispatch in flight sees its own earlier contribution this pass, not the one it is staging |
| `PreviousPassVotes` | 27 | same | The vote vector at the previous pass's end; zeros in pass one and at tick start |
| `CommitCounts` | `VOTE_KIND_COUNT` (4) | `VoteKind` index (Eat, Move, Reproduce, StealEnergy) | `commit_counts[i] as f32`, the bar T19.F04 raises by one per commit |
| `DynamicIntrospection(HopsThisTick)` | 1 | ignored, as `EnergyConsumedThisTick` | `work_counters.mesh_hops as f32`: dispatches this tick including the one in flight (the counter is raised before dispatch) |
| `PreviousOutcome` | `OUTCOME_CHANNEL_COUNT` (4) | `OutcomeChannel` discriminant | The stored block of invariant 4 |

   On the four compound references, `i` at or past the width reads `0.0`;
   the scalar ignores `sub_idx` like every dynamic scalar today. The VM's
   `ReadInput` and the graph's `InputLeaf` reach every one of them through
   `resolve_input` (`runtime/inputs.rs`); `ResolveCtx` carries the three
   vectors and the hop count beside `action_queue`. Where a construction
   site clones the queue because `side_outputs` is borrowed mutably (the
   graph's post-plasticity and effects contexts, `cgp/execute.rs`), it may
   copy the vectors the same way: at most two 27-value copies per graph
   dispatch, no borrowing refactor.
2. **Live reads.** `ActionVotes`, `CommitCounts`, and `HopsThisTick` resolve
   live at every read, like `EnergyCurrent`: a graph node's evaluation,
   post-plasticity, and effects contexts read the same vote vector (nothing
   commits mid-dispatch), and the VM reads per instruction. The mesh loop
   copies `votes` into `previous_pass_votes` in `begin_pass` before clearing
   it; both are tick-scoped, so a new tick starts from zeros with no
   inherited decision state (T19.F04 invariant 1: `V = 0` at tick start).
3. **Frozen read.** `PreviousOutcome` is perception: assembled into
   `StaticInputs` at tick start from `CreatureState`, never re-read during
   the tick. Sensor-boundary rules stand (`v3-sensor-spec.md` Section 2).
4. **Outcome store.** Phase 2.5 computes the outcome signal bank once for
   every living creature (energy at Phase 2.5 entry minus the post-Phase-0
   snapshot, as the reward pass does today, before any reward debit) and
   stores it on the creature;
   the reward pass consumes that same bank. A newborn stores zeros until its
   first full tick, and inherits nothing from its parent. The read scales
   the two energy channels:

| Channel | `sub_idx` | Stored (unchanged reward definition) | Read |
| --- | --- | --- | --- |
| `EnergyDelta` | 0 | `energy_after - energy_before` | `/ max_energy`, clamped to `[-1, 1]` |
| `ActionSuccess` | 1 | succeeded / attempted, `0` when none attempted; a `NoOp` succeeds | as stored |
| `DamageDelta` | 2 | `-damage_received` | `/ max_energy`, clamped to `[-1, 0]` |
| `OffspringSuccess` | 3 | offspring spawned this tick | as stored |

5. **Draw and kinds.** `random_input_reference_for_food_types` draws
   `gen_range(0u8..27)`: indices `0..=13` keep their catalog entries, `14..=18`
   are `ActionVotes`, `PreviousPassVotes`, `CommitCounts`,
   `DynamicIntrospection(HopsThisTick)`, `PreviousOutcome`, and `_` is the
   upstream slot (8 weighted, as today). The five new entries cost the key
   draw alone; the existing branches keep their sub-draws (a food type with
   more than one food type, the upstream slot), so the branch-specific
   draw-count tests stay as they are with `27` for `22`.
   `input_reference_universe` lists the five so `Swap` sees them.

| Reference | `MeshReadClass` | Kind `(class, width)` | Swap partners |
| --- | --- | --- | --- |
| `ActionVotes`, `PreviousPassVotes` | `Decision` (new; serde `decision`) | `(Decision, 27)` | each other |
| `CommitCounts` | `Decision` | `(Decision, 4)` | none |
| `HopsThisTick` | `Introspection` | `(Introspection, 1)` | `AgeTicks`, `EnergyCurrent`, `EnergyConsumedThisTick` |
| `PreviousOutcome` | `Introspection` | `(Introspection, 4)` | none |

6. **Neutral at birth, not byte-identical.** The founders reference none of
   the five, so every founder genome, `FOUNDER_GENOME_SIZE_UNITS` (97), the
   founder digest, and `founder_only_trajectory_digest_is_pinned` (mutation
   off) are unchanged. Every seeded run with mutation on moves through the
   draw remap and the introspection kind's third alternative;
   `legacy_default_short_run_identity` is re-pinned after two agreeing runs.
   `config_digest` and the v3-cli recipe pin are unchanged.
7. **Determinism.** No read draws RNG; every value is per-creature state or
   the frozen snapshot, so the parallel cognition phase stays
   order-independent; the outcome store is one write per creature keyed by
   id. The mutation draw stays one `gen_range` per key plus the existing
   branch sub-draws; which branch a given RNG state lands in moves, which
   is the remap.
8. **Census.** `CreatureSensorCensus` gains a set of the decision-state
   references a live, reachable read addresses (five keys: the four variants
   and `HopsThisTick`), counted the way world keys are (live VM `ReadInput`,
   wired graph `InputLeaf`); the bench `SensorCensus` gains
   `decision_inputs`, five rows always present in catalog order with
   `creatures` counts, zero included, beside `world_inputs`. The field is
   `#[serde(default)]` so the T19.F04 summaries still load for comparison;
   `SCHEMA_VERSION` and `SUMMARY_VERSION` do not move (additive, as the
   T14 blocks were).
9. **Compile-coupled consumers, all owned here.**

| Consumer | Change |
| --- | --- |
| `contracts/inputs.rs`, `runtime/inputs.rs`, `runtime/types.rs`, `runtime/mesh.rs`, `cgp/execute.rs`, `vm.rs` | Variants, `ResolveCtx` fields, `previous_pass_votes`, resolution |
| `sensors/static_inputs.rs`, `creature/state.rs`, `simulation/tick.rs`, `simulation/outcomes.rs` | The stored block, its Phase 2.5 write, its assembly and scaling |
| `mutation/sampling.rs`, `mutation/compound.rs`, `mutation/input_ref/mod.rs`, `mesh_annotations.rs`, `cgp_mesh_annotations.rs` | Draw, widths, universe, class |
| `creature/sensor_census.rs`, v3-cli `bench/tracking.rs` and its tests | Census set and rows |
| v3-server `http/creature.rs` class test | `Decision` in the sorted-class fixture |
| `frontend/src/types/genome.ts`, `creature-detail.ts`, `inspector/inputRefUtils.ts`, `mesh/meshSemantics.ts` and their tests | Type unions, label and color, `decision` read class |
| `v3-sensor-spec.md` (1, 3.2, 3.3, new 3.6 and 3.7, 8, 10), `v3-mutation-spec.md` (draw and the `Swap` kind table), `v3-mesh-execution-spec.md` (2, 4), `v3-vm-isa-spec.md` (`ReadInput` and upstream resolution), `v3-tick-orchestration-spec.md` (Phase 2.5) | Catalog of 27, widths, classes, the outcome store; no reference doc lists the read classes (grep `action_queue`, 2026-09-22) |

10. **Worked cases.** Fixtures the implementer realizes on either backend.
    The control of every case is the same genome with the new read replaced
    by the constant `0`, so the read is the only difference. `Move N` is
    sink 1; a cycle is a self-target; `max_mesh_hops` 64,
    `max_actions_per_turn` 10.

| Case | Genome | Control queue (read = 0) | Queue with the read |
| --- | --- | --- | --- |
| V1 lateral inhibition | A votes `Eat 1`, routes to B; B votes `Move N = 1 - ActionVotes[Eat]` | `[Eat, Move N]` (pass 1 ties at 1, lowest index wins; pass 2 `Move N`; pass 3 `NoDecision`) | `[Eat]`, tick `NoDecision` |
| V2 decision history | One node votes `Eat 1` and `Move N = PreviousPassVotes[Eat]` | `[Eat]` | `[Eat, Move N]`: pass 2 reads `1`, pass 3 ends `NoDecision` |
| V3 bar-aware plan | One node votes `Eat = 2 - CommitCounts[Eat]` | `[Eat, Eat]` | `[Eat]` |
| V4 deliberation cost | Self-routing node, no `Decide`, votes `Move N 2` and `Terminate = HopsThisTick - 64` | `[Move N, Move N]`, 192 hops, `NoDecision` (`Terminate` stays `-64`) | `[Move N]`, 128 hops, `TerminateVoted` (pass 2 ends `PassCapReached` with `Terminate` 64 >= effective 1) |
| V5 previous outcome | One node votes `Move N = PreviousOutcome[ActionSuccess]` | `NoOp` on both ticks | Tick 1 `NoOp`; tick 2 reads `1.0` and moves; `assemble_static_inputs` after an `Eat` on food reads `[ΔE/max_energy, 1, 0, 0]`; a newborn reads zeros |
| V6 staging boundary | A self-targeting VM node, no `Decide`, reads `ActionVotes[Eat]` before and after its own `AddVote Eat 1` | — | Two 64-hop passes: on each pass's first dispatch both reads are `0` (the vector is cleared per pass), on every later dispatch `1`; pass 1 commits `Eat`, pass 2 ends `NoDecision`, queue `[Eat]`; traced and untraced execution agree on every read and the queue |

## Implementation Tasks

- [x] Catalog, resolution, `previous_pass_votes`, and the frozen outcome
      block (invariants 1 to 4), tests first (the worked cases).
- [x] Draw, widths, universe, class, census, and the bench rows (5, 8).
- [x] Frontend types and minimal rendering; server class fixture (9).
- [x] Reference specs (9); re-pin `legacy_default_short_run_identity` (6).
- [ ] Readings file: worked-case transcripts, the pin before and after, the
      births probe and drift walk against T19.F04, the census rows from the
      goal summary.

## Verification

- [ ] Worked cases V1 to V6 with their controls and the founder
      byte-identity checks (`cargo test -p v3-core`): names and transcripts
      in [`docs/progress/readings/t19-f05.md`](../../progress/readings/t19-f05.md).
- [ ] `cargo test -p v3-core --test viability` first, then `make check` ->
      exit 0 (the tick loop gains the Phase 2.5 store).
- [ ] Draw and kind checks: the 27-entry draw covers every new reference at
      one draw each, the existing branch sub-draws unchanged; the swap table
      of invariant 5 (`Swap` never crosses a class or width).
- [ ] Census: a genome reading each new reference is counted once per
      reference; `decision_inputs` carries five rows in the gate and goal
      summaries.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      and every survivor resolved as killed, equivalent, or deferred; full
      survivor list here.
- [ ] Benchmark summaries stored at
      `docs/progress/features/t19-f05-decision-state-inputs.json` and
      `-goal.json` (`make bench PROFILE=gate FEATURE=t19-f05-decision-state-inputs`,
      `make bench PROFILE=goal FEATURE=t19-f05-decision-state-inputs`), local
      raw hash and byte count and verification time checked, series entries
      point to the summaries, no full report staged.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: corollary
discharge (efference copy) and interoceptive fatigue: a nervous system senses
its own motor commands and its own effort. It reaches creatures through the
body alone: what the brain voted, committed, and paid, and what happened to
the body last tick; no world sensor is added and no environmental pressure,
so the three-world integration rule does not apply. Predeclared compute: per hop,
passing three references and a count, plus at most two 27-value copies per
graph dispatch where the queue is already cloned; one 27-value copy per
pass; one 4-value store per creature per tick. Every seeded trajectory with
mutation on moves by the draw remap (track "Readings that become
historical"), so counter movement is trajectory noise, not mechanism cost.

References: gate epoch and previous closure are both
`t19-f04-vote-based-action-selection.json`; goal-worlds epoch and previous
closure are both its `-goal.json`. Standing thresholds: counters flag at 10%,
severe at 50%; wall flags at 25%, severe at 100%. The goal profile runs once.
Epoch: the track leaves the decision to closure. This spec predeclares no
justified cost, so neither epoch is re-pinned by this feature; a severe is
reported to the user with the draw remap as its only predeclared cause, and
the user decides the re-pin, as at T11.F21 (epoch kept).

| Reading | Predeclaration |
| --- | --- |
| Gate and goal `mesh_hops`, `graph_relax_iters`, `vm_steps`, `passes`, `decided_passes`, `pass_cap_hits`, `plasticity_updates`, `actions_applied`, `births` | No direction; a severe on any is investigated as trajectory movement before it is presented, and a mechanism cost found there does not fit and is escalated |
| Cognition wall per creature-tick (`cognition_ms`, `wall_clock_ms_per_creature_tick`) | Unchanged within the flag threshold on a matching host (`MacBookPro.lan`), else `wall_clock: null` and reported |
| `sensor_census.decision_inputs`, final checkpoint per goal world | First reading: creatures reading each of the five, zero included; no direction |
| Births probe (founder half): changed, silent, dead per mutated birth | Against T19.F04, ceiling twice its dead value; no other direction (a new reference is 5 of 27 draws by construction) |
| Drift walk changed, silent, dead per birth (depths 0 to 2,000) | Against T19.F04, no floor; re-based knowingly by the draw remap |
| Goal persistence (final, minimum, plateau) | No direction; a world whose final population falls below half of T19.F04's is investigated before the result is presented |
| `config_digest`, founder digest, `FOUNDER_GENOME_SIZE_UNITS` | Unchanged (`inputs_changed` false) |
| `legacy_default_short_run_identity` | Moves; re-pinned and listed |

**Measured verdict.** Not yet measured.

- Summaries: [gate](../../progress/features/t19-f05-decision-state-inputs.json),
  [goal](../../progress/features/t19-f05-decision-state-inputs-goal.json).
- Full readings: [`docs/progress/readings/t19-f05.md`](../../progress/readings/t19-f05.md).

## Success Criteria

- [ ] All five references resolve as invariant 1's table says on both
      backends, and the worked cases V1 to V6 pass with their controls.
- [ ] The draw enumerates 27 entries, the kinds and swap partners are as
      invariant 5's table says, and the census reports the five rows.
- [ ] Founder bytes, size pin, founder digest, founder-only trajectory,
      `config_digest`, and the recipe pin are unchanged; the short-run
      identity is re-pinned.
- [ ] Gate and goal summaries stored with the predeclared readings recorded;
      the track row checked and this spec Complete.

## Notes for AI Agents

- Decision: the user declined a derived queued-displacement input and kept
  the five inputs as listed (track "Decisions recorded", 2026-09-21).
