# T12.F01 — Seeded Terrain in the World Config

**Status**: In Progress
**Last updated**: 2026-09-08
**Feature**: T12.F01
**Track**: [T12 — World Composition and Baseline Worlds](../../roadmaps/t12-world-composition-and-baseline-worlds.md)

## Goal

Bedrock and water: config and seeds generate barrier terrain before food and
founders, so passability and existing barrier occlusion apply from tick zero.
The same inputs reproduce the world across processes and thread counts;
production defaults remain barrier-free with unchanged trajectories.

## Non-Goals

- No recipe save/load or CLI partial-config merge (T12.F02), differentiated
  food (T12.F03), baseline world set or `FbmThreshold` generator (T12.F04).
- No bitmap, erase layers, runtime terrain regeneration, new perception
  behavior, founder policy, terrain editor framework, or checkpoint format.
- No production tuning, epoch re-pin, RNG redesign, or unrelated pattern
  generator cleanup.

## Inputs and Invariants

The owning track defines scope and dependencies (none). Evidence is the
[world-seeding research note](../../strategy/world-seeding-research-2026-09-08.md),
[T10.F11 reproducibility finding](t10-f11-cross-process-reproducibility-of-seeded-runs.md),
and the existing world, startup, server API, and sensor reference contracts.
The implementation extends `config/simulation.rs`, `simulation/seeding.rs`,
`kernel/fertility/mod.rs`, `patterns/`, the server's existing startup/config
handlers, and the current startup preset and panel. No second world schema.

Research recheck, 2026-09-08: extending the existing procedural config and five
`PatternParams` variants is smaller than storing a bitmap or adding a world
format; the latter options add serialization without helping parameterized
terrain. The existing generators and fertility machinery satisfy this feature.
For RNG identity, retain `SmallRng` with version evidence as the track's
fallback versus naming `rand_xoshiro` 0.6 `Xoshiro256PlusPlus` conditionally;
ChaCha changes every default draw and is excluded. Primary local sources are
the locked `rand` 0.8.6 `src/rngs/small.rs` and `rand_core` 0.6.4
`src/lib.rs::SeedableRng::seed_from_u64`. They reveal that the wrapper inherits
PCG32 seed expansion, unlike the underlying Xoshiro's SplitMix64 override.
Thus the research note's expected same-`u64` stream identity is unproved and
likely false. The implementer's executed comparison settles it. External
documentation fetches for the exact versions failed during this recheck;
the note's [named-generator source](https://docs.rs/rand_xoshiro/0.6.0/src/rand_xoshiro/xoshiro256plusplus.rs.html)
is a reference to verify, not evidence that the test passed.

Required config and behavior:

- `world.terrain: Vec<TerrainLayer>` defaults to `[]`, including when absent
  during deserialization. Each layer is `{ params: PatternParams, bounds?:
  PatternBounds, seed?: u64 }`; use the existing tagged parameter variants
  without a parallel pattern enum. Optional fields accept absence or null.
  Reject unknown layer fields consistently with config schema discipline.
- `world.world_seed: Option<u64>` defaults to absent/null. The effective map
  seed is `world_seed.unwrap_or(run_seed)`. Terrain layer `i` uses its explicit
  seed or `effective_map_seed.wrapping_add(i as u64)`. Fertility receives the
  effective map seed and preserves its existing per-layer override/index rule.
  Terrain and fertility derive independently; terrain consumes no run-RNG
  draws. Food, founder placement, and subsequent runtime draws retain the run
  seed and existing ordering. Fixing the map seed fixes barriers and fertility,
  not food positions, founder positions, or the runtime trajectory.
- Apply layers in list order after `WorldState::new`, before fertility, food,
  or founder placement, using `WorldState::set_barrier(pos, true)`. Layers
  combine by union; they never clear barriers. Repeated cells are idempotent.
  Existing food coverage and founder clamping operate over passable cells.
- Absent bounds mean the entire world. Explicit bounds are intersected with
  the world without wrapping: clip width/height using saturating subtraction
  from the origin; zero-sized or wholly outside rectangles are empty. Compute
  endpoints in a widened type or use safe subtraction before narrowing. This
  new startup rule does not change the runtime pattern endpoint's clipping.
  Preserve each existing generator's parameter normalization. Startup must
  support the production 1600² world; the HTTP pattern endpoint's 1,000,000-cell
  cap stays confined to that endpoint.
- Sort clustered Noise's candidate cells before its seeded shuffle/truncation.
  Other generators need no output-order rewrite where barrier application
  observes only the set. Tests must exercise the excess-cell trimming branch.
- Before replacing any RNG, compare the locked `SmallRng` and named generator
  streams at the same `u64` seeds on the current 64-bit platform. If identical,
  use the named generator for terrain and fertility; a run RNG rename is
  optional only with identical draws and no broader refactor. If different,
  keep `SmallRng`, record the actual `rand` version in new benchmark environment
  metadata and this spec, and retain the evidence explaining the fallback.
  Historical reports remain readable with this metadata absent/unmeasured;
  never backfill a guessed version or rewrite a stored baseline.
  Do not invent seed adapters to defeat the condition or change defaults.
  Under fallback, reproducibility is for the locked dependencies and platform;
  do not claim cross-platform/version identity. `noise` stays pinned by the
  lockfile; upgrading it requires re-recording affected future baselines.
- Startup accepts the fields through the existing serde/deep-merge schema;
  GET config returns the effective fields and tick-zero projections contain
  the applied barriers. Runtime config PATCH rejects both new keys as
  restart-only, including null/empty values, atomically without changing state.
- Add a Terrain section beside World Topology in the startup panel, reusing
  existing field and pattern types/defaults. Users can add/remove layers,
  choose any of the five patterns, edit that pattern's parameters, choose
  whole-world or explicit bounds, and set/clear layer and world seeds. Preserve
  list order through edits, hydration, and startup request construction. Label
  map seed versus run seed by their actual effects. Numeric seed entry must
  not silently round beyond JavaScript's safe integer range; reuse the current
  numeric seed convention with safe input limits, while Rust/JSON retain u64.
- Update canonical world config Section 4 and startup seeding Section 4,
  their directly affected summaries, and the server API startup and runtime
  patch contracts in the same feature. No unrelated reference repairs.

## Implementation Tasks

- [ ] Establish failing tests for terrain config, startup ordering/seed
  isolation, clustered Noise trimming, and runtime patch rejection; execute
  and record the RNG compatibility probe before choosing its branch.
- [ ] Add the core schema/seeding behavior and bounded reproducibility fix;
  implement the selected RNG branch and its required report evidence.
- [ ] Carry the fields through server startup/projections, frontend types,
  preset hydration/request construction, and the Terrain controls; update
  canonical references and integration tests.
- [ ] Review the diff for reuse, simplification, and efficiency; resolve fresh
  mutation survivors through tests; store gate/goal evidence and update the
  existing progress series. Complete closure records after required checks.

## Verification

- [ ] Record TDD red and green commands. Core examples cover absent/default
  fields, serialization, all five patterns, whole-world and explicit bounds,
  overlapping layers, empty/all-barrier maps, no food/founder on barriers,
  exact eligible-cell food coverage, and founder count clamping. Include a
  1600² startup terrain case proving the runtime endpoint cap is not reused.
- [ ] Property tests establish bounded/idempotent barrier union and deterministic
  generation for every drawn case, explicit seed precedence, wrapping seed
  addition, and clustered Noise exact target/set reproducibility. Explicitly
  exercise every pattern and the trimming path, rather than assuming a random
  draw reaches them. Keep any `proptest-regressions/` files.
- [ ] Extend `crates/v3-core/tests/reproducibility.rs` with a terrain-bearing
  fixture that includes clustered Noise trimming. Compare barriers, fertility,
  food, and founders at tick zero and applied short-run state under independently
  initialized simulations and different Rayon thread counts. Preserve the
  existing mutation stress fixture and its coverage. Fresh randomized hash
  states exercise the process-order failure as in T10.F11; do not add a second
  goal run or subprocess harness merely to duplicate it.
- [ ] Fixed map seed plus distinct run seeds yields equal barrier/fertility
  grids and distinct run-seeded placement on an explicit nondegenerate fixture;
  absent map seed preserves the prior seeding behavior. Compare the default
  gate/goal deterministic behavior to T11.F18, allowing only config identity
  metadata caused by the new default fields.
- [ ] Server tests cover partial startup terrain overrides, unknown fields,
  restart-only PATCH rejection with state/config unchanged, effective config,
  and applied tick-zero barrier projection. Frontend tests cover layer edits,
  hydration, bounds and optional seeds, pattern switches, safe integer input,
  and the actual startup request; inspect the rendered Terrain section.
- [ ] Run `cargo test -p v3-core --test viability` first after seeding/RNG changes,
  then `cargo check --workspace --all-targets` after coherent Rust edits and
  focused core/server/frontend checks. Load relevant Rust and React skills.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants` after self-review: record
  summary, output path, and complete missed/timeout list, with each killed by
  tests and a fresh rerun, equivalent with reason, or explicitly deferred.
- [ ] `make bench PROFILE=gate FEATURE=t12-f01-seeded-terrain-in-the-world-config`
  stores `docs/progress/features/t12-f01-seeded-terrain-in-the-world-config.json`;
  run the corresponding `PROFILE=goal` once for `...-goal.json`. Fill the
  performance section and append the existing benchmark series/progress row.
- [x] Second goal determinism run: Not applicable by the workflow's 2026-09-05
  decision. Existing reproducibility and gate two-run checks remain mandatory.
- [ ] `make roadmap-check` on document edits and at handoff; `make check` on
  final feature code; `make check-docs` on closure documents. Record command
  evidence and the tested commit before integration.

## Performance and Goal Impact

Natural analog: bedrock and water constrain movement, food occupancy, founder
placement, and existing line-of-sight opacity through the world. No new sensor
or diversity/cognition indicator is introduced.

Predeclared cost: opt-in terrain generation and clustered candidate sorting at
startup only. Empty terrain and absent world seed preserve all production
draws and trajectories. No severe compute allowance, new epoch, baseline edit,
threshold reduction, or extra goal-profile run is authorized.

Compare all six normalized gate counters and wall time per creature-tick to
previous closure T11.F18 and pinned gate epoch `remove-complementary-nutrition`;
compare goal evidence to T11.F18 and pinned goal epoch T11.F17. Preserve the
existing +10%/+50% work and +25%/+100% matching-host wall flag/severe thresholds.
Record host mismatch and raw wall deltas honestly. Default deterministic work
and existing goal indicators must equal T11.F18; report identity metadata
changes separately. Record dated persistence/population, births, lineage
diversity, structure, memory sensitivity, founder/evolved neighborhoods,
drift rows, and observation times. No improvement claim is required.

The T11 track explicitly makes floors track-level, due by T11.F10; the
program-wide per-feature rule is no regression versus the preceding closure.
T11.F18 records drift changed/all births of 0.010000 at generation 1,000 and
0.005000 at 2,000; the latter is below the standing 0.008000 target (its
feature-specific exception does not lower that target). Preserving these
readings meets T12.F01's no-regression rule, without claiming the outstanding
track target is met or importing F18's exception. Escalate any new decrease,
failed required check, or conflicting floor requirement before closure.

Caps remain founder neighborhood 10 seconds, evolved neighborhood 180 seconds
summed across goal seeds, drift observation 30 seconds, and total goal-profile
investigation 900 seconds. Keep profile parameters, seeds, samples/trials, and
mutation floors unchanged. No world-set sweep is due before that set exists.

Measured reports and acceptance: pending implementation.

## Success Criteria

- [ ] Configured terrain and seed rules produce the specified applied world
  before food/founders, with deterministic terrain-bearing tests passing.
- [ ] The startup UI and API expose the complete terrain config; runtime
  mutation is rejected; canonical documentation describes applied behavior.
- [ ] Production defaults and deterministic trajectory are unchanged; required
  checks, fresh mutation triage, and gate/goal reports are complete.

## Notes for AI Agents

- Plan/readiness owner: persistent Astra `high`; orchestrator Astra `low`,
  persistent implementer Astra `low`, fresh final reviewer Astra `medium`.
  Planning/readiness is not an advisor consultation. Execution follows
  `docs/workflow.md`; no additional agents or workflow machinery.
- Readiness self-review (2026-09-08): **Ready after one revision**. P1: 0.
  P2: 2, both resolved: track status used the master's `Active` instead of
  track `In Progress` (caught by `make roadmap-check`); RNG fallback metadata
  needed an explicit historical-report missing-value rule to avoid breaking
  baseline loading. The revision fixes both. No remaining readiness findings.
  Review limits: no implementation, RNG probe, or runtime validation yet;
  this self-review is not the independent final diff review. The research
  seed-expansion assumption is explicitly corrected above; the mandatory
  executed probe remains. The standing drift target and per-feature
  no-regression distinction remain explicit, with no new exception granted.
- Closure record: advisor consultations, decisive guidance, reviewer counts,
  remediation passes, requirement corrections, user interventions, tested
  commit, and task-specific usage pending; record `usage unavailable` if absent.
