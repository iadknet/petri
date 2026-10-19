# T25.F06 — Learning Per Edge

**Status**: In Progress
**Last updated**: 2026-10-18
**Feature**: T25.F06
**Track**: [T25 — Edge Groups and Per-Edge Learning](../../roadmaps/t25-edge-groups-and-learning.md)

## Goal

Synapse-type-specific plasticity with three-factor traces: `PlasticityConfig`
(rule, learning rate, weight clamp, Lamarckian flag, optional reward source
and trace decay) moves from `ComputeNode` to `GraphEdge`, so each edge on
either surface carries whether it is fixed, Hebbian or reward-modulated,
which outcome channel it listens to and how long its trace lasts, and a sink
edge — onto a vote, a drive, an action parameter, a memory write, a router
gate or a custom output — learns like a compute-input edge under today's four
Hebbian laws and the three-factor update. Learned state is built once at
birth from the genome and the parent's seeds, every plastic edge learns from
the pre and post values its own evaluation used, and the nine plasticity
operators pick edges on both surfaces. The laws, the four rules, the four
outcome channels, the clamps and the Lamarckian seed rule are unchanged; a
`ClearSlot` edge never learns, and a group member stays static until T25.F07
gives it a scope.

## Non-Goals

- Scope, `inherit_scale`, learned scales and group-scope credit (T25.F07);
  randomized creation and detach (T25.F08).
- A new rule, outcome channel or plasticity parameter, or any change to the
  update laws, their clamps or their draw ranges.
- A learning cost or a genome unit for a learning attribute (learning is free,
  T25.F02).
- A new dashboard: the existing plasticity panels are redrawn on the new
  families.
- Reading a stored genome whose compute node carries a `plasticity` key: it is
  refused at the load boundary.

## Inputs and Invariants

| Input | Where | What F06 takes from it |
| --- | --- | --- |
| Track row, success criterion 6, Notes | `docs/roadmaps/t25-edge-groups-and-learning.md` | The goal; "learning mode, reward source, trace decay and the Lamarckian flag are per-edge heritable attributes on both surfaces; vote edges can learn; no cost knob or lazy initializer remains"; the 2026-10-13 decisions (learning attributes carry no unit, plasticity cost 0, machinery deleted); gate + goal, viability first, mutation gate and slow tests run; a text-only commit before the behavior commit. |
| Design note | `docs/strategy/edge-groups-and-learning-design-2026-10-13.md` §3 (`GraphEdge`, I1, I11, learned state, clamp rule), §4 "Learning on edges", §7 (boundaries, vocabulary, `graph_weights`), §8 F06 contract, §9 | The contract below; the track row is authoritative where the two differ. |
| Dependency outputs | T25.F01 (`BirthSeeds`, container primitives, `EdgeMut`), T25.F03 (`EdgeOccurrence`, `edge_sites`, the sink weight table, `graph_weights`, I10), T25.F05 (`v3alpha8`, `GENOME_FORMAT` 3) | What the learned state, the seeds and the payload extend. |
| Today's mechanism | `runtime/plasticity/{hebbian,reward,traces}.rs`, `runtime/cgp/{execute,effects,traced}.rs`, `simulation/tick.rs:1262-1324`, `creature/{state,newborn,plasticity_census}.rs`, `simulation/actions/{reproduction.rs:385-410,cgp_reproduction.rs}`, `mutation/graph/{hebbian,mod}.rs`, `creature/genome/{mod.rs:137-196,cgp/*}` | The laws, counters, seed rule and operators that move; the readers table below. |
| Readers | `companions.rs`, `cgp_mesh_annotations.rs`, `recruitment_paths/{mod,records}.rs`, `mutation_effects/records.rs`, `v3-telemetry/src/metrics.rs:1036-1067` and `metrics/tests.rs`, `scripts/telemetry-dashboards-check`, `telemetry/grafana/dashboards/petri-run.json:2552-2700`, `v3-cli/src/bench/{tracking,run,schema,comparison}.rs`, `v3-lab/src/eval.rs:250`, `v3-server/src/http/creature.rs:55-150`, the frontend files in the table | Each reader's post-change expectation. |
| Pins | `tests/viability.rs` founder-only digests (must not move), `tests/baseline_worlds.rs:203`, `tests/applied_trajectory.rs:141-157`, `mutation/engine/recruitment_tests.rs:42-56`, v3-cli trajectory and deterministic fixtures, `docs/progress/benchmark-series.json` (gate and goal-worlds epochs at F05) | The predeclared re-pins. |
| Primary sources | Miconi 2018; Gerstner 2018 (readings file, both read in full) | Per-connection plasticity and per-synapse-type flag rules with a signed global third factor and an area-specific trace time constant: the natural analog. |

Invariants (the container asserts 1–2 at its exits and `validate_graph` at
the load boundary; the proptests in the Verification table assert 8):

| # | Invariant |
| --- | --- |
| 1 | **Placement.** `GraphEdge { source, own_weight, plasticity: Option<PlasticityConfig>, membership }`, `Copy`, private fields with a `plasticity()` getter, `size_of` 36 (const-asserted and tested); JSON key `plasticity`, omitted when `None`, `deny_unknown_fields` kept. `GraphEdge::new` builds a static edge. `ComputeNode { kind, inputs }` and `ComputeNode::new(kind, inputs)`: the node field, `set_node_plasticity` and `node_plasticity_mut` are deleted. `PlasticityConfig`, `RewardModulationConfig`, `HebbianRule` and `OutcomeChannel` keep shape and JSON. `EdgeMut::set_plasticity(Option<PlasticityConfig>)` is the only writer and changes no seed. I1 for plasticity (finite; learning rate and trace decay in `[0, 1]`, weight clamp in `[0.01, 10]`) holds by construction: the enable draws and perturbation laws keep every value in range, `set_plasticity` debug-asserts it, and `validate_graph` checks it with invariant 2's exclusions at the load boundary and in the parseability gate. The laws and the census read the stored parameters directly; today's runtime `clamp` calls on them (`hebbian.rs:98-99`, `reward.rs:46-47`, `traces.rs:31`, `plasticity_census.rs:166`) go. |
| 2 | **Eligibility.** A plastic edge has `plasticity: Some`. Every edge on either surface may be plastic except an edge of a `ClearSlot` sink (its sum is ignored) and, until T25.F07, a group member; `validate_graph` rejects both, so no half-state exists. `CgpGraphBackendDef::plastic_edges()` enumerates plastic edges in canonical order and is the one definition of "carries learning" for `companions.has_plasticity`, the mesh annotations, the recruitment-paths record and the lab's `expressed()`. |
| 3 | **Learned state.** `GraphRuntimeState::for_genome` is the only constructor and builds once a per-backend learning layout: `has_learning`, the plastic-edge slot list over both surfaces in canonical order, a learned own weight per slot, a trace per reward-modulated slot and a visited flag per backend. A graph without a plastic edge allocates no learned state and runs today's loops. `runtime::plasticity::applied_weight(occurrence)` is the only resolver: the learned own weight of a plastic edge, else the genomic weight (`own_weight` on the compute surface, the T25.F03 table on the sink surface). Storage is `pub(crate)`; readers use the module's read-only view, which yields `(occurrence, genomic, learned: Option<f32>, trace: Option<f32>, bound, kind)` per plastic edge in canonical order, `learned` and `trace` `Some` only for an initialized slot and `trace` only for a reward-modulated one. |
| 4 | **Initialized.** `initialized(slot) = backend visited ∨ slot seeded`; the visited flag is set at visit entry, before the pass-cost check, for the creature's life. It gates the Phase 2.5 update, `capture()` and the view's `learned` (`None` when not initialized), which keeps today's "initialized rows only" reading for the census, the server payload and the trajectory hash. |
| 5 | **Samples.** On a learning graph the executor records, for every plastic edge, the pre value its evaluation used — a compute edge's ordered read (lower sources this visit's outputs; self and higher the last committed) and a sink edge's value as the sink sum read it, in sink order, with shared memory as earlier sinks had written it — and the post value: the node's committed output; the sink's `sanitize_f32(wsum)`. No source is re-resolved after evaluation; the three `action_queue` clones go. Sample and weight buffers are runtime-state scratch; the effects pass allocates nothing per visit (its weighted-input buffer is scratch; the sink trace list is built only when the tracer records). |
| 6 | **Order and laws.** Evaluation reads the applied weight in force at visit start. After effects, each pure-Hebbian plastic edge's Δ is computed from visit-start values and its samples with its own rule and rate (Classic `η·pre·post`; Oja `η·post·(pre − w·post)`; AntiHebb `−η·pre·post`; Covariance `η·(pre − 0.5)·(post − 0.5)`), then contributions apply in canonical order, each bounded by its edge's clamp. Each reward-modulated plastic edge adds its rule's activity (no η) to its trace per committed visit; traces decay once per world tick in `begin_tick` over the modulated slot list; Phase 2.5 applies `w ← clamp(w + η·signal[source]·trace, ±clamp)` to every initialized modulated edge and counts each; the zero-energy floor stays its own step and reaches exactly the creatures with an outcome bank and a reward-modulated edge. A failed visit changes no learned value or trace. On a recurrent compute edge the Hebbian pre value is the committed value the evaluation read, where today's replay reads this visit's output: predeclared, pinned by the self-edge regression. |
| 7 | **Clamp rule.** Every learning write bounds the stored own weight by its edge's `weight_clamp` (in `[0.01, 10]` by I1); the applied weight is never clamped; at-clamp counts use that quantity and bound; a seed is clamped by the child's edge clamp at construction (the one intended change to today's seeding). |
| 8 | **Seeds.** `LearnedState::capture()` returns the parent's seeds before mutation: `None` for a backend with no initialized plastic edge (the fast path; `BirthSeeds` has the one form `Rows`, absence is `None`, and a blank backend a topology operator creates carries none), else `BirthSeeds::Rows { compute, sinks }`, one row per compute node and one per sink, each sized to its inputs (I11 extended), a value per initialized plastic edge. The container keeps sink rows aligned as it keeps compute rows: `push_edge` inserts `None` at the I10 position, `remove_edge` removes, `relocate_group` moves a member's row slot with it (always `None` in F06, since a member is never plastic), `split_edge` keeps the consumer's seed and gives the identity edge none, `set_weight` and a source-changing `set_source` clear, a same-source retarget and `set_plasticity` keep. `from_seeds` reads the child's final flags: a Lamarckian plastic edge with a seed starts at the clamped seed and that slot alone counts as initialized (invariant 4); every other plastic edge, a Darwinian one with a seed included, starts at its genomic weight and stays uninitialized until its backend's first visit; traces start at zero. A backend with one seeded Lamarckian edge and one unseeded or Darwinian edge therefore exposes, updates and captures exactly the seeded slot before its first visit, on either surface. Reproduction writes no runtime field. |
| 9 | **Operators.** The nine keys, weights (`EnableHebbian` 1, `DisableHebbian` 1, `MutateHebbianRule` 2, `MutateHebbianRate` 4, `ToggleHebbianLamarckian` 2, `EnableRewardModulation` 1, `DisableRewardModulation` 1, `MutateRewardSource` 2, `MutateTraceDecay` 4), catalog positions (29 / 58 / 62) and complexity effects are unchanged. Each picks one edge uniformly among the edges matching its predicate in `edge_sites` order over both surfaces (count-then-select, one draw) and applies today's value law to it: enable draws rule, rate `0.01..0.2`, clamp `1.0..5.0`, Lamarckian coin; enable-modulation draws channel and decay `0.5..0.99`; rule and source draw among the others; rate and decay step by today's perturbation. Predicates: enable — `None` and eligible (invariant 2); the four plastic operators — `Some`; enable-modulation — `Some` without modulation; the three modulated operators — with modulation. `applies_to` is "any matching edge". A `ClearSlot` edge or a member never matches. |
| 10 | **Structure.** Plasticity travels with the edge: retarget, split (the consumer keeps it, the identity edge is static), `CopyInternalNode`, `CopySubgraph` and `CopyEdgeBundle` copy it with the edge; `AddGraphEdge`, recruitment and group creation push static edges; `RemoveGraphEdge` removes it with the edge. |
| 11 | **Accounting.** `genome_size`, functional complexity and the founder (80 units, no plastic edge) are unchanged; a learning attribute carries no unit. |
| 12 | **Counters.** `plasticity_updates` and `plasticity_changes` (the work counters, the `SimStats` totals and the Hebbian / reward-modulated split) count one per plastic-edge assignment on both surfaces: per committed visit for a pure-Hebbian edge, per initialized modulated edge per tick for a reward-modulated one; a change is a stored `f32` that differs. Two definition changes, recorded as one `counter_definition_changes` entry in `benchmark-series.json`: sink edges count, and a seeded slot that is reward-modulated counts from birth (zero-delta assignments until its backend's first visit), where today a seeded child counts none until its traces exist (`plasticity/tests.rs:532-552`, whose count expectation moves with it); an unseeded slot of the same backend counts nothing before that visit. |
| 13 | **Identity and determinism.** Commit 1 (text only) adds the edge field: `Debug` prints `plasticity: None` on every edge, JSON omits it, no behavior moves; the pins that hash the genome's `Debug` text move (`tests/baseline_worlds.rs:203` and any other `{:?}` digest) with "the hash without genome text is unchanged" as the check, and JSON-keyed pins (`tests/applied_trajectory.rs`, the recruitment supply pin, the v3-cli fixtures) do not. Commit 2 moves the mechanism: `ComputeNode` loses its field (`Debug` text moves again; the JSON of a genome without plasticity is unchanged, since both keys omit `None`), `GENOME_FORMAT` becomes 4, `PROTOCOL_VERSION` `v3alpha9` with a §2 history line; every plasticity draw changes (one edge among edges, not one node among nodes), so every mutation-on pin moves. With `mutation.per_unit_rate` 0 the founder-based fixtures (`tests/viability.rs`, `tests/applied_trajectory.rs` at rate 0, `tests/baseline_worlds.rs`), whose genomes carry no plastic edge, are byte-identical across both commits apart from genome `Debug` text and the learned-state bytes, and the founder-only viability digests, which hash neither genome text nor learned state, do not move; a fixture that constructs a plastic genome is not covered by that promise (invariants 6 and 7 change its learning). `tests/applied_trajectory.rs` hashes the view's learned values and traces in canonical order in place of the deleted storage, which serialized one empty row per compute node for every reproduced child where the view serializes nothing for a creature without learning (invariant 3); so its rate-0 digest with learned-state bytes re-pins once at commit 2 with that representation change as its mechanism, and its restoring check is the rate-0 digest without learned-state bytes, which is byte-identical across both commits. |
| 14 | **Transport.** `graph_weights` lists every member and every plastic edge: `{surface, container, edge, genomic, applied?, learned?}`, `applied` the weight in force, `learned` present only when initialized; computed on every poll (the inspector's incremental fetch excludes `genome` only) and omitted when the creature has neither. The sink trace carries `weighted_inputs` as the node trace does. `types/genome.ts` moves `plasticity` to the edge, and the node inspector's edge rows show a per-edge plasticity badge (rule, pathway, inheritance). |

Readers, with their post-change expectations:

| Reader | After F06 |
| --- | --- |
| `companions.has_plasticity`, cognitive class `plasticity`, `has_plasticity` census | any plastic edge on either surface of a reachable node |
| `cgp_mesh_annotations.has_stateful_behavior` | true when a live compute node's inputs or any wired sink carry a plastic edge |
| recruitment-paths `stateful` classification | per node, a plastic edge on either surface; `VERSION` `recruitment-paths-v3` |
| mutation-effects state records | learned values and traces from the view in canonical order; `VERSION` `mutation-effects-v3` |
| `plasticity_census` | through the view: `creatures`, `carriers`, and per (pathway, surface) `edges`, `drift_sum`, `at_clamp`; `edges` per rule; `edges` per inheritance |
| v3-telemetry | the families in Telemetry; `metrics/tests.rs` expected families; `scripts/telemetry-dashboards-check` `always_names`; the three `petri-run.json` panels on the split families (`:2612` drift per edge by surface, `:2671` at-clamp share by surface, `:2732` edges on `plastic_edges_by_rule`); the counter and carrier panels keep their expressions, and their descriptions say "plastic edge on either surface" where they said "plastic compute node" (invariant 12 redefines what they count) |
| bench goal summary | `PersistenceSample.plasticity_census` block beside `sensor_census`, the census rows above per retained sample; the closure readings are taken from it |
| lab | `expressed()` masks every plastic edge on both surfaces; `GENOME_FORMAT` 4 refuses format-3 files as format 2 is refused today |
| server | invariant 14; `v3alpha9` |
| frontend | `types/genome.ts`, `types/creature-detail.ts` (`applied?`, `learned?`), `types/trace.ts` (sink `weighted_inputs`), `graphFullModel.ts` (a member at `genomic`, a plastic edge at `applied`), the badge, `useCreatureDetailResource.ts` (incremental exclude `genome`), `creatureInspector.ts` (`graphWeights` replaced per poll), fixtures and tests |
| reference specs | graph-backend §2, §8 (visited flag, initialized), §9 (sink samples, `weighted_inputs`), §13 rewritten per edge; genome §5; reproduction §4 "Graph state"; mutation plasticity rows; mesh-execution, tick-orchestration, evolution-observability (`plasticity_updates`), cli-contract; server §2 and §4.11 |

Options weighed, the primary sources and the preparatory refactor audit:
[the readings file](../../progress/readings/t25-f06.md).

## Telemetry

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `petri.tick.plasticity_carriers` | gauge | living creatures with a plastic edge on either surface of a reachable node (census) | `petri.run_id` |
| `petri.tick.plastic_edges` | gauge | reachable plastic edges by pathway and surface | `petri.plasticity_learning` (`hebbian`, `reward_modulated`), `petri.surface` (`compute`, `sink`) |
| `petri.tick.plastic_edges_by_rule` | gauge | the same edges by rule | `petri.plasticity_rule` |
| `petri.tick.plastic_edges_by_inheritance` | gauge | the same edges by inheritance | `petri.plasticity_inheritance` |
| `petri.tick.plastic_edge_drift`, `petri.tick.plastic_edges_at_clamp` | gauge | sum of `|learned − genomic|` over initialized plastic edges; initialized edges at their clamp | `petri.surface` |
| `petri.run.plasticity_updates`, `plasticity_changes`, `hebbian_updates`, `hebbian_changes`, `reward_modulated_updates`, `reward_modulated_changes` | counter | invariant 12 (names unchanged, both surfaces) | `petri.run_id` |
| Recorded configuration | — | none added: no config value or mechanism switch; the `reward_learning` phase span is unchanged | — |

## Verification Plan

**Question.** F06 claims that plasticity on edges lets a sink edge learn under
today's laws while a graph without a plastic edge pays nothing new; a learned
value that differs from the applied weight, a learned state allocated on a
non-learning graph, or a gate wall-clock `severe` would show it wrong, and
the goal run reports whether sink edges become plastic at all.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate + goal` | A tick-loop, reproduction and operator change whose question (cost held, sink edges learning, no indicator crossing) is answered at 2,000 ticks; the track Notes declare F06 `gate + goal`, and T25's goal-long carriers (F02, F04) are spent. |
| Mutation gate | `run` | New production logic in `v3-core` (learned state, samples, edge operators, seed rows) and `v3-server` (the payload) whose behavior tests must pin. |
| Slow tests (`make rust-test-slow`) | `run` | The nine operators' draws and the recruitment code's seed rows change, which the tier's recruitment-paths rows pin. |

## Implementation Tasks

- [x] Commit 1, text only: `GraphEdge::plasticity` (invariant 1's field,
      getter, serde, `size_of` 36) with no writer; re-pin every genome-text
      hash (Inputs, Pins) with "the hash without genome text is unchanged"
      in each pin's comment; old → new in the readings file.
- [x] Tests first (Verification table), red against commit 1 where the API
      exists, otherwise beside their task.
- [x] Genome: delete the node field and its two setters; `EdgeMut::set_plasticity`;
      `plastic_edges()`; `validate_graph` (invariants 1–2);
      `BirthSeeds::Rows { compute, sinks }` with every primitive aligned
      (invariant 8); the exhaustive destructuring test.
- [x] Runtime: `LearnedState` in `runtime::plasticity` (layout, slots,
      `from_seeds`, `capture`, `applied_weight`, the view, the visited flag);
      `for_genome` the only constructor; callers (`CreatureState::new*`,
      `newborn`, founder, battery, discovery, lab `eval.rs`, tests).
- [x] Executor: `execute_graph_impl` takes the backend's `Learner` from the
      runtime state's layout (`None` for a non-learning backend, whose
      compute loop is today's `collect_cgp_weighted_inputs`) and passes it to
      `apply_cgp_graph_effects`; samples; delete `ensure_*`,
      `effective_weight`, `collect_weighted_inputs_hebbian`, the replays and
      the `action_queue` clones; updates after effects in canonical order;
      scratch buffers in effects; sink `weighted_inputs` through the tracer's
      per-sink `on_sink`, with `on_effects()` kept as the argument-free
      effects-pass event the `input_use` reads tracer consumes.
- [x] Tick: `begin_tick` decay over the modulated slot list; Phase 2.5
      iterates `sim.creatures.iter_mut()` directly (the per-tick id `Vec` and
      its borrow comment go: `sim.stats` is a disjoint field and the config
      read it guarded left with T25.F02) over initialized modulated slots
      keyed on the layout flag; the floor step unchanged; counters per
      invariant 12.
- [x] Reproduction: `capture()` replaces the storage clone and
      `capture_def_seeds`; `newborn` consumes `Rows { compute, sinks }`.
- [x] Operators: `hebbian.rs` on edges (predicates, `select_eligible` over
      `edge_sites`, `applies_to`), draw tails ported.
- [x] Readers: companions, annotations, recruitment-paths v3,
      mutation-effects v3, census through the view, lab `expressed()`.
- [x] Telemetry: the families, `metrics/tests.rs`, the dashboards check, the
      three panels.
- [x] Bench: `PersistenceSample.plasticity_census`, kept in the v1 and v2
      summaries; the `counter_definition_changes` entry below, appended to
      `benchmark-series.json` (an entry in
      that list is spec content under invariant 12, not a baseline: no
      `epoch_baseline` line moves until closure re-pins them).

| Field | Value |
| --- | --- |
| `feature` | `T25.F06` |
| `counter` | `plasticity_updates` (and `plasticity_changes`, `hebbian_updates`, `hebbian_changes`, `reward_modulated_updates`, `reward_modulated_changes`) |
| `before` | one per input edge of a plastic compute node per committed visit (Hebbian) or per initialized modulated compute edge per tick; a seeded child counts none until its traces exist |
| `after` | one per plastic edge on either surface per committed visit (Hebbian) or per initialized modulated edge per tick, a seeded reward-modulated slot from birth |
| `comparison` | sink edges and seeded slots count where they did not, and each enable now lands on one edge where it covered a node's fan-in, so per-creature-tick rates are not like-for-like across F06 |
- [x] Server and frontend: invariant 14, `PROTOCOL_VERSION`, `GENOME_FORMAT`.
- [x] Reference specs (readers table).
- [x] Re-pins listed in the readings file, one cause per commit; delete the
      lines of any proptest whose strategy changed from
      `proptest-regressions/` and commit new files.
- [ ] Verification Plan: `cargo test -p v3-core --test viability` first,
      `make check` once, the mutation gate, the slow tier, gate + goal, the
      epochs re-pinned.

## Verification

- [x] Focused tests: viability first, the touched crates' tests, fmt,
      check, clippy and the frontend tests on the final code -> the readings
      file's Verification results; the table below names what each pins.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output
      path, every survivor resolved.
- [ ] `make rust-test-slow` on the final code.
- [ ] Gate and goal summaries at
      `docs/progress/features/t25-f06-learning-per-edge.json` and
      `-goal.json`, raw hash and byte count in the readings file, series
      entries re-pinned, no full report staged.

| Test | What it pins |
| --- | --- |
| `plasticity/tests.rs:137-661`, ported | every numeric learning expectation unchanged with each node's former config on its compute-input edges, except the seeded child's pre-visit reward count (invariant 12); lazy-empty assertions become `initialized` assertions |
| `hebbian.rs:352-630` tails on edges; one draw-order test per operator over both surfaces | invariant 9 |
| `plasticity_census_tests.rs` through the view (`:121` becomes an `initialized` test) | invariant 3–4 readers |
| `sink_edge_learns_from_the_recorded_sample_not_a_replay` (a `WriteSlot` earlier in sink order changes the memory a later sink edge read) | invariant 5 |
| `a_vote_edge_learns_and_its_vote_follows` (a plastic `ActionVote` edge moves its learned weight after a visit and the next visit's contribution changes) | the goal |
| `a_sink_energy_current_edge_samples_its_evaluation_read` | invariant 5 |
| `a_non_learning_graph_allocates_no_learned_state` | invariant 3 |
| the self-edge regression (recurrent pre value) | invariant 6 |
| `clear_slot_edges_are_never_eligible`, `members_are_not_eligible_until_scope_exists`, `validate_graph` rejects both | invariant 2 |
| model-based proptest of seed rows: uniquely tagged seeds on both surfaces through random operator sequences against a per-surface `Vec<Option<tag>>` model (push, remove, split, relocate, routing's clone, recruitment) | invariant 8 |
| `inherited_weight_above_a_mutated_child_clamp_is_clamped`; a Darwinian edge ignores its seed; a seedless backend is uninitialized; `a_mixed_seed_backend_initializes_only_its_seeded_slot` (one seeded Lamarckian and one unseeded edge per surface: reward count, `capture()` and the view before and after the first visit) | invariants 4, 7–8, 12 |
| `plasticity/tests.rs:512-530` (decay −1 and 2) and `plasticity_census_tests.rs:136-146` (clamp 50) become `validate_graph` rejection tests; `set_plasticity` debug-asserts I1 | invariant 1 |
| the zero-energy floor at cost 0, reached through a reward-modulated sink edge | invariant 6 |
| `size_of::<GraphEdge>() == 36`; the serde round trip with and without `plasticity`; a node-level key refused | invariant 1 |
| `graph_weights` carries `applied` and `learned` for a plastic edge and neither for a static member; the frontend model draws a plastic edge at `applied` | invariant 14 |
| `tests/viability.rs` founder-only digests unmoved across both commits; `applied_trajectory.rs` byte-identical at rate 0 | invariant 13 |

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: plasticity set
by synapse type with a per-synapse eligibility flag whose time constant
differs between circuits and a signed global third factor that converts it
(Gerstner 2018 Eq. 1–2, §4.4; Miconi 2018 Fig. 5). It reaches creatures
through mutation only: the founder has no plastic edge, so every mutation-off
reading is identical, and the nine operators now land on single edges of
either surface.

Compute: a non-learning graph runs today's loops, so the per-tick cost moves
only through the edge width (16 → 36 bytes on both surfaces) and, on learning
graphs, through sample recording (4 B per plastic edge per visit) against the
removed replays and clones. Gate `wall_clock_ms_per_creature_tick` (F05
epoch 0.00265 ms) is predeclared within the 25% flag threshold, expected
within ±5%; a `flag` is reported with its mechanism and the user decides
whether the width stays or option B (the config side table in the readings
file) replaces it; `severe` is a defect. `reward_learning_ms` and the Phase 0
decay are expected at or below today's (slot lists in place of genome
walks). A dhat heap comparison on the named harness
`cargo run -p v3-core --bin profile_ticks --features dhat-heap -- --ticks 500`
before and after commit 2 is recorded in the readings file; total bytes and
blocks are expected not to rise.

Counters: `plasticity_updates` per creature-tick (gate 0.0164, goal 0.264)
has no predeclared direction: sink edges become eligible and each enable now
lands on one edge where it covered a node's whole fan-in; the definition
change is recorded (invariant 12) and a `severe` on it is a reading, not a
cost. Every per-seed counter moves with the draw stream, with no direction;
a `severe` elsewhere is a trajectory reading reported to the user. The gate
and goal-worlds epochs are re-pinned in the closing commit (genome text and
draws move by construction).

Goal (2,000 ticks): no predeclared direction for any indicator; the
`plasticity` cognitive class and `has_plasticity` widen to sink edges.
Readings from the goal summary's `plasticity_census` block at the last
retained sample per world: plastic edges by pathway and surface, by rule and
by inheritance; drift and at-clamp share by surface (the vote-edge runaway
question: a sink at-clamp share far above the compute surface's is the
signal); occurrences of plastic sink edges reported per world, including
zero.

**Measured verdict.** Pending.

- Summaries: [gate](../../progress/features/t25-f06-learning-per-edge.json),
  [goal](../../progress/features/t25-f06-learning-per-edge-goal.json).
- Full readings: [`docs/progress/readings/t25-f06.md`](../../progress/readings/t25-f06.md).

## Success Criteria

- [ ] Learning mode, reward source, trace decay and the Lamarckian flag are
      per-edge heritable attributes on both surfaces; a plastic vote edge
      learns and its vote follows (focused test); no cost knob and no lazy
      initializer remains.
- [ ] A graph without a plastic edge allocates no learned state; the gate
      wall-clock is within the flag threshold or the user has decided the
      width.
- [ ] Every reader derives "carries learning" from `plastic_edges()` and
      learned values from the view; no `pub` learned storage remains.
- [ ] Both commits re-pinned with one cause each; the founder-only viability
      digests unmoved; the declared checks run.

## Notes for AI Agents

- Decision: the eight preparatory refactors in the readings file's audit
  (learned state built once with one resolver; samples in place of replays
  and `action_queue` clones; `pub(crate)` storage behind a view; two-attribute
  telemetry families; one `plastic_edges()` definition; birth layout flags in
  place of genome walks; effects scratch buffers; `capture()` in place of the
  storage clone) are in scope as Implementation Tasks; no workaround is
  accepted and no `WORKAROUND(T25.F06)` marker is written.
- Decision: `GraphEdge` is 36 bytes (design note, 2026-10-13); a gate
  wall-clock flag is reported to the user with option B as the alternative,
  never absorbed silently.
- Decision: the design note's "visits per tick of learning nodes" reading
  is not taken (no counter exists and none is added); `plasticity_updates`
  per creature-tick and the census block carry the learning-activity reading.
- Deferred: the long-run reading (retention of plastic sink edges, vote-edge
  drift over 40,000 ticks) to the next goal-long carrier; T25's carriers
  (F02, F04) are spent.
