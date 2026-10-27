# T27 — Social Signals and Exchange

**Status**: In Progress
**Last updated**: 2026-10-27
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
- [ ] Giving and stealing each move a share of the source's energy, set by a heritable per-kind share and an effort field that is neutral when unwired, through named ledger flows; a zero transfer is a failure with no kill credit, and the steal amount parameter is gone.
- [ ] Signals and giving run in all three goal worlds, and their goal readings report occurrences and use, including zero.

## Executable Features

- [x] **T27.F01 — Colour Vision** — Depends on: None
  - Goal: Colour vision: a creature sees the red, green and blue of each nearby creature and knows its own, so a visible marking can come to mean kin, rival or prey.
- [ ] **T27.F02 — Held Signals** — Depends on: T27.F01, T25.F11, T27.F03, T29.F03, T30.F05, T30.F09
  - Goal: Song and scent: a creature writes eight held signal channels and its neighbours read them, so meaning can arise from who sends what.
- [x] **T27.F03 — Giving Energy** — Depends on: T26.F01
  - Goal: Trophallaxis and parental feeding: a creature can pass part of its store to a neighbour, and stealing takes part of the victim's the same way; how much is inherited and pushed by effort, so care and exploitation are both possible.

## Notes for AI Agents

- F02 follows `T25.F11` (user, 2026-10-22), so its signal facts join the
  NearbyCreature slot's field set and a `DirectionPair` bundle wires them.
  Held signals: runtime emits the committed writes, the creature record
  holds them for the creature's life, and sensors read the previous tick's
  values. F03's shares become body genes in `T29.F02`.
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
  (2026-10-20): the eight signal fields are ordinary slot facts, so the
  slot's `DirectionPair` bundle wires them into the drive with the slot's
  other facts (16 facts after F01 retires two fields: 12 pair members plus
  16 facts, one gene by T25.F10). F02 waits for T25.F11, which makes the
  bundle the one wiring route (user, 2026-10-22). F02's goal-long run
  also carries T25.F10's deferred reading: group, materialized-member and
  corrected-member counts beside genome size, compute and energy.
- F03 amount (user decisions, 2026-10-21): the user rejected the fixed
  `exchange.bite` of the first implementation (closed at d36f7be6, reverted
  at 58121149) as a global amount no creature shapes. This row supersedes the
  design note's §3 C and takes up the share form of archived T17.F03. The
  amount comes from three creature factors:
  - physiology: one heritable share per kind, `steal_share` and
    `give_share`, in T26.F01's `HeritableBody`. Every genome carries both
    from the founder (start near 0.2), mutated at birth by relative steps and
    cryptic until a give or steal sink is wired. The spec sets the range
    within (0, 1], the start, the bounded-share mutation and its boundary
    handling (T11.F23's `max(|c|, 1)` floor makes steps absolute across this
    range, so it is background, not the rule), and predeclares the moved
    inheritance draws. Direction only picks the
    target: one share per kind, not per direction sink;
  - effort: `StealEffort` replaces `StealEnergyAmount` and `GiveEffort` is
    new, both decoded action-parameter fields reachable through T11.F25's
    decoded-field draws. An unwired field reads 0 and means ×1, as an
    unwired `EatLeaveShare` eats the whole cell; the old amount reads 0
    when unwired, so wiring a steal sink alone requests nothing. The
    modifier is multiplicative and never negative; the spec chooses its
    shape and bounds;
  - state: the amount is share × modifier of the source's current energy,
    the victim's for a steal and the giver's own for a give (Polyworld's give
    is energy × give activation × a configured fraction,
    `TSimulation::GetGiveStatus` in its `src/library/sim/Simulation.cc`).

  The only constant is the existing effort rate. Give and steal share one
  effort rule: the charge grows with the attempted intensity, with the 0.4
  floor on every attempt (user decision: give is graded like steal). A
  no-victim attempt has no absolute amount, so the spec states the charge
  base. Give is lossless: the recipient is credited through T26.F01's cap
  function, the excess booked as clamp loss, and the gift as paired transfer
  flows. Payoff (design review finding 4): at neutral effort, share 0.2,
  population mean energy 24–33 and the recorded 4–9% blind victim-hit
  rates give illustrative gross returns of about 0.2–0.6 per attempt
  against the 0.4 floor. Profitability is unestablished; the spec
  predeclares the payoff calculation under its charge base, the
  complexity and age multiplier and the applied-transfer limits. Expect parental care first (Taylor-Davies et
  al. 2024); do not target it.
- F03 lands the zero-transfer fix as its own commit with a predeclared
  re-pin: a give or steal that moves nothing is a failed action, never
  `Transferred`. A steal on a victim already at or below zero energy
  (exhausted in Phase 1, not yet removed) moves nothing, so it credits no
  kill and no `Predation` death cause. Live telemetry shows these kills
  outnumbering real ones: one run logged 3,824 kills against 2,730 energy
  taken.
- F03 folds in the directed-kind catalog refactor (`VoteSink::Directed`,
  a `voteSinks.json` fixture covering `VoteKind`, `ActionParamField` and
  `OutputSinkKind`, charge arrays indexed by action type). The reverted
  implementation (586605fd..d36f7be6) holds a reusable version of it and of
  the zero-transfer fix. T26.F02 also extends `HeritableBody`; whichever
  lands second adds its field beside the other's.
- Steal under the old decoder evolved late (live telemetry, 2026-10-21):
  21 of 247 runs moved real energy, every one at least 2,566 ticks long and
  most beyond 20,000, with takeoff bursts between ticks about 13,000 and
  48,000 in the runs examined. These observations concern the old amount
  decoder and do not establish onset under F03. The graded steal effort has
  been live since predation landed; T16.F02 retired only the separate
  failed-action penalty.
- F02 depends on T29.F03 and T30.F05 (user, 2026-10-27: "so all
  architecture debt is paid first"); with their chains that covers every
  debt payer in `docs/architecture.md` section 5. It also depends on
  T30.F09 (user, 2026-10-27: "Add T30.F09 as a dependency of T27.F02 and
  T28.F01, like T29.F03 and T30.F05").
- F02 depends on F03 (review round 5 finding 8), so the track's goal-long
  run on F02 includes giving. It reads colour, giving, stealing and
  slot-fact discovery together. A 2,000-tick goal reading is an early
  integration reading for every T27 row: it reports giving and stealing,
  including zero, and the goal-long run supplies the longer-horizon
  reading.
- F02's signal writes and next-tick NearbyCreature reads operate in all
  three goal worlds, with a focused write-to-read fixture; the reading
  reports per-channel wiring and value variance, including zero, with no
  pressure tuning. F03 is a creature-authored pressure under the
  [shared baseline contract](../workflow.md#environmental-pressures-in-the-standard-baseline).
- Verification: F01 gate + goal; F02 viability first, gate + goal +
  goal-long (the track's one goal-long carrier); F03 viability first, gate +
  goal. Each spec predeclares its expected changes by artifact: trajectory
  pins, serialization fingerprints and mutation draws.
