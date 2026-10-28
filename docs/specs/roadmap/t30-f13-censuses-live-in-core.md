# T30.F13 — Censuses Live in Core, Summaries Follow Their Types

**Status**: In Progress
**Last updated**: 2026-10-28
**Feature**: T30.F13
**Track**: [T30 — Core Boundaries](../../roadmaps/t30-core-boundaries.md)

## Goal

Every population census is computed once, in core's readers
(`neighborhood/`), and `v3-cli`, `v3-telemetry` and `v3-server` only
format those results for a report, a metric or the wire. Each benchmark
persistence sample is one composed record whose keys each declare their
retention beside their type, so a new sample block is one field, one
retention entry and one reader call, and the summary keep-lists derive from
those declarations. Trajectories, draw streams, every value a benchmark
report, the NDJSON stream, a telemetry gauge or a server payload emits,
and the committed summary for a given full report are unchanged;
any pin that moves is a defect.

## Non-Goals

- New censuses, new sample blocks, new metrics or wire fields.
- Changing what a census counts: weight floors, reachability filters,
  extinction semantics (`null`, true zero or `UNDEFINED`), percentile
  rules, key labels, version strings.
- The summary keep-lists outside persistence samples (`KEPT_CASE_BLOCKS`,
  `KEPT_PERSISTENCE_FIELDS`, `PER_SEED_INDICATORS`, the neighborhood and
  mutation-effects projections) and the v2 summary's shape.
- The per-creature behavioral probes `observe_final_actions` and
  `observe_temporal_actions` (`simulation/tick.rs`), counter reads from
  `SimStats`, and world (food-cell) readings such as `inspect.rs`'s table.
- The inspector's slot diagnostics (T30.F14).

## Inputs and Invariants

Inputs: the T30 row and its Notes entry for F13 (user direction,
2026-10-27; the [F13–F14 design review](../../progress/readings/t30-f13-f14-design-review.md)),
`docs/architecture.md` rule 6 and section 3, and the code on `79ef601ad`.
The census inventory (C1–C22), today's sample retention table and the
preparatory audit (R1–R10) are in the
[readings](../../progress/readings/t30-f13.md).

Invariants:

1. **Readers own every census.** Each calculation in the inventory moves
   into `v3-core/src/neighborhood/` with its arithmetic: integer sums,
   `f64` accumulation where the benchmark report uses it, the same
   iteration or key order, the same nearest-rank and rounded-index
   percentile rules. Reader outputs are numbers and core keys; `six()`,
   `UNDEFINED`, `fraction_or_undefined`, `Indicator` wrapping, applying
   the key labels (`as_key()`, `world_input_key_label`) and wire or serde
   types stay in the outer crate. After the move no outer crate folds over
   `sim.creatures`, or over per-creature readings it took from simulation
   state, to produce an aggregate; it may iterate the population only to
   build per-creature wire records (the server's frame) or to pass
   creatures to a reader. The same holds for readings the summary
   converter extracts from a stored report's JSON: it passes them to a
   reader and keeps only extraction, labels and JSON construction.

   | Today | Reader home | Readers' consumers |
   | --- | --- | --- |
   | C1 `Bundles::census` | `neighborhood/bundles.rs` | CLI `bundles` block; `RingRuleCensus` (invariant 3) |
   | C2 `SensorCensus::observe` fold | `neighborhood/sensor_census.rs` | CLI `sensor_census` block |
   | C3 `bucket_surviving_clades`, C6 `clade_diversity`, C14 lineage counts | one clade census module | CLI sample, `lineage_diversity`, `surviving_clade_profiles`; server `lineages_alive`, `lineages_effective` |
   | C4 pending-birth fold | the population census | CLI `gestation` |
   | C5 `CapacityDistribution::of`, `ShareDistributions` | one trait-distribution module | CLI `capacity`, `shares`; telemetry `mean_capacity` (its mean) |
   | C7 `structural_companions_census` | readers | CLI `structural_companions` |
   | C8 `generation_distribution`, C9 `structure_size_distribution` | readers | CLI evolved neighborhood, `reachable_structure_size_distribution` |
   | C10 `memory_sensitivity` counts | readers | CLI `memory_sensitivity`, `temporal_memory_sensitivity` |
   | C11 `structure_means`, C12 and C18 mean energy, C13 means, C14 generation and genome-size means, world-inputs-read mean, generation max, C15 complexity, C16 live VM world-input counts | one population census | CLI sample and NDJSON `tick_sample`/`run_completed`; telemetry `Census`; server status and health |
   | C17 `kernel/occupancy_grid.rs` | `neighborhood/occupancy.rs` (moved whole) | CLI `occupancy_grid` |
   | C19 `evolved_neighborhood_for_seed`'s per-genome evaluation and pooled operator tallies, births and steering; C20 `neighborhood_read_for_seed`'s per-genome evaluation, five sums, pooled births and exposure; C21 `sorted_creature_ids` and `selected_cohort`'s sample selection | readers (the evolved and read neighborhood observations) | CLI `mutational_neighborhood` evolved half, `neighborhood_read`, `mutation_effects` cohort |
   | C22 `artifacts.rs::mesh_summary` (`:189-238`): node, knockout, pass and execution sums and route-flag counts over sampled genomes' `mesh_execution` rows | a readers accumulator over per-genome mesh readings whose fields are optional | the summary converter, which extracts each row's scalars from JSON; a row lacking a key adds nothing, a within-snapshot count is absent when no row carries its key, and no row gives `null`, as today |

   Module names and how the population census is split are the
   implementer's; one census per concept, each computed in one place. For
   C19–C21 the reader owns sample selection (sorted ids, ranks), the
   per-genome evaluation calls, their seeds and the pooling; the CLI keeps
   the report rows, formula strings, version strings and fractions.
2. **Every output keeps its arithmetic.** Each value any consumer emits
   today (benchmark report, NDJSON, telemetry gauge, server payload) is
   bit-identical after the move. Where two consumers compute one quantity
   with different arithmetic, both definitions move into the one reader
   census as two named outputs computed side by side, each marked
   `WORKAROUND(T30.F13): one definition of <quantity>`; unifying them is a
   numerical-definition change the row does not grant:

   | Quantity | Benchmark definition | Live definition |
   | --- | --- | --- |
   | mean energy | `f64` sum / count (`run.rs:189-192`): sample `mean_energy` | `f32` sum / count (`Simulation::mean_energy`, which moves to readers): NDJSON, server status and health, `petri.tick.mean_energy` |
   | clade entropy | lineage-id order (`clade_diversity`) | ascending counts (server `lineages_effective`) |
   | mean capacity | sorted-order `f64` sum (`CapacityDistribution`) | creature-order `f64` sum (`petri.tick.mean_capacity`) |

   Every other output has one definition today and keeps it, the server's
   `f32` complexity mean (C15) and `f32` casts of `f64` means (C14)
   included.
3. **Bundles.** One readers constant replaces `drives::PATH_WEIGHT_FLOOR`,
   `ring_rules::PATH_WEIGHT_FLOOR` and `bundles.rs`' `NONZERO_FLOOR`
   (value `1e-3`, `>` comparison). The Graph (`genome/cgp/container.rs`)
   gains an iterator over `(BundleId, &Bundle)`; the six hand conversions
   of a slice index to a `BundleId` outside the genome
   (`bench/bundles.rs:111`, `ring_rules.rs:240`,
   `v3-server/src/transport/brain.rs:519`,
   `runtime/plasticity/learned.rs:190`,
   `mutation/graph/bundle.rs:249, 408`) read it. `RingRuleCensus` takes
   `ring_bundles`, `ring_bundles_nonzero`, `ring_bundle_members`,
   `ring_bundles_unreachable` and `ring_bundle_carriers` from the bundle
   census's `Ring` totals per creature instead of counting them again,
   and keeps its `(family, kind, offset)` rows and rule coverage; the
   `ring-rules-v3` and `bundles-v1` blocks are unchanged. That equality
   relies on a `Ring` wiring always having a sensor source; the
   implementer confirms it in the genome (`plans`, `resolve_in`) with a
   test, and a constructible counterexample is a requirement question.
4. **One sample record.** `PersistenceSample` holds `tick`, `population`,
   `births_total`, the composed blocks record and the flattened
   `WorldTracking`. The blocks record (today's `PopulationReadings`,
   mean energy included) is built by one observe call from reader outputs,
   only on sampled ticks, applies each block's extinction rule inside
   itself, and is moved into the sample whole; the per-field transfer in
   `PersistenceAccumulator::observe` is deleted. Serialized, a sample has
   today's keys and values (`#[serde(flatten)]`), and a full report stored
   before this feature still loads.
5. **Retention declared with the type.** Every sample key declares one
   retention, beside its field: kept whole in the summary; full report
   only (named in `omitted_details`); or, for `bundles`, aggregates on
   every checkpoint and `rows` on each seed's last checkpoint only, with
   the aggregate set derived from `Bundles`' fields. `WorldTracking`
   declares which of its fields a sample summary keeps. The v1-stage list
   in `persistence()`, `KEPT_SAMPLE_FIELDS` and `KEPT_BUNDLE_FIELDS` are
   derived from these declarations and deleted as literals. A test fails
   when a serialized sample key has no declaration, and a test pins the
   derived sets to the readings' retention table, so the minimum summary
   (the telemetry commit rule's) changes only by an edit to that pin.
6. **Order and identity.** Summary bytes depend on key sets, not list
   order (`serde_json` without `preserve_order`). For any full report, the
   feature build's `bench-summarize` writes the same `deterministic` and
   `omitted_details` as main's.

## Architecture

Owners touched: readers (`neighborhood/`) gain every population census
and become their one owner, as section 3's "censuses and indicators" row
states; the genome gains a read API (the bundle iterator) so callers stop
applying its id rule (rule 1); the world (`kernel`) loses the occupancy
census, which never was world state; `simulation` loses
`Simulation::mean_energy` to readers; `v3-cli`, `v3-telemetry` and `v3-server` keep
export, formatting and wire types only (rule 6 and section 2). No section
5 debt names these censuses, so none is paid; the plan sharpens the
readers row in `docs/architecture.md` to say every population calculation
lives there and outer crates format it. Imports stay downward: readers
already import the genome, runtime, creature and simulation, and nothing
below imports readers (`scripts/architecture-check.mjs`). The server and
telemetry already depend on `v3-core` and call readers today (`DriveCensus`,
`PlasticityCensus`, `sensor_use_census`).

## Telemetry

No signal or configuration is added or removed. The gauges `Census::of`
feeds are re-sourced from the readers:

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `petri.tick.mean_energy`, `mean_capacity` | gauge | readers population census, live definitions (invariant 2) | unchanged |
| `petri.tick.population`, `mean_genome_size`, `mean_mesh_nodes`, `mean_generation` | gauge | readers population census, same arithmetic | unchanged |

Every gauge value is bit-identical to today's on the same state.

## Verification Plan

**Question.** F13 claims every census moves into readers and the sample
record is composed with no trajectory, report value or retained summary
set changing; a differing gate deterministic block, a moved census pin, or
a different summary from the same full report would show it wrong.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate` | The gate's deterministic blocks carry every persistence-sample block and the founder neighborhood, so byte identity against the epoch answers the trajectory and sample questions in seconds; the indicators the gate leaves `Undefined` (lineage diversity, both memory sensitivities) and the goal-only case blocks are answered by the census pins captured before the move and by converter parity on the stored goal report, which a goal run would not improve on, since no population reading is in question. |
| Slow tests (`make rust-test-slow`) | `not run` | No `crates/v3-lab` code changes; the lab's one census call (`sensor_census` in `genome::sensor_analysis`) is not moved. |

## Implementation Tasks

- [x] Census pins first: tests capturing each inventory calculation's
      output, and the NDJSON, telemetry `Census` and server status and
      health values built from them, on fixture populations (bundles of
      every wiring on reachable and unreachable nodes, several clades,
      pending births, plastic edges, an empty population), committed
      against the pre-move code.
- [x] The Graph bundle iterator and the one weight floor; the six id
      conversions and the three floors read them (invariant 3).
- [x] Move C1–C22 into readers (invariant 1) with both definitions of
      each invariant-2 quantity and their `WORKAROUND(T30.F13)` comments,
      `RingRuleCensus` reading the bundle census (invariant 3); the CLI,
      telemetry and server read the readers; the pins stay green.
- [x] The composed sample record and the retention declarations; the
      three keep-lists derived; the declaration and retained-set tests
      (invariants 4 and 5).
- [x] A `WORKAROUND(T30.F13)` comment at the server's
      `vm_live_read_world_inputs_current` key conversion
      (`v3-server/src/state.rs`) naming the intended design: wire keys
      carry the food type (`world_input_key_label`), so food-parameterized
      inputs never share a key (readings R11).
- [x] `docs/architecture.md` readers row edit (in the plan commit) and
      any doc that cites a moved path: no current-state doc does; closed
      feature specs and readings keep the paths of their time.

## Verification

- [x] Census pins, committed against the pre-move code at `b945d8cd8`:
      `bench::census_pins` (report, crafted-population, extinct, bundle and
      NDJSON pins), `artifacts::tests::mesh_summary_keeps_its_pre_move_reading_of_partial_rows`,
      `metrics::tests::the_tick_census_keeps_its_pre_move_values` (telemetry),
      `state::census_pins` (server status and health); every literal is
      unchanged after the move.
- [ ] Workspace: `make check` exit 0.
- [x] No outer-crate census remains: a `grep` for folds over
      `sim.creatures` in `v3-cli/src`, `v3-telemetry/src`,
      `v3-server/src`, its hits each a per-creature record or a reader
      call, listed in the readings.
- [x] Converter parity: `bench-summarize` on
      `.bench-artifacts/t25-f11/goal.json` and
      `.bench-artifacts/t30-f10/gate.json` (main checkout) with main's
      and the feature's build; `deterministic` and `omitted_details`
      equal; hashes in the readings (a report main's build cannot load is
      replaced by the newest stored one it can, and the readings say so).
- [ ] Gate: summary at `docs/progress/features/t30-f13.json`, raw hash and
      bytes in the readings, series entry, no full report staged.

Commands run on the feature head (2026-10-28), each exit 0: `cargo check
--workspace --all-targets`; `cargo fmt --all --check`; `make rust-clippy`;
`cargo test -p v3-core -p v3-cli -p v3-server -p v3-telemetry` (viability
30 passed within it; no default or tick-loop change since the build, where
`--test viability` ran first); `node
scripts/architecture-check.mjs`. `v3-cli bench-summarize` parity on both
stored reports ran on the build at `aa3cc6402` (readings); the later commits
change no converter code. Every pin literal is unchanged. `make check` and the gate are not yet run.

## Notes for AI Agents

- Decision: R1–R3 and R7–R10 of the readings' audit are in-feature.
- Deferred: unpaid accept, the paired benchmark and live definitions of
  mean energy, clade entropy and mean capacity in the readers census
  (invariant 2, R4–R6); payer proposed to the user at closure.
- Deferred: unpaid accept, `v3-server/src/state.rs`
  `vm_live_read_world_inputs_current` keys counts by
  `WorldInputKey::as_key()`, which drops the food type, so two food types'
  counts collide and one survives (today the last in key order; before
  this feature, whichever `HashMap` order put last); payer proposed to the
  user at closure, with the same keying in
  `mutation_added_node_world_inputs_total_by_operator` (readings R11).
- Exception: a fourth Codex job (spec-round3 template, priors rounds 1–2, round 3 via readings) confirmed 3.F3's C22 fix and upheld no blocking finding; its not-ready verdict rested only on the superseded 2.F3 rebuttal, withdrawn as round 4 directed; orchestrator decision 2026-10-28.
