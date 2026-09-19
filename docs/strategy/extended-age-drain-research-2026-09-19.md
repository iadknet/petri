**Extended-age-drain world: evolution, energetic costs, and slow population collapse**

Research date: 2026-09-19. Source revision: `925d5d76efa7d8811f7cd44b3d07d65f32ce6000`. The supplied recipe was treated as data. This is an investigation, not a feature specification or an executable roadmap change.

The user's reported run maintained roughly 10,000 creatures before declining to extinction around tick 15,000. The recipe fixes the world/map seed at `5786586509898421`, but does not contain the separate simulation seed, a saved population, or the history of live configuration changes. The experiments below use simulation seed 11 and the current code. They diagnose mechanisms and a controlled trajectory; they cannot establish the exact history of the user's original run.

The strongest new controlled measurement concerns an interaction between size-proportional mutation supply and executed-node-biased mutation targeting. Making a genome larger without changing its active controller sharply increases the probability of disrupting that controller. Separately, the recipe charges larger genomes more to reproduce and multiplies that charge by an unusually steep aging factor. These liabilities grow while the population evolves, even though the world configuration does not change. A terminal reproduction branch then converts many unaffordable reproductive decisions into missed feeding opportunities.

Population counts alone cannot identify this mechanism. Selection does not optimize population size, and the oldest survivors are not necessarily the genotypes contributing the next generation. Conversely, seeing surviving nonbreeders does not establish that a sterile allele swept through the reproducing population: fully sterile individuals cannot transmit themselves. Mutation supply, reproductive contribution, demographic filtering, and within-lifetime learning must be distinguished.

**The full-world result and its limit**

The control reproduced a large decline, but **not extinction**. It reached 48,514 at the tick-100 checkpoint, recovered into the 10–14k range after the first trough, then fell to 6,292 at tick 3,000, 3,324 at tick 6,000, 501 at tick 10,000, and 645 at tick 15,000. The lowest sampled count was 332 at tick 13,300. The last 1,000 ticks had 5,619 births and 5,624 deaths: this run was fluctuating in a low-population regime, not demonstrably in an irreversible terminal decline.

By tick 15,000 it had produced 325,370 offspring; mean lineage depth was 341.52, mean genome size 613.97 units, and median age 51 ticks. This is not simply the original founders aging out. Primary food stock was about 430,657 density units, with about 79,946 fruit units. Global resource exhaustion is not the explanation, although aggregate stocks cannot establish that the food is locally accessible to a given creature. Fruit contributed 1,369 energy out of about 17.23 million total food energy (0.00795%). The first positive fruit-intake checkpoint was tick 4,400.

All interventions use the same map, initial recipe, and simulation seed; stochastic paths diverge after an intervention. The cells below are population counts at **matched horizons**, not estimates of eventual survival:

| Intervention | Tick 3,000 | Tick 6,000 | Tick 15,000 |
| --- | ---: | ---: | ---: |
| Original recipe | 6,292 | 3,324 | 645 |
| Genome mutation and phenotype drift off | 15,866 | Not run | Not run |
| Maximum aging multiplier 10, cap still 600 | 22,175 | Not run | Not run |
| Existing forage-first sparse founder | 6,023 | Not run | Not run |
| Genome replication surcharge off | 6,555 | 2,487 | Not run |
| Executed-node mutation bias off | 5,244 | 1,954 | Not run |

These results prevent a one-knob recommendation. Turning off mutation or greatly reducing aging pressure substantially increased population at tick 3,000. Turning off executed bias did not rescue the world over the measured horizon. Removing replication cost did not rescue it either; at tick 6,000 its mean genome size had reached 1,018 units versus 272 in the control, exposing much more material to the mutation process. The current surcharge constrains growth even while directly limiting fecundity. The existing forage-first profile also failed to improve the tick-3,000 population. None of these single-seed comparisons establishes long-run survival probabilities or the exact cause of the user's original extinction.

A final convenience sample of 256 survivors further separates cognitive capability from physiological eligibility: 231 emitted reproduction somewhere in the fresh-state battery, 237 emitted eating, and only 21 were already too old/large to reproduce even at maximum energy. Nevertheless, 176 lacked the energy to meet the physiological gate at their recorded state. This is not evidence that every late survivor had become genetically sterile.

![Population, birth rates, and genome size across measured interventions](../../.bench-artifacts/extended-age-drain-research-2026-09-19/trajectories.png)

**What the recipe actually does**

The current reproduction implementation differs from the older collapse research. `default_offspring_energy = 100` is an upper limit. The founder requests a transfer of two-thirds of its post-cost energy, with a minimum child endowment of 20. Physiology requires at least 30 energy *after the reproduction charge but before the transfer*. This is not a 30-energy parental reserve after provisioning the child. The founder requests reproduction above 0.16 fullness, or 32 energy at this recipe's cap of 200.

The aging multiplier is

`M(age) = 1 + 304 × min(age / 600, 1)^2`.

It multiplies action costs. It does not multiply basal energy decay, genome maintenance, or cognitive execution costs. A zero-cost eat remains free. A rejected reproduce attempt is uncharged, and the recipe's failed-action penalty is also zero. Movement is charged even when blocked.

| Age | Movement charge | Reproduction charge, founder-sized genome | Minimum parent energy for the founder's requested litter |
| --- | ---: | ---: | ---: |
| 20 | 0.134 | 1.338 | 31.338 |
| 50 | 0.311 | 3.111 | 33.111 |
| 100 | 0.944 | 9.444 | 39.444 |
| 200 | 3.478 | 34.778 | 64.778 |
| 300 | 7.700 | 77.000 | 107.000 |
| 400 | 13.611 | 136.111 | 166.111 |
| 600 | 30.500 | 305.000 | 335.000 |

From age 448 onward, even a founder-sized creature at the maximum 200 energy cannot meet the reproduction gate. Larger genomes lose this capacity earlier. This is a fertility cutoff, not a mandatory death age. A creature making free rejected attempts can linger while paying mostly maintenance. At the other extreme, a creature eating a full grass cell and moving once per tick loses energy from age 241 onward: grass yields at most 5, while movement alone then exceeds 5. Rich fruit could change that balance, but the initial founder is wired to sense and eat food type 0 only.

The genome replication multiplier is

`R(G) = 1 + 0.01 × max(G − 111, 0)`.

The actual reproduction charge is `M(age) × R(G)`. Disabling `complexity_cost` does not disable this surcharge or the per-unit carrying cost. Likewise, `genome_size_cap = 1200` is not an enforced ceiling here: size pressure is disabled.

The effective mutation rule is `Binomial(G, 0.005)` requested events per birth. The displayed legacy probability 0.44, legacy event maximum 10, and continuation probability do not govern that rule while per-unit supply is enabled. Expected events rise from 0.555 at 111 units to 1.665 at 333 and 4.995 at 999. The executed-node bias remains 0.9 throughout.

**A controlled test of the mutation interaction**

Construct three parents with the same active two-node founder. The larger parents carry two or eight additional disconnected copies of both founder nodes, with their own node IDs. Assert that all three produce exactly the same action signature on the 24-scenario battery before mutation. Apply the current mutation engine to 10,000 offspring per condition, with the executed and reachable parent sets both `[0, 1]`. Repeat with only the executed bias changed to zero. Trial seeds are `900000..909999`.

| Genome units | Executed bias | Applied events / birth | Changed action signature | No reproduction anywhere in battery |
| ---: | ---: | ---: | ---: | ---: |
| 111 | 0.9 | 0.5513 | 13.40% | 2.93% |
| 333 | 0.9 | 1.6480 | 28.97% | 6.97% |
| 999 | 0.9 | 4.9810 | 60.07% | 18.99% |
| 111 | 0.0 | 0.5513 | 13.33% | 2.89% |
| 333 | 0.0 | 1.6480 | 13.17% | 3.26% |
| 999 | 0.0 | 4.9810 | 13.54% | 3.17% |

The history explains why both settings exist. [T11.F17](../specs/roadmap/t11-f17-executed-biased-mutation-targeting.md) deliberately concentrated a fixed per-birth mutation budget on executed nodes so accumulated dormant material would not dilute functional mutation. [T11.F19](../specs/roadmap/t11-f19-per-unit-mutation-supply.md) subsequently made the total budget proportional to genome size while explicitly retaining that bias. Earlier research measured increased behavior change and warned that growth was not bounded; this experiment specifically holds the active controller constant and measures loss of its reproductive response. The two mechanisms are implemented according to their specifications. The defect is in their combined evolvability tradeoff, not an arithmetic error or a claim that either feature failed its implementation contract.

![Disconnected-copy mutation assay](../../.bench-artifacts/extended-age-drain-research-2026-09-19/mutation-coupling.png)

This isolates a liability of the current genotype-to-phenotype mutation map. A larger dormant genome raises mutation supply, but the targeting rule directs most of that extra supply back toward executed nodes. With unbiased targeting, the disconnected material absorbs much more of the extra mutation supply. These are behavior-preserving additions before mutation, not fitness-neutral additions in the live world: carrying and replication costs still apply there.

The battery varies age over 20/100/400 ticks, energy over 20/40/100/190, and primary food here over 0/1. It supplies asymmetric primary and fruit neighbor rings, fruit on the current cell, no barriers or neighbors, and zero extended-perception fields. Each scenario starts with fresh runtime state. “No reproduction in battery” is a bounded observation, not proof of sterility in every possible environment or learned state. Action change is not synonymous with damage; the loss of a demonstrated reproductive response is the more specific signal here.

A separate 10,000-offspring founder probe at seeds `500000..509999` found 4,357 offspring with applied mutation events, 1,398 changed action signatures, 324 without reproduction in the battery, five with only NoOp, and three that emitted a fruit-eating action. Most mutation-bearing offspring preserved the tested signature. The problem is not that every mutation wrecks a brain; it is the accessibility and retention of useful changes, especially as genomes grow.

The same direction appears in actual saved genomes, with smaller effects. For each of 256 genomes at each checkpoint, draw 40 offspring using seeds `2000000 + 100 × genome_index + trial`. Use the existing production `Battery` to derive a proxy executed-node set, and the 24-scenario assay to compare parent and offspring behavior. Among births whose parent reproduced in the assay, loss of that response was:

| Parent sample | Current bias 0.9 | Bias 0 |
| --- | ---: | ---: |
| Tick 1,000 | 341 / 9,440 = 3.61% | 325 / 9,440 = 3.44% |
| Tick 7,000 | 459 / 9,720 = 4.72% | 276 / 9,720 = 2.84% |

Mean mesh nodes rose from 2.79 to 12.71 across those samples, while battery-executed nodes rose from 2.32 to 4.16. Units inside battery-executed nodes were 88.3% of total size in the early sample and 52.4% in the late sample. These are fresh-state battery measurements, not recorded live dispatch sets. They support a retention liability in actual evolved material while also showing that useful claims require more than the synthetic worst-case example. The late genomes were not simply identical founders with more junk; selection and mutation had changed their controllers.

**Young, well-fed assays separate behavior from the replication surcharge**

For each saved genotype, construct a fresh age-zero creature at energy 20 in an 8×8 wrapping world without terrain. Replenish primary food to density 1 before every tick. Disable mutation, grazing, and occupancy depletion. Keep the supplied action costs and aging, and remove newborns after each tick so offspring do not block the parent's targets. Run up to 600 ticks or parent death. This measures the parent's capacity to produce offspring under standardized generous conditions; it does not measure offspring survival, invasion fitness, or success in the original patchy habitat.

| Genotype source | Genomes | Mean offspring | Median offspring | Zero offspring |
| --- | ---: | ---: | ---: | ---: |
| Founder | 1 | 7 | 7 | 0 |
| Tick 1,000 sample | 256 | 5.934 | 7 | 17 |
| Tick 4,000 sample | 256 | 7.410 | 9 | 23 |
| Tick 7,000 sample | 256 | 5.402 | 6 | 28 |
| Same tick-7,000 genomes, replication surcharge zero | 256 | 9.375 | 11 | 28 |

Removing that one charge improved offspring production for 209 of the 256 tick-7,000 genomes, reduced it for none, and left the founder unchanged. This establishes a direct energetic contribution without changing those evolved controllers. The temporary increase in standardized reproductive performance from tick 1,000 to 4,000 also rejects a simple story of uniform deterioration in every behavioral capacity.

The saved sets are the first 256 living slot-map entries at each checkpoint, not random population samples. Their averages must not be presented as unbiased population estimates or as matched evolutionary lineages. Reconstructed creatures begin with fresh memory and graph state; current learned weights and inherited epigenetic state were not saved. The paired surcharge comparison uses exactly the same genomes and assay conditions in both arms, so that local causal comparison is stronger than cross-time comparisons.

A second paired assay zeroed each plasticity configuration’s learning rate before constructing the same tick-7,000 creatures. Mean offspring fell slightly from 5.402 to 5.340, with median 6 and 28 zero-offspring genomes unchanged. Four genomes improved and 23 worsened. This bounded assay does not support a general claim that learning is corrupting the controller; it does not isolate learning’s effects over the original evolving trajectory.

**Why available cognition does not guarantee adaptation**

The founder's graph has no plasticity configuration on its three compute nodes. Its action decisions are in a VM, with reproduction and forage on alternative terminal paths. Enabling free learning in the recipe makes an evolved learning mechanism cheap; it does not install learning or a useful teaching signal into every creature. Memory, plasticity, reward modulation, typed sensors, and extended perception need functioning connections to behavior before they can help.

The code supports reward-modulated learning, but its signals are immediate energy change, action success, damage, and offspring count. Energy loss from provisioning offspring is negative on the energy channel; it is not automatically combined with a positive offspring signal. There is no general optimizer repairing an unaffordable reproduction policy, choosing a new food, or protecting reproductive competence. This is a constraint of the present substrate, not a measured claim that reward learning caused this particular decline. A learning-disabled trajectory and saved live-state replays would be required to isolate that explanation.

The population can therefore acquire more graph nodes, more plasticity-bearing structure, or larger genomes without acquiring more effective adaptation. The repository's mutation “helpful” classification is also an observational composite of survival, energy, a reproduced-once flag, and invalid-action rate. It is not measured lifetime genetic contribution and does not direct selection. A good score or long survival cannot establish that a lineage is maintaining replacement.

**What I would address, and what the evidence does not justify**

First, make mutation targeting and mutation supply obey a coherent locality rule. If disconnected material is added while the active controller is unchanged, its mutation hazard should not automatically multiply with total genome size. The tested short-term alternative is `executed_bias = 0`. A more durable extension of the existing engine would make the function-targeted budget scale with the functional material being targeted, while size-proportional background mutations fall on the material that generated them. This needs operator/domain accounting, not simply a smaller global mutation probability. The disconnected-copy experiment provides a concrete acceptance assay. It does not establish that zero bias is the optimal production setting.

Second, remove the coupling between a refused reproductive request and forfeiting the whole feeding opportunity. The current founder explicitly ends the VM after queuing reproduction. Its gate was calibrated around a reproduction charge of 1; the actual charge varies with age and genome size. The existing T18 founder architecture work addresses composing multiple actions and separating motor paths. This investigation does not start that feature or alter its scope. Merely choosing the existing forage-first profile is a weaker intervention: it still uses alternative terminal branches rather than generally composing feeding, movement, and reproduction.

Third, calibrate senescence and genome costs together against viable reproduction over the intended life history. Genome maintenance and replication costs can be legitimate ecological constraints. Their multiplicative interaction with 305× action aging is much stronger than either parameter suggests in isolation. Removing the surcharge is a useful causal probe, not a recommendation to make all extra genetic material free. Lowering age pressure is another useful intervention, but a larger surviving census can include nonbreeders and must be judged with birth and recruitment rates.

Fourth, judge persistence beyond the standard 2,000-tick horizon, using several simulation seeds and the existing observation/reporting paths. Retain births and deaths by interval, newborn-to-breeder recruitment, age and genome-size distributions, food intake by type, reproductive rejection causes, and saved genomes *with live memory/weights*. Measure actual functional retention and reproductive contribution alongside structure. Do not use increasing genome size or a composite survival score as a proxy for improved cognition.

A population floor or automatic reseeding can keep a display populated but would not repair either demonstrated interaction. Moving reproduction entirely out of the brain would prevent some lesions while changing the experiment's premise. Neither is needed to explain these results. Replacing the whole brain with a different neuroevolution package is also not justified: the current implementation already has duplication, dormant structure, modular graph actions, and learning mechanisms. The measured problem is how those mechanisms are coupled and rewarded.

The scientific literature supports distinguishing these explanations. [Parvinen and Dieckmann (2013)](https://pure.iiasa.ac.at/10708/1/IR-13-058.pdf) demonstrate that selection can drive extinction through ecological feedback; selection does not promise maximum population size. [Lynch and Gabriel (1990)](https://epub.ub.uni-muenchen.de/5071/1/Gabriel_wilfried_5071.pdf) model mutation load and extinction in finite populations. That makes mutation-driven decline a legitimate possibility, but a shrinking population alone does not diagnose their formal mechanism or Muller's ratchet. Here the directly demonstrated result is increasing functional damage with genome expansion under biased targeting.

[Stanley and Miikkulainen's NEAT paper (2002)](https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf) emphasizes incremental structural growth and protecting new structures while they optimize. The transferable lesson is that structural variation needs a route to retention; the paper is not evidence that Petri should import NEAT's entire selection scheme. [Ofria, Bryson, and Wilke's Avida chapter (2009)](https://www.cse.msu.edu/~ofria/pubs/2009AvidaIntro.pdf) explicitly discusses the possibility of nonreplicating organisms persisting without a removal mechanism. Avida's age/death policy solves a different demographic problem; an action-cost multiplier that leaves eating and failed reproduction free is not an equivalent maximum lifespan.

Relevant current code:

- [Mutation supply and targeting](../../crates/v3-core/src/mutation/engine/mod.rs), especially `requested_event_count` and the construction of `TargetSets`/`TargetSelector`; [executed-node selection](../../crates/v3-core/src/mutation/reachability.rs).
- [Age multiplier](../../crates/v3-core/src/config/simulation.rs), `AgeEnergyCostConfig::multiplier`; [replication multiplier and birth gates](../../crates/v3-core/src/simulation/actions/reproduction.rs).
- [Founder energy policy and terminal branches](../../crates/v3-core/src/creature/founder.rs); [founder graph and absent initial plasticity](../../crates/v3-core/src/creature/cgp_founder.rs).
- [Basal and genome maintenance](../../crates/v3-core/src/simulation/tick.rs), `phase_0_energy_charge`; [eating, movement, and NoOp charges](../../crates/v3-core/src/simulation/actions/mod.rs).
- [Immediate learning signals](../../crates/v3-core/src/simulation/outcomes.rs); [reward-modulated weight updates](../../crates/v3-core/src/runtime/plasticity/reward.rs).
- [Observational mutation score](../../crates/v3-core/src/simulation/simulation.rs) and [its retrospective classification](../../crates/v3-core/src/simulation/stats.rs).
- [Existing T17 scope and terminal-branch hazard](../roadmaps/t17-brain-boundary-evolvability.md); [existing T18 founder architecture work](../roadmaps/t18-founder-architecture.md).

Earlier local audits are historical evidence, not measurements of this revision. In particular, the September 4 founder shape, older mutation schedules, and pre-T17 raw energy/age scaling have changed. Their quantitative defect rates were not substituted for fresh measurements here.

**Reproduction and verification**

The [machine-readable results](extended-age-drain-research-2026-09-19.results.json) contain every sampled population/birth/genome checkpoint, assay totals, the final full telemetry, and SHA-256 identities for local raw artifacts. Full NDJSON, genome samples, effective configs, plotting source, and the standalone Rust probes are under `.bench-artifacts/extended-age-drain-research-2026-09-19/` (ignored local research artifacts). The supplied recipe and generated counterfactual recipes are retained there. Runtime code was not edited.

The external Cargo package links to this checkout's `v3-core`. Its lockfile derives from the repository lockfile. It was built in release mode; world probes use a four-thread Rayon pool and the fed assays use one thread. Some independent runs overlapped, so their wall times are **not performance benchmarks**. The world probe calls the existing `seed_simulation` and `run_tick`; it samples every 100 ticks and saves at most 256 genotypes every 1,000. Population accounting was checked at every recorded world sample: initial population plus cumulative births minus all recorded deaths equals the living population.

Example reproduction from the repository root (the archived manifest records this checkout's absolute core dependency path):

```sh
cargo build --offline --release --manifest-path .bench-artifacts/extended-age-drain-research-2026-09-19/probe/Cargo.toml
.bench-artifacts/extended-age-drain-research-2026-09-19/probe/target/release/age-drain-probe \
  .bench-artifacts/extended-age-drain-research-2026-09-19/original-recipe.json \
  11 15000 /tmp/age-drain-replay > /tmp/age-drain-replay.ndjson
```

`assay` takes a recipe and optionally a saved genotype file. `junk_probe` takes the recipe. `evolved_probe` takes a recipe and saved genotype file. `fed_lifetime` takes the same two arguments and optionally `no-learning`, which zeroes plasticity learning rates. The raw results distinguish these research assays from the production neighborhood benchmark.

Verification: release probes completed; disconnected-copy parent signatures were asserted identical; population ledgers balanced; `make check-docs` passed. This investigation does not implement a roadmap feature or change an evolution parameter in production.
