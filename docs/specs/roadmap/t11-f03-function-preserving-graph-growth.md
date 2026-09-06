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
  keeping its old weight. The new node's output passes through
  `sanitize_output` (NaN → 0, clamp to ±1e9) like every compute node's
  output, so exact reproduction of the old source holds for values already
  inside that range; a source value outside it (already sanitized identically
  whether read directly or through the new node) is unaffected. When the
  consumer is a compute node at index `c`,
  insert the split node at index `c` and remap every `ComputeNode(i >= c)`
  reference across all five surfaces to `i + 1`, so Gauss-Seidel pass order is
  preserved: the new node reads its source in the same pass phase the consumer
  used to, and the consumer reads the new node in the current pass. When the
  consumer is a sink, action slot, or execute gate, append. Final outputs are
  identical either way; a split of a backward or self edge may extend
  convergence by at most one pass because the new node's delta lags its
  source's by one pass. **Documented exception**: an append-branch split
  whose retargeted edge's old source was an `InputLeaf` resolving to
  `DynamicIntrospection(EnergyCurrent)`, on a graph carrying plasticity, is
  not neutral. The new identity node's cached value is read from
  `curr_outputs` at effects time (the value it computed during the last
  relaxation pass, before the post-convergence plasticity-cost deduction),
  whereas a direct edge on the same non-compute surface is resolved fresh at
  effects time (after that deduction). The two differ by
  `plasticity_cost * weight`. Under the production default
  (`plasticity_update_cost = 0.0`) this has no observable effect; it applies
  once that cost is configured nonzero. A code fix (skip that source for
  post-convergence consumers) is deferred to T11.F08 (see Notes for AI
  Agents). A split never targets an edge whose source is
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

- [x] Write failing tests first: property tests, seeded by an RNG drawn over
      `any::<u64>()`, applied to two fixed hand-built fixtures (`base_def`,
      forward/backward/self-loop edges and no plasticity; `plasticity_def`,
      added post-review for a Hebbian-plasticity compute node and a
      `DynamicIntrospection(EnergyCurrent)` reference) and three fixed
      scenarios, not arbitrary generated graph defs, that each growth
      operator (three add-node forms, copy node, copy subgraph,
      input-reference add on graph and VM nodes) yields identical execute
      outputs, actions, and shared-memory writes with ample energy and
      passes on both fixtures; that a compute-consumer split preserves every
      node's per-pass value (on `base_def`); a property that over seeds
      every drawn `InputLeaf.sub_idx` stays within its reference's width
      (the reach claim itself is a separate seeded example test); a property
      that raw-field mutation changes at most one compute-node edge and
      never its variant (the one-unit-step magnitude is example-tested
      separately); example tests for the skip cases and index remapping.
- [x] Implement the add-node forms with insert-with-remap, the faithful copy,
      unwired input-reference add on both backends, config-driven sub-index
      sampling, and the one-field raw step; delete the obsolete tests that
      pinned spraying, backlinks, and auto-wiring.
- [x] Update `v3-mutation-spec.md` (taxonomy, graph and InputRef domain
      entries, node-type contract owner text) and `v3-graph-backend-spec.md`.
- [x] Self-review the diff (`simplify`), then fresh `make rust-mutants`; record
      evidence below.
- [x] Store gate and single goal reports at
      `docs/progress/features/t11-f03-function-preserving-graph-growth.json`
      and `...-goal.json`, append both to `docs/progress/benchmark-series.json`,
      and add the T11.F03 row to `docs/progress.md`.

## Verification

- [x] TDD evidence: wrote `crates/v3-core/src/mutation/graph/tests/operators.rs`
      (10 property/example tests: neutrality of the three add-node forms,
      `CopyComputeNode`, `CopySubgraph`, unwired `InputRef.Add` on graph and
      VM nodes — each a seeded-RNG property run over two fixed fixtures,
      `base_def` and, post-review, `plasticity_def` — split per-pass
      preservation on `base_def`, an `InputLeaf.sub_idx`-range property (the
      reach claim is a separate seeded example test), and a raw-field
      at-most-one-compute-edge-changed/variant-preserved property (the
      one-unit-step magnitude is example-tested separately)) against the old
      operator signatures first; `cargo check` failed until the operators
      were rewritten. During
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
      production edit (25 passed), again after the production edits (25
      passed), and again at final closure after the mutation-survivor
      remediation tests and the rustfmt/clippy fixes (25 passed; that
      remediation was test-only, so no production behavior was at risk);
      `cargo check --workspace --all-targets` after every coherent Rust edit
      (enforced by the implementer-compile-check hook); focused
      `mutation::graph` (65 passed), `mutation::input_ref` (23 passed),
      `mutation::engine` (all passed, including the two stress tests fixed
      above), `mutational_neighborhood` (4 passed), and `reproducibility` (1
      passed) tests pass; full `cargo test -p v3-core --lib` (1126 passed)
      and `make check` (exit 0, see the roadmap-check/`make check` bullet
      below) pass.
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
- [x] `make bench PROFILE=gate FEATURE=t11-f03-function-preserving-graph-growth`
      and one `PROFILE=goal` run stored at
      `docs/progress/features/t11-f03-function-preserving-graph-growth.json`
      and `...-goal.json`; both `severe=false`; second goal run: not
      applicable per the 2026-09-05 workflow decision.
- [x] `make roadmap-check` passed after every document edit. `make check`
      ran to completion with an explicitly captured `exit=0` at commit
      `4722a362` (rustfmt and one clippy `nonminimal_bool` fix on a test
      assertion were needed first and are included in that commit and
      `4324e19c`; both are test/format-only, postdate the closure mutants
      run, and cannot change its `3 missed, 84 caught, 12 unviable`
      result). This spec's own text changed after `4722a362`; no other file
      changed after that commit, so the `make check` result still describes
      the tree. The orchestrator confirms `make check` again at the commit
      that actually lands on `main`.

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

### Measured results

Reports: `docs/progress/features/t11-f03-function-preserving-graph-growth.json`
(gate) and `...-goal.json` (goal); both `comparison.severe=false` against
both stated references.

Gate current / delta vs T10.F10 / delta vs T11.F02:
| counter | current | vs T10.F10 | vs T11.F02 |
|---|---:|---:|---:|
| mesh hops | 1.999410 | +0.052443% | +0.022962% |
| VM steps | 28.031758 | +0.012584% | -0.043239% |
| graph relax | 3.000000 | +0.045888% | +0.045888% |
| plasticity | 0.000000 | n/a (both zero) | n/a (both zero) |
| actions | 1.000000 | 0.000000% | 0.000000% |
| births | 0.001148 | -5.280528% | -4.013378% |

Goal current / delta vs T01.F12 / delta vs T11.F02:
| counter | current | vs T01.F12 | vs T11.F02 |
|---|---:|---:|---:|
| mesh hops | 2.998942 | -5.098574% | -1.501159% |
| VM steps | 151.579212 | -86.122256% | -71.440711% |
| graph relax | 5.571117 | -2.073774% | +3.388342% |
| plasticity | 0.115330 | +14.765354% (flagged) | -7.036918% |
| actions | 1.056764 | -2.679356% | -8.712358% |
| births | 0.007915 | -2.716323% | -3.687028% |

VM steps fall sharply on both references because input-reference `Add` on
VM nodes no longer auto-inserts a `ReadInput` instruction (the removed
auto-wiring), so the goal-horizon evolved population carries far fewer VM
instructions per creature; this is the intended effect of this feature's
`InputRef.Add` change, not a per-opcode runtime change. The flagged
plasticity delta is a goal-population difference (see lineage note below),
not a work-counter definition change; it is not severe and is within the
predeclared "populations differ, so every counter may move" allowance.

Founder neighborhood rows (silent/changed/dead; applied/skipped), current
vs the T11.F02 gate reference — every VM, topology, and other graph/input-ref
row not listed below is byte-identical to T11.F02:
| operator | T11.F02 | current |
|---|---|---|
| graph:AddInternalGraphNode | 0.020000/0.980000/0.000000 (50/0) | 1.000000/0.000000/0.000000 (50/0) |
| graph:CopyInternalNode | 0.580000/0.420000/0.000000 (50/0) | 1.000000/0.000000/0.000000 (50/0) |
| graph:CopySubgraph | 1.000000/0.000000/0.000000 (50/0) | 1.000000/0.000000/0.000000 (50/0) |
| graph:GraphRawFieldMutation | 0.100000/0.900000/0.000000 (50/0) | 0.080000/0.920000/0.000000 (50/0) |
| graph:AddGraphEdge | 0.900000/0.100000/0.000000 (50/0) | 0.880000/0.120000/0.000000 (50/0) |
| graph:RetargetGraphEdge | 0.020000/0.980000/0.000000 (50/0) | 0.020000/0.980000/0.000000 (50/0) |
| input_ref:Add | 0.660000/0.340000/0.000000 (50/0) | 1.000000/0.000000/0.000000 (50/0) |

`AddInternalGraphNode`, `CopyInternalNode`, `CopySubgraph`, and
`input_ref:Add` land exactly on the predeclared "about 1.00" target.
`AddGraphEdge` and `RetargetGraphEdge` moved within the "either way, report
without adjustment" allowance (-0.02 and unchanged respectively).
`GraphRawFieldMutation` moved **against** the predeclared direction: silent
fell from 0.10 to 0.08 (changed rose to 0.92) instead of rising toward 0.74.
Measured as one trial's worth of movement on the 50-trial battery (5 silent
→ 4 silent), within the same trial-coarseness already applied to the birth
buckets below, not a claim that the one-field-one-unit step is now more
likely to produce an observable change in general. This is reported as
measured, not adjusted toward the prediction; no floor is asserted here
(floor (c) is T11.F10's).

Founder mutated-birth outcomes (silent/changed/dead; trials), current vs
T11.F02 gate reference:
| bucket | T11.F02 | current |
|---|---|---|
| any events | 0.068182/0.727273/0.204545 (44) | 0.068182/0.704545/0.227273 (44) |
| 1 event | 0.500000/0.500000/0.000000 (4) | 0.500000/0.500000/0.000000 (4) |

Silence is flat (not risen) and dead rose slightly (9→10 of 44), against
the predeclared "silence should rise and dead fall" — the opposite of the
predicted direction on the dead fraction, within the stated 44-birth
coarseness (one birth's outcome bucket). The 1-event bucket (4 births) is
unchanged, as T11.F01 warned its coarseness would likely show.

Goal-profile founder rows reproduce the same gate-row movements exactly
(same 50-trial founder battery, run under the goal profile); no additional
divergence. Evolved-sample, lineage, and persistence readings moved in both
directions across seeds (e.g. seed 11 lineage entropy 3.296956→3.148096,
clade count 118→147; seed 33 entropy 2.698148→2.844166, clade count
96→123; final populations moved -1.7% to -21.9% across seeds with no
extinction) — these are population-composition differences from the
inherited mutation map, not cognition claims, consistent with the
predeclared "may shift either way" allowance. Goal memory-sensitivity
(different-from-either fraction, T11.F02→current): seed 11 0.001107→
0.000000, seed 22 0.000211→0.002685 (zeroed fraction also newly nonzero at
0.001718, vs. 0.000000 in every prior closure), seed 33 0.000000→0.000000 —
also a changed-population reading, not a cognition claim. No operator family
was disabled or down-weighted.

## Success Criteria

- [x] Every growth operator is neutral at fire time under the stated
      definition, proven by property tests and reflected in the founder
      reading (`AddInternalGraphNode`, `CopyInternalNode`, `CopySubgraph`,
      and input-reference `Add` all read silent 1.00 on the T11.F03 gate
      report).
- [x] New graph edges can reach every input sub-value, and the graph raw-field
      operator changes one field by one step.
- [ ] The taxonomy and changed semantics are recorded in the reference specs;
      mutation triage, benchmark reports, progress row, and review are
      complete; the checked row and Complete spec land on clean `main`. Not
      done: review and the roadmap row/Complete status are the
      orchestrator's to close.

## Notes for AI Agents

- Planning readiness review (orchestrator, 2026-09-05): checked against the
  template, the T11.F03 track note, the node-type contract, floor (c), and the
  runtime's Gauss-Seidel and energy rules. One revision: named the index shift
  the split insert shares with node removal (T11.F09 owns learned-weight
  correspondence), corrected the pass-count claim for backward-edge splits,
  and simplified the raw-field `ref_idx` rule. Ready.
- Post-review remediation (2026-09-05, reviewer counts P1=0, P2=2, P3=7): all
  7 items addressed except one deferred finding recorded here.
  - Deferred (P2-2): the split append-branch `InputLeaf(EnergyCurrent)` +
    plasticity exception (see Inputs and Invariants and the
    `AddComputeNode` reference entry) has a code fix — skip that source for
    post-convergence consumers, or resolve it fresh at effects time instead
    of caching it mid-relaxation — deferred to T11.F08. No observable effect
    exists today because production's `plasticity_update_cost` default is
    0.0; the fix matters once that cost is configured nonzero.
  - Deferred (P3-7): `graph_def_mut_with_input_refs` clones `input_refs`
    (`crates/v3-core/src/mutation/graph/operators.rs`) where a disjoint
    field borrow (splitting `&mut NodeGenome` into its `backend_def` and
    `input_refs` fields separately) could avoid the allocation. Deferred: a
    production edit here would invalidate this closure's mutation-testing
    run and stored benchmark reports, both already recorded as this
    feature's evidence.
  - The deferred mutation-testing survivor at
    `crates/v3-core/src/mutation/graph/operators.rs:191` (`random_graph_source`,
    `<` → `<=`) is unchanged by this remediation pass (test-only comment
    reword, no test or production semantics changed); see the `make
    rust-mutants` Verification bullet for its full disposition.
