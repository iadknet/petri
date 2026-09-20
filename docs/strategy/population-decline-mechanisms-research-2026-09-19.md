# Population decline: reproduction, mutation, and ecological feedback

Research date: 2026-09-19. Source snapshot: `45c8063722fa3aa509a32d523f5e477e9fcd3853`, with per-file hashes and initial working-tree status archived. This investigation changes no production behavior, defaults, or user recipes. It is not an executable feature specification.

**The strongest conclusion is a failure of replacement arising from interacting mechanisms, not established universal deterioration of genomes.** The founder's reproductive decision does not track its physiological cost. Correcting one consequence of that mismatch greatly improves reproduction with abundant food, but the tested correction performs poorly in ecology. Mutation imposes a measurable reproductive load, yet sampled late offspring remain reproductively competent and outperform early offspring in a matched feeding assay. Aging merits further controlled testing; neither smaller genomes nor a larger census establishes sustained adaptive evolution.

The original reported extinction near tick 15,000 remains unexplained at the level of a particular historical run: its simulation seed, live changes, and population state were not supplied. The earlier full-world control declined severely but still had 645 creatures and ongoing births at tick 15,000. These are different observations.

[Machine-readable results](population-decline-mechanisms-research-2026-09-19.results.json) contain the new run trajectories, resolved configurations, seeds, assay summaries, uncertainty estimates, source identity, and artifact hashes. The local [research archive](../../.bench-artifacts/population-decline-2026-09-19/) contains frozen source, executable probes, input genomes, raw output, analysis, and replay scripts.

![Reproductive capacity, ecological outcomes, and census are different measurements](../../.bench-artifacts/population-decline-2026-09-19/findings.png)

**Current code and historical controls**

I read the two supplied reports and their result JSONs, inspected the archived probes, and checked the implementation rather than treating historical labels such as “current” as today's defaults.

| Setting | Production default now | Original recipe loaded now | Relaxed/reduced-copy recipe used here |
| --- | ---: | ---: | ---: |
| Large-copy relative weight, percent | 25 | 25, because omitted | 25, omitted in relaxed / explicit in reduced-copy |
| Executed-node targeting bias | 0.9 | 0.9 | 0 |
| Mutation events | Binomial(G, 0.005) | Same | Same |
| Genome size pressure | Disabled | Disabled | Disabled |
| Genome carry cost/unit/tick | 0.0001 | approximately 0.0001 | 0.00005 |
| Genome replication coefficient | 0.1 | approximately 0.01 | 0.005 |
| Basal decay/tick | 0.5 | 0.02 | 0.02 |
| Maximum age action multiplier / age cap | 10 / 500 | 305 / 600 | 305 / 600 |

Thus the new primary control is **current code with the supplied reduced-copy recipe**, not `SimulationConfig::default()`. The historical mutation control used copy weight 100. Replaying an old recipe with an omitted field today silently selects 25; the archive includes an explicit historical-100 recipe. The prior 4/15, 5/15, and 3/15 extinction counts remain historical experimental arms and are not pooled with the new seeds.

Relevant owners: [configuration](../../crates/v3-core/src/config/simulation.rs), [reproduction](../../crates/v3-core/src/simulation/actions/reproduction.rs), [founder](../../crates/v3-core/src/creature/founder.rs), [mutation supply](../../crates/v3-core/src/mutation/engine/mod.rs), and [copy weights](../../crates/v3-core/src/mutation/topology/mod.rs).

**Causal map and priorities**

| Rank | Mechanism → observable prediction | Evidence and discriminating experiment | Present conclusion |
| --- | --- | --- | --- |
| 1 | Fixed reproductive gate + rising charges → rejected attempts displace foraging; age reduces reproductive opportunities | Trace actual charges; paired feeding-fallback assay; age intervention | Decision/physiology mismatch confirmed; feeding-only correction is unsuitable as a persistence fix |
| 2 | Aging × movement/replication costs → declining ability to obtain food and replace deaths | Separate costs on matched genomes; compare gentler aging; inspect interval deaths | Strong candidate interaction; global age intervention remains insufficiently resolved into its components |
| 3 | Growing mutation supply + disruptive operators → fewer reproductively competent offspring | Actual paired births with mutation on/off, preserving inherited state | Measurable local reproductive load; no monotonic loss of competence across saved cohorts |
| 4 | Local food depletion/diet/movement → global surplus cannot support local reproduction | Typed local food; occupancy-off control; fixed-founder interaction | Spatial/resource feedback matters; total resource exhaustion and occupancy depletion alone are inadequate explanations |
| 5 | Bottlenecks, clade concentration, selection for local success → loss of replacement or diversity despite survivors | Lineage trajectories, cohort follow-up, reciprocal transplants | Clade concentration observed; drift, adaptive suicide, and irreversible mutation accumulation are not established |
| 6 | Missing/poor learning or dormant recruitment → useful strategies are not discovered, retained, or inherited | Trace execution and rewards; matched live-state inheritance and learning ablations | Substrate limitations identified; a general cognition or inheritance implementation defect is not demonstrated |

The feedback worth testing is: increasing costs or behavioral disruption reduce recruitment; demographic contraction changes selection and accessible resources; persistent old nonbreeders can inflate the census; local feeding and movement determine whether lower density restores replacement. Every arrow needs an intervention or a temporally resolved measurement. Extinction alone does not identify the initiating arrow.

**Reproductive economics: what actually happens**

The tick first grows food, increments age, debits basal decay and genome maintenance, and removes deaths. Cognition reads a frozen world snapshot. Priority ordering and cognitive costs precede sequential action effects; reward learning follows actions. Cognition can therefore choose an action whose affordability or spatial target differs when it is applied. See [tick orchestration](../../crates/v3-core/src/simulation/tick.rs).

Reproduction checks target, population cap, age, post-charge parental energy, and child endowment before charging. Invalid targets are checked before energy, so energy-rejection counts do not include every physiologically unaffordable request. Successful births debit the actual cost and then the transfer; the child receives the transfer. A rejected request is free under these recipes, including their zero failed-action penalty. The old charge-before-gate defect is not the current behavior.

For the new control, with complexity cost disabled:

`charge(age, G) = [1 + 304 × min(age/600, 1)²] × [1 + 0.005 × max(G−111, 0)]`.

The founder requests a child with two-thirds of post-charge energy. Its transfer must be at least 20, while post-charge parental energy must be at least 30; the child cap of 100 is an upper bound, not a fixed transfer. Consequently the founder needs `30 + charge` before reproduction. The 30 is not a retained reserve after provisioning.

The founder's controller instead uses fullness greater than 0.16, or energy greater than 32 at a cap of 200. Even at founder size the true charge exceeds 2 around age 35. At age 100 its minimum energy is about 39.44, and at age 200 about 64.78. Its terminal reproductive branch does not then execute its forage branch. Fixed-gate attempts can occupy much of its useful reproductive period.

Even at maximum energy, the approximate last affordable ages are 447, 306, and 189 for sizes 111, 333, and 999 under the new recipe. These are physiological upper bounds, not measured reproductive lifespan. Actual reproduction can stop much earlier. Aging multiplies movement and reproduction charges, but not basal decay, carrying, or cognition. Free eating stays free. There is no mandatory death at age 600; well-fed nonbreeders can survive much longer.

**Controlled feeding intervention**

The research-only intervention invokes the existing primary-food Eat executor immediately after an actual energy-rejected reproductive action. It preserves reproductive gates, charges, provisioning, and food consumption accounting. It adds a feeding action and outcome, bypasses controller choice, and does not add movement. It is a causal diagnostic, not a sensor implementation or a production-ready policy.

The assay uses an 8×8 world, replenishes primary food to density 1 every tick, disables terrain, fertility, grazing, and occupancy depletion, starts each saved genome with fresh state at age zero and energy 20, and removes newborns. Stop at death or 600 ticks. Fruit remains governed by the recipe's ordinary food dynamics. Genome mutation is disabled; newborn phenotype changes cannot feed back because newborns are removed. The parent therefore experiences generous feeding without descendant competition.

| Parent genome cohort | Control offspring | Fallback offspring | Paired difference, conditional 95% bootstrap interval | Improved / worsened |
| --- | ---: | ---: | --- | ---: |
| Founder | 7 | 22 | Single reference genotype | 1 / 0 |
| Historical tick 1,000, n=256 | 6.168 | 21.828 | +15.660 [14.711, 16.625] | 242 / 0 |
| Historical tick 7,000, n=256 | 6.813 | 12.695 | +5.883 [5.102, 6.727] | 234 / 0 |
| Historical tick 15,000, n=256 | 7.680 | 14.727 | +7.047 [6.301, 7.813] | 240 / 0 |

The number of zero-offspring genomes fell from 17 to 13, 28 to 16, and 28 to 12 respectively. These results establish substantial recoverable reproductive capacity in the sampled controllers under this environment. They do not prove ecological rescue. Living parents at tick 600 are censored: 23/25/13 controls and 39/42/41 fallback parents survived. Counts are offspring by the horizon, not complete lifetime fecundity for those survivors.

The cohorts are the earlier study's first 256 slot-map entries from one originating world. They share ancestry and are not random independent population samples. Bootstrap intervals resample sampled parents and describe conditional assay uncertainty; they exclude uncertainty across worlds and genealogies.

**Ecological screen and fresh-seed follow-up**

The screen used paired seeds 11, 29, and 47 in 64×64 worlds with 16 founders, homogeneous fertility, no terrain, and the supplied food/aging/cost settings. It ran five arms to extinction or tick 5,000: control, feeding fallback, genome mutation plus phenotype drift off, maximum aging multiplier 10 with cap still 600, and occupancy depletion off. Other settings were held fixed initially. This preserves approximate initial density but not the full world's patch geometry, founder diversity, or drift regime.

| Screen arm | Extinct by 5,000 | Runs with births in final 1,000 ticks | Final populations, seeds 11 / 29 / 47 |
| --- | ---: | ---: | --- |
| Control | 1/3 | 1/3 | 2,460 / 0 / 2 |
| Feeding fallback | 2/3 | 1/3 | 0 / 534 / 0 |
| Mutation and phenotype drift off | 0/3 | 2/3 | 178 / 154 / 12 |
| Age maximum 10 | 0/3 | 3/3 | 106 / 79 / 100 |
| Occupancy depletion off | 0/3 | 1/3 | 874 / 2 / 11 |

A surviving population with no recent births is not evidence of sustained evolution. In control seed 11, the 2,460 survivors had median age 2,460 and no births in ticks 4,001–5,000. All descended from one founding lineage. The earlier study's fruit innovation in this run increased food utilization and census, but did not establish sustained replacement at the endpoint.

After the screen, both fallback and gentler aging were tested on **eight fresh paired seeds, 201–208**, with the horizon and outcomes fixed in advance. These seeds were not used in the earlier 15-seed study.

| Fresh-seed arm | Extinct by 5,000 | Runs with late births | Median final population | Median births in final 1,000 |
| --- | ---: | ---: | ---: | ---: |
| Control | 1/8 | 6/8 | 145.5 | 622.5 |
| Feeding fallback | 3/8 | 1/8 | 1.5 | 0 |
| Age maximum 10 | 0/8 | 8/8 | 128 | 577.5 |

The paired mean late-birth difference for gentler aging was +114, with a seed-bootstrap 95% interval of approximately −417 to +603. For fallback it was −1,263 [−2,908, −203]. Eight seeds are few, outcomes are skewed, and the candidates were selected after screening. Gentler aging's 0/8 extinctions still has a Wilson 95% upper bound of about 32%. Its 8/8 late-breeding fraction has a lower bound of about 68%. These are promising diagnostics, not proof of a lower long-run extinction rate or an optimal setting. “Some late births” also does not mean births balance deaths or that newborns themselves recruit.

The full 1,600×1,600 map with 10,000 founders was then checked at simulation seed 11 through tick 1,000:

| Arm | Births | Deaths | Living population |
| --- | ---: | ---: | ---: |
| New recipe control | 104,816 | 100,264 | 14,552 |
| Feeding fallback | 115,046 | 112,226 | 12,820 |

More births coexisted with more deaths and fewer survivors. Both arms had zero recorded fruit intake through this early horizon. This preserves the original spatial scale but does **not** validate collapse prevention at tick 15,000. A full-horizon replicated experiment remains outstanding.

**Mutation load and actual inheritance**

The engine draws `Binomial(parent size, 0.005)` requested events, without the legacy probability/minimum/maximum/continuation gate. Reducing the three copy weights changes normalized operator frequencies; it does not reduce event supply. Historical operator assays established that these copies generated most added units and that their reduction slowed expansion. They did not establish ecological rescue.

The new assay tests actual applied births rather than constructing children from genomes alone. It takes parent indices 0, 8, …, 248 from each historical cohort and eight seeds per parent, `40000 + 100 × parent_index + replicate`. Each parent starts fresh in generous feeding conditions and runs until its first birth or 600 ticks. Paired arms permit or disable genome mutation and phenotype drift for that birth. The selected child retains its actual energy endowment, shared memory, and inherited plasticity weights, is moved to a matched isolated feeding world, and is followed for up to 600 ticks with its own descendants removed and mutation disabled. An assertion verifies exactly one birth contributed each attributed mutation count.

| Cohort | Born pairs / attempted | Applied events per mutated birth | Net size change | Child's offspring: no mutation → mutation | Lost ability to produce offspring by horizon |
| --- | ---: | ---: | ---: | ---: | ---: |
| Tick 1,000 | 248 / 256 | 0.540 | +0.431 | 5.903 → 5.528 | 12/248 (4.84%) |
| Tick 7,000 | 234 / 256 | 1.761 | +3.419 | 7.812 → 7.419 | 9/234 (3.85%) |
| Tick 15,000 | 248 / 256 | 3.173 | +3.899 | 8.992 → 8.823 | 7/248 (2.82%) |

Requested and applied event means were equal in this sample. Failed parental births were identical across paired arms and excluded from the child comparison. All unmutated children in the born pairs reproduced; no mutation arm gained reproduction from a nonbreeding paired child. Conditional parent-cluster bootstrap intervals for the mean offspring differences were −0.589 to −0.185, −0.592 to −0.213, and −0.363 to −0.012 respectively. Operator counts are retained in the JSON; outcomes of multi-event births must not be attributed causally to each participating operator.

This confirms a reproductive cost of the sampled mutation process. It does **not** demonstrate progressively worsening mutation load: event supply increased, but reproductive-response losses decreased across these convenience cohorts, and late offspring produced more descendants in this assay. Selection for robustness, changed strategies, survivor sampling, and other cohort differences remain possible. The assay measures grandchildren produced, not their eventual ecological recruitment.

Historical executed-bias experiments remain relevant to production defaults: adding disconnected copies increased damage to an unchanged active controller at bias 0.9, while bias 0 greatly attenuated it. That coupling is absent from this study's recipe control but remains possible in production defaults and the original recipe. Uniform targeting is not necessarily uniform per-instruction hazard, because domain and node selection still matter. No biological insertion/deletion ratio or universal complexity penalty follows from these results.

**Cognition: discovery, expression, retention, transmission**

The founder senses primary food, energy fullness, age, and nearby occupancy. It does not initially exploit fruit. Graph and VM backends can express multiple actions, memory, plasticity, reward modulation, and broader perception, but the founder's branching determines which are actually used. There is no direct introspection key for the true reproductive charge or the requested litter's affordability; available introspection exposes age and energy. Age sensing saturates at its reference horizon, while recipe action aging continues to age 600. A learner therefore has neither an automatic affordability repair nor an automatically informative reproductive reward.

The active limits include 10 actions per turn, 10,000 steps per VM execution, and 1,024 mesh hops. `mutation.action_queue_cap = 4` sizes newly created graph action banks; it is not a separate four-action VM runtime ceiling. The stored graph relaxation limit of 15 and convergence settings are inactive under the current world-tick graph clock. Raising them cannot supply extra learning or recurrent integration. Those capacities do not prove that a particular evolved circuit can reach or use them. No new trace evidence establishes execution-budget exhaustion or dormant-circuit recruitment as the initiating cause here. Budget/termination traces and actual sensor/route usage should precede raising limits or replacing the controller.

[Outcome signals](../../crates/v3-core/src/simulation/outcomes.rs) expose post-Phase-0 energy change, action-success fraction, damage, and offspring count as separate channels. Provisioning can produce negative energy reward despite reproductive success. Mutation must establish useful learning circuitry and a suitable channel; zero learning cost does not install a learner. Nor does zero learning energy debit prove that learning did not occur.

Genome inspection found plasticity configurations in 18/256 early, 217/256 middle, and 52/256 late saved creatures. Corresponding configuration counts were 21/705/64, Lamarckian flags 12/363/44, and reward-modulated configurations 1/72/3. These are structural opportunities, not measurements of active or helpful learning.

The actual birth path copies shared memory and selectively transmits learned weights when the relevant plasticity is Lamarckian. Birth correspondence follows structural mutation; recurrent graph state and eligibility credit reset. The new child assay preserves that actual newborn state. Existing reproduction tests covering this behavior pass. Historical archived genomes lack their parents' original live learned state, so this is not a replay of learning in the original collapsing world. There is no demonstrated general inheritance corruption, and the earlier learning-disabled fed assay did not support a large systematic learning penalty.

**Food, movement, and population structure**

The historical full-world control had large residual food stocks, almost no fruit use, and movement as the dominant terminal energy sink. Reanalysis of interval ledgers gives:

| Historical interval | Births | Deaths | Movement-attributed deaths |
| --- | ---: | ---: | ---: |
| 2,001–3,000 | 40,583 | 43,072 | 38,687 |
| 6,001–7,000 | 13,733 | 15,303 | 13,228 |
| 9,001–10,000 | 6,472 | 7,004 | 6,097 |
| 14,001–15,000 | 5,619 | 5,624 | 4,990 |

A terminal sink is not necessarily the root cause: an unfed creature may die on a movement charge because earlier foraging failed. The final interval was approximately balanced at low population, not demonstrably an irreversible collapse.

Food types coexist; the untyped `food_at` accessor sums their densities. A fruit-rich cell may have little primary food. The instrumentation audit caught initial diagnostic fields that mistakenly labeled this total as primary/grass. Those fields are excluded from conclusions. Corrected typed-food replays retained identical primary trajectories. In seed 201, mean primary density at an energy-rejected request was 0.753 in control and 0.00115 with fallback. The control eventually had only two old survivors despite both having nearby accessible primary food. Low density and food recovery therefore did not automatically restore reproduction in that run.

Occupancy depletion recovers by 0.03 per unoccupied tick and has a growth-multiplier floor of 0.35; grazing has a separate, much longer recovery process. Neither is permanent destruction of the whole food supply. The occupancy-off screen failed to establish restored replacement. It does not rule out grazing, patch disconnection, type-specific resources, or the interaction of movement and aging.

A secondary fixed-founder/fallback interaction ran seeds 11/29/47: final populations were 0/2/11, with 0/0/1,132 late births. Corresponding fixed-founder controls were 178/154/12 with 1,015/918/0 late births. Thus failure can occur without new genome mutations, and the intervention's effect changes with ecological history. The plausible mechanism is repeated feeding without relocation suppressing local primary-food regrowth and changing time spent waiting versus moving. This interpretation needs action/position replays or a controlled relocation intervention; the current intervention also changes learning outcomes and competition.

Founder-lineage concentration is visible, but lineage labels do not measure within-clade genetic diversity or effective population size. Old sterile survivors cannot transmit sterility themselves. Their abundance can reflect long residence time, while their fertile relatives or parents generated them earlier. Demonstrating mutation accumulation, drift-driven collapse, or evolutionary suicide requires tracking reproductive lineages and showing the relevant genotype/environment feedback, not just a late census dominated by nonbreeders.

**Research basis and transfer limits**

Primary sources checked on 2026-09-19; recent work is combined with foundational experiments where directly relevant.

| Source | Material finding and implication for Petri | Transfer limit |
| --- | --- | --- |
| [Luiselli et al., 2025, structural mutation equilibrium](https://pubmed.ncbi.nlm.nih.gov/41315007/) | Mutation neutrality and structural hazard can jointly regulate noncoding genome fraction; measure payload and functional hazard | Petri's graph/VM operators and targeting differ from genomic breakage; the paper supplies no Petri cost coefficient |
| [Luiselli et al., 2024, Aevol streamlining](https://pubmed.ncbi.nlm.nih.gov/39566106/) | Higher population size and mutation rate can both shrink genomes but preserve different amounts of coding material | Smaller genomes do not establish better behavior or maintained reproduction |
| [Kumawat et al., 2025, digital evolvability](https://pubmed.ncbi.nlm.nih.gov/39739809/) | Mutation rate and the distribution of mutational effects can evolve together; repeated and novel environments reward different capacities | Petri's global mutation policy is externally configured; test functional neighborhoods and changed environments rather than importing rates |
| [LaBar and Adami, 2016](https://journals.plos.org/ploscompbiol/article?id=10.1371/journal.pcbi.1005066) and [2017](https://www.nature.com/articles/s41467-017-01003-7) | Digital populations can reach complexity through different population-size paths and can evolve drift robustness | Avida's replacement/fitness regime differs; Petri's tiny worlds cannot estimate full-world drift or collapse probabilities |
| [Luo et al., 2024, robot inheritance study](https://arxiv.org/abs/2403.19545) | Comparing newborns and learned controllers can expose benefits of inherited learning | This arXiv study uses designed robot learning/evaluation; Petri has online ecological selection and must evolve useful learning circuitry |
| [Gupta et al., 2021, embodied learning and evolution](https://www.nature.com/articles/s41467-021-25874-z) | Evolution can favor structures that learn faster under explicit training and selection | Its training budget and fitness protocol cannot be assumed for Petri's local reward signals |
| [Parvinen and Dieckmann, 2013](https://pubmed.ncbi.nlm.nih.gov/23583808/) | Optimizing selection can lead to extinction through ecological feedback | Theoretical possibility does not diagnose Petri's run |
| [Zamorano et al., 2023](https://www.sciencedirect.com/science/article/pii/S0960982223008412) | Reciprocal ecological/evolutionary feedback can also stabilize a community | Feedback is not inherently destabilizing; reciprocal interventions are needed to determine its sign here |

These studies motivate the experiments and alternatives, rather than supplying numeric tuning rules. None establishes that keeping a census high maximizes evolvability.

**Changes worth testing, with acceptance criteria**

The decision is to extend Petri's existing execution, sensing, and experimental paths. The strongest alternatives are already available: configured founder profiles, isolated cost changes, and operator-level mutation controls. A wholesale controller replacement or an externally rewarded neuroevolution package changes the scientific question and is not supported by an identified substrate failure. Automatic reseeding preserves a display population but does not meet the goal of sustained adaptive evolution.

| Priority / proposed change | Evidence and expected effect | Risk / strongest alternative | Acceptance criterion before production use |
| --- | --- | --- | --- |
| 1. Give reproductive decisions faithful affordability information and test a composed forage/move/reproduce policy within the existing founder work | The 32-energy gate ignores applied age/size costs; generous-food fecundity is recoverable | Free feeding fallback failed; merely switching to the existing forage-first profile already lacked a historical full-world benefit. A cue might also reduce exploratory behaviors | Cue agrees with actual gate at execution-relevant age, size, fraction, caps, and action ordering; boundary tests preserve free refusals and energy conservation. In ecological tests it improves recruited descendants, not just request acceptance, without suppressing movement or diet innovations |
| 2. Split aging experiments into movement aging, reproduction aging, and their interaction; include matched genome-cost arms | Gentler aging maintained late births in all eight fresh seeds, while movement dominates terminal deaths | Blanket age relief changes selection, turnover, and carrying capacity; multiplier 10 is a probe, not a calibrated optimum | Paired full-size runs beyond 15,000 show sustained replacement and retained adaptation; age-specific first birth, offspring recruitment, and realized selection distinguish which component helps |
| 3. Add applied rejection/eligibility and birth-cohort observability through existing reports | Census and global food repeatedly mislead; actual outcomes already have authoritative owners | Parallel telemetry calculations can drift, as the research's initial food accessor demonstrated | Population and energy ledgers balance; typed resources remain distinct; recorded fraction/cost/age/target explain the actual outcome; living cohorts remain censored; mutation counts attribute to the relevant child |
| 4. Preserve copy weight 25 as the current control and separately test mutation locality at default bias 0.9 | Growth bias is established and actual births show load; disconnected material can amplify active-controller hazard under biased targeting | Broad addition suppression also suppresses learning toggles; disabling all mutation sacrifices future adaptation | Behavior-preserving dormant additions do not multiply active-controller reproductive damage merely through increased targeted supply; full-world adaptation and offspring recruitment remain viable. Do not infer optimality from genome size |
| 5. Improve learning or recruitment only after measuring a failed capability | Learning mechanisms and selective inheritance exist, but availability is not use | Adding learning machinery or raising budgets without a demonstrated bottleneck adds mutation targets and cost | A matched task shows discovery, expression, retention, and transmission separately; useful learned behavior increases descendant reproduction and survives environmental changes |

These are research priorities, not authorization to implement the proposed production changes.

**What would change the conclusions**

1. **Affordability:** a controller-aware cue/composition intervention that improves recruitment in depleted, patchy environments would elevate this from a confirmed individual constraint to an ecological remedy. Another abundant-food gain alone would not.
2. **Aging:** separate movement-only and reproduction-only aging arms on identical seeds and genomes would identify the smallest useful change. If benefits disappear at full scale or mainly prolong nonbreeders, de-prioritize blanket age relief.
3. **Mutation:** freeze mutations in descendants of populations immediately before, during, and after decline. Persistence of failure with unchanged descendants would weaken ongoing load as the main driver; improved newborn-to-breeder recruitment would strengthen it. Freezing at founding alone changes the entire evolutionary path and cannot answer this fully.
4. **Genotype versus environment:** save complete live states and world/RNG state, then cross early/late controllers with early/late resource fields at matched density and age/endowment. Hold controllers fixed while swapping resources, and separately replace controllers while retaining environment. Include inherited-state and fresh-state arms. This is the highest-value missing reciprocal-transplant experiment.
5. **Selection and cognition:** track successful parental lineages, offspring-to-breeder transitions, phenotype retention, actual reward updates, dormant recruitment, and budget termination before collapse. Evidence that a locally favored strategy invades and then destroys its own resource base would support evolutionary suicide; declining matched-environment descendant performance would support deterioration.
6. **Confirmation:** run the original 1,600×1,600 map to at least 15,000 ticks, preferably 20,000, with independent paired seeds and predefined endpoints. The provisional next panel is seeds 301–308. Analyze time to extinction with right censoring, births/deaths by interval, cohort replacement, diet retention, and clade diversity; use the seed pair as the experimental unit. Eight is a starting panel, not a power claim; estimate variance before fixing a larger confirmation sample.

**Reproduction and verification**

Completed: 15 screening worlds, 24 fresh-seed worlds, two full-size early-horizon worlds, three fixed-founder interaction worlds, and two typed-food replays; 1,542 fed evaluations; 768 paired actual-birth attempts yielding 730 pairs of newborns and 1,460 child evaluations. Duplicate assay replays used for attribution verification are not independent observations.

The initial and corrected research instrumentation are both retained. Initial fields with misleading primary-food labels and the one-tick cohort-bin convention discrepancy are documented in `PLAN.md` and excluded from affected inference. Corrected seed-201 replays matched population, births, deaths, attempts, size, and food-intake trajectories exactly. The unmodified-production-linked parity binary matched every shared recorded field through tick 500, apart from wall time. All recorded world population ledgers balanced.

The feeding test failed before the isolated patch and passed afterward. All 28 core viability tests and 16 reproduction unit/property tests passed; the final fallback test passed. The child assay asserted age-zero newborns, a single attributed birth, exact no-mutation genome inheritance, and zero applied mutations in that arm. `make check-docs`, shell syntax checks, and `git diff --check` passed. Production core hashes remained unchanged. Research runs used one simulation thread. Wall times are not performance benchmarks; independent research processes sometimes overlapped.

From the repository root, replay into a fresh temporary directory:

```sh
sh .bench-artifacts/population-decline-2026-09-19/reproduce.sh
```

This uses the archived core and lockfile. Replays use corrected diagnostics; initial raw diagnostic schema differences are explained above. The ignored archive must be copied separately to reproduce this work on another machine; the Markdown and JSON are the durable report and summaries, not a substitute for its input genomes and source.

An exact outstanding full-horizon control command, after building the archive:

```sh
archive=.bench-artifacts/population-decline-2026-09-19
out=$(mktemp -d "${TMPDIR:-/tmp}/petri-decline-full.XXXXXX")
unset PETRI_RESEARCH_FEED_FALLBACK
for seed in 301 302 303 304 305 306 307 308; do
  for arm in control mild-age; do
    "$archive/target/release/world" "$archive/inputs/full-$arm.json" \
      "$seed" 20000 "$out/$seed.$arm" > "$out/$seed.$arm.ndjson"
  done
done
```

That command compares the existing broad age probe. Movement-only/reproduction-only aging, faithful affordability cues, checkpoint mutation freezes, reciprocal transplants, and complete live-state snapshots require additional isolated instrumentation and have **not** been implemented or run here. The new full-size experiments stopped at 1,000 ticks for practical session duration; no surviving run is labeled permanently stable.
