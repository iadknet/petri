# T30.F14 — The Inspector Reads Resolved Views

**Status**: In Progress
**Last updated**: 2026-10-28
**Feature**: T30.F14
**Track**: [T30 — Core Boundaries](../../roadmaps/t30-core-boundaries.md)

## Goal

The creature inspector shows what the genome resolves. The live-circuit
slot diagnostics come from one genome read API under the mesh annotations'
liveness rule, a bundle's birth coordinates each carry the location of the
member that sits there, and the brain context is one identity the client
compares whole. `v3-server` forwards these facts and the frontend matches
nothing the core already matched. No simulation behavior changes.

## Non-Goals

- Any change to a tick, birth, mutation, sensor, energy or learning rule,
  to `derive_mesh_annotations` or its callers on the birth path
  (`creature::state::GenomeDerived::of`, `sensor_analysis::has_barrier_reader`),
  or to `MeshNodeAnnotation`'s fields.
- The other live-circuit diagnostics (node, stateful, barrier and class
  counts over the annotations), the current-input and recent-action
  diagnostics, and the brain view's `NodeAnnotation`.
- Members outside their bundle's birth coordinates, which the brain view
  already lists from the edge list (T30.F01 invariant 3).
- Validating the sink catalog (T30.F12) and any census or population
  statistic (T30.F13, T30.F15).

## Inputs and Invariants

Inputs: the T30 row and its Notes entry for F14 (user direction,
2026-10-27; the [F13–F14 design review](../../progress/readings/t30-f13-f14-design-review.md),
whose finding 6 chose the annotations' liveness rule and recorded the
change as a diagnostic correction), `docs/architecture.md` sections 1–3,
and the code on `2ce6e4632`. The preparatory audit (R1–R5) is in the
[readings](../../progress/readings/t30-f14.md).

| # | Invariant |
| --- | --- |
| 1 | **No simulation behavior change.** `v3-core` gains one read API (invariant 2) and `BirthCoordinate` gains a member location (invariant 4); nothing on the tick, birth or mutation path calls either, and `derive_node_annotation`, `derive_cgp_annotations` and `GenomeDerived::of` are unchanged, so no trajectory, draw or birth cost moves. `make check`'s viability and baseline-world digests are unchanged. |
| 2 | **Bus slots are a genome fact.** `genome::mesh_annotations` exposes a read API that takes a node and its `MeshNodeAnnotation` and returns the node's bus slots: the upstream bus slots its live reads reach and the payload (output bus) slots its live writes reach. A live read is a `ReadInput` at one of the annotation's `live_instruction_indices` (VM) or an `InputLeaf` edge on the wired surface of its `live_internal_node_indices` (`cgp_analysis::wired_surface_edges`, Graph); its slot is `InputReference::bus_slot(sub_idx)` of the reference it names, for `UpstreamSlot` and `UpstreamBus` alike. A live write is a `WriteInternalPayload` at a live instruction index (VM) or a wired `CustomOutput` sink (Graph), counted when its slot is below `OUTPUT_SLOT_COUNT`, the bound `runtime/vm.rs` and `runtime/cgp/effects.rs` apply. The API reuses the annotation's liveness and never re-derives it. |
| 3 | **Slot diagnostics forward the genome fact.** `live_circuit` carries `distinct_upstream_slots_read` and `distinct_payload_slots_written`, each the sorted union of invariant 2's sets over reachable nodes; `distinct_custom_output_slots_written` and the server's slot walks (`upstream_slots_read` and the payload and custom-output loops in `http/creature.rs`) are deleted. These are the inspector-diagnostic corrections this feature makes, against today's output: a declared `UpstreamSlot` no live read reaches, or a read off every live slice or wired edge, no longer adds an upstream slot; Graph custom-output slots join the payload list, since both sinks write the node's `output_slots` (`MeshWriteClass::Payload` already classes them together); the frontend's "Custom Outputs" row is removed. Every other diagnostic value is unchanged. |
| 4 | **Coordinates carry their member.** `CgpGraphBackendDef::birth_coordinates` returns, for each birth coordinate, the location of the first member of the bundle reading its field on its site, in `members` order (sink order, then edge order), and none for a hole, in place of `present`. The brain view's bundle coordinate carries that location as `(surface, container, edge)`, the key the `learning` payload uses, omitted for a hole, and drops `present`: a coordinate is present exactly when it carries a location. The frontend reads the member's edge at that location; `coordinateKey` and `indexMembers` in `EdgeBundleList.tsx` are deleted. Presence and every displayed member are the same as today for every valid genome. |
| 5 | **The brain context is an identity.** `brain_context` is a string the server builds from the run's `SensorContext` by exhaustive destructuring, so a field added to `SensorContext` fails to compile until the identity carries it; equal contexts give equal strings and different contexts different ones. The client compares it with `===` and holds no field of it. On any committed config change the client polls the selected creature, and the identity comparison decides whether the brain is refetched; the subscription to `runtime.max_actions_per_turn` alone is deleted. |
| 6 | **Wire.** `PROTOCOL_VERSION` becomes `v3alpha17` in the server and the frontend; the generated types under `frontend/src/types/generated/` are regenerated through the wire-type test (`PETRI_WRITE_TS_TYPES=1`, then `npm run lint:fix`), and the test passes without the flag. |

## Architecture

Owners touched: the genome's analysis (`genome::mesh_annotations`,
`genome::cgp_mesh_annotations`, `genome::cgp_analysis`) and its Graph
container (`genome::cgp::container`), the server's inspector projection
(`v3-server/src/http/creature.rs`, `transport/brain.rs`), and the frontend
inspector. No debt in section 5 names this work and no boundary changes:
section 2 already holds the desired state ("the genome and runtime resolve
what the inspector shows through their own read APIs; the server only
selects, structures and serializes it"), and this feature removes three
places the code departed from it, so `docs/architecture.md` is not edited.
Shared rules: the annotations' liveness rule is owned by
`genome::mesh_annotations` (`collect_live_vm_instruction_indices`) and
`genome::cgp_analysis` (`cgp_live_compute_indices`, `wired_surface_edges`),
applied today by `derive_node_annotation`, `derive_cgp_annotations` and
`collect_consumed_input_refs`; invariant 2's API reads the annotation's live
indices and the same `wired_surface_edges`. The bus-slot rule is owned by
`contracts` (`InputReference::bus_slot`), applied by `runtime/inputs.rs`
and `neighborhood/ring_rules.rs`; the API calls it. The coordinate-member
rule is owned by `birth_coordinates` (`reads_source`, `member_field`,
`site_of`); the frontend's copy is deleted. The sensor context is owned by
`SimulationConfig::sensor_context`; the frontend's copy of its inputs is
deleted.

## Telemetry

Not applicable: the feature changes the inspector's HTTP payload only; no
run, counter, span or record changes.

## Verification Plan

**Question.** F14 claims the inspector's slot sets, coordinate members and
context check come from core facts with no simulation change; a moved
viability or baseline-world digest, a slot set that disagrees with the
annotations' live reads and writes, or a displayed member that differs from
today's for a valid genome would show it wrong.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `none` | Server and frontend transport plus two core read APIs that nothing on the tick, birth or mutation path calls (invariant 1), so no trajectory or simulation cost can move; `make check`'s digests answer the no-change claim. |
| Slow tests (`make rust-test-slow`) | `not run` | No `crates/v3-lab` code changes. |

## Implementation Tasks

- [x] Core: add invariant 2's bus-slot read API in `genome::mesh_annotations` with unit tests for each liveness case (a declared but unread `UpstreamSlot`, a dead VM `ReadInput`, an `InputLeaf` edge into a non-live compute node, an `UpstreamBus` field read, an `UpstreamSlot` read through any field, a VM payload write, a wired and an unwired `CustomOutput`), moving the server test `a_bus_field_read_is_the_slot_its_upstream_slot_twin_reads` here.
- [x] Core: `BirthCoordinate` carries the member location (invariant 4); update the container tests and extend the bundle proptest (`mutation/graph/tests/bundle.rs`) so every located member is a member of its bundle reading the coordinate's field on its site.
- [x] Server: diagnostics forward the API (invariant 3); bundle coordinates forward the location; `BrainContext` becomes the identity (invariant 5); `PROTOCOL_VERSION` `v3alpha17`; regenerate the wire types; update `tests/server.rs`' brain-context test to compare identities, and add a server test pinning `live_circuit`'s two slot lists for a genome with an unreachable node and a Graph `CustomOutput` write.
- [x] Frontend: `EdgeBundleList.tsx` reads members by location and presence from the location; `useCreatureDetailResource.ts` compares the identity whole and polls on any committed config change, with hook tests for a commit that changes the identity (brain refetched) and one that does not (brain not refetched); drop the "Custom Outputs" row; update the protocol pin and the affected tests and fixtures.

## Verification

- [x] Core slot and coordinate tests: `cargo test -p v3-core mesh_annotations` and `cargo test -p v3-core bundle` -> result recorded in the readings.
- [x] Core bus-slot properties (final review lens1 P2): the `bus_slot_tests` proptests `a_live_upstream_slot_read_and_its_bus_field_twin_read_one_slot`, `a_live_bus_field_read_past_the_bus_reads_no_slot` and `a_live_payload_write_reaches_only_on_bus_slots` under `cargo test -p v3-core mesh_annotations` -> pass (25 tests).
- [x] Server: `cargo test -p v3-server` (wire-type test without `PETRI_WRITE_TS_TYPES`) -> pass.
- [x] Frontend: `npm test`, `npm run lint` and `npx tsc -b` in `frontend/` -> pass (lint warnings only in files F14 does not touch).
- [x] `cargo fmt --all -- --check` and `make rust-clippy` -> pass.
- [ ] `make check` -> exit 0, viability and baseline-world digests unchanged.
- [ ] Benchmark profiles: `Not run: Verification Plan`.

## Performance and Goal Impact

**Predeclaration.** No simulation or birth-path code changes, so no
compute cost, indicator or threshold applies; no profile runs.

**Measured verdict.** Not run: Verification Plan.

## Success Criteria

- [x] `http/creature.rs` contains no walk over a node's program or edges; its slot lists come from the genome API.
- [x] `EdgeBundleList.tsx` contains no coordinate join and `useCreatureDetailResource.ts` no field of the brain context.
- [ ] Invariants 1–6 hold and the Verification items pass.

## Notes for AI Agents

- Decision: the slot sets are a read API over a node and its annotation, not fields of `MeshNodeAnnotation`, because the annotations are derived at every birth (`GenomeDerived::of` through `has_barrier_reader`) and only the inspector reads the slot sets (row: "the slot sets become genome annotations under the annotations' liveness rule").
- Decision: the slot-set changes in invariant 3 are inspector-diagnostic corrections (T30 Notes, F14; design review finding 6), not behavior changes.
- Deferred: the hand-built bundle arrangement of `the_widest_queue_bundle_lists_every_coordinate_present` (`container_tests.rs`, marked `WORKAROUND(T30.F11)`) retires when T30.F11 gives the owner ordered bulk construction; T30.F11 pays it (final review lens3 P2).
