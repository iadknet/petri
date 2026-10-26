# T30.F04 — Config and World Without Cycles

**Status**: Complete
**Last updated**: 2026-10-26
**Feature**: T30.F04
**Track**: [T30 — Core Boundaries](../../roadmaps/t30-core-boundaries.md)

## Goal

Worlds are configured, painted and stocked with food exactly as before, by
layers that build on each other in one direction: the pattern parameter
types are config data, so `config` imports nothing above itself, and the
food stream's hash is a `contracts` function, so the world (`kernel`)
imports nothing from the creature record. Debts G10 and G11 are paid and
deleted. Every trajectory pin, map digest, pattern output and food-growth
draw is identical to main's; any pin that moves is a defect.

## Non-Goals

- No change to any pattern algorithm, to `generate_pattern`'s draws, to the
  food stream's words or draws, to kin-tag derivation, or to any default.
- No wire or file change: the `pattern_type`-tagged JSON of `PatternParams`,
  the `TerrainLayer` fields, the world files under `experiments/worlds/`,
  the pattern preview and apply endpoints, and the benchmark deterministic
  blocks are byte-identical.
- No frontend change beyond two `WORKAROUND` comments:
  `frontend/src/types/pattern.ts` (`PatternParams`, `PatternBounds`) and
  `frontend/src/types/config.ts:131` (`TerrainLayer`) are hand copies of
  config-schema shapes that section 2 forbids and no debt row names; they
  are marked here and their payer is proposed to the user at closure
  (audit R5), not taken on.
- No other `config/simulation.rs` split: only the terrain schema moves out.
- No edit to `kernel::paint` or to paint's wire types (T30.F01).

## Inputs and Invariants

Inputs: the T30 row and its Notes entry for F04 ("makes pattern parameter
types config data and moves `splitmix64` to contracts"), the track
[design review](../../progress/readings/t30-design-review.md) finding 5,
`docs/architecture.md` sections 1, 3 and 5 (G10, G11),
`scripts/architecture-debts.txt`, and the code on `44e53179`:
`config/simulation.rs:356-362` (`TerrainLayer`), `patterns/mod.rs`
(`PatternBounds`, `PatternParams`, `normalize`, `generate_pattern`),
`kernel/ordinary_food/stream.rs:10`, `creature/identity.rs:49-56`
(`splitmix64`), `simulation/seeding.rs:38-56`,
`v3-server/src/http/pattern.rs:8,42-80`, and
`crates/v3-telemetry/src/windows.rs:145-151`. The options weighed and the
preparatory audit are in the
[readings](../../progress/readings/t30-f04.md).

Invariants:

1. `PatternBounds`, `PatternParams` (with `normalize`) and `TerrainLayer`
   live in `config/terrain.rs` and are exported from `crate::config`; they
   have one home. `patterns` imports them from `config` and re-exports
   nothing; every caller outside `patterns` names `v3_core::config`.
2. `config` imports only `contracts`; `kernel` imports only `contracts`
   and `config`; `patterns` imports `config` and `kernel`.
   `scripts/architecture-check.mjs` passes with no G10 or G11 entry in
   `scripts/architecture-debts.txt` and no G10 or G11 row in
   `docs/architecture.md`.
3. `splitmix64` is `pub fn contracts::splitmix64`, the one definition in
   the workspace: `creature::identity`, `kernel::ordinary_food::stream` and
   `v3-telemetry`'s window selection call it. Its output for every input is
   today's: `FoodStreamKey`, every `FoodStream` word and draw, every founder
   `kin_tag`, and every telemetry window selection are unchanged.
4. `PatternBounds::clipped_to(width, height)` is the one world intersection
   of a bounds value: width and height each clipped to the world minus the
   origin, saturating, origin unchanged. `seed_simulation` and the server's
   pattern validation call it; the server's clamp of `x` and `y` to the last
   cell and its zero-area rejection stay in front of the call, so each site
   returns exactly what it returns today.
5. The serde form of every moved type is unchanged: the `pattern_type` tag,
   variant and field names, `deny_unknown_fields` on `TerrainLayer`, and
   `SimulationConfig::normalize`'s call order.
6. No behavior change, read on both moved paths. Terrain: a pinned seed-time
   map digest per recipe in `RECIPES` (the four worlds
   `crates/v3-core/tests/baseline_worlds.rs` loads: barrier and fertility
   cells of the map seeded by its `seed_baseline_map`), whose values are computed on
   the untouched code at `44e53179` before any move and hold after it
   (Canyon's and Confluence's layers exercise `FbmThreshold`, `Noise` and
   `ParallelLines`; `Maze`, `Spiral` and `Star` change only their import
   line and keep `tests/terrain.rs`'s structural tests). Food stream and
   kin tags: the gate's deterministic blocks, which run the default world,
   and the food-stream and identity unit tests. No existing expected value
   is edited.

## Architecture

Owners touched: `config` (gains the terrain schema it is named owner of),
`patterns` (keeps the generation algorithms, now reading config types),
`contracts` (gains the deterministic hash it is named home of), `kernel`
(the food stream reads the contracts hash), the creature record (kin tags
read the contracts hash), `simulation` (seeding calls the bounds
operation), and outside core `v3-server`'s pattern endpoint and
`v3-telemetry`'s window selection as callers. Debts paid: G10 (the
config→patterns→kernel cycle) and G11 (the kernel's creature hash import);
their rows and debt lines are deleted at closure. No boundary changes and
`docs/architecture.md`'s desired state is not edited: section 3 already
places config below patterns and kernel, and section 3's table already
assigns pattern parameters to config and deterministic hashing to
contracts. Shared rules this feature touches: the terrain-layer
normalization rule, whose owner is config and whose existing paths are
`SimulationConfig::normalize` (`config/simulation.rs:1134`) and
`generate_pattern` (`patterns/mod.rs:116`); both keep calling the one
`PatternParams::normalize`, now a config method. The bounds intersection
rule, applied today by `seed_simulation` and the server's
`validate_request`, becomes the owner operation of invariant 4. The
`PatternParams` and `PatternBounds` values stay on the pattern endpoints'
wire under section 2's config-schema exemption.

## Telemetry

Not applicable: the feature moves types and a pure function between
modules and adds no counter, config value, mechanism switch or work to a
run; `run.started`'s config body is byte-identical.

## Verification Plan

**Question.** F04 claims the terrain schema, the bounds intersection and
the food-stream hash can change owner with no change to any world, draw or
cost; a moved trajectory pin, map digest or deterministic block, or a
changed food-stream or pattern test value, would show it wrong.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate` | The moved hash sits under every food-growth draw of every tick, with no population reading in question; the gate's deterministic blocks against the epoch are the byte-identity reading of the food stream, in seconds, and the terrain reading is the map-digest test of invariant 6, so no goal run is needed. |
| Slow tests (`make rust-test-slow`) | `not run` | No mutation operator, catalog, recruitment or lab code changes. |

## Implementation Tasks

- [x] First, on the untouched code: the map-digest test of invariant 6 in
      `crates/v3-core/tests/baseline_worlds.rs`, one pinned `u64` per
      `RECIPES` entry, its values computed at `44e53179` and recorded in
      the readings; it passes before any move and after every task below.
- [x] `config/terrain.rs`: move `PatternBounds`, `PatternParams` and
      `normalize` from `patterns/mod.rs` and `TerrainLayer` from
      `config/simulation.rs`, with their doc comments (`patterns`'s
      generation tests stay); export the three from `config/mod.rs`; `patterns`
      imports them from `crate::config`; `simulation/seeding.rs`,
      `v3-server/src/http/pattern.rs` and the `terrain`, `baseline_worlds`
      and `reproducibility` integration tests import from `v3_core::config`
      (invariants 1, 5).
- [x] `PatternBounds::clipped_to(width, height)` with a proptest property
      test (the result lies inside the world and inside the input, origin
      unchanged); `seed_simulation` and `validate_request` call it in place
      of their inline arithmetic (invariant 4, audit R3), with one
      `validate_request` fixture pinning the origin clamp: in a 40×40 world,
      bounds `(50, 0, 10, 10)` return `(39, 0, 1, 10)`.
- [x] `contracts/hash.rs`: `pub fn splitmix64` with its determinism test,
      exported from `contracts`; `creature/identity.rs`,
      `kernel/ordinary_food/stream.rs` and `crates/v3-telemetry/src/windows.rs`
      import it and the two local definitions are deleted (invariant 3,
      audit R2).
- [x] Delete the G10 and G11 lines from `scripts/architecture-debts.txt`
      and the G10 and G11 rows from `docs/architecture.md` section 5
      (invariant 2).
- [x] `WORKAROUND(T30.F04): generated from or tested against the server's
      config schema` comments on the hand-copied types in
      `frontend/src/types/pattern.ts` and `config.ts` (audit R5).
- [x] `make check`; the gate profile.

## Verification

- [x] Architecture: `node scripts/architecture-check.mjs` -> exit 0 with no
      G10 or G11 entry; `grep -rn "fn splitmix64" crates/` -> one definition
      (`contracts/hash.rs`); the pattern generators import `PatternBounds`
      from `crate::config`.
- [x] Map digests: pinned at `44e53179` in commit `59832f7b` before any move
      (values in the readings); `cargo test -p v3-core --test
      baseline_worlds` -> 20 passed after the move; the digests are computed
      over `RECIPES` and `PINNED` is sized by `RECIPES.len()`, so a recipe
      without a pinned entry fails.
- [x] Focused tests: `cargo test -p v3-core --test terrain --test
      reproducibility` -> 14 passed; `cargo test -p v3-core
      ordinary_food::stream` -> 4 passed; `cargo test -p v3-core
      config::terrain` -> 1 passed (the `clipped_to` property); `cargo test
      -p v3-server pattern` -> 9 passed (the origin-clamp fixture among
      them); `cargo test -p v3-telemetry` -> 80 passed; no existing expected
      value edited.
- [x] Crate suites: `cargo test -p v3-core` -> 2400 passed; `cargo test -p
      v3-server -p v3-telemetry` -> 249 passed; `cargo check --workspace
      --all-targets`, `cargo fmt --all --check`, `make rust-clippy` -> exit 0.
- [x] Frontend comments: `npx biome check src/types/pattern.ts
      src/types/config.ts` (in `frontend/`) -> no diagnostics.
- [x] `make check` -> exit 0 at tested commit `af5e3ca5`.
- [x] Gate profile: summary at `docs/progress/features/t30-f04.json`, local
      raw hash and byte count and verification time checked, series entry
      points to the summary, deterministic blocks identical to the epoch,
      no new full report staged.

## Performance and Goal Impact

**Predeclaration — written before the run.** A code move with no new work:
the hash is the same four instructions inlined from a different module, the
bounds operation is the same arithmetic, and no draw, value or default
changes. Expected compute cost: none; the gate's compute reading is compared
against the current epoch with the standard thresholds and no re-pin. Every
deterministic block is expected identical to the epoch; a differing block
is a defect, not a cost to accept. No indicator can move.

**Measured verdict.** Matches the predeclaration. `make bench PROFILE=gate
FEATURE=t30-f04` at `c4874ee1`: outer exit 0, CLI exit 0 (v3-cli status),
`severe=false`; all nine work counters `ok` at 0.000000% against the epoch
(`t27-f03-giving-energy.json`); the `deterministic` block is identical to the
epoch. Raw 132,624 bytes (SHA-256 `5f645975...1ab77`, local), summary 27,475
bytes; readings in the file below.

- Summary of the declared profile:
  [gate](../../progress/features/t30-f04.json).
- Full readings: [`docs/progress/readings/t30-f04.md`](../../progress/readings/t30-f04.md).

## Success Criteria

- [x] `config` imports only `contracts`, `kernel` imports nothing from
      `creature`, and `make check-docs` passes with G10 and G11 deleted from
      both `docs/architecture.md` and `scripts/architecture-debts.txt`.
- [x] One `splitmix64` in the workspace, in `contracts`, called by the kin
      tag, the food stream and telemetry's window selection, with every
      output unchanged.
- [x] `PatternBounds`, `PatternParams` and `TerrainLayer` have one home in
      `config`; the bounds intersection has one owner operation.
- [x] The gate's deterministic blocks, the pinned map digest of every
      `RECIPES` world and every existing test value are unchanged.

## Notes for AI Agents

- Decision: audit R1–R4 (readings) are in-feature: the terrain schema moves
  into `config/terrain.rs` with no re-export from `patterns`, `splitmix64`
  moves to `contracts` and `v3-telemetry`'s copy is deleted, and the bounds
  intersection becomes `PatternBounds::clipped_to`.
- Deferred: unpaid accept, `frontend/src/types/pattern.ts` and
  `frontend/src/types/config.ts:131` hand-copy the config-schema types
  `PatternParams`, `PatternBounds` and `TerrainLayer` (audit R5); payer
  proposed to the user at closure (T30.F01's generator extended to the
  config schema, or a new T30 row).
- Deferred: final review (three Codex Sol high lenses on 63cdc027):
  correctness and design raised no findings; code health raised two P3s —
  the PINNED/RECIPES sync (fixed in this pass) and the frontend schema-copy
  note having no named payer, which stays the workflow's unpaid-accept form
  with the payer proposed to the user at closure.
- Cost: `/usage` totals not collected (unattended goal run); implementer passes 3 (build, simplify, review remediation) with advisor consults 2, 2, 1; spec-owner resumes after Plan 0; Codex challenge rounds 2, final verdict `ready`; final review findings: correctness 0, design 0, code health 2 P3.
