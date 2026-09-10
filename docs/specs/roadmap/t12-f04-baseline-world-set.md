# T12.F04 — Baseline World Set

**Status**: Complete
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
  (Codex's maze collapsed the population to 7). Founders are barrier-blind
  (no barrier input is wired), and the screening bisection showed the
  interaction that matters: poor grass (1.6 energy per bite) persists on open
  ground, production grass persists among barriers, poor grass among barriers
  collapses; the edge rule made no difference.
- Small fertility patches are grazed to nothing and never return; large ones
  are inexhaustible. The production 5–15-cell blobs cannot hold a population
  once half of them sit under rock (both fBm canyon drafts collapsed to single
  digits on the production fertility), while sixty 40–90-cell meadows hold a
  plateau in the low thousands to low tens of thousands.
- Codex's `FbmThreshold` samples the noise field in bounds-local
  coordinates, so two layers with different bounds read different parts of
  the same field; nested same-seed layers can only grade a region's edge when
  the field is sampled at world coordinates.
- Drift depth reads only the mutation config and the food-type count, never
  the world. Its depth-2,000 changed/all-birth reading is 0.005 with one food
  type (byte-identical to T11.F18's accepted exception) and 0.006 with two,
  both below the 0.008 floor T11.F17 set; T12.F04 cannot move the reading.
  At closure the user re-based the standing floor to 0.005 (see Notes).

**Design decisions (recorded assumptions).**

- Maps are fixed: each recipe pins `world.world_seed`, so terrain and
  fertility are identical across run seeds and closures; run seeds still vary
  food placement, founder placement, and the run RNG. Resolves the research
  note's open Section 7 decision in favor of "saved baseline maps".
- Recipes stay partial configs of environmental differences only; the goal
  profile applies its own size and founder count, and a test asserts the
  checked-in recipes set neither.
- **Orchards in grassland** (food differentiation, `world_seed` 1104): type 0
  Grass is the founders' food, diffuse across 65% of the world at density
  0.4, 4 energy per unit, growth 0.05 on a background fertility of 0.15 (very
  slow) except in sixty meadows of radius 40–90 (fertility up to 2.0); type 1
  Fruit is rich (15 per unit), placed only in twenty-four orchards of radius
  50–110 (`initial_fertility_only`), fast-regrowing there, and edible only by
  a lineage that changes its Eat parameter. A literal diffuse-only grass went
  extinct by tick 400 in screening; the reversed assignment (founders on the
  rich patches) sat at the 100,000 cap for 1,000 ticks.
- **Canyon country** (barrier topology, `world_seed` 2204): one whole-world
  thresholded fBm field (frequency 0.005, threshold 0.02: about 47% barrier,
  98% of passable cells in one component) plus a 3% rubble field; production
  one-food defaults except fertility, which is forty-five valley meadows of
  radius 30–70 over a 0.1 background.
- **Confluence** (composite, `world_seed` 3304): Bounded edges; a canyon
  massif in the north-east (one fBm field at four nested thresholds stepping
  from 0.02 to 0.32 so rock density fades), an archipelago field in the
  south-west (0.0 to 0.3), a jagged ridge line system, a boulder field, both
  foods from Orchards with grass at density 0.5 and 5 per unit (the poor
  grass does not persist among barriers), sixty meadows, eighteen orchards,
  and a low-frequency fertility gradient; about 77% passable, 99.5% in one
  component. Regions overlap rather than tile.
- Balance targets, measured before storing: no extinction, no case pinned at
  the population cap past the boom, plateau populations in the low thousands
  to low tens of thousands, and the complete `make bench PROFILE=goal` under
  600 s on the recording host (target 480 s).

## Implementation Tasks

- [x] Food substrate, `FbmThreshold`, connectivity, per-case goal profile,
      frontend and reference carry-through (first pass; reviewed here).
- [x] `v3-cli world inspect --config <recipe> --seed <n> [--png <path>]`:
      prints connectivity and per-type habitat, food-cell, and standing-crop
      readings as JSON and writes a downsampled PNG preview (barriers,
      per-type fertility, tick-zero food). Replaces the ignored
      `inspect_saved_world_layouts` test.
- [x] `FbmThreshold` samples the noise field at world coordinates
      (`bounds.x + x`, `bounds.y + y`) instead of bounds-local ones, so a
      bounded layer equals the matching sub-rectangle of the whole-world
      layer at the same seed; the runtime pattern endpoint follows.
- [x] Redesign the three recipes under `experiments/worlds/` with pinned
      `world_seed`, regenerate `previews/*.png` with the command above, and
      rewrite `experiments/worlds/README.md` (design intent, applied
      readings, how to inspect and run).
- [x] Per-world tracking in the report: persistence samples and each
      `GoalCaseObservation` carry cumulative typed-eat counts per type (from
      applied successful Eat actions), per-type standing food density (from
      the applied growth summary), move attempts, moves blocked by barrier,
      and avoidable blocked moves by barrier-reader state.
- [x] World-set comparisons: profile identity for `goal-worlds-v1` ignores
      per-case digests; each reference comparison gains a per-case block (by
      case name) with `inputs_changed` and both digests, deltas for
      persistence, per-case work counters, lineage, memory, drift,
      neighborhood, structure, typed eats, and blocked moves. A digest change
      is labeled, never an error, and never a claim of unchanged inputs.
      Missing cases in a reference are recorded absent.
- [x] Dashboard: the Goal worlds tab shows, per world, series over closure
      order for the readings above, population-trajectory overlays by
      closure, and markers where inputs changed; `docs/progress.md` gains a
      world-set table; `docs/reference/v3-cli-contract-spec.md` records the
      command, the sample fields, and the comparison rule.
- [x] Store gate and goal reports, list the goal report in the
      `goal_worlds` series, and record the readings below.

## Verification

- [x] `world inspect` tests: JSON readings match `passable_connectivity` and
      direct grid counts on a small recipe; PNG decodes to the downsampled
      size; a missing recipe or seed is a validation error.
- [x] Recipe tests: all four load; the three goal recipes pin `world_seed`
      and set neither size nor population; Orchards' fruit is strictly richer,
      patchier, and faster-regrowing than grass and grass has positive
      fertility on every passable cell; Canyon's largest component holds at
      least 95% of passable cells and at least 15% of cells are barriers;
      Confluence has both foods, Bounded edges, and terrain.
- [x] Telemetry tests (TDD): typed-eat counts rise only on successful typed
      eats; standing density equals the growth summary; blocked-move fields
      equal the stats maps; samples carry the fields at the 100-tick cadence;
      property test that per-case counters sum to the profile totals.
- [x] Comparison tests: a reference with changed case digests compares with
      `inputs_changed: true` and correct deltas; a reference with a different
      case list records the absent case; gate and single-config behavior are
      unchanged (existing tests).
- [x] Dashboard checked in a browser against the stored report (orchestrator,
      2026-09-09, after storing the second-pass goal report: the Goal worlds
      tab renders every per-world chart for all three cases with no console
      errors; the implementer's earlier check ran against the first-pass
      report and its placeholders).
- [x] `make check` on the final feature code: exit 0 at `9287048c` (the
      tested commit; every later commit on the branch is documentation and
      stored reports); `make check-docs` exit 0 on the closure documents.
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants` after the simplify pass:
      summary line, output path, and every survivor resolved.
- [x] Reports: `docs/progress/features/t12-f04-baseline-world-set.json`
      (gate) and `...-goal.json` (goal, replacing the first pass's unbalanced
      reading, which is preserved as
      `...-goal-first-pass.json` and stays out of the series index).
- [x] Second goal run: not applicable (workflow decision 2026-09-05).

### Mutation survivor record (final, fold-in pass)

Earlier passes' mutation runs are in the readings file.

**Mutation record (fold-in pass).** One fresh `MUTANTS_ITERATE=0
make rust-mutants`, diffing against the merge base `c95457d9` so the whole
feature diff was mutated (85 of the caught mutants are in `bench.rs`):

```
333 mutants tested in 21m: 262 caught, 69 unviable, 2 timeouts
```

Output path: `~/.local/share/petri-tools/mutants/t12-f04/mutants.out`
(`run-mode.txt` records `fresh`). `missed.txt` is empty — nothing was missed by
every test. Survivor list, complete; both are the same pre-existing timeouts
every earlier pass deferred:

| Survivor | Resolution |
| --- | --- |
| `crates/v3-core/src/kernel/world.rs:57:48: replace && with \|\| in WorldState::passable_connectivity` | **deferred** — timed out at 120 s rather than failing; the mutated visited guard never terminates. See "Notes for AI Agents". |
| `crates/v3-core/src/kernel/world.rs:57:32: delete ! in WorldState::passable_connectivity` | **deferred** — same guard, same reason. |

Caught is the same 262 as the remediation pass's run and the mutant count rose
by eight, all unviable: cargo-mutants' only mutation of this pass's one new
function replaces `by_cause_readings`'s whole return value with a repeat
expression over a non-`Copy` `(String, Option<f64>)` tuple, which does not
compile. Its behavior is pinned by
`case_readings_follow_the_case_seed_and_observation` — three distinct multiples
of the seed, each reading asserted by name, plus the `None` case — rather than
by mutation. Every mutant of `observe` and of the fields it fills, including
`replace WorldTracking::observe -> Self with Default::default()`, is caught.

No production code was edited to kill a mutant, and no `#[mutants::skip]` or
`exclude_re` entry was added. The only edit made after this run, and after the
browser check above, was to the chart's subtitle: its trailing "split by what
blocked it" described the two avoidable series, which are split by reader
state, so it was dropped. No Rust changed.

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

**Measured, 2026-09-09, at `9287048c` (the final feature code) on the
recording host (Apple M1 Pro, eight threads), reports stored under
`docs/progress/features/`.** An earlier measurement at `e6757fa3` (before
the review remediation added the per-reader-state barrier-block rate and the
by-cause totals) read byte-identical persistence, lineage, memory, drift,
neighborhood and structure values; the final report's comparison block
compares against that reading (same path, since this feature's report is
the series epoch) and finds every case `inputs_changed: false` with all
deterministic readings equal. The stored JSON therefore names its own path
as the reference; the reading it compared against is the goal report
committed at `948d0bc3`, recoverable with `git show`, not a second file.

Gate (`t12-f04-baseline-world-set.json`): exit 0, `severe=false`; all six
counters `ok` against both the pinned epoch and T11.F18; wall-clock per
creature-tick 0.0013002 ms (-16.64% against the epoch, -34.27% against
T11.F18). The gate profile and its epoch are unchanged.

Goal (`t12-f04-baseline-world-set-goal.json`, the first `goal-worlds-v1`
reading): `/usr/bin/time -p make bench PROFILE=goal` took **474.33 s** end
to end (512.69 s at the earlier measurement, with review agents sharing the
host); simulation 460.7 s (Orchards 174.3 s, Canyon 116.8 s, Confluence
169.6 s); founder observation 111 ms, evolved neighborhoods 457 ms total,
drift 11.18 s, final-state observation 0.81 s, all inside their caps.
Profile totals per creature-tick: VM steps 23.34, mesh hops 2.248, graph
visits 1.029, plasticity 0.047, actions 1.371, births 0.0186. Every case
survived the horizon; every peak is the 100,000 cap at tick 61–65 and no
case sits there afterwards.

| Reading | Orchards / 11 | Canyon / 22 | Confluence / 33 |
| --- | --- | --- | --- |
| Final / minimum / plateau population | 8,818 / 1,273 / 7,804.98 | 7,715 / 3,202 / 8,143.92 | 21,818 / 1,054 / 12,003.77 |
| Births; creature-ticks | 372,811; 22.25 M | 351,803; 17.75 M | 432,991; 22.21 M |
| Final mean energy | 21.90 | 21.85 | 24.27 |
| Passable; largest component of passable | 100%; 100% | 52.72%; 98.06% | 76.63%; 99.54% |
| Applied eats type 0 / type 1 (type-1 share) | 7,176,755 / 2,199 (0.031%) | 4,883,443 / – | 5,811,299 / 304,166 (4.97%) |
| Final standing density type 0 / type 1 | 738,829 / 468,253 | 409,865 / – | 539,023 / 193,724 |
| Moves attempted | 20,336,298 | 16,132,634 | 19,647,038 |
| Blocked by barrier / occupied / world edge | 0 / 4,261,110 / 0 | 2,426,094 / 4,347,239 / 0 | 1,492,516 / 5,606,009 / 95,319 |
| Barrier-blocked fraction of all moves | 0% | 15.04% | 7.60% |
| Attempts beside a barrier, reader / no reader | 0 / 0 | 145,582 / 4,524,928 | 25,401 / 2,700,477 |
| **Barrier-block rate beside a barrier, reader / no reader** | Undefined / Undefined | **58.52% / 51.73%** | **52.75% / 54.77%** |
| Avoidable blocked moves of any cause by reader state, share of all moves (reader / no reader) | 0.17% / 20.75% | 1.27% / 40.47% | 0.46% / 35.80% |
| Surviving founder clades; entropy (nats) | 22; 2.360513 | 24; 2.625223 | 14; 0.882285 |
| Memory sensitivity (different from either) | 0.000113 | 0.000518 | 0.000092 |
| Drift changed/all births at 1,000 (floor 0.0015) / 2,000 (floor 0.005, re-based from 0.008 at this closure) | 9/2,000 = 0.0045 / **12/2,000 = 0.006** | 20/2,000 = 0.010 / **10/2,000 = 0.005** | 9/2,000 = 0.0045 / **12/2,000 = 0.006** |
| Drift hop-cap hits | 0 | 0 | 0 |
| Founder changed / dead | 0.480769 / 0 | 0.447115 / 0 | 0.480769 / 0 |
| Evolved changed / dead (pooled) | 0.270000 / 0.030909 | 0.290000 / 0 | 0.282727 / 0.011818 |
| Structure size median (p25–p75; mean) | 82 (73–115; 98.80) | 90 (71–131; 104.48) | 87 (84–106; 101.36) |

What the readings say. Orchards' fruit was eaten 2,199 times, so the latent
niche is touched by mutants at a trace rate; Confluence's fruit share of
4.97% and its late rise from 7,083 at tick 1,600 to 21,818 at 2,000 with
entropy collapsing to 0.88 nats over 14 clades are one lineage exploiting
the rich food, the first thing the tracking exists to show. No barrier
awareness has evolved by tick 2,000: standing beside rock, genomes that
carry a barrier reader are blocked as often as those that do not (Canyon
58.5% against 51.7%, Confluence 52.7% against 54.8%), which is the baseline
later closures compare against; the any-cause avoidable share (1.27% against
40.47%) only reflects how few genomes carry a reader and is kept as a
crowding reading. Occupancy blocks outnumber barrier blocks in every world,
and Confluence's 95,319 edge blocks are its Bounded edges. The depth-2,000
drift readings are the substrate's (identical to T11.F18 for one food
type), below the standing floor, and recorded for the user's decision under
Notes.

Full readings — comparison tables, per-seed dumps, and neighborhood rows —
relocated 2026-09-09 to
[`docs/progress/readings/t12-f04-baseline-world-set.md`](../../progress/readings/t12-f04-baseline-world-set.md).

## Success Criteria

- [x] `make bench PROFILE=goal` runs Orchards, Canyon, and Confluence once
      each from fixed maps, all three persist to tick 2,000 without pinning
      at the cap, and the command completes under 600 s on the recording host.
- [x] A later closure's goal report can be compared per world against this
      one, with changed recipe inputs labeled, and the dashboard shows each
      world's readings over closure order.
- [x] Reviewed diff, mutation record, stored reports, and closure checks
      pass; the drift-floor reading is recorded with the user's decision.

## Notes for AI Agents

- Second pass, 2026-09-09: Fable 5.1 orchestrator took over the Codex
  branch (`codex/t12-f04`, rebased onto `main` at `c95457d9` in
  `.claude/worktrees/t12-f04`, branch `worktree-t12-f04`). Recipe design and
  balancing were done by the orchestrator (config data, tuned by screening
  runs), a recorded deviation from "write no feature code yourself"; all Rust,
  frontend, and dashboard code went through the implementer. The follow-up
  pass (world-coordinate fBm sampling, preview blending) ran on a fresh
  implementer because this environment exposes no way to message a finished
  agent; both passes' records are under Verification.
- Independent review at `948d0bc3` (Fable, fresh context): P1 1, P2 2,
  P3 5. P1: the "avoidable blocked-move fraction by reader state" was
  described as a per-state barrier-block rate but divides avoidable blocks of
  any cause by all move attempts (remediated: wording corrected here and in
  the README; the true per-state barrier-block rate added from the existing
  barrier-neighbor stats). P2: `docs/progress.md`'s first-pass row linked
  reports that now hold second-pass readings (fixed: retargeted to the
  preserved first-pass goal report; the first-pass gate report was
  overwritten and is recorded as lost). P2: the track roadmap still described
  the maze Canyon (fixed: dated amendment in the track note, flagged for the
  user's sign-off). P3s: stale deferred map-change note (resolved above);
  a weakened `> 2` assertion in `reproducibility.rs` (pinned in remediation);
  `GOAL_RECIPES` coupled to seed position (seed carried in the tuple with an
  equal-length assertion in remediation); browser-check attribution (fixed);
  the `"goal"`/world-set string profile kind threaded as booleans, deferred
  below. Remediation pass 1 went to a fresh implementer.
- Deferred P3 (maintainability): the profile kind is still a string compared
  in several places (`params.name == "goal"`, `world_set` booleans); a
  `ProfileKind` enum is the smallest refactor before the next profile
  variant. Pre-existing pattern, not extended in scope here.
- Recipe screening evidence (orchestrator, `v3-cli run` at 1600² with 10,000
  founders, one run each, population at ticks 100/500/1000 unless noted):
  literal diffuse-only grass (uniform fertility 0.2, growth 0.03) extinct by
  tick 400; uniform 0.4 fertility, growth 0.06: 31 at 800; meadow-textured
  grass (the shipped Orchards, density 0.5 variant) 100,000 / 15,076 / 3,526
  and 11,885 at 2,000; founders on rich patches (fruit as type 0) pinned at
  100,000 through tick 1,000 for both standing-crop settings; fBm canyon on
  production fertility 44,855 / 119 / 7 and 46,674 / 192 / 3; canyon with 70
  valley meadows 81,301 / 8,244 / 9,977, with 45 meadows (shipped) 45,014 /
  3,947 / 4,102; composite drafts with diluted meadows 60,807 / 11 / 1 and
  72,767 / 140 / 514, with Wrap edges 73,094 / 142 / 77, with production
  grass 100,000 / 12,754 / 15,208, without terrain 99,998 / 6,061 / 1,517,
  with grass at density 0.5 and 5 per unit (shipped, before adding five
  meadows) 100,000 / 6,556 / 1,779. Final 2,000-tick runs of the shipped
  recipes under host contention: Orchards 184 s ending at 8,818 (range
  2,517–17,790 after the boom), Canyon 136 s ending at 7,715 (3,534–8,778),
  Confluence 265 s ending at 21,818 (2,979–21,818, rising late).
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
  the user's written exception on 2026-09-08) and 12/2,000 with two. On
  2026-09-09 the user decided to re-base the standing floor permanently to
  0.005 (10/2,000), so later closures pass unless the reading regresses
  further, and to merge; the decision is recorded in the T11 track's dated
  note, and both readings here meet the re-based floor.
- Deferred P2 (mutation): the two `passable_connectivity` visited-guard
  mutants time out rather than fail; killing them needs a watchdog test,
  which is not worth adding for score. Confirmed still the only survivors by
  the implementer's fresh run on 2026-09-09 (`world.rs:57:48` `&&` to `||`,
  `world.rs:57:32` deleted `!`); the mutated traversal revisits cells forever,
  so the mutant hangs instead of producing a wrong reading, and every other
  mutant in the feature diff is caught. Reconfirmed by the follow-up pass's own
  fresh run on 2026-09-09 (`309 mutants tested in 26m: 256 caught, 51 unviable,
  2 timeouts`): still the only two survivors, still the same lines, and nothing
  missed. Reconfirmed again by the remediation pass's fresh run (`325 mutants
  tested in 21m: 262 caught, 61 unviable, 2 timeouts`, same two survivors).
- Resolved (was a deferred P3, found 2026-09-09): the follow-up pass's
  world-coordinate `FbmThreshold` sampling moved Confluence's barrier map;
  all three previews and the README readings were regenerated with
  `world inspect` at `948d0bc3`, and the goal report was measured after the
  change. Orchards and Canyon were unaffected (no terrain and whole-world
  bounds respectively, verified by identical `world inspect` output).
- Deferred P3 (test wiring, found 2026-09-09): `crates/v3-core/tests/
  mutational_neighborhood.rs` is run by no `make` target, the same gap this
  pass closed for `baseline_worlds.rs`. Left alone here because it belongs to
  T11, not to this feature; a T11 feature that touches it should add
  `rust-test-mutational-neighborhood` to `rust-test-all`.
