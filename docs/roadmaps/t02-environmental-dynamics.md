# T02 — Environmental Dynamics

**Status**: Planned
**Last updated**: 2026-09-02
**Master**: [Program Roadmap](../roadmap.md)

## Goal

Create temporal and spatial environmental dynamics in which recent history,
latent context, prediction, and robust switching can be adaptive without making
the world an externally scripted puzzle with one permanent solution.

## Track Success Criteria

- [ ] Environmental state can vary globally and locally under versioned deterministic and stochastic schedules.
- [ ] At least one treatment cannot be solved equally well from the current perception snapshot by a memoryless controller.
- [ ] Disturbance and recovery treatments support measurable resilience, recolonization, and strategy turnover.
- [ ] Memory-dependent advantages persist across independent runs and held-out environmental schedules.

## Executable Features

- [ ] **T02.F01 — Time-Varying Resource Schedules** — Depends on: T01.F01, T10.F01
- [ ] **T02.F02 — Partial and Noisy Environmental Cues** — Depends on: T02.F01
- [ ] **T02.F03 — Local Asynchronous Resource Cycles** — Depends on: T02.F01
- [ ] **T02.F04 — History-Dependent Resource Payoffs** — Depends on: T02.F02, T02.F03
- [ ] **T02.F05 — Disturbance and Recovery Regimes** — Depends on: T02.F01
- [ ] **T02.F06 — Temporal-Memory Selection Characterization Campaign** — Depends on: T02.F04, T09.F01, T10.F08
- [ ] **T02.F07 — Environmental Transfer and Robustness Characterization Campaign** — Depends on: T01.F06, T02.F05, T10.F08

## Notes for AI Agents

- Build on typed foods, fertility layers, occupancy depletion, and annealing. Preserve applied world state as the source of telemetry and perception.
- Simple seasonality plus a perfect current-state sensor is a reactive task. Memory pressure requires partial or noisy cues, delayed consequences, local phase differences, switching costs, or transition effects.
- Keep authored schedules bounded diagnostic tools. Endogenous changing pressures belong to T05 and T06.
- T02.F06 and T02.F07 are characterization campaigns used to qualify tasks and estimate effects. They cannot be cited as confirmatory program evidence; final temporal and transfer claims are tested by the T01.F09-gated T09.F07 campaign.
- Research basis reviewed 2026-09-02: [Environmental memory alters the fitness effects of adaptive mutations in fluctuating environments](https://doi.org/10.1038/s41559-024-02475-9).
- Options considered were global seasons, stochastic variation, local cycles, and external curriculum generation. Start with composable schedules and local cycles because they fit Petri's existing world model and support controlled counterfactuals without adding an external optimizer.
