# T30 — Core Boundaries

**Status**: Planned
**Last updated**: 2026-10-22
**Master**: [Program Roadmap](../roadmap.md)

## Goal

The code follows [`docs/architecture.md`](../architecture.md): `v3-core`
dependencies point down with no cycles, each concept has its one owner, and
the frontend reads a brain view the server owns rather than the genome's
internal layout. Every row pays known debts and changes no simulation
behavior.

## Track Success Criteria

- [ ] `v3-core` follows the architecture's dependency order and named prohibitions, and none of the known debts this track's rows name remains.
- [ ] The creature inspector and the sampler read a server-owned brain view whose frontend types are generated from, or tested against, the Rust types; no frontend file copies a genome type or recomputes a genome rule.
- [ ] Every row leaves trajectories, populations and draw streams unchanged: any pin that moves is a defect.

## Executable Features

- [ ] **T30.F01 — Inspector Brain View** — Depends on: None
  - Goal: A person inspecting a creature sees its brain as the simulation applies it, every weight and learner resolved by the core, with no behavior change.
- [ ] **T30.F02 — Brain Execution Stands Alone** — Depends on: None
  - Goal: A creature thinks, spends energy and dies exactly as before, its brain's execution owning its own state and drawing on one shared energy accounting, with no behavior change.
- [ ] **T30.F03 — Sensors Read Views** — Depends on: None
  - Goal: A creature perceives exactly what it perceived before, from explicit views of the world and of each body, with no behavior change.
- [ ] **T30.F04 — Config and World Without Cycles** — Depends on: None
  - Goal: Worlds are configured, painted and stocked with food exactly as before, by layers that build on each other in one direction, with no behavior change.
- [ ] **T30.F05 — Genome Owns Its Analysis** — Depends on: None
  - Goal: Offspring mutate exactly as before, with every random choice drawn by mutation and every structural fact read from the genome's own analysis, with no behavior change.
- [ ] **T30.F06 — Actions as Transactions** — Depends on: None
  - Goal: Creatures act exactly as before, each action resolved as one transaction whose single outcome the creature's history, its learning reward and the run's totals all read, with no behavior change.

## Notes for AI Agents

- Placed 2026-10-22 at the user's direction from the architecture's debts
  that no row owned; trimmed from ten rows to six the same day after a
  simplicity review (user). The
  [design review](../progress/readings/t30-design-review.md) records the
  challenges. Debts each row pays: F01 G17; F02 G7, G8, G16; F03 G9, G14
  (sensor caches), G20; F04 G10, G11; F05 G2 (routes and jumps), G12, G15;
  F06 G18, G19. Every row is behavior-preserving in the T25.F01 shape.
- F01 projects today's applied semantics, not a future model: a scale whose
  contributors differ in rule, rate, reward source and trace, and
  `DirectionPair` fact members outside the scale, are shown as they run and
  are its preservation cases; T25.F11, T25.F12 and T29.F02 change the view
  with their semantics. The core resolves member pattern, composed weight
  and learner initialization through read APIs, so the frontend's copies
  (`memberPattern`, `learnerInitialized` in `frontend/src/components/inspector/`)
  are deleted, not moved into the server. Frontend types are generated or
  held by an exhaustive parity test. It covers the sampler route too
  (`v3-server/src/transport/sample_assembler.rs:162`).
- F02 moves the energy primitives below runtime, moves runtime's state types
  (`GraphRuntimeState`, `DispatchRecord`) out of the creature record, and
  splits traces so runtime keeps brain-execution records and the simulation
  composes tick records around them. Mutation reads the parent's dispatch
  history through a narrow query, resolved lazily as today
  (`simulation/actions/reproduction.rs:392` already passes it explicitly).
- F03 keeps the sensor caches computed once per birth (`163a68b0` added them
  for speed); the declared and wired key sets and the perception
  requirements sensors read from the genome today move to genome analysis.
  F03 and T29.F01 both touch `CreatureState` construction; concurrent
  sessions reconcile that file at integration.
- F04 makes pattern parameter types config data and moves `splitmix64` to
  contracts.
- F05: route resolution and VM jump targets become genome analysis; the
  parseability gate moves into the genome; mutation picks slice anchors and
  calls the existing deterministic slice functions with the same
  `gen_range` call, so draws do not move.
- F06 moves the per-action executors, action energy flooring and the
  removals actions cause from `simulation/tick.rs` into mechanics with
  narrow inputs (no whole `Simulation`), composes action costs there with
  the body supplying factors (today's `CreatureState::move_effort`), and
  returns one applied result per action, carrying the attempt effort, from
  which the action log, the reward success count and run totals derive with
  today's classifications. Attempt effort stays charged on a rejected give
  or steal (T27.F03). Action deaths, upkeep deaths and paint eviction reuse
  one removal-settlement path (`simulation/simulation.rs:102`), with no
  observation logic pulled into mechanics. Delayed deliveries and
  world-update removals stay lifecycle records, not action results. It
  overlaps T27.F02, T28.F01 and T29.F01 in action and reproduction code;
  whichever lands later extends the applied-result path.
- The master order puts F01 before T25.F11 and F02 and F06 before T28, as
  preferences, not prerequisites.
- Verification (assistant proposal): each row gate with pins unchanged; F01
  adds the frontend's own tests and the type-parity test.
