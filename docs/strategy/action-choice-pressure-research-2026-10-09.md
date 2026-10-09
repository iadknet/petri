# What pressure makes creatures choose actions that will succeed: research note

**Date**: 2026-10-09 (research date for every external citation)
**Feeds**: the user's question after the failed-action penalty reading of the
same day: if a penalty is not the right answer, what pressure makes creatures
choose the actions that will succeed? This note decides nothing and implements
nothing. It gives the evidence, the candidates, and a recommendation with the
proof of concept that would have to come first.

## 1. Problem and criteria

**Problem.** In the live world at tick 31,588 (penalty 0.02, ramp off,
`reproduce_cost` 0), 61% of logged actions fail by count. Reproduce is a third
of all actions and 98% of attempts are rejected; 68% of attempts are made with
energy under the 30 gate, which the founder itself never does (it attempts only
while `EnergyCurrent` clears its threshold, `creature/founder.rs`). Eat fails
82% of the time (empty cell), Move 19% (barrier or occupied). The dominant
per-tick program is "Move, Reproduce(rejected)". Failures are 9% of energy
spend, so by energy they are cheap; by count they are most of what creatures do.

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
| Rejected reproduce charge | about 10.8 (charge before gate, pre T16.F01) | 0.02 |
| Reproduce share of last-tick actions | 0.9% (1,447 of 161k) | 28% (13,309 of 48k) |
| Reproduce rejections vs births in sampled logs | 109 vs 315 | 10,261 vs 212 |
| Energy sweep switches action kind | 296 of 400 creatures | not re-run |

Sources: [live survey](live-survey-2026-09-16.md) Sections 2 and 3.3, the
antipattern review Section 3, and this session's sample of 296 creatures
(seed 11, uniform over the detail view, logs read without pausing).

Reading: the energy gate the founder carries survived a large per-attempt
charge and decayed under a near-zero one. The two readings are confounded (a
different population, T16.F01, the vote surface, born-connected wiring and
twenty other features landed between them), so this is suggestive, not causal,
and it bounds nothing about a *small* charge: the September charge on a
rejected reproduce was about 10.8 energy, not the 0.45 penalty.

The same survey found the opposite for discriminations the founder does not
carry: at penalty 0.45 a barrier on the chosen direction changed the action in
0 of 12,501 trials, and eat-on-empty sat inside fixed programs. The
[diagnosis run](evolvability-diagnosis-run-2026-10.md) Section 1 names the
bottleneck **variation**: one authored sensor-to-motor edge raises births 20 to
43% in mutation-off competitions, while in the live worlds the same sensor
families are executed on up to 11 of 20 sampled parents and causal on none.

So the question splits:
- **Retention.** Keeping a discrimination the founder already has (the energy
  gate on reproduce) is a cost question, and the record says cost works.
- **Origination.** Getting a discrimination that does not exist yet (barrier
  avoidance, eating only where food is) is a variation question, and no cost
  level in the record has produced one. That is T11/T20 work, not a pressure.

### 2.2 What a careful creature gains today, from the logs

For each sampled creature, every logged tick was replayed with the blind
attempts removed (reproduce with energy under 30; eat that found no food) and
the per-tick net energy compared. This is an energy-side bound: it values a
move at its cost, so it can read only rules that leave the action set
otherwise unchanged.

| rule | blind net/tick | careful net/tick | careful advantage |
| --- | ---: | ---: | ---: |
| A: current (penalty 0.02) | -0.110 | -0.068 | 0.042 |
| B: 0.1 effort price on every rejected reproduce attempt | -0.195 | -0.104 | 0.092 |

271 creatures with at least 20 logged actions; one eat success is +7.05. Under
the current rule, carefulness is worth two basal decays a tick; a 0.1 effort
price doubles that. Whether 0.09 a tick retains the gate against drift is
untested; the September reading only shows that about 10.8 did.

A plan-break rule (a failed step ends the tick's remaining plan) cannot be
valued from a frozen log, because energy accounting counts a successful move
as a pure cost. What the log does give is the rule's footprint on today's
programs, 12,239 sampled ticks:

| reading | value |
| --- | ---: |
| multi-action ticks | 10,940 |
| of which every step succeeded | 156 (1%) |
| ticks with a failure before later steps | 7,455 (61%) |
| steps after the first failure | Move+ 4,610, Eat- 3,097, Reproduce- 2,743, Move- 1,176, Eat+ 625 |
| energy gained by steps after a failure | 3,789 of the sample's 8,067 (47%) |

So almost no current plan is clean, and the rule would cut about half of
today's intake until creatures order or gate their blind steps. It is a
strong pressure, in the direction the multi-action tick exists for; its
magnitude is a reason to read it in a lab arm first.

### 2.3 What the engine already has

- **Effort pricing is already the rule for Move and Eat.** A blocked move costs
  the move (`apply_move`: "energy cost is always deducted even if the move is
  rejected"); an eat on an empty cell costs `eat_cost`. Reproduce is the one
  action whose attempt costs nothing: T16.F01 moved the gate before the charge
  ("an animal that cannot afford a litter does not conceive"), and
  `reproduce_cost` is 0 in the live config. Charging effort on a reproduce
  attempt reopens that rationale and needs its own analog (courtship or
  gamete investment, which is spent whether or not a mate or a litter results).
- **A step ending the tick's plan has a precedent.** T03.F12 ends the parent's
  action queue on conception
  ([spec](../specs/roadmap/t03-f12-genome-length-dependent-reproductive-time.md),
  Commit row). The pass loop commits one action per pass up to
  `max_actions_per_turn` (10) with a per-kind rising bar (`runtime/vote_select.rs`);
  a failure feeds nothing back into the next pass.
- **Outcome learning exists, with a credit-assignment limit.** Reward-modulated
  plasticity reads one of four `OutcomeChannel`s; `ActionSuccess` is the
  success *fraction* over the window (`simulation/outcomes.rs`), not a
  per-action signal. 262 of the 300 sampled genomes carry reward modulation on
  some node (83 nodes read `ActionSuccess`); whether any is load-bearing is
  unverified. A creature can learn "I fail a lot", not "reproduce under 30
  fails".
- **The lab suppresses reproduce by masking** (`min_reproduce_energy` above
  `max_energy`) and keeps the penalty; a masked action still spends the vote
  and the pass.

## 3. Candidates

| | candidate | analog | criterion 1 (differential) | criterion 2 (scales with opportunity) | fit | lab-testable |
| --- | --- | --- | --- | --- | --- | --- |
| a | Effort-priced attempts: a reproduce attempt costs its effort, success or not | courtship and gamete investment | 0.09/tick at 0.1 (energy bound); retention threshold untested | no: flat per attempt | one charge in `apply_reproduce` before the gate; reopens T16.F01 | yes, config-only for reproduce (Section 6 item 3) |
| b | Plan break: a failed step ends the tick's remaining plan (the multi-action tick stays; a plan whose steps all succeed runs in full) | a missed grab means no bite; an open-loop motor sequence cannot continue past a failed step | structural reading only: 61% of sampled ticks have a failure before later steps, and the steps after it carry 47% of the sample's energy gain (Section 2.2), so the rule bites hard on today's programs and leaves a creature that orders or gates its blind steps untouched | yes: the lost steps are whatever the plan had left | keeps the multi-action tick and rewards exactly the plan whose steps were chosen to succeed; T03.F12 precedent; one change in the action-execution loop, no new sensor | proto code, branch-only, policy-deviation |
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
penalty 0: about 90 and 103 invalid actions per episode, return 0.25); any
penalty suppresses them and the task collapses ("setting rinvalid = −1 seems
to have an adverse effect of discouraging exploration by the agent", p. 5–6);
masking solves every map, but agents trained under a mask and evaluated
without it still issue about 129 invalid actions per episode (p. 6). The free
rows are Petri's pattern; the penalty rows are a different failure from
Petri's zero-avoidance tax; the mask-removed row is the evolvability caution:
with masking the environment holds the knowledge, not the genome.

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
  Charnov), and in every working instance one executed step forecloses the
  next in the slice. Petri keeps its multi-action tick, so the transferable
  form is within the tick: a failed step forecloses the plan's remaining
  steps (the plan break of Section 3 and Section 5.2), not one attempt per tick.
- (c) Masking: beats a penalty inside a learner and scales, but the policy
  never internalizes validity.
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
  (survey, 2026-09-16). Zero avoidance means no mutation built the avoidance
  circuit; that is a supply reading, not evidence against the pressure. (The
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
| effort-priced reproduce attempt | `Reproduce` gated on `EnergyCurrent` at the threshold | retention: the founder carries it |
| plan break | a blind step (`Eat` without `FoodHere`, `Reproduce` under threshold, `Move` into `NeighborBarrierRing`) placed last or gated out of a multi-step plan | ordering: retention of the founder's order plus origination of gates |
| plan break, origination targets | `Eat` gated on `FoodHere`; `Move(d)` gated on `NeighborBarrierRing[d]` | origination: the founder does not carry them |

### 5.3 Two instruments per pressure

The decisive control that no past abandonment ran: put the authored solution
under the pressure, mutation off. Two constructions, because the targets
differ in kind:

- **Retention target (the reproduce gate).** The founder *is* the authored
  solution, so the check is the inverse: founder (gated) against founder with
  the gate lesioned, under the price, mutation off. If the gated founder wins,
  the price is the right shape and any later loss of the gate is a supply
  reading. If it does not, the price is too small to matter at that horizon.
- **Origination target (eat gate, barrier gate, plan ordering).** Founder
  against founder plus one authored edge, under the pressure, mutation off,
  in E1's shape (the T20.F01 opportunity assay's `--arm-set`, three goal
  worlds, 8 replicates, 1,000 ticks). If the authored circuit wins and never
  arises on its own, variation is the fault and the pressure stays. If it
  does not win, the pressure is wrong or too weak. If neither it nor the
  shortcut wins, exposure is the fault.

The **variation check** is the second instrument: the same baseline with the
pressure on, mutation on, over generations, read by the why-not ladder
(exposure, supply, viability, benefit, retention) with the candidate mechanism
as its arm. The shortcut shows up as a benefit-rung win for the wrong circuit
and is reported as that, with counts, to the owning track.

Instrument fit, checked in source on 2026-10-09: the opportunity assay takes
its config from the goal case (`opportunity.rs::worlds`) and its arms are
genomes, so a pressure value per arm is not a configuration knob there; the
pressure check in E1's shape is a branch-only `instr:` change. Plain
`v3-cli run --config` takes a saved config but no authored circuit. So the
retention check for the reproduce gate can run as paired `v3-cli run` worlds
today (the founder gate against its lesioned twin needs the lesion authored,
which is code), and "no code" in Section 6 item 3 holds only for the
penalty-on versus penalty-off world pair, not for the authored arms.

### 5.4 Process rules

1. A pressure proposal names its target circuit in sensor-to-sink terms.
   "Creatures choose carefully" is not a target.
2. The authored-solution arm runs before anyone reads the result.
3. A stall is reported as a rung with counts and routed to the owning track,
   as the T22 contract already says; the pressure stays on in the baseline
   while that track works, so the next mechanism feature is read against it.
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
timing guard stands whenever it is tested: it drops 47% of today's intake
(Section 2.2), so its arm reads population survival across the horizon, with
a ramp from zero as the fallback.

## 6. Recommendation

1. **Reframe the question.** Pressure retains discriminations; in this
   project's record it has not created one. "What keeps the energy gate" is a
   cost question. "What lets barrier avoidance arise" is the diagnosis's
   variation verdict and belongs to T11/T20, whatever the pricing.
2. **Price the attempt by its effort, for reproduce as Move and Eat already
   are.** Reproduce is the one action whose attempt is free, and it is the one
   whose gate decayed. PolyWorld is the precedent that effort pricing on
   concurrent attempts yields selective expression, and courtship biology is
   the analog: a mating attempt costs effort whether or not a litter results.
   This reopens T16.F01's "does not conceive, costs nothing" reading; the
   resolution is that the gate still charges no *replication* cost, only the
   attempt's effort, and the charge belongs beside T16.F02 as a T16 row. The
   size of that effort is the open number: the record shows about 10.8
   retained the gate and 0.02 lost it; the logs bound a 0.1 price at 0.09 a
   tick of careful advantage.
3. **Proof of concept before any production change, no code: paired plain
   runs with a small per-attempt cost.** For reproduce the flat penalty and an
   effort price are the same number, because reproduce's effort price is zero,
   so `v3-cli run --ticks N --seed S --config <saved config>` with
   `failed_action_penalty` about 0.1 and the ramp off, against the same seeds
   at the live 0.02, is the test. The reading is
   `reproduction_actions_rejected_by_reason` (on `SimStats`, exported to
   telemetry as `petri.run.reproduction_actions_rejected_by_reason` and to the
   server state) over attempts, per checkpoint, with blocked moves and failed
   eats read beside it since the arm re-taxes them too. The T22 solo lab is the
   wrong instrument for this one: it masks reproduce (`min_reproduce_energy`
   above `max_energy`), so every attempt fails the gate by construction and a
   per-attempt cost there selects for never attempting, the sterility shortcut
   run 2 found and the exploration plan zeroed the refused-reproduce penalty to
   remove. If the gate is retained in the world runs, item 2 is the feature;
   if not, the price is too small or the gate is not under selection at that
   horizon, and the per-checkpoint trajectory says which.
4. **The multi-action tick stays; the balancing pressure inside it is the
   plan break.** The tick rewards a successful multi-step plan; what has been
   missing is the cost of a blind step inside one. "A failed step ends the
   tick's remaining plan" supplies it without touching plan length or the
   per-kind bar: a plan whose steps all succeed runs in full, a blind step in
   the middle forfeits the rest, and a creature that gates or orders its
   steps keeps everything. This is the within-tick form of what Avida and
   Tierra do across a slice (a failed instruction forecloses the next), with
   the T03.F12 queue end as the engine precedent. Its footprint today is
   large (Section 2.2: 61% of ticks, 47% of intake), so it needs a
   branch-only proto arm under the T22 exception, read after item 3 and the
   Section 5.3 pressure check, with
   the reading being the fraction of multi-action ticks whose steps all
   succeed, per checkpoint, beside intake and persistence. Effort pricing
   (item 2) and the plan break are complementary: the first prices a trailing
   blind step, the second prices one inside a plan.

   **The predicted shortcut.** Section 2.1's split applies to this rule too:
   cost retains, variation originates. With 1% of multi-action ticks clean
   today, the cheapest adaptation to a plan break is to issue one or two
   steps per tick, which carries no truncation risk and shrinks the capacity
   the tick exists to reward. That is the shape of run 2's sterility shortcut
   (abandon the capacity rather than refine it). The arm's primary readings
   are therefore actions per tick and the plan-length distribution per
   checkpoint, with a predeclared disposition: a collapse toward single-step
   ticks is the shortcut, not success, and "fraction of clean multi-action
   ticks" alone is gamed by shrinkage. The same holds for effort pricing in
   weaker form: both are "attempt less" pressures until variation can supply
   "attempt right", and the arm must show the plans got cleaner at a held
   length, not shorter.
5. **Masking alone is not an answer.** It removes the waste and the pressure
   with it; Huang's mask-removed agents show the knowledge then lives in the
   environment, not the genome. The lab's reproduce suppression is masking and
   should stay labelled as such.
6. **Do not expect pricing to fix eat-on-empty or blocked moves.** They need
   the sensor-to-motor edge to arise and persist; that is variation.

## 7. Remaining uncertainty

- The September-to-October contrast is confounded; the lab arm in item 2 is the
  controlled version.
- The energy-side bound in Section 2.2 says nothing about the plan-break
  rule's value; only its footprint is read.
- Whether reward modulation on `ActionSuccess` is load-bearing anywhere is
  unread.
- PolyWorld's avoidance evolved for fighting, a behavior with a large per-step
  cost; whether a 0.1 effort price is "large relative to the benefit" for a
  reproduce attempt here is exactly what item 3 of Section 6 reads.
- Holling 1959, Stephens and Krebs 1986 and Cordts and Partridge 1996 were
  not read; claims resting on them are marked secondary.
