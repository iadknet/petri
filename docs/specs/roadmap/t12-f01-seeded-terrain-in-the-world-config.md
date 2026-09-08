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

- [x] Establish failing tests for terrain config, startup ordering/seed
  isolation, clustered Noise trimming, and runtime patch rejection; execute
  and record the RNG compatibility probe before choosing its branch.
- [x] Add the core schema/seeding behavior and bounded reproducibility fix;
  implement the selected RNG branch and its required report evidence.
- [x] Carry the fields through server startup/projections, frontend types,
  preset hydration/request construction, and the Terrain controls; update
  canonical references and integration tests.
- [ ] Review the diff for reuse, simplification, and efficiency; resolve fresh
  mutation survivors through tests; complete closure records after required
  checks, recording the user-authorized measured-benchmark exemption below.

## Verification

- [x] Record TDD red and green commands. Core examples cover absent/default
  fields, serialization, all five patterns, whole-world and explicit bounds,
  overlapping layers, empty/all-barrier maps, no food/founder on barriers,
  exact eligible-cell food coverage, and founder count clamping. Include a
  1600² startup terrain case proving the runtime endpoint cap is not reused.
- [x] Property tests establish bounded/idempotent barrier union and deterministic
  generation for every drawn case, explicit seed precedence, wrapping seed
  addition, and clustered Noise exact target/set reproducibility. Explicitly
  exercise every pattern and the trimming path, rather than assuming a random
  draw reaches them. Keep any `proptest-regressions/` files.
- [x] Extend `crates/v3-core/tests/reproducibility.rs` with a terrain-bearing
  fixture that includes clustered Noise trimming. Compare barriers, fertility,
  food, and founders at tick zero and applied short-run state under independently
  initialized simulations and different Rayon thread counts. Preserve the
  existing mutation stress fixture and its coverage. Fresh randomized hash
  states exercise the process-order failure as in T10.F11; do not add a second
  goal run or subprocess harness merely to duplicate it.
- [x] Fixed map seed plus distinct run seeds yields equal barrier/fertility
  grids and distinct run-seeded placement on an explicit nondegenerate fixture;
  absent map seed preserves the prior seeding behavior. Focused seeding and
  reproducibility tests verify unchanged defaults; full trajectory comparison
  to T11.F18 is unmeasured under the explicit benchmark exemption.
- [x] Server tests cover partial startup terrain overrides, unknown fields,
  restart-only PATCH rejection with state/config unchanged, effective config,
  and applied tick-zero barrier projection. Frontend tests cover layer edits,
  hydration, bounds and optional seeds, pattern switches, safe integer input,
  and the actual startup request; inspect the rendered Terrain section.
- [x] Run `cargo test -p v3-core --test viability` first after seeding/RNG changes,
  then `cargo check --workspace --all-targets` after coherent Rust edits and
  focused core/server/frontend checks. Load relevant Rust and React skills.
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants` after self-review: record
  summary, output path, and complete missed/timeout list, with each killed by
  tests and a fresh rerun, equivalent with reason, or explicitly deferred.
- [x] Measured gate/goal reports: Not applicable for this closure by explicit
  user direction on 2026-09-08, recorded below. Neither profile was started.
  No F01 benchmark JSON or measured series row is produced; existing reports
  and epoch baselines remain unchanged. `make check` and its ordinary gate
  determinism tests remain required.
- [x] Second goal determinism run: Not applicable by the workflow's 2026-09-05
  decision. Existing reproducibility and gate two-run checks remain mandatory.
- [ ] `make roadmap-check` on document edits and at handoff; `make check` on
  final feature code; `make check-docs` on closure documents. Record command
  evidence and the tested commit before integration.

### Implementation evidence (2026-09-08)

- RNG probe: `cargo run --manifest-path /tmp/t12-f01-rng-probe/Cargo.toml`
  exited 0, comparing 16 `next_u64` values for each seed with exact `rand`
  0.8.6 and `rand_xoshiro` 0.6.0. First draws (SmallRng / named) were:
  0: 8251690495967107212 / 5987356902031041503;
  1: 13159342511175687856 / 14971601782005023387;
  11: 9512672960468237878 / 15860195524371628672;
  22: 2580056678536133757 / 4966237793422419871;
  33: 14475685761841174007 / 17129061156981078465;
  u64::MAX: 11345198270385851335 / 6254647548650071986.
  Every stream differed. Retained SmallRng, no seed adapter or dependency
  change. Its wrapper inherits PCG32 seed expansion whereas the named
  generator overrides it with SplitMix64. New reports derive `rand_version`
  from the compiled Cargo.lock; old reports deserialize missing metadata as
  None/unmeasured. Probe log: `/tmp/t12-f01-logs/rng-probe.log`.
- TDD red: `cargo test -p v3-core --test terrain` exited 101 with the expected
  unknown `terrain` field and distinct clustered Noise trimmed sets for seed
  42 (the observed set mismatch proves the trimming branch ran).
  `cargo test -p v3-server --test server terrain_fields_are_restart_only`
  exited 101 because the keys were generic unknown fields rather than
  restart-only. Frontend TerrainSection test initially failed to import the
  unimplemented component. Logs: `core-red.log`, `server-red.log`,
  `frontend-red.log` under `/tmp/t12-f01-logs/`.
- After production seeding edits, FIRST `cargo test -p v3-core --test viability`
  exited 0, 24 passed (`viability.log`). Explicit coherent-burst
  `cargo check --workspace --all-targets` checks passed; final log
  `compile-final.log`. One server test compile error used the wrong frame
  wrapper and was corrected to assert the actual published projection.
- `cargo test -p v3-core --test terrain`: 10 passed (`core-final.log`).
  Property cases explicitly iterate all five patterns and include clipping,
  exact expected sets, idempotence, serde roundtrip, override precedence,
  wrapping at u64::MAX, and clustered target reproducibility. No property
  regression file appeared. An initial food coverage test used the shared
  value instead of the authoritative type entry; correcting the fixture
  to typed coverage made the 400/800 eligible-cell assertion pass.
- `cargo test -p v3-core --test reproducibility`: 3 passed (`repro.log`),
  preserving the mutation stress test and adding terrain initialization,
  1/4-thread applied short-run comparisons, and fixed-map/run-seed isolation.
- `cargo test -p v3-server --test server terrain`: 2 passed (`server-green.log`),
  including effective GET config, partial startup merge, unknown layer fields,
  actual tick-zero projection, and atomic null/empty runtime rejection.
- `cargo test -p v3-cli build_environment`: passed (`metadata.log`), including
  actual locked version and historical missing-field loading.
- `npm test -- --run src/components/config-panel/startup/TerrainSection.test.tsx
  src/stores/startupConfig.test.ts src/components/ControlBar.test.tsx
  src/components/ConfigPanel.test.tsx`: 58 passed (`frontend-green-2.log`).
  `npm run build`: passed (`frontend-build-2.log`); existing chunk-size warning.
  `npm run lint:fix`: completed with existing index-key warnings plus the
  stateless ordered terrain layer's index key; no schema IDs were invented.
- Browser skill inspection at localhost:5312 verified Terrain adjacent to
  World Topology, adding a layer, and switching Maze to Noise with correct
  controls, labels and seeds. Screenshot inspected:
  `/tmp/t12-f01-logs/terrain-ui.png`; browser and development stack stopped.
- Self-review for reuse/simplification/efficiency: extracted the existing
  pattern panel for shared use; reused all five existing parameter editors
  and defaults; used React useId for parameter labels so multiple layers
  have unique input IDs. Kept clipping/seeding local, no framework or new
  dependency. Property-only fixtures omit unnecessary founders. Affected
  checks reran successfully. `git diff --check` passed.
- `make roadmap-check`: passed on reference/spec changes (`roadmap-1.log`).
  Fresh mutation evidence is recorded below. Full measured benchmarks are exempted by the
  user's later direction below; other required checks remain in force.

- Full frontend completion checks: `npm run test` exited 0, 59 files / 311
  tests (`frontend-all.log`); `npm run build` exited 0
  (`frontend-build-final.log`); `npm run lint` exited 0
  (`frontend-lint-final-2.log`, four index-key warnings). The initial final lint
  found only a test formatting issue; formatting that file and rerunning lint
  passed. No behavior changed. Logs remain under `/tmp/t12-f01-logs/`.

### Mutation evidence

- First fresh `MUTANTS_ITERATE=0 make rust-mutants` exited 0:
  `18 mutants tested in 5m: 2 missed, 13 caught, 3 unviable`.
  Log `/tmp/t12-f01-logs/mutants.log`; output
  `/Users/istefanek/.local/share/petri-tools/mutants/t12-f01/mutants.out`.
  Full missed list, no timeouts:
  - `crates/v3-core/src/simulation/seeding.rs:54:30: replace || with && in seed_simulation`
    — **equivalent**: when exactly one clipped dimension is zero, the
    shared `generate_pattern` returns an empty list before consuming RNG;
    the changed early-return condition cannot alter the world or run stream.
  - `crates/v3-server/src/http/status.rs:87:13: replace || with && in patch_touches_world_topology`
    — **killed** in the final fresh run: expanded the existing
    topology test to independently reject width, height, and edge_mode
    PATCHes with restart-only reasons. Production code was not changed.
- Test remediation self-review found no additional changes needed.
  `cargo check --workspace --all-targets` exited 0 (`compile-remediation.log`);
  `cargo test -p v3-server --test server patch_config_world_topology_fields_are_restart_only`
  exited 0, 1 passed (`server-remediation.log`).
- Second fresh `MUTANTS_ITERATE=0 make rust-mutants` exited 0:
  `18 mutants tested in 5m: 1 missed, 14 caught, 3 unviable`.
  Log `/tmp/t12-f01-logs/mutants-final.log`; same output directory above,
  `run-mode.txt` reads `fresh`. Complete second-run survivor list:
  - `crates/v3-core/src/simulation/seeding.rs:54:30: replace || with && in seed_simulation`
    — **equivalent**, for the empty-generator reason above.
  `timeout.txt` is empty. No unresolved/deferred mutation finding remains.
  No mutation skip or exclusion was added.

- Final registered-test fresh `MUTANTS_ITERATE=0 make rust-mutants` exited 0:
  `18 mutants tested in 4m: 1 missed, 14 caught, 3 unviable`.
  Log `/tmp/t12-f01-logs/mutants-registered.log`; standard output directory
  above, `run-mode.txt` is `fresh`. Complete final survivor list:
  - `crates/v3-core/src/simulation/seeding.rs:54:30: replace || with && in seed_simulation`
    — **equivalent** for the empty-generator reason above.
  No timeouts. The server topology mutant remains killed.
- `make rust-test-terrain` exited 0, 10 passed (`terrain-target.log`).
  `make roadmap-check quality-check` exited 0 (`registration-check.log`),
  including actionlint, shell checks, benchmark-wait and mutation-wrapper
  regression checks. Final `git diff --check` passed.

## Performance and Goal Impact

Natural analog: bedrock and water constrain movement, food occupancy, founder
placement, and existing line-of-sight opacity through the world. No new sensor
or diversity/cognition indicator is introduced.

Predeclared cost: opt-in terrain generation and clustered candidate sorting at
startup only. Empty terrain and absent world seed preserve all production
draws and trajectories. No severe compute allowance, new epoch, baseline edit,
threshold reduction, or extra goal-profile run is authorized.

**User-authorized verification exception, 2026-09-08.** The user stated:
"I don't think we need to do a full benchmark test for this feature, since it
doesn't change evolution/behavior. We will have to once we change the baseline
benchmark maps." This overrides the planned measured gate and goal runs for
T12.F01, whose opt-in terrain leaves production defaults unchanged. Neither
profile was started. Focused behavior/reproducibility tests, fresh mutation
triage, independent review, `make check` (including ordinary gate determinism
tests), and closure documentation checks remain required.

F01 normalized compute deltas, wall-clock deltas, goal indicators, and
observation timings are **unmeasured**. There is no new report, no copied
historical reading presented as current evidence, and no measured-series
entry. Unchanged default trajectories remain an implementation invariant,
supported by the focused tests and diff; full benchmark identity is not
claimed as measured. Existing thresholds and epoch baselines are unchanged.
Full benchmark readings are required when baseline benchmark maps change;
this exemption does not edit the global workflow or authorize later skips.

Preserve the program-wide no-regression rule and the specific standing drift
floors. T11.F17 applies strict not-below floors to later closures; T11.F18's
feature-specific exception explicitly retains the generation-2,000 floor of
0.008000 for later features. These specific drift requirements govern over the
earlier general T11 paragraph about track floors due by T11.F10. Historical
T11.F18 changed/all births were 0.010000 at generation 1,000 and 0.005000 at
2,000; the latter remains below the standing 0.008000 floor. The
generation-1,000 floor remains 0.001500. These are historical findings, not
new F01 measurements or a newly waived measured floor failure. Under the
explicit exemption F01 has no fresh drift-floor evaluation; closure relies on
the remaining authorized checks and does not require an unrelated evolution
repair. No floor is lowered or claimed satisfied, and F18's exception is not
transferred. Any failure of a remaining required check still blocks closure.
No world-set sweep is due before that set exists.

## Success Criteria

- [ ] Configured terrain and seed rules produce the specified applied world
  before food/founders, with deterministic terrain-bearing tests passing.
- [ ] The startup UI and API expose the complete terrain config; runtime
  mutation is rejected; canonical documentation describes applied behavior.
- [ ] Production defaults and deterministic trajectory are unchanged; required
  checks and fresh mutation triage are complete, with full trajectory
  measurement explicitly exempted and its limits recorded above.

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
  executed probe remains. The initial drift-floor interpretation was corrected
  by advisor consultation 2 below; readiness did not waive a required check.
- Advisor consultation 1, before-approach checkpoint (2026-09-08): accepted
  localized schema/seeding and existing UI reuse. Accepted all decisive
  guidance because it matched the spec: clip before generation and skip empty
  intersections; derive seeds independently without run draws; execute the
  stream probe before fallback; test trimming and applied projections;
  reject runtime fields by presence, including null/empty. No optional scope.
- Advisor consultation 2, requirement correction (2026-09-08): the orchestrator
  identified T11.F18's explicit statement retaining the 0.008000 floor for
  later features. Re-reading that decision and T11.F17's strict not-below rule
  showed the planning interpretation was wrong: matching F18's 0.005000 is
  insufficient for closure even though it meets no regression. The spec owner
  corrected the acceptance wording in this serialized document edit. Decisive
  guidance: finish independent implementation and the single required goal
  reading, report an inherited floor failure, and obtain explicit user
  direction before dependent closure work. No code repair, extra goal run,
  threshold change, or new exception was authorized at that consultation.
  The later user-directed benchmark exemption supersedes the proposed reading
  and measured-failure escalation for this closure only.
- Advisor consultation 3, user intervention and verification exception
  (2026-09-08): the user explicitly declined full benchmarks for this unchanged
  default feature and required them when baseline benchmark maps change.
  Decisive guidance: omit measured gate/goal runs and new report/series entries;
  mark their readings unmeasured; retain focused/reproducibility tests, fresh
  mutation triage, review, `make check`, and documentation checks. Preserve
  the historical drift shortfall and standing floor without pretending to
  measure or repair either. This serialized correction grants no global
  workflow change and no production-behavior change.
- Advisor consultation 4, before-done checkpoint (2026-09-08): accepted the
  implementation and equivalent survivor reasoning, but found that the
  Makefile enumerates integration suites and omitted the new terrain suite.
  Accepted as a correctness blocker: registered `rust-test-terrain` in the
  existing `rust-test-all` aggregation and CI target matrix. No parallel
  workflow or unrelated CI repair. Test-selection change triggers another
  fresh mutation run; remaining parent review/completion checks are pending.
  Self-review of this remediation found the minimal existing-target approach
  sufficient; registration checks and the final fresh mutation run passed. Total advisor consultations so far: 4 (two implementer
  checkpoints and two orchestrator requirement/exception consultations).
- Implementer handoff: 5 advisor consultations; 2 pre-review remediation
  passes (mutation coverage and advisor-requested test registration), and one
  post-review remediation pass (Clippy, metadata and test coverage).
  Requirement corrections and user intervention
  are recorded above. Reviewer findings are recorded below; full `make check`, tested
  commit and closure documents remain orchestrator-owned and pending.
  Task-specific usage unavailable.


### Post-review remediation

- Parent `make check` at implementation commit 5eb38dc4 exited 2
  (`/tmp/t12-f01-make-check.log`): Clippy `items_after_test_module` found
  `locked_rand_version` below the benchmark test module. Moved the unchanged
  helper before the tests; no lint allowance. Self-review confirms a pure
  ordering fix with no runtime behavior change.
- Independent review: P1: 0, P2: 1 (lockfile metadata currently selects the first
  rand package, correct for this lock but vulnerable to future dependency
  ordering changes), P3: 1 (stale mutation-pending prose, corrected).
  P2 resolved in the same bounded pass: select the version from v3-core's
  dependency identity, with a unique-package fallback only when Cargo omits
  its version. No dependency or general lockfile framework. Focused fixtures
  cover multiple/reordered packages, dependency upgrade, and unversioned
  unique/ambiguous cases; historical absent metadata remains None.
- Advisor consultation 5 (2026-09-08): accepted bounded metadata correction
  with the Clippy fix; accepted integration onto main 720dc7e2 preserving its
  founder_profile PATCH rejection, selected-profile startup docs, deleted
  experiments and roadmap additions. No overlays or behavior/default changes.
  Guidance accepted because it resolves the specific review issue and keeps
  independent main work. Total consultations: 5.
- Metadata test TDD: `cargo test -p v3-cli lockfile_identity` initially exited
  101 (unimplemented helper), then exited 0 after implementation. Logs
  `metadata-identity-red.log` and `metadata-identity-green.log` under
  `/tmp/t12-f01-logs/`. `cargo check --workspace --all-targets` and
  `cargo clippy -p v3-cli --all-targets -- -D warnings` passed
  (`metadata-identity-check.log`, `metadata-identity-clippy.log`).
- Self-review of the post-review pass: unchanged generator/default behavior;
  dependency-specific metadata resolution, no lint allowance or new dependency;
  no other changes needed. One post-review remediation pass. Fresh mutation
  evidence after the authorized rebase is recorded below; no benchmarks ran.

- Rebased onto main `720dc7e22d2dc0e8f656df6c10a27f2404206fff`, producing
  source commit `2380a11ed9d25613cd21c2965bb7ac2178227054`. The only conflicts
  were startup reference override domains and founder-step wording. Resolution
  preserves run-level selected founder profiles, terrain/world seed fields,
  and founder step 5 after terrain/fertility/food. Main's founder-profile
  PATCH rejection/test, retired experiments, and roadmap additions remain.
  Logs: `/tmp/t12-f01-logs/rebase.log` and `rebase-continue.log`.
- Rebased checks passed: viability FIRST (24), workspace/all-target Cargo
  check, v3-cli all-target Clippy with `-D warnings`, server `restart_only`
  tests (4), CLI `lockfile_identity` (1), and `make roadmap-check`.
  Logs `/tmp/t12-f01-logs/rebased-{viability,check,clippy,server,metadata,roadmap}.log`.
- Rebased fresh mutation run exited 0:
  `24 mutants tested in 4m: 3 missed, 18 caught, 3 unviable`, log
  `/tmp/t12-f01-logs/mutants-rebased.log`, standard output directory above.
  Complete survivor list, no timeouts:
  - `crates/v3-cli/src/bench.rs:2207:45: replace == with != in rand_version_from_lock`
    — strengthened fixture verifies a core without a rand dependency returns
    None; **killed** by the strengthened test in the final fresh run.
  - `crates/v3-cli/src/bench.rs:2211:59: replace == with != in rand_version_from_lock`
    — fixture now includes v3-core's own version, proving non-rand package
    versions are excluded from the unversioned fallback; **killed** by the
    strengthened test in the final fresh run.
  - `crates/v3-core/src/simulation/seeding.rs:54:30: replace || with && in seed_simulation`
    — **equivalent**, unchanged empty-generator reasoning.
  Only tests changed to address the new metadata survivors. Self-review kept
  the same bounded fixture approach. Workspace/all-target Cargo check and
  `cargo test -p v3-cli lockfile_identity` passed (`metadata-survivor-check.log`,
  `metadata-survivor-test.log`). Final fresh confirmation passed below.

- Final rebased fresh `MUTANTS_ITERATE=0 make rust-mutants` exited 0:
  `24 mutants tested in 4m: 1 missed, 20 caught, 3 unviable`.
  Log `/tmp/t12-f01-logs/mutants-rebased-final.log`, output
  `/Users/istefanek/.local/share/petri-tools/mutants/t12-f01/mutants.out`;
  `run-mode.txt` is fresh. Complete final survivor list:
  - `crates/v3-core/src/simulation/seeding.rs:54:30: replace || with && in seed_simulation`
    — **equivalent**: the generator returns an empty set before RNG use when
    either dimension is zero, so skipping that early return changes no
    applied barriers or run stream. No timeouts, exclusions, or deferred
    survivors. Metadata survivors were killed exclusively by tests.
- Review remediation complete: P1 0; P2 1 resolved; P3 1 resolved. Five
  advisor consultations, one post-review remediation pass. Parent owns the
  fresh full `make check`, closure status, and tested-commit record after this
  clean rebased handoff. No benchmark or performance claim was added.
