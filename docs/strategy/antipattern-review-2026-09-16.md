# Antipattern Review: What Other Evolutionary Systems Avoid (2026-09-16)

A review of Petri's design surface against named evolutionary systems, asked after the [live survey](live-survey-2026-09-16.md) found one such pattern (the scalar direction decode, now T11.F21). Question: what else does Petri do that established systems explicitly avoid, and what does it cost here?

Method: the reference specs and live config were read in full; four costs were measured on the survey's 400 dumped creatures and 97,018 action-log entries (world at tick 28,315, main at 6e9d4db7); each candidate was then checked against a primary source (Avida's `avida.cfg`, Stanley and Miikkulainen 2002, Yaeger 1994, Huang and Ontañón 2022, Riolo, Cohen and Axelrod 2001). Candidates already owned by a roadmap feature are listed once in Section 5 and not re-argued. This note edits no roadmap file; the main session owns those.

## Verdict

Two findings are glaring by the standard of "measured cost here, avoided elsewhere, no owner". One is an oracle sensor that would fail the project's own rule today, with an unisolated cost. Two are known and cheap. Four suspected patterns turned out clean.

| # | Pattern | Measured here | What others do | Owner | Rank |
| --- | --- | --- | --- | --- | --- |
| 1 | **Failed actions are charged energy** (blocked move, eat on empty cell, invalid reproduce target: `failed_action_penalty` 1.0, ramped, 0.45 now) | 31% of all action energy goes to penalized failures (7,196 of 23,460 over 37,844 creature-ticks, excluding Finding 2's 1,173); a blocked move costs 5x a successful one; zero avoidance evolved | Avida: a failed divide just fails; Polyworld: cost follows effort, not outcome; RL: invalid-action penalty "does not scale", masking is standard | none | **1** |
| 2 | **Reproduce pays its full replication cost before the energy gate** (`reproduction.rs` Step 5 before Step 6) | a parent below the 30-energy threshold pays about 10.8 energy and spawns nothing; 109 such failures against 315 successes (26% of age-and-target-valid attempts) | Avida: divide validity is checked first; a failed divide charges nothing beyond the cycle | none; T03.F11 (replication cost) is the nearest | **2** |
| 3 | **`lineage_match` reads the founder lineage id** (sensor spec 6.5) | live in this world: `lineage_id` is the founder *placement* index, so every seeded founder is its own lineage; identity inputs change the action in 40% of sampled creatures (survey), share due to this field unmeasured | kin recognition elsewhere is phenotypic: Polyworld colors, Riolo tags; Petri's own natural-analog rule forbids a sensor invented for a feature | none | 3 |
| 4 | **Up to 10 actions per tick** (`max_actions_per_turn`) | 89% of ticks run 2 or 3 actions; `Move W, Eat, Move S` is the most common tick: speed is a genome variable bounded only by cost | Polyworld: one continuous step per tick; Avida: about 30 instructions per update, so not universal | `goals.md` lists it as a non-goal to preserve; no roadmap row | 4 (known) |
| 5 | **Priority bid: pay energy for turn order** | 220 of 399 creatures bid on most ticks; median 0.004 per tick, 1.7% of action spend; all-pay | no ALife precedent found; biological all-pay auctions are studied and bids evolve low, which is what happened | none | 5 (low cost) |
| 6 | Input references that change meaning (`InputRef.Swap`, `Remove`) | worst helpful shares after `ChangeEntryNode` (0.446, 0.457) | NEAT never changes an existing connection's endpoints; only weights and the enable bit | main session's T11.F22 proposal | cross-reference |
| 7 | Scalar-decoded categorical parameters | survey | bid-per-action (TPG), one-output-per-action (NEAT) | T11.F21 | owned |
| — | 54-operator zoo with uniform draw | 0.3% dead, 5.5% changed per applied event on the live population; no operator above 6.5% dead | NEAT uses 3 operators, Avida 3 rates | clean | — |
| — | Child inherits parent memory; optional Lamarckian weights | deliberate (T11.F09), analog: maternal effects | | clean | — |
| — | Asexual, single founder | standard (Avida, Tierra) | T08 owns recombination | clean | — |
| — | `kin_tag` drifting family signature | | Riolo, Cohen and Axelrod 2001 tags: an observable, mutable trait | clean | — |

## 1. The world and the data

Same world and sample as the live survey: tick 28,315, 60,074 creatures, median generation 410, 400 creatures sampled with seed 11, each with up to 302 action-log entries carrying `energy_before`, `energy_after`, `result` and `priority_bid`. Config: move 0.1, noop 0.05, reproduce 1.0, decay 0.02 per tick, `failed_action_penalty` 1.0 ramping from 0 at tick 0 to 1.0 at tick 62,680 (0.452 at the reading), `max_actions_per_turn` 10, `min_reproduce_energy` 30, `genome_replication_cost_per_unit` 0.01, age cost up to 20x at age 600.

## 2. Finding 1: failed actions are taxed, and the tax is a third of the action budget

Mean energy change per logged action, pooled over the 400 creatures:

| action : result | mean energy change | n |
| --- | ---: | ---: |
| Move : Success | -0.147 | 50,737 |
| Eat : Success | +0.771 | 35,009 |
| Move : Blocked | **-0.745** | 5,899 |
| Eat : NoFood | **-0.618** | 3,942 |
| Reproduce : InvalidTarget | -0.596 | 349 |
| Reproduce : AgeConstraints | -0.455 | 345 |
| Reproduce : EnergyConstraints | **-10.760** | 109 |
| Reproduce : Success | -24.087 (transfer to the child) | 315 |
| NoOp : Success | -0.067 | 311 |

Budget over 37,844 creature-ticks: eat gains 26,996; total action spend 23,460, of which ordinary successful actions 15,091, penalized failures (blocked moves 4,395, eat on empty cells 2,436, invalid-target and under-age reproduce 365) 7,196, and Finding 2's energy-gate rejections 1,173; priority bids about 446; base decay 757 before the age multiplier. Penalized failures are 30.7% of action spend and 26.7% of eat intake; with Finding 2 the failed share is 35.7%. The blocked-move loss of 0.745 against a 0.147 successful move is the 0.1 move cost plus the 0.452-ramped penalty scaled by the creature's age and complexity multiplier (`debit_failed_action` calls `adjusted_action_cost`), as measured. The ramp is at 45% of its target; at tick 62,680 the same behavior costs about twice as much.

The survey established that none of this is being selected against: a barrier on the chosen direction changes the action in 0 of 12,501 trials, and eating on an empty cell is part of the fixed per-tick program in creatures whose `food_here` sensor is unconsumed (58% of the sample). So the penalty is not shaping behavior; it is a flat tax on the population that is paid most by creatures in crowded, walled, or grazed places, which are the places where the successful clades live.

What others do:

- **Avida.** `avida.cfg` has no energy or merit penalty for a failed operation; the only failure setting is `DIVIDE_FAILURE_RESETS 0` ("When Divide fails, organisms are internally reset"), off by default. A failed divide costs the CPU cycle it consumed and nothing else.
- **Polyworld.** "The total energy expended in a time step is then the activation (0. to 1.) of the corresponding output/behavior neuron multiplied by that behavior's energy-conversion factor" (Yaeger 1994): the cost follows the effort, and "unless an organism encounters a barrier, or the edge of the world, it will move forward by an amount proportional to the activation"; a blocked move pays the movement cost and no more.
- **Reinforcement learning.** Huang and Ontañón (2022) compare masking invalid actions against penalizing them: "Invalid action penalty is able to achieve good results in 4 x 4 maps, but it does not scale to larger maps. As the space of invalid action gets larger, sometimes it struggles to even find the very first reward"; "setting r_invalid = -1 seems to have an adverse effect of discouraging exploration by the agent, therefore achieving consistently the worst performance across maps." Gradient RL is not evolution, but the mechanism is the same: a penalty on an action the controller cannot yet tell apart from a valid one is noise added to the fitness signal, not a gradient toward telling them apart.

The natural analog the penalty was meant to carry (bumping into a wall hurts) is real, but the cost of a bump in nature is proportional to the effort spent, which is what the move cost already charges. The extra 1.0 has no analog and, on the survey's evidence, no selective effect.

## 3. Finding 2: a parent below the reproduction threshold pays the full replication cost for nothing

`simulation/actions/reproduction.rs`, in order: Step 2 target validity, Step 3 population cap, Step 4 minimum age, **Step 5 charge `reproduce_cost x age multiplier x (1 + 0.01 x (genome_size - 111))`**, Step 6 reject if energy is below `min_reproduce_energy` (30), Step 7 reject if the requested transfer cannot be met. Steps 6 and 7 both return `RejectedEnergyConstraints` after the charge has been taken.

For the median sampled creature (genome 475 units, so a 4.64x replication multiplier, further scaled by age) the measured loss on an energy-constraint rejection is 10.76, half the median creature's energy of 21. The logs show 109 such rejections against 315 successful births. A creature with 28 energy that tries to reproduce drops to about 17 and has to eat its way back; a creature that tries again next tick (the fixed programs do) drops toward death. The age gate at Step 4 is ordered correctly (a too-young parent loses only the penalty, -0.455); the energy gate is not.

Avida's divide checks its validity conditions (allocated child size, copied length) before anything is spent; a failed divide is a no-op. Reordering so that the gate is evaluated on `energy - cost` (the same condition Step 6 tests today) and the transfer is checked feasible from `energy - cost` (Step 7's condition) *before* Step 5 charges anything removes the loss without changing the price or the threshold of a successful birth. T03.F11's size brake is unaffected: it is still paid on every actual replication.

## 4. Finding 3: `lineage_match` is an oracle

Sensor spec 6.5: `lineage_match = 1.0 if lineage IDs match, else 0.0`, where `lineage_id` is "assigned from final founder placement order" (startup seeding spec) and inherited unchanged (identity spec). Nothing in the world carries that value; it is read from the creature record. Because each seeded founder individual is its own lineage, the field is live in every world, this one included: it tells a creature whether its neighbor descends from the same placed founder, exactly and for free.

The sensor predates the project's natural-analog rule (2026-09-04), which says a mechanism reaches creatures through the world and body rather than through a sensor invented for the feature; proposed today, it would fail that rule. `kin_affinity` passes that rule: `kin_tag` is inherited and drifts, so it is a tag in the sense of Riolo, Cohen and Axelrod (2001), "a marking, display, or other observable trait" that agents mutate and match against a tolerance; `phenotype_similarity` passes it outright. Polyworld's only identity channel is color, which the organisms' own behavior sets ("the activation level of its mating neuron is mapped onto its blue color component ... this coloration is visible to other organisms").

Measured cost: not isolated. The survey found identity inputs change the action in 160 of 400 creatures, but the perturbation moved all three identity fields together, so the share due to `lineage_match` alone is not known; the dumps carry no `lineage_id`, so how many lineages survive in this world is not readable from them (T14.F04's `surviving_founder_clade_count` reads it in the goal reports). Retiring the field, or deriving it from `kin_tag` distance, is a one-line sensor change and a re-pin of any report that carries it.

## 5. Already owned or deliberate

- Scalar-decoded direction: T11.F21 and the [motor output encoding note](motor-output-encoding-research-2026-09-16.md).
- Meaning-changing input references (`Swap`, `Remove`): the main session's T11.F22 proposal. Precedent added here: NEAT's structural mutations are "add connection" and "add node" only; an existing connection gene's in-node and out-node never change, and weight mutation is a perturbation with a 90% / 10% split (Stanley and Miikkulainen 2002, Section 3.1 and the experimental parameters). Petri's `InputRef.Swap` and `Remove` have no NEAT equivalent.
- Genome bloat and supply: T03.F08, T11.F19, T11.F20.
- Recombination: T08. Population structure and speciation protection: T04 (NEAT's speciation is the explicit form; Petri's 1600² world is the implicit one, and the survey's node-4 sweep shows how little it protects in one founder's clade).
- The chain restarting from the entry node each tick: mesh-evolvability note, item 9 (a design choice; Avida runs about 30 instructions per update with a persistent instruction pointer, `AVE_TIME_SLICE 30`).
- Multi-action ticks: `goals.md` GP-01 non-goal ("Preserving transitional multi-action tick semantics as the long-term model"). No roadmap row owns the transition. The measured shape is 2 or 3 actions in 89% of ticks, so speed is currently a modest lever; it becomes a large one the moment any lineage discovers that `Move, Move, Move, Eat` out-harvests `Move, Eat, Move`, and the 0.1 move cost is the only brake.
- Priority bid: 55% of creatures bid, median 0.004 per tick, 1.7% of action spend. All-pay contests have a biological literature (scramble competition, "biological auctions"), and the theory's prediction, that all-pay bids evolve toward small values, is what the population shows. Low cost; no change proposed.
- Memory inheritance and the Lamarckian weight flag: T11.F09 decisions with a named analog (maternal provisioning); one line, not a finding.

## 6. Suspected and cleared

- **Operator zoo.** 54 operators drawn uniformly by family looked like the opposite of NEAT's three. Measured on 60 live genomes with the T11.F01 instrument (20 trials per operator): 61,540 applied events, 94.2% silent, 5.5% changed, 0.3% dead. The worst are `RetargetNodeTarget` (6.5% dead) and `ChangeEntryNode` (5.7% dead, 61% changed); no operator is lethal at a rate that matters, and the live-counter helpful shares sit in a 0.45 to 0.53 band around noise. T11's repairs hold. What the zoo costs is supply dilution, and T11.F17 and T11.F19 already answered that.
- **Single dominant resource.** Fruit is 96% of eats. That is world composition (T02, T12), not a system pattern.

## 7. Where this points (nothing implemented)

Items 1 to 3 became track [T16 — Cost and Cue Fidelity](../roadmaps/t16-cost-and-cue-fidelity.md) on 2026-09-16 at the user's direction (T16.F01 gate before charge, T16.F02 effort-priced failed actions, T16.F03 phenotypic kin recognition); the multi-action tick and priority bid were judged designs to keep.

1. **Retire the failed-action energy penalty** (or reduce it to the effort cost the action already charges). The move cost, eat cost, and reproduce cost are the natural prices; the penalty adds a charge with no analog and no measured selective effect. Read at closure: failed-action share of action energy, blocked-move fraction, and the survey's avoidance fraction, so a later feature that makes avoidance one edge away (T11.F21) is read on a substrate where the penalty is not the confound.
2. **Move the reproduce energy gate ahead of the charge** and check transfer feasibility before charging. A bug-sized change with a TDD test on the `RejectedEnergyConstraints` path asserting no energy change.
3. **Retire or derive `lineage_match`.** Derive from `kin_tag` distance or drop the field; re-pin the identity block of any report that carries it.
4. Leave the multi-action tick and the priority bid as recorded; give the multi-action transition a roadmap owner only when a lineage is seen using queue length as speed.

## Sources

- Avida configuration, `avida-core/support/config/avida.cfg` (devosoft/avida, master): `BIRTH_METHOD`, `DIVIDE_FAILURE_RESETS`, `COPY_MUT_PROB`, `DIVIDE_INS_PROB`, `DIVIDE_DEL_PROB`, `AVE_TIME_SLICE`. https://github.com/devosoft/avida/blob/master/avida-core/support/config/avida.cfg
- Stanley, K. O. and Miikkulainen, R. (2002). Evolving neural networks through augmenting topologies. Evolutionary Computation 10(2). https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf
- Yaeger, L. (1994). Computational genetics, physiology, metabolism, neural systems, learning, vision, and behavior or PolyWorld. Artificial Life III. https://shinyverse.org/larryy/Yaeger.ALife3.pdf
- Huang, S. and Ontañón, S. (2022). A closer look at invalid action masking in policy gradient algorithms. FLAIRS / arXiv:2006.14171v3. https://arxiv.org/abs/2006.14171
- Riolo, R. L., Cohen, M. D. and Axelrod, R. (2001). Evolution of cooperation without reciprocity. Nature 414, 441–443. https://www.nature.com/articles/35106555
- Chatterjee, K., Reiter, J. G. and Nowak, M. A. (2012). Evolutionary dynamics of biological auctions. Theoretical Population Biology. https://pmc.ncbi.nlm.nih.gov/articles/PMC3279759/
- Local: `docs/reference/v3-tick-orchestration-spec.md` Section 6, `v3-reproduction-spec.md` Section 5, `v3-sensor-spec.md` Section 6.5, `v3-creature-identity-spec.md`; `crates/v3-core/src/simulation/actions/reproduction.rs` Steps 5 to 7; the live survey's dumps and `status.json`.

## Appendix: reproducing the measurements

Energy-per-action table, budget, and bid readings: a Python pass over the 400 `genomes/<id>.json` dumps of the live survey (Appendix A there), grouping `action_log` entries by `action_type:result` and by tick. Per-operator rows: `evaluate_genome(genome, &battery, &config.mutation, &ctx, 20, 0, seed)` on the first 60 dumps, printing `operator_rows`, run as a temporary v3-core integration test and removed afterward (the T11.F01 instrument, no new code).
