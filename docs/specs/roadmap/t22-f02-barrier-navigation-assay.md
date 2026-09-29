# T22.F02 — Barrier-Navigation Assay

**Status**: In Progress
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
passable path plus the efficiency of the first reach, minus the
blocked-move fraction, graded against a composed
area-food + ring-inhibition comparator, the random-walk floor and the
barrier-blind founder. Every evaluation masks lifetime learning, so
improvement across generations is inherited change. The food-seeking
assay's calibration numbers for a seed are T22.F01's.

## Non-Goals

- No change outside `crates/v3-lab` and docs; a knob the lab needs and
  core lacks is a recorded finding for the owning track.
- No new sensor, no arm that enables lifetime learning (the mask applies
  to every arm), no per-elite brain readings (T22.F03) and no why-not
  ladder (T22.F04).
- No mixed-arena run and no seeded variation inside a layout file.

## Inputs and Invariants

**Contract.** The T22.F02 row, the track's "F02 reuses F01's harness" note
and the F01 contract note are the source of truth; F01's spec fixes the
harness and everything below is a delta on it. Diff scope (lab exemption):
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

**Arenas.** Every arena carries F01's lab invariants. A scene is
`{seed, start, food, barriers, redraws}`;
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

**Exposure predicate (barrier-navigation).** (a) F01's rule (food within
`vision_radius` of the start, none within distance 1);
(b) the geodesic distance from the start to a food cell is finite; (c) the
barrier-blind greedy walk — repeat F01's `step_toward` step toward F01's
target (the Chebyshev-nearest remaining food, first in draw order on ties)
— meets a barrier cell before it stands on food. The food-seeking assay
keeps predicate (a) only.

**Scoring.** Distances are geodesic: `dist(c)` is the length of the
shortest king-move path over non-barrier cells from `c` to the nearest
remaining food cell (a step is passable when its destination is, as
`apply_move` checks only the destination, so a diagonal passes between two
corner-adjacent barriers), computed as one multi-source BFS distance field per
interval (at the scene start and after every tick with a bite) and read per
tick. F01's `progress` definition is otherwise unchanged (`d0` at the
interval's open, `1 − min(d_t)/d0` clamped, 1 when `d0 = 0`), except that
under `barrier-navigation` an interval opened after a bite with no
reachable food left has value 1 (the task is complete), while one with no reachable food
from the scene start stays 0; `food-seeking` keeps F01's 0, so its rows
and calibration table are untouched. Tick-boundary observation is F01's:
`ticks_to_first_food` is the scene-relative 1-based tick of the first bite,
a death-tick bite included, and `progress` is the last living state's.
Per scene
`blocked_fraction = moves_blocked / moves_attempted` (0 when no move was
attempted), `efficiency = min(1, d_start / ticks_to_first_food)` with
`d_start` the first interval's `d0` (0 with no bite; the oracle scores
`d_start / (d_start + 1)`), and

`s = food_eaten + progress + efficiency_weight × efficiency − blocked_weight × blocked_fraction`

with both weights finite values ≥ 0 recorded in the summary: defaults 1.0
for `barrier-navigation` (wasting every move forfeits one bite; the
shortest path earns one), 0 for `food-seeking`, so F01's `s` is untouched. On a bounded block every
competent policy exhausts the food, so only efficiency and blocked moves
grade "without wasting moves" and separate the half-seeker from the
oracle. The selection scalar, threshold rule, validation candidate and
censoring are F01's.

**Instruments.** The scripted oracle step becomes path-aware: from `c` with
`dist(c) = d`, step to a neighbour with `dist = d − 1`, taking
`step_toward(c, target)` for F01's target when that neighbour qualifies and
otherwise the first qualifying direction in `Direction::ALL` order; when no
food remains or none is reachable (`dist(c)` infinite) the oracle takes the
random-walk step with its one stream draw, as F01 does for exhausted food.
On a barrier-free arena the `step_toward` neighbour always has
`dist = d − 1`, so F01's instrument scores are unchanged. The built-in
comparator
of `barrier-navigation` is `barrier-comparator`: `controller(&founder,
Family::Vector, false)` plus the Ring controller's structure appended by the
lab (reference `NeighborBarrierRing` at index 8; edges
`InputLeaf { ref_idx: 8, sub_idx: d }` → `Move(d)` at weight
`RING_INHIBITION` for d ∈ {0, 2, 4, 6}); `--comparator file` replaces it and
stays `instrument`. The comparator is a competence witness, not a ceiling:
its constants are never tuned on assay outcomes.

**Lifetime learning is masked.** The only lifetime-learning mechanism in
`v3-core` is the per-node `PlasticityConfig` on Graph compute nodes; the
founder carries none and `mutation/graph/hebbian.rs` can add it. Every
evaluation, in every arm and phase, runs a copy with every compute node's
`plasticity` set to `None` and asserts `plasticity_updates_total == 0`;
the stored, mutated, hashed and identity-compared genotype is unmasked. The
fidelity block records `lifetime_learning: "masked"`; no F02 arm lifts the
mask.

**Calibration gate.** F01's gate over the arena's axis × lifetimes:
`sparse-food-v1` keeps `calibration_fractions`; `wall-v1` and `ring-v1` use
`calibration_scales` (default {1, 2, 3}) and select the passing point with
the **largest** scale, then the shortest lifetime; a layout has one point
per lifetime. Exposure, competence (margin default 1.0, wins ≥
`ceil(0.75 × scenes)`), sensitivity (floor < half < oracle by ≥ 0.1, and the
comparator's mean `progress` above the floor's) and the validation rule are
unchanged; each point also records the floor's and comparator's mean
`blocked_fraction`. `uncalibrated` stops before any campaign (exit 2); on
a built-in arena it is a recorded finding for the track with the failing
check per point, never a reason to lower the margin. `--scale` restricts its axis as
`--food-fraction` does in F01; the gate still runs on the remaining
points. An axis flag the selected arena does not use is a config error.

**Outputs.** `row_version` stays 1: no row field changes (`efficiency`
is not a row field; `s` carries it). The summary is
`summary_version: 2`: `provenance.arena.spec` is a canonical descriptor
(`arena_version: 1`, id, size, start, food type, axis value, geometry and
sampling rules, or a layout's `arena_format`, path and rows) and
`provenance.arena.sha256` hashes it serialized with sorted keys, so two
arenas or scales never share a hash (tested); `sizes` gains `scale`, `blocked_weight` and
`efficiency_weight`; `fidelity` gains
`lifetime_learning`; each calibration point and `selected` carry
`food_fraction` or `scale` (the other null), the floor's and comparator's
blocked-fraction means and the floor, half, oracle and comparator
efficiency means; `v3-lab report` renders the axis, both weights and those
means, and refuses a v1 summary (backward compatibility is not a goal).
A 128-row layout adds ≤ 17 KB to the summary against the 1 MiB reserve.
The CLI commits nothing; a spec or note citing a run commits its
`summary.json` as `docs/progress/lab/<id>-<label>.json` (the `v3-lab
report` input); rows and elites stay ignored.

| CLI parameter | Default |
| --- | --- |
| `--assay` | `food-seeking` \| `barrier-navigation` |
| `--arena` | `sparse-food-v1` for food-seeking, `wall-v1` for barrier-navigation; `ring-v1` selectable |
| `--layout` | none; replaces `--arena` |
| `--scale`, `--calibration-scales` | selected by calibration; `1,2,3` |
| `--blocked-weight`, `--efficiency-weight` | 1.0 (barrier-navigation) / 0 (food-seeking) |

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
- [x] Calibration grid over the arena's axis, largest-scale selection,
      blocked-fraction means, the new CLI flags, summary v2 and report.
- [x] Tiny barrier-navigation fixture inside `cargo test -p v3-lab`
      (both arenas, a layout file, a layout whose food is unreachable
      reporting `exposure: false`, cross-thread byte identity).
- [x] Quick runs of both built-in arenas and the food-seeking regression,
      recorded in the readings file.
- [x] Scorer revision: the per-assay completed-interval rule, `efficiency`
      and `--efficiency-weight`, with fixtures for an exhausted block
      (progress 1 under barrier-navigation, 0 under food-seeking), the
      oracle's `d_start / (d_start + 1)`, a no-bite scene (0), a reach
      faster than `d_start` (capped at 1) and a death-tick first bite;
      the F01 regression (all pinned numbers) and both quick runs re-run
      and re-recorded on the revised scorer.

## Verification

- [x] `cargo test -p v3-lab` -> 109 unit + 17 integration tests green;
      `make check` exit 0 at 68ac188f.
- [x] Food-seeking regression: `make lab` (seed 1, `--quick`, release)
      reproduces F01's calibration table (founder 5.13 / 9.73 / 20.84,
      comparator 19.26 / 27.59 / 37.33–38.42, floor 2.67 / 3.60 / 6.31,
      selected 0.04 × 200, reach threshold 15.956), reconfirmed at
      78545c85; rows baseline sha256 `cd2a2c5e…45dcbf`, 948 rows,
      identical through the self-review commit; summary committed as
      `docs/progress/lab/t22-f02-food-seeking.json`.
- [x] `make lab LAB_ARGS="run --assay barrier-navigation --quick"` and the
      same with `--arena ring-v1`: `wall-v1` `calibrated`, scale 2 × 200,
      reach threshold 5.419, rows sha256 `c9a5c866…5a5e8e`; `ring-v1`
      `uncalibrated` on competence at
      every scale, exit 2 — accepted by the user decision below. Tables,
      reached fractions, timing and hashes:
      [`docs/progress/readings/t22-f02.md`](../../progress/readings/t22-f02.md);
      summaries committed as `docs/progress/lab/t22-f02-{wall,ring}.json`.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output
      path, and every survivor resolved as killed, equivalent, or deferred.
- [x] Benchmark summary: `Not applicable: lab feature`.

## Performance and Goal Impact

**Predeclaration — written before the run.** Not applicable: lab feature.
No trajectory, default, founder, recipe or mutation policy changes, so no
profile runs and no series entry. The natural-analog and
environmental-pressure rules do not apply. Lab cost, as complexity only:
one BFS over the arena (≤ 16,384 cells) per interval, at most one per
bite, and an O(1) read per tick.

**Measured verdict.** Not applicable: lab feature. The diff against
78545c85 is confined to `crates/v3-lab/**` and docs:
`git diff --stat 78545c85 -- . ':!crates/v3-lab' ':!docs'` is empty, so
`Cargo.toml`, `Cargo.lock` and the `Makefile` are unchanged.

- Full readings: [`docs/progress/readings/t22-f02.md`](../../progress/readings/t22-f02.md).

## Success Criteria

- [x] `v3-lab run --assay barrier-navigation` calibrates on `wall-v1`
      (scale 2 × 200 at seed 1), runs every F01 arm, and reports per
      replicate reach, generation to threshold (censored otherwise) and the
      comparator and floor blocked-fraction means; `ring-v1` stays
      available but uncalibrated on comparator competence (user decision
      below).
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
- Decision: (user, 2026-09-28) T22.F02 closes with `wall-v1` as the
  calibrated barrier arena. `ring-v1` stays available but `uncalibrated`:
  the composed area-food + ring-inhibition comparator wins 5/10/9 of 16
  scenes at scales 1/2/3 (12 needed), recorded as a track finding;
  comparator, margin, gaps and geometry untuned.
