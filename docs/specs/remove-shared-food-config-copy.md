# Remove Shared Food Config Copy

**Status**: Implemented
**Last updated**: 2026-09-24
**Scope**: Maintenance cleanup; no roadmap feature ID, benchmark, or mutation gate.
Base: `main` at `2a346774`.

## Goal

`world.food.shared` (`FoodResourceConfig`) stops carrying `initial_density`,
`initial_coverage`, `fertility`, and `annealing`. Those values already live in
their canonical homes (`world.food.types[i].initial_*`, `world.food.fertility`,
`world.food.annealing`); the shared copy is only overwritten by normalization
and never read by the simulation. After the change there is one place to
configure each value, in Rust, recipes, the server API, the frontend, and the
reference docs.

## Non-Goals

- Any simulation behavior, RNG stream, or trajectory change.
- Backward compatibility with recipe files carrying the removed keys (AGENTS.md:
  not a default goal). Old files are rejected by `deny_unknown_fields`.
- Editing historical records: `docs/progress/**`, `docs/strategy/**`,
  `docs/archive/**`, and closed specs under `docs/specs/` (including
  `runtime-config-apply-fixes.md`) keep their text.
- The concurrent dead-code deletion in another worktree
  (`kernel/food_resource/{growth,depletion,reconfigure}.rs`,
  `dominant_food_type_at`, `food_type_configs`, `is_increasing`,
  `as_view_rect`, `MutationSkipReason::BudgetExhausted`). These files are not
  compiled (`kernel/food_resource/mod.rs` is a single re-export and declares no
  submodules) and are
  not edited here even where they name the removed fields.
- Fixing the intent of test fixtures whose shared-copy writes were dead (see
  Invariants); they are recorded, not changed.

## Evidence (base `2a346774`)

The copy and its sync:

- `crates/v3-core/src/config/simulation.rs:16-33` `FoodResourceConfig` fields;
  `:35-52` `Default`.
- `:1154-1159` `normalize_food_config` calls `normalize_food_shared` (clamps the
  copy, `:1167`, `:1185-1190`), then `normalize_food_types`, then
  `sync_shared_from_canonical_food` (`:1220-1227`), which overwrites
  `shared.initial_*` from `types[0]` and `shared.fertility/annealing` from the
  top-level fields. Any value a recipe or test writes into the copy is therefore
  replaced before use.
- `:282-293` `FoodConfig::single_type(shared)` reads `shared.initial_*`,
  `shared.fertility`, `shared.annealing` to build `types[0]` and top-level
  fertility/annealing.
- `:296-310` `FoodConfig: Deref/DerefMut<Target = FoodResourceConfig>`: so
  `food.initial_coverage` on a `FoodConfig` resolves to the shared copy, while
  `food.fertility` / `food.annealing` resolve to `FoodConfig`'s own fields
  (a struct field shadows Deref).

Simulation reads use the canonical values only:

- `kernel/ordinary_food/ecology.rs:258-262` seed-time fertility range from
  `config.fertility` / `config.annealing` (`config: &FoodConfig`);
  `:270-278` seeding reads `entry.config.initial_coverage/initial_density`
  (`FoodTypeConfig`); `:390-397` growth reads `full_config.fertility/annealing`.
- `kernel/ordinary_food/mod.rs:165-180` `effective_fertility_grid` reads
  `self.config.fertility/annealing` (`FoodConfig`).
- `ecology.rs:196-217` `apply_config_transition` clones `next.shared` into the
  live shared config but reads only `max_density`, `occupancy_depletion`,
  `grazing`.
- Compiler proof (exploratory, reverted): deleting the four fields and running
  `cargo check --workspace --all-targets` errors in non-test library code only
  inside `config/simulation.rs` (`Default`, `single_type`,
  `normalize_food_shared`, `sync_shared_from_canonical_food`) and one write at
  `neighborhood/recruitment_paths/mod.rs:35` (a write followed by the per-type
  write at `:38-40`). No kernel, sensor, action, runtime, server, or CLI
  library code reads the copy.
- `single_type` is compiled production code only at `kernel/world.rs:95`
  (`WorldState::new` placeholder, replaced by `reconfigure_food` before
  seeding), always with `FoodResourceConfig::default()`. Its output is
  unchanged by this cleanup: `FoodTypeConfig::default()` already has
  `initial_density 1.0`, `initial_coverage 0.54` (`simulation.rs:83-84`), equal
  to the removed shared defaults, and fertility/annealing come from the same
  `Default` impls.

Frontend and API consumers:

- `frontend/src/types/config.ts:39-50` `FoodSharedConfig` declares
  `initial_density/initial_coverage` (it already omits fertility/annealing).
- `frontend/src/stores/startupConfig.ts:78-100` `defaultFoodShared`,
  `:207-210` normalize writes the copy from `types[0]`, `:259-262`
  `fromServerFoodConfig`, `:399-402` `buildStartupFoodRequest`.
  `buildStartupFoodRequest` lists shared fields explicitly, so a stale in-memory
  preset carrying the old keys cannot leak them into a request.
- `frontend/src/test/fixtures.ts:15-16`, `components/ConfigPanel.test.tsx:140`,
  `components/ControlBar.test.tsx:35-36` (typed local `MOCK_CONFIG`).
- Server: the only production reference is
  `v3-server/src/http/status.rs:63-73` `patch_touches_fertility_generation_layers`,
  which also rejects a runtime patch naming `world.food.shared.fertility.layers`.
  With the key gone such a patch is rejected by `deny_unknown_fields` during
  patch deserialization, so the `shared_layers` branch is removed.
  `v3-server/src/query/cache.rs:262`
  (test) builds `FoodResourceConfig { fertility, .. }`;
  `v3-server/tests/server.rs:3640` asserts the patch-rejection reason names
  `world.food.shared.initial_density`. The reason is computed generically by
  diffing submitted vs normalized JSON (`http/config_patch.rs:135-168`); after
  the removal the moved leaf is `world.food.types.0.initial_density`
  (`normalize_food_types`, `simulation.rs:1202-1203`), so the patch is still
  rejected with a different path.

Stored data:

- The seven root `world-recipe-*.json` carry all four keys under
  `world.food.shared` and are compiled into `config/mod.rs:67-112`
  (`root_world_recipes_resolve`).
- `experiments/worlds/*.json` and `crates/*/tests/fixtures/*.json` carry no
  shared copy keys (JSON scan of every tracked `.json`/`.jsonl`).
- 26 stored goal summaries under `docs/progress/features/*-goal.json` embed
  `deterministic.goal_indicators.recruitment_paths.config.world.food.shared.*`
  with the keys; six sweep configs under `docs/progress/sweeps/` and three
  `docs/strategy/*.results.json` do too. Every goal summary is
  `kind: petri-benchmark-summary`; `Summary.deterministic` is
  `serde_json::Value` (`v3-cli/src/bench/artifacts.rs:94`) and only its
  `profile` and `per_creature_tick` sub-objects are typed-parsed
  (`artifacts.rs:555-567`). No test or tool loads the sweep or strategy files.
  Decision: leave them untouched ("ignore"); they stay loadable by every
  current reader. A future typed parse of an embedded historical
  `SimulationConfig` would reject them, which is the correct outcome for a
  removed field.
- Full (raw) benchmark reports are typed-parsed as `Report`
  (`bench/artifacts.rs:460`, `:580`), whose
  `recruitment_paths::Report.config` is a `SimulationConfig`
  (`neighborhood/recruitment_paths/records.rs:1330`). No full report is
  committed (every `docs/progress/features/*.json` carries `kind`); raw reports
  are local, ignored artifacts. Decision: reject. A local raw report written
  before this change no longer loads or converts; it must be regenerated.
  Justification: backward compatibility is not a goal, and a lenient reader
  would need a second config type or an `ignore` escape hatch on a
  `deny_unknown_fields` contract for artifacts nobody commits. The same applies
  to the opt-in `#[ignore]` corpus test
  `v3-cli/tests/bench_artifacts.rs:291-362` (`PETRI_HISTORICAL_MANIFEST`),
  which typed-parses preserved raw bytes; earlier config-key retirements
  (for example T11.F20's `MutationConfig` keys, `8354b042`) already put those
  bytes outside the current reader. Historical bytes stay unmodified; read
  them with the reader of their own revision. `docs/benchmark-artifacts.md:5-6`
  gets one sentence saying the committed summaries, not old full reports, are
  the durable comparison inputs.

## Changes

Rust (`crates/v3-core/src/config/simulation.rs`):

- Remove the four fields from `FoodResourceConfig` and its `Default`.
- `single_type(shared)` becomes `Self { shared, ..Self::default() }`; doc
  updated (types, fertility, annealing come from defaults).
- `normalize_food_shared`: drop the `initial_coverage` / `initial_density`
  clamps. Delete `sync_shared_from_canonical_food` and its call.
- Tests: rewrite sync/shared assertions (`:1385-1392`, `:1545-1553`,
  `:2548-2560`, `:2600-2610`, `:2688-2694`, `:2724-2753`, `:2772-2800`,
  `:2837-2851`) to target `types[i]` / top-level fields, delete tests whose only
  subject is the sync, and add one test that a recipe carrying
  `world.food.shared.initial_density` is rejected as an unknown field.

Test and helper migrations elsewhere (rule below):

- `kernel/world.rs` tests `:328`, `:424-425`; `neighborhood/recruitment_paths/mod.rs:35`;
  `runtime/tests/vm_io_memory.rs:453-454`; `sensors/static_inputs.rs:211-212,230-231`;
  `simulation/actions/predation.rs:214,269`; `simulation/tick/tests/{phase0.rs:33,
  support.rs:69,122,162, actions.rs:1018}`; `v3-core/tests/{applied_trajectory.rs,
  reproducibility.rs, viability.rs, vm_all_opcodes_e2e.rs, terrain.rs, common/mod.rs}`;
  `v3-cli/tests/cli.rs:9-10`; `v3-cli/src/bench/profiles.rs:285` and
  `profiles/tests.rs:42`; `v3-server/src/query/cache.rs:262`
  (`FoodConfig { fertility, ..FoodConfig::default() }`);
  `v3-server/tests/server.rs:1482-1483, 3640`; the full list is whatever
  `cargo check --workspace --all-targets` reports.

Recipes: remove the four keys from `world.food.shared` in the seven root
`world-recipe-*.json`.

Frontend: drop `initial_density/initial_coverage` from `FoodSharedConfig`,
`defaultFoodShared`, `normalizeFoodConfig`, `fromServerFoodConfig`,
`buildStartupFoodRequest`, `test/fixtures.ts`, `ControlBar.test.tsx` mock; move
`ConfigPanel.test.tsx:140` to `types[0].initial_density`.

Server: drop the `shared_layers` half of
`status.rs:patch_touches_fertility_generation_layers`; add a server test that a
runtime patch naming `world.food.shared.initial_density` is rejected (422).

Reference docs:

- `docs/reference/v3-world-grid-spec.md`: delete the two shared-field rows
  (`:120-121`), rewrite the sync note (`:165-166`) and the seeding rule
  (`:253-255`) in terms of `world.food.types[i]`.
- `docs/reference/v3-server-api-protocol-spec.md`: remove the two keys from
  both `shared` examples (`:114-115`, `:441-442`).
- `docs/benchmark-artifacts.md:5-6`: clarify that committed summaries remain
  valid comparison inputs; old full reports may not load after a config-key
  retirement.
- `docs/reference/v3-runtime-config-spec.md`: no edit (it only lists
  `shared.occupancy_depletion.*` and `shared.grazing.*`).

## Invariants

- No simulation behavior, RNG, or trajectory change. Every value the kernel
  reads is unchanged because the kernel never read the copy.
- Migration rule for tests: a write to the shared copy (directly or through
  `DerefMut`) was dead, because nothing reads the copy and normalization
  overwrites it. Such a line is deleted, not redirected to `types[i]`;
  redirecting would change fixture worlds. Where the same site already writes
  `types[i]`, only the shared line goes. The rule covers writes only: a site
  that passes a `FoodResourceConfig` through `single_type` so its copy values
  become live (e.g. `v3-server/src/query/cache.rs:262-281` supplying
  `fertility`) is migrated to the canonical field with the same value, which
  keeps its behavior. Tests whose dead write shows an
  unmet intent (e.g. `initial_coverage = 0.0` in `tests/common/mod.rs`,
  `simulation/tick/tests/support.rs`, `viability.rs`) are listed in the commit
  record as follow-ups, not changed.
- Digest pins: `config_digest` hashes the serialized `SimulationConfig`, so a
  pin may move only because the four keys vanished. Expected: the three
  `experiments/worlds` goal recipe digests in
  `v3-cli/tests/bench_artifacts.rs:186-192`; re-pinned with old→new recorded
  below. The gate profile has `recipe: None`, so its `ProfileBlock.config_digest`
  is `None` (`bench/profiles.rs:308-311`) and the gate series comparison is
  unaffected. If the gate benchmark series test fails, a benchmark epoch would
  need re-pinning, or any trajectory/reproducibility digest moves, work stops
  and is reported instead.

## Verification

1. `cargo check --workspace --all-targets` clean.
2. `cargo test -p v3-core --test viability --test applied_trajectory
   --test reproducibility` pass unchanged (confirms the deleted writes were dead).
3. `make check` once, output to a log; exit 0.
4. `git grep -nE "shared\.(initial_(density|coverage)|fertility|annealing)"`
   finds nothing outside historical docs and the uncompiled
   `kernel/food_resource/` files.

## Digest re-pins

`v3-cli/tests/bench_artifacts.rs::checked_in_goal_recipe_identities_are_unchanged_by_json_precision`:

| Recipe | Old | New |
| --- | --- | --- |
| orchards-in-grassland | `sha256:0a8056385815650cf43da24435aaca43d95dde2829142427eca213602575a890` | `sha256:7f91cd5ccb476cc4013e8dc3fe5f050dcf159f39dce621be55a92fdc0aac47ff` |
| canyon-country | `sha256:4edca49b17a6a16ce8e8cfbc9460c3076b70be99841a4ac1c7af974e71133cbc` | `sha256:88ef5abaec2d99ba5934e517dcdd045344b73a4c73149f90133f8fb4f692e7b4` |
| confluence | `sha256:09398394d4769af786833bc7e20c530b6511d3a1b07b5ce6aa6c50a345b341f6` | `sha256:48722f834746b54ff6964afdb406dc69e321fa2946ebf4571d6977a609bba9b2` |

Cause, checked: the new resolved configs were dumped as canonical JSON (the
`config_digest` input); re-adding `shared.initial_density/initial_coverage`
from `types[0]` and `shared.fertility/annealing` from the top-level fields
reproduces all three old digests byte-for-byte, so the only difference is the
four removed keys. No other pin moved. Future goal and recruitment runs will
report per-case `inputs_changed` against stored summaries for the same reason;
that is the comparison contract's expected response to a config shape change
(`bench/comparison.rs:600-608`), not a trajectory change.

## Follow-ups (not changed here)

Dead shared-copy writes deleted under the Invariants rule whose values differ
from the fixture's effective per-type values, so the fixture never had the
food it names: `kernel/world.rs` `seed_food_skips_barrier_cells` (1.0),
`runtime/tests/vm_io_memory.rs` (1.0), `simulation/actions/predation.rs`
two tests (0.0), `simulation/tick/tests/{phase0.rs, support.rs ×3}` (0.0),
`v3-core/tests/applied_trajectory.rs` ×2 (1.0), `tests/common/mod.rs` (0.0),
`tests/reproducibility.rs` (1.0), `tests/viability.rs` (0.0 ×5, 1.0 ×2, 0.8),
`tests/terrain.rs` (0.0), `tests/vm_all_opcodes_e2e.rs` (0.0),
`v3-cli/tests/cli.rs` (0.6), `v3-server/tests/server.rs` (0.8). Redirecting
them to `types[i]` would change those fixtures' worlds and is a separate
decision.

## Review

| Round | Codex job | Verdict |
| --- | --- | --- |
| PRD 1 | `task-mufxycp5-gv06qp` | not-ready: missed `ControlBar.test.tsx` mock (fixed); advisories on `status.rs` branch, `cache.rs` live fertility, raw-report typed parse (all applied) |
| PRD 2 | `task-mufy3gsy-q4b0aj` | ready; advisory on `docs/benchmark-artifacts.md` and the opt-in historical corpus test (applied) |
