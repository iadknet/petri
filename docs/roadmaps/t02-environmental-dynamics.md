# T02 — Environmental Dynamics

**Status**: In Progress
**Last updated**: 2026-09-23
**Master**: [Program Roadmap](../roadmap.md)

## Goal

Give the world weather, terrain, and history: seasons, places that differ, and
consequences that arrive later, so remembering and predicting can pay without
the world becoming a scripted puzzle with one permanent solution.

## Track Success Criteria

- [ ] Environmental state varies over time and place through natural drivers that reach creatures only through food, terrain, and body cost.
- [ ] At least one production-world condition cannot be solved equally well from the current perception snapshot by a memoryless controller.
- [ ] Disturbance and recovery support measurable resilience, recolonization, and strategy turnover.
- [ ] Memory-dependent advantages persist across independent seeds and held-out weather.

## Executable Features

- [ ] **T02.F01 — Seasonal Resource Dynamics** — Depends on: T01.F12
  - Goal: Seasons. Food regrowth rises and falls on a smooth global cycle with year-to-year variance, felt only through food and metabolic cost and never through a season input, so where food is now is not where it will be.
- [ ] **T02.F02 — Occlusion, Creature Opacity, and Sensory Noise** — Depends on: T02.F01, T12.F01
  - Goal: Terrain blocks sight. On T12.F01's seeded terrain, barriers hide food and creatures behind them, creatures may be opaque too, and sensor noise is an off-by-default treatment, so a creature must act on what it last saw.
- [ ] **T02.F03 — Regional Season Offsets** — Depends on: T02.F01
  - Goal: Latitude and terrain. Spring arrives at different times in different places, so a creature that remembers where it fed last year can move ahead of the season.
- [x] **T02.F04 — Grazing Recovery and Overuse** — Depends on: T12.F04
  - Goal: Overgrazing. Every bite halves the grazed cell's fertility for that food type, repeated bites compound down to a floor, and a bitten cell takes about 500 ticks to recover, so a patch grazed too recently yields less and the value of a place depends on its history and not only on what is visible now.
- [ ] **T02.F05 — Natural Disturbance and Recovery** — Depends on: T02.F01
  - Goal: Fire and flood. Rare stochastic events clear regions of food, which then recolonize, so resilience, recolonization, and strategy turnover can be measured.
- [ ] **T02.F06 — Temporal-Memory Selection Characterization Campaign** — Depends on: T02.F01, T02.F04, T09.F01, T10.F08
  - Goal: Deferred proof phase. Long replicated runs that estimate how strongly the seasonal world selects for memory.
- [ ] **T02.F07 — Environmental Transfer and Robustness Characterization Campaign** — Depends on: T01.F06, T02.F05, T10.F08
  - Goal: Deferred proof phase. Do evolved strategies hold up under weather they never saw?

## Notes for AI Agents

- T02.F01–F05 must add their seasonal, perception, regional, grazing, and disturbance pressures to all three standard goal environments and retain them in later baseline runs. Apply the [shared baseline contract](../workflow.md#environmental-pressures-in-the-standard-baseline); a standalone seasonal or occluded-world condition is insufficient.

- T02.F04 (re-scoped 2026-09-15 at the user's direction, and made the next new start): the grazing pressure is a per-food-type, per-cell fertility modifier in `[floor, 1.0]`, starting at 1.0. On every consuming bite (`OrdinaryFoodState::consume_type` returning a positive amount) the modifier for that type at that cell is multiplied by a `factor` (default 0.5) and clamped to `floor` (default 0.05); each tick it recovers linearly toward 1.0 by `1 / recovery_ticks` (default 1000, so a single bite recovers in about 500 ticks and a floored cell in about 950; the median goal-run generation is 141 ticks, so a grazed patch stays poor across several generations). The modifier multiplies the type's `cell_fertility` in `crates/v3-core/src/kernel/ordinary_food/ecology.rs` wherever fertility is read: local growth, spread into the cell as the target, and recovery spawns. Because a bite zeros the cell and local growth skips empty cells, the modifier acts on the grazed cell's recolonization from dense neighbors and on recovery spawns, not on growth in place; re-grazing therefore happens no faster than recovery lets food return, which is why compounding needs a floor and not a rate limit. Grazing type A does not touch type B's modifier. The existing occupancy depletion layer (`deposit_per_occupied_tick` 0.08, hardcoded 0.03 per-tick recovery, 0.35 growth floor; triggered by standing, not eating) stays on and multiplies beside it: it is a short-term crowding penalty, the grazing modifier carries the history. All four values (`enabled`, `factor`, `floor`, `recovery_ticks`) are config under the shared food settings, surfaced in the runtime panel, never constants. Its dependencies on T02.F02 and T02.F03 were removed on 2026-09-15 (a grazing memory needs neither occlusion nor regional seasons); it depends on T12.F04 for the per-type fertility grids and the three goal worlds it must ship in under the shared baseline contract. It changes the production trajectory, so its gate and goal comparisons move and any severe cost must be predeclared.
- Build on typed foods, fertility layers, occupancy depletion, and annealing. Preserve applied world state as the source of telemetry and perception.
- T02.F01 and T02.F02 follow the T11 foundation sequence in the master execution order and provide its first seasonal, occluded-world evaluation; their ecological dependencies are unchanged. Each spec applies its pressure to all three T12.F04 goal environments within the existing profile budget. Both must show no severe compute regression on the gate profile and record their goal-profile indicator readings at closure. T02.F01's closure reading names the T11.F14 evolved-half route-variation fraction and reads it against a prediction recorded in the [mesh note's Section 10](../strategy/mesh-evolvability-research-2026-09-06.md): Ikeda, Kaneko, and Hatakeyama 2026 find that a fixed environment favors "high penetrance and mutational robustness" while "Frequent environmental change instead favors mutational accessibility at the expense of penetrance," so the fraction is expected to sit near its T11.F15 closure value in the static world and to rise once seasons run; if the season period is a config field, the spec reads it at two periods. This is a closure reading on the existing goal profile, not a sweep and not a new feature. Added 2026-09-07 from the [depth note](../strategy/mesh-depth-research-2026-09-07.md): T02.F01 stays immediately after the T11 supply repair and characterization (T11.F17, T11.F13), not before them, because a static-world reading of a substrate whose mutational exposure has decayed cannot separate the two causes; the evidence that a static world itself selects against evolvability is Canino-Koning, Wiser, and Ofria 2019 ("static environments select solely for immediate optimization, at the expense of long-term evolvability"), Kumawat et al. 2024 (mutation rates declined in constant environments), and Petak et al. 2025 (static populations "climbed to the nearest narrow local optima and stayed there"), and the run's one strong pressure, blocked moves and crowding, was answered by sensors that only 35 of 400 executed cores read.
- Seasons reach creatures through the world, never through a sensor. The season driver is smooth rather than stepped, and years vary (a late spring, a mild winter), because variance is what makes prediction worth more than a fixed rhythm. Simple seasonality plus a perfect current-state sensor is a reactive task; memory pressure comes from occlusion, delayed consequences, regional offsets, and recovery lag.
- T02.F02 builds on machinery that already exists: the world grid has a barrier layer, the sensor visibility contract in `docs/reference/v3-sensor-spec.md` already treats barriers as opaque with line-of-sight ray casting and a strict-corner rule, and from 2026-09-08 seeded terrain generation belongs to T12.F01 (`world.terrain` in the world config, applied at startup), which this feature depends on and no longer implements. The feature verifies the occluded condition across all three standard goal environments, with persistence and passable-area readings, creature opacity as a config option, and sensor noise as a parameter that remains off in production defaults but is explicitly enabled in the three baseline cases. T04.F03 reuses T12.F01's terrain for habitat patches. The re-scope and its evidence are in the [world seeding research note](../strategy/world-seeding-research-2026-09-08.md).
- Authored fixed schedules are experimental treatments, never the production world. Endogenous changing pressures belong to T05 and T06.
- T02.F06 explicitly depends on F01's seasons: F04's grazing no longer supplies that prerequisite transitively. T02.F06 and T02.F07 belong to the deferred proof phase and cannot be cited as confirmatory program evidence; final temporal and transfer claims are tested by the T01.F09-gated T09.F07 campaign if that phase runs.
- Research basis reviewed 2026-09-02: [Environmental memory alters the fitness effects of adaptive mutations in fluctuating environments](https://doi.org/10.1038/s41559-024-02475-9).
- Options considered were global seasons, stochastic variation, local cycles, and external curriculum generation. Start with seasons and regional offsets because they fit Petri's existing world model, feel natural, and support controlled counterfactuals without adding an external optimizer.
