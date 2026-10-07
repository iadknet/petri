# Genome-length-dependent reproductive time: design research for T03.F12

**Date**: 2026-10-06 (research date for every external citation)
**Feeds**: the T03.F12 spec (track [T03](../roadmaps/t03-functional-traits-and-metabolism.md),
row added 2026-10-06). This note decides nothing; it gives the spec owner the
evidence and a recommendation for each of the row's open questions and for the
two the user raised (an extra energy drain during the span; concurrent
gestations). Nothing here is implemented.

## 1. Problem and criteria

The row's phenomenon: a parent that has just reproduced is occupied for a span
that grows with its genome's length, so a long genome pays in reproductive time
at every birth, beside the T03.F11 token surcharge it already pays. The tested
realization is the follow-up diagnosis run's N1 cell
([note](evolvability-diagnosis-run-2026-10-followup.md)): a committed
`Reproduce` that spawns at tick *T* holds the parent on ticks *T*+1 …
*T*+⌈size/97⌉ (the parent commits no action, cognition still runs and is
charged, death cancels the hold). Validated on Canyon seeds 1022 and 2022 over
20,000 ticks: window-mean genome size 4.57 × and 9.14 × the founder against the
default's 7.09 × and 22.38 ×, persistence on both seeds, population 0.55 × and
2.19 × the default's.

Open questions the row leaves to the spec, plus the user's two:

1. Form: a post-birth hold (N1), a pre-birth gestation (the child appears at
   the end of the span), or a reduced action rate.
2. Whether cognition is charged during the span.
3. The unit: the founder's size or a per-unit fraction of a tick.
4. Whether the span carries an additional energy drain.
5. Whether a creature can gestate more than one birth at a time.

Criteria, in order: the natural-analog rule (reaches the creature through
world and body, no feature-specific sensor); not re-using a lever already
shown not to bound size; the smallest realization that delivers the
phenomenon; what N1 did and did not validate; determinism and instrument
continuity (births per creature-tick, the T14.F12 funnel, the viability test).

## 2. Local evidence

- **Economy today** (`crates/v3-core/src/config/simulation.rs`): decay 0.5
  energy per tick; carry cost 1e-4 per unit per tick (T03.F08); the replication
  surcharge multiplies the 0.1 reproduce charge by `1 + 0.1 × (size − 97)`
  (T03.F11), that is 0.01 energy per unit above the founder per birth; the
  activity cost on VM steps (T03.F10) is charged with cognition. Canyon at tick
  20,000 (E2 via the E3 cell): mean living age 34.2 ticks, mean living energy
  33.7, per-birth energy 22.4.
- **The surcharge alone does not bound size** (diagnosis run E2 and E8: goal
  seeds 16 × and 20 × the founder at 20,000 ticks, 118 × at 50,000, Canyon
  extinct at 36,652). At 22 × the founder the surcharge is already 20 energy
  per birth, 91 % of a Canyon birth's energy, and size still grew. Energy at
  the birth is therefore not the channel that bounds size here.
- **What the N1 hold costs in energy, implicitly.** With no eating possible
  while held, the span drains decay plus carry plus the cognition charge
  (cognition charge omitted below; it is a further drain, not a saving):

  | genome | hold (ticks) | decay + carry over the hold | share of a Canyon birth (22.4) | share of mean life (34.2 ticks) | T03.F11 surcharge per birth |
  |---|---|---|---|---|---|
  | 97 (founder) | 1 | 0.51 | 2 % | 3 % | 0 |
  | 485 (5 ×) | 5 | 2.74 | 12 % | 15 % | 3.88 |
  | 2,134 (22 ×) | 22 | 15.7 | 70 % | 64 % | 20.4 |

  The hold's energy drain is smaller than the surcharge at every size, yet the
  hold reduced size where the surcharge did not. The inference from N1 with
  E2 and E8 is that the hold's channel is time: foregone intake, foregone
  moves, a fixed share of a 34-tick life spent committing nothing. N1
  measured aggregate births per creature-tick and population, so it cannot
  separate a per-genome-size mechanism from density compensation; this is an
  inference, not a measured mechanism, and it is the reasoning behind the
  answer to question 4.
- **Prototype** (branch `worktree-evolvability-diagnosis-2`,
  `crates/v3-core/src/simulation/tick.rs`): `held_until_tick` on
  `CreatureState`, set after a spawning `Reproduce` in `execute_reproduce`,
  checked once at the top of the creature's Phase 2 slot (`continue` when
  held); `reproduce_hold_ticks(size) = size.div_ceil(97)`; about 35 lines plus
  tests. It is default-off and gated by a bool; the production feature wants a
  rate.
- **Related mechanics.** `apply_reproduce` resolves the target cell (Step 1),
  validates it (2), checks the population cap (3), the age gate (4), computes
  and deducts the charge (5, 8), checks `min_reproduce_energy` (6), sizes the
  litter (7, T17.F01), builds and mutates the child (9 to 12), and spawns it
  (13 to 14). A creature's tick output is a queue of actions; the prototype
  lets the rest of the birth tick's queue run and holds from the next tick.
  Orchards' earlier collapse was sterility, not starvation
  ([note](orchards-collapse-2026-09-17.md)); T17.F01 and T17.F02 reversed it.
  Any mechanism that lengthens the inter-birth interval is a candidate to
  re-trigger it, and N1 ran on Canyon only.
- **Penalty tax**: failed actions cost 31 % of action spend
  ([antipattern review](antipattern-review-2026-09-16.md)); a design that
  rejects a `Reproduce` during the span must say whether it draws the penalty.
- **Pins**: the founder's size is `FOUNDER_GENOME_SIZE_UNITS = 97` in
  `crates/v3-core/src/creature/founder.rs`; T03.F11 anchored its formula on it
  rather than a literal. A new `RuntimeConfig` field breaks the v3-cli recipe
  digest pin test (T19.F01 closure).

## 3. External evidence

- **Avida** (Ofria & Wilke 2004, *Artificial Life* 10(2):191–229,
  [publisher record](https://gpbib.pmacs.upenn.edu/gp-html/ofria_2004_AL.html);
  Lenski et al. 2003, *Nature* 423:139–144, Methods, as the row cites):
  gestation time is the number of instructions an organism executes to produce
  an offspring; fitness is merit over gestation time. The copy loop is serial
  (one instruction copied per iteration), so copy time grows with length, the
  offspring exists only at `divide`, an organism killed mid-copy loses the
  partial copy, and one offspring is produced at a time. Avida's energy model
  charges per instruction, so a longer copy also costs proportionally more
  energy, which in Petri is T03.F11.
- **Tierra** (Ray, *Tierra* documentation, "the Tierran operating system",
  [node8](https://www.tomray.me/pubs/tierra/node8.html)): each creature's time
  slice is proportional to its genome size raised to a power; power 1 is
  "size neutral" (an instruction's chance of executing does not depend on the
  creature's size), below 1 favors small creatures. With a serial copy loop,
  copy time is proportional to length under the neutral slicer. One daughter
  at a time.
- **Cell biology** (Gregory 2001, "Coincidence, coevolution, or causation? DNA
  content, cell size, and the C-value enigma", *Biological Reviews* 76:65–101,
  [publisher abstract](https://resolve.cambridge.org/core/journals/biological-reviews/article/abs/coincidence-coevolution-or-causation-dna-content-cell-size-and-the-cvalue-enigma/D89556A9876A29239E048A55C0143A8E)):
  the nucleotypic effect; genome size correlates positively with cell-cycle
  time across plants and animals, with DNA replication as a rate-limiting
  step. This is the natural analog the row names, stated at the level of the
  cell, which is the level a Petri creature sits at.
- **Cost of bearing offspring** (Ginther, Cameron, White & Marshall 2024,
  "Metabolic loads and the costs of metazoan reproduction", *Science*
  384:763–767, [NSF public copy](https://par.nsf.gov/servlets/purl/10559374)):
  across 81 metazoans the indirect cost of reproduction, the metabolic load of
  bearing offspring, exceeds the energy in the offspring; in mammals about
  90 % of the cost is indirect. The analog for "the span itself is costly" is
  the organism's ordinary metabolism continuing while it bears, which Petri's
  per-tick decay and carry cost already realize during a hold.
- **Concurrent gestation in nature**: superfetation is rare and hormonally
  blocked in humans and most mammals, documented in a few species (badgers,
  mink, some rodents and lagomorphs), and absent from the serial copy loops of
  Avida and Tierra. The default across the analogs is one at a time.
- **Other ALife platforms**: Polyworld (Yaeger 1994) mates with an energy
  transfer and no gestation span; no platform I found scales a reproductive
  delay with genome length other than the Avida and Tierra copy loops. Searched
  2026-10-06; this is an absence of findings, not a survey.

## 4. Options and recommendation per question

### Q1 Form: hold, gestation, or reduced rate

| option | what N1 says | extra brake | complexity | instruments |
|---|---|---|---|---|
| **A. Post-birth hold** (child at *T*, parent idle *T*+1 … *T*+h) | validated | none beyond time | prototype, ~35 lines | unchanged |
| **B. Pre-birth gestation** (commit at *T*, parent idle, child born at *T*+h) | rate-equivalent to A in steady state | the parent's death during the span loses the child | pending-birth state, delivery step, lost-litter accounting, cell fallback | births counted at delivery, unchanged in meaning |
| C. Reduced action rate (the parent acts on a fraction of ticks, or may eat and move but not reproduce) | not tested | weaker: only reproduction is rate-limited | a skip rule plus a rejected-action path | a rejected `Reproduce` needs a penalty decision |

A and B produce the same inter-birth interval, h + 1 ticks plus whatever the
brain needs, so per living parent the birth rate is the same; B shifts each
birth later by h and adds one loss: a parent that dies while gestating leaves
no child. That loss is proportional to h and so to size, which is the Avida
analog exactly (a partial copy dies with its parent), and it is the single
thing in B that N1 did not test. Seed 1022's population ratio sat at the 0.5
floor under A alone.

**Recommendation: B, the pre-birth gestation, with the hold's semantics
inside it**, because it is the user's framing, the closer analog (the
offspring exists at the end of the copy, not the start), and its one addition
is the natural one; A is the predeclared retreat written into the spec: if
the goal profile's population check fails under B, the spec falls back to A
without a new decision, since A is validated and differs from B only by the
death-loss. The retreat is free as a decision, not as work: it is a second
implementation pass and a re-run of the gates. C is not recommended: it is
untested and weaker by construction, since a held parent under A or B also
loses its intake and moves while C keeps them, and it opens the penalty
question for nothing.

Sub-decisions B needs, each cheap to decide and costly to leave implicit:

- **When the parent pays.** Recommend at commit: Steps 1 to 8 of
  `apply_reproduce` run at *T* as today (target resolved and validated, cap,
  age gate, charge deducted, `min_reproduce_energy` checked, litter energy
  removed from the parent and held as the pending birth), so the parent's own
  energy input drops at once and the body is the signal. If the parent dies
  during the span the held litter energy is lost; it needs its own
  `energy_flows` bucket so conservation checks still balance (the existing
  death and decay buckets do not describe it). Paying at delivery instead
  would let the parent spend the span's energy twice and would evaluate the
  gates against a parent that has decayed for h ticks, which is a different
  feature.
- **Delivery.** At the parent's Phase 2 slot on tick *T*+h, before the hold
  check: store the *direction* chosen at *T*, not the cell, and resolve it
  against the parent's position at delivery (a tick's queue holds up to
  `max_actions_per_turn` committed actions since T19.F04, so a `Move` can
  follow the `Reproduce` on the commit tick and the parent may no longer be
  where it was); if the resolved cell is occupied or barriered, take the
  first valid adjacent cell in the world's fixed neighbor order; if none, the
  birth is lost (litter energy to the same lost bucket) and the parent is
  released. Re-check the population cap at delivery too (recommend yes, a
  lost birth into the same bucket): other births during the span can fill
  the cap, and letting in-flight gestations exceed it would change a hard
  invariant. No indefinite waiting: a parent cannot be held past *T*+h by
  crowding, which keeps the span a function of size only. Build and mutate
  the child at delivery from the parent's genome (a genome does not change
  during life), so mutation events, `birth_mutation_operators` and the funnel
  keep their birth-time meaning.
- **In-flight energy is a stock.** Between commit and delivery the litter
  energy is in no creature and in no flow. No ledger identity test over
  `energy_flows` was found in `crates/v3-core` on 2026-10-06; if the spec
  adds or relies on one, it needs an in-flight term (the sum of pending
  litters) beside the lost-litter bucket, or the identity breaks on every
  tick with a gestation open.
- **The rest of the commit tick's queue** runs as today (as the prototype
  did); the span starts at *T*+1.
- **Death cancels** the gestation, as in N1, with the loss recorded.

### Q2 Cognition during the span

Keep cognition running and charged, as N1 did, outputs discarded. The brain
keeps perceiving (a bearing animal still senses), T03.F10's activity charge
stays consistent, and the prototype's validation holds for exactly this. Reward
learning sees no action outcomes while held; that is the existing prototype
behavior and needs no new rule. Not charging cognition would make the span a
partial energy refund and was not tested.

### Q3 Unit

h = ⌈size / `FOUNDER_GENOME_SIZE_UNITS` × rate⌉ with a config rate
(`gestation_ticks_per_founder_genome`, f32, default 1.0, 0 disables; name is
the spec's). At rate 1.0 this is N1's ⌈size/97⌉: the founder pays one tick, a
twice-founder genome two. The founder paying one tick, not zero, is what N1
validated, matches Avida (a founder has a non-zero gestation), and keeps the
mechanism size-neutral in Tierra's sense only above the anchor. Pin the anchor
against the founder assertion, as T03.F11 did, not against a literal. A
per-unit fraction of a tick (h = ⌈size × k⌉) is the same formula with
k = rate / 97 and adds nothing.

### Q4 Additional energy drain during the span

**No explicit drain.** Three reasons, the first decisive:

1. Energy at the birth is the lever already shown not to bound size (E2, E8:
   91 % of a birth's energy at 22 × the founder, size still growing). The
   hold reduced size with a smaller energy footprint (Section 2 table), so its
   effect is time, and an added drain would re-use the failed lever.
2. The span already drains: decay, carry and the cognition charge continue
   with no intake, 12 % of a Canyon birth at 5 × the founder and 70 % at 22 ×.
   This is the Science 2024 analog realized by the existing metabolism, not a
   new rule.
3. A drain would double-count T03.F11 (the per-unit energy cost of copying)
   and add a parameter with no independent reading.

The spec should state the implicit drain as a derived quantity so the goal run
can read it (lifecycle decay and carry over held creature-ticks are already
partitioned flows; held creature-ticks is a one-counter addition).

### Q5 Concurrent gestations

**No, by construction.** Under A or B the parent commits nothing during the
span, so a second `Reproduce` cannot be committed and no rejection path or
penalty decision exists. Analog: the serial copy loop (Avida, Tierra) and the
hormonal block on a second conception in most mammals. Mechanically, stacking
would let an energy-rich long genome queue births and void the rate limit,
which is the feature's whole effect. If the spec ever takes option C (eat and
move allowed, reproduce not), a `Reproduce` during the span must be rejected
and the spec must decide whether it draws `failed_action_penalty`, given the
31 % penalty tax; recommend no penalty in that case, since the brain has no
sensor for the span and the natural-analog rule forbids adding one.

## 5. Implicit requirements for the spec

- Natural-analog rule: no gestation sensor or input; the creature feels the
  span through its energy and age inputs only.
- Delivery happens at the parent's Phase 2 slot, so tick-order determinism and
  cross-process reproducibility are untouched; a `Simulation` with
  gestations in flight must serialize them (checkpoint round-trip test).
- Births, `offspring_spawned_count`, births per creature-tick, the funnel and
  the census read at delivery, so their definitions do not move.
- Telemetry: gestations started, delivered, lost to death, lost to no cell,
  held creature-ticks; nothing else.
- `cargo test -p v3-core --test viability` first (the founder now pays one
  tick per birth); then the goal profile reads mean genome size at the goal
  checkpoints as a smaller multiple of the founder than the previous closure,
  size per generation lower, population not under half the previous
  closure's, Canyon persisting at its goal seed past tick 36,652, and
  Orchards' plateau and final population against the T17.F02 closure, since
  Orchards is the sterility-sensitive world N1 never ran.
- Move the recipe digest pins in the first implementer brief.

## 6. Remaining uncertainty

- B's death-loss is unmeasured. Its size is bounded above by the share of
  parents dying within h ticks of a commit; at Canyon's 34-tick mean life and
  h of 1 to 5 on the sizes N1 reached, that is a minority of births, but the
  goal run is the reading. The retreat to A costs no new decision.
- N1 is two seeds of one world over 20,000 ticks, size still growing on one
  seed; the feature's closure question (does size level off) is the goal
  run's, as the row says.
- The cognition charge during the span is not in the Section 2 table; it
  raises the implicit drain and is read from the existing `action_charges`
  and cognition flows in the goal run.
