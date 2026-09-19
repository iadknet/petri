# T11.F23 — Scale-Relative VM Constant Steps

**Status**: In Progress
**Last updated**: 2026-09-18
**Feature**: T11.F23
**Track**: [T11 — Brain Genotype-Phenotype Map](../../roadmaps/t11-brain-genotype-phenotype-map.md)

## Goal

A mutation nudges a number by a share of what it is, not by a fixed amount.
`VmConstantMutation` (`apply_constant_mutation`,
`crates/v3-core/src/mutation/vm/operators.rs`) moves one constant of a VM
program's pool by `c += u × max(|c|, 1)` with `u` drawn uniformly from
[−0.1, 0.1]: a unit-scale constant moves as a graph parameter does (±0.1,
`graph/operators.rs`), a raw-scale constant moves by up to a tenth of its
magnitude. On the founder's reproduce transfer fraction (2/3, VM constant
index 5 since T17.F01) one event lands in [0.5667, 0.7667], so no single draw
sterilizes the line (fraction ≤ 0) or makes it semelparous (fraction ≥ 1);
today those shares are about 17% and 33% of the slot's draws.

## Non-Goals

- Per-slot or per-use step sizes (rejected in the track note: constants are a
  shared pool and the operator would need program analysis).
- Rescaling the transfer slot or any other boundary value back to an integer
  scale (T17.F02/F03 move the rest of the boundary onto the unit scale).
- Any new operator weight, sensor, or `MutationConfig` value; `VmConstantMutation`
  keeps weight 4 (`mutation/vm/mod.rs`).
- Changing the empty-pool draw (a new constant in [−1, 1]), the graph
  parameter step, `VmCopyConstantBlock`, or any other operator.
- Clamping or sign-restricting constants; sanitizing non-finite constants.
- Changing T13.F05's recruitment-path starting forms, `MAX_PATH_EVENTS`,
  `SEARCH_RANGE`, or any acceptance predicate.
- Famine or population-collapse protection.

## Inputs and Invariants

Sources of truth: the T11.F23 row and the track's "Constant steps,
2026-09-18" note; T17.F01's "Known consequence" paragraph (Inputs and
Invariants) and its goal reading (`founder_changed_per_all_births` fell
against a predeclared rise); `apply_constant_mutation`;
`mutate_compute_parameter`'s ±0.1 steps (`graph/operators.rs`); the founder
pool `[0.0, 1.0, 2.0, 4.0, 6.0, 2/3]` (`creature/founder.rs`,
`node1_vm_decision`); T13.F05's pinned-seed replay
(`neighborhood/recruitment_paths/qualification.rs`,
`tests/recruitment_paths.rs`).

Options (the user settled the form in the row; recorded so a later reader
knows what was weighed, not re-verified against external sources): (a) an
additive step scaled by `max(|c|, 1)` — chosen: keeps the graph-parameter
step on the unit scale, can move a constant off zero and across zero, and
needs no distribution change; (b) a log-normal multiplicative step (evolution
strategies' self-adapted σ, CMA-ES) — cannot leave zero (the founder's slot 0
and every constant that has drifted small would freeze) and cannot cross
zero; (c) per-slot step tables — rejected in the track note; (d) a Gaussian
draw — changes the distribution family the graph steps use and is outside the
row. Natural analog: mutational effects on a quantitative trait scale with the
trait's magnitude (proportional-effect and log-scale mutation models;
evolution strategies' step sizes track the parameter, not a global unit).

Per-constant width under the rule, founder pool:

| Index | Value | Feeds | Half-width (was ±1) | Decode-changing share of hits (was) |
| --- | ---: | --- | ---: | --- |
| 0 | 0.0 | unreferenced | 0.1 | silent (silent) |
| 1 | 1.0 | unreferenced | 0.1 | silent (silent) |
| 2 | 2.0 | direction code 1 | 0.2 | 0 (0.5: `round()` boundary at ±0.5) |
| 3 | 4.0 | direction code 2 | 0.4 | 0 (0.5) |
| 4 | 6.0 | direction code 3 | 0.6 | 1/6 (0.5): the tails past ±0.5 |
| 5 | 2/3 | reproduce transfer fraction | 0.1 | changed wherever a scenario reproduces (same); sterile 0 (0.167), semelparous 0 (0.333) |

Invariants:

1. Step: with a non-empty pool the operator draws the index uniformly, then
   `u` in [−0.1, 0.1] (`gen_range(-0.1f32..=0.1)`, the graph parameter
   form), and sets `c += u * c.abs().max(1.0)` in `f32`. The two draws and
   their order are unchanged, so an RNG stream downstream of the event is
   unchanged and every trajectory difference is the applied value.
2. Empty pool: unchanged — one constant drawn from [−1, 1] is pushed.
3. Bound (pure invariant, property-tested): for every finite `c` and every
   seed, `|c' − c| ≤ 0.1 × max(|c|, 1)` up to one f32 rounding of the larger
   operand (the committed regression `c = 0.0` needs it); for `|c| ≤ 1` the step is the
   ±0.1 graph parameter step; `c'` is finite for every `|c| ≤ f32::MAX / 1.1`.
   A non-finite `c` stays non-finite (it did before); nothing is sanitized.
4. No clamp: a constant may cross zero and grow without bound, as a graph
   `Constant`/`Threshold` parameter may.
5. Transfer slot: one event on the V3Alpha1 founder's index 5 yields a
   fraction in [0.5667, 0.7667]; the sterile (fraction ≤ 0) and semelparous
   (fraction ≥ 1) shares among slot-5 hits are exactly 0. Read before and
   after by a deterministic seed sweep of `VmMutator::apply(VmConstantMutation)`
   on the founder (at least 10,000 seeds, slot-5 hits classified; the
   "before" row is the same sweep run against the parent commit's operator,
   the red half of the test, so both rows share one seed set) and by the
   founder battery's
   `operator_rows[VmConstantMutation]` tally
   (`deterministic.goal_indicators.mutational_neighborhood.founder.operator_rows`
   in the gate summary; before 50 trials: 13 changed, 37 silent, 0 dead).
   Both tables live in the readings file.
6. Determinism: same seed, same config, same trajectory. Every trajectory
   moves from the first `VmConstantMutation` event; each pinned trajectory
   test is re-pinned and listed in the readings file with old and new values.
7. T13.F05 replay stays truthful. A constant walk from 0.0 to [1.5, 2.5)
   needs at least 15 maximal draws under the rule (0.1 per event until
   |c| ≥ 1, then ×1.1), so `vm_unprepared_plan` reaches east through the
   existing operators instead: one `VmInstructionMutation` replace
   (`direction_doubled`, seed 205,178) turns the direction constant's
   `LoadConst { dst: 3, const_idx: 1 }` at program index 4 into
   `Add { dst: 3, a: 0, b: 0 }`, the cue register doubled into the direction
   register as `vm_blank`'s `double_to_east` computes it. The plan is six
   events, `vm_unprepared` qualifies, and its `GrowthGap` is gone (qualified
   forms 7 of 9). The step's predicate is structural and exact (same length,
   same program before and after index 4, that instruction equal); every
   other step, form, seed, predicate, `MAX_PATH_EVENTS` (6), `SEARCH_RANGE`,
   and `qualified_paths().len() == 9` are unchanged; the seed is the module's
   own first-accepted search; the readings file lists old and new seeds.
8. Documentation: the `VmConstantMutation` entry in
   `docs/reference/v3-mutation-spec.md` states the step rule.

## Implementation Tasks

- [x] Step rule in `apply_constant_mutation` (invariants 1–4), TDD: focused
      tests and the proptest for invariant 3.
- [x] Transfer-slot probe (invariant 5): seed sweep before and after, table in
      `docs/progress/readings/t11-f23.md`.
- [x] T13.F05 `vm_unprepared` route re-derived and pinned (invariant 7);
      `tests/recruitment_paths.rs` green.
- [x] Pinned trajectories re-pinned and listed (invariant 6).
- [x] `docs/reference/v3-mutation-spec.md` entry (invariant 8).
- [ ] Gate and goal baseline runs recorded; goal-worlds epoch re-pinned in the
      closing commit (Performance).

## Verification

- [x] Focused tests: `cargo test -p v3-core mutation::vm` (step bound, unit-scale
      equality with the graph step, empty-pool draw, RNG draw count) and
      `cargo test -p v3-core --test recruitment_paths` -> results and test
      names in `docs/progress/readings/t11-f23.md`.
- [x] Property test for invariant 3 (proptest; `proptest-regressions/`
      committed if produced) -> test name in the readings file.
- [x] Transfer-slot seed sweep before/after and founder battery
      `operator_rows[VmConstantMutation]` before/after -> tables in the
      readings file (invariant 5); the authoritative after battery row is the
      gate summary's `operator_rows[VmConstantMutation]` (changed 6, silent
      44, dead 0 of 50; before 13/37/0), which matches the earlier local
      `evaluate_genome` reading.
- [x] `cargo test -p v3-core --test viability` first (27 passed), then
      `make check` -> exit 0.
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants` (mode `fresh`) at
      `~/.local/share/petri-tools/mutants/t11-f23/`: `13 mutants tested in
      5m: 1 missed, 10 caught, 2 unviable`, no timeouts. Survivors:
      1. `crates/v3-core/src/mutation/vm/operators.rs:27:45: delete - in
         apply_constant_mutation` (empty-pool draw becomes `1.0..=1.0`) ->
         killed by
         `vm_constant_mutation_on_empty_pool_draws_the_constant_from_the_signed_unit_range`
         (`mutation/vm/tests.rs`; pins the draw over 256 seeds, some
         negative); `MUTANTS_ITERATE=1` pass: `1 mutant tested in 2m: 1
         caught` (`mutants.out/caught.txt`; fresh `missed.txt`/`timeout.txt`
         under `mutants.out.old/`; `run-mode.txt` now `incremental`).
         Test-only remediation; no second fresh run.
- [x] Benchmark summaries stored at
      `docs/progress/features/t11-f23-scale-relative-vm-constant-steps.json`
      (gate, 97,683 bytes) and `...-goal.json` (goal, 6,870,431 bytes); local
      raw at the main checkout's `.bench-artifacts/t11-f23-scale-relative-vm-constant-steps/`
      (`gate.json` 94,841 bytes sha256 `2c41167b…ada980`, `goal.json`
      534,930,639 bytes sha256 `cdeaeac3…d5146`), `verified_local` at
      2026-09-19T04:23:31Z / 04:32:31Z and re-checked with `shasum -a 256`;
      `make` exit 0 and CLI exit 0 on both profiles; series entries
      (`gate-v1`, `goal-worlds-v1`) point to the summaries; no full report
      staged. Full hashes and the per-profile tables in the readings file's
      Benchmark section.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: the size of a
mutation's effect on a quantitative trait scales with the trait's magnitude
(proportional-effect and log-scale mutation models; evolution strategies'
step sizes track the parameter). It reaches creatures through the genome
alone: the mutation operator's step, no sensor, no reward, no config.

Expected compute cost: none from the arithmetic (one `abs`, one `max`, one
multiply per event). Every work counter can move through the trajectory (the
first `VmConstantMutation` event changes a descendant's constants). This is
not an environmental pressure; the three-world integration rule does not
apply beyond the ordinary goal run.

References. Gate: epoch `remove-complementary-nutrition.json`, latest closure
`t17-f01-offspring-investment-fraction.json`. Goal (goal-worlds-v1): epoch and
latest closure are both `t17-f01-offspring-investment-fraction-goal.json`
(re-pinned at T17.F01's closure). Standard +10%/+50% work and +25%/+100% wall
flags. Epoch re-pin, as the row predeclares: the goal-worlds epoch is
re-pinned to this feature's goal summary in the closing commit; the gate epoch
only if the user accepts a severe gate result. A severe work counter on either
profile, or an extinction in any goal world, is a user decision under the
blocker rule. Wall-clock moves are flag-only.

Before/after readings the row requires, tabled in the readings file: the
founder battery `operator_rows[VmConstantMutation]` tally and the T11.F14
founder changed/silent/dead rows (gate summary and each goal world's
`case_readings`; before = the T17.F01 summaries); the transfer-slot sterile
and semelparous shares (invariant 5); goal `per_creature_tick.births` per
world (before = the T17.F01 goal summary; gate founder births before
0.026268).

| Indicator | Predeclared direction |
| --- | --- |
| Transfer slot: sterile share, semelparous share of slot-5 hits | 0.167 → 0 and 0.333 → 0 exactly (arithmetic; a nonzero reading fails the feature) |
| Gate: founder `operator_rows[VmConstantMutation].changed` (13/50) | falls: direction constants 2.0/4.0 no longer cross a `round()` boundary, 6.0 does on 1/6 of its hits; slot-5 hits stay changed |
| Gate and goal: `founder_changed_per_all_births` | falls or holds; no rise; recorded per world |
| Gate and goal: `founder_dead_per_all_births` | no direction (0 before); recorded |
| Gate: `births` per creature-tick, per-seed `births` and `final_population` | move; no sign; standard thresholds |
| Gate and goal work counters (`vm_steps`, `mesh_hops`, `graph_relax_iters`, `plasticity_updates`, `actions_applied`) | no direction; standard thresholds; severe is a user decision |
| Goal: `births` per creature-tick, `final_population`, `plateau_population`, `mean_energy`, per world | move; no sign; extinction is a blocker |
| Goal: `energy_flows.offspring_energy_credit / births` (mean litter), per world | no direction; recorded (a slot-5 line now varies its litter within ±15% per event instead of dying or halving) |
| Goal: evolved changed/dead, `neighborhood_read_*`, drift depth, lineage diversity, recruitment paths | no direction; recorded |

**Measured verdict.** Measured 2026-09-18 at `ea498567` (benchmark
specialist; commands, exit statuses, sizes and tables in the readings file).
Gate: `make` exit 0, CLI exit 0, `severe=false` against both
`remove-complementary-nutrition.json` and
`t17-f01-offspring-investment-fraction.json`; every work counter `ok`
(largest move `plasticity_updates` +9.435938% vs the latest closure,
−25.145466% vs the epoch); wall `ok` (−1.569492% / +1.704988%). Goal:
`make` exit 0, CLI exit 0, `severe=false` against
`t17-f01-offspring-investment-fraction-goal.json` (epoch and latest
closure); every work counter `ok` (`plasticity_updates` −32.885797%,
`vm_steps` +0.400436%); wall `ok` (+16.476621%, under the 25% flag). No
extinction in any goal world (`extinction_tick` null; final populations
Orchards 7,208, Canyon 5,683, Confluence 4,648). Caps:
`neighborhood_evolved_wall_clock_ms_total` 695.156 ms of 180,000;
`neighborhood_founder_wall_clock_ms` 115.480 ms (goal) and 46.371 ms (gate)
of 10,000. Founder battery `operator_rows[VmConstantMutation]` in the gate
summary: changed 6, silent 44, dead 0 of 50 (before 13/37/0), the same in
every goal world. `founder_changed_per_all_births` fell in the gate
(0.400000 → 0.395238; gate source
`mutational_neighborhood.founder.births.any_events.changed_fraction`, 83/210,
as the gate summary has no `case_readings`) and every goal world (0.438095 → 0.433333, 0.4 →
0.395238, 0.438095 → 0.433333); `founder_dead_per_all_births` 0 everywhere.
Gate `births` per creature-tick 0.026042 (before 0.026268). No indicator
missed its predeclared direction; no severe flag; no user decision raised
by the measurements. The goal-worlds epoch re-pin to this feature's goal
summary is the closing commit's step, not this record's.

- Summaries: [gate](../../progress/features/t11-f23-scale-relative-vm-constant-steps.json),
  [goal](../../progress/features/t11-f23-scale-relative-vm-constant-steps-goal.json).
- Full readings: [`docs/progress/readings/t11-f23.md`](../../progress/readings/t11-f23.md).

## Success Criteria

- [ ] `apply_constant_mutation` applies `c += u × max(|c|, 1)`, `u` uniform in
      [−0.1, 0.1], with the empty-pool draw and RNG draw order unchanged
      (invariants 1–4, tests and proptest green).
- [ ] The founder's transfer-slot sterile and semelparous shares under one
      `VmConstantMutation` read 0 after (0.167 / 0.333 before), tabled in the
      readings file with the founder battery operator row before and after.
- [ ] T13.F05's replay is green with a truthfully re-derived `vm_unprepared`
      route (six events, qualified); no predicate weakened.
- [ ] Pinned trajectories re-pinned and listed; `make check` and the mutation
      gate pass with every survivor resolved.
- [ ] Gate and goal summaries stored, goal births per creature-tick read before
      and after per world, goal-worlds epoch re-pinned in the closing commit.

## Notes for AI Agents

- Decision: the user chose the scale-relative step over rescaling the transfer
  slot back to an integer scale and over per-slot step sizes (track note
  "Constant steps, 2026-09-18"); graph parameter steps stay ±0.1.
- Decision: spec-owner ruling 2026-09-18 — the T13.F05 `vm_unprepared` form
  qualifies through the `direction_doubled` instruction replace (invariant 7);
  the earlier "no six-event route exists with the existing operators" record
  was a constant-walk reading only, since the same replace was available
  under the ±1 operator. Later readers (T13.F06, T11.F13) cite 7 qualified
  forms of 9 from this closure, not the 7-event growth gap.
- Cost: Opus substitution for all roles (spec owner, implementer without
  advisor, benchmark specialist, reviewer, mutation specialist); /usage totals
  pending.
