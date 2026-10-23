# T29 — Heredity

**Status**: Planned
**Last updated**: 2026-10-22
**Master**: [Program Roadmap](../roadmap.md)

## Goal

Everything a creature inherits is either a gene or a birth payload, built by
one birth transaction: the body is a chromosome of the genome beside the
brain, mutated by the same per-unit law and counted in genome size, and
whether a child receives its parent's memory is itself heritable. Design,
user decisions and review:
[heredity architecture note](../strategy/heredity-architecture-2026-10-22.md).

## Track Success Criteria

- [ ] One birth transaction builds every creature, founders and children, from a draft (genome, birth payload, provenance) and one finalization step; learned seeds travel in the payload keyed by stable learnable ids, never inside the genome type; energy provisioning and its cap run inside finalization; no trait draws its own randomness outside the mutation policies.
- [ ] Colour, kin tag, capacity, steal share and give share are body genes in `CreatureGenome`, one genome unit each, mutated at the same per-unit rate as brain units with no gate on brain mutation; `lineage_id` is observation-only provenance; the founder is re-anchored and its supply per birth is unchanged.
- [ ] A heritable 16-bit memory mask decides, slot by slot, which of the parent's memory slots a child starts with; the founder passes every slot, and the goal readings report bit frequencies.

## Executable Features

- [ ] **T29.F01 — One Birth Transaction** — Depends on: T25.F12
  - Goal: An egg is a nucleus and its cytoplasm, assembled once: every creature, founder or child, is built by one birth transaction from its genome, a payload of what the parent passes on, and its provenance, with no behavior change.
- [ ] **T29.F02 — Body Genes** — Depends on: T29.F01
  - Goal: One genome encodes both the body plan and the nervous system: colour, kin markings, fat reserves and how much a creature gives or takes are genes that cost what any gene costs and mutate as often.
- [ ] **T29.F03 — Heritable Memory Inheritance** — Depends on: T29.F02
  - Goal: Maternal effects whose strength is itself heritable: a lineage evolves which of its remembered values a child starts with and which it learns afresh.

## Notes for AI Agents

- Architecture: serves [`docs/architecture.md`](../architecture.md) A4
  body chromosome, A5 offspring construction, birth payload, learnable
  identity, birth randomness, provenance, inherited-store policy and energy
  provisioning, and A6 birth. Gaps: G4: `T29.F01`; G5: `T29.F02`; G6:
  `T29.F03`. F01 keeps the dispatch record a named mutation input; G8's
  contracts-level view is `T30.F07`, which follows F01.
- Decisions (user, 2026-10-22): body traits live in the genome under one
  mutation law (multiple choice "Genome, one law"), accepting that capacity
  and the shares mutate about 16 to 18 times less often than today's 0.1 per
  birth; memory inheritance is one 16-bit per-slot mask gene, founder
  all-on (multiple choice "One gene, per-slot mask"); the rows go in a new
  track rather than reopening T26.
- Supersedes T26.F01's criterion that the body is "one record passed as one
  constructor argument and inherited by one call". T26's body rules and
  T26.F02's capacity law are kept. T27.F03's `steal_share` and `give_share`
  become body genes in F02.
- F01 is behavior-preserving in the T25.F01 shape: any pin that moves is a
  defect. Scope from the note: the coordinator in a creature-birth module,
  called by seeding and delivery with world context; the payload object;
  provenance on the zygote; stable ids for every learnable number, carried
  by T25.F12's learning layout, so seeds leave `CgpGraphBackendDef`;
  finalization owns cache reuse on an unchanged genome, resets of per-life
  state, heading and the energy cap; `mutate_kin_tag` and
  `mutate_phenotype` randomness moves into mutation policies with today's
  draw order kept. The parent's dispatch record stays a named input to
  mutation.
- F02 predeclares its re-pins: genome format and equality (the fast path,
  mutation-effect screens and digests include the body), draw streams,
  trajectories, both epochs. The founder grows from 80 to 85 units (F03's
  mask makes 86); each of F02 and F03 re-pins `per_unit_rate` so the
  founder's supply stays 0.485 events per birth, as T25.F02 did, and moves
  the replication-cost and gestation anchors with the founder size. Supply is drawn per chromosome,
  `Binomial(brain units, rate)` and `Binomial(body units, rate)`, so brain
  dispatch is unchanged; a body event picks one body gene uniformly and
  applies today's per-trait step. The `applied_events > 0` gate on colour
  and kin tag is deleted (T26.F01 kept it only to preserve behavior). Kin
  tags then drift about 70 times more slowly; the spec reads `KIN_AFFINITY`
  discrimination beside it.
- F03: the child's mask decides, as the child's `Learning` decides seeds
  (assistant proposal; a maternal-effect reading by the parent's mask is
  the alternative the spec owner may raise). Whether T27.F02's signals join
  the payload under their own mask is T27's decision.
- Open after T25.F11: whether every edit touching a node's sensor
  declarations goes through one node edit transaction (the note's L7).
- Verification (assistant proposal): F01 gate, pins unchanged; F02
  viability first, gate + goal + goal-long (the track's one goal-long
  carrier: body-gene distributions move slowly), reading capacity, share
  and colour distributions beside T26.F02's baseline, kin-tag diversity and
  body events by gene; F03 viability first, gate + goal, reading mask bit
  frequencies. Slow tests run on all three.
