# T24.F02 — Parallel Food Regrowth

**Status**: In Progress
**Last updated**: 2026-10-07
**Feature**: T24.F02
**Track**: [T24 — Simulation Throughput](../../roadmaps/t24-simulation-throughput.md)

## Goal

Food regrows by the repaired rules on every core at once. `World::grow_food`
(`crates/v3-core/src/kernel/ordinary_food/ecology.rs` `grow`) runs each of its
full-grid passes on the rayon pool in effect, draws every food random choice
from a keyed per-cell stream instead of the run RNG, and sums each cell's
contributions in a fixed order, so a seeded run is byte-identical at any
thread count and across processes. On the default 1600-by-1600 world the
`food_growth` phase (T24.F03's instrument) takes at most half the time per
tick at 8 threads that the build on `main` at 3b904894 (T24.F03 and T24.F01
closed) takes on the same host. No world rule, default or config field
changes; the trajectory move comes only from the changed random stream and
summation order and is predeclared below.

## Non-Goals

- No change to what food does: no growth, spread, recovery, grazing,
  depletion or inhibition rule, threshold, default or config field moves
  (track criterion 5). The reference rule stays `docs/reference/v3-world-grid-spec.md` §5.
- No Philox or other RNG crate; no new dependency (track Notes).
- No parallelization outside `grow_food`: sensor assembly, actions, creature
  aging and death removal stay as they are.
- No GPU path (deferred by the track).
- No change to the `food_growth` phase, its pass spans, the metric family,
  the dashboard or the benchmark report fields that T24.F03 defined.
- No sequential fallback switch, thread-count setting or grid-size
  configuration: the pass runs on whatever pool is in effect, as cognition does.

## Inputs and Invariants

| Input | Where | What F02 takes from it |
| --- | --- | --- |
| Track row and Notes | `docs/roadmaps/t24-simulation-throughput.md`, row T24.F02 (depends on T24.F01, T24.F03), criteria 3–5, Notes "Why T24.F02 is a rewrite rather than a `par_iter` swap", "`grow_food` makes about seven full-grid passes", "Every T24 feature runs the ordinary gate…" | The goal; the four design points (keyed stream by run seed, tick, food type and cell index; gather form summing in a fixed neighbour order; `f32` telemetry sums reduced in a fixed order; recovery spawns may stay sequential); speedup measured as interleaved pairs on a short default-size seeded workload against the parent build under `scripts/bench-wait`; the spec reads the pass decomposition before fixing it; no Benchmark gate exemption. |
| Requirement | The user's goal: T24.F02's speedup is measured against the build then on `main`, using T24.F03's food-growth phase | The reference build is `main` at 3b904894 (the T24.F01 closure), not an older parent. |
| The pass today | `ecology.rs` `grow` (lines 362–647 at 3b904894): `recover_and_deposit` (three grid passes), `GrazingLayer::recover` (one per type), `build_type_inhibition_weighted_sums` (one per inhibiting type), then per type `total_food_by_type`, `vec![0.0; total_cells]`, the growth-and-spread loop, the recovery spawn loop, the write-back loop; finally the inhibited-flag count | Every pass is a row-major loop over the grid reading the pre-growth snapshot (`state`), the fertility grid, the grazing grid, the depletion grid, the barriers and the occupancy grid, and writing either its own cell or (spread, recovery) another cell. Each `next_density` buffer is allocated per type per tick (10 MB at 1600², zeroed). |
| Draws today | `rng.gen_range(0..count)` once per spreading cell in row-major order (line 520), `rng.gen_range(0..total_cells)` once per recovery attempt (line 563); `rng` is `sim.rng` (`SmallRng`), which also feeds the turn-queue shuffle and `reproduce_rng` (`tick.rs` 1370, 1386) | Taking food off `sim.rng` shifts every later run-RNG draw from tick 0 of every run, so both profiles' trajectories move once (predeclared). |
| Spread term | `spread_after = local_after_inhibition(S) × spread_density_ratio × neighbor_fertility(T)`, `spread_base = local_base(S) × spread_density_ratio × base_neighbor_fertility(T)`, `spread_delta = spread_after × multiplier(T)` (lines 521–548); `neighbor_fertility(T)` and `base_neighbor_fertility(T)` depend on the target's own fertility, grazing modifier and inhibition penalty | A target can finish every spread term from two source-side numbers (`local_base`, `local_after_inhibition`) and its own cell-side numbers, which is what makes the gather form cost no recomputation. |
| Per-cell own term (T24.F01) | `next_density[idx] = clamp(next_density[idx] + source + local_delta)` (line 495); T24.F01 invariant 1: the written density is `clamp(own + Σ spread + Σ recovery, 0, max_density)`, every term non-negative, incremental clamping exact | The rule this feature keeps; T24.F01 invariant 2 named the two order dependences this feature removes (draw assignment in visit order; accumulation in visit order). |
| Keyed hash | `crates/v3-core/src/creature/identity.rs` `splitmix64` (private finalizer, a 64-bit bijection); T24 Notes: a small in-tree keyed hash, no Philox crate | The stream is the splitmix64 finalizer over a mixed key: a per-tick-per-type word hoisted out of the loop, one finalizer round per cell. |
| Rayon in `v3-core` | `tick.rs` Phase 1b `par_iter_mut`; `neighborhood/*` `into_par_iter`; `crates/v3-cli/src/bench/run.rs` runs a profile inside a private pool under `--threads` and records `environment.threads`; `tests/reproducibility.rs` builds pools of 1 and 4 threads | The pool in effect is whatever the caller installed; `--threads 8` on this host (8 logical cores, 6 performance and 2 efficiency) equals the global pool. |
| Scratch plumbing | `ordinary_food/mod.rs` `FoodResource` fields `claim_scratch`, `owner_snapshot`, `density_snapshot` (allocated, unused, passed to `grow` as `_claim_scratch`, `_owner_snapshot`, `_density_snapshot`) | The home for the per-cell buffers this feature keeps between ticks. |
| Constructors | `simulation/simulation.rs` `Simulation::new(world, creatures, tick, config, seed)` seeds `rng` from `seed` and calls `world.reconfigure_food`; `seed_simulation` and every other constructor go through it; `v3-server` patches config mid-run | The food stream key is a `Simulation` field derived from `seed` in `Simulation::new`, so it exists for every constructor and survives `reconfigure_food`. |
| Pass spans (T24.F03) | `pass_clock.rs` `FoodPass` (eight names), `PassClock::stop` around each region; T24.F03 invariants 3, 5 and 8 (span set, order, check 6 of `scripts/telemetry-dashboards-check`) | Unchanged: a span wraps its region whatever runs inside; `growth_spread` covers the claim and gather passes of one type. |
| Comparison rule | T10 track Notes (flag above +10% work per creature-tick or +25% wall; severe above +50% work or a doubling of wall; a predeclared, justified cost re-pins the epoch in the closing commit); `crates/v3-cli/src/bench/comparison.rs` (nine `COUNTER_NAMES`, zero-baseline rule at `:542`, wall per creature-tick at `:565`) | The Performance predeclaration and the closure order (re-pin before the mutation gate). |
| References | T24.F01 summaries `docs/progress/features/t24-f01-order-free-food-spread.json` (gate) and `-goal.json` (goal: its paired per-world readings are the reference this feature holds trajectories against); epochs: gate T11.F25, `goal_worlds` T24.F01, in `docs/progress/benchmark-series.json`. Measured at T24.F01: `food_growth_ms` per tick about 0.36 ms (gate), Orchards 62.7, Canyon 37.1, Confluence 54.0 ms (goal) | The comparison and the speedup context. |
| Pins | `tests/viability.rs` `founder_only_trajectory_digest_is_pinned`; `tests/applied_trajectory.rs` (two pins); `tests/baseline_worlds.rs` `legacy_default_short_run_identity`; `crates/v3-cli/src/bench/tests.rs` `the_real_run_path_carries_a_full_occupancy_grid_at_every_checkpoint` (seed reselect rule in its comment); `tests/reproducibility.rs` operator-coverage assertions | All move with the stream and are re-pinned once after two agreeing runs (each file's own rule). |
| Speedup workload | `v3-cli bench --profile sweep` (`--width 1600 --height 1600 --founders 10000 --ticks 100 --seeds 11 --threads 8`) writes a full report whose `environment.phase_wall_clock_ms_per_seed[0].food_growth_ms` is the seed's cumulative phase time; the sweep runs seeding, the tick-zero connectivity read and the per-tick persistence and world observations, and no neighborhood half (`run.rs:462–470`) | One run is estimated at 15–30 s under load (the goal profile ran about 110 ms per tick at T24.F03, plus seeding; unverified until the first pair); the phase reading is in-process elapsed time, so a pair's ratio is robust to load that both sides share but not to a spike that hits one side. |

Decomposition. The form is fixed: every pass over the grid becomes a parallel
pass over rows (`par_chunks`/`par_chunks_mut` of `width` cells, or an
equivalent whose unit of work is a row); a pass writes only the cell it
visits; anything a pass sums is a per-row partial reduced in row order; the
spread is claim then gather on the keyed stream. The region-by-region plan
below is provisional until the pass readings of implementation task 1 exist
(the track Notes ask for the reading before the decomposition is fixed; the
fresh worktree has no build, so the spec could not take it): the readings may
fuse passes or change buffers inside the T24.F03 span boundaries, and the
readings file records what they changed and why.

| Region (T24.F03 span) | Form after F02 | Thread-independent because |
| --- | --- | --- |
| `depletion_recovery` | three row-parallel passes (decay, deposit, mean) | each writes its own cell; `mean_depletion` and `occupied_cells` are per-row partials reduced in row order |
| `grazing_recovery` | one row-parallel pass per type | same; `mean_modifier`, `grazed_cells` per row, reduced in row order |
| `inhibition_sums` | one row-parallel pass over all inhibiting types | writes its own cell |
| `density_total` | row-parallel per-row sums reduced in row order | the reduction order is fixed (it differs from today's flat sum at rounding level: predeclared) |
| `growth_spread` | **claim** (row-parallel, per cell `c`: `own = source + local_delta`, `local_base`, `local_after_inhibition`, and the drawn target `pick(c)` as a cell index or none, from the keyed stream over the candidate list built as today, `ecology.rs:502–518`: the N, E, S, W entries that resolve under the edge mode and are passable, entries repeating on a wrapped grid of width or height 1 or 2 and resolving to `c` itself on 1×1; own inhibited flag) then **gather** (row-parallel, per cell `t`: `next = clamp(own(t) + Σ_s spread_delta(s, t))` over the *distinct* source cells `s` among t's N, E, S, W neighbours, in first-occurrence order, each counted once when `pick(s) = t`; the target-side inhibited flag and the suppression partials) | every write is to the visited cell; the pick of a source is a pure function of the key, not of scheduling; the sum order is N, E, S, W over distinct sources, so a source that is two of t's neighbours on a narrow wrapped grid delivers its one contribution once, as the scatter at `:548` does |
| `recovery_spawns` | sequential, as today, drawing attempt `k` from the keyed stream | few draws (`total_cells × recovery_spawn_rate`), sequential |
| `write_back` | row-parallel copy into the type's grid with per-row `occupied_cells` and `total_density` partials reduced in row order | writes its own cell |
| `inhibited_count` | parallel count | integer |

Invariants:

1. Rule unchanged. For each food type and each non-barrier cell the density
   written back is `clamp(own + Σ spread + Σ recovery, 0, max_density)` with
   the terms T24.F01 invariant 1 defines, every density read from the
   pre-growth snapshot, barrier cells written 0 and never a target. Each
   spreading cell (`source ≥ spread_threshold`, `local_after_inhibition > 0`)
   still sends exactly one contribution to one uniformly chosen entry of its
   candidate list (the N, E, S, W neighbours that resolve under the edge mode
   and are passable, with the repeats a narrow wrapped grid produces); a cell
   with an empty list sends none. The telemetry fields keep their
   definitions (`growth_suppressed_by_*`, `cells_with_type_inhibition`,
   grazing and depletion fields, `occupied_cells`, `total_density`).
2. Keyed stream. Every draw `grow` makes is a pure function
   `food_draw(key, tick, type, purpose, index) -> u64` of the simulation's
   food stream key (`splitmix64(seed ^ FOOD_STREAM_TAG)`, a `Simulation`
   field set in `Simulation::new` from the run `seed`, never from
   `world_seed`), the tick, the food type index, a purpose tag (spread
   target, recovery cell) and the cell index or attempt index, built from the
   splitmix64 finalizer. A draw is mapped to a range `n` by the 64-bit
   multiply-shift (`(draw as u128 * n as u128) >> 64`), whose bucket bias is
   below `n · 2^-64` (below `2^-42` for the 1600² recovery range, below
   `2^-61` for four neighbours): uniform for every purpose here, with no
   rejection loop. `grow` takes the key and no `Rng`; `sim.rng` is not
   touched by food growth.
3. Byte identity. For a given seed the run is byte-identical across
   processes and across rayon pools of any size: every parallel pass has a
   per-cell or per-row unit of work that does not depend on the pool, and
   every reduction is in row order. The `f32` results therefore depend on the
   order fixed here (N, E, S, W; rows ascending), not on the thread count.
4. Spans unchanged. The eight `FoodPass` names, their order, their
   per-type flags and their "runs when" conditions (T24.F03 table) hold;
   `growth_spread` covers claim and gather; no span is added or split.
5. No grid-sized allocation per tick on the timed path. The per-cell buffers
   (`own`, `local_base`, `local_after_inhibition`, `pick`, `next_density`,
   the inhibited flags, the inhibition sums) live in `FoodResource` in place
   of the unused scratch fields and are resized only when the grid or the
   catalog changes; bounded per-type summaries (`per_type`, the grazing
   summaries) and the seam's pass timings may still be allocated per tick as
   today.
6. No field, default or digest change. The effective-config digests of both
   summaries equal T24.F01's; the pinned recipe digests do not move; no
   `SimulationConfig` or `RuntimeConfig` field is added.
7. Cost. At 8 threads on the default 1600-by-1600 world the `food_growth`
   phase takes at most half the time per tick of the 3b904894 build on the
   same host (track criterion 3, measured as the Performance section says).
   Memory: the retained buffers are about `(4 + 4 + 4 + 4 + 4) × cells` bytes
   plus the flags and sums (about 60 MB at 1600², once), replacing the
   per-tick 10 MB allocation.

## Telemetry

No new signal. The `food_growth` phase and its pass spans (T24.F03) are the
instrument; the benchmark report's `environment.threads` and
`phase_wall_clock_ms_per_seed[].food_growth_ms` carry the reading. No
mechanism switch or config value is added, so nothing new reaches
`run.started` or a phase group.

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `food_growth` phase and its eight pass spans | existing span set | unchanged pass regions; `growth_spread` now covers claim and gather | unchanged (T24.F03 invariant 6) |

## Implementation Tasks

- [ ] Read the pass decomposition on the 3b904894 build before changing it:
      per-pass durations of one default-size tick (`v3-cli run` at 1600² with
      tick traces on the running `petri-telemetry` stack, or the
      `telemetry-seams` `last_tick_food_passes` seam), recorded as a table in
      `docs/progress/readings/t24-f02.md`. The table says which passes
      dominate and what sequential remainder the plan must beat; the
      implementer may then fuse passes or change buffers inside the T24.F03
      span boundaries (the form of the decomposition stays fixed) and records
      each change and its reason next to the table.
- [ ] Keyed stream: `food_draw` and the range mapping in
      `ordinary_food/` (reusing `creature::identity::splitmix64` at
      `pub(crate)` visibility, one definition; no new crate);
      `food_stream_key` on `Simulation`; `grow`/`FoodResource::grow`/
      `World::grow_food` take the key and tick, not an `Rng`; every caller and
      test fixture updated. TDD: a test that the same key, tick, type and
      index give the same draw and that two of them differing in any one
      argument differ; a test that spread targets on a uniform saturated
      patch are shared across the four directions within a bound (a
      fixed-seed count, not a distribution proof).
- [ ] Claim and gather: the growth-and-spread loop becomes the two
      row-parallel passes of the decomposition table, with the retained
      buffers of invariant 5. TDD: the conservation proptest
      (`growth_pass_total_keeps_every_spread_contribution`) stays as it is
      (it needs no picks and guards against lost or doubled spread on its
      own); a new per-cell oracle computes each source's single delivery from
      `food_draw` and the §5 rule with grazing, depletion and inhibition
      enabled and asserts every cell's written density (the existing oracle
      at `ecology.rs:1305–1315` disables those mechanisms); fixtures: two
      neighbours picking one target whose own growth plus both contributions
      exceeds `max_density`; wrapped 1×1, 1×2, 2×2 and 1×N grids where a
      source is two of a target's neighbours or its own target; a cell with
      an empty candidate list.
- [ ] The other passes: depletion recovery, grazing recovery, inhibition
      sums, density total, write-back and the inhibited count become
      row-parallel with row-order reductions; recovery spawns stay
      sequential on the keyed stream.
- [ ] Reproducibility: a `tests/reproducibility.rs` test runs a seeded
      simulation with barriers, grazing, depletion, an inhibiting type, a
      spreading patch and a type below its recovery floor (so every pass of
      the table runs) for 20 ticks inside pools of 1, 4 and 8 threads and
      asserts per tick that the world fingerprint (every type's density,
      grazing and depletion grids by bits, fertility, barriers), the food
      growth summary fields by bits (pass timings excluded) and the
      population and accounting fingerprints are equal, with coverage
      assertions that spread, recovery spawns, grazing suppression and
      depletion suppression each occurred (`> 0`) in the run; the existing
      same-seed identity and terrain thread-count tests keep passing, and
      cross-process identity is the pinned digests reproducing.
- [ ] Pins: re-pin the five trajectory pins and the operator-coverage seed
      or horizon if needed, each after two agreeing runs; the bench fixture
      reselects its seed by its comment's rule.
- [ ] `phase_seam` tests, check 6 and the trace tests pass unchanged.

## Verification

- [ ] `cargo test -p v3-core --test viability` first (defaults and tick-loop
      mechanics change), then `cargo test -p v3-core` (ecology unit tests,
      the oracle and proptest with keyed picks, the stream tests), `cargo
      test -p v3-core --test reproducibility` (the new thread-count test
      named above), `cargo test -p v3-core --features telemetry-seams
      phase_seam`, and `cargo test -p v3-cli --test bench` -> results and
      the re-pinned values in `docs/progress/readings/t24-f02.md`.
- [ ] Speedup pairs (track criterion 3): the method, the per-pair
      `food_growth_ms` readings, the median ratio, the cap use and the
      verdict in `docs/progress/readings/t24-f02.md`; the Performance section
      states the method and the verdict.
- [ ] `make check` -> exit 0 (includes
      `gate_profile_has_no_severe_regression_against_series_references`,
      green at the mutation baseline after any re-pin).
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      and every survivor resolved as killed, equivalent, or deferred. The
      full survivor list stays here.
- [ ] Benchmark summaries stored at
      `docs/progress/features/t24-f02-parallel-food-regrowth.json` and
      `-goal.json`, local raw hash/byte count and verification time checked,
      series entries point to the summaries, and no new full report staged.

## Performance and Goal Impact

**Predeclaration — written before the run.** An engineering feature (no
mechanism, sensor, default or config field; the natural-analog rule does not
apply, track Notes). The reference build for the speedup is `main` at
3b904894 on this host (8 logical cores); the benchmark references are the
T24.F01 summaries and the epochs T11.F25 (gate) and T24.F01 (`goal_worlds`).

| Item | Predeclared expectation and acceptance |
| --- | --- |
| Speedup method | Interleaved pairs under `scripts/bench-wait`, both binaries built `--release` with the same toolchain: the parent `v3-cli` from a scratch `git worktree` of 3b904894 outside the feature worktree (removed afterwards), the feature `v3-cli` from the feature worktree. Workload per run: `bench --profile sweep --width 1600 --height 1600 --founders 10000 --ticks 100 --seeds 11 --threads 8 --telemetry off --out <scratch>/<side>-<i>.json --summary-out <scratch>/<side>-<i>-summary.json` (the production default config at the default size). Pairs: 6, order alternating pair by pair (parent then feature, feature then parent, ...), no other build, test, server or measurement running (the `petri-telemetry` stack stays up and is not a competing server). Reading per run: `environment.phase_wall_clock_ms_per_seed[0].food_growth_ms` divided by that seed's `ticks`; pair ratio `r = feature / parent`. Cap: 900 s of measured run time over the pairs; a run that fails or is killed voids its pair. |
| Speedup verdict | Pass when at least 4 pairs complete and the median `r ≤ 0.45` (criterion 3 met; min, max and all ratios recorded). Fail when at least 4 pairs complete and the median `r > 0.55`: one remediation pass by the implementer guided by the pass readings (the sequential remainder, rayon granularity, buffer reuse), then the pairs rerun once; a second fail is a blocker for the user (criterion 3 unmet). Near the line (median in `(0.45, 0.55]`): pairs continue, alternating as before, up to 12 pairs or the cap, and the verdict is pass when the final median is `≤ 0.50` with at least 8 pairs, else fail as above. Inconclusive when fewer than 4 pairs complete inside the cap, or the near-the-line extension reaches the cap under 8 pairs: rerun once when the host's 1-minute load has been below 10 for 60 s, or when the user directs; a second inconclusive is a blocker for the user (criterion 3 not shown), never a waived check. The budget is one cap per attempt (the pairs of a remediation rerun are a new attempt); each run is charged its process elapsed time; the 1-minute load before and after each attempt is recorded. |
| Secondary readings | Not verdicts: the goal summary's `food_growth_ms` per tick per world against T24.F01's 62.7 / 37.1 / 54.0 ms (unpaired, host load differs), and the gate's against 0.36 ms. |
| What moves and why | Both trajectories move from tick 0: spread targets come from the keyed stream instead of `sim.rng`, every later run-RNG draw (turn-queue shuffle, `reproduce_rng`) shifts, and the density total and telemetry sums round in row order. No rule changes, so the move is a different sample of the same distribution, not a direction. |
| Forecast bands | Against T24.F01's paired per-world readings. Persistence: plateau and final population per goal world and gate seed within −30% to +50%; `minimum_population` not read against a band; `extinction_tick` null in all three worlds. Food: `food_density_total` per type at the final sample within −30% to +30%. Miss rule: a reading outside its band is recorded in the Measured verdict and the readings and does not block; an extinction in a world that persisted at T24.F01 is a world-compatibility question for the user under the blocker rule. |
| Counters | The diff touches no creature code path, so every `per_creature_tick` move is a population-composition effect of the moved trajectory. Expected: `mesh_hops`, `vm_steps`, `graph_relax_iters`, `actions_applied`, `passes` within ±25% of T24.F01 on both profiles; `births`, `plasticity_updates`, `decided_passes`, `pass_cap_hits` may flag or go severe on either profile (small denominators, evolved-lineage share; `plasticity_updates` went severe at T24.F01, T19.F02, T11.F19, T17.F02); the gate's `decided_passes` (0 at T24.F01) goes severe under the zero-baseline rule if the moved gate run makes any decided pass. **Acceptance scope and epoch step:** a severe level on any counter, named above or not, is this feature's predeclared and justified cost only when all three hold: (a) the closing diff's production change is confined to the food kernel (`crates/v3-core/src/kernel/ordinary_food/`), the key plumbing (`simulation/simulation.rs`, `simulation/tick.rs`, `kernel/world.rs`, the `splitmix64` helper's home) and tests, pins, summaries and documents, which the reviewer checks against the diff; (b) the rule-equivalence evidence is green: the per-cell oracle with every mechanism enabled, the conservation proptest and the narrow-grid fixtures (Implementation Tasks), which pin the §5 rule under the keyed picks independently of the implementation, and the reviewer spot-checks that the gather's `spread_delta`, suppression and flag terms are the scatter's formulas (Spread term row); (c) the readings record, per severe counter, its attribution to the moved trajectory's composition (`creature_ticks`, `births`, the evolved-lineage share, the per-world persistence readings against the bands). With (a)–(c) that profile's `epoch_baseline` re-pins to this feature's summary in the closing commit, before the mutation gate. A severe level with (a) or (b) failing is unexplained and a blocker under the gate's P1 rule; (c) missing blocks closure until written. A profile with no severe level keeps its epoch and its summary is appended to `closed`. No user decision is needed on either branch. |
| Wall clock | Goal: `food_growth_ms` per tick is expected at or below half of T24.F01's per world on a comparable host, so wall per creature-tick falls (food was 55% of the goal tick); a fall is never a flag. A goal wall flag or doubling (wall severity is separate from the counter-only `severe` flag, `comparison.rs:565–598`) is attributed before it is classified: with `food_growth_ms` per tick per world at or below T24.F01's and `creature_ticks` per world accounting for the ratio, it is the moved trajectory's denominator or host load, recorded as this feature's justified cost, and an accepted doubling re-pins the `goal_worlds` epoch in the closing commit under the T10 rule even when every counter is non-severe; with `food_growth_ms` per tick above T24.F01's in any world, the goal profile is rerun once under `scripts/bench-wait` when the 1-minute load has been below 10 for 60 s, and a phase rise that persists is a blocker, not a predeclared cost. Gate (128 by 128, 16,384 cells): rayon dispatch of about ten row-parallel regions per tick is of the same order as the 0.36 ms pass, so `food_growth_ms` per tick is expected between 0.5× and 1.5× of T24.F01's; the gate's wall per creature-tick is expected within +25% (food is 14% of the gate tick). A gate wall flag is attributed before it is classified: with `food_growth_ms` per tick within 1.5× it is the trajectory's denominator or host load, recorded; above 1.5× the implementer sets a minimum rows-per-job (`with_min_len`, a constant, not a config) and the gate is rerun under `scripts/bench-wait`; a gate wall doubling that persists is a blocker, not a predeclared cost. |
| Indicators | No diversity or cognition measure is introduced; none remains `Undefined` because of this feature. Observation caps unchanged. |
| Overhead check | Not run: T24 is exempt from nothing; the gate and goal profiles measure this feature's cost directly, and the ceiling ratio compares two builds of the same commit that both carry the change. |

Run sequentially after benchmark-affecting work is final, without competing
builds, tests, servers or measurements (the `petri-telemetry` stack stays up):

```sh
make bench PROFILE=gate FEATURE=t24-f02-parallel-food-regrowth
make bench PROFILE=goal FEATURE=t24-f02-parallel-food-regrowth
```

**Measured verdict.** Pending.

- Summaries: [gate](../../progress/features/t24-f02-parallel-food-regrowth.json),
  [goal](../../progress/features/t24-f02-parallel-food-regrowth-goal.json).
- Full readings: [`docs/progress/readings/t24-f02.md`](../../progress/readings/t24-f02.md).

## Success Criteria

- [ ] `grow_food` runs its full-grid passes on the rayon pool in effect with
      the claim-and-gather form and the keyed stream; `sim.rng` makes no food
      draw; no rule, default, config field or digest changes (invariants 1,
      2, 6).
- [ ] Seeded runs are byte-identical across pools of 1 and 4 threads with
      every pass exercised, asserted by
      `crates/v3-core/tests/reproducibility.rs`, and across processes, the
      pinned digests reproducing (track criterion 4).
- [ ] The speedup pairs pass: median `food_growth_ms` per tick at 8 threads
      on the default 1600-by-1600 world at most 0.50 of the 3b904894 build's
      on this host (track criterion 3), recorded in the readings.
- [ ] The pass spans, the phase metric and the benchmark report fields are
      unchanged (T24.F03 tests and check 6 pass).
- [ ] Gate and goal profiles run, the predeclaration's acceptance scope is
      applied as written and any epoch re-pin is in the closing commit; the
      mutation gate runs with every survivor resolved.

## Notes for AI Agents

- Decision: the speedup reference is `main` at 3b904894 (T24.F03 and T24.F01
  closed), per the user's goal; the parent binary is built from a scratch
  worktree of that commit, never from an older parent.
