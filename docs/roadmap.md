# Diverse Ecosystems and Complex Cognition

**Status**: Active
**Last updated**: 2026-10-27

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
- [x] **T26 — Body Traits** — [Roadmap](roadmaps/t26-body-traits.md) — Depends on: None
- [ ] **T27 — Social Signals and Exchange** — [Roadmap](roadmaps/t27-social-signals-and-exchange.md) — Depends on: T26, T29, T30
- [ ] **T28 — Niche Construction** — [Roadmap](roadmaps/t28-niche-construction.md) — Depends on: T26, T27, T29, T30
- [ ] **T29 — Heredity** — [Roadmap](roadmaps/t29-heredity.md) — Depends on: T25
- [ ] **T30 — Core Boundaries** — [Roadmap](roadmaps/t30-core-boundaries.md) — Depends on: None

## Final Success Criteria

- [ ] The goal readings' ecological-diversity indicators (surviving lineage count and lineage entropy first, richer strategy measures as they are added) rise across the series of closed features that declare a goal profile, relative to the series' first reading, and each rise survives at least one later reading.
- [ ] The goal readings' cognition indicators (memory sensitivity first, learning and prediction measures as they are added) rise across the series of closed features that declare a goal profile, relative to the series' first reading, and each rise survives at least one later reading.
- [ ] No severe unjustified compute regression is present in the goal series, and the gate profile still finishes inside `make check`.

## Notes for AI Agents

Standing rules. Execution policy stays in [`docs/workflow.md`](workflow.md).

- Every mechanism feature is grounded in a natural analog and reaches
  creatures through the world or the body, never through a sensor or a
  computation invented for the feature. Only geometry may be built in: which
  sensor field corresponds to which direction. The one sensed exception is
  self-referent matching: how closely another creature's carried cue
  matches one's own, as an animal smells kin (user, 2026-10-20; today the
  kin-tag affinity). Ordinary condition readings of another body (its
  energy, its readiness) are facts, not comparisons. Its roadmap goal line names
  the analog first. Measurement features are exempt.
- Every track, row and spec follows [`docs/architecture.md`](architecture.md),
  the desired architecture: one owner per concept, dependencies pointing
  down, composition over inheritance.
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
  past 20,000 ticks. The unbounded steal request and the zero-energy live
  steals (2026-10-12 probe) are owned by T27.F03.

Priority order for new starts: T30.F03, T26.F01, T25.F10, T27.F01, T26.F02,
T27.F03, T30.F01, T30.F07, T30.F08, T25.F11, T30.F10, T25.F12, T29.F01, T29.F02, T29.F03,
T30.F02, T30.F04, T30.F05, T30.F06, T30.F09, T30.F11, T30.F12, T30.F13, T30.F14, T27.F02, T28.F01, T28.F02.

- T27.F02 was moved after T29.F03 and T30.F05 on 2026-10-27 at the user's
  direction: "Move T27.F02 after T29.F03 and T30.F05 so all architecture
  debt is paid first". The review found that a priority position alone does
  not enforce this under the concurrent-start rule, so the user made it a
  dependency: T27.F02 and T28.F01 depend on T29.F03 and T30.F05, whose
  chains cover every debt payer in `docs/architecture.md` section 5, and on
  T30.F09 (added 2026-10-27, user: "like T29.F03 and T30.F05"), and
  T28.F02 follows through T28.F01. No remaining new mechanism can start
  before that debt is paid
  ([T27.F02 reorder review](progress/readings/t27-f02-reorder-design-review.md)).

- T30 (core boundaries) was placed on 2026-10-22 at the user's direction
  from the known debts in [`docs/architecture.md`](architecture.md) that no
  row owned, and trimmed to six rows the same day after a simplicity review;
  the same change made T27.F02 depend on T25.F11 (user) and entered T29's
  rows, missing from this order since that track was added. T30.F01 is
  ordered before T25.F11 so the genome refactors change a core read API
  rather than the frontend's copy of genome types, and T30.F07 (added
  2026-10-25 by user decision) moves the genome to its own module before the
  genome rows build on its old path; T30.F08 follows it, so the rows that
  extend creature construction (T25.F11, T25.F12, T29.F01) build on a
  record whose genome and birth caches are already private (assistant
  proposal, 2026-10-26). T30.F03 comes first (user, 2026-10-25):
  it refactors sensors onto explicit views now, rather than guarding the
  whole-creature and genome reads (G9, G20) until it lands, and T27.F02 and
  T29.F02 then put their new sensed state on those views; a row still ordered before its debt's
  payer extends the debt only as `docs/architecture.md` rule 3 of "Use this
  file" allows. The other rows follow
  T29 and precede T28, whose transfers build on T30.F02 and F06. These are
  scheduling preferences, not prerequisites (assistant proposal;
  [design review](progress/readings/t30-design-review.md)).

- T26–T28 were placed on 2026-10-20 at the user's direction from the
  [interaction features design note](strategy/interaction-features-design-2026-10-20.md),
  which records the user's decisions and the prior art. The rows are grouped
  by code surface so rows can run in concurrent sessions, reconciling shared
  catalogs at integration: wave 1 T26.F01, T25.F10 and T27.F01; wave 2
  T26.F02 and T27.F03; wave 3 T27.F02 and T28.F01; then T28.F02. T25.F10 (group
  genes cost as genes) was added to the completed T25 track the same day,
  because it revises T25.F02's size rule. The
  [T26–T28 design review](progress/readings/t26-t28-design-review.md) records
  the Codex plan-design-review and its dispositions.

- T25.F11 (sensor wiring as orthogonal mutations) was added to T25 on 2026-10-22 at the
  user's direction, from the
  [sensor wiring note](strategy/group-creation-unification-2026-10-22.md)
  and the [T25 intent audit](progress/readings/t25-intent-audit-2026-10-22.md):
  three mutation routes built groups by three rule sets, and the most
  frequent one built none, the gap T25 set out to close. Placed before
  T27.F02 (assistant proposal), whose goal-long run reads group counts.
  T25.F12 (learning as one type on edges and bundles) was added the same
  day from the
  [bundle learning note](strategy/bundle-learning-ownership-2026-10-22.md)
  and follows F11.

- T25 was placed on 2026-10-13 at the user's direction from the
  [edge groups and learning design note](strategy/edge-groups-and-learning-design-2026-10-13.md),
  after a live probe found that only the ring-shaped creature input became
  behavior while the multi-field creature sensors and whole-family bundles
  did not. The note's §8 contracts are what each spec owner reads; the track
  rows are authoritative. Version 3 of the note (eight features) followed
  three Codex review rounds and two rounds of nine Opus facet reviews on
  2026-10-13.
