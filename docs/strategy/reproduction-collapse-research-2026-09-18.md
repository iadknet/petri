# How Artificial-Life Systems Keep Evolvable Reproduction From Evolving Itself Extinct (2026-09-18)

Research-first planning for the collapse diagnosed in [orchards-collapse-2026-09-17.md](orchards-collapse-2026-09-17.md), prompted by the user's report of a second instance: a manually run world that was stable to about tick 8,000 and then collapsed (no dump; treated here as a reported second instance, not diagnosed). Research date 2026-09-18, main at `7f5d8994`. Sources were read in full where they are open (paths in Section 3); counterfactuals were run on a scratch worktree (`.worktrees/reproduce-counterfactual`, branch `scratch/reproduce-counterfactual`, nothing committed) with the probe in Appendix A.

## 1. The problem, stated so the prior art can be sorted against it

The Orchards mechanism has three parts. Any remedy removes at least one:

- **(a) The selection differential.** In a famine, not reproducing beats reproducing. The founder breeds whenever energy > 30 (15% of the 200 cap) and hands 20 energy to an offspring that starves, so a fertile creature lives at ~10–16 energy through the famine while a creature that cannot breed holds ~40 and later 200. Fertile share of the population fell 96% → 7.5% between ticks 100 and 250.
- **(b) The lesion is cheap.** Reproduce is one mutable circuit (`EnergyCurrent → Threshold(30) → Multiply ← Threshold(19.5) ← AgeTicks → CustomOutput(1) → VM slot 1 → CmpGt → Reproduce`). Measured this session on 2,000 founder births with the executed-biased targeting layer off (`ParentExecuted::NONE`; both founder nodes execute every tick, so the production number should be close): **3.5% of all births (8.2% of mutated births) are sterile in vitro** (`InputRefSwap` 31, `TopologyChangeEntryNode` 16, `InputRefRawFieldMutation` 8, `GraphRemoveInternalGraphNode` 7, …). "Never breed" is one event away; "breed when fed" is a construction.
- **(c) The lesion is one-way.** Nothing restores a broken gate, the Hebbian gate weight never relearns once its output is constant, and sterile creatures are not purged: no maximum age, so they lived 1,980 ticks at the energy cap.

Constraints from the repository: every mechanism names a natural analog and reaches creatures through world or body, not a feature-specific sensor ([roadmap.md](../roadmap.md)); reproduction is a brain-emitted action ([v3-reproduction-spec.md](../reference/v3-reproduction-spec.md)) and nothing on record settles that it must stay one; T03.F07 ("Life-History and Offspring Investment Traits … when to mature, as heritable choices") is the unscheduled owner of the founder's life-history rule; T16.F01 just closed with the goal "an animal that cannot afford a litter does not conceive". Decision criteria: does the option remove (a), (b), or (c); does it keep reproduction evolvable; is the analog natural; what does it do to every trajectory under the epoch rules.

## 2. Local evidence

- `docs/strategy/orchards-collapse-2026-09-17.md`: the diagnosis, all 59 tick-400 survivors sterile in vivo.
- Counterfactuals below reproduce the committed T16.F01 trajectory exactly under the control rule (96,879 / 1,350 / 131 / 59 / 7), so the founder-rebuild harness is behavior-neutral.
- The second bust: every variant that survived the first trough on seed 11 (reserve-50, food gate, both) rebounded to 700–1,300 by tick 700 and busted again to 10–26 by tick 1,400–1,600. Orchards' grass dynamics (grazing floor 0.05 with a 1,000-tick recovery over a diffuse 65%-coverage crop) produce recurring famines; every famine is a new sterility lottery. Hypothesis for the reported 8,000-tick collapse, not a finding: a later famine drew the losing ticket; it is testable only with births-before-population data from such a run.
- Nothing in `docs/strategy/goals.md`, the roadmap, or the tracks records brain-controlled reproduction as a settled design; T16's own goal text already frames the reproduce gate in life-history terms.

## 3. Prior art

### 3.1 Fixed-population replicators: the problem cannot arise, and the sterile are purged

- **Tierra** (Ray 1991, "An approach to the synthesis of life", LaTeX source read in full from `tomray.me/pubs/alife2/tierra.tex`): "Self-replicating creatures in a fixed size soup would rapidly fill the soup and lock up the system. To prevent this from occurring, it is necessary to include mortality." The reaper kills from the top of an age queue; error-generating instructions move a creature up the queue, so "algorithms which are fundamentally flawed … rise to the top of the queue and die." A non-replicating parasite whose copy call fails "generate[s] errors, causing them to rise to the top of the reaper queue and die."
- **Avida** (Ofria, Bryson & Wilke 2009, "Avida: A Software Platform for Research in Computational Evolutionary Biology", `cse.msu.edu/~ofria/pubs/2009AvidaIntro.pdf`, read in full): a birth replaces a neighbor (random, or oldest by default), and organisms may also "be killed after it has executed a specified number of instructions, which can either be a constant or proportional to the organism's genome length, the default. **Without this setting, it is possible in some cases for a population to lose all ability to self-replicate, but persist since organisms have no means by which to be purged.**"

What these remove: **(c)** only. There is no famine, so (a) never arises; a sterile organism is aged out and its cell reused by whoever still replicates. They are not the answer for an energy ecology, but Avida's sentence is a precise description of Orchards' end state, and Petri has no purge at all.

### 3.2 Energy ecologies with evolvable reproduction: Petri's class

**Polyworld** (Yaeger 1994, "Computational Genetics, Physiology, Metabolism, Neural Systems, Learning, Vision, and Behavior or PolyWorld: Life in a New Context", `shinyverse.org/larryy/Yaeger.ALife3.pdf`, 25 pages read in full; parameters checked against `etc/worldfile.wfs` and `src/library/sim/Simulation.cc`, `src/library/agent/agent.cc` in `github.com/polyworld/polyworld`, cloned 2026-09-18):

- Mating is a brain output ("both organisms must express their mating behavior in excess of a specifiable threshold"), so the reproduce path is as lesionable as Petri's.
- **Offspring energy is a heritable fraction of the parent's current energy**, not a fixed transfer: "The final physiology gene controls the fraction of an organism's remaining energy that it will donate to its offspring upon birth." In the current source the gene `MateEnergyFraction` is bounded `[0.2, 0.8]` (`MinEnergyFractionToOffspring` / `MaxEnergyFractionToOffspring`) and `agent::mating` charges `MateEnergyFraction × fEnergy`. A starving parent therefore gives a starving-sized litter and keeps at least 20% of what it has; it cannot breed itself from 30 down to 10 as Petri's founder does.
- An optional physiological gate: `MinMateEnergyFraction` (default 0.0) — `preventedByEnergy = NormalizedEnergy() <= fMinMateFraction`; plus `MateWait` (25-tick cooldown) and `DieAtMaxAge` (lifespan is a gene, "a few hundred to a few thousand time-steps").
- **A population floor with re-creation**: "a minimum number of organisms may be guaranteed to populate the world. If the number of deaths causes the number of organisms extant in the world to drop below this minimum, either another random organism may be created by the system, or the offspring of two organisms from a table of the N fittest may be created, or, rarely, the best organism ever may be returned to the world unchanged." Defaults `MinAgents 90 / InitAgents 180 / MaxAgents 300`. The ad hoc fitness for that table "rewards organisms for eating, mating, living their full life span, dying with reserve energies, and simply moving" (`FitnessWeightMating 100`, `FitnessWeightEnergyAtDeath 5`, `FitnessWeightLongevity 25`, `FitnessWeightEating 10`).
- **Density-dependent costs, on by default**: `EnergyBasedPopulationControl = True` scales every energy debit (`agent::damage`) by `EnergyScaleFactor`: 1.0 inside the middle of the `[MinAgents, MaxAgents]` band, falling linearly to `PopControlMinScaleFactor` (default **0.0**) at `MinAgents`, rising to `PopControlMaxScaleFactor` (5.0, quartic) toward `MaxAgents`. With population control on, `agent` deaths are additionally blocked while the count is at or below `MinAgents` unless `AllowMinDeaths` (default false). `ApplyLowPopulationAdvantage` is the same idea keyed to `InitAgents`.
- The authors' own success criterion: a run is "successful" only when births alone sustain the population "with no further creations"; "Some simulation runs acquire an SBS in the first seed population and never require this on-line GA stage. Others never acquire an SBS, and are considered unsuccessful simulations." Runs that lean on the floor are failures by definition, not fixes.

What Polyworld removes: (a) partly, through proportional investment and the optional energy-fraction gate; (c) through `DieAtMaxAge`; and it hides the residue behind the floor and the cost scaling.

**Bibites** (developer wiki, `the-bibites.fandom.com/wiki/Reproduction`, `/Brain`, `/Bibite_Spawn_Rate`, `/Bibite_Spawn_Cap`, read through the browser pane): `EggProduction` and `Want2Lay` are output neurons, so laying is brain-driven, but "In order to start laying eggs, Bibites must first reach adulthood. It also need to be Healthy enough (≥ 50%)", and the egg must be paid in full ("the sum of the energy needed to grow the bibite to the size it has at birth, the cost of the bibite's physical traits, and the cost of the bibite's brain"). Input and output neurons "are locked and can't be subjected to mutation" (synapses can). Extinction is handled by immigration: "New Bibites are periodically spawned according to the initial genes stub … expressed in the unit Hz … in the hopes that one Bibite will thrive and achieve a stable population", capped by a target count. What it removes: (a) partly (a proportional condition gate), and it masks the rest with immigration.

**Darwinbots** (`wiki.darwinbots.com/index.php?title=Evolution_Sims`, read through the browser pane): reproduction is a DNA command with an evolvable energy threshold; the community's list of "Common Mutation Areas" includes "Increased energy required for reproduction", "inc/dec energy given to young at birth", "continual reproduction", and "**Non-reproducing**"; plants are repopulated by a "repopulation threshold". The same lesion is a recognized outcome there; the handling is manual (save the best bot, reseed).

### 3.3 Theory: why a famine selects for sterility, and why nature does not go extinct that way

- **Rankin, Bargum & Kokko 2007**, "The tragedy of the commons in evolutionary biology", TREE 22:643–651 (open-access copy from the University of Helsinki repository, fetched through the browser pane, read in full). They define a **collapsing tragedy** as "a situation where selfish individual behaviour results in the entire resource vanishing … This type of tragedy can lead to the extinction of the whole group, if the resource or the social good was essential for its survival," versus a **component tragedy** where "the resource has been depleted, but not to the extent that it disappears completely." Resolutions they list: direct benefits of restraint ("a tragedy of the commons will not arise if there are direct benefits to restraint"), population structure and kin selection, coercion and punishment, and **density feedback** ("individual bacteria reduce their production of bacteriocins when the population density is low"). Orchards is two tragedies stacked: the boom overgrazes the common grass to the floor (collapsing tragedy on the resource), and the famine then favors an individual restraint — not breeding — whose only cheap genetic form is permanent.
- **Parvinen & Dieckmann 2013**, "Self-extinction through optimizing selection", J. Theor. Biol. 333:1–9 (open access, `user.iiasa.ac.at/~dieckman/reprints/ParvinenDieckmann2013.pdf`, read in full): "Evolutionary suicide is a process in which selection drives a viable population to extinction" through "environmental feedback" that "disconnect[s] individual-level and population-level interests"; even optimizing selection can do it, and "its harbingers are strong population fluctuations." They cite Haldane 1932: once "a species becomes fairly dense … Its members inevitably begin to compete with one another." The Orchards boom-bust is exactly the fluctuation that flips the sign of selection on breeding. Reviews they point to and I could not open (paywalled; not read): Parvinen 2005 *Acta Biotheoretica* 53:241–264; Rankin & López-Sepulcre 2005 *Oikos* 111:616–619; Webb 2003 *Am. Nat.* 161:181–205; Gyllenberg & Parvinen 2001 *Bull. Math. Biol.*
- **The natural analog — intermittent breeding on a body-condition threshold.** Desprez 2018, "Reproductive skipping as an optimal life history strategy in the southern elephant seal", *Ecology and Evolution* (PMC6194220, open access, read in full): "One form of dealing with this trade-off, that can sometimes be an optimal strategy, includes skipping current breeding opportunities (termed 'intermittent breeding') (Bull & Shine 1979)—that is, the prudent parent hypothesis (Drent & Daan 1980). Theory predicts that intermittent breeding is expected to occur as the cost of reproduction increases in terms of survival, energetic demand, or recovery time (Shaw & Levin 2013)." The model's result: skipping is optimal because "intermittent breeding resulted in rapid gains in body mass that subsequently led to increased survival (because survival was assumed to be mass dependent)," with a lower critical mass below which the animal does not breed. Bull & Shine 1979 (*Am. Nat.* 114:296–303) and Naulleau & Bonnet 1996 ("Body condition threshold for breeding in a viviparous snake", *Oecologia* 107:301–306) are the classic statements; both paywalled and cited through Desprez.

In nature the famine does not fix sterility because breeders **skip** rather than break: the capacity survives the famine unexpressed, and the population is rebuilt from it. Petri's founder is a capital breeder with its threshold at 15% of capacity and no skip rule, and the mutational neighborhood offers "never" at 3.5% of births and "when fed" at roughly zero.

## 4. Options considered

| Option | Removes | Keeps reproduction evolvable | Natural analog | Prior art |
| --- | --- | --- | --- | --- |
| A. Reserve threshold in the founder (breed only above half of max energy) | (a) | yes (threshold is a graph parameter, `GraphMutateGraphOperatorParam` moves it) | body-condition threshold, prudent parent | Bibites health ≥ 50%; Polyworld `MinMateEnergyFraction`; Naulleau & Bonnet 1996 |
| B. Physiological reserve: the parent must still hold a floor (a fraction of `max_energy`, or a raised `min_reproduce_energy`) after cost and transfer; the decision stays a brain output | (a), durably (the floor is not a mutable parameter) | yes (world/body rule, brain unchanged) | body-condition threshold; "cannot afford a litter does not conceive" (T16.F01's own goal) | Polyworld's world-bounded `MateEnergyFraction` gene and `MinMateEnergyFraction`; Bibites health ≥ 50% |
| C. Food gate in the founder (breed only with food on the cell) | (a) barely | yes | none clean (breeding on the food patch) | — |
| D. Population floor with re-creation | nothing; masks | yes | none (immigration from nowhere) | Polyworld `MinAgents`; Bibites spawn rate; Darwinbots reseed |
| E. Density-dependent cost scaling | (a) partly (shallower trough) | yes | weak (per-capita metabolic cost does not fall with density) | Polyworld `EnergyBasedPopulationControl` |
| F. Maximum lifespan (purge) | (c) | yes | senescence | Tierra reaper, Avida age-death, Polyworld `DieAtMaxAge` |
| G. Reproduction as physiology (the decision itself taken from the brain: breed automatically above a threshold) | (b) | **no** | none in the roadmap's sense | Sugarscape-style rules |
| H. Fertility indicator in the goal report | none; diagnoses | — | — | Polyworld's born/created ratio |

## 5. Measured counterfactuals

Orchards goal case, 1600², 10,000 founders, 2,000 ticks; the founder rule is rewritten in node 0's graph at tick zero and inherited. "Fertile" is in vitro (emits `Reproduce` from a fresh runtime state at energy 150, age 100, food on every side). Wall ≈ 3–4 min per run.

| Run | Peak | Minimum (tick) | Tick 300 fertile/total | Tick 400 fertile/total | Minimum after tick 500 | Final population / fertile | Births, ticks 1,500–2,000 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| control, seed 11 (= committed T16.F01) | 100,000 | 7 (1,800) | 13/131 | 6/59 | 7 | 7 / 1 | 0 |
| control, seed 12 | 100,000 | 10 (1,900) | 13/147 | 41/98 | 10 | 10 / 1 | 0 |
| A′ reserve 50 (= 30 + transfer), seed 11 | 97,629 | 26 (1,400) | 11/136 | 52/113 | 26 | 121 / 110 | 650 |
| A′ reserve 50, seed 12 | 97,752 | 10 (2,000) | 12/118 | 100/147 | 10 | 10 / 0 | 0 |
| C food gate, seed 11 | 100,000 | 15 (1,600) | 11/125 | 85/152 | 15 | 230 / 205 | 322 |
| C food gate, seed 12 | 100,000 | 13 (1,900) | 8/138 | 35/94 | 13 | 13 / 0 | 0 |
| A′ + C, seed 11 | 94,886 | 11 (1,600) | 13/111 | 39/82 | 11 | 198 / 176 | 234 |
| A′ + C, seed 12 | 95,269 | 59 (350) | 10/117 | 50/92 | 86 (1,200) | 1,127 / 1,037 | 4,171 |
| **A reserve 100 (half of max), seed 11** | 62,353 | 112 (400) | 1,165/1,471 | 66/112 | 444 (500) | 2,876 / 2,537 | 9,802 |
| **A reserve 100, seed 12** | 62,286 | 131 (400) | 1,113/1,391 | 92/131 | 669 (500) | 3,320 / 3,122 | 8,509 |
| A reserve 100 + C, seed 11 | 61,889 | 117 (400) | 1,154/1,479 | 78/117 | 375 (500) | 3,037 / 2,862 | 9,204 |
| D floor 100 (Polyworld `MinAgents`), seed 11 | 100,000 | 102 (350) | 13/131 | 172/229 | 151 (1,200) | 7,711 / 6,872 | 87,951 (59 founders created) |
| E cost scaling (decay and move charges → 0 from 2,000 down to 100 creatures), seed 11 | 100,000 | 499 (1,200) | 86/539 | 550/899 | 499 (1,200) | 7,268 / 6,421 | 39,952 |
| Transfer as a fraction 0.5 of post-cost energy, founder 30 threshold, seed 11 | 100,000 | 127 (350) | 24/144 | 179/247 | 450 (1,100) | 5,088 / 4,430 | 22,664 |
| same, seed 12 | 100,000 | 7 (1,900) | 13/157 | 15/76 | 7 | 7 / 0 | 0 |
| same, seed 12, `failed_action_penalty` = 0 | 100,000 | 126 (1,300) | 29/162 | 288/370 | 126 (1,300) | 2,029 / 1,725 | 6,704 |
| fraction 0.5 of the surplus above a 100 reserve, litter ≥ 20, founder threshold 100, seed 11 (penalty on) | 10,000 | 307 (2,000) | 9,982/9,982 | 9,619/9,619 | 307 | 307 / 307 | **0 (no birth ever)** |
| same, seeds 11 and 12, `failed_action_penalty` = 0 | 10,000 | 464 / 411 (2,000) | 9,982 / 9,972 | 9,663 / 9,623 | — | 464 / 411, all fertile | **0 (no birth ever)** |
| control, `failed_action_penalty` = 0, seeds 11 / 12 | 100,000 | 6 / 7 | 4/115, 18/162 | 3/58, 78/160 | 6 / 7 | 6 / 1, 7 / 0 | 0 |
Readings:

1. **Threshold at 50 and the food gate do not remove the differential.** Fertile creatures still sit at their threshold (mean 10–15 in the famine, 25–35 after) because they breed the moment they cross it; on seed 12 reserve-50 and the food gate each end with zero fertile creatures, and their combination survives with a post-500 minimum of 86. On seed 11 all three survive the first trough by luck and bust again at ticks 1,400–1,600. The coin flip is real and the coin is the same.
2. **Threshold at half of max energy carries fertility through the famine.** It does not erase the differential — in the deep famine fertile creatures still read 31/20/13 against sterile 53/50/40 at ticks 200/250/300 — but they enter it at 47–63 energy instead of 10–16, so far more of them last. The boom is slower (peak 62k instead of the 100k cap, grass lasts 50 ticks longer), the trough is 112–131 with **59–70% fertile** (control: 10%), and both seeds rebuild to 2,900–3,300 with 88–94% fertile and no second collapse in 2,000 ticks. The world still cycles (5,789 → 2,074 → 2,876 on seed 11), but every trough so far is survivable. Threshold drift was measured on a re-run of seed 11: the living population's CN0 threshold stays at 100.0 from the 10th to the 90th percentile at every checkpoint, 5 of 2,790 genomes below 60 at tick 2,000 (re-indexed nodes after a removal, not selected drift); `GraphMutateGraphOperatorParam` moves it by ~0.1 per event, so selection toward a lower threshold during booms is possible in principle but was not visible in 2,000 ticks.
3. **The floor keeps the world alive without fixing it**: 59 founders had to be created, and the population it carries cycles on the same famine dynamics (grass at 314k and falling at tick 2,000).
4. **Cost scaling shallows the trough without touching the differential**: the minimum is 499 instead of 131, fertile mean energy is still 10–30 against sterile 40–54 through the famine, and the fertile share at the trough is 16% (86/539) — enough fertile creatures survive in absolute terms (86 vs 13) for the world to rebuild to 7,268. It is Polyworld's default and it works by making the famine cheaper, which is not what a famine is.
5. **A fraction rule alone is the same lottery.** Transfer as 0.5 of post-cost energy at the founder's 30 threshold: seed 11 lives (min 127, 5,088 at the end), seed 12 dies (0 fertile by 1,500) with the penalty on and lives (min 126, 2,029) with it off — luck, not mechanism. The fraction changes the child's share; it does not carry fertile creatures through a famine.
6. **Retiring the failed-action penalty alone changes nothing about the collapse** (T16.F02's world): both penalty-off controls die (6 and 7).
7. **The founder pins itself at any threshold physiology refuses, penalty or not.** Fraction-of-surplus with a 100 reserve and a 20 minimum litter (acceptance needs ≥ 140) gave zero births in 2,000 ticks on both seeds, with the penalty on and off. Instrumented: between ticks 100 and 200 the 10,000 founders logged ~800k refused reproduce attempts and only ~190k eats and ~200k moves over 1M creature-ticks, against 80% eating before they reached 100. The founder VM's reproduce branch ends in `ExecuteActionQueue`, which the ISA spec makes **terminal** — the forage code after it never runs that tick — so a creature whose brain says breed and whose body refuses spends the tick idle, decays 0.5, dips under its threshold, forages back to it, and repeats. Any feature that can make physiology refuse a founder attempt must keep the founder's gate at or above the acceptance region or let the founder queue reproduce and forage together (the multi-action tick allows ten actions; the founder never uses it that way).
8. Founder sterility per birth is 3.5% (8.2% of mutated births) under every rule; the food-gated founder is 5.6% because its longer reproduce path has more targets. None of the rules touch (b).

## 6. Recommendation

*Superseded in part on 2026-09-18: the user split this into an evolvability feature (the transfer as a fraction, now T17.F01 in Section 8's track) and a separate famine/collapse-protection decision still under discussion. The measurements below stand; the placement does not.*

**Adopt B (a physiological reserve: after cost and transfer the parent must still hold a floor of half its maximum energy, applied in the reproduce gate) with A (the founder's own threshold raised to match, so it does not attempt what the body refuses), as one feature; recommend it to the user for a T16 row or the T03.F07 slot; do not add it here.** The floor lives in physiology because a founder-side threshold is a mutable graph parameter that every boom selects downward (breed at 40 out-reproduces breed at 100 while food lasts); Polyworld bounds its investment gene in the world for the same reason and Bibites' health gate is not evolvable. B blunts the selection differential at its source (a famine no longer taxes fertility down to 10), it is the direct analog of the body-condition threshold that Bull & Shine, Naulleau & Bonnet, and Desprez describe, and it keeps the decision a brain output whose evolution can still move within the floor. Small in code — Step 6/7 of `crates/v3-core/src/simulation/actions/reproduction.rs` gating on `after_cost - transfer >= reserve` with the reserve a lifecycle config value, and `node0_graph_sensor`'s threshold in `crates/v3-core/src/creature/founder.rs` raised to match — but it moves every trajectory: the boom peak falls from the 100k cap to ~62k and every births-based indicator changes, so the goal epoch re-pin is certain, not conditional. `cargo test -p v3-core --test viability` first.

**Against the strongest alternative, D (Polyworld's floor):** the floor guarantees the world survives and is one harness, but Yaeger's own criterion calls a run that needs it unsuccessful, it has no natural analog (creatures from nowhere), and the counterfactual shows it carrying a population that still cycles through the same lottery — 59 creations in 2,000 ticks. A is the fix; D is the bandage. E (density-scaled costs) is Polyworld's other default and did keep seed 11 alive (trough 499, 16% fertile), but only by making the famine cheaper per capita, the weakest analog in the table; it also leaves the differential intact, so a deeper world or a longer run reopens the lottery.

**Also recommend, separately:**
- **H, a fertility reading in the goal report** (T14): in-vivo fertile share (creatures with a `Reproduce` attempt in the last N ticks over creatures older than the age gate) and births per capita per checkpoint. The 8,000-tick collapse could not be told from starvation with the current fields; `births_total` deltas are already in the samples and were what exposed Orchards.
- **F, a maximum lifespan**, as a T03.F07 life-history trait, not a collapse fix: it purges the sterile (Avida's argument) and is the missing senescence analog, but it makes an all-sterile world die faster, not recover.
- **Not G.** Making reproduction unevolvable removes (b) at the cost of the project's premise (GP-01); the prior art keeps it evolvable and manages (a) instead.
- **(c) stays open.** The Hebbian gate that learned itself shut and never relearns is a plasticity question for T11.F09's owner; a max lifespan bounds its damage.

## 7. Remaining uncertainty

- The reserve-100 result is two seeds on one world and 2,000 ticks; the recurrent-famine shape says a longer run (the user's 8,000-tick class) is the real test, and Canyon and Confluence have not been run. A proof of concept before the feature: one 10,000-tick Orchards run under A+B, births-before-population logged every 100 ticks.
- Threshold 100 is Bibites' half; nothing here tunes it. The reserve-50 failure and the reserve-100 success bracket it; a T03.F07 heritable threshold would let the world find its own.
- The theory reviews (Parvinen 2005; Rankin & López-Sepulcre 2005; Webb 2003) were not opened; the framing rests on Rankin et al. 2007 and Parvinen & Dieckmann 2013, both read in full.
- B was not run as physiology in the counterfactuals; A at 100 is its founder-side equivalent (the founder never attempts below its threshold, so the two coincide for founders and differ only for evolved thresholds). A′ at 50 shows the floor must be well above 30 + transfer.
- Plains (the production default) and the gate profile were not run under A; the food variant's genome is 116 units, so it paid the T03.F11 replication surcharge on 5 extra units — a small confound against C.

## 8. Boundary audit: where else the brain's units block a one-edge rule (2026-09-18)

Raised in a side conversation with a separate agent after the transfer framing; every claim below was re-verified against main at `7f5d8994`. The mutation operators assume a unit-scale brain: a new `Threshold` or `Constant` is drawn in [−1, 1] (`crates/v3-core/src/mutation/graph/operators.rs:222,229`), a new or copied edge weight in [−1, 1] (`:507`, `:767`), `MutateGraphOperatorParam` steps ±0.1 (`:902,905`), VM constants step ±1.0 (`mutation/vm/operators.rs:24`), and the Covariance Hebbian rule is `(pre − 0.5)(post − 0.5)` (`runtime/plasticity/hebbian.rs:164`). Anything crossing the boundary outside [0, 1] is outside what those operators reach or tune:

| Boundary | Units | Status | Evidence |
| --- | --- | --- | --- |
| Move / Reproduce / Steal direction (`meta[0]`) | index 0–7 | fixed by T11.F21's bank | `action_decode.rs:33–41` |
| Reproduce transfer (`meta[1]`) | energy, clamp [0, 100] | ✗ | a [0, 1] sensor routed here gives a 0–1-energy child; `transfer > 0` passes the gate (`reproduction.rs:200`), the child dies of 0.5/tick decay next tick — a hidden lethal, not a null |
| Steal amount (`meta[1]`) | energy, capped by victim | ✗ | same: sensor-driven steals take ≤ 1 energy; goal reports show ~5 per transfer |
| Eat `type_idx` (`meta[0]`) | integer index | ~ | a [0, 1] sensor rounds to type 0/1 — one-edge for exactly two food types, breaks at three; per-type bank is the T11.F21 model |
| Priority bid | energy, spent | ~ | relative competition, so unit-scale bids work; VM-only (`vm.rs:457`) |
| `EnergyCurrent` input | raw 0–200 | ✗✗ | a new `Threshold` in [−1, 1] is always on; the founder's 30 needs ~700 ±0.1 steps to reach 100 (measured: p10–p90 at 100.0 for 2,000 ticks); Covariance with `pre` = 20–200 slams the weight to the clamp in one tick — the six Orchards Hebbian carriers. `NearbyCreatureVitals` meanwhile reports other creatures' energy as `energy_ratio` ∈ [0, 1] (`perception.rs:89`) |
| `AgeTicks` input | raw ticks | ✗ | same three failures; the founder's 19.5 gate is frozen |
| `Generation` input | raw count | ✗ | 14 of the 59 sterile Orchards survivors had it swapped in for energy or age — a raw-unit key makes a constant gate |
| `EnergyConsumedThisTick` | energy | ✗ mild | `inputs.rs:50` |
| Food here, rings, area summaries, nearby core / vitals / identity | normalized | ✓ | reducers divide by radius or max |

Three of the seven lesion classes in the Orchards note's table are this mismatch in different clothes. The refactor is one principle in both directions — inputs as fractions of `max_energy` or a reference span, outputs as fractions of what the parent holds or the victim carries — and the operators need no change; that is the point. It moves every trajectory (Hebbian carriers and evolved thresholds behave differently) but the founder is re-expressed identically by construction. At the user's direction this became track T17 ([t17-brain-boundary-evolvability.md](../roadmaps/t17-brain-boundary-evolvability.md)): F01 the transfer fraction (the user's framing, from Section 6), F02 unit-scale introspection, F03 the steal fraction, F04 a per-type eat bank when a third food type exists. Famine and collapse protection (Sections 5–7's reserve, floor, cost-scaling, and lifespan options) is deliberately not in T17; the user has asked to discuss it before placing any of it.

## Appendix A: the probe

`crates/v3-core/tests/zz_probe_reproduce_cf.rs` on the scratch worktree; public `v3_core` APIs only. Environment: `PETRI_VARIANT` ∈ `control | reserve | reserve100 | food | both | r100food`, `PETRI_SEED`, `PETRI_TICKS`, `PETRI_FLOOR=N` (Polyworld `MinAgents`), `PETRI_POPCTL=N` (Polyworld cost scaling, lower band only), `PETRI_NO_PENALTY=1`; the fraction runs also carry a scratch patch to Step 7 of `reproduction.rs` (`PETRI_TRANSFER_FRACTION=f`, `PETRI_RESERVE=r`, minimum litter = `initial_energy` when a reserve is set) that lives only on the worktree. `probe_founder_sterility_rate` gives Section 1(b).

```rust
//! TEMPORARY PROBE (not for commit): founder reproduce-rule counterfactuals on
//! the Orchards goal case, plus the founder's single-birth sterility rate.
//!
//! `PETRI_VARIANT` selects the founder rule swapped into every founder at tick
//! zero (offspring inherit it): `control`, `reserve` (energy gate 50 instead of
//! 30, so a parent never drops below the 30 gate by breeding), `food` (breed
//! only with food on the current cell), `both`. `PETRI_FLOOR=N` re-creates
//! founders whenever the population falls below N (Polyworld's MinAgents).
//! `PETRI_SEED` (default 11), `PETRI_TICKS` (default 2000).
use std::collections::BTreeMap;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use v3_core::config::{resolve_config, RuntimeConfig, SimulationConfig};
use v3_core::contracts::{Position, WorldAction};
use v3_core::creature::genome::cgp::{ComputeNode, ComputeNodeKind, GraphEdge, GraphSource};
use v3_core::creature::genome::{BackendDef, CreatureGenome};
use v3_core::creature::state::{CreatureState, GraphRuntimeState};
use v3_core::creature::genome::analysis::mesh_reachable_nodes;
use v3_core::mutation::reachability::ParentExecuted;
use v3_core::mutation::MutationEngine;
use v3_core::runtime::mesh::execute_creature_mesh;
use v3_core::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
use v3_core::sensors::static_inputs::StaticInputs;
use v3_core::sensors::typed_food::TypedFoodLocalSnapshot;
use v3_core::simulation::energy_accounting::DeathCause;
use v3_core::simulation::seed_simulation;
use v3_core::simulation::tick::run_tick;
use v3_core::simulation::Simulation;

fn orchards_config() -> SimulationConfig {
    let recipe: serde_json::Value = serde_json::from_str(include_str!(
        "../../../experiments/worlds/orchards-in-grassland.json"
    ))
    .unwrap();
    let mut config = resolve_config(&SimulationConfig::default(), recipe).unwrap();
    config.world.width = 1600;
    config.world.height = 1600;
    config.population.initial_creatures = 10_000;
    config.normalize();
    config.apply_startup_overrides();
    if std::env::var("PETRI_NO_PENALTY").is_ok() {
        // T16.F02's world: a failed action costs only its own charge.
        config.startup.ramps.failed_action_penalty.enabled = false;
        config.energy.costs.failed_action_penalty = 0.0;
    }
    config
}

fn sensors(food_types: usize, food_here: f32) -> SensorSnapshot {
    SensorSnapshot {
        local: StaticInputs {
            food_here,
            neighbor_food: [0.5; 8],
            neighbor_barrier: [0.0; 8],
            neighbor_occupied: [0.0; 8],
            generation: 1.0,
            age_ticks: 100.0,
        },
        typed_local_food: TypedFoodLocalSnapshot {
            food_here_by_type: vec![food_here; food_types],
            neighbor_food_by_type: vec![[0.5; 8]; food_types],
        },
        perception: PerceptionSnapshot::zeroed(food_types),
    }
}

fn emits_reproduce(genome: &CreatureGenome, s: &SensorSnapshot, runtime: &RuntimeConfig, energy: f32) -> bool {
    let mut energy = energy;
    let mut shared = [0.0f32; 16];
    let prev = [0.0f32; 16];
    let mut gr = GraphRuntimeState::new();
    gr.begin_tick(&genome.nodes, 100);
    let out = execute_creature_mesh(genome, s, &mut energy, &mut shared, &prev, &mut gr, runtime);
    out.actions.iter().any(|a| matches!(a, WorldAction::Reproduce { .. }))
}

/// The founder with its reproduce rule rewritten in node 0's graph.
fn variant_genome(founder: &CreatureGenome, variant: &str) -> CreatureGenome {
    let mut genome = founder.clone();
    let BackendDef::Graph(def) = &mut genome.nodes[0].backend_def else {
        panic!("founder node 0 is a graph");
    };
    if variant == "reserve" || variant == "both" {
        // CN0 gates on energy > 30; make it 50 = 30 + the 20 transferred.
        def.compute_nodes[0].kind = ComputeNodeKind::Threshold(50.0);
    }
    if variant == "reserve100" || variant == "r100food" {
        // Bibites' rule: breed only above half of maximum energy (200).
        def.compute_nodes[0].kind = ComputeNodeKind::Threshold(100.0);
    }
    if variant == "food" || variant == "both" || variant == "r100food" {
        // CN3: food_here > 0; CN4 = CN2 * CN3; CustomOutput(1) <- CN4.
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Threshold(0.0),
            inputs: vec![GraphEdge { source: GraphSource::InputLeaf { ref_idx: 0, sub_idx: 0 }, weight: 1.0 }],
            plasticity: None,
        });
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Multiply,
            inputs: vec![
                GraphEdge { source: GraphSource::ComputeNode(2), weight: 1.0 },
                GraphEdge { source: GraphSource::ComputeNode(3), weight: 1.0 },
            ],
            plasticity: None,
        });
        def.output_sinks[1].inputs = vec![GraphEdge { source: GraphSource::ComputeNode(4), weight: 1.0 }];
    }
    genome
}

fn rebuild_founders(sim: &mut Simulation, genome: &CreatureGenome) {
    let ids: Vec<_> = sim.creatures.keys().collect();
    for id in ids {
        let c = &sim.creatures[id];
        let rebuilt = CreatureState::new(
            c.id,
            genome.clone(),
            c.position,
            c.energy,
            c.generation,
            c.phenotype_channels,
            c.phenotype_active_channel,
            c.phenotype_channel_polarity,
            c.identity,
            c.shared_memory,
        );
        sim.creatures[id] = rebuilt;
    }
}

/// Polyworld's MinAgents: create founders on free cells until the population
/// is back at `floor`. Returns how many were created.
fn respawn_to_floor(sim: &mut Simulation, genome: &CreatureGenome, floor: usize, rng: &mut SmallRng, next_founder: &mut usize) -> usize {
    let mut created = 0;
    let width = sim.config.world.width;
    let height = sim.config.world.height;
    let energy = sim.config.energy.lifecycle.initial_energy;
    let template = sim.creatures.values().next().map(|c| (c.phenotype_channels, c.phenotype_active_channel, c.phenotype_channel_polarity));
    let (channels, active, polarity) = template.unwrap_or(([0; 6], 0, [false; 6]));
    while sim.creatures.len() < floor {
        let pos = Position { x: rng.gen_range(0..width), y: rng.gen_range(0..height) };
        if sim.world.is_barrier(pos) || sim.world.creature_at(pos).is_some() {
            continue;
        }
        let identity = v3_core::creature::identity::CreatureIdentityState::founder(*next_founder, 11);
        *next_founder += 1;
        let id = sim.creatures.insert_with_key(|id| {
            CreatureState::new(id, genome.clone(), pos, energy, 0, channels, active, polarity, identity, [0.0; 16])
        });
        sim.world.place_creature(pos, id);
        created += 1;
    }
    created
}

#[test]
fn probe_reproduce_counterfactual() {
    let variant = std::env::var("PETRI_VARIANT").unwrap_or_else(|_| "control".into());
    let seed: u64 = std::env::var("PETRI_SEED").ok().and_then(|v| v.parse().ok()).unwrap_or(11);
    let ticks: u64 = std::env::var("PETRI_TICKS").ok().and_then(|v| v.parse().ok()).unwrap_or(2000);
    let floor: usize = std::env::var("PETRI_FLOOR").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    let config = orchards_config();
    let food_types = config.world.food.types.len();
    let runtime = config.runtime.clone();
    let fed = sensors(food_types, 1.0);
    let unfed = sensors(food_types, 0.0);

    let mut sim = seed_simulation(config, seed);
    let founder = sim.creatures.values().next().unwrap().genome.clone();
    let genome = variant_genome(&founder, &variant);
    rebuild_founders(&mut sim, &genome);
    println!("{}", serde_json::json!({
        "kind": "variant", "variant": variant, "seed": seed, "floor": floor, "popctl": std::env::var("PETRI_POPCTL").unwrap_or_default(), "no_penalty": std::env::var("PETRI_NO_PENALTY").is_ok(), "transfer_fraction": std::env::var("PETRI_TRANSFER_FRACTION").unwrap_or_default(), "reserve": std::env::var("PETRI_RESERVE").unwrap_or_default(),
        "genome_size": genome.genome_size(),
        "reproduce_fed_e150": emits_reproduce(&genome, &fed, &runtime, 150.0),
        "reproduce_fed_e40": emits_reproduce(&genome, &fed, &runtime, 40.0),
        "reproduce_unfed_e150": emits_reproduce(&genome, &unfed, &runtime, 150.0),
    }));

    let mut sample_ticks: Vec<u64> = vec![0, 50, 75, 100, 125, 150, 175, 200, 225, 250, 275, 300, 350, 400, 500];
    let mut t = 600;
    while t <= ticks {
        sample_ticks.push(t);
        t += 100;
    }
    let mut last_deaths = [0u64; DeathCause::ALL.len()];
    let mut last_births = 0u64;
    let mut created_total = 0usize;
    let mut respawn_rng = SmallRng::seed_from_u64(seed ^ 0xF100);
    let mut next_founder = 10_000usize;
    let mut minimum = usize::MAX;
    // Polyworld's EnergyBasedPopulationControl, lower band only: per-tick decay
    // and move charges scale from 1.0 at `popctl` creatures down to 0.0 at 100.
    let popctl: usize = std::env::var("PETRI_POPCTL").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    let base_decay = sim.config.energy.lifecycle.energy_decay_per_tick;
    let base_move = sim.config.energy.costs.move_cost;
    for tick in 0..=ticks {
        if tick > 0 {
            if popctl > 100 {
                let pop = sim.creatures.len();
                let factor = ((pop.saturating_sub(100)) as f32 / (popctl - 100) as f32).clamp(0.0, 1.0);
                sim.config.energy.lifecycle.energy_decay_per_tick = base_decay * factor;
                sim.config.energy.costs.move_cost = base_move * factor;
            }
            run_tick(&mut sim, &mut None);
            if floor > 0 && sim.creatures.len() < floor {
                created_total += respawn_to_floor(&mut sim, &genome, floor, &mut respawn_rng, &mut next_founder);
            }
        }
        minimum = minimum.min(sim.creatures.len());
        if !sample_ticks.contains(&tick) {
            continue;
        }
        let mut fertile = 0u64;
        let mut fertile_energy = 0f64;
        let mut sterile_energy = 0f64;
        let mut gen0 = 0u64;
        // The founder's CN0 energy threshold as it stands in each living genome
        // (drift under GraphMutateGraphOperatorParam); None when CN0 is gone.
        let mut thresholds: Vec<f32> = Vec::new();
        let mut acts = [0u64; 5];
        let mut eats_applied = 0u64;
        let mut blocked = 0u64;
        for c in sim.creatures.values() {
            for (i, a) in c.lifetime_actions_attempted_by_type.iter().enumerate() {
                acts[i] += *a;
            }
            eats_applied += c.lifetime_eats_applied_by_type.first().copied().unwrap_or(0);
            blocked += c.lifetime_blocked_move_count;
            if c.generation == 0 {
                gen0 += 1;
            }
            if let BackendDef::Graph(def) = &c.genome.nodes[0].backend_def {
                if let Some(ComputeNodeKind::Threshold(t)) = def.compute_nodes.first().map(|n| n.kind.clone()) {
                    thresholds.push(t);
                }
            }
            if emits_reproduce(&c.genome, &fed, &runtime, 150.0) {
                fertile += 1;
                fertile_energy += f64::from(c.energy);
            } else {
                sterile_energy += f64::from(c.energy);
            }
        }
        let pop = sim.creatures.len() as u64;
        let deaths: BTreeMap<String, u64> = DeathCause::ALL
            .iter()
            .enumerate()
            .filter_map(|(i, cause)| {
                let d = sim.stats.mortality.by_cause[i] - last_deaths[i];
                (d > 0).then(|| (format!("{cause:?}"), d))
            })
            .collect();
        last_deaths = sim.stats.mortality.by_cause;
        let births = sim.stats.reproduction_actions_spawned_total;
        thresholds.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = thresholds.len();
        let quant = |q: f64| thresholds.get(((n as f64 - 1.0) * q) as usize).copied().unwrap_or(0.0);
        let below60 = thresholds.iter().filter(|t| **t < 60.0).count();
        println!("{}", serde_json::json!({
            "kind": "sample", "tick": tick, "population": pop, "fertile": fertile, "sterile": pop - fertile,
            "pop_lifetime_actions_noop_eat_move_repro_steal": acts, "pop_lifetime_eats_applied": eats_applied, "pop_lifetime_blocked_moves": blocked, "repro_rejected_total": sim.stats.reproduction_actions_rejected_total, "repro_attempted_total": sim.stats.reproduction_actions_attempted_total,
            "cn0_threshold": {"n": n, "min": quant(0.0), "p10": quant(0.1), "median": quant(0.5), "p90": quant(0.9), "max": quant(1.0), "below_60": below60},
            "gen0": gen0,
            "fertile_mean_energy": if fertile > 0 { fertile_energy / fertile as f64 } else { 0.0 },
            "sterile_mean_energy": if pop - fertile > 0 { sterile_energy / (pop - fertile) as f64 } else { 0.0 },
            "births_since_last": births - last_births,
            "births_total": births,
            "created_total": created_total,
            "minimum_so_far": minimum,
            "grass": sim.world.total_food_by_type(v3_core::config::OrdinaryFoodTypeId::default()),
            "mean_energy": sim.mean_energy(),
            "deaths_since_last": deaths,
        }));
        last_births = births;
    }
}

/// How often does one birth's worth of mutation sterilize the founder? 2,000
/// births from the founder (and from each variant), classified in vitro.
#[test]
fn probe_founder_sterility_rate() {
    let config = orchards_config();
    let food_types = config.world.food.types.len();
    let runtime = config.runtime.clone();
    let fed = sensors(food_types, 1.0);
    let sim = seed_simulation(config.clone(), 11);
    let founder = sim.creatures.values().next().unwrap().genome.clone();
    for variant in ["control", "reserve", "food", "both"] {
        let genome = variant_genome(&founder, variant);
        let reachable = mesh_reachable_nodes(&genome);
        let mut births_with_events = 0u64;
        let mut sterile = 0u64;
        let mut sterile_ops: BTreeMap<String, u64> = BTreeMap::new();
        let births = 2000u64;
        for b in 0..births {
            let mut child = genome.clone();
            let mut rng = SmallRng::seed_from_u64(500_000 + b);
            let summary = MutationEngine::apply_mutations_with_food_type_count(
                &mut child,
                &config.mutation,
                &reachable,
                ParentExecuted::NONE,
                &mut rng,
                food_types,
            );
            if summary.applied_events == 0 {
                continue;
            }
            births_with_events += 1;
            if !emits_reproduce(&child, &fed, &runtime, 150.0) {
                sterile += 1;
                for (op, n) in &summary.applied_by_operator {
                    *sterile_ops.entry(format!("{op:?}")).or_default() += u64::from(*n);
                }
            }
        }
        println!("{}", serde_json::json!({
            "kind": "sterility", "variant": variant, "births": births,
            "births_with_events": births_with_events, "sterile": sterile,
            "sterile_per_birth": sterile as f64 / births as f64,
            "sterile_per_mutated_birth": sterile as f64 / births_with_events.max(1) as f64,
            "sterile_ops": sterile_ops,
        }));
    }
}
```
