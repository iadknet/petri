# T25.F05 — The NearbyCreature Family

**Status**: In Progress
**Last updated**: 2026-10-18
**Feature**: T25.F05
**Track**: [T25 — Edge Groups and Per-Edge Learning](../../roadmaps/t25-edge-groups-and-learning.md)

## Goal

One sense of another creature, as an animal reads a neighbor's place,
condition and kinship in one glance: `NearbyCreatureCore`,
`NearbyCreatureVitals` and `NearbyCreatureIdentity` become one
`WorldInputKey::NearbyCreature` family, 4 ranked slots × 9 fields = 36
sub-values, declared once in `contracts::inputs`, so a slot's position,
vitals and identity arrive from one declaration. Slot `k` occupies channels
`9k..9k+9` in the order `present, rel_x, rel_y, dist, energy_ratio,
reproduce_ready, kin_affinity, lineage_match, phenotype_similarity`; every
field keeps its F04 value. The family declares one direction pair per slot
whose facts are the slot's seven other fields, so recruitment wires a whole
neighbor (its offset to the eight direction votes, its facts to the drive)
from one draw. The sampler keeps 27 buckets with indices 10–12 all drawing
the family; recruitment's family universe lists it three times; the `Slots`
refinement key, every reader, the inspector, the lab and the telemetry keys
read the merged layout. No sensor value, ranking rule or normalization
changes.

## Non-Goals

- The reducer's ranking order, visibility contract and field normalization
  (sensor spec §4–6): unchanged.
- The area families' pair case, `AddDirectionPairGroup`, the group operators
  and their weights (`ALL` 29, default arm 58, `all()` 62): unchanged.
- Changing the sampler's bucket count (D13 keeps 27) or any other bucket.
- The `Slots` refinement key's keep-or-delete decision (F08 reads the
  readings this feature records).
- Reading stored genomes, reports or lab summaries written before this
  feature; no conversion of the old key names.
- A new telemetry family, counter or dashboard panel.
- The VM backend; founder rewiring; the design note's D11 non-goals.

## Inputs and Invariants

| Input | Where | What F05 takes from it |
| --- | --- | --- |
| Track row, success criterion 5 and Notes | `docs/roadmaps/t25-edge-groups-and-learning.md` | The goal; "the three creature families are one `NearbyCreature` family declared once in `contracts::inputs`"; the user decision of 2026-10-13 that the families merge; the verification line (gate + goal, mutation gate and slow tests run); F08 decides the `Slots` disposition from the refinement-by-key readings F04 and F05 record. |
| Design note | `docs/strategy/edge-groups-and-learning-design-2026-10-13.md` §4 (recruitment emits groups; "one slot everywhere keeps the 30-edge bound"; D13), §8 F05 contract (scope, predeclared re-pins, tests first, readings) | The layout, the sampler rule, the recruitment rule, the re-pin list; the track row is authoritative where the two differ. |
| F04 outputs | `docs/specs/roadmap/t25-f04-direction-pair-groups.md` invariants 1 (declaration, pairs in `contracts::inputs`), 3–6 (pair groups), 9 (recruitment's pair case: draw order family, destination, pair; preflight), 11 (refinement by key), 12 (`pair_groups` block) | Everything the merged family must keep working; `direction_pairs()` as the one pair declaration. |
| Declaration | `crates/v3-core/src/contracts/inputs.rs:39-61` (`core_idx`, `vitals_idx`, `identity_idx`), `:130-152` (`CORE_PAIRS`), `:155-176` (the three variants), `:228-236` (`direction_pairs`), `:255-268` (`compound_width` 16/8/12) | The site that becomes one declaration. |
| Sensors | `sensors/perception.rs:7` (`NEARBY_SLOTS`), `:23-58` (`PerceptionSnapshot` three arrays), `:67-77` (`is_extended_key`), `:132-134` (`resolve_compound`); `reducers.rs:307-421` (`rank_nearby_creatures`, `fill_nearby_banks` returning a 3-tuple); `reads.rs:17-40` (`UNTYPED_BITS` 7, bits 4–6) | The values and their one array. |
| Sampler and recruitment | `mutation/sampling.rs:26-57` (27 buckets, 10–12 the three families), tests `:110-200` (`catalog_draw`); `mutation/graph/recruitment.rs:42-50` (`pair_count`: a draw only on a directed sink), `:62-96` (`append`), `:150-233` (`plan`: `WholeFamily` expands every field), `:235-258` (`families` from `input_reference_universe`), `:277-320` (`recruit`: family, channel, destination, pair); `recruitment_tests.rs:200-260` (the 1..=30 bound), `:619-657` (draw order); `mutation/input_ref/mod.rs:42-63` (`input_reference_universe` from `world_input_key_universe`, once per key) | The slot draw, the weighting site, the bound. |
| Refinement | `mutation/graph/refinement.rs:17-62` (`SlotKey`, `correspondence`: `width = compound_width() / 4`, `field = channel % width`, `slot = channel / width`); `mutation/types` `RefinementKey {family, field}` | The re-key: one arm, nine fields. |
| Readers | `creature/sensor_census.rs:246-262` (`world_input_key_universe`: seven untyped keys), `:266` (`world_input_key_label`); `neighborhood/input_use/catalog.rs:15-45` (`Family`, `input-use-v1`), `input_use/mod.rs:48`; `neighborhood/mutation_effects/contexts.rs:18-34` (`AUTHORED_CONTEXTS` 24, `CHANNEL_GROUPS` 8, three nearby groups), `:46-48`, `:70-84` (`set_group`), `mutation_effects/mod.rs:44-45` (`VERSION`, `COVERAGE_VERSION`); `mutation/types/mod.rs:157-173` (`MutationAddedNodeInputClass::Neighbor`); `creature/genome/mesh_annotations.rs:170-185` (`MeshReadClass::Neighbor`); `crates/v3-lab/src/readings.rs:465-483` (`census_covers`), `v3-lab/src/summary.rs:21` (`GENOME_FORMAT` 2); `runtime/trace/domain.rs:190-205` (`PerceptionDebugSnapshot`); `crates/v3-telemetry/src/windows.rs:471-473` (`petri.nearby_*` span attributes), `metrics.rs:1020-1031` (sensor gauges keyed by `world_input_key_label`), `metrics/tests.rs:168-172, :505-530, :908`, `tests.rs:1339`; `crates/v3-cli/tests/telemetry.rs:619-631`; `crates/v3-server/src/transport/sample_protocol.rs:93-100`, `sample_assembler.rs:55`, `types.rs:5` (`v3alpha7`) | Each reader's post-change expectation (table below). |
| Frontend | `frontend/src/components/inspector/inputRefUtils.ts:38-62` (labels, descriptions), `:91-107` (`DIRECTION_PAIRS`), `:155-176` (`NEARBY_FIELDS`, `NEARBY_SLOTS`), `:390`; `directionPairs.json` (pinned both sides by `the_frontend_fixture_equals_the_declared_pairs` and a frontend test); `inputRefUtils.test.ts:146-177`; `types/trace.ts:19-21` | The hand mirror, one family. |
| Pins | `tests/viability.rs` founder-only digests (must not move: the founder declares no creature family), `tests/baseline_worlds.rs`, `tests/applied_trajectory.rs`, `mutation/engine/recruitment_tests.rs` supply pin, `mutation/graph/recruitment_tests.rs:223` and its regression file, the slow-tier and `--lib` recruitment-paths rows, the v3-cli trajectory fixtures, the gate and world-set deterministic blocks, `docs/progress/benchmark-series.json` (gate and goal epochs at F04) | The predeclared re-pins (Performance section). |
| Reference specs | `docs/reference/v3-sensor-spec.md` §1 (`:54-56`), §3.1 (`:106-108`), §5.4–5.7, §6.4–6.5; `v3-mutation-spec.md:577-588` (pair case), `:637-650` (sampler table); `v3-server-api-protocol-spec.md` §2 (`:53-70`) and the perception payload | Every sentence that names a creature family, rewritten for one. |

Invariants (the layout pin and the partition test assert 1–3; the
recruitment proptest asserts 6; `validate_graph` is unchanged):

| # | Invariant |
| --- | --- |
| 1 | **Declaration.** `WorldInputKey::NearbyCreature` (`as_key` `"NearbyCreature"`, JSON tag `"NearbyCreature"`, `compound_width` 36) replaces the three variants, which are deleted with `core_idx`, `vitals_idx` and `identity_idx`. `contracts::inputs::nearby_idx` declares `SLOTS = 4`, `FIELDS_PER_SLOT = 9`, `PRESENT 0`, `REL_X 1`, `REL_Y 2`, `DIST 3`, `ENERGY_RATIO 4`, `REPRODUCE_READY 5`, `KIN_AFFINITY 6`, `LINEAGE_MATCH 7`, `PHENOTYPE_SIMILARITY 8`; slot `k` field `f` is channel `9k + f`; `sensors::perception` re-exports the module and `NEARBY_SLOTS` is `nearby_idx::SLOTS`. Every reader that enumerated the three keys (readers table) carries one arm. |
| 2 | **Layout pin.** `merged_nearby_creature_equals_core_vitals_identity_per_slot` is written first, while the three banks exist, over the real reducer on a ranked fixture (at least two visible creatures with distinct vitals and identity): for every slot `k`, `nearby[9k..9k+9] == core[4k..4k+4] ++ vitals[2k..2k+2] ++ identity[3k..3k+3]`. After the merge it is kept as the pin of the same values at the `nearby_idx` constants, so no field's value, normalization or ranking changes (sensor spec §5.4–5.6, §6.4–6.5 preserved verbatim under one heading). |
| 3 | **Pairs.** `direction_pairs()` for `NearbyCreature` declares four pairs, slot `k`: `x = 9k + 1`, `y = 9k + 2`, `facts = [9k, 9k + 3, 9k + 4, 9k + 5, 9k + 6, 9k + 7, 9k + 8]` in that order (presence, distance, energy ratio, reproduce-ready, kin affinity, lineage match, phenotype similarity). The four pairs partition the layout: every channel belongs to exactly one pair's `{x, y} ∪ facts`; `WorldInputKey::slot_pairs(self) -> bool` is true for `NearbyCreature` only, and a test checks the partition property against `direction_pairs()` and `compound_width()` for every key (`true` exactly where the pairs partition). Area families and F04 invariant 1's distinctness rule are unchanged; `directionPairs.json` carries the merged entry. |
| 4 | **Snapshot and reads.** `PerceptionSnapshot.nearby: [f32; 36]` replaces `nearby_core`, `nearby_vitals`, `nearby_identity`; `fill_nearby_banks` returns the one array; `resolve_compound`, `is_extended_key`, `compound_width_matches_array_sizes` carry one arm; `PerceptionDebugSnapshot.nearby`, the server's `PerceptionDebugSnapshotPayload.nearby` and `frontend/src/types/trace.ts` follow; a creature window's tick span carries one `petri.nearby` attribute of 36 doubles in place of the three. `WorldReads`: `NearbyCreature` is bit 4, `UNTYPED_BITS` 5, typed bits start at 5; every valid key keeps a bit. |
| 5 | **Sampler (D13).** `random_input_reference_for_food_types` keeps 27 buckets; indices 10, 11 and 12 each return `World(NearbyCreature)` with one draw; every other index is unchanged (`typed_sampler_draws_the_merged_family_at_three_indices`). |
| 6 | **Recruitment, slot rule.** Under `WholeFamily`, a plan for a family with `slot_pairs()` draws one slot `gen_range(0..direction_pairs().len())` immediately after the destination on every destination (F04's pair draw, now taken whatever the destination): on a directed sink `K(d)` the F04 pair case with pair = slot, one silent `DirectionPair` group at offset `d` (twelve members) plus seven born-0 facts edges into `Drive K` in `facts` order (19 edges, Δ`genome_size` +20 plus one for a new declaration); on any other destination nine born-0 plain edges, the slot's channels `9k..9k+9` ascending, into that sink (Δ +9 plus the declaration), never the family's 36. Area families keep F04 invariant 9 verbatim (pair draw only on a directed sink, whole-family edges elsewhere); `SingleChannel` keeps family, channel (`0..36`), destination; the preflight is F04's (`slot_pairs` families leave the `WholeFamily` universe when `hosts_pair_plan` is false); the `1..=30` added-edge bound holds; the draw-order replay test covers the slot draw on an undirected destination. |
| 7 | **Recruitment universe.** `Admission::families` lists `World(NearbyCreature)` three times under both arms, one entry per family it replaces, so on a graph that admits the family its draw share equals the three families' combined share today (facet 7: the sampler and the recruitment universe are separate draws); every other entry appears once; `input_reference_universe` (the `Swap` partition, where the family is alone in its kind at width 36) lists it once. The admissibility differs where F04's did not: under `WholeFamily` on a graph where `hosts_pair_plan` is false (a direction or drive sink absent, or `u16::MAX` groups), all three entries leave the universe, where F04 kept `Vitals` and `Identity` as plain-edge plans; that supply loss is predeclared and confined to such graphs (the founder and every descendant that keeps its 30 vote sinks admit the family). Pinned by `the_merged_family_is_three_entries_of_the_recruitment_universe` (full sinks) and the partial-sink and exhausted-capacity cases of `pair_families_leave_the_whole_family_universe_without_a_host_or_a_drive`, which asserts no entry remains. |
| 8 | **Refinement by key.** `correspondence` keys the merged family only: `field = channel mod 9`, `slot = channel div 9`, written with `nearby_idx::FIELDS_PER_SLOT` and `SLOTS`, never a literal; `RefinementKey {family: NearbyCreature, field: 0..9}`; `petri.run.refinement_applications_by_key` attributes `petri.world_input = "NearbyCreature"`, `petri.field` 0–8; bench summary keys `"NearbyCreature:<field>"`. The `Slots` fixtures are rewritten to the merged channels. |
| 9 | **Catalog, classes, contexts.** `input_use::catalog::Family::NearbyCreature` once at the sampler's position (17 named families), `input_use::VERSION` `input-use-v2`; `MutationAddedNodeInputClass::Neighbor`, `MeshReadClass::Neighbor`, `census_covers`, `world_input_key_universe` (five untyped keys) and `world_input_key_label` carry one arm. `mutation_effects::contexts`: `CHANNEL_GROUPS` has six entries, the three nearby groups replaced by `"nearby creatures"` with `NEARBY_SLOT: [Range; 9] = [Binary, Signed, Signed, Unit, Unit, Binary, Unit, Binary, Unit]` cycled over the 36 values; context `i` sets group `i mod 6`; `AUTHORED_CONTEXTS` stays 24 (four per group); `COVERAGE_VERSION` `neighborhood-coverage-v2` (the module's own rule) and `mutation_effects::VERSION` `mutation-effects-v2` (its family keys change). |
| 10 | **Accounting.** One unit per edge and per record (F02); the founder stays 80; no operator weight or catalog position changes. |
| 11 | **Identity and determinism.** The genome text of every genome carrying a creature reference changes (tag and channels); the lab's `GENOME_FORMAT` becomes 3; `PROTOCOL_VERSION` becomes `v3alpha8` with a §2 history line (the `nearby` payload, the genome key). Draw streams change for every mutation-on run (the sampler's result at 11–12, the slot draw, the family universe); with mutation off every trajectory is byte-identical and the founder-only viability digests do not move. |
| 12 | **Telemetry keys.** `petri.tick.sensor_{declared,wired,read}`, `petri.run.mutation_added_node_world_inputs_by_operator` and `petri.run.refinement_applications_by_key` carry `petri.world_input = "NearbyCreature"`; no family is added or removed; `scripts/telemetry-dashboards-check` and the dashboards name no creature family and are unchanged. |

Readers, with their post-change expectations:

| Reader | After F05 |
| --- | --- |
| `ring_rules`, `drives`, `pair_groups`, `EdgeGroupList` | unchanged definitions; a pair row reads `NearbyCreature.rel_x/rel_y` through `direction_pairs()` |
| `sensor_census`, `cgp_mesh_annotations`, consumed references, discovery | one key where three were counted; a creature that declared any of the three declares `NearbyCreature` |
| `input_use` (`input-use-v2`), lab `readings` | `Family::NearbyCreature`, channels 0–35, the slot and field from `nearby_idx` |
| `mutation_effects` (`mutation-effects-v2`, `neighborhood-coverage-v2`) | six channel groups, four authored contexts each |
| Telemetry | key values only (invariant 12); window attribute `petri.nearby`; `metrics/tests.rs` expected keys and the `refinement_applications_by_key` fixture over nine fields; `crates/v3-cli/tests/telemetry.rs` absent-attribute list |
| Server | `PerceptionDebugSnapshotPayload.nearby: [f32; 36]`; `v3alpha8` |
| Inspector | label `Nearby`, description "Position, vitals and relatedness of the 4 nearest visible creatures."; `NEARBY_FIELDS.NearbyCreature` nine fields (`present, dx, dy, dist, energy, ready, kin, lineage, similar`); `formatInputRefWithSubIndex({World: "NearbyCreature"}, 13)` → `Nearest2.energy`; `DIRECTION_PAIRS.NearbyCreature` the four slot pairs. The fixture test pins `DIRECTION_PAIRS` only, so `inputRefUtils.test.ts` gains a parity test over the nine ordered field names for all four ranks (channels 0–35, 35 → `Nearest4.similar`) and the boundary (36 falls back to the raw index) |
| Lab | `GENOME_FORMAT` 3; a stored genome file at format 2 is refused by the load guard (`v3-lab/src/lib.rs:62`) as format 1 is today |
| Reference specs | sensor spec §1, §3.1, §5.4 (one heading, the nine fields, the slot formula), §5.7 (the pair row: slot `k` `(rel_x, rel_y)` `9k+1`, `9k+2`, facts `9k, 9k+3..9k+8`), §6.4–6.5 under the one family; mutation spec sampler table row `9 to 13` and the pair case (19 edges for a creature slot, the slot drawn on every destination); protocol spec §2 and the perception payload |

Options weighed and the preparatory refactor audit:
[the readings file](../../progress/readings/t25-f05.md).

## Telemetry

No signal is added. A person looking at a run sees the merged family where
three keys stood:

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `petri.tick.sensor_declared` / `sensor_wired` / `sensor_read` | gauge (sampled) | the sensor census over `NearbyCreature` | `petri.world_input = "NearbyCreature"` (one series where three were) |
| `petri.run.refinement_applications_by_key` | counter | committed `RefineHeritableStructure` events on the merged `Slots` sets | `petri.world_input = "NearbyCreature"`, `petri.field` 0–8 |
| `petri.run.mutation_added_node_world_inputs_by_operator` | counter | the world references of a node added by `Topology.AddNode`, `Topology.SpliceNode` or `Graph.AddInternalGraphNode` (`engine/mod.rs:477`; recruitment and `InputRef.Add` are not counted, in production or by this feature) | `petri.world_input = "NearbyCreature"` |
| `petri.nearby` on a creature window's tick spans (`windows.rs` `tick_attributes`) | span attribute | `PerceptionDebugSnapshot.nearby` | 36 doubles in place of `petri.nearby_core`, `petri.nearby_vitals`, `petri.nearby_identity`; absent, never zero, when the creature read no extended perception |

No new mechanism switch or config value; `run.started` is unchanged.

## Verification Plan

**Question.** F05 claims that one `NearbyCreature` family carries the three
families' values unchanged and keeps their place in mutation and
recruitment; a merged layout whose values differ from the three banks, or a
goal reading where the merged family's declared or recruited share falls far
below the three old families' combined share or an indicator crosses its
threshold, would show it wrong.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate + goal` | A production behavior change in the sampler, recruitment and the sensor layout whose question (share and supply held, no indicator crossing) is answered at 2,000 ticks; the track Notes declare F05 `gate + goal`, and T25's goal-long carriers (F02, F04) are spent. |
| Mutation gate | `run` | New production logic in `v3-core` (the slot rule, the weighting, the re-keyed refinement, the merged reducer) whose behavior tests must pin. |
| Slow tests (`make rust-test-slow`) | `run` | The input catalog, the sampler and recruitment change, which the tier's recruitment-paths rows pin. |

## Implementation Tasks

Two behavior commits so every moved pin has one cause.

Commit 1, the merge:

- [x] Tests first, while the three families exist: the layout pin
      (invariant 2) over the real reducer; then the sampler index test
      (invariant 5), the partition test (invariant 3), the draw-order
      replay extended with the slot draw on an undirected destination
      (invariant 6), the `Slots` fixtures at `field = channel mod 9`
      (invariant 8), the frontend fixture test and the field-name parity
      test (readers table, Inspector row).
- [x] `contracts::inputs`: `NearbyCreature`, `nearby_idx`, the four slot
      pairs, `slot_pairs`, `compound_width` 36; delete the three variants
      and index modules; `sensors::reducers` writes against `nearby_idx`.
- [x] Sensors: `PerceptionSnapshot.nearby`, `fill_nearby_banks`,
      `resolve_compound`, `is_extended_key`, `WorldReads` bits.
- [x] Sampler (invariant 5); recruitment's slot rule (invariant 6);
      refinement re-key (invariant 8); catalog, classes, census universe and
      label, mutation-effects contexts and versions, lab `census_covers` and
      `GENOME_FORMAT` 3 (invariants 9, 11).
- [x] Trace domain, telemetry window attribute and metrics tests, server
      payload and `v3alpha8`, frontend (`inputRefUtils.ts`,
      `directionPairs.json`, `types/trace.ts`, tests), the v3-cli telemetry
      test's attribute list.
- [x] Reference specs (readers table, last row).
- [x] Re-pin the predeclared pins moved by this commit; record each in the
      readings file's re-pin table with its mechanism.

Commit 2, the recruitment universe:

- [x] `Admission::families` lists the family three times (invariant 7) with
      its full-sink test and the partial-sink and exhausted-capacity cases;
      re-pin the pins moved by this commit.

Closure:

- [ ] Benchmark profiles, mutation gate and slow tier as declared; readings:
      the merged family's wired prevalence and pair-group count against
      F04's stored goal summary (Performance section), and
      `refinement_applications_by_key` by field for F08.

## Verification

- [x] Focused tests: `cargo test -p v3-core --test viability` first (30
      passed, founder-only digests held), `cargo test -p v3-core` (the
      layout pin, partition, sampler, draw order, universe weighting,
      refinement fixtures), `cargo test -p v3-telemetry`,
      `cargo test -p v3-server`, `cargo test -p v3-lab`,
      `cargo test -p v3-cli`, frontend `npm test`, clippy, fmt and
      `cargo check` clean on `842bf12f` -> results and the tree each ran on
      in [`docs/progress/readings/t25-f05.md`](../../progress/readings/t25-f05.md).
- [ ] `make check` -> exit 0 on the final code.
- [x] `make rust-test-slow` -> exit 0 on the final code, re-pinned rows
      recorded in the readings file.
- [ ] Mutation gate: fresh `MUTANTS_ITERATE=0 make rust-mutants`, its
      summary line, output path, and every survivor resolved as killed,
      equivalent, or deferred (the full list stays here).
- [ ] Benchmark profiles: summaries at
      `docs/progress/features/t25-f05-nearby-creature-family.json` (gate)
      and `t25-f05-nearby-creature-family-goal.json`, local raw hash/byte
      count and verification time checked, series entry points to the
      summary, no new full report staged.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: a social
animal reads a conspecific's place, condition and kinship in one glance, the
one percept of a neighbor from which approach, avoidance and recognition
follow. It reaches creatures through the same 36 sensor values at new
channels, through recruitment (a whole neighbor wired from one draw: its
offset to the eight direction votes and its seven facts to the drive, or its
nine fields to another sink) and through mutation (every channel addressable
by the edge operators). The founder declares no creature family, so every
mutation-off reading is identical.

Compute: the reducer writes the same 36 values into one array instead of
three, the snapshot holds the same 144 bytes, and the sink loop reads one
table entry per edge as before, so no per-tick cost is predeclared; the
gate's `wall_clock_ms_per_creature_tick` is compared against the current
epoch under the ordinary thresholds, and a `severe` flag on wall-clock is a
defect to find, not a cost to accept. With mutation on, every per-seed
counter may move through the draw stream (invariant 11) with no predeclared
direction; a `severe` flag on such a counter is a trajectory reading reported
to the user, who decides whether to accept it; the deterministic blocks
change by construction and the gate epoch is re-pinned in the closing
commit.

Goal (2,000 ticks): supply moves both ways and has no predeclared direction
for `mean_genome_size` (edges and record, the one-unit declaration aside):
a creature-slot pair plan charges 20 where F04 charged 15 for a
`NearbyCreatureCore` slot (and 8 or 12 plain edges for `Vitals` or
`Identity` on a directed sink); a whole-family plan into another sink
charges 9 where the three families charged 16, 8 and 12. Recruitment by family
is not recorded in production (`engine/mod.rs:302` tracks it under
`cfg!(test)` only) and this feature adds no counter; the design note's
"declaration and recruitment share" readings are taken from two blocks the
goal summary already stores per persistence sample, against the same
blocks in F04's stored goal summary
(`docs/progress/features/t25-f04-direction-pair-groups-goal.json`), per
world at the last retained sample: the wired prevalence of the merged key
(`sensor_census.world_inputs` row `creatures` over the sample's living
population; F04's three rows bound its any-of-three prevalence below by the
largest single row) and the merged family's pair-group count
(`pair_groups.rows` with `family = "NearbyCreature"`, `count` summed over
kinds and offsets, against F04's `NearbyCreatureCore` rows: both routes
that create a creature pair group). The sampler share (3 of 27) and the
universe entries (three) are unchanged, so both are expected comparable.
The floor is read only on worlds whose F04 baseline is positive (F04's
seed 22 ends with zero creature pair groups, so a zero baseline cannot be
undercut): it is a defect to find, not a reading to accept, when on every
world with a positive baseline the wired prevalence is below half of F04's
largest single-row prevalence, or on every world with a positive baseline
the pair-group count is below half of F04's; a world whose baseline is
zero is reported with its F05 value and takes no part in the floor. `refinement_applications_by_key` is recorded by
field for F08. Births, gestation, persistence, clade count and entropy have no
predeclared direction; a `severe` flag or threshold crossing is reported
with its reading and goes to the user. The goal epoch is re-pinned in the
closing commit.

Predeclared re-pins, one cause per commit: commit 1 moves the sampler index
tests, every mutation-on digest (`baseline_worlds`, `applied_trajectory`,
the v3-cli trajectory fixtures, the gate and world-set deterministic
blocks), the recruitment supply pin and the `recruitment_tests.rs:223`
regression, the slow-tier and `--lib` recruitment-paths rows, the genome
identity of every genome with a creature reference, `GENOME_FORMAT`,
`PROTOCOL_VERSION` and the telemetry key values; commit 2 moves the
mutation-on digests and the recruitment rows again. The founder-only
viability digests do not move under either.

**Measured verdict.** Pending.

- Summaries of the declared profiles:
  [gate](../../progress/features/t25-f05-nearby-creature-family.json),
  [goal](../../progress/features/t25-f05-nearby-creature-family-goal.json).
- Full readings:
  [`docs/progress/readings/t25-f05.md`](../../progress/readings/t25-f05.md).

## Success Criteria

- [ ] `WorldInputKey::NearbyCreature` is the only creature family in
      `contracts::inputs`; the three variants and their index modules are
      gone from every crate, the frontend and the reference specs.
- [ ] The layout pin holds: every slot's nine values equal the F04 core,
      vitals and identity values at the `nearby_idx` constants.
- [ ] The sampler's indices 10–12 draw the family with one draw each;
      recruitment lists it three times, draws one slot after the destination
      on every `WholeFamily` destination, and never adds more than 30 edges.
- [ ] `Slots` refinement keys on nine fields and the goal readings carry
      `NearbyCreature:0..8` for F08.
- [ ] Gate and goal run with no unaccepted `severe` flag; the merged family's
      wired prevalence and pair-group count are recorded per world against
      F04's stored goal summary, zero-baseline worlds reported separately,
      and on at least one positive-baseline world each reading holds half
      its baseline (the predeclared floor).
- [ ] Mutation gate and slow tier run on the final code; `make check` exits 0.

## Notes for AI Agents

- Decision: (user, 2026-10-13) the three creature families merge into one
  `NearbyCreature` family; the sampler keeps 27 buckets with indices 10–12
  drawing it (D13); recruitment takes one slot everywhere.
- Decision: (spec owner, 2026-10-18) the family's seven non-pair fields are
  the slot's facts, so recruitment's pair case wires a whole neighbor from
  one draw (19 edges); the slot draw is F04's pair draw taken on every
  `WholeFamily` destination, so no earlier draw moves.
- Deferred: the merged family's long-run reading (whether a whole-neighbor
  recruitment is retained past 2,000 ticks) to the next goal-long carrier;
  T25's carriers (F02, F04) are spent, so a later track's carrier takes it.
- Deferred: the `Slots` refinement key's keep-or-delete decision to F08,
  from the `refinement_applications_by_key` readings this feature records.
