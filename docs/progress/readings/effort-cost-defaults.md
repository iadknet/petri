# Effort-cost defaults (2026-10-10)

User-directed production default change, not a roadmap feature:
`noop_cost` 0.05 → 0.4, `reproduce_attempt_cost` 0.05 → 1.0,
`move_cost` 0.2 → 1.6 (defaults and normalization fallbacks in
`crates/v3-core/src/config/simulation.rs`). `eat_cost` (0.05) and
`reproduce_cost` (0.1) are unchanged; the 0.4 floor lifts every eat to 0.4.

The seven root `world-recipe-*.json` files set `move_cost` 0.1 and
`noop_cost` 0.05 explicitly and keep them; none sets
`reproduce_attempt_cost`. The `experiments/worlds/` recipes set none of the
three, so the goal worlds inherit the new defaults.

## Sweeps that chose the values

Goal profile (`goal-worlds-v1`, one seed per world, 2,000 ticks), each arm a
scratch build with the default edited, run with `make bench PROFILE=goal`.
Summaries are local only, under the main checkout's ignored
`.bench-artifacts/noop-floor-exp/`. The first sweep's baseline is T16.F02's
committed goal summary. Readings are Orchards / Canyon / Confluence.

| Local summary | sha256 | Bytes |
| --- | --- | ---: |
| `summary-0p1.json` | 3e1bfbfd8607e903… | 335651 |
| `summary-0p2.json` | 53eeeba3eff268aa… | 331057 |
| `summary-0p4.json` | 4ee3393fba3dbc6f… | 333301 |
| `summary-0p8.json` | 0fb04cd0867e01dd… | 351593 |
| `summary-floor0p4-attempt-1p0.json` | 82136e8c34ba9795… | 329631 |
| `summary-floor0p4-attempt-4p0.json` | 83866c2df21b7f38… | 302077 |
| `summary-floor0p4-attempt-16p0.json` | 29824b600ff5b20a… | 329571 |
| `summary-floor0p4-att1-move-0p6.json` | 63d22764281fb1a7… | 283969 |
| `summary-floor0p4-att1-move-1p0.json` | 68fe52b1b7579d01… | 343564 |
| `summary-floor0p4-att1-move-1p6.json` | e61b858bd0574a7b… | 370561 |

Sweep 1, `noop_cost` (the floor); other costs at T16.F02 defaults:

| Reading | 0.05 | 0.1 | 0.2 | 0.4 | 0.8 |
| --- | --- | --- | --- | --- | --- |
| Failed eat fraction | 0.61 / 0.19 / 0.52 | 0.33 / 0.42 / 0.57 | 0.53 / 0.30 / 0.43 | 0.27 / 0.13 / 0.18 | 0.01 / 0.01 / 0.03 |
| Food intake per creature-tick | 1.07 / 0.75 / 0.88 | 0.95 / 0.86 / 0.99 | 1.21 / 0.89 / 1.00 | 1.61 / 1.17 / 1.16 | 1.94 / 1.77 / 1.84 |
| Births per reproduce attempt | 0.20 / 0.08 / 0.08 | 0.14 / 0.09 / 0.10 | 0.24 / 0.15 / 0.09 | 0.30 / 0.19 / 0.29 | 0.40 / 0.28 / 0.33 |
| Plateau population | 4263 / 4233 / 5142 | 5513 / 4020 / 5008 | 4032 / 5296 / 4888 | 4050 / 5324 / 3377 | 5344 / 869 / 2934 |
| Surviving founder clades | 1 / 7 / 8 | 3 / 13 / 5 | 2 / 8 / 2 | 5 / 11 / 3 | 59 / 11 / 7 |

Sweep 2, `reproduce_attempt_cost` at floor 0.4:

| Reading | 0.05 (floored to 0.4) | 1 | 4 | 16 |
| --- | --- | --- | --- | --- |
| Births per reproduce attempt | 0.30 / 0.19 / 0.29 | 0.37 / 0.26 / 0.41 | 0.29 / 0.34 / 0.43 | 0.34 / 0.22 / 0.26 |
| Attempts rejected on parent energy | 0.21 / 0.18 / 0.14 | 0.17 / 0.14 / 0.14 | 0.51 / 0.27 / 0.34 | 0.53 / 0.66 / 0.62 |
| Plateau population | 4050 / 5324 / 3377 | 3825 / 4358 / 1737 | 3320 / 3891 / extinct (tick 846) | 3418 / 377 / 2798 |

Sweep 3, `move_cost` at floor 0.4 and attempt 1.0:

| Reading | 0.2 (floored to 0.4) | 0.6 | 1.0 | 1.6 |
| --- | --- | --- | --- | --- |
| Plateau population | 3825 / 4358 / 1737 | 2035 / 2771 / extinct (tick 573) | 6214 / 338 / 3155 | 4536 / 2500 / 3884 |
| Trough population | 17 / 618 / 30 | 12 / 405 / 0 | 566 / 95 / 80 | 3702 / 342 / 586 |
| Food intake per creature-tick | 1.36 / 1.24 / 1.11 | 1.52 / 1.37 / 1.28 | 1.84 / 1.68 / 1.99 | 2.44 / 2.32 / 2.43 |
| Blocked move fraction | 0.12 / 0.31 / 0.23 | 0.11 / 0.26 / 0.20 | 0.10 / 0.20 / 0.17 | 0.08 / 0.14 / 0.13 |
| Surviving founder clades | 5 / 10 / 2 | 3 / 10 / 0 | 49 / 5 / 4 | 81 / 21 / 28 |

One seed per world: behavior trends (failed eats, blocked moves, intake,
births per attempt) are consistent across worlds; population outcomes are
non-monotonic (an extinction at an intermediate value in sweeps 2 and 3) and
read as early-bottleneck noise. The user chose 0.4 / 1.0 / 1.6 on these
readings.

## Decision: founder margin before senescence

User ruling (2026-10-10): the founder reproduce margin (T17.F01 invariant 5)
is guaranteed only before senescence. Older founders may waste attempts, as
fertility declines with age.

The energy gate needs `energy − effort − cost ≥ min_reproduce_energy` (30),
with effort `1.0 × m` and cost `0.1 × m`, where the age multiplier is
`m = 1 + 9 × ((age − 100) / 100)²` (grace 100, cap 200, max 10). For the
tightest profile (V3Alpha1, gate 32, fraction 2/3):

| Age | m | 32 − (effort + cost) | Gate (30) |
| --- | --- | --- | --- |
| ≤ 100 | 1.0 | 30.90 | passes |
| 130 | 1.81 | 30.01 | passes |
| 131 | 1.86 | 29.95 | rejected |
| ≥ 200 | 10.0 | 21.0 | rejected |

The property test `founder_profiles_are_accepted_at_every_energy_above_their_gate`
draws ages 20 to `FOUNDER_MARGIN_MAX_AGE` (130); the other four profiles hold
further (Conservative to m ≤ 2.60, Balanced 5.05, RichOffspring 6.06).
`founder_past_the_margin_age_is_refused_at_its_gate` pins the boundary:
V3Alpha1 just above its gate at age 131 is `RejectedEnergyConstraints`. The
regression seed for age 131 is removed from
`crates/v3-core/proptest-regressions/simulation/actions/mod.txt`, since the
narrowed strategy cannot draw it. The tick-level check
`founders_only_run_has_no_energy_rejected_reproduce_attempts` passes.

## Re-pinned values

Trajectory pins were confirmed by a second run.

| Pin | Old | New |
| --- | --- | --- |
| `viability.rs` `founder_only_trajectory_digest_is_pinned`, default rate (births, final population, digest) | (195, 0, `89d60dc0f467a597a6dc32229cf7cb0c02c378c0931f705c511f9adeb1a4c477`) | (99, 0, `0948b6d22e1ff9e8186c9e8622c4c6fbc5aa8d1ad6a7bbb735eb678b4d424c85`) |
| same, gestation rate 0.0 | (297, 0, `4a5941e899a878853adf125db91a0430fbbbce2139aad6ebab9cf43de646d61c`) | (94, 0, `50f761554d2f14dacd30a58b74fe5e33aa8c07f7b034ef1008a25d47bec5dd2b`) |
| `viability.rs` `founder_eats_primary_type_and_preserves_other_food` | `energy > 20.0` | `food_intake_by_type == [2.5, 0.0]` (the tick's eat + move nets 20 + 2.5 − 0.4 − 1.6 − 0.5 decay − 0.0097 carrying = 19.99) |
| `applied_trajectory.rs` sampled trajectory, default | `98e6517eec9def7283343fd0a914713379a3638c5c9a2bd1b4e73cb21ecc3d2f` | `33f3a73ba7900f92f76da8e2699e6536cdb0b355cfa3fb328580151a91d92776` |
| same, rate 0.0 | `0a6131485f7f14b565d8e3daf2f7d0712610ce90da9c1d63023278f9818413a8` | `7a674e6747390ed9d6afcec21482d74f8acd02b17b90e0f3faa6f21df2b3c760` |
| `applied_trajectory.rs` applied guard, default | `8e839ff92e03d52141afcc563197321c81ea56650062e129d32c67c3a20306d0` | `60861ddd3d70c0a4b0be08d1e40a1e4601f46065390bf3cd6c40a72b8ae1d7f4` |
| same, rate 0.0 | `54fab293c4d516a3a99d680899bfd5c97da2ac3d008d1a87cd7319b9f19ccc9a` | `c08aab7d93216e455424b63cc9cb03754cb1562ceb2a8b385eb503df378efe8c` |
| `baseline_worlds.rs` `legacy_default_short_run_identity`, default | 11682689850618140524 | 7229388965272740622 |
| same, rate 0.0 | 17199981815595828116 | 7526238675349254538 |
| `v3-cli` `bench_artifacts.rs` goal recipe config digests (Orchards, Canyon, Confluence) | `sha256:12ea63f7…`, `sha256:6112372c…`, `sha256:39d3a045…` | `sha256:ff17c388774137ca48df2e21e7e0949f648c1aa6e04799f439cdae7772022a4b`, `sha256:ece666508c5c6886b80afb723d4651e8092dad5bfcb40650d6873da798193210`, `sha256:f1752c872a0e45ae9cd74787940be0b29bd38ae7f8c870a0c14c17d563f8c420` |
| `v3-cli` `bench/tests.rs` occupancy-grid fixture seed | 2 (extinct at 153) | 22 (6 creatures over 6 cells) |
| `v3-cli` `bench/tests.rs` post-tick and census fixture seed | 16 (extinct at 150) | 22 |
| `v3-cli` `bench/run/tests.rs` persistence-cadence fixture seed (24², 16 founders) | 1 (extinct at 91) | 15 |
| `v3-cli` `bench/run/tests.rs` goal-case structure horizon | 60 ticks (Orchards 9 creatures) | 40 ticks |
| v3-core tests asserting the 0.05 floor (two typed-eat, three steal, one exhausting steal) | 0.05 | 0.4 |
| `effort.rs` `reproduce_below_cost_pays_only_the_attempt_effort` parent energy | 1.0 (killed by the 1.0 attempt) | 1.5 |
| `window_seam.rs` `traced_and_untraced_runs_are_identical_for_vm_founders` seed (telemetry-seams; a traced target dies in its window and its action-log oracle goes with it) | 9 | 14 |
| `gestation.rs` identity fixture costs (extinct before any delivery) | defaults | explicit floor 0.05, attempt 0.05, move 0.2 |
| `v3-lab` `comparator.rs` barrier comparator start energy, lifetime | 100, 100 (died at 53) | 200, 50 |
| `v3-lab` `eval.rs` boxed scripted actor `move_cost` (died at 96) | default | 0.5 |
| `v3-lab` `eval.rs` unreachable-food start energy | 100 | 200 |
| `v3-lab` `readings/tests.rs` founder-shape start energy (2 deaths) | 100 | 200 |
| frontend `EnergyCostsSection` defaults (move, noop, attempt) | 0.2, 0.05, 0.05 | 1.6, 0.4, 1.0 |

The recipe digest guard gains a step: restoring `noop_cost` 0.05,
`reproduce_attempt_cost` 0.05 and `move_cost` 0.2 recovers the T16.F02
digests, so the config moved only by these three values.

Small sweep worlds rarely survive the founding crash: in the 32², 8-founder
sweep config only seeds 22 and 50 of 0..=60 reach tick 250, and in the 24²,
16-founder config only seeds 15 and 33 of 0..=40 reach tick 105.

## Verification (2026-10-10)

- `cargo test -p v3-core --test viability`: 28 passed, twice.
- `cargo check --workspace --all-targets` and `make rust-clippy`: clean.
- `make -k check`: exit 2. The only failing test is
  `gate_profile_has_no_severe_regression_against_series_references`, which
  stays expected until the benchmark epochs are re-pinned: "counter vm_steps
  regressed severely against docs/progress/features/t16-f02-effort-priced-actions.json:
  current=0.095108 reference=Some("0.027946") delta%=Some("240.327775")".
- `cargo test -p v3-cli --no-fail-fast --test bench_artifacts --test cli --test telemetry`:
  37, 13 and 13 passed.

## Benchmarks and epoch re-pin

| Profile | Command | Exit | Against | Severe |
| --- | --- | --- | --- | --- |
| gate | `make bench PROFILE=gate FEATURE=effort-cost-defaults` at `345aefd6` | make 2 (CLI 3, severe) | T16.F02 gate | `vm_steps` +240.3%, `pass_cap_hits` +71.0% |
| goal | `make bench PROFILE=goal FEATURE=effort-cost-defaults` at `4c8c0d0a` | 0 | T16.F02 goal | none (`vm_steps` −56.9%, `actions_applied` −27.0%, `births` +22.3% flag) |

| File | sha256 | Bytes |
| --- | --- | ---: |
| raw gate (main `.bench-artifacts/effort-cost-defaults/gate.json`) | 0391537e769668ff… | 102059 |
| raw goal (main `.bench-artifacts/effort-cost-defaults/goal.json`) | 60edc5f0adc1e60f… | 666982191 |
| `docs/progress/features/effort-cost-defaults.json` | 737272ce681a14f7… | 19984 |
| `docs/progress/features/effort-cost-defaults-goal.json` | 40f4bb1979d881de… | 370069 |

- Gate populations end at 563 / 546 / 593 (seeds 11 / 22 / 33) against T16.F02's
  2,399 / 2,272 / 2,357; none extinct. Goal final populations 4,283 / 2,354 /
  4,213 (Orchards / Canyon / Confluence); none extinct.
- Identity: the goal summary's `deterministic` block equals the sweep arm
  `summary-floor0p4-att1-move-1p6.json` exactly, so the landed change is the
  configuration the user chose from.
- Both severes are the user's 2026-10-10 decision to adopt these defaults.
  `gate.epoch_baseline` and `goal_worlds.epoch_baseline` in
  `docs/progress/benchmark-series.json` now point at the two summaries above,
  and both are appended to their `closed` lists; thresholds are unchanged.
