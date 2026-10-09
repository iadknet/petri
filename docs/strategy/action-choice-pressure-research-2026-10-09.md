# What pressure makes creatures choose actions that will succeed: research note

**Date**: 2026-10-09 (research date for every external citation)
**Feeds**: the user's question after the failed-action penalty reading of the
same day: if a penalty is not the right answer, what pressure makes creatures
choose the actions that will succeed? This note decides nothing and implements
nothing. It gives the evidence, the candidates, and a recommendation; the
executable form is the
[experiment plan](action-choice-pressure-experiment-plan-2026-10-09.md), whose
gates govern everything here. Revised after a Codex adversarial review the
same day (Section 8).

## 1. Problem and criteria

**Problem.** In the user's live world at tick 31,588 (penalty 0.02, ramp off,
`reproduce_cost` 0), in a sample of the living creatures of **one 200 × 200
viewport at (600, 600)** (724 creatures of 17,125 in a 1600 × 1600 world; 296
with action logs), 61% of logged actions fail by count. Reproduce is a third
of logged actions and 98% of attempts are rejected; 68% of attempts are made
with energy under the 30 gate, which the founder itself never does (it
attempts only while `EnergyCurrent`, a fraction of `max_energy`, clears 0.16,
`creature/founder.rs`). Eat fails 82% of the time (empty cell), although the
founder also gates Eat on `food_here` (`Eat = f − 2g − 2q`,
`creature/cgp_founder.rs`). Move fails 19% (barrier or occupied). The
dominant per-tick program is "Move, Reproduce(rejected)". Failures are 9% of
energy spend. These rates are for one neighborhood's survivors over their
logged active ticks (zero-action ticks, gestation holds included, are not in
the logs); a whole-world, fixed-horizon reading is what the experiment plan's
window counters provide. The world-level status counters agree in direction:
reproduce is 28% of last-tick actions (13,309 of 48k).

**The conflict the user named.** The penalty was the intended pressure toward
choosing well; the evolvability research found it a tax that shaped nothing and
the lab found it taxing exploration. Both readings are in the record:
[antipattern review](antipattern-review-2026-09-16.md) Section 2 (31% of action
energy on penalized failures with 0 of 12,501 avoidances) and the
[exploration plan](evolvability-exploration-plan-2026-10-02.md) (a refused
reproduce "forfeits a foraging tick and pays the failed-action penalty").

**Constraints.**
- The natural-analog rule: a mechanism names an analog and reaches creatures
  through the world and body, not through a sensor invented for it.
- The multi-action tick is a required feature (user ruling, restated
  2026-10-09): it is the mechanism that rewards a creature for building a
  successful multi-step plan. No candidate may remove it or reduce it to one
  action per tick. An earlier draft of this note read GP-01's non-goal line
  as the tick being transitional; the user corrected it, and the ruling above
  is what applies.
- Evolvability, not target abilities: the pressure must make choosing well pay
  for whatever the creature can sense, not drive one authored behavior.
- Lab readings before production changes (T22 exploration contract); production
  changes run as roadmap features.

**Decision criteria.**
1. Does the pressure create a fitness difference between a creature that gates
   an attempt on what it can sense and one that attempts blindly, large enough
   to beat drift at the current supply of variation?
2. Does it scale with opportunity (bite when something better was available,
   cost nothing when nothing was), so it does not tax exploration in a barren
   moment?
3. Does it have an analog and fit the engine's existing seams?
4. Can it be tested in the lab before it touches production?

## 2. Local evidence

### 2.1 Cost retains a discrimination; no cost in the record created one

Two readings of the user's own world, one month apart:

| | September 16 (tick 28,315) | October 9 (tick 31,588) |
| --- | --- | --- |
| Failed-action penalty | 1.0 ramped, 0.45 at the reading | 0.02, ramp off |
| Charge on an energy-constraint rejection | about 10.8 (charge before gate, pre T16.F01) | 0.02 |
| Charge on an invalid-target or age rejection | about 0.6 and 0.46 (the ramped penalty with multipliers) | 0.02 |
| Reproduce share of last-tick actions (world status) | 0.9% (1,447 of 161k) | 28% (13,309 of 48k) |
| Reproduce rejections vs successes in sampled logs, all causes | 803 vs 315 (energy 109, invalid target 349, age 345) | 10,261 vs 212 |
| Energy sweep switches action kind | 296 of 400 creatures | not re-run |

Sources: [live survey](live-survey-2026-09-16.md) Sections 2 and 3.3 (400
creatures sampled over the whole world), the antipattern review Sections 2
and 3, and this session's sample (seed 11, uniform over the 724 creatures of
one 200 × 200 viewport, logs read without pausing; not whole-world).

Reading, as a **hypothesis**: reproduce attempts were rare when an
energy-constraint rejection cost about 10.8 and are common now that every
rejection costs 0.02. That is consistent with a charge retaining the
founder's energy gate, but it does not show it: the samples differ in scope
(whole world against one viewport), the populations differ, and T16.F01, the
vote surface, born-connected wiring and some twenty other features landed
between them. The energy sweep counts switching among action kinds, not
retention of the reproduce gate specifically. Nothing here bounds what a
*small* charge would do.

The same survey found no avoidance for a discrimination the founder does not
carry: at penalty 0.45 a barrier placed on the chosen direction never stopped
the creature moving there (0 of 12,501 trials), though it changed some other
part of the action queue in 24. The
[diagnosis run](evolvability-diagnosis-run-2026-10.md) Section 1 names the
bottleneck **variation**: one authored sensor-to-motor edge raises births 20 to
43% in mutation-off competitions, while in the live worlds the same sensor
families are executed on up to 11 of 20 sampled parents and causal on none.

So the question splits, as working hypotheses the experiment plan tests:
- **Retention.** Keeping a discrimination the founder already has (the energy
  gate on reproduce, the `food_here` gate on Eat) may be a cost question; the
  record is consistent with that and does not establish it.
- **Origination.** Getting a discrimination the founder never had (barrier
  avoidance on a move) is a variation question, and no cost level in the
  record has produced one. That is T11/T20 work, not a pressure.

### 2.2 A frozen-outcome action-energy ledger, from the logs

For each sampled creature, the recorded energy deltas of every logged active
tick were summed with the blind attempts deleted (reproduce with energy under
30; eat that found no food), every other action and outcome frozen as logged.
This is bookkeeping, **not a replay and not a bound**: deleting a debit would
in reality change later energy gates and transfers (a later rejected
reproduce could become a conception, which ends the queue and starts a
gestation hold), and deleting an empty eat by its recorded outcome uses
hindsight a sensor gate may not have.

| ledger | blind net/tick | careful net/tick | difference |
| --- | ---: | ---: | ---: |
| A: as logged (penalty 0.02) | -0.110 | -0.068 | 0.042 |
| B: an additional, unscaled 0.1 on each rejected reproduce | -0.195 | -0.104 | 0.092 |
| B': the base penalty on rejected reproduces raised from 0.02 to 0.1, with the engine's age and complexity multipliers (each recorded rejection debit × 5) | -0.181 | -0.097 | 0.084 |

271 creatures with at least 20 logged actions; per tick means per logged
active tick. Neither B nor B' is the experiment plan's effort price (charged
on every attempt, successful ones included, before the gates); they are
rough scales, not predictions. Whether any of them retains a gate against
drift is untested.

A plan-break rule (a failed step ends the tick's remaining plan) cannot be
valued from a frozen log. What the log gives is the rule's **exposure** on
today's logged programs, 12,239 sampled active ticks:

| reading | value |
| --- | ---: |
| multi-action ticks | 10,940 |
| of which every step succeeded | 156 (1%) |
| ticks with a failure before later steps | 7,455 (61%) |
| steps after the first failure | Move+ 4,610, Eat- 3,097, Reproduce- 2,743, Move- 1,176, StealEnergy- 741, Eat+ 625, Reproduce+ (conceptions) 52, StealEnergy+ 36 |
| food intake earned by steps after a failure | 4,308 of 8,797 (49%) as gross food flow (food × reward, the engine's `food_intake` flow, before the max-energy clamp); 3,820 of 8,125 (47%) as energy actually credited |

So almost no current plan is clean, and the historical tail that a plan
break would have removed carries about half of the sample's food intake
(49% gross, 47% credited). That is exposure, not a forecast: under the rule creatures would act
differently. Its size is the reason to read it in a lab arm with a survival
guard first.

### 2.3 What the engine already has

- **Effort pricing is already the rule for Move and Eat.** A blocked move costs
  the move (`apply_move`: "energy cost is always deducted even if the move is
  rejected"); an eat on an empty cell costs `eat_cost`. Reproduce is the one
  action whose attempt costs nothing: T16.F01 moved the gate before the charge
  ("an animal that cannot afford a litter does not conceive"), and
  `reproduce_cost` is 0 in the live config. Charging effort on a reproduce
  attempt reopens that rationale and needs its own analog (courtship or
  gamete investment, which is spent whether or not a mate or a litter results).
- **A queue end exists in the engine, after a success.** T03.F12 ends the
  parent's action queue on a successful conception that starts a gestation
  hold ([spec](../specs/roadmap/t03-f12-genome-length-dependent-reproductive-time.md),
  Commit row). That is the seam a plan break would use, not a precedent for
  ending a plan on failure. The pass loop commits one action per pass up to
  `max_actions_per_turn` (10) with a per-kind rising bar
  (`runtime/vote_select.rs`); cognition builds the whole queue before any
  action executes, so a failure feeds nothing back into the tick's later
  passes.
- **Outcome learning exists, without explicit per-action attribution.**
  Reward-modulated plasticity reads one of four `OutcomeChannel`s;
  `ActionSuccess` is a per-tick success fraction (`simulation/outcomes.rs`),
  multiplied by each edge's eligibility trace (`runtime/plasticity/reward.rs`).
  262 of the 300 sampled genomes carry reward modulation on some node (83
  nodes read `ActionSuccess`); whether any is load-bearing is unverified, and
  whether eligibility traces let a creature learn a state-dependent rule
  ("reproduce under the threshold fails") is unresolved without an assay.
- **The lab suppresses reproduce by masking** (`min_reproduce_energy` above
  `max_energy`) and keeps the penalty; a masked action still spends the vote
  and the pass.

## 3. Candidates

| | candidate | analog | criterion 1 (differential) | criterion 2 (scales with opportunity) | fit | lab-testable |
| --- | --- | --- | --- | --- | --- | --- |
| a | Effort-priced attempts: a reproduce attempt costs its effort, success or not | courtship and gamete investment (Wang et al. 2025) | ledger scale 0.08 to 0.09 per active tick near a 0.1 price (Section 2.2), not a prediction; retention untested | no: flat per attempt | one charge in `apply_reproduce` before the gates; contradicts T16's current invariant that a gate-rejected attempt leaves the parent's energy unchanged, which a feature would have to amend explicitly | yes, as a `proto:` field (experiment plan) |
| b | Plan break: a failed step ends the tick's remaining plan (the multi-action tick stays; a plan whose steps all succeed runs in full) | **a new hypothesis, not an established mechanism**: recovery after a failed attempt (a collision, a missed strike, a refused courtship) occupies the rest of the tick. A missed grab supports a dependent sequence; why a refused reproduce should cancel an unrelated later move is the modeling choice the arm tests | exposure only: 61% of sampled active ticks have a failure before later steps, carrying about half of food intake (49% gross, 47% credited; Section 2.2) | yes: the lost steps are whatever the plan had left | keeps the multi-action tick; uses the T03.F12 queue-end seam; one change in the action-execution loop, no new sensor | proto code, branch-only, policy-deviation |
| c | Masking: an attempt that cannot succeed is not issued | reflex inhibition | zero: it removes the waste and the pressure with it | n/a | removes the symptom; nothing to select | n/a |
| d | Outcome learning: per-action success feedback into plasticity | reinforcement | unknown; the channel is windowed | partial | exists; a per-action channel is a new sensor-like input and needs the analog rule | yes, a channel variant |
| e | Flat failure penalty (the rejected answer) | none beyond effort | 31% of spend at 0.45 with 0 avoidances; a tax | no | retired by T16.F02's goal | already measured |

## 4. External evidence

Primary texts read in full on 2026-10-09 unless marked secondary; quotes are
verbatim with the locator of the copy read.

**PolyWorld, Yaeger (1994)**, *Artificial Life III*; preprint
https://shinyverse.org/larryy/Yaeger.ALife3.pdf. The per-step cost is "the
activation (0. to 1.) of the corresponding output/behavior neuron multiplied by
that behavior's energy-conversion factor, summed over all behaviors" (p. 7),
with no term for whether the behavior had an effect; eating, mating, fighting
and moving are all expressed in the same step from one synchronous brain
update, with no per-step action budget. This is Petri's shape: concurrent
attempts, effort-priced. Under it Yaeger reports one evolved avoidance,
fighting back only when attacked "as opposed to expressing the fight behavior
continuously which would expend unnecessary energy" (p. 14), and one that
stayed rare, ceasing to move while energy rises: "It is still not a
wide-spread phenomenon, though a few instances have now been observed"
(p. 15). So effort pricing on concurrent attempts does yield selective
expression, where the saving is large against the behavior's benefit. This
does not contradict Section 2.1: Move and Eat are already effort-priced in
Petri and no avoidance originated for either, so the difference between
PolyWorld and this world is the supply of variation that can build the
discrimination, which is the diagnosis verdict, not the pricing.

**Avida, Ofria and Wilke (2004)**, *Artificial Life* 10(2), author PDF
https://cse.msu.edu/~ofria/pubs/2004OfriaEtAl.pdf, plus the live `avida.cfg`.
Time is the currency: "Each organism has associated with it a value called
merit. The merit indicates how fast the CPU should run" (p. 203), fitness is
merit over "its gestation length (the number of instructions it executes to
produce an offspring...)" (p. 201), and "Failure of a command means
essentially that the command is ignored" (p. 199) while "If an instruction
fails, it will simply do nothing, but still cost the CPU cycles to execute"
(p. 207). Punishing failures is optional and off (`DIVIDE_FAILURE_RESETS 0`).
The selection against wasted attempts comes entirely from the lost cycle.

**Tierra, Ray (1991)**, *Artificial Life II*, author source
http://tomray.me/pubs/alife2/tierra.tex. "CPU time is thought of as the
analog of the energy resource" (Introduction); a failed template search means
"the JMP instruction will be ignored" (The Tierran Language) but occupied its
slot in the slice, and errors move a creature up the reaper queue, whose
purpose is "to cause algorithms which are fundamentally flawed to rise to the
top of the queue and die" (Mortality). The error penalty culls broken
genomes; Ray claims no avoidance behavior from it.

**Invalid action masking, Huang and Ontañón (2022)**, FLAIRS 35, arXiv
2006.14171v3. With failures free the agent keeps issuing them (Table 2, 10×10,
penalty 0: about 90 and 103 invalid actions per episode, return 0.25). On the
small 4 × 4 map a penalty works (return 40 at −0.01 with about 0.02 invalid
actions); on the larger maps it suppresses invalid actions and the task
collapses ("setting rinvalid = −1 seems to have an adverse effect of
discouraging exploration by the agent", p. 5–6). Masking solves every map;
agents trained under a mask and evaluated without it keep much of their
useful behavior but still issue about 129 invalid actions per episode on
10 × 10 (p. 6). The free rows are Petri's pattern; the penalty rows are a
different failure from Petri's zero-avoidance tax and depend on the size of
the action space; the mask-removed row is the evolvability caution: with
masking, validity is learned incompletely and the environment holds part of
the knowledge.

**Foraging theory, Charnov (1976)**, *Theoretical Population Biology* 9(2),
mirror https://paulseabright.com/wp-content/uploads/2014/08/Charnov-1976-TPB.pdf.
"The predator is assumed to make decisions so as to maximize the net rate of
energy intake during a foraging bout" (p. 131); every activity sits in the
time denominator, and the same currency decides when a male dung fly should
"terminate copulation at a time that maximizes the eggs fertilized/unit time"
(p. 133–134). Opportunity cost of time is the mechanism; a failed attempt as
such is not discussed. Holling (1959) and Stephens and Krebs (1986) were not
accessible; secondary statements of the disc equation charge handling time
per captured prey, so "a failed attack costs handling time" is not verified
here.

**Courtship cost, Wang et al. (2025)**, *Ecology and Evolution* 15(8): e71932,
https://doi.org/10.1002/ece3.71932. "courtship rather than ejaculation is the
main factor responsible for the decreased longevity of mated males"
(Abstract); males "that only displayed courtship behaviours (i.e., introduced
to a mated female) for 5 min" lost lifespan (Results). A mating attempt costs
effort and time whether or not it produces offspring. Cordts and Partridge
(1996) on Drosophila is the standard precedent (secondary; abstract
inaccessible).

**What the sources say about each candidate.**

- (a) Effort-priced attempts: PolyWorld's equation and its evolved
  non-continuous fighting, and the courtship biology, support it; PolyWorld's
  rare grazing warns that it selects only where the saving is large relative
  to the benefit.
- (b) Time as the currency: the most repeated mechanism (Avida, Tierra,
  Charnov), and in every working instance a failed instruction is ignored
  but **costs its own cycle**; none aborts the rest of the slice on a
  failure. The plan break is therefore not their within-tick form; it is a
  new hypothesis (Section 3), and the experiment plan tests it as one.
  Petri keeps its multi-action tick in every case.
- (c) Masking: beats a penalty inside a learner and scales; validity is
  learned incompletely when the mask is removed.
- (d) Outcome learning: thin; no source reports feedback alone producing
  selective attempting.
- Flat penalty: no source reports a success-independent fee producing
  selective attempting; Tierra uses it only as a reaper, Huang shows
  suppression plus collapse.

The full report with every quote and the inaccessible-source log is in the
session scratchpad (`action-pricing-sources-report.md`); nothing in it is
reproduced in the repository beyond this section.

## 5. The pattern behind the question, and the process that breaks it

Added after the user's side chat of 2026-10-09, whose diagnosis the user
agreed with; the figures below were re-verified against the record.

### 5.1 The pattern

A pressure is installed, creatures take the cheapest adaptation to it, the
shortcut looks like a dead end, the pressure is removed. Each removal also
removes what would have rewarded the real solution once variation could
reach it. Three cases in the record:

- **The failed-action penalty.** Dropped after 0 avoidances in 12,501 trials
  (survey, 2026-09-16). Zero avoidance is consistent with no mutation having
  built the avoidance circuit, a supply reading, and is not evidence against
  the pressure by itself; the ladder would name the rung. (The
  flat penalty was still the wrong price, Section 2.1: more than the effort,
  charged equally for failures no sensor could predict. Its intent survives in
  effort pricing and the plan break; the flat fee does not come back.)
- **Run 2's sterility shortcut.** When a refused reproduce cost a foraging
  tick plus the penalty, the accessible gain was abandoning the reproduce
  drive, not gating it ([run 2](evolvability-exploration-2026-10-run2.md)).
  The cheap edit (break the circuit) was available; the good one (gate it on
  energy) was not.
- **The diagnosis run measured the gap directly.** One authored sensor-to-motor
  edge raises births 20 to 43% (E1); a correctly signed edge is proposed at
  5.5 × 10⁻⁶ per birth and the silent declaration is lost 2,564 to 5,073 times
  more often than it is usefully connected (E4). The pressure is there; the
  supply is not.

**Ruling.** A shortcut under a pressure is a measurement, not a verdict: it
names the circuit variation cannot build yet, and routes to the variation
tracks (T11/T20), not to removing the pressure. One exception is timing, not
price: a pressure that kills before variation can act (Orchards, where famine
selected for reproduce lesions and the population died within a few thousand
ticks) is softened or ramped to buy time, and the note says so in those words.

### 5.2 Target circuits, in sensor-to-sink terms

A pressure proposal names what it expects to retain or originate:

| pressure | target circuit | kind |
| --- | --- | --- |
| effort-priced reproduce attempt | `Reproduce` gated on `EnergyCurrent` at 0.16 | retention: the founder carries it |
| plan break, retention target | `Eat` gated on `food_here` (the founder's `Eat = f − 2g − 2q`) | retention: the founder carries it |
| plan break, origination target | `Move(d)` gated on `NeighborBarrierRing[d]` | origination: the founder does not carry it |

### 5.3 Two instruments per pressure

The decisive control that no past abandonment ran: put the authored solution
under the pressure, mutation off. Two constructions, because the targets
differ in kind:

- **Retention targets (the reproduce and eat gates).** The founder *is* the
  authored solution, so the check is the inverse: founder (gated) against
  founder with exactly that gate lesioned, with the pressure off and on,
  mutation off. The reading is the **interaction**: does the gated founder's
  advantage grow with the pressure over its advantage without it? A gate
  that already wins without the pressure says nothing about the pressure.
- **Origination target (the barrier gate).** E1's authored edge against its
  zero-weight twin, with the pressure off and on, mutation off, in E1's
  shape (three goal worlds, 8 replicates, 1,000 ticks), again read as the
  interaction.

Both are short-horizon opportunity readings. A later loss of a gate under
mutation can be supply, viability, retention, mutation load, drift or a
changed ecology; the T22 ladder names the rung with counts, and nothing here
attributes it to supply by default.

The **variation check** is the second instrument: the same baseline with the
pressure on, mutation on, over generations, read by the why-not ladder
(exposure, supply, viability, benefit, retention) with the candidate mechanism
as its arm. The shortcut shows up as a benefit-rung win for the wrong circuit
and is reported as that, with counts, to the owning track.

Instrument fit, checked in source on 2026-10-09: the opportunity assay runs
one config per replicate with genome arms competing inside it, so a pressure
is a replicate-wide treatment and each setting is its own set of
competitions; the lesions are branch-only `instr:` arm sets. The experiment
plan carries the exact constructions and their competence fixtures.

### 5.4 Process rules

1. A pressure proposal names its target circuit in sensor-to-sink terms.
   "Creatures choose carefully" is not a target.
2. The authored-solution arm runs before anyone reads the result.
3. A stall is reported as a rung with counts and routed to the owning track,
   as the T22 contract already says. Within the lab, the pressure stays on in
   the named experimental arms while that track works; production
   availability is governed by Section 5.5 and T22's rule that a lab result
   cannot alone justify production defaults.
4. Sizes are fixed once from the A/A control (run 4 showed six arms
   indistinguishable from an A/A control at quick sizes; E1's sizes were
   quiet), not chosen per run.
5. The number that says a pressure has started working on its own is the
   supply ratio (E4's useful-connect against loss), not the outcome.

### 5.5 Not adopted as a production rule (user, 2026-10-09)

The user agrees with the diagnosis but declines to bake "pressures stay on"
into production, for a reason the record supports: once a cheap shortcut
fixes the population (shorter plans, sterility), the fitness gap between the
shortcut and the real solution is what remains of the gradient, and if that
gap is small the pressure is a barrier to the complex solution, not a reward
for it. So effort pricing and the plan break stay lab assays until two
readings exist per pressure:

1. The authored-solution arm of Section 5.3 (does the real circuit win under
   the pressure, mutation off).
2. An invasion read: the authored circuit introduced into a population that
   has already taken the shortcut. If it cannot invade, the shortcut is a
   barrier and the pressure changes shape before it ships; if it invades,
   the pressure is safe to keep on while variation work runs.

The focus stays on why solutions do not evolve and what the evolution
framework needs (Section 6, item 1, and the queued variation work: the
deletion-rate plan committed 2026-10-09, T11.F29 feedback routes, the
vote-surface hypotheses the diagnosis follow-up left open). The plan break's
timing guard stands whenever it is tested: its historical exposure is about
half of the sample's gross food intake (Section 2.2), so its arm reads
population survival, with a later start tick as the fallback.

## 6. Recommendation

Everything in this section is subordinate to the
[experiment plan](action-choice-pressure-experiment-plan-2026-10-09.md)'s
gates: the authored-solution rows (P1, P2) and the invasion gate (P4). A
positive reading short of those recommends further exploration or a draft
feature; it does not settle production disposition.

1. **Reframe the question.** In this project's record, no pressure has
   created a discrimination; whether a pressure retains one is the open
   question. "What keeps the reproduce and eat gates" may be a cost question.
   "What lets barrier avoidance arise" is the diagnosis's variation verdict
   and belongs to T11/T20, whatever the pricing.
2. **Price the reproduce attempt by its effort, as Move and Eat already
   are.** Reproduce is the one action whose attempt is free. PolyWorld is the
   precedent that effort pricing on concurrent attempts can yield selective
   expression, and courtship biology is the analog: a mating attempt costs
   effort whether or not a litter results. A feature would contradict T16's
   current invariant that a gate-rejected attempt leaves the parent's energy
   unchanged, and would have to amend it explicitly: the gate still charges
   no replication cost, only the attempt's effort. The size of that effort is
   the open number the plan's P1 and P3 read.
3. **The test is the experiment plan, not a pair of plain runs.** An earlier
   draft proposed paired `v3-cli run` worlds at `failed_action_penalty` 0.1;
   that re-taxes failed moves, eats and steals as well, its reading
   (`reproduction_actions_rejected_by_reason`) is cumulative, mixes the
   parent-threshold and transfer rejections, and is not printed by plain
   `v3-cli run`. The plan replaces it with a `proto:` attempt price, window
   counters with the rejection split, and an energy-sweep reading of the
   gate itself. The T22 solo lab stays the wrong instrument for this
   question: it masks reproduce, so a per-attempt cost there selects for
   never attempting.
4. **The multi-action tick stays; the plan break is a hypothesis to test
   inside it.** The tick rewards a successful multi-step plan; what has been
   missing is a cost of a blind step inside one. "A failed step ends the
   tick's remaining plan" would supply it without touching the cap or the
   per-kind bar: a plan whose steps all succeed runs in full, a blind step in
   the middle forfeits the rest. It has no direct precedent in Avida or
   Tierra (a failed instruction there costs only its own cycle) and its
   analog, recovery time after a failed attempt, is the modeling choice under
   test. Its exposure today is large (Section 2.2), so it runs as a
   branch-only proto arm with a survival guard.

   **The predicted shortcut.** The cheapest adaptation to a plan break is
   to queue one or two steps per tick, which carries no truncation risk and
   shrinks the capacity the tick exists to reward. The reading must separate
   the rule's direct effect (an unchanged genome executes a shorter prefix)
   from evolved shrinkage (a genome queues fewer actions), so the plan
   records **queued** length before execution, the executed prefix, the
   break position and the discarded tail, and reads plan capacity from the
   queued length. A collapse of queued length is the shortcut, not success.
   The same holds for effort pricing in weaker form: both are "attempt less"
   pressures until variation can supply "attempt right".
5. **Masking alone is not an answer.** It removes the waste and the pressure
   with it; Huang's mask-removed agents learned validity incompletely. The
   lab's reproduce suppression is masking and should stay labelled as such.
6. **Expect pricing to bear on retention targets only.** The reproduce and
   eat gates are founder circuits a pressure might help keep; barrier
   avoidance needs the sensor-to-motor edge to arise and persist, which is
   variation.

## 7. Remaining uncertainty

- The September-to-October contrast is confounded and compares a
  whole-world sample with one viewport; the experiment plan's P1 to P3 are
  the controlled version.
- The ledger in Section 2.2 is frozen-outcome bookkeeping; it values neither
  the effort price nor the plan break.
- The October rates describe one neighborhood's survivors over their logged
  active ticks; whole-world rates need the plan's window counters.
- Whether reward modulation on `ActionSuccess` is load-bearing anywhere is
  unread.
- PolyWorld's avoidance evolved for fighting, a behavior with a large per-step
  cost; whether a small effort price is "large relative to the benefit" for
  a reproduce attempt here is what the plan's P1 and P3 read.
- Holling 1959, Stephens and Krebs 1986 and Cordts and Partridge 1996 were
  not read; claims resting on them are marked secondary.

## 8. Review dispositions (Codex adversarial review, 2026-10-09)

| # | Finding | Disposition |
| --- | --- | --- |
| 1 | Section 6 bypassed the authored-solution and invasion gates; "stays on" was unrestricted; the price contradicts T16's invariant | Adopted: Section 6 subordinate to the plan's gates; "stays on" limited to experimental arms; the T16 amendment named |
| 2 | Avida, Tierra and Charnov do not establish a failure-triggered plan abort; T03.F12's queue end follows a success | Adopted: the plan break is a new hypothesis with its analog marked as the modeling choice under test; the T03.F12 seam described as a seam, not a precedent |
| 3 | Rule B modelled an additional raw 0.1, not the proposed charge | Adopted: B relabelled, B' added (the reviewer's recomputation, reproduced here exactly), both marked as scales, not predictions |
| 4 | September's 109 was energy rejections only (803 in all); 10.8 is that category's charge; "cost works" overreached; 24 queues did change in the barrier trials | Adopted: matched categories and charges; retention stated as a hypothesis; the 24 changed queues restored |
| 5 | The October sample is one 200 × 200 viewport, survivor- and action-conditioned, zero-action ticks omitted | Adopted: the scope stated wherever the rates appear; whole-world rates deferred to the plan's window counters |
| 6 | The "replay" was frozen-outcome bookkeeping, not a bound | Adopted: renamed; bound claim removed; the dynamics it ignores listed |
| 7 | The founder already gates Eat on `food_here` | Adopted: the eat gate reclassified as retention in Sections 1, 2.1, 5.2, 5.3 and 6 |
| 8 | Applied-action logs cannot separate the break's direct truncation from evolved shrinkage | Adopted: queued length, executed prefix, break position and discarded tail recorded; capacity read from queued length (plan's window counters) |
| 9 | "47% of intake" was positive net deltas; conceptions omitted from the tail | Adopted: food intake stated both ways (49% gross food flow before the max-energy clamp, the reviewer's figure; 47% credited after it, recomputed here); every tail outcome listed; described as exposure, not a forecast |
| 10 | Winning under a pressure does not show the pressure caused it; later loss is not supply by default; plain `v3-cli run` does not print rejection reasons | Adopted: gated or lesioned × pressure off and on, read as the interaction; the ladder names the rung; collection via the plan's window counters |
| 11 | Huang's masking and penalty results, and Petri's learning limit, were overstated | Adopted: penalty failure scoped to the larger maps; "never internalizes" replaced by incomplete; the learning limit restated as no explicit per-action attribution, state-dependent learning unresolved |
