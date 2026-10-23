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
- [ ] Every row leaves trajectories, populations and draw streams unchanged: any pin that moves is a defect.

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

## Notes for AI Agents

- Architecture: serves [`docs/architecture.md`](../architecture.md) A2
  shared shapes, A3 (the domain graph and its import rule), A5 owners and
  A7 rules 1, 3, 5 and 8. Gaps: G17: `T30.F01`; G7, G16: `T30.F02`; G9, G14
  (sensor caches): `T30.F03`; G10, G11: `T30.F04`; G2 (route resolution and
  jump targets), G12, G15: `T30.F05`; G13 (the edit paths T25.F11 leaves):
  `T30.F06`; G8: `T30.F07`.
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
  for speed); the declared and wired key sets move to genome analysis and
  the census reads them. F03 and T29.F01 both touch `CreatureState`
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
- Scheduling, not prerequisites: the master order puts F01 before T25.F11
  and F02 before T28, as preferences; either integration order works.
- Verification (assistant proposal): each row gate with pins unchanged;
  F01 adds the
  frontend's own tests and the type-parity test.
