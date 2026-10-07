# T24 — Simulation Throughput

**Status**: In Progress
**Last updated**: 2026-10-06
**Master**: [Program Roadmap](../roadmap.md)

## Goal

The world runs by the same rules in less wall-clock time, so a default-size run
reaches a given tick sooner and every goal-profile closure costs less. Each
optimization targets a measured hot phase of the tick, keeps every world rule,
default and config field, and keeps seeded runs byte-identical across processes
and thread counts. Measurement comes first: food growth is timed apart from the
rest of world update before anything is changed. The one rule change is a
repair: food spread is then made to match its reference spec, so the parallel
rewrite builds on the documented rule rather than on a loop-order artifact.

## Track Success Criteria

- [ ] Food growth's time per tick is exported apart from creature aging, energy
  decay and death removal, its grid passes appear as child spans in sampled
  tick traces, and Petri / Performance shows both.
- [ ] Every food-spread contribution reaches its target whatever order cells are
  visited in, as `docs/reference/v3-world-grid-spec.md` §5 already specifies,
  and the repair's ecological effect is read on the gate and goal profiles on
  its own.
- [ ] Food regrowth on the default 1600-by-1600 world runs on the rayon pool,
  and at 8 threads its food-growth time per tick (T24.F03's phase) is at most
  half the parent build's on the same host.
- [ ] Seeded runs stay byte-identical across processes and thread counts with
  every parallel path the track adds exercised by
  `crates/v3-core/tests/reproducibility.rs`.
- [ ] Apart from T24.F01's repair, no T24 closure changes a world rule, default
  or config field. A trajectory move comes only from a changed random stream or
  summation order, is predeclared in the feature spec, and closes under the
  epoch re-pin rule.

## Executable Features

- [ ] **T24.F03 — Food Growth Phase Timing** — Depends on: None
  - Goal: The microscope sees inside world update: food growth is timed apart from creature aging, energy decay and death removal in the phase metric and the sampled tick traces, each of its grid passes is a child span on sampled ticks, and Petri / Performance shows them.
- [ ] **T24.F01 — Order-Free Food Spread** — Depends on: None
  - Goal: Seeds disperse evenly in every direction: food spreading from a dense cell keeps its share whichever neighbour it lands on, so a cell that already holds food no longer drops spread from its north and west neighbours, and food inside patches no longer drifts north-west.
- [ ] **T24.F02 — Parallel Food Regrowth** — Depends on: T24.F01, T24.F03
  - Goal: Food regrows by the repaired rules on every core at once: `World::grow_food` runs on the rayon pool with its own seeded per-cell stream, so a goal run spends at most half as long per tick on regrowth at 8 threads and still reproduces byte-for-byte at any thread count.

## Notes for AI Agents

- Origin, 2026-10-06: added at the user's direction after a question about
  MPS/GPU offloading. The only goal-length phase breakdown is the ignored local
  raw report `.bench-artifacts/t20-f09-inherited-discovery-qualification/goal.json`
  (rev 8829257d, 2026-09-27, Apple M1 Pro, 8 threads, goal profile: 1600 by
  1600, 10,000 founders, 2,000 ticks). Across its three seeds, world update
  takes 65–72% of tick time (41–68 ms per tick), cognition 12–14% (already
  parallel), sensor assembly 8–11% and actions 7–8%. No commit since that rev
  touches `crates/v3-core/src/kernel/`. Arithmetic on seed 11 (about 96 ms per
  tick, not a measurement): free cognition caps the speedup at 1.13×, free food
  regrowth at 3.4×, and regrowth about 6× faster on 8 cores gives about 2.4×.
  The gate profile (128 by 128, 75 ticks) spends 0.4 ms per tick in world
  update, so the speedup shows in the goal profile, not the gate.
- Order: T24.F03, then T24.F01, then T24.F02. T24.F03 was added on 2026-10-06
  out of ID order at the user's direction, as the track's first feature.
  T24.F01 does not depend on it; T24.F02 depends on both.
- T24.F03 origin and state, 2026-10-06. T21 already exports the five coarse
  phases two ways:
  - the cumulative metric `petri.run.phase_wall_clock` (Prometheus
    `petri_run_phase_wall_clock_seconds_total{petri_phase}`);
  - sampled `tick` traces with one child span per phase.

  The Petri / Performance dashboard
  (`telemetry/grafana/dashboards/petri-performance.json`) was added the same day
  outside the roadmap. It shows time per tick by phase, phase share, tick rate,
  cognition cost per creature-tick, run means, the slowest sampled phase spans,
  and a per-run comparison. Food growth cannot be separated from aging, energy
  decay and death removal inside `world_update`; T24.F03 fills that gap. A live
  `v3-server` run on 2026-10-01 spent 116 ms per tick in world update, against
  15 ms in cognition. When implementing T24.F03:
  - Panels that group by `petri_phase` pick up a new label without edits. The
    Phase spans table's name regex, the check script's span count (check 6:
    "tick and the five phases") and its `always_names` list must change.
  - A new `PhaseWallClock` field reaches `phase_wall_clock_ms_per_seed` in
    benchmark reports, so the spec checks the summary keep-list.
  - The observability exemption is keyed to T21 and T23 by track. T24.F03
    therefore runs the ordinary gate and goal profiles and the mutation gate,
    unless the user extends the exemption to it.
- T24.F01, the spread bug, was found while drafting this track and the user
  chose to fix it first (2026-10-06). The code is `grow` in
  `crates/v3-core/src/kernel/ordinary_food/ecology.rs`, unchanged since
  `ad1f97f0` (2026-03-21, multi-food refactor). The loop visits cells in
  row-major order and writes a separate next-density buffer. A cell holding
  food assigns its own value (`next_density[idx] = source + local_delta`),
  which overwrites spread already added by every neighbour with a lower
  row-major index (north and west, except across a wrapped edge). Reference
  spec §5 says local growth and spread both add.
  - Effect: about half of the spread landing on occupied cells is discarded,
    and what survives flows only north and west. Spread into empty cells is
    unaffected, so patches still expand evenly.
  - Fix: accumulate instead of assign. Within a tick the fix changes only what
    each cell keeps, not which draws the tick makes, so no stream change is
    mixed into the repair. Later ticks diverge because more cells cross the
    spread threshold.
  - TDD: a fixture where a north or west neighbour spreads into a cell holding
    food fails today and passes after the fix.
  - Predeclaration: gate and goal trajectories move, with more standing food
    inside dense patches; the spec predeclares the rest. The T01.F11
    persistence numbers and the grazing calibration (T02.F04) were measured
    with the bug.
- Why T24.F02 is a rewrite rather than a `par_iter` swap.
  1. `grow` draws from the run RNG (`sim.rng`, passed in `run_phase_0`) in
     row-major order: once per spreading cell, then once per recovery spawn.
     Moving food onto its own stream keyed by run seed, tick, food type and cell
     index makes the draws independent of scheduling. It also stops food
     shifting every later run-RNG draw, so both gate and goal trajectories move
     once (predeclared; epochs re-pin if the comparison flags severe). Philox
     (Salmon et al., SC11) is the standard counter-based design; a small keyed
     hash written in-tree is enough, so no new dependency is needed.
  2. Spread scatter-adds into one uniformly chosen passable neighbour. The
     parallel form gathers: each target recomputes its neighbours' picks from
     the keyed stream and sums all their deltas in a fixed neighbour order.
     After T24.F01 no ordering rule is needed.
  3. Telemetry sums are `f32`. They are reduced in a fixed order (for example
     per-row partials summed in row order) and keep their current meaning.
  4. Recovery spawns are few (cells × `recovery_spawn_rate`, about 2,560 at
     0.001) and may stay sequential on the food stream.
- `grow_food` makes about seven full-grid passes for two food types:
  depletion recovery, grazing recovery, inhibition sums, then growth and
  write-back for each type. The pass is not memory-bandwidth bound
  (single-digit GB/s on the measured run). Before T24.F03 closes, no profile
  inside world update exists. T24.F02's spec reads T24.F03's per-pass spans,
  or a profile from `crates/v3-core/src/bin/profile_ticks.rs`, before fixing
  the decomposition. It measures the speedup as interleaved pairs on a short
  default-size seeded workload against the parent build, under
  `scripts/bench-wait`.
- GPU offload was considered on 2026-10-06 and deferred. The GPU-shaped work,
  cognition, is 12–14% of a goal tick and already parallel. Every creature runs
  a different mutated graph, which diverges on SIMT hardware; FLAME GPU 2
  ([Richmond et al.](https://eprints.whiterose.ac.uk/199416/)) names agent
  heterogeneity as the core GPU difficulty. Regrowth would need T24.F02's keyed
  stream and gather form anyway. A GPU path adds further problems:
  - CI is `ubuntu-24.04` only, so the path has no coverage.
  - [WGSL §15.7](https://www.w3.org/TR/WGSL/) leaves floating-point results
    implementation-defined and permits fused multiply-add.
  - Metal enables fast math by default
    ([`fastMathEnabled`](https://developer.apple.com/documentation/metal/mtlcompileoptions/fastmathenabled.md)).
  - Thousands of tiny test simulations would pay device setup costs.
  Options weighed: wgpu (v29.0.1, March 2026), objc2-metal (metal-rs is
  deprecated), parallel CPU, or nothing. MPS offers fixed kernels (matrix
  multiply, convolution, image filters) and Petri has no such work. Revisit GPU
  only if world update still dominates after T24.F02.
- Candidates that are not yet rows: sensor assembly (sequential, 8–11%). Actions
  are sequential by design, because execution order decides conflicts.
- T24.F03 is measurement and T24.F02 and later optimization rows are
  engineering features. The natural-analog rule does not apply to either kind,
  because they add no mechanism, sensor, default or config field. T24.F01 is a
  correctness repair of an existing mechanism and names its analog. Every T24
  feature runs the ordinary gate and goal profiles, mutation gate and Codex
  review; no Benchmark gate exemption covers T24.
- Priority: not in the order of new starts until the user places it.
