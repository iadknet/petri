# T19.F04 — Vote-Based Action Selection

**Status**: In Progress
**Last updated**: 2026-09-22
**Feature**: T19.F04
**Track**: [T19 — Mesh Action Selection and Live State](../../roadmaps/t19-mesh-action-selection-and-live-state.md)

## Goal

Motor programs compete and the winner habituates, as in basal ganglia
selection and competitive queuing of serial order. A tick is a sequence of
passes: each runs the mesh from the entry with the votes cleared and the bus
carried, ends at the genome's `Decide` vote or the chain's end, and commits
one action of the kind with the largest effective vote (best sink vote minus
the kind's bar), raising that bar by one. The tick ends at `NoDecision`,
`TerminateVoted`, or the action cap. The push machinery, the action bank, the
execute gate, and `MutateActionSlotBehavior` are deleted in the same commit;
every founder profile is re-expressed in votes and acts identically.

## Non-Goals

- T19.F05's inputs; T19.F06's retirements (`action_queue_cap`, the inspector
  redesign, historical markings, the `route_varies_with_input` split, the
  deleted-name grep). Only the minimal vote rendering ships here.
- Meanings for the unread parameter slots (T17.F03, T17.F04); the T18
  layouts; the census on goal-sampled genomes and the discovery protocol
  (T11.F10).
- Changing the ramp constants, `max_mesh_hops`, or `max_actions_per_turn`;
  adding any config field; plasticity on vote-sink edges; a heritable unit;
  votes persisting across passes or ticks; any stall or convergence exit.

## Inputs and Invariants

Sources of truth: the track row and its "Decisions recorded", "Two rules",
"Scope, T19.F04", "Decomposition rules", "Epochs", "Contract text", and
"Readings that become historical" notes; the
[mesh action-selection review](../../strategy/mesh-action-selection-review-2026-09-20.md)
Sections 0, 1.4, 1.7 to 1.9, 2.3 to 2.8, 3, and 5 (2.3 is the transition
contract; W1 to W19 are the acceptance fixtures); T19.F02 invariants 2, 5, 6
and T19.F03 invariants 1 to 8; the code named in each invariant, from
`execute_creature_mesh_impl` (`crates/v3-core/src/runtime/mesh.rs`).

Options: settled by the note (Section 7 read the primary texts; the Section
2.3 table compares six commit rules). Adopted: reset per pass, per-kind
rising bar, within-kind argmax (the only rule that ports the founder and
commits a unit edge once). Rejected: action cap only, one global bar,
per-sink bar, winner-only suppression, unit
debit, random tie-breaking (the parallel phase is RNG-free), queue discard at
exhaustion (order of computation would decide survival), a decide register.
No external research is repeated here.

1. **Pass loop.** The shared executor runs this in every execution mode; `V`
   is the vote vector, `C` the per-kind bars, `unit` 1.0, `E` the effective
   vote per kind:

```text
tick start: V = 0, C = 0, queue empty, bus zeroed, parameter surface zeroed,
            per-node contributions empty; internal state untouched
pass:
  hops_this_pass = 0; V = 0; per-node contributions cleared
  run the chain from the entry (bus = the last dispatched node's output slots
  of the previous pass; state, memory, weights, traces live)
    after each committed dispatch: that node's contribution replaces its
    earlier one this pass; V = sanitized sum over nodes visited this pass
    if V[Decide] > 0 and (max_K E[K] > 0 or (queue non-empty and V[Terminate] > 0)):
      end the pass (Decided)
  the pass also ends at NoTargets, MissingNode, PassCapReached (hops_this_pass
  reaches max_mesh_hops), or EnergyExhausted; all keep V and the queue
  pass end:
    for K in {Eat, Move, Reproduce, StealEnergy}:
      best[K] = lowest-index argmax over K's sinks of V
      E[K] = V[best[K]] - C[K] * unit
    K* = argmax E  (ties: the kind committed in the previous pass, then lowest index)
    if EnergyExhausted: end the tick (EnergyExhausted), queue kept
    if E[K*] <= 0: end the tick (NoDecision)
    if queue non-empty and V[Terminate] >= E[K*]: end the tick (TerminateVoted)
    commit best[K*] decoded with the parameter surface of K*; C[K*] += 1
    if queue full: end the tick (ActionCapReached)
    next pass
exit: settle the bid once; actions = queue, or NoOp when empty
```

2. **Guards and ties.** `Decide` ends a pass only when the pass end would do
   something: some kind's effective vote is positive, or the queue is
   non-empty and `Terminate` is positive (the note's Section 2.4, both
   clauses; the second lets a genome leave a cycle by one `Terminate` edge
   instead of a capped pass). W17 votes no `Terminate`, so its second pass
   runs to the cap (`PassCapReached`, `NoDecision`, queue `Move W`); the
   row's "six hops" assumed a reset bus and is superseded. W17b adds a static
   `Terminate` vote and ends pass two at its first `Decide` boundary.
   `Terminate` never commits, has no bar, wins its ties, and cannot end an
   empty tick (W15). A revisited node replaces its own contribution (W11 to
   W14). Comparisons are on sanitized `f32`; no RNG is drawn.
3. **Commit decode.** The direction is the winning sink; the parameter surface
   `action_params[kind][i]` (T19.F03 invariant 6: zeroed at tick start,
   overwritten per visit, read at commit) supplies the rest through the
   existing `action_decode.rs` clamps. Index `i` keeps today's `meta[i]`
   meaning minus the direction, so `ReadActionQueueParam` and the
   `ActionQueue` compound input keep theirs; the other slots are unread until
   T17.F03/F04 name them.

| Kind | Sink | Parameter reads |
| --- | --- | --- |
| `Eat` | `Eat` | `type_idx = decode_food_type_idx(params[Eat][0])` |
| `Move` | `Move(d)` | none |
| `Reproduce` | `Reproduce(d)` | `energy_transfer_fraction = clamp_unit_interval(params[Reproduce][1])` |
| `StealEnergy` | `StealEnergy(d)` | `amount = clamp_non_negative_finite(params[StealEnergy][1])` |

4. **Reasons and trace.** `TerminationReason` becomes the tick reasons
   `NoDecision`, `TerminateVoted`, `ActionCapReached`, `EnergyExhausted`; a
   new `PassEndReason` carries `Decided`, `PassCapReached`, `NoTargets`,
   `MissingNode`, `EnergyExhausted`. A missing entry node is a pass ending
   `MissingNode` with zero votes and a tick ending `NoDecision` (`NoOp`).
   `TickTrace` carries one record per pass (end reason, final `V`, `E`, the
   committed action or none, hops) beside the tick reason and final bars;
   `MeshHopTrace` carries its pass index; `MeshOutput` carries the final bars
   and the pass count. Every soft-default row keeps its outcome.
5. **Budgets and counters.** `max_mesh_hops` (64) bounds hops per pass and
   resets per pass; `work_counters.mesh_hops` and the T19.F01 ramp index never
   reset within the tick, so cycling every pass pays the ramp across passes
   (640 hops cost 18.5 energy at the defaults). Passes per tick are at most
   `max_actions_per_turn` (10). `WorkCounters` gains `passes` and
   `decided_passes`; both join bench `COUNTER_NAMES` after `pass_cap_hits`
   (level `new` against a reference without them) and the T11.F14 block,
   where `hop_cap_hits` is deleted (no tick reason names the cap), the
   capped fraction is capped passes over passes, and `passes` and
   `decided_passes` are totals; `MESH_EXECUTION_VERSION` becomes
   `mesh-execution-v2` because the block's shape and reason names changed.
6. **Exhaustion and bid.** Exhaustion at any point (compute, ramp, bid) keeps
   the actions committed before it. The bid settles once on every exit,
   `paid = min(bid, energy)`, nothing when energy is already gone; an all-in
   pins energy to 0, ends the tick `EnergyExhausted` with
   `DeathCause::PriorityBid`, and keeps the queue with `priority_bid = paid`.
   The mesh spec's exhaustion rows for the queue and the bid say so; T14.F03's
   mortality partition moves by construction.
7. **Bus and state.** `upstream_slots` is zeroed at tick start and carried
   from the last dispatched node of one pass to the entry of the next.
   Nothing internal resets at a pass boundary; T19.F02's state, exhaustion,
   and scratch rules stand. The T19.F03 contribution container is per pass.
8. **Surfaces deleted, replaced, and kept.** Both T19.F03 draw exclusions are
   lifted (the one draw remap): `random_vm_instruction` draws
   `gen_range(0u8..39)` with `AddVote` drawable, and `pick_random_surface`
   enumerates `ActionVote` and `ActionParam` sinks like every other sink, so
   every edge operator reaches them through the sink surface. A stored genome
   carrying `action_bank` or `execute_gate` is regenerated, never migrated
   (no committed fixture carries one). Vote-sink edges are not plastic.

| Surface | Disposition |
| --- | --- |
| `PushAction`, `PopAction`, `ExecuteActionQueue`, `WriteDirectionBid` | Deleted from `VmInstruction`, the interpreter, the fresh draw, nudge, classification, the ISA table, the all-opcodes test (39 opcodes), `frontend/src/types/genome.ts` |
| `WriteWorldActionMeta { slot_idx, src }` | Becomes `WriteActionParam { slot_idx, src }` at the same opcode position and cost: `slot_idx` in `0..8` addresses `params[slot_idx / 2][slot_idx % 2]`, overwrite, invalid slot ignored, draw and nudge ranges unchanged |
| `ReadActionQueue*`, `SetPriorityBid`, `Halt`, `AddVote` | Kept; `Halt` is the dispatch end |
| `NodeResult::terminal`, `DirectionBank`, `select_direction`, `TerminationReason::ActionEmitted`, `MaxHopsReached` | Deleted |
| `action_bank`, `execute_gate`, `ActionSlot`, `ActionSlotBehavior`, `ExecuteGate`, `GraphActionSlotTrace`, `GraphExecuteGateTrace`, the `EdgeSurface` variants addressing them | Deleted from `cgp.rs`, `effects.rs`, `traced.rs`, `trace/domain.rs`, the mutation operators |
| `GraphOperator::MutateActionSlotBehavior`, `MutationOperator::GraphMutateActionSlotBehavior` | Deleted; the operator name leaves every operator list and telemetry key |

9. **Founders.** Node 0 (graph sensor) is unchanged. Node 1 becomes a graph
   node reading upstream slots 0 to 5 and the `ActionQueue` compound input,
   with the vote edges below realizing W1 to W3 (`f` = `[food_here > 0]`,
   `can` = slot 1, `ring[d]` = slots 2 to 5 on the cardinal sinks `d` in
   `{0, 2, 4, 6}` so the lowest-index tie rule reproduces the VM's N, E, S, W
   first-argmax, `q` = "queue slot 0 holds a `Reproduce`", `g` the profile's
   reproduce gate):

| Sink | V3Alpha1 (`g = can`) | ForageFirst* (`g = can · (1 - f)`) |
| --- | --- | --- |
| `Eat` | `f - 2g - 2q` | `1 - 2g - 2q` |
| `Move[d]` (cardinal) | `0.5 + 0.4·ring[d] - 2g - 2q` | same |
| `Reproduce[d]` (cardinal) | `g·(0.5 + 0.4·ring[d])` | same |
| `Terminate` | `q` | `q` |
| `ActionParam(Reproduce, 1)` | `Constant` = the profile's transfer fraction | same |
| `ActionParam(Eat, 0)` | unwired (type 0) | unwired |

   The tick outcomes are exactly today's, with `d` the first cardinal argmax:

| Profile | `can` | `f` | Queue |
| --- | --- | --- | --- |
| V3Alpha1 | 1 | any | `Reproduce d` |
| V3Alpha1 | 0 | 1 | `Eat, Move d` |
| V3Alpha1 | 0 | 0 | `Move d` |
| ForageFirst* | any | 1 | `Eat, Move d` |
| ForageFirst* | 1 | 0 | `Reproduce d` |
| ForageFirst* | 0 | 0 | `Eat, Move d` |

   `q` closes the reproduce branch because live energy falls between passes
   and could cross the gate. `[Eat]` alone at `max_actions_per_turn` 1;
   thresholds and fractions unchanged. Verified by the founder tests
   (`founder.rs`, the `limit 1` case, the ring proptest), a 2,000-ring check
   against `expected_direction`, and `tests/viability.rs`.
   `FOUNDER_GENOME_SIZE_UNITS` is re-pinned to the measured `genome_size()`
   (97); the per-unit rate stays `0.005`, the `0.55` assertion is re-pinned
   to `0.005 × 97`, and the T11 track's walk-anchor wording names 97. The
   founder digest and `legacy_default_short_run_identity` move and are
   re-pinned after reproduction; a pin on positions, energy, or ages moves
   only through the founder's changed actions.
10. **Compile-coupled consumers, all owned here.**

| Consumer | Change |
| --- | --- |
| v3-server `sample_protocol.rs`, `sample_assembler.rs` | Pass records, bars, reasons; `PROTOCOL_VERSION` becomes `v3alpha3` (fields are removed; T19.F03's additive reasoning does not cover removal) |
| Every frontend source and test referencing the bank, the gate, the four opcodes, or `ActionEmitted` (11 sources, 17 tests by grep, 2026-09-22) | Types mirrored; the T19.F03 vote block shown per pass with the committed action and pass reason; nothing more |
| `tests/vm_all_opcodes_e2e.rs`, the memory-sensitivity and cycle fixtures | Rebuilt on votes |
| T13 recruitment-path fixtures and qualification | Activation is one edge into a vote sink, so all nine forms qualify and the recorded `graph_blank` and `vm_blank` growth gaps close by construction (a finding, not a re-pin); stored records become historical at T19.F06 |
| Steering structural reading | "The executed node contributes to a `Move` sink" |
| Mesh annotations write classes | `ActionVote` and `ActionParam` are class `Action`; the slot classes go |

11. **Config.** No `RuntimeConfig` or `MutationConfig` field changes; every
    `config_digest` and the v3-cli recipe digest pin (`tests/cli.rs`) are
    unchanged. Adding a field would move those pins and is out of scope.
12. **Readings.** The births probe adds `reordered` (the same multiset of
    actions on every differing execution, in another order) and `recount`
    (the same action kinds, different counts) as tallies inside `Changed`,
    whose definition is unchanged so the series compares. The one-edge census
    is a founder reading: every unit-weight edge from an input-leaf sub-value
    of node 1 into each of the 27 vote sinks, classified on the neighborhood
    battery, with W4, W5, W6, W7, W10 as exact fixtures. The T11.F14 evolved
    half reports tick reasons by the new names and passes per execution.
13. **Docs, rewritten for passes, citing the worked cases as fixtures.**

| Reference | Sections |
| --- | --- |
| `v3-mesh-execution-spec.md` | 1 to 5: pass loop, commit rule, reasons, soft defaults, exhaustion rows, bid |
| `v3-vm-isa-spec.md` | 2, 3, 6, 7: 39 opcodes, `WriteActionParam`, the output lifecycle as votes |
| `v3-graph-backend-spec.md` | Bank and gate sections replaced by the vote and parameter sinks |
| `v3-mutation-spec.md` | Operator list, both draws, `AddGraphEdge` |
| `v3-startup-seeding-spec.md` | 5: the founder as votes |
| `v3-server-api-protocol-spec.md`, `v3-genome-spec.md` | Section 4 and the version; catalog and bank lines |

## Implementation Tasks

- [x] Fixtures first (TDD): W1 to W19 as executor tests on hand-built vote
      genomes with the three execution modes agreeing, the founder truth
      table, the `limit 1` case, the 2,000-ring check.
- [x] Pass loop, reasons, counters, exhaustion and bid rules, bus carry, and
      the per-pass contribution container (invariants 1 to 7).
- [x] Deletions, `WriteActionParam`, both draws lifted (invariant 8).
- [x] Founder port and pin moves (invariant 9); run
      `cargo test -p v3-core --test viability` first.
- [x] Consumers, server protocol, frontend types and rendering (invariant 10).
- [x] Births-probe classes, one-edge census, T11.F14 and steering
      re-expression (invariant 12); readings file.
- [ ] Reference docs (invariant 13); `make check`.

## Verification

- [ ] Focused tests: `cargo test -p v3-core --test viability` -> 28 passed;
      workspace -> 2,092 passed, 1 failed (gate severe, pre re-pin); `npx vitest run` -> 327 passed; `make check` pending.
      Transcripts in [`docs/progress/readings/t19-f04.md`](../../progress/readings/t19-f04.md).
- [x] Grep proof: none of `PushAction`, `PopAction`, `ExecuteActionQueue`,
      `WriteDirectionBid`, `ActionSlot`, `ExecuteGate`, `action_bank`,
      `execute_gate`, `MutateActionSlotBehavior`, `ActionEmitted`,
      `DirectionBank` under `crates/` or `frontend/src` (historical specs keep
      theirs) -> result in the readings file.
- [x] Founder exactness: truth table, `limit 1`, proptest, and 2,000-ring
      check pass; the moved pins listed with before and after values in the
      readings file.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      every survivor resolved as killed, equivalent, or deferred (a long gate
      is budgeted: the deleted surface touches every hub file).
- [ ] Benchmark summaries stored at
      `docs/progress/features/t19-f04-vote-based-action-selection.json` and
      `-goal.json`, local raw hash, byte count, and verification time checked,
      series entries point to the summaries, no full report staged.
- [ ] Readings file: births probe with the new classes, the one-edge census,
      drift walk against T19.F03, the T11.F14 and steering evolved halves,
      Orchards seed 12 to 2,000 ticks (`v3 run`: command, final and minimum
      population, tick reasons), passes and `Decided` against capped passes,
      cognition wall per creature-tick, founder per-tick compute cost before
      and after.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: basal ganglia
action selection with habituation of the winner and competitive queuing of
serial order; it reaches creatures through the body (what the genome votes,
what the ramp charges), never a sensor. Predeclared compute: a forage tick with
food runs three passes of two hops where today's founder runs one graph visit
and one VM dispatch, so `mesh_hops` and `graph_relax_iters` rise by
construction, `vm_steps` falls to what evolved VM nodes execute, and cognition
wall per creature-tick rises with passes.

References: gate epoch and previous closure are
`t19-f02-live-internal-state-and-legal-cycles.json` and
`t19-f03-vote-surface-as-inert-data.json`; goal-worlds epoch and previous
closure are their `-goal.json` counterparts. Standing thresholds: counters
flag at 10%, severe at 50%; wall flags at 25%, severe at 100%. The goal
profile runs once. **Both epochs re-pin in the closing commit** (track
"Epochs"); the predeclared severes are still reported to the user with the
re-pin before closure, as at T19.F02; the gate's series test is red until
that re-pin, which is the predeclared severe, not a waived check.

| Reading | Predeclaration |
| --- | --- |
| Gate and goal `mesh_hops`, `graph_relax_iters` | Up, severe expected; ceiling: goal mean `mesh_hops` per creature-tick at most 5 times T19.F03's per world; above it the result does not fit and is escalated before anything is presented |
| Gate and goal `vm_steps` | Down, severe expected (the founder has no VM node); no floor |
| `plasticity_updates`, `actions_applied`, `births` | No direction; gate `births` are expected up because the graph founder's cognition is cheaper per tick than the VM program's (the founder-only trajectory's births moved 185 to 323); the readings file records the founder's per-tick compute cost before and after so the move is attributed, and a severe is reported to the user with the re-pin |
| `passes`, `decided_passes` | Level `new`; goal mean passes per creature-tick at most 4 in every world (the founder runs 2 or 3); `Decided` passes reported against capped passes, no direction |
| `pass_cap_hits` | At most 10% of creature-ticks per goal world (T19.F02's rule); capped passes at most 10% of passes |
| Births probe (founder half): dead and sterile per mutated birth | Against T19.F03, ceiling twice its value for each; `reordered` and `recount` reported, no direction (first reading) |
| One-edge census (founder) | Fraction of unit edges adding exactly one action, and dead per edge, reported; W4 to W7 and W10 exact; no floor (first reading) |
| Drift walk changed, silent, dead per birth (depths 0 to 2,000) | Against T19.F03, no floor (withdrawn 2026-09-14); re-based knowingly |
| Goal persistence (final, minimum, plateau), lineage diversity, recruitment paths, memory sensitivity, T11.F14 and steering evolved halves, `mutation_supply` operator mix | No direction; a world whose final population falls below half of T19.F03's is investigated before the result is presented; the survey's contributing-node median (2) and route-variation share (3.3%) are read, not gated (T11.F10 owns the hypothesis) |
| Orchards seed 12 to 2,000 ticks | Final and minimum population beside seed 11's goal run; one seed, no direction |
| Energy flows and mortality `mesh_ramp`, `priority_bid` | Reported; `mesh_ramp` up where passes push a creature past 32 hops; no direction |
| Cognition wall per creature-tick (`cognition_ms`, `wall_clock_ms_per_creature_tick`) | Up; ceiling 4 times T19.F03's per world when the host matches (`MacBookPro.lan`), else `wall_clock: null` and reported; the goal run stays inside `wall_caps` |
| `config_digest`, gate and goal | Unchanged (`inputs_changed` false) |
| Founder digest, short-run identity, `FOUNDER_GENOME_SIZE_UNITS` | Move by construction; re-pinned and listed |

**Measured verdict.** Pending.

- Summaries: [gate](../../progress/features/t19-f04-vote-based-action-selection.json),
  [goal](../../progress/features/t19-f04-vote-based-action-selection-goal.json).
- Full readings: [`docs/progress/readings/t19-f04.md`](../../progress/readings/t19-f04.md).

## Success Criteria

- [ ] The executor runs the pass loop of invariant 1 in every execution mode;
      W1 to W19 pass as fixtures; tick and pass reasons, bars, passes, and
      committed actions are on the trace, the server payload, and the
      inspector.
- [ ] The deleted surfaces of invariant 8 are gone from the runtime, the ISA,
      the genome, the mutation engine, the server, and the TypeScript types
      (grep proof); both draw exclusions are lifted; `make check` is green.
- [ ] Every founder profile acts exactly as the truth table on its tests, the
      `limit 1` case, and the ring checks; the moved pins are re-pinned and
      listed; no config digest moves.
- [ ] Readings of invariant 12 recorded; docs of invariant 13 rewritten;
      mutation gate run with every survivor resolved; gate and goal run once
      with both epochs re-pinned in the closing commit and any severe reported
      to the user first.

## Notes for AI Agents

- Decision: the parameter surface index keeps today's `meta[i]` meaning minus
  the direction (`Eat` type at 0, `Reproduce` fraction and `StealEnergy`
  amount at 1); T17.F03 and T17.F04 name the unread slots.
- Decision: the `Decide` guard is the note's 2.4 rule, both clauses (a
  positive effective vote, or a non-empty queue with a positive `Terminate`);
  the note's W17 row hop count is superseded (invariant 2).
- Decision: exhaustion of any kind keeps the committed queue; an all-in bid
  keeps the queue with `priority_bid` equal to the energy paid.
- Decision: the per-unit mutation rate stays `0.005`; the founder's requested
  events per birth become `0.005 × the new founder size`, and the `0.55`
  equivalence is historical once the size moves.
- Decision: `reordered`/`recount` hold on every differing execution
  (kinds are `WorldAction` variants); a mix is neither.
