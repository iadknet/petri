# T12.F04 — Baseline World Set

**Status**: In Progress
**Last updated**: 2026-09-09
**Feature**: T12.F04
**Track**: [T12 — World Composition and Baseline Worlds](../../roadmaps/t12-world-composition-and-baseline-worlds.md)

## Goal

Three saved, fixed-map baseline worlds replace the goal profile's three
plains seed replicates: a food-differentiation world, a barrier-topology
world, and a composite. Each runs once per closure, under ten minutes for
the whole profile, and every closure from now on records the same per-world
readings so a world's trajectory can be followed as later features land.

This is the second pass at the feature. The first pass (Codex, 2026-09-09,
commits `398a21a9`..`73aaa379` on this branch) built the substrate and the
profile plumbing but shipped three unbalanced recipes and no way to follow a
world across closures; this pass reviews that work, redesigns the recipes,
and adds the tracking.

## Non-Goals

- No change to creature economics, founders, mutation, or the short gate.
- No claim that food specialization or barrier awareness evolves; the set
  only makes those pressures present and measurable.
- No seasons, disturbance, manual overlays (T12.F05), or new recipe format.
- No repair of the substrate facts recorded under Inputs; they belong to
  T11 and T03.F04.

## Inputs and Invariants

**Dependencies.** [T12.F01](t12-f01-seeded-terrain-in-the-world-config.md)
(seeded terrain layers), [T12.F02](t12-f02-world-recipe-save-and-load.md)
(partial-config recipes, digests). The
[research note](../../strategy/world-seeding-research-2026-09-08.md) holds
the archetype evidence. The track roadmap and the workflow's
[standard-baseline contract](../../workflow.md#environmental-pressures-in-the-standard-baseline)
fix the protocol: Orchards/11, Canyon/22, Confluence/33, one 2,000-tick run
each at 1600² with 10,000 production founders, `goal-worlds-v1` series.

**Already implemented by the first pass, kept after review.**
Per-type `energy_per_unit`, `growth_rate`, `recovery_spawn_rate` (null
inherits the shared value) and `initial_fertility_only` on
`world.food.types[]`; the `FbmThreshold` terrain variant; tick-zero
`passable_connectivity`; per-case goal profile execution with per-case
neighborhood, drift, and structure observations; frontend and reference-spec
carry-through. Their tests live in `crates/v3-core/tests/baseline_worlds.rs`
and the `v3-cli` bench tests.

**Substrate facts that shape the design.**

- Founders eat only food type 0: `decode_food_type_idx` rounds the Eat
  parameter, the founder emits 0, and a world with only type-1 food is extinct
  by tick 50 (probe, 2026-09-09). Codex's diagnostic found 0 of 1,000 mutant
  births selecting another type. A second food type is therefore a latent
  niche, not a choice the living population makes; the typed-eat counter added
  here reads whether that ever changes.
- Eaten cells hold zero density and regrow only by spread from a neighbor at
  or above 80% of max density or by recovery spawns below the 1% floor, so
  each type's growth rate and fertility set how fast grazed ground returns.
- Population booms to the 100,000 cap within about 50 ticks in every world;
  the boom's length is set by the tick-zero standing crop
  (coverage × density × energy per unit), and the plateau by regrowth reachable
  by creatures. Runtime is dominated by creature-ticks in the boom plus a
  fixed food-update cost of roughly 40 ms per tick per food type at 1600².
- A blocked move still pays `move_cost` and the tick; the failed-action
  penalty is only 3% ramped at tick 2,000. Barrier pressure comes from
  frequent blocked moves in a looped landscape, not from a spanning-tree maze
  (Codex's maze collapsed the population to 7).
- Drift depth reads only the mutation config and the food-type count, never
  the world. Its depth-2,000 changed/all-birth reading is 0.005 with one food
  type (byte-identical to T11.F18's accepted exception) and 0.006 with two,
  both below the T11.F17 track floor of 0.008. T12.F04 cannot move it.

**Design decisions (recorded assumptions).**

- Maps are fixed: each recipe pins `world.world_seed`, so terrain and
  fertility are identical across run seeds and closures; run seeds still vary
  food placement, founder placement, and the run RNG. Resolves the research
  note's open Section 7 decision in favor of "saved baseline maps".
- Recipes stay partial configs of environmental differences only; the goal
  profile applies its own size and founder count, and a test asserts the
  checked-in recipes set neither.
- **Orchards in grassland** (food differentiation): type 0 Grass is the
  founders' food, diffuse across most of the world at low density, lower
  energy per unit, very slow regrowth; type 1 Fruit is rich, patchy
  (fertility blobs, `initial_fertility_only`), fast-regrowing inside its
  patches, and edible only by a lineage that changes its Eat parameter.
- **Canyon country** (barrier topology): thresholded fBm mesas for large-scale
  winding routes, a rubble field for frequent local blocks, and ridge lines
  in bounded sub-regions; well looped, largest component holding nearly all
  passable cells; production one-food substrate.
- **Confluence** (composite): Bounded edges; overlapping regions that each
  carry one pressure at small scale (an orchard belt, a canyon block, fBm
  islands with straits, a sparse-fertility basin) with both foods, so borders
  between pressure combinations exist.
- Balance targets, measured before storing: no extinction, no case pinned at
  the population cap past the boom, plateau populations in the low thousands
  to low tens of thousands, and the complete `make bench PROFILE=goal` under
  600 s on the recording host (target 480 s).

## Implementation Tasks

- [x] Food substrate, `FbmThreshold`, connectivity, per-case goal profile,
      frontend and reference carry-through (first pass; reviewed here).
- [ ] `v3-cli world inspect --config <recipe> --seed <n> [--png <path>]`:
      prints connectivity and per-type habitat, food-cell, and standing-crop
      readings as JSON and writes a downsampled PNG preview (barriers,
      per-type fertility, tick-zero food). Replaces the ignored
      `inspect_saved_world_layouts` test.
- [ ] Redesign the three recipes under `experiments/worlds/` with pinned
      `world_seed`, regenerate `previews/*.png` with the command above, and
      rewrite `experiments/worlds/README.md` (design intent, applied
      readings, how to inspect and run).
- [ ] Per-world tracking in the report: persistence samples and each
      `GoalCaseObservation` carry cumulative typed-eat counts per type (from
      applied successful Eat actions), per-type standing food density (from
      the applied growth summary), move attempts, moves blocked by barrier,
      and avoidable blocked moves by barrier-reader state.
- [ ] World-set comparisons: profile identity for `goal-worlds-v1` ignores
      per-case digests; each reference comparison gains a per-case block (by
      case name) with `inputs_changed` and both digests, deltas for
      persistence, per-case work counters, lineage, memory, drift,
      neighborhood, structure, typed eats, and blocked moves. A digest change
      is labeled, never an error, and never a claim of unchanged inputs.
      Missing cases in a reference are recorded absent.
- [ ] Dashboard: the Goal worlds tab shows, per world, series over closure
      order for the readings above, population-trajectory overlays by
      closure, and markers where inputs changed; `docs/progress.md` gains a
      world-set table; `docs/reference/v3-cli-contract-spec.md` records the
      command, the sample fields, and the comparison rule.
- [ ] Store gate and goal reports, list the goal report in the
      `goal_worlds` series, and record the readings below.

## Verification

- [ ] `world inspect` tests: JSON readings match `passable_connectivity` and
      direct grid counts on a small recipe; PNG decodes to the downsampled
      size; a missing recipe or seed is a validation error.
- [ ] Recipe tests: all four load; the three goal recipes pin `world_seed`
      and set neither size nor population; Orchards' fruit is strictly richer,
      patchier, and faster-regrowing than grass and grass has positive
      fertility on every passable cell; Canyon's largest component holds at
      least 95% of passable cells and at least 15% of cells are barriers;
      Confluence has both foods, Bounded edges, and terrain.
- [ ] Telemetry tests (TDD): typed-eat counts rise only on successful typed
      eats; standing density equals the growth summary; blocked-move fields
      equal the stats maps; samples carry the fields at the 100-tick cadence;
      property test that per-case counters sum to the profile totals.
- [ ] Comparison tests: a reference with changed case digests compares with
      `inputs_changed: true` and correct deltas; a reference with a different
      case list records the absent case; gate and single-config behavior are
      unchanged (existing tests).
- [ ] Dashboard checked in a browser against the stored report; `make
      check` (records the tested commit) and `make check-docs` at closure.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants` after the simplify pass:
      summary line, output path, and every survivor resolved.
- [ ] Reports: `docs/progress/features/t12-f04-baseline-world-set.json`
      (gate) and `...-goal.json` (goal, replacing the first pass's unbalanced
      reading, which is preserved as
      `...-goal-first-pass.json` and stays out of the series index).
- [x] Second goal run: not applicable (workflow decision 2026-09-05).

## Performance and Goal Impact

Natural analogs: food profitability against return rate (orchard fruit in
grass), landscape barriers (canyon country), and a patchy composite habitat;
all reach creatures through what they can eat, where they can walk, and what
their existing sensors report.

Predeclared cost: the gate profile is unchanged and must read within
threshold against T11.F18 and the pinned epoch. The goal profile is a new
series; its budget is the 600 s wall-clock bound above, and the new
telemetry adds one counter increment per applied eat and per blocked move
plus a per-type density read already produced by the growth summary.

Readings to record here at closure, per case: final, minimum, peak, and
plateau population; births; mean energy; passable fraction and largest
component; typed eats per type; blocked-move fraction and avoidable fraction
by reader state; lineage entropy and clades; memory sensitivity; drift
changed/all at 1,000 and 2,000 against the 0.0015 and 0.008 floors; founder
and evolved neighborhood fractions; structure median; wall-clock per case and
for the whole command.

## Success Criteria

- [ ] `make bench PROFILE=goal` runs Orchards, Canyon, and Confluence once
      each from fixed maps, all three persist to tick 2,000 without pinning
      at the cap, and the command completes under 600 s on the recording host.
- [ ] A later closure's goal report can be compared per world against this
      one, with changed recipe inputs labeled, and the dashboard shows each
      world's readings over closure order.
- [ ] Reviewed diff, mutation record, stored reports, and closure checks
      pass; the drift-floor reading is recorded with the user's decision.

## Notes for AI Agents

- Second pass, 2026-09-09: Fable 5.1 orchestrator took over the Codex
  branch (`codex/t12-f04`, rebased onto `main` at `c95457d9` in
  `.claude/worktrees/t12-f04`, branch `worktree-t12-f04`). Recipe design and
  balancing were done by the orchestrator (config data, tuned by screening
  runs), a recorded deviation from "write no feature code yourself"; all Rust,
  frontend, and dashboard code went through the implementer.
- First-pass review findings (Fable, 2026-09-09): (1) Orchards and
  Confluence used grass fertility blobs covering nearly the whole world, so
  regrowth held the population at the 100,000 cap for 1,400 ticks and the run
  took 1,047 s; (2) Canyon was a whole-world perfect maze (a spanning tree)
  and collapsed to 7 creatures; (3) `compare_against_path` errors on any
  profile difference and `bench` exits before writing the report, so the
  first recipe edit required by the standard-baseline contract would discard
  a ten-minute run; (4) maps derived from the run seed although the goal asks
  for saved baseline maps; (5) the goal profile silently overrides a recipe's
  size and founders; (6) the first report lacked per-case structure
  distributions (already remediated); (7) previews came from an ignored test
  writing to `/private/tmp`. Items 1, 2, 4, 5, 7 are fixed by the tasks above;
  3 by the comparison task; 6 stays fixed.
- First-pass verification worth keeping: viability gate ran first after the
  food edits (24 tests); the pre-feature short-run fingerprint
  `13138541837675773035` is pinned by `legacy_default_short_run_identity`;
  the gate report at `5f87c8f4` read every deterministic field equal to
  T11.F18; three fresh mutation passes ended at `112 mutants tested: 84
  caught, 26 unviable, 2 timeouts`, the two timeouts being the visited-guard
  mutations in `WorldState::passable_connectivity` (`world.rs:57`), deferred
  as P2 because the mutated traversal never terminates. Its first goal report
  is kept as `t12-f04-baseline-world-set-goal-first-pass.json` for the
  record and is not a series member.
- Drift floor: the depth-2,000 changed/all-birth floor is a T11.F17 track
  floor (0.008, strict not-below). The reading here is the substrate's, not
  the worlds': 10/2,000 with one food type (identical to T11.F18, closed on
  the user's written exception on 2026-09-08) and 12/2,000 with two. Closure
  needs the user's decision to extend that exception or to hold this feature
  open until a T11 repair lands; the orchestrator does not grant it.
- Deferred P2 (mutation): the two `passable_connectivity` visited-guard
  mutants time out rather than fail; killing them needs a watchdog test,
  which is not worth adding for score.
