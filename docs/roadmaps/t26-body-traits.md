# T26 — Body Traits

**Status**: Planned
**Last updated**: 2026-10-20
**Master**: [Program Roadmap](../roadmap.md)

## Goal

What a creature inherits besides its brain lives in one heritable body
record, the energy-credit cap and movement charge each pass through one body
rule, and energy reserve capacity evolves under a carrying cost and an upkeep
that leave today's capacity neutral. Design, options and user decisions:
[interaction features design note](../strategy/interaction-features-design-2026-10-20.md);
review: [T26–T28 design review](../progress/readings/t26-t28-design-review.md).

## Track Success Criteria

- [ ] Phenotype, kin identity and every later heritable body trait are one record passed as one constructor argument and inherited by one call; the upper energy cap and movement effort each come from one function, with every caller's floor, ordering and signed flows unchanged.
- [ ] Energy capacity is a heritable per-creature trait bounded to a fixed range; movement effort and upkeep are linear in capacity and both neutral at 200, so the founder's economy is unchanged; a newborn provisioned above its own capacity is handled and booked by a stated rule.
- [ ] The goal-long run reports the capacity distribution, boundary mass and its trend per world, and the readings distinguish selection toward a bound from a tradeoff that is broken or effectively free.

## Executable Features

- [ ] **T26.F01 — One Body Record and Body Rules** — Depends on: None
  - Goal: Heredity builds a body as well as a brain, as an egg carries its pigment and markings beside its nervous system: colour and kin identity travel in one inherited body record, and the energy cap and the cost of moving are each one body rule, with no behavior change.
- [ ] **T26.F02 — Evolvable Energy Reserves** — Depends on: T26.F01
  - Goal: Fat: a creature inherits how much energy it can store; a larger store survives lean times but costs more to move and keep, a smaller one is cheaper, and today's capacity is the neutral point.

## Notes for AI Agents

- F01 is behavior-preserving in the T25.F01 shape: any pin that moves is a
  defect. Scope (design note §4, corrected by review findings 2 and 12):
  - the heritable record replaces `CreatureState::new`'s loose phenotype and
    identity arguments on both constructor paths and keeps the phenotype then
    kin-tag draw order;
  - the energy-credit refactor extracts only the common upper-cap operation.
    Eat's zero floor and observe-after-clamp order, predation's
    observe-before-cap order and its signed reverse transfers, and birth's
    direct litter assignment stay as they are, with focused characterization
    tests beside the unchanged pins;
  - one move-effort function replaces the flat `move_cost` charge.
  The `applied_events > 0` gate on phenotype and kin-tag mutation is
  inherited coupling; F01 keeps it.
- F02 (design note §3 D): capacity mutates in log space within [50, 800];
  move effort `× (1 + β·(C/200 − 1))`, upkeep `+ u·(C − 200)`. β and u come
  from a short default sweep, not a lab; the spec validates that no
  coefficient yields negative movement effort. F02 also owns:
  - founder initialization and when a child's capacity mutates relative to
    the litter deducted at conception; a birth whose capacity is below its
    provisioned energy is a required fixture, with the excess handling and
    its ledger bucket stated;
  - its ledger bucket for upkeep, how a negative upkeep nets against decay
    and genome carrying in Phase 0's decay-first partition without
    subsidizing either, and death attribution;
  - sensor normalization. Own-capacity normalization moves inherited
    fullness thresholds: the founder's 0.16 gate opens at 8 energy when
    C = 50. The spec states whether the trait is meant to change
    fullness-based behavior, absolute reproductive condition, or both. It
    runs an unchanged-brain capacity comparison before the β/u sweep,
    reporting reproduction failures separately from any storage benefit.
    Mean energy in the F09 goal run is 24–33, far below 200, so a benefit
    above 200 is unestablished. It also rules on nearby `ENERGY_RATIO` and
    the previous-outcome scalings;
  - inspection: the server and inspector report the creature's own capacity,
    not the global maximum;
  - the global `max_energy`, kept as the evolvable range's upper bound and
    the v3-lab birth sentinel, distinct from a creature's capacity.
  Rejected: a stored-energy move cost (shifts every creature's economy) and
  sub-linear costs (runaway).
- Verification: F01 gate, pins unchanged; F02 viability first, gate + goal +
  goal-long (the track's one goal-long carrier), reading the capacity
  distribution descriptively. Each spec predeclares its expected changes by
  artifact: trajectory pins, serialization fingerprints and mutation draws.
