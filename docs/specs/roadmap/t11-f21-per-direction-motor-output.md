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
A `steering-v1` battery reports exact-hit, within-45-degree, avoidance, and
bank-written fractions beside the T11.F14 block.

## Non-Goals

- No sensor, operator family, config value, or edit to `neighborhood-v1`.
- No environmental pressure: the change is the body's output decode, not a
  world, so the standard-baseline pressure rule does not apply.
- No comparison-chain keys, floors, or progress-page rendering for the
  steering readings.
- Follow-ons recorded, not built: topographic operator bias (ring slot d
  preferring bank slot d); a bank for the `Eat` food-type parameter; inspector
  rendering of the bids and the server trace payload; any sensor change.

## Inputs and Invariants

- Sources: the track's T11.F21 row and note; the [motor output encoding
  note](../../strategy/motor-output-encoding-research-2026-09-16.md), "Contract
  for T11.F21" (form, decode rule, backend shapes, readings, predeclared
  directions); the [live survey](../../strategy/live-survey-2026-09-16.md) it
  repairs (Section 3.2; Appendix B is the reference implementation of readings
  (a) and (b)); [T11.F14](t11-f14-mesh-execution-observability.md) for the
  battery seam and executed set; [T11.F15](t11-f15-mesh-routing-connection-semantics.md)
  and [T11.F18](t11-f18-backend-neutral-mesh-node-growth.md) for the
  neutrality and observation conventions.
- Research: the note weighs per-direction winner-take-all (TPG/SBB bids, NEAT
  one-output-per-action) against heading vectors, egocentric turns, and a
  discretized scalar and adopts the bank. Local seams suffice:
  `runtime/action_decode.rs` decodes for both backends,
  `mutation/graph/operators.rs::EdgeSurface` enumerates the five edge
  surfaces, `neighborhood::battery::draw_scenarios` builds fixed scenarios,
  `Battery::mesh_execution_sets` returns the executed set. No package.

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
`param_inputs`, so `pick_random_surface` gains one surface per slot. Blank and
founder graphs carry empty banks. `GraphActionSlotTrace` gains
`direction_bids: Option<[f32; 8]>` and `chosen_direction: Option<u8>`, `Some`
only when the bank was written; server payload and frontend are unchanged.

**VM form.** One opcode, `WriteDirectionBid { direction: u8, src }`, writes
`bids[direction] = regs[src]` into a per-dispatch eight-slot buffer beside
`meta` and marks the bank written; `direction >= 8` is ignored and marks
nothing. `PushAction` for `action_type` 2, 3, or 4 applies the decode rule;
buffer and flag persist between pushes within a dispatch and reset with `meta`
at node end. Cost 0.14 beside `WriteWorldActionMeta`. The opcode joins the
fresh-instruction draw, the operand nudge (`direction` or `src`), the proptest
strategy, Display, register-use analysis, and the existing `action` write
class: no new `MeshWriteClass`, so `creature-detail.ts` is unchanged and the
inspector renders the opcode through its lowercase fallback.

**Neutral at birth.** Founder programs and graphs write no bank and every
stored genome deserializes with empty banks; for them the decode is
bit-identical, so the unmutated founder's battery signature, `mesh_execution`,
and `steering` readings are unchanged. Because the fresh-instruction draw is
`gen_range(0..N)` over the opcode catalog and `pick_random_surface` is uniform
over surfaces, one more opcode and surface change the mapping of every such
draw: founder rows for VM insertion-class operators and `AddGraphEdge`, the
gate and goal trajectories, and the drift walk diverge from the previous
closure, as at T13.F05. The contract's "founder rows and the gate trajectory
unchanged" therefore holds for the unmutated founder's readings; Performance
predeclares the rest.

**Readings, `steering-v1`.** Separate from `neighborhood-v1`. Bases:
`STEERING_BASE_COUNT = 6` scenarios from `draw_scenarios` with
`STEERING_SEED = 9`. (a) For each base and each `d` in `0..8`: every food
type's `food_here` 0, the primary ring one-hot at d, other rings zero, barrier
and occupancy rings zero, the rest from the base; one tick from zeroed shared
memory and fresh graph state through the production executor, as the
`neighborhood-v1` snapshots run; a move is a queue leading with `Move(c)`, an
exact hit `c == d`, within-45 `c` in `{d-1, d, d+1}` mod 8. (b) For each (a)
scenario leading with `Move(c)`: the same scenario with `barrier[c] = 1`;
avoided when the lead is no longer `Move(c)`. (c) `bank_written`: a node in
the T11.F14 executed set structurally writes a bank (a VM program containing
`WriteDirectionBid`, or a graph slot with `Emit(Move | Reproduce |
StealEnergy)` behavior and a non-empty bank). Per genome: `scenarios` (48),
`moves`, `exact_hits`, `within_45`, `avoidance_trials`, `avoided`,
`bank_written`. Per seed, pooled over the sampled genomes: the sums and
`exact_hit_fraction`, `within_45_fraction`, `avoidance_fraction`,
`bank_written_fraction`, with `chance: {exact: 0.125, within_45: 0.375}`
beside them; a zero denominator gives `Undefined`, never 0. Stored in
`deterministic` as `steering` beside `mesh_execution` (founder half in the
gate report, each sampled genome in the goal report) and `steering_pooled` on
each evolved seed, serde-defaulted to `Undefined` on old reports, timed within
the existing observation caps; other profiles acquire nothing.

## Implementation Tasks

- [x] Decode rule: shared direction selection in `runtime/action_decode.rs`
      with property tests; `cgp/effects.rs` calls it.
- [x] Graph form: `DirectionBidEdge`, `ActionSlot::direction_bids`, effects
      pass, trace fields, blank/founder construction, serde tests.
- [x] VM form: opcode, executor buffer and flag, cost, analysis, annotation,
      Display, proptest strategy, mutation draw and nudge.
- [x] Graph surface: `EdgeSurface::ActionBid`, `edge_sites`,
      `pick_random_surface`, edge accessors, raw-field `direction`.
- [x] `steering-v1`: `neighborhood/steering.rs`, bench schema, indicators,
      run assembly, JSON and thread-count tests, gate and goal wiring.
- [x] Reference documents: `v3-vm-isa-spec.md` (opcode row, cost table,
      action encoding), `v3-graph-backend-spec.md` (`ActionSlot`, Section 6
      decode, trace), `v3-mutation-spec.md` (six surfaces, `direction` raw
      field, the opcode in the insertion pool).

## Verification

- [x] Decode property tests (`runtime::action_decode`): no bank equals the
      scalar decode; all-tie yields the scalar direction; a unique maximum its
      index; a tie containing the scalar the scalar, else the lowest tied
      index; non-finite bids sanitized. Test names in the readings file.
- [x] One-edge fixtures on both backends from the founder: one positive
      `neighbor_food[d] -> bid d` edge gives an exact hit on food-at-d for every
      d and changes no other battery action; one negative
      `neighbor_barrier[d] -> bid d` edge avoids on every base where the
      founder leads with `Move(d)` and changes nothing else.
- [x] Neutrality: the gate summary's founder `mesh_execution` block is
      byte-identical to the previous closure's (jq transcript in the readings
      file), and the founder's `steering` reading is deterministic with
      `bank_written` false; its values are the recorded reference.
- [x] Mutation coverage with controlled RNG: `AddGraphEdge` lands on a bank
      surface and the four edge operators and the raw-field operator act on
      it; the VM fresh draw can produce `WriteDirectionBid`; the nudge covers
      both operands; genomes without `direction_bids` deserialize.
- [x] `steering-v1` unit tests: the one-edge fixtures read above chance; old
      reports read `Undefined`; byte-identical across thread counts.
- [ ] `cargo test -p v3-core --test viability` first, then `make check` ->
      exit 0 on the final feature commit (hash in the readings file).
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      and every survivor resolved as killed, equivalent, or deferred. The full
      survivor list stays here; `docs/workflow.md` requires it in the spec.
- [x] `make bench PROFILE=gate FEATURE=t11-f21-per-direction-motor-output`
      and one `PROFILE=goal` run: summaries at
      `docs/progress/features/t11-f21-per-direction-motor-output.json` and
      `...-goal.json`, local raw hash/byte count and verification time checked,
      series entry points to the summary, no new full report staged; one goal
      run by the 2026-09-05 decision.
- [ ] `make roadmap-check` and `make check-docs` on the document edits.

Test names per item, the self-review command table, jq checks, and
per-world tables live in `docs/progress/readings/t11-f21.md`. Working-tree
results on 2026-09-16: viability 26 passed; `cargo test --workspace
--no-fail-fast` 1950 passed, 0 failed, 4 ignored; clippy `-D warnings`,
`cargo fmt --check`, `make roadmap-check`, and `make check-docs` clean; no
`proptest-regressions/` file. The neutrality jq transcript ran clean against
the gate summary (readings file); the founder half (deterministic `steering`,
`bank_written` false) matches its unit-tested reference.

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

**Measured verdict.** Gate/goal `severe=false`; founder neutral; no extinction;
bank not yet written (predeclared); Canyon/Confluence `changed`/`dead` moved
against the previous closure — flagged in the readings, not resolved here.

| World | exact_hit_fraction | avoidance_fraction | bank_written_fraction | final_population | changed (cur/prev) | dead (cur/prev) |
| --- | --- | --- | --- | --- | --- | --- |
| Orchards in grassland | 0.484375 | 0.000000 | 0.000000 | 8 | 0.386332 / 0.354354 | 0.000000 / 0.004505 |
| Canyon country | 0.285012 | 0.000000 | 0.000000 | 4121 | 0.283382 / 0.328422 | 0.005831 / 0.001842 |
| Confluence | 0.238208 | 0.000000 | 0.000000 | 8105 | 0.360515 / 0.389446 | 0.016452 / 0.002672 |

- Summaries: `docs/progress/features/t11-f21-per-direction-motor-output.json`
  and `...-goal.json`; full readings `docs/progress/readings/t11-f21.md`.

## Success Criteria

- [ ] Both backends decode a written bank by the shared rule and an unwritten
      bank by the unchanged scalar decode; the unmutated founder's readings
      are bit-identical to the previous closure.
- [ ] One bank edge from a neighbor-food slot seeks and one negative edge from
      a neighbor-barrier slot avoids, on each backend, proven by fixtures.
- [ ] Every gate and goal report carries `steering-v1` beside the T11.F14
      block with chance levels printed, and the predeclared directions are
      read and recorded, including the finding-7 or not-exposed case.
- [x] Reference documents describe the bank, the opcode, and the sixth surface.

## Notes for AI Agents

- Decision: added at the user's direction on 2026-09-16 from the live survey;
  the form, decode rule, readings, and predeclared directions are fixed in the
  motor output encoding note's "Contract for T11.F21", and placement directly
  after T11.F20 in the order of new starts is recorded on the master roadmap.
