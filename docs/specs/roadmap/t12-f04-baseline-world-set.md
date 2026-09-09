# T12.F04 — Baseline World Set

**Status**: In Progress
**Last updated**: 2026-09-09
**Feature**: T12.F04
**Track**: [T12 — World Composition and Baseline Worlds](../../roadmaps/t12-world-composition-and-baseline-worlds.md)

## Goal

Plains, orchards in grassland, canyon country, and confluence: creatures can
encounter foods trading payoff against abundance and return rate, and three
saved procedural worlds add resource differentiation, barriers, and overlapping
tick-zero pressures beside the default plains. Stored persistence readings
identify the actual world and the evolutionary substrate they measured.

## Non-Goals

- No default tuning, nutrient budget, digestion specialization, founder or
  mutation repair, new sensor behavior, or claim that specialization evolves.
- No seasons, disturbance, new grazing dynamics, manual overlays (T12.F05),
  checkpoints, new recipe format, recipe service, or benchmark runner.
- No per-type spread/max-density/recovery-floor overrides, erase layers,
  guaranteed-connectivity generator, dependency upgrades, or epoch re-pin.

## Inputs and Invariants

The owning track is the dependency and world-set contract.
[T12.F01](t12-f01-seeded-terrain-in-the-world-config.md) supplies seeded union
terrain before fertility, food, and founders; map seeds remain independent of
run-seeded placement. [T12.F02](t12-f02-world-recipe-save-and-load.md) supplies
partial-config recipes, exact app save/load, and sweep config path/digest.
Their feature-specific benchmark exceptions do not apply here.
Use the existing architecture and canonical world/startup/action/CLI contracts.

Research recheck, 2026-09-09: the
[world-seeding note](../../strategy/world-seeding-research-2026-09-08.md)
records the user-selected archetypes and their evidence; this feature does
not reopen that choice. Extending `FoodTypeConfig`, typed eating, and the
existing per-type ecology loop is smaller than separate food engines or a
nutrition model. Optional fields with ordinary
[Serde defaults](https://serde.rs/field-attrs.html) preserve the shared fallback.
Extending `PatternParams` with thresholded existing
`kernel::fertility::generate_fbm` is smaller than a new noise dependency,
stored bitmap, or island/corridor framework. The locked
[noise 0.9.0 Fbm](https://docs.rs/noise/0.9.0/noise/struct.Fbm.html)
already provides seeded Perlin fBm and octave/frequency/lacunarity/persistence
controls. Reuse the existing sweep report for persistence and add its missing
connectivity observation, rather than introducing a parallel experiment format.
Remaining empirical uncertainty is persistence and evolved barrier/food use;
unfavorable readings are valid evidence, not a reason to tune production.

Required food behavior:

- Add optional `energy_per_unit`, `growth_rate`, and `recovery_spawn_rate` to
  each `world.food.types[]` entry. Absent or null means inherit, respectively,
  `energy.costs.eat_reward_per_food`, `world.food.shared.growth_rate`, and
  `world.food.shared.recovery_spawn_rate`. Preserve null/absence as inheritance
  through normalization/save/load, so subsequent shared runtime edits affect
  inheriting types. Explicit zero is an override. Normalize finite reward to
  nonnegative and finite rates to `[0,1]`; nonfinite overrides revert to
  inheritance. Keep shared spread, max density, recovery floor, occupancy
  depletion, and all action costs unchanged.
- Typed Eat applies consumed density times that type's effective reward, then
  the existing energy cap and action cost. Invalid/empty types retain existing
  failure behavior. Resolve growth/recovery per type outside its cell loop;
  apply growth rate consistently to local growth, spread-derived growth and
  recovery density. Recovery rate controls existing spawn attempt count; it
  does not invent a new recovery clock. Applied telemetry follows those values.
- Existing coverage/density fields remain per type. Add
  `initial_fertility_only: bool` default false to opt a type into tick-zero
  patch placement: eligible cells are passable cells with that type's effective
  fertility at tick zero strictly greater than zero. With fertility disabled
  every passable cell is eligible. Use the existing annealing mapping when
  enabled. Shuffle eligible cells once with the existing run RNG and seed
  `round(initial_coverage * eligible_count)` cells, clamped to eligible count;
  zero eligible cells means zero seeded cells, with no fallback elsewhere.
  Preserve the old candidate order, shuffle and draws exactly when false.
  This is an initial placement rule, not a permanent prohibition on growth.
- This opt-in closes an inspected contract gap: `SingleType` layers currently
  target growth/recovery, while `ordinary_food::seed_density` ignores fertility.
  Without it, the requested orchard would start with fruit uniformly scattered
  across the world. No existing placement mechanism expresses fertile-only
  tick-zero placement. Coverage remains a fraction of eligible cells; reports
  and recipe descriptions distinguish it from achieved whole-world coverage.
- Carry fields through existing server startup/effective-config validation,
  frontend types, hydration and startup requests, and food cards with clear
  shared-value/override controls. Food type edits remain restart-only as today;
  inherited shared runtime updates remain live. Exact recipe transport and
  unknown-field rejection remain intact. Document units and inheritance.

Required terrain and world set:

- Add `FbmThreshold` to the existing `PatternParams` enum with `octaves`,
  `frequency`, `lacunarity`, `persistence`, and `threshold`. Generate local
  bounds-sized fBm through the existing helper using one u64 draw from the
  pattern's seeded RNG; barrier cells have raw value `> threshold`, translated
  to the bounds origin. The layer seed rule, clipping, union semantics and
  locked RNG/dependencies remain unchanged. Normalize octaves to `1..=32`,
  frequency to `[0.000001,1]`, lacunarity to `[1,4]`, persistence to `[0,1]`,
  threshold to `[-1,1]`; nonfinite values use defaults 0.02, 2, 0.5, 0 for
  the four float fields. Defaults use four octaves. All output is unique and
  in bounds, including zero-sized/edge bounds. Expose the variant through
  existing pattern types/defaults/controls and startup Terrain controls;
  retain the runtime pattern endpoint's existing area limit.
- Store recipes under `experiments/worlds/`: `plains.json` is `{}` (the
  current default recipe), plus `orchards-in-grassland.json`,
  `canyon-country.json`, and `confluence.json`. A concise README identifies
  each recipe, seed rule, pressures, dimensions, measured outcomes and reading
  commands. Use partial configs containing environmental differences only;
  founders, energy/mutation/runtime/population policies remain production
  defaults. Start at 1600². Keep map seed absent so seeds 11/22/33 sample maps;
  explicit layer seeds may separate otherwise aligned layers.
- Orchards: two foods with fruit strictly richer, lower coverage, slower
  growth and slower recovery than grass. Fruit uses positive-fertility patch
  placement and `SingleType` large blobs with effective minimum fertility zero
  (a positive minimum would make every passable cell eligible). Grass habitat partly overlaps fruit
  and extends beyond it. Verify nonempty overlap and exclusive habitat from
  generated grids, rather than assuming different seeds imply this.
- Canyon: whole-world Maze, corridor width near default vision radius 5
  (start at 5), substantial walls, and production food substrate. Record actual
  barrier fraction/connectivity. The recipe supplies frequent constrained
  navigation/occlusion opportunities; a persistence run alone cannot prove
  barrier awareness is necessary or evolved. Do not claim that causal result.
- Confluence: Bounded edges, thresholded fBm fragmentation with narrow
  connected routes, few large fertility blobs with minimum fertility zero,
  both differentiated foods, and spatially overlapping terrain/fertility/food
  regions. Use several regions smaller than the whole world but large enough
  to support populations. Inspect the generated layout and state which
  pressures overlap. Existing production dynamics stay at their defaults;
  no new treatment is enabled. Do not require all passable cells connected.
- Use recipe parameters and dimensions to obtain useful worlds. If an initial
  recipe fails persistence, diagnose its connectivity and allow a bounded
  recipe-only adjustment preserving its named pressure, then measure that
  final recipe. Store failed readings if used in the decision. The contract
  permits a world to remain explicitly failing; do not silently weaken its
  pressure or alter creature economics to force persistence.
- Each non-plains final recipe gets an existing sweep report under
  `docs/progress/sweeps/t12-f04/`, seeds 11/22/33, 2,000 ticks, production
  founder/policy defaults. Record extinction tick, peak/plateau/final
  population, births and final mean energy from existing persistence fields.
  Label persistence per seed as survival through the full horizon; it is not
  proof of long-term viability. Reuse this closure's single goal run for plains.
- Add deterministic tick-zero connectivity readings to measured report data
  for each seed: total/passable cell counts, passable fraction, largest
  connected passable component cell count and fraction of passable cells.
  Derive from applied barriers; adjacency is the actual eight-direction Move
  topology with the world's Wrap/Bounded rule, ignoring temporary creature
  occupancy. No passable cells means component size and fraction zero.
  Compute once per seeded world, outside simulation tick work and timed tick
  measurements. Absent historical fields mean unmeasured, never guessed.
- Document the live world-set reading protocol in existing roadmap/progress
  documentation: T02, T04.F03/F04, T06 and T12 closures read the three recipes
  beside the goal run; other features do not. This user-selected early F04
  execution precedes the planned T11 repairs. Keep priorities unchanged and
  label this substrate explicitly; later consumers use the track's paired
  main-at-start readings when intervening changes would confound attribution.

## Implementation Tasks

- [ ] Establish failing behavior/property tests, then implement food overrides
  and opt-in fertile placement with unchanged default behavior.
- [ ] Add thresholded fBm through existing pattern machinery; carry food and
  pattern fields through server/frontend and update affected canonical refs.
- [ ] Add truthful tick-zero report connectivity, compose/inspect the four
  recipes, and run/store the three production-default persistence sweeps.
- [ ] Review the diff for reuse/simplification/efficiency, triage fresh mutation
  survivors, and complete benchmark, review and closure records.

## Verification

- [ ] TDD red/green evidence for inherited/overridden/zero typed rewards and
  growth/recovery, energy cap/cost order, invalid type, normalization, config
  roundtrip and subsequent shared-value changes. Property-test normalization
  and effective-value invariants across all drawn cases.
- [ ] Placement examples/properties cover fertile-only exact eligible coverage,
  zero eligible, disabled fertility, annealing at tick zero, barriers,
  overlapping typed habitats and legacy false behavior. Preserve default
  short-run state/RNG identity against the pre-feature behavior and confirm
  measured default gate/goal deterministic trajectories remain unchanged.
- [ ] fBm tests cover normalization, threshold direction/boundaries,
  zero/translated/clipped bounds, seed determinism, uniqueness and threshold
  monotonicity with fixed noise parameters. Extend the existing reproducibility
  fixture with fBm and differentiated foods across independent initialization
  and thread counts. Do not assume random property draws hit every variant.
- [ ] Connectivity examples cover all-passable, no-passable, separated islands,
  diagonal passage and Wrap versus Bounded edges; property-test count/fraction
  bounds and invariance to occupancy. Reports identify the effective recipe
  digest/path and applied tick-zero map. Recipe smoke tests load all four,
  check pressure/overlap conditions, and do not run the full sweeps in tests.
- [ ] Server/frontend tests verify restart-only type edits, inherited shared
  runtime edits, new fields and pattern hydration/requests, optional controls
  and recipe roundtrip. Inspect the rendered edited controls and recipe maps.
- [ ] Record a bounded founder-neighborhood diagnostic of whether production
  mutations can change the selected Eat food type on a two-food fixture,
  using existing mutation/runtime APIs: `Battery::generate(2)`, the production
  founder and mutation/runtime defaults, 1,000 fresh offspring with seeds
  `9000 + birth_index`, and the battery's executed parent node set. Compare
  parent/offspring actions at corresponding battery executions; record the
  number of offspring with an Eat index change where both selected Eat, and
  separately offspring that select the non-primary food anywhere. Record
  zero-event births and corpus seed/size, including zero results honestly;
  no favorable-count gate, new permanent
  indicator, changed mutation rate or evolutionary repair is required.
- [ ] Run `cargo test -p v3-core --test viability` first after affected Rust
  edits, then `cargo check --workspace --all-targets` after coherent changes
  and focused tests. Use relevant Rust/React skills and retain property
  regression files. Run `make roadmap-check` on documentation changes.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants` after self-review/remediation:
  store summary/output path/full missed and timeout lists with each survivor
  killed by tests plus fresh rerun, equivalent with reason, or deferred.
- [ ] Store gate `docs/progress/features/t12-f04-baseline-world-set.json`, goal
  `docs/progress/features/t12-f04-baseline-world-set-goal.json`, and the three
  final recipe sweeps. Use `make bench` sequentially with no competing loads.
  Record report commits, exact commands, persistence/connectivity readings and
  threshold comparisons; update the existing measured series/progress table.
- [x] Second goal determinism run: Not applicable by the workflow's 2026-09-05
  decision; existing reproducibility and gate two-run tests remain required.
- [ ] Fresh independent final review, final `make check`, closure
  `make check-docs`, and exact tested/committed-content evidence.

## Performance and Goal Impact

Natural analogs are food profitability and return rate, landscape barriers,
and patchy habitats. These reach creatures through consumed energy, applied
food growth/placement, passability and existing perception.
Predeclared cost: three per-type fallback resolutions, reward lookup when
eating, opt-in startup placement filtering, optional startup fBm work and one
linear connectivity observation per benchmark seed. Default simulation draws
and deterministic trajectories must be unchanged. No severe normalized
compute regression or epoch re-pin is justified in advance. Compare measured
gate work and wall-clock per creature-tick against the last measured closure
and pinned epoch; distinguish unmeasured intervening closures. Record all
dated goal indicators and observation timing, retaining the 10-second founder,
180-second aggregate evolved-neighborhood and 15-minute investigation limits.
World-set sweeps are separate readings, not replacements for goal indicators
or inputs to a composite score. Measurements are pending.

## Success Criteria

- [ ] Foods differ in applied payoff, initial habitat, regrowth and recovery
  through recipes alone while default behavior and inheritance remain intact.
- [ ] Four named worlds load in app/CLI, faithfully express their pressures,
  and carry attributable persistence/connectivity readings, including failures.
- [ ] Required verification, mutation/review records and exact-content closure
  evidence pass, with T12.F04 Complete on clean main after authorized cleanup.

## Notes for AI Agents

- Starting main: `f7673fc29655b5213a4da14c88295ca5bf6d16a5`.
  Worktree: `/Users/istefanek/projects/petri/.worktrees/t12-f04`,
  branch `codex/t12-f04`. Track already In Progress and master Active;
  no planning rollup promotion is needed. Other track criteria remain open.
- Roles: Astra (`gpt-6-astra`) low orchestrator (session metadata verified by
  orchestrator); persistent high spec owner/advisor; persistent low
  implementer; fresh medium final reviewer. Record consultations, review
  findings, remediation passes and task usage at closure.
- Requirement correction 1, planning: add explicit opt-in fertile-only
  initial placement because inspected seeding ignores the existing fertility
  layers. The orchestrator accepted this smallest correction on 2026-09-09;
  it fulfills the roadmap's tick-zero orchard requirement and preserves
  existing behavior. No user intervention or verification waiver implied.
- User intervention: explicitly selected F04 before its planned priority
  position. Readings use the current substrate and the existing paired-main
  fallback; roadmap priorities and prerequisite ownership remain unchanged.
- Readiness self-review (2026-09-09): **Ready after one revision**. P1: 0;
  P2: 2 resolved: require zero minimum fertility for actual orchard patch
  eligibility, and pin the small food-choice diagnostic's corpus/denominators
  so it cannot become an open-ended evolutionary experiment. P3: 1 resolved,
  a malformed research paragraph line break. No unresolved questions.
  Limits: this is author self-review of requirements and inspected code;
  implementation behavior and measurements are not yet verified. Planning is
  not an advisor consultation or independent final review.
- Plan `make roadmap-check` passed before readiness revision (exit 0);
  post-revision validation is recorded at handoff. No code/build/benchmark
  workloads ran during planning. Advisor consultations: 0 so far;
  usage unavailable.
