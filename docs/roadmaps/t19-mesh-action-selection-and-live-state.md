# T19 — Mesh Action Selection and Live State

**Status**: In Progress
**Last updated**: 2026-09-21
**Master**: [Program Roadmap](../roadmap.md)

## Goal

A creature decides what to do the way a nervous system does: its internal
state runs live for the whole tick, its brain may revisit any part of itself
while it deliberates, and its actions are chosen by competition among graded
votes rather than pushed by position in a program. The world it perceives is
frozen at the start of the tick; everything inside the creature (memory,
operator state, learned weights, the plan so far) changes as it computes.
The queue exists to reward forward planning: a creature that projects a route
to food, or several eats, or a burst escape, from what it knew at the start of
the tick, and gets it right, eats several times in one tick; one that plans
badly wastes actions and energy; one that does not plan takes no risk and eats
what it stumbles on. Afterward a new behavior is one edge into a vote sink, a
repeated action is a larger vote, a conditional plan is a vote that reads the
plan, and no mutation on one contribution has to coordinate with a push, a pop,
a terminal instruction, or a fixed output slot. The evidence and every decision
are in the
[mesh action-selection review](../strategy/mesh-action-selection-review-2026-09-20.md);
this track is that note's Section 5, and every acceptance claim below is one
of the worked cases in its Section 2.3.

## Track Success Criteria

- [ ] No mesh node is ever ineligible to execute because it already executed
  this tick, and no graph temporal state, output, or eligibility trace is read
  from or restored to a tick-start snapshot; the node-type contract, the graph,
  mesh, and ISA reference specs, and every fixture say so, and a creature that
  loops is bounded only by its own `Decide` vote, the per-pass hop cap, the
  per-tick hop ramp, the action cap, and its energy, none of which is death by
  itself.
- [ ] The runtime builds the action queue from vote sinks alone: no push, pop,
  execute instruction, action bank, or execute gate exists in the runtime, the
  ISA, the genome, the mutation engine, or the inspector, and a grep over
  `crates/`, `frontend/src`, and `docs/reference` finds no reference to them
  (historical specs keep theirs).
- [ ] The worked cases of the note's Section 2.3 hold on the runtime as
  fixtures: one fresh edge into an unused kind adds exactly one action (W4),
  into a used kind's other sink steers (W5), into a sink already voted on
  raises the count and the rank (W6, recorded as a recount, not a defect); a
  static kind vote of size `v` commits `ceil(v)` times (W7); a plan that reads
  the queue moves and then eats after arrival (W8) and turns (W9); a
  self-loop without an exit ends at the per-pass cap and pays the ramp (W11);
  a self-loop with one `Decide` edge from a bus counter ends its own pass
  (W17); a stray `Decide` edge before any vote is ignored (W18); a creature
  tunes how much it deliberates with a constant and a memory slot feeding
  Decide (W19); Terminate cannot end an empty tick (W15).
- [ ] The canonical founder and the forage-first profiles behave identically
  on the founder tests and the 2,000-ring direction check before and after the
  cutover (W1 to W3, including the queue-reading closure of the reproduce
  branch), and both epochs are re-pinned on the cutover's closing commit with
  the births probe (changed, silent, dead, sterile, reordered, recount), the
  one-edge census, the drift walk, the T11.F14 and steering evolved halves, and
  passes per tick recorded against predeclared directions.
- [ ] T11.F10's discovery and retention protocol has been read on the
  pre-track commit, on the T19.F02 closure, and on the T19.F04 closure, so the
  claim that the interface was a block on evolvable behavior is answered by
  measurement, either way.

## Executable Features

- [x] **T19.F01 — Per-Tick Hop Ramp** — Depends on: None
  - Goal: Sustained neural activity costs metabolism, so a creature that deliberates past a per-tick allowance of mesh hops pays a charge that rises with every further hop, summed over the tick, beside T03.F10's per-dispatch VM ramp.
- [x] **T19.F02 — Live Internal State and Legal Cycles** — Depends on: T19.F01
  - Goal: A nervous system's dynamics run faster than its behavior and a circuit may reverberate, so a mesh node may run any number of times in a tick on its live state, a cycle ends at a survivable per-pass hop cap that keeps the queue, and no tick-start snapshot of internal state exists.
- [x] **T19.F03 — Vote Surface as Inert Data** — Depends on: T19.F02
  - Goal: A motor pool is wired before it is ever driven, like a silent synapse, so the vote sinks (the four kinds, Terminate, and Decide), the `AddVote` opcode, the vote vector, and the per-kind counters exist everywhere a genome and a trace are represented while nothing reads or draws them.
- [ ] **T19.F04 — Vote-Based Action Selection** — Depends on: T19.F03
  - Goal: Motor programs compete and the winner habituates, as in basal ganglia selection and competitive queuing of serial order, so a tick becomes a sequence of passes, each ended by the genome's `Decide` vote or by the chain's end, in which the action kind with the largest effective vote commits one action and its bar rises by one unit, with the push, pop, execute, bank, and gate machinery deleted in the same commit and the founders re-expressed exactly.
- [ ] **T19.F05 — Decision-State Inputs** — Depends on: T19.F04
  - Goal: Corollary discharge, a brain sensing what it just decided and how tired it is, so the current and previous pass's votes, the per-kind commit counts, hops this tick, and the previous tick's outcome channels become drawable inputs.
- [ ] **T19.F06 — Retirement and Observability** — Depends on: T19.F04, T19.F05
  - Goal: Removal of a rule with no natural analog, so one execution model remains, the bank's last config field and readings are retired or marked historical, and the inspector shows passes.

## Notes for AI Agents

- Origin, 2026-09-21: created at the user's direction from the
  [mesh action-selection review](../strategy/mesh-action-selection-review-2026-09-20.md),
  an adversarial review of a Codex handoff, and revised the same day after two
  adversarial reviews of this track (a separate agent, then Codex); their
  findings are folded into the scope bullets below and into the note's Section
  2.3, which is now the transition contract with worked cases W1 to W19. The
  user's requirements in the note's Section 0 are authoritative over existing
  code, specs, tests, and history: no node is ineligible to execute because it
  already executed, no tick-start internal snapshot is read or restored,
  external perception alone is frozen for the tick, and neither restriction
  may return under the label of safety, neutrality, optimization,
  determinism, or evolvability repair. A spec or review that proposes either
  must say it conflicts with the user's requirements.
- Decisions recorded in the note and binding here (user, 2026-09-21): votes
  reset per pass with a rising bar; the bus persists within the tick and is
  zeroed at tick start (no second bus); `prev_shared_memory` stays as a
  delay-line input; within-tick Hebbian updates are allowed and priced by the
  ramp; the F05 inputs are the five listed and not a derived queued
  displacement; perception is never re-resolved at an imagined position; the
  note's Section 8 scratch probe was skipped, so T19.F04 is the first
  measurement of the interface and T11.F10 is the falsification reading; a
  genome ends a pass from inside a cycle with a `Decide` sink (one edge from
  state the loop changes, honored at a node boundary only when the pass end
  would do something), an exact-equality stall detector and an epsilon
  convergence rule were rejected, and a dedicated decide-threshold register
  was declined in favor of the constant-plus-memory construction (note
  Sections 2.3 W19 and 2.4).
- Two rules set at the reviews and confirmed by the user (2026-09-21), both
  in the note's Sections 2.3 and 2.5: the bar is per action *kind* (Eat, Move,
  Reproduce, Steal; Terminate and Decide are sinks, never committed, with no
  bar) and T11.F21's within-kind argmax picks the direction or type, because a
  per-sink bar made every positive direction commit in turn and the founder
  could not be ported; and a node revisited inside a pass replaces its own
  earlier contribution, so a static cycle re-judges rather than inflates a
  vote.
- Scope, T19.F01: `hop_ramp_allowance` and `hop_ramp_cost` in
  `RuntimeConfig`, charged per hop across the tick in the shared executor
  loop; the ISA and mesh spec cost sections. Founders unchanged. The allowance
  is chosen at spec time from a measurement of the maximum executed chain
  length on the three goal seeds (no stored reading exists; the live survey's
  median 7, max 19 is one world at one tick). Predeclared: gate and goal work
  counters unchanged, energy flow `mesh_ramp` reported, wall a flag, no epoch
  move.
- Scope, T19.F02: retire the visited filter and `tick_start_state`,
  `tick_start_outputs`, `tick_start_eligibility_traces`; visits read and
  commit the last committed operator state and outputs; eligibility activity
  adds per visit and decays once per world tick; pure Hebbian updates stay per
  visit; the priority bid is settled once at tick end; `max_mesh_hops` becomes
  a per-pass cap that ends the chain keeping the queue (under push semantics
  the whole tick is one pass); the exhaustion section of the mesh spec states
  per substrate what a failed visit leaves (learned weights applied and kept,
  as today; operator state and outputs not committed; effects and a VM
  dispatch's memory copy not applied; the bus unchanged); `RetargetNodeTarget`
  and `AddRouteTarget` may target the node itself; the dormant convergence
  config is deleted and stripped from the seven tracked world recipes and the
  frontend fixtures with the `config_digest` change named (`RuntimeConfig` is
  `deny_unknown_fields`); node-type contract property (3), graph spec Sections
  8 and 10, mesh spec Sections 2 to 4, and the ISA's ramp-reset and
  unvisited-target lines are rewritten; the T11.F05 fixtures invert under
  TDD; the memory-sensitivity perturbation moves to committed state; the five
  F15 runtime tests are replaced by cycle fixtures (W11, W12, W14). Founders
  byte-identical on their tests. Predeclared: the gate profile has births (256
  founders, 75 ticks), so the relaxed self-target draw and revisits move it:
  gate counters up with a flag ceiling and a possible gate re-pin; the goal
  trajectories move where evolved genomes revisit or loop (epoch re-pin);
  pass-cap events are counted per pass, not only as a final
  `MaxHopsReached` (the T11.F14 reading would otherwise miss capped passes
  that end as `NoDecision` or `ActionCapReached`), with a ceiling and a
  dead-per-birth reading for looping lineages; the route-exit construction
  is named (a second target plus a gate on state, since a sole self-target
  never exits; the one-edge `Decide` exit arrives with T19.F04); loop productivity is *measured* before and after, distinguishing
  structural cycles, executed revisits, and productive revisits, because since
  T11.F15 a self-looping node that pushes and halts already keeps its action
  and the pre-F02 productive fraction is not zero; looped memory versus delay,
  the T11.F14 evolved half, and the drift walk are read.
- Scope, T19.F03: `OutputSinkKind::ActionVote(sink)` for the four kinds'
  sinks, Terminate, and Decide, and `ActionParam(kind, i)`, appended to the
  fixed graph sink catalog after index 63 (the catalog is built by index and `FIXED_SINK_COUNT` is asserted) and
  excluded from `pick_random_surface`, which today enumerates every sink wired
  or not; `AddVote { sink, src }` in the ISA and interpreter and excluded from
  the fresh-instruction draw; `MeshSideOutputs` accumulates the vote vector
  (each node's latest contribution, summed over nodes) and per-kind counters
  that nothing reads; trace types, the server sample protocol, and a
  read-only inspector panel carry the vector. Predeclared: applied behavior,
  RNG consumption, and every work counter identical, which holds only because
  of the two draw exclusions; genome size unchanged because no wired sink
  exists; serialized genomes and the Debug-based trajectory fingerprints
  change because the catalog grew, and per-visit allocation and wall time rise
  because the effects pass iterates and traces every sink; the reviewer proves
  nothing reads or draws the surface.
- Scope, T19.F04: the pass loop of the note's Section 2.3 (votes cleared per
  pass, bus carried, internal state live, per-kind effective vote = best sink
  vote minus the kind's bar, one commit per pass, Terminate never committed
  and ending the tick only against a non-empty queue, `Decide` never
  committed and ending the pass at a node boundary only when the pass end
  would do something, `NoDecision` when no positive kind vote remains, ties
  to the kind committed in the previous pass then lowest index, per-kind
  overwrite parameter surfaces read at commit, the committed queue and the
  settled priority bid kept at energy exhaustion, and the per-pass reasons
  `Decided`, `PassCapReached`, `NoTargets`, `MissingNode` on the trace beside
  the tick reasons);
  in the same commit the push, pop, execute, and direction-bid opcodes, the
  action bank, `ActionSlot`, `ExecuteGate`, `NodeResult::terminal`, and
  `MutateActionSlotBehavior` are deleted from the runtime, the ISA, the
  genome, the mutation engine, and the TypeScript types, and the feature owns
  every compile-coupled consumer so the build stays green: the six frontend
  sources and seventeen frontend tests that render or fixture the bank (a
  minimal vote rendering; the redesign is T19.F06's), the v3-server sample
  protocol and assembler, the VM opcode end-to-end test, the T13
  recruitment-path fixtures and qualification, the steering structural
  reading, the mesh annotations' write classes; the two draw exclusions of
  T19.F03 are lifted (the one draw remap); V3Alpha1 and the forage-first
  profiles are re-expressed exactly as W1 to W3 (static votes, inhibitory
  edges from `can_reproduce`, the reproduce direction from the food ring, and
  one queue-reading node that closes the reproduce branch, because live energy
  falls between passes and could cross the gate) and verified on the founder
  tests and the 2,000-ring check; `FOUNDER_GENOME_SIZE_UNITS` and the
  per-unit rate assertion are re-pinned and T11.F20's "111" wording updated;
  the new sinks are counted in genome size; any retirement is recorded by
  reason; the mesh and ISA reference specs are rewritten for passes with the
  worked cases as fixtures. Readings: the births probe with reordered and
  recount classes; the one-edge census (W4, W5, W6, W7, W10); the drift walk;
  the T11.F14 and steering evolved halves; Orchards seeds 11
  and 12 to 2,000 ticks; passes per tick, `Decided` against capped passes,
  and cognition wall per creature-tick against a predeclared ceiling; both
  epochs re-pinned in the closing commit. Cost named
  up front: 67 Rust files reference the retired surface, including every hub
  file the mutation gate runs on, so the mutation gate and the frontend
  rewrite are budgeted in the spec.
- Scope, T19.F05: `ActionVotes` (the current pass's vector),
  `PreviousPassVotes` (the previous pass's final vector), `CommitCounts` (the
  per-kind bars), `HopsThisTick` (a dynamic introspection scalar beside
  `EnergyConsumedThisTick`), and the previous tick's outcome channels
  (`EnergyDelta`, `ActionSuccess`, `DamageDelta`, `OffspringSuccess`) as a
  perception-boundary self input. Neutral at birth, but not byte-identical:
  `random_input_reference` is a `0..22` enumeration and new variants move
  every mutated birth, as T11.F21's closure recorded; the draw remap is
  predeclared and the epoch decision is taken at closure; the sensor census is
  extended. A derived queued-displacement input was considered and declined by
  the user.
- Scope, T19.F06: `mutation.action_queue_cap` is retired with the
  `ActionQueue` input's width held at today's four slots as a named constant
  (today `cap * 3` in `mutation/compound.rs`; widening is a later remap) and
  stripped from the seven tracked world recipes and the frontend fixtures
  with the `config_digest` change named; the T13 recruitment records and the
  pre-cutover steering structural readings are marked historical in
  `docs/progress`; the inspector is redesigned for passes, bars, votes, and
  Terminate, including the T19.F05 inputs; `route_varies_with_input`
  distinguishes within-snapshot (state-driven) from across-snapshot
  (input-driven) variation; `docs/reference` and the T11.F15 spec
  cross-references are updated; the grep of criterion 2 is enforced.
  Predeclared: applied behavior and counters identical; the digest changes.
  F06 depends on F05 as well as F04 so that it is the track's completion join.
- Decomposition rules: one cutover of the action interface (T19.F04);
  T19.F02 changes execution semantics deliberately and is read on its own;
  T19.F01 is a cost change with founders neutral by construction; T19.F03 is
  additive with applied behavior unchanged; T19.F05 and T19.F06 add inputs and
  delete dead surface after the cutover. Between T19.F03 and T19.F04 the vote
  surface is dormant data, not a competing execution model, and T19.F04
  deletes the push machinery in the same commit that makes votes live so the
  compiler enforces one model. T19.F04 cannot be split further (the note's
  Section 5 says why); everything that could leave it has.
- Epochs: T19.F02 and T19.F04 move the goal trajectories by construction and
  re-pin under the master roadmap's epoch rule; T19.F02 may also move the gate
  (births under the relaxed self-target draw) and T19.F04 re-pins the gate
  because the founder changes; T19.F05 is a draw remap whose epoch decision is
  taken at closure; T19.F01, T19.F03, and T19.F06 predeclare applied behavior
  and counters unchanged and must not re-pin (T19.F03 and T19.F06 change
  fingerprints or the config digest, not behavior).
- Order of new starts: T19.F01 through T19.F06 in ID order, inserted on
  2026-09-21 ahead of T11.F20 in the master roadmap; T11.F10 now depends on
  T19.F06 and runs its discovery protocol on the pre-track commit, the T19.F02
  closure, and the T19.F04 closure; T17.F03 and T17.F04 now depend on T19.F04
  because the steal parameter and the eat bank are surfaces the cutover
  replaces; T18 is on hold until this track closes and is then re-planned on
  the vote surface (its layout needs no executor node and its router reading
  changes meaning), so T18.F01 depends on T19.F06, the completion join, and
  T13.F08 waits behind it; T13.F08's "acyclic reachable mesh" wording encodes
  the retired rule and is revisited when T18 is re-planned. T02.F01 (seasons)
  follows T11.F10 so demand is read on the new substrate.
- Contract text that encodes the rejected model and must be rewritten by
  T19.F02 and T19.F04, not merely re-tested: `docs/reference/v3-mutation-spec.md`
  node-type contract property (3) and its T11.F06 section, the dormancy-proof
  clause "even with single-visit filtering", `v3-graph-backend-spec.md`
  Sections 8 and 10, `v3-mesh-execution-spec.md` Sections 2 to 5, and
  `v3-vm-isa-spec.md` lines on the ramp reset and unvisited targets. The
  static visited sets in reachability analysis, the knockout bypass, and the
  inspector's mesh traversal are graph-traversal correctness and stay.
- Readings that become historical at T19.F04 and are re-based knowingly: the
  T13 recruitment-path records (hand-built action banks), the `steering-v1`
  structural reading ("executed node writes a bank"), the memory-sensitivity
  series (perturbed substrates change at T19.F02), and every seeded trajectory
  (the draw remaps at T19.F02, T19.F04, and T19.F05). None is a regression;
  each closure names them.
- These are mechanism features under the natural-analog rule; each goal line
  names its analog and none adds a world sensor. T19.F03 and T19.F06 are
  engineering steps of the mechanism, not measurement features, and carry
  their unchanged-behavior predeclarations as their reading.
