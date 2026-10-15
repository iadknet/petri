# T25.F03 — Ring Groups Replace Ring Projections

**Status**: In Progress
**Last updated**: 2026-10-15
**Feature**: T25.F03
**Track**: [T25 — Edge Groups and Per-Edge Learning](../../roadmaps/t25-edge-groups-and-learning.md)

## Goal

One developmental program wires a whole map, as Eph/ephrin gradients wire a
retinotopic sheet and central-complex columns wire a compass: a ring
projection is a `Ring` edge group, one `Group` record `{shape: Ring
{ring_ref}, kind, offset, scale}` plus eight member sink edges, one per
direction sink of the kind, each a real `GraphEdge` with its own correction
(`own_weight`, born 0) and a `membership` naming the group. A member's
genomic weight is `scale + own_weight`; the sink loop reads it from a table
built once at creature construction. Any one member can be re-weighted,
retargeted, split or removed on its own while the group survives; removing
the last member removes the group. `RingProjection`, its evaluator, its list
and its five operators are deleted outright, with no converter, and five
group operators take their catalog positions at the same weights. Every
reader counts members as edges and reads membership from the genome; the
derived wiring analyses keep their own definitions over non-member edges.
The feature lands as a text-only identity commit followed by the behavior
commit, each with its re-pins predeclared below.

## Non-Goals

- The `DirectionPair` shape and pair metadata in `contracts::inputs` (F04);
  the `NearbyCreature` merge (F05); per-edge learning, `Scope`,
  `inherit_scale`, learned scales, sink-edge seeds and the `applied` and
  `learned` payload values (F06, F07); randomized creation, a detach
  operator and the `Slots` refinement disposition (F08).
- `Drive` sinks as group targets, compute-input edges as members, the VM
  backend, founder rewiring, type or module renames (design note D11).
- A migration for stored genome JSON or lab genome files: a file carrying
  `projections` or `weight` fails to load (no backward compatibility).
- Any reader of ring groups the track row does not name, such as a
  per-creature carrier-keyed blocked-move instrument.

## Inputs and Invariants

| Input | Where | What F03 takes from it |
| --- | --- | --- |
| Track row, success criterion 3 and Notes | `docs/roadmaps/t25-edge-groups-and-learning.md` | The goal; eight member edges with one shared scale, each member deletable, re-weightable and learnable on its own; every reader counts members as edges and reads membership from the genome; user decisions of 2026-10-13 (only geometry is built in, group creation never randomizes the pattern, size-neutral replacement after F02's nine-unit charge); verification gate + goal (goal-long dropped by user decision, 2026-10-15), viability first, mutation gate and slow tests run; a text-only commit before the behavior commit. |
| Design note | `docs/strategy/edge-groups-and-learning-design-2026-10-13.md` §3 (domain model, invariants I2–I11, accounting contract), §4 (shape operators, removal, edge operators on members, recruitment, ablation D3, draw contracts, refinement D7), §7 (boundary table, dependency direction, determinism and pins), §8 F03 contract | The types, the container API additions, the operator list and draws, the readers, the serde and payload shapes, the re-pin list, the tests-first list and the readings. |
| F02 outputs | `docs/specs/roadmap/t25-f02-uniform-genome-size.md` invariants 1–3 | A projection costs nine units, so `genome_size` is unchanged by the replacement; anchor 80 and rate `0.0060625` stay; `cached_genome_size == genome_size()` after every edit. |
| Projection today | `creature/genome/cgp/mod.rs:105-160` (`RingProjection`, `channel_for`, `offset_for`, `ablated_channels`), `cgp/container.rs:184-186, :297, :402-412, :588-602` (field, builder, `enters_visit`, `projects_kind`, reindex), `runtime/cgp/effects.rs:48-99, :117-157` (`projection_terms`, the sink term), `mutation/graph/projection.rs`, `mutation/graph/mod.rs:42-46, :74-78, :111-115, :151-170, :235-239, :395-413`, `mutation/types/mod.rs:234-238, :302-306, :369-373, :428-449, :458-516`, `mutation/engine/mod.rs:731-741`, `genome/mod.rs:288-290`, `cgp_analysis.rs:94-101, :135` | Everything the behavior commit deletes or replaces. |
| Container (F01) | `cgp/container.rs`: `EdgeOccurrence`, `edge_sites` (canonical order), `push_edge`, `remove_edge`, `split_edge`, `EdgeMut::{set_source, set_weight}`, `reindex_input_refs_after_removal`, `redirect_leaf_reads`, `BirthSeeds` (compute rows only), the hand-written `Debug`/`PartialEq` | The one writer F03 extends with `groups`, `add_group`, `remove_group`, `relocate_group`, `detach_at`, `members`, `genomic_weight`, `GroupMut::set_scale`, I10 insertion and `validate_graph`. |
| Edge draws | `mutation/graph/operators.rs:219-235` (`pick_edge` over `edge_sites`), `:327-350` (split), `:486-540` (add, remove, retarget, alter), `:749-790` (raw field), `:842-862`; `refinement.rs:14-25, :41-66, :86-130` (`GroupKey::Ring`, `correspondence`, `groups`); `recruitment.rs:31-62` (`Plan::append` ring case), `:83-90` (dangling projection reference) | The draw order every operator keeps and the sites that change on a member. |
| Readers | `creature/ring_rules.rs` (`ring-rules-v1`, paths i–iii, `projection_*` fields), `drives.rs:57-84`, `sensor_census.rs:224-230`, `genome/cgp_mesh_annotations.rs:68-76`, `neighborhood/input_use/consumers.rs:102-145, :238-270`, `neighborhood/opportunity/discovery.rs:123-135`, `neighborhood/steering.rs:101-120`, `mutation/input_ref/mod.rs:101-111`, `crates/v3-lab/src/readings.rs:399-450, :630-695`, `crates/v3-lab/src/ladder/track.rs:55-69`, `crates/v3-lab/tests/live_barrier_use.rs`, `crates/v3-cli/src/lib.rs:172-215` (`structure_means.projections`), `crates/v3-cli/src/bench/ring_rules.rs`, `crates/v3-cli/src/bench/schema.rs:316` (`REACHABLE_STRUCTURE_VERSION`) | Each projection branch and its post-change expectation (table below). |
| Runtime state | `creature/state.rs:76-130` (`GraphRuntimeState`, `new()`), `:323-345, :397-416` (constructors), `neighborhood/battery.rs:334, :432`, `neighborhood/opportunity/discovery.rs:298`, `simulation/tick.rs:407, :475` (observation clones), `crates/v3-lab/src/eval.rs:151`, `runtime/cgp/execute.rs:109, :178` (compute-node learned weights, untouched) | Every site that builds runtime state, migrated to `for_genome`. |
| Transport and inspector | `crates/v3-server/src/http/creature.rs:44-70, :344-385`, `crates/v3-server/src/types.rs:5` (`v3alpha5`), `docs/reference/v3-server-api-protocol-spec.md` §2, §4.11; `frontend/src/types/genome.ts:105-124`, `components/inspector/RingProjectionList.tsx` and its test, `GraphInternalsViz.tsx:174`, `graphFullModel.ts`, `graphFlowAdapter.ts` (`EDGE_TINTS`), `mesh/meshSemantics.ts:100-110, :181-290` (`infer*`), `mesh/meshPresentation.ts:102-140` (`collectNodeBadges`), `NodeInspector.tsx:52`, `hooks/useCreatureDetailResource.ts:32-35`, `petriVizDrift.test.ts`; `.agents/skills/petri-visuals/{spec.md, viz/petri-viz.js, viz/petri-viz.min.js}` and `.agents/skills.lock.json` | The serde shape, the `graph_weights` payload, the edge-group list and the inference deletion. |
| Lab and telemetry | `crates/v3-lab/src/summary.rs:19` (`GENOME_FORMAT`), `crates/v3-lab/src/lib.rs:36-70` (`GenomeFile::load`); `crates/v3-telemetry/src/metrics.rs`, `metrics/tests.rs`, `scripts/telemetry-dashboards-check:109` (`always_names`); `mutation/types/mod.rs:26-41` (`MutationSkipReason`), `mutation/engine/mod.rs:580-598` (`apply_gated`) | The format bump, the two new counters, the gate skip reason. |
| Pins | `tests/viability.rs` founder-only digests (both siblings: must not move), `tests/baseline_worlds.rs:142`, `tests/applied_trajectory.rs` four digests, `mutation/engine/recruitment_tests.rs:49`, `tests/reproducibility.rs`, `neighborhood/mutation_effects/mod.rs:60` (`genome_identity`), the slow-tier recruitment-paths rows, the gate and world-set deterministic blocks, `runtime/cgp/neutral_connection_tests/projection.rs:212`, `mutation/graph/recruitment_tests.rs:223` and its regression file, `docs/progress/benchmark-series.json` | The predeclared re-pins (Performance section). |
| Reference specs | `docs/reference/v3-graph-backend-spec.md` §2, §6 ("Ring projections"), §8 (entered-visit clause), §9, §14, §16; `v3-genome-spec.md` §3, §4; `v3-mutation-spec.md` §3 Graph domain (projection and recruitment rows), §4.2 step 7, §5, §6; `v3-runtime-config-spec.md:185-190`; `v3-server-api-protocol-spec.md` §2, §4.11 | Every sentence that states a projection, rewritten to groups. |

Invariants (the container asserts I2, I3, I5, I6, I10 and `offset < 8` at
every exit; the group module asserts I4, I7 and I8 with `input_refs`;
`validate_graph(&NodeGenome)` checks all of them at `GenomeFile::load` and
in the mutation gate):

| # | Invariant |
| --- | --- |
| 1 | **Types.** `GraphEdge { source, own_weight: f32, membership: Option<Membership> }`, private fields, `Copy`, `size_of` pinned at 16; `Membership { group: GroupId }`; `GroupId(u16)`; `Group { shape: Shape, kind: DirectedVoteKind, offset: u8, scale: f32 }` with private fields written only through `GroupMut::set_scale` and the group module; `Shape::Ring { ring_ref: u16 }`. `GraphEdge::new(source, own_weight)` builds a static, ungrouped edge; members are built only by `add_group`. `compose(own, scale, pattern) = scale * pattern + own` is the only formula; a non-member's genomic weight is its `own_weight` itself. A `Ring` member's pattern is 1, so its genomic weight is `scale + own_weight`, bitwise equal to an ungrouped edge of that weight. The pattern resolves from the group's `ring_ref`, the member's `sub_idx` and its target direction, never from the member's own `ref_idx`, and is 0 when `ring_ref` does not resolve to a ring. |
| 2 | **Membership is a validated coordinate (I2, I4, I5, I7, I8).** Only a sink edge carries a membership. A `Ring` member reads `InputLeaf { ref_idx: ring_ref, sub_idx: c }` with `c < 8` and sits on sink `K[(c + offset) mod 8]`; at most one member per `(group, source, target)`; `input_refs[ring_ref]` is a ring family (`InputReference::is_ring`); `offset < 8`; `ring_ref ≤ u16::MAX`. An operation that changes a member's source outside a shape operator detaches it first: `RetargetGraphEdge` with a changed source, every `GraphRawFieldMutation` step on a member (a raw-field step always changes the source), and `AddInternalGraphNode`'s split. Detach is genomic: `own_weight := scale + own_weight`, `membership := None`, bitwise (`detach_preserves_genomic_weight_bitwise`). Because detach moves the edge to the end of its sink's independent prefix (invariant 4), `detach(def, occurrence, input_refs)` returns the edge's new `EdgeOccurrence` and the calling operator continues its edit on that occurrence (`detaching_a_later_groups_member_ahead_of_another_member_keeps_the_edit_on_the_detached_edge`). `AddGraphEdge` never joins a group. |
| 3 | **Groups and members (I3, I6).** Every `GroupId` is below `groups.len() ≤ u16::MAX`; a group has at least one member; `RemoveGroup`, `RemoveGraphEdge` on a last member and a detach of a last member remove the record and renumber every higher `GroupId` in one container operation; reference reindexing never removes a group (decrement-only, debug-asserted). One predicate, `can_host_ring_group(def)` (all 24 direction sinks present and `groups.len() < u16::MAX`), is `AddRingGroup`'s applicability condition beside a ring reference, filters ring families out of recruitment's family universe in its preflight (invariant 9), and is asserted by `add_group`; no draw is spent on a creation that cannot complete, and in production, where the catalog is the fixed 98, it is false only at capacity. Creation makes eight members at `own_weight` 0 and `scale` 0: a new group is silent, as a projection is today. |
| 4 | **Sink order (I10).** Within each sink, independent edges come first in insertion order, then members sorted by `(GroupId, channel)`. `push_edge` on a sink inserts an independent edge before the sink's first member; `detach_at` moves the detached edge to the end of the independent prefix; `relocate_group` (`ShiftGroupOffset`, `RetargetGroupKind`) removes every member from its sink and re-inserts each at its new sink in this position, so a shift and its inverse return an equal genome. Sink sums are sequential over this order, so it is both identity and arithmetic; the summation order on a sink carrying members changes from today's `edge_sum + projection_term` by construction. |
| 5 | **Runtime table.** `GraphRuntimeState::for_genome(&CreatureGenome, plasticity_weights: Vec<Vec<Box<[f32]>>>)` replaces `GraphRuntimeState::new()` as the only constructor; it stores the scaffold it is given and builds the weight table `[mesh node][sink][edge]`, one `f32` per sink edge through `genomic_weight`. The seed handoff is untouched: the creature constructors still pass `newborn::take_plasticity_weights(&mut genome)` (every Graph node's seeds taken, layouts byte-identical, the empty outer vector for a seedless genome), and every other site passes `Vec::new()`. The sink loop reads the table and has no projection term; `enters_visit` is true with a compute node or a wired sink (a group's members wire their sinks). Compute-node learning is untouched. Every construction site in the Inputs table goes through `for_genome`; observation clones carry the state; an ablation copy (`redirect_leaf_reads`) rebuilt through `for_genome` keeps every member's genomic weight bitwise (the pattern resolves from `ring_ref`) while the member reads 0.0. |
| 6 | **Identity and serde (I9).** JSON key `own_weight` (no alias); `membership` omitted when `None`; `groups` omitted from JSON when empty and printed by `Debug` unconditionally; `BirthSeeds` is the only `#[serde(skip)]` field (`ablated_channels` is gone); the `cgp` genome types carry `deny_unknown_fields`; `Debug`, `PartialEq` and JSON cover every heritable field and nothing else; an exhaustive destructuring test makes a new def field a compile error. The text commit changes genome text for every graph genome; the behavior commit changes it only where a projection existed. |
| 7 | **Accounting.** `genome_size`: one unit per edge on both surfaces plus one per group record; a projection's nine units become eight members plus one record, so the replacement is size-neutral and the founder stays 80. Functional complexity: a group record counts one (replacing the projection's one), a wired sink counts one whether its edges are members or not, and a consumed reference counts through edges only (the projection reference chain goes). Per-operator `Δgenome_size`: `AddRingGroup` +9, `RemoveGroup` −(n + 1) for n members, `RemoveGraphEdge` on a last member −2, `AlterGroupScale`, `ShiftGroupOffset` and `RetargetGroupKind` 0. `cached_genome_size` and `cached_complexity` equal their functions after every edit. |
| 8 | **Operators.** `AddRingProjection` → `AddRingGroup` (2), `AlterRingProjectionWeight` → `AlterGroupScale` (4), `ShiftRingProjectionOffset` → `ShiftGroupOffset` (2), `RetargetRingProjectionKind` → `RetargetGroupKind` (2), `RemoveRingProjection` → `RemoveGroup` (2, `Decreasing`): `GraphOperator::ALL` stays 28, the default arm's total weight 56, `MutationOperator::all()` 61, keys `Graph.AddRingGroup` etc. Draws: `AddRingGroup` draws ring, kind, offset as `add` does today; the four unit operators pick a group uniformly first; `AlterGroupScale` steps `scale` with `perturb_weight`; `ShiftGroupOffset` and `RetargetGroupKind` keep today's second draw. Restricted size pressure admits `RemoveGroup` of the five. `GroupKey::Ring` and the ring arm of `correspondence` are deleted: refinement's `GroupKey`, `groups()`, `has_group()`, `Group` and `Member` become `SlotKey`, `inferred_sets()`, `has_inferred_set()`, `InferredSet` and `InferredMember`, and refinement never steps a member. Edge operators keep today's draws over `edge_sites`, which now enumerates members. |
| 9 | **Recruitment.** A ring family landing on a directed sink `K(d)` emits, under both arms, one `Ring` group of eight born-0 members with `offset = (d − channel) mod 8` (`SingleChannel`) or `offset = d` (`WholeFamily`, channel 0), in place of today's projection; every other plan is unchanged; admission treats a dangling member reference as a dangling leaf. Draw order is today's (family, channel, destination); the family universe drops ring families when `can_host_ring_group` is false, so a ring family's plan can always complete at whatever destination is drawn, and the "no local skip" rule of `recruit` holds. |
| 10 | **Validation (D5).** `validate_graph` checks invariants 2–4 and 7's structural half (I2–I10); the mutation gate (`apply_gated`, shared by all four domains) runs it on every Graph backend after every applied event of any domain (`InputRef.Prune` reindexes shape references, Topology events clone and swap backends) under `MutationSkipReason::GraphInvariantViolation` (key `GraphInvariantViolation`), rolling the draft back as a parseability violation does, with rollback tests in the Graph, InputRef and Topology domains; `GenomeFile::load` refuses a genome that fails it. An engine-level proptest over random grouped genomes × random events in all four domains asserts `Ok` after every applied event and that the counter stays 0; that holds because `InputRef.Swap` replaces a reference only within its kind partition (a ring stays a ring, `input_ref/mod.rs:20-82`) and `Prune` removes only an unaddressed reference, which a live group's members always address. |
| 11 | **Ablation (D3).** Members are plain sink edges: the core's `ablated` and the lab's `ablate` mask them through `redirect_leaf_reads`; `ablated_channels`, `ALL_RING_CHANNELS` and both projection branches are deleted; the lab's sentinel scheme stays one-for-one (`masking_a_ring_reference_zeroes_every_member_read`, `ablating_a_grouped_genome_leaves_size_complexity_groups_memberships_and_member_weights_unchanged`). `live_barrier_use.rs` is deleted. |
| 12 | **Hot path.** Per entered visit a graph with `P` groups does the same eight resolves and multiply-adds per group it does today, inside the sink loop instead of a separate pass; the sink loop reads a table entry per edge instead of a struct field; `GraphEdge` grows 12 → 16 bytes and the table adds 4 bytes per sink edge, built once per construction (one pass over the sink edges, at birth and at every observation or instrument construction). No per-visit allocation is added: the sink loop's existing 8-capacity `buf` (`effects.rs:117`) grows once to the longest sink and is not re-created per sink, and its per-visit allocation is F06's to remove. |
| 13 | **Determinism.** No new RNG draw. Draw streams change only for genomes with a group or with ring-channel edges refinement inferred today: `edge_sites` enumerates members, `push_edge` inserts before members, the Ring refinement key is gone, and the group operators replace the projection operators at the same catalog positions. The founder carries none of these, so the founder-only viability digests do not move, in either commit. |

Options weighed: [the readings file](../../progress/readings/t25-f03.md).

Readers, with their post-change expectations:

| Reader | Today | After F03 |
| --- | --- | --- |
| `ring_rules` (`ring-rules-v2`) | path (iii) is a projection with `\|w\| > 1e-3`; `projection_*` fields and rows | path (iii) is gone: paths (i) and (ii) run over every sink edge at its genomic weight, members included (a member's rule offset comes from its channel and sink as any edge's does), so coverage reads the effective connection `scale + own_weight`; `direction_general_edges_only` counts non-member edges only; the group records are reported in their own fields: `ring_group_carriers`, `ring_groups`, `ring_groups_nonzero` (`\|scale\| > 1e-3`), `ring_groups_unreachable`, `ring_group_members` (members present on reachable groups, summed), `ring_group_row_format` `family kind offset count positive negative members` keyed by `(family, kind, offset)` with the sign of `scale`; a `Ring` group at scale 1 and corrections 0 reads `direction_general` yes and `direction_general_edges_only` no |
| `drives` | fan-outs over sink edges | skips members (a group is not a fan-out) |
| `sensor_census`, `cgp_mesh_annotations`, `cgp_analysis`, `input_ref` consumed refs, recruitment admission | a projection consumes its ring; writes `Action` | members are wired sink edges: the same reads and writes fall out of the edge walks; the projection loops are deleted; a member-only node is an action writer |
| `discovery.rs` vote surface, `ladder/track.rs` `motor_node`, `steering::votes_move` | discovery leaves projection channels out; `motor_node` reports a projection-only node as not voting; `votes_move` reads projections through `projects_kind` | members are vote edges, so all three read them through the sink walk: `group_member_channels_are_on_the_vote_surface`, `a_node_voting_only_through_group_members_is_a_motor_node`, `votes_move` through members; `projects_kind` is deleted |
| `structure_means` (`PersistenceSample`) | `mean_projections` | `mean_ring_groups` (groups per creature) and `mean_ring_group_members` (members per group, `null` with no group); keep-list and `benchmark-series.json` `counter_definition_changes` updated |
| `REACHABLE_STRUCTURE_VERSION` | `reachable-structure-v1` | `reachable-structure-v2`: members are edges in every size count |
| Server `CreatureDetailResponse` | genome, `mesh_annotations` with the genome | `graph_weights` beside `genome`, computed independently of `include_genome`, excluded by the token `graph_weights`, omitted when the creature has no member: `[{node_id, scales: [{group, genomic}], edges: [{surface: "compute" \| "sink", container, edge, genomic}]}]` in genome order, one `edges` entry per member edge at F03 (F06 adds learning edges), keyed to `graphFullModel.ts`'s edge ids; `mesh_annotations` required whenever the genome is present; `PROTOCOL_VERSION` `v3alpha6` with a §2 history line |
| Inspector | `RingProjectionList`; edges drawn at `weight`; `infer*` and `collectNodeBadges` fallbacks | `EdgeGroupList`: one row per group (shape, kind, offset, scale, members present over 8 with holes), expandable member rows, a per-group collapse defaulting open; a member drawn dashed in a `group_member` tint at the server's genomic value, never at its own weight; `types/genome.ts` on the new shape; `useCreatureDetailResource` forwards `graph_weights` to the inspector store's static detail beside the genome, the incremental poll excludes `genome,graph_weights` and keeps the stored value, a selection change clears it; `infer*` and `collectNodeBadges` deleted (annotations are required); petri-visuals `edgeGroups` replacing `projections`, the `TINT` mirror and the min build rebuilt, `skills.lock.json` updated |

## Telemetry

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `petri.run.graph_edges_detached` | counter (`SimStats`, bench totals, family in `metrics.rs`, test row, `always_names`) | every detach the container performs (retarget, raw-field step, split) | — |
| `petri.run.groups_removed` | counter, same wiring | every group removal: `RemoveGroup`, a last member's `RemoveGraphEdge`, and a last member's detach (retarget, raw-field step, split) | — |
| both counters | committed events only | the engine's event summary reports detaches and removals, and a rolled-back event (parseability or invariant violation) reports none | — |
| `petri.run.mutation_events_applied_by_operator`, `..._skipped_by_operator` | existing families, key set changed | the five group operator keys replace the five projection keys; stored artifacts stay historical | — |
| `petri.run.mutation_events_skipped_by_reason` | existing family, one new key `GraphInvariantViolation` | the gate's `validate_graph` rejection | — |
| `ring_rules` v2 block, `mean_ring_groups`, `mean_ring_group_members` | bench report fields (not metrics) | the readers table | world, seed, tick; a `ring_rules` entry in `counter_definition_changes` |
| `graph_weights` | server payload | genomic member weights | `PROTOCOL_VERSION` |

## Verification Plan

**Question.** F03 claims a `Ring` group is bitwise its eight edges and that
members are ordinary edges to every operator and reader; the readings that
would show it wrong are the bitwise oracle or an invariant proptest failing,
a founder-only viability digest moving, a reader disagreeing with the genome
(`direction_general` moving at tick 0 between v1 and v2 on an unevolved
population), or a goal run whose `ring_rules` v2 block or group counters
disagree with the genomes they are read from.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate + goal` | The gate pins the hot-path claim and the 2,000-tick series carries the early counters and the `ring_rules` v2 definition change; goal-long is dropped by user decision (2026-10-15: at most once or twice per track, T25 ran it at F02 and F04's success criterion needs one), so the 40,000-tick readings F03 predeclared are deferred to F04's run (Notes). |
| Mutation gate | `run` | New production logic in `v3-core` (container primitives, group module, five operators, `validate_graph`, the weight table) and in `v3-server` whose behavior tests must pin. |
| Slow tests (`make rust-test-slow`) | `run` | Five operators change name and key, members enter every edge draw and recruitment's ring case emits a group: the tier pins the recruitment-paths rows those draws move. |

Viability first: `cargo test -p v3-core --test viability` before any other
test, after each commit.

## Implementation Tasks

- [x] **Commit 1 (text-only identity, `c3994a6b`).** `GraphEdge` gains `own_weight`
      (JSON key, no alias) and `membership: Option<Membership>` (always
      `None`), `size_of` 16; `Group`, `GroupId`, `Membership`, `Shape::Ring`,
      `compose` and the `groups` container field (always empty, no writer
      yet); `Debug` prints `groups` unconditionally and `projections` only
      when non-empty; `deny_unknown_fields` on the `cgp` genome types; the
      exhaustive destructuring test; `GENOME_FORMAT` 2; `PROTOCOL_VERSION`
      `v3alpha6`; `frontend/src/types/genome.ts` and every frontend read of
      `edge.weight` on `own_weight`. No draw, no sum, no counter changes:
      the only pins that move hash genome text (re-pin table), each checked
      by "the hash without genome text is unchanged".
- [x] **Commit 2 (behavior, `238ad78d`; the inspector split off as
      `6b356439`, which moves no pin), tests first** (the tests-first table below):
      the container primitives (`add_group`, `remove_group`,
      `relocate_group`, `detach_at`, `members`, `genomic_weight`,
      `GroupMut::set_scale`, I10 insertion, `validate_graph` and the
      invariant list beside it); `mutation/graph/group.rs` with the five
      operators and the detach calls from retarget, raw-field and split;
      `projection.rs` and `RingProjection` deleted; `GraphRuntimeState::for_genome`
      with the table and every construction site migrated; `effects.rs`
      without the projection term; the refinement renames; recruitment's
      ring case; the readers table; the counters; the gate skip reason;
      the serde shape; in the lab, `ablate` loses its projection branches
      (members are masked through `redirect_leaf_reads`), `expressed()`
      and its callers are kept unchanged, and only `live_barrier_use.rs`
      is deleted; the server payload and protocol spec;
      the inspector. Re-pins by the draw stream, one cause per commit (a
      later commit that moves no pin may be split off, such as the
      inspector).
- [x] Reference specs and `benchmark-series.json` rewritten to groups
      (Inputs table, last row); `docs/benchmark-artifacts.md` untouched.

Tests written first (file → test):

| File | Tests |
| --- | --- |
| `runtime/cgp/neutral_connection_tests/projection.rs` (renamed `group.rs`) | a `Ring` group (scale s, corrections `c_i` drawn nonzero) equals eight ungrouped edges of weight `s + c_i` bitwise with members in I10 position (the `eight_edges` oracle); `:99, :126, :155, :189` swapped to the group builder, strategies restricted to valid groups, the invalid arms moved to `validate_graph`-rejects tests; `:212` rewritten by hand to the new summation order; `:246, :264` as "an unresolvable `ring_ref` reads pattern 0" |
| `creature/genome/cgp/container_tests.rs`, `tests/projection_tests.rs` (renamed `group_tests.rs`) | `remove_group` leaves no member and renumbers links; `shift_then_unshift_is_identity` under I10; `detach_preserves_genomic_weight_bitwise`; `detaching_a_later_groups_member_ahead_of_another_member_keeps_the_edit_on_the_detached_edge`; a last member's detach removes the group; `push_edge` inserts before the first member; `can_host_ring_group` false at capacity (a def with `u16::MAX` groups) and with a partial catalog, and `AddRingGroup` inapplicable then; a JSON round trip and the two-way identity test; `size_of::<GraphEdge>() == 16`; nine-unit arithmetic as eight members plus one record, `n + 1` for a surviving group of `n` members |
| `mutation/graph/tests/projection.rs` (renamed `group.rs`), `mutation/types/tests.rs` | the catalog bijection test with `:70`'s weight, arm, key and admission assertions ported to the group names and `complexity_effect` agreement; draw-order tail tests per operator; the engine-level invariant proptest with the Δ column per operator; `refinement_never_steps_a_member` on a `Slots` fixture |
| `mutation/graph/recruitment_tests.rs` | `:223` on the group shape (one group of eight born-0 members, `projected` → `grouped`), regression file deleted and re-committed if it reappears; ring families absent from the universe when `can_host_ring_group` is false, present otherwise, every other family unchanged |
| `neighborhood/opportunity`, `crates/v3-lab/src/ladder`, `creature/genome/cgp_mesh_annotations.rs`, `neighborhood/steering.rs` | `group_member_channels_are_on_the_vote_surface`; `a_node_voting_only_through_group_members_is_a_motor_node`; an annotation-driven test that a member-only node is an action writer; `votes_move` through members |
| `creature/ring_rules_tests.rs`, `drives.rs`, `crates/v3-cli/src/bench/ring_rules.rs`, `crates/v3-cli/src/lib.rs` | the per-field ring-rules-v2 expectation for a `Ring` group; a group is not a fan-out; `structure_means` counts groups and members |
| `neighborhood/input_use/tests.rs`, `crates/v3-lab/src/readings/tests.rs` | `ablating_a_grouped_genome_leaves_size_complexity_groups_memberships_and_member_weights_unchanged`; `masking_a_ring_reference_zeroes_every_member_read` |
| `mutation/engine`, `crates/v3-telemetry/src/metrics/tests.rs`, `crates/v3-server/tests/server.rs` | the gate rejects an invariant-violating event under `GraphInvariantViolation` and restores the draft, one test per domain (Graph, InputRef, Topology); a rolled-back event adds nothing to either counter; the two counter families; `graph_weights` present for a member, absent without, excluded by token |
| `frontend/src/components/inspector/EdgeGroupList.test.tsx` (replacing `RingProjectionList.test.tsx`), `graphFullModel.test.ts`, `useCreatureDetailResource.test.tsx`, `petriVizDrift.test.ts` | the group row and member rows; a member at scale 0.5 and correction 0 draws at 0.5 from `graph_weights`; the fetch-to-store-to-inspector path carries `graph_weights` on the initial load, keeps it across incremental polls that exclude it, and clears it on a selection change; the `group_member` tint asserted and the composition source in the drift fingerprint |

Every `RingProjection` test site is ported, rewritten or deleted:
`creature/genome/tests/projection_tests.rs`,
`runtime/cgp/neutral_connection_tests/projection.rs`,
`mutation/graph/tests/projection.rs`, `sensor_census.rs:752-990`,
`steering.rs:237-245`, `crates/v3-cli/src/lib.rs:330-358`,
`crates/v3-telemetry/src/tests.rs:1269`.

## Verification

- [x] Viability first: `cargo test -p v3-core --test viability` -> 29
      passed after commit 1 (`c3994a6b`), commit 2 (`238ad78d`) and the
      self-review commit (`2d300ef9`); the founder-only siblings did not move.
- [ ] Focused tests (the tests-first table) and `make check` -> result,
      commit hash. Focused: `cargo test --workspace --no-fail-fast` all
      passed at `238ad78d`; at `2d300ef9` `cargo test -p v3-core` passes,
      `cargo clippy --workspace --all-targets` is clean,
      `cargo fmt --all --check` and `cargo check --workspace --all-targets`
      pass, and frontend `npx vitest run` (630 passed) and
      `npx tsc --noEmit` pass. `make check` pending.
- [ ] Mutation gate: fresh `MUTANTS_ITERATE=0 make rust-mutants`, its
      summary line, output path, and every survivor resolved as killed,
      equivalent, or deferred.
- [ ] Slow tier: `make rust-test-slow` -> result, with the re-pinned
      recruitment-paths rows recorded in the readings file.
- [x] Gate and goal summaries at
      `docs/progress/features/t25-f03-ring-groups-replace-ring-projections.json`
      and `-goal.json`; raw hash and byte count, verification time, series
      entries; no full report staged.
      Recorded 2026-10-15 (`make bench`, gate then goal): CLI exit 0 and
      `severe` false for both, outer exit not observed (recording agent
      stopped; source is the summaries), raw 105,736 and 691,454,296 bytes
      verified against the summaries' hashes, verified at 15:27:04Z and
      15:35:26Z, series entries added; the Performance section and readings
      file carry the verdicts. goal-long not run (dropped).
- [ ] Readings in [`docs/progress/readings/t25-f03.md`](../../progress/readings/t25-f03.md)
      (goal-profile readings recorded under "Benchmark runs"; the readers'
      post-change expectations remain open):
      the three readers' post-change expectations checked on the goal
      populations; ring groups per creature, members per group,
      `graph_edges_detached` and `groups_removed` totals as the goal profile
      reads them; the size series continued from F02.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: Eph/ephrin
gradients and central-complex columns, one developmental program wiring a
whole map whose synapses are nonetheless individual synapses afterwards. It
reaches creatures through the vote surface (members are real edges on the
direction sinks, summed where their sinks sum) and through mutation (each
member is now addressable by the edge operators). The founder carries no
group and no ring-channel edge into a direction sink (its four ring reads
feed `CustomOutput` sinks on node 0, which no group or inferred ring set
touches), so every mutation-off reading is identical.

Compute: no per-visit cost expected (invariant 12); the predeclared costs
are the table build at every construction (one pass over the sink edges)
and memory of 4 bytes per edge plus 4 per sink edge. The gate's
`wall_clock_ms_per_creature_tick` is compared against the current epoch
under the ordinary thresholds; a `severe` flag on wall-clock is a defect to
find, not a cost to accept. `plasticity_updates` has no mechanism to move
except trajectory divergence (learning stays per node). With mutation on,
every per-seed counter may move through the draw stream (invariant 13)
with no predeclared direction, and a `severe` flag on such a counter is a
trajectory reading reported to the user, who decides whether to accept it;
the deterministic block changes by construction (operator keys, `ring_rules` v2, `reachable-structure-v2`, the
persistence fields) and the gate epoch is re-pinned in the closing commit.

Goal: `mean_genome_size` is unchanged by construction at tick 0 and
tracks the F02 rule thereafter. The `ring_rules` v2 block,
`mean_ring_groups`, `mean_ring_group_members`, `graph_edges_detached` and
`groups_removed` enter the 2,000-tick series under their new definitions
with no predeclared direction (a group is born silent and the series is
too short for its erosion to be read). Births, gestation, persistence,
clade count and entropy have no predeclared direction. A `severe` flag or
threshold crossing is reported with its reading and goes to the user. The
40,000-tick readings this feature would have predeclared (ring-carrier
retention, member erosion) are deferred to F04's goal-long run (Notes).

Predeclared re-pins, one cause per commit, each restorable by reverting the
named value:

| Commit | Pins | Mechanism | Restoring check |
| --- | --- | --- | --- |
| 1 | `tests/baseline_worlds.rs:142`, `tests/applied_trajectory.rs` two sampled digests (`:59-66`, which hash genome bytes), `mutation/engine/recruitment_tests.rs:49`, `neighborhood/mutation_effects/mod.rs:60` `genome_identity` consumers (world-set deterministic block), `bench_artifacts.rs` digests that include genome text; any other pin that moves at commit 1 is named in the readings with the same check. The two `applied_trajectory.rs` guard digests (`:147-155`, no genome bytes) and the founder-only viability siblings must not move | genome `Debug` and JSON text: `own_weight`, `membership: None`, `groups: []` printed | the digest recomputed without genome bytes equals the merge-base value: the implementer computes the stripped digests at the merge base and records them in the readings before commit 1 (the T02.F08 and T19.F03 precedent in each file's header) |
| 1 | `crates/v3-lab` genome-file tests, `v3-server` detail-response tests, frontend fixtures carrying `weight` | `GENOME_FORMAT` 2, `v3alpha6`, the key rename | reverting the rename and both version constants restores every value |
| 2 | `applied_trajectory.rs` four digests (the two guard digests may move here and only here; outcome: they held, the readings record which listed pins moved), `baseline_worlds.rs:142`, `recruitment_tests.rs:49`, the slow-tier and `--lib` recruitment-paths rows, `recruitment_tests.rs:223` and its regression file, `neutral_connection_tests/projection.rs:212`, the v3-cli trajectory fixtures (`the_real_run_path_*`, `persistence_samples_*`; re-searched with every assertion and oracle unchanged, a weakened assertion being a requirement change for the spec owner), the gate and world-set deterministic blocks | draw stream (invariant 13) and the summation order on sinks carrying members; `ring_rules` v2 and `reachable-structure-v2` tokens; the renamed operator keys | the causal check is mutation off: with no draws, every trajectory of any genome without a group is byte-identical across the commit (the founder-only viability siblings, and any `applied_trajectory.rs` sampled run the implementer takes with `mutation.per_unit_rate` 0; its existing `Some(0.0)` argument is the gestation rate, not the mutation rate), while the mutation-on movers are attributed by the two-run agreement rule; a pin that moves under mutation off, or for a reason outside invariant 13, is reported to the spec owner |
| 2 | `counter_definition_changes` entries for `ring_rules` and `mean_projections` → `mean_ring_groups` | definition change | the entries name before and after |

**Measured verdict** (2026-10-15, summaries at `21f440ba`, code `2d300ef9`; command lines, hashes and tables in the readings file, "Benchmark runs"). The recording agent was stopped after both profiles completed: the outer exit status was not observed (source: the summaries only; their `outer_exit` is null).

| Profile | CLI exit (source: `measurement_evidence.cli_exit`) | Outer exit | `severe` | Verdict |
| --- | ---: | --- | --- | --- |
| gate | 0 | not observed | false | no counter crossed a threshold; wall +4.7% against F02 (ok); the gate epoch is still to be re-pinned in the closing commit |
| goal | 0 | not observed | false | flags only: `vm_steps` +21.9% and `plasticity_updates` +22.6% against F02; wall -3.9% (ok); per-case readings the CLI does not level moved in Canyon and Confluence (readings, "Anything that does not fit") |

Raw and summary sizes: gate 105,736 / 20,410 bytes, sha256 `e487961aac0c26ed5cb4c69a05ea9a9599aca25900af951df31d5852987054d2`; goal 691,454,296 / 386,157 bytes, sha256 `73c460cf00b697e5f5abc4bd2536c2916fd40f4a0af28076931fe69cfa4aa8ec`. Raw reports sit under `.bench-artifacts/t25-f03-ring-groups-replace-ring-projections/` and are not staged. Each summary's recorded raw hash and byte count equal the local raw file. The goal summary records `dirty: true` (an untracked gate summary sat in the worktree).

Goal predeclaration readings: mean ring groups per creature 0.325 / 1.494 / 0.669 (Orchards / Canyon / Confluence), mean members per group 7.87 / 7.86 / 7.81 (below 8 on every world), `graph_edges_detached` 469 and `groups_removed` 811 in total (both above 0), no extinction, `mean_genome_size` 81.5 to 81.9 at the first sample.

- Summaries of the declared profiles:
  [gate](../../progress/features/t25-f03-ring-groups-replace-ring-projections.json),
  [goal](../../progress/features/t25-f03-ring-groups-replace-ring-projections-goal.json).
- Full readings: [`docs/progress/readings/t25-f03.md`](../../progress/readings/t25-f03.md).

## Success Criteria

- [ ] A `Ring` group is born as eight member edges with one shared scale
      and survives with one to eight; the bitwise oracle, the I10 identity
      tests and the invariant proptest pass; `RingProjection` and its five
      operators no longer exist.
- [ ] Any one member can be re-weighted, retargeted, split or removed on
      its own while the group survives; the last member's removal removes
      the group; the two counters and `ring_rules` v2 read it on the goal
      runs.
- [ ] Every reader in the readers table counts members as edges and reads
      membership from the genome; no reader re-derives explicit membership;
      `drives` and the edge-only ring-rule paths keep their definitions over
      non-member edges.
- [ ] Size-neutral: the founder is 80 units, a group is nine at creation
      and `n + 1` with `n` surviving members, and the founder-only
      viability digests are unchanged in both commits.
- [ ] Track row checked, spec Complete, `make check` green on main.

## Notes for AI Agents

- Decision: (user, 2026-10-13) only geometry is built in: the ring pattern
  is 1 on every member, a scale sign is evolved, and group creation never
  randomizes the pattern; learning attributes and scope arrive in F06–F08
  and carry no genome unit.
- Decision: (user, 2026-10-13) the replacement is size-neutral under F02's
  nine-unit projection charge; the founder anchor stays 80 and the rate
  `0.0060625`.
- Decision: (user, 2026-10-15) goal-long runs at most once or twice per
  track, only where absolutely necessary; T25 ran it at F02 and F04's
  success criterion needs one, so F03 declares `gate + goal`.
- Deferred: to F04's goal-long run, the 40,000-tick readings F03 planned:
  `ring_group_carriers` of the order of F02's `projection_carriers` (698 of
  832 on Canyon, 4,492 of 4,936 on Confluence, 11,801 of 12,221 on
  Orchards) on at least one world; the mean members per reachable group
  (`ring_group_members / ring_groups`, `mean_ring_group_members`) below 8
  on at least one world, a member being one of its graph's `E` edges for
  `RemoveGraphEdge` (weight 2 of 56, uniform over `edge_sites`) and
  surviving `n` such events with probability about `(1 − 1/E)^n` (a
  constant-size approximation ignoring detaches, whole-group removals and
  the node draw); `graph_edges_detached` and `groups_removed` above 0 on
  every world that evolves groups.
