# T14.F10 — Spatial Occupancy by Clade

**Status**: In Progress
**Last updated**: 2026-09-12
**Feature**: T14.F10
**Track**: [T14 — Runtime Telemetry and Report Integrity](../../roadmaps/t14-runtime-telemetry-and-report-integrity.md)

## Goal

Every persistence checkpoint the benchmark already samples carries a coarse map
of where life is: for each cell of a fixed 16x16 grid laid over the world, the
number of living creatures standing in it and the number of distinct founder
clades among them. On Orchards, Canyon and Confluence, two clades living in
different parts of the same world read as two occupied regions rather than
dissolving into one world total.

## Non-Goals

- Any new mechanism, charge, rate, default, threshold or RNG draw; this feature
  reads positions the simulation already maintains.
- New checkpoints or a cadence change: `SAMPLE_EVERY_TICKS` and the
  sampled-tick rule are T14.F04's.
- New goal indicators, thresholds, comparison-block entries, indicator version
  or definition tokens, and the no-regression rule's coverage set. This is a
  checkpoint reading inside the stored `population_persistence` block, not an
  indicator.
- Per-creature positions, a full-resolution map, a per-cell clade breakdown, and
  any per-cell field beyond the two the track note caps the artifact at. A
  dominant-clade field is optional in that note and is declined here.
- Spatial statistics derived from the grid — partitioning indices, niche
  overlap, ecotype descriptors — which are T01.F04 and T01.F06; and the ancestry
  graph, which is T04.F01.
- The clade persistence timeline (T14.F09), the sensor census (T14.F08), and the
  progress-page presentation of the series (T14.F11).
- Rewriting historical reports. A report stored before this block existed reads
  it as absent, never as an empty grid.
- Any claim that creatures sharing a cell interact, or that clades in different
  cells were selected to separate. The grid records where creatures are.

## Inputs and Invariants

Sources of truth: `crates/v3-cli/src/bench.rs` (`PersistenceSample`,
`PopulationReadings::observe`, `PersistenceAccumulator`, `run_one_seed`,
`SAMPLE_EVERY_TICKS`, `build_config`),
`crates/v3-core/src/contracts/position.rs` (`Position { x: u16, y: u16 }`),
`crates/v3-core/src/creature/state.rs` (`CreatureState::position`),
`crates/v3-core/src/creature/identity.rs` (`lineage_id: u32`),
`crates/v3-core/src/kernel/world.rs` (`WorldState::width`, `::height`), and the
track's F10 note.

**A fixed 16x16 grid, independent of world size.** `OCCUPANCY_CELLS_PER_AXIS`
is `16`, a constant and not a config value, so a cell is a fraction of the world
rather than a fixed distance and the artifact is the same size in every world.
A position bins by proportion:

| Quantity | Value |
| --- | --- |
| `cell_x` | `(u32::from(position.x) * 16) / u32::from(width)` |
| `cell_y` | `(u32::from(position.y) * 16) / u32::from(height)` |
| flat index | `cell_y * 16 + cell_x`, row-major |

The multiply is in `u32`: `u16` overflows at `x >= 4096`, and the goal worlds
are 1600 wide. The form needs no divisibility precondition and never yields
`16` for an in-world position, because `x <= width - 1`. It coincides exactly
with cell edges every 100 positions on the 1600x1600 goal worlds and every 8 on
the 128x128 gate world, which is the "divisor of the world size" the track note
asks for, without making divisibility a precondition a sweep could violate. The
`cell_size = width / 16` form is rejected: on a width not divisible by 16 it
indexes past the last cell. A world narrower than 16 leaves some cells
unreachable; they report `0`, which is what an unoccupied cell reports anyway.

**Distinct clades, from an ordered structure.** Per checkpoint the aggregation
collects one `(flat_index, lineage_id)` pair per living creature into a `Vec`,
`sort_unstable`s it, and counts distinct `lineage_id` runs within each index
run. No `HashMap` or `HashSet` is iterated and no hash order reaches the report,
per T14.F02's constraint. The sorted-`Vec` form is chosen over 256 per-cell
`BTreeSet`s because it is one allocation per checkpoint rather than 256, and
because a sort is an explicit total order rather than a set whose determinism
rests on its element type.

**Two parallel row-major arrays, always full.** The block carries `cells_x`,
`cells_y`, and two vectors of length `cells_x * cells_y` in row-major order —
`population` and `distinct_clades` — rather than one row object per cell. The
arrays are always emitted in full, zeros included, so an unoccupied cell is a
`0` and not an absence, and the reader never reconstructs the grid shape from
which cells appear. The encoding costs about 1.5 KB per checkpoint against
roughly 14 KB for row objects, on a goal report already near 3.2 MB.

**Determinism is the binding constraint.** Seeded runs reproduce byte-for-byte
across processes and thread counts, and the gate profile's two-run
byte-identical check inside `make check` exercises these fields:

- Every value is an integer count. No float is accumulated and no division by
  population is taken.
- Every value is read after `run_tick` on an already-sampled tick, from state
  the tick produced. No production RNG is consumed and no execution path
  changes.
- The bin is integer arithmetic on the `u16` coordinates the tick already wrote.

**Empty population.** Both arrays are emitted at full length with every entry
`0` — a true zero grid, following `surviving_founder_clade_count` and T14.F08's
census rather than the `Option` means beside them.

**Placement.** The aggregation is a pure `v3-core` function from an iterator of
`(Position, lineage_id)` plus the world's width and height to the grid, so its
invariants carry property tests in the crate that hosts them. `bench.rs` calls
it from inside `PopulationReadings::observe`, which already fires only on
sampled ticks — at most 21 times per seed. `CreatureState`, the birth path, the
`v3-cli run` tick sample and the server payload are untouched.

**Serialization.** The grid is one structured block of its own type, carried as
`occupancy_grid` on `PersistenceSample`, `Option` and `#[serde(default)]`,
placed beside `sensor_census` and before the `#[serde(flatten)] tracking` field
— the same shape T14.F04's deferred note proposed and T14.F08 used. The
serialized type lives in `bench.rs` beside `SensorCensus`; the pure counting
lives in `v3-core`, as T14.F08 split them.

## Implementation Tasks

- [x] A pure `v3-core` binning and aggregation function: `(Position, u32)` pairs
      plus world width and height to a fixed 16x16 grid of population and
      distinct-clade counts, under the index, ordering and full-array rules
      above.
- [x] Carry the grid on `PersistenceSample` as a structured optional block,
      aggregated only on sampled ticks and only from post-tick state, under the
      empty-population and historical-report rules above.
- [x] Tests: the bin edges of a width divisible by 16 and of one that is not;
      the far corner of a world landing in the last cell; a clade counted once
      per cell however many of its creatures stand there and counted separately
      in each cell it occupies; two clades in one cell and two clades split
      across cells distinguished; the grid at every checkpoint of the real
      `run_one_seed` loop with `population` summing to the sample's own
      `population`; the all-zero grid at extinction; a stored report predating
      the block still loading; plus `v3-core` property tests for the pure
      invariants — every index below 256, the population sum equal to the
      creature count, each cell's clade count at most its population and at most
      the number of distinct clades present.

## Verification

- [ ] `make check` -> exit 0, run once on the final feature code; record the
      tested commit.
- [ ] Focused tests: `cargo test -p v3-core -p v3-cli`,
      `cargo clippy -p v3-core -p v3-cli --all-targets` and
      `cargo fmt --all -- --check`; test names in the readings file.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      and every survivor resolved as killed, equivalent, or deferred. The full
      survivor list stays here.
- [ ] Checkpoint samples in the stored goal report carry the grid at every
      checkpoint of all three world cases, with both arrays at full length and
      the per-checkpoint population sum equal to the sample's `population`. Key
      diff and the three worlds' occupied-cell counts in the readings file.
- [ ] Benchmark reports stored at
      `docs/progress/features/t14-f10-spatial-occupancy-by-clade.json` and its
      `-goal` companion.

## Performance and Goal Impact

**Predeclaration — written before the run.** Measurement feature; the track's
observation contract exempts it from the natural-analog rule and it adds no
mechanism. Both profiles compare against the epoch baselines the series index
names, under the existing thresholds.

Expected compute cost: negligible, and smaller than T14.F08's. The work is one
integer bin per living creature plus one sort of the resulting pairs, on the
same at most 21 sampled ticks per seed that T14.F04's population scan already
walks; no genome is read and no liveness analysis is repeated. The stored
reports grow by about 1.5 KB per checkpoint sample. A severe regression is a
blocker to report, not a cost to justify here, and no epoch re-pin is
authorized.

Predeclared direction for every goal indicator: **none**. This feature changes
nothing the simulation applies, so every indicator is expected to be unchanged
and any movement in one is a defect rather than a result.

Goal impact: habitat partitioning, one of the ways of coexisting the success
definition names, becomes readable at closure instead of staying invisible
behind world totals.

**Measured verdict.**

- Reports: [gate](../../progress/features/t14-f10-spatial-occupancy-by-clade.json),
  [goal](../../progress/features/t14-f10-spatial-occupancy-by-clade-goal.json).
- Full readings: [`docs/progress/readings/t14-f10.md`](../../progress/readings/t14-f10.md).

## Deviations

Authorized by the user for this feature only, because the Fable 5.1 budget is
exhausted; no model configuration reaches `main`.

- The orchestrator is Opus 5 at effort `high` in place of Fable 5.1 at effort
  `medium`; the contract's model check passes on that basis.
- `roadmap-reviewer` is spawned with the Agent tool's `model` parameter set to
  `opus`, which overrides the agent definition's `fable` frontmatter while its
  `effort: high` frontmatter stays in force, so the reviewer is Opus 5 at effort
  `high`. `.claude/agents/roadmap-reviewer.md` is not edited.
- The implementer's advisor is Opus 5, set by a worktree-local
  `.claude/settings.local.json` containing `{"advisorModel": "opus"}` — an
  ignored path; the tracked `.claude/settings.json` is not edited.

None of the three is a precedent for later features.

## Success Criteria

- [ ] Every checkpoint sample of a stored benchmark report carries the 16x16
      occupancy grid, on all three goal world cases, with population and
      distinct-clade counts per cell.
- [ ] Two clades occupying different parts of one world are distinguishable in
      the stored grid from two clades sharing the same cells.
- [ ] The readings are reproducible byte-for-byte across processes and thread
      counts, consume no production RNG and change no execution.
- [ ] Reports stored before this feature still load with the grid absent, and
      the existing checkpoint readings, the `v3-cli run` tick sample and the
      server payload are unchanged.

## Notes for AI Agents

- Decision: The grid is a fixed 16x16 of the world's extent, binned by
  proportion rather than by a fixed cell distance, so the artifact is the same
  size in every world and needs no divisibility precondition.
- Decision: Per cell the report carries population and distinct clade count and
  nothing else; the track note's optional dominant-clade field is declined, on
  the user's cap of population plus clade count per cell with no per-creature
  positions and no full-resolution map.
- Decision: The distinct-clade count comes from a sorted `Vec` of
  `(cell_index, lineage_id)` pairs, so no hash iteration order reaches the
  stored report.
- Exception: The Deviations section's three model substitutions were authorized
  on 2026-09-12 for T14.F10 only; they set no precedent and reach no file on
  `main`.
