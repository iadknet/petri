# T23 — Behavior and Cognition Readings

**Status**: Planned
**Last updated**: 2026-10-01
**Master**: [Program Roadmap](../roadmap.md)

## Goal

A running world answers, on its dashboards and while it runs, the questions
the program is about: are creatures using their senses, how much of their
brain does anything, how do they live and die, is selection favoring the ones
that sense, and how does this run differ from another build or config. T21
delivered the pipe and exported the counters the runtime already kept; none of
those counters measures behavior against what a creature sensed, so the
dashboards could only show health (population, energy, rates). This track adds
the readings, each with its chance baseline, and lays the dashboards out by
question. Like T21, it is an instrument for looking at runs.

## Track Success Criteria

- [ ] Every run's metrics carry its build, config digest, world and preset, and a compare dashboard shows chosen runs side by side against ticks and grouped by build and config.
- [ ] A run's dashboard shows how often moves land on food, fruit, barriers and occupied cells, next to what chance would give from what the creature sensed.
- [ ] A run's dashboard shows, per input family, the share of sampled creatures whose chosen action changes when that input changes, next to a no-change control.
- [ ] A run's dashboard shows the distribution of mesh nodes total, reachable, executed and contributing, and the share of creatures with a load-bearing world-input read.
- [ ] A run's dashboard shows distributions of lifespan, offspring, food intake and energy spend by category for creatures that died, not only population means.
- [ ] A run's dashboard shows reproductive success by sensor use and brain use, and those classes' population share over time.
- [ ] A run's dashboard shows the largest lineages' population share over time, each with its action mix and its move-against-chance reading.
- [ ] Readings taken from recorded creature windows chart per run with their sample counts, and any recorded window opens as a tick-by-tick view of what the creature sensed, chose and got.
- [ ] A long run keeps its newest 300 creature windows in full and a nested, age-thinned sample of all older ones, never below one in ten, inside a per-run byte cap.
- [ ] Every reading keeps T21's guarantees: byte-identical seeded output with telemetry on or off, no production RNG, bounded memory, and total telemetry cost under the ceiling T21.F05 recorded.

## Executable Features

- [ ] **T23.F01 — Run Grouping and Question-First Dashboards** — Depends on: None
  - Goal: Measurement tooling. Every run's metrics carry its build, config digest, world and preset, the dashboards are laid out by the question each panel answers, runs can be compared grouped by build and config, and one creature's recorded window reads as a tick-by-tick table of what it sensed, chose and got.
- [ ] **T23.F02 — Movement Against Chance** — Depends on: None
  - Goal: Measurement tooling. For every applied move, the world records whether it landed on food, fruit, a barrier or another creature, and what chance would have given from the creature's eight sensed neighbors, so seeking and avoiding read as a ratio to chance.
- [ ] **T23.F03 — Sensor Reactivity Probe** — Depends on: None
  - Goal: Measurement tooling. A bounded sample of living creatures is re-run each interval on a scratch copy with one input family changed at a time, and the share whose chosen action changes is recorded per family, beside a no-change control.
- [ ] **T23.F04 — Brain Use Census** — Depends on: T23.F03
  - Goal: Measurement tooling. The same sampled creatures report mesh nodes total, reachable, executed and contributing (by knockout), which world inputs their contributing nodes read, and whether input-dependent routing is load-bearing, exported as distributions.
- [ ] **T23.F05 — Life Histories and Energy Budgets** — Depends on: None
  - Goal: Measurement tooling. Each creature that dies or is removed adds its lifespan, offspring, food eaten by type, energy spent by category and cause of death to run-level distributions.
- [ ] **T23.F06 — Selection on Sensing** — Depends on: T23.F03, T23.F04, T23.F05
  - Goal: Measurement tooling. Creatures the probe and census classified are followed to removal, so reproductive success and lifespan read by sensor use and brain use, beside each class's share of the population over time.
- [ ] **T23.F07 — Lineage Dynamics** — Depends on: T23.F02
  - Goal: Measurement tooling. The largest lineages' population share is recorded over time, each with its action mix and movement-against-chance reading, beside the lineage-diversity indicator T01.F12 defined.

- [ ] **T23.F08 — Creature Window Readings** — Depends on: None
  - Goal: Measurement tooling. When a creature window closes, its ticks add to run-level Prometheus counts of where first moves land against chance, how many moves are blocked, how often it eats with and without food underfoot, whether it ran a fixed program, and how many nodes ran and voted, so those readings chart and compare across runs with no work added to the tick loop.
- [ ] **T23.F09 — Creature Window Viewer** — Depends on: T23.F10
  - Goal: Measurement tooling. Any retained creature window opens as a tick-by-tick view of the neighborhood it sensed, the moves it chose and what the world applied, its energy, and the brain nodes that ran and voted.
- [ ] **T23.F10 — Thinned Creature Window Store** — Depends on: None
  - Goal: Measurement tooling. When a creature window closes, a compact record of it goes to Loki, and a scheduled job in the telemetry stack thins each run's records by age, keeping all of the newest 300, every second one of the next 300 and every tenth one older than that, so a long run keeps its recent windows in full and a sample of its whole history.

## Notes for AI Agents

- Window prototype (2026-09-30): a private page built from the 269 windows of run `62fd103e` showed the F08 readings and the F09 viewer before any code (moves into a sensed barrier at 2.4× chance, 60% of first moves blocked, 63% of windows on a 1–3 tick fixed program, one voting node in the median tick). A tick may hold several actions, which the world applies in order until the tick ends: each applied result pairs with its chosen action by the action event's index, a chosen action with no event was not applied, only the first move is read against the sensed ring, and only an Eat chosen before the first move is read against food underfoot, because later actions start from another cell. F08 is a sample (one window about every 10 s) and its dashboards show counts beside every share.

- Window store (user decisions, 2026-10-01). Per-window records stay inside the telemetry stack. Readings go to Prometheus (F08): a few series per run, kept for the whole run without thinning. Window details go to Loki (F10), not Prometheus, because a window is a structured record (eight cells per tick, the chosen actions and results, the brain hops) that would become hundreds of series per window labelled by window and creature, against T21's rule that the run ID is the only high-cardinality metric label. Tempo keeps full traces under T21.F04's budget and cannot drop single traces, so thinning happens in Loki. Retention bands nest so thinning never needs a deleted record back: by age in windows from the newest, 0 to 299 keep all, 300 to 599 keep even window indexes, 600 and older keep indexes divisible by ten (the user's 10% floor). Each record carries its keep level (2 if its index divides by ten, else 1 if even, else 0), and the job deletes, per run and band, the records below that band's level through Loki's delete API with one request per band. The job is a small extra service in `telemetry/compose.yaml` (the Grafana image has no scheduler), a POSIX `sh` loop. A compact record is about 4 KB against 240 to 900 KB for the full trace. Readings over a thinned history weight each kept record by the windows it stands for (1, 2 or 10). The store has a per-run byte cap (D7).
- Origin (user, 2026-09-30): the T21 dashboards gave no insight into how creatures live, what they do, how they use their senses, or how runs compare. The panels that existed were run health. An interim fix was made on the same day outside any feature: a Behaviour row on `petri-run` (barrier bumps by barrier reader, offspring and lifespan by cognitive class, eat hit rate, blocked-move share, predation results), a `petri-compare` dashboard (identity table, Trend panels against tick, run totals side by side) and links from `petri-runs`. F01 starts from that.
- Evidence: the [live survey](../strategy/live-survey-2026-09-16.md) is the hand-run version of F02 to F04. Its readings (moves onto the single fruit neighbor 62% against 12.5% chance when fruit is West and 5% otherwise; ring-slot toggles changing the action in 6, 297, 16 and 1 of 400 creatures; 2 contributing nodes in the median creature; load-bearing routing in 3.3%) are what these features make continuous. Its method is the reference for the probe and the census; reuse the T11.F14 battery and the survey's paired perturbation, not a new assay.
- `has_barrier_reader` in the existing counters means the genome has a reachable node reading the barrier input, not that the input steers anything (`creature/state.rs::compute_has_barrier_reader`). F03's per-family reactivity is the reading of use; dashboards say which one they show.
- Chance baselines are part of every reading. A move-onto-food share without the share of food among the sensed neighbors, or a reactivity share without the no-change control, does not ship.
- Cost and determinism. F02 and F05 are counters updated where the action and removal phases already touch the data. F03 and F04 run brains on scratch copies: they never commit state, draw from production RNG, or change the order of anything the simulation does, and their sample size per interval is a parameter whose cost is measured against T21.F05's ceiling. Probe results are aggregated in memory and exported as counters and distributions, not one record per creature.
- Not closure evidence, and not an objective. T21's program-tracking rule stands for every reading here: no dashboard reading closes a feature, sets a default, or becomes a goal-profile indicator. A reading that should become an indicator is proposed as its own change to T01/T14 and the workflow. No feature may optimize against these readings (the master rule on indicators).
- Ownership. This track owns the readings it adds (their definitions, counters, probe and export mapping) and the dashboards' question layout. T14 keeps the definitions of the counters it already owns; T21 keeps the stack, sampling policy and export contract; T22 keeps capability assays in the lab. `v3-core` exposes plain data and takes no OpenTelemetry dependency.
- Boundary with T22. T22 asks whether a capability can evolve, on lab arenas under lab selection. This track reads what the live production population is doing, continuously, with no selection, no arena and no task. F03's probe and F04's knockouts read live creatures for observation only and answer no capability or reachability question; any such question still goes to the lab.
- Verification (user decision, 2026-10-01). T23 features take the observability exemption in the [workflow's Benchmark gate](../workflow.md#benchmark-gate): no gate or goal profile and no mutation gate, under the reviewed claim that the feature changes no simulation behavior. The three track checks T21 delivered stand in, run with T21's commands: the telemetry-neutrality test inside `make check`, the parent comparison when the diff touches `v3-core`, the tick loop or a timed region, and the overhead check when the feature adds work to a run, against the ceiling T21.F05 recorded. Every T23 addition to `v3-core` and the binaries' run loops sits behind the reference build's switch. The F10 thinning job and Loki writes run outside the tick loop.
- Measurement features: the natural-analog rule and the environmental-pressure baseline rule do not apply, and no feature here adds a pressure, default, founder change or mutation policy.
- Open user decisions (draft, 2026-09-30): D1 was settled on 2026-10-01 (the observability exemption, see Verification above); D2, the probe sample size and whether this track shares T21's 10% ceiling or gets its own; D3, where the track sits in the order of new starts relative to T22 and T20; D4, whether any reading (movement against chance is the obvious candidate) should also enter the goal profile and `docs/progress.md` as a stored cognition indicator, which would be a separate change to T01/T14 and the workflow; D5, whether F08's sampled reading is enough to drop F02's every-move counters. D6 (where the viewer lives) was settled on 2026-09-30: `telemetry/reports/creature-windows.html`, mounted into the stack and served by Grafana at `/public/petri/`, reads any run's windows from Tempo and its identity from Loki through Grafana's data source proxy, computes the F08 readings in the page, and is linked from the run dashboard and the runs list. F09 is that page made durable: tests for its readings against a fixed trace, and behavior on runs with thousands of windows; once F10 lands it reads retained windows from Loki instead of Tempo, which removes its dependence on T21.F04's lifetime window budget. D7 (user decision, 2026-10-01): the F10 store's per-run byte cap is 500 MB. At about 4 KB a record that holds about 125,000 records, which with the 10% floor covers about 1.25 million windows, about 144 days at one window every 10 s. When a run reaches the cap, the job thins the oldest band below the 10% floor so recent windows keep arriving (user decision, 2026-10-01).
