# T27 — Social Signals and Exchange

**Status**: In Progress
**Last updated**: 2026-10-21
**Master**: [Program Roadmap](../roadmap.md)

## Goal

Creatures see each other's colour, broadcast held signals on eight channels
that neighbours perceive, and can give energy to a neighbour as well as take
it, with no meaning or cooperation assigned by the simulation. Design,
options and user decisions:
[interaction features design note](../strategy/interaction-features-design-2026-10-20.md);
prior art: [prior-art note](../strategy/interaction-features-prior-art-2026-10-20.md);
review: [T26–T28 design review](../progress/readings/t26-t28-design-review.md).

## Track Success Criteria

- [x] Each NearbyCreature slot carries the seen creature's red, green and blue, and the creature senses its own; the lineage oracle `LINEAGE_MATCH` and the genotype comparison `PHENOTYPE_SIMILARITY` are retired, and `KIN_AFFINITY` stays as the one self-referent matching sense.
- [ ] A creature holds eight signal values it writes like memory, held until rewritten, and its neighbours read them per slot one tick later; no channel carries an assigned meaning.
- [ ] Giving and stealing each move one fixed bite per action through named ledger flows; a zero transfer is a failure, and the steal amount parameter is gone.
- [ ] Signals and giving run in all three goal worlds, and their goal readings report occurrences and use, including zero.

## Executable Features

- [x] **T27.F01 — Colour Vision** — Depends on: None
  - Goal: Colour vision: a creature sees the red, green and blue of each nearby creature and knows its own, so a visible marking can come to mean kin, rival or prey.
- [ ] **T27.F02 — Held Signals** — Depends on: T27.F01, T25.F10, T27.F03
  - Goal: Song and scent: a creature writes eight held signal channels and its neighbours read them, so meaning can arise from who sends what.
- [ ] **T27.F03 — Giving Energy** — Depends on: T26.F01
  - Goal: Trophallaxis and parental feeding: a creature can pass a mouthful of energy to a neighbour, and stealing takes a mouthful the same way, so care and exploitation are both possible.

## Notes for AI Agents

- User decisions (2026-10-20): eight held signal channels sensed per
  NearbyCreature slot (the display-colour option was declined, so heritable
  colour stays a pure inherited label); the research's one-channel finding
  is recorded in the design note §3 B.
- F01 retires two identity fields (user ruling B, 2026-10-20, after review
  round 5 finding 2):
  - `LINEAGE_MATCH`, an oracle on the founder index that no body could
    sense (flagged by the 2026-09-16 antipattern review; its fix, archived
    T16.F03, was dropped at the program reset);
  - `PHENOTYPE_SIMILARITY`, a comparison of hidden colour genes that reads
    about 1.0 for every pair because all founders share one colour.

  `KIN_AFFINITY` stays: matching another's carried tag against one's own is
  self-referent matching, the master's one sensed exception to the
  geometry-only rule. `REPRODUCE_READY` is unchanged by this ruling. Raw
  neighbour and own RGB leave any colour comparison to evolved computation.
  `lineage_id` stays as observation-only telemetry. NearbyCreature becomes
  72 fields (18 per slot) once F02 adds the eight signals. RGB is derived from the phenotype channels when perception is
  assembled, not cached on the creature, so F01 does not touch the
  constructors T26.F01 reorganizes. F01 also lands the single nearby field
  table with a Rust-written fixture the frontend imports, the input-bucket
  table with sampler/universe agreement coverage (its own RGB inputs are the
  first new self inputs; review round 2 finding 5), and disposes of the
  `PerceptionSnapshot` 256-byte test on a measured tick time, inventing no
  replacement ceiling. Rejected: a colour-distance ring and a retina.
- F02: `signal: [f32; 8]` written by fixed graph sinks and a VM opcode
  reachable from the instruction sampler. Writes are sparse: a channel no
  sink or opcode writes this tick keeps its held value, with no decay.
  Signals start at zero at birth and are applied after the mesh through
  `MeshOutput`; neighbours read last tick's value. Signal writes keep the
  existing graph/VM compute charging, affordability and commit rules, as
  memory writes do, with traced/plain parity; the opcode gets an ordinary
  opcode-cost entry. User decision (2026-10-21): signals carry no energy
  charge of their own and no other mechanism (no sustain cost, decay or
  special range; the same value range as other sink outputs); a creature
  writes sinks and others read them. F02 owns the
  inspector, trace and frontend type surfaces for signals, measures tick
  time, and reports per-channel wiring and value variance. User decision
  (2026-10-20): the eight signal fields are ordinary slot facts, so a slot
  recruitment wires them into the drive with the slot's other facts (16
  facts after F01 retires two fields, so a directed recruitment builds 12
  pair members plus 16 facts). T25.F10 makes that recruitment cost one
  gene rather than 28 silent edges, so F02 waits for it. F02's goal-long run
  also carries T25.F10's deferred reading: group, materialized-member and
  corrected-member counts beside genome size, compute and energy.
- F03 folds in the directed-kind catalog refactor (`VoteSink::Directed`,
  a `voteSinks.json` fixture covering `VoteKind`, `ActionParamField` and
  `OutputSinkKind`, charge arrays indexed by action type). It lands the
  zero-transfer fix as its own commit with a predeclared re-pin. Give is
  lossless: the recipient is credited through T26.F01's cap function, the
  excess booked as clamp loss, and the gift as paired transfer flows. A
  successful give and a failed one each pay the effort floor. Before
  implementing, the spec predeclares a bite-size payoff calculation from the
  recorded victim-hit rates (F09 goal: about 4–9% of steal attempts find a
  victim), so the bite is chosen against the targeting economics rather than
  the amount alone. Expect
  parental care first (Taylor-Davies et al. 2024); do not target it.
- F02 depends on F03 (review round 5 finding 8), so the track's goal-long
  run on F02 includes giving. It reads colour, giving and slot-fact
  discovery together. A 2,000-tick goal reading is an early integration
  reading for every T27 row; zero use there is inconclusive.
- F02's signal writes and next-tick NearbyCreature reads operate in all
  three goal worlds, with a focused write-to-read fixture; the reading
  reports per-channel wiring and value variance, including zero, with no
  pressure tuning. F03 is a creature-authored pressure under the
  [shared baseline contract](../workflow.md#environmental-pressures-in-the-standard-baseline).
- Verification: F01 gate + goal; F02 viability first, gate + goal +
  goal-long (the track's one goal-long carrier); F03 viability first, gate +
  goal. Each spec predeclares its expected changes by artifact: trajectory
  pins, serialization fingerprints and mutation draws.
