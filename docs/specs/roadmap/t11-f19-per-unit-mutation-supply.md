# T11.F19 — Per-Unit Mutation Supply

**Status**: In Progress
**Last updated**: 2026-09-14
**Feature**: T11.F19
**Track**: [T11 — Brain Genotype-Phenotype Map](../../roadmaps/t11-brain-genotype-phenotype-map.md)

## Goal

Every `genome_size()` unit a parent carries is an independent chance of one
mutation event at birth: the requested event count is
`Binomial(genome_size(), rate)` with no minimum, maximum, or continuation
rule, so a larger genome pays its size in exposure while the two-node founder
(111 units) keeps about 0.55 requested events per birth (0.005 × 111 = 0.555). Natural analog: per-base
copy error, the fidelity of a replicating polymerase being a property of each
site copied, so a genome's mutation load is its size times its rate (Drake
1991; Sung, Ackerman, Miller, Doak, and Lynch 2012). It reaches creatures
through the body only: how much structure a parent carries sets how many
events its offspring receive. The per-birth rule is retired from production
and kept as a disabled legacy rule that the drift walk and T11.F13 use as the
fixed-count control.

## Non-Goals

- No cap, pruning rule, or new cost on structure: the bounds are T03.F08's
  carrying cost and the load itself; T03.F11 (replication cost) follows this
  feature and depends on it.
- No change to the layer split (`mesh_layer_probability` 0.2 per event), to
  targeting (`executed_bias` 0.9 with the uniform-per-node residual; the
  unit-weighted node draw is not adopted), to operator weights or semantics,
  or to the `genome_size_pressure_enabled` lever.
- No rate characterization: 0.005 is the founder-equivalent anchor, not an
  optimum; T11.F13 owns that reading on this rule, and T08.F05 inherits the
  scalar later.
- No repair of the default plains world's persistence at this revision
  (flagged separately on 2026-09-14).
- No new telemetry counter: `mutation_events_*_total` and
  `reproduction_actions_spawned_total` already give events per birth.

## Inputs and Invariants

Sources of truth: the track's T11.F19 row and its two Notes entries; the
[per-unit mutation supply research note](../../strategy/per-unit-mutation-supply-research-2026-09-14.md)
(Sections 4, 5.1, 5.2, 5.3, 7, 8; artifacts under
`docs/progress/sweeps/per-unit-supply-2026-09-14/`); the T11 track's
2026-09-14 floor amendment; the [T14.F12 spec](t14-f12-neighborhood-read-of-selected-genomes.md)
(closure indicator and its per-world floors); `crates/v3-core/src/mutation/engine/mod.rs`
(`requested_event_count`, `apply_mutations_with_food_type_count`);
`crates/v3-core/src/config/simulation.rs` (`MutationConfig`, `normalize`);
`crates/v3-core/src/neighborhood/{births,drift}.rs`;
`crates/v3-cli/src/bench/{indicators,schema}.rs`; the founder assertion
`genome.genome_size() == 111` in `crates/v3-core/src/creature/state.rs`.

Evidence that fixes the design (the note): at rate 0.005 the founder row is a
no-op (requested 0.56 against 0.54, changed 19.9% against 21.2%, one dead
birth in 2,000 either way); on production-walked genomes at generation 2,000
(5,791 units) the same rate requests 28 events per birth and reads 15.65%
changed and 3.70% dead with `executed_bias` 0.9, against the baseline's 0.45%
and 0.10%; holding topology events on the per-birth rule reads 0.5% changed
at the same count, so the pure per-unit draw is adopted; under drift every
per-unit arm doubles the mesh every 20 to 30 generations and passes 400 nodes
by generation 190 to 240, so the walk cannot run on this rule; under selection
on Orchards with the carrying cost, the biased arm reads 217.0 units in 5.07
nodes at tick 2,000 against the control's 213.3 in 5.06, with population
10,260 against 11,356, and no arm levels off by tick 6,000.

Research summary (2026-09-14). Prior art is the note's Section 4: Avida's
`COPY_MUT_PROB`, Aevol's per-base rates, Markov Brains' per-site points, and
biology's per-base rate all charge mutation per site, each with a size bound
that selection or a window supplies. Local options for the count draw: a loop
of one `gen_bool(rate)` per unit (adopted: exact, uses the existing `rand`
dependency, `O(genome_size())` draws per birth, about 111 to 250 at goal
depth); `rand_distr::Binomial` (rejected: a new dependency T11.F04 already
declined, for a cost the counters below show is negligible); geometric-gap
sampling (rejected: `O(events)` draws but float math for no measured need).
For the retired rule: remove the four fields (rejected: the drift walk must
reproduce today's `drift-depth-v3` rows byte for byte, which needs the exact
legacy draw, and T11.F13 needs it as a control) or keep them behind an
explicit switch (adopted, the T03.F08 precedent for
`genome_size_pressure_enabled`). The switch is a boolean because the runtime
panel has boolean and numeric field kinds and no select control.

Invariants: `attempted_events = applied_events + skipped_events`; requested,
applied, and skipped stay distinct and skipped events are never retried; the
genome is immutable after birth, so `genome_size()` read once at the top of
the birth's engine call is the count's population; with `per_unit_rate` 0.0
every birth is a clone; with the legacy rule selected, every RNG stream and
every row of the pre-feature engine is byte-identical; the walk's structural
rows read the mutation map, never the supply.

Fixed design, decided before implementation:

| Decision | Value |
| --- | --- |
| Config | `mutation.per_unit_supply_enabled: bool`, default `true`, `#[serde(default)]` to `true`; `mutation.per_unit_rate: f64`, default `0.005`, `#[serde(default)]`; finite values clamp to `[0, 1]`, non-finite normalize to `0.005`. Stored configs and recipes that omit both load onto the production rule, which is intended: the goal worlds must exercise it. |
| Count draw | When enabled, `requested_event_count` reads `genome.genome_size()` once, draws `gen_bool(rate)` once per unit, and requests one event per success. No trigger roll, minimum, maximum, or continuation. Rate 1.0 requests one event per unit; rate 0.0 requests none. |
| Legacy rule | `mutation_probability`, `per_birth_mutation_events_min`, `per_birth_mutation_events_max`, and `per_birth_mutation_event_continuation_probability` keep their names, defaults (0.44 / 1 / 10 / 0.2), normalization, and draw code; they run only when `per_unit_supply_enabled` is `false`. They are not enabled in production. |
| Founder pin | A unit test asserts `MutationConfig::default().per_unit_rate * founder.genome_size()` is within 1% of 0.55, where the founder is the canonical V3Alpha1 genome whose size the state test pins at 111 (0.005 × 111 = 0.555). The rate is never a bare literal in a second place. |
| Everything else in the engine | Unchanged: the executed-set derivation, `TargetSets`, the layer split, operator selection, pressure handling, and the event loop run as they do today on the drawn count. |
| Drift walk | `drift::observe` clones the mutation config it is handed and forces `per_unit_supply_enabled = false`, so every walk birth and every checkpoint birth uses the legacy rule at the config's per-birth fields, whose defaults are the founder-equivalent 0.55. Nothing else in the config is overridden. `DriftDepth` gains a `#[serde(default)]` string `supply_rule` recording the rule and the four values in force; `VERSION` stays `drift-depth-v3` because the rows are byte-identical when nothing else changes. |
| Fixture walks | The `recruitment_paths` experiment (T13.F06 goal indicator, `neighborhood/recruitment_paths/experiment.rs`) walks authored lineages with no selection and pins recorded baselines, so it is a mutation-map instrument of the same kind as the drift walk: it builds its proposal config from the default on the legacy rule (`per_unit_supply_enabled` forced `false`) and says so in its `mutation_context` string. Every other instrument that reads genomes the world produced uses the production rule. |
| Neighborhood readings | The founder and evolved halves and T14.F12's neighborhood read pass the production config unchanged, so they read the per-unit supply. |
| Determinism | Every production birth's RNG consumption changes, so the short-run identity hash in `crates/v3-core/tests/baseline_worlds.rs` (`legacy_default_short_run_identity`) is re-pinned once, as at T13.F04, after two runs agree. The two new fields enter every `config_digest`, so the three goal recipes' digests in `crates/v3-cli/tests/bench_artifacts.rs` are re-pinned; the comparator reports the digest change per case as `inputs_changed` and keeps the cases comparable. |
| Frontend | `MutationSection.tsx`: a toggle "Per-Unit Supply" (default on, tooltip naming the Binomial rule) in `MUTATION_TOGGLES`, and a numeric row "Rate / Unit" (min 0, max 0.1, step 0.0001, default 0.005) in `MUTATION_FIELDS`; the four legacy rows stay with tooltips stating they apply only when the toggle is off. `types/config.ts`, `test/fixtures.ts`, and the `ControlBar.test.tsx` fixture carry both fields. |
| Runtime patch path | Both fields are patchable through the existing config-apply route; normalization of an out-of-range rate is reported by field path like the existing mutation fields. |
| Reference docs | `docs/reference/v3-mutation-spec.md` Section 4.1 and the selection randomization rules; `docs/reference/v3-runtime-config-spec.md` mutation table and "Mutation randomization semantics" steps 1 and 2; the supply paragraph in `docs/reference/v3-reproduction-spec.md`; the mutation example in `docs/reference/v3-server-api-protocol-spec.md`. Each states the per-unit rule as production and the per-birth rule as the disabled legacy rule. |
| Tests that need a fixed count | Existing tests that set `mutation_probability` and the min/max bounds to force an exact count (engine, actions, viability, reproducibility, server) select the legacy rule explicitly; tests of the per-unit rule are added beside them. Property tests (proptest) cover: `0 <= requested <= genome_size()`, rate 0.0 requests 0, rate 1.0 requests `genome_size()`, and normalization of the rate. Zero-supply fixtures may select the legacy rule with `mutation_probability` 0.0 (no RNG draw, existing streams preserved) or `per_unit_rate` 0.0; the clone invariant under `per_unit_rate` 0.0 is covered by its own engine test. |

Predeclared readings, taken from the stored closure reports and read against
the T14.F12 gate and goal reports (previous closure), the pinned goal epoch
(`t13-f03-mutation-target-applicability-goal.json`), and the gate epoch
(`remove-complementary-nutrition`):

| Reading | T14.F12 reference (Orchards 11 / Canyon 22 / Confluence 33) | Predeclaration |
| --- | --- | --- |
| Neighborhood read `changed_per_all_births` (closure indicator) | 0.146400 / 0.130000 / 0.143400 (standing floors, strict not-below) | Gate: not below the floor on any world (strict). Direction: up on every world. A world below its floor rejects the default rate, not the mechanism |
| Neighborhood read `dead_per_all_births` | 0.003400 / 0.002200 / 0.003200 (no floor) | Reported; ceiling 0.010000 per world (50 of 5,000). A reading above the ceiling is investigated before closure; if the investigation confirms it, the default rate is rejected |
| Neighborhood read sample depth | mean generation 47.0 / 48.5 / 52.7 | Reported beside the reading, per world |
| Goal persistence | final 11,356 / 9,646 / 9,733; minimum 2,272 / 3,898 / 956; `surviving_founder_clade_count` 24 / 26 / 17 | No world extinct; final population not below 80% of the reference (9,085 / 7,717 / 7,786) and clade count not below 80% (19 / 20 / 13). The track note's direction is "not down"; this spec deliberately turns it into a 20% band, because the note's prototype on the same Orchards run read 10,260 at tick 2,000 (-9.7%) and bloomed earlier afterward, so a strict not-down gate would be predeclared to fail on ecological noise. A world below the band rejects the default rate |
| Goal terminal `mean_genome_size` / `mean_mesh_nodes` (persistence sample at tick 2,000) | 213.343255 / 225.859838 / 246.159252 units; 5.063138 / 5.162865 / 4.993116 nodes | Reported; ceiling 1.5 times the reference per world for both (320.0 / 338.8 / 369.2 units; 7.59 / 7.74 / 7.49 nodes); the note's biased arm read 217.0 in 5.07 at this depth. Above the ceiling is investigated before closure |
| Goal `mutation_supply.events_applied_total` per birth (`births_total` at tick 2,000) | 210,880 / 384,054 = 0.549; 203,442 / 369,038 = 0.551; 198,771 / 359,734 = 0.553 | Up on every world; reported. Expected about 0.8 to 1.0 (the prototype's interval rate at tick 2,000 was 1.01) |
| Founder neighborhood block, gate and goal | 500 births; zero-event 292; changed 88 (0.176 of all) gate, 95 (0.190) goal; dead 0 in both | Unchanged in outcome, not byte-identical (the founder now consumes 111 draws per birth): pooled changed per all births within 0.04 of the reference; dead not above 2 of 500; zero-event births between 250 and 320 (expectation 0.574 × 500 = 287) |
| Drift walk block, per world | `drift-depth-v3`; changed/all 0.004500 / 0.003500, 0.008500 / 0.004500, 0.004500 / 0.003500 at depths 1,000 / 2,000 | Every checkpoint row byte-identical to T14.F12's per world; only the new `supply_rule` metadata string differs. Regression instrument under the no-regression rule; the withdrawn floors are not gates |
| `recruitment_paths` block (goal, one block) | `recruitment-paths-v1` at T14.F12 | `arms`, `pairs`, `starts`, `opportunities`, `constructed`, and `total_proposals` byte-identical to T14.F12's; only `config`, `config_digest`, and the `mutation_context` string differ, because the serialized config carries the two new fields and the string names the legacy rule |
| Evolved half (12 genomes, 200 births) | reported per world | Reported; confounded by the changed population, as every closure since T11.F17 records |
| Observation budgets | workflow caps | Founder neighborhood below 10 s; summed evolved below 180 s; drift walk below 30 s; neighborhood read below 10 s summed; whole goal run below 15 minutes |

## Implementation Tasks

- [x] Add `per_unit_supply_enabled` and `per_unit_rate` to `MutationConfig`
      with defaults, serde defaults, normalization, and the founder pin test;
      route `requested_event_count` through the per-unit draw when enabled and
      the untouched legacy draw otherwise; add the engine unit and property
      tests.
- [x] Force the legacy rule inside `drift::observe`, record `supply_rule` in
      `DriftDepth`, and pin with a test that the walk's births are identical
      whether or not the config it receives enables the per-unit rule.
- [x] Switch existing fixed-count tests to the legacy rule explicitly; add
      server patch and normalization coverage for both fields.
- [x] Frontend: toggle, rate row, types, fixtures, tests.
- [x] Update the four reference documents.

## Verification

- [x] `cargo test -p v3-core --test viability` first (24 passed; 25 once
      the remediation pass adds the per-unit sibling), then
      `make check` (exit 0) -> command tables in
      [`docs/progress/readings/t11-f19.md`](../../progress/readings/t11-f19.md).
- [x] Focused tests: engine count draw (rate 0, rate 1, founder mean, legacy
      byte-identity), config normalization and serde defaults, founder pin,
      drift-walk supply override, frontend panel fields -> test names and
      results (all pass) in the readings file, with the determinism re-pins
      (`legacy_default_short_run_identity` 11753828254793484309 ->
      1397923697343438469, three goal recipe digests).
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      and every survivor resolved as killed, equivalent, or deferred. The full
      survivor list stays here.
- [ ] Benchmark summaries stored at
      `docs/progress/features/t11-f19-per-unit-mutation-supply.json` and
      `docs/progress/features/t11-f19-per-unit-mutation-supply-goal.json`,
      local raw hash/byte count and verification time checked, series entries
      point to the summaries, no new full report staged; every predeclared
      reading above checked against the stored reports and tabulated in the
      readings file, including the field-for-field drift-block comparison.
      Benchmark specialist pass (2026-09-14): stored, hashed, and tabulated
      in the readings file; decisions recorded under Performance and Goal
      Impact; box closes with the closure checklist.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: per-base copy
error; the load reaches creatures through the body, as a count of events at
birth set by the parent's carried structure, with no sensor, reward, or
authored script. Expected compute cost: one `gen_bool` per `genome_size()`
unit per birth (about 111 to 250 draws at goal depth) in place of one or two
draws, and roughly twice the applied events per birth at the goal profile's
depth; at about 0.016 births per creature-tick this is a few RNG draws and
about one extra operator application per hundred creature-ticks. The six work
counters are ecology counters and do not count mutation work; they are read
under the usual thresholds (flag +10%, severe +50%) with no severe budgeted.
Wall time per creature-tick may flag (+25%) from the mutation-time cost and
must not reach severe (+100%); a wall comparison is only made on a matching
host. Gate references: T14.F12 gate and the gate epoch. Goal references:
T14.F12 goal and the pinned goal epoch. No epoch re-pin is budgeted. The
`deterministic` blocks of both profiles differ from every prior report for
every seed, since every birth's RNG consumption changes; the drift-walk and
`recruitment_paths` rows are predeclared identical as tabulated above, and
every goal case's `config_digest` changes (`inputs_changed`, comparable). Expected directions for every
indicator this feature can move are in the readings table; every other stored
indicator (lineage diversity, memory and temporal sensitivity, cognition,
recruitment) is reported with no direction, because the population evolves
under a different supply and any movement is ecology, not this mechanism's
measure.

**Measured verdict.** Gate: not severe (`comparison.severe=false` against both
`remove-complementary-nutrition` and `t14-f12-neighborhood-read-of-selected-genomes`
epochs); the flagged `plasticity_updates` delta is +24.4% against T14.F12
(under the +50% severe threshold). Goal: **severe** — `comparison.severe=true`
against both `t13-f03-mutation-target-applicability-goal` and
`t14-f12-neighborhood-read-of-selected-genomes-goal`; `plasticity_updates`
reads +82.3% and +75.9% respectively, both above the +50% severe threshold,
against a predeclaration that budgeted no severe on the six work counters.
Two predeclared ceilings are also exceeded: Confluence `dead_per_all_births`
0.0104 (ceiling 0.010000) and Orchards terminal `mean_genome_size` 340.944597
units (ceiling 320.0149). Every other predeclared reading (drift-block
field-for-field identity, `recruitment_paths` identity modulo `config`/
`config_digest`/`mutation_context`, `changed_per_all_births` floors, goal
persistence band, `events_applied_total` per birth, founder block, wall-clock
caps) is met. Reported to the orchestrator as a regression and two
threshold exceedances; not remediated here. Full tabulation in
[`docs/progress/readings/t11-f19.md`](../../progress/readings/t11-f19.md).

**Escalation decisions (spec owner, 2026-09-14; readings in the readings
file's "Escalation readings" table; the predeclaration is unchanged).**

1. `plasticity_updates` severe: not the mutation-time cost (the six counters
   do not count mutation work, and the goal wall total fell 3.6% against
   T14.F12). The reference itself spans five times across worlds, so the
   counter tracks which clade dominates, and under the new supply two worlds
   carry a plasticity-bearing lineage where one did before. Read as ecological
   composition; it does not reject the mechanism or the rate. Recommendation
   to the user: accept the observed cost and re-pin the goal-worlds epoch to
   this report, as at T13.F03 on the same counter. Accepted by the user on
   2026-09-14; the goal-worlds epoch is re-pinned in commit f4d9f644.
2. Confluence `dead_per_all_births` 52 / 5,000 against 50: pooled dead across
   worlds is 0.0067, the excess is 0.3 standard deviations, and 13 sampled
   genomes carry every dead birth. Lineage fragility in the sample, not
   supply-wide lethality; not confirmed, so an exceedance, ceiling and rate
   unchanged.
3. Orchards terminal `mean_genome_size` 1.60 times T14.F12's against 1.5: the
   series decelerates (increments 98, 78, 25 units per 500 ticks against a
   drift doubling that would read about 3.6 times by this generation), mesh
   nodes sit under their ceiling, and the executed core is flat while
   scaffold grows, which is the load T03.F11 brakes next. Not runaway; an
   exceedance, ceiling and rate unchanged.

The user confirmed items 2 and 3 on 2026-09-14; the default rate stands.

- Summaries: [gate](../../progress/features/t11-f19-per-unit-mutation-supply.json),
  [goal](../../progress/features/t11-f19-per-unit-mutation-supply-goal.json).
- Full readings: [`docs/progress/readings/t11-f19.md`](../../progress/readings/t11-f19.md).

## Success Criteria

- [ ] Production births request `Binomial(genome_size(), 0.005)` events, the
      founder's requested supply is within 1% of 0.55 per birth, and the
      legacy rule runs only when explicitly selected.
- [ ] The drift walk's checkpoint rows are field-for-field identical to
      T14.F12's per world and its metadata names the legacy rule it ran.
- [ ] T14.F12's `changed_per_all_births` is above its floor on all three
      worlds, and every other predeclared reading is met or its miss is
      recorded with the user's decision.
- [ ] Reference documents, the config panel, and the runtime patch path carry
      the two fields with the per-unit rule as production.

## Notes for AI Agents

- Decision: (user, 2026-09-14) the default rate is the founder equivalent
  0.005 per `genome_size()` unit, not 0.01; no cap or pruning rule is added;
  T03.F11 is sequenced directly after this feature and depends on it.
- Decision: (user, 2026-09-14) the severe goal `plasticity_updates` cost
  (+82.3% against the pinned epoch) is accepted as ecological composition and
  the goal-worlds epoch is re-pinned to this feature's goal summary.
- Decision: (user, 2026-09-14) the depth-1,000 and depth-2,000 drift floors
  are withdrawn as closure gates; the walk stays a mutation-map regression
  instrument and T14.F12's neighborhood read is the changed-births indicator.
- Exception: (spec owner, 2026-09-14; confirmed by the user the same day) Confluence
  `dead_per_all_births` 0.010400 against the 0.010000 ceiling is an
  exceedance, not a rate rejection; applies to this closure only.
- Exception: (spec owner, 2026-09-14; confirmed by the user the same day) Orchards terminal
  `mean_genome_size` 1.60 times T14.F12's against the 1.5 ceiling is an
  exceedance, not a rate rejection; applies to this closure only, and T03.F11
  reads it as the size it brakes.
