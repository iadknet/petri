# T02 — Environmental Dynamics

**Status**: Planned
**Last updated**: 2026-09-04
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
- [ ] **T02.F02 — Terrain, Occlusion, and Sensory Noise** — Depends on: T02.F01
  - Goal: Terrain blocks sight. Seeded barrier terrain hides food and creatures behind it, creatures may be opaque too, and sensor noise is an off-by-default treatment, so a creature must act on what it last saw.
- [ ] **T02.F03 — Regional Season Offsets** — Depends on: T02.F01
  - Goal: Latitude and terrain. Spring arrives at different times in different places, so a creature that remembers where it fed last year can move ahead of the season.
- [ ] **T02.F04 — Grazing Recovery and Overuse** — Depends on: T02.F02, T02.F03
  - Goal: Overgrazing. A patch grazed too recently yields less and recovers slowly, so the value of a place depends on its history and not only on what is visible now.
- [ ] **T02.F05 — Natural Disturbance and Recovery** — Depends on: T02.F01
  - Goal: Fire and flood. Rare stochastic events clear regions of food, which then recolonize, so resilience, recolonization, and strategy turnover can be measured.
- [ ] **T02.F06 — Temporal-Memory Selection Characterization Campaign** — Depends on: T02.F04, T09.F01, T10.F08
  - Goal: Deferred proof phase. Long replicated runs that estimate how strongly the seasonal world selects for memory.
- [ ] **T02.F07 — Environmental Transfer and Robustness Characterization Campaign** — Depends on: T01.F06, T02.F05, T10.F08
  - Goal: Deferred proof phase. Do evolved strategies hold up under weather they never saw?

## Notes for AI Agents

- Build on typed foods, fertility layers, occupancy depletion, and annealing. Preserve applied world state as the source of telemetry and perception.
- T02.F01 and T02.F02 follow the T11 foundation sequence in the master execution order and provide its first seasonal, occluded-world evaluation; their ecological dependencies are unchanged. Each spec defines one production-world condition that runs on the T01.F12 goal profile within its budget and is verified there before any richer composition is added. Both must show no severe compute regression on the gate profile and record their goal-profile indicator readings at closure.
- Seasons reach creatures through the world, never through a sensor. The season driver is smooth rather than stepped, and years vary (a late spring, a mild winter), because variance is what makes prediction worth more than a fixed rhythm. Simple seasonality plus a perfect current-state sensor is a reactive task; memory pressure comes from occlusion, delayed consequences, regional offsets, and recovery lag.
- T02.F02 builds on machinery that already exists: the world grid has a barrier layer, and the sensor visibility contract in `docs/reference/v3-sensor-spec.md` already treats barriers as opaque with line-of-sight ray casting and a strict-corner rule. Nothing generates barriers today; the production world is barrier-free except through the paint surface. The feature adds seeded terrain generation with a few knobs for density and shape, a persistence check that the goal-profile world still survives with fewer passable cells, creature opacity as a config option, and sensor noise as an off-by-default treatment parameter. T04.F03 reuses this terrain for habitat patches.
- Authored fixed schedules are experimental treatments, never the production world. Endogenous changing pressures belong to T05 and T06.
- T02.F06 and T02.F07 belong to the deferred proof phase and cannot be cited as confirmatory program evidence; final temporal and transfer claims are tested by the T01.F09-gated T09.F07 campaign if that phase runs.
- Research basis reviewed 2026-09-02: [Environmental memory alters the fitness effects of adaptive mutations in fluctuating environments](https://doi.org/10.1038/s41559-024-02475-9).
- Options considered were global seasons, stochastic variation, local cycles, and external curriculum generation. Start with seasons and regional offsets because they fit Petri's existing world model, feel natural, and support controlled counterfactuals without adding an external optimizer.
