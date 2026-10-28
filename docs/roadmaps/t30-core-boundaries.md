# T30 — Core Boundaries

**Status**: In Progress
**Last updated**: 2026-10-27
**Master**: [Program Roadmap](../roadmap.md)

## Goal

The code follows [`docs/architecture.md`](../architecture.md): `v3-core`
dependencies point down with no cycles, each concept has its one owner, and
the frontend reads a brain view the server owns rather than the genome's
internal layout. Every row pays known debts and changes no simulation
behavior.

## Track Success Criteria

- [ ] `v3-core` follows the architecture's dependency order and named prohibitions, and none of the known debts this track's rows name remains.
- [ ] The creature inspector, the sampler, the action log and the paint API read server-owned wire types whose frontend types are generated from, or tested against, the Rust types; no frontend file copies a core type or recomputes a genome rule.
- [ ] Every row leaves trajectories, populations and draw streams unchanged: any pin that moves is a defect.

## Executable Features

- [x] **T30.F01 — Inspector Brain View** — Depends on: None
  - Goal: A person inspecting a creature sees its brain as the simulation applies it, every weight and learner resolved by the core, and the inspector and the paint tools read types the server owns, with no behavior change.
- [x] **T30.F02 — Brain Execution Stands Alone** — Depends on: None
  - Goal: A creature thinks, spends energy and dies exactly as before, its brain's execution owning its own state and drawing on one shared energy accounting, with no behavior change.
- [x] **T30.F03 — Sensors Read Views** — Depends on: None
  - Goal: A creature perceives exactly what it perceived before, from explicit views of the world and of each body, with no behavior change.
- [x] **T30.F04 — Config and World Without Cycles** — Depends on: None
  - Goal: Worlds are configured, painted and stocked with food exactly as before, by layers that build on each other in one direction, with no behavior change.
- [ ] **T30.F05 — Genome Owns Its Analysis** — Depends on: None
  - Goal: Offspring mutate exactly as before, with every random choice drawn by mutation and every structural fact read from the genome's own analysis, with no behavior change.
- [x] **T30.F06 — Actions as Transactions** — Depends on: None
  - Goal: Creatures act exactly as before, each action resolved as one transaction whose single outcome the creature's history, its learning reward and the run's totals all read, with no behavior change.
- [x] **T30.F07 — The Genome Stands on Its Own** — Depends on: None
  - Goal: Offspring inherit and brains run exactly as before, with the genome a domain of its own beside the creature record rather than inside it, with no behavior change.
- [x] **T30.F08 — A Creature's Genome and Its Derived State Agree** — Depends on: None
  - Goal: A creature's genome and everything computed from it at birth are set together by construction and cannot drift apart, so tests build the creature they need instead of swapping genomes and resyncing caches by hand, with no behavior change.
- [ ] **T30.F09 — Retire the Pre-Lab Experiments in Core** — Depends on: None
  - Goal: The two capability experiments placed in `v3-core` before the T22 lab rule, recruitment paths (2026-09-12) and T20.F01's discovery and ecological-opportunity assay (2026-09-25), leave core together with their goal indicator, CLI commands and the recruitment half of the slow tier, and the lab's reachability assays answer their supply, discovery and retention questions, with no simulation behavior change.
- [ ] **T30.F10 — One Operator Catalog** — Depends on: None
  - Goal: Offspring mutate exactly as before, with each mutation operator defined once and selected and dispatched by one engine path for every domain, so adding an operator edits one catalog, with no behavior change.
- [ ] **T30.F11 — Operators Report Their Own Edits** — Depends on: None
  - Goal: Offspring mutate exactly as before, with every Graph edit reporting what it changed and keeping its own container rules, so the engine never infers an edit by comparing genome copies, with no behavior change.
- [ ] **T30.F12 — The Sink Catalog Is Whole** — Depends on: T30.F09
  - Goal: Every Graph node carries the full vote and parameter sink catalog by validation, not by convention, so no edit or reader handles a partial catalog, with no behavior change.

## Notes for AI Agents

- Placed 2026-10-22 at the user's direction from the architecture's debts
  that no row owned; trimmed from ten rows to six the same day after a
  simplicity review (user). The
  [design review](../progress/readings/t30-design-review.md) records the
  challenges. Debts each row pays: F01 G17, G22 (paint); F02 G7, G8, G16;
  F03 G9, G14 (sensor caches), G20; F04 G10, G11; F05 G2 (routes and jumps),
  G12, G15, G23; F06 G18, G19, G22 (action log); F07 G24, G25, G26; F08 G27. Every row is
  behavior-preserving in the T25.F01 shape. On 2026-10-25 an architecture
  review added G22–G25 and F07 (user decisions: F07 moves the genome; paint's
  wire types join F01; F06 stays one row in two halves), and on 2026-10-26
  G26 (user: F07 also moves the census readers).
- F01 projects today's applied semantics, not a future model: a scale whose
  contributors differ in rule, rate, reward source and trace, and
  `DirectionPair` fact members outside the scale, are shown as they run and
  are its preservation cases; T25.F11, T25.F12 and T29.F02 change the view
  with their semantics. The core resolves member pattern, composed weight
  and learner initialization through read APIs, so the frontend's copies
  (`memberPattern`, `learnerInitialized` in `frontend/src/components/inspector/`)
  are deleted, not moved into the server. Frontend types are generated with
  `ts-rs` in `v3-server` (user, 2026-10-25, from the tooling research in the
  [design review](../progress/readings/t30-design-review.md)): a Rust test
  renders the declarations and compares them with committed files under
  `frontend/src/types/generated/`, rewriting them only on an explicit flag;
  64-bit integers render as `number`, and Biome formats the output. Since core
  types do not derive `TS`, a core struct inside a wire type fails to compile;
  each `#[ts(type)]`, `#[ts(as)]` or `#[ts(skip)]` is a review item. The spec
  owner decides whether core's exempt types (`Direction`, `Position`, the id
  types, the config schema) get a feature-gated derive or a server mirror. A
  proof of concept comes first on the internally tagged `ServerMessage`, the
  `[f32; N]` fields serialized with `serialize_with`, and the
  `skip_serializing_if` fields (`#[ts(optional)]`). It covers the sampler route too
  (`v3-server/src/transport/sample_assembler.rs:162`), and gives paint's API
  server-owned wire types in place of `PaintTool` and `PaintStats` (G22).
- F02 moves the energy primitives below runtime, moves runtime's state types
  (`GraphRuntimeState`, `DispatchRecord`) out of the creature record, and
  splits traces so runtime keeps brain-execution records and the simulation
  composes tick records around them. Mutation reads the parent's dispatch
  history through a narrow query, resolved lazily as today
  (`simulation/actions/reproduction.rs:392` already passes it explicitly),
  whose type is a contracts- or mutation-owned view of executed nodes: once
  `DispatchRecord` is runtime's, `mutation::reachability` cannot import it.
  F02 and F06 both change `ledger/mod.rs` (formerly
  `simulation/energy_accounting.rs`) and the action energy flooring in
  `simulation/tick.rs`, and F02 and F03 both change
  `creature/state.rs`; concurrent sessions reconcile those files at
  integration.
- F03 keeps the sensor caches computed once per birth (`163a68b0` added them
  for speed); the declared and wired key sets and the perception
  requirements sensors read from the genome today move to genome analysis,
  which returns contracts keys, or `WorldKeySet` (`sensors::reads`) moves to
  contracts (G9), since the genome imports only contracts.
  F03 and T29.F01 both touch `CreatureState` construction; concurrent
  sessions reconcile that file at integration.
- F04 makes pattern parameter types config data and moves `splitmix64` to
  contracts.
- F05: route resolution and VM jump targets become genome analysis; the
  parseability gate moves into the genome; mutation picks slice anchors and
  calls the existing deterministic slice functions with the same
  `gen_range` call, so draws do not move. The program edits mutation applies
  today, splice and insertion with jump-reference repair
  (`splice_program_with_reference_repair`, also used by
  `mutation::topology::routing`), and the register-count resize with its
  admissible-width predicate and program canonicalization
  (`register_count_moves`, `apply_register_count_mutation`), and the
  dormant-tail copy with its terminal classification, conditional `Halt` and
  copied-instruction provenance (`copy_span_to_dormant_tail`, which commit
  `ba806236` keeps whole so jumps and fall-through never enter the copied
  span), and the route composition that calls that repair through gate
  writes (`add_gate_write`, `apply_add_route_target`,
  `apply_add_feedback_route` in `mutation::topology::routing`) become
  genome-owned edit operations with their semantics unchanged;
  mutation supplies the sampled indices and direction, and all other sampling
  stays in mutation (G23). F05 also deletes the test-only samplers
  `VmOperator::random` and `TopologyOperator::random`
  (`WORKAROUND(T25.F08)`, T25.F11's unpaid accept), so their distribution
  tests draw through the engine's sampler over the enabled catalog (user,
  2026-10-27: "Add the VM/Topology test-only sampler cleanup to T30.F05's
  scope"). When the gate moves into the genome, it carries
  `apply_gated`'s graph-validation and rollback contract
  (`mutation/engine/mod.rs:581-607`; the T25 design note requires the check
  after every event in every domain), and F05 removes only the checks shown
  redundant because an owner edit in `genome/node_edit.rs` already enforces
  them before writing; operators that rely on the restore
  (`add_bundle`/`add_sensor` errors and `RetargetBundle`'s skip mapped
  after their draws) keep their draws in order (user, 2026-10-27, from the
  same audit).
- F06 moves the per-action executors, action energy flooring and the
  removals actions cause from `simulation/tick.rs` into mechanics with
  narrow inputs (no whole `Simulation`), composes action costs there with
  the body supplying factors (today's `CreatureState::move_effort`), and
  returns one applied result per action, a contracts type carrying the
  outcome kind and the attempt effort (the simulation interprets outcomes
  under rule 1 and the creature's log only stores them, so the result is a
  shared contracts value, as `ActionType` already is), from
  which the action log, the reward success count and run totals derive with
  today's classifications. Attempt effort stays charged on a rejected give
  or steal (T27.F03). Action deaths, upkeep deaths and paint eviction reuse
  one removal-settlement path (`simulation/simulation.rs:102`), with no
  observation logic pulled into mechanics. Delayed deliveries and
  world-update removals stay lifecycle records, not action results. The
  action log reaches the frontend through a server-owned wire type built from
  the applied result (G22); if F06 lands before F01's generator, it ships that
  wire type with a parity test the generator later replaces. F06 lands in two
  implementation passes under one spec, the gate run after each with pins
  unchanged and one closure: first the executors with narrow inputs and the
  cost composition, then the one applied result and the records derived from
  it (user, 2026-10-25). The per-tick reproduction RNG derivation in
  `simulation/tick.rs` stays as the tick's design (`docs/architecture.md`
  section 4), not a workaround (user, 2026-10-26). It overlaps T27.F02,
  T28.F01, T29.F01 and T30.F02 in action and reproduction code; whichever
  lands later extends the applied-result path.
- F07 moves `creature/genome/` to a top-level `genome/` with a crate-wide
  import rewrite and no compatibility re-exports, and deletes the import
  checker's special case for the old path (G24). It also moves
  `v3-core/src/bin/profile_ticks.rs` into `v3-cli` with its `dhat-heap`
  feature and `dhat` dependency, where the per-binary telemetry flag then
  applies, and deletes the checker's `bin/` skip (G25, user, 2026-10-25). It
  moves the census readers `plasticity_census`, `sensor_census`, `drives` and
  `ring_rules` from `creature/` into `neighborhood/` and deletes the checker's
  hand list of them (G26, user, 2026-10-26). It touches the import lines
  of every row that edits genome code, so it lands while no such row is in
  flight.
- F08 comes from the T30.F03 design-lens step-back (user, 2026-10-26:
  "encapsulate finalized genomes and their derived state behind
  construction operations, so fixtures stop swapping genomes and resyncing
  caches by hand"). `CreatureState`'s genome and its birth-derived state
  (the `cached_*` fields and the learned runtime built from birth seeds)
  become private behind read accessors, so the record's documented "genome
  is immutable after creation" is enforced rather than stated (G27). Tests
  that swap a genome today (about 50 writes in 13 files across `v3-core`,
  `v3-cli` and `v3-server`, among them `final_action_observation_*` in
  `simulation/tick.rs`, whose swap leaves the perception cache stale) build
  the creature through a construction operation instead, and tests that set
  a cache no genome has (`cached_genome_size = 4` in
  `simulation/tick/tests/{energy_accounting,phase0}.rs`) build a genome
  with that property. No constructor accepts derived state apart from the
  genome it derives from: `new_with_cached_fields` (`creature/state.rs`),
  which takes complexity, size and reachability as independent arguments
  and whose unit test keeps values no genome has, is replaced by an owner
  operation that establishes the correspondence, so the no-mutation
  reproduction fast path may still reuse its parent's analyses, and that
  test is replaced. F08 changes no
  birth path: T29.F01 later replaces the constructors F08 routes through
  with one birth transaction (G4), and T30.F02 moves `GraphRuntimeState`
  out of the record (G8); both edit `creature/state.rs`, and concurrent
  sessions reconcile it at integration.
- F09 was added on 2026-10-27 at the user's direction, paying T25.F11's
  unpaid accept on `neighborhood/recruitment_paths`: asked at T25.F11's
  closure whether to move the experiment to `v3-lab` or retire it, the user
  answered "Propose retiring the recruitment-paths slow tier instead of
  moving it", then "Add the T30.F09 retirement row as drafted". The design
  review then found T20.F01's discovery assay (`neighborhood/opportunity`,
  added `a0939c050` on 2026-09-25, before the lab rule `496245d24` on
  2026-09-28) in core with no payer; the user widened F09: "Widen T30.F09
  to retire the T20.F01 discovery experiment and the input-opportunity
  command too". F09 deletes `neighborhood/recruitment_paths` and
  `crates/v3-core/tests/recruitment_paths.rs`; the goal profile's
  `recruitment_paths` indicator and its wall-clock field (`v3-cli`
  `bench/profiles.rs`, `indicators.rs`, `run.rs`, `schema.rs`,
  `artifacts.rs`); the `recruitment` and `input-opportunity` CLI commands
  (`v3-cli/src/recruitment.rs`, `opportunity.rs`, their `main.rs` arms and
  measurement readings, and their names in `v3-telemetry`'s command list);
  `neighborhood/opportunity`'s assay, discovery, fixtures and verdict; the
  recruitment line of `make rust-test-slow` and the `recruitment_paths`
  invocation in `make rust-test-temporal-fixtures`. The authored
  controllers and `Family` (`opportunity/controllers.rs`), which the lab's
  comparator, calibration and evaluation use, move into `v3-lab`, with one
  shared Ring-wiring operation that both `controller()` and
  `barrier_comparator` (which today repeats the Ring construction by hand)
  call, their reference indices, constants and genomes unchanged. The
  workflow's slow-tests row keeps only the lab half, and
  `docs/progress/benchmark-series.json` records the dropped block under
  `counter_definition_changes`. Past readings stay as history. The goal
  summary's shape changes; simulation trajectories do not
  ([F09 design review](../progress/readings/t30-f09-design-review.md)).
  F09 also deletes `GraphMutator::apply`'s default-`SensorContext` shim
  (`mutation/graph/mod.rs:290-310`), whose only non-test caller is
  `recruitment_paths/qualification.rs:81`.
- F10, F11 and F12 were added on 2026-10-27 at the user's direction from a
  post-closure refactor audit of the areas T25.F11 rebuilt: "Draft rows A,
  B and C in T30, the T30.F05/T29.F01 note lines and the F09 shim note".
  Each is behavior-preserving in the T25.F01 shape: any pin that moves is a
  defect, and a finding that a skip path fires on production genomes (so
  removing it would move draws) is a blocker for the user.
- F10: each operator is defined once. Today every operator is defined in
  its domain enum and again in the flat `MutationOperator`
  (`mutation/types/mod.rs:239-620`), mapped by three free functions
  (`topology_operator_key`, `vm_operator_key`, `input_ref_operator_key` in
  `mutation/engine/mod.rs:688-750`) and a Graph method, and the engine's
  `apply_mutations_on_units` (`mutation/engine/mod.rs:120-370`, carrying
  the area's only `too_many_lines` allowance) repeats selection and
  dispatch once per domain. `MutationOperator` becomes a nested enum
  (`Graph(GraphOperator)`, …) from which domain and key derive, and one
  generic selection and dispatch serves every domain with its draws in
  today's order. T25.F12 adds report fields to this path, so F10 is ordered
  before it.
- F11: every Graph edit reports what it changed. The engine infers the node
  an operator added by diffing whole genome clones (`tracked_before`,
  `operator_requires_added_node_input_tracking`,
  `collect_added_node_input_classes`, `collect_added_node_world_inputs_for_operator`
  in `mutation/engine/mod.rs:198-575`) for TopologyAddNode,
  TopologySpliceNode and GraphAddInternalGraphNode, on top of
  `apply_gated`'s own snapshot; T25.F11 removed the same pattern for
  bundles. Each operator returns the node it added. Also in F11: the five
  wrappers built on `graph_def_mut_with_input_refs`
  (`mutation/graph/operators.rs:36-47`), which clone `input_refs` per event
  to avoid a borrow, use the split borrow `mutation/graph/bundle.rs` already
  has; `EdgeMut::set_source` (`genome/cgp/container.rs:292-303`), which
  relies on three callers detaching a member first, gains one container
  retarget operation that detaches itself, and `split_edge`
  (`genome/cgp/container.rs:1031`), whose caller `split_existing_edge`
  detaches first and which writes the source directly, becomes an
  owner-complete split that detaches and returns its effects, with split's
  weight, seed and draw behavior unchanged; and `insert_member`'s per-member
  position scan, which makes building a wide bundle quadratic, becomes
  linear.
- F12: the sink catalog is validated. `with_output_sinks`
  (`genome/cgp/container.rs:534-541`) accepts any list and neither
  `validate_graph` nor parseability checks it, so `hosts`/`hosts_kind`, the
  present-sink filter in `plans`, `Option`-returning `sink`/`site_sink`,
  `insert_member`'s `expect`, `can_add_edge` and `RetargetBundle`'s two skip
  orders all handle partial catalogs production never builds. Validation
  requires the full catalog, sinks are found by index, and that hosting
  layer is deleted. F12 follows F09, which deletes the only non-test
  builder of a partial catalog (`recruitment_paths/fixtures.rs:535`).
- The master order puts F03 first (user, 2026-10-25), F01 and F07 before
  T25.F11, F08 after F07 (assistant proposal), F02 and F06 before T28, and
  F09 after F06 (assistant proposal), as preferences, not prerequisites.
  F09 is also a dependency of T27.F02 and T28.F01, like T29.F03 and
  T30.F05 (user, 2026-10-27). F10 precedes T25.F12 as a dependency
  (assistant proposal the user accepted with the row); F11 and F12 follow
  F09 (assistant proposal).
- Verification (assistant proposal): each row gate with pins unchanged; F01
  adds the frontend's own tests and the type-parity test.
