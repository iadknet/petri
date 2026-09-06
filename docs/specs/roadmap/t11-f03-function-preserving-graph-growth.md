# T11.F03 — Function-Preserving Graph Growth

**Status**: In Progress
**Last updated**: 2026-09-05
**Feature**: T11.F03
**Track**: [T11 — Brain Genotype-Phenotype Map](../../roadmaps/t11-brain-genotype-phenotype-map.md)

## Goal

Every graph growth operator (add compute node, copy compute node, copy
subgraph, add input reference) leaves the brain acting exactly as its parent
did at the moment it fires: a new node joins disconnected, with one bootstrap
input, or by splitting an existing edge with function preserved; a copy never
rewires a live node; a new input reference is not wired into a live surface.
New graph edges can reach every sub-value of every input reference, and the
graph raw-field operator changes one field by one step. The mutation reference
records the growth-versus-connection operator taxonomy the node-type contract
relies on.

## Non-Goals

- No change to `MutationConfig` probabilities, operator weights, or
  reachability bias (T11.F04); no founder graph change; no graph clock or
  relaxation-semantics change (T11.F06); no learned-weight correspondence
  (T11.F09); no neutral duplication qualification beyond fire-time
  neutrality (T11.F08); no NEAT speciation or fitness sharing.
- No claim that connection or parameter operators (add, retarget, remove
  edge, copy edge bundle, swap kind, mutate parameter, input-reference swap,
  remove, raw field) are neutral; they may change behavior in one step.
- Topology-family operators (`AddNode`, `CopyNode`, mesh slices, `SpliceNode`)
  already read 1.00/0.94 silent on the founder and are not modified.
- No benchmark threshold change, baseline re-pin, or second goal run.

## Inputs and Invariants

- Sources: the track's T11.F03 note and node-type contract; floors (c) and
  the no-regression rule in [T11.F01](t11-f01-mutational-neighborhood-indicator.md);
  the T11.F02 precedent for neutrality wording and closure evidence; the
  [graph backend](../../reference/v3-graph-backend-spec.md) and
  [mutation](../../reference/v3-mutation-spec.md) references.
- Seams: `mutation/graph/operators.rs` (`add_compute_node`,
  `copy_compute_node`, `copy_cgp_subgraph`, `random_graph_source`,
  `raw_field_mutation`, the private `input_ref_width` duplicate),
  `mutation/graph/mod.rs::GraphMutator::apply`, `mutation/input_ref/mod.rs::apply_add`,
  `mutation/compound.rs::sub_value_count`, `CgpGraphBackendDef::remove_compute_node_at`
  and its private `remap_compute_sources` (the inverse insert-with-remap
  belongs beside it), `runtime/cgp/execute.rs` and `eval.rs` for the neutrality
  proof, and the `neighborhood` battery. Extend these; add no alternate path.
- Research (reviewed 2026-09-04 in the track's basis, rechecked 2026-09-05
  against local code): NEAT's add-node splits an edge with incoming weight 1
  and the old outgoing weight ([Stanley and Miikkulainen 2002](https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf));
  CGP evolvability rests on inactive genes and neutral drift
  ([Miller and Smith 2006](https://pure.york.ac.uk/portal/en/publications/redundancy-and-computational-efficiency-in-cartesian-genetic-prog),
  [Atkinson, Plump, and Stepney 2021](https://eprints.whiterose.ac.uk/id/eprint/154735/1/Atkinson2019_Article_EvolvingGraphsWithSemanticNeut.pdf)).
  Options: (1) repair the existing operators in place (selected by the track's
  option table; smallest change, keeps the genome format, frontend, and
  founder); (2) replace the graph backend with a NEAT-style network (rejected:
  discards the CGP backend, plasticity, and tracing work for no measured gap);
  (3) leave growth as is and rely on T11.F04 supply changes (rejected: the
  audit's 1.0% silent add-node is a semantics defect, not a rate defect). No
  external dependency is needed; `std` and existing crates suffice.
- Neutrality definition (T11.F02 wording): identical action, output-slot, and
  shared-memory behavior when both executions have enough energy and
  relaxation passes. A grown node still costs `graph_node_base_cost` per pass;
  do not hide that cost or claim neutrality at energy exhaustion.
- Add-node policy: draw one of three forms with equal probability. (a)
  Disconnected: a random kind with no inputs, appended. (b) Bootstrap: a random
  kind with exactly one input edge from `random_graph_source`, appended; no
  surface reads it. (c) Split: pick one existing edge on any of the five
  surfaces whose source is any `GraphSource`; insert a new `Add` node (the
  identity kind: `evaluate_compute_kind` returns the weighted sum, so one
  input of weight 1.0 reproduces the source exactly in f32) whose single input
  is the old source with weight 1.0, and retarget the old edge to the new node
  keeping its old weight. When the consumer is a compute node at index `c`,
  insert the split node at index `c` and remap every `ComputeNode(i >= c)`
  reference across all five surfaces to `i + 1`, so Gauss-Seidel pass order is
  preserved: the new node reads its source in the same pass phase the consumer
  used to, and the consumer reads the new node in the current pass. When the
  consumer is a sink, action slot, or execute gate, append. Final outputs are
  identical either way; a split of a backward or self edge may extend
  convergence by at most one pass because the new node's delta lags its
  source's by one pass. A split never targets an edge whose source is
  `ComputeNode(u16::MAX)` or otherwise out of range; skip with
  `NoApplicableTarget` when the graph has no edge. Remove the per-sub-value
  spraying loop and the duplicate `input_ref_width`. Insert-with-remap shifts
  compute-node indices exactly as `remove_compute_node_at` already does;
  learned-weight correspondence across that shift is T11.F09's, and this
  feature must not change it for graphs the operator did not touch.
- Copy-node policy: push a faithful copy (kind, inputs, plasticity) of one
  random compute node and nothing else; drop the backlink and the coin-flip
  input clearing (a cleared copy is the disconnected add-node form).
  `copy_cgp_subgraph` is already disconnected; keep it and prove it.
- Input-reference `Add`: push the reference and wire nothing on either
  backend. The VM `ReadInput` auto-insertion is removed; the reference becomes
  addressable by later connection operators. Its previously repaired jump
  handling becomes moot. Record the spare-register `ReadInput` alternative as
  deferred to T11.F08 if the indicator later shows references never go live.
- Sub-value reach: `random_graph_source` takes the node's `input_refs` and
  `MutationConfig` and draws `sub_idx` uniformly in
  `0..sub_value_count(reference, config)`; every caller (bootstrap edge,
  `add_edge`, `retarget_edge`) gets it. `GraphMutator::apply` gains a
  `&MutationConfig` parameter; the engine and `neighborhood/operators.rs`
  pass the config they already hold.
- Raw-field step: `GraphRawFieldMutation` selects one parameterized node or one
  edge uniformly as today, then changes exactly one field by one unit: a
  parameter by the existing `MutateGraphOperatorParam` step; `ComputeNode(i)`
  by ±1 inward at the bounds `0..compute_count`; `InputLeaf.ref_idx` by ±1
  inward within `0..input_refs.len()`, a direction being offered only when
  the current `sub_idx` is within the new reference's width;
  `InputLeaf.sub_idx` by ±1 inward within the reference's width;
  `SharedMemory.slot` by ±1 modulo 16; `SharedMemory.previous` flipped. The
  variant is never replaced. When the chosen target has no valid unit move the
  event skips with `NoApplicableTarget`.
- Taxonomy for the reference: growth operators (`AddInternalGraphNode`,
  `CopyInternalNode`, `CopySubgraph`, input-reference `Add`, and the topology
  `AddNode`, `CopyNode`, slices, `SpliceNode`) are neutral at the moment they
  fire; connection and parameter operators change behavior in one step
  (`CopyEdgeBundle` remains an explicit multi-edge macro, as the T11.F02
  paired-slot macro is). Write this into `v3-mutation-spec.md` under the
  node-type contract and the graph and InputRef domains, and update the graph
  backend spec's T11.F03 pointer and the mutation spec's `Add` description.
- Determinism: seeded runs stay byte-identical across processes; the
  `reproducibility` test in `make check` remains the check.

## Implementation Tasks

- [x] Write failing tests first: property tests over arbitrary graph defs and
      battery inputs that each growth operator (three add-node forms, copy
      node, copy subgraph, input-reference add on graph and VM nodes) yields
      identical execute outputs, actions, and shared-memory writes with ample
      energy and passes, and that a compute-consumer split preserves every
      node's per-pass value; a property that over seeds every `sub_idx` of a
      compound reference is drawn by `random_graph_source`; a property that
      raw-field mutation changes exactly one field by one unit and never the
      variant; example tests for the skip cases and index remapping.
- [x] Implement the add-node forms with insert-with-remap, the faithful copy,
      unwired input-reference add on both backends, config-driven sub-index
      sampling, and the one-field raw step; delete the obsolete tests that
      pinned spraying, backlinks, and auto-wiring.
- [x] Update `v3-mutation-spec.md` (taxonomy, graph and InputRef domain
      entries, node-type contract owner text) and `v3-graph-backend-spec.md`.
- [x] Self-review the diff (`simplify`), then fresh `make rust-mutants`; record
      evidence below.
- [ ] Store gate and single goal reports at
      `docs/progress/features/t11-f03-function-preserving-graph-growth.json`
      and `...-goal.json`, append both to `docs/progress/benchmark-series.json`,
      and add the T11.F03 row to `docs/progress.md`.

## Verification

- [x] TDD evidence: wrote `crates/v3-core/src/mutation/graph/tests/operators.rs`
      (10 property/example tests: neutrality of the three add-node forms,
      `CopyComputeNode`, `CopySubgraph`, unwired `InputRef.Add` on graph and
      VM nodes, split per-pass preservation, `random_graph_source` sub_idx
      reach, raw-field one-step) against the old operator signatures first;
      `cargo check` failed until the operators were rewritten. During
      implementation two real defects surfaced only once the property tests
      ran against production code paths (not caught by the unit-level
      example tests alone): `insert_compute_node_at`'s remap overflowed on
      `u16::MAX` dangling sentinels (`mutation::engine::tests::stress_parseability_10000_chained_mutations`),
      and `valid_edge_field_moves`'s `InputLeaf.ref_idx` bounds check
      indexed `input_refs` without checking the candidate index was in
      range. Both fixed; no `proptest-regressions/` files were produced (no
      regression failures survived the fixes). Deleted the five auto-wiring
      pins in `mutation/input_ref/tests.rs` and the two spraying pins in
      `mutation/graph/operators.rs`'s inline tests, replacing each with its
      inverse (zero edges wired, VM program byte-identical).
- [x] `cargo test -p v3-core --test viability` ran before the first
      production edit (25 passed) and again at closure (25 passed);
      `cargo check --workspace --all-targets` after every coherent Rust edit
      (enforced by the implementer-compile-check hook); focused
      `mutation::graph` (48 passed), `mutation::input_ref` (23 passed),
      `mutation::engine` (all passed, including the two stress tests fixed
      above), `mutational_neighborhood` (4 passed), and `reproducibility` (1
      passed) tests pass; full `cargo test -p v3-core --lib` (1119 passed)
      and `cargo test -p v3-core` (all integration suites) pass.
- [x] `make rust-mutants` (fresh, `MUTANTS_ITERATE=0`): output path
      `/Users/istefanek/.local/share/petri-tools/mutants/t11-f03/mutants.out`.
      First fresh run: `99 mutants tested in 10m: 27 missed, 60 caught, 12
      unviable`. Added 6 tests (`add_graph_edge_wrapper_increases_edge_count`,
      `random_graph_source_input_leaf_is_the_majority_when_compute_is_empty`,
      `add_compute_node_reaches_all_three_forms_with_distinct_signatures`,
      `valid_edge_field_moves_compute_node_bounds`,
      `valid_edge_field_moves_input_leaf_bounds`,
      `apply_edge_field_move_moves_by_the_exact_signed_delta`) closing 21 of
      27 survivors; a second fresh run: `99 mutants tested in 8m: 6 missed, 81
      caught, 12 unviable`. Extended two of those tests with two more exact
      boundary cases (an `InputLeaf` sub_idx-equals-neighbor-width case
      killing both remaining `valid_edge_field_moves` `<=` survivors on the
      `RefIdx` width checks, and a new
      `valid_edge_field_moves_shared_memory_always_offers_three_moves` test
      killing the `SharedSlot(-1)` sign-deletion survivor); simplify pass
      (see below) deduped one test's helper. Third fresh run, the closure
      run: `99 mutants tested in 8m: 3 missed, 84 caught, 12 unviable`.
      Remaining 3 survivors, all in `crates/v3-core/src/mutation/graph/operators.rs`:
      - `191:20: replace < with <=` in `random_graph_source` — **deferred**.
        `rand` 0.8.6's `Standard` impl for `f32` (`distributions/float.rs`,
        `float_impls!` macro, precision 24) draws `scale * (rng.gen::<u32>()
        >> 8) as f32` where `scale = 2^-24`, i.e. every multiple of `2^-24`
        in `[0, 1)` is reachable with probability `2^-24`. `0.8f32`'s bit
        pattern (`0.800000011920929`) lies in the `[0.5, 1)` binade where the
        f32 ULP is exactly `2^-24`, so it is one of those reachable grid
        points (not proven impossible, contradicting an earlier equivalence
        hypothesis checked against `rand`'s own source before this
        disposition was written) — but hitting that exact draw needs a
        targeted search over up to 2^24 seeds, impractical within the test
        budget. Mirrors the T11.F02 precedent's "~2.1 billion elements"
        deferral for a similar unreachable-in-practice boundary.
      - `812:28: replace < with <=` and `812:24: replace + with *`, both on
        the `if ref_idx + 1 < ref_count` guard in `valid_edge_field_moves` —
        **equivalent**. `ref_count == input_refs.len()` and the guarded block
        immediately re-derives the same candidate index (`ref_idx + 1`,
        computed on its own unmutated line) and looks it up with
        `input_refs.get(candidate)`; that `Option` check alone already
        excludes every `candidate >= ref_count`, so the outer guard cannot
        change which branch executes for any input — the mutated and
        unmutated code produce byte-identical `Vec<EdgeFieldMove>` output on
        every call.
- [ ] `make bench PROFILE=gate FEATURE=t11-f03-function-preserving-graph-growth`
      and one `PROFILE=goal` run stored; second goal run: Not applicable per
      the 2026-09-05 workflow decision.
- [ ] `make roadmap-check` on document edits; final `make check` exits 0 at
      the commit that lands on `main`.

## Performance and Goal Impact

Natural analog: new neural connections. A neuron that grows in disconnected,
or splices into an existing pathway passing its signal through unchanged,
reaches the creature through its inherited brain, not through a sensor.

Predeclared cost: no work-counter definition changes. Neutral growth lets
disconnected nodes accumulate, so per-pass graph energy and wall time per
creature-tick may rise, and populations differ, so every counter may move; a
forward-edge split adds no relaxation pass because pass order is preserved,
and a backward-edge split adds at most one. No severe regression is budgeted
and no epoch re-pin is authorized.
Record all six gate counter deltas against T10.F10 and T11.F02 and the goal
deltas against T01.F12 and T11.F02.

Neighborhood expectations before implementation, founder rows against the
T11.F02 gate report: `AddInternalGraphNode` silent 0.02 → about 1.00,
`CopyInternalNode` 0.58 → about 1.00, input-reference `Add` 0.66 → about
1.00 (floor (c) is 0.95 and is due by T11.F10, not asserted here);
`CopySubgraph` stays 1.00; `GraphRawFieldMutation` silent 0.10 should rise
toward the single-field rows (0.74). `AddGraphEdge` (0.90) and
`RetargetGraphEdge` (0.02) may move either way because sub-index sampling
changes which sources are drawn; report them without adjustment. Mutated-birth
silence should rise and dead fall, within the coarseness of 44 mutated and 4
single-event births. Evolved-sample, lineage, persistence, and memory readings
may shift either way because the inherited map changes the population; record
them against T11.F02 without a cognition claim. No operator family is
disabled or down-weighted.

## Success Criteria

- [ ] Every growth operator is neutral at fire time under the stated
      definition, proven by property tests and reflected in the founder
      reading.
- [ ] New graph edges can reach every input sub-value, and the graph raw-field
      operator changes one field by one step.
- [ ] The taxonomy and changed semantics are recorded in the reference specs;
      mutation triage, benchmark reports, progress row, and review are
      complete; the checked row and Complete spec land on clean `main`.

## Notes for AI Agents

- Planning readiness review (orchestrator, 2026-09-05): checked against the
  template, the T11.F03 track note, the node-type contract, floor (c), and the
  runtime's Gauss-Seidel and energy rules. One revision: named the index shift
  the split insert shares with node removal (T11.F09 owns learned-weight
  correspondence), corrected the pass-count claim for backward-edge splits,
  and simplified the raw-field `ref_idx` rule. Ready.
