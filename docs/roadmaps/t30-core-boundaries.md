# T30 — Core Boundaries

**Status**: Planned
**Last updated**: 2026-10-22
**Master**: [Program Roadmap](../roadmap.md)

## Goal

The code matches the desired architecture's domain graph: no import cycle
between `v3-core` domains, each concept with the one owner
[`docs/architecture.md`](../architecture.md) names, and the frontend reading a
brain view the server owns rather than the genome's internal layout. Every row
closes gaps listed in that file's A8 and changes no simulation behavior.

## Track Success Criteria

- [ ] `v3-core` has no import cycle between the domains of `docs/architecture.md` A3, and none of the A8 gaps this track's rows name remains.
- [ ] The creature inspector and the sampler read a server-owned brain view whose frontend types are generated from, or checked field by field against, the Rust types; no frontend file copies a genome type or recomputes a genome rule.
- [ ] Every row but F10 leaves trajectories, populations and draw streams unchanged: any pin that moves is a defect; F10 moves draw streams by design and predeclares its re-pins.
- [ ] Each mechanism draws from its own named stream allocated at run start; no stream seeds another during a run.

## Executable Features

- [ ] **T30.F01 — Inspector Brain View** — Depends on: None
  - Goal: A person inspecting a creature sees its brain as the simulation applies it, every weight and learner resolved by the server, with no behavior change.
- [ ] **T30.F02 — Energy Ledger Below Runtime** — Depends on: None
  - Goal: A creature spends energy and dies by one accounting whether it thinks, acts or ages, with no behavior change.
- [ ] **T30.F03 — Sensors Read Views** — Depends on: None
  - Goal: A creature perceives exactly what it perceived before, from explicit views of the world and of each body, with no behavior change.
- [ ] **T30.F04 — Config and World Without Cycles** — Depends on: None
  - Goal: Worlds are configured, painted and stocked with food exactly as before, by layers that build on each other in one direction, with no behavior change.
- [ ] **T30.F05 — Genome Owns Its Analysis** — Depends on: None
  - Goal: Offspring mutate exactly as before, with every random choice drawn by mutation and every structural fact read from the genome's own analysis, with no behavior change.
- [ ] **T30.F06 — Node Edit Transaction** — Depends on: T25.F11
  - Goal: Offspring carry exactly the sensors and wiring they carried before, every edit to a node's declarations and their uses kept consistent by the node itself, with no behavior change.
- [ ] **T30.F07 — Runtime Owns Runtime State** — Depends on: T29.F01, T30.F02
  - Goal: Creatures think and inherit exactly as before, their running brain state owned by brain execution and the parent's dispatch history reaching mutation through the birth transaction, with no behavior change.
- [ ] **T30.F08 — Scheduling and Mechanics** — Depends on: None
  - Goal: Creatures act exactly as before, each action resolved as one transaction by the simulation's mechanics while the tick only orders them, with no behavior change.
- [ ] **T30.F09 — One Applied Result** — Depends on: T30.F08
  - Goal: Every action has one outcome that the creature's history, its learning reward and the run's totals all read, with no behavior change.
- [ ] **T30.F10 — Independent Random Streams** — Depends on: T30.F08
  - Goal: Each kind of chance a creature meets (who acts first, how a child varies, where food grows) comes from its own stream, so a change to one mechanism's draws never shifts another's.

## Notes for AI Agents

- Architecture: serves [`docs/architecture.md`](../architecture.md) A2
  shared shapes, A3 (the domain graph and its import rule), A5 owners and
  A7 rules 1, 3, 5 and 8. Gaps: G17: `T30.F01`; G7, G16: `T30.F02`; G9, G14
  (sensor caches), G20: `T30.F03`; G10, G11: `T30.F04`; G2 (route resolution and
  jump targets), G12, G15: `T30.F05`; G13 (the edit paths T25.F11 leaves):
  `T30.F06`; G8: `T30.F07`; G19: `T30.F08`; G18: `T30.F09`; G21:
  `T30.F10`.
- Placed 2026-10-22 at the user's direction ("Aren't all of those good
  things?") from the gaps the architecture document found without an owning
  row; the [design review](../progress/readings/t30-design-review.md)
  records the Codex challenge. Every row is behavior-preserving in the
  T25.F01 shape: any pin that moves is a defect.
- F01 projects today's applied semantics, not the desired model: a scale
  whose contributors differ in rule, rate, reward source and trace, and
  `DirectionPair` fact members outside the scale, are shown as they run, and
  are its preservation cases. T25.F11, T25.F12 and T29.F02 change the view's
  schema with their semantics. The view supplies resolved facts (member
  pattern, composed weight, learner initialization), so the frontend's
  copies of those rules (`memberPattern`, `learnerInitialized` in
  `frontend/src/components/inspector/`) are deleted, not moved. Frontend
  types are generated from the Rust DTOs or held by an exhaustive
  field-by-field parity test; `vote-sinks.json` supplies catalog data only
  and is not that pattern. It covers the sampler route too
  (`v3-server/src/transport/sample_protocol.rs` aliases VM instructions to
  JSON values; `sample_assembler.rs:162` serializes the core instruction).
  The telemetry genome record stays the canonical serialization (A2).
- F02 also splits traces: runtime owns brain-execution trace records and the
  simulation composes tick records (action outcomes, phase timings) around
  them (A5 traces). It keeps every debit's arithmetic and recording.
- F03 keeps the sensor caches computed once per birth (`163a68b0` added them
  for speed); the declared and wired key sets, and the perception
  requirements sensors read from the genome today (G20), move to genome
  analysis; sensors receive the requirements value and the census reads
  the key sets. F03 and T29.F01 both touch `CreatureState`
  construction; concurrent sessions reconcile that file at integration.
- F04 removes the config→patterns→kernel cycle (pattern parameter types
  become config data) and the kernel's import of the creature hash helper
  (`splitmix64` moves to contracts).
- F05: route resolution and VM jump targets, which mutation imports from
  runtime, become genome analysis; the parseability gate moves into the
  genome; mutation picks slice anchors and calls the existing deterministic
  slice functions, keeping candidate order, the empty case and the one
  `gen_range` call, so draws do not move. The brain I/O vocabulary the
  sensor descriptor needs (vote and outcome counts, `OUTPUT_SLOT_COUNT`) is
  T25.F11's, not F05's.
- F06: T25.F11 builds the minimum node transaction its sensor additions need
  (a declaration and its bundles applied together); F06 moves every other
  edit of a node's declarations onto it (topology copies, backend swaps,
  declaration removal). Unused declarations and the parseability gate's
  soft defaults stay supported.
- F07 waits for T29.F01, whose birth transaction hands mutation the dispatch
  history view, and for F02, which removes the trace types' dependency on
  the creature record; mutation keeps resolving the record lazily.
- F08 and F09 were added after the architecture review of 2026-10-22 (user:
  "Yes, add them"). F08 moves the per-action executors, the action energy
  flooring and the removals actions cause out of `simulation/tick.rs` into
  mechanics; gives mechanics narrow inputs instead of the whole
  `Simulation`, with no call back into scheduling; and moves action-cost
  composition into mechanics, the body supplying factors (today's
  `CreatureState::move_effort`). Scheduling keeps tick order, phase
  contracts and stream allocation. Attempt effort stays charged on a
  rejected give or steal (T27.F03's contract). The per-tick reproduction
  RNG seeded from the queue RNG stays as it is, marked
  `WORKAROUND(T30.F10)`, so F08's draws do not move.
- F09 makes mechanics return one applied result per action, carrying the
  attempt effort, and derives the action log, the reward success count and
  run totals from it (A5 applied action result); every derived reading
  keeps today's classification. Delayed deliveries and world-update
  removals are lifecycle records of their own, not action results.
- F10 is the track's one behavior-changing row: scheduling allocates the
  queue, reproduction and food-growth streams at run start, and the
  reproduction stream stops being reseeded from the queue stream each tick.
  It predeclares the moved draw pins and trajectories and runs viability
  first.
- Integration: F08 and F09 overlap T27.F02 (`simulation/tick.rs`,
  `MeshOutput`), T28.F01 (action dispatch, stocks, removal), T29.F01
  (reproduction) and T30.F02 (energy recording, traces); whichever lands
  later extends the applied-result path rather than adding a second one.
- Scheduling, not prerequisites: the master order puts F01 before T25.F11
  and F02, F08, F09 and F10 before T28, as preferences; either integration order
  works.
- Verification (assistant proposal): each row gate with pins unchanged;
  F01 adds the
  frontend's own tests and the type-parity test.
