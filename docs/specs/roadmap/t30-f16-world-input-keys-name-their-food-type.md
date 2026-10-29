# T30.F16 — World-Input Keys Name Their Food Type

**Status**: In Progress
**Last updated**: 2026-10-28
**Feature**: T30.F16
**Track**: [T30 — Core Boundaries](../../roadmaps/t30-core-boundaries.md)

## Goal

A world input key has one string key, defined by its owner in `contracts`,
that names its food type as well as its family, and every world-input count
that leaves core (server wire, telemetry, benchmark, lab) is keyed by it. Two
food types' counts never share a key, overwrite or sum into each other.
Trajectories, draw streams and every count's value are unchanged; only the
keys listed under Inputs and Invariants change.

## Non-Goals

- No change to simulation behavior, to what is counted, or to the
  inspector's wire shape (`genome_wire.rs` sends the family and food type
  as separate fields and keeps doing so).
- No compatibility alias for the old family-only keys (repository rule:
  no backward compatibility).
- No change to the frontend, its generated types (the maps stay
  string-keyed) or the Grafana dashboards.
- T30.F11's change to how the engine collects the added node's world inputs
  is not part of this feature; this feature does not touch the engine or
  `simulation/stats.rs`.

## Inputs and Invariants

Inputs: the T30 row and its Notes entry for F16 (user direction,
2026-10-28: "Plan the two payer rows (paired definitions R4–R6, and the
food-typed world-input wire keys R11 with its sibling)"; the
[F15–F16 design review](../../progress/readings/t30-f15-f16-design-review.md)),
T30.F13's audit row R11 and its invariant 1 (reader outputs are numbers and
core keys; outer crates apply the labels), and the code on `4a5684cd7`. The
inventory of every site that turns a world input key into a string, the
pin table and the preparatory audit (A1–A6) are in the
[readings](../../progress/readings/t30-f16.md).

Invariants:

1. **One key, owned by contracts.** `WorldInputKey::as_key` returns the
   key's transport key: the family name, followed by `:<food type index>`
   for `FoodHere`, `NeighborFoodRing` and `AreaFoodSummary` (`FoodHere:1`,
   `AreaBarrierSummary`), the format readers' `world_input_key_label`
   produces today. It is injective over `WorldInputKey::universe` for every
   food-type set (the existing property test moves with it). The family
   name without the food type is `WorldInputKey::family_name`, used only
   where a family-level key is meant: the inspector's `directionPairs.json`
   fixture test and the sampler's bucket tests.
2. **No second definition.** `world_input_key_label` is deleted;
   `ring_rules`, the benchmark sensor census and telemetry's sensor gauges
   call `as_key`. `input_use::Family::label` gives a world family its
   `WorldInputKey`'s `as_key` (through the inverse of `Family::of` for
   world families) and keeps the variant name for every other family; no
   family or food-typed key string is spelled out anywhere else in
   production code.
3. **Every count that leaves core keys by it.** The server's
   `vm_live_read_world_inputs_current` and
   `mutation_added_node_world_inputs_total_by_operator` (`v3-server/src/state.rs`)
   and telemetry's `petri.world_input` label on
   `petri.run.mutation_added_node_world_inputs_by_operator`
   (`v3-telemetry/src/metrics.rs`) key by `as_key`. With two food types
   present each type's count appears under its own key with its own value;
   the `WORKAROUND(T30.F13)` marker at `state.rs` is deleted.
4. **Only the named outputs move.** The keys of those three outputs change
   for the three food families; every value, every other key, the benchmark
   report (whose sensor-census, ring-row and input-use keys already carry
   the food type), the NDJSON stream and the lab outputs are unchanged. The
   one pin that moves is listed in the readings' pin table; any other moved
   pin is a defect.

## Architecture

The feature touches `contracts`, which `docs/architecture.md` section 3
makes the owner of shared vocabulary "and pure functions over them", readers
(`neighborhood/`), and the outer crates section 2 limits to transport and
export. The shared rule it changes is the string key of a world input key.
Its owner becomes `WorldInputKey::as_key` (`contracts/inputs.rs`); today the
rule is applied in readers (`world_input_key_label`,
`neighborhood/sensor_census.rs`; `Family::label`,
`neighborhood/input_use/catalog.rs`) and interpreted by `ring_rules.rs`,
`v3-cli/src/bench/tracking.rs`, `v3-telemetry/src/metrics.rs` (two
families) and `v3-server/src/state.rs` (two maps). Every path reuses the
owner's operation after the change; no separate policy remains. The
collisions were T30.F13's unpaid accept R11, not a section 5 debt, so no
debt line is deleted, no boundary changes and `docs/architecture.md` is not
edited.

## Telemetry

The one changed signal; no signal, attribute or configuration is added.

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `petri.run.mutation_added_node_world_inputs_by_operator` | counter (monotonic sum) | `SimStats::mutation_added_node_world_inputs_total_by_operator`, unchanged | `petri.operator`; `petri.world_input` becomes `WorldInputKey::as_key` (`FoodHere:0` where it was `FoodHere`), one series per food type in place of their sum; the series stay bounded by the run's key universe (5 + 3 × food types) |

The sensor gauges' `petri.world_input` label already carries the food type
and does not change.

## Verification Plan

**Question.** Is every world-input count that leaves core keyed by its food
type while every value, trajectory and benchmark block stays as it was? Two
food types' counts sharing a key in either server map or the telemetry
family, a gate deterministic block differing from the epoch's, a work
counter off 0%, or a pin moving outside the readings' pin table would show
it wrong.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate` | The gate's persistence samples carry the sensor census keyed by the moved key function (`FoodHere:0` in `t25-f12.json`), so the gate seeds confirm on real worlds that those keys, the deterministic blocks and the nine work counters are unchanged; ring-row and input-use labels are observed only in goal runs and are covered by their focused tests; no population reading is in question. |
| Slow tests (`make rust-test-slow`) | `not run` | No `crates/v3-lab` change; the lab's input-use labels are covered by its fast tests. |

## Implementation Tasks

- [x] Write the failing tests first: with two food types, both server maps
      and the telemetry mutation family carry `FoodHere:0` and `FoodHere:1`
      with their own counts (invariant 3).
- [x] A1: rename `WorldInputKey::as_key` to `family_name` and fix every
      caller the compiler lists; then add `as_key` returning the transport
      key (invariant 1), move `world_input_key_label`'s tests and property
      test to contracts, and switch each lossy caller to `as_key`.
- [x] A2: delete `world_input_key_label`; `ring_rules.rs`,
      `bench/tracking.rs` and `metrics.rs`'s sensor gauges call `as_key`.
- [x] A6: `Family::label` takes a world family's key from `as_key`.
- [x] A3–A5: key both server maps and the telemetry mutation family by
      `as_key`; delete the `WORKAROUND(T30.F13)` marker.
- [x] Move the pin in the readings' pin table and update doc comments that
      describe the old key (`as_key`, `WorldInputCensusRow::key`).

## Verification

- [x] Focused tests: the new invariant-3 tests, the moved contracts tests
      and property test, and the existing ring-row and input-use label
      tests (`bench/ring_rules.rs`, `bench/input_use/tests.rs`,
      `v3-lab/src/readings/tests.rs`) unchanged, via
      `cargo test -p v3-core -p v3-server -p v3-telemetry -p v3-cli -p v3-lab` -> pass;
      recorded in the readings.
- [x] Pins: only the readings' pin-table entry moves.
- [x] `grep` finds no `world_input_key_label`, no `WORKAROUND(T30.F13)` on
      a world-input key, and no hand-written food-typed key format in
      production code.
- [ ] Gate: summary stored at `docs/progress/features/t30-f16.json`, local
      raw hash and byte count checked, series entry points to the summary,
      no new full report staged.

## Performance and Goal Impact

**Predeclaration — written before the run.** Not a mechanism feature; no
simulation code changes, so no indicator can move and the epoch is not
re-pinned. Reference: the series' epoch baseline
(`docs/progress/features/t25-f12.json`, named by `docs/progress/benchmark-series.json`), at the standard thresholds (flag
10%, severe 50%). Expected: the gate's deterministic blocks identical to the
epoch's and all nine work counters at 0.000000%, since key formatting runs
outside simulation work and produces the strings it does today for every
benchmark block. Expected cost: none; each key is still formatted once per
emitted row. Wall-clock time per creature tick is compared on the same host
and labelled (flag 25%, severe 100%) but cannot mark the comparison severe;
expected inside the flag threshold. A counter off 0%, or a deterministic
block that differs, is a defect, not a cost.

**Measured verdict.** Pending.

- Summary of the declared profile: [gate](../../progress/features/t30-f16.json).
- Full readings: [`docs/progress/readings/t30-f16.md`](../../progress/readings/t30-f16.md).

## Success Criteria

- [ ] With two food types, the server's two world-input maps and the
      telemetry mutation family hold one entry per food type with its own
      count.
- [ ] `WorldInputKey::as_key` is the only definition of a world input
      key's string key, and every count leaving core uses it.
- [ ] Trajectories, draw streams, benchmark blocks and every pin outside
      the readings' pin table are unchanged.

## Notes for AI Agents

- Decision: `WorldInputKey::as_key` is the food-typed transport key, as
  every other key type's `as_key` is its transport key, and `family_name`
  is the family-only key; three outputs collided by calling `as_key` under
  that convention (readings A1).
