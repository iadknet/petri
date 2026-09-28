# T22.F01 — Lab Harness and Food-Seeking Assay

**Status**: In Progress
**Last updated**: 2026-09-28
**Feature**: T22.F01
**Track**: [T22 — Capability Assays and Evolvability Lab](../../roadmaps/t22-capability-assays-and-evolvability-lab.md)

## Goal

Measurement tooling. `make lab` (or `cargo run --release -p v3-lab -- run
--assay food-seeking`) evolves a small population of production founders on a
fixed sparse-food arena under the production mutation engine and lab
truncation selection, and writes, in seconds at quick sizes, one NDJSON row
per replicate per generation plus a compact summary that says, per replicate,
whether food seeking reached the calibrated threshold, in which generation
(censored when not), and how each of the five controls scored on the same
scenes. The NDJSON is byte-identical for a seed regardless of thread count.
Nothing in `crates/v3-lab` is reachable from a production crate.

## Non-Goals

- Barrier arenas, layout files and the ring comparator (T22.F02); per-elite
  brain and sensor readings and the mutant signature (T22.F03); the why-not
  ladder (T22.F04).
- Lexicase; cohort (non-solo) evaluation; lifetime
  learning attribution; a production default, founder, recipe or mutation
  policy; any change to `v3-core`, `v3-cli`, `v3-server` or the frontend.
- A benchmark profile or series entry, and any per-tick trace.

## Inputs and Invariants

**Contract.** The T22.F01 row and the track's "F01 contract" and
"Answered from source" notes are the source of truth; this spec fixes what
they leave open. The design, the options rejected and the
ring-probe precedent are in the
[capability assay research note](../../strategy/capability-assay-research-2026-09-28.md).

**Diff scope (lab exemption).** A knob the lab needs and core lacks is
recorded under "Notes for AI Agents" as a finding for the owning track, never
patched.

| Allowed change | Content |
| --- | --- |
| `crates/v3-lab/**` | Library, `v3-lab` binary, tests |
| `Cargo.toml`, `Cargo.lock` | The `members` entry; the `v3-lab` package entry alone. Dependencies are workspace entries already in the lock (`clap`, `serde`, `serde_json` with `float_roundtrip` as `v3-cli` enables it, `sha2`, `rand`, `rayon`, `slotmap`) plus dev-dependency `proptest`; no new external dependency, and existing versions, features and profiles unchanged |
| `Makefile` | `rust-test-lab` (added to `rust-test-all`) and `lab` |
| Documentation | This spec, the readings file, the track checkbox |

**Seams verified in code (2026-09-28).**

| Need | Seam |
| --- | --- |
| Build a world and place a creature | `kernel::WorldState::{new, set_food, set_barrier, place_creature, remove_creature}`, `creature::state::CreatureState::new`, `simulation::Simulation::new(world, creatures, tick, config, seed)` (calls `reconfigure_food`; food is placed after construction, as the ring probes do) |
| Step | `simulation::run_tick(&mut sim, &mut None)`; a dead creature is gone from `sim.creatures` after the tick that killed it |
| Founder as production seeds it | `creature::founder::founder_genome_with_age_gate(config.population.founder_profile, &config.energy.lifecycle)`; the phenotype triple is read from one `seed_simulation` founder at startup because `seeding.rs` keeps `FOUNDER_CHANNELS`, `FOUNDER_ACTIVE_CHANNEL` and `FOUNDER_POLARITY` private |
| Per-tick counters | `sim.stats.energy_flows.{food_intake_by_type, failed_action_penalty}`, `sim.stats.{eat_actions_applied_total_by_type, move_actions_attempted_total, move_actions_blocked_total_by_cause}` (cumulative, so the harness differences them), `creature.{energy, position, age}` |
| Variation | `mutation::MutationEngine::apply_mutations_with_food_type_count(&mut child, &config.mutation, &reachable, ParentExecuted::Record(&record, age), &mut rng, config.world.food.types.len())`; `creature::genome::analysis::mesh_reachable_nodes`; the engine resolves the record with `config.mutation.executed_window_ticks` (default 100); `MutationSummary` carries requested, applied and skipped counts by operator; `CreatureGenome: PartialEq + Serialize` with no hash-map fields, so equality and a SHA-256 over its `serde_json` bytes are exact |
| Reproduction suppressed | `energy.lifecycle.min_reproduce_energy = max_energy + 1.0` (test precedent `reproduction.rs:578`); the rejected vote is still charged the failed-action penalty, ramped to its endpoint at `startup.ramps.failed_action_penalty.target_tick` (default 62,680) |
| Authored comparator | `neighborhood::opportunity::controllers::controller(&founder, Family::Vector, false).0`, the area-food controller `v3-cli opportunity` builds on the same founder; it steers on `AreaFoodSummary` nearest-food offsets within `runtime.perception.vision_radius` (default 5) |
| Scripted instruments on production accounting | `simulation::actions::{apply_move, apply_typed_eat, apply_noop}` are public and charge the production costs, blocked-move rule and eat reward |
| Overlay validation | `SimulationConfig` is `serde` with `deny_unknown_fields`; `SimulationConfig::normalize` |

**Arena `sparse-food-v1`.** A `SimulationConfig` at production defaults with
these overrides only: `world.width = world.height = size` (48–64, default
64), `world.terrain = []`, `world.food.shared.growth_rate = 0`,
`world.food.shared.recovery_spawn_rate = 0`, every `world.food.types[i]`
growth and recovery override `None`, `population.initial_creatures = 1`,
`energy.lifecycle.min_reproduce_energy = max_energy + 1.0`. Edge mode, costs,
ramps, perception and every creature policy stay at production defaults. A
scene is one seeded placement: `round(food_fraction × size²)` cells of food
type 0 at `max_density`, drawn uniformly without replacement from cells at
toroidal Chebyshev distance ≥ 2 from the start cell (the arena centre). The
exposure predicate — at least one food cell within Chebyshev distance
`vision_radius` of the start and none within distance 1 — holds for every
scene by construction: a draw that fails it is redrawn from the same stream,
the redraw count is recorded per scene, and 100 consecutive failures make the
density infeasible for the calibration verdict.

**Evaluation.** Each individual is evaluated alone, per scene, from a fresh
`Simulation` started at world tick `target_tick`, with `start_energy`
(default 100.0, half of `max_energy`)
for `lifetime` ticks or until death. Observation is at tick boundaries:
after each `run_tick` the harness differences the cumulative counters (so
every action of the tick, including a bite in the death tick, is counted)
and, while the creature is still in `sim.creatures`, snapshots its position,
energy, age and dispatch record; when it is gone, `death_tick` = that tick
and the last living state is the snapshot after the previous tick (the
initial state for a death in the first tick; the death tick's own dispatches
are not observable). The action log is never read. Scene score `s = food_eaten + progress`; the selection
scalar is the mean of `s` over the generation's training scenes.

| Per-scene field | Definition |
| --- | --- |
| `food_eaten`, `intake` | Applied typed-Eat events; energy units taken |
| `ticks_to_first_food` | Tick of the first bite, null if none |
| `progress` ∈ [0, 1] | Defined on intervals delimited at tick boundaries only (a tick may hold several actions; positions inside it are unobserved). An interval opens at the scene start and at the boundary after any tick in which `food_eaten` increased, with `d0` = toroidal Chebyshev distance from the position at that boundary to the nearest remaining food cell (≥ 2 at the start by construction); `d_t` is that distance at each later observed boundary; the value is `1 − min(d_t)/d0` clamped to [0, 1], and 1 when `d0 = 0`. The field is the open interval's value at the last observed boundary (the scene's end, or the last living state on death), or 0 when no food remains |
| `moves_attempted`, `moves_blocked`, `penalty_charged` | Counter differences |
| `energy_end`, `death_tick` | Last living energy; tick of removal, null if alive |

**Instruments and controls.** Every run carries the arms below; the two
sensitivity instruments run in calibration only. Scripted actors die at
energy ≤ 0 like a genome and run on a lab stepper that builds the actor's `CreatureState` from
the founder genome and applies `apply_typed_eat` when food is under it, else
`apply_move`, then subtracts `energy_decay_per_tick`; they pay no brain
compute, carrying or penalty charge, so their `energy_end` is labelled
`scripted` and the selection scalar carries no energy term. The comparator is
a genome and runs the full production tick.

| Arm | Role / policy | Definition |
| --- | --- | --- |
| `native` | `reference` / `native` | Founder start, production `MutationConfig`, truncation selection |
| `founder-only` | `control` / `native` | The founder scored every generation on that generation's scenes; no mutation, no selection |
| `mutation-off` | `control` / `policy-deviation` | Overlay `mutation.per_unit_rate = 0` (kept by `normalize`); selection on; every child identical |
| `shuffled-score` | `control` / `native` | Native mutation; the scalars are permuted by the selection stream before truncation |
| `comparator` | `instrument` / `native` | `controller(&founder, Family::Vector, false)` scored every generation; never a start |
| `random-walk` | `instrument` / `native` | Scripted: eat if food here, else a uniform draw over the eight directions; the floor |
| `half-seeker` | `instrument` / `native` | Scripted: oracle step on even ticks, random-walk step on odd; calibration sensitivity only |
| `oracle-seeker` | `instrument` / `native` | Scripted: one step along the toroidal Chebyshev-shortest direction to the nearest remaining food; calibration sensitivity only |
| `--arm name=overlay.json[:genome.json]` | `user` / by resolved `mutation` | User overlay, optional genome file as the start |

An overlay is an RFC 7396 merge patch over the arena config's JSON, applied
in the order production defaults → arena → overlay, then deserialized (an
unknown key is an error), normalized, and only then given the lab
invariants: `min_reproduce_energy = resolved max_energy + 1.0`,
`initial_creatures = 1`, growth and recovery zero, size, terrain and
`edge_mode` (`Wrap`: distances are toroidal) as the arena; an overlay naming
one of those keys is refused, and a resolved config
is an error unless that threshold is finite and strictly greater than
`max_energy` in `f32` (the sum rounds to `max_energy` at 2²⁴) and
`start_energy ≤ max_energy`. Every arm carries two labels: `role` ∈ {`reference`, `control`,
`instrument`, `user`} and `policy` ∈ {`native`, `policy-deviation`}, the
latter whenever the resolved `mutation` block differs from the reference's;
`mutation-off` is `control` + `policy-deviation`. Both labels appear in
every row, summary entry and report line. The reach test runs for each
`reference`, `control` and `user` arm, never for an instrument; the reached
fraction and interval are native reachability only for `policy: native`
arms, and a `policy-deviation` arm's reach fields are printed under a
`diagnostic` label. A forced-event or ×k supply arm is such
a diagnostic.

**Calibration gate.** Before any campaign, on the grid
`calibration_fractions` × `calibration_lifetimes` (defaults {0.02, 0.04,
0.08} × {200, 400}), the founder, comparator and the three scripted
instruments are scored on `calibration_scenes` (default 16) from the
calibration stream. The selected point is the passing point with the lowest
fraction, then the shortest lifetime, that also passes competence on the
`validation_scenes` (default 8, `validation` stream); the founder's `s` is
recorded at every point wherever it falls. No passing point is
`uncalibrated`: the summary records the grid and the run exits 2 before any
campaign. `--calibrate-only` stops after the gate; `--food-fraction` and
`--lifetime` given explicitly skip the grid and gate that single point.

| Check | Pass rule |
| --- | --- |
| Exposure | Every scene met the exposure predicate within the redraw limit |
| Competence | Comparator mean `s` − floor mean `s` ≥ `calibration_margin` (default 1.0, one bite), and the comparator scores above the floor on ≥ `ceil(0.75 × scene_count)` scenes (same rule on the validation scenes) |
| Sensitivity | Floor < half-seeker < oracle on mean `s`, each gap ≥ 0.1, and the comparator's mean `progress` exceeds the floor's, so partial progress below one bite is graded |

**Selection and reach.** Truncation with elites: `survivors = max(1,
floor(elite_fraction × population))` (default 0.25; population ≥ 2, scenes,
validation scenes, generations and replicates ≥ 1, `elite_fraction` in
(0, 1]) individuals ranked by scalar, ties broken by the selection stream,
survive unchanged; the rest are mutants of parents drawn uniformly with
replacement from the survivors. Elites are re-scored every generation on the
new scenes. Reach threshold default: floor + 0.5 × (comparator − floor), both
means on the validation scenes at the selected point; `--reach-threshold`
overrides and is recorded. The validation candidate is the single top-ranked
individual of a generation by the true scalar in every arm (in
`shuffled-score` the permuted scalars drive truncation only); a replicate
reaches at the first generation whose
candidate has training scalar ≥ threshold and, evaluated only then on a
separate `Simulation` that never feeds reproduction, validation mean ≥
threshold. A replicate that completes the horizon unreached is `censored:
true` with `generation_to_threshold` null; one stopped by the byte cap or
never started is `incomplete: true` and neither reached nor censored. The
replicate (one seed lineage-population) is the sole independent unit: the
reached fraction and its Wilson 95% interval have the completed replicates
as denominator and are `null` when any replicate is incomplete, with
`incomplete_replicates` reported; individuals and identical offspring are
never interval samples.

**Variation fidelity.** A child is the parent's genome clone passed once
through the engine with the production `MutationConfig` of its arm,
`mesh_reachable_nodes(parent)`, and `ParentExecuted::Record` frozen as the
parent's last living dispatch record and age from its last training scene:
the snapshot after its final tick when it survives, the snapshot after the
tick before its death otherwise (empty for a death in the first tick, so
that birth derives nothing); a validation evaluation never replaces it. A
union across scenes is not computed in F01. Phenotype-channel mutation and learned-weight capture are not part of the
clone path (evaluation starts from fresh state; the phenotype triple stays the
founder's) and the fidelity block says so. Requested, applied and skipped
events by operator, exact genotype identity with the parent (`==`, distinct
from zero applied events) and elite carry-overs are reported separately;
neutral offspring are kept.

**RNG streams and determinism.** All seeds are assigned before parallel
evaluation; evaluations run in parallel over individuals and are collected
in index order; by-operator maps are written with sorted keys; NDJSON rows
carry no wall-clock or host field. Two runs with the same seed, in separate
processes and with any `--threads`, therefore produce byte-identical NDJSON.
The summary's `timing` block is the only non-deterministic content.

| Stream | Seed |
| --- | --- |
| Replicate `i` | `r_i = hash(seed, i)` |
| `scenes`, `mutation`, `selection` (per replicate, `SmallRng`, shared by every arm of the replicate; `observation` is reserved for T22.F03) | `hash(r_i, tag)` |
| Child engine RNG | `hash(mutation_seed, generation, child_index)` |
| Scene `Simulation` seed | One `scenes` draw |
| Scripted actor | `hash(r_i, "scripted", arm, scene)` |
| `calibration`, `validation` | `hash(seed, tag)` |

**Outputs.** Run directory `.bench-artifacts/lab/<assay>-<seed>-<utc>/`
under a root the caller injects: the CLI resolves it with `git rev-parse
--show-toplevel` and refuses to run without a checkout; tests pass a
temporary root. `--out` must resolve inside `<root>/.bench-artifacts/` or is
refused. It holds `rows.ndjson`,
`summary.json`, and `elites/<arm>-<replicate>.json` written at the end only.
A byte cap (`--byte-cap`, default 64 MiB per run directory) is checked before
every write against the directory's size plus the pending bytes. A summary
reserve of 1 MiB plus the overlay bytes is set aside at startup: rows and
elites may use only `cap − reserve`, a cap below twice the reserve is
refused, and a summary larger than the reserve is an error (exit 1) rather
than a growth. A row or elite write that would cross its budget is not made,
the run stops, and the summary records `incomplete: "byte_cap"`, exit 3.
Nothing is committed. Projected sizes: a row ≤ 8 KB at population 64, a
campaign of three evolving arms × 8 × 100 rows ≈ 20 MB.

| NDJSON row (`row_version: 1`) | Content |
| --- | --- |
| identity | `arm`, `role`, `policy`, `replicate`, `generation`, `scene_seeds` |
| population | `best`, `median`, `mean` scalar; best individual's per-scene vectors; `carried_over` count |
| fidelity | `requested`, `applied`, `skipped` totals and by operator; `identical_offspring_fraction` |
| individuals | `[id, parent_id, requested, applied, identical, carried, scalar]` per member |
| elite | `genome_size`, `reachable_nodes`, `executed_nodes` (record resolved at the production window) |
| reach | `validation_mean` when evaluated, else null |

| Summary keep-list (`kind: petri-lab-summary`, `summary_version: 1`) | Consumer |
| --- | --- |
| `provenance`: `git_revision`, `dirty` (both `null` with `git: "unavailable"` only when a library caller injects a root without a checkout, as tests do; a CLI run never records that), `config_digest` (SHA-256 of the resolved reference config JSON), `overlays` (name, content, order), `genomes` (name, SHA-256, `genome_format: 1`, `v3_core_version`), `arena` (spec and SHA-256), `seeds`, `threads`, `sizes` | report, research notes |
| `calibration`: grid points with founder, floor, half, oracle, comparator means, checks, redraws; `selected`, `verdict`, `reach_threshold` | report |
| `arms[]`: `name`, `role`, `policy`, `reach_reported` (true only when the arm's reach counts as native reachability: tested and `policy: native`; a tested `policy-deviation` arm is false), per replicate `reached`, `generation_to_threshold`, `censored`, `incomplete`, final best scalar; `reached_fraction` with `wilson_95` (null with `incomplete_replicates` when any replicate is incomplete) | report, research-note tables |
| `fidelity` (reference arm): `per_unit_rate`, `executed_bias`, `executed_window_ticks`, event totals by operator, `identical_offspring_fraction`, `phenotype_mutation: false`, `learned_weight_capture: false` | report |
| `incomplete` (null or reason), `exit_code`, `timing` (wall, creature-ticks, per-creature-tick ms) | report, readings |

`v3-lab report <summary.json>` renders the assay report from the summary
alone: calibration table, per-arm reached k/n with the interval, median
generation-to-threshold among reached, censored counts, fidelity.

| CLI parameter | Default (campaign / `--quick`) |
| --- | --- |
| `--assay` | `food-seeking` (only value in F01) |
| `--seed`, `--replicates` | 1; 8 / 4 |
| `--generations`, `--population`, `--elite-fraction` | 100 / 40; 64 / 16; 0.25 |
| `--scenes`, `--validation-scenes`, `--lifetime` | 4; 8; selected by calibration |
| `--arena-size`, `--food-fraction`, `--start-energy` | 64; selected by calibration; 100.0 |
| `--calibration-fractions`, `--calibration-lifetimes`, `--calibration-scenes`, `--calibration-margin` | 0.02,0.04,0.08; 200,400; 16; 1.0 |
| `--reach-threshold`, `--arm`, `--genome`, `--comparator` | calibrated; none; founder; built-in area-food controller (a file replaces it, still `instrument`) |
| `--threads`, `--byte-cap`, `--out`, `--calibrate-only` | available parallelism; 64 MiB; run directory; off |

## Implementation Tasks

- [x] Register `crates/v3-lab` (library + `v3-lab` binary) in the workspace
      with workspace dependencies only; add `rust-test-lab` to `rust-test-all`
      and a `lab` target (`LAB_ARGS`, default `run --assay food-seeking --quick`).
- [x] Arena, scene generator with the exposure predicate and redraw record,
      solo evaluator with per-tick counter differencing, dispatch-record
      snapshot and death tick.
- [x] Scripted stepper (random-walk, half-seeker, oracle-seeker) on the
      production appliers; comparator genome from `controllers::controller`.
- [x] Overlay merge, lab-invariant refusal, `policy-deviation` classification.
- [x] Variation (engine call, frozen record, identity and event bookkeeping),
      truncation selection with elites, shuffled-score control, reach test on
      validation scenes, Wilson interval.
- [x] Calibration gate with the grid-selection rule and the `uncalibrated`
      exit; `--calibrate-only`.
- [x] NDJSON and summary writers under the byte cap; provenance; `report`.
- [x] End-to-end pilot: record wall, creature-ticks and per-creature-tick
      cost of the measured quick-size runs in the readings, with the campaign
      projection derived from that cost labelled as a projection; fix the
      `--quick` sizes so a quick run finishes under 60 s on the development
      host; update the CLI table.

## Verification

- [x] `cargo test -p v3-lab` (in `make check` via `rust-test-lab`, tiny
      sizes, under 10 s), `cargo clippy -p v3-lab --all-targets -- -D
      warnings`, `cargo check --workspace --all-targets` -> clean; cases
      in the [readings](../../progress/readings/t22-f01.md).
- [x] `make check` -> exit 0; `git diff --stat main` confined to the diff
      scope above -> recorded in the readings.
- [x] Quick run: `make lab` -> wall under 60 s, calibration verdict, founder
      position, per-arm reached fractions, `sha256` of two same-seed
      `rows.ndjson` files equal -> readings.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      every survivor resolved as killed, equivalent, or deferred.
- [x] Benchmark summary: `Not applicable: lab feature` (diff scope above).

## Performance and Goal Impact

**Predeclaration — written before the run.** Not applicable: lab feature. The
diff is confined to `crates/v3-lab`, the workspace `members` entry, the
`v3-lab` package entry in `Cargo.lock`, the `rust-test-lab` and `lab`
Makefile targets, this spec, the readings file and the track checkbox; no
simulation trajectory, default, founder, recipe or mutation policy changes,
so no gate or goal profile runs and no series entry is added. Measurement
tooling: the natural-analog and environmental-pressure rules do not apply.
Lab wall time is a reading in the readings file, not a gate.

**Measured verdict.** Not applicable: lab feature; the reviewer checks the
scope claim against the diff.

- Full readings: [`docs/progress/readings/t22-f01.md`](../../progress/readings/t22-f01.md).

## Success Criteria

- [ ] `make lab` runs the food-seeking assay end to end on the development
      host in under 60 s at the fixed quick sizes and two same-seed runs
      produce byte-identical `rows.ndjson`.
- [ ] The calibration gate reports exposure, competence and sensitivity per
      grid point, selects a point by the predeclared rule or exits
      `uncalibrated`, and records the founder's position at every point.
- [ ] Every run carries the `native` reference arm with the fidelity block and
      the five controls; a user arm whose overlay changes `mutation` is
      labelled `policy-deviation` everywhere it is reported.
- [ ] The summary reports, per replicate, reached, generation to threshold
      with censoring and `stopped_by`, and a Wilson interval on the reached
      fraction; `v3-lab report` renders the assay report from the summary
      alone.
- [ ] Output stays under the byte cap, is never committed, and contains no
      per-tick trace; `make check` passes with the diff confined to the scope
      above.

## Notes for AI Agents

- Decision: lab exemption (user, 2026-09-28) — no gate or goal profile, no
  benchmark specialist; the mutation gate, the Codex review and `make check`
  apply.
