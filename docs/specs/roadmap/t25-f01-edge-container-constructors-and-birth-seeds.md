# T25.F01 — Edge Container, Constructors and Birth Seeds

**Status**: In Progress
**Last updated**: 2026-10-14
**Feature**: T25.F01
**Track**: [T25 — Edge Groups and Per-Edge Learning](../../roadmaps/t25-edge-groups-and-learning.md)

## Goal

Development is separated from heredity, as a cell's machinery for building
synapses is separate from what the egg carries. After this feature the graph
genome has three owners and no other: the container module
(`creature::genome::cgp::container`) is the only code that edits a graph's
compute nodes and edges and the only code that keeps the birth payload
aligned with them; reproduction (`simulation::actions::cgp_reproduction`)
captures a parent's learned weights as the child draft's `BirthSeeds` before
mutation and writes no runtime field; the creature constructors
(`CreatureState::new`, `new_with_cached_fields`) consume the seeds and build
the newborn's runtime state. No operator keeps birth alignment by hand; no
struct literal builds an edge or node outside the constructors. Nothing a
run computes changes: every seeded trajectory, genome text, draw stream
and pin is identical to main.

## Non-Goals

- No new domain object: `Group`, `Membership`, `Scope`, learned scales,
  `validate_graph`, the genomic weight table and `for_genome` are F03
  onward; per-edge plasticity, `LearnedState`, sink-edge seeds and the
  removal of lazy initialization are F06.
- No size, cost or rate change (F02). No `RingProjection` change: the
  `projections` list, its five operators and the observation-only
  `#[serde(skip)]` field `ablated_channels` (`cgp.rs:150-160`) stay as they
  are until F03 deletes them.
- No genome text change: JSON keys, `Debug` rendering and `PartialEq` of
  every genome type are unchanged (`own_weight` is F03's text-only
  commit). No frontend change: the wire shape is unchanged and the
  frontend carries no `birth_weights`.
- No operator catalog, draw-order or applicability change; no re-pin.
- `BirthEdgeClass` stays as it is.
- No type or module rename (design note D11); `cgp.rs` splits into
  submodules with its public paths re-exported.

## Inputs and Invariants

| Input | Where | What F01 takes from it |
| --- | --- | --- |
| Track row and Notes | `docs/roadmaps/t25-edge-groups-and-learning.md` | The goal; "F01 is behavior-preserving: any pin that moves under it is a defect"; verification plan "F01 gate", mutation gate and slow tests on all eight. |
| Design note | `docs/strategy/edge-groups-and-learning-design-2026-10-13.md` §2, §3 (I9, I11, "Birth seeds (D1)"), §4 "Ablation (D3)", §7 (boundary table, dependency direction, "Determinism and pins"), §8 F01 contract | Scope, the container API restricted to today's write sites, the owners, the oracles and the tests-first list. |
| Genome types | `crates/v3-core/src/creature/genome/cgp.rs` | `GraphEdge {source, weight}` (`Copy`, 12 bytes), `ComputeNode {kind, inputs, plasticity}`, `OutputSink {kind, inputs}`, `CgpGraphBackendDef {birth_weights (serde skip, doc hidden), compute_nodes, output_sinks, projections}` with a hand-written `Debug`/`PartialEq` that skip `birth_weights` and print `projections` only when non-empty; the alignment-bearing methods `remove_compute_node_at`, `insert_compute_node_at`, `duplicate_compute_nodes_in_place`, `reindex_input_refs_after_removal`, `clamp_sub_idx_after_swap`, `retain_edges`. |
| Birth payload | `simulation/actions/cgp_reproduction.rs` (`capture_birth_weights`, `build_cgp_child_plasticity_weights`), `reproduction.rs:19-40, :404, :520-567` | Capture runs before `MutationEngine::apply_mutations_with_food_type_count`, build after, and the result is assigned to `child.graph_runtime.plasticity_weights` after construction (`:567`). All-empty parent rows give `None`, the fast path. |
| Alignment sites | `mutation/graph/operators.rs:345, :424, :647, :667, :693, :752`; `cgp.rs:378, :424, :467, :532` | The twelve hand-maintained seed edits listed under "Seed rules". |
| Engine wrappers | `mutation/engine/mod.rs:581-688` | Four identical snapshot, apply, validate, commit-or-rollback functions, one per domain. |
| Direct writers outside mutation | `mutation/input_ref/mod.rs:374` (`connect_declared_input`), `mutation/topology/routing.rs:82`, `mutation/graph/refinement.rs:173`, `creature/cgp_founder.rs`, `neighborhood/input_use/consumers.rs:243-300` (`ablated`), `crates/v3-lab/src/readings.rs:655-710` (`ablate`), `crates/v3-lab/src/comparator.rs` | The sites that write `pub` fields today and move onto the container or `redirect_leaf_reads`. |
| Literals and whole-list writes | `grep -rn 'GraphEdge {' crates` = 256; `'ComputeNode {'` = 184; `'CgpGraphBackendDef {'` = 108 (literals and destructuring patterns); about 15 local test helpers (`edge`, `leaf`, `memory`, `vote_edge`, `from_node`, `compute`, `node`, ...); 15 fixture sites assign `def.compute_nodes = ...`, `def.output_sinks = ...` or `node.inputs = ...` (`cgp_reproduction.rs:219, :406`, `cognition_tests.rs:213`, `mutation_effects/controls.rs:20`, `reproduction.rs:610`, `tick.rs:1740`, `genome/mod.rs:590, :641, :987`, `plasticity/tests.rs:160, :482, :640`, `applicability_tests.rs:119`, `recruitment_tests.rs:201`, `mesh_execution.rs:1105`, `recruitment_paths/fixtures.rs:161`, `neutral_connection_tests/refinement.rs:424`); fixtures build sink catalogs that are not the fixed one: repeated kinds (`cognition_tests.rs:137`), out-of-range kinds (`:213`), no sinks (`operators.rs:2314`) | Every literal migrates to `GraphEdge::new`, `ComputeNode::new` and the def builder; every whole-list write becomes a builder call. |
| Tests of the deleted reindex branches | `cgp.rs:999` `reindex_input_refs_removes_matching_and_decrements_higher`, `:1072` `reindex_across_vote_and_param_sinks`, `:1135` and `:1189` (clamp), `cgp_reproduction.rs:267` `birth_source_deletion_and_input_pruning_keep_occurrence_alignment`, `genome/tests/projection_tests.rs:86` `reindex_keeps_each_projection_on_its_reference`, `genome/mod.rs:976` `reindex_input_refs_after_removal_graph` | Test contracts that change (not pins): rewritten to the decrement-only contract below. The VM reindex tests (`input_ref/tests.rs:1210, :1280`) are untouched. |
| Runtime layouts | `state.rs:338, :404` (empty outer vector for every non-reproduction construction), `reproduction.rs:30-40` and `cgp_reproduction.rs:42-66` (every birth: one row per graph node, one slice per compute node, empty unless inherited), `reproduction.rs:885` `lamarckian_with_no_parent_weights_returns_empty`; `tests/applied_trajectory.rs:104` hashes the nested shape | Both layouts are pins: a birth gets the scaffold whether or not anything was inherited, a founder gets none. |
| Pins that hash genome text or draw streams | `tests/baseline_worlds.rs:142`, `mutation/engine/recruitment_tests.rs:49`, `tests/applied_trajectory.rs`, `neighborhood/drift.rs` (`the_walk_dates_modules_from_their_birth_and_from_every_refresh`), `tests/viability.rs:1113` (`founder_only_trajectory_digest_is_pinned`), the slow tier, every parallel-RNG tail test | Oracles: all unchanged. `neighborhood/mutation_effects/mod.rs:58` `genome_identity` is the `Debug` rendering. |
| Reference spec | `docs/reference/v3-reproduction-spec.md` §4 "Graph state" | The inheritance rules F01 preserves verbatim; its payload name becomes `BirthSeeds`. |

Invariants:

1. **Identity (I9, F01 part).** For every genome, JSON bytes and the `Debug`
   rendering are byte-identical to main, and `PartialEq` has the same truth
   table. The three `Vec` newtypes are `#[serde(transparent)]` with `Debug`
   delegating to the inner `Vec`; `GraphEdge` keeps the keys `source` and
   `weight`. `BirthSeeds` is the only `#[serde(skip)]` field on the def and
   the only one outside `RingProjection::ablated_channels`, which keeps its
   skip until F03 deletes it; the def's hand-written `Debug`/`PartialEq`
   skip exactly `BirthSeeds`; an exhaustive destructuring test makes a new
   def field a compile error.
2. **Seed shape (I11).** When `BirthSeeds` is present it has one row per
   compute node, each sized to that node's `inputs`. Every container exit
   asserts it (`debug_assert`); sink rows and group slots arrive in F06 and
   F07.
3. **One writer.** The compute-node list, the sink list and both `inputs`
   lists are newtypes with `Deref<Target = [T]>`, no `DerefMut`, no public
   constructor and no `From<Vec>`; the def's `compute_nodes` and
   `output_sinks` fields are private, read through `compute_nodes()` and
   `output_sinks()`, so no whole-list replacement can bypass the seeds;
   `sink_mut` is removed. Readers change the field read to the accessor
   (417 and 227 sites); element reads are unchanged. No `&mut GraphEdge` and no `&mut ComputeNode` leaves the
   container; `GraphEdge` fields are private behind `source()` and
   `weight()`. The container draws no RNG and imports nothing from
   `runtime`, `sensors` or `simulation`.
4. **Seed rules unchanged.** Every primitive applies the rule its
   hand-written site applies today (table below); a same-source
   `set_source` keeps the seed. Seeds travel with `Clone` (topology
   clones) and roll back with the engine's genome snapshot.
5. **Behavior.** Every deterministic output is identical to main: the
   gate's and world-set deterministic blocks (their descriptive strings
   included, `v3-cli/src/bench/mutation_effects.rs:346`), every pin in
   the Inputs table, every draw stream, and the corpus hashes.
6. **Construction.** Every reproduction draft carries `BirthSeeds`:
   either rows (I11-aligned, when the parent had a learned value) or the
   explicit empty form (no rows, no alignment, no allocation; today's
   `None` at `cgp_reproduction.rs:16-17`). Both constructors take the
   seeds out of the genome they receive and build `plasticity_weights`
   with today's rule (`reproduction.rs:30-40`, `cgp_reproduction.rs:42-66`:
   one row per graph node, one slice per compute node, a value per edge
   only on a Lamarckian node with at least one inherited value, the
   genomic weight filling the gaps; the empty form yields today's empty
   scaffold). A Graph def a mutation operator creates on the draft
   (`topology/birth.rs:15` `blank_graph_backend`, used by `detour` and
   `copy_attached` with `swap_backend`) is created with the empty form, so
   a child of a parent with no Graph node still gets today's scaffold. A
   genome with no seeds at all (seeding, the lab, instruments, observation
   clones) keeps the empty outer vector (`state.rs:338`). A built
   creature's genome carries no seeds, and `reproduction.rs` writes no
   `graph_runtime` field. Every layout is byte-identical to today's,
   which `tests/applied_trajectory.rs:104` hashes after every tick,
   newborns before their first visit included.
7. **Decrement-only reindex.** `Prune` removes only an index nothing
   addresses (`prunable_indices`, `input_ref/mod.rs:127`), so the reindex
   decrements every higher `InputLeaf.ref_idx` and projection `ref_idx`,
   removes nothing, and `debug_assert`s the removed index was unaddressed;
   the removal branches and `clamp_sub_idx_after_swap` (a within-kind
   `Swap` keeps the width, `input_ref/mod.rs:398-416`) are deleted and
   their tests (Inputs table) rewritten to this contract.
8. **Observation copies.** `redirect_leaf_reads(f: FnMut(ref_idx, sub_idx)
   -> Option<u16>)` rewrites `InputLeaf.ref_idx` on a copy with no seed
   change and no assertion; callers: `consumers::ablated` (for
   `input_use/mod.rs:249, :271`, `opportunity/assay.rs:267`) and the lab's
   `ablate`; a copy never re-enters mutation or validation. Projection
   masking through `ablated_channels` is untouched.
9. **Hot path.** No new work per visit or per birth: the empty seed form
   is a tag, not an allocation, and the scaffold is built where it is
   built today; getters are `#[inline]`; `GraphEdge` stays `Copy` at 12
   bytes (main's layout: a 6-byte `GraphSource`, padding, an `f32`).

Seed rules, today's site → the container primitive that owns it:

| Today | Primitive | Rule |
| --- | --- | --- |
| `cgp.rs:370` `remove_compute_node_at` | `remove_compute_node(idx)` | row removed; the seed of every edge reading the removed node cleared; sources remapped as today |
| `cgp.rs:406` `insert_compute_node_at` | `insert_compute_node(idx, node)` | a `None` row sized to the node's inputs; sources shifted as today |
| `cgp.rs:446` `duplicate_compute_nodes_in_place` | `duplicate_compute_nodes(sources)` | rows copied beside their originals |
| `cgp.rs:490` `reindex_input_refs_after_removal`, `:530` `retain_edges` | `reindex_input_refs_after_removal(removed)` (decrement-only) | no edge removed, rows untouched |
| `operators.rs:343` `append_compute_node` | `append_compute_node(node)` | a `None` row; no source remap |
| `operators.rs:402-445` `split_existing_edge` | `split_edge(occurrence, identity_node)` | consumer keeps its seed; the new identity edge has none |
| `operators.rs:640` `add_edge`, `operators.rs:295` `push_edge` | `push_edge(occurrence surface, edge)` | a `None` entry on a compute surface |
| `operators.rs:662` `remove_edge`, `:303` `remove_edge_at` | `remove_edge(occurrence)` | the entry removed |
| `operators.rs:674` `retarget_edge`, `:941` `raw_field_mutation` | `EdgeMut::set_source` | cleared only when the source changes |
| `operators.rs:698` `alter_edge_weight_in_def`, `refinement.rs:173` | `EdgeMut::set_weight` | cleared |
| `operators.rs:740` copy edge bundle | `copy_edges(from, to)` | rows appended with the edges |
| `operators.rs:532`, `hebbian.rs:202, :218, :229` | `set_compute_kind(idx, kind)`, `set_node_plasticity(idx, Option<PlasticityConfig>)`, `node_plasticity_mut(idx)` | no seed change |

Read API: `compute_nodes()`, `output_sinks()`, `edges()`,
`edge_at(EdgeOccurrence)`, `sink(kind)`, the canonical enumeration
(today's `edge_sites`, compute inputs then sink inputs) defining
`EdgeOccurrence { surface: EdgeSurface, index }`, and `birth_seeds()`.
Constructors: `GraphEdge::new(source, weight)`, `ComputeNode::new(kind,
inputs, plasticity)`, `CgpGraphBackendDef::new_with_fixed_outputs()`
unchanged, and a seedless builder (`with_compute_nodes`,
`with_output_sinks`, `with_projections`, consuming `self`) that stores the
supplied lists exactly as given, with no filling, sorting, deduplication or
validation of sink kinds, so every existing fixture catalog is
reproducible. Seeds: `seed_birth(BirthSeeds)` (asserts I11 on rows; the
empty form carries none), `birth_seeds()` and `take_birth_seeds()`.

Options weighed: [the readings file](../../progress/readings/t25-f01.md).

## Telemetry

Not applicable: a behavior-preserving refactor that adds no counter, config
value, mechanism switch or span.

## Verification Plan

**Question.** F01 claims edit, capture and build can move to three owners
with no behavior change; a differing gate deterministic block, a moved
pin, a changed draw-stream tail or a corpus hash mismatch would show it
wrong.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate` | The change touches tick-loop and reproduction code with no population reading in question; the gate's deterministic blocks against the epoch are the byte-identity reading, in seconds. |
| Mutation gate | `run` | The container primitives, the seed rules and the generic engine wrapper are new production logic in `v3-core` whose behavior the ported tests must pin. |
| Slow tests (`make rust-test-slow`) | `run` | The mutation operators and both edge containers are rewritten onto the container, and the slow tier pins the recruitment-paths readings that such a change would move. |

## Implementation Tasks

- [x] Oracles first, on main's behavior before any production change, in
      one commit: the golden Lamarckian corpus (below); the carrying-cost-0
      sibling of `founder_only_trajectory` (`genome_carry_cost_per_unit =
      0`, same digest fields: id, position, energy bits, age, generation,
      food); run `cargo test -p v3-core --test viability` first.
- [x] `edge` module: private `GraphEdge` fields, `new`, getters;
      `ComputeNode::new`; the def builder; every literal and local helper
      migrated across `v3-core`, `v3-cli`, `v3-lab`, `v3-telemetry`.
- [x] `container` module: the newtypes and private list fields, the
      seedless builder, `BirthSeeds` as the private seed field, the read
      API, the primitives of the seed-rule table, `EdgeOccurrence` and the
      canonical enumeration, the decrement-only reindex with its tests
      rewritten and `clamp_sub_idx_after_swap` deleted,
      `redirect_leaf_reads`, the I11 assertion, the exhaustive
      destructuring test.
- [x] Writers moved: graph operators, hebbian, projection (edge-free),
      recruitment, refinement, routing, `connect_declared_input`, the
      founder, instruments, `consumers::ablated`, the lab's `ablate` and
      comparator.
- [x] Reproduction: `cgp_reproduction` captures through `seed_birth`;
      `CreatureState::new` and `new_with_cached_fields` take seeds out and
      build `plasticity_weights` with today's rule; `reproduction.rs:567`
      deleted; `reproduction.rs:806-1016` and `plasticity/tests.rs:554`
      ported scenario by scenario with identical expected values
      (`reproduction.rs:938` as an explicit backend-replacement test,
      same-source retargets included); `v3-reproduction-spec.md` §4 names
      `BirthSeeds` as the temporary correspondence.
- [x] Engine: one generic snapshot, validate, commit-or-rollback wrapper;
      `engine/tests.rs:1757` ported to the seed accessor.
- [x] `the_walk_dates_modules_from_their_birth_and_from_every_refresh`
      rewritten to a constructed lineage asserting the dating properties
      (birth date, refresh folding, internal-change dating) without
      pinning draw history, beside an `observe` test that fails without
      the refresh fold; the old pins are in the readings.
- [x] Closing commit: the corpus test retired, its hashes kept in
      `docs/progress/readings/t25-f01.md`.

**Golden Lamarckian corpus.** A seeded `v3-core` unit test over a fixed
multi-node Lamarckian genome and parent weights: each scenario captures,
applies one named operator or engine event, builds the child weights and
hashes the child genome's `Debug` text with the weights' bits. It asserts
at least one payload-carrying applied scenario for every site and branch
in the table (two by nature: the rollback is a payload-carrying draft whose
event is rejected, `engine/mod.rs:653-654`, seeds unchanged; the all-empty
line is a seedless capture followed by a build):

| Coverage line | Today's write site |
| --- | --- |
| remove, insert, append, duplicate compute node | `cgp.rs:370, :406`, `operators.rs:343` (disconnected and bootstrap forms), `cgp.rs:446` |
| split: compute-input consumer; sink consumer | `operators.rs:402-445`, both branches |
| push edge on a compute surface; on a sink surface | `operators.rs:640` |
| remove edge on each surface | `operators.rs:662` |
| retarget: source changed; same source drawn | `operators.rs:674` |
| raw field: `InputLeaf` ref, `InputLeaf` sub, `SharedMemory` slot, previous flag, `ComputeNode` index | `operators.rs:941` |
| alter weight; refinement weight write | `operators.rs:698`, `refinement.rs:173` |
| copy edge bundle | `operators.rs:740` |
| compute kind; plasticity enable, disable, parameter change | `operators.rs:532`, `hebbian.rs:202, :218, :229` |
| `Prune` reindex; recruitment and routing sink pushes | `input_ref/mod.rs:391, :374`, `routing.rs:82` |
| topology: `CopyNode`, `CopyMeshBackwardSlice`, `CopyMeshForwardSlice`, `RemoveNode`, `Swap` | `reproduction.rs:938` scenarios |
| one rollback (`ParseabilityViolation`); one all-empty parent | `engine/tests.rs:1757`; `cgp_reproduction.rs:9-17` |

## Verification

- [x] `cargo test -p v3-core --test viability` -> all pass, both existing
      digests and the carrying-cost-0 sibling unchanged (`2ecbbb10`).
- [x] Corpus: hashes recorded on main equal the hashes after every
      production commit; recorded in the readings.
- [x] Seeded births through both constructors: a Lamarckian parent with
      learned weights reproduces on the mutated branch (`CreatureState::new`)
      and on the fast path (`new_with_cached_fields`, `applied_events ==
      0`); each child carries no seeds, holds the inherited weights in its
      own storage, starts with empty traces, node state, outputs and
      dispatch record, and leaves the parent's genome and runtime state
      unchanged (`birth_applied_reproduction_keeps_parent_and_resets_all_newborn_credit`
      ported, with a fast-path twin); a parent with nothing learned gives
      a child whose `plasticity_weights` before its first visit is today's
      scaffold (one row per graph node, one empty slice per compute node),
      so does a child of a VM-only parent whose batch created a Graph
      node, and a seeded founder's stays empty.
- [ ] `cargo test --workspace` -> every existing pin unchanged (Inputs
      table).
- [x] `grep -rn 'GraphEdge {\|ComputeNode {' crates` -> only the
      constructors and destructuring patterns; `grep -rn birth_weights
      crates frontend` -> only the legacy `genome_identity` string in
      `v3-cli/src/bench/mutation_effects.rs:346` (world-set deterministic
      output, text unchanged); `grep -rn 'compute_nodes = \|output_sinks =
      ' crates` -> only inside the container module.
- [ ] `make check` -> 0, `make rust-test-slow` -> 0 with recorded readings
      unchanged.
- [ ] Mutation gate: fresh `MUTANTS_ITERATE=0 make rust-mutants`, its
      summary line, output path, and every survivor resolved as killed,
      equivalent, or deferred.
- [x] Gate profile: summary at
      `docs/progress/features/t25-f01-edge-container-constructors-and-birth-seeds.json`,
      deterministic blocks identical to the epoch, no new full report
      staged.

## Performance and Goal Impact

**Predeclaration — written before the run.** A refactor: no mechanism
reaches creatures, no indicator has a direction, no compute change is
expected. The gate's wall-clock is compared against the current epoch under
the ordinary thresholds and its deterministic blocks must be
byte-identical; a severe flag, a threshold crossing or any deterministic
difference is a defect to fix, not a cost to accept. Predeclared re-pins:
none.

**Measured verdict.** Exit 0, `severe=false`, deterministic block equal to the epoch.

- Summaries of the declared profiles:
  [gate](../../progress/features/t25-f01-edge-container-constructors-and-birth-seeds.json).
- Full readings: [`docs/progress/readings/t25-f01.md`](../../progress/readings/t25-f01.md).

## Success Criteria

- [x] One owner edits the graph: every write to compute nodes and edges in
      every crate goes through the container; no `&mut GraphEdge` or `&mut
      ComputeNode` escapes it; no operator touches the birth payload.
- [x] One owner captures: `cgp_reproduction` writes seeds before mutation
      and no runtime field; one owner builds: the creature constructors
      consume the seeds.
- [x] No struct literal builds an edge, a compute node or a def outside the
      constructors, in any crate or test.
- [ ] No behavior change: every pin, draw-stream tail, corpus hash and
      gate deterministic block is unchanged.
- [ ] `make check`, the mutation gate and the slow tier pass.

## Notes for AI Agents

- Decision: (user, track Notes 2026-10-13) F01 is behavior-preserving; any
  pin that moves under it is a defect, never a re-pin.
- Decision: (design note D11) no type or module is renamed; `cgp.rs` splits
  into submodules with its public paths re-exported.
