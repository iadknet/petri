# T12.F04 — Baseline World Set

**Status**: In Progress
**Last updated**: 2026-09-09
**Feature**: T12.F04
**Track**: [T12 — World Composition and Baseline Worlds](../../roadmaps/t12-world-composition-and-baseline-worlds.md)

## Goal

Plains, orchards in grassland, canyon country, and confluence: creatures can
encounter foods trading payoff against abundance and return rate, and three
saved procedural worlds replace the goal baseline's three default-world seed
replicates with one run per environment. Plains remains a loadable control.
Stored readings identify each world and the evolutionary substrate measured.

## Non-Goals

- No default tuning, nutrient budget, digestion specialization, founder or
  mutation repair, new sensor behavior, or claim that specialization evolves.
- No seasons, disturbance, new grazing dynamics, manual overlays (T12.F05),
  checkpoints, new recipe format, recipe service, or benchmark runner.
- No per-type spread/max-density/recovery-floor overrides, erase layers,
  guaranteed-connectivity generator, dependency upgrades, or gate epoch re-pin.
- No change to the short gate profile, numerical thresholds or observation
  budgets; no mandatory fourth plains run or nine-run recipe sweep set.

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
retains those archetypes. The user corrected the additive reading protocol
during implementation: three environments replace the goal baseline's three
seed replicates, while the short gate stays unchanged. Extending
`FoodTypeConfig`, typed eating, and the existing per-type ecology loop is smaller than separate food engines or a
nutrition model. Optional fields with ordinary
[Serde defaults](https://serde.rs/field-attrs.html) preserve the shared fallback.
Extending `PatternParams` with thresholded existing
`kernel::fertility::generate_fbm` is smaller than a new noise dependency,
stored bitmap, or island/corridor framework. The locked
[noise 0.9.0 Fbm](https://docs.rs/noise/0.9.0/noise/struct.Fbm.html)
already provides seeded Perlin fBm and octave/frequency/lacunarity/persistence
controls. Extend the existing goal profile and report with three named recipe
cases and connectivity, rather than introducing another experiment runner.
The earlier research note's additive sweep recommendation is superseded by
the user's explicit goal-only replacement decision.
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
- Inspect layouts before measurement and adjust recipe parameters/dimensions
  only while preserving named pressures. Run the final goal profile once.
  A world that fails persistence remains explicitly failing; do not silently
  weaken its pressure, tune creature economics, or repeat the goal run to
  select a favorable reading. A concrete implementation failure follows the
  workflow's correction and re-verification rules.
- Standard `make bench PROFILE=goal` now runs exactly three fixed cases:
  Orchards in grassland with seed 11, Canyon country with seed 22, and
  Confluence with seed 33. Each uses its checked-in recipe once, the existing
  2,000-tick horizon, production founder/policy defaults, and full goal
  indicators. There are no per-environment seed replicates, mandatory extra
  sweep reports, or fourth plains run. Generic explicitly requested sweeps
  retain their existing arbitrary recipe/seed-list support.
- Extend existing profile/report handling to identify every case by stable
  name, recipe path, effective config digest and run seed. The config used for
  seeding and for each founder, drift, memory and evolved observation must be
  that case's config and food-type count. A single one-food battery must not
  silently stand in for two-food worlds. Report per-case results; reuse an
  observation only when all its relevant inputs match and its attribution is
  explicit. Preserve existing sample/trial counts and mutation floors.
  Reuse the existing neighborhood/drift result types under named case
  observations; singular top-level fields are explicitly `Undefined` with
  a per-case attribution reason for a multi-config goal, not a first-case
  value. Gate and single-config report behavior remain intact.
- Record extinction tick, peak/plateau/final population, births and final mean
  energy from existing persistence fields for each case. Survival means
  nonextinction through the full horizon, not proof of long-term viability.
  Store the complete reading in this feature's goal report. Plains' previous
  reports remain historical controls, not a current unmeasured fourth result.
- Start a labeled `goal-worlds-v1` series with this first world-set report as
  its initial reference. Preserve the existing `goal-v1` series, references,
  reports and acceptance results as history. A different recipe/case profile
  cannot compare as identical: old-versus-new deltas are unavailable, not
  zero or an asserted improvement. Subsequent matching world-set reports use
  the existing comparison thresholds. This is a user-authorized goal-profile
  definition reset, not a numerical-threshold waiver or a gate epoch re-pin.
  Existing progress/report consumers must retain access to historical series,
  distinguish the profile boundary and identify named case observations;
  a seed-mean curve must not imply that heterogeneous environments are
  interchangeable replicates. No dashboard redesign is required.
- Keep `make bench PROFILE=gate` and its ordinary tests unchanged: default
  world at 128², 256 founders, 75 ticks, 100% coverage, seeds 11/22/33,
  current founder observation, numerical thresholds and epoch reference.
  Gate remains the comparable compute control for this implementation.
- Add deterministic tick-zero connectivity readings for each measured case:
  total/passable cell counts, passable fraction, largest connected passable
  component cell count and fraction of passable cells. Derive from applied
  barriers; adjacency is the actual eight-direction Move topology with the
  world's Wrap/Bounded rule, ignoring temporary creature occupancy. No
  passable cells means component size and fraction zero. Compute once per
  seeded world, outside simulation tick work and existing timed measurements.
  Absent historical fields mean unmeasured, never guessed.
- Update the existing CLI/reference/progress documentation for automatic
  world-set selection by the standard goal command. All features already
  required to read the goal profile now read the three environments; there
  is no extra environmental-track-only sweep protocol. This user-selected
  early F04 execution precedes the planned T11 repairs. Keep roadmap priority
  order unchanged, label this substrate explicitly, and compare later
  closures against the preceding matching world-set reading. No extra paired
  main-at-start full profile is mandated by the retired additive protocol.

## Implementation Tasks

- [ ] Establish failing behavior/property tests, then implement food overrides
  and opt-in fertile placement with unchanged default behavior.
- [ ] Add thresholded fBm through existing pattern machinery; carry food and
  pattern fields through server/frontend and update affected canonical refs.
- [ ] Add truthful tick-zero report connectivity, compose/inspect the four
  recipes, and integrate them into the existing goal profile/report as three
  cases with one production-default run each.
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
  measured default gate deterministic trajectory remains unchanged. No extra
  old-goal run is required to prove unchanged defaults.
- [ ] fBm tests cover normalization, threshold direction/boundaries,
  zero/translated/clipped bounds, seed determinism, uniqueness and threshold
  monotonicity with fixed noise parameters. Extend the existing reproducibility
  fixture with fBm and differentiated foods across independent initialization
  and thread counts. Do not assume random property draws hit every variant.
- [ ] Connectivity examples cover all-passable, no-passable, separated islands,
  diagonal passage and Wrap versus Bounded edges; property-test count/fraction
  bounds and invariance to occupancy. Reports identify the effective recipe
  digest/path and applied tick-zero map. Recipe smoke tests load all four,
  check pressure/overlap conditions, and do not run the full goal profile in
  tests. A small production-path fixture proves exactly three distinct recipe
  configs execute once each with their own observation context; report tests
  reject cross-profile comparison and preserve historical series records.
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
  `docs/progress/features/t12-f04-baseline-world-set-goal.json` with all three
  named cases. Use `make bench` sequentially with no competing loads. Record
  report commits, exact commands, per-case persistence/connectivity/goal
  readings and applicable comparisons; update the measured series/progress
  table while preserving retired series history. Gate profile inputs,
  thresholds and epoch remain unchanged; all goal timing budgets apply to
  the complete three-case profile, never separately multiplied per case.
- [x] Second goal determinism run: Not applicable by the workflow's 2026-09-05
  decision; existing reproducibility and gate two-run tests remain required.
- [ ] Fresh independent final review, final `make check`, closure
  `make check-docs`, and exact tested/committed-content evidence.

### Implementation verification record (before measurements)

All commands ran in the feature worktree with the workflow PATH prefix. Logs
are retained under `/private/tmp/t12-f04-*.log` for this execution.

- Red: `cargo test -p v3-core --test baseline_worlds -- --nocapture` rejected
  the new food fields and FbmThreshold variant before implementation. The
  pre-feature short-run fingerprint was `13138541837675773035`; the regression
  retains it. An initial compile-only fixture correction removed access to a
  private RNG; the dedicated legacy-placement unit test compares RNG state
  against the original candidate/shuffle algorithm including zero coverage.
- `cargo test -p v3-core --test viability` ran first before implementation
  and first after food/ecology edits: both exit 0, 24 tests. Subsequent coherent
  `cargo check --workspace --all-targets` passed. `cargo clippy --workspace
  --all-targets -- -D warnings` passed after the assembly simplification.
- `cargo test -p v3-core --test baseline_worlds --test reproducibility`: 14
  ordinary behavior/property tests and 3 reproducibility tests passed. The two
  explicit diagnostics below are intentionally ignored by ordinary suites;
  both were separately executed successfully, not waived.
- `cargo test -p v3-core legacy_placement_keeps_candidate_order` passed.
  `cargo test -p v3-cli` passed all unit, CLI and benchmark tests. After the
  report-assembly extraction, `cargo test -p v3-cli goal_world_set_executes`
  passed the three-case production-path fixture again.
- `cargo test -p v3-server --test server
  recipe_export_preserves_complete_config_and_large_seeds` passed with food
  overrides, null inheritance, explicit zero and FbmThreshold in the recipe.
- `npm test -- src/stores/startupConfig.test.ts
  src/components/config-panel/startup/FoodTypeCard.test.tsx
  src/components/config-panel/startup/TerrainSection.test.tsx`: 23 passed.
  `npx tsc -b` passed. Rendered browser inspection exercised inheritance,
  reward override, fertile-only placement and the fBm editor. It caught and
  corrected unreadable float tails in the new shared-value labels. Inspection
  servers and the isolated browser were stopped afterward.
- `cargo test -p v3-core --test baseline_worlds
  food_choice_mutation_diagnostic -- --ignored --nocapture`: exit 0. Production
  founder/mutation/runtime defaults, Battery::generate(2), 80 executions/genome,
  executed parent nodes `[0, 1]`, 1,000 fresh births, seeds 9000..9999:
  **585 zero-event births, 0 offspring with corresponding Eat-index changes,
  0 offspring selecting non-primary food anywhere**. This is a bounded
  substrate reading, not evidence of specialization or a favorable-count gate.
- `cargo test -p v3-core --test baseline_worlds inspect_saved_world_layouts --
  --ignored --nocapture`: exit 0. Full-size generated maps inspected and saved
  as previews beside the recipes. The README records exact habitat overlap,
  whole-world food coverage, connectivity and preview legend. No recipe
  adjustment or persistence selection run was used.
- Self-review for reuse/simplification/efficiency: retained Option fallbacks,
  the original seeding shuffle, existing fBm helper and ecology equations;
  connectivity is a single barriers-only traversal outside measured tick work.
  Reused existing goal observation types per case and extracted cohesive
  observation assembly instead of adding a runner or suppressing the length
  lint. No speculative abstractions, dependencies or production tuning added.
- `make roadmap-check` passed on document edits. Fresh mutation evidence is
  recorded below; measured gate/goal reports, reviewer and closure evidence
  remain pending.

### Mutation remediation record

All passes use `MUTANTS_ITERATE=0 make rust-mutants`, the unfiltered affected
package suites and ordinary timeout caps. The initial sandbox could not inspect
host processes; the command did not start until the ordinary process/cache
permissions were granted. No test-selection, tool-configuration or production
change was made for mutation remediation.

- First fresh pass: `112 mutants tested in 11m: 10 missed, 74 caught,
  26 unviable, 2 timeouts`. Full output retained at
  `/private/tmp/t12-f04-mutants-first.out`; log
  `/private/tmp/t12-f04-mutants.log`.
- Second fresh pass: `112 mutants tested in 10m: 3 missed, 81 caught,
  26 unviable, 2 timeouts`. Full output retained at
  `/private/tmp/t12-f04-mutants-second.out`; log
  `/private/tmp/t12-f04-mutants-second.log`.
- Test-only remediation adds a births-rate property over complete profile
  totals (including zero ticks on every draw), exact strict-zero and coordinate
  clipping witnesses, and zero-initialized founder/drift timing accumulation.
  A first maximum-corner witness caught overflow but missed division because
  both expressions equal one there; the near-corner witness retains exactly
  six cells. A positive aggregate timer alone could miss multiplication of a
  tiny positive initial duration; the helper fixture starts from exact zero.
  `cargo test -p v3-core --test baseline_worlds`: 15 passed, two separately
  executed diagnostics ignored. Both new CLI properties/examples passed;
  `cargo clippy --workspace --all-targets -- -D warnings` passed. Self-review:
  these assertions exercise existing interfaces and invariants without a new
  abstraction, watchdog, test filter, performance threshold or production edit.
- Final fresh pass: `112 mutants tested in 10m: 84 caught, 26 unviable,
  2 timeouts`, exit 0; `run-mode.txt` is `fresh`. Full output:
  `/Users/istefanek/.local/share/petri-tools/mutants/t12-f04/mutants.out`;
  log `/private/tmp/t12-f04-mutants-final.log`. `missed.txt` is empty.
  No equivalent classifications or mutation exclusions were added.

Complete survivor history and final resolutions (locations are the unchanged
production locations reported by cargo-mutants):

| File and location | Mutation | Resolution |
| --- | --- | --- |
| `crates/v3-cli/src/bench.rs:1795:48` | `==` to `!=` in `assemble_goal_indicators` | Killed by births-rate property; final fresh caught list. |
| `crates/v3-cli/src/bench.rs:1798:56` | `*` to `+`; `*` to `/` in `assemble_goal_indicators` | Both killed by births-rate property; final fresh caught list. |
| `crates/v3-cli/src/bench.rs:1798:34` | `/` to `%`; `/` to `*` in `assemble_goal_indicators` | Both killed by births-rate property; final fresh caught list. |
| `crates/v3-cli/src/bench.rs:1738:17` | `+=` to `*=` in `prepare_goal_case` | Killed by zero-accumulator timing example; final fresh caught list. |
| `crates/v3-core/src/patterns/mod.rs:127:59` | `-` to `+`; `-` to `/` in `generate_pattern` | Both killed by clipped-bound examples; final fresh caught list. |
| `crates/v3-core/src/patterns/mod.rs:128:61` | `-` to `+`; `-` to `/` in `generate_pattern` | Both killed by clipped-bound examples; final fresh caught list. |
| `crates/v3-core/src/patterns/mod.rs:140:29` | `>` to `>=` in `generate_pattern` | Killed by exact-zero threshold example; final fresh caught list. |
| `crates/v3-core/src/kernel/world.rs:57:48` | `&&` to `||` in `WorldState::passable_connectivity` | Deferred P2: final fresh timeout, 3 s build + 120 s test; see Notes for AI Agents. |
| `crates/v3-core/src/kernel/world.rs:57:32` | delete `!` in `WorldState::passable_connectivity` | Deferred P2: final fresh timeout, 3 s build + 120 s test; see Notes for AI Agents. |

## Performance and Goal Impact

Natural analogs are food profitability and return rate, landscape barriers,
and patchy habitats. These reach creatures through consumed energy, applied
food growth/placement, passability and existing perception.
Predeclared cost: three per-type fallback resolutions, reward lookup when
eating, opt-in startup placement filtering, optional startup fBm work and one
linear connectivity observation per benchmark case. Default simulation draws
and trajectories must be unchanged, verified by focused tests and the
unchanged gate. No severe normalized compute regression or gate epoch re-pin
is justified in advance. Compare measured gate work and wall-clock per
creature-tick against the last measured closure and pinned epoch; distinguish
unmeasured intervening closures. The user-authorized goal definition reset
starts `goal-worlds-v1`, with unavailable old-profile comparisons. Record
per-case indicators and truthful observation timing, retaining existing
mutation floors, the 30-second drift observation cap, 10-second founder,
180-second aggregate evolved-neighborhood and 15-minute investigation limits
across the complete profile. No timing/sample threshold is increased to
accommodate the new environments. The world set is the standard goal baseline,
with per-case evidence rather than a composite score. Measurements are pending.

## Success Criteria

- [ ] Foods differ in applied payoff, initial habitat, regrowth and recovery
  through recipes alone while default behavior and inheritance remain intact.
- [ ] Four named worlds load in app/CLI, faithfully express their pressures,
  and the standard goal command runs exactly the three non-plains environments
  once each with attributable full readings, including failures.
- [ ] Required verification, mutation/review records and exact-content closure
  evidence pass, with T12.F04 Complete on clean main after authorized cleanup.

## Notes for AI Agents

- Deferred finding P2, mutation coverage: the two visited-guard mutations at
  `crates/v3-core/src/kernel/world.rs:57:48` (`&&` to `||`) and `:57:32`
  (delete `!`) repeatedly enqueue visited cells and do not terminate. The
  final fresh suite timed out at 120 seconds for each. Ordinary in-process
  assertions cannot finish with the guard missing; these are not equivalent
  or killed. Connectivity examples/properties and the unmutated full baseline
  pass. Advisor 6 accepted retaining this explicit coverage debt rather than
  adding subprocess/watchdog test machinery solely for mutation score.
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
- User interventions: explicitly selected F04 before its planned priority
  position; clarified that the standard baseline must be three total runs,
  one per environment, and approved replacing the goal profile while keeping
  the short gate unchanged. Roadmap priority order and prerequisite ownership
  remain unchanged.
- Requirement correction 2 (advisor consultation 3, 2026-09-09): the original
  request to replace testing seed replicates was recorded in research note
  Section 7.5, but its additive recommendation became the track/spec protocol.
  User clarification restores that intent: Orchards/11, Canyon/22,
  Confluence/33 replace goal default-world replicates, with a labeled new goal
  series. The prior nine recipe runs, fourth plains reading and paired-main
  fallback are removed. Gate inputs/epoch and all numerical thresholds remain.
  Dependent measurement/profile work paused while documents were serialized;
  independent food/terrain implementation continued. This corrects requirements
  and does not waive checks.
- Advisor consultation 1 accepted existing-component extension and identified
  default shuffle/draw preservation, unresolved Option inheritance, consistent
  growth/telemetry formulas and eight-direction connectivity as correctness
  constraints. No optional framework or adjacent refactor recommended.
- Advisor consultation 2 accepted a test-only correction after the roundtrip
  assertion twice compared JSON f64 0.01 with serialized f32
  0.009999999776482582: deserialize serialized FoodTypeConfig and compare typed
  f32 fields, preserving Some/None/zero assertions. No production rounding
  or serialization change is warranted.
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
  workloads ran during planning. Advisor consultations: 3 so far;
  usage unavailable.

- Advisor consultation 4: a repeated report-function length lint led to a
  cohesive private GoalIndicatorInputs/assembly helper; advice accepted and
  Clippy passed. The advisor clarified that unknown-field preservation refers
  to existing food/config validation, not enum-wide tightening of historical
  PatternParams. No new pattern validation wrapper or lint waiver was added.
  Implementer consultations so far: 4; usage unavailable.
- Advisor consultation 5 accepted measuring the goal once without substrate
  tuning. The T11.F18 depth-2,000 drift result of 10/2,000 (0.005000) had a
  feature-local exception; it does not waive F04's standing 0.008000 floor.
  Compare Canyon's relevant observation inputs and full drift rows with F18;
  two-food cases have a different battery/mutation context. Any measured
  failure remains an unmet criterion, with no terrain-causality claim,
  favorable rerun or automatic closure exception.
- Advisor consultation 6 accepted test-only remediation of finite mutation
  gaps and explicit deferral of nonterminating visited-guard mutants if they
  remain in the final fresh run. They are neither equivalent nor killed.
  In-process connectivity assertions cannot complete once traversal repeatedly
  revisits cells; adding subprocess/watchdog machinery solely for mutation
  score is not required. Preserve normal timeout caps, test selection and
  unmutated connectivity tests/properties. Consultations so far: 6; usage
  unavailable.
- Advisor consultation 7 accepted two further test-only witnesses after the
  coordinate-division survivors recurred: bounds at (65534, 65533) with a
  requested 4-by-5 region must retain exactly six coordinates, while division
  incorrectly retains one. Retain the broad property test. Test founder timing
  accumulation from exactly zero with reduced observation sizes and a positive
  elapsed duration assertion; do not assert exact time or change performance
  thresholds. Finish each active mutation pass and run fresh after remediation.
  Consultations so far: 7; usage unavailable.
- Advisor consultation 8 accepted hoisting the dashboard's existing relative
  link helper after self-review found it inaccessible to the new case cards.
  Load the new series' epoch reference through the existing deduplicating path,
  count reports rather than closures, and label an epoch outside `closed` as
  not yet closed. Keep `closed` empty until acceptance and retain historical
  tabs. Verify the rendered report; this HTML-only correction does not change
  the Rust mutation diff. Consultations so far: 8; usage unavailable.
