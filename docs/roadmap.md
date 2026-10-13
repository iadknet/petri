# Diverse Ecosystems and Complex Cognition

**Status**: Planning
**Last updated**: 2026-10-13

Program 2. Program 1 (2026-09-02 to 2026-10-12) is archived with what it
closed, what made evolution start working and the findings it carried
forward in [`docs/archive/program-1/README.md`](archive/program-1/README.md).

## Success Definition

Petri succeeds when its world feels alive: long-running seeded runs sustain
many coexisting ways of making a living, and lineages keep evolving behavior
that depends on remembering, learning, and predicting rather than reacting
alone. Progress is tracked, not proved. A small set of diversity and cognition
indicators, read on the goal profiles at the close of the features that
declare them, rises across the series of closed features, and each rise is
attributed to the feature whose closure moved it.

## Track Roadmaps

- [ ] **T25 — Edge Groups and Per-Edge Learning** — [Roadmap](roadmaps/t25-edge-groups-and-learning.md) — Depends on: None

## Final Success Criteria

- [ ] The goal readings' ecological-diversity indicators (surviving lineage count and lineage entropy first, richer strategy measures as they are added) rise across the series of closed features that declare a goal profile, relative to the series' first reading, and each rise survives at least one later reading.
- [ ] The goal readings' cognition indicators (memory sensitivity first, learning and prediction measures as they are added) rise across the series of closed features that declare a goal profile, relative to the series' first reading, and each rise survives at least one later reading.
- [ ] No severe unjustified compute regression is present in the goal series, and the gate profile still finishes inside `make check`.

## Notes for AI Agents

Standing rules. Execution policy stays in [`docs/workflow.md`](workflow.md).

- Every mechanism feature is grounded in a natural analog and reaches
  creatures through the world or the body, never through a sensor or a
  computation invented for the feature. Only geometry may be built in: which
  sensor field corresponds to which direction. Its roadmap goal line names
  the analog first. Measurement features are exempt.
- Brain and sensorimotor features improve what evolution can find on its
  own; they never target a specific behavior. Specific sensors and scenes are
  yardsticks only.
- Energy is ecological accounting: every new capacity has an applied cost or
  tradeoff owned by the feature that introduces it, unless the user grants an
  explicit exception recorded in the feature's design note.
- Backward compatibility is never a constraint: no existing world, creature,
  genome, artifact, catalog numbering, hash pin or draw sequence constrains a
  design, and nothing is converted or migrated; a running world restarts
  from the founder on the new build. Each feature predeclares the re-pins it
  expects.
- Each spec declares its verification plan (benchmark profiles, mutation
  gate, slow tests) with one reason each, choosing the cheapest plan that
  answers the feature's question. The 2,000-tick goal profile does not
  register long-horizon effects; a feature whose question lives past 20,000
  ticks declares `goal-long`.
- Goal readings are the program's success evidence; a single reading is not
  proof. Lab runs are expensive and run only when a feature's own readings
  cannot answer its question.
- Track IDs organize durable areas of responsibility, not phases. The
  feature dependency graph is the execution order; the next-feature rule in
  `docs/workflow.md` picks from the priority list below.
- Findings carried from program 1 and not yet owned by a row: genome bloat
  at depth with size pressure off; reproduction possibly decoupled from
  condition; edge and VM barrier avoiders that fail live; memory visible only
  past 20,000 ticks; the unbounded steal request, and that every live steal
  requests 0 energy (2026-10-12 probe).

Priority order for new starts: T25.F01, T25.F02, T25.F03, T25.F04, T25.F05,
T25.F06, T25.F07, T25.F08.

- T25 was placed on 2026-10-13 at the user's direction from the
  [edge groups and learning design note](strategy/edge-groups-and-learning-design-2026-10-13.md),
  after a live probe found that only the ring-shaped creature input became
  behavior while the multi-field creature sensors and whole-family bundles
  did not. The note's §8 contracts are what each spec owner reads; the track
  rows are authoritative. Version 2 of the note (eight features) followed
  three Codex review rounds and nine Opus facet reviews on 2026-10-13.
