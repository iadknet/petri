# T12.F02 — World Recipe Save and Load

**Status**: Complete
**Last updated**: 2026-09-08
**Feature**: T12.F02
**Track**: [T12 — World Composition and Baseline Worlds](../../roadmaps/t12-world-composition-and-baseline-worlds.md)

## Goal

A world is a file: the app and CLI save the effective procedural world config
and load that recipe with a run seed to regenerate the same tick-zero world.
CLI run events identify the applied config, and recipe-backed sweeps record
which recipe and effective config they used.

## Non-Goals

- No checkpoints, living population/state persistence, manual terrain overlays
  (T12.F05), differentiated food or baseline world recipes (T12.F04).
- No new recipe schema, version envelope, migration system, config editor,
  storage service, dependency framework, or changes to evolution/defaults.
- No measured benchmark runs or stored benchmark reports for this closure,
  by the user's explicit exception recorded below.

## Inputs and Invariants

The owning roadmap row defines dependencies. [T12.F01](t12-f01-seeded-terrain-in-the-world-config.md)
provides procedural terrain, `world.world_seed`, per-layer seeds, and
reproducibility for the locked dependencies/platform. Preserve its seed rule:
map seed overrides terrain/fertility only; food, founders, and runtime still
use the separately supplied run seed. The recipe is a `SimulationConfig` JSON
object, partial or complete. Run seed remains outside this config, supplied
through the existing app Run Seed or CLI `--seed` / sweep `--seeds`. The UI
and documentation must say that identical regeneration requires the same run
seed, including when `world_seed` is fixed. Saving terrain parameters/seeds
does not save runtime paint strokes or an evolved world.

Local evidence: `v3-server/src/types.rs` already owns recursive object merge
and sorted-JSON SHA-256 config digest; `http/lifecycle.rs` merges startup over
its baseline, validates ramps, normalizes and applies startup overrides.
`v3-cli/src/main.rs` currently deserializes a complete config and the benchmark
builder starts from defaults. The frontend's startup preset is only a subset
of `SimulationConfig`; its restart handler also reapplies prior runtime fields.
Recipe loading must neither drop fields outside that subset (for example
`population.founder_profile`, mutation, runtime and energy settings), nor let
that reapplication overwrite the recipe.

Research recheck, 2026-09-08: extend those existing merge/digest functions and
Serde JSON versus a JSON Merge Patch dependency or separate world format.
The existing functions win on schema fit and preserving the established null
and array semantics; a generic merge-patch implementation would interpret
null as deletion ([RFC 7396](https://www.rfc-editor.org/rfc/rfc7396)) and adds
an unnecessary dependency. The existing workspace
`sha2` and `hex` dependencies suffice. [Serde JSON Value](https://docs.rs/serde_json/latest/serde_json/value/enum.Value.html)
provides the object/array representation already used locally; preserve the
existing recursively sorted digest encoding rather than claiming a new JSON
canonicalization standard. Browser-native file input and Blob downloads are
sufficient versus the File System Access API: no directory access or durable
handles are needed. [URL.createObjectURL](https://developer.mozilla.org/en-US/docs/Web/API/URL/createObjectURL_static)
documents Blob URLs and their release with `revokeObjectURL`. No package
adoption or broad browser compatibility redesign is needed.

Required behavior:

- Reuse one Rust config merge/resolution path for server startup and CLI
  recipe consumers, located at the existing core config boundary so neither
  CLI nor core depends on the server. Merge recursively only when both values
  are objects; arrays and other values replace wholesale, including null.
  Merge over current defaults (the server keeps its injected test baseline),
  deserialize with existing unknown-field rejection, preserve existing startup
  ramp validation, normalize, and apply tick-zero startup overrides. No
  loosening of schemas or silently ignored malformed recipes. Input must be
  an object; absent fields use defaults, `{}` is valid, stale/unknown fields
  and invalid shapes fail before replacing a live simulation or emitting a run.
- Preserve the existing digest format `sha256:<lowercase hex>` over compact,
  recursively key-sorted JSON of the effective config. File whitespace/key
  order and omitted defaults do not change it. Arrays retain order. Hash the
  config actually used after normalization/startup overrides, not file bytes,
  an unnormalized patch, or a config reconstructed from report dimensions.
  Reuse the same implementation for startup responses and `run_started`.
- `v3-cli run --config <path>` accepts the partial recipe rule above.
  Add `--save-config <path>` to save its full effective config as readable
  JSON before running; it works with or without `--config`, excludes run seed
  and NDJSON, and reports read/parse/write failures clearly with nonzero exit.
  The saved config reloads unchanged under the same code and run seed.
  `run_started` gains `config_digest`; existing event ordering remains intact.
- Add visible Save Recipe and Load Recipe actions to the app's existing
  configuration/startup controls. Save fetches the current server effective
  config through `GET /v3/simulation/config`, rather than exporting stale
  cached data, unsent startup edits, or the frontend's subset preset. Download
  just the config as JSON. A small response option on the existing config
  endpoint is acceptable if needed to preserve raw JSON number precision.
- Load reads a recipe file and sends the entire recipe with the current run
  seed through the existing startup endpoint, using the existing restart
  confirmation policy for a running/paused simulation. After success, refresh
  server config, startup fields and simulation state to the applied tick-zero
  world; do not reapply the previous simulation's runtime patch. Preserve all
  accepted config fields through this request, including nested fertility
  seeds and founder profile. File/validation/network errors are visible and
  do not misleadingly report a loaded world. Cancellation makes no request.
  Loading is a deliberate restart, not a runtime config patch.
- Recipe transport/save must preserve Rust u64 seeds exactly, including values
  above JavaScript's safe integer range; avoid a parse-to-Number/stringify
  roundtrip for opaque imported/exported JSON. Existing numeric form inputs
  retain their safe-integer limits. Do not introduce a custom JSON parser or
  a general arbitrary-precision editor to solve opaque file transfer.
- `bench --profile sweep --config <path>` resolves the same recipe. With a
  recipe, omitted width/height/founders come from its effective config;
  explicitly supplied width/height/founders and `--food-coverage` override
  their existing domains afterward, then normalize/apply startup overrides
  before hashing or seeding. Without a recipe, preserve the current
  required sweep arguments and default behavior. `--seeds`, `--ticks` and
  sweep output requirements stay unchanged. Reject `--config` for gate/goal
  rather than silently changing a predeclared profile.
- Recipe sweeps use the complete resolved config for every seed, not a lossy
  reconstruction from `ProfileParams`. Record the supplied recipe path and
  effective `config_digest` in the report's profile block; dimensions,
  founder count and coverage description must agree with what ran. Recipe
  coverage must not be called production-default coverage. Different effective
  recipe digests cannot compare as the same profile. Historical reports and
  recipe-free gate/goal reports keep their existing comparison behavior:
  absent recipe metadata is None/unmeasured and is not fabricated or backfilled.
- Update affected CLI, server API and startup reference contracts plus concise
  usage examples in existing documentation. No benchmark series/baseline edits.

## Implementation Tasks

- [x] Establish failing tests for partial-config resolution and effective
  digest/save roundtrip, then share the existing Rust helpers and wire CLI
  run save/load and digest metadata without changing default behavior.
- [x] Add sweep recipe resolution, explicit override precedence and truthful
  profile metadata; cover the config actually passed to seeding.
- [x] Add app save/load controls using the existing API and startup flow;
  preserve opaque recipe fields/seed precision and refresh applied state.
- [x] Update directly affected reference/usage docs, self-review the feature
  diff for reuse/simplification/efficiency, triage fresh mutation survivors,
  and record final verification and review evidence.

## Verification

- `make check` exited 0 on all implementation code (2026-09-08), log
  `/tmp/t12-f02-make-check.log`: registered Rust suites including viability,
  core properties, CLI, server, reproducibility and ordinary guard tests passed;
  frontend 60 suites / 319 tests and build passed; OSV reported no issues.
  Subsequent test-only self-review strengthened digest sensitivity/format checks;
  `cargo test -p v3-core --lib recipe_tests` passed 3 tests and
  `cargo check --workspace --all-targets` exited 0 afterward (logs
  `/tmp/t12-f02-core-strengthened.log`, `/tmp/t12-f02-final-compile4.log`).
  The orchestrator's final `make check` exited 0 on rebased tested code commit
  `ef4af21f4d8da33af02a0285369888c2c7b3b469` (log
  `/tmp/t12-f02-orchestrator-final-check.log`).
- Final fresh `MUTANTS_ITERATE=0 make rust-mutants` exited 0 (2026-09-08):
  `38 mutants tested in 4m: 32 caught, 6 unviable`; `rust-mutants: no survivors`.
  Output: `/Users/istefanek/.local/share/petri-tools/mutants/t12-f02/mutants.out`,
  `run-mode.txt` is `fresh`; `missed.txt` and `timeout.txt` are empty.
  Log: `/tmp/t12-f02-rust-mutants-final.log`. All seven initial survivors below
  are killed. Additional frontend file-read/export-network failure coverage
  passed (5 RecipeControls tests, `/tmp/t12-f02-final-ui-tests.log`).
- First fresh `MUTANTS_ITERATE=0 make rust-mutants` exited 0:
  `38 mutants tested in 4m: 7 missed, 25 caught, 6 unviable`.
  Output: `/Users/istefanek/.local/share/petri-tools/mutants/t12-f02/mutants.out`;
  retained first-run log: `/tmp/t12-f02-rust-mutants.log`. No timeouts.
  Full initial survivor list (each **killed** by test-only assertions and the
  final fresh rerun):
  - `crates/v3-cli/src/bench.rs:112:9`: replace Recipe equality with `true`.
  - `crates/v3-cli/src/bench.rs:112:9`: replace Recipe equality with `false`.
  - `crates/v3-cli/src/bench.rs:113:13`: replace `&&` with `||` in equality.
  - `crates/v3-cli/src/bench.rs:112:19`: replace path `==` with `!=`.
  - `crates/v3-cli/src/bench.rs:114:17`: replace config `==` with `!=`.
  - `crates/v3-core/src/config/recipe.rs:57:9`: replace ConfigError display with
    `Ok(Default::default())`.
  - `crates/v3-core/src/config/recipe.rs:84:30`: replace ramp target `<` with `<=`.
  Recipe equality now covers identical, different-path and different-config
  cases; core assertions pin the diagnostic and accept target tick 1. Also
  strengthened CLI save/reload with terrain/fertility u64 seeds and hidden
  founder/runtime fields, comparing binary output with direct core-backed CLI
  output. Focused core (4 tests), recipe CLI tests and coherent compile passed
  (`/tmp/t12-f02-mutant-{core-tests,cli-tests,compile}.log`). Repeated self-review
  found the test changes scoped and reused existing fixtures; production code
  was not edited to kill mutants. No skips, exclusions or deferred survivors.
- TDD evidence (2026-09-08): core `cargo test -p v3-core --lib recipe_tests`
  first failed on missing resolver/hash API, then passed all 3 tests (including
  2 proptests). CLI `cargo test -p v3-cli --test cli
  recipe_cli_save_reload_and_failures` failed on absent `--save-config`, then
  the full CLI suite passed 11 tests. Sweep binary recipe test failed on required
  width without recipe support, then all 10 argument tests passed. Default
  export test failed on missing save helper, then passed after extraction.
  Server raw-export test failed because the envelope lacked root config fields,
  then passed with exact u64 seeds and identical barriers/fertility/food/founders
  after reload and comparison with direct core seeding. Profile metadata test
  passed on a tiny ordinary fixture without storing a report.
- Frontend TDD: raw API and store tests first failed on missing recipe APIs,
  seed guard and forced recipe refresh; controls test failed on missing component.
  After implementation all 60 suites / 319 tests passed, and `make frontend-check`
  passed after correcting test-only TypeScript indexed-access errors. Browser
  inspection used a temporary 8×8/two-founder tick-zero server (removed afterward):
  Save displayed `Recipe downloaded`; Load retained Run Seed 42, applied hidden
  founder/energy fields, and server readback preserved both u64::MAX seeds.
  Screenshots: `/tmp/t12-f02-recipe-loaded.png`, `/tmp/t12-f02-recipe-final.png`.
  No production world was run for file-handling verification.
- Focused logs are `/tmp/t12-f02-{core,cli,sweep,server,ui}-*.log`,
  `/tmp/t12-f02-profile-test.log`, `/tmp/t12-f02-default-save-*.log`, and
  `/tmp/t12-f02-frontend-check2.log`. Coherent Rust compile checks passed
  (`/tmp/t12-f02-final-compile2.log`); final check/mutation evidence is recorded above.

- Prerequisite audit repair (2026-09-08), before recipe implementation:
  `npm install --save-dev 'vitest@^4.1.11'` in `frontend/` exited 0 after
  retrying outside the network-restricted sandbox (initial attempt exited 1,
  `ENOTFOUND registry.npmjs.org`). `make frontend-check` exited 0: lint
  checked 204 files, all 59 test files / 311 tests passed, and TypeScript/Vite
  production build passed. `AQUA_ROOT_DIR="$HOME/.local/share/aquaproj-aqua"
  make dependency-audit` exited 0: 211 Cargo and 310 npm packages scanned,
  `No issues found`. Logs: `/tmp/t12-f02-vitest-install-retry.log`,
  `/tmp/t12-f02-vitest-frontend-check.log`, `/tmp/t12-f02-vitest-audit.log`.
  Full feature `make check` remains required before closure.
- [x] Record TDD red/green commands. Core config unit/property tests cover
  default merge, nested object preservation, array replacement, null behavior,
  strict field rejection, normalization/startup override equivalence, and
  config save/reload/digest invariants for every generated case. Include full
  u64 seed values; no property assertion depends on which cases were drawn.
- [x] Server and CLI tests use a small terrain/fertility recipe with fixed
  seeds and non-default hidden startup/runtime fields. Compare effective
  configs/digests and regenerated barriers, fertility, food and founders at
  tick zero; cover load failure leaving server state unchanged. Reuse existing
  registered suites, or register new suites in `make check` and CI.
- [x] CLI tests cover partial and complete files, default save, save/reload,
  malformed/non-object/stale files, save I/O failure, digest in `run_started`,
  and unchanged event order. Sweep tests cover no-recipe compatibility,
  recipe-derived dimensions/founders, flag precedence, config delivered to
  seeding, metadata serialization/comparison and gate/goal rejection. Use
  ordinary tiny fixtures; do not invoke measured benchmark workloads or
  persist reports.
- [x] Frontend tests cover current-config export, exact large-seed transport,
  file selection/cancellation, complete startup payload, no stale runtime
  reapplication, success refresh and visible failure. Inspect the rendered
  controls and one small save/load interaction; do not run a production world
  or benchmark to demonstrate file handling.
- [x] Load relevant Rust/React/UI skills for affected changes. Run
  `cargo check --workspace --all-targets` after coherent Rust edits and focused
  Rust/frontend checks. The viability-first rule applies if changes reach
  production defaults, founder behavior or tick-loop mechanics; these are
  otherwise outside scope. Required `make check` retains its viability gate.
- [x] After self-review, run fresh `MUTANTS_ITERATE=0 make rust-mutants` and
  record the summary, output path and complete missed/timeout list. Resolve
  each as killed through tests plus fresh rerun, equivalent with a reason, or
  deferred in Notes. No unrecorded skips/exclusions; incremental results are
  never closure evidence.
- [x] Measured benchmark report: Not applicable by explicit user direction
  for T12.F02 (2026-09-08). Run no measured benchmark workloads/scripts and
  create no benchmark reports. Ordinary `make check` gate tests, mutation
  checks, focused tests and documentation verification remain required.
  `scripts/bench-wait` as the mutation concurrency guard and its ordinary
  regression tests do not run a benchmark and remain part of those checks.
- [x] Second goal determinism run: Not applicable by the workflow's standing
  2026-09-05 decision and this closure's benchmark exception.
- [x] `make roadmap-check` on document edits/handoff; orchestrator `make check`
  on final feature code with tested commit recorded; `make check-docs` on
  closure documents. Preserve actual logs and command results.

## Performance and Goal Impact

This is recipe/config infrastructure, not an ecological mechanism or new
cognition/diversity indicator. Predeclared cost is file parsing/serialization
and one config hash at startup/report construction. There is no per-tick cost,
new random draw, seeding algorithm change or production-default change.

**User-authorized exception, 2026-09-08:** do not run benchmark scripts or
create benchmark reports because recipe save/load does not change core
evolution behavior. This exception is feature-specific and does not waive
`make check`, `make check-docs`, fresh mutation triage, ordinary gate tests or
other nonbenchmark checks. The mutation concurrency guard
(`scripts/bench-wait`) and `bench-wait-test` exercise process coordination,
not a measured benchmark; their use by required checks is retained.
Gate/goal compute and wall-clock deltas, indicator
readings and observation times are **unmeasured**. Do not copy historical
readings as current evidence, add a measured-series row, weaken thresholds,
re-pin the epoch or alter stored baselines. Production-default preservation
is verified by code/tests; full trajectory measurement is not claimed.

## Success Criteria

- [x] App and CLI recipes preserve the full effective procedural config and
  regenerate identical tick-zero state with the same run seed; errors are
  explicit and manual/evolved state is not represented as saved.
- [x] CLI run digest and recipe sweep path/digest identify the applied config;
  existing recipe-free behavior and fixed profiles remain intact.
- [x] Required tests, fresh mutation triage, independent final review and
  closure records pass with the benchmark exception accurately represented.

## Notes for AI Agents

- Main start: `0bd178dd40b1ea19905d354d76e5e19b8a82bfbd`; feature worktree:
  `/Users/istefanek/projects/petri/.worktrees/t12-f02`, branch `codex/t12-f02`.
  Track is already In Progress and master Active; no rollup promotion needed.
- Model/effort allocation: orchestrator `gpt-6-astra` low (session metadata
  verified by orchestrator); persistent spec owner/advisor high; persistent
  implementer low; fresh final reviewer medium. All roles use Astra; the
  orchestrator verified every role setting from session metadata.
- Readiness self-review (2026-09-08): **Ready after one revision**. P1: 0;
  P2: 1 resolved: distinguish forbidden measured benchmarks from the
  concurrency guard/regression tests required by mutation and `make check`.
  The revision also makes post-flag normalization explicit and links the
  researched merge-patch semantics. No unresolved questions; run seed remains
  a separate input by the existing CLI/config contract. Limits: requirements
  and local code reviewed, implementation/runtime behavior not yet verified.
  Planning/readiness is not an advisor consultation or independent review.
- Plan `make roadmap-check` exited 0 before and after the readiness revision
  (2026-09-08); no implementation checks or benchmark workloads ran.
- Advisor consultation 1 (prerequisite repair): accepted the bounded direct
  Vitest update to 4.1.11 and its lockfile dependency family because the plan
  commit's existing dependency-audit gate failed on GHSA-82fw-gwwq-j7x9 in
  Vitest / `@vitest/mocker` 3.2.7. No compatibility edits were needed; Vite 6,
  Node 24, jsdom, all existing assertions and `min-release-age=7` are retained.
  No overrides, audit exclusions or policy weakening were added. This is
  a required gate repair, not recipe feature scope.
- Advisor consultation 2 (approach): accepted shared core resolution and
  existing endpoint reuse, exact raw JSON transport, rejection of embedded run
  seed, preserved selected Run Seed on refresh, a safe-integer guard on ordinary
  Restart, final-config sweep metadata and distinct post-restart refresh errors.
  Each closes a concrete recipe correctness requirement; no optional refactors.
- Advisor consultation 3 (clarification): accepted that profile `founders` means
  normalized requested `initial_creatures`; actual placement retains the existing
  passable-cell clamp. No new aggregate or requirement correction.
- Self-review for reuse/simplification/efficiency: moved existing merge/hash code
  rather than duplicating it; extracted the existing report-profile construction
  to test metadata without a measured workload; isolated config-file writing so
  default export is tested without running a production world. Shared fetch error
  handling serves raw recipe responses. No per-tick work or new external package.
- Advisor consultation 4 (before handoff): accepted the no-change recommendation.
  The advisor found no concrete scope/correctness blocker in shared config
  semantics, raw seed transport, stale-runtime bypass or sweep metadata, and
  inspected the final fresh 38/32/6 mutation result with no survivors. The
  equivalent Vitest repair on main does not change recipe behavior.
- Independent final review (fresh Astra medium, 2026-09-08): P1: 0; P2: 0;
  P3: 0. Reviewer read the original contract, full diff including untracked
  files, and fresh mutation evidence. Its initial token-revocation startup
  failure was retried successfully with the same reviewer agent; no review
  requirement was waived. Post-review remediation passes: 0.
- Mutation remediation: 1 test-only pass addressed all seven initial survivors;
  the final fresh run found none. No production changes were made to kill mutants.
- Advisor consultations: 4. Requirement corrections: none; consultation 3
  clarified existing requested-founder semantics. User intervention: the
  explicit benchmark exception above.
- Integration preparation: main advanced to
  `ea23febbb8ff6475115a7f3232649043aae06d07` with the equivalent Vitest repair.
  The orchestrator rebased successfully; `git diff 2c26c429 HEAD` was empty,
  preserving the exact reviewed content. Final `make check` passed on rebased
  code commit `ef4af21f4d8da33af02a0285369888c2c7b3b469` before these
  documentation-only closure edits. T12 remains In Progress and the master
  remains Active because the remaining track criteria/features are incomplete.
  Documentation-only closure `make check-docs` exited 0, log
  `/tmp/t12-f02-closure-check-docs.log`. Its first attempt stopped because the
  Complete spec still had unchecked closure boxes; after preparing those boxes,
  the full documentation check passed without waiving a check. Fast-forward and
  worktree/branch cleanup follow separately.
- Task-specific usage snapshot, 2026-09-09T00:01:36Z, aggregated across the four
  role sessions: input 29,394,567 tokens, including cached input 28,853,888;
  output 70,034, including reasoning 12,266; total 29,464,601. Uncached input
  plus output: 610,713. Cached input is a subset of input, and reasoning is a
  subset of output. This timestamped snapshot excludes subsequent closure
  turns and is not a final bill; no dollar cost is inferred.
