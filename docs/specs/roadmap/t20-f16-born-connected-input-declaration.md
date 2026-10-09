# T20.F16 — Born-Connected Input Declaration

**Status**: In Progress
**Last updated**: 2026-10-08
**Feature**: T20.F16
**Track**: [T20 — Input Evolvability and Structured Variation](../../roadmaps/t20-input-evolvability-and-structured-variation.md)

## Goal

A new sensory input is expressed into an existing pathway rather than into
nothing, as a receptor gene duplicated with its parent's expression enters the
parent's cascade. The ordinary `InputRef.Add` event on a Graph node appends the
reference and, in the same event, one ordinary edge from that reference into one
of the node's `ActionVote` sinks, so the declaration is live and exposed to
selection at birth instead of waiting, silent, for a separate connection event.
VM nodes are unchanged. The intended outcome, read by the goal profile in three
worlds, is more executed and causal target-family reads on the selected cohort
with population not under half the previous closure's.

## Non-Goals

- No VM change (T20.F03), no new sensor, family, channel or catalog draw, no
  correspondence between source and sink, no score in any draw.
- No configuration switch, no new operator, no change to operator weights, the
  requested-supply law, event accounting or summary schema; `RecruitNeutralInput`
  (T20.F04) is untouched.
- No destination other than a vote sink of the same node, no compute-node edge,
  no multi-edge projection, no smaller gain than the new-edge law.
- No discovery, usefulness or retention verdict, no floor rebased, no change to
  the F09 to F12 candidate chain, no matched control separating immediate
  connection from the changes coupled to it.

## Inputs and Invariants

Sources of truth: the F16 row and the Notes entries dated 2026-10-09 in the
[T20 roadmap](../../roadmaps/t20-input-evolvability-and-structured-variation.md);
the [born-connected run note](../../strategy/born-connected-run-2026-10.md)
(prototype section, gate B0, row B1, plan forward); `crates/v3-core/src/mutation/input_ref/mod.rs`
(`apply_add`, `prunable_indices`), `crates/v3-core/src/mutation/graph/operators.rs`
(`add_edge`, the new-edge weight law `gen_range(-1.0f32..=1.0)`),
`crates/v3-core/src/creature/genome/vote.rs` (`VOTE_SINK_COUNT` 27,
`VoteSink::from_index`), `crates/v3-core/src/mutation/compound.rs`
(`sub_value_count`); the [mutation spec](../../reference/v3-mutation-spec.md)
Section 3 and the [graph backend spec](../../reference/v3-graph-backend-spec.md);
the [T20.F04](t20-f04-general-neutral-input-recruitment.md) and
[T14.F12](t14-f12-neighborhood-read-of-selected-genomes.md) specs; the previous
closure's goal summary (T03.F12). Reference only, never merged: the kept
branch `worktree-born-connected`, `765e429c` and `a172c734` (`born_connect`,
`born_connected_tests.rs`).

**Evidence and choice.** The realization is the one B1 validated and B0 showed
on the founder; every alternative below is untested and is rejected for that
reason, not on principle. NEAT's add-connection (Stanley and Miikkulainen 2002,
Section 3.1), a random-weight link exposed to selection at once, is the nearest
prior art and decides neither destination class nor gain here.

| Question | Decision | Alternatives considered and why not |
| --- | --- | --- |
| Destination class | One `ActionVote` sink of the same node, drawn uniformly by index over all 27 | Every edge-bearing surface (compute-node inputs, other sink kinds): untested, needs `birth_weights` and plasticity handling on compute inputs, and a vote sink is the pathway that reaches an action, which is what the analog's "existing cascade" means here |
| Weight law | `add_edge`'s new-edge law, one `gen_range(-1.0f32..=1.0)` draw, so the sign is uniform and the edge is an ordinary edge in every respect | A smaller gain: untested, adds a second weight law to the Graph domain; zero weight is T20.F04's `RecruitNeutralInput`, never qualified |
| Switch | None; the behavior is the Graph-node `Add`'s definition. `MutationConfig` schema and digest are unchanged, so every stored `config_digest` holds and `inputs_changed` stays false | The prototype's `mutation.born_connected` flag: opt-in machinery the track's disposal rule discourages, and attribution against the off path is the goal profile's comparison with the previous closure, as for every mechanism; the flagged prototype stays on the kept branch for any later paired control |
| `RecruitNeutralInput` | Kept, untouched, `Off` by default | Retiring it belongs to T20.F09's restart, which compares one-event neutral access as an arm; F16 neither needs nor changes it |
| Missing sink, index overflow | The three draws are taken; the edge is pushed only when the appended index fits `u16` (checked conversion, never `expect`) and the drawn sink exists. Production Graph nodes always carry the fixed sink catalog and a 65,536-entry table is unreachable under the carrying cost, so either case is a test genome, and the event then stays a plain declaration: one applied `InputRefAdd`, one reference, no edge | Skipping or panicking: a skip would make a test genome's catalog or table length change production accounting, and a panic would make a birth lethal to the process; one rule for both cases keeps the draw stream identical whether or not the edge lands |

| Boundary | Required behavior |
| --- | --- |
| Trigger | `InputRefOperator::Add` applied through the engine's ordinary target selection to a node whose `backend_def` is `Graph`. `InputRefOperator::ALL`, every weight, `applies_to`, `complexity_effect` (`Increasing`) and the restricted-size growth classification are unchanged. |
| Reference draw | Unchanged: `random_input_reference` or `random_input_reference_for_food_types`, pushed at index `input_refs.len() - 1`. |
| Connection | Exactly one `GraphEdge { source: GraphSource::InputLeaf { ref_idx, sub_idx }, weight }` appended to `inputs` of `OutputSinkKind::ActionVote(sink)`, where `ref_idx` is the appended index as `u16`. When `u16::try_from(input_refs.len() - 1)` fails (pre-append length 65,536 or more) or the drawn sink is absent, the reference is appended, the three draws are still taken, no edge is pushed, and the event is one applied `InputRefAdd`; both boundary lengths (pre-append 65,535 and 65,536) are tested. |
| Draw order | After the reference draw, three draws in this order: `sub_idx = gen_range(0..width)` with `width = sub_value_count(&new_ref)` (1 for a scalar, 7 for a food summary, 8 for a ring; never 0); `sink = VoteSink::from_index(gen_range(0..VOTE_SINK_COUNT))`; `weight = gen_range(-1.0f32..=1.0)`. Nothing else is drawn; the order is pinned by a test. |
| VM nodes | `Add` on a VM node is the current path draw for draw: one reference appended, no instruction, no extra random number. A mutation sequence that applies no `Add` to a Graph node reproduces its current trajectory; a sequence that converts a VM node to Graph (`SwapNodeBackend`) and then applies `Add` to it takes the Graph path from that event on. |
| Other nodes and surfaces | Every other node, every compute node, every other sink and every existing edge, weight, plasticity entry and `birth_weights` entry is unchanged; vote sinks carry no `birth_weights`, so none is added. |
| Afterwards | The born edge is an ordinary edge: `AlterGraphEdgeWeight`, `RetargetGraphEdge`, `RemoveGraphEdge`, `CopyEdgeBundle`, copies, serialization and the T11.F22 `Prune` and `Swap` rules apply to it as to any other; no provenance is recorded. The connected reference leaves `prunable_indices` until `RemoveGraphEdge` or retargeting removes its last consumer. |
| Accounting | One applied `InputRefAdd` event with the same skip reasons, target reachability, `mutation_supply` counters and neighborhood operator identity as today; the requested-supply law `Binomial(genome_size, per_unit_rate)` is unchanged. Within a birth, later events may differ from the old path's because the draw stream and the edge pools moved; that is the mechanism, not a defect. |
| Taxonomy | `Add` on a Graph node is a connection event, no longer a neutral growth operator; the mutation spec's growth-versus-connection sentence, its InputRef-domain `Add` paragraph, the graph backend spec's "unwired push" clause, the `apply_add` doc comment and the T20.F04 spec's `Separate paths` row say so explicitly, preserving the VM wording. |
| Determinism | Three production RNG draws per Graph-node `Add`, in the pinned order; every pin that covers a mutated Graph-node `Add` moves once and is re-pinned in this feature with the reason in its comment: trajectory digests (`applied_trajectory`, `baseline_worlds` legacy identity, drift-walk counts), the founder mutation hash of `recruitment_off_preserves_pre_feature_genomes_events_and_rng` (recruitment `Off` still adds no draw relative to the current path), the `recruitment_paths` seeded step plans and their accepted seeds (VM plans re-seed only; the three Graph plans that declare a reference are re-expressed on the born edge per the Implementation Task below, as T19.F04 re-expressed them on votes), the `experiment.rs` recorded lineage counts (the T13.F06 progress record stays historical and is not rewritten), and tiny-fixture seeds whose population no longer reaches a sampled horizon, each with the fixture's extinction tick before and after stated in the readings. Founder-only and zero-mutation pins and every config digest are unchanged. |

**Natural analog.** Expression inherited with a receptor duplication (opsin,
olfactory receptors) motivates immediate integration into an existing cascade;
the mechanism approximates it by a fresh random connection, modelling neither
duplication nor pathway inheritance. Variation still discovers the reference,
the sink among 27, the sign and the weight.

## Telemetry

No new signal or configuration (no counter, gauge, span attribute, config
field or switch); a run shows existing signals moved by the changed births.

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `petri.run.mutation_events_applied` (and the other `mutation_*` families) | counter, existing | the same applied `InputRefAdd` events; unchanged wiring | `run.started` body carries the unchanged `mutation` config |
| `cases[].input_use` selected and drift cohort rows (`family_declared`, `family_connected`, `family_executed`, `family_causal`) | goal summary block, existing | T20.F01's funnel on the terminal population | `config_digest` per case, unchanged by this feature |
| `cases[].neighborhood_read`, `drift_depth`, founder `mutational_neighborhood` | goal summary blocks, existing | production births under the changed operator | same |

## Implementation Tasks

- [x] `cargo test -p v3-core --test viability` passes before any edit and after
      every production edit (the feature touches every birth).
- [x] Write the failing tests first, then change `apply_add` in
      `crates/v3-core/src/mutation/input_ref/mod.rs` to the Boundary table:
      Graph node appends one reference and one vote edge (source the appended
      index, `sub_idx` under the width, weight in [−1, 1], every other node,
      surface and edge untouched); VM node byte-identical and draw for draw;
      draw order pinned; a truncated catalog and a pre-append length of 65,536
      stay plain declarations while 65,535 connects; the
      connected reference is not prunable; over many seeds the born edge reaches
      all 27 sinks and all 8 sub-values of a ring; a proptest holds the
      one-reference, at-most-one-edge, source and untouched-node invariants.
- [x] Rewrite `add_input_ref_to_graph_node_wires_nothing` to the new contract
      and `add_input_ref_to_graph_node_leaves_existing_edges_untouched` to
      compare incumbent structure (every pre-existing edge, weight, compute node
      and sink) rather than the whole backend, on a populated fixture with
      existing compute and sink edges of nonzero weight, a plasticity entry and
      `birth_weights`, and a full sink catalog so the appended edge is
      exercised; keep
      `add_input_ref_to_vm_node_leaves_program_unchanged` passing; re-pin the
      trajectory digests the Determinism row names, each with its reason.
- [ ] Re-express the three `recruitment_paths` Graph plans that declare a
      reference (`crates/v3-core/src/neighborhood/recruitment_paths/qualification.rs`)
      on the born edge; VM plans and `graph_copy`/`graph_split` only re-seed.
      `input_ref_added` accepts when the table becomes exactly `expected`,
      `targets`, compute nodes and every sink but `Move(E)` are unchanged, and
      `Move(E)` gained exactly one edge from
      `InputLeaf { ref_idx: expected.len() - 1, sub_idx }` at the task's
      sub-index (cue 0, ring east 2) with weight > 0. The born edge is the
      activation edge, so `east_vote_edge_added` leaves `graph_blank` (3 to 2
      events) and `graph_detour` (2 to 1), and `graph_unprepared` becomes
      `ring_added`, `north_vote_removed`, `activated` (5 to 3; keep
      `cue_edge_retargeted` only if the 3-event path fails `qualified()`, and
      record why). New first seeds via `search_seeds`, `SWAP` 0,
      `MAX_PATH_EVENTS` 6. Negative assertions kept:
      `north_vote_removed_rejects_a_changed_compute_node` on `graph_unprepared`
      and `east_vote_edge_added_rejects_a_zero_weight_edge` replaying
      `graph_copy`; added mirrors reject a born edge at weight 0, into another
      sink, or at another sub-index. Shortened paths, the `Add` step's
      `neutral()` flip and the `recruitment_paths` block are a recorded finding
      in the readings, with this run's headline as the reference because
      T03.F12's raw report is gone.
- [x] Amend the five documents the Taxonomy row names, preserving the VM
      wording; no other reference document changes.
- [ ] `make check` exits 0 in the worktree.
- [ ] Gate and goal profiles by the benchmark specialist, then the mutation gate
      by the mutation specialist, sequentially and alone.

## Verification

- [x] `cargo test -p v3-core --test viability` -> exit 0, before and after
      (28 passed each run).
- [x] Focused tests: `cargo test -p v3-core input_ref` -> exit 0 (54 passed),
      with the test names and the re-pinned digests listed in
      [readings](../../progress/readings/t20-f16.md) ("Re-pinned values" and
      "Build verification"). `cargo test --workspace --no-fail-fast` -> 2,703
      passed, 1 failed: only the user-accepted `decided_passes` severe in
      `gate_profile_has_no_severe_regression_against_series_references`,
      cleared by the gate-epoch re-pin of the benchmark stage.
- [x] Self-review (`simplify` on the crates diff; the Graph `Add` acceptance
      shares `only_east_edge_appended` with `vote_edge_added`), then
      `cargo fmt --all`, `cargo check --workspace --all-targets` -> exit 0;
      viability -> 28 passed; `cargo test -p v3-core --lib recruitment_paths`
      -> 71 passed, 1 ignored; `cargo test -p v3-core --lib input_ref` -> 54
      passed. Pins unchanged.
- [ ] `make check` -> exit 0, recorded in the readings.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path, and
      every survivor resolved as killed, equivalent, or deferred, listed here.
- [x] Benchmark summaries stored at the two paths below, series entries added,
      raw hashes in the readings, goal run once.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: receptor
duplication with inherited expression, reaching creatures through genome
mutation at birth and the existing vote surface and motor machinery. Expected
compute cost: none worth a budget: three RNG draws and one edge push per
Graph-node `Add` at birth (about 2% of founder births apply one), and one more
vote-sink edge per such declaration evaluated under the existing graph visit
charge; wall time per creature-tick flat within the flag threshold; no severe
and no epoch re-pin budgeted for either profile. Costs coupled to the
mechanism, carried by creatures and not by the host: each Graph-node `Add` now
adds to `genome_size()` the edge and, when the sink was empty, the sink
(`genome/mod.rs` counts a wired sink as `1 + edges`), so the event adds two or
three units where the old declaration added one, raising that lineage's
carrying cost, gestation span and size-derived requested supply under the
existing laws, and the connected reference is no longer prunable; the
requested-supply law is unchanged while
the realized exposure (which births change behavior) is what the feature
changes. Gate references: the T24.F02 gate, the gate epoch, and the T03.F12
gate, the previous closure, both verdicts recorded. Goal references: the
T03.F12 goal, which is both the pinned goal-worlds epoch and the previous
closure. The work counters are read
under the usual thresholds (flag +10%, severe +50%); `vm_steps`,
`plasticity_updates`, `decided_passes` and `actions_applied` read severe from
clade composition at T03.F12, T11.F27 and T19.F05, and a feature that changes
which births change behavior can move composition again, so a goal severe is
possible here; it is read against the per-world evidence and, if it is
composition, reported for the user's decision on acceptance and re-pin, not
absorbed and not attributed to compute. Every indicator not in the table is
reported with no direction. The target rows are the selected cohort's
`AreaFoodSummary:0`, `AreaFoodSummary:1` and `NeighborBarrierRing` family rows,
B1's typed rows, read at the goal profile's terminal input-use observation (tick
2,000) per world; `FoodHere:1`, T20.F01's other positive shape, is reported
beside them without a direction.

| Reading | Reference (Orchards 11 / Canyon 22 / Confluence 33) | Predeclaration |
| --- | --- | --- |
| Goal persistence | T03.F12 final 4,158 / 4,042 / 4,441; plateau 3,456.75 / 5,825.58 / 8,139.20; minimum 28 / 1,076 / 200 | **Floor:** no world extinct, and final and plateau each not under half of T03.F12's on every world. A world under the floor is a blocker reported to the user with its checkpoint trajectory; the spec owner neither waives nor rebases it. Canyon is the decisive risk (T03.F12 closed there at 0.441× T24.F02 under an accepted exception; B1's seed-2022 trough reached 10 living). Minimum reported. |
| Target rows, selected cohort: Σ`family_executed` and Σ`family_causal` over the three rows, each on its own | T03.F12: executed 0 / 0 / 0 and causal 0 / 0 / 0 (Orchards `AreaFoodSummary:1` 1/0/0/0, Canyon `NeighborBarrierRing` 2/0/0/0 and `AreaFoodSummary:0` 1/0/0/0, Confluence `NeighborBarrierRing` 1/0/0/0; absent rows are 0) | The row's intended outcome, judged separately: Σ executed above T03.F12's on each world, and Σ causal above T03.F12's on each world, that is at least one executed parent and at least one causal parent among the three rows per world. A world missing either increase is a contrary result reported to the user under the blocker rule with its four counts per row, not absorbed and not rejected by the spec owner; a 20-parent cohort at generation about 47 is a screen in which one lineage can carry a count, and no population-wide claim is made. |
| Connected tracking declared: Σ`family_connected` / Σ`family_declared` over the target rows, selected and drift cohorts | T03.F12 selected 0/1, 0/3, 0/1; drift 7/12 (Orchards: `NeighborBarrierRing` 2/5, `AreaFoodSummary:0` 3/3, `AreaFoodSummary:1` 2/4), 3/5, 7/12 | The mechanism's fingerprint: the ratio at or above T03.F12's on each world in both cohorts, the drift cohort being production births with no selection. A cohort with no target declaration (0/0) reads `Undefined`: reported as such, never a pass, and listed with the target-row result for the user. `family_declared` itself expected up (a connected reference is not prunable), reported without a floor. |
| `neighborhood_read.changed_per_all_births` | Standing floors 0.146400 / 0.130000 / 0.143400 (T14.F12, strict not-below); T03.F12 read 0.1192 / 0.1586 / 0.1398 under accepted exceptions on Orchards and Confluence | Expected **up** on every world (a Graph-node `Add` birth can now change behavior); not below the standing floor on any world; a reading below a floor goes to the user under the blocker rule as at T03.F12. `dead_per_all_births` (0.0096 / 0.0020 / 0.0102) reported with no reliable direction. |
| Drift walk `changed_per_all_births` at depths 0 / 22 / 250 / 1,000 / 2,000; founder neighborhood `InputRefAdd` row | T03.F12 Orchards and Confluence 0.1530 / 0.1225 / 0.0220 / 0.0025 / 0.0015; Canyon 0.1470 / 0.1130 / 0.0175 / 0.0065 / 0.0025; founder `InputRefAdd` silent fraction 1.00 since T11.F03 (raw report) | The walk is a per-world mutation-map regression instrument against the previous closure with no absolute depth floor (T11 track criteria). Expected **up** at every depth on every world; a depth-and-world reading below T03.F12's is a regression this spec did not predeclare, and it goes to the user under the blocker rule with the walk's `dead` and executed-node rows beside it, never absorbed. The founder `InputRefAdd` silent fraction expected below 1.00, reported from the raw report. `dead` rows reported, no direction. |
| Goal `mean_genome_size`, `mean_mesh_nodes` at tick 2,000 | T03.F12 113.08 / 158.14 / 134.36 units; 2.97 / 4.09 / 4.38 nodes | No reliable direction (B1 read one seed each way); tabulated at the 20 checkpoints in the readings. |
| Gate (75 ticks, founders) | T24.F02 gate (epoch) and T03.F12 gate (previous closure) | `deterministic` differs where a founder birth applies a Graph-node `Add`; counters within flag thresholds against both references; no severe budgeted. |
| Observation budgets | Workflow caps (`measurement_evidence.wall_caps`); T03.F12 `recruitment_paths` 230.8 s against 120 s under an accepted exception | Founder neighborhood below 10 s; summed evolved below 180 s; neighborhood read below 10 s; drift walk below 30 s per world; input use below 60 s; `recruitment_paths` read against 120 s with the pre-existing breach noted; goal run below 15 minutes. |

Run sequentially after benchmark-affecting work is final, without competing
builds, tests, servers or measurements:

```sh
make bench PROFILE=gate FEATURE=t20-f16-born-connected-input-declaration
make bench PROFILE=goal FEATURE=t20-f16-born-connected-input-declaration
```

**Readings outside the predeclaration (not edited above; each went to the
user under the blocker rule and is decided in Notes).** Gate `decided_passes`
severe against a zero reference (a born vote edge lets a founder pass decide
inside 75 ticks; `decided_passes_total` 9, all seed 11; the gate report has
no per-operator birth tally, so the founder Graph-`Add` birth count is not
recordable); goal `decided_passes` severe; Confluence's selected cohort
without a target row; Confluence's read floor; Canyon drift depth 22. None is
a compute cost (`vm_steps` −11.8%, `pass_cap_hits` +11.3% flag only). Both
epochs are re-pinned to this feature's summaries in
`docs/progress/benchmark-series.json` (`gate.epoch_baseline` before
`make check`, `goal_worlds.epoch_baseline` after the goal run); the stored
severe comparisons are unedited. Persistence, Orchards and Canyon target rows
(4/4, 3/3 against 0/0), the other floors and every other drift reading meet
the predeclaration.

**Measured verdict ([readings](../../progress/readings/t20-f16.md)).** Both
profiles ran once, CLI exit 3 (severe). Gate: only the accepted
`decided_passes`; epoch re-pinned. Goal: `decided_passes` severe. Persistence
floor met. Contrary: Confluence executed and causal, its read floor, Canyon
drift depth 22.

- Summaries: [gate](../../progress/features/t20-f16-born-connected-input-declaration.json),
  [goal](../../progress/features/t20-f16-born-connected-input-declaration-goal.json).
- Full readings: [`docs/progress/readings/t20-f16.md`](../../progress/readings/t20-f16.md).

## Success Criteria

- [ ] An `InputRef.Add` on a Graph node appends one reference and one vote-sink
      edge per the Boundary table; a VM node is unchanged draw for draw; the
      tests named in Verification pass.
- [ ] The five documents the Taxonomy row names state the Graph-node behavior
      and preserve the VM behavior.
- [ ] Gate and goal profiles stored with the predeclared readings taken; every
      floor miss or contrary result in the table is a recorded user decision,
      never a silent omission.
- [ ] Mutation gate recorded with every survivor resolved.

## Notes for AI Agents

- Decision: (user, 2026-10-09, track Notes) F16 is a general mutation-policy
  feature outside the F09 to F12 candidate chain; T20.F04's separate-path
  contract is superseded for Graph nodes when this feature closes; VM nodes
  stay unchanged until T20.F03.
- Decision: (spec owner, 2026-10-08, from the tested realization) destination
  one uniform `ActionVote` sink, weight the new-edge law, no configuration
  switch, `RecruitNeutralInput` kept untouched.
- Decision: (user, 2026-10-08) the gate `decided_passes` severe (0.000034 per
  creature-tick against 0.000000 in both the T24.F02 epoch and the T03.F12
  previous-closure gate, a zero reference) is accepted as the mechanism's
  behavioral signature, and the gate epoch is re-pinned to this feature's gate
  summary before the mutation gate, as at T19.F04; thresholds and the stored
  severe comparison are unchanged.
- Decision: (user, 2026-10-09) the goal `decided_passes` severe (0.008151 per
  creature-tick against T03.F12's 0.000104, +7,737%; born vote edges make
  more passes decide, not a compute cost) is accepted and the goal-worlds
  epoch is re-pinned to this feature's goal summary, as at T03.F12;
  thresholds and the stored severe comparison are unchanged.
- Exception: (user, 2026-10-09) Confluence's selected cohort holds no target
  row at tick 2,000 (Σ executed 0, Σ causal 0 against 0), the predeclared
  contrary result, accepted as recorded; the row's intended outcome is met on
  Orchards (4/4) and Canyon (3/3) only, and Confluence's drift-cohort
  fingerprint (22/23) holds.
- Exception: (user, 2026-10-09) Confluence `neighborhood_read.changed_per_all_births`
  0.1396 under its 0.1434 standing floor and one birth in 5,000 under
  T03.F12's 0.1398, accepted for this closure; the T14.F12 floors stand.
- Exception: (user, 2026-10-09) Canyon drift-walk depth 22 0.1120 against
  T03.F12's 0.1130 (two births in 2,000), accepted as sampling noise and
  recorded; the walk's regression rule stands.
