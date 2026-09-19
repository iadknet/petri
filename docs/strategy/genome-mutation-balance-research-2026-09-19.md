# Genome growth and mutation balance: experimental investigation

Date: 2026-09-19. Research source revision: `925d5d76efa7d8811f7cd44b3d07d65f32ce6000`. The checkout advanced to `65d1083d39a932c9fc4ba6eb9bc33dee6eba5e38` during the investigation; `crates/v3-core` was unchanged between these revisions. All interventions ran in a separate source copy. Production code, defaults, and the user's recipes were not changed.

## Decision

**Keep genome-size pressure disabled. Investigate the rate of large structural copies before suppressing all additions or increasing all deletions.** The experiments identify a substantial mutation-generated growth bias, dominated by three whole-node/slice copying operators. Quartering their relative weights reduced mean units added per offspring by 49–62% across five parent cohorts, with much smaller behavioral effects than quartering all operators labeled increasing.

This is a candidate for further validation, not an established extinction fix or a calibrated optimum. In 45 small-world runs, reducing large copies did not establish a reliable persistence advantage. A larger controlled feeding assay found nearly unchanged reproductive output in most cohorts. These results do not justify changing production defaults yet.

This investigation follows the [earlier age-drain research](extended-age-drain-research-2026-09-19.md) and the [research supporting per-unit mutation supply](per-unit-mutation-supply-research-2026-09-14.md). It evaluates mutation policy under the relaxed-cost recipe; it does not re-estimate the best energy taxes, fix reproductive affordability, or take over the parent task's full-world run.

## Research basis

The literature motivates measuring rates, edit sizes, and fitness consequences separately:

- **Structural mutation and equilibrium:** Luiselli et al. (2025) model an equilibrium between the greater neutrality of additions and the greater structural-mutation hazard of larger genomes. This permits regulation without an imposed size ceiling, under their model's assumptions. It does not guarantee that arbitrary digital operators reach the same equilibrium. [Primary paper](https://pubmed.ncbi.nlm.nih.gov/41315007/).
- **Smaller is not necessarily better:** Aevol experiments distinguish losing noncoding material under larger populations from losing both coding and noncoding material under higher mutation rates. Genome size alone cannot distinguish useful streamlining from loss of function. [Genome Streamlining, 2024](https://academic.oup.com/gbe/article/16/12/evae250/7905804).
- **Expansion can precede innovation:** Avida experiments show expansion followed by modification into new functions. A separate experiment found that a strong deletion bias reduced extinction in very small populations but removed their prior advantage in evolving complexity. [Gupta et al., 2016](https://pmc.ncbi.nlm.nih.gov/articles/PMC4867773/); [LaBar and Adami, 2016](https://journals.plos.org/ploscompbiol/article?id=10.1371/journal.pcbi.1005066).
- **Balanced mutation as an engineering control:** UMAD balances additions and deletions for no expected size change before selection; recent work studies adaptive rates on that foundation. This is a useful comparison for program evolution, not a universal biological ratio to import into Petri. [Ni and Spector, GECCO 2024](https://arxiv.org/abs/2406.15976).

The research-first comparison favors testing operator-level balance over reinstating a size threshold. It does not supply the numeric factor one quarter; that factor is an experimental intervention here.

## Methods and scope

The resolved config comes from `world-recipe-extended-age-drain-relaxed-genome-costs.json`: executed bias zero, carry cost `0.00005`, replication coefficient `0.005`, and the original age multiplier of 305 at age 600. Per-unit supply remains `Binomial(parent genome size, 0.005)`. Mutation domains, targeting, repair, action physiology, and operator implementations remain unchanged.

Five parent cohorts were tested:

1. The configured founder.
2. Original recipe, seed 11, tick 1,000: 256 saved genomes.
3. Original recipe, seed 11, tick 7,000: 256 saved genomes.
4. Relaxed recipe, seed 11, tick 4,000: 256 saved genomes.
5. Original recipe, seed 11, tick 15,000: 256 saved genomes.

The saved genomes are convenience samples: the first 256 slotmap entries, not random population samples or independent lineages. Cohort labels identify the population they came from; assays use the relaxed recipe for every cohort. Conclusions concern these genomes, not estimated prevalence across the world.

The initial policies were current weights, half increasing weights, quarter increasing weights, and double decreasing weights. The narrower large-copy policy was added after the first operator accounting identified the growth sources. It quarters only `TopologyCopyNode`, `TopologyCopyMeshBackwardSlice`, and `TopologyCopyMeshForwardSlice`.

Weights are normalized within the selected mutation domain. Thus reducing some weights reallocates events to others; it does not simply skip additions or preserve every other operator's absolute frequency. Requested event supply remains unchanged. The large-copy intervention leaves the within-domain distribution of node-internal operators unchanged, although mutation RNG histories can diverge between policies.

Experiments:

- **Single-event assay:** force exactly one attempted event, then reset to the same parent. 10,000 trials for the founder and 40 per saved parent. Record actual genome-size deltas, applied/skipped events, operator identity, and behavior.
- **Ordinary birth assay:** use production per-unit event supply, with the same trial counts. Single-event plus ordinary-birth assays total **509,600 trials** across five policies and five cohorts.
- **Initial lifetime assay:** 192 offspring per policy/cohort, **4,800 offspring** total. Evolved cohorts use 24 systematically spaced parents with eight offspring each.
- **Lifetime confirmation:** current versus large-copy-quarter only, with fresh per-parent RNG seeds. Use 128 parents at indices 0, 2, …, 254 and 16 offspring each, or 2,048 founder offspring. **20,480 additional offspring**.
- **Ecological assay:** current, all-increasing-quarter, and large-copy-quarter, 15 seeds each, through 5,000 ticks or extinction: **45 runs**.

Behavior uses the earlier 24-scenario battery: ages 20/100/400, energies 20/40/100/190, local grass absent/present, asymmetric neighboring grass and fruit, local fruit present, no neighbors or barriers, and zeroed extended perception. Each scenario starts with fresh runtime state. Loss of reproductive response means a parent emitted reproduction somewhere in this battery and its child did not; it does not mean universal sterility. Similarly, identical battery behavior does not prove universal behavioral neutrality or equal fitness.

Lifetime assays use a fresh 8×8 world, grass reset to full each tick, no terrain/fertility/grazing/occupancy depletion, fresh age zero and energy 20, and no child mutation. Children are removed after birth. Parents run until death or 600 ticks. This measures reproductive capacity under generous feeding, not descendant survival or ecological fitness. Learned runtime state is not replayed.

The ecological world is 64×64 with 16 founders, no terrain or fertility layers, and the original food, grazing, occupancy, age and relaxed genome costs. It is spatially homogeneous in fertility, with stochastic food initialization. Seeds are 11, 29, 47, and 101–112. The last twelve seeds were added for every policy after the first three revealed large variation. This is a different landscape, not a scaled reproduction of the user's 1,600×1,600 world. Its small populations and bottlenecks limit transfer to the original case.

## Result 1: growth is strongly biased before selection

Under current weights, single-event trials give:

| Parent cohort | Units added | Units removed | Mean net units/event | Share of added units from three large-copy operators |
| --- | ---: | ---: | ---: | ---: |
| Founder | 52,900 | 1,120 | +5.178 | 73.9% |
| Original 1,000 | 48,606 | 2,107 | +4.541 | 74.4% |
| Original 7,000 | 33,744 | 3,503 | +2.953 | 72.9% |
| Relaxed 4,000 | 34,658 | 2,218 | +3.168 | 69.9% |
| Original 15,000 | 32,889 | 3,003 | +2.919 | 74.6% |

These are realized edits, not merely counts of available operators. The founder row has 10,000 trials; the other rows have 10,240 each. No offspring survival filter was used.

For the original tick-7,000 cohort, 75 forward-slice copy events added 11,805 units, 92 backward-slice copies added 7,888, and 117 node copies added 4,901. Together, just 284 of 10,240 attempted events generated 24,594 added units. In contrast, 114 node-removal events removed 2,476 units, and 100 VM instruction deletions removed 100.

This supports auditing **units changed and edit-size distributions**, not setting equal insertion/deletion event probabilities. With event supply proportional to genome size, such a positive edit bias also creates a mechanism for additional growth in later generations. It does not prove that every added unit is useless.

## Result 2: broad suppression reduces growth but changes behavior more

Ordinary births from original tick-7,000 parents:

| Mutation policy | Mean net units/birth | Lost reproduction response | Lost eating response |
| --- | ---: | ---: | ---: |
| Current | +5.949 | 2.70% | 2.00% |
| Increasing weights × 1/2 | +3.405 | 2.97% | 2.34% |
| Increasing weights × 1/4 | +1.811 | 3.18% | 2.52% |
| Decreasing weights × 2 | +5.225 | 3.00% | 1.91% |
| Three large-copy weights × 1/4 | +2.241 | 2.67% | 2.07% |

Behavior-loss percentages use only births from parents that had the respective response. They are not percentages of all attempted mutation events.

The narrower candidate across cohorts:

| Cohort | Current net units/birth | Large-copy-quarter | Reduction | Reproduction loss, current → candidate |
| --- | ---: | ---: | ---: | ---: |
| Founder | 2.836 | 1.435 | 49.4% | 2.94% → 3.00% |
| Original 1,000 | 3.004 | 1.303 | 56.6% | 3.07% → 3.14% |
| Original 7,000 | 5.949 | 2.241 | 62.3% | 2.70% → 2.67% |
| Relaxed 4,000 | 4.773 | 1.830 | 61.7% | 3.61% → 3.72% |
| Original 15,000 | 9.739 | 3.682 | 62.2% | 1.10% → 1.13% |

Paired parent-resampling intervals for the reduction in mean units/birth exclude zero in all four evolved cohorts. These are conditional summaries of the saved parents, not population-level confidence intervals; parents share ancestry and selection history. The small response-loss differences are not evidence of exact equivalence. All policies still have positive mean size drift.

![Mutation growth and reproductive responses](../../.bench-artifacts/genome-mutation-balance-2026-09-19/mutation-balance.png)

## Result 3: no substantial immediate reproductive benefit established

The larger, fresh-seed lifetime confirmation gives mean offspring produced per tested child:

| Cohort | Current | Large-copy-quarter | Difference |
| --- | ---: | ---: | ---: |
| Founder | 6.552 | 6.552 | -0.0005 |
| Original 1,000 | 5.913 | 5.927 | +0.0137 |
| Original 7,000 | 6.593 | 6.591 | -0.0024 |
| Relaxed 4,000 | 5.985 | 6.035 | +0.0498 |
| Original 15,000 | 7.583 | 7.598 | +0.0151 |

Each cell represents 2,048 offspring. The initial small screen's tick-7,000 disadvantage (-0.120 births) did not reproduce at comparable magnitude in this larger screen. The relaxed cohort's small gain has a conditional parent-bootstrap interval of approximately +0.020 to +0.083 births; the other evolved-cohort intervals include zero. These intervals omit uncertainty from sampling other worlds and genealogies and should not be treated as a universal fitness result.

The evidence supports reducing unnecessary mutation-generated expansion with little observed immediate reproductive effect. It does not show that the candidate increases long-run adaptation.

## Result 4: ecological outcomes do not establish a rescue

| Policy | Extinct by tick 5,000 | At most 10 survivors | Median final population | Mean final population | Median final genome size among surviving runs |
| --- | ---: | ---: | ---: | ---: | ---: |
| Current | 4/15 | 10/15 | 5 | 65.2 | 157.0 |
| Increasing weights × 1/4 | 5/15 | 11/15 | 2 | 44.5 | 121.6 |
| Large-copy weights × 1/4 | 3/15 | 9/15 | 9 | 238.4 | 158.6 |

All three policies have a median of zero births in the last 1,000 ticks. Thus the continued existence of a few creatures must not be interpreted as a functioning replacement regime.

The large-copy policy's seed-11 run reached 2,460 creatures and derived about 80% of cumulative food energy from fruit. It is the only run across the three policies with fruit exceeding 1% of cumulative food energy. This demonstrates that the intervention does not prevent this particular innovation, not that it makes innovation reliably more likely. Its single large population heavily influences the candidate's mean; medians and extinction counts show the uncertainty.

Broad suppression produced smaller surviving genomes without better persistence. Narrower copy suppression reduced growth in mutation assays but did not reduce median final genome size among surviving worlds. Mutation bias and the genomes retained after selection are different measurements.

![Small-world population outcomes](../../.bench-artifacts/genome-mutation-balance-2026-09-19/microcosms.png)

## Implementation findings relevant to tuning

1. **The current size restriction excludes neutral refinements.** The engine filters on `complexity_effect().is_decreasing()` when restricted. The helper's description of non-increasing operations does not describe the stricter engine behavior. Re-enabling the threshold is not a clean experiment in slowing additions. See [engine](../../crates/v3-core/src/mutation/engine/mod.rs).
2. **The effect tags do not exactly measure genome-size changes.** `VmInstructionMutation` is labeled neutral but internally chooses insertion, replacement, or deletion. `EnableHebbian` and `EnableRewardModulation` are labeled increasing, yet their applications added zero counted units in the single-event assays. `genome_size()` does not count those configuration toggles. Consequently, suppressing all increasing operators also suppresses access to learning mechanisms. See [VM mutation](../../crates/v3-core/src/mutation/vm/operators.rs), [graph classifications](../../crates/v3-core/src/mutation/graph/mod.rs), and [genome accounting](../../crates/v3-core/src/creature/genome/mod.rs).
3. **Deletions differ in disruption.** In the tick-7,000 one-event cohort, node removal removed 2,476 units with zero lost reproductive responses, while graph internal-node removal removed 150 units and lost reproductive response on 30 trials. This is cohort- and battery-specific, not a general safety guarantee. Doubling every decreasing operator mixes very different operations.
4. **Large copies have disproportionate payloads.** The three dominant copy operators have the same nominal structural weight as a single-node deletion, but can add a much larger connected slice. Exposing separate rates or measuring copied units would let tuning address this asymmetry directly.

## Recommended next changes and experiments

Implementation follow-up: `mutation.large_copy_weight_percent` now defaults to `25` and is editable in the runtime configuration panel. It scales only `TopologyCopyNode`, `TopologyCopyMeshBackwardSlice`, and `TopologyCopyMeshForwardSlice`; `100` restores their base relative weights and `0` disables them. The integer range is 0–100. Other operators keep their relative weights, so normalized topology-event frequencies change, while the requested event supply and node-internal operator weights do not. Genome size pressure remains disabled by default. Recipes that omit this field receive the new default when loaded; already-running worlds require a configuration update. `world-recipe-extended-age-drain-reduced-copy-growth.json` combines this setting with the previously relaxed costs and disabled executed targeting. The archived experiments remain records of their original source and resolved configurations, not tests of this subsequent implementation.

**Highest-confidence engineering recommendation:** make observed genome-size deltas available by operator, alongside behavioral and reproductive effects. Preserve the distinction between size edits, value/refinement edits, and learning-configuration changes. This research instrumentation need not imply renaming every existing enum; a future feature should define the intended semantics explicitly.

**Best bounded tuning candidate:** expose and test a separate rate for whole-node/slice duplication, beginning with the quarter-weight arm studied here. Keep small additions, refinement, and learning activation available. One quarter is a tested candidate, not an optimum. Its deployment must not be represented as a proven population repair.

**Do not adopt from these results:** a blanket quarter-addition rule, a blanket double-deletion rule, or the existing size-pressure switch. The first two failed to show a consistent functional advantage; the third was intentionally excluded because it changes refinement and targeting as well as size growth.

**Further mechanistic alternative:** compare size-aware structural mutation supply and deletion payloads, including whether useful module duplication has a corresponding non-disruptive route for removing unused modules. Expected units added/removed are a better diagnostic than operator counts, but forcing zero drift at every size is not itself an established goal. This alternative remains untested here.

The decisive next ecological experiment is the original 1,600×1,600 recipe, current versus large-copy-quarter, with multiple seeds through at least the original collapse horizon. Keep energy costs and age settings fixed for that comparison. Measure sustained births, extinction, retained food-use innovations, and functional behavior as well as genome size. Separately testing the reproductive affordability mismatch identified earlier would avoid attributing an age/energy problem to mutation policy. This report does not implement either change or claim that the smaller-world results substitute for that test.

## Reproducibility and verification

- [Machine-readable results](genome-mutation-balance-research-2026-09-19.results.json) include every assay summary, conditional resampling results, all 45 world endpoints, input/source/result SHA-256 hashes, and the archive path.
- The ignored local archive is `.bench-artifacts/genome-mutation-balance-2026-09-19/`. It includes the complete research core source, engine patch, Cargo lockfile, probes, input genomes, resolved-config records, raw NDJSON, figures, analysis scripts, and the staged experiment plan. The JSON and Markdown are durable repository artifacts; the ignored archive is local and must be copied separately when sharing full reproduction materials.
- Run `sh .bench-artifacts/genome-mutation-balance-2026-09-19/reproduce.sh` from this repository to replay all primary assays and worlds. It builds the isolated research package offline and writes fresh results into a new temporary directory. Metadata input paths will differ from the initial run; the inputs and seeds are archived.
- An independently production-linked binary matched the research control byte-for-byte on 10,000 founder single-event trials and 10,240 ordinary births from tick-15,000 parents. The final research control also matched the founder parity output after the additional policy was introduced.
- Every trial asserted mutation accounting and event-record length; aggregate signed size deltas were checked. Every ecological checkpoint asserted the living/birth/death ledger. Shell runners passed POSIX shell syntax checks. Both figures were rendered and inspected.
- `make check-docs` passed. The archived workspace resolves successfully with offline, locked Cargo metadata. `git diff --check` passed.
- No performance claims are made: the parent task's full-world run may overlap these computations. The investigation used one simulation thread per probe and did not alter or stop that run.

Limitations: one originating large-world seed, convenience genotype samples with shared ancestry, limited behavioral scenarios, fresh rather than inherited learning state, a finite fed-lifetime horizon, 15 small-world seeds per ecological arm, and an adaptively selected large-copy candidate. Neither formal equivalence of fitness nor original-world rescue is established.
