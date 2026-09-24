# T18 — Founder Architecture

**Status**: Planned
**Last updated**: 2026-09-23
**Master**: [Program Roadmap](../roadmap.md)

## Goal

Give the founder a modular Graph brain whose sensing, memory, routing and
motor contributions can vary without destroying its other abilities. Functional
segregation into sensory circuits and motor pools is the natural analog; the
profile uses T19's votes and live internal state, with its layout chosen through
bounded comparisons against the current canonical founder.

## Track Success Criteria

- [ ] A specialized Graph founder profile uses the current vote interface and
  demonstrates useful memory and routing contributions through matched lesions.
- [ ] The profile preserves the canonical founder's first-argmax cardinal-food
  steering on the founder fixtures and the 2,000-ring direction check, and can
  eat, move and reproduce in one tick when physiology and action budgets permit.
- [ ] Both profiles have current-substrate readings for changed, silent, dead
  and sterile births, genome size, applied costs, persistence and fertility.
- [ ] The specialized profile becomes the default in F02, with both benchmark
  epochs re-pinned and V3Alpha1 retained as a selectable control.

## Executable Features

- [ ] **T18.F01 — Specialized-Node Founder Profile** — Depends on: T11.F21, T11.F22, T19.F06
  - Goal: Functional segregation of a nervous system: a selectable Graph founder separates useful sensory, memory, routing and motor contributions on the vote surface while preserving foraging competence and permitting reproduction in the same tick.
- [ ] **T18.F02 — Specialized Founder as the Default** — Depends on: T18.F01, T17.F02
  - Goal: Every world starts from the qualified specialized founder on the current unit scale, with V3Alpha1 selectable as the control and all founder-dependent anchors and readings updated.

## Notes for AI Agents

- Scope revision, 2026-09-23: the user-approved roadmap cleanup replaces the
  pre-T19 layout. F01 and F02 remain unscheduled; T19's completion does not
  add them to the master priority order. The earlier decision to introduce a
  separate profile and then make it the default remains split across these
  two features. T13.F08 still depends on F01.
- The [founder refactor research note](../strategy/founder-refactor-research-2026-09-18.md)
  records the original motivation and measurements. Its action-bank and
  execute-gate layouts, fixed node counts, genome sizes and outcome estimates
  are historical evidence, not the design or numerical baseline for F01.
  The [T19 contract](t19-mesh-action-selection-and-live-state.md) governs
  votes, per-kind bars, Decide/Terminate, legal revisits, live internal state
  and frozen external perception. No executor node or chain-order-to-action-order
  assumption carries forward.
- F01 uses existing Graph primitives and inputs. Its just-in-time spec chooses
  a compact layout and compares a router that makes a real decision with a
  serial vote-based alternative. A router repeated by an equivalent downstream
  gate is not a useful contribution. Demonstrate each claimed memory/routing
  role with a matched lesion; authored founder modules are not evolved recruits.
- Express action order through votes and available decision-state inputs,
  including the queue or commit counts where needed. Reproduction targeting
  must account for earlier planned movement while using frozen perception;
  verify the applied sequence, target occupancy and physiology rather than
  assuming node order determines the queue. Gates use the current unit scale
  and actual acceptance region, not the old threshold or transfer constants.
- Predeclare the routed-versus-serial comparison, selection rule, costs and
  expected indicator directions before measuring. Retake founder fixtures,
  births/neighborhood and mesh-execution readings on the current substrate;
  compare both candidates with the current canonical founder in Orchards
  seeds 11 and 12 through 2,000 ticks, including fertility at troughs, rebuilding
  and persistence. Historical success does not establish current viability.
  Keep the default founder and ordinary gate/goal trajectories unchanged in F01.
- F02 changes the default only after F01 delivers its verified profile.
  Update `FounderProfile::default()`, the canonical genome-size and replication
  anchors, startup-seeding reference, and every founder-based indicator
  (neighborhood, births, drift, steering and mesh execution). Re-pin gate and
  goal epochs and read births per creature-tick before and after. Record how
  the new profile handles rejected reproduction without losing its foraging
  opportunity; retain V3Alpha1's own behavioral contract as the control.
- The track changes founder wiring, not reproduction physiology, famine rules,
  sensors, mutation operators, learning mechanisms or runtime scheduling.
  T20 owns general input recruitment and structured variation. Use current
  observation machinery; new founder wiring alone proves neither evolution
  of modularity nor ecological demand for additional decisions.
