# T14.F11 — Progress Page Information Design

**Status**: In Progress
**Last updated**: 2026-09-15
**Feature**: T14.F11
**Track**: [T14 — Runtime Telemetry and Report Integrity](../../roadmaps/t14-runtime-telemetry-and-report-integrity.md)

## Goal

`docs/progress/index.html` answers the five questions the master success
definition asks, in this order and one section each: is the world alive; how
many ways of living, for how long; is cognition present and paying; is the
substrate still evolvable; what did this closure cost. A headline row at the
top answers the first four in five tiles — cognition split into present and
paying — each tile reading Orchards, Canyon and Confluence side by side against
its floor or first reading, with its exposure denominator printed and its delta
against the previous closure labelled single-run. Every indicator below is one
row of three world panels drawn from the same reports the page reads today; the
historical `goal-v1` series appears inside those panels left of a labelled
break rather than in a tab; a reading a report did not measure is drawn as not
measured, never as zero. The page stays one static file in Git with no build
step, service, database or composite score.

## Non-Goals

- No Rust, no new telemetry, no change to any stored report or to
  `benchmark-series.json`; the page reads what exists.
- No strategy descriptors, strategy counts or niche-overlap measures derived
  from the clade rows (T01.F04, T01.F06); the page shows raw stored counts,
  sums and their ratios only.
- No rendering for the T14.F09 loss curve: its slot under "ways of living" is
  named here and stays empty until F09 lands.
- No charting library or CDN dependency; the existing SVG primitives stay.
- No new tests beyond the existing page test, and no edit to
  `docs/progress.md` in the feature diff; its page description (lines 10–17)
  describes the tabbed layout and is refreshed by the closure entry.

## Inputs and Invariants

Sources of truth: the track's F11 scope note and 2026-09-15 amendment;
`docs/progress/index.html` (74,604 bytes; helpers `num`, `get`, `card`,
`addTableToggle`, `table`, `legend`, `showTip`, `lineChart`, `stackedColumns`,
`heatmap`, `statTile`, `group`, `loadAll`, `validateArtifact`, `artifactNotice`,
`meshSums`, the tick-zero map card); `scripts/benchmark-artifacts.test.mjs`;
`docs/progress/benchmark-series.json`; the latest goal-worlds summary,
`docs/progress/features/t14-f07-surviving-clade-behavioral-profile-goal.json`
(2026-09-16T01:35Z at `bdfb7675`), plus `t02-f04-grazing-recovery-and-overuse-goal.json`
for the grazing fields (F07's run predates its rebase onto T02.F04); the
T14.F12 spec's floor paragraph; the T11 track's 2026-09-14 floor amendment.

**Survey of the current page** (served over HTTP, 1280 px wide, 2026-09-15;
supersedes the scope note's 129-card figure):

| Tab | Cards | Tiles | Charts | Height |
| --- | --- | --- | --- | --- |
| Goal worlds | 66 (3 × 21 charts + 3 map cards) | 15 (5 per world) | 63 | 14,092 px |
| Historical goal | 30 | 6 | 29 (+2 heatmaps) | 8,494 px |
| Gate profile | 28 | 6 | 28 (+2 heatmaps) | 7,694 px |

**Telemetry verified in the latest summary.** Every path below exists in the
T14.F07 goal summary unless marked; `<w>` is the case whose `case.name` is the
world and `<s>` its `case.seed` (11, 22, 33), which also keys the
`goal_indicators.*.per_seed[]` and `deterministic.per_seed[]` rows.

| Reading | Path (under `deterministic.goal_indicators`) | Denominator or bound |
| --- | --- | --- |
| Population, minimum, plateau; births | `population_persistence.per_seed[<s>].{final_population,minimum_population,plateau_population}`; `deterministic.per_seed[<s>].births` | 20 checkpoints at ticks 100–2,000 |
| Checkpoint trajectory (F04) | `population_persistence.per_seed[<s>].samples[].{tick,population,surviving_founder_clade_count,shannon_entropy_nats,mean_generation,mean_genome_size,mean_mesh_nodes,food_density_total[]}` | — |
| Deaths by cause (F03) | `cases[<w>].mortality.{by_cause{16 keys},deaths_total}` | `deaths_total` |
| Energy flows (F03) | `cases[<w>].energy_flows.*`: `food_intake_by_type[]`, five `*_credit` keys, `genome_size_creature_ticks` (an exposure, not a flow), `definition`, and the charge keys (`lifecycle_decay`, `action_charges`, `genome_carrying`, `vm_compute`, `graph_compute`, `priority_bid`, `failed_action_penalty`, `hebbian_learning`, `reward_learning`, `parental_transfer_debit`, `predation_victim_debit`, `external_removal_loss`, `maximum_energy_clamp_loss`) | — |
| Grazing (T02.F04) | `cases[<w>].{grazing_modifier_mean[],grazed_cell_share[]}` per food type, and the same keys per checkpoint sample — present from T02.F04 on, absent in F07's summary | — |
| Surviving clades, entropy | `lineage_diversity.per_seed[<s>].{surviving_founder_clade_count,shannon_entropy_nats}` | — |
| Clade profile (F07) | `cases[<w>].surviving_clade_profiles.rows[]` (27 / 28 / 17 rows) with `size`, `mean_energy`, `mean_age`, `mean_generation`, `mean_genome_size`, `eats_by_type[]`, `actions_by_type{5}`, `predation_kills`, `predation_hits_taken` | rows = surviving clades |
| Occupancy grid (F10) | `population_persistence.per_seed[<s>].samples[19].occupancy_grid.{cells_x,cells_y,distinct_clades[256],population[256]}` | 16 × 16 |
| Typed eat share, predation (F02) | `cases[<w>].typed_eat_share[]`; `cases[<w>].predation.{actions_attempted_total,actions_transferred_total,kills_total}` | — |
| Reproductive success (F06) | `cases[<w>].reproductive_success_by_cognitive_class.by_class.{none,plasticity,shared_memory,stateful}.{creatures_observed_total,offspring_spawned_sum,survival_ticks_sum}` | `creatures_observed_total` |
| Memory sensitivity | `memory_sensitivity.per_seed[<s>].{different_from_either_count,different_from_either_fraction,final_creature_count}`; `temporal_memory_sensitivity.per_seed[<s>].{previous_slots,persisted_outputs,operator_state}.*` | `final_creature_count` |
| Structural companions | `structural_companions.per_seed[<s>].{has_plasticity,has_stateful_compute_node,reads_shared_memory,writes_shared_memory,final_creature_count}` | `final_creature_count` |
| Changed plasticity (F05) | `cases[<w>].cognition.{plasticity,hebbian,reward_modulated}_{changes,updates}_total`, `shared_memory_writes_changed_total` | `*_updates_total` |
| Sensor census (F08) | `population_persistence.per_seed[<s>].samples[19].sensor_census.{world_inputs[]{key,creatures},creatures_reading_shared_memory,creatures_with_any_stateful_read,creatures_with_stateful_node}` | checkpoint `population` |
| Neighborhood read (F12) | `cases[<w>].neighborhood_read.{changed,silent,dead}_per_all_births`, `sample_size` × `birth_trials` (50 × 100) | 5,000 births |
| Drift depth | `cases[<w>].drift_depth.readings[]` at `depth` 0/22/250/1,000/2,000: `changed_per_all_births`, `route_varying_fraction`, `mean_executed_nodes` | `births_total` 2,000 |
| Mutated births | `cases[<w>].mutational_neighborhood.founder.births.any_events.*`, `.evolved.per_seed[]` (`mesh_summary` via `meshSums`) | — |
| Mutation supply, outcomes (F02) | `cases[<w>].mutation_supply.{events_applied_total,executed_target_total,reachable_target_total}`; `cases[<w>].mutation_outcome_summary.{carriers_observed_total,survived_short_horizon_total,survived_long_horizon_total}` | `events_applied_total`; `carriers_observed_total` |
| Structure size | `cases[<w>].reachable_structure_size_distribution.{min,p25,median,p75,max}` | — |
| Compute | `deterministic.per_creature_tick.{6 counters}`, `deterministic.per_seed[<s>]`, `environment.{wall_clock_ms_per_creature_tick,wall_clock_ms_per_seed[],host,generated_at}` | — |
| Deltas | `comparison.references[].{path,severe,counters[]{name,level,percent_delta},wall_clock{level,percent_delta},cases[]{case,inputs_changed,absent_in_reference,readings[]{name,current,reference,percent_delta}}}` (38 reading names per case; readings carry no `level`) | — |

**Not measured**, drawn as such: the nine `Undefined` goal indicators, which
each section's lede names — ways of living: `strategy_count`,
`strategy_causal_distinctness`, `evolutionary_activity`, `adaptive_novelty`,
and the F09 loss curve; cognition: `learning_dependence`, `memory_dependence`,
`prediction_dependence`, `information_integration`, `reciprocal_interaction`;
evolvable: `evolutionary_activity`, `adaptive_novelty` — and T13.F02's
goal-worlds closure (`report_omissions`). The gate summaries carry the six
counters and wall clock; their goal indicators are `Undefined`.

**Reference lines.** The only standing floor on the page is
`neighborhood_read.changed_per_all_births`: the F12 first reading per world,
strict not-below — Orchards 0.1464, Canyon 0.1300, Confluence 0.1434. The
depth-2,000 drift floor was withdrawn on 2026-09-14; drift depth draws its
first goal-worlds reading. The founder single-event silence line keeps the T11
reference the current page already draws. Compute counters draw the epoch
baseline of the series each point belongs to (`goal_worlds.epoch_baseline`,
T11.F19; `goal.epoch_baseline`, T11.F17, left of the break;
`gate.epoch_baseline`). Everything else draws the first reading of the
`goal-worlds-v1` series. The rule, in order: floor, else epoch baseline for
compute, else first reading; the line is labelled with what it is and which
closure set it.

**Options considered.** Rewrite the single static file on its own SVG
primitives — taken. Rejected: a CDN charting library (network dependency,
nothing the primitives lack) and re-grouping the existing cards under new
headings (the per-world stacks and tabs are the problem).

Invariants: `fetch()` paths and `loadAll`'s reading of `benchmark-series.json`
are unchanged; historical reports are read, never rewritten; no localStorage
state is needed once the tabs go; the page stays usable at 1280 px with three
panels per row and stacks panels below about 900 px.

## Implementation Tasks

- [x] Remove the profile tabs and their localStorage state; keep `loadAll`,
      `validateArtifact`, `artifactNotice`, `meshSums`, `num`, `get` and the
      `  // ── Boot` marker with their current semantics so the page test is
      unchanged; keep the status line and the summary/raw notice.
- [x] Implement once, and use for every card, the five behaviours below.

| Behaviour | Rule |
| --- | --- |
| World row | Every indicator is one row of three panels, Orchards, Canyon, Confluence, sharing one y-scale; each panel keeps its JSON-path subtitle, table toggle and hover readout naming closure, host and date. |
| Series break | Charts over closure order draw `goal-v1` closures left of a vertical break labelled "goal-worlds-v1 from T12.F04"; the left segment is the same seed's default-world reading, muted and labelled "goal-v1, default world, seed N" in legend and hover. No delta crosses the break. A reading with no `goal-v1` counterpart (every per-case block) starts at the break. |
| Not measured | A position with no reading (T13.F02, an `Undefined` indicator, a pre-T02.F04 grazing sample, `absent_in_reference`) is a gap with a "not measured" marker, never zero. |
| Deltas | A percent delta shown anywhere is read from `comparison.references[]` — walking the series' closed list backward from the closure, the first closed summary the report names — by reading or counter name, and is printed as "single run, vs <that closure>" with the `level` when the block carries one. A tile or chart whose name is not in that block shows no percentage. |
| Denominators | A fraction prints its count and denominator beside the value ("6 of 9,772"); a fixed-sample reading prints the sample ("5,000 births from 50 genomes"). |

- [x] Headline row, in the first screen at 1280 × 800:

| Question | Tile | Value per world | Beside it | Reference |
| --- | --- | --- | --- | --- |
| Alive | Final population | `final_population` | `minimum_population`; delta `final_population` | first reading |
| Ways of living | Surviving founder clades | `surviving_founder_clade_count` | `shannon_entropy_nats`; delta `surviving_founder_clade_count` | first reading |
| Cognition present | Memory-sensitive creatures | `memory_sensitivity` `different_from_either_fraction` | count of `final_creature_count`; delta `memory_different_from_either_fraction` | first reading |
| Cognition paying | Offspring per creature, stateful vs none | `by_class.stateful` and `.none` `offspring_spawned_sum / creatures_observed_total` | both `creatures_observed_total`; no stored delta | none |
| Evolvable | Changed births at depth | `neighborhood_read.changed_per_all_births` | "5,000 births"; delta `neighborhood_read_changed_per_all_births`; floor and pass/fail | F12 floor |

- [x] Sections in this order, each with an `h2` phrased as its question, a
      one-line lede naming what it cannot yet measure, and these cards:

| Section | Cards (each a world row unless stated) | Source |
| --- | --- | --- |
| 1 Is the world alive? | Population trajectory (20 checkpoints; latest closure emphasized, earlier ones faded) | `samples[]` |
| | Final, minimum and plateau population over closures | `population_persistence.per_seed` |
| | Births and deaths per closure | `deterministic.per_seed[<s>].births`, `mortality.deaths_total` |
| | Deaths by cause, stacked columns over closures | `mortality.by_cause` |
| | Energy flows: the thirteen charge keys as a stacked column per closure, total food intake as a marker or line over it | `energy_flows` |
| | Food density by type and grazed-cell share at checkpoints | `samples[].food_density_total`, `grazed_cell_share`, `grazing_modifier_mean` |
| 2 How many ways of living, for how long? | Surviving clades and entropy over closures (two panels sharing the row) | `lineage_diversity.per_seed` |
| | Clades and entropy at checkpoints, latest closure emphasized | `samples[]` |
| | Surviving-clade profile: one table per world, rows by `size` descending, eats and actions as in-row shares, kills and hits | `surviving_clade_profiles.rows` |
| | Occupancy: distinct clades per cell, 16 × 16 heatmap at the last checkpoint | `occupancy_grid` |
| | Eat share by food type, and predation attempts, transfers and kills over closures | `typed_eat_share`, `predation` |
| | Loss curve (T14.F09): named in the lede, nothing rendered | — |
| 3 Is cognition present and paying? | Offspring per creature and survival ticks per creature by class, latest closure | `reproductive_success_by_cognitive_class` |
| | Memory sensitivity over closures, y from 0, count of denominator printed | `memory_sensitivity.per_seed` |
| | Temporal memory sensitivity, three components | `temporal_memory_sensitivity.per_seed` |
| | Creatures carrying cognitive structure, share of final population | `structural_companions.per_seed` |
| | Plasticity, Hebbian and reward updates that changed a weight, share | `cognition` |
| | Sensor reach: creatures reading each world input at the last checkpoint | `sensor_census` |
| 4 Is the substrate still evolvable? | Changed births at depth vs floor (changed bold; silent and dead muted) | `neighborhood_read` |
| | Drift depth at 2,000 (1,000 muted) over closures, first reading as reference | `drift_depth.readings` |
| | Mutated births by outcome: founder and pooled evolved changed, dead, silent; founder single-event silence with its T11 reference | `mutational_neighborhood` |
| | Mutation supply delivered to executed nodes, share of applied events; carriers surviving the short and long horizon, share | `mutation_supply`, `mutation_outcome_summary` |
| | Reachable structure size (median, p25–p75 band); genome size and mesh nodes at checkpoints | `reachable_structure_size_distribution`, `samples[]` |
| | Mesh execution over sampled evolved genomes (table, existing) | `meshSums` |
| 5 What did this closure cost? | Six work counters as percent of their series' epoch baseline, one line each, `level` in hover | `per_creature_tick`, `comparison` |
| | Wall clock per creature-tick, host and date in hover | `environment` |
| | Gate profile, one row: six counters as percent of its epoch baseline, and wall clock (seed mean) | gate summaries |
| 6 World detail (collapsed `<details>` per world, after cost) | Tick-zero map and applied config (existing card); barrier and blocked-move rates by cause and reader state; avoidable blocked share; mean energy; moves attempted | `cases[<w>]`, `samples[]` |

- [x] Drop the cards that fail the question test: the six per-world
      work-counter charts, creature-ticks per second and total wall clock
      (folded into the cost row), the founder operator-silence heatmap, the
      indicator-coverage heatmap (folded into each section's lede), the
      standalone mean-energy and blocked-moves charts (in world detail), the
      "Latest closure" tile groups of the old tabs, and the per-world headline
      tiles.
- [x] Rewrite every prose surface on the page for a reader with ADHD, per
      `~/.claude/skills/i-have-adhd/SKILL.md` (user direction, 2026-09-15).
      Surfaces: page title and caption, status line, tile labels and captions,
      section `h2`s and ledes, card titles, card notes, legend labels, hover
      readout text, the load-failure notice, and the world-detail prose.
      JSON-path subtitles stay as they are (data provenance, not prose). Rules
      for static copy: lead with the answer — what the number is and whether
      it is good — before any qualifier; one idea per line; no jargon without
      the plain word beside it ("clade — a founder's descendants"); small
      working sets — no sentence asks the reader to hold more than one fact,
      no list group longer than five; concrete not vague ("9,772 creatures",
      not "a healthy population"); no idioms or figurative phrases;
      matter-of-fact wording for missing data ("not measured in this
      closure", never "unfortunately" or "oops"); no preamble and no closing
      pleasantry anywhere.
- [x] Run the impeccable detector once when the page is finished
      (`~/.claude/skills/impeccable/scripts/impeccable detect --json docs/progress/index.html`)
      and record its findings and the after-survey (cards, tiles, charts,
      height per section) in `docs/progress/readings/t14-f11.md`.

## Verification

- [x] `make check-docs` -> exit 0 on 2026-09-15, rerun after the simplify
      pass, including `node --test scripts/benchmark-artifacts.test.mjs`
      (3 pass) against the rewritten page with the test file unchanged; the
      detector reported no finding and a headless Chrome load over HTTP logged
      nothing ([readings](../../progress/readings/t14-f11.md)).
- [x] Simplify pass renders the same page: the static DOM (inline script
      stripped) and all 3,721 hover readouts across 113 plots are identical
      before and after ([readings](../../progress/readings/t14-f11.md),
      "Verification commands").
- [x] Orchestrator browser check over HTTP before review — passed
      2026-09-15 on port 8791; every item below observed
      (`python3 -m http.server --directory docs/progress <port>`; port 8765 is
      taken on this host by a service that answers "ok" to every path): no
      console error; the headline row and section 1's first card in the first
      screen at 1280 × 800; sections 1–5 in order, world detail last; every
      chart three panels in one row; the break labelled in a chart that spans
      both series; the three floor lines and pass/fail on the evolvable tile;
      every delta labelled single-run; T13.F02 drawn as a gap; a table toggle
      and a hover readout showing host and date; the tick-zero map inside world
      detail. The orchestrator ticks this item with the date and port.
- [x] Copy check — passed 2026-09-15 on port 8791 (404 notice checked from a
      data-less copy on port 8792); no line failed a rule. The check read
      the page title, status line,
      every `h2` and lede, the five tile captions, one card note per section,
      one hover readout and the load-failure notice (serve with
      `benchmark-series.json` unreadable) against the static-copy rules in the
      ADHD task; any line that fails a rule is named in
      `docs/progress/readings/t14-f11.md` and fixed before review. The
      orchestrator's check of brief 1's page passed (clean console, 5 tiles,
      5 question sections in order, table toggle and hover readout with host,
      date and revision, the break label and "first reading, T12.F04"
      references, T13.F02 as not measured, F12 floors with pass verdicts); the
      simplify pass is proven render-identical to that page.
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: Not applicable, no Rust
      changes; the exception is recorded in Notes for AI Agents (user
      direction, 2026-09-15).
- [x] Benchmark summary: Not applicable, the feature cannot change simulation
      cost; the exception is recorded in Notes for AI Agents (user direction,
      2026-09-15).

## Performance and Goal Impact

Not applicable: the feature changes one static HTML file and reads stored
reports; it runs no benchmark, re-pins no epoch and moves no indicator
(amendment of 2026-09-15, user-directed).

## Success Criteria

- [ ] A reader at 1280 × 800 sees, without scrolling, whether each world is
      more alive, more diverse, more cognitive and still evolvable than at the
      previous closure, with every delta labelled single-run and the
      evolvability floor shown as pass or fail.
- [ ] Every indicator card is one row across the three worlds; no chart or tile
      is repeated per world in separate stacks; there are no tabs.
- [ ] The `goal-v1` history is visible inside the charts behind a labelled
      break; T13.F02 and every `Undefined` indicator read as not measured.
- [ ] Each landed T14 reading (F03, F05, F06, F07, F08, F10, F12) and T02.F04's
      grazing fields appear under the section named above, from the stored
      paths, with denominators printed; nothing is rendered for F09.
- [ ] The chart, heatmap, table-toggle, hover-readout and tick-zero-map helpers
      remain and the page test passes unchanged.

## Notes for AI Agents

- Exception: no benchmark gate and no mutation gate for this feature; no
  benchmark or mutation specialist is spawned (user direction, 2026-09-15).
- Exception: Performance and Goal Impact is recorded as Not applicable.
- Exception: verification is `make check-docs`, which loads the page script,
  calls `loadAll`, `meshSums`, `artifactNotice`, `num` and `get`, checks the
  unknown-summary-version rejection and slices at the `  // ── Boot` marker.
- Exception: the orchestrator serves `docs/progress` over HTTP and checks the
  page in a browser before review, because the implementer has no browser.
- Exception: the spec owner and the implementer use the `impeccable` skill;
  the implementer runs as Fable 5.1 at effort `medium` (orchestrator passes
  `model: "fable"`); the `roadmap-implementer` definition is unchanged.
- Exception: rewriting `docs/progress/index.html` wholesale is permitted while
  the chart, heatmap, table-toggle, hover-readout and tick-zero-map helpers stay.
- Decision: the page-script contract stays and `scripts/benchmark-artifacts.test.mjs`
  does not move: the five exported names, their semantics and the Boot marker
  are kept in the rewritten file.
- Decision: the historical goal and gate tabs are removed; `goal-v1` is the
  left segment of each chart behind a labelled break, and the gate profile is
  one row in the cost section.
- Decision: every percent delta on the page is read from the stored
  `comparison.references[]` block and labelled single-run; nothing is
  recomputed from two reports. The block used is the first closed summary the
  report names, walking the series backward from the closure: T14.F07 names
  `t11-f19` and `t03-f11` but not `t02-f04` (its run predates the rebase), so
  its deltas read "vs T03.F11" and are 0 %, the simulation being identical.
- Decision: the cost row's y-scale fits the goal-worlds-v1 segment; goal-v1
  points above it (T11.F01 seed 11 ran 863 VM steps per creature-tick against
  24.4 at T11.F17) draw as clipped marks at the top edge with the value in the
  hover, so the current segment stays legible without hiding the history.
- Decision: the only floor drawn is the F12 per-world first reading; drift
  depth and every other indicator draw their first `goal-worlds-v1` reading;
  compute draws its series' epoch baseline.
- Decision: on 2026-09-15 the user directed, mid-implementation, that the
  page's prose be rewritten for a reader with ADHD using
  `~/.claude/skills/i-have-adhd/SKILL.md`; the static-copy rules in the
  Implementation Tasks are that direction applied to a page rather than to a
  chat reply, and JSON-path subtitles are exempt as provenance.
- Decision: the impeccable `shape` interview was replaced by the F11 scope
  note as the brief, since the spec owner runs without a user channel; no
  PRODUCT.md or DESIGN.md is written, as neither is in the amendment's file set.
- Decision: the closure entry in `docs/progress.md` also refreshes that file's
  page description (lines 10–17), which names the removed tabs.
