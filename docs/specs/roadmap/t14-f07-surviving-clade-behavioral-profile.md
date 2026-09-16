# T14.F07 — Surviving-Clade Behavioral Profile

**Status**: In Progress
**Last updated**: 2026-09-15
**Feature**: T14.F07
**Track**: [T14 — Runtime Telemetry and Report Integrity](../../roadmaps/t14-runtime-telemetry-and-report-integrity.md)

## Goal

Each founder clade with at least one living creature at the end of a benchmark
run has one stored row saying how its survivors made their living: how many
there are, their mean energy, age, generation and genome size, the eats they
applied by food type, the actions they attempted by type, and the predation
kills they made and hits they absorbed. On Orchards, Canyon and Confluence, the
horizon count of surviving clades becomes a table in which two clades that eat
different food, move differently or prey while others graze read as living
differently, and two that do the same thing read as the same.

## Non-Goals

- Any new mechanism, charge, rate, default, threshold or RNG draw. The feature
  counts what Phase 2 already applies and reads it once at the end of a run.
- Per-lineage cumulative telemetry inside `WorldTracking` or `SimStats` keyed
  by founder id. The audit refuted it because those structs source world
  totals by design and because 10,000 founders is not a bound; this artifact
  is bounded by surviving clades and built only at the terminal read.
- Rows for extinct clades, profiles of dead creatures, or any read at death.
  A creature's counters leave with it; T14.F09 owns clade duration.
- Strategy names, ecotype descriptors, strategy counts, niche overlap and
  ecological diversity measures derived from these rows (T01.F04, T01.F06);
  trophic edges by clade, predator clade by prey clade (T05.F03).
- Rows on the persistence checkpoints. The block is terminal-only, like every
  T14.F02 transferred block.
- New goal indicators, thresholds, comparison-block entries, and the
  no-regression rule's coverage set. This is a stored reading, not an indicator.
- Exposing the new per-creature counters over the server payload or frontend
  protocol; `docs/progress/index.html` presentation (T14.F11).
- Rewriting historical reports: a report stored before this block reads it as
  absent, never as an empty table.

## Inputs and Invariants

Sources of truth: `crates/v3-core/src/creature/state.rs` (`CreatureState`, its
two constructors, the existing `lifetime_action_attempted_count`,
`lifetime_blocked_move_count`, `offspring_spawned_count`),
`crates/v3-core/src/creature/action_log.rs` (`ActionType`, `ACTION_TYPE_COUNT`
= 5, `ActionType::as_key`), `crates/v3-core/src/simulation/tick.rs` (the five
`execute_*` functions of Phase 2, each of which increments
`lifetime_action_attempted_count` once), `crates/v3-core/src/simulation/actions/predation.rs`
(`apply_steal_energy`, `PredationActionResult`),
`crates/v3-core/src/creature/identity.rs` (`lineage_id: u32` = founder index),
`crates/v3-cli/src/bench/tracking.rs` (`WorldTracking`,
`with_transferred_counters`, `by_food_type`, `MortalityTracking` as the
definition-token precedent), `crates/v3-cli/src/bench/run.rs` (`run_one_seed`,
the single terminal `with_transferred_counters` site), the T14.F02 spec's
determinism constraints, and the track's F07, pull-forward and "Not in this
track" notes.

**Options considered.** (a) Per-creature lifetime counters bucketed by
`lineage_id` at the terminal read — the track note's design and the one taken.
(b) A per-lineage map accumulated during the run — refuted by the audit and
bounded by founders, not survivors. (c) Replaying `sim.action_logs` at the end —
`ActionLog` is a capped ring buffer whose capacity can be zero, so lifetime
totals are not recoverable from it. (a) is the only bounded, complete route and
adds per-action work of one integer increment.

**Per-creature counters.** `CreatureState` gains, beside the existing lifetime
fields and zeroed in both constructors:

| Field | Incremented at | Meaning |
| --- | --- | --- |
| `lifetime_actions_attempted_by_type: [u64; ACTION_TYPE_COUNT]` | the same line as `lifetime_action_attempted_count += 1` in each of the five `execute_*` functions, indexed by `ActionType as usize` | attempts by type; the array's sum always equals `lifetime_action_attempted_count` |
| `lifetime_eats_applied_by_type` (indexed by `OrdinaryFoodTypeId`, sized by the run's `config.world.food.types.len()`; a fixed-length or lazily grown vector, never a map) | the `succeeded` branch of `execute_eat`, the same branch that increments `eat_actions_applied_total_by_type` | applied eats by food type; its sum never exceeds `lifetime_actions_attempted_by_type[Eat]` |
| `lifetime_predation_kills_count: u64` | the attacker, in the kill branch of `apply_steal_energy` beside `predation_kills_total += 1` | kills this creature made |
| `lifetime_predation_hits_taken_count: u64` | the victim, in the victim-survived branch of `apply_steal_energy` beside the `Transferred` result | transfers this creature absorbed and outlived; a killed victim is removed inside the kill branch and carries nothing forward |

A child starts at zero on every counter; nothing is inherited, unlike
`shared_memory`. All four are touched only in the sequential Phase 2 path, never
inside the parallel mesh phase, so no order-dependent sum is introduced.

**The terminal table.** `WorldTracking` gains
`surviving_clade_profiles: Option<SurvivingCladeProfiles>` with the T14.F02
serde shape (`#[serde(default, skip_serializing_if = "Option::is_none")]`),
filled only by `with_transferred_counters` so it appears once per case in the
end-of-run block and never in a checkpoint sample.

| Field | Content |
| --- | --- |
| `definition` | `"surviving-clade-profile-v1"` |
| `rows` | one row per founder `lineage_id` with at least one living creature, ascending by `lineage_id`; empty at extinction (`[]`, not absent) |
| row `lineage_id` | founder index, `u32` |
| row `size` | living creatures in the clade; the sum over rows equals the final population, and the row count equals the last checkpoint's `surviving_founder_clade_count` |
| row `mean_energy`, `mean_age`, `mean_generation`, `mean_genome_size` | six-decimal strings via the existing `six` formatter; energy is `f32` summed in `f64` in the same `creatures.values()` order the existing `mean_energy` sample uses, the other three are integer sums divided once |
| row `eats_by_type` | `Vec<u64>` of length `food_type_count`, indexed like `typed_eats_total` |
| row `actions_by_type` | `BTreeMap<String, u64>` keyed by `ActionType::as_key`, all five keys always present |
| row `predation_kills`, `predation_hits_taken` | `u64` sums of the two predation counters |

Bucketing uses a `BTreeMap<u32, _>` or a sort by `lineage_id`; no `HashMap`
reaches the report. The pass is one walk over the living population, reads no
genome (genome size is `cached_genome_size`), executes no brain, and consumes no
RNG. Seeded runs stay byte-identical across processes and thread counts, which
the gate profile's two-run check inside `make check` covers.

**Row bound.** Surviving clades at the T03.F11 goal horizon are 27, 28 and 17
on the three worlds; the track note's historical range is 22–142. A row is
about a dozen scalars plus two short vectors, so the table is well under the
~12 KB per-checkpoint stored figure T14.F10 recorded, and it is emitted once
per case rather than per checkpoint.

**Cross-checks a reader can make** from one stored goal report, per world: the
sum of `eats_by_type` over rows is at most the case's `typed_eats_total`
(survivors' eats are a subset of all applied eats); the sum of
`actions_by_type["StealEnergy"]` over rows is at most
`predation.actions_attempted_total`; `predation_kills` summed over rows is at
most `predation.kills_total`. These are inequalities, not identities, because
dead creatures' counters are gone.

**Observation only.** This feature adds no environmental pressure and does not
invent one for the three standard goal environments; the existing recipes run
unchanged.

## Implementation Tasks

- [x] Add the four per-creature counters to `CreatureState`, zero them in both
      constructors, and increment them at the sites named above.
- [x] Add `SurvivingCladeProfiles` and its row type to `tracking.rs`, the
      `surviving_clade_profiles` field on `WorldTracking`, and its construction
      in `with_transferred_counters`.
- [x] Tests: counter increments at each Phase 2 site including the failed-eat,
      rejected-steal and kill branches, child counters start at zero, the
      by-type/attempted sum invariant after a mixed action sequence, a
      two-clade fixture producing sorted rows whose sizes sum to the population,
      and the extinction case yielding `rows: []`.
- [x] The bucketing is a pure function over an iterator of per-creature
      readings (`lineage_id` plus the counted fields) with a proptest in
      `v3-cli`: rows ascending and unique by `lineage_id`, one row per distinct
      input lineage, sizes summing to the input count, and each row's integer
      sums equal to the sums of its members' inputs, for any drawn input.
- [ ] Confirm every checkpoint sample's key set is unchanged and the new block
      is present in each case's end-of-run block on both profiles.

## Verification

- [ ] `make check` -> exit 0 on the final feature code; commit named here.
- [ ] Focused tests: `cargo test -p v3-core -p v3-cli` -> exit 0, test names
      and counts in the readings file; `cargo clippy -p v3-core -p v3-cli
      --all-targets` and `cargo fmt --all -- --check` -> exit 0.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path
      under `~/.local/share/petri-tools/mutants/t14-f07/`, and every survivor
      resolved as killed, equivalent, or deferred. The full survivor list stays
      here.
- [ ] Stored goal report: for each of the three worlds, the row count equals
      the horizon checkpoint's `surviving_founder_clade_count`, the sizes sum to
      the final population, every row's `actions_by_type` carries five keys, and
      the three inequality cross-checks above hold; no checkpoint sample carries
      `surviving_clade_profiles`. jq transcript and the per-world row table in
      `docs/progress/readings/t14-f07.md`.
- [ ] Benchmark summaries stored at
      `docs/progress/features/t14-f07-surviving-clade-behavioral-profile.json`
      and its `-goal` companion, raw reports left in the main checkout's ignored
      `.bench-artifacts/t14-f07/`, series entry pointing at the summaries, no
      full report staged.
- [ ] Second goal run: `Not applicable`: the goal profile runs once per closure
      under the one-goal-run rule.

## Performance and Goal Impact

**Predeclaration — written before the run.** Measurement feature; the track's
observation contract exempts it from the natural-analog rule and it adds no
mechanism. Both profiles compare against the epoch baselines the series index
names, under the existing thresholds.

Expected compute cost: negligible. Per applied action one extra integer
increment beside the one already made; per birth at most one small allocation
for the eats vector, beside the genome clone and three boxed slices a birth
already allocates; one `O(population)` bucketing pass per seed at the end of
the run, on a population of roughly 10,000–14,000. Predeclared direction for
every goal indicator — lineage diversity, memory sensitivity, temporal memory
sensitivity, mutational neighborhood, neighborhood read, drift depth,
population persistence, births per 100 ticks — is **none**: the feature changes
nothing the simulation applies, so every indicator and every checkpoint sample
is expected to be byte-identical to the T03.F11 goal report apart from the new
block, and any movement is a defect rather than a result. A severe compute
regression is a blocker to report, not a cost to justify, and no epoch re-pin
is authorized. Stored report growth is bounded by surviving clades: about one
row per clade, once per case.

Goal impact: the master success definition's "many coexisting ways of making a
living" becomes readable as differentiation rather than as a count, and if any
surviving clade's row shows predation kills, that is the track's trigger to
schedule T05.F03.

**Measured verdict.** Written after the runs: one line per profile with CLI and
observed exit statuses, the `severe` flag, whether any threshold was crossed,
and whether the epoch was re-pinned.

- Summaries: [gate](../../progress/features/t14-f07-surviving-clade-behavioral-profile.json),
  [goal](../../progress/features/t14-f07-surviving-clade-behavioral-profile-goal.json).
- Full readings: [`docs/progress/readings/t14-f07.md`](../../progress/readings/t14-f07.md).

## Success Criteria

- [ ] Every living creature carries lifetime attempts by action type, applied
      eats by food type, predation kills and hits taken, incremented only in
      sequential Phase 2 and zero at birth.
- [ ] Each goal case's end-of-run block carries `surviving_clade_profiles`
      with one row per surviving founder clade, sorted, sizes summing to the
      final population; checkpoint samples are unchanged.
- [ ] Both benchmark profiles pass with `severe=false`, every indicator
      unmoved, and no epoch re-pin.
- [ ] The readings file shows the per-world row table with the cross-checks.

## Notes for AI Agents

- Decision: `predation_hits_taken` replaces the track note's "losses" because
  the only loss a living creature can carry is a transfer it survived; a
  clade's dead victims are T14.F09's duration reading and T05.F03's edge
  content, not this row.
- Decision: the per-creature predation counters are created, not transferred —
  none existed — at the single sequential site in `apply_steal_energy`, the
  same rule T14.F02 used for its two created counters.
