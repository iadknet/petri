# T30.F10 — One Operator Catalog

**Status**: In Progress
**Last updated**: 2026-10-28
**Feature**: T30.F10
**Track**: [T30 — Core Boundaries](../../roadmaps/t30-core-boundaries.md)

## Goal

Offspring mutate exactly as before, with each mutation operator defined
once: `MutationOperator` is a nested enum over the four domain enums, from
which its domain, key, weight and complexity effect derive, and the engine
selects and dispatches every event through one path for every domain, with
its draws in today's order. Adding an operator edits its domain enum's
tables and dispatch and the readers that classify it, never the engine or
a parallel catalog. Every pin, deterministic block and existing test value
is identical to main's; any that moves is a defect.

## Non-Goals

- Operators reporting the node they added and the `tracked_before` clone
  (T30.F11); the sink catalog (T30.F12); the gate and route resolution
  moving into the genome and the deletion of the test-only
  `TopologyOperator::random` and `VmOperator::random` samplers (T30.F05,
  user 2026-10-27); retiring `neighborhood/recruitment_paths` (T30.F09).
- Any change to an operator's weight, key string, complexity effect,
  applicability predicate, draws or dispatch target; to the domain draw
  (`mesh_layer_probability`, then three node-internal domains at equal
  probability); or to the reachable-bias values.
- New report fields (T25.F12 adds them to the one report this feature
  leaves).
- Any wire type, benchmark schema or frontend change.

## Inputs and Invariants

Inputs: the T30 row and its Notes entry for F10 (user direction,
2026-10-27), the [F10–F12 design review](../../progress/readings/t30-f10-f12-design-review.md),
`docs/architecture.md` rules 1, 3 and 4 and section 3, the T30 notes on
F05, F09 and F11, the T25.F12 row (depends on F10), and the code on
`ac3747005`. Code sites, draw sequence, options and the preparatory audit:
[readings](../../progress/readings/t30-f10.md).

Invariants:

1. **One catalog.** `mutation/types` defines
   `enum MutationOperator { Topology(TopologyOperator), Vm(VmOperator), Graph(GraphOperator), InputRef(InputRefOperator) }`,
   variants in that order, deriving `Clone, Copy, PartialEq, Eq,
   PartialOrd, Ord, Hash`. Its `Debug` is written by hand and prints
   today's flat name: the domain name then the inner variant's `Debug`,
   the domain name omitted when the inner name already starts with it
   (`TopologyAddNode`, `VmConstantMutation`, `GraphRawFieldMutation`),
   because `engine/draw_stream_tests.rs:31` hashes the event records'
   `Debug` text and that pin must not move; a unit test checks those three
   names and one `InputRef` name. The four domain enums keep their
   variants and `ALL` lists in today's order and add `PartialOrd, Ord`, so
   the nested enum's derived order equals today's flat order (`BTreeMap`
   and serialized key order unchanged). The flat
   variants, `MutationOperator::all()`, the `domain()` and `as_key()`
   tables, `topology_operator_key`, `vm_operator_key`,
   `input_ref_operator_key` and `GraphOperator::operator` are deleted.
2. **Derived facts.** `domain()` reads the variant. `as_key()` delegates to
   the domain enum's `as_key(self) -> &'static str`, which carries today's
   exact string for every operator (`Topology.AddNode`,
   `Vm.VmConstantMutation`, `Vm.CopyInstructionBlock`,
   `Graph.GraphRawFieldMutation`, …, verbatim from `types/mod.rs:319-387`).
   `complexity_effect()` delegates to the domain enum, which owns its table
   with today's values (`GraphOperator` gets its own; the flat table goes).
   `weight(&MutationConfig) -> u16` returns
   `tuned_weight(config.large_copy_weight_percent)` for Topology and
   `u16::from(weight())` for the other three, so each domain's total keeps
   today's units. `MutationOperator::ALL: [Self; 69]` is built from the
   four domain `ALL`s in domain order; `MutationDomain::catalog(self) ->
   &'static [MutationOperator]` is that domain's slice of it;
   `MutationDomain::reachable_bias(self, &ReachableBiasConfig) -> f64`
   picks today's field (`config` imports nothing from `mutation`). Four
   `impl From<XOperator> for MutationOperator` replace the deleted
   mappings. A unit test asserts the 69 keys are distinct, each starts
   with its domain's `as_key()` and a dot, `ALL` has one entry per domain
   variant, `complexity_effect` agrees with the domain enum for every
   entry, and every entry serializes to its `as_key()` string and
   deserializes back to itself (invariant 3).
3. **One serde form.** `MutationOperator` serializes as its `as_key()`
   string and deserializes from it (serde's `into`/`try_from` attributes
   or a hand impl, the implementer's choice); the derived variant-name form
   (`GraphAddGraphEdge`) ends. Its only exporters follow: the lab's
   `key_of` (`v3-lab/src/ladder/track.rs:117`, its tests and
   `tests/fixtures/summary-v2-wall.json`) and the recruitment-paths records
   (`recruitment_paths/records.rs`; the `v3-cli/tests/bench_artifacts.rs:1517`
   literal). Every benchmark summary key, server payload and frontend
   label already reads `as_key()` and is unchanged.
4. **One report and one dispatch.** `mutation/types` defines
   `EventReport { class: Option<BirthEdgeClass>, attribute_scope: Option<Scope>, bundle_edits: BundleEdits }`
   (today's `GraphEventReport`, moved and renamed, `Default`). Each
   domain's production entry (`TopologyMutator::apply_in`,
   `VmMutator::apply`, `GraphMutator::apply_classified`,
   `InputRefMutator::apply_classified`) returns
   `Result<(TargetReachability, EventReport), MutationSkipReason>`:
   Topology and VM `EventReport::default()`, Graph as today, InputRef its
   class and bundle edits; `InputRefEventReport`, the engine's
   `ClassifiedResult` and its four `apply_*_event` wrappers are deleted.
   One `pub(crate)` ungated dispatch in `mutation/engine`
   (`dispatch(genome, operator, targets, rng, config, context)`, or the
   implementer's name) matches the nested variant and calls the domain
   entry; the engine's `apply_event` is `apply_gated` over it, the gate's
   snapshot, parseability and `validate_graph` checks and restore
   unchanged. The per-operator battery (`neighborhood/operators.rs:45-124`)
   deletes its private `OperatorKind` (a nested operator enum with its own
   `all`, `family`, `name`, `seed_base`, `reachable_bias` and ungated
   per-domain `apply`) and iterates `MutationOperator` through that
   dispatch and `MutationDomain::reachable_bias`, keeping its row order
   (`Vm`, `Graph`, `Topology`, `InputRef` catalogs), `family` labels, seed
   bases and the inner enum's `Debug` as each row's name, so every tally
   is unchanged.
5. **One selection.** In `apply_mutations_on_units`, each event draws its
   domain as today (`gen_bool(config.mesh_layer_probability)`; when false
   `gen_range(0u8..3)`: 0 Vm, 1 Graph, 2 InputRef) and then calls one
   function (`select_and_apply(domain, ..)` or the implementer's name)
   that: builds `available` from `domain.catalog()` keeping operators that
   pass `!restricted || complexity_effect().is_decreasing()` and
   `weight(config) > 0` (a no-op outside Topology); loops `select_weighted_index` by `weight(config)`; takes
   `tracked_before` for the three tracked operators (`WORKAROUND(T30.F11)`,
   marker kept); builds the selector from
   `domain.reachable_bias(&config.reachable_bias)` through today's
   `pressure_adjusted_bias` and `executed_bias`; applies through
   `apply_event`; on `NoApplicableTarget` records the discard with the
   selector's first pick and `swap_remove`s the operator; and returns the
   `SelectedEvent` (now holding the `EventReport`) or `None` when the list
   is exhausted. The recording that follows (`record_domain_skip`,
   `record_attempt`, `record_event`, the applied branch with its bundle,
   scope, class, added-node and membership updates) is unchanged in order.
   The `too_many_lines` allowance on `apply_mutations_on_units` is deleted;
   `vm/operators.rs:102` keeps its own.
6. **Callers.** Each site below changes as its row says and nothing else
   about it; `MutationDomain`, `MutationSkipReason`, `BirthEdgeClass`,
   `MutationSummary`, `MutationEventRecord` and `SimStats` keep their
   shapes.

   | Site | Change |
   | --- | --- |
   | `EditSurface::of`, `neighborhood/recruitment.rs:117` | matches `Topology(op)` on `TopologyOperator` and the other three variants by domain; today's classification for every operator, no wildcard |
   | `ProductionEvent::operator`, `recruitment_paths/qualification.rs:49`; `recruitment_paths/fixtures.rs:242` | use `From`; `ProductionEvent` carries `WORKAROUND(T30.F09): a second nested operator enum; MutationOperator is the one catalog and T30.F09 deletes this module` |
   | `birth_edge_class`, the three added-node functions in `engine/mod.rs`, `v3-cli/src/bench/bundles.rs:258` | name the nested variants |
   | every test naming a flat variant (17 files, readings) | names the nested one or `XOperator::V.into()`; assertions and expected values kept; no test deleted or ignored |
7. **No behavior change.** `engine/draw_stream_tests.rs` keeps its pinned
   hash; `tests/reproducibility.rs` passes unchanged; the gate's
   deterministic blocks (the mutational-neighborhood battery's rows among
   them) equal the epoch; every `as_key` string, weight,
   `ComplexityEffect` and applicability predicate is the value on
   `ac3747005`. The only output whose spelling changes is invariant 3's,
   outside every pin and stored benchmark key.

## Architecture

Owners touched: `mutation` (owner of supply, dispatch and every birth
draw), whose catalog type, engine path and report shape change; readers
(`neighborhood/{recruitment,operators}.rs`, `recruitment_paths/`),
`v3-cli`, `v3-lab` and tests across crates, which rename variants, follow
the one key or call the one dispatch. No debt in section 5 is paid or
extended; `scripts/architecture-debts.txt` is untouched. Shared rule
changed: "an operator's identity, domain, key, weight and complexity
effect" had two owners, the domain enum and the flat enum in
`mutation/types`, kept in agreement by hand; after the feature the domain
enum owns each fact and the nested enum derives it (rules 1 and 3), and
the battery calls mutation's dispatch instead of repeating it. Rule 4 is
unchanged: the engine samples, the domain mutators apply, and
`apply_gated` stays until T30.F05 moves the gate. Boundaries do not
change and the desired state needs no edit (section 4 already describes
this dispatch). T30.F11's `tracked_before` clone is used as it is under
its marker: the node an operator added is F11's to report.

## Telemetry

Not applicable: a type and control-flow refactor adding no signal, config
value or mechanism switch; nothing a run does changes.

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| none | — | — | — |

## Verification Plan

**Question.** F10 claims the four selection blocks and the duplicate
catalog can become one path and one nested enum without moving a single
draw, key, weight or deterministic block; a changed draw-stream hash, a
failed reproducibility test, a differing gate block or a gate compute
regression past its threshold would show it wrong.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate` | The track's verification note declares each row gate with pins unchanged; the gate's deterministic blocks against the epoch are the byte-identity reading and its compute threshold the cost reading for a hot path that now goes through one generic function, in seconds; no population reading is in question. |
| Slow tests (`make rust-test-slow`) | `run` | The workflow's trigger names a change to mutation operators or their draws and to recruitment code; this feature rewrites the engine path those draws come through and edits `recruitment_paths/qualification.rs`, and the tier's recruitment-paths pins read that path end to end. |

## Implementation Tasks

- [x] 1. In `mutation/types`: write invariant 2's catalog test first; then
      the nested `MutationOperator`, `EventReport`, `ALL`,
      `MutationDomain::catalog` and `reachable_bias`, the derived facts,
      the four `From` impls, the hand-written `Debug` and the `as_key()`
      serde form (invariants 1–4); `as_key` and `Ord` on the four domain
      enums, `GraphOperator`'s own `complexity_effect` table.
- [x] 2. The four domain entries return `(TargetReachability, EventReport)`
      (invariant 4); delete `GraphEventReport` (moved),
      `InputRefEventReport` and `GraphOperator::operator`.
- [x] 3. In `engine/mod.rs`: the ungated `dispatch`, `apply_event`, the one
      selection function and the rewritten event loop (invariants 4 and
      5); delete the three key functions, the four wrappers,
      `ClassifiedResult` and the `too_many_lines` allowance; the battery
      in `neighborhood/operators.rs` drops `OperatorKind` for the dispatch.
- [ ] 4. Callers of invariant 6 (compiler-driven): readers,
      `recruitment_paths` with the `WORKAROUND(T30.F09)` comment on
      `ProductionEvent`, `v3-cli`, `v3-lab` (`key_of`, tests, fixture),
      and every test file naming a flat variant.
- [ ] 5. `make check` (the draw-stream pin and `tests/reproducibility.rs`
      run inside it).
- [ ] 6. At close: run the declared gate profile and the slow tier.

## Verification

- [x] Invariant 1: `grep -rnE "topology_operator_key|vm_operator_key|input_ref_operator_key|MutationOperator::all\(\)|GraphEventReport|InputRefEventReport|OperatorKind" crates`
      is empty, and `grep -n "fn operator" crates/v3-core/src/mutation/graph/mod.rs`
      is empty (`ProductionEvent::operator` in `recruitment_paths` stays
      until T30.F09).
- [x] Invariant 2: `cargo test -p v3-core mutation::types` passes with the
      catalog test; the `as_key` strings of every operator equal the list
      on `ac3747005` (`types/mod.rs:319-387`), by a diff recorded in the
      readings.
- [x] Invariant 5: `grep -n too_many_lines crates/v3-core/src/mutation/engine/mod.rs`
      is empty; `cargo test -p v3-core draw_stream` passes with hash
      `14_628_003_425_138_151_748` unchanged.
- [ ] Invariants 6 and 7: `make check` exits 0, no test deleted or ignored.
- [ ] Gate profile: summary at `docs/progress/features/t30-f10.json`, blocks
      equal to the epoch, raw hash and size in the readings, series entry
      added, no full report staged.
- [ ] Slow tier: `make rust-test-slow` -> exit 0 on the final code.

## Performance and Goal Impact

**Predeclaration — written before the run.** No cost-bearing change: the
one selection function does the same list build, weighted draws and gated
apply the four blocks do, with one extra `match` on the nested variant per
attempt and no new allocation or clone. Expected compute cost: none
measurable. The gate is compared against the current epoch with the
standard thresholds and no re-pin; every deterministic block is expected
identical, a differing block is a defect, and a compute regression past
the threshold goes to the spec owner, not to acceptance. No indicator can
move; the goal profile is not run (Verification Plan).

**Measured verdict.** Not run yet.

- Summary of the declared profile: [gate](../../progress/features/t30-f10.json).
- Full readings: [`docs/progress/readings/t30-f10.md`](../../progress/readings/t30-f10.md).

## Success Criteria

- [ ] Every operator is defined in one place: its domain enum; the nested
      `MutationOperator` derives domain, key, weight and complexity effect,
      and no mapping function or parallel list remains.
- [ ] One engine function selects and dispatches every domain's events,
      returning one report shape, and the `too_many_lines` allowance is gone.
- [ ] The draw-stream hash, `tests/reproducibility.rs` and the gate's
      deterministic blocks are identical to main's; `make check` and the
      slow tier pass.

## Notes for AI Agents

- Decision: `MutationOperator`'s serde form is its `as_key()` string; the
  derived variant-name spelling in the lab summary and the recruitment-paths
  block is gone, and no stored benchmark key changes.
- Deferred: `tracked_before` and the added-node diff stay under
  `WORKAROUND(T30.F11)` in the one path; T30.F11 pays it.
- Deferred: the test-only `TopologyOperator::random` and `VmOperator::random`
  samplers stay under `WORKAROUND(T25.F08)`; T30.F05 pays it (user,
  2026-10-27).
- Deferred: `ProductionEvent` in `recruitment_paths/qualification.rs` stays
  as a second nested operator enum under `WORKAROUND(T30.F09)`; T30.F09
  deletes the module.
