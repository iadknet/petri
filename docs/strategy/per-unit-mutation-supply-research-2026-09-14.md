# Per-unit mutation supply: prior art and counterfactuals

**Date**: 2026-09-14
**Question**: The depth-2,000 drift reading has sat at 0.0035 to 0.0045 changed births per birth for the last four closures, below the 0.005 floor. The user reads that as evolution stopping once genomes reach a size, and proposes replacing the fixed per-birth event rule (probability 0.44, then a geometric tail between a minimum of 1 and a maximum of 10) with a per-unit rate: the expected number of events per birth is `rate × genome_size()`, unbounded, so a 500-unit genome at rate 0.01 draws about five events. Is this established practice, and should it become a feature?
**Method**: the current engine and config read in code; the depth note's earlier per-node arm and the T03.F08 paired run re-read; primary texts on how Avida, Aevol, Markov Brains, NEAT, Tierra, and the biology (Drake, Sung and Lynch, Eigen and Ochoa) set rates against genome length, read at the depth marked per source in Section 4; a prototype of the per-unit rule built in the scratch worktree `.worktrees/per-unit-supply-probe` (Appendix) and read three ways: the neighborhoods of production-walked genomes at depth under seven supply arms, drift walks under five per-unit arms, and a paired selection run on the Orchards goal world (control to tick 8,500, two per-unit arms to tick 6,000).

## Summary

The proposal is the universal design: every digital-evolution platform read here charges mutation per site or per gene, and biology's rate is a per-base rate whose product with genome size is what selection sees (Drake 1991; Sung et al. 2012). Petri's fixed per-birth count is the outlier, and the depth note said so on 2026-09-07. On the current substrate a per-unit rate of 0.005 (the founder's 0.55 events per birth divided by its 111 units) leaves the founder's neighborhood unchanged and, on genomes walked to generation 2,000 by the production policy, raises behavior-changing births from 0.45% to 8.1% with the executed bias off and to 15.7% with it on, at 1.7% and 3.7% dead births. The user's 0.01 doubles the founder's supply and reads 14.6% changed at depth.

Two facts qualify it. First, no per-unit variant is bounded under drift: with the count proportional to size and every growth operator drawing from that count, the mesh reaches 400 nodes by generation 190 to 240, and holding topology events on the per-birth rule only moves the growth inside nodes (12,000 units by generation 600). This is Luiselli et al.'s "indels only" regime, and every platform that uses a per-site rate pairs it with a bound: a hard size window (Markov Brains), a load that selection converts into compaction (Avida, Aevol), or both. Petri's bound is T03.F08's carrying cost, which acts only under selection; the paired selection run below is the reading that decides whether cost plus load holds size where the per-unit supply stays useful. Second, the drift walk that motivates the proposal cannot read a per-unit policy: without selection it explodes, so the feature cannot be gated by the depth-2,000 floor (withdrawn as a gate on 2026-09-14) and its spec has to say how the walk runs as the map-regression instrument it remains.

Under selection on the Orchards world with the carrying cost, neither per-unit arm shows the drift walk's doubling through generation 137 to 149: size stays on the control's order of magnitude at two to five times the control's events per birth, with higher mean energy and a larger population, but no arm, the control included, levels off within the run, and the late means are shaped by a population bloom (Section 5.3).

Recommendation (Section 7): adopt as a bounded T11 delivery-repair feature before T11.F10, at the founder-equivalent rate 0.005 per `genome_size()` unit rather than 0.01, keeping the executed bias and the layer split, adding no cap, and reading its closure through T14.F12 rather than the drift walk. The first draft of this note (Sections 1 to 5.2) was committed at `930ce9be` together with the user's 2026-09-14 roadmap decisions: the drift floors withdrawn as closure gates, T14.F12 added as the depth indicator, and T03.F11 added as a replication cost. Sections 5.3, 6, and 7 were completed after that commit; on reading them the user adopted the feature as T11.F19 on 2026-09-14, placed immediately before T11.F10 in the priority order with T11.F10 depending on it. Nothing in the production code is changed by this note.

## 1. The diagnosis, checked

The floor reading is a drift walk (`drift-depth-v3`): 50 lineages take one production birth per generation with no selection, and at generations 0, 22, 250, 1,000, and 2,000 the first 20 lineages each produce 100 fresh births that the 80-execution battery classifies as silent, changed, or dead. T13.F06's reading at generation 2,000 (`docs/progress/features/t13-f06-recruitment-and-retention-qualification-goal.json`, world 11):

| Reading | Value |
| --- | ---: |
| Mean total nodes / reachable / executed | 143.7 / 14.3 / 4.18 |
| Zero-event births | 1,135 of 2,000 (56.8%) |
| Mutated births silent / changed / dead | 856 / 7 / 2 of 865 |
| Changed per all births | 0.0035 |
| Events landing on executed targets over the walk | 57.3% |

So at depth the supply is not the only limit: 57% of events already land on executed nodes (T11.F17's bias), and 99% of mutated births are still silent. The core evolved robustness, as the depth note's Section 3.4 read on the live population. A per-unit supply does not remove a delivery bottleneck; it buys exposure by count, and the trade it makes is dead births (Section 5.1).

The walk is also no longer the live population's twin. The depth note matched drift to the user's 281,405-tick run (140 against 136 nodes) before T03.F08. With the carrying cost, the T03.F08 paired run reads 386.5 units in 10.1 nodes at mean generation 177 against the control's 1,093.8 units in 18.4 nodes at generation 301 (`docs/progress/sweeps/t03-f08/`), and the drift walk at generation 250 reads 700 units in 18.5 nodes. Under selection the genome is a quarter to a half the size the walk gives it at the same depth, and still growing at 1.55 units per generation; the drift floor is a reading of a substrate the world no longer produces.

## 2. What Petri does today

`requested_event_count` in `crates/v3-core/src/mutation/engine/mod.rs` draws `mutation_probability` (0.44), then starts at `per_birth_mutation_events_min` (1) and adds one event per success of `per_birth_mutation_event_continuation_probability` (0.2) up to `per_birth_mutation_events_max` (10): 0.55 requested events per birth, independent of the genome. Each event picks the Topology layer with `mesh_layer_probability` 0.2, otherwise VM, Graph, or InputRef with equal chance; the target node is drawn from the parent's recently executed set with `executed_bias` 0.9 and uniformly per node otherwise (`reachable_bias` 0.0 everywhere), then the operator picks a position inside the node. `genome_size()` (`creature/genome/mod.rs`) counts one unit per node plus its input refs, targets, VM instructions and constants, or graph compute nodes, edges, and wired sinks; the founder is 111 units. T03.F08 charges `1e-4` energy per unit per tick; T03.F10 charges VM steps on a ramp; `genome_size_pressure_enabled` is off.

The founder-equivalent per-unit rate is therefore `0.55 / 111 = 0.00495`. Its clone fraction, `(1 - 0.00495)^111 = 0.577`, matches the current 0.56 to 0.58 zero-event fraction, which is why Section 5.1's founder rows are indistinguishable between the two rules.

## 3. Prior local evidence

- The [depth note](mesh-depth-research-2026-09-07.md), Section 5, ran a per-node supply arm ("scaled": 0.275 events per node as a geometric burst) on the user's generation-1,990 genomes: 36% behavior-changing births with 5% dead, against 1.0% and 0.2% at baseline. Under drift the same arm passed 400 nodes by generation 155. Its option P2 recorded the design as "the universal design," verdict "characterize in T11.F13 only together with a junk bound."
- T11.F13's roadmap note (`docs/roadmaps/t11-brain-genotype-phenotype-map.md`) already carries the arm: "per-node supply (requested events proportional to node count, drawn independently per node rather than as a burst; natural analog: per-base copy error, the design of Avida, Aevol, Markov Brains, and Cartesian GP) and a junk bound that does not freeze the core (a per-node cost through T03.F08 ...)."
- T11.F17's closing note names it as the fallback: "If that pair does not favor the bias, the production default becomes 0.0, the machinery stays as an experimental arm, and the per-node supply with a junk cost (the universal design in the depth note's Section 6, with T03.F08 as the cost) is the natural repair." The paired long run that would decide the bias has not been run.
- T03.F08 closed with the cost arm strictly below the control on size and nodes at tick 12,000 but with both arms still growing; the spec records the charge at 386 units as about 8% of decay.
- T08.F05 (heritable mutation policy) "begins with an inherited scalar mutation rate, with bounds informed and justified by T11.F13." A single per-unit scalar is that trait; the current rule is a four-field tuple.

## 4. What established systems do

Read depth per row: *full text* means the primary text was read in full; *summary* means the full text was fetched and a section-level extraction with verbatim quotations was read; *abstract* means the abstract only; *config* means the shipped configuration file was read.

| System or study | Rate rule | Bound on size | Bearing on the proposal |
| --- | --- | --- | --- |
| **Avida**, [mutation settings wiki](https://github.com/devosoft/avida/wiki/Mutation-settings) (*summary*) and [`avida.cfg`](https://github.com/devosoft/avida/blob/master/avida-core/support/config/avida.cfg) (*config*) | `COPY_MUT_PROB` 0.0075, "tested each time an organism copies a single instruction"; researchers "typically use 0.0025." Insertions and deletions are per divide: `DIVIDE_INS_PROB` and `DIVIDE_DEL_PROB` 0.05, "at most one divide mutation of each type is possible during a single divide" | `MIN_GENOME_SIZE` and `MAX_GENOME_SIZE` both 0 (off); `OFFSPRING_SIZE_RANGE` 2.0 caps the parent-to-offspring length ratio | Point mutations per site, structural mutations per genome. At 100 instructions the typical rate gives 0.25 to 0.75 events per genome, Petri's 0.55 |
| **Genome size under per-site rates in Avida**, Gupta, LaBar, Miyagi, and Adami 2016 (*summary*, [Sci. Rep.](https://pmc.ncbi.nlm.nih.gov/articles/PMC4867773/)) | Point rates 0.0025 to 0.1 per locus; "insertions and deletions occurred with equal frequency at a constant rate of 0.05 indels per generation"; "genome sizes can change every generation by at most 10%" | Load: "genome size is negatively correlated with the mutation rate (Spearman's ρ = −0.72)"; "genome size evolution is the result of a compromise between acquiring phenotypic complexity and restricting the mutational load"; the evolved genomic rates "ranged from 0.13 to 24.85," so the product is not constant | The bound is selection against load, not a cap; growth at low rates is driven by beneficial insertions (87% of innovations preceded by an insertion) |
| **LaBar and Adami 2016** (*summary*, [PLOS Comput. Biol.](https://journals.plos.org/ploscompbiol/article?id=10.1371/journal.pcbi.1005066)) | "0.01 mutations per instruction copied, and insertions and deletions at 0.005 events per division," giving an ancestral genomic rate of 0.15 that "varied as genome length changed" | Genomes grew from 15 to a median of 35 to 36 instructions | The genomic rate rises with size by design and is tolerated because size is held by load |
| **Critical rates in Avida**, Comas, Moya, and González-Candelas 2005 (*summary*, [BMC Evol. Biol.](https://www.ebi.ac.uk/europepmc/webservices/rest/PMC546199/fullTextXML)) | `COPY_MUT_PROB` "is the mutation rate that results from dividing the genomic mutation rate by the genome size"; fixed lengths 54 to 272 | Fixed length | The robust genotype beat the fit one at 0.5 to 3.0 mutations per genome per replication, "in the vast majority of cases ... larger than 2." Petri's founder-equivalent rate gives 28 events per birth at 5,800 units (Section 5.1): far above this band, which is what the dead fraction measures |
| **Aevol**, Knibbe et al. 2007 (*summary*, from the depth note) and the 2024 streamlining study (*summary*, [GBE](https://academic.oup.com/gbe/article/16/12/evae250/7905804)) | "10⁻⁶ mutations per base pair for each mutation type: substitutions, small insertions, small deletions, duplications, deletions, translocations, and inversions" | Load again: "increasing the mutation rate drastically reduces the total genome size"; "under constant N×μ ... the coding fraction remains constant" | All per base, including rearrangements whose size scales with the genome, so the hazard of carrying junk grows with the junk; no cap |
| **Markov Brains**, Hintze et al. 2017 (*full text*, [arXiv](https://arxiv.org/pdf/1709.05601)) | "Each site has a chance to mutate at every replication event"; "a 20% chance for a section of the genome to be deleted randomly," 256 to 512 sites, and the same for duplication | "Deletions can only occur if the genome is larger than 1000 bytes"; duplication "is not allowed if the genome is already above 20,000 sites" | Per-site points, per-replication structure, hard window |
| **NEAT**, Stanley and Miikkulainen 2002 (*full text*, [EC](https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf)) | Per genome: "an 80% chance of a genome having its connection weights mutated, in which case each weight had a 90% chance of being uniformly perturbed"; add-node 0.03 and add-link 0.05 per genome | None: "allowing genomes to grow unbounded"; speciation protects innovation | The one per-genome design read here, and its weight step is per weight once the genome is chosen, so exposure per weight does not dilute |
| **Tierra**, Ray, [documentation](https://tomray.me/pubs/doc/index.html) (*summary*) | `GenPerBkgMut = 16`, `GenPerMovMut = 8`, "mutation rate control by generations"; the run reports "one mutation for every 1216/2 instructions copied" | Not read | The parameter names say the copy-error rate is set per genome per generation; the conversion to the per-instruction rate was not found in the pages read, so this row is a reading of names, not of a formula |
| **Sims 1994** (*full text*, from the mesh note) | Mutation frequencies "scaled by an amount inversely proportional to the size of the current graph" | Unconnected nodes removed | Per-genome constant supply plus garbage collection: Petri's current supply rule without the pruning |
| **Drake 1991** (*summary*, [PNAS](https://pmc.ncbi.nlm.nih.gov/articles/PMC52253/)) | Per-genome rates in DNA microbes vary "by only approximately 2.5-fold, apparently randomly, around a mean value" of 0.0033 per replication; "the average mutation rate per base pair is inversely proportional to genome size" | | The rate is per base; evolution tunes it so the per-genome product stays near 0.003 across a 6,500-fold size range. A fixed per-genome count is the *outcome* of per-base rates under selection, not the mechanism |
| **Drift-barrier hypothesis**, Sung, Ackerman, Miller, Doak, and Lynch 2012 (*summary*, [PNAS](https://pmc.ncbi.nlm.nih.gov/articles/PMC3494944/)) | "The magnitude of selection operating to reduce the mutation rate is not simply a function of the per-site mutation rate but of the genome-wide deleterious mutation rate"; the effective genome-wide rate "scales inversely with Ne" | | Selection sees rate × coding target size. Under a per-unit rule Petri's junk is charged at the same rate as the core, which is the load Gupta and Aevol turn into compaction |
| **Error thresholds**, Eigen via Ochoa 2006 (*abstract*, [Evol. Comput.](https://pubmed.ncbi.nlm.nih.gov/16831105/)); Bäck 1993 (*abstract*) | The error threshold is "a critical mutation rate (error rate) beyond which structures obtained by an evolutionary process are destroyed more frequently than selection can reproduce them"; thresholds "depend mainly on selection pressure and genotype length"; a per-locus rate of 1/L "is normally recommendable" | | The rate to set is per locus, of order 1/L; the per-genome product near 1 is the working band, and Petri's 28 events at depth is above it |

The pattern is the same as the depth note found: per site for point changes, and a bound on size that is either a hard window or the mutational load itself. The proposal is per site. Its missing half is the bound.

## 5. Counterfactuals

Prototype in the scratch worktree (Appendix): three probe-only config fields. `per_unit_rate` draws each `genome_size()` unit independently at that rate and requests one event per success (a Binomial count, the per-site model; `mutation_probability`, the minimum, the maximum, and the continuation probability are bypassed); `structural_per_birth` keeps Topology events on the per-birth rule and draws only node-internal events per unit (the Avida split); `unit_weighted_targets` draws the target node in proportion to its unit count instead of uniformly (the faithful per-site draw, since Petri picks a node and then a position). Arms are named `pu<rate>_b<executed_bias>[_h][_w]`.

### 5.1 Neighborhoods of production-walked genomes (`probe_a`)

Fifty lineages walked under the production policy exactly as `drift-depth-v3` walks them (seeds 90,000 + lineage, executed set refreshed every 10 generations from the battery), so the genomes at each checkpoint are the ones the closure instrument reads. At each checkpoint the first 20 lineages each produce 100 births under each arm; classification is `neighborhood::births::per_birth_result`, the production classifier. Changed and dead are per all births, including clones.

| Depth (nodes / units / executed) | Arm | Requested per birth | Zero-event | Changed | Dead |
| --- | --- | ---: | ---: | ---: | ---: |
| 0 (2 / 111 / 2) | baseline | 0.54 | 56.4% | 21.15% | 0.05% |
| | pu0.005_b0.9 | 0.56 | 57.9% | 19.90% | 0.05% |
| | pu0.01_b0.0 | 1.14 | 32.5% | 36.20% | 0.20% |
| | pu0.0025_b0.0 | 0.28 | 75.3% | 10.85% | 0.00% |
| 250 (18.5 / 700 / 2.84) | baseline | 0.57 | 55.2% | 1.15% | 0.05% |
| | pu0.005_b0.9 | 3.82 | 4.6% | 7.05% | 0.95% |
| | pu0.005_b0.0 | 3.82 | 4.6% | 2.45% | 0.50% |
| | pu0.005_b0.0_w | 3.82 | 4.6% | 3.85% | 0.55% |
| | pu0.005_b0.0_h | 3.95 | 4.1% | 1.90% | 0.15% |
| | pu0.01_b0.0 | 7.68 | 0.8% | 6.00% | 0.70% |
| | pu0.0025_b0.0 | 1.86 | 19.8% | 1.15% | 0.20% |
| 1,000 (73.7 / 2,898 / 4.22) | baseline | 0.54 | 56.6% | 0.85% | 0.05% |
| | pu0.005_b0.9 | 13.83 | 0.05% | 12.60% | 2.05% |
| | pu0.005_b0.0 | 13.83 | 0.05% | 5.60% | 1.35% |
| | pu0.005_b0.0_w | 13.83 | 0.05% | 7.40% | 0.70% |
| | pu0.005_b0.0_h | 13.94 | 0.05% | 1.10% | 0.20% |
| | pu0.01_b0.0 | 27.75 | 0% | 9.10% | 2.35% |
| | pu0.0025_b0.0 | 6.88 | 1.2% | 3.00% | 0.45% |
| 2,000 (150.2 / 5,791 / 4.60) | baseline | 0.54 | 56.8% | 0.45% | 0.10% |
| | pu0.005_b0.9 | 27.99 | 0% | **15.65%** | **3.70%** |
| | pu0.005_b0.0 | 27.99 | 0% | 8.10% | 1.65% |
| | pu0.005_b0.0_w | 27.99 | 0% | 7.00% | 1.65% |
| | pu0.005_b0.0_h | 28.10 | 0% | 0.50% | 0.35% |
| | pu0.01_b0.0 | 56.25 | 0% | 14.60% | 3.30% |
| | pu0.0025_b0.0 | 13.97 | 0.05% | 4.95% | 0.85% |

Readings:

- **The founder-equivalent rate is a no-op on the founder** (rows at depth 0: 0.56 against 0.54 requested, 19.9% against 21.2% changed, one dead birth in each). The user's 0.01 doubles the founder's supply and its changed fraction; 0.0025 halves them.
- **At depth the per-unit supply restores exposure and pays in dead births.** At generation 2,000 the founder-equivalent rate requests 28 events per birth and every birth is mutated. With the executed bias on, 15.7% of births change behavior (35 times baseline) and 3.7% are dead (37 times baseline); with it off, 8.1% and 1.65%. The bias does not turn lethal at 28 events, which the depth note's geometric-burst arm suggested it might: at 0.9 bias about 25 events land on a 4.6-node core and four in five births are still silent. The core's robustness is the ceiling here, not the supply.
- **Topology events carry the exposure.** The hybrid arm (`_h`), which holds the mesh layer at its per-birth 0.11 events and draws only node-internal events per unit, reads 0.5% changed at depth against 8.1% for the pure arm at the same count. The difference between the two is about 5.5 Topology events per birth. Node-internal events drawn uniformly per node hit the 4.6 executed nodes of 150 about 3% of the time and are mostly silent even at 22 per birth; the Avida split, which is the safe design for growth (Section 5.2), throws away the part of the supply that changes behavior on this substrate.
- **The unit-weighted draw does not help** (7.0% against 8.1% at depth). Weighting by unit count sends more events into large nodes, and at depth the large nodes are junk-heavy VM programs; the executed core is small. The faithful per-site targeting is not the right targeting for this genome.

### 5.2 Drift walks under per-unit policies (`probe_b`)

The same walk with the arm's policy shaping the walk itself, aborted when the mean exceeds 400 nodes or 12,000 units.

| Arm | Nodes / units at generation 100 | Generation 250 | Abort |
| --- | ---: | ---: | --- |
| pu0.005_b0.0 | 19.7 / 704 | 170.9 / 5,750 at 200 | 442 nodes / 14,623 units at 240 |
| pu0.005_b0.9 | 23.1 / 995 | 109.2 / 4,619 at 150 | 373 nodes / 16,320 units at 190 |
| pu0.0025_b0.9 | | 39.1 / 1,583 | 267 nodes / 13,656 units at 370 |
| pu0.005_b0.0_h (topology per birth) | 7.3 / 382 | 15.5 / 1,115 | 37 nodes / 12,152 units at 600 |
| pu0.005_b0.9_h | 7.0 / 415 | 17.8 / 1,932 | 33 nodes / 12,331 units at 460 |
| production (Section 1) | | 18.5 / 700 | 150 nodes / 5,791 units at 2,000, no abort |

Every per-unit arm is unbounded. The pure arms double the mesh every 20 to 30 generations once past a few dozen nodes, as the depth note's burst arm did. Holding Topology on the per-birth rule holds the node count to the production slope (37 nodes at generation 600 against the production walk's 18 at 250 and 74 at 1,000) but the units still grow exponentially, through `VmInsert*`, `CopyGene*`, `AddInternalGraphNode`, and the other node-internal growth operators, which draw from the per-unit count. A bound has to cover growth-class operators in every domain, or it has to be selection.

The fresh-birth rows of these walks (not tabulated) say the same thing as 5.1 from the other side: at generation 250 the hybrid walk reads 1.0% changed and the pure biased walk 5.75%.

### 5.3 Paired selection run on Orchards in grassland

Three `v3-cli run` arms on the checked-in Orchards recipe (`experiments/worlds/orchards-in-grassland.json`, seed 11, 1600², 10,000 founders, two food types, `genome_carry_cost_per_unit` `1e-4`), sampled every 500 ticks: the control at production mutation defaults, and two per-unit arms at rate 0.005 with `executed_bias` 0.9 (production targeting) and 0.0. The control ran to tick 8,500 and the per-unit arms to 6,000 (34 and 36 minutes each; the control's wall time overlapped a probe run and is not reported). The pair was first attempted on the production default plains world, which collapsed under the control (Section 8), so the plains data are not used. Artifacts, configs, and the runner are stored under `docs/progress/sweeps/per-unit-supply-2026-09-14/`. "Events per birth" is the interval increment of `mutation_events_applied_total` over `reproduction_actions_spawned_total`.

| Tick | Arm | Population | Mean energy | Mean units | Mean nodes | Mean generation | Events per birth |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 500 | control | 15,784 | 21.4 | 139.9 | 2.73 | 11.4 | 0.55 |
| 500 | per-unit, bias 0.9 | 16,457 | 21.7 | 138.1 | 2.78 | 11.1 | 0.60 |
| 500 | per-unit, bias 0.0 | 16,019 | 21.6 | 144.9 | 3.00 | 11.4 | 0.60 |
| 2,000 | control | 11,356 | 21.4 | 213.3 | 5.06 | 46.8 | 0.55 |
| 2,000 | per-unit, bias 0.9 | 10,260 | 23.2 | 217.0 | 5.07 | 44.2 | 1.01 |
| 2,000 | per-unit, bias 0.0 | 7,857 | 22.5 | 258.3 | 6.50 | 42.8 | 1.14 |
| 4,000 | control | 18,161 | 19.3 | 317.8 | 7.12 | 90.8 | 0.55 |
| 4,000 | per-unit, bias 0.9 | 20,746 | 25.6 | 442.2 | 10.80 | 86.8 | 2.04 |
| 4,000 | per-unit, bias 0.0 | 22,040 | 26.9 | 329.9 | 10.15 | 83.1 | 1.73 |
| 5,000 | control | 21,309 | 20.8 | 403.8 | 9.22 | 116.3 | 0.55 |
| 5,000 | per-unit, bias 0.9 | 24,883 | 25.7 | 547.9 | 15.06 | 108.7 | 2.59 |
| 5,000 | per-unit, bias 0.0 | 28,374 | 21.8 | 344.9 | 12.07 | 113.9 | 2.22 |
| 6,000 | control | 21,861 | 20.5 | 468.6 | 10.88 | 141.5 | 0.55 |
| 6,000 | per-unit, bias 0.9 | 84,657 | 25.3 | 443.6 | 14.56 | 137.1 | 2.02 |
| 6,000 | per-unit, bias 0.0 | 59,479 | 22.4 | 482.0 | 17.13 | 148.7 | 2.61 |
| 7,000 | control | 36,648 | 22.6 | 532.9 | 13.76 | 171.7 | 0.55 |
| 8,500 | control | 79,035 | 22.9 | 574.2 | 16.05 | 212.1 | 0.55 |

Readings:

- **Not explosive under selection, and not shown to level off.** Neither per-unit arm shows the drift walk's doubling: through tick 5,000 (generation 109 to 116) the biased arm runs up to 36% above the control on units (548 against 404) and the unbiased arm runs below it from tick 3,500 to 5,500 (330 to 370 against 290 to 420) with more nodes, so growth stays on the control's order of magnitude rather than doubling every 20 to 30 generations. The tick-6,000 means (444 and 482 against 469) are not a match to read anything from: between ticks 5,000 and 6,000 both arms bloom from about 25,000 to 60,000 to 85,000 creatures, the biased arm's mean falls and the unbiased arm's rises, and a mean during a bloom is the blooming clade's composition, not the cost acting on size. No arm levels off within the run, the control included. Events per birth rise with size from 0.6 to 2.0 to 2.6, four to five times the control's 0.55, and mean energy is higher in both arms at every sample after tick 2,000.
- **The population is not harmed, and blooms earlier.** All three arms bloom from about 20,000 to 55,000 or more creatures; the control at tick 7,000 to 7,500, both per-unit arms by tick 5,500. Whether the bloom is the same event (a lineage taking the second food type) was not read (Section 8), so this is not claimed as an adaptation reading, only as the absence of harm through generation 137 to 149.
- **Selection is what the drift walk lacks**, as Gupta et al. and the Aevol studies predict: the same supply rule that reaches 400 nodes by generation 200 without selection reaches 15 to 17 nodes by generation 140 to 150 with it and the carrying cost. Whether size then equilibrates under either rule, or keeps rising with the control's own slope (2.5 units per generation), is beyond this run; a size reading under selection belongs in the feature's closure conditions.

## 6. Options

| # | Option | Kind | Evidence and fit | Cost and risk | Verdict |
| --- | --- | --- | --- | --- | --- |
| O1 | **Per-unit supply as proposed**: `events ~ Binomial(genome_size, rate)`, no minimum or maximum, bound left to T03.F08's cost and the load itself | Extend the engine (Appendix: about 60 lines plus config) | The universal design (Section 4); founder-neutral at 0.005; 8 to 16% changed births at depth (5.1); Gupta and Aevol show load compacting genomes under selection | Unbounded under drift (5.2); dead births 1.7 to 3.7% at the sizes the walk produces; not explosive under selection with the cost through generation 149, though no arm levels off within the run (5.3); cannot be gated by the drift walk | **Adopt** (Section 7) |
| O2 | **Avida split**: node-internal per unit, Topology per birth | Extend | Bounds the node count to the production slope | Units still explode (5.2); 0.5% changed at depth (5.1), no better than today | Reject |
| O3 | Per-unit supply plus a hard size window (Markov Brains) | Extend, reuse `genome_size_pressure` | Bounded by construction | The existing pressure flag restricts every domain to shrinking operators above the cap and froze the core at 0.1% changed births (depth note 5.2); a window that blocks only growth-class operators above the cap is a new rule and contradicts the 2026-09-07 decision that the cost, not a cap, is the junk bound | Reject unless 5.3 shows the cost does not hold |
| O4 | Keep the fixed count, raise the Topology share or the count | Tuning | Topology events carry the exposure (5.1) | Dilution unchanged in proportion; the depth note's P6 rejected raising the count for the same reason | Reject |
| O5 | Per-unit rate on mesh nodes rather than `genome_size()` units | Extend | Charges structure, not instructions; the depth note's burst arm | Rewards packing junk into few large nodes, the same blind spot T03.F08's spec rejected for its unit | Reject |
| O6 | Leave the arm in T11.F13 as characterization | Roadmap as is | T11.F13's spec already lists it with the cost as the bound | T11.F13 waits on T11.F10, which waits on the substrate this repair would change; the drift floor keeps failing meanwhile; 5.3 already gives the bound reading T11.F13 was waiting for | Reject; T11.F13 keeps rate characterization on the new rule |

## 7. Recommendation

**Adopt, as a bounded T11 delivery-repair feature placed after T13.F06 and before T11.F10, not as a T11.F13 arm; depends on T14.F12 for its closure reading, with T03.F11 sequenced directly after it (user decision, 2026-09-14).** Working title "Per-Unit Mutation Supply"; natural analog: per-base copy error, the fidelity of a replicating polymerase being a property of each site copied, so that a genome's mutation load is its size times its rate (Drake 1991; Sung et al. 2012). The reasons, in order of weight:

1. It is the design every platform in Section 4 uses, and the depth note, T11.F13's note, and T11.F17's closing note all name it as the natural repair. The evidence that was missing on 2026-09-07 was a bound; T03.F08 has since shipped the cost, and Section 5.3 reads the two together under selection: no runaway through generation 137 to 149, size on the control's order of magnitude, and the population no worse. Section 5.3 does not show size leveling off under either rule.
2. At the founder-equivalent rate it is a no-op on the founder (Section 5.1, depth 0), so the change is invisible to every founder floor and to the early goal readings, and it restores exposure exactly where the drift readings lose it: 8 to 16% changed births at generation 2,000 against 0.45%.
3. It replaces four coupled fields (`mutation_probability`, the minimum, the maximum, the continuation probability) with one scalar, which is the trait T08.F05 says it will inherit.

What the spec has to decide, with the readings that inform each:

| Decision | Recommendation | Basis |
| --- | --- | --- |
| Count rule | `events ~ Binomial(genome_size(), rate)`, one independent draw per unit; no minimum, no maximum; the four per-birth fields removed or left as a disabled legacy rule for T11.F13's fixed-count control | Section 2's identity `0.55 / 111`; Section 5.1's founder rows |
| Unit | `genome_size()` units, as T03.F08 chose for the cost | O5 in Section 6; the cost and the supply then charge the same quantity |
| Default rate | 0.005 (founder-equivalent, `0.55 / 111 = 0.00495`), not 0.01 | 0.01 doubles the founder's supply (36% changed, 0.2% dead at the founder) and requests 56 events per birth at the walk's depth-2,000 size; 0.005 sits in Avida's experimental band (0.0025 to 0.0075 per site at about 100 sites) and its clone fraction matches today's |
| Layer split | Unchanged: every event picks Topology with `mesh_layer_probability` 0.2 | The Avida split (O2) reads 0.5% changed at depth; Topology events carry the exposure on this substrate (Section 5.1) |
| Targeting | Keep `executed_bias` 0.9 and the uniform-per-node residual; do not add the unit-weighted draw | Section 5.1 on the walked genomes: 15.7% against 8.1% changed at depth with dead 3.7% against 1.65%; the bias is not lethal at 28 events; the weighted draw reads 7.0%. The selection pair does not separate the two biases beyond bloom timing and mid-run size |
| Bound | None added by this feature: T03.F08's cost plus the load itself, with a size reading under selection predeclared as a closure condition; T03.F11 follows it directly (user decision, 2026-09-14) | Section 5.3 shows no runaway but no plateau either; a cap contradicts the 2026-09-07 decision and the existing pressure flag freezes the core |
| Closure instrument | Read changed births at depth through T14.F12's neighborhood read of the goal population's surviving genomes (the user withdrew the drift floors as closure gates and added T14.F12 on 2026-09-14 after this note's first draft), so the feature depends on T14.F12. The drift walk stays a mutation-map regression instrument under the no-regression rule, and this feature is an event-count change, so its spec must predeclare how the walk runs: run it with the fixed-count rule as a walk-only setting (0.55 events per birth, the founder-equivalent), since under the per-unit rule the walk cannot reach generation 2,000 at all (Section 5.2) and its structural rows would be meaningless rather than regressed | Section 5.2; the T11 track's floor note as amended 2026-09-14; T14.F12's scope |
| Predeclared directions | Founder rows unchanged in outcome; the evolved half's changed fraction up and dead not up; the goal worlds' final population and clade count not down; mean `genome_size()` at the end of the goal run within a stated multiple of the previous closure's; `mutation_events_applied` per birth up and reported | Sections 5.1 and 5.3 |

What this feature does not do: it does not make the core less robust (four in five births at depth are still silent under the biased per-unit arm), it does not bound junk by itself, and it does not settle the rate. T11.F13 keeps rate characterization on the new rule, with the fixed-count rule as its control.

## 8. Remaining uncertainty

- **Selection depth.** The selection arms reach mean generation 137 at tick 6,000 (the control 212 at 8,500). The drift readings are at 1,000 and 2,000; whether size equilibrates under the cost or only slows is read from a slope, not an equilibrium, and the control's own size is still rising at 2.5 units per generation. A deeper paired run is the feature's own long reading.
- **The instrument.** After this note's first draft the user withdrew the drift floors as closure gates, added T14.F12 (fresh births from the goal population's surviving genomes, classified by the same battery) as the closure indicator for changed births at depth, and added T03.F11 (a replication cost proportional to `genome_size()` above the founder's 111) as a third brake on structure. T14.F12 reads the live substrate at the goal profile's depth (about generation 22 to 45), shallower than the walk's 2,000 and shallower than Section 5.3's 137 to 149; a longer goal arm on one world (6,000 ticks took 34 to 36 minutes here) is the reading that would close that gap and is not budgeted. On T03.F11: Section 5.3 does not show whether the carrying cost alone levels size under either supply rule, so T03.F11's rationale (both T03.F08 arms still growing) stands unchanged at the Orchards world; the user sequenced it directly after T11.F19 on 2026-09-14, so its paired run reads the brake on the per-unit rule. Its spec should size the rate against the Orchards control's trajectory rather than against the drift walk's explosion, which no cost can see.
- **The bloom.** Both selection arms and the control bloom to 50,000 to 85,000 creatures late in the run; the biased per-unit arm blooms about 2,000 ticks earlier. `tick_sample` carries no clade or typed-eat counters, so whether that is the same lineage discovering the second food type, and whether the earlier bloom is the supply's doing, was not read.
- **The default plains world collapsed** on seed 11 at this revision (population under 200 by tick 3,500, against over 7,000 at T03.F08's closure), which is why the selection pair moved to the Orchards recipe. That is a separate question, flagged as its own task, and it means the T03.F08 paired run is not directly comparable to Section 5.3.
- **Classification limits.** The battery zeroes shared memory and reads 80 executions; silence is an upper bound on reactive silence (the audit's own limit). Dead is NoOp on every execution.
- **Targeting.** Petri draws a node and then a position; the faithful per-site draw (`_w`) was measured only at bias 0.0. The right targeting for a per-unit count on this genome is a spec question, and 5.1 says executed bias 0.9 is not lethal at 28 events.
- **Tierra's rule** was read from parameter names, not a formula.
- **Rate value.** 0.005 is the founder-equivalent anchor and sits in Avida's experimental range (0.0025 to 0.0075 per site at 100 sites). It is not an optimum; T11.F13 owns that reading, and T08.F05 would inherit the scalar.

## 9. Sources

- [Avida wiki, Mutation settings](https://github.com/devosoft/avida/wiki/Mutation-settings); [avida.cfg](https://github.com/devosoft/avida/blob/master/avida-core/support/config/avida.cfg)
- [Gupta, LaBar, Miyagi, Adami, Evolution of Genome Size in Asexual Digital Organisms, Scientific Reports 2016](https://pmc.ncbi.nlm.nih.gov/articles/PMC4867773/)
- [LaBar and Adami, Different Evolutionary Paths to Complexity for Small and Large Populations of Digital Organisms, PLOS Computational Biology 2016](https://journals.plos.org/ploscompbiol/article?id=10.1371/journal.pcbi.1005066)
- [Comas, Moya, González-Candelas, Validating viral quasispecies with digital organisms, BMC Evolutionary Biology 2005](https://pmc.ncbi.nlm.nih.gov/articles/PMC546199/)
- [Genome Streamlining: Effect of Mutation Rate and Population Size on Genome Size Reduction, Genome Biology and Evolution 2024](https://academic.oup.com/gbe/article/16/12/evae250/7905804); Knibbe et al. 2007 as read for the depth note
- [Hintze et al., Markov Brains: A Technical Introduction, arXiv 2017](https://arxiv.org/abs/1709.05601)
- [Stanley and Miikkulainen, Evolving Neural Networks through Augmenting Topologies, Evolutionary Computation 2002](https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf)
- [Ray, Documentation for the Tierra Simulator](https://tomray.me/pubs/doc/index.html)
- [Drake, A constant rate of spontaneous mutation in DNA-based microbes, PNAS 1991](https://pmc.ncbi.nlm.nih.gov/articles/PMC52253/)
- [Sung, Ackerman, Miller, Doak, Lynch, Drift-barrier hypothesis and mutation-rate evolution, PNAS 2012](https://pmc.ncbi.nlm.nih.gov/articles/PMC3494944/); Lynch et al. 2016 (Nature Reviews Genetics) is not in PMC and was not read
- [Ochoa, Error Thresholds in Genetic Algorithms, Evolutionary Computation 2006](https://pubmed.ncbi.nlm.nih.gov/16831105/) (abstract); [Bäck, Optimal Mutation Rates in Genetic Search, ICGA 1993](https://dl.acm.org/doi/10.5555/645513.657408) (abstract)
- Local: [depth note](mesh-depth-research-2026-09-07.md); [T03.F08 spec](../specs/roadmap/t03-f08-genome-size-maintenance-cost.md) and [sweep](../progress/sweeps/t03-f08/); [T11 track](../roadmaps/t11-brain-genotype-phenotype-map.md) (T11.F13 and T11.F17 notes); [T13.F06 goal report](../progress/features/t13-f06-recruitment-and-retention-qualification-goal.json)

## Appendix: the prototype

Scratch worktree `.worktrees/per-unit-supply-probe`, detached at `1c09c26d` (T13.F06's closure), uncommitted. Three files changed (`config/simulation.rs`, `mutation/engine/mod.rs`, `mutation/reachability.rs`, 130 lines) plus the probe `crates/v3-core/tests/probe_per_unit_supply.rs`; the patch, the probe source, the drift-walk output, the selection NDJSON streams, their configs, and the runner are stored under `docs/progress/sweeps/per-unit-supply-2026-09-14/` so the readings survive the worktree. Run with `cargo test --release -p v3-core --test probe_per_unit_supply -- --nocapture --test-threads=1`; `PETRI_ARMS` and `PETRI_WALK_ARMS` select arms. The selection pair used the worktree's `v3-cli run --ticks 12000 --sample-every 500 --seed 11 --config <arm>.json` where each arm config is the applied default recipe with `mutation.per_unit_rate` and `mutation.executed_bias` overridden.

The engine hook, in full:

```rust
// apply_mutations_with_food_type_count, before the event loop
let plan: Option<Vec<bool>> = if config.per_unit_rate > 0.0 {
    let units = genome.genome_size();
    let mut plan = Vec::new();
    if config.structural_per_birth {
        let per_birth = requested_event_count(config, rng);
        for _ in 0..per_birth {
            if rng.gen_bool(config.mesh_layer_probability) { plan.push(true); }
        }
        for _ in 0..units {
            if rng.gen_bool(config.per_unit_rate.min(1.0)) { plan.push(false); }
        }
    } else {
        for _ in 0..units {
            if rng.gen_bool(config.per_unit_rate.min(1.0)) {
                plan.push(rng.gen_bool(config.mesh_layer_probability));
            }
        }
    }
    Some(plan)
} else {
    None
};
let event_count = match &plan {
    Some(plan) => plan.len() as u32,
    None => requested_event_count(config, rng),
};
// inside the loop, replacing the per-event layer roll
let topology_now = match &plan {
    Some(plan) => plan[event_index as usize],
    None => rng.gen_bool(config.mesh_layer_probability),
};
```

`unit_weighted_targets` adds an optional per-node weight slice to `TargetSets` and a weighted draw in `TargetSelector::select` when the executed layer does not fire. With every probe field at its default the production RNG stream is untouched: the `baseline` rows in Section 5.1 reproduce `drift-depth-v3`'s zero-event counts at every checkpoint (1,128 / 1,135 / 1,103 / 1,132 / 1,135 of 2,000, as in T13.F06's world-11 report). Their changed and dead counts differ from that report (9 against 7 changed at generation 2,000) because the probe classifies with the default recipe's battery and the report with world 11's food-type count; the walk itself is the same.
