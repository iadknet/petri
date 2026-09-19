# T17.F02 — Unit-Scale Introspection

**Status**: In Progress
**Last updated**: 2026-09-19
**Feature**: T17.F02
**Track**: [T17 — Brain Boundary Evolvability](../../roadmaps/t17-brain-boundary-evolvability.md)

## Goal

A creature feels how full, how tired, and how old it is as a share of itself.
`EnergyCurrent` and `EnergyConsumedThisTick` reach the brain as fractions of
`max_energy` in [0, 1]; `AgeTicks` reaches it as `min(age /
age_reference_ticks, 1)` on a new lifecycle config field; `Generation` leaves
the input key set. Every founder profile's energy gate is re-expressed on the
unit scale and its age gate on the configured span, so a founder-only world
runs the same trajectory as before. A `Threshold` or `Constant` drawn in
[−1, 1], a ±0.1 parameter step, and the Covariance rule now act on these
inputs at the scale they were designed for.

## Non-Goals

- No change to the reproduce transfer, its floor, or the profile fractions
  (T17.F01); no change to the steal amount (T17.F03).
- No change to any mutation operator, parameter draw range, or step (the
  input catalog only loses its `Generation` bucket, invariant 3); no new
  sensor, operator, assay, founder layout (T18), or environmental pressure.
- No age reference other than a config field: T03.F07's lifespan replaces it
  later. No coupling to `age_cost.age_cap`, which is the cost ramp's own knob.
- No load-time validation that a non-default lifecycle keeps a founder's unit
  gate inside physiology's acceptance region (`SimulationConfig` has no
  error-returning validation path; the constraint is documented, invariant 7).
- No compatibility shim: a serialized genome that names `Generation` no
  longer loads (the repository holds none).

## Inputs and Invariants

Sources of truth: the T17.F02 row and the track's "Order" and "Hazard" notes;
the [reproduction-collapse research note](../../strategy/reproduction-collapse-research-2026-09-18.md)
Section 8 (the boundary audit); T17.F01's profile table and invariant 5
(thresholds 32/32/60/40/50 since F01, so the row's "30 → 0.15" reads
"32 → 0.16"); `runtime/inputs.rs::resolve_input` (the VM path passes
`effective` energy, which can be negative, and `energy_consumed + debt`);
`sensors/static_inputs.rs`; `creature/cgp_founder.rs`;
`mutation/sampling.rs`; `config/simulation.rs::EnergyLifecycleConfig`.

Options for `Generation`, decided here: removal. It has no natural analog (a
body cannot sense its ancestor count), it is constant over a life so as an
input it can only be a constant gate — the audit's 14-of-59 lesion class —
and a saturating fraction would need a reference with no physical meaning.
Options for the age span: a new `age_reference_ticks` field (chosen; the
row's "configured reference span"), or reuse of `age_cost.age_cap` (rejected,
Non-Goals). Default 500 ticks: the physiology's senescence horizon
(`age_cost.age_cap` 500), above the surviving clades' mean ages (36–57 ticks
in the T17.F01 goal summary).

Founder re-expression is exact under f32 division, checked by scanning every
f32 energy in [t − 1, t + 1] for each profile: `e > t` and `e / 200 > t / 200`
agree everywhere for 32, 40 and 50; for 60 they disagree at the single value
60.000004 (one ulp above 60; the quotient rounds down onto 0.3). The age gate
`a / S > 19.5 / S` agrees with `a > 19.5` at every integer age for S in
{500, 1000, 1024, 2000, 2048, 4096}.

Invariants:

1. Values. `EnergyCurrent = clamp(energy / max_energy, 0, 1)`;
   `EnergyConsumedThisTick = clamp(consumed / max_energy, 0, 1)`;
   `AgeTicks = min(age / age_reference_ticks, 1)`, computed as f32 division
   by the config value (not multiplication by a stored reciprocal). The VM's
   mid-dispatch read keeps its semantics (`effective` energy and owed debt)
   before the division; a negative `effective` reads 0.
2. Denominators come from `config.energy.lifecycle.max_energy` and the new
   `config.energy.lifecycle.age_reference_ticks` (`u64`, serde default 500,
   `normalize` maps 0 to 500), threaded from the tick loop to the resolve
   path (through `SensorSnapshot`/`StaticInputs` or `ResolveCtx`; the
   implementer chooses) with no second copy of a default in runtime code.
   `PerceptionConfig::max_energy` stays the vitals reducer's own copy.
3. `StaticIntrospectionKey::Generation` is deleted; the sampling catalog
   loses its entry and the draw range shrinks accordingly; the key set in
   `mutation/input_ref/mod.rs`, the test sources, and the frontend fixtures
   that name it follow (the compiler finds the rest).
4. Founder energy gate: `founder_reproduce_policy` carries a unit threshold
   per profile equal to the F01 raw threshold divided by 200 —
   V3Alpha1/ForageFirstSparse 0.16, Conservative 0.30, RichOffspring 0.20,
   Balanced 0.25 — as the founder's own constant (the founder gates on
   fullness; it is not recomputed from `max_energy` at seeding). Each
   literal is the f32 nearest the quotient the scan used; division is
   monotone, so the scanned window covers the range.
5. Founder age gate: CN1 becomes `Threshold((min_reproduce_age − 0.5) /
   age_reference_ticks)` with both values from config at seeding, so the
   gate is exact for every integer age (evidence above) and the existing
   `min_reproduce_age` threading is unchanged. It is config-derived, unlike
   the energy gate, because physiology refuses an attempt below
   `min_reproduce_age` and a refused attempt costs the founder its tick.
   Constraint, same class as invariant 7: `min_reproduce_age ≤
   age_reference_ticks`. Above it the threshold exceeds the saturated 1.0
   and the founder never attempts a birth; `v3-runtime-config-spec.md`
   states this beside `age_reference_ticks`, and the span is not normalized
   up (a span shorter than the breeding age is a sterile life history the
   T03.F07 lifespan must be able to express). The exactness claim holds
   within the constraint; the exactness test carries one case above it
   asserting no attempt.
6. Founder-only identity: with mutation disabled and default lifecycle, every
   founder profile makes the same gate decision at every f32 energy in
   [0, `max_energy`] and every integer age as before, except Conservative at
   exactly 60.000004 (listed, accepted as a 1-ulp artefact of f32 division).
   A V3Alpha1 founder-only digest (the viability scenario's configuration:
   32×32, 10 founders, full coverage, seed 2026, 2,000 ticks,
   `mutation_probability` 0 and `per_unit_supply_enabled` false; births,
   final population, and a world digest) pinned before the change is
   unchanged after it.
7. Founder acceptance (T17.F01 invariant 5) holds at default lifecycle by
   the same arithmetic (0.16 × 200 = 32). A lifecycle where `unit_threshold ×
   max_energy − 1.0 < min_reproduce_energy` pins the founder at its gate (the
   track hazard); `v3-runtime-config-spec.md` states the constraint beside
   `max_energy`, and T18.F02 discharges it for the default founder.
8. Range and monotonicity are pure invariants: for every finite `energy`,
   `consumed`, `age`, `max_energy ≥ 1`, `age_reference_ticks ≥ 1`, each value
   is in [0, 1] and non-decreasing in its numerator (property tests).
9. Determinism: same seed, same config, same trajectory. Every
   mutation-on trajectory moves from the first birth that draws an
   introspective reference or parameter; pinned trajectories are re-pinned
   and listed in the readings file with old and new values.
10. Reference docs describe the state after this feature: `v3-sensor-spec.md`
    Sections 3.2–3.3, `v3-runtime-config-spec.md` (new row, hazard
    constraint), `v3-startup-seeding-spec.md` Section 5.1 (unit thresholds),
    `v3-mutation-spec.md` (catalog and key-set text naming `Generation`).

| Profile | Raw threshold (F01) | Unit threshold | Scan mismatches in [t−1, t+1] |
| --- | ---: | ---: | --- |
| V3Alpha1 (default) | 32 | 0.16 | none |
| ForageFirstSparse | 32 | 0.16 | none |
| ForageFirstSparseConservative | 60 | 0.30 | 60.000004 (gate off on the unit scale) |
| ForageFirstSparseRichOffspring | 40 | 0.20 | none |
| ForageFirstSparseBalanced | 50 | 0.25 | none |

## Implementation Tasks

- [x] Viability baseline run first; founder-only (mutation-off) digest
      pinned before any production change (invariant 6).
- [x] `age_reference_ticks` on `EnergyLifecycleConfig` with default,
      normalization, and config-spec row (invariants 2, 7).
- [x] Unit-scale resolution of the three introspective values on both the
      graph and VM read paths (invariants 1–2): `StaticInputs` carries
      `max_energy` and the age fraction from the tick loop's lifecycle config;
      `resolve_input` divides both energy reads by it.
- [x] Delete `Generation`; update the sampling catalog, key sets, census and
      analysis code, and test sources that named it (invariant 3).
- [x] Founder profiles on the unit scale (invariants 4–5); F01's acceptance
      proptest and the profile-table pin updated to the unit thresholds.
- [x] Gate-scan test per profile and the range/monotonicity property tests
      (invariants 6, 8); the one `proptest-regressions/` file a red run
      created (`simulation/actions/mod.txt`) is kept.
- [x] Re-pinned mutation-on tests and fixtures listed in the readings file;
      the mutation-off pin unchanged.
- [x] Reference docs per invariant 10 (`v3-mutation-spec.md` never named
      `Generation`, unchanged); frontend fixtures updated.
- [x] `make check` -> pass.
- [ ] Invariant 5 constraint: config-spec text beside `age_reference_ticks`
      and one `min_reproduce_age > age_reference_ticks` case in
      `founder_age_gate_is_exact_at_every_integer_age` asserting no attempt.

## Verification

- [x] `cargo test -p v3-core --test viability` -> 27 passed before the
      change; 28 passed after (the founder-only digest pin added).
- [x] Founder-only digest pin: `viability.rs::founder_only_trajectory_digest_is_pinned`,
      digest `5ad9e8e1…de36a9` (177 births, final population 0) before and
      after, identical; tabled in `docs/progress/readings/t17-f02.md`.
- [x] Gate-scan test: `founder_energy_gates_scan_identically_on_the_unit_scale`
      finds the mismatch sets tabled above (empty ×4, `[60.000004]` for
      Conservative); `founder_age_gate_is_exact_at_every_integer_age`;
      property tests `energy_fraction_is_bounded_and_monotone` and
      `age_fraction_is_bounded_and_monotone`; all named in the readings file.
- [x] `cargo test --workspace --no-fail-fast` -> ok (v3-core 1579 lib
      tests, 2 ignored; viability 28); `make check` -> exit 0 on the build
      commit. Re-pinned tests and fixtures tabled in the readings file with
      old and new values.
- [x] Simplification pass (test code only, no pinned value moved): one
      profile table in `creature/founder.rs` carries raw and unit
      thresholds; `tests/temporal_fixtures.rs` derives its age constants.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      and every survivor resolved as killed, equivalent, or deferred. The full
      survivor list stays here.
- [x] Benchmark summaries stored at
      `docs/progress/features/t17-f02-unit-scale-introspection.json` and
      `-goal.json`; local raw sha256/bytes re-checked (`verified_local`);
      series entries point to the summaries; no full report staged; goal
      run once (2026-09-05 decision); details in the readings file.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: interoception —
a body reports fullness as a share of its capacity (stretch and satiety
signals, not a calorie count) and age as a share of its expected span. It
reaches creatures only through the values the body already hands the brain;
no sensor, operator, or reward is added.

Expected compute cost: none from the arithmetic (one division and one clamp
per introspective read); every work counter can move through the trajectory,
since a drawn `Threshold` on energy or age is no longer constant and a
Covariance edge on `EnergyCurrent` no longer saturates in one tick.

References. Gate (gate-v1): epoch `remove-complementary-nutrition.json`,
latest closure `t11-f23-scale-relative-vm-constant-steps.json`. Goal
(goal-worlds-v1): epoch and latest closure
`t11-f23-scale-relative-vm-constant-steps-goal.json`. Standard +10%/+50% work
and +25%/+100% wall flags. Epoch re-pin, as the row predeclares: the
goal-worlds epoch is re-pinned to this feature's goal summary in the closing
commit; the gate epoch only if the user accepts a severe gate result. A
severe work counter on either profile, or an extinction in any goal world, is
a user decision under the blocker rule. Wall-clock moves are flag-only.

Before/after readings the row and track criterion 3 require, from the stored
summaries: the founder's gate `per_creature_tick.births` (before 0.026042);
the founder neighborhood `mutational_neighborhood.founder.births.any_events`
changed/dead fractions (before 0.395238 / 0.000000); and the T11.F14 rows of
each goal world's `case_readings` (before = the T11.F23 goal summary; both
sides tabled in the readings file). The survey's paired-perturbation probe is
not in the repository (T16.F01 deferral) and is not taken here.

| Indicator | Predeclared direction |
| --- | --- |
| Gate: `births` per creature-tick, per-seed `births` and `final_population` | move (mutation-on trajectory); no sign; standard thresholds |
| Gate: `vm_steps`, `mesh_hops`, `graph_relax_iters`, `plasticity_updates`, `actions_applied` | no direction; standard thresholds; severe is a user decision |
| Founder-only (mutation-off) digest, a test not a bench indicator | unchanged (invariant 6) |
| Goal: `births` per creature-tick, `final_population`, `plateau_population`, `mean_energy`, per world | move; no sign; recorded |
| Goal: T11.F14 founder and evolved changed / silent / dead per mutated birth | move; no sign (a param step on an introspective edge is now behavior-relevant, which can raise changed and dead alike); recorded |
| Goal: `neighborhood_read_silent`, drift depth, lineage diversity, recruitment | no direction; recorded |
| Goal: sensor census count of `Generation` references | 0 by construction |
| Goal work counters | no direction; standard thresholds; severe is a user decision |

**Measured verdict** (benchmark specialist, 2026-09-18, `ae5eef80`, each
profile once). Gate: `make` exit 0, `cli_exit` 0, `severe`
**false**, every counter and wall `ok`. Goal: `make` exit **2**, `cli_exit`
**3**, `severe` **true**: `plasticity_updates` 0.049588 → 0.088735 per
creature-tick, **+78.944503%** (per world in the readings file);
the other five counters and wall `ok`; no extinction; neighborhood caps met
(1.503 s of 180 s, 0.225 s of 10 s). Predeclared readings, before → after: gate `births`
0.026042 → 0.025998; gate founder changed/dead 0.395238/0 → 0.414286/0;
goal founder changed of 210 (Orchards/Canyon/Confluence) 91/83/91 →
96/87/96; goal evolved changed 0.2709/0.2696/0.3320 → 0.3947/0.3086/0.3570,
dead 0.0431/0.0006/0 → 0.0228/0.0163/0; `Generation` string count 0 in the
raw goal report. Every goal case reports `inputs_changed: true` (the
serialized `age_reference_ticks` moved the config digests).

- Summaries: [gate](../../progress/features/t17-f02-unit-scale-introspection.json),
  [goal](../../progress/features/t17-f02-unit-scale-introspection-goal.json).
- Full readings: [`docs/progress/readings/t17-f02.md`](../../progress/readings/t17-f02.md).

## Success Criteria

- [ ] `EnergyCurrent`, `EnergyConsumedThisTick`, and `AgeTicks` resolve in
      [0, 1] on both read paths from the configured denominators;
      `Generation` no longer exists as a key.
- [ ] Every founder profile carries the tabled unit threshold and the
      config-derived age threshold; the founder-only digest is unchanged and
      the gate-scan mismatch sets match the table.
- [ ] The gate and goal summaries are stored, the before/after readings are
      tabled, and the goal-worlds epoch is re-pinned in the closing commit.
- [ ] Reference docs describe the unit-scale boundary and the hazard
      constraint.

## Notes for AI Agents
- Decision: 2026-09-19, the user accepted the severe goal `plasticity_updates` (+78.9%, `severe=true`, `cli_exit` 3) as this feature's predeclared cost; the goal-worlds epoch is re-pinned to `t17-f02-unit-scale-introspection-goal.json` in the closing commit.
