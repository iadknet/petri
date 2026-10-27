# T25.F11 — Sensor Wiring as Orthogonal Mutations

**Status**: In Progress
**Last updated**: 2026-10-27
**Feature**: T25.F11
**Track**: [T25 — Edge Groups and Per-Edge Learning](../../roadmaps/t25-edge-groups-and-learning.md)

## Goal

One developmental program wires a sensory map however the gene that calls it
was reached, and a new synapse may transmit from the start or wait silent
until activity unsilences it. After this feature a sensor added to a Graph
node arrives whole: one **bundle** per field set, each a gene of one genome
unit, built by one birth policy and one graph transaction whatever operator
reached it. A bundle can wrap a sensor already declared. Live and silent are
separate operators (`AddSensorLive` / `AddSensorSilent`, `AddBundleLive` /
`AddBundleSilent`), not a draw inside one. Every bundle is built, composed,
learned, decayed, inherited, detached, retargeted and removed by one member
law; `Ring` and `DirectionPair` differ from `Plain` only in geometry-aware
wiring. The three routes of today (`InputRefAdd`, `RecruitNeutralInput`, the
four `Add*Group` operators) and the `neutral_input_recruitment` config are
deleted.

Natural analogs: retinotectal map formation, one program wiring a whole map
(T25.F03, F04); silent synapses beside transmitting ones (Isaac, Nicoll and
Malenka 1995; Liao, Hessler and Malinow 1995). It reaches creatures through
the genome: what a mutation can build, and at what cost, is what selection
can find.

## Non-Goals

- The learning representation (per-member `PlasticityConfig` and `Scope`,
  bundle `inherit_scale` and `retention`) is unchanged and applied the same
  way to every wiring; T25.F12 replaces it.
- The birth transaction, inherited payload and body (T29); birth seeds stay
  in the container (G4).
- A new sensor beyond `UpstreamBus`; a change to any sensor's fields other
  than `ActionQueue`'s window (invariant 1); an `inherit_scale` edit
  operator; wiring on VM nodes; a cap on `max_actions_per_turn`.
- `sensor_read` beside `sensor_wired` in the bench census and the by-family
  pair read (T27.F02's goal-long); any goal-long run (T25's allowance is
  spent).

## Inputs and Invariants

| Input | Where | What F11 takes from it |
| --- | --- | --- |
| Track row and Notes (2026-10-22) | `docs/roadmaps/t25-edge-groups-and-learning.md` | The goal; debts G1, G13, G14 and G2's vocabulary; the node's declaration edits; unused declarations and soft defaults stay; the acceptance guard (every relevant operator applies at least once on applicable fixtures; whole-sensor wiring is a creation postcondition); "viability first, gate + goal; slow tests run; no goal-long". |
| Design note | `docs/strategy/group-creation-unification-2026-10-22.md` | The user's requirements verbatim (Decision section), the model (objects, wirings, birth law, operators, mutations), ownership and seams, the three commits, the open items for this spec. |
| T25.F10 | its spec, Goal and invariants 1–5 | One unit per record, members free; fact members, which F11 folds into the one member law; founder 80 units and rate `0.0060625` unchanged. |
| T25.F08, F07 | their specs | Creation forms (`Uniform`, `Randomized`), `draw_learning`'s draw order, the `inherit_scale` coin, the pattern never randomized; detach's composed weight and seed (F07 invariant 6). |
| Routes today | `mutation/input_ref/mod.rs:303-345`, `mutation/graph/recruitment.rs`, `mutation/graph/group.rs`, `mutation/graph/mod.rs:116-266` (weights 79, `enabled`, `random`), `mutation/sampling.rs:56-90` (`INPUT_BUCKETS`, 30) | What is deleted and what the draws re-pin. |
| Container today | `genome/cgp/container.rs` (`can_host_group:1108`, `can_host_facts:1120`, `add_group:1206`, `add_pair_group:1236`, `relocate_group:1319`, `detach_at:1355`, `pattern_at:698`, `check_local_invariants:1515`, `validate_graph:431`, `birth_coordinates:1155`), `genome/cgp/group.rs` (`Group`, `Shape`, `MemberSite`, `pattern`, `compose`), `genome/cgp/edge.rs` (`Membership { group, scope }`) | The record and member model the bundle replaces. |
| Descriptor sources | `contracts/inputs.rs` (`DirectionPair`, `direction_pairs`, `slot_pairs`, `is_ring`, `compound_width`; `nearby_idx`), `mutation/compound.rs` (non-world widths), `genome/vote.rs`, `genome/mod.rs:156-190` (`OutcomeChannel`), `runtime/types.rs:9` (`OUTPUT_SLOT_COUNT`), `contracts::SHARED_MEMORY_SLOTS` | The fields, field sets and geometry the descriptor declares; the vocabulary that moves. |
| Readers | `v3-cli/src/bench/pair_groups.rs`, `v3-cli/src/lib.rs:205-230`, `v3-cli/src/bench/artifacts.rs` keep-list, `simulation/actions/reproduction.rs:125-150`, `simulation/stats.rs:342-352`, `v3-telemetry/src/metrics.rs:552`, `scripts/telemetry-dashboards-check:131-133`, `neighborhood/mutation_effects/recruitment_reading.rs`, `neighborhood/recruitment_paths/qualification.rs:651`, `v3-server/src/transport/brain.rs:96-320`, `frontend/src/components/inspector/EdgeGroupList.tsx` | Who moves by rename, who is replaced, who is retired. |
| Config and fixtures | `config/simulation.rs:939-959`, `experiments/worlds/blobs-and-fruit-substrate.json:45`, `frontend/src/components/config-panel/runtime/MutationSection.tsx`, `frontend/src/types/generated/NeutralInputRecruitment.ts`, `docs/reference/v3-runtime-config-spec.md:99` | Every consumer of the deleted config. |
| Pins | `tests/viability.rs` founder-only trajectory digests; every mutation-on pin (`tests/baseline_worlds.rs`, `tests/applied_trajectory.rs`, `mutation/**/tests`, `runtime/cgp/neutral_connection_tests`, v3-cli deterministic fixtures, `crates/v3-cli/tests/bench_artifacts.rs`, the slow-tier recruitment-paths readings); `docs/progress/benchmark-series.json` (gate and goal epochs at T27.F03) | The predeclared re-pins (invariant 10). |
| Reference specs | `docs/reference/v3-mutation-spec.md` (23 lines name a retired operator), `v3-graph-backend-spec.md:19`, `v3-genome-spec.md`, `v3-runtime-config-spec.md:99` | Every sentence that states a route, a group or the config. |

Vocabulary. A **sensor** is a declared input reference. The **descriptor**
(`contracts::inputs`) gives each sensor its width and its **field sets**,
each with its fields and geometry. A **bundle** is one gene: a record
`{ source, wiring, scale, inherit_scale, retention }` whose **members** are
sink edges carrying `Membership { bundle, scope }`. A member's **coordinate**
is `(field, site)`, site one of `Sink` (the one sink of a `Plain` wiring),
`Direction(d)` or `Drive`. A **plan** is one admissible `(source, wiring)` on
a node. A **kind** is one directed action's nine vote sinks.

| # | Invariant |
| --- | --- |
| 1 | **Descriptor.** `SensorDescriptor::of(&InputReference)` in `contracts::inputs` owns every sensor's width and field sets; `mutation/compound.rs` is deleted. Field sets: `NearbyCreature` four (slot `k`: its 10 fields, geometry `Pair { x: rel_x, y: rel_y, facts: the other 8 }`); each area summary two (pair `k`: `x`, `y` and the family's three facts, so the facts belong to both sets); each neighbor ring one (8 fields, geometry `Ring`); every other sensor one (all its fields, no geometry), a width-1 sensor included. `InputReference::UpstreamBus` is new: width `OUTPUT_SLOT_COUNT` (24), `resolve_input` reads `upstream_slots[sub_idx]` (0 past the end), class `Upstream`, one sensor-census key; the sampler's last `Upstream` bucket becomes `Fixed(UpstreamBus)` (30 buckets, 7 single-slot draws + 1 bus). The two shared-memory banks are field sets without a sensor: `SHARED_MEMORY_SLOTS` (16) fields each, no geometry, `BundleSource::Memory { previous }`; a member reads `GraphSource::SharedMemory { slot, previous }`. Non-world widths move with today's values (`ActionVotes` 39, `PreviousPassVotes` 39, `CommitCounts` 5, `PreviousOutcome` 4) except `ActionQueue`, whose width follows the configured queue (user, 2026-10-27): `min(3 × max_actions_per_turn, 65 536)` as a `u32` (30 at the default 10; the bound is the `u16` address space of `GraphSource::InputLeaf.sub_idx` and `resolve_input`, `genome/cgp/edge.rs:16`, `runtime/inputs.rs:53`, reached at 21,846 slots), so every queue slot `resolve_input` reads (`runtime/inputs.rs:55-62`) is addressable up to that space and no config cap is added; the Graph draws and raw-field moves use checked arithmetic at the boundary. The descriptor therefore takes a `SensorContext { food_type_count, action_queue_slots }` (`contracts`), built once per run by `SimulationConfig::sensor_context(food_type_count)` (`config`; `action_queue_slots = runtime.max_actions_per_turn` after normalization) and passed where a bare `food_type_count` is passed today: the engine's entry (`apply_mutations_with_food_type_count` becomes `apply_mutations_in(…, &SensorContext)`), every mutator signature carrying `food_type_count`, `input_ref_kind` (swap kinds), the sampler's food-typed draw, `random_graph_source` and the raw-field moves (`graph/operators.rs:166-181,705-726`), the birth policy's field sets, and the readers that enumerate or bound sub-values (`neighborhood/census.rs:50`, `input_use/catalog.rs:138-144`, where an `ActionQueue` channel at or past the width is `OutOfWidth` like a decision compound's). VM `ReadInput` draws (`vm/operators.rs:221,375-380`), `resolve_input`, perception's world widths, the sensor census keys and the inspector's slot-generic queue labels read no width and are unchanged. A live config patch changes the context for later births; a `ReadInput` or edge field at or past the width reads the soft default 0 as today. `genome/vote.rs` moves whole to `contracts::votes`; `OutcomeChannel` and `OUTCOME_CHANNEL_COUNT` to `contracts`; `OUTPUT_SLOT_COUNT` to `contracts` (`cgp::CUSTOM_OUTPUT_COUNT` derives from it). The descriptor never names an action. |
| 2 | **Bundle.** `Bundle { source: BundleSource, wiring: Wiring, scale: f32, inherit_scale: bool, retention: f32 }`; `BundleSource::Sensor { ref_idx, field_set: u8 } \| Memory { previous }`; `Wiring::Plain { sink: VoteSink } \| Ring { kind, offset } \| DirectionPair { kind, offset }`, `offset < 8`. The graph is the aggregate root: it stores every edge; a bundle stores no member list; `BundleId` indexes `bundles`. Genome JSON: `groups` → `bundles`, `Membership.group` → `bundle`, `Shape` → `source` + `wiring`; `GraphEdge` stays 40 bytes. `Group`, `Shape`, `GroupId` and every `group` key, counter, metric, operator key and reader name are renamed to bundle; `MemberSite` becomes `Site { Sink, Direction(u8), Drive }`; `CopyEdgeBundle` becomes `CopyNodeInputs` (it copies a compute node's input edges). |
| 3 | **Wiring.** The wiring owns the legal coordinates, the pattern and the sites it needs: `Plain { sink }`: every field of the set on site `Sink`, pattern 1, needs that one vote sink present; `Ring { kind, offset }`: field `c` on `Direction((c + offset) mod 8)`, pattern 1, needs the kind's eight direction sinks; `DirectionPair { kind, offset }`: `x` and `y` on the twelve non-zero compass directions (`pair_birth_members`), pattern `compass_x` / `compass_y` of `(d − offset) mod 8`; each fact on `Drive`, pattern 1; needs the eight direction sinks and the drive. A member's genomic weight is `compose(own, scale, pattern)` for every member of every wiring; the runtime's learned term, `sink_weight_rows`, the server's composed weight and the detached edge's weight all read the one `pattern(occurrence)`. `pattern_at`'s zero for an off-site member, `Group::pattern`'s zero for a non-`x`/`y` field and the `Ring`-reference check inside `pattern` are deleted: an off-site or wrong-field member is a `MemberShape` violation. |
| 4 | **One member law.** Identical for every member: composition (invariant 3); learning and decay (F06–F09, unchanged); seeds (`insert_member`, `take_members`, `detach_at` carry the entry at the member's site); detach composes the weight and the seed (F07 invariant 6) for every member, so `detach_at`'s fact branch, `is_fact_member`, `can_host_facts` and `MemberSite::Drive`'s special cases go; a source-changing edit detaches first; member removal leaves a hole; the record goes with its last member. Validation (`check_local_invariants`, `validate_graph`): members of one bundle are keyed `(field, site)` unique, every member reads its bundle's source (the sensor's `ref_idx` and a field of its field set; a memory bank's slot `< 16`; for `ActionQueue`, whose width follows the context, any field is legal and one at or past the current width reads the soft default, so validity never depends on the configured queue), sits on a site its wiring maps that field to, in I10 order `(BundleId, field)` after the independent prefix, no member on a compute input, no empty bundle, `offset < 8`, retention in `[0, 1]`; a `Sensor` source must resolve (`ShapeReference`). Relocation (`ShiftBundleOffset`, `RetargetBundle`) moves every member to its new site by coordinate, holes preserved. |
| 5 | **Plans and transactions (graph domain, no RNG).** `plans(def, &FieldSet) -> Vec<Wiring>` depends only on the set's geometry and the sinks present (the descriptor yields a sensor's sets and the two memory banks' sets, `field_set` 0), in deterministic order: a set without geometry yields one `Plain { sink }` per present vote sink in catalog order; `Ring` and `Pair` sets yield `(kind, offset)` for every kind whose required sites are present, kinds in `DirectedVoteKind::ALL` order, offsets `0..8`. A `BundleBirth { field_set: u8, wiring: Wiring, scale: f32 }` is one sampled plan with its birth law. Two owner operations on `NodeGenome`, which holds the declarations the descriptor resolves, apply births: `add_sensor(reference, births, &SensorContext) -> Result<u16, NodeEditError>` declares the sensor and builds one bundle per birth in one edit, and `add_bundle(source, birth, &SensorContext) -> Result<BundleId, NodeEditError>` builds one for a declared sensor's set or a memory bank. Both resolve the `FieldSet` from the node's `input_refs` (or the bank) under the context inside the genome (`NodeGenome` stores no context) and hand the resolved set to the container's primitive, so a caller supplies choices only and never a layout. Each checks everything before writing: the declaration index fits `u16`, `births` covers the sensor's field sets exactly once on a Graph node and is empty on a VM node, every wiring is admissible for its set, and `bundles.len() + births.len() <= u16::MAX` (the count validation admits, `container.rs:1517`); on failure nothing is written and `Err` is returned (the engine's gated restore sees a returned error, never a panic). A bundle is created with every member of its set in canonical order, own weight 0, static, scope `Edge`, no seed, the seed row extended. Duplicates are allowed (F03, F04). `can_host_group`, `add_group`, `add_pair_group` are deleted; `exhaust_group_capacity` is renamed. |
| 6 | **Birth policy (mutation).** One function serves the four creation operators. For each bundle to be born, in field-set order, it draws the plan (`Plain`: `gen_range(0..n)` over the set's admissible sinks in catalog order; geometric: the kind `gen_range` over the hostable kinds in `ALL` order, then `gen_range(0..8)` offset) and the birth law (live: scale `gen_range(-1.0f32..=1.0)`; silent: scale 0, no draw); the owner operation of invariant 5 then applies every birth at once; then, per bundle in the same order, learning as F08 on the materialized members: the form coin `gen_bool(0.5)` (true `Randomized`), `draw_learning`'s mode and scope draws in that form, the `inherit_scale` coin. Every member's own correction is 0 and the pattern is never drawn. |
| 7 | **Operators.** `InputRef.AddSensorLive` / `AddSensorSilent` (weight 1 each; `Add`'s 2 split): `applies_to` any node; draws the sensor (the existing 30-bucket sampler under the `SensorContext`, food-typed where `food_type_count > 1`); on a Graph node the births of invariant 6 for every field set, then `add_sensor`; on a VM node `add_sensor` with no births and no further draw. A field set with no admissible plan skips the event with `NoApplicableTarget` before that set's draws (the sensor draw and earlier sets' draws already taken), and an `Err` from `add_sensor` skips it after them, both under the engine's gated restore, so no unwired declaration or partial bundle set survives (never fires under the fixed sink catalog below capacity). `Graph.AddBundleLive` / `AddBundleSilent` (weight 4 each; the four `Add*Group`'s 8 split): `applies_to` a Graph node with at least one admissible `(source, field set)`; one draw `gen_range` over the node's admissible sets in order (declared sensors by `ref_idx`, field sets ascending, then `Memory { previous: false }`, `Memory { previous: true }`), then invariant 6 through `add_bundle`. `RetargetBundle` (was `RetargetGroupKind`): a `Plain` bundle moves to another present vote sink, `gen_range` over the others in catalog order, `NoApplicableTarget` before that draw when no other vote sink is present; a geometric bundle to another kind, `gen_range(0..3)` over `ALL` minus its own, skipping after the draw when the kind lacks a required site. `ShiftBundleOffset` applies to geometric bundles only. `AlterBundleScale`, `RemoveBundle`, `DetachGraphEdge`, `SetScope`, the nine plasticity operators, `EnableRetention` / `DisableRetention` / `MutateRetention` and `EnableBundleRetention` / `DisableBundleRetention` / `MutateBundleRetention` keep their draws and apply to every wiring. `AddGraphEdge` stays the way to add a lone edge. `ComplexityEffect`: the four creation operators `Increasing`; others unchanged. Removed: `InputRef.Add`, `Graph.RecruitNeutralInput`, the four `Graph.Add*Group`, `NeutralInputRecruitment`, `GraphOperator::enabled`, `MutationSummary.recruitment_deltas` and the recruitment reading. `GraphOperator::random` / `TOTAL_WEIGHT` and `InputRefOperator::random` / `TOTAL_WEIGHT` are deleted; tests draw through the engine's sampler (audit R6). Graph total weight 78, InputRef 12. |
| 8 | **Cost and reports.** One unit per bundle (F10): `AddSensor*` on a Graph node Δ = 1 + bundles made (`NearbyCreature` 5, an area summary 3, else 2), on a VM node 1; `AddBundle*` +1; `RemoveBundle` −1; detach rules as F10. The founder has no bundle: 80 units, `0.0060625`, the viability trajectory digests unchanged (pinned). `GraphEventReport` gains `BundleEdits { created: [[u8; 2]; 3] by wiring and creation form, detached: u8, removed: u8 }` filled from the owner operations' return values and the sampled `CreationForm`; `InputRefMutator` reports the same for `AddSensor*`; the engine's `group_tallies` / `group_edits` are deleted (audit R7). `MutationSummary` and `SimStats` carry `bundles_created_total` keyed by operator, wiring (`plain`, `ring`, `pair`) and form (`uniform`, `randomized`), new, so a wiring is attributed to the operator that built it when one event builds several; `groups_removed_total` → `bundles_removed_total`; `graph_edges_detached_total` unchanged; `pair_group_inherited_nonzero_births_total` and `inherits_nonzero_pair_group` are retired (G14). `birth_edge_class` keeps its one-event rule over `AddGraphEdge` and the four creation operators; a creation reports a class only when it made exactly one `Plain` bundle, `BirthEdgeClass::of_sink(sink)`. |
| 9 | **Declaration edits (G13).** `NodeGenome` owns every edit of `input_refs`: `add_sensor` (invariant 5), `prune(ref_idx)` (only an entry no backend consumer addresses; removes and renumbers every higher `ref_idx` in the backend and every bundle source in one edit) and `swap(ref_idx, reference)` (same `InputRefKind`, validated in the genome); `InputRefMutator`'s `Prune`, `Swap` and `RawFieldMutation` (its `UpstreamSlot` and food-type replacements at `input_ref/mod.rs:420-434`, same selection and draws) call them; no mutation code writes `node.input_refs`. An unused declaration stays legal; the parseability gate's soft defaults for an out-of-range read stay. |
| 10 | **Identity and determinism, three commits.** Commit 1 (seams): the descriptor, the vocabulary moves, `plans`, `add_bundle`, `add_sensor` and the birth policy exist and the three routes call them with their exact draw order, count and answers; no trajectory, draw-stream, digest or fixture moves (a moved pin is a defect). Commit 2 (rename, text only): bundle names in types, JSON, operator keys, counters, metrics, wire types and readers; trajectory and draw-stream pins hold; text digests (genome JSON, summary fixtures, `bench_artifacts` digests), `PROTOCOL_VERSION` and the generated frontend types move. Commit 3 (the model): invariants 1–9 and the readings; every Graph and InputRef draw, every mutation-on pin, the slow-tier readings, the reference specs and both epochs (gate, goal-worlds) re-pin with that one cause; commit 1's per-route answers are deleted, none survives as a switch. In every commit the founder-only viability digests do not move. No new per-tick work: a bundle member is composed exactly as a group member is today; a fact member's term is `scale × 1` where it was `scale × 0`. |
| 11 | **Readers.** The `bundles` block (`bundles-v1`) replaces `pair_groups` (`pair-groups-v2`) and `structure_means.groups` / `.members`: per sample, creatures carrying a bundle on a reachable node; per wiring (`plain`, `ring`, `pair`) `bundles`, `genomic_nonzero` (`\|scale\| > 1e-3`), `learned_nonzero` (bundles whose runtime learned scale, read through the runtime's learned-state view the inspector's `learned_scale` uses, is initialized and `\|value\| > 1e-3`; absent, never zero, where no runtime is sampled), `members`, `own_nonzero` (`\|own\| > PATH_WEIGHT_FLOOR`), `unreachable`; `created` by operator, wiring and form from `bundles_created_total`; rows by (wiring, kind or sink). The summary projection's keep-list retains the block's per-wiring aggregates under the existing last-sample-only row policy with a conversion fixture. The plasticity census and `ring_rules` keep their definitions over bundles. `BrainBundle { source, wiring, scale, inherit_scale, retention, learned_scale, coordinates: [{ field, site, present }] }` replaces `BrainGroup`, `GroupShape`, `GroupCoordinate`, `GroupFact`; the inspector's bundle card lists members by coordinate from the generated types. `recruitment_paths` qualifies through `AddSensorLive`. `docs/progress/benchmark-series.json` records `counter_definition_changes` for `pair_groups` → `bundles`, `structure_means` (groups and members dropped) and `groups_removed_total` → `bundles_removed_total`. |

Options weighed, nominal operator shares, the preparatory refactor audit
and the Codex rounds: [the readings file](../../progress/readings/t25-f11.md).

## Architecture

Owners touched: `contracts` (the sensor descriptor, `UpstreamBus`, the vote
catalog, outcome channels and `OUTPUT_SLOT_COUNT` moved in), genome (the
bundle, its wiring, plan enumeration, the `add_bundle` and `add_sensor`
transactions, the node's declaration edits), `mutation` (four creation operators, one birth
policy, the catalog), `simulation` (the witness retired), readers (the
`bundles` census), and the outer crates for the renamed wire, metrics and
summary blocks. Debts paid: G1, G13, G14, and G2's vocabulary half (the five
vocabulary lines leave `scripts/architecture-debts.txt`; the G2 row keeps
routes and jumps for T30.F05). No boundary changes: `docs/architecture.md`
already places descriptors in `contracts`, bundles and wiring in the genome
as aggregate root, and sampling apart from application, so the plan commit
does not edit it; closure deletes the paid rows. Shared rules this feature
changes, each kept under its owner: the member composition law (owner
`genome::cgp::compose`; applied by `container.genomic_weight`,
`runtime/plasticity/learned.rs:324-329`, `v3-server/src/transport/brain.rs:531`
and the detach path) keeps one formula and changes only the pattern a fact
or `Plain` member reads; the sensor width and field-set rule (owner the
descriptor under a `SensorContext` that `config` builds; read by
`input_ref_kind`, the Graph source and raw-field draws and the reader
censuses) replaces `mutation::compound` and the bare `food_type_count`
parameter; the vote catalog (owner
`contracts::votes` after the move; the VM's `AddVote`, the graph sink
catalog, decoding and traces update imports only). Retained debts used as
they stand: G4 (birth seeds ride in the container, `spawn_child` builds the
child) because T29.F01 owns the birth transaction; G3 (learning on edges,
`Scope::Group` branches) because T25.F12 owns learning.

## Telemetry

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `petri.run.bundles_created` (`bundles_created_total`) | counter, attributes `petri.wiring` ∈ {`plain`, `ring`, `pair`} and `petri.form` ∈ {`uniform`, `randomized`}, summed over operators (the operator key stays in the summary) | bundles the four creation operators committed, from `BundleEdits.created` | the mutation config on `run.started` |
| `petri.run.bundles_removed` (was `petri.run.groups_removed`) | counter | records removed by `RemoveBundle`, a last member's removal or detach | as above |
| `petri.run.graph_edges_detached` | counter, unchanged | — | — |
| `petri.run.pair_group_inherited_nonzero_births` | retired with the witness (metric, expected family, dashboards name) | — | — |
| `mutation.neutral_input_recruitment` | recorded configuration, removed from the `run.started` body with the field | — | — |
| `runtime.max_actions_per_turn` | recorded configuration, already in the `run.started` body; it now also governs `ActionQueue`'s sensor width | — | — |

Each new or renamed family gets its row in `metrics/tests.rs` and its
Prometheus name in `scripts/telemetry-dashboards-check` (`keyed_names` for
the counter keyed by wiring and form).

## Verification Plan

**Question.** Every sensor added to a Graph node arrives as whole bundles,
live or silent by one law, and the population under that law carries
bundles of every wiring at a readable live share; a goal world whose
creatures carry no `Plain` bundles, or an applied `AddSensor*` whose
declaration is unwired on a Graph node, shows it wrong.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate + goal` | A production behavior change (the operator catalog, every birth draw, the member law) answered by the 2,000-tick reading of bundle counts by wiring, live share and genome size; no long-run question is asked and T25's goal-long allowance is spent (track Notes). |
| Slow tests (`make rust-test-slow`) | `run` | The mutation operators, the input catalog and the recruitment code change, which move the recruitment-paths readings the tier pins. |

`cargo test -p v3-core --test viability` runs first (the catalog and tick
mechanics change).

## Implementation Tasks

- [x] Commit 1 (seams, no pin moves): the descriptor with `UpstreamBus` and
      the memory-bank field sets and the `SensorContext` (`ActionQueue`
      still 12 here; the width follows the context from commit 3); `genome/vote.rs`, `OutcomeChannel` and
      `OUTPUT_SLOT_COUNT` into `contracts` with the five debt lines deleted;
      `plans`, `add_bundle` and `add_sensor`; the birth policy; the node's
      `prune` / `swap`; `BundleEdits` reported by the operators with
      `group_tallies` deleted; the three routes rewired onto the seams with
      every pin held.
- [x] Commit 2 (rename): `Group` → `Bundle` everywhere named in invariant 2,
      `CopyEdgeBundle` → `CopyNodeInputs`, wire types, `PROTOCOL_VERSION`,
      generated frontend types, metric and counter names, with the
      predeclared text re-pins.
- [ ] Commit 3 (the model): the four creation operators and their weights,
      the deletions of invariant 7, the one member law of invariants 3–4 with
      the fact branches removed, the config and its consumers deleted, the
      witness retired, `bundles-v1`, the server bundle view, the reference
      specs, the series entries, and every re-pin of invariant 10.
- [ ] The `WORKAROUND(T25.F08)` marker on the VM and Topology samplers
      names those two domains only.
- [ ] Tests: the in-domain guard of the track Notes (each creation operator
      applies at least once on applicable fixtures; descriptor-defined fields
      and facts; the live or silent law; one learning-birth policy; whole-set
      wiring as a creation postcondition); property tests over the container's
      invariants and seed alignment generate bundles of all three wirings and
      both sources; the engine's unit-delta test covers each creation
      operator; `birth_edge_class` for a `Plain` bundle and for a geometric
      one; the descriptor's field sets partition each slot family;
      `add_sensor` one record below capacity returns `Err` and writes
      nothing; `ActionQueue`'s width is `3 × max_actions_per_turn` (30 at
      the default, 60 at 20) and every slot below it is addressable.

## Verification

- [ ] `cargo test -p v3-core --test viability` -> founder digests unchanged
      in every commit.
- [ ] `make check` -> 0 on the final commit; `make rust-test-slow` -> re-pinned
      readings recorded in the readings file.
- [ ] Gate and goal summaries at `docs/progress/features/t25-f11.json` and
      `t25-f11-goal.json`; series entries point at them; no full report
      staged; raw hashes and byte counts in the readings file.
- [ ] Goal readings in the readings file: per world, `bundles` by wiring with
      `nonzero` share, `created` by wiring, `mean_genome_size`,
      `mutation_events_attempted_total` per birth, compute per tick.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: map-forming
developmental programs and silent synapses (Goal); it reaches creatures
through the genome's operator catalog and the per-unit cost of what it
builds. Mutation-off trajectories and the founder-only viability digests are
identical in every commit; every mutation-on trajectory moves in commit 3
with one cause.

Compute: no per-tick work is added by construction; the composed weight of a
member is computed as today. What changes is how much a sensor addition
builds: a `Plain` bundle materializes every field of its set (`ActionVotes`
39 members, `ActionQueue` 30 at the default queue, `NearbyCreature` four
bundles of 10) where
`InputRefAdd` pushed one edge, so mean sink edges per creature and the cognition cost per visit
are expected to rise with bundle count. Gate `wall_clock_ms_per_creature_tick`
(T27.F03 epoch) is predeclared within the 25 % flag threshold, expected
within ±10 % as a hypothesis; a `flag` is reported with its mechanism; a
`severe` is a defect unless the goal readings attribute it to edge count,
which the user then accepts or remediation reduces.

Hypotheses for the goal run, descriptive, no threshold: `Plain` bundles
present in every world at tick 2,000 (they arrive at the sensor-addition
rate, 5.6 % of node-internal events, where no `Plain` group could exist
before); the genomic live share (`genomic_nonzero` / `bundles`) between the
silent floor and one, with `AlterBundleScale` moving it, and
`learned_nonzero` at or above it where `Group`-scope learners unsilence
bundles; `mean_genome_size` and
`mutation_events_attempted_total` per birth above T27.F03's where multi-set
sensors are added (five units per `NearbyCreature` arrival against two),
read beside the bloat deferral below. Every per-seed counter moves with the
draw stream with no predeclared direction; a `severe` there is a trajectory
reading that blocks closure until the user accepts it or remediation removes
it. The `pair_groups` → `bundles` and `structure_means` definition changes
are marked in the series, so those columns are not like-for-like.

**Measured verdict.** Pending.

- Summaries: [gate](../../progress/features/t25-f11.json),
  [goal](../../progress/features/t25-f11-goal.json).
- Full readings: [`docs/progress/readings/t25-f11.md`](../../progress/readings/t25-f11.md).

## Success Criteria

- [ ] The catalog carries `AddSensorLive`, `AddSensorSilent`, `AddBundleLive`,
      `AddBundleSilent` and none of the six retired operators or the config;
      each creation operator applies at least once on an applicable fixture.
- [ ] A sensor added to a Graph node is wired whole, one bundle per field
      set, live or silent as the operator says; the one member law holds for
      every wiring and both sources (property tests).
- [ ] G1, G13, G14 and G2's vocabulary lines are deleted from
      `docs/architecture.md` and `scripts/architecture-debts.txt`;
      `make check` passes.
- [ ] Gate and goal summaries stored and read as predeclared; the slow tier
      re-pinned.

## Notes for AI Agents

- Decision: live and silent are separate operators in both domains, never
  a draw inside one (user, 2026-10-22); a single-field sensor is a `Plain`
  bundle of one; a sensor arrives whole, one bundle per field set.
- Decision: weights `AddSensorLive` 1, `AddSensorSilent` 1,
  `AddBundleLive` 4, `AddBundleSilent` 4: each pair splits its
  predecessors' weight evenly; recruitment's 1 is retired with it.
- Decision: the inheritance witness is retired, not moved (readings file,
  options table); the upstream bus is the compound sensor `UpstreamBus`; the
  memory banks are bundle sources without a declaration; creations are read
  by operator, wiring and form from applied reports, and surviving bundles
  carry no birth tag.
- Decision: (user, 2026-10-27, "B") `ActionQueue`'s width follows
  `max_actions_per_turn` (`3 ×` the normalized value), reversing T19.F06's
  constant-width premise (`d0ddcd205`, "a constant 12 whatever
  `max_actions_per_turn`"); the queue size reaches every width reader
  through the `SensorContext`; no config cap is added.
- Decision: a field set with no admissible plan skips after its draws
  under the engine's gated restore; no sensor redraw and no filtered
  universe.
- Deferred: bloat under bundles arriving at the declaration rate is read
  by T27.F02's goal-long (bundle count, members, own corrections, with
  compute and energy), naming every feature closed in between.
- Deferred: `sensor_read` beside `sensor_wired` in the bench census and
  the by-family pair read belong to T27.F02's run (intent audit gaps 3–4).
