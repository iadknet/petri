# T30.F12 — The Sink Catalog Is Whole

**Status**: Complete
**Last updated**: 2026-10-29
**Feature**: T30.F12
**Track**: [T30 — Core Boundaries](../../roadmaps/t30-core-boundaries.md)

## Goal

Every Graph node holds the fixed 108-sink catalog in catalog order, and
nothing else can be built or loaded: the genome builds the catalog from one
kind-to-index map, rejects any other sink list when a genome is
deserialized, and finds every sink by its index. The layer that handled a
partial catalog (`hosts`, `hosts_kind`, the present-sink filter in `plans`,
`Option`-returning sink lookups, `insert_member`'s `expect`, `can_add_edge`,
`RetargetBundle`'s two skip orders and the other skips listed below) is
deleted. Simulation behavior, draw streams and serialized genomes are
unchanged.

## Non-Goals

- The genome's JSON shape: each sink keeps its `kind`, now checked against
  its index; genome files and canonical hashes stay byte-identical.
- Typed slot indices for `CustomOutput`, `RouterGate`, `WriteSlot` and
  `ClearSlot`: a kind outside the catalog stays representable and is a
  caller bug at lookup.
- T30.F11's work on `insert_member` (its quadratic position scan) and
  `BundleCreations`; T30.F05's move of the routing edits in
  `mutation/topology/routing.rs` (G2, G23).
- Readers that iterate the catalog and filter by kind (`neighborhood/drives.rs`
  `node_fanouts`, `ring_rules.rs`, the server's brain view): they read the
  whole catalog already and keep doing so.

## Inputs and Invariants

Inputs:

- The row and its Notes entry: `with_output_sinks` accepts any list and
  neither `validate_graph` nor parseability checks it; "Validation requires
  the full catalog, sinks are found by index, and that hosting layer is
  deleted"; F12 follows F09, which left no non-test builder of a partial
  catalog. Both are behavior-preserving in the T25.F01 shape: a skip path
  that fires on a production genome is a blocker for the user.
- T30.F09's Goal and invariant 2 (the lab comparators' canonical-JSON
  hashes); its Non-Goals name this row as the payer of partial-catalog
  tolerance.
- The code on `768dde1fc`, the options and the preparatory audit in the
  [readings](../../progress/readings/t30-f12.md).

The catalog is enforced where foreign data enters. After `with_output_sinks`
is deleted no primitive changes a sink's kind or the list's length, so the
only way to hold another list is a deserialized genome; validating there
(the `GraphEdge` precedent, `#[serde(try_from)]` at `genome/cgp/edge.rs:46`)
makes every in-memory Graph whole, and `validate_graph`, which the lab runs
after deserializing (`v3-lab/src/lib.rs:61-90`) and the mutation gate runs
per event, never sees a partial catalog and gains no per-event check.

Invariants:

1. No behavior change: the gate report's `deterministic` block equals the
   epoch's byte for byte; the reproducibility and draw-stream pins in
   `make check` are unchanged. A difference is a defect, never a re-pin.
2. Whole by construction: every `CgpGraphBackendDef` holds exactly
   `FIXED_SINK_COUNT` sinks, sink `i` of the `i`-th catalog kind.
   `new_with_fixed_outputs` is the only constructor of the sink list;
   deserializing a list that is shorter, longer, reordered, duplicated or
   of a kind outside the catalog fails with an error naming the catalog.
   No `pub` or `pub(crate)` item replaces, reorders, adds or removes a sink.
3. One catalog order: one `OutputSinkKind` ↔ catalog-index map in
   `genome/cgp/mod.rs` is the only encoding of the order; the constructor
   builds from it, and `FIRST_ACTION_VOTE_SINK` and `FIXED_SINK_COUNT`
   derive from or are compile-time checked against it. The index of a kind
   outside the catalog panics and never aliases another sink.
4. Total lookups: the sink of a kind and its surface are returned without
   `Option`, by index; `site_sink` is absent only where `Site::sink` is (a
   site the wiring has no sink for), and `insert_member`'s one `expect`
   states that caller precondition, never a missing catalog sink.
5. `hosts` and `hosts_kind` are gone; `plans` takes only the field set and
   returns, in the same order, what `plans` returns today on a full
   catalog (Plain: one `Plain` per vote sink in catalog order; Ring and
   Pair: every kind of `DirectedVoteKind::ALL`, offsets `0..8`).
6. Skips that only a partial catalog took are deleted and every draw that
   runs keeps its range and order: `draw_births`' empty-plans error,
   `bundle_sources`' filter and `add_bundle_by_law`'s empty-sources skip
   (`mutation/graph/bundle.rs`), `RetargetBundle`'s
   Plain no-other-sink skip and post-draw `hosts` skip, `pick_random_surface`'s
   `None` and `can_add_edge` (`AddGraphEdge` applies to every Graph node), and
   `gate_sink`'s `Option` with its `ok_or` and `feedback_slot`'s Graph test
   (`mutation/topology/routing.rs`). The Graph effect pass applies every
   wired sink with no slot-range or vote-index guard
   (`runtime/cgp/effects.rs`, the `CustomOutput`, `RouterGate`,
   `WriteSlot`, `ClearSlot` and `ActionVote` arms), since an out-of-catalog
   kind can no longer exist. The sink trace's `applied` flag, which only
   such a guard set apart from `wired`, is deleted from
   `GraphOutputSinkTrace` (`runtime/trace/domain.rs`), the tracer's sink
   callback, and the server's `GraphOutputSinkTracePayload` with its
   generated frontend type; `applied_value` stays.
7. After the change `grep -rnE "with_output_sinks|hosts_kind|cgp::hosts|fn hosts|can_add_edge" crates`
   matches nothing.
8. No test is deleted other than tests of deleted behavior (a catalog
   missing or adding a sink); a test whose subject survives is kept and
   retargeted onto the full catalog.

## Architecture

The feature touches the genome (`genome/cgp`), `mutation` (`mutation/graph`,
`mutation/topology/routing.rs`), `runtime` (`runtime/cgp/effects.rs`), readers (`neighborhood/drives.rs`,
`census.rs`), `creature/cgp_founder.rs`, two outer callers
(`v3-cli/src/bench/census_pins.rs`, `v3-lab/src/controllers.rs`) and the
server's sample payload, a wire type it owns
(`v3-server/src/transport/sample_protocol.rs`). The shared
rule it changes is which sinks a Graph carries and where each sits. Section
3 makes the genome its owner ("genome structure") and rule 1 gives the owner
building and validating; `contracts` keeps the vote-sink and parameter
vocabulary the catalog is made of. Today the rule is applied by
`new_with_fixed_outputs` and the offset constants (`genome/cgp/container.rs:425-452`,
`mod.rs:132-148`) and interpreted by scans and presence tests in
`bundle.rs:448-492` (`hosts`, `plans`), `container.rs:488-500,1111,1404`,
`mutation/graph/bundle.rs:103,194,376-395`, `mutation/graph/operators.rs:310-320,941`,
`routing.rs:46-52,83,250`, `drives.rs:109` and the slot guards in
`runtime/cgp/effects.rs:129-172`. After the change every path
reuses the owner's index map and total lookups; no separate policy remains.
No section 5 debt names this deficiency, so none is deleted. `routing.rs`
carries G2 and G23 (T30.F05); this feature changes only its sink lookup and
neither moves nor extends that debt. No boundary changes;
`docs/architecture.md` is not edited.

## Telemetry

Not applicable: no signal, attribute or configuration is added or changed,
and every deleted branch is one no production genome takes, so every
counter, skip tally and span a run emits is unchanged. The server's sample
payload, transport rather than telemetry, loses its redundant `applied`
flag (invariant 6).

## Verification Plan

**Question.** F12 claims the partial-catalog layer can be deleted and every
sink found by index without moving a draw, a trajectory or a serialized
genome; a differing gate `deterministic` block, a changed draw-stream or
reproducibility pin, a changed lab comparator hash, or a gate compute
regression past its threshold would show it wrong.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate` | The track's verification note declares each row gate with pins unchanged; the edits sit on the birth path's mutation draws, so the gate's deterministic blocks are the byte-identity reading and its compute threshold the cost reading, and no population reading is in question. |
| Slow tests (`make rust-test-slow`) | `run` | The workflow's trigger names a change to `crates/v3-lab`: the lab's comparator wiring changes its lookup and its genome-file load now rejects a partial catalog at deserialization, and the tier holds the lab's end-to-end runs that build comparators and reload genome files. |

## Implementation Tasks

- [x] Before any edit: record the comparator digests on `768dde1fc`
      (Verification) in the readings.
- [x] Catalog map (audit rows 4, 8): one `OutputSinkKind` ↔ index map in
      `genome/cgp/mod.rs`; `new_with_fixed_outputs` builds from it; the
      offset constants derive from or are checked against it; a property
      test that the map round-trips over every catalog index and that an
      out-of-catalog kind panics.
- [x] Validation (rows 1, 2): delete `with_output_sinks`; the sink list
      deserializes through a validating conversion (invariant 2); tests
      that a missing, extra, reordered, duplicated and out-of-catalog list
      is rejected and that a founder genome round-trips byte-identically.
- [x] Total lookups (row 4): `sink`, `sink_surface`, `site_sink` and
      `insert_member` by index; `relocate_bundle`'s panic clause for a
      missing sink deleted; callers in `creature/cgp_founder.rs`,
      `neighborhood/census.rs`, `neighborhood/drives.rs`, `genome/mod.rs`,
      `v3-lab/src/controllers.rs` and fixtures in every crate drop their
      `expect`, `unwrap` and `if let`.
- [x] Hosting layer (row 3): delete `hosts` and `hosts_kind`; `plans(set)`;
      callers in `genome/node_edit.rs`, `genome/cgp/container.rs`,
      `mutation/graph/bundle.rs`, `v3-cli/src/bench/census_pins.rs` and
      `genome/cgp/test_support.rs`.
- [x] Skip paths (rows 5, 6): invariant 6's deletions; `draw_births` and
      `pick_random_surface` become total; `MutationSkipReason` keeps its
      variants that other paths use.
- [x] Fixtures (row 7): `test_support::edit_sinks` edits inputs only, kinds
      fixed; every `with_output_sinks` fixture in `v3-core`, `v3-cli` and
      `v3-lab` starts from the full catalog, addressing a sink by its
      catalog index; tests of partial catalogs (`container_tests.rs:944-964`,
      `genome/tests/node_edit_tests.rs:75-94`, the skip half of
      `retarget_skips_after_its_draws_when_the_kind_lacks_a_site`, the
      empty-def and generated-sink-list cases of
      `pick_random_surface_matches_the_reference`) are deleted or reduced
      to the full catalog.
- [x] Runtime guards and trace flag (rows 9, 10): invariant 6's last two
      sentences; the trace and sample-assembler tests read `wired` where
      they read `applied`; the generated frontend type is regenerated.

## Verification

- [x] Focused tests: `cargo test -p v3-core genome::cgp` (103 passed) and
      `cargo test -p v3-core mutation::graph` (119 passed) -> pass; the
      catalog-map tests (`catalog_map_round_trips`,
      `a_kind_outside_the_catalog_has_no_index`) and deserialization tests
      (`any_other_sink_list_is_rejected_at_deserialization`,
      `a_founder_def_round_trips_byte_identically`) exist.
- [x] On `4591d0776` (the simplify pass, v3-core only):
      `cargo test -p v3-core --test viability` (30 passed) and
      `cargo test -p v3-core` (2,346 passed) -> pass;
      `cargo check --workspace --all-targets`, `cargo fmt --all --check`
      and `make rust-clippy` -> clean. `cargo test -p v3-server` and, in
      `frontend/`, `npx tsc -b`, `npx biome check src/` and
      `npx vitest run` (680 passed) -> pass on `b9e4071c6`; the pass after
      it touches neither.
- [x] `make check` -> exit 0 on tested commit `19029eeb7`, reproducibility and draw-stream pins unchanged.
- [x] Invariant 7's grep -> no match; output in readings.
- [x] Lab comparator hashes, T30.F09 invariant 2's method: the SHA-256 of
      each comparator genome's canonical JSON on `768dde1fc`, taken before
      any edit, equals the one on the final code; both in readings.
- [x] `make rust-test-slow` -> exit 0 on `19029eeb7` (6 passed).
- [x] Gate profile: summary stored at `docs/progress/features/t30-f12.json`,
      deterministic block identical to the epoch, raw hash and byte count
      checked, series entry points to the summary, no full report staged.
      Command `make bench PROFILE=gate FEATURE=t30-f12`, exit 0, severe=false,
      verdict ok; readings in `docs/progress/readings/t30-f12.md`.

## Performance and Goal Impact

**Predeclaration — written before the run.** No cost-bearing change: sink
lookups on mutation paths become index reads instead of scans, deleted
branches never ran, and the tick loop's execution iterates the same
catalog. Expected compute cost: none measurable. The gate is compared
against the current gate epoch (`docs/progress/features/t25-f12.json`,
`benchmark-series.json`) with the standard thresholds and no re-pin; every
deterministic block is expected identical, a differing block is a defect,
and a compute regression past the threshold goes to the spec owner, not to
acceptance. No indicator can move; the goal profile is not run
(Verification Plan).

**Measured verdict.** `make bench PROFILE=gate FEATURE=t30-f12` -> make exit 0,
CLI exit 0; `severe=false`; threshold verdict ok on every counter (deltas
0.000000) against `t25-f12.json`; `deterministic` block identical to the
epoch (SHA-256 `448cbef2...d21d7` both); wall clock +6.73% ms/creature-tick,
level ok. Raw 159,914 bytes, SHA-256 `ade33cf956a8864943f9c6edf8f5da48027506702d06f1d67d221f2d4b45525f`,
at `.bench-artifacts/t30-f12/gate.json` (ignored, unstaged); summary 36,763 bytes.

- Summary: [gate](../../progress/features/t30-f12.json).
- Full readings: [`docs/progress/readings/t30-f12.md`](../../progress/readings/t30-f12.md).

## Success Criteria

- [x] No Graph node can be built or loaded without the full catalog, and no
      edit or reader handles a partial one (invariants 2–7).
- [x] Draws, trajectories and serialized genomes are unchanged (invariant 1,
      the comparator hashes).

## Notes for AI Agents

- Decision: the catalog is enforced at deserialization, the one entry for a
  foreign sink list, not re-checked in `validate_graph` per mutation event
  (readings, options table).
- Decision: in-feature refactors from the preparatory audit, rows 1–8:
  `with_output_sinks` deleted, validating deserialization, `plans(set)`
  without `hosts`, total index lookups, the partial-catalog skips deleted,
  `gate_sink` by index, `edit_sinks` inputs only, one catalog order; rows
  9–10, found during the build: the effect pass's slot guards and the sink
  trace's `applied` flag deleted.
- Decision: fixtures set a catalog sink's inputs through the public
  `with_sink_inputs(kind, inputs)` builder (audit row 1's one fixture
  builder), unchecked like `with_compute_nodes`; it never adds, removes or
  reorders a sink.
- Deferred: final review lens 3 P2 — the widest-queue fixture at
  `genome/cgp/container_tests.rs:2740` still duplicates `add_bundle`'s
  private writes to bypass insertion cost; pre-existing, already marked
  `WORKAROUND(T30.F11)`, and paid by T30.F11's ordered bulk construction.
- Cost: `/usage` not recorded (user command); 3
  implementer passes with 2, 2 and 2 advisor consults; 1 spec-owner resume
  after Plan; 2 Codex rounds, final `ready`; final review
  lens 1 none, lens 2 none, lens 3 one P2; Opus 5.5 high in every Fable
  role (user override 2026-10-29).
