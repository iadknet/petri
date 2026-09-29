# T22.F02 — Barrier-Navigation Assay

**Status**: Blocked
**Last updated**: 2026-09-28
**Feature**: T22.F02
**Track**: [T22 — Capability Assays and Evolvability Lab](../../roadmaps/t22-capability-assays-and-evolvability-lab.md)

## Goal

Measurement tooling. `v3-lab run --assay barrier-navigation` evolves
production founders with T22.F01's harness on a built-in arena whose food
sits behind a barrier wall (`wall-v1`) or inside a barrier ring with one gap
(`ring-v1`), or on any arena loaded from a JSON layout file, and reports per
replicate whether lineages come to reach that food without wasting moves:
the scene score is food eaten plus distance closed along the shortest
passable path, minus the blocked-move fraction, graded against a composed
area-food + ring-inhibition comparator, the random-walk floor and the
barrier-blind founder. Every evaluation runs an expression-masked copy of
the genome with lifetime learning off, so improvement across generations is
inherited change. The food-seeking assay's calibration numbers for a seed
are T22.F01's.

## Non-Goals

- No change to `v3-core`, `v3-cli`, `v3-server`, the frontend, shared build
  settings or any dependency; a knob the lab needs and core lacks is a
  recorded finding for the owning track.
- No new sensor, no arm that enables lifetime learning (the mask applies
  to every arm), no per-elite brain readings (T22.F03) and no why-not
  ladder (T22.F04).
- No mixed-arena run, no arena parameter beyond the scale grid, no seeded
  variation inside a layout file, and no tuning of the comparator's
  constants on assay outcomes.

## Inputs and Invariants

**Contract.** The T22.F02 row, the track's "F02 reuses F01's harness" note
and the F01 contract note are the source of truth; F01's spec fixes the
harness (evaluation, arms, selection, reach, RNG streams, outputs, byte cap)
and everything below is a delta on it. Diff scope (lab exemption):
`crates/v3-lab/**`, this spec, the readings file and the track checkbox.
`Cargo.toml`, `Cargo.lock` and the `Makefile` are unchanged: the crate is
registered, no dependency is added, and `make lab LAB_ARGS=...` already
passes any arguments through.

**Design evidence (2026-09-28).**

| Decision | Options | Choice and reason |
| --- | --- | --- |
| Progress behind barriers | Chebyshev distance (F01); geodesic (shortest passable path); the `pathfinding` crate | Geodesic, built in the lab (~40 lines of multi-source BFS): straight-line distance is deceptive around obstacles (the hard-maze result of Lehman and Stanley 2011, *Evolutionary Computation* 19(2)); a new crate is a lock change beyond registering the crate, which the lab exemption forbids; on a barrier-free torus the geodesic equals the Chebyshev distance, so F01's numbers are unchanged |
| Layout file | Cell lists; ASCII rows inside JSON; production `world.terrain` generators | ASCII rows inside JSON: authorable by hand in minutes, one `serde` struct with `deny_unknown_fields`, hashable like a genome file; `terrain` is a lab-owned key an overlay may not name |
| Comparator | The Vector controller alone; a lab-authored wall follower; Vector + Ring composed | Composed, as the track names both existing controllers; `controller()` asserts seven references, so the lab appends the Ring reference at index 8 with the same four `RING_INHIBITION` edges (`controllers.rs`) instead of calling it twice |

**Seams verified in code (2026-09-28).**

| Need | Seam |
| --- | --- |
| Barriers | `WorldState::{set_barrier, is_barrier}`; `apply_move` fails on a barrier cell (`is_valid_target_cell`) and charges the move; `Simulation::new` keeps a world's barriers (it only calls `reconfigure_food`), and `run_scripted` already takes a prepared world |
| Blocked moves | `SimStats::move_actions_attempted_total` counts every move, blocked ones included; `move_actions_blocked_total_by_cause` sums the causes (only `Barrier` can occur with one creature on a `Wrap` world); F01's `Tally` already carries both |
| Composition | `controller(&founder, Family::Vector, false)`, `Family::Ring.reference()`, `RING_INHIBITION`, `NodeGenome::input_refs`, `BackendDef::Graph`, `CgpGraphBackendDef::sink_mut`, `GraphEdge { source, weight }`, `GraphSource::InputLeaf { ref_idx, sub_idx }`, `OutputSinkKind::ActionVote(VoteSink::Move(d))` are all public |
| Founder is barrier-blind | No `Barrier` input in `creature/founder*` or `cgp_founder`; `cached_has_barrier_reader` is false for it |
| Sight through walls | `sensors::reducers` builds `AreaFoodSummary` over every visible cell with no occlusion, so food behind a wall within `vision_radius` (5) is sensed |
| Directions | `Direction::ALL` order N, NE, E, SE, S, SW, W, NW; `to_index` N 0, E 2, S 4, W 6; king moves |

**Arenas.** Every arena is a `SimulationConfig` with F01's lab invariants
(size, empty `terrain`, no growth or recovery, one creature, reproduction
suppressed, `Wrap`). A scene is `{seed, start, food, barriers, redraws}`;
food cells are type 0 at `max_density`; barrier cells are set with
`set_barrier` before the creature is placed; start, food and barrier cells
are disjoint. Built-in barrier scenes draw, from the `scenes` stream after
the `Simulation` seed, a cardinal direction `D` uniformly (N, E, S, W) and
place the food block `F`: the 3 × 3 cells centred at `start + 5·D` in
row-major world order (y, then x; layout food is ordered the same way), so
the nearest food is at Chebyshev distance 4 (within vision) and none within
distance 1. "Forward" is along `D`, "lateral" is perpendicular; every
coordinate wraps. `scale` is an integer in 1..=3 for both built-in arenas
(at 4 the ring would hold the start); any other value is refused.

| Arena | Axis | Layout |
| --- | --- | --- |
| `sparse-food-v1` | `food_fraction` | Unchanged from F01 |
| `wall-v1` | `scale` k ∈ {1, 2, 3} | Barriers: the 2k + 1 cells at forward 3, lateral −k..k. Exposure holds by construction for every k ≥ 1 and every target tie (the greedy walk's lateral offset is at most 1 when it reaches forward 3), so redraws are 0 |
| `ring-v1` | `scale` k ∈ {1, 2, 3} | Barriers: the cells at Chebyshev distance exactly k + 1 from `start + 5·D` (a square ring of side 2k + 3 enclosing `F`), minus one gap cell drawn uniformly (one stream draw) from the ring's non-corner cells enumerated in row-major world order; a draw whose gap lets the greedy walk through (predicate (c) below) redraws `D` and the gap together from the same stream, the `Simulation` seed unchanged, with F01's redraw limit and per-scene record; seeded fixtures pin one accepted and one rejected draw |
| `layout` (`--layout file.json`) | none (one point × lifetimes) | `{"arena_format": 1, "rows": [...]}`: square, 16–128 rows of equal length, characters `.` empty, `#` barrier, `F` food (at least one), `S` start (exactly one); the row count is the arena size and `--arena-size` given alongside is a config error; every scene is the file's cells with a fresh `Simulation` seed; usable with either assay, the assay's exposure predicate applies, and an infeasible layout is `exposure: false` for the point (no redraw exists) |

**Exposure predicate (barrier-navigation).** (a) F01's rule: at least one
food cell within `vision_radius` of the start and none within distance 1;
(b) the geodesic distance from the start to a food cell is finite; (c) the
barrier-blind greedy walk — repeat F01's `step_toward` step toward F01's
target (the Chebyshev-nearest remaining food, first in draw order on ties)
— meets a barrier cell before it stands on food. (c) is what makes the arena a navigation task rather than
a food-seeking one. The food-seeking assay keeps predicate (a) only.

**Scoring.** Distances are geodesic: `dist(c)` is the length of the
shortest king-move path over non-barrier cells from `c` to the nearest
remaining food cell (a step is passable when its destination is, as
`apply_move` checks only the destination, so a diagonal passes between two
corner-adjacent barriers), computed as one multi-source BFS distance field per
interval (at the scene start and after every tick with a bite) and read per
tick. F01's `progress` definition is otherwise unchanged (`d0` at the
interval's open, `1 − min(d_t)/d0` clamped, 1 when `d0 = 0`, 0 when no food
remains or the nearest remaining food is unreachable). Per scene
`blocked_fraction = moves_blocked / moves_attempted` (0 when no move was
attempted) and

`s = food_eaten + progress − blocked_weight × blocked_fraction`

with `blocked_weight` a finite value ≥ 0 recorded in the summary: default
1.0 for `barrier-navigation` (a lineage wasting every move forfeits one
bite), 0 for `food-seeking`, so F01's `s` is untouched. The selection
scalar, threshold rule, validation candidate and censoring are F01's.

**Instruments.** The scripted oracle step becomes path-aware: from `c` with
`dist(c) = d`, step to a neighbour with `dist = d − 1`, taking
`step_toward(c, target)` for F01's target when that neighbour qualifies and
otherwise the first qualifying direction in `Direction::ALL` order; when no
food remains or none is reachable (`dist(c)` infinite) the oracle takes the
random-walk step with its one stream draw, as F01 does for exhausted food.
On a barrier-free arena the `step_toward` neighbour always has
`dist = d − 1`, so the step is F01's and its instrument scores are
unchanged. The built-in comparator
of `barrier-navigation` is `barrier-comparator`: `controller(&founder,
Family::Vector, false)` plus the Ring controller's structure appended by the
lab (reference `NeighborBarrierRing` at index 8; edges
`InputLeaf { ref_idx: 8, sub_idx: d }` → `Move(d)` at weight
`RING_INHIBITION` for d ∈ {0, 2, 4, 6}); `--comparator file` replaces it and
stays `instrument`. The comparator is a competence witness, not a ceiling:
its constants are never tuned on assay outcomes.

**Lifetime learning is masked.** The only lifetime-learning mechanism in
`v3-core` is the per-node `PlasticityConfig` on Graph compute nodes
(learned per-edge weights in `CreatureState::plasticity_weights`); the
founder carries none and `mutation/graph/hebbian.rs` can add it. Every
evaluation, in every arm and phase, runs a copy of the genome with every
compute node's `plasticity` set to `None` and asserts
`plasticity_updates_total == 0` at the scene's end; the genotype that is
stored, mutated, hashed and compared for identity is the unmasked one. The
fidelity block records `lifetime_learning: "masked"`. A mutant whose only
change is plasticity is silent in the lab by design; no F02 arm lifts the
mask.

**Calibration gate.** F01's gate over the arena's axis × lifetimes:
`sparse-food-v1` keeps `calibration_fractions`; `wall-v1` and `ring-v1` use
`calibration_scales` (default {1, 2, 3}) and select the passing point with
the **largest** scale, then the shortest lifetime (the hardest calibrated
detour, as F01 selects the sparsest calibrated density); a layout has one
point per lifetime. Exposure, competence (margin default 1.0, wins ≥
`ceil(0.75 × scenes)`), sensitivity (floor < half < oracle by ≥ 0.1, and the
comparator's mean `progress` above the floor's) and the validation rule are
unchanged; each point also records the floor's and comparator's mean
`blocked_fraction`. `uncalibrated` stops before any campaign (exit 2) and,
for a built-in arena, is a recorded blocker naming the failing check per
point: the comparator's incompetence on an arena is a finding for the track,
never a reason to lower the margin. `--scale` restricts the scale axis to
that value and `--lifetime` the lifetime axis, as `--food-fraction` does in
F01; the gate still scores and validates the remaining points, and neither
flag bypasses it. An axis flag that the selected arena does not use is a
config error.

**Outputs.** `row_version` stays 1: no row field changes. The summary is
`summary_version: 2`: `provenance.arena.spec` is a canonical descriptor
with `arena_version: 1`, the arena id, size, start, food type, the axis
value and the geometry and sampling rules (for a built-in: food block,
barrier distance, gap rule; for a layout: `arena_format`, the path and the
rows), and `provenance.arena.sha256` is the SHA-256 of that descriptor
serialized with sorted keys, so two arenas or two scales never share a
hash (tested); `sizes` gains `scale` and `blocked_weight`; `fidelity` gains
`lifetime_learning`; each calibration point and `selected` carry
`food_fraction` or `scale` (the other null) and the two blocked-fraction
means; `v3-lab report` renders the axis, the blocked means and the blocked
weight, and refuses a v1 summary (backward compatibility is not a goal).
Projected size: a 128-row layout adds ≤ 17 KB to the
summary against the 1 MiB reserve; rows are unchanged in count and size.

| CLI parameter | Default |
| --- | --- |
| `--assay` | `food-seeking` \| `barrier-navigation` |
| `--arena` | `sparse-food-v1` for food-seeking, `wall-v1` for barrier-navigation; `ring-v1` selectable |
| `--layout` | none; replaces `--arena` |
| `--scale`, `--calibration-scales` | selected by calibration; `1,2,3` |
| `--blocked-weight` | 1.0 (barrier-navigation) / 0 (food-seeking) |

Every other parameter, stream, control, fidelity reading, byte-cap rule and
exit code is F01's. The BFS and the scene draws are pure functions of the
seeded streams, so F01's same-seed, any-thread byte identity of
`rows.ndjson` holds for both assays.

## Implementation Tasks

- [x] Geodesic `Progress` (multi-source BFS distance field) and the
      path-aware oracle step; property tests that on a barrier-free torus
      the field is the Chebyshev distance and the step is `step_toward`;
      exact-distance fixtures for a wall detour, a wrapped path and a
      diagonal corner passage with the oracle descending by one per step;
      a direct `run_scripted` fixture with only unreachable food showing
      the fallback direction and one random-walk draw per tick.
- [x] `Scene` gains `start` and `barriers`; both evaluators place barriers
      before the creature; `blocked_fraction` and `blocked_weight` enter
      the scene score.
- [x] Arena generators `wall-v1` and `ring-v1` with the three-part exposure
      predicate and redraw record; the `arena_format: 1` layout loader.
- [x] `barrier_comparator` composition with a structural fixture: nine
      references on the vote node (the founder's seven, `AreaFoodSummary`
      at index 7, `NeighborBarrierRing` at index 8), the four
      `RING_INHIBITION` edges into `Move(0, 2, 4, 6)`, and the genome
      evaluating on a barrier scene (competence itself is the gate's job).
- [x] The expression mask on every evaluation, with a fixture showing a
      plasticity-carrying genome evaluates with zero plasticity updates
      while its stored genotype keeps the plasticity.
- [x] Calibration grid over the arena's axis, the largest-scale selection
      rule, blocked-fraction means; `--assay barrier-navigation`, `--arena`,
      `--layout`, `--scale`, `--calibration-scales`, `--blocked-weight`;
      summary v2 and report.
- [x] Tiny barrier-navigation fixture inside `cargo test -p v3-lab`
      (both arenas, a layout file, a layout whose food is unreachable
      reporting `exposure: false`, cross-thread byte identity).
- [x] Quick runs of both built-in arenas and the food-seeking regression,
      recorded in the readings file.

## Verification

- [x] `cargo test -p v3-lab` -> all green; `make check` -> exit 0.
      At 68ac188f: 102 unit + 17 integration tests; `make check` exit 0.
- [x] Food-seeking regression: `make lab` (seed 1, `--quick`, release)
      reproduces F01's calibration table (founder 5.13 / 9.73 / 20.84,
      comparator 19.26 / 27.59 / 37.33–38.42, floor 2.67 / 3.60 / 6.31,
      selected 0.04 × 200, reach threshold 15.956), reconfirmed on the base
      commit before any change; the run's `rows.ndjson` sha256 and row
      count are recorded as the new baseline (the mask may move mutants
      that carry plasticity, so F01's hash is not a claim).
      Table reproduced exactly at 78545c85 and at 68ac188f; new baseline
      `cd2a2c5e…45dcbf`, 948 rows (readings file).
- [x] `make lab LAB_ARGS="run --assay barrier-navigation --quick"` and the
      same with `--arena ring-v1`: calibration verdict and selected point,
      per-point founder, floor, half, oracle and comparator means with the
      blocked-fraction means, reached fractions per native arm, wall time,
      creature-ticks, `rows.ndjson` sha256 and row count, recorded in
      [`docs/progress/readings/t22-f02.md`](../../progress/readings/t22-f02.md).
      An `uncalibrated` verdict on a built-in arena is recorded as a blocker
      with the failing check per point.
      Recorded: both arenas `uncalibrated` at every point (sensitivity
      everywhere; competence on the ring and wall scale 3): a blocker for
      the user (readings file).
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output
      path, and every survivor resolved as killed, equivalent, or deferred.
- [ ] Benchmark summary: `Not applicable: lab feature`.

## Performance and Goal Impact

**Predeclaration — written before the run.** Not applicable: lab feature.
The diff is confined to `crates/v3-lab`, this spec, the readings file and
the track checkbox; `Cargo.toml`, `Cargo.lock` and the `Makefile` are
untouched; no simulation trajectory, default, founder, recipe or mutation
policy changes, so no gate or goal profile runs and no series entry is
added. Measurement tooling: the natural-analog and environmental-pressure
rules do not apply. Lab cost, as complexity only: one BFS over the arena
(4,096 cells at 64², at most 16,384 for a 128-row layout) per interval,
opened at most once per bite, and an O(1) distance read per tick replace
F01's per-tick scan over the remaining food; the quick-run wall time is
measured and recorded, not predicted.

**Measured verdict.** Not applicable: lab feature; scope checked in review.

- Full readings: [`docs/progress/readings/t22-f02.md`](../../progress/readings/t22-f02.md).

## Success Criteria

- [ ] `v3-lab run --assay barrier-navigation` calibrates on `wall-v1` and
      on `ring-v1`, runs every F01 arm on each, and reports per replicate
      whether the reach threshold was met, in which generation (censored
      otherwise), and the blocked-fraction means of comparator and floor.
      An `uncalibrated` verdict on either built-in arena is a recorded
      blocker that leaves the feature incomplete until the user decides.
- [x] A JSON layout file runs as an arena without any Rust change, and an
      infeasible layout is reported as `exposure: false`.
- [x] Every evaluation is expression-masked and the food-seeking quick run
      for seed 1 reproduces F01's calibration table.
- [x] Nothing outside `crates/v3-lab` and the documentation changes.

## Notes for AI Agents

- Decision: lab exemption (user, 2026-09-28) — no gate or goal profile, no
  benchmark specialist; the mutation gate, the Codex review and `make check`
  apply.
- Decision: the track's "clone-only expression mask" is implemented in the
  lab as the evaluation-time plasticity mask above (no `v3-core` construct
  exists for it); F01's fresh-state rule and its `learned_weight_capture:
  false` reading stand unchanged.
