# T25.F04 — Direction-Pair Groups

**Status**: In Progress
**Last updated**: 2026-10-15
**Feature**: T25.F04
**Track**: [T25 — Edge Groups and Per-Edge Learning](../../roadmaps/t25-edge-groups-and-learning.md)

## Goal

The sensory map sits in register with the motor map, as the superior
colliculus lays a retinotopic sheet over a motor sheet so a place seen is a
place turned toward: a creature's sensed `(x, y)` offset to food, a barrier,
a crowd or another creature can reach all eight direction votes of one kind
`K` as one silent gene, a `DirectionPair` edge group `{shape: DirectionPair
{ref_idx, pair}, kind, offset, scale}` whose twelve member sink edges carry
the compass pattern: the pair's `x` field into `K[d]` at pattern
`compass_x((d − offset) mod 8)`, its `y` field at `compass_y(...)`, so at a
positive scale the group votes toward the sensed thing, at offset 4 away from
it, and the sign of the scale is evolved. The same thing's plain facts (its
amount, distance, presence) reach `Drive K` as ordinary born-0 edges when
recruitment wires the pair. Families declare their pairs once in
`contracts::inputs`; `AddDirectionPairGroup` joins the catalog; the five
group operators, recruitment, every reader and the inspector treat a pair
group as F03 treats a `Ring` group; a `pair_groups` block on every
persistence checkpoint and the goal-long run read whether evolution keeps
such a group and moves its scale off zero.

## Non-Goals

- Any computation built in beyond geometry: no bundle gene, no learned
  pattern, no pair on a compute node (user decision, 2026-10-13).
- Learning attributes, scope, `inherit_scale`, randomized or learned creation
  (F06–F08); the `NearbyCreature` merge and one-slot recruitment everywhere
  (F05); the `Slots` refinement decision (F08).
- `Drive` sinks as group targets; the VM backend; founder rewiring; type or
  module renames beyond the one predicate below.
- A carrier-keyed per-creature instrument (blocked-move or steal-hit
  fractions of carriers against non-carriers): no row names it, and F03's
  rebuttal stands (readings file, options table).
- A `drives-v2` block: a pair group's per-kind counts are the per-kind sums
  of the `pair_groups` block, and the facts edges are drive edges `drives-v1`
  already counts once `|w| > 1e-3`.

## Inputs and Invariants

| Input | Where | What F04 takes from it |
| --- | --- | --- |
| Track row, success criterion 4 and Notes | `docs/roadmaps/t25-edge-groups-and-learning.md` | The goal; a group born silent on a creature, food, barrier or crowd offset retained with its scale off zero in a 40,000-tick goal-long run on at least one standard world; F04 is a goal-long carrier and carries F03's deferred 40,000-tick ring readings; F04 records the refinement-by-key readings F08 reads; mutation gate and slow tests run; user decisions of 2026-10-13 (only geometry, never a computation; the pairs declared in `contracts::inputs`). |
| Design note | `docs/strategy/edge-groups-and-learning-design-2026-10-13.md` §3 (`Shape::DirectionPair`, I4 pair role, I7), §4 (compass pattern, pair metadata D12, recruitment's pair case, draw contracts), §8 F04 contract | The pattern derivation, the declaration site, the operator draws, the tests-first list, the readings; the track rows are authoritative where the two differ (Non-Goals). |
| F03 outputs | `docs/specs/roadmap/t25-f03-ring-groups-replace-ring-projections.md` invariants 1–13 and Notes | `Group`, `Shape`, `Membership`, `compose`, the container primitives (`add_group`, `remove_group`, `relocate_group`, `detach_at`, `members`, `genomic_weight`, I10 insertion, `validate_graph`), the five operators, `for_genome`'s weight table, the gate's `GraphInvariantViolation`, the ablation redirect, `graph_weights`, `EdgeGroupList`; the deferred 40,000-tick ring readings. |
| Layouts | `crates/v3-core/src/sensors/perception.rs:12-67` (`food_idx`, `barrier_idx`, `occupancy_idx`, `core_idx`, `vitals_idx`, `identity_idx`), `reducers.rs:89-150, :217-290, :354-420` (`dx = cell − self`, `x` right, `y` down, normalized by the vision radius), `docs/reference/v3-sensor-spec.md` §5–6 | The pairs: `AreaFoodSummary(t)` `(gradient_x, gradient_y)` 1,2 and `(nearest_dx, nearest_dy)` 3,4, facts 0,5,6; `AreaBarrierSummary` 2,3 and 4,5, facts 0,1,6; `AreaOccupancySummary` `(center_x, center_y)` 1,2 and 3,4, facts 0,5,6; `NearbyCreatureCore` slot `k` `(rel_x, rel_y)` `4k+1, 4k+2`, facts `4k, 4k+3`. |
| Compass | `crates/v3-core/src/contracts/direction.rs:28-40` (`Direction::delta`, `to_index`, `ALL`) | `compass_x(d)` = `dx` normalized, `compass_y(d) = compass_x((d − 2) mod 8)`. |
| Group code | `creature/genome/cgp/group.rs` (`Shape`, `Group::pattern(input_refs)`, `detach`, `ring_offset`, `compose`), `container.rs:307-310` (`member_direction`), `:320-339` (`validate_graph`), `:850-866` (`can_host_ring_group`, `direction_sink`), `:887-1000` (primitives), `:1006-1030` (`member_key`, `member_sink`, `insert_member`), `:1062-1120` (`check_local_invariants`, the 8-bit channel mask); `mutation/graph/group.rs` (`ring_indices`, `can_add`, `add`, `pick`, the four unit operators); `mutation/graph/mod.rs:42-160, :236, :396-413` (`ALL` 28, weights 56, `enabled`, apply); `mutation/types/mod.rs:246-250, :470` (`all()` 61, keys) | Everything a second shape generalizes. |
| Recruitment | `mutation/graph/recruitment.rs` (`Plan::append` ring case, `Admission::{new, plan, families}`, `recruit`'s draw order: family, channel under `SingleChannel`, destination), `recruitment_tests.rs:218-236` (the 1..=30 added-edge bound) | The pair case and its draw order. |
| Readers | `creature/ring_rules.rs` (`Ring` only), `drives.rs:62-95` (`node_fanouts` skips members), `mutation/graph/refinement.rs:17-62` (`SlotKey`), `mutation/input_ref/mod.rs:20-82` and `genome/mesh_annotations.rs:156-185` (Swap's `(class, width)` partition: every pair-declaring key is alone in its kind, `AreaFoodSummary` across food types), `crates/v3-cli/src/lib.rs:188` (`structure_means`), `bench/tracking.rs:765-800, :960-990, :1034-1080` (`PersistenceSample`, `PopulationReadings`), `bench/artifacts.rs:594-640` (keep-lists), `simulation/stats.rs:272, :342-344` (by-operator maps, the F03 counters), `crates/v3-telemetry/src/metrics.rs:347-440, :700-730` (keyed families), `scripts/telemetry-dashboards-check:221-228` (`keyed_names`), `crates/v3-server/src/http/creature.rs:64-140` (`graph_weights`), `docs/reference/v3-server-api-protocol-spec.md:760` | Each reader's post-change expectation (table below). |
| Frontend | `frontend/src/components/inspector/EdgeGroupList.tsx` (`collectGroupMembers`, the ring row over eight cells), `inputRefUtils.ts:78-120` (`AREA_FIELDS`, the hand mirror of `perception.rs`), `types/genome.ts`; `.agents/skills/petri-visuals/{spec.md:19, viz/petri-viz.js, viz/petri-viz.min.js}`, `.agents/skills.lock.json` | The pair row and the viz form. |
| Pins | `tests/viability.rs` founder-only digests (must not move), `tests/baseline_worlds.rs` (the digest its header comment at `:142` explains), `tests/applied_trajectory.rs` four digests, `mutation/engine/recruitment_tests.rs` (the supply pin its comment at `:49` explains), `mutation/graph/recruitment_tests.rs:223` and its regression file, the slow-tier and `--lib` recruitment-paths rows, the v3-cli trajectory fixtures, the gate and world-set deterministic blocks, the catalog bijection test, `docs/progress/benchmark-series.json` | The predeclared re-pins (Performance section). |
| Reference specs | `docs/reference/v3-graph-backend-spec.md` "Ring edge groups (T25.F03)", `v3-mutation-spec.md` edge-group operator table and `RecruitNeutralInput`, `v3-sensor-spec.md` §5, `v3-server-api-protocol-spec.md` §2, §4.11 | Every sentence that states a group shape, rewritten for two shapes. |

Invariants (the container asserts the def-local ones at every exit; the
group module asserts I4's pair role and I7 with `input_refs`;
`validate_graph` checks all at `GenomeFile::load` and in the mutation gate):

| # | Invariant |
| --- | --- |
| 1 | **Declaration.** `contracts::inputs` owns the layouts: the six `*_idx` modules move there from `sensors/perception.rs` (re-exported from `sensors::perception`, so every reader compiles unchanged) and `WorldInputKey::direction_pairs(self) -> &'static [DirectionPair]`, `DirectionPair { x: u16, y: u16, facts: &'static [u16] }`, is written in terms of them: the pairs and facts of the Layouts row, empty for every other key. Eligibility depends on the `WorldInputKey` variant only, never on a food type, so every `InputRef.Swap` alternative of a pair-declaring reference declares the same pairs. A pair's `x`, `y` and `facts` are distinct and below `compound_width`; `sensors::reducers` writes against the moved modules. |
| 2 | **Compass.** `compass_x(d)` is `Direction::ALL[d].delta().0` normalized: exactly `0.0`, `±1.0` or `±FRAC_1_SQRT_2`; `compass_y(d) = compass_x((d + 6) mod 8)`, which equals the normalized `dy` (`py_equals_normalized_delta_dy`); every row has norm 1 within 1 ulp. A pair member reading field `x` on sink `K[d]` has pattern `compass_x((d − offset) mod 8)`, field `y` `compass_y(...)`; the applied weight is `compose(own, scale, pattern)` as two rounded operations, no `mul_add`, and sink sums stay sequential. |
| 3 | **Shape and pattern.** `Shape::DirectionPair { ref_idx: u16, pair: u8 }`, JSON `{"DirectionPair": {"ref_idx", "pair"}}`; `Shape::Ring` and every F03 type are unchanged. `Group::pattern(&self, input_refs, sub_idx, direction)` replaces `pattern(input_refs)`: a `Ring` ignores the member coordinate; a `DirectionPair` resolves `input_refs[ref_idx]` to a `World` key whose `direction_pairs()[pair]` exists and returns the `x` or `y` pattern for `sub_idx` equal to that field, else `0.0`. The pattern resolves from the group's `ref_idx`, never the member's own source, so the ablation redirect keeps every member's genomic weight bitwise. Because identical-source pair members sit on different sinks at different patterns, `genomic_weight(occurrence, input_refs)` is the one resolver and `edge_weight(edge, input_refs)` (`container.rs:478`) becomes `edge_weight(edge, sink, input_refs)`: `sink_weight_rows` (`container.rs:498`), `ring_rules.rs:192` and the runtime oracle (`runtime/cgp/effects.rs:82`) pass the member's sink (`identical_source_members_on_different_sinks_resolve_different_weights`). |
| 4 | **Members (I4, I5, I7, I8).** A pair member is a sink edge `InputLeaf { ref_idx, sub_idx ∈ {x, y} of the group's pair }` on a direction sink of `kind`; its coordinate is `(field, direction)`, the direction being the sink it sits on; at most one member per `(group, source, target)`; `pair < direction_pairs().len()` of `input_refs[ref_idx]`, which is a `World` key; `offset < 8`; `ref_idx ≤ u16::MAX`. Creation materializes the twelve members whose pattern is non-zero (`x` at relative directions NE, E, SE, SW, W, NW; `y` at N, NE, SE, S, SW, NW: `pair_creation_materializes_12_members_not_16`) at own weight 0 and scale 0: a new group is silent, and twelve is its birth count, not a bound on a surviving group. A member's relative coordinate `(field, (d − offset) mod 8)` is preserved by every operation; a member on a zero-pattern direction is valid (a loaded genome may carry one) and is never created. Detach, `RemoveGraphEdge`, retarget, raw-field and split on a pair member behave as on a `Ring` member (F03 invariant 2); the last member's removal removes the group. |
| 5 | **Sink order and relocation (I10).** Within a sink, independent edges first, then members sorted by `(GroupId, sub_idx)`; a pair group has at most two members per sink (`x ≠ y`), so the key is unique. `relocate_group(group, kind, offset)` moves a pair member on `K[d]` to `kind[(d + offset − old_offset) mod 8]`, preserving every member's relative coordinate (holes, and any zero-pattern member, included), so a shift and its inverse return an equal genome; a `Ring` member moves by channel as today. The container carries a pair member's direction explicitly at insertion; a `Ring` member's derives from its channel. |
| 6 | **Validation.** `check_local_invariants` keys a group's member set by `(channel)` for `Ring` and `(field role, direction)` for `DirectionPair` (the 8-bit channel mask generalizes); `validate_graph` adds I7 for pairs and I4's role check; the gate's `GraphInvariantViolation` and its proptest (random grouped genomes of both shapes × random events over all four domains, `Ok` after every applied event, counter 0) cover both shapes. `InputRef.Prune` never removes a pair reference (its members address it); `reindex_input_refs_after_removal` decrements both shapes' references. |
| 7 | **Accounting.** One unit per edge plus one per record: `AddDirectionPairGroup` is `Δgenome_size` +13, `RemoveGroup` −(n + 1) for `n` members, the unit operators 0; functional complexity counts a record one and references through edges; `cached_genome_size` and `cached_complexity` equal their functions after every edit; the founder stays 80. |
| 8 | **Operators.** `AddDirectionPairGroup` (weight 2, `Increasing`, key `Graph.AddDirectionPairGroup`) enters `GraphOperator::ALL` directly after `AddRingGroup`: `ALL` 29, default-arm total weight 58, `MutationOperator::all()` 62, the bijection test and `complexity_effect` agreement extended; it is `enabled` unconditionally like `AddRingGroup` and excluded under restricted size pressure. Applicability: a Graph node with a pair-declaring reference at an index ≤ 65,535 and `can_host_group(def)` (renamed from `can_host_ring_group`, the same predicate: all 24 direction sinks and `groups.len() < u16::MAX`). Draws: one `gen_range` over the flat list `(ref_idx ascending, pair ascending)` of declared pairs, then kind `gen_range(0..3)` over `DirectedVoteKind::ALL`, then offset `gen_range(0..8)`; duplicates allowed. The four unit operators pick one group uniformly over both shapes and keep their draws; `AlterGroupScale`'s law is unchanged on a pair's scale. Edge operators keep today's draws over `edge_sites`. |
| 9 | **Recruitment.** Under `WholeFamily`, a pair-declaring family landing on a directed sink `K(d)` draws one pair `gen_range(0..direction_pairs().len())` after the destination (the only new draw; for `NearbyCreatureCore` the pair is the slot) and emits one silent `DirectionPair` group at `offset = d` (twelve members) plus one born-0 ungrouped edge into `Drive K` per field of the pair's `facts`, in `facts` order: 15 edges for an area family, 14 for a creature slot, within the 30-edge bound. `SingleChannel` keeps today's single-edge plan and draw order (family, channel, destination); a non-directed destination keeps today's whole-family plain-edge plan; a ring family keeps F03's case and F03's admission rule unchanged (`ring_admission_is_unchanged_by_the_pair_case`). The preflight drops pair-declaring families from the `WholeFamily` universe when `can_host_group` is false or any of the three `Drive(K)` sinks, `K` over `DirectedVoteKind::ALL`, is absent (the kind is fixed by the destination, drawn after the family), so a drawn plan always completes; the reference is declared once and shared by the group and the facts edges. |
| 10 | **Runtime.** `for_genome` resolves a pair member's pattern once at construction through `genomic_weight`, which reads the member's direction from its sink; the sink loop is unchanged and reads one table entry per edge: no shape-resolution cost per visit beyond ordinary edge evaluation, no new allocation. |
| 11 | **Refinement by key.** `RefineHeritableStructure` never steps a member of either shape; every applied refinement increments `SimStats.refinement_applications_total_by_key` keyed by the stepped set's `(family, field)`, reported by the engine's event summary for committed events only and exported as `petri.run.refinement_applications_by_key` (attributes `petri.world_input` = the family's `as_key()` (the existing attribute), `petri.field` = the decimal field index; bounded by the creature families' widths), and carried in the bench summary as `MutationSupply.refinement_applications_by_key: BTreeMap<String, u64>` (`bench/schema.rs:1067`, key `"<family.as_key()>:<field>"`, `#[serde(default)]`, absent in older reports) inside the `mutation_supply` case block that `KEPT_CASE_BLOCKS` keeps whole (`a_committed_refinement_count_survives_summary_projection`). |
| 12 | **`pair_groups` block (`pair-groups-v1`).** Every `PersistenceSample` carries `pair_groups: {version, creatures, carriers, groups, groups_nonzero, groups_unreachable, members, add_applied_total, rows: [{family, kind, offset, count, nonzero, positive, negative, members}]}`: `DirectionPair` groups on reachable nodes of the living population, `nonzero` being `\|scale\| > 1e-3` (`PATH_WEIGHT_FLOOR`), `positive`/`negative` the sign of `scale` among them, `members` the members present, rows keyed by `(family.as_key(), kind, offset)` in that order, `add_applied_total` the cumulative `Graph.AddDirectionPairGroup` applied count (recruitment's creations are not separated: they share `Graph.RecruitNeutralInput`), `inherited_nonzero_births_total` the cumulative inheritance witness below; `null` at extinction. `pair_groups` joins `KEPT_SAMPLE_FIELDS`. **Inheritance witness.** `SimStats.pair_group_inherited_nonzero_births_total` counts a conception once when the child's genome, after its mutation events are committed (rolled-back events excluded), holds a `DirectionPair` group with `\|scale\| > 1e-3` and no committed event at that conception could have created a group (no `Graph.AddDirectionPairGroup` and no `Graph.RecruitNeutralInput` in the event summary, read where `graph_edges_detached_total` is read; a litter lost in gestation still counts, as every mutation counter does): every pair group the child carries then came from the parent's genome, so the count witnesses a parent-to-child inheritance of a group whose scale is off zero, whether the parent carried it nonzero or an `AlterGroupScale` at this birth stepped it. It is exported as `petri.run.pair_group_inherited_nonzero_births` and carried in `MutationSupply` beside `groups_removed_total`. `mean_ring_groups` and `mean_ring_group_members` keep their `Ring`-only definitions. |
| 13 | **Identity and determinism.** No existing genome's `Debug` or JSON text changes (an externally tagged enum gains a variant); `GENOME_FORMAT` stays 2. Draw streams change for every mutation-on run (a new weighted operator remaps every default-arm Graph draw; recruitment's pair draw and plan); with mutation off every trajectory is byte-identical, so the founder-only viability digests do not move. |

Readers, with their post-change expectations:

| Reader | After F04 |
| --- | --- |
| `ring_rules` (stays `ring-rules-v2`) | `Ring` groups only in `ring_group_*`; paths (i) and (ii) are ring-sourced, so pair members do not enter them; unchanged |
| `drives` (stays `drives-v1`) | `node_fanouts` skips members of either shape; a facts edge into `Drive K` is a drive edge once `\|w\| > 1e-3` |
| `sensor_census`, `cgp_mesh_annotations`, `cgp_analysis`, consumed references, recruitment admission, discovery's vote surface, `motor_node`, `votes_move` | pair members are wired sink edges and vote edges: the same reads and writes fall out of the edge walks (`pair_group_member_channels_are_on_the_vote_surface`) |
| `refinement` | `SlotKey` sets never include a member; applications counted by key (invariant 11) |
| `structure_means`, `PersistenceSample` | `pair_groups` block (invariant 12); `REACHABLE_STRUCTURE_VERSION` unchanged (members are edges) |
| Lab | members are edges; `ablate` masks them through `redirect_leaf_reads` and a pair member's genomic weight survives bitwise (`masking_a_pair_reference_zeroes_every_member_read`); `GENOME_FORMAT` 2 |
| Server | the genome's `groups` list carries the shape variant; `graph_weights` carries one `edges` entry per actual member of a pair group (twelve at birth) under the F03 payload shape; `PROTOCOL_VERSION` `v3alpha7` with a §2 history line |
| Inspector | `EdgeGroupList`: a pair row `Pair <family>.<x>/<y> #n` with kind, offset, scale and members present over the twelve birth coordinates (holes by field × direction; a zero-pattern member, if present, listed as an extra row), expandable member rows; members drawn dashed at the server's genomic value as today; `inputRefUtils.ts` gains `DIRECTION_PAIRS`, pinned against `direction_pairs()` through one shared fixture `frontend/src/components/inspector/directionPairs.json` that a `v3-core` test and a frontend test both compare to; petri-visuals `edgeGroups` gains the pair form (`pair: {family, x, y}`, `members` as `x:E`-style cells, default all twelve), the min build rebuilt, `skills.lock.json` updated |

Options weighed: [the readings file](../../progress/readings/t25-f04.md).

## Telemetry

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `petri.run.mutation_events_attempted_by_operator`, `..._applied_by_operator`, `..._skipped_by_operator` | existing families, key set gains `Graph.AddDirectionPairGroup` | the catalog | — |
| `petri.run.refinement_applications_by_key` | counter family (`SimStats`, `MutationSupply.refinement_applications_by_key`, family in `metrics.rs`, test row; in `scripts/telemetry-dashboards-check` `keyed_names` only if the check workload is shown to key it, otherwise named in that script's comment of families the workload gives no key to), attributes `petri.world_input`, `petri.field` | every committed `RefineHeritableStructure` application, keyed by the stepped set | — |
| `pair_groups` block | bench persistence field (not a metric), `pair-groups-v1`, in the keep-list | invariant 12 | world, seed, tick |
| `petri.run.pair_group_inherited_nonzero_births` | counter (`SimStats`, `MutationSupply`, family in `metrics.rs`, test row, `always_names`) | the inheritance witness of invariant 12: conceptions with committed mutation events, read at the site of the F03 counters; gestation losses are not subtracted | — |
| `graph_weights` | server payload | genomic member weights of both shapes | `PROTOCOL_VERSION` |
| Recorded configuration | none added: the operator sits in the fixed catalog and no switch or config value is introduced | — | — |

## Verification Plan

**Question.** F04 claims a `DirectionPair` group at scale `s` and offset 0
votes toward the sensed offset of its family's thing as twelve ordinary
edges (offset 4 away from it) and that evolution keeps such a group and moves
its scale off zero within 40,000 ticks; the readings that would show it
wrong are a per-family argmax test or the invariant proptest failing, a
founder-only viability digest moving, or no world's goal-long run meeting
the retention reading (Performance: the inheritance witness rising over
the last 10,000 ticks and a reachable pair group with `\|scale\| > 1e-3`
at every sample from tick 30,000 through 40,000).

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate + goal + goal-long` | The gate pins the no-per-visit-cost claim; a new weighted operator remaps every Graph draw, so the 2,000-tick series is attributable and carries the new block's early counts; retention and a scale off zero are visible only past 20,000 ticks, F04 is the track's named goal-long carrier (Notes, user 2026-10-15), and the run also carries F03's deferred 40,000-tick ring readings. |
| Mutation gate | `run` | New production logic in `v3-core` (declaration, pattern, container generalization, operator, recruitment case, counter, block), `v3-cli` and `v3-server` whose behavior tests must pin. |
| Slow tests (`make rust-test-slow`) | `run` | The catalog gains an operator and recruitment's `WholeFamily` plan and draws change: the tier pins the recruitment-paths rows those draws move. |

Viability first: `cargo test -p v3-core --test viability` before any other
test.

## Implementation Tasks

- [x] **Commit 1 (the shape and the operator), tests first** (table
      below): invariants 1–8 and 10–13 in the order declaration → compass
      and pattern → container → operator → counter and block → readers,
      server, inspector and petri-visuals; recruitment untouched. Re-pins
      by the catalog draw stream only.
- [x] **Commit 2 (recruitment's pair case), tests first**: invariant 9.
      Re-pins by the recruitment draw and plan only.
- [x] No text-only commit: no existing genome text changes. A later commit
      that moves no pin (inspector, viz, docs) may be split off.
- [x] Reference specs rewritten for two shapes (Inputs table, last row);
      `docs/progress/benchmark-series.json` gains no entry unless a
      definition changes (none is predeclared); `docs/benchmark-artifacts.md`
      untouched.

Tests written first (file → test):

| File | Tests |
| --- | --- |
| `contracts/inputs.rs` | every declared pair's `x`, `y` and `facts` distinct and below the family width; the four layouts of the Layouts row; the moved `*_idx` modules still match the array sizes; every `InputRef.Swap` alternative of a pair-declaring reference declares the same pairs (`every_swap_alternative_keeps_the_pairs`); the `directionPairs.json` fixture equals `direction_pairs()` |
| `creature/genome/cgp/group.rs`, `container_tests.rs`, `tests/group_tests.rs` | `py_equals_normalized_delta_dy` and norm-1 rows; `pair_creation_materializes_12_members_not_16`; an unresolvable or non-pair `ref_idx` reads pattern 0; shift and retarget on a pair group with holes and with a hand-built zero-pattern member (`shift_then_unshift_is_identity` for pairs; every member's relative coordinate preserved); `identical_source_members_on_different_sinks_resolve_different_weights`; detach of a pair member is bitwise `compose`; a last member's removal removes the group; `validate_graph` rejects a bad role, a `pair` out of range and a duplicate `(field, direction)`; thirteen-unit arithmetic; JSON round trip of the shape variant |
| `runtime/cgp/neutral_connection_tests/group.rs` and per pair-declaring family | `a_pair_group_at_offset_0_votes_toward_a_thing_at_each_direction` and `..._offset_4_votes_away_...` through `assemble_perception` on a world holding one food cell, one barrier or one other creature at `Direction::delta(d)`, scale 1: `K[d]` (offset 0) or `K[(d + 4) mod 8]` (offset 4) is the argmax of the kind's eight direction votes, for the nearest and gradient/center pairs and for `NearbyCreatureCore` slot 0; for slots 1–3 a ranked fixture places `k` nearer creatures off the tested direction and the target at rank `k`, with only slot `k`'s pair grouped, so the vote isolates that slot; a pair group equals its twelve ungrouped edges bitwise in I10 position |
| `mutation/graph/tests/group.rs`, `mutation/types/tests.rs` | catalog bijection 29 / 58 / 62 with weight, arm, key, admission and `complexity_effect` assertions; the draw-order tail test of `AddDirectionPairGroup` (flat list, kind, offset); the unit operators over a mixed-shape group list; the engine-level invariant proptest over both shapes with the Δ column; restricted size pressure excludes it; `refinement_never_steps_a_member` on a two-slot pair-group fixture |
| `mutation/graph/recruitment_tests.rs` | the pair case: one group of twelve born-0 members at `offset = d` plus the facts edges into `Drive K`, draw order (family, destination, pair) pinned by a parallel-RNG tail; `SingleChannel` and non-directed destinations unchanged; `:223` on both shapes, regression file re-committed if it reappears; pair-declaring families absent from the `WholeFamily` universe when `can_host_group` is false or a `Drive(K)` sink is absent, ring families present under the F03 rule alone (`ring_admission_is_unchanged_by_the_pair_case`); the added-edge bound holds (15 or 14 for the pair case) |
| `neighborhood/opportunity`, `crates/v3-lab/src/ladder`, `neighborhood/steering.rs`, `drives.rs`, `creature/ring_rules_tests.rs`, `refinement_tests.rs` | `pair_group_member_channels_are_on_the_vote_surface`; a node voting only through pair members is a motor node; `votes_move` through pair members; `node_fanouts` skips pair members; `ring_rules` ignores a pair group; a refinement application lands in the by-key map |
| `neighborhood/input_use/tests.rs`, `crates/v3-lab/src/readings/tests.rs` | `masking_a_pair_reference_zeroes_every_member_read`; ablating a pair-grouped genome leaves size, complexity, groups, memberships and member weights unchanged |
| `crates/v3-cli/src/bench/tracking/tests.rs`, `artifacts` tests, `crates/v3-telemetry/src/metrics/tests.rs`, `crates/v3-server/tests/server.rs` | the `pair_groups` block from a fixture population (reachable and unreachable groups, signs, holes, extinction `null`); `pair_groups` kept by the summary projection; the inheritance witness (in `simulation/actions` tests): a birth with no creation event and a nonzero inherited pair group counts one, a birth whose events created a group counts zero even when a group is nonzero, a rolled-back creation event does not suppress the count, a zero-scale inherited group counts zero; `a_committed_refinement_count_survives_summary_projection`; the by-key family row; `graph_weights` with one entry per member of a pair group (twelve at birth, fewer after a removal); `v3alpha7` |
| `frontend/src/components/inspector/EdgeGroupList.test.tsx`, `inputRefUtils.test.ts`, `petriVizDrift.test.ts` | the pair row and member rows with holes; `DIRECTION_PAIRS` equals `directionPairs.json`; the pair form in the drift fingerprint |

## Verification

- [x] Viability first: `cargo test -p v3-core --test viability` -> 29 passed
      before each commit; the founder-only digests do not move.
- [x] Focused tests: the tests-first table -> `cargo test --workspace
      --no-fail-fast` (v3-core lib 2,164 with `telemetry-seams`, every
      integration binary, v3-cli, v3-server, v3-lab, v3-telemetry) all
      passed at each commit; `cargo clippy --workspace --all-targets -- -D
      warnings` clean; `cd frontend && npx vitest run` -> 98 files, 634
      tests passed. Old and new value of every re-pin, three of them not
      predeclared (fixture seeds), in the readings file.
- [ ] `make rust-test-slow` -> the recruitment-paths rows re-pinned, one
      cause per commit -> result.
- [ ] Mutation gate: fresh `MUTANTS_ITERATE=0 make rust-mutants`, its
      summary line, output path, and every survivor resolved as killed,
      equivalent, or deferred (full list here).
- [ ] Benchmark profiles: summaries at
      `docs/progress/features/t25-f04-direction-pair-groups.json` (gate),
      `...-goal.json`, `...-goal-long.json`; local raw hash/byte count and
      verification time checked, series entry points to the summary, no
      full report staged.
- [ ] `make check` -> result.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: the superior
colliculus, whose sensory layers sit in register with its motor layers so a
stimulus at a retinal place drives a saccade toward that place, and whose
inhibitory routes drive away from it. It reaches creatures through the vote
surface (twelve real edges on the direction sinks, summed where their sinks
sum) and, when recruitment wires the pair, through `Drive K` (the facts
edges) and through mutation (every member addressable by the edge
operators). The founder carries no pair group and no pair-declaring
reference on a direction sink, so every mutation-off reading is identical.

Compute: no shape-resolution cost per visit beyond ordinary edge evaluation
(invariant 10); the predeclared cost is one pattern resolve per pair member
at every construction (two lookups and a modulo) and memory of 16 bytes per
member in the genome plus 4 per member in the runtime table plus one
record; the gate's
`wall_clock_ms_per_creature_tick` is compared against the current epoch
under the ordinary thresholds and a `severe` flag on wall-clock is a defect
to find, not a cost to accept. With mutation on, every per-seed counter may
move through the draw stream (invariant 13) with no predeclared direction;
a `severe` flag on such a counter is a trajectory reading reported to the
user, who decides whether to accept it; the deterministic block changes by
construction (operator keys, the `pair_groups` field) and the gate epoch is
re-pinned in the closing commit.

Goal (2,000 ticks): `mean_genome_size` tracks the F02 rule; the new supply
is `AddDirectionPairGroup` at +13 per application (2 of 58 default-arm
Graph draws, commit 1) and recruitment's pair case, which charges 16 units
where today's whole-family plan charges 7 for an area family (+9) and 15
where it charges 16 for `NearbyCreatureCore` (−1) (commit 2), so a small
rise against F03 is expected and is not a threshold crossing; `pair_groups.groups` above 0 on at least one world by
the last sample is expected (the operator and recruitment both create
them), `groups_nonzero` has no predeclared direction (a group is born silent
and the series is short); births, gestation, persistence, clade count and
entropy have no predeclared direction. A `severe` flag or threshold crossing
is reported with its reading and goes to the user.

Goal-long (40,000 ticks, the three standard worlds): the success-criterion
reading, retention made operational without group identity, has two
parts on at least one world: the inheritance witness
`inherited_nonzero_births_total` rises between tick 30,000 and 40,000 (at
least one birth in the window at which a child inherited a pair group with
its scale off zero), and `pair_groups.groups_nonzero ≥ 1` on reachable
nodes at every persistence sample from tick 30,000 through 40,000 (101
consecutive samples: prevalence, not identity, since a group can be created
and scaled within one birth and replaced between samples); the readings
record the per-sample `groups_nonzero`, `carriers`, `members` and
`inherited_nonzero_births_total` series over the window, the block at tick
`T − 10,000` and `T` beside `add_applied_total` between them, the family × kind × offset rows with signs, `members` per
group, `mutation_outcome_summary.detrimental_total`
per `carriers_observed_total` against F03's goal value (no predeclared
direction; reported), `refinement_applications_by_key` and `groups_removed`.
F03's deferred ring readings are taken on the same run: `ring_group_carriers`
of the order of F02's `projection_carriers` on at least one world,
`mean_ring_group_members` below 8 on at least one world, `graph_edges_detached`
and `groups_removed` above 0 on every world that evolves groups. Extinction
on a world is a persistence reading with no predeclared direction, reported
to the user; the criterion needs one world.

Predeclared re-pins, one cause per commit, each restorable by reverting the
named value:

| Commit | Pins | Mechanism | Restoring check |
| --- | --- | --- | --- |
| 1 | the catalog bijection test (28 / 56 / 61 → 29 / 58 / 62); `applied_trajectory.rs` four digests; `baseline_worlds.rs` digest; `mutation/engine/recruitment_tests.rs` (the supply pin its comment at `:49` explains); `mutation/graph/recruitment_tests.rs:223` and its regression file; the slow-tier and `--lib` recruitment-paths rows; the v3-cli trajectory fixtures (re-searched with every assertion and oracle unchanged, a weakened assertion being a requirement change for the spec owner); the gate and world-set deterministic blocks; the fixture seeds `reproduce_updates_domain_and_operator_mutation_stats` (100 → 101), `tick/tests/gestation.rs` lowered-cap identity (2733 → 3249) and `window_seam.rs` VM founders (21 → 24, `telemetry-seams`), assertions unchanged; the recruitment-paths known-deleting lineage (arm 12 batch 22 lineage 0 → batch 1 lineage 3, the first `Deleted` lineage of arm 12 in batch, lineage order, oracle unchanged; ratified by the spec owner 2026-10-16) | draw stream: the new weighted operator remaps every default-arm Graph draw (invariant 13); recruitment untouched | simulation state: with mutation off every trajectory of every genome is byte-identical across the commit (the founder-only viability siblings, and an `applied_trajectory.rs` sampled run with `mutation.per_unit_rate` 0); with mutation on, reverting the catalog entry alone restores every moved pin to its merge-base value (the implementer records old and new values per pin in the readings); a pin that moves under mutation off, or that hashes genome text and moves with mutation off, is reported to the spec owner. Reports: the `pair_groups` field and the `mutation_supply` map are expected additions, not pin movements |
| 2 | the same pin set, re-pinned again | draw stream: recruitment's pair draw and plan (invariant 9) | the same mutation-off check; with mutation on, reverting the pair case alone restores every pin to its commit-1 value (old and new values recorded per pin) |
| 1 | `v3-server` detail-response tests and frontend fixtures naming the protocol version | `v3alpha7` | reverting the constant restores every value |

**Measured verdict.** Pending.

- Summaries of the declared profiles:
  [gate](../../progress/features/t25-f04-direction-pair-groups.json),
  [goal](../../progress/features/t25-f04-direction-pair-groups-goal.json),
  [goal-long](../../progress/features/t25-f04-direction-pair-groups-goal-long.json).
- Full readings: [`docs/progress/readings/t25-f04.md`](../../progress/readings/t25-f04.md).

## Success Criteria

- [ ] A `DirectionPair` group is born as twelve silent members with one
      shared scale on any declared pair of the four families; at scale 1 it
      votes toward (offset 0) or away from (offset 4) a thing at each of the
      eight directions through the real reducers; the invariant proptest,
      the relocation identity and the bitwise oracle pass; the catalog is
      29 / 58 / 62. (Tests-first table.)
- [ ] Recruitment's `WholeFamily` pair case wires one group and the facts
      edges into `Drive K` in the pinned draw order; every other plan is
      unchanged. (Recruitment tests; the slow-tier rows.)
- [ ] Every reader in the readers table meets its expectation; the
      `pair_groups` block is on every persistence sample and kept;
      `refinement_applications_by_key` is exported; the inspector and
      petri-visuals show a pair group. (Reader tests; the goal summary.)
- [ ] On at least one standard goal world `inherited_nonzero_births_total`
      rises between tick 30,000 and 40,000 and the `pair_groups` series
      shows `groups_nonzero ≥ 1` on reachable nodes at every sample in that
      window (the operational retention reading of the Performance
      section); F03's deferred ring readings are recorded. (Goal-long
      summary and readings.)
- [ ] Track row checked, spec Complete, `make check` green on main.

## Notes for AI Agents

- Decision: (user, 2026-10-13) only geometry is built in: the compass
  pattern and the `(x, y)` pairs declared in `contracts::inputs`; never a
  computation; group creation never randomizes the pattern.
- Decision: (user, 2026-10-15) goal-long runs at most once or twice per
  track; F04 is the T25 carrier after F02 and takes F03's deferred
  40,000-tick ring readings on its run.
