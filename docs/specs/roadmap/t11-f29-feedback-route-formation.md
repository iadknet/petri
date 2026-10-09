# T11.F29 — Feedback Route Formation

**Status**: In Progress
**Last updated**: 2026-10-09
**Feature**: T11.F29
**Track**: [T11 — Brain Genotype-Phenotype Map](../../roadmaps/t11-brain-genotype-phenotype-map.md)

## Goal

A mesh can grow a route branch back to a node on its own working path, itself
included, in one ordinary topology mutation: a new operator, `AddFeedbackRoute`,
appends a losing branch with its own gate write whose destination is the source
node or a node one or two hops upstream on the parent's executed path. The
branch is born a static loser one unit below the incumbent, so the incumbent
keeps the route for every gate value in the sensor range and silence at birth
is measured, not disguised; a loop that repeats a step or keeps state is one
mutation away instead of a by-product of an eight-node slice copy, and its
later activation is ordinary small-step variation (`MutateGateBias`, a
gate-source or incumbent change).

## Non-Goals

- No runtime change: the 64-hop pass cap, T19.F01's ramp, the `Decide` guard
  and T19.F02's cycle semantics are a loop's cost and stay.
- No change to any T11.F15 operator, its eligibility or its measured rows.
- No new instrument, config field, knob or signal; no inspector styling of
  back edges; no claim that loops help discovery (T22); T11.F15's exclusions
  (per-hop cost, addressing, fan-out, persistent routing position) stand.

## Inputs and Invariants

Sources of truth: the track row, its success criterion ("a mesh gains a loop
through one ordinary mutation that is silent at birth, with loop-carrying
offspring inside T19.F02's dead-per-birth ceiling") and the "Loop formation,
2026-10-09" track note with its five constraints; T11.F15's branch
construction (`apply_add_route_target`, `crates/v3-core/src/mutation/topology/routing.rs`),
T19.F02's cycle legality and readings, T19.F04's pass loop; the code named
below. Planning references (T20.F16, the previous closure and both epochs) are
tabulated in [readings](../../progress/readings/t11-f29.md).

The track note records why no topology operator but the two slice copies can
close a loop today (every other one routes downstream or inserts a forwarding
detour; `clone_and_remap_slice` alone keeps external targets) and the live
reading of 15 loop-carrying creatures in 390. Options and prior art:

| Option | Disposition |
| --- | --- |
| Prior art | NEAT's add-connection mutation admits recurrent links (Stanley and Miikkulainen 2002, cited in T11.F15); Recurrent CGP gates them by a per-connection probability (Turner and Miller, PPSN 2014), whose counterpart here is the operator weight |
| A new operator that appends a back edge as a losing branch | Adopted: its own `operator_rows` and births rows, T11.F15's rows untouched |
| Upstream nodes in `RetargetNodeTarget`'s destination set | Rejected: a retarget of the static incumbent replaces the live route with a loop at birth (not silent) and moves T11.F15's measured row |
| `AddRouteTarget`'s detour forwarding to an upstream node | Rejected: the detour's equivalence is what makes that row 1.000 silent; a shared row loses attribution |
| Raising `large_copy_weight_percent` | Rejected: a loop stays a by-product of an eight-node copy and genome size moves |
| A package | None applies; the existing helpers suffice |

1. **Operator.**

| Item | Value |
| --- | --- |
| Variant, position | `TopologyOperator::AddFeedbackRoute`, listed after `AddRouteTarget` in `ALL` |
| Weight, class | 10 (structural tier); `ComplexityEffect::Increasing`, so genome-size pressure filters it like every growth operator |
| Key | `MutationOperator::TopologyAddFeedbackRoute`, `as_key` `Topology.AddFeedbackRoute`, `EditSurface::Split`, no added-node input tracking (it creates no node) |
| Totals | Base topology total 221 (`ChangeEntryNode` 1 in 221; tuned total at most 22,100; 198.5 at the default `large_copy_weight_percent` 25) |

2. **Eligibility and draw.** The source is selected through the
   `TargetSelector` like every biased topology operator. A node is eligible
   when all of these hold; otherwise the operator skips with
   `NoApplicableTarget` and no partial edit, as `AddRouteTarget` does:

| Condition | Rule |
| --- | --- |
| Static incumbent | `static_incumbent` resolves to an existing node, its `gate_bias` is finite, and the backend has no write to its slot (`writes_gate` false), so the incumbent's runtime score is its bias |
| Slot | The first slot unused by the node's targets and not written by its backend (`safe_predecessors`' rule), so an orphan write left by an earlier removal (preserved by T11.F15) never feeds the new branch; none free skips |
| Gate write possible | VM: `register_count > 0`; Graph: a `RouterGate(slot)` sink exists for that slot |
| Destinations | `feedback_destinations` (below) is non-empty |

   Draw order after the source: the gate source (VM register uniform, or
   `random_graph_source` with full sub-value sampling), then the destination
   uniform over the candidate list sorted by node index and deduplicated by
   `NodeId`, so the pick is a function of genome content, the parent's sets
   and the seed alone (T10.F11); a skipped attempt consumes no draw after the
   source selection.

3. **Destinations: near, and on the executed path.** `E` is the
   `TargetSelector`'s executed set (T11.F17: the parent's dispatch record
   within `mutation.executed_window_ticks`, mapped to the child's indices by
   `BirthMembership`; the battery's dispatched nodes in the `neighborhood`
   harnesses), exposed by a read accessor. When the source is in `E`, `P1` is
   the nodes of `E` with a target naming the source and `P2` the nodes of `E`
   with a target naming a node of `P1`; when the source is outside `E`, `P1`
   and `P2` are empty.
   `feedback_destinations = ({source} ∪ P1 ∪ P2) − {ids already among the
   source's targets} − {missing ids}`, collected as `NodeId`s from node
   indices: the current predecessor edges within two hops between
   parent-executed nodes, plus the source regardless of its own membership
   (the selector's fallback can pick an unexecuted source, which then
   self-loops only), T11.F15's locality taken upstream (track note,
   constraint 2). `E` holds node membership, not traversed edges; a node the
   parent never executed is never a destination other than the source itself,
   and a parent with no dispatch record (a newborn, a fixture without an
   executed set) yields only the source. A node that is both predecessor and successor (the source
   already lies on a cycle) qualifies; successors and siblings that are not
   predecessors are `RetargetNodeTarget`'s set.

4. **Born a static loser, with its own gate write.** The event appends
   `RouteTarget { target_id: destination, slot, gate_bias }` as the last
   target, where `gate_bias` is the `f32` obtained by subtracting 1.0 from
   `incumbent.gate_bias` and stepping down one ulp until
   `gate_bias + 1.0 <= incumbent.gate_bias` holds in `f32` arithmetic
   (`0.1 − 1.0 + 1.0` rounds above `0.1`), with no clamp (biases are not
   range-validated; only `MutateGateBias` clamps its own step), and, in the
   same event, the source's
   own write to that slot by T11.F15's construction: VM
   `WriteRouteGate { slot, src }` inserted before the first `Halt` (or at the
   end) with reference repair; Graph one weight-1 edge into the
   `RouterGate(slot)` sink. The backend is cloned, edited and committed with
   the target, so a failed insertion skips atomically (propagated as
   `NoApplicableTarget`, as in `AddRouteTarget`, not an eligibility
   pre-check). Proven: under zero
   gate scores the incumbent wins (`resolve_gated_route`, strict `>`), and
   with its slot unwritten its runtime score is its bias, so (f32 addition
   being monotonic) the branch loses on every input whose gate value on the
   new slot is at most 1.0, which holds for the normalized sensor leaves
   (ratios, clamped similarities and signed unit components in `[−1, 1]`;
   `reducers.rs`, `static_inputs.rs`). Not claimed, following T11.F15 ("a
   static loser is not necessarily a runtime loser"): a VM register, a graph
   compute output or a memory source can exceed 1.0 at birth and select the
   branch; that birth reads `changed` or `dead` in the births probe and counts
   against the founder-row and dead-per-birth predeclarations. A T11.F15 tie
   is not used because a loop has no forwarding detour to make a winning bid
   silent: the founder's only eligible source is node 0, its only destination
   itself, and a self-loop there runs to the pass cap without a vote, so every
   positive sensor value would be a dead founder birth. Activation needs a
   gate value above 1.0, accumulated `MutateGateBias` changes on the branch or
   the incumbent that close the gap, a gate-source change or a weaker
   incumbent; it is measured, never assumed.

5. **Genome and cost.** The event adds one target unit and one instruction or
   edge unit (two with a newly wired sink) to `genome_size()`, so the lineage
   pays the existing carrying, gestation and supply laws. An active loop pays
   hops under the 64-hop pass cap and the ramp past 32 hops per tick (T19.F04:
   640 hops cost 18.5 energy) and ends its pass by the `Decide` guard or
   `PassCapReached`.

6. **Determinism and identity.** Adding an operator to `ALL` remaps every
   topology draw, so gate and goal trajectories move (track note, constraint
   5). No config field is added, so every `config_digest` and the v3-cli
   recipe-digest pins are unchanged; the founder genome is not edited, so
   `founder_only_trajectory_digest_is_pinned` and `FOUNDER_GENOME_SIZE_UNITS`
   (97) are unchanged.

7. **Compile-coupled consumers, all owned here.**

| Consumer | Change |
| --- | --- |
| `mutation/topology/mod.rs` | `ALL`, `weight`, `complexity_effect`, the `ALL.len()` guard, the weight comment; `tests.rs` `TOTAL_WEIGHT` 211 → 221 |
| `mutation/types/mod.rs`, `types/tests.rs` | `MutationOperator` variant, `as_key`, the mapping test |
| `mutation/engine/mod.rs` | `topology_operator_key` |
| `neighborhood/recruitment.rs` | `EditSurface::of` and the operator grouping where `AddRouteTarget` sits |
| `mutation/reachability.rs` | A read accessor for `TargetSelector`'s executed set |
| Operator-keyed maps (`operator_rows`, `discarded_*_by_operator`, outcome rows) | Gain the key; no schema change. The frontend names no topology operator |

8. **Docs.** `docs/reference/v3-mutation-spec.md`: the operator list, a
   connection-semantics bullet for `AddFeedbackRoute` (destinations, margin,
   gate write, skips), the 211 → 221 macro odds, and its place as a
   connection operator in the taxonomy.

## Telemetry

No new signal or configuration (no counter, gauge, span attribute, config
field or switch); a run shows existing signals moved by the changed draws.

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `petri.run.mutation_events_applied`, `_attempted`, `_skipped`, `_reachable_target`, `_unreachable_target` | counter, existing | the same applied events; `AddFeedbackRoute` events count under them | `run.started` body carries the unchanged `mutation` config |
| `operator_rows`, `discarded_*_by_operator` and every other operator-keyed map, keyed `Topology.AddFeedbackRoute` | goal-report rows, existing maps | births under the new operator | `config_digest` per case, unchanged |
| `cycle_classes`, `by_cycle_carrying`, `pass_cap_fraction`, `pass_cap_hits`, `mesh_ramp` | goal-report blocks and counters, existing (T19.F02, T19.F04) | loops the operator forms and selection retains | same |

## Implementation Tasks

- [x] TDD: tests first for the properties below.

| Property | Cases |
| --- | --- |
| Destination set | Itself included; executed predecessors within two hops only; never a current or missing target; an empty executed set, or a source outside it with executed nodes targeting it, yields the source alone; a source already on a cycle; sparse and reordered `NodeId`s; duplicate incoming edges |
| Born losing | Static incumbent unchanged after the event; over property-drawn finite incumbent biases, `0.1` included, a gate value in `[−1, 1]` on the new slot does not select the branch against an unwritten incumbent in `resolve_gated_route`, a value above 1.0 can |
| Gate write | Present on the new slot on both backends; the slot skips orphan writes |
| Atomic skips | Zero-register VM, missing `RouterGate` sink, no free slot, no candidates, non-finite incumbent bias, written incumbent slot: `NoApplicableTarget`, genome unchanged |
| Founder | Node 0 eligible with destination set `{0}`; node 1 ineligible |
| Loop and replay | `mesh_cycle_nodes` empty before and non-empty after one event; identical offspring and RNG continuation for identical inputs |
- [x] Implement `apply_add_feedback_route` in `mutation/topology/routing.rs`
      reusing `static_incumbent`, `writes_gate` (invariant 2's slot rule, not
      `lowest_unused_slot`), `insert_new_instruction_with_reference_repair`
      and `random_graph_source`; wire the operator through every consumer in
      invariant 7.
- [x] One battery fixture: a production `AddFeedbackRoute` on T11.F15's
      conditional fixture leaves the battery's actions identical and makes it
      `cycle_carrying` but not `revisiting`; production follow-up events
      (`MutateGateBias` on the branch or incumbent, seeded as needed, or a gate
      source reading above 1.0) make the same fixture `revisiting` on both
      backends, with the number of events each path needed recorded in the
      readings.
- [ ] Update `docs/reference/v3-mutation-spec.md` (invariant 8) and the
      `mod.rs` weight comment; then the Verification items in order.

## Verification

- [x] `cargo test -p v3-core --test viability` -> exit status, before and
      after. Exit 0 both (28 passed).
- [ ] Focused tests: `cargo test -p v3-core topology` -> exit status and
      count, test names in [readings](../../progress/readings/t11-f29.md);
      `cargo test --workspace --no-fail-fast` -> result, with any series-test
      failure attributed to a user-accepted re-pin. Topology: exit 0, 101
      passed. Workspace: exit 101, only the gate series test (`pass_cap_hits`
      +271%, severe) awaits the user; eleven seeded pins re-pinned (readings).
- [ ] `make check` -> exit 0, tested commit in the readings.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      and every survivor resolved as killed, equivalent, or deferred, listed
      here.
- [ ] Benchmark summaries stored at
      `docs/progress/features/t11-f29-feedback-route-formation.json` and
      `-goal.json`, local raw hash and byte count and verification time in the
      readings, series entries point to the summaries, no full report staged.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: feedback
projections, a later stage of a nervous system wiring back to an earlier one
(recurrent collaterals, cortico-thalamic return paths); it reaches creatures
through inherited brain wiring and dispatch, what the genome routes and what
the ramp charges, never a sensor. Predeclared compute: one more operator in a
weighted draw; per attempted event, eligibility over the nodes (incumbent,
slot and write checks, the two-hop upstream scan over the executed set) and
one backend clone on the chosen source, work of the order `AddRouteTarget` and
`CopyNode` already do; per applied event, one route entry and one gate write.
An active loop costs hops under the unchanged cap and ramp, so `mesh_hops`
rises only where selection retains loops. No severe is budgeted; a severe is
first investigated against the per-world cycle-class and `operator_rows`
readings and, when it is selection or composition and not compute, goes to
the user with those readings for acceptance and re-pin, never absorbed.
References for
both profiles: T20.F16's summaries, the gate epoch, the goal-worlds epoch and
the previous closure at once. Standing thresholds: counters flag at 10%,
severe at 50%; wall flags at 25%, severe at 100%; `wall_clock: null` on a
host mismatch. **Both epochs re-pin in the closing commit** because the draw
remap moves every trajectory (track note, constraint 5); severes are still
reported before closure. The goal profile runs once; every indicator not in
the table is reported with no direction; per-world values are Orchards 11 /
Canyon 22 / Confluence 33.

| Reading | Reference (T20.F16) | Predeclaration |
| --- | --- | --- |
| Gate counters (`mesh_hops`, `vm_steps`, `graph_relax_iters`, `plasticity_updates`, `actions_applied`, `births`, `passes`, `decided_passes`, `pass_cap_hits`) | T20.F16 gate | `deterministic` differs by the draw remap; every counter within the flag thresholds expected, no severe budgeted |
| Goal `mesh_hops` per creature-tick | 10.6037 / 6.7451 / 8.7616 | Up where loops are retained; ceiling twice the reference per world; a reading above it is escalated before anything is presented |
| Goal `pass_cap_hits` per creature-tick | 0.001139 / 0.001659 / 0.001370 | Ceiling 10% of creature-ticks per world (T19.F02's rule) |
| T11.F14 evolved half `pass_cap_fraction` (T19.F04's name for the track note's `hop_cap_fraction`) | 0/7,606, 0/3,399, 0/4,078 | At most 0.10 per world (T19.F02's rule) |
| Goal `mesh_ramp` deaths per creature-tick; `energy_flows.mesh_ramp` | 242/15,904,544, 95/13,027,583, 126/17,067,382; 15,298.30 / 8,500.99 / 10,384.72 | Deaths at most twice the reference per world; the energy flow reported, no direction |
| Births probe (`neighborhood_read`): `by_cycle_carrying.cycle_carrying` dead per applied birth | 0/4, 0/0 (`Undefined`), 0/1 | At most twice the world's `any_events.dead_fraction` (T19.F02's ceiling; its one selection reading is Canyon 1/148 over the ceiling by one death); `Undefined` reported as such, never a pass |
| Births probe `any_events.dead_fraction`, pooled over the three worlds | 54/7,402 = 0.007295 (4/2,644, 42/2,732, 8/2,026) | Not above the pooled reference ("dead per birth does not rise"); per world reported |
| Births probe `any_events.silent_fraction`, pooled | 4,724/7,402 = 0.6382 | Not below 0.618, an explicit two-point tolerance ("the silent share holds"), chosen above the operator's estimated share of classified births (about 1.5%: 10 of the tuned topology total 198.5 at `large_copy_weight_percent` 25, 20% of events mesh events, about 1.4 events per classified birth; an estimate, not a bound, since inapplicable-operator redraws and applied-event conditioning raise it); a lower reading is read beside the operator's single-operator `operator_rows` as diagnostic evidence, not attribution, and goes to the user under the blocker rule |
| `neighborhood_read.changed_per_all_births` | 0.2086 / 0.1766 / 0.1396; standing floors 0.1464 / 0.1300 / 0.1434 (T14.F12) | Not below the standing floor on any world; a reading below one goes to the user under the blocker rule, as at T20.F16 |
| Founder `operator_rows` `Topology.AddFeedbackRoute` | none (new row); founder `any_events` 0/139 dead | Applied trials above zero; silent at least 0.95; founder `any_events` dead stays 0 of its births. Every other founder row within its series as untouched operators |
| Drift walk `cycle_classes.cycle_carrying_lineages` of 50 (Orchards and Confluence share one walk; Canyon is the other) | depths 22 / 250 / 1,000 / 2,000: 2 / 5 / 11 / 15 and 1 / 4 / 11 / 14 | **The row's intended outcome:** above the reference at depths 250, 1,000 and 2,000 on both walks ("the loop share rises"); depth 22 reported. A depth missing the increase is a contrary result reported to the user with the operator's applied count, not absorbed |
| Drift walk `by_cycle_carrying.cycle_carrying` dead per applied birth, at each depth on each walk | 0/61, 0/147, 1/256 and 0/1, 0/179, 0/240 at depths 250, 1,000, 2,000 | At most twice that depth's `any_events.dead_fraction` on that walk (T19.F02's ceiling applied per checkpoint; the reference already exceeds it at Orchards/Confluence depth 2,000 by one death, 1/256 against 2/774); a depth over the ceiling is reported to the user with its counts, as at T19.F02, never pooled away; pooled counts (1/464, 0/420) are supplementary |
| Drift walk `revisiting_lineages`, `productive_cycle_lineages`, `pass_cap_fraction`, `changed_per_all_births`, `dead` per depth | tabulated in the readings | Reported, no direction; the walk has no selection, so no cap ceiling applies to it |
| T11.F14 evolved half `cycle_carrying`, `revisiting`, `productive_cycle` (12 sampled genomes per world) | 0 / 0 / 0 everywhere | Reported, no direction: selection decides retention (the live survey read 3.8%); T11.F10 and T22 own the discovery question |
| Goal persistence (final, minimum, plateau) | final 4,423 / 7,582 / 6,048; minimum 39 / 865 / 170; plateau 5,923.74 / 6,795.29 / 6,159.02 | **Floor:** no world extinct, final and plateau each not under half the reference on every world; a world under the floor is a blocker reported to the user, neither waived nor rebased |
| `config_digest` (gate and goal), founder digest, `FOUNDER_GENOME_SIZE_UNITS`, recipe-digest pins | current pins | Unchanged (`inputs_changed` false) |
| Observation budgets | workflow caps | Founder neighborhood below 10 s; summed evolved below 180 s; drift walk below 30 s per world; goal run below 15 minutes |
| Wall time, both profiles | T20.F16 | Flat within the flag threshold when the host matches; a flag is tolerated, a severe is investigated against the per-hop cost before it is presented |

Run sequentially after benchmark-affecting work is final, without competing
builds, tests, servers or measurements:

```sh
make bench PROFILE=gate FEATURE=t11-f29-feedback-route-formation
make bench PROFILE=goal FEATURE=t11-f29-feedback-route-formation
```

**Measured verdict.** Pending.

- Summaries: [gate](../../progress/features/t11-f29-feedback-route-formation.json),
  [goal](../../progress/features/t11-f29-feedback-route-formation-goal.json).
- Full readings: [`docs/progress/readings/t11-f29.md`](../../progress/readings/t11-f29.md).

## Success Criteria

- [ ] `AddFeedbackRoute` is in the live catalog at weight 10 with its key in
      every operator-keyed row, and one applied event closes a loop onto the
      source or an executed upstream node while the static incumbent keeps
      the route (tests in Verification).
- [ ] The drift walk's cycle-carrying share rises at depths 250, 1,000 and
      2,000 on both walks; every other predeclared bound holds or its excess
      carries a recorded user decision.
- [ ] `make check` exits 0, the mutation gate has no unresolved survivor, both
      summaries are stored, and `v3-mutation-spec.md` describes the operator.

## Notes for AI Agents

- Decision: (spec owner, 2026-10-09) the feedback branch is born one unit
  below an unwritten incumbent's bias rather than tied as at T11.F15, because
  a loop has no forwarding detour to make a winning bid silent; the margin is
  a code constant, not a config field, and proves loss only for gate values
  up to 1.0 (runtime loss beyond that is measured, as T11.F15 states).
- Decision: (spec owner, 2026-10-09) "the creature's own executed path" is
  T11.F17's executed set, read from the `TargetSelector`, not structural
  reachability; a parent without a dispatch record can only self-loop.
