# T17.F01 — Offspring Investment as a Fraction of Parent Energy

**Status**: In Progress
**Last updated**: 2026-09-18
**Feature**: T17.F01
**Track**: [T17 — Brain Boundary Evolvability](../../roadmaps/t17-brain-boundary-evolvability.md)

## Goal

A parent gives its young a share of what it has, not a fixed ration. The
reproduce action's `meta[1]` leaves the brain as a fraction in [0, 1] of the
parent's post-cost energy; `apply_reproduce`
(`crates/v3-core/src/simulation/actions/reproduction.rs`) makes the child's
starting energy `min(fraction × after_cost, default_offspring_energy)`.
Physiology keeps the `min_reproduce_energy` gate on `after_cost` (T16.F01)
and adds a minimum litter: a child under `initial_energy` is not conceived,
rejected free before anything is deducted. Every founder profile emits a
constant fraction behind a gate at or above what physiology accepts, so a
founder never loses a tick to a refused attempt. A [0, 1] sensor routed into
the transfer slot is now an investment rule instead of a hidden lethal.

## Non-Goals

- No famine or collapse protection (reserve, population floor,
  density-dependent cost, lifespan): a separate user discussion.
- No Polyworld-style [0.2, 0.8] bound: the row says [0, 1]; the floor and
  the cap are the world's bounds. A fraction of 1.0 is a parent that gives
  everything and dies — visible, not hidden.
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
itself: 0 births in 2,000 ticks, because its reproduce branch ends in the
terminal `ExecuteActionQueue`) and Section 8; `apply_reproduce` Steps 5–8;
`decode_world_action`; `founder.rs` and `cgp_founder.rs` (the energy gate is
a strict `Threshold(t)`: the founder attempts when `EnergyCurrent > t`).

Options: (a) a fraction of post-cost energy — chosen (Polyworld's
`MateEnergyFraction`, capital breeding); (b) a fraction of the surplus above
a reserve — the excluded collapse half; (c) a fraction of `max_energy` — a
fixed ration in other units. The cap stays; T17.F03 reuses the model.

Measured at production defaults: `initial_energy` 20, `min_reproduce_energy`
30, `default_offspring_energy` 100, `max_energy` 200; `complexity_cost` is
disabled and the age multiplier is `1 + 9·min(1, age/500)²`, so a founder
genome's reproduce cost is 0.1 at age 0 and at most 1.0. A mutated descendant
pays the T03.F11 surcharge and is outside the founder acceptance claim.

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
7. The cap clamps; only the floor rejects. A config with
   `default_offspring_energy < initial_energy` refuses every birth, free, and
   is neither rejected nor normalized at load (lifecycle validation is
   unchanged, Non-Goals); the refusal is visible in the rejection counters
   where the old engine bore a child that starved.

| Profile | Threshold (was) | Fraction (was transfer) | Child at threshold, cost 1.0 |
| --- | ---: | ---: | ---: |
| V3Alpha1 (default) | 32 (30) | 2/3 (20) | ≥ 20.67 |
| ForageFirstSparse | 32 (30) | 2/3 (10) | ≥ 20.67 |
| ForageFirstSparseConservative | 60 (60) | 0.35 (10) | ≥ 20.65 |
| ForageFirstSparseRichOffspring | 40 (40) | 0.60 (20) | ≥ 23.4 |
| ForageFirstSparseBalanced | 50 (50) | 0.45 (15) | ≥ 22.05 |

V3Alpha1 keeps its 20-of-30 ratio; its threshold rises by 2 so the age-1.0
cost and the floor clear with an `f32` margin. Profiles whose 10 and 15
litters fall under the floor take the smallest clean fraction that clears it
at their own threshold. T17.F02 re-expresses these thresholds (32 → 0.16).

Known consequence, recorded not fixed: `VmConstantMutation` adds a uniform
[−1, 1] draw — ±5% of a 20-energy ration before; on a 2/3 fraction a draw
below −0.667 (p ≈ 0.17) sterilizes the line (every litter under the floor)
and above +0.333 (p ≈ 0.33) makes it semelparous. Rescaling the VM constant
step is a T11 question for the user.

## Implementation Tasks

- [x] Viability baseline run first (26 passed).
- [x] Decode: `clamp_unit_interval(meta[1])`;
      `WorldAction::Reproduce { energy_transfer_fraction }`; frontend trace
      type and test renamed.
- [x] `apply_reproduce` Step 7 per invariant 2 (the fraction is clamped again
      there so `after_cost >= transfer` holds for every caller).
- [x] Founder: `founder_reproduce_policy(profile)` in `founder.rs` is the
      one (threshold, fraction) table every profile is built from;
      `proptest-regressions/runtime/action_decode.txt` added.
- [x] Re-pinned tests and fixtures listed in the readings file (two digests,
      five contract pins, eight fixtures whose cap sat under their
      `initial_energy`).
- [x] Reference docs: `v3-reproduction-spec.md`, `v3-runtime-config-spec.md`,
      `v3-vm-isa-spec.md`, `v3-startup-seeding-spec.md` describe the fraction,
      the floor, and the profile table.
- [x] `v3-runtime-config-spec.md` `default_offspring_energy` row states that
      a cap under `initial_energy` refuses every birth (invariant 7).
- [x] `make check` -> pass.

## Verification

- [x] `cargo test -p v3-core --test viability` -> 26 before, 27 after.
- [x] Decode tests (`runtime/action_decode.rs`):
      `reproduce_fraction_is_sanitized_to_unit_interval` (the seven spec
      inputs, bit-exact) and the proptest
      `reproduce_fraction_always_lands_in_unit_interval`.
- [x] `apply_reproduce` tests (`simulation/actions/mod.rs`, bit-exact
      energies through the `birth_split` fixture):
      `apply_reproduce_child_starts_at_fraction_of_post_cost_energy`,
      `apply_reproduce_caps_child_at_default_offspring_energy`,
      `apply_reproduce_litter_floor_rejection_is_free`,
      `apply_reproduce_accepts_litter_exactly_at_initial_energy`,
      `apply_reproduce_full_fraction_leaves_parent_at_exactly_zero`.
- [x] Founder acceptance property test (invariant 5):
      `founder_profiles_are_accepted_at_every_energy_above_their_gate`
      (all five profiles, energy in (threshold, 200], age in [20, 10 000]);
      `founder_profiles_execute_energy_age_boundaries_and_priorities` and
      `founder_reproduce_policy_matches_the_profile_table` pin the table.
- [x] Tick-level acceptance:
      `viability::founders_only_run_has_no_energy_rejected_reproduce_attempts`
      (10 founders, mutation off, 2,000 ticks): no `RejectedEnergyConstraints`
      entry, `reproduction_actions_spawned_total > 0`.
- [x] `cargo test --workspace` and `make check` -> pass; re-pinned tests and
      old/new values in
      [`docs/progress/readings/t17-f01.md`](../../progress/readings/t17-f01.md).
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      and every survivor resolved as killed, equivalent, or deferred. The full
      survivor list stays here.
- [x] Benchmark summaries stored at
      `docs/progress/features/t17-f01-offspring-investment-fraction.json` and
      `...-goal.json`, local raw hash/byte count and verification time
      checked, series entries point to the summaries, no new full report
      staged; goal run once (2026-09-05 user decision).

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

References. Gate: epoch `remove-complementary-nutrition.json`, latest
closure `topology-weight-rescale.json` (at `7f5d8994`; docs-only to base
`b6431985`). Goal (goal-worlds-v1): epoch
`t11-f19-per-unit-mutation-supply-goal.json`, latest closure
`t16-f01-gate-before-charge-in-reproduction-goal.json` (at `bf7499e0`, before
the `ChangeEntryNode` rescale `936280fb`, which has no goal record, so goal
deltas include it). Standard +10%/+50% work and +25%/+100% wall flags.

Epoch re-pin, as the row predeclares: the goal-worlds epoch is re-pinned to
this feature's goal summary in the closing commit; the gate epoch only if the
user accepts a severe gate result. A severe work counter on either profile,
or an extinction in any goal world, is a user decision under the blocker
rule. Wall-clock moves are flag-only.

Before/after readings the row requires, from the stored summaries: the
founder's gate `per_creature_tick.births` (before 0.025055, per seed 3,019 /
3,023 / 3,009) and the T11.F14 rows of each goal world's `case_readings`
(before = the T16.F01 goal summary; both sides tabled in the readings file).
Reproduction rejections are not a reported bench counter. The survey's
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

**Measured verdict** (2026-09-18, code `02b3981e`):

- Gate: `make` exit 0 (observed), `cli_exit` 0 (summary); `severe` false,
  all `ok`; predeclaration met (`births` 0.026268, +4.84%). No re-pin.
- Goal: `make` exit 2 after `Error 3` (observed), `cli_exit` 3 (summary);
  `severe` **true** vs the T16.F01 closure (`plasticity_updates` +76.53%),
  `ok` vs the epoch (−56.87%); wall `flag` +40.88% vs the epoch only, no
  comparable prior (T16.F01's wall record against it was null), flag-only,
  no action; no extinction; caps met (evolved 560.5 ms, founder 96.5 ms).
  Spec-owner ruling 2026-09-18: the severe is a user decision under the
  blocker rule; the row's re-pin covers births-based indicators, not a work
  counter. The table below supports reading it as population composition
  (the counter counts every plasticity node with inputs, T11.F08), not
  compute: Orchards ×7 as the collapsed remnant became a 3,693 plateau,
  Canyon ×2 at an unchanged population (cause not in the summaries),
  Confluence flat. Recommendation: accept with the predeclared goal-worlds
  re-pin. Mean litter rose in every world as predeclared (19.9 → 32.7 /
  36.2 / 35.1). Miss, recorded not blocking: `founder_changed_per_all_births`
  fell in every world (0.457 → 0.438 Orchards and Confluence, 0.419 → 0.400
  Canyon); the likeliest cause — a battery scenario between energy 30 and 32
  no longer reaching the reproduce branch — is a hypothesis, not a
  measurement. Evolved dead per mutated birth rose in every world (Confluence
  0 → 0.0273), no direction predeclared. Re-pin pending the user's decision.

| World | `plasticity_updates` per creature-tick, T16.F01 → T17.F01 | `final_population` | `plateau_population` |
| --- | --- | --- | --- |
| Orchards in grassland | 0.011695 → 0.081413 | 7 → 4,309 | 7.988 → 3,693.256 |
| Canyon country | 0.041795 → 0.081997 | 4,323 → 4,250 | 3,573.072 → 3,687.84 |
| Confluence | 0.061891 → 0.062887 | 4,341 → 8,902 | 4,515.858 → 7,731.054 |

| Profile | Summary (committed) | Summary bytes | Raw (main checkout `.bench-artifacts/t17-f01-offspring-investment-fraction/`, not staged) | Raw bytes |
| --- | --- | --- | --- | --- |
| gate | `docs/progress/features/t17-f01-offspring-investment-fraction.json` | 97,408 | `gate.json` | 94,582 |
| goal | `docs/progress/features/t17-f01-offspring-investment-fraction-goal.json` | 6,899,463 | `goal.json` | 534,873,264 |

- Summaries: [gate](../../progress/features/t17-f01-offspring-investment-fraction.json),
  [goal](../../progress/features/t17-f01-offspring-investment-fraction-goal.json).
- Full readings: [`docs/progress/readings/t17-f01.md`](../../progress/readings/t17-f01.md).

## Success Criteria

- [ ] `meta[1]` decodes to a fraction in [0, 1] and the child's starting energy
      is `min(fraction × after_cost, default_offspring_energy)`.
- [ ] A litter under `initial_energy` is rejected before any charge; the
      parent's energy and every reproduce flow are unchanged by the rejection.
- [ ] Every founder profile's attempt is accepted at every energy above its
      threshold and eligible age (property test and founders-only run), and
      the viability test passes.
- [ ] Reference docs and the frontend trace type describe the fraction; no
      surface still calls it an energy amount.
- [ ] Gate and goal summaries stored with the before/after readings above in
      the readings file; goal-worlds epoch re-pinned in the closing commit.

## Notes for AI Agents

- Decision: the founder fractions and thresholds are the table in Inputs and
  Invariants; T17.F02 re-expresses those thresholds, not the pre-F01 values.
- Deferred: `v3-server/tests/server.rs`'s mutation-counter frame test can
  end its 20-tick horizon without a birth (vacuous reconciliation
  assertions); pre-existing, outside this contract.
- Deferred: the track's paired-perturbation probe is not taken (probe genomes
  live in session scratchpads, not the repo; T16.F01 deferral stands); the
  `VmConstantMutation` step size on a [0, 1] fraction is surfaced to the user
  as a T11 question.
- Cost: Opus-only substitution (spec owner, implementer, benchmark, reviewer,
  mutation all Opus 5; implementer without advisor); `/usage` totals and
  pass counts recorded at closure.
