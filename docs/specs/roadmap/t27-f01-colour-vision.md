# T27.F01 — Colour Vision

**Status**: Complete
**Last updated**: 2026-10-21
**Feature**: T27.F01
**Track**: [T27 — Social Signals and Exchange](../../roadmaps/t27-social-signals-and-exchange.md)

## Goal

Colour vision: an animal reads a neighbour's markings. A creature whose brain
reads the `NearbyCreature` family sees the red, green and blue of each of the
four nearest visible creatures, and any creature can sense its own three
channels as self inputs, so a visible marking can come to mean kin, rival or
prey through evolved computation. The two built-in identity comparisons the
family carried, `LINEAGE_MATCH` (an oracle on the founder index) and
`PHENOTYPE_SIMILARITY` (a comparison of hidden colour genes), are retired;
`KIN_AFFINITY` stays as the master's one sensed exception (self-referent
matching). The feature also lands the single nearby field table with a
Rust-written fixture the frontend imports, the input-bucket table with
sampler/universe agreement coverage, and disposes of the `PerceptionSnapshot`
256-byte test on the gate's measured tick time.

## Non-Goals

- A colour-distance ring (design note §3 A2, rejected) or a retina (A3,
  deferred): raw RGB leaves every comparison to evolution.
- Any change to the heritable colour itself: founder channels, the
  HSL-to-RGB mapping, `PhenotypeConfig` or when it mutates.
- The display-colour coupling of signals (B1, declined); held signals are
  T27.F02.
- A new energy cost: a wired input already costs one genome unit (design
  note §3 A, "Cost").
- Any change to `kin_tag`, `KIN_AFFINITY`, `REPRODUCE_READY` or `lineage_id`
  (which stays observation-only telemetry).
- A sensor-use census row for the self inputs: `AgeTicks` has none, and the
  goal profile's `input_use` block already reports every declared family and
  channel per evaluated cohort (a family no parent declares has no row,
  under the "absent, never zero" convention).
- Touching `CreatureState` or its constructors (T26.F01 reorganizes them in
  the same wave).

## Inputs and Invariants

| Input | Where | What it fixes |
| --- | --- | --- |
| Roadmap row and Notes | `docs/roadmaps/t27-social-signals-and-exchange.md` F01 row; Notes: user decisions (2026-10-20), "F01 retires two identity fields", "RGB is derived ... when perception is assembled", "F01 also lands ...", "Verification: F01 gate + goal" | Scope, the ruling, the three folded refactors, the profiles. |
| Design note | `docs/strategy/interaction-features-design-2026-10-20.md` §2 (colour, NearbyCreature, the 256 B test), §3 A (options, analog, cost, recorded expectation), §4 (what folds into A), §7 decisions 1 and 4 | The chosen shape and the user's rulings. |
| Design review | `docs/progress/readings/t26-t28-design-review.md` round 2 finding 5 (bucket table into F01), round 5 finding 2 (option B) | Why the bucket table is here and what is retired. |
| Options and audit | [`docs/progress/readings/t27-f01.md`](../../progress/readings/t27-f01.md) | The alternatives weighed and the refactor table behind the `Decision:` notes. |
| Colour today | `creature/state.rs` (`phenotype_channels: [u8; 6]`), `mutation/phenotype.rs:15-79` (`hsl_to_rgb`, `channels_to_rgb`), `simulation/seeding.rs:17` (every founder is `[204, 61, 61]`), `actions/reproduction.rs:517-528` (a channel steps by `channel_step` 1 only at a birth whose brain mutated) | The colour a creature sees: a slow heritable label that diverges by about 1/255 per mutated birth. |
| Declaration | `contracts/inputs.rs:40-57` (`nearby_idx`), `:124-163` (facts, pairs), `:166-190` (`WorldInputKey`), `:196-210` (`universe`), `:308-316` (`StaticIntrospectionKey`), tests `:388-640` | The one layout and its pins. |
| Sensors | `sensors/perception.rs:7-14` (the `NEARBY_*` aliases and the index re-export), `:23-39` (`PerceptionSnapshot`), `:200-206` (the 256 B test); `reducers.rs:352-437` (`fill_nearby_banks`, the two comparisons); `static_inputs.rs:16-31`, `:89-92`, `:104-142` (`StaticInputs`, `resolve_static`, `assemble_static_inputs`); `simulation/tick.rs:292-299` (perception assembled only for a genome that declares an extended key) | Where the values are produced. |
| Sampler and universe | `mutation/sampling.rs:23-56` (27 buckets), tests `:108-230`; `mutation/input_ref/mod.rs:42-63`; `mutation/graph/recruitment.rs:271-314` (`families`, `universe_entries`); `mutation/compound.rs:19-30`; `creature/genome/mesh_annotations.rs:156-168` | The lists the bucket table replaces; width and class of a static key. |
| Catalog and contexts | `neighborhood/input_use/catalog.rs:15-36` (`Family`), `:127-145` (`addressed`), `input_use/mod.rs:48` (`input-use-v2`); `mutation_effects/contexts.rs:46-50` (`NEARBY_SLOT`), `mutation_effects/mod.rs` (`VERSION`, `COVERAGE_VERSION`); `crates/v3-lab/src/readings.rs:465-483` (`census_covers`), `v3-lab/src/summary.rs:26` (`GENOME_FORMAT` 6) | The readers that carry a family or a field count. |
| Transport | `runtime/trace/domain.rs:194`, `crates/v3-server/src/transport/sample_protocol.rs:99` (`nearby: [f32; WIDTH]`), `v3-server/src/types.rs:5` (`v3alpha11`), `v3-server/src/state.rs:9,434`, `http/creature.rs:14,430` (`channels_to_rgb` for display); `crates/v3-telemetry/src/windows.rs:471` (`petri.nearby`), `metrics/tests.rs:499` | Payload widths and the colour-map importers. |
| Frontend | `frontend/src/components/inspector/inputRefUtils.ts:44,58` (label, description), `:91-105` (`DIRECTION_PAIRS`), `:152-169` (`NEARBY_FIELDS`, `NEARBY_SLOTS`), `:194-205` (`SCALAR_LABELS`); `directionPairs.json`; `inputRefUtils.test.ts`; `types/trace.ts:19-20` | The hand mirrors the fixtures replace. |
| Pins | `tests/viability.rs` founder-only digests (must not move), `tests/baseline_worlds.rs`, `tests/applied_trajectory.rs`, `mutation/engine/recruitment_tests.rs` supply pin, `mutation/graph/recruitment_tests.rs` and its regression file, the slow-tier and `--lib` recruitment-paths rows, the v3-cli trajectory fixtures, the gate and world-set deterministic blocks, `docs/progress/benchmark-series.json` epochs | The predeclared re-pins (Performance section). |
| Reference specs | `docs/reference/v3-sensor-spec.md` §1 (`:54`), §3.1, §5.4, §5.7 (pair row), §6.5; `v3-creature-identity-spec.md:50-56`, §6; `v3-mutation-spec.md:686-699` (sampler table) and the pair case; `v3-server-api-protocol-spec.md` §2 and the perception payload | Every sentence that names a retired field, the width or the bucket count. |

Invariants:

| # | Invariant |
| --- | --- |
| 1 | **Layout.** `nearby_idx` declares `SLOTS 4`, `FIELDS_PER_SLOT 10`, `WIDTH 40`: `PRESENT 0`, `REL_X 1`, `REL_Y 2`, `DIST 3`, `ENERGY_RATIO 4`, `REPRODUCE_READY 5`, `KIN_AFFINITY 6`, `RED 7`, `GREEN 8`, `BLUE 9`; slot `k` field `f` is channel `10k + f`. `LINEAGE_MATCH`, `PHENOTYPE_SIMILARITY` and `compute_phenotype_similarity` are deleted. Fields 0–6 keep their value, normalization and ranking exactly (sensor spec §5.4–5.6, §6.4 and the `kin_affinity` formula unchanged); an empty slot is all zeros. |
| 2 | **Seen colour.** For the creature in slot `k`, `RED`, `GREEN`, `BLUE` are `channels_to_rgb(target.phenotype_channels)[c] as f32 / 255.0`, in `[0, 1]`, computed when perception is assembled; nothing is cached on `CreatureState`, which gains no field or method. |
| 3 | **Own colour.** `StaticIntrospectionKey` gains `OwnRed`, `OwnGreen`, `OwnBlue` (JSON tags of the same name). `StaticInputs.own_colour: [f32; 3]` is assembled in `assemble_static_inputs` for every creature from its own channels by the same formula; `resolve_static` returns the matching channel. Each key is scalar (`sub_value_count` 1), `MeshReadClass::Introspection`, and swappable with the other width-1 introspection keys. **Symmetry:** what a creature reads as its own colour is what a neighbour's slot reads of it in the same tick (one test over the real reducer and `assemble_static_inputs`). **Outside a live world**, a scenario that carries no creature (the neighborhood battery's `draw_scenario`, the opportunity fixtures) assembles `own_colour` as the founder's colour, `unit_rgb(FOUNDER_CHANNELS)` ≈ `[0.80, 0.24, 0.24]`, read from the one founder declaration (`simulation::seeding::FOUNDER_CHANNELS`, made crate-visible), because that is the colour of every creature those genomes are seeded into and `[0, 0, 0]` is not a creature colour (lightness is mapped into `[0.25, 0.75]`): a zero would make an own-colour edge silent in the battery and live in the world. |
| 4 | **Colour module.** `hsl_to_rgb` and `channels_to_rgb` move to `crates/v3-core/src/creature/colour.rs` (`creature::colour`), with their tests; `mutation::phenotype` keeps `mutate_phenotype` and imports nothing it does not use; the server's two imports follow; no re-export is left behind. |
| 5 | **Pairs and recruitment.** `direction_pairs()` for `NearbyCreature` declares slot `k`: `x = 10k + 1`, `y = 10k + 2`, `facts = [10k, 10k + 3, 10k + 4, 10k + 5, 10k + 6, 10k + 7, 10k + 8, 10k + 9]` (eight facts); the four pairs still partition the layout (`slot_pairs` test). A `WholeFamily` slot recruitment on a directed sink places one twelve-member `DirectionPair` group plus eight born-0 facts edges (20 edges); on any other destination ten plain edges `10k..10k+10`; the `1..=30` bound and every other T25.F04/F05 rule hold. Recruitment's family multiplicity is its own declaration and is unchanged: `universe_entries` stays 3 for `NearbyCreature` and 1 for every other concrete reference (e06c641b records the sampler's buckets and recruitment's weights as separate draws); only its enumeration order changes (invariant 7). |
| 6 | **Field table and fixture.** `nearby_idx::FIELDS: [NearbyField; FIELDS_PER_SLOT]` declares each field's `name` and `description` in layout order (`present`, `dx`, `dy`, `dist`, `energy`, `ready`, `kin`, `red`, `green`, `blue`). `frontend/src/components/inspector/nearbyFields.json` (`{ "slots": 4, "fields": [{ "name", "description" }, ...] }`) is pinned by a Rust test against the table, as `directionPairs.json` is against the pairs. The frontend imports both fixtures: `NEARBY_FIELDS`, `NEARBY_SLOTS` and the hand-written `DIRECTION_PAIRS` are deleted, and `formatInputRefWithSubIndex({World: "NearbyCreature"}, 17)` → `Nearest2.red`; channel 40 falls back to the raw index. The family description becomes "Position, vitals, kinship and colour of the 4 nearest visible creatures."; `SCALAR_LABELS` gains the three own-colour keys. |
| 7 | **Bucket table.** `mutation/sampling.rs` declares one `INPUT_BUCKETS` table of 30 buckets in this order: `FoodHere`, `NeighborFoodRing`, `NeighborBarrierRing`, `NeighborOccupiedRing`, `AgeTicks`, `OwnRed`, `OwnGreen`, `OwnBlue`, `EnergyCurrent`, `EnergyConsumedThisTick`, `ActionQueue`, `AreaFoodSummary`, `AreaBarrierSummary`, `NearbyCreature` ×3, `AreaOccupancySummary`, `ActionVotes`, `PreviousPassVotes`, `CommitCounts`, `HopsThisTick`, `PreviousOutcome`, `UpstreamSlot` ×8. The sampler is one `gen_range(0..30)` into the table; a food-typed bucket then draws its type (with more than one food type) and an upstream bucket draws its slot over `0..OUTPUT_SLOT_COUNT` (24), exactly as today: eight buckets is the upstream family's draw weight, not its slot count. `input_reference_universe` is the table expanded over the world's food types and all 24 upstream slots, deduplicated in table order (so `AgeTicks`, the three own-colour keys and the rest follow the table rather than today's `WorldInputKey`-first order: recruitment's family draw maps to different references for the same draw, a predeclared re-pin cause of commit 2). Recruitment's `universe_entries` is not derived from the table (invariant 5). **Agreement test:** for food type counts 1 and 3, the set of references the buckets can produce (every food type, every slot including `UpstreamSlot(23)`) equals `input_reference_universe` as a set, every `WorldInputKey::universe` key is in it, and `Family::of` labels every member (three new variants after `AgeTicks`; `input_use::VERSION` `input-use-v3`, "20 named families"). |
| 8 | **Contexts and classes.** `contexts::NEARBY_SLOT: [Range; nearby_idx::FIELDS_PER_SLOT]` is built in a `const` block through the named indices (`Unit` everywhere, then `PRESENT` and `REPRODUCE_READY` set `Binary`, `REL_X` and `REL_Y` set `Signed`), never as a positional literal, so a reordered or added field cannot silently take another field's range; `COVERAGE_VERSION` `neighborhood-coverage-v3` (from `v2`), `mutation_effects::VERSION` `mutation-effects-v5` (from `v4`). The own-colour keys take the arms `AgeTicks` takes in `addressed` (channel 0), `census_covers` (not covered), `MutationAddedNodeInputClass` and every other classifier. |
| 9 | **Snapshot and transport.** `PerceptionSnapshot.nearby: [f32; nearby_idx::WIDTH]` (40); the `≤ 256` byte test is deleted and no ceiling replaces it; `PerceptionDebugSnapshot`, the server payload, `types/trace.ts` and the window attribute `petri.nearby` (40 doubles) follow the constant; `WorldReads` bits are unchanged. The compatibility re-export in `sensors/perception.rs:11-14` (`pub use crate::contracts::{barrier_idx, food_idx, nearby_idx, occupancy_idx}`, kept "so every reader compiles unchanged") and the `NEARBY_SLOTS` / `NEARBY_WIDTH` aliases are deleted; `reducers.rs`, `contexts.rs`, `trace/domain.rs`, `opportunity/controllers.rs`, `opportunity/fixtures.rs` and the `inputs.rs` test that pinned the alias import `contracts::{..._idx}` directly. |
| 10 | **Identity and determinism.** The lab's `GENOME_FORMAT` becomes 7 and the server's `PROTOCOL_VERSION` `v3alpha12` with a §2 history line (the `nearby` payload width, the three static keys). Draw streams change for every mutation-on run, by commit and artifact: commit 1 changes the sensed value at slot fields 7 and 8 (red and green where lineage and similarity stood) and adds field 9, the family width (36 → 40, so every input-edge sub-index draw over it) and the slot recruitment's edge count (20 where 19 stood, so genome sizes and every later draw); commit 2 changes the key draw range (27 → 30), the universe's enumeration order behind recruitment's family draw, and the width-1 introspection `Swap` alternatives. With mutation off, a founder-only trajectory is byte-identical, because the founder declares neither the creature family nor an own-colour key: the founder-only viability digests and the founder's genome size (`FOUNDER_GENOME_SIZE_UNITS`, 80) do not move. A loaded genome that reads slot field 7 or 8 reads colour where it read a comparison; no stored genome in the repository does. |
| 11 | **Telemetry keys.** No metric family is added or removed; `petri.world_input = "NearbyCreature"` keys are unchanged; `scripts/telemetry-dashboards-check` is unchanged. |
| 12 | **Reference specs.** Sensor spec §1, §3.1, §5.4 (ten fields, `10k + f`), §5.7 (the pair row), §6.5 ("Nearby Creature Identity and Colour", fields 6–9); identity spec visibility posture and §6 (`lineage_match` removed; `lineage_id` observation-only); mutation spec sampler table (30 buckets, the one table) and pair case (20 edges); protocol spec §2 and the perception payload. |

Readers, with their post-change expectations:

| Reader | After F01 |
| --- | --- |
| `fill_nearby_banks`, layout pin test | fields 0–6 as before, 7–9 the target's RGB over 255; the pin is rewritten first over the real reducer with two visible creatures of distinct colour, and asserts fields 0–6 bit-identical to today's values |
| `assemble_static_inputs`, `resolve_static`, `resolve_input` | `own_colour` beside `age_ticks`; the three keys resolve it |
| Sampler, universe, `Swap` partition | from the one table; `catalog_draw` in the sampler tests divides by the table length. Recruitment enumerates the (table-ordered) universe with its own unchanged weights |
| `input_use` (`input-use-v3`), lab readings, bench `input_use` block | `Family::OwnRed/OwnGreen/OwnBlue`; `NearbyCreature` channels 0–39 |
| `mutation_effects` | `NEARBY_SLOT` ten ranges, `mutation-effects-v5`, `neighborhood-coverage-v3`; six channel groups unchanged |
| Server | payload `[f32; 40]`, `v3alpha12`, `creature::colour::channels_to_rgb` |
| Inspector | labels and descriptions from the two fixtures; `Nearest1.red`; `OwnRed` "Own red: this creature's own red channel (0 to 1)." and likewise green, blue |
| Lab | `GENOME_FORMAT` 7; a format-6 file is refused by the existing load guard |

## Telemetry

No signal is added and no configuration value or switch exists: colour is
perception, always on. A person looking at a run sees:

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| goal summary `input_use` block rows `NearbyCreature` channels 7–9 of each slot (7–9, 17–19, 27–29, 37–39) and families `OwnRed`, `OwnGreen`, `OwnBlue` | per-world funnel rows (existing block, `input-use-v3`) | declared, connected, executed and causal parents over colour channels | the family and channel labels; a colour row absent from an evaluated cohort is reported in the readings file as zero use for that cohort, and a cohort the block does not evaluate stays unavailable |
| `petri.tick.sensor_declared` / `sensor_wired` / `sensor_read` | gauge (existing) | the census over `NearbyCreature` (ten fields behind one key) | `petri.world_input = "NearbyCreature"` |
| creature window tick span `petri.nearby` | span attribute (existing) | the perception snapshot | 40 doubles |

## Verification Plan

**Question.** F01 claims that a brain can read each nearby creature's colour
and its own as ordinary inputs, that every other sensed value is unchanged,
and that the tick loop pays only a few colour conversions per creature; a
failing layout pin or symmetry test is a defect, a gate `wall_clock.level`
of `severe` is a defect (the retired 256-byte test's replacement reading;
the level exists only when the run's host matches the epoch's, so a run on
another host is inconclusive for this reading and is re-run on the epoch
host before closure), and a gate or goal counter `severe` (the aggregate
flag covers counters) is a trajectory reading under the changed draw stream
(Performance: no predeclared direction), reported to the user and re-pinned
in the closing commit, never tuned away. The goal run's colour-channel use
rows are descriptive, zero included (design note §3 A: a zero-use reading is
a valid result), and cannot show the feature wrong.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate + goal`, viability first (`cargo test -p v3-core --test viability` before the gate) | the track note names F01 "gate + goal": a perception-assembly change whose cost the gate reads, and a production input change whose 2,000-tick declaration and use rows the goal profile takes; goal-long is rationed to T27.F02, the track's one carrier, and the long-run colour reading is deferred there. |
| Mutation gate | `run`: fresh `make rust-mutants-ci`, with local `MUTANTS_ITERATE=0 make rust-mutants` as the fallback when Actions is unavailable | new production logic in `v3-core` (the reducer's colour fields, the own-colour static inputs, the bucket table, the fixture pins) that tests must pin. |
| Slow tests (`make rust-test-slow`) | `run` | the input catalog changes (30 buckets, eight slot facts), which moves the recruitment-paths recorded readings the tier pins. |

## Implementation Tasks

- [x] Commit 1 (layout, tests first): the layout pin rewritten over the real reducer before the change; `creature::colour` (invariant 4); `nearby_idx` ten fields with `FIELDS` and the two retired fields deleted (1, 2); pairs, partition test, `directionPairs.json` and `nearbyFields.json` with their Rust pins (5, 6); the frontend importing both fixtures with its parity tests over them; the 256-byte test, the index re-export and the aliases deleted, and the width readers (9); the named-index contexts array with `neighborhood-coverage-v3` and `mutation-effects-v5`, and `input-use-v3` because the family's channel meanings change (8, 7); `GENOME_FORMAT` 7 and `PROTOCOL_VERSION` `v3alpha12` whose §2 history line names this commit's surface, the 40-wide `nearby` payload (10); the sensor, identity and protocol reference specs and the mutation spec's pair case (12); every mutation-on pin this commit moves re-pinned, each with its cause from invariant 10's commit-1 list (sensed values, width, slot edge count), founder-only pins unmoved. Verified by the layout pin, the partition test, the two fixture pins, the frontend tests and `cargo test -p v3-core`.
- [x] Commit 2 (self inputs and the bucket table, tests first; depends on commit 1): `OwnRed`, `OwnGreen`, `OwnBlue`, `StaticInputs.own_colour`, resolution and the symmetry test (3); `INPUT_BUCKETS` with the sampler and the universe reading it, `universe_entries` unchanged, and the agreement test (7); the three `Family` variants under the `input-use-v3` already in force (7, 8); `SCALAR_LABELS` (6); the mutation spec sampler table and the protocol history line extended with the three static keys (10, 12); every mutation-on pin this commit moves re-pinned, each with its cause from invariant 10's commit-2 list (draw range, universe order, `Swap` alternatives). Verified by the symmetry and agreement tests, the sampler index tests and `cargo test -p v3-core`.
- [x] Verification Plan: viability first, then gate and goal through the benchmark specialist; mutation gate and slow tier through the mutation specialist; epochs re-pinned in the closing commit.

## Verification

- [x] Tests: `cargo test -p v3-core --no-fail-fast` (the layout pin, the symmetry test, the agreement test, the two fixture pins, the partition test), `cargo test -p v3-cli --lib bench`, `cargo test -p v3-server`, `cargo test -p v3-telemetry`, `cargo test -p v3-lab` → results recorded here. At b697f58d: v3-core 2,310 passed, 0 failed, 10 ignored. At 1e28f469, unchanged since outside v3-core tests: v3-cli `--lib bench` 132 passed, `--test cli` 20 and `--test bench` 13 passed; v3-server 168, v3-telemetry 78, v3-lab 211 passed, 0 failed. Re-pins by artifact and cause: the readings file.
- [x] Frontend: `npx vitest run src/components/inspector` and `npx biome check` on the touched files → results recorded here. At 1e28f469, no frontend change since: `npx vitest run src/components/inspector src/types`: 44 files, 311 tests passed; `npx biome check src/components/inspector src/types` clean (110 files); `npx tsc -b` exit 0.
- [x] Viability first: `cargo test -p v3-core --test viability` → founder-only digests unchanged: 30 passed after each commit through b697f58d. The mutation-off trajectory digests (`applied_trajectory` bd8dca14..., `baseline_worlds` 8324246536663749152) hold at both commits; b697f58d is test- and comment-only and `applied_trajectory` (2) and `baseline_worlds` (19) pass there unchanged.
- [x] Static checks: `cargo check --workspace --all-targets`, `cargo fmt --all --check`, `make rust-clippy` → clean at b697f58d.
- [x] Mutation gate: CI fresh (39573198443) at 1aeb2ee4: `144 mutants tested: 14 missed, 117 caught, 13 unviable, 0 timeouts`; `~/.local/share/petri-tools/mutants/t27-f01/mutants.out`, all `colour.rs`. Killed by `channels_to_rgb_matches_reference_across_every_hue` (8/8 on iterate): 31:36, 45:10 `==`, 46:11 ×2, 49:18, 49:23 ×2, 49:28. Equivalent, each boundary's sides agree (0 reachable inputs differ): `<=` at 22:18, 42:10, 48:10, 51:10, 54:10; `>=` at 45:10.
- [x] Gate and goal (2026-10-21, HEAD 182c43f0): summaries at `docs/progress/features/t27-f01-colour-vision.json` and `-goal.json`, local raw hash and byte count and verification time checked, series entry points to the summary, no new full report staged; the gate's `wall_clock.level` quoted; the goal `input_use` rows for the colour channels and own-colour families quoted in the readings file per evaluated cohort, an absent row reported as zero use and an unevaluated cohort as unavailable. Viability 30 passed (exit 0). Gate: CLI exit 0 (`measurement_evidence.cli_exit`), `make` exit 0, `severe=false`, `wall_clock.level` `ok` (+17.6%), `vm_steps` flag +28.2%. Goal: CLI exit 3 (`measurement_evidence.cli_exit`), `make` exit 2, `severe=true` on `decided_passes` +72.1%, accepted by the user. Raw gate 129,167 B `98253a75...b26585`, raw goal 706,943,675 B `cb502110...745cb` (both `verified_local`, 2026-10-21); summaries 25,889 B and 533,714 B. Series entries and both epochs re-pinned at 1772dda7.
- [x] Slow tier: `make rust-test-slow` → exit 0 at 10e7e50c (recruitment-paths 3 passed, lab 6 passed) after one re-pin (readings file).
- [x] Close: `make check` exited 0 at the tested commit 10e7e50c.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: colour vision;
the marking reaches a creature through its body (the heritable channels it
already carries) and its eyes (the ranked nearby family), never through a
computation invented for the feature; the only comparison kept,
`KIN_AFFINITY`, is the master's ruled exception. Compute: per tick, one
`channels_to_rgb` per creature in `assemble_static_inputs` and up to four
per creature whose genome declares an extended key, each about forty float
operations beside the 121-cell visible scan; the gate's
`wall_clock_ms_per_creature_tick` is expected within its noise band against
the F09 epoch on the same host, a `wall_clock.level` of `severe` is a
defect, not a reading, and a missing level (host mismatch) is inconclusive
and re-run on the epoch host. Every mutation-on counter is a trajectory
reading under a changed draw stream (invariant 10's lists per commit, and
`UpstreamSlot`'s sampler share 8/27 → 8/30): no
direction is predeclared for any gate counter or goal indicator, a `severe`
there is reported to the user as a reading and the epoch is re-pinned in the
closing commit, never tuned. Goal readings: the `input_use` rows for the
colour channels and the own-colour families are reported per world, zero
included; the recorded expectation is low or zero use at 2,000 ticks
(heritable colour diverges by about 1/255 per mutated birth, and tag
cooperation needs repeated encounters), so a zero-use reading is a valid
result and the long-run reading is deferred to T27.F02's goal-long run. No
diversity or cognition indicator is added. Both epochs are re-pinned in the
closing commit.

**Measured verdict (2026-10-21, HEAD 182c43f0, final code b697f58d, each profile run once).**
Viability 30 passed. Gate: v3-cli exit 0 (`measurement_evidence.cli_exit`),
outer `make` exit 0; `severe=false`; `wall_clock.level` `ok` (present, so the
host matches the epoch host): `wall_clock_ms_per_creature_tick` 0.0031029 vs
the F09 epoch 0.0026384 (+17.6%), inside the 25% flag band; one counter flag,
`vm_steps` 0.040014 vs 0.031219 (+28.2%). Goal: v3-cli exit 3
(`measurement_evidence.cli_exit`, successful artifact pair with a severe),
outer `make` exit 2 (`Error 3`); `severe=true`, one severe: `decided_passes`
0.003873 vs 0.002251 (+72.1%, severe at 50%), which the user accepted
(2026-10-21) as a trajectory reading; both epochs re-pinned here; two flags, `graph_relax_iters` +10.4% and
`actions_applied` +12.9%; goal wall clock 0.0125132 vs 0.0093403 (+34.0%,
`flag`, not fatal). Colour-channel use rows (descriptive): the founder cohort
declares none; at tick 2000 the selected cohorts show causal colour use only in
Confluence (slot 2 red causal in 4 of 7 parents, `OwnBlue` in 9 of 9) and one
causal `OwnGreen` in Orchards, none in Canyon; rows in the readings file. Raw
gate 129,167 B, sha256 `98253a75...b26585`; raw goal 706,943,675 B, sha256
`cb502110...745cb`; summaries 25,889 B and 533,714 B.

- Summaries of the declared profiles: [gate](../../progress/features/t27-f01-colour-vision.json),
  [goal](../../progress/features/t27-f01-colour-vision-goal.json).
- Full readings: [`docs/progress/readings/t27-f01.md`](../../progress/readings/t27-f01.md).

## Success Criteria

- [x] A creature's nearby slot carries the seen creature's red, green and blue, its own three channels are drawable, resolvable self inputs, `LINEAGE_MATCH` and `PHENOTYPE_SIMILARITY` no longer exist, and `KIN_AFFINITY` is unchanged (invariants 1–3, pinned).
- [x] The nearby field names, the pairs and the input buckets each have one declaration: the two fixtures are Rust-pinned and frontend-imported, and the sampler/universe agreement test passes (invariants 6, 7).
- [x] Founder-only trajectories are byte-identical; every moved mutation-on pin is re-pinned with its one cause (invariant 10).
- [x] The Verification Plan's checks ran on the final code and the goal readings report colour-channel use per world, zero included.

## Notes for AI Agents

- Decision: own colour is three scalar `StaticIntrospectionKey` variants, not a compound static key: each costs one declaration and reaches a brain by a single-channel recruitment or by a `Swap` of an existing width-1 introspection input, and no compound-static resolution path is invented; the cost is three units to declare all three where one compound key would cost one (readings file, options table).
- Decision: the colour mapping lives in `creature::colour`; `mutation::phenotype` is the operator only.
- Decision: a creature-less scenario (battery, opportunity fixtures) reads the founder colour as its own colour, never zero (invariant 3); a later self input with no zero-valued live state follows the same rule, its scenario value being the founder's.
- Decision: the input buckets have one declaration, `INPUT_BUCKETS` in `mutation/sampling.rs`, from which the sampler draws and the universe is expanded; recruitment's family weights (`universe_entries`) are a separate policy and stay their own declaration. A feature that adds an input adds one bucket (T28.F01 extends this table with its slot inputs; T27.F02 adds no input key).
- Decision: the `mutation_effects` per-field ranges are built through named `nearby_idx` indices, never a positional literal; the `sensors::perception` index re-export and `NEARBY_*` aliases are gone, and readers import `contracts` directly.
- Decision: the nearby field names and the direction pairs are declared in Rust and reach the frontend through Rust-pinned fixtures; the frontend keeps no hand copy of a layout (T27.F02 extends `FIELDS` to 18 and `nearbyFields.json` follows).
- Decision: the `PerceptionSnapshot` byte ceiling is gone; the gate profile's tick time is the reading, and no replacement ceiling is invented.
- Decision: `lineage_id` stays as observation-only telemetry (lineage indicators, server lineage map, window records).
- Deferred: the long-run reading of whether colour channels become behavior (retention of a colour-reading recruitment past 2,000 ticks, any colour-conditioned action) goes to T27.F02's goal-long run, the track's carrier.
- Deferred: the retina (per-direction RGB, design note §3 A3) until a reading shows slot colour in use.
- Cost: `/usage` not collected; implementer 4 passes, 8 advisor consults; 1 spec-owner resume; 3 Codex rounds plus a confirm job, final `ready`; review P2 1 correctness, P3 1 code health, design none.
