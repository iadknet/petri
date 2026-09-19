# T17.F01 — Offspring Investment as a Fraction of Parent Energy

**Status**: In Progress
**Last updated**: 2026-09-18
**Feature**: T17.F01
**Track**: [T17 — Brain Boundary Evolvability](../../roadmaps/t17-brain-boundary-evolvability.md)

## Goal

A parent gives its young a share of what it has, not a fixed ration. The
reproduce action's `meta[1]` leaves the brain as a fraction in [0, 1] of the
parent's post-cost energy, and `apply_reproduce`
(`crates/v3-core/src/simulation/actions/reproduction.rs`) turns it into the
child's starting energy: `transfer = min(fraction × after_cost,
default_offspring_energy)`. Physiology keeps the `min_reproduce_energy` gate on
`after_cost` (T16.F01) and adds a minimum litter: a child that would be born
with less than `initial_energy` is not conceived, rejected free of charge
before anything is deducted. Every founder profile emits a constant fraction
behind a brain gate at or above what physiology accepts, so a founder never
loses a tick to a refused attempt. A [0, 1] sensor routed into the transfer
slot is now an investment rule instead of a hidden lethal.

## Non-Goals

- No famine or collapse protection (reserve, population floor,
  density-dependent cost, lifespan): a separate user discussion.
- No Polyworld-style [0.2, 0.8] bound: the row says [0, 1]; the litter floor
  and the `default_offspring_energy` cap are the world's bounds. A fraction of
  1.0 is a parent that gives everything and dies — visible, not hidden.
- No change to the lifecycle config values, the age gate, the cost formula,
  or the charge order (T16.F01 invariants 1–4 hold under the new Step 7).
- No change to `VmConstantMutation`, the steal amount (T17.F03), or the
  introspective inputs (T17.F02).
- No new sensor, operator, assay, founder layout (T18), or environmental
  pressure.

## Inputs and Invariants

Sources of truth: the T17.F01 row and the track's "Hazard" note; the
[reproduction-collapse research note](../../strategy/reproduction-collapse-research-2026-09-18.md)
Section 5 finding 7 (a founder gated below what physiology accepts pins
itself: 0 births in 2,000 ticks on two seeds, penalty on and off, because its
reproduce branch ends in the terminal `ExecuteActionQueue`) and Section 8;
`apply_reproduce` Steps 5–8; `decode_world_action`
(`crates/v3-core/src/runtime/action_decode.rs`); `founder.rs` and
`cgp_founder.rs` (the energy gate is a strict `Threshold(t)`: the founder
attempts when `EnergyCurrent > t`); the four reference docs named in the
tasks.

Options: (a) a fraction of post-cost energy — chosen, Polyworld's
`MateEnergyFraction` and capital breeding; (b) a fraction of the surplus
above a reserve — the note's counterfactual, the excluded collapse half; (c) a
fraction of `max_energy` — a fixed ration in other units. The
`default_offspring_energy` cap stays: it keeps the config contract and is the
model T17.F03 reuses.

Measured at production defaults: `initial_energy` 20, `min_reproduce_energy`
30, `default_offspring_energy` 100, `max_energy` 200; `complexity_cost` is
disabled and the age multiplier is `1 + 9·min(1, age/500)²`, so a founder
genome's reproduce cost is 0.1 at age 0 and at most 1.0 (age ≥ 500). A
mutated descendant with more units pays the T03.F11 surcharge and is outside
the founder acceptance claim.

Invariants:

1. Decode: `WorldAction::Reproduce` carries the fraction under a name that
   says so (`energy_transfer_fraction` or equivalent; the frontend trace type
   in `frontend/src/types/trace.ts` follows, no shim). `meta[1]` is sanitized
   to [0, 1]: NaN, ±∞, and negatives to 0.0, values above 1.0 to 1.0. The
   `meta_param` accessor and the action-log input ("param1") therefore expose
   a [0, 1] value.
2. Step 7 predicate: `transfer = min(fraction × after_cost,
   default_offspring_energy)` in `f32`; the attempt is rejected as
   `RejectedEnergyConstraints` iff `transfer <= 0.0` or `transfer <
   initial_energy`. `after_cost >= transfer` holds by construction (fraction
   ≤ 1); Step 6 (`after_cost < min_reproduce_energy`) is unchanged.
3. Rejection stays free (T16.F01 invariant 2): parent energy,
   `action_charges.reproduce`, `parental_transfer_debit`, and
   `pending_death_cause` unchanged; the attempt and rejection counters
   increment.
4. On acceptance the parent pays `cost` then `transfer` as two successive
   `f32` subtractions (T16.F01 invariant 3) and the child starts at exactly
   `transfer`; `offspring_energy_credit` equals the sum of transfers.
5. Founder profiles: each emits the fraction below as VM constant index 5 and
   carries the strict threshold below; for every profile, every energy above
   its threshold up to `max_energy`, and every age at or above
   `min_reproduce_age`, `apply_reproduce` accepts (cost ≤ 1.0, so
   `after_cost ≥ threshold − 1 ≥ 30` and `fraction × after_cost ≥ 20` with at
   least 0.65 energy of margin above the litter floor). Tick-level: the gate
   reads `EnergyCurrent` in Phase 1a and `apply_reproduce` runs in Phase 2;
   between them a founder pays only cognition cost (`graph_node_base_cost`
   1e-5 per node-pass, `opcode_cost_multiplier` 1e-6 per VM step, no priority
   bid), under 1e-3 at defaults, so the ≥ 1.0 energy between the threshold
   and the engine's `30 + cost` covers it. This closes T16.F01 invariant 6's
   window (`[30, 30 + cost)`) for founders.
6. Determinism: same seed, same config, same trajectory. Every trajectory
   moves from the first birth; pinned-trajectory tests are re-pinned and
   listed in the readings file with old and new values.
7. The cap binds only when `fraction × after_cost > 100`; it clamps, never
   rejects.

| Profile | Threshold (was) | Fraction (was transfer) | Child at threshold, cost 1.0 |
| --- | ---: | ---: | ---: |
| V3Alpha1 (default) | 32 (30) | 2/3 (20) | ≥ 20.67 |
| ForageFirstSparse | 32 (30) | 2/3 (10) | ≥ 20.67 |
| ForageFirstSparseConservative | 60 (60) | 0.35 (10) | ≥ 20.65 |
| ForageFirstSparseRichOffspring | 40 (40) | 0.60 (20) | ≥ 23.4 |
| ForageFirstSparseBalanced | 50 (50) | 0.45 (15) | ≥ 22.05 |

V3Alpha1 keeps its 20-of-30 ratio; its threshold rises by 2 (four ticks of
decay) so the age-1.0 cost and the litter floor clear with an `f32` margin,
not by rounding luck. Profiles whose 10 and 15 litters fall under the floor
take the smallest clean fraction that clears it at their own threshold.
T17.F02 re-expresses these thresholds (32 → 0.16 of 200).

Known consequence, recorded not fixed: `VmConstantMutation` adds a uniform
[−1, 1] draw to a constant — ±5% of a 20-energy ration before, but on a 2/3
fraction a draw below −0.667 (p ≈ 0.17) makes the line sterile (every litter
under the floor, every attempt refused) and above +0.333 (p ≈ 0.33)
semelparous. `founder_changed_per_all_births` therefore moves (sign below);
rescaling the VM constant step is a T11 question for the user.

## Implementation Tasks

- [ ] Run `cargo test -p v3-core --test viability` first and record the
      baseline pass.
- [ ] Decode: sanitize `meta[1]` to [0, 1] and rename the contract field;
      update `frontend/src/types/trace.ts`, its test, and the `inputs.rs`
      comment.
- [ ] `apply_reproduce` Step 7 per invariant 2; rewrite
      `apply_reproduce_default_cap_allows_twenty_energy_transfer` and add the
      litter-floor and fraction tests; keep the rejection-free tests green.
- [ ] Founder: constants and thresholds per the table; re-pin `PROFILES` and
      the founder scenario tests; add the acceptance property test of
      invariant 5 (proptest over energy and age; `proptest-regressions/`
      committed if produced).
- [ ] Re-pin trajectory-dependent tests; list each in the readings file.
- [ ] Reference docs: `v3-reproduction-spec.md` Steps 7–8 and the
      `initial_energy` line, `v3-runtime-config-spec.md` transfer sequencing
      and the `default_offspring_energy` row, `v3-vm-isa-spec.md` action
      table row 3, `v3-startup-seeding-spec.md` Section 5.1 profile table.
- [ ] `make check` (frontend deps present in this worktree).

## Verification

- [ ] `cargo test -p v3-core --test viability` -> pass, before and after.
- [ ] Decode tests (`runtime/action_decode.rs`): NaN, ±∞, −0.5, 0.0, 0.42,
      1.0, 7.0 → 0.0, 0.0, 0.0, 0.0, 0.42, 1.0, 1.0 -> test names in the
      readings file.
- [ ] `apply_reproduce` tests (`simulation/actions/mod.rs` or
      `reproduction.rs`): fraction × after_cost child energy; cap at 100;
      litter-floor rejection with unchanged parent energy and flows; a
      fraction of 1.0 leaves the parent at exactly 0.0 -> test names in the
      readings file.
- [ ] Founder acceptance property test (invariant 5) over all five profiles
      -> test name; `founder_profiles_execute_energy_age_boundaries_and_priorities`
      re-pinned to the table.
- [ ] Tick-level acceptance: a founders-only run (default profile, mutation
      off, at least 2,000 ticks, one seed) ends with
      `reproduction_actions_rejected_by_reason[RejectedEnergyConstraints]`
      equal to 0 and `reproduction_actions_spawned_total > 0` -> test name
      (viability or a focused tick test).
- [ ] `make check` -> pass; the re-pinned trajectory tests and their old/new
      values listed in [`docs/progress/readings/t17-f01.md`](../../progress/readings/t17-f01.md).
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      and every survivor resolved as killed, equivalent, or deferred. The full
      survivor list stays here.
- [ ] Benchmark summaries stored at
      `docs/progress/features/t17-f01-offspring-investment-fraction.json` and
      `...-goal.json`, local raw hash/byte count and verification time
      checked, series entries point to the summaries, no new full report
      staged.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: Polyworld's
`MateEnergyFraction` and capital breeding — a litter is provisioned as a share
of the parent's reserves, and a body refuses a litter too small to live. It
reaches creatures through the action boundary and the body's energy ledger
alone: no sensor, no operator, no reward.

Expected compute cost: none from the arithmetic (one multiply and one
comparison per attempt); every work counter can move through the trajectory
(richer parents make richer children, parents keep a third instead of
`energy − 20`, the founder threshold is 32 not 30).

References. Gate: epoch `docs/progress/features/remove-complementary-nutrition.json`,
latest closure `topology-weight-rescale.json` (measured at `7f5d8994`; only
docs changed to this worktree's base `b6431985`). Goal (goal-worlds-v1): epoch
`t11-f19-per-unit-mutation-supply-goal.json`, latest closure
`t16-f01-gate-before-charge-in-reproduction-goal.json` (measured at
`bf7499e0`, before the `ChangeEntryNode` rescale `936280fb`, which has no goal
record — goal deltas include it). Standard +10%/+50% work and +25%/+100% wall
flags apply.

Epoch re-pin, as the row predeclares: the goal-worlds epoch is re-pinned to
this feature's goal summary in the closing commit. The gate epoch is re-pinned
only if the user accepts a severe gate result; a severe work-counter result on
either profile, or an extinction in any goal world, is a user decision under
the blocker rule. Wall-clock moves are flag-only.

Before/after readings the row requires, from the stored summaries: the
founder's gate `per_creature_tick.births` (before 0.025055; per seed 3,019 /
3,023 / 3,009 births, final 1,924 / 1,823 / 1,896) and the T11.F14 rows in
each goal world's `case_readings`
(`founder_changed_per_all_births` 0.457143, `founder_dead_per_all_births` 0,
`evolved_changed_per_mutated_births` 0.358891, `evolved_dead_per_mutated_births`
0.001631, `neighborhood_read_silent_per_all_births` 0.271429 in Orchards;
Canyon and Confluence copied from the same T16.F01 goal summary into the
readings file). Reproduction rejections are not a
reported bench counter and no indicator is added for them. The survey's
paired-perturbation probe is not in the repository (T16.F01 deferral) and is
not taken here.

| Indicator | Predeclared direction |
| --- | --- |
| Gate: `births` per creature-tick, per-seed `births` and `final_population` | move (nonzero delta certain); no sign; standard thresholds |
| Gate: `vm_steps`, `mesh_hops`, `graph_relax_iters`, `plasticity_updates`, `actions_applied` | no direction; standard thresholds; severe is a user decision |
| Goal: `births` per creature-tick, `final_population`, `plateau_population`, `mean_energy`, per world | move; no sign; Orchards trajectory recorded beside T16.F01's as a reading |
| Goal: `energy_flows.offspring_energy_credit / births` (mean litter, raw goal report), per world | rises in every world (founder litter ≥ 20.67, richer parents give more; T16.F01 mean was 20 by construction) |
| Goal: `founder_changed_per_all_births` | rises (a `VmConstantMutation` on slot 5 now changes litter size or fertility) |
| Goal: `founder_dead_per_all_births` | no direction (a refused attempt is not a dead birth); recorded |
| Goal: evolved changed/dead, `neighborhood_read_silent`, drift depth, lineage diversity, recruitment | no direction; recorded |
| Goal work counters | no direction; standard thresholds; severe is a user decision |

**Measured verdict.** Written at closure: one line per profile with CLI and
outer-process exit statuses and sources, the `severe` flag, thresholds
crossed, and the re-pin.

- Summaries: [gate](../../progress/features/t17-f01-offspring-investment-fraction.json),
  [goal](../../progress/features/t17-f01-offspring-investment-fraction-goal.json).
- Full readings: [`docs/progress/readings/t17-f01.md`](../../progress/readings/t17-f01.md).

## Success Criteria

- [ ] `meta[1]` decodes to a fraction in [0, 1] and the child's starting energy
      is `min(fraction × after_cost, default_offspring_energy)`.
- [ ] A litter under `initial_energy` is rejected before any charge; the
      parent's energy and every reproduce flow are unchanged by the rejection.
- [ ] Every founder profile's attempt is accepted at every energy above its
      threshold and every eligible age (property test), a founders-only run
      records zero energy-constraint rejections, and the viability test
      passes.
- [ ] Reference docs and the frontend trace type describe the fraction; no
      surface still calls it an energy amount.
- [ ] Gate and goal summaries stored with the before/after readings above in
      the readings file; goal-worlds epoch re-pinned in the closing commit.

## Notes for AI Agents

- Decision: the founder fractions and thresholds are the table in Inputs and
  Invariants; T17.F02 re-expresses those thresholds, not the pre-F01 values.
- Deferred: the track's paired-perturbation probe is not taken (probe genomes
  live in session scratchpads, not the repo; T16.F01 deferral stands); the
  `VmConstantMutation` step size on a [0, 1] fraction is surfaced to the user
  as a T11 question.
- Cost: Opus-only substitution (spec owner, implementer, benchmark, reviewer,
  mutation all Opus 5; implementer without advisor); `/usage` totals and
  pass counts recorded at closure.
