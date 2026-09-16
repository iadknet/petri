# T11.F21 — Per-Direction Motor Output

**Status**: In Progress
**Last updated**: 2026-09-16
**Feature**: T11.F21
**Track**: [T11 — Brain Genotype-Phenotype Map](../../roadmaps/t11-brain-genotype-phenotype-map.md)

## Goal

A motor map with winner-take-all selection: a movement action (`Move`,
`Reproduce`, `StealEnergy`) may carry an eight-slot direction bank, one bid per
neighbor cell, and the body commits the direction with the highest bid, ties to
the scalar-decoded direction. One positive edge from a neighbor-food slot, or
one negative edge from a neighbor-barrier slot, is a complete selectable
steering step on either backend. A genome that writes no bank decodes its
direction from the scalar parameter exactly as today; the founder writes none.
A `steering-v1` battery reads seeking and avoidance beside the T11.F14 block.

## Non-Goals

- No sensor, operator family, config value, or edit to `neighborhood-v1`.
- No environmental pressure: the change is the body's output decode, not a
  world.
- No comparison-chain keys, floors, or progress-page rendering for the
  steering readings.
- Follow-ons recorded, not built: topographic operator bias; a bank for the
  `Eat` food-type parameter; inspector rendering of the bids and the server
  trace payload; any sensor change.

## Inputs and Invariants

- Sources: the track's T11.F21 row and note; the [motor output encoding
  note](../../strategy/motor-output-encoding-research-2026-09-16.md), "Contract
  for T11.F21" (form, decode rule, backend shapes, readings, predeclared
  directions); the [live survey](../../strategy/live-survey-2026-09-16.md) it
  repairs (Appendix B is the reference probe for readings (a) and (b));
  [T11.F14](t11-f14-mesh-execution-observability.md) for the battery seam and
  executed set; [T11.F15](t11-f15-mesh-routing-connection-semantics.md) and
  [T11.F18](t11-f18-backend-neutral-mesh-node-growth.md) for conventions.
- Research: the note weighs per-direction winner-take-all (TPG/SBB bids, NEAT
  one-output-per-action) against heading vectors, egocentric turns, and a
  discretized scalar and adopts the bank; existing seams carry it, no package.

**Decode rule** (`runtime/action_decode.rs`, one function both backends call).
If no bank was written for the action, the scalar decode applies unchanged.
Otherwise bids are sanitized to finite values; the direction is the index of
the maximum bid; when several tie at the maximum, the scalar-decoded direction
wins if it is among them, else the lowest tied index. Consequences the readings
rely on: one positive edge `food[d] -> bid d` changes nothing while `food[d]`
is 0 and selects d when it is positive; one negative edge `barrier[d] -> bid d`
alone never selects d and otherwise leaves the scalar decode in force. `Eat`,
`Pop`, and `NoOp` never consult a bank.

**Graph form.** `ActionSlot` gains `#[serde(default)] direction_bids:
Vec<DirectionBidEdge>` (a `GraphEdge` plus `direction: u8`) on every slot,
evaluated in the effects pass beside `param_inputs`: bid d is the weighted sum
of the edges with `direction == d`, unwired 0, and the bank is written when at
least one edge has `direction < 8`. It is the sixth edge surface,
`EdgeSurface::ActionBid(slot)`, for `AddGraphEdge` (`direction` uniform in
`0..8`, source through `random_graph_source`), `RemoveGraphEdge`,
`RetargetGraphEdge`, `AlterGraphEdgeWeight`, and `GraphRawFieldMutation`
(`direction` is one more one-unit field); applicability follows
`param_inputs`, so `pick_random_surface` gains one surface per slot.
`GraphActionSlotTrace` gains
`direction_bids: Option<[f32; 8]>` and `chosen_direction: Option<u8>`, `Some`
only when the bank was written; the server payload is unchanged.

**VM form.** One opcode, `WriteDirectionBid { direction: u8, src }`, writes
`bids[direction] = regs[src]` into a per-dispatch eight-slot buffer beside
`meta` and marks the bank written; `direction >= 8` is ignored and marks
nothing. `PushAction` for `action_type` 2, 3, or 4 applies the decode rule;
buffer and flag persist between pushes within a dispatch and reset with `meta`
at node end. Cost 0.14 beside `WriteWorldActionMeta`. The opcode joins the
fresh-instruction draw, the operand nudge (`direction` or `src`), the proptest
strategy, Display, register-use analysis, and the existing `action` write
class (no new `MeshWriteClass`, so the frontend is unchanged).

**Neutral at birth.** Founder programs and graphs write no bank and every
stored genome deserializes with empty banks; for them the decode is
bit-identical, so the unmutated founder's battery signature, `mesh_execution`,
and `steering` readings are unchanged. Because the fresh-instruction draw is
`gen_range(0..N)` over the opcode catalog and `pick_random_surface` is uniform
over surfaces, one more opcode and surface change the mapping of every such
draw: founder rows for VM insertion-class operators and `AddGraphEdge`, the
gate and goal trajectories, and the drift walk diverge from the previous
closure, as at T13.F05; the contract's "founder rows and the gate trajectory
unchanged" holds for the unmutated founder's readings.

**Readings, `steering-v1`.** Bases:
`STEERING_BASE_COUNT = 6` scenarios from `draw_scenarios` with
`STEERING_SEED = 9`. (a) For each base and each `d` in `0..8`: every food
type's `food_here` 0, the primary ring one-hot at d, other rings zero, barrier
and occupancy rings zero, the rest from the base; one tick from fresh state
as the `neighborhood-v1` snapshots run; a move is a queue leading with `Move(c)`, an
exact hit `c == d`, within-45 `c` in `{d-1, d, d+1}` mod 8. (b) For each (a)
scenario leading with `Move(c)`: the same scenario with `barrier[c] = 1`;
avoided when the lead is no longer `Move(c)`. (c) `bank_written`: a node in
the T11.F14 executed set structurally writes a bank (a VM program containing
`WriteDirectionBid`, or a graph slot with `Emit(Move | Reproduce |
StealEnergy)` behavior and a non-empty bank). Per genome the counts and
`bank_written`; per seed, pooled over the sampled genomes, the sums and
`exact_hit_fraction`, `within_45_fraction`, `avoidance_fraction`,
`bank_written_fraction`, with `chance: {exact: 0.125, within_45: 0.375}`
beside them; a zero denominator gives `Undefined`, never 0. Stored in
`deterministic` as `steering` beside `mesh_execution` (founder half in the
gate report, each sampled genome in the goal report) and `steering_pooled` on
each evolved seed, serde-defaulted to `Undefined` on old reports.

## Implementation Tasks

- [x] Decode rule: shared direction selection in `runtime/action_decode.rs`
      with property tests; `cgp/effects.rs` calls it.
- [x] Graph form: `DirectionBidEdge`, `ActionSlot::direction_bids`, effects
      pass, trace fields, blank/founder construction, serde tests.
- [x] VM form: opcode, executor, cost, analysis, mutation draw and nudge.
- [x] Graph surface: `EdgeSurface::ActionBid` through every edge accessor
      and the raw field.
- [x] `steering-v1`: `neighborhood/steering.rs`, bench schema and
      indicators, gate and goal wiring.
- [x] Reference documents: `v3-vm-isa-spec.md`, `v3-graph-backend-spec.md`,
      `v3-mutation-spec.md`.

## Verification

- [x] Decode property tests (`runtime::action_decode`): no bank equals the
      scalar decode; all-tie yields the scalar direction; a unique maximum its
      index; a tie containing the scalar the scalar, else the lowest tied
      index; non-finite bids sanitized.
- [x] One-edge fixtures on both backends from the founder: one positive
      `neighbor_food[d] -> bid d` edge gives an exact hit on food-at-d for every
      d and changes no other battery action; one negative
      `neighbor_barrier[d] -> bid d` edge avoids on every base where the
      founder leads with `Move(d)` and changes nothing else.
- [x] Neutrality: the gate summary's founder `mesh_execution` block is
      byte-identical to the previous closure's (jq transcript in the readings
      file), and the founder's `steering` reading is deterministic with
      `bank_written` false.
- [x] Mutation coverage with controlled RNG: every edge operator and the
      raw-field operator act on a bank surface; the VM fresh draw yields
      `WriteDirectionBid` and the nudge covers both operands; genomes without
      `direction_bids` deserialize.
- [x] `steering-v1` unit tests: the one-edge fixtures read above chance; old
      reports read `Undefined`; byte-identical across thread counts.
- [ ] `cargo test -p v3-core --test viability` first, then `make check` ->
      exit 0 on the final feature commit (hash in the readings file).
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      and every survivor listed here as killed, equivalent, or deferred.
- [x] `make bench PROFILE=gate FEATURE=t11-f21-per-direction-motor-output`
      and one `PROFILE=goal` run (the 2026-09-05 decision): summaries at
      `docs/progress/features/t11-f21-per-direction-motor-output.json` and
      `...-goal.json`, raw hash/byte count checked, series entries added, no
      full report staged.
- [ ] `make roadmap-check` and `make check-docs` on the document edits.

Test names, command tables, jq checks, and per-world tables live in
`docs/progress/readings/t11-f21.md`; no `proptest-regressions/` file.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: a motor map with
winner-take-all selection, direction-tuned outputs competing the way a superior
colliculus map or basal ganglia selection commits one movement; the eight
neighbor cells are the map. It reaches creatures through the body's output
decode, not through a sensor or the world.

Expected compute cost: none measurable (a flag test per movement push, an
eight-float reset per VM dispatch, an empty-vector pass per fired graph slot;
evolved banks add one weighted sum per bid edge). Gate and goal are compared
against the epoch baselines the series index names under the existing
thresholds; no cap is added, and the steering readings run inside the existing
observation caps.

Trajectories. Gate and goal diverge from the previous closure from the first
VM insertion or `AddGraphEdge` draw (Inputs, "Neutral at birth"), before any
bank exists. A goal-epoch re-pin at closure is the predeclared outcome, not a
regression; the user's acceptance of any severe flag is recorded here verbatim.
No gate-epoch re-pin is expected. Extinction in any goal world is a blocker.

Directions for every indicator this feature can move: founder
`mesh_execution` and `steering` identical to the previous closure; founder
neighborhood rows identical except VM insertion-class operators and
`AddGraphEdge`, which move by the draw change under the standing T11.F01
floors; evolved per-birth `changed` not down and `dead` not up against the
previous goal summary; `neighborhood_read.changed_per_all_births` not below
the T14.F12 floors 0.146400 / 0.130000 / 0.143400; population persistence,
lineage diversity, sensor census, recruitment paths, drift depth, evolved
`mesh_execution`, and the plasticity counters: none. Steering, evolved half
pooled over the three worlds' sampled genomes with per-world values stored:
`exact_hit_fraction` above 0.125, `avoidance_fraction` above 0,
`bank_written_fraction` above 0. The goal sample is 22 to 50 generations deep:
a `bank_written_fraction` of 0 reads as not yet exposed at this depth, with (a)
and (b) at chance by construction, and the feature closes on the founder,
neutrality, and one-edge readings (the deep reading is the survey probe on the
user's running world). If (a) and (b) stay at chance while (c) is above 0, the
note's finding 7 (the ecology keeps steering rare) is the recorded explanation
and the feature still closes.

**Measured verdict.** Gate and goal exit 0, `severe=false`, all six counters
ok against the epochs, so no re-pin; founder `mesh_execution` byte-identical
to T14.F07, founder `steering` deterministic with `bank_written` false.
Spec-owner rulings, 2026-09-16: (1) no sampled genome writes a bank, so the
exact-hit values are scalar-decode seeking, not attributable to the feature;
the predeclaration's "at chance by construction" was wrong, the founder reads
0.5 with no bank (ring-derived direction), and the founder's 0.5 and the
table, not 0.125, are the next closure's reference.
(2) Canyon and Confluence read `changed` down and `dead` up against T14.F07:
a predeclaration miss recorded, not remediated; the mechanism cannot produce
a dead birth, the samples are post-bottleneck lineages, and the T14.F12
floors are cleared. (3) Minimum populations 8 / 171 / 8 against 880–4,275 in
every stored goal summary; Orchards never recovered and its sample is eight
generation-1–2 creatures. A user-authorized control at 72845446 with the two
draws restored reproduced the T02.F04 goal closure integer-for-integer
(readings file), so the collapse is the diverged trajectory of the draw
re-mapping, not the bank; closure proceeds and the epoch is not re-pinned.

| World | exact_hit | avoidance | bank_written | pop min / tick-200 / final (prev min / final) | changed cur / prev | dead cur / prev | neighborhood_read cur / prev / floor |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Orchards in grassland | 0.484375 | 0.000000 | 0.000000 | 8 / 1,474 / 8 (1,194 / 9,772) | 0.386332 / 0.354354 | 0.000000 / 0.004505 | 0.183750 / 0.207200 / 0.146400 |
| Canyon country | 0.285012 | 0.000000 | 0.000000 | 171 / 2,211 / 4,121 (4,152 / 11,534) | 0.283382 / 0.328422 | 0.005831 / 0.001842 | 0.180400 / 0.196400 / 0.130000 |
| Confluence | 0.238208 | 0.000000 | 0.000000 | 8 / 3,012 / 8,105 (1,010 / 14,110) | 0.360515 / 0.389446 | 0.016452 / 0.002672 | 0.190000 / 0.201400 / 0.143400 |

- Summaries: `docs/progress/features/t11-f21-per-direction-motor-output.json`
  and `...-goal.json`; full readings `docs/progress/readings/t11-f21.md`.

## Success Criteria

- [ ] Both backends decode a written bank by the shared rule and an unwritten
      bank by the unchanged scalar decode; the unmutated founder's readings
      are bit-identical to the previous closure.
- [ ] One bank edge from a neighbor-food slot seeks and one negative edge from
      a neighbor-barrier slot avoids, on each backend, proven by fixtures.
- [ ] Every gate and goal report carries `steering-v1` beside the T11.F14
      block with chance levels, and the predeclared directions are read and
      recorded, including the finding-7 or not-exposed case.
- [x] Reference documents describe the bank, the opcode, and the sixth surface.

## Notes for AI Agents

- Decision: added at the user's direction on 2026-09-16 from the live survey;
  the contract in the motor output encoding note fixes the form and readings,
  and the master roadmap records its placement after T11.F20.
- Decision: the user authorized the 2026-09-16 control (readings file), which
  attributes this closure's Orchards collapse to the draw re-mapping. The
  `goal_worlds` epoch stays T11.F19; this goal summary stays in the closed
  list as the true record, but the next closure predeclares population,
  evolved per-birth, and `neighborhood_read` directions against T02.F04's goal
  summary, explains previous-closure flags against this summary by the
  collapse, and compares steering against this spec's table and the
  founder's 0.5. Orchards' fragility is a T12/T02 finding.
