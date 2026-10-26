# T30.F03 — Sensors Read Views

**Status**: In Progress
**Last updated**: 2026-10-26
**Feature**: T30.F03
**Track**: [T30 — Core Boundaries](../../roadmaps/t30-core-boundaries.md)

## Goal

A creature perceives exactly what it perceived before, from explicit views
of the world and of each body: `sensors` reads `WorldState` and a `BodyView`
per creature that the creature record builds and the scheduler collects once
per tick, never a `CreatureState`, a `SlotMap` of them, the genome or the
colour helper. What a brain needs assembled is a
`PerceptionRequirements` value the genome's own analysis computes once per
birth, and the same analysis computes the declared and wired key sets the
sensor-census gauges read. Debts G9, G14 (the caches)
and G20 of [`docs/architecture.md`](../../architecture.md) are paid, and
every trajectory, population and draw stream is unchanged.

## Non-Goals

- Any new sensed state: T27.F02 (held signals) and T29.F02 (body genes) put
  theirs on the views afterwards.
- A change to which creatures get typed food or extended perception
  assembled, or to any sensor value or rule.
- The outcome-channel vocabulary (G2: T25.F11), the genome directory and
  the readers' home (G24, G26: T30.F07), the runtime state in the creature
  record (G8: T30.F02), the birth constructor (G4: T29.F01).
- Moving `WorldKeySet` to `contracts`: once sensors stop importing the
  creature record, `creature → sensors::reads` points down.
- Parallel sensor assembly, the frontend.

## Inputs and Invariants

Inputs: the row and the track's F03 notes; `docs/architecture.md` section 3
(sensors' prohibition and its views) and debts G2, G9, G14, G20; the code
named below; the [readings file](../../progress/readings/t30-f03.md)
(options, the preparatory audit R1–R7).

The invariants state the closing state.

1. **Sensors' imports.** Production code under `sensors/` imports
   `contracts`, `config`, `kernel`, `sensors` and `slotmap` (the
   `SecondaryMap<CreatureId, BodyView>` of invariant 2; never a map of
   creatures), plus `creature::genome::{OutcomeChannel,
   OUTCOME_CHANNEL_COUNT}` in `static_inputs.rs` alone, the one site G2
   names, marked `WORKAROUND(G2): OutcomeChannel moves to contracts
   (T25.F11)`. No `CreatureState`, no `creature::colour`. The debts file
   gains no line; its G9 lines (four) and G20 lines (two) leave;
   `scripts/architecture-check.mjs` passes.
2. **The body view.** `sensors::static_inputs::BodyView` (in the module G2
   already names, so the import footprint is unchanged), `Copy`: `position:
   Position`, `age: u64`, `energy: f32`, `energy_scale: f32` (the body's
   sensor denominator, `HeritableBody::energy_scale`), `colour: [f32; 3]`
   (the seen colour, `creature::colour::unit_rgb`), `kin_tag: u32`,
   `previous_outcome: [f32; OUTCOME_CHANNEL_COUNT]` (the stored bank).
   `BodyViews = SecondaryMap<CreatureId, BodyView>` beside it. The one
   builder is `CreatureState::sensed_body(&self, &EnergyCapacityConfig) ->
   BodyView`; sensors compute no body rule (no colour mapping, no capacity
   rule).
3. **Assembly signatures.** `assemble_static_inputs(&WorldState, &BodyView,
   &EnergyConfig) -> StaticInputs` and `assemble_perception(observer:
   CreatureId, &[VisibleCell], &WorldState, &BodyViews, &PerceptionConfig)
   -> PerceptionSnapshot`. The observer is `views[observer]`; a nearby
   candidate is `world.creature_at(cell)` looked up in the views, skipped
   when absent, as `creatures.get` skips today. Every field reads the same
   value as today: `age_fraction`, `previous_outcome_read`, kin affinity, the
   reproduce-ready test and the energy ratio are unchanged sensor rules.
4. **The scheduler builds the views.** `assemble_sensor_inputs_with` fills
   one `BodyViews` over every living creature (`sim.creatures`) before the
   queue is walked, once per call, and passes it to both assemblers; the
   server's creature diagnostics (`v3-server/src/http/creature.rs`) build the
   one view they need with `sensed_body`.
5. **Perception requirements.** `contracts::PerceptionRequirements {
   typed_local_food: bool, extended_perception: bool }` with `of_key`
   (`FoodHere`, `NeighborFoodRing` → `typed_local_food`; `AreaFoodSummary`,
   `AreaBarrierSummary`, `AreaOccupancySummary`, `NearbyCreature` →
   `extended_perception`; the barrier and occupied rings → neither), a
   union, and `ALL`. `genome::sensor_analysis::perception_requirements
   (&CreatureGenome)` folds `of_key` over every node's `input_refs`, every
   node, exactly the set `genome_uses_extended_perception` and
   `genome_uses_typed_local_food` walk today; those two and
   `PerceptionSnapshot::is_extended_key` are deleted.
6. **Cached at birth.** `CreatureState::cached_perception_requirements`,
   computed in `new_with_cached_fields` beside the other genome-derived
   caches. The tick's assembly reads it (`|creature|
   creature.cached_perception_requirements`), `assemble_full_sensor_inputs`
   passes `PerceptionRequirements::ALL`, and the traced tick's
   `TickOutcome.{uses_typed_local_food, uses_extended_perception}` read the
   cache. A creature is assembled typed food exactly when
   `typed_local_food`, visibility and perception exactly when
   `extended_perception`, as today.
7. **Genome sensor analysis.** `creature/genome/sensor_analysis.rs` holds
   `SensorCensus` (today's `CreatureSensorCensus`, fields unchanged:
   `world_inputs: BTreeSet<WorldInputKey>`, `decision_inputs`,
   `reads_shared_memory`, `holds_stateful_node`), `DecisionInputKey`,
   `sensor_census(genome, reachable)`, `declared_world_keys(genome,
   reachable) -> BTreeSet<WorldInputKey>`, `perception_requirements`,
   `has_barrier_reader(genome, reachable)` and `live_vm_world_reads(genome,
   reachable)`, each returning what today's function returns on every input
   (their tests move with them). `creature::sensor_census` keeps
   `sensor_use_census`, `SensorUseRow` and `world_input_key_label` only.
   `v3-cli/src/bench/tracking.rs` and `v3-lab/src/readings.rs` import the
   new path.
8. **The creature record computes nothing.** `new_with_cached_fields`
   calls the analysis module for `cached_has_barrier_reader`,
   `cached_live_vm_world_inputs`, `cached_declared_world_inputs`,
   `cached_wired_world_inputs` (collected into `WorldKeySet`, which stays in
   `sensors::reads` beside `WorldReads`) and the requirements;
   `compute_has_barrier_reader`, `compute_live_vm_world_inputs`,
   `sensor_caches` and `refresh_sensor_caches` are deleted.
9. **Perception-mix seeding.** `seed_simulation_with_perception_mix` edits
   the first `n` founder genomes before construction through a per-founder
   genome hook on `seed_simulation`'s path; the hook draws nothing, so
   `seed_simulation`'s positions, headings and bodies are byte-identical to
   today's. Every cache of a mixed founder derives from the built genome
   (today four other caches stay stale after the edit; audit R5). The
   fixture feeds `benches/tick.rs` and seeding tests only; no benchmark
   profile uses it.
10. **Dead arm.** `StaticInputs::resolve_world` is deleted;
    `SensorSnapshot::resolve_world` resolves `FoodHere` through the typed
    snapshot and returns `0.0` for any other key, as today.
11. **No behavior change.** No config field, catalog key, RNG draw, wire
    type or frontend file changes. Every `deterministic` value of the gate
    summary equals the reference's. The sensor-census gauges,
    `last_mesh_world_reads` and the window records' `uses_*` flags carry the
    same values. The four snapshot types keep their fields, so runtime, trace
    and server code that reads them is untouched.
12. **Overlap.** T29.F01 and T30.F02 also change `creature/state.rs`; at plan
    time neither has a spec; concurrent sessions reconcile that file at
    integration (track note).

## Architecture

Owners touched: `contracts` (a sensor descriptor: the requirements value and
the key family classification, beside `compound_width`), genome (one new
analysis module), `sensors` (the views it defines), `creature` (builds one
view; its birth caches call genome analysis), the scheduler (collects the
views once per tick; seeding's fixture hook), readers and outer crates
(import paths only). Debts paid at
closure: G9 and G20 deleted from section 5 and the debts file; G14 reduced
to its inheritance-witness half (T25.F11). Debts used and left: G2, used in
place and not widened (invariant 2) because the outcome-channel vocabulary
is T25.F11's move; G7 and G8 in `creature/state.rs`, untouched; G24 and
G26, so the analysis module and the reader stay under `creature/` until
T30.F07 moves them. Shared rules read, not changed: the body's sensor denominator
(`HeritableBody::energy_scale`, `creature/body.rs:113`) and the seen colour
(`creature::colour::unit_rgb`, `colour.rs:85`), resolved into the view by
the creature record, their owner; sensors reuse the resolved values. The
desired state in `docs/architecture.md` already describes this feature's
shape (section 3's sensors prohibition, section 1 rule 5 on derived data), so
the plan commit does not edit it.

## Telemetry

Not applicable: the feature changes nothing a run does. The three sensor
census gauges and the window records' perception flags keep their names,
sources (the birth caches and `last_mesh_world_reads`) and values.

## Verification Plan

**Question.** This feature claims every creature perceives and acts exactly
as before while sensors read views and the genome analysis computes the
caches; a gate `deterministic` value differing from the reference, or a
moved sensor test value, would show it wrong.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate` | tick-loop code changes (sensor assembly, birth caches) with no population reading in question: the gate pins the deterministic blocks and the compute counters. |
| Slow tests (`make rust-test-slow`) | `run` | `v3-lab/src/readings.rs` imports the moved census, and the workflow runs the tier for any change to `crates/v3-lab`; no operator, draw, catalog or recruitment change. |

## Implementation Tasks

- [x] `contracts`: `PerceptionRequirements` (`of_key`, union, `ALL`) and the
      `WorldInputKey` family classification, with `is_extended_key`'s test
      moved and a typed-food case added (invariant 5).
- [x] `creature/genome/sensor_analysis.rs`: move the census, the declared
      keys and `DecisionInputKey` from `creature::sensor_census`, the barrier
      reader and live VM reads from `creature::state`, add
      `perception_requirements`; tests move with them; `v3-cli`, `v3-lab`
      and in-crate tests follow the path (R2, R3, R4; invariant 7).
- [x] `BodyView` and `BodyViews` in `sensors/static_inputs.rs`, its genome
      import marked `WORKAROUND(G2)`;
      `assemble_static_inputs` and `assemble_perception` on views; the genome
      predicates, `is_extended_key` and `StaticInputs::resolve_world` deleted;
      sensors tests build views, the equality tests
      (`assemble_perception_matches_reference_reducers`,
      `own_colour_equals_what_a_neighbour_sees`, the nearby vitals cases)
      kept on views (R1, R6; invariants 1–3, 10).
- [x] `creature/state.rs`: `cached_perception_requirements`, `sensed_body`,
      caches through the analysis module, the four helpers deleted
      (invariants 2, 6, 8).
- [x] `simulation/seeding.rs`: the per-founder genome hook; the perception
      mix edits before construction; `refresh_sensor_caches` deleted; the
      `world_with` fixture of `tick/tests/full_sensor_inputs.rs`, which today
      assigns `creature.genome` after seeding, gives its genome through the
      hook so the caches match it (R5; invariant 9).
- [x] `simulation/tick.rs`: the views built once per assembly call, one
      requirements closure, trace flags from the cache; `benches/tick.rs`,
      `neighborhood/opportunity/discovery.rs`, `v3-server/src/http/creature.rs`
      and `v3-telemetry` on the new APIs (invariants 4, 6).
- [x] `scripts/architecture-debts.txt`: G9 and G20 lines and the G14
      `creature/state.rs` line removed; `make check-docs` architecture check
      green (invariant 1).
- [ ] At closure: G9 and G20 rows deleted from `docs/architecture.md`, G14
      reduced to the witness; the gate run and its summary recorded.

## Verification

- [x] `cargo test -p v3-core --test viability` -> 30 passed (run first).
- [x] Focused tests: the views equality tests in `sensors/reducers.rs` and
      `sensors/static_inputs.rs`, the contracts classification test, the
      moved analysis tests, `perception_mix_*` in `simulation/seeding.rs`
      (the caches-consistent case replacing
      `perception_mix_refreshes_the_birth_sensor_caches`),
      `tick/tests/full_sensor_inputs.rs` -> 96 passed.
- [ ] `make check` -> `<result>`; `scripts/architecture-check.mjs` passes
      with the debts file of invariant 1; `make rust-test-slow` at Close ->
      `<result>`.
- [ ] Gate: summary stored at
      `docs/progress/features/t30-f03-sensors-read-views.json`, local raw
      hash and byte count and verification time checked, series entry points
      to the summary, no new full report staged; every `deterministic` value
      equal to the reference's. Goal: `Not run: Verification Plan`.

## Performance and Goal Impact

**Predeclaration — written before the run.** A refactor with no mechanism
and no natural analog to name. Expected cost: none. The per-tick `BodyViews`
build is one ~70-byte value per living creature and replaces the per-creature
`unit_rgb` and `energy_scale` calls static assembly makes today and the
per-target ones the nearby banks make; the two per-creature-per-tick genome
walks of the requirement predicates become one cached read. Reference: the
gate's selected reference, the last measured closure's summary; thresholds:
the gate's own. Expected direction: the per-creature-tick work counters
identical, since no sensor work is counted; wall-clock neutral within the
gate's noise, a regression there attributed before closure. No indicator can
move; nothing is re-pinned.

**Measured verdict.** `<pending>`

- Summary of the declared profile:
  [gate](../../progress/features/t30-f03-sensors-read-views.json).
- Full readings: [`docs/progress/readings/t30-f03.md`](../../progress/readings/t30-f03.md).

## Success Criteria

- [ ] Sensors' production imports are those of invariant 1, and
      `scripts/architecture-debts.txt` holds no G9 or G20 line and no G14
      `creature/state.rs` line.
- [ ] The sensors, analysis and seeding tests of the Verification section
      pass, and `make check` exits 0.
- [ ] The gate's `deterministic` values equal the reference's.
- [ ] `docs/architecture.md` section 5 no longer lists G9 or G20, and G14
      names the witness only.

## Notes for AI Agents

- Decision: in-feature refactors R1–R6 of the preparatory audit in
  `docs/progress/readings/t30-f03.md` are this feature's scope (the views,
  the requirements value and cache, the analysis module, the creature record's
  helpers, the seeding hook, the dead arm).
- Deferred: R7, the `OutcomeChannel` import the body view needs, stays at
  G2's one sensors site (`static_inputs.rs`), marked `WORKAROUND(G2)`;
  T25.F11 pays it.
- Decision: the requirements are computed over every node, as today's
  predicates are; deriving them from the reachable set is a later row's
  behavior question.
- Cost: see the readings file at closure.
