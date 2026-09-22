# T19.F02 — Live Internal State and Legal Cycles

**Status**: In Progress
**Last updated**: 2026-09-22
**Feature**: T19.F02
**Track**: [T19 — Mesh Action Selection and Live State](../../roadmaps/t19-mesh-action-selection-and-live-state.md)

## Goal

A nervous system's dynamics run faster than its behavior and a circuit may
reverberate. After this feature a mesh node may run any number of times in a
tick on its live state: no destination is ineligible because it already
executed, a graph visit starts from the last committed operator state and
outputs, eligibility credit adds per visit, and no tick-start snapshot of
internal state exists. A cycle ends at a survivable per-pass hop cap
(`max_mesh_hops`, now 64) that keeps the queue and is paid through T19.F01's
ramp; the priority bid is settled once per tick. Founders are byte-identical
on their tests. The change is read on its own: the goal trajectories move
where evolved genomes revisit or loop, and loop productivity is measured
before and after.

## Non-Goals

- The vote surface, passes, the per-kind bar, `Decide`, and the within-tick
  bus (T19.F03 to T19.F05); under push semantics the whole tick is one pass
  and `TerminationReason` keeps its names.
- Whether committed actions survive energy exhaustion (note 1.8, T19.F04);
  here exhaustion still discards the queue.
- The hop-ramp constants: re-read for the cap choice, not changed.
- Graph-traversal correctness stays: `RemoveNode`'s distinct-successor
  bypass, `CopyNode`'s attachment proof, the static visited sets in
  reachability, the knockout bypass, and the inspector's traversal.
- Removing the legacy trace fields `converged`, `stable_passes_count`, and
  `max_delta` (T19.F06; invariant 3 only changes what `max_delta` measures
  against); T11.F10's protocol on this closure.

## Inputs and Invariants

Sources of truth: the track row and its "Scope, T19.F02", "Decomposition
rules", "Epochs", and "Contract text" notes; the
[mesh action-selection review](../../strategy/mesh-action-selection-review-2026-09-20.md)
Sections 0, 1.2, 1.5, 1.7, 2.4 to 2.6, and 5; the T11 track's "T19 and the
execution contract" note; in `crates/v3-core/src`: `runtime/mesh.rs`,
`runtime/routing.rs`, `runtime/cgp/execute.rs`, `creature/state.rs`,
`runtime/plasticity/traces.rs`, `runtime/vm.rs`, `mutation/topology/routing.rs`,
`config/simulation.rs`, `simulation/tick.rs`, `neighborhood/mesh_execution.rs`;
`crates/v3-cli/src/bench/indicators.rs`.

Options, settled by the note against R1 and R2: the visited filter or a
tick-start freeze under any label (rejected, Section 0); a stall detector or
epsilon convergence as a cycle exit (rejected, 2.4); a cap that discards the
queue or kills (rejected, 2.5); a queue-keeping per-pass cap priced by the
ramp (adopted); the bid paid per visit against one settlement (adopted, 1.7);
convergence fields ignored against deleted (deleted, `AGENTS.md`'s
no-compatibility default). Analog: recurrent controllers integrate several
steps per sensorimotor step and the count is part of the controller (7.2).

1. **Routing.** Every existing target is eligible; the resolver is
   `resolve_gated_route` (earliest argmax of `gate_bias + gate score`), the
   `visited` set is gone, and a missing winner still terminates softly. The
   soft-default rows "No unvisited target remains" and "Route points to a
   visited node" become "No target" and are otherwise deleted.
2. **Per-pass cap.** `RuntimeConfig.max_mesh_hops` default `64` (was 1,024),
   `>= 1`, invalid values fall back to `64`; the check stays before dispatch,
   so a pass dispatches at most 64 nodes. Reaching it ends the evaluation
   `MaxHopsReached` with the queue kept (`into_actions_or_noop`, as today) and
   increments the new `WorkCounters.pass_cap_hits`, reported beside the other
   counters in gate and goal `counters` and as `pass_cap_hits` in the T11.F14
   block beside `hop_cap_hits` (the termination count, which it equals until
   T19.F04 makes passes plural). It is the seventh bench `COUNTER_NAMES`
   entry: an older summary on the current side reads zero, a reference
   without it compares `new` (never a literal zero, which the zero-reference
   rule would call severe), and from T19.F03 on it sits under the standing
   counter thresholds. The seven tracked `world-recipe-*.json` and the
   `RuntimeSection.tsx` row default set `64`; the goal recipes carry no
   runtime block and take the default.
   Calibration against T19.F01's constants: 64 is 2.9 times the survey's
   longest chain (22) and 7 times the goal maximum (9); one capped pass pays
   `1e-4 * 32 * 33 / 2 = 0.0528`, a NoOp's worth, survivable and not free;
   under T19.F04 eleven capped passes cost 22.6 per tick, lethal in one to two
   ticks (the T03.F10 standard). A 32-hop cap would be free under the
   allowance, which the note's "paid through the ramp" excludes.
3. **Live graph state.** A visit reads `candidate_state` from the committed
   `node_state` and `prev_outputs` from the committed `node_outputs` (the last
   successful visit, this tick or earlier); self and higher-index edges read
   that last committed output; a successful visit commits both. The fields
   `tick_start_state`, `tick_start_outputs`, and
   `tick_start_eligibility_traces` are deleted; `begin_tick` keeps only the
   dispatch record's age and the once-per-world-tick trace decay. Internal
   time is counted in visits, world time in ticks; unvisited modules hold;
   disconnected growth adds no visit. The trace's `max_delta` compares the
   candidate with the last committed outputs.
4. **Eligibility.** Each successful visit adds: `trace += activity`. Decay
   runs once per world tick in Phase 0 and is never rolled back; a failed
   visit leaves the trace untouched; a module first initialized this tick
   starts from zero. Pure Hebbian updates stay per visit (unchanged); the ramp
   bounds both because it bounds visits.
5. **Priority bid.** `SetPriorityBid` records `max(0, regs[src])` (non-finite
   reads as 0) as the creature's bid, last-write-wins, paying only its opcode
   cost; it no longer debits or exhausts. The bid is settled exactly once, in
   the shared loop, on every exit that is not `EnergyExhausted`: `paid =
   min(bid, energy)`; if `bid >= energy` the creature goes all-in, energy is
   `0.0`, the evaluation ends `EnergyExhausted` with `NoOp`,
   `DeathCause::PriorityBid`, and `priority_bid` 0.0; otherwise `energy -= paid`
   and `MeshOutput.priority_bid = paid`. A zero bid never settles. The
   `priority_bid` flow and death cause keep their keys; the settled bid is
   outside every dispatch, so `ComputeCostReport.vm_cost` no longer contains
   bids. A creature that exhausts on compute pays no bid.
6. **Exhaustion matrix.** What a failed visit leaves, per substrate; the mesh
   spec's Section 5 states it:

| Substrate | On exhaustion during the visit |
| --- | --- |
| Learned (pure Hebbian) weights | Applied and kept, as today |
| Operator state and outputs | Not committed; the last committed values stand |
| Eligibility traces | Unchanged (updated only after commit) |
| Graph effects: memory writes, actions, gates, params | Not applied |
| A VM dispatch's memory copy | Not committed |
| The bus (`upstream_slots`) | Unchanged |
| Queue and bid | Queue discarded, bid unpaid (T19.F04 revisits) |

7. **Mutation.** `RetargetNodeTarget`'s candidate union no longer excludes
   the source node itself (it still excludes the current target and missing
   IDs). `AddRouteTarget` no longer requires a non-self successor; the detour
   forwards to the old successor even when that is the node itself, so a sole
   self-target has no route exit (the resolver always selects an existing
   target); a route exit is a second target plus a gate on state. The RNG
   draw count per event is unchanged; applicability and outcomes are not,
   which is the predeclared draw remap.
8. **Config.** `max_graph_relax_iters`, `graph_convergence_epsilon`, and
   `graph_convergence_stable_passes` are deleted from `RuntimeConfig`, its
   `Default`, `normalize`, the seven tracked recipes, `frontend/src/types/config.ts`,
   `frontend/src/test/fixtures.ts`, `ControlBar.test.tsx`, the
   `RuntimeSection.tsx` rows, the two server API examples, and the
   runtime-config spec. `RuntimeConfig` is `deny_unknown_fields`, so a stored
   config still carrying them is rejected, not ignored. Every case's
   `config_digest` changes (`inputs_changed`, never severe), and the v3-cli
   recipe-digest pins move in the first implementer brief.
9. **Founders.** V3Alpha1 is a two-hop DAG with no bid and no stateful revisit,
   so the founder tests and `founder_only_trajectory_digest_is_pinned`
   (`63498f8d…`) are unchanged.
10. **Readings.** The T11.F14 block gains three per-lineage classes, each a
    count and a fraction of lineages: `cycle_carrying` (the reachable mesh
    from the entry contains a cycle, self-targets included), `revisiting` (some
    battery execution dispatched a node more than once), and
    `productive_cycle` (some execution dispatched a node that lies on a cycle
    of the reachable mesh and returned at least one action other than
    `NoOp`). The births probe partitions classified births under
    `by_cycle_carrying` into `cycle_carrying` and `acyclic` offspring, each
    with changed, silent, and dead counts, their merge equal to `any_events`. The memory-sensitivity probe perturbs the committed
    `node_outputs` and `node_state`; `memory-sensitivity-v1` keeps its
    definition and its series re-bases knowingly.
11. **Docs.** Rewritten, not merely re-tested:

| Reference | Change |
| --- | --- |
| `v3-mutation-spec.md` | Property (3) becomes "persistent state advances once per visit; visits per tick are bounded by the pass cap and priced by the ramp; unvisited modules hold; eligibility decays once per world tick"; the T11.F06 section and the `CopyNode` clause "even with single-visit filtering" rewritten; `RetargetNodeTarget` and `AddRouteTarget` rules drop their self exclusions |
| `v3-graph-backend-spec.md` | Sections 8, 10, 11, and 13 (clock and activity) |
| `v3-mesh-execution-spec.md` | Sections 2 to 5: algorithm without `visited`, termination, soft-default matrix, exhaustion per substrate, bid settlement |
| `v3-vm-isa-spec.md` | Section 3's ramp bullet, the `WriteRouteGate` "unvisited targets" line, Section 9 |
| `v3-tick-orchestration-spec.md` | Phase 0 paragraph |
| `v3-runtime-config-spec.md`, `v3-server-api-protocol-spec.md` | Rows, disposition text, and examples for the deleted fields and the new cap default |

## Implementation Tasks

- [x] Instrument first, on the unflipped executor: `pass_cap_hits`, the
      three cycle classes, and the births-probe partition (invariants 2, 10);
      run the drift walk (the goal profile's `drift_depth` instrument, through
      an existing entry point or a throwaway harness as T19.F01's readings
      did) on the three goal worlds and store the before reading in
      `docs/progress/readings/t19-f02.md`.
- [x] `cargo test -p v3-core --test viability` first (tick-loop mechanics
      change), then TDD on the tests below.

| Tests | Change |
| --- | --- |
| The five `runtime/f15_tests.rs` tests and `f15_self_loop_dispatches_once` (`runtime/mesh.rs`) | Replaced by cycle fixtures. W11: a Halt-only self-loop runs 64 hops, `NoOp`, ramp 0.0528 at the defaults, `pass_cap_hits` 1. W12: a self-looping non-terminal push node (`PushAction`, `Halt`) fills the queue to `max_actions_per_turn` and keeps it at the cap. W14: a two-target cycle with a gate on a memory counter exits after the counted visits and keeps its actions |
| `repeated_and_skipped_visits_use_frozen_base_and_last_success`, `stateful_module_and_blank_neighbor_each_step_once_per_world_tick`, `temporal_probe_detects_each_substrate_and_preserves_live_state`, `birth_applied_reproduction_keeps_parent_and_resets_all_newborn_credit`, the mode-parity assertions on `tick_start_*` in `runtime/mesh.rs` | Inverted to per-visit advance and committed state |
| T11.F05 fixtures D1 to D3, E1 to E3 (`tests/temporal_fixtures.rs`) | The fixture module's doc comments rewritten to the per-visit clock (the closed T11.F05 spec and its table stay as history; this spec is the record); their values stand because each visits once per tick; revisit companions added: an integrator visited twice per tick steps twice, a trace visited twice adds twice |
| Priority bid (`runtime/mesh.rs`, `vm.rs`) | Single charge, all-in, unpaid on compute exhaustion, last-write-wins across a revisit |
| Mutation (`mutation/topology`) | A self-target is drawn by each operator; RNG draw count unchanged |
- [x] Executor and state: invariants 1 to 6 in `mesh.rs`, `routing.rs`
      (resolver call site), `cgp/execute.rs`, `state.rs`, `traces.rs`,
      `vm.rs`; three-mode parity (production, observed, traced) kept.
- [x] Mutation operators (invariant 7) with tests that a self-target is drawn.
- [x] Config deletion (invariant 8) across crates, recipes, frontend, and
      the recipe-digest pins.
- [x] Memory-sensitivity probe on committed state (invariant 10).
- [x] Reference docs and frontend types (invariants 8 and 11).

## Verification

- [x] Viability: `cargo test -p v3-core --test viability` -> 28 passed,
      before and after the flip.
- [x] Focused tests: the table above plus the config tests -> commands and
      counts in [`docs/progress/readings/t19-f02.md`](../../progress/readings/t19-f02.md).
- [x] No snapshot survives: `grep -rn 'tick_start' crates frontend/src`
      and `grep -rn 'graph_convergence\|max_graph_relax_iters'` over
      `crates`, `frontend/src`, `world-recipe-*.json`, and `docs/reference`
      both return nothing.
- [x] Founder pin: `founder_only_trajectory_digest_is_pinned` unchanged on
      `63498f8d36346079f8827c382e2978510357b374ca37759af857afa263f2d0be`.
- [x] Recipe digest pins moved (`d9a4dc8c…`, `d1aff3f5…`, `b24c1225…`) and
      `cargo test -p v3-cli` green.
- [ ] Whole-repo gate: `make check` exit 0 on the tested commit.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      and every survivor resolved as killed, equivalent, or deferred, listed here.
- [ ] Before and after loop-productivity readings (invariant 10) in the
      readings file: the drift walk on the unflipped executor (stored)
      against the closure goal report's `drift_depth` rows and evolved half.
- [ ] Benchmark summaries stored at
      `docs/progress/features/t19-f02-live-internal-state-and-legal-cycles.json`
      and `-goal.json`, local raw hash/byte count and verification time
      checked, series entries appended, no full report staged.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: neural dynamics
run faster than behavior and circuits reverberate; the mechanism reaches
creatures through the body (what their brain computes, what the ramp charges),
never a sensor. Predeclared compute: no per-hop `HashSet` insert or lookup
(a small saving per hop), against more hops wherever an evolved genome
revisits or loops, bounded at 64 per creature-tick.

References: the gate against the gate epoch
(`remove-complementary-nutrition.json`) and the previous closure
(`t19-f01-per-tick-hop-ramp.json`); the goal world set against its epoch
(`t17-f02-unit-scale-introspection-goal.json`) and the previous closure
(`t19-f01-per-tick-hop-ramp-goal.json`). Thresholds are the standing ones:
counters flag at 10% and severe at 50%; wall flags at 25% and severe at 100%.
The goal epoch is re-pinned in the closing commit by construction (track
"Epochs"); the gate epoch is re-pinned if any gate `deterministic` counter
differs from T19.F01's, since the gate has births under the relaxed draw.

| Reading | Predeclaration |
| --- | --- |
| Gate counters (`mesh_hops`, `vm_steps`, `graph_relax_iters`, `plasticity_updates`, `actions_applied`, `births`) | Up or unchanged; a flag is the predeclared consequence of births under the relaxed self-target draw; a severe on any gate counter is an escalation, not an accepted cost |
| Goal `mesh_hops`, `vm_steps`, `graph_relax_iters`, `plasticity_updates`, `actions_applied` | Up; ceiling: mean `mesh_hops` per creature-tick at most 4 times T19.F01's per world (9.1 against 2.28); above it the result does not fit and the cap is re-read before anything is presented |
| Goal `pass_cap_hits` per world | Reported; ceiling 10% of creature-ticks; the T11.F14 evolved half's `hop_cap_fraction` at most 0.10 |
| Births probe: dead per birth among `cycle_carrying` offspring | Reported; ceiling twice the world's overall dead per birth |
| Drift walk changed, silent, dead per birth (depths 0 to 2,000) | Against T19.F01, no floor (withdrawn 2026-09-14): dead flat or down (a cycle no longer returns `NoOp`), changed flat or up, silent no direction |
| Loop productivity (`cycle_carrying`, `revisiting`, `productive_cycle`) | Before: `revisiting` is 0 by construction; `productive_cycle` is expected above 0 where cycles exist (T11.F15's push-and-halt self-loop keeps its action) and is measured, not assumed; after: all three reported, no direction |
| Goal persistence (final, minimum, plateau), lineage diversity, recruitment paths, memory sensitivity | No direction; a world whose final population falls below half of T19.F01's is investigated before the result is presented |
| Energy flow `mesh_ramp` per goal world | Above 0 where any creature exceeds 32 hops; `priority_bid` flow no direction |
| Goal and gate `config_digest` | Changes in every case (three fields fewer): `inputs_changed`, not a trajectory change |
| Founder digest | Unchanged |
| Wall time, both profiles | Up with hops; a flag is tolerated, a severe is investigated against the per-hop cost before it is presented |

A severe on a goal work counter is the predeclared consequence of legal
cycles and goes to the user with the epoch re-pin; it is not accepted here.

**Measured verdict.** Pending the run.

- Summaries: [gate](../../progress/features/t19-f02-live-internal-state-and-legal-cycles.json),
  [goal](../../progress/features/t19-f02-live-internal-state-and-legal-cycles-goal.json).
- Full readings: [`docs/progress/readings/t19-f02.md`](../../progress/readings/t19-f02.md).

## Success Criteria

- [ ] No node is ineligible to execute because it already executed, and no
      `tick_start_*` field exists; a graph visit reads and commits the last
      committed state and outputs, traces add per visit.
- [ ] `max_mesh_hops` defaults to 64, ends the chain keeping the queue,
      counts `pass_cap_hits`, and the seven recipes carry 64.
- [ ] The priority bid is charged exactly once per tick, at settlement.
- [ ] `RetargetNodeTarget` and `AddRouteTarget` can produce a self-target.
- [ ] The three convergence fields are gone everywhere named in invariant 8.
- [ ] Founders are byte-identical on their tests and the founder digest holds.
- [ ] The cycle fixtures, inverted tests, and rewritten reference sections
      exist; before and after loop-productivity readings are stored.
- [ ] Mutation gate run with every survivor resolved; gate and goal run with
      the epoch handling above.

## Notes for AI Agents

- Decision: a stored config carrying `max_graph_relax_iters`,
  `graph_convergence_epsilon`, or `graph_convergence_stable_passes` is
  rejected by `deny_unknown_fields`; no alias or ignore shim is added.
- Decision: `max_mesh_hops` 64 is the per-pass cap with the T19.F01 constants
  unchanged; a later change of either re-reads invariant 2's calibration.
- Deferred: the legacy graph trace fields `converged`, `stable_passes_count`,
  and `max_delta` are left for T19.F06.
