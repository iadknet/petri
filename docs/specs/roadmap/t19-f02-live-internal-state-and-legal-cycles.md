# T19.F02 — Live Internal State and Legal Cycles

**Status**: In Progress
**Last updated**: 2026-09-21
**Feature**: T19.F02
**Track**: [T19 — Mesh Action Selection and Live State](../../roadmaps/t19-mesh-action-selection-and-live-state.md)

## Goal

A nervous system's dynamics run faster than its behavior and a circuit may
reverberate. A mesh node may run any number of times in a tick on its live
state: no destination is ineligible because it already executed, a graph
visit starts from the last committed state and outputs, eligibility credit
adds per visit, and no tick-start snapshot exists. A cycle ends at a
survivable per-pass hop cap (`max_mesh_hops` 64) that keeps the queue and is
paid through T19.F01's ramp; the priority bid is settled once per tick.
Founders are byte-identical on their tests; loop productivity is measured
before and after.

## Non-Goals

- The vote surface, passes, the per-kind bar, `Decide`, and the within-tick
  bus (T19.F03 to T19.F05); under push semantics the whole tick is one pass
  and `TerminationReason` keeps its names.
- Whether committed actions survive energy exhaustion (note 1.8, T19.F04);
  here exhaustion still discards the queue.
- The hop-ramp constants: re-read for the cap choice, not changed.
- Graph-traversal correctness stays: `RemoveNode`'s bypass, `CopyNode`'s
  attachment proof, reachability's visited sets, the knockout bypass, the
  inspector's traversal.
- Removing the legacy trace fields `converged`, `stable_passes_count`, and
  `max_delta` (T19.F06; invariant 3 only changes what `max_delta` measures
  against); T11.F10's protocol on this closure.

## Inputs and Invariants

Sources of truth: the track row and its "Scope, T19.F02", "Decomposition
rules", "Epochs", and "Contract text" notes; the
[mesh action-selection review](../../strategy/mesh-action-selection-review-2026-09-20.md)
Sections 0, 1.2, 1.5, 1.7, 2.4 to 2.6, and 5; the T11 track's "T19 and the
execution contract" note; the code named in each invariant.

Options, settled by the note against R1 and R2: a visited filter or freeze
under any label (rejected, 0); a stall or epsilon exit (rejected, 2.4); a cap
that discards or kills (rejected, 2.5); a queue-keeping cap priced by the ramp
(adopted); one bid settlement over per-visit payment (adopted, 1.7); deleting
the convergence fields over ignoring them (`AGENTS.md`). Analog: recurrent
controllers integrate several steps per sensorimotor step (7.2).

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
   dispatch record's age and the once-per-world-tick trace decay. Unvisited
   modules hold; disconnected growth adds no visit. The trace's `max_delta`
   compares the candidate with the last committed outputs.
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
   `Default` and `normalize`, the seven tracked recipes, the frontend type,
   fixtures, tests, and `RuntimeSection.tsx` rows, the server API examples,
   and the runtime-config spec. `RuntimeConfig` is `deny_unknown_fields`, so
   a stored config still carrying them is rejected. Every case's
   `config_digest` changes (`inputs_changed`, never severe) and the v3-cli
   recipe-digest pins move.
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
      run the drift walk on the three goal worlds and store the before
      reading in `docs/progress/readings/t19-f02.md`.
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
- [x] Before and after loop-productivity readings (invariant 10):
      [`docs/progress/readings/t19-f02.md`](../../progress/readings/t19-f02.md);
      one ceiling exceeded (Canyon `cycle_carrying` dead-per-birth), below.
- [x] Benchmark summaries stored, hash/byte count and series entries checked;
      byte/hash figures in Performance below; no raw report staged.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: neural dynamics
run faster than behavior and circuits reverberate; the mechanism reaches
creatures through the body (what their brain computes, what the ramp charges),
never a sensor. Predeclared compute: no per-hop `HashSet` insert or lookup
(a small saving per hop), against more hops wherever an evolved genome
revisits or loops, bounded at 64 per creature-tick.

References: gate epoch `remove-complementary-nutrition.json` and previous
closure `t19-f01-per-tick-hop-ramp.json`; goal-worlds epoch
`t17-f02-unit-scale-introspection-goal.json` and previous closure
`t19-f01-per-tick-hop-ramp-goal.json`. Standing thresholds: counters flag at
10%, severe at 50%; wall flags at 25%, severe at 100%. The goal epoch is
re-pinned in the closing commit by construction (track "Epochs"); the gate
epoch is re-pinned if any gate `deterministic` counter differs from
T19.F01's, since the gate has births under the relaxed draw.

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

**Measured verdict.** Commands and exit statuses in the table below; goal ran once.

| Row | Verdict |
| --- | --- |
| `make bench PROFILE=gate FEATURE=t19-f02-live-internal-state-and-legal-cycles` | Exit 0 |
| `make bench PROFILE=goal FEATURE=t19-f02-live-internal-state-and-legal-cycles` | Outer `make` exit 2; CLI `bench --profile goal` exit 3 (`measurement_evidence.cli_exit`: "successful artifact-pair completion" with a severe comparison); both raw and summary artifacts written; `measurement_evidence.dirty: true` at `6c0f6c35` because the gate's own summary and its series entries had been written to the worktree minutes earlier, not from a code change |
| Gate counters vs gate epoch (`remove-complementary-nutrition.json`) | Not severe; all six counters `ok`, `pass_cap_hits` `new` (0.0) |
| Gate counters vs previous closure (`t19-f01-per-tick-hop-ramp.json`) | Not severe, all `ok`; `mesh_hops` 2.040238 vs 2.040249 (−0.000539%) and `graph_relax_iters` 0.999831 vs 0.999842 (−0.001100%) differ (within "unchanged" at the predeclaration's resolution, not 0 under the re-pin rule), the other four are 0.000000% — by the rule above the gate epoch is re-pinned to this feature's gate summary (`epoch_baseline` staged in `benchmark-series.json`) |
| Goal counters vs goal epoch (`t17-f02-unit-scale-introspection-goal.json`) | Not severe; all `ok` |
| Goal counters vs previous closure (`t19-f01-per-tick-hop-ramp-goal.json`) | **Severe**: `plasticity_updates` +80.48% (0.047907→0.086461); every other counter `ok`; predeclared consequence of legal cycles, not accepted here — goes to the user with the epoch re-pin |
| Goal `mesh_hops` per creature-tick ceiling (≤9.1) | 2.203591 — within ceiling |
| Goal `pass_cap_hits` per world, ceiling 10% of creature-ticks | seed 11: 84/11,791,629 = 0.00071%; seed 22: 211/10,463,518 = 0.00202%; seed 33: 478/16,747,134 = 0.00285%; total 773/39,002,281 = 0.00198% — all within ceiling |
| T11.F14 evolved-half `hop_cap_fraction`, ceiling ≤0.10 | 0/960 = 0 in all three worlds — within ceiling |
| `cycle_carrying` dead-per-birth, ceiling twice overall | Orchards/Confluence 0/343 = 0 (within); **Canyon country 1/148 = 0.00676 vs twice-overall 0.003664 — exceeds ceiling**; reported, not remediated |
| Drift walk dead/changed/silent vs T19.F01 (no floor) | Dead flat or down at every depth except Canyon depth 2,000 (0→1); changed flat at depths 0 and 22 in every world and at every Canyon depth, down at Orchards/Confluence 250 (45→43), 1,000 (9→7), and 2,000 (2→0), nothing up — a miss against "flat or up", recorded; silent no clear direction — see readings file for the full before/after table |
| Loop productivity (`cycle_carrying`, `revisiting`, `productive_cycle`) | `revisiting` no longer 0 (was 0 by construction pre-flip); all three lineage counts reported in the readings table, no direction required |
| Goal persistence (final population vs half of T19.F01) | seed 11: 5,111 vs 2,924.5; seed 22: 3,349 vs 1,753; seed 33: 8,909 vs 4,542 — all above half; no investigation triggered |
| Energy flow `mesh_ramp` per goal world | Orchards 4.435176, Canyon 11.140935, Confluence 25.238420 — above 0 in every world |
| `config_digest` (goal and gate) | Changed in both profiles (`inputs_changed`), as predeclared |
| Founder digest | Not independently re-checked by this run; Verification section reports it unchanged via `founder_only_trajectory_digest_is_pinned` |
| Wall time | Evolved-neighborhood total 438.66 ms (cap 180,000 ms); founder-neighborhood 87.61 ms (cap 10,000 ms); goal total wall 415,298.80 ms ≈ 6.92 min (cap 900 s / 15 min) — all within cap; all four comparisons (gate and goal, epoch and previous) record `wall_clock: null` (host `MacBookPro.lan` matches none of the references), so no wall flag could be computed |
| Memory sensitivity (`memory-sensitivity-v1`) | Perturbs committed `node_outputs` and `node_state` from this feature on; the series re-bases here as invariant 10 and the track note promise, no direction predeclared |

| Artifact | Raw bytes | Raw sha256 | Summary bytes |
| --- | ---: | --- | ---: |
| Gate | 96,081 | `4c5dc42acc85d3e5a50286ba2b7c761226f6dc5afbae207bdccd2b4e41a711ce` | 98,951 |
| Goal | 592,059,046 | `79d694a42bd21a26ec2289acb34ee3129d0f9501e867dd155f46e82c778ea0b8` | 7,698,308 |

- Summaries: [gate](../../progress/features/t19-f02-live-internal-state-and-legal-cycles.json),
  [goal](../../progress/features/t19-f02-live-internal-state-and-legal-cycles-goal.json).
  Raw reports stay local under the main checkout's ignored `.bench-artifacts/`.
- Full readings: [`docs/progress/readings/t19-f02.md`](../../progress/readings/t19-f02.md).

## Success Criteria

- [ ] No node is ineligible because it already executed; no `tick_start_*`
      field exists; visits read and commit committed state; traces add per visit.
- [ ] `max_mesh_hops` 64 keeps the queue and counts `pass_cap_hits`; the
      seven recipes carry 64; the bid is charged once per tick.
- [ ] `RetargetNodeTarget` and `AddRouteTarget` can produce a self-target;
      the convergence fields are gone everywhere named in invariant 8.
- [ ] Founders byte-identical and the founder digest holds; cycle fixtures,
      inverted tests, rewritten references, and both loop readings exist.
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
- Decision: pending the user at closure. Goal `plasticity_updates` +80.48% vs
  T19.F01 (not severe vs the epoch) is the predeclared "up" with no ceiling;
  hops per creature-tick are flat (2.20 vs 2.28) and revisits are rare, so it
  is a trajectory shift under the draw remap, as at T11.F19 (+82%) and
  T17.F02 (+78.9%), not per-visit multiplication. The goal-worlds epoch
  re-pins to this feature's goal summary by the track's Epochs rule, and the
  gate epoch by the gate rule above; both `epoch_baseline` values are staged
  in `benchmark-series.json`, the severe itself needs the user's acceptance.
- Deferred: bench counter registration is spread across `profiles.rs`,
  `comparison.rs`, `run.rs`, and `schema.rs` (`pass_cap_hits` touched all
  four); not consolidated here, T19.F04 reads all four before adding one.
- Exception: pending the user at closure. Canyon `cycle_carrying` dead per
  birth 1/148 (0.00676) exceeds twice overall (0.003664) by one death at
  depth 2,000; at the overall rate the chance of at least one death in 148
  births is 0.24, so the ceiling has no power at this sample. Reported, not
  remediated; the ceiling is unchanged.
