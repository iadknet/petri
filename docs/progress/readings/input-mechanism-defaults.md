# Input-mechanism defaults (2026-10-12)

User-directed production default change, not a roadmap feature: T20.F04
neutral input recruitment becomes `WholeFamily` and T20.F05 structured
heritable refinement becomes `true` by default, no longer experimental arms.
Both serde defaults change with the struct defaults, so a recipe that omits
them gets them on. Setting `Off` or `false` selects the previous path, and
the recruitment-Off hash pin keeps its previous value.

Verification plan: benchmark profiles `gate + goal`, mutation gate `not run`
(config defaults only), slow tests `run` (mutation draws change).

Tested commit `903039e9` (rebased onto `f4c1c315`): `make check` exit 0,
`make rust-test-slow` exit 0 (3 recruitment-paths pins and 6 lab runs), and
`cargo test -p v3-core --test viability` 28 passed.

## Benchmark verdicts

Both profiles ran once on `903039e9` with `BENCH_WAIT_BLOCKERS=mutants`
beside the user's live server, so wall-clock readings are on a shared host.

| Profile | Reference | Exit | Severe | Flags |
| --- | --- | --- | --- | --- |
| Gate | `t11-f30-kind-drives.json` | 0 | no | `plasticity_updates` +15.6% |
| Goal | `t11-f30-kind-drives-goal.json` | 3 | yes | `vm_steps` +72.1% (severe), `pass_cap_hits` +11.4% |

Summaries: [gate](../features/input-mechanism-defaults.json),
[goal](../features/input-mechanism-defaults-goal.json). Raw reports are in the
main checkout's ignored `.bench-artifacts/input-mechanism-defaults/`.

The user chose to compare Canyon over more seeds before landing (below), and
the change landed with both summaries appended to the series as ordinary
closed entries; no epoch was re-pinned.

## Goal readings against T11.F30

| World / seed | Plateau | Minimum | Final | Extinct at | VM steps per creature-tick | Changed per birth |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Orchards / 11, T11.F30 | 5,068 | 3,231 | 4,562 | — | 0.873 | 0.1608 |
| Orchards / 11, new | 4,544 | 3,122 | 4,259 | — | 0.681 | 0.1558 |
| Canyon / 22, T11.F30 | 668 | 16 | 1,249 | — | 0.692 | 0.1920 |
| Canyon / 22, new | 0 | 0 | 0 | 290 | 0.042 | Undefined |
| Confluence / 33, T11.F30 | 3,376 | 247 | 3,676 | — | 0.576 | 0.1716 |
| Confluence / 33, new | 3,710 | 256 | 3,439 | — | 2.439 | 0.1638 |

- **Canyon died in the founding crash.** Population at ticks 100, 200 and 290
  was 8,560, 28 and 0. At T11.F30 the same seed fell to 48 at tick 200 and to
  16 later, then recovered to 1,249. One seed of the world that already missed
  its persistence floor at T11.F30 cannot separate chance from cause.
- **The severe VM-steps reading is a trajectory effect.** It is not spread
  across worlds: Orchards fell and one Confluence population quadrupled its
  VM steps, while the Graph mechanisms this change enables run no VM code.
  VM steps remain a small share of creature energy.
- Temporal memory sensitivity (operator state, creatures that differ from
  both perturbations): Orchards 8 to 1, Canyon 0 to 0, Confluence 39 to 188.

## Canyon over eight seeds

The Canyon recipe at 2,000 ticks, seeds 22, 122, 222, 322, 422, 522, 622 and
722, with `bench --profile sweep` on the tested binary. The old arm overlays
`neutral_input_recruitment: Off` and `structured_heritable_refinement: false`.
Its config digest is the effort-cost-defaults Canyon digest, and the new arm's
is this change's, so each arm is the production Canyon config of its side. The
new arm reproduces the goal run's seed 22 extinction at tick 290. The old arm
runs on this binary, so it includes T02.F08 partial bites, which the T11.F30
goal reading predates.

| Seed | Old final | Old extinct at | New final | New extinct at |
| ---: | ---: | ---: | ---: | ---: |
| 22 | 1,471 | — | 0 | 290 |
| 122 | 1,765 | — | 957 | — |
| 222 | 361 | — | 679 | — |
| 322 | 0 | 629 | 738 | — |
| 422 | 1,552 | — | 301 | — |
| 522 | 1,423 | — | 0 | 313 |
| 622 | 1,274 | — | 1,190 | — |
| 722 | 339 | — | 1,536 | — |

- Extinctions: 1 of 8 old, 2 of 8 new, both new ones in the founding crash.
- Final population: median 1,349 old against 709 new, mean 1,023 against
  675. An exact two-sided rank test over the 16 runs gives p = 0.23, so the
  difference is within seed spread at this horizon.
- Births per creature-tick: 0.0217 to 0.0231 old, 0.0221 to 0.0229 new.

Raw sweep reports stayed in the session scratchpad and are not kept.

## Re-pinned values

Trajectory pins were confirmed by a second run.

| Pin | Old | New |
| --- | --- | --- |
| `applied_trajectory.rs` sampled trajectory, default | `08bcb284…` | `1d2ecdf7…` |
| same, rate 0.0 | `b7d16c67…` | `1e29f964…` |
| `applied_trajectory.rs` applied guard, default | `fce26031…` | `8d8a3874…` |
| same, rate 0.0 | `876245ce…` | `fb71cb3b…` |
| `baseline_worlds.rs` `legacy_default_short_run_identity`, default | 11106923287936488614 | 1518722661182035271 |
| same, rate 0.0 | 11416820348583511793 | 1085458628830102075 |
| `bench_artifacts.rs` goal recipe config digests | `ff17c388…`, `ece66650…`, `f1752c87…` | `896d889f…`, `9044de71…`, `c0f27465…` (restoring false and Off recovers the old three) |
| `drift.rs` module-dating walk (created, dispatched, internal-change censored present) | 6, 3, 5 | 7, 4, 6 |
| slow tier, prepared lineages Graph Drift / Selection | `[20,17,15,7]` / `[20,20,20,19]` | `[22,17,16,5]` / `[22,22,22,21]` |
| slow tier, prepared lineages VM Drift / Selection | `[24,18,18,11]` / `[24,24,24,23]` | `[25,19,19,11]` / `[25,25,25,24]` |
| slow tier, cost selection Graph / VM (censored) | `[21×4]` (11) / `[23×4]` (9) | `[23×4]` (9) / `[24×4]` (8) |
| `gestation.rs` lowered-cap fixture seed | 433 | 740 |
| `bench/tests.rs` real-run-path fixture seed (32², 8 founders, 250 ticks) | 207 (extinct at 127) | 6 |
| `bench/run/tests.rs` persistence-cadence fixture seed (24², 16 founders) | 1 (extinct at 99) | 4 |

The recruitment-Off pin in `recruitment_tests.rs` keeps 13078250158860285847
with both mechanisms set off explicitly.
