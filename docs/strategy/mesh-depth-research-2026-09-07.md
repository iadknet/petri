# Depth Research Note: Why the Overnight Run Stalled

**Status**: Research (companion to the [T11 track](../roadmaps/t11-brain-genotype-phenotype-map.md), the [brain evolvability audit](brain-evolvability-audit-2026-09-04.md), and the [mesh evolvability research note](mesh-evolvability-research-2026-09-06.md); not executable roadmap guidance)
**Date**: 2026-09-07
**Code**: `main` at 32468c20 (T11.F08 closed); the inspected server binary was built at 22:28 on 2026-09-06, after that commit, and its counters show the T11.F15 operator set (no `RewriteNodeId` row)
**Subject**: the user's overnight `serve` run, paused at tick 281,405 with 4,742 creatures, production defaults, 1600 by 1600, 10,000 V3Alpha1 founders
**Method**: server counters and config read over HTTP; a 400-genome random sample of the live population pulled from `GET /v3/simulation/creature/:id`; a functional census of those genomes on the T11.F01 battery (executed nodes, route variation, knockouts, behavioral signatures, per-birth neighborhood); drift walks to generation 2,000 under six mutation policies; primary literature (read depth marked per source in Section 6)
**Prompted by**: the user's observation that after 281,405 ticks the creatures "were still very simple," the mesh "had grown to a huge size," and "there were still only about 2 simple active nodes"

## Question

Is there a basic problem with the mesh structure, or some other more basic problem, that keeps creatures simple over a long run?

## Verdict

The mesh structure is not the block. The more basic problem is that the mutation supply policy starves the functional core as junk accumulates, and the closure instruments cannot see it because they read at 22 generations while the overnight run reached 2,000.

At tick 281,405 the median creature is 1,990 generations deep. Its mesh carries 124 nodes, of which 17 are reachable and 4 execute on the battery; three of those four contribute to behavior. That core is not the founder: half the population routes conditionally on its input (the T11.F15 branch, retained under selection where drift loses it), 70% of executed cores read shared memory, and the executed decision program grew from 105 to 155 instructions and the executed graph from 6 to 14 compute nodes. But evolution has almost stopped: 97.3% of mutated births are behaviorally silent, so only 1.0% of all births change behavior, against 22.5% for the founder. Junk grows by drift at 0.06 nodes per generation with no cost and no bound, the per-birth event count is fixed at 0.55, and targets are drawn uniformly per node, so the executed core receives about 3% of node-internal events. Every established digital-evolution platform avoids this with per-site or per-gene mutation rates and a bound on genome size; Petri has neither. A static world compounds it: the literature is consistent that constant environments select for robustness and immediate optimization at the expense of evolvability, and the live population reads as three dominant phenotypes with 97% silent neighborhoods.

| Reading | Founder (T11.F15 gate report) | Goal profile (gen 22, T11.F15 goal report) | Live population (gen ~1,990, this note) |
| --- | ---: | ---: | ---: |
| Total mesh nodes, mean | 2 | 3.14 | 136 |
| Executed mesh nodes, mean | 2 | 2.31 | 3.40 |
| Executed share of nodes (where node-internal events land) | 1.00 | 0.73 | 0.025 |
| Births that change behavior, per all births | 23.2% | 16.8% | 1.0% |
| Mutated births silent | 44% | 63% | 97.3% |
| Creatures routing conditionally | 0 | 3 of 36 | 200 of 400 |

The founder column is the stored gate report (500 births). The drift probe's own founder row in Section 3.5 (5,000 births, different seeds) reads 22.5% changing and 50% silent; the two agree within their sampling.

Counterfactuals on the live genomes (Section 5) put the cause where the code puts it: aiming the same 0.55 events per birth at the executed set raises behavior-changing births from 1.0% to 9.1%; a per-node supply raises them to 36% but, under drift without a junk bound, grows the mesh exponentially; the existing size-pressure flag bounds junk but freezes the core. A drift walk of fifty lineages, which takes seconds, predicts the whole decline from generation 22 onward.

## 1. Problem, constraints, decision criteria

**Problem.** Mesh evolution at depth. The goal profile closes features at 2,000 ticks (22 generations at the median); the user runs for 281,405 ticks (1,990 generations). Whatever the T11 repairs did at 22 generations, the question is what the substrate does at 2,000.

**Constraints carried from the program** ([roadmap notes](../roadmap.md#notes-for-ai-agents)): ecological selection only; every mechanism names a natural analog and reaches creatures through world or body; seeded runs stay byte-for-byte reproducible; the T11.F04 neutral scaffold stays unless measured; the 5% dead-birth and 60% single-event-silent floors are targets, not scores; brain execution is on the hot path.

**Decision criteria.** Whether an option restores mutational exposure of the functional core at depth without raising dead births above the floor; whether it bounds junk without freezing the core; hot-path and mutation-time cost; whether it is a bounded T11 feature with a natural analog; whether the closure instruments would detect a regression of it.

## 2. What the roadmap covers today

| Item | Covers | Does not cover |
| --- | --- | --- |
| T11.F04 ([spec](../specs/roadmap/t11-f04-mutation-supply-and-neutral-scaffold.md)) | 0.55 requested events per birth as mostly single events; `reachable_bias` 0.0 in every domain so inactive structure gets the same target opportunity as live code; size pressure off | The consequence at depth: as inactive structure grows, the live core's share of events falls in proportion. The mesh note recorded this as "a known trade, not a T11 repair target" (its Section 4, item 10) from a structure-only reading; the functional cost is measured here |
| T11.F13 (after T11.F10) | Fixed-rate arms below, at, and above 0.55; treatment arms for size-scaled supply, depth-weighted targeting, and reachable bias with CGP's single active-gene mutation as the reference arm | Executed-biased targeting (the reachable set is five times the executed set on the live population, Section 3.3); a junk bound; a depth reading. Its Sims arm is described as the answer to dilution, but Sims' rule holds per-genome supply constant, which is Petri's current regime (Section 6) |
| T11.F14 | Executed count, route variation, knockouts, hop-cap hits, generation depth of the goal profile | Any reading deeper than the goal profile's 22 generations; no drift-to-depth component |
| T11.F15 | Branch with bid, activation-neutral destinations, single-visit routing | Nothing missing for this question; its effect is confirmed at depth (Section 3.3) |
| T03.F08 (after T03.F01, T11.F10) | Maintenance cost for larger or more active controllers, after measured realized costs | The realized cost is now measurable: brain compute is 0.03% of metabolic decay per creature-tick, so junk is free (Section 3.1) |
| T02.F01, T02.F03 | Seasons and regional offsets, the world's modularly varying pressure | Nothing missing; Section 6 adds the evidence that a static world selects against evolvability, which argues for keeping them next after the supply repair |
| `genome_size_pressure_enabled` ([config spec](../reference/v3-runtime-config-spec.md)) | Above `genome_size_cap` (1,200; the live mean is 6,449) restrict births to complexity-decreasing operators | It restricts every domain, so it freezes the core along with the junk (Section 5.3) |

## 3. Local evidence

### 3.1 The server at tick 281,405

Read from `GET /v3/simulation/status`, `/config`, and a whole-world `/snapshot` at detail zoom.

| Quantity | Value |
| --- | ---: |
| Population; births over the run; births per tick | 4,742; 8,208,809; 29.2 |
| Generation of the living population: p10 / p25 / median / p75 / p90 / max | 1,744 / 1,778 / 1,990 / 2,066 / 3,140 / 3,858 |
| Ticks per generation at the median | 141 |
| Mutation targets over the run: reachable / unreachable / not applicable | 1,352,307 / 2,718,000 / 443,225 (66.8% of targeted draws hit unreachable structure) |
| Reproduction attempts rejected | 25,703,292 of 33,912,101 (76%); invalid-target causes occupied 7,745,145, barrier 2,363,849 |
| Moves blocked: barrier / occupied | 30,868,445 / 66,716,514; 60.1% of avoidable blocked moves were by creatures with no barrier reader |
| Invalid actions | 46,581,809 of 475,876,566 (9.8%), each costing the `failed_action_penalty` of 1.0 energy (ramped in by tick 62,680), five times a move |
| Brain compute energy per creature-tick, mean | 0.000152, which is 0.03% of the 0.5 per-tick decay |
| Energy: p25 / median / p75 of the living | 19 / 38 / 162 of 200 |
| `mutation_events_applied_total_semantic_noop` | 0 of 4,513,532 (the counter classifies by operator category, not by measured behavior; it cannot see silence) |

Production defaults are unchanged from the goal profile: `mutation_probability` 0.44, continuation 0.2 (0.55 requested events per birth), `mesh_layer_probability` 0.2, `reachable_bias` 0.0 in all four domains, `genome_size_pressure_enabled` false, `complexity_cost.enabled` false, no seasons.

### 3.2 Structure of the live population (400 random genomes)

From each creature's serialized genome and the server's per-node mesh annotations.

| Quantity | min | p25 | median | p75 | max | mean |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| generation | 1 | 1,773 | 1,968 | 2,065 | 3,856 | 2,064 |
| total mesh nodes | 2 | 114 | 124 | 156 | 270 | 136.1 |
| reachable mesh nodes | 1 | 13 | 17 | 25 | 50 | 19.5 |
| `genome_size` (every instruction, compute node, reference, target) | 178 | 5,970 | 6,246 | 7,167 | 11,787 | 6,449 |
| functional complexity (`complexity()`) | 1 | 510 | 532 | 764 | 1,549 | 637 |
| reachable nodes with stateful behavior | 0 | 7 | 8 | 13 | 18 | 9.2 |
| reachable nodes writing shared memory | 0 | 7 | 8 | 11 | 14 | 8.6 |
| reachable nodes with two or more route targets | 0 | 4 | 5 | 9 | 23 | 6.4 |

Total node count correlates with generation at 0.83 with a slope of 0.0595 nodes per generation, and a drift walk with no selection reaches 140 nodes at generation 2,000 (Section 3.5), so the growth is drift: selection neither adds nor removes junk. The entry node is a graph node in 395 of 400 creatures, as in the founder. Among reachable nodes, 399 creatures read food, introspection, and upstream slots; 254 read occupancy; 175 read barriers; 169 read nearby creatures.

### 3.3 Function of the live population (the T11.F01 battery)

Each of the 400 genomes was run on the 48 single-tick snapshots and 8 four-tick sequences the goal profile uses, with T11.F14's executed, route-variation, hop-cap, and knockout readings.

| Quantity | min | p25 | median | p75 | max | mean |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| executed mesh nodes | 1 | 2 | 4 | 4 | 7 | 3.40 |
| knockout count (executed node whose bypass changes nothing) | 0 | 0 | 1 | 1 | 4 | 0.88 |
| contributing executed nodes (executed minus knockout) | 0 | 2 | 3 | 3 | 3 | 2.52 |
| chain length, per-creature median over scenarios | 1 | 2 | 2 | 2 | 7 | 2.34 |
| distinct action outputs over the 48 snapshots (founder: 7) | 1 | 5 | 6 | 6 | 7 | 5.85 |

| Fraction of the 400 creatures | Value |
| --- | ---: |
| route varies with input (T11.F14 reading; the note's Probe F1 method agrees exactly) | 200 (50.0%) |
| any scenario hits the hop cap | 0 |
| executed node writes a route gate / reads shared memory / writes shared memory | 398 / 282 / 396 |
| executed nodes read barriers / occupancy / nearby creatures | 35 / 47 / 118 |
| signature identical to the founder | 0 |
| distinct behavioral signatures among the 400 | 36; the three most common cover 72, 64, and 52 creatures (47%) |
| executed share of all nodes; executed share of reachable nodes | 2.98%; 23.1% |

Two readings settle the user's observation and the T11.F15 question:

- **The "two active nodes" are the executed chain, and the chain is a three-node core.** The inspector's hop timeline shows the nodes dispatched in one sampled tick (median chain length 2, maximum 7). Over the battery the median creature executes four nodes, one of which is a silent pass-through detour, leaving three that contribute. Among executed VM nodes of at least 50 instructions (618 of 929; the rest are 19-instruction detours and small copies), the median program is 159 instructions long (founder 105) with 72 live (46%; the founder's were all live), and the executed graph nodes carry 14 compute nodes at the median (founder 6). So the core grew, and it also grew introns.
- **Conditional routing is load-bearing.** Every one of the 200 route-varying creatures has exactly three contributing executed nodes; 187 of the 200 non-varying creatures have two. The population is split between a two-node reactive phenotype and a three-node phenotype whose third node is selected by input and changes the action when bypassed. Under drift alone route variation falls to 0 to 2 of 50 lineages at generation 2,000 (Section 3.5), so the 50% is selection retaining the branch, not drift producing it. This is the reading T11.F15 predicted and the goal profile could only show as 3 of 36.

What the executed cores do not do is also visible: only 35 of 400 read barriers and 47 read occupancy in an executed node, while 60% of the run's 97.6 million avoidable blocked moves came from creatures without a barrier reader, at a penalty five times the cost of a move. A one-event input-reference addition on the executed graph node would begin to fix that, and the next section says why it is not being found.

### 3.4 The neighborhood at depth (60 live genomes, 200 production births each)

The T11.F01 per-birth battery, run on the first 60 sampled genomes with the production mutation config.

| Reading | Founder (T11.F15 gate report) | Goal profile evolved half (gen 22, T11.F15) | Live population (gen ~1,990) |
| --- | ---: | ---: | ---: |
| zero-event births | 292 of 500 | 3,900 of 7,200 | 6,735 of 12,000 |
| mutated births: silent / changed / dead | 92 / 116 / 0 of 208 | 2,069 / 1,210 / 21 of 3,300 | 5,125 / 120 / 20 of 5,265 |
| conditional silent fraction | 0.442 | 0.627 | **0.973** |
| single-event births silent | 80 of 164 (0.488) | 1,823 of 2,736 (0.666) | 4,080 of 4,168 (**0.979**) |
| behavior-changing births per all births | 0.232 | 0.168 | **0.010** |
| dead births per all births | 0.000 | 0.003 | 0.002 |

At 29 births per tick, 1.0% is still about 0.3 behavior-changing births per tick at the current rate, so evolution has slowed twenty-fold from the founder and is still slowing, not stopped; the rate was higher earlier in the run, so the run's total is not the current rate times its births. The silence has two parts, and the counterfactuals in Section 5 separate them: about 97% of node-internal events land on nodes that never execute (targeting), and, of the events that reach the core, about four in five are silent because the core itself carries 54% introns and neutral operand steps (robustness). The founder's core had no introns; a lineage that lives 2,000 generations at 0.55 events per birth evolves a robust core, as Wilke and Wagner predict and as Wright and Laue measure for complex circuits (Section 6).

### 3.5 The decline is predictable in seconds and invisible at closure

Fifty lineages walked from the founder with the production engine and no selection, read at each checkpoint on the same battery, with 100 production births per lineage on twenty of them.

| Generation | Total nodes, mean | Reachable, mean | Executed, mean | Route varies | Behavior-changing births per all births |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 0 (founder) | 2.00 | 2.00 | 2.00 | 0 of 50 | 0.2246 |
| 22 (the goal profile's depth) | 3.52 | 2.96 | 2.20 | 6 of 50 | 0.1016 |
| 250 | 17.88 | 6.40 | 2.34 | 3 of 50 | 0.0090 |
| 1,000 | 68.44 | 9.04 | 2.60 | 2 of 50 | 0.0032 |
| 2,000 | 140.02 | 16.84 | 3.16 | 0 of 50 | 0.0026 |

Drift at generation 2,000 matches the live population on structure (140 against 136 total nodes, 16.8 against 19.5 reachable) and lands below it on function (executed 3.16 against 3.40, changing births 0.26% against 1.0%): selection keeps the core a little larger and a little more exposed than drift would, and that is all it can do at this supply. The whole curve takes seven seconds in a release build. The goal profile reads the first row after the founder and cannot distinguish a substrate that will freeze by generation 250 from one that will not.

## 4. Where the stall comes from

Each item was read in code and confirmed by a probe above or in Section 5. Paths are relative to `crates/v3-core/src`.

1. **The per-birth event count does not know the genome's size.** `requested_event_count` in [`mutation/engine/mod.rs`](../../crates/v3-core/src/mutation/engine/mod.rs) draws from `mutation_probability` and the continuation probability only. A 2-node founder and a 270-node descendant receive the same 0.55 events per birth.
2. **Targets are drawn uniformly per node.** Each domain builds its eligible node list and calls `biased_select_from` ([`mutation/reachability.rs`](../../crates/v3-core/src/mutation/reachability.rs)) with `reachable_bias` 0.0, which is a uniform draw over eligible nodes; the VM mutator then picks a position inside the chosen node. A 155-instruction decision program and a 1-instruction detour have the same chance of being chosen. With 136 nodes and 3.4 executed, the core's share of node-internal events is 2.5%; the run's own counter agrees (66.8% of draws unreachable over the whole run, which includes the early generations when the mesh was small).
3. **Junk is free and unbounded.** No per-node or per-hop cost exists (brain compute is 0.03% of decay), size pressure is off, and T11.F15's `RemoveNode` prefers unreachable nodes, so removal of junk is neutral and no operator's hazard grows with size. Growth operators fire at a fixed rate per birth, so junk accumulates linearly forever. This is the T11.F04 scaffold working as designed; its cost was a structural reading before and is a functional one now.
4. **The reachable set is not the executed set.** T11.F04's `reachable_bias` knob and T11.F13's reference arm both target reachability. On the live population reachable nodes are 19.5 and executed 3.4, because T11.F15 deliberately births branches reachable-but-losing. A reachable bias of 1.0 aims at the scaffold as much as at the core (Section 5.1).
5. **The core evolved robustness.** 54% of the executed decision program is dead code and four in five events that reach the core are silent. This is the expected outcome of 2,000 generations under a fixed supply in a fixed world (Section 6), and a supply repair restores exposure without restoring the founder's brittle neighborhood.
6. **The world is static, and the population is space-limited.** 76% of reproduction attempts are rejected, mostly by occupied targets, and a quarter of the living sit above 160 of 200 energy. Whatever pressure exists is on crowding and blocked moves, and the sensors that would answer it are unread in 90% of executed cores. This is descriptive; selection strength was not measured.
7. **The closure profile reads at generation 22.** Every stored goal reading in `docs/progress.md` is at that depth. T11.F14 made the depth visible; nothing makes a deeper reading.

## 5. Counterfactuals

All arms reuse the production engine, battery, and classification. Live-genome arms change only the mutation config passed to the births probe; drift arms change the config the walk itself uses, so junk growth responds to the policy.

### 5.1 Supply policy on the live genomes (60 genomes, 200 births each)

| Arm | What changes | Zero-event births | Mutated births silent / changed / dead | Behavior-changing per all births | Dead per all births |
| --- | --- | ---: | ---: | ---: | ---: |
| baseline | production config | 6,735 of 12,000 | 0.973 / 0.023 / 0.004 | 0.0100 | 0.0017 |
| bias1 | `reachable_bias` 1.0 in all domains (T11.F13's CGP reference arm) | 6,735 | 0.911 / 0.086 / 0.003 | 0.0377 | 0.0015 |
| exec1 | bias 1.0 toward the **executed** set (the battery's executed node indices passed as the target slice) | 6,772 | 0.787 / 0.209 / 0.005 | **0.0910** | 0.0020 |
| scaled | per-node supply: requested mean 0.275 events per node (founder-equivalent), geometric tail | 0 | 0.587 / 0.363 / 0.050 | **0.3632** | 0.0495 |

The exec1 arm uses its own birth loop (the target slice is a probe input, which `evaluate_genome` does not take), so its birth seeds differ and its zero-event count is 6,772 rather than 6,735; classification is the same. Reading: aiming the same supply at what executes recovers nine times the exposure with no dead-birth cost; the remaining 79% silence is the core's own robustness. A per-node supply recovers founder-level exposure and pays for it in dead births at the floor, and its geometric burst (mean 37 events at 136 nodes) is a crude stand-in for independent per-node draws.

### 5.2 Supply policy under drift to generation 2,000 (50 lineages; births on 20)

| Arm | Generation | Total nodes | `genome_size` | Reachable | Executed | Route varies | Changing births per all births | Dead per all births |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| baseline | 2,000 | 141.4 | 5,021 | 22.1 | 3.40 | 2 of 50 | 0.0035 | 0.0020 |
| bias1 | 2,000 | 147.1 | 5,352 | 12.5 | 3.78 | 1 of 50 | 0.0045 | 0.0025 |
| exec1 (executed set refreshed every 10 generations) | 2,000 | 149.6 | 6,536 | 16.8 | 4.68 | 0 of 50 | 0.0105 | 0.0010 |
| scaled | 50 / 100 / 150 / 155 | 9.4 / 47.7 / 329.6 / 401.1 (aborted) | | | | | | |
| pressure (`genome_size_pressure_enabled`, cap 1,200) | 2,000 | 44.0 | 701 | 3.64 | 3.14 | 0 of 50 | 0.0010 | 0.0005 |
| scaled + pressure | 2,000 | 99.3 | 666 | 1.92 | 1.92 | 0 of 50 | 0.0000 | 0.0010 |

Reading, arm by arm:

- **scaled is explosive without a bound.** Per-node supply makes growth events proportional to size, so the mesh grows five- to seven-fold every fifty generations (doubling in about twenty) and passes 400 nodes by generation 155. This is the same mechanism Luiselli et al. name for point-mutation-only genomes ("likely to grow indefinitely," Section 6); a per-site rate needs a size hazard or a cap to be stable, and every platform that uses one has one.
- **pressure freezes the core.** Above the cap the engine restricts every domain to complexity-decreasing operators, so the core cannot gain an instruction or an edge either; changing births fall to 0.1%. As written the flag is not a junk bound, it is a growth stop.
- **scaled + pressure eats the core under drift.** Twenty-seven events per birth restricted to removals reduce the founder's chain to 2.06 executed nodes at generation 1,000 and 1.92 at 2,000, with a fully silent neighborhood from generation 1,000. Drift is the wrong regime to judge a heavy supply (selection would hold the core), but it shows the combination is not safe by itself.
- **exec1 is the only arm that raises executed nodes and exposure together under drift**, and under selection on the live genomes it is nine times baseline. It changes no event count and needs no bound, because junk keeps growing at the baseline rate and is simply not targeted.

### 5.3 What the arms leave open

No arm here was run under selection in the world; the live-genome readings are neighborhoods of selected genomes, not persistence. Executed-biased targeting needs an executed-node set at birth, which the runtime does not keep (the T11.F14 observation mode records hops; a per-creature "executed in the last N ticks" bitmap is the natural source and is cheap). The per-node supply needs a junk bound that does not freeze the core: a maintenance cost (T03.F08), a use-dependent pruning of nodes that have not executed for many generations (Sims' rule), or a node budget with growth blocked only above it. Those are design decisions for the feature spec, not for this note.

## 6. What established systems do

Read depth is marked per row: *full text* means the primary text was read in full; *summary* means the full text was fetched and a section-level summary with verbatim quotations was extracted; *abstract* means the abstract only.

| System or study | Mutation supply | Genome-size bound | Petri today |
| --- | --- | --- | --- |
| **Avida**, [mutation settings wiki](https://github.com/devosoft/avida/wiki/Mutation-settings) (*summary*) | `COPY_MUT_PROB` is per site, "tested each time an organism copies a single instruction"; `POINT_MUT_PROB` is "a probability for each site that it will be mutated each update"; the divide insertion and deletion mutations are per divide, "at most one divide mutation of each type" | Genome length is bounded by the per-site load: Gupta, LaBar, Miyagi, and Adami 2016 (*abstract*, [Sci. Rep.](https://www.nature.com/articles/srep25786)) find "mutation rate is inversely correlated with genome size in asexual populations," from "the tradeoff between evolving phenotypic innovation and limiting the mutational load" | Per birth, size-blind; no load, no bound |
| **Aevol**, Knibbe, Coulon, Mazet, Fayard, and Beslon 2007 (*summary*, [MBE](https://academic.oup.com/mbe/article/24/10/2344/1074262)) | Per base; rearrangements whose hazard grows with the noncoding length: "longer intergenic sequences tend to enhance the level of nonneutral variation," being "mutagenic for the genes they surround" | "Under low mutation rates, the indirect selection of variability promotes the accumulation of noncoding sequences" while "high mutation rates lead to compact genomes," with no direct cost of genome size; the evolved neutral-offspring fraction approaches 1/W | Petri's junk carries no hazard for the core, so nothing selects against it |
| **Genome-size theory**, Luiselli et al. 2025 (*summary*, [MBE](https://pmc.ncbi.nlm.nih.gov/articles/PMC12690204/)) | Point mutations, indels, and structural mutations | "With only indels, our model predicts that genomes are likely to grow indefinitely"; the equilibrium noncoding fraction is set by "robustness selection arising from ... structural mutations" and the product of effective population size and mutation rate | The unbounded growth case, exactly (Section 5.2, scaled arm) |
| **Markov Brains**, Hintze et al. 2017 (*full text*, [arXiv](https://arxiv.org/abs/1709.05601)) | "Each site has a chance to mutate at every replication event," plus a 20% chance per replication of a 256-to-512-site deletion and the same for a duplication | Deletions "can only occur if the genome is larger than 1000 bytes"; duplication "is not allowed if the genome is already above 20,000 sites"; "both limits on insertions and deletions prevent genomes from disappearing or becoming intractably large" | Per-site supply and a hard cap, both absent |
| **Cartesian GP**, Miller and Thomson 2000 (*abstract*, [EuroGP](https://gpbib.cs.ucl.ac.uk/gp-html/miller_2000_CGP.html)); Turner and Miller 2014 (*abstract*, [EuroGP](http://gpbib.cs.ucl.ac.uk/gp-html/turner_2014_EuroGP.html)); Dang, Kalkreuth, and Opris 2026 (*full text*, read for the mesh note) | Per-gene rates over a genotype that "is just a list of node connections and functions"; single active-gene mutation "repeatedly modifies genes chosen uniformly at random until an active node is modified" | The node count is a parameter of the representation (the D gates of Dang et al.'s analysis), so "Cartesian Genetic Programming (CGP) does not exhibit program bloat," and "using a very large number of nodes considerably increases the effectiveness of the search" | The fixed length is the bound; single active-gene mutation is the exec1 arm |
| **Evolving Virtual Creatures**, Sims 1994 (*full text*, quoted in the mesh note) | Mutation frequencies "scaled by an amount inversely proportional to the size of the current graph," which holds per-genome supply constant | "At least the unconnected newly added ones are removed to prevent unnecessary growth in graph size" | Petri has Sims' constant per-genome supply without Sims' garbage collection. The roadmap's T11.F13 arm records Sims as the answer to dilution; it is the current regime minus the pruning, and the arm should be restated |
| **Tangled Program Graphs**, Kelly and Heywood 2017, 2018 (*full text*, mesh note) | Per-team operator probabilities applied to root teams only: "only root teams are subject to modification by the variation operators" | Interior teams are frozen, so supply on the new material does not dilute as the graph grows to about 60 teams | The nearest analog of executed-biased targeting: mutate where the graph is being built, not uniformly |
| **Large-scale ecological neuroevolution**, 2510.18221 (*summary*, [arXiv](https://arxiv.org/html/2510.18221)) | "Gaussian noise with standard deviation 3×10⁻² to all mutable weight values," a per-parameter rate on fixed-size networks of 10k to 25k parameters | Fixed architecture | Per-parameter supply cannot dilute |

Environment and robustness, the second half of the diagnosis:

| Study | Finding | Petri today |
| --- | --- | --- |
| **Static versus fluctuating environments in Avida**, Canino-Koning, Wiser, and Ofria 2019 (*summary*, [PLOS Comput. Biol.](https://pmc.ncbi.nlm.nih.gov/articles/PMC6474582/)); fixed 121-instruction genomes at 0.00075 per site | "Static environments select solely for immediate optimization, at the expense of long-term evolvability"; fluctuating environments produced "reservoirs of pseudogene-like" vestigial code and a higher phenotypic diffusion rate | The production world is static; Section 3.3 reads three dominant phenotypes |
| **Evolvability under environmental change in Avida**, Kumawat, Lalejini, Acosta, and Zaman 2024 (*summary*, [PNAS](https://pmc.ncbi.nlm.nih.gov/articles/PMC11725885/)) | In constant environments mutation rates declined; "the Cyclic and Random regimes were conducive to the evolution of higher mutation rates"; cyclic-evolved genotypes "had a markedly larger pool of mutants expressing the alternative phenotype" (median 45,802.5 against 1.0) | Petri's rate is fixed, so the decline shows up as robustness of the core instead (Section 3.4) |
| **Dynamic fitness landscapes**, Petak, Frati, Vroomans, Pespeni, and Cheney 2025 (*summary*, [PNAS](https://pmc.ncbi.nlm.nih.gov/articles/PMC12745803/)) | "In most static experiments, populations climbed to the nearest narrow local optima and stayed there"; "GRNs from variable runs were more robust to random mutations than GRNs from static runs" | Same reading; robustness is what a static world buys |
| **Evolving complexity is hard**, Wright and Laue 2022 (*full text*, mesh note; abstract re-read) | "Due to the inherent structure of the G-P map, including the predominance of rare phenotypes, large interconnected neutral networks, and the high mutational load of low robustness, complex phenotypes are difficult to discover using evolution" | Robustness is the silent fraction; Section 3.4's 0.973 is a population that has moved as far toward robustness as the supply allows |
| **Modularly varying goals**, Kashtan and Alon 2005; **noise and environmental change in a G-P map**, Ikeda, Kaneko, and Hatakeyama 2026 (both as read for the mesh note) | Fixed goals gave low modularity; "frequent environmental change instead favors mutational accessibility at the expense of penetrance" | T02.F01 seasons are Petri's modularly varying pressure |

Also scanned and set aside: Akhtyrchenko, Katsnelson, and Ustyuzhanin 2026 (*abstract*, [arXiv](https://arxiv.org/abs/2606.17091)), a physics-derived complexity metric for cellular automata, not an evolutionary mechanism; Hamon, Nisioti, and Moulin-Frier 2023 (*abstract*, [arXiv](https://arxiv.org/abs/2302.09334)) and JaxLife, Lu et al. 2024 (*abstract*, [arXiv](https://arxiv.org/abs/2409.00853)), non-episodic neuroevolution on fixed-size recurrent networks, which cannot dilute and are scale references only. Dolson, Vostinar, and Ofria 2015, "What's holding artificial life back from open-ended evolution?", could not be retrieved from any host and is not cited for any claim.

The pattern is uniform. Every platform in the table either charges mutation per site or per gene, so that a growing genome carries a growing load and the live code's exposure does not depend on how much junk surrounds it, or fixes the genome's size, or both. Petri's fixed per-birth count with a uniform per-node draw is the one design in which junk is both free to carry and free of consequence, and the consequence is the one measured here: the live core's exposure decays as one over the node count.

## 7. Options

| # | Option | Kind | Evidence and fit | Cost and risk | Verdict |
| --- | --- | --- | --- | --- | --- |
| P1 | **Executed-biased targeting.** Draw node-internal targets from the nodes that executed in the parent's recent ticks with high probability, uniformly otherwise, keeping 0.55 events per birth | Extend the engine; add a per-creature executed bitmap maintained by the mesh executor | Section 5.1: 1.0% to 9.1% behavior-changing births on live genomes with dead unchanged; the only drift arm that raises executed nodes and exposure together. CGP's single active-gene mutation; TPG's root-only mutation. Natural analog: transcription-associated mutagenesis, where actively expressed genes mutate more | One bitmap per creature on the hot path; the scaffold still drifts at the residual rate. Does not bound junk, which keeps costing memory and mutation-time work | **Adopt** as T11.F17 (Section 10), not as a T11.F13 arm |
| P2 | **Per-node supply.** Requested events proportional to node count, drawn independently per node rather than as a geometric burst | Extend the engine | Section 5.1: 36% changing births with dead at the 5% floor; the universal design (Section 6). Natural analog: per-base copy error | Section 5.2: exponential junk growth without a bound; dead births rise with size; mutation-time cost grows with junk | **Characterize** in T11.F13 only together with P3 |
| P3 | **A junk bound that does not freeze the core.** Candidates: per-node maintenance cost (T03.F08; analog: neurons are expensive), use-dependent pruning of nodes unexecuted for N generations (Sims' rule; analog: synaptic pruning), or a node budget above which only growth is blocked | Extend | Section 5.2: the existing `genome_size_pressure` restricts every domain and freezes the core (0.1% changing births), so it is not this option as written | Any bound taxes the T11.F04 scaffold; a cost changes ecology and belongs to T03.F08; pruning contradicts "junk is free by design" and needs the executed bitmap of P1 | **Characterize** with P2; P1 alone needs none |
| P4 | **Depth indicator.** A drift-to-generation-1,000 and -2,000 reading of total, reachable, executed, route-variation, and behavior-changing births per birth, in the gate or goal profile | Extend T11.F14 and the benchmark harness | Section 3.5: seven seconds; would have shown the 0.9% at generation 250 at T11.F04's closure | None on the simulation; predeclared floors needed | **Adopt** as T11.F16 (Section 10) |
| P5 | Reachable bias 1.0 (T11.F13's existing reference arm) | Tuning | 3.8% on live genomes, 0.45% under drift; the reachable set is five times the executed set | None | Keep as the reference arm; insufficient alone |
| P6 | Raise the per-birth supply globally | Tuning | Founder dead fraction rises with bursts (audit); dilution is unchanged in proportion | | **Reject** |
| P7 | Seasons and regional offsets (T02.F01, T02.F03) | Scheduled | Section 6: static worlds select for robustness and immediate optimization; Section 3.3: the sensors that would answer the world's one pressure are unread | Already on the priority list after T11.F13 | Keep next after the supply repair; do not run before it, since a static-world reading of a frozen substrate cannot separate the two causes |
| P8 | Replace the controller or the mesh | Adopt | Section 3.3: the mesh routes conditionally, reads memory, and grows under selection; nothing measured implicates it | Months | **Reject**, unchanged from the audit |
| P9 | Correct the T11.F13 Sims arm | Roadmap text | Section 6: Sims held per-genome supply constant and pruned unconnected nodes; the arm as written describes Petri's current regime | None | **Adopt** |

## 8. Recommendation

Three changes. They were applied to the roadmap on 2026-09-07 at the user's direction; Section 10 records what was written where.

1. **A new bounded T11 delivery-repair feature, placed before T11.F09 and T11.F10: executed-biased mutation targeting (P1) plus the depth reading (P4).** This respects the 2026-09-05 decision that separates delivery repair (T11.F04's class) from rate characterization (T11.F13's class): P1 changes where the existing 0.55 events per birth land, not how many there are, and P4 is measurement. The feature keeps a per-creature record of which mesh nodes executed in recent ticks, draws node-internal targets from that set with a predeclared bias and uniformly otherwise, and predeclares its floors from Section 5: behavior-changing births per all births at generation 1,000 under drift not below a stated multiple of the baseline's 0.32%, executed nodes at depth not below the baseline's, dead births not up. Natural analog: transcription-associated mutagenesis, where actively expressed genes mutate more than silent ones. It goes before T11.F10 because T11.F10 measures whether founders discover and retain a remembered decision through viable mutations, and on the current policy the substrate's exposure decays to 1% of births within a few hundred generations, so a null result at any exposure T11.F10 can afford would be uninterpretable. Adding the feature ahead of T11.F10 changes T11.F13's dependency chain only indirectly; T11.F13 itself stays after T11.F10 as written.
2. **Leave T11.F13 as characterization, with its arms corrected.** It keeps the fixed-rate arms and the reachable-bias reference arm (P5), adds per-node supply with a junk bound (P2 with P3) as the characterized alternative to P1, and restates its Sims arm per P9: Sims held per-genome supply constant and garbage-collected unconnected nodes, so the arm to test is the pruning, not the scaling. Natural analogs for that pair: per-base copy error for the supply; synaptic pruning or metabolic cost for the bound. The depth reading from item 1 is what makes those arms comparable: under the no-regression rule it would have blocked T11.F04's bias change until its depth cost was measured.
3. **Keep T02.F01 immediately after.** Section 6 is unanimous that a static world selects against evolvability; Section 3.3 shows the run's one strong pressure, blocked moves and crowding, is answered by sensors the executed cores do not read. Seasons on a substrate that can route conditionally and can still receive mutations is the first fair test of the whole T11 investment.

Also worth recording without a feature: the inspector's "active nodes" are the hop timeline of one sampled tick, which is the executed chain (median 2); the reachable and executed counts of Section 3.3 are the right readings for "how much brain is in use," and the T11.F14 battery already produces them.

## 9. Remaining uncertainty and the cheap proofs

- **No arm was run under selection in the world.** The live-genome arms read neighborhoods of selected genomes; the drift arms read policy dynamics without selection. The cheap proof for P1 is a goal-profile seed run for 2,000 ticks under executed-biased targeting (about ten minutes), read with the Section 3.3 census; the deep proof is a sweep-profile run to generation 500 or more, which nobody has budgeted and which P4 makes unnecessary as a gate.
- **The battery zeroes shared memory on snapshots.** 70% of executed cores read memory, so memory-conditioned behavior differences are undercounted in "silent" (the audit's own limit). The 97.3% is an upper bound on reactive silence, not proof that nothing changes.
- **The scaled arm is a geometric burst**, not independent per-node draws, so its dead fraction overstates a real per-site policy and its changed fraction is directional.
- **Executed-biased targeting needs a runtime executed set.** The battery's executed set was used as a stand-in. A per-creature bitmap over recent ticks will differ from it (the world's inputs are not the battery's), and the feature spec has to decide the window.
- **The population's selection regime was described, not measured.** 76% rejected reproductions and a quarter of creatures near full energy suggest crowding-limited selection, and the failed-action-penalty ramp ended at tick 62,680; whether the population fell from the goal profile's 11,000 because of the ramp or the depth was not read.
- **Junk's cost in wall clock was not measured.** At 136 nodes and 6,449 genome-size units per creature, mutation-time reachability and the frontend's inspector both scale with it; a bound may be wanted for throughput reasons that this note did not read.
- **Literature depth.** Knibbe 2007, Luiselli 2025, Canino-Koning 2019, Kumawat 2024, Petak 2025, and the 2510.18221 methods were read as fetched summaries with verbatim quotations, not in full; Gupta et al. 2016 and the CGP papers at abstract level; Markov Brains, Sims, TPG, Wright and Laue, and Dang et al. in full (the last four for the mesh note). No claim above rests on a source read more shallowly than marked.

## 10. Decisions applied to the roadmap on 2026-09-07

Applied after the user reviewed Sections 8 and 9 and decided that the disabled `energy.complexity_cost` and `mutation.genome_size_pressure_enabled` levers stay in the code rather than being removed.

| # | Decision | Ground | Where |
| --- | --- | --- | --- |
| T11.F16 | New measurement feature, Drift-Depth Indicator: the Section 3.5 walk lifted into the benchmark harness, recording total, reachable, executed, and knockout counts, route variation, and per-birth silent, changed, and dead tallies at generations 250, 1,000, and 2,000; no floor of its own; depends on T11.F14. Placed after T11.F08 and before T11.F09 in the master priority order | Sections 3.5 and 5.2; P4 | T11 track features, criteria, and notes; master priority order |
| T11.F17 | New delivery-repair feature, Executed-Biased Mutation Targeting: node-internal targets drawn from the nodes the parent's brain executed in recent ticks with a predeclared bias and a uniform residual; event count, weights, and operator semantics unchanged; depends on T11.F04, T11.F15, T11.F16; predeclared directions from Section 5; natural analog transcription-associated mutagenesis (Park, Qian, and Zhang 2012). Placed after T11.F16 and before T11.F09 | Section 5.1 (1.0% to 9.1%), Section 5.2 (exec1 arm); P1 | T11 track features, criteria, and notes; master priority order; pending-owner sentence in `v3-mutation-spec.md` |
| Split of recommendation 1 | The note proposed one feature; the roadmap splits it into measurement first and repair second, following the T11.F14 and T11.F15 precedent, so the repair predeclares its floors against a stored baseline | Roadmap no-regression rule | T11 sequencing note |
| T11.F13 | Stays after T11.F10; Sims arm restated (pruning, not scaling); per-node supply and a junk bound added as paired arms with the Section 5 readings; both existing levers stay disabled until it reads them | Sections 5.2, 6, and 7; P2, P3, P5, P9 | T11.F13 note |
| T03.F08 | Records the first realized cost reading (0.03% of decay) and that `complexity_cost` taxes `complexity()`, not junk | Section 3.1 | T03 notes |
| T02.F01 | Stays after the supply repair; the static-world evidence recorded | Section 6 | T02 notes |
| Research basis | The Section 6 sources with read depth, plus Park, Qian, and Zhang 2012 for the T11.F17 analog | Section 6 | T11 notes |

Not applied: the master roadmap's final success criteria are unchanged, since the drift-depth readings become indicator components under the existing no-regression criterion; no feature was removed or retired.

## Sources

- [Avida wiki, Mutation settings](https://github.com/devosoft/avida/wiki/Mutation-settings)
- [Gupta, LaBar, Miyagi, and Adami, Evolution of Genome Size in Asexual Digital Organisms, Scientific Reports 2016](https://www.nature.com/articles/srep25786) ([arXiv](https://arxiv.org/abs/1511.05548))
- [Knibbe, Coulon, Mazet, Fayard, and Beslon, A Long-Term Evolutionary Pressure on the Amount of Noncoding DNA, Molecular Biology and Evolution 2007](https://academic.oup.com/mbe/article/24/10/2344/1074262)
- [Luiselli et al., Structural Mutations Set an Equilibrium Noncoding Genome Fraction, Molecular Biology and Evolution 2025](https://pmc.ncbi.nlm.nih.gov/articles/PMC12690204/)
- [Hintze et al., Markov Brains: A Technical Introduction, 2017](https://arxiv.org/abs/1709.05601)
- [Miller and Thomson, Cartesian Genetic Programming, EuroGP 2000](https://gpbib.cs.ucl.ac.uk/gp-html/miller_2000_CGP.html)
- [Turner and Miller, Cartesian Genetic Programming: Why No Bloat?, EuroGP 2014](http://gpbib.cs.ucl.ac.uk/gp-html/turner_2014_EuroGP.html)
- [Dang, Kalkreuth, and Opris, Runtime Analysis of Cartesian Genetic Programming in Evolving Boolean Functions, 2026](https://arxiv.org/abs/2606.15923)
- [Sims, Evolving Virtual Creatures, SIGGRAPH 1994](https://www.karlsims.com/papers/siggraph94.pdf)
- [Kelly and Heywood, Emergent Tangled Graph Representations for Atari Game Playing Agents, EuroGP 2017](https://web.cs.dal.ca/~mheywood/OpenAccess/open-kelly17a.pdf)
- [The Emergence of Complex Behavior in Large-Scale Ecological Environments, 2025](https://arxiv.org/abs/2510.18221)
- [Canino-Koning, Wiser, and Ofria, Fluctuating environments select for short-term phenotypic variation leading to long-term exploration, PLOS Computational Biology 2019](https://pmc.ncbi.nlm.nih.gov/articles/PMC6474582/)
- [Kumawat, Lalejini, Acosta, and Zaman, Evolution takes multiple paths to evolvability when facing environmental change, PNAS 2024](https://pmc.ncbi.nlm.nih.gov/articles/PMC11725885/)
- [Petak, Frati, Vroomans, Pespeni, and Cheney, The variability of evolvability, PNAS 2025](https://pmc.ncbi.nlm.nih.gov/articles/PMC12745803/)
- [Wright and Laue, Evolving Complexity is Hard, 2022](https://arxiv.org/abs/2209.13013)
- [Kashtan and Alon, Spontaneous evolution of modularity and network motifs, PNAS 2005](https://pmc.ncbi.nlm.nih.gov/articles/PMC1236541/)
- [Ikeda, Kaneko, and Hatakeyama, 2026](https://arxiv.org/abs/2608.24704)
- [Park, Qian, and Zhang, Genomic evidence for elevated mutation rates in highly expressed genes, EMBO Reports 2012](https://www.embopress.org/doi/full/10.1038/embor.2012.165) (abstract, via Europe PMC: "the rate of point mutation in a gene increases with the expression level of the gene"; the natural analog named by T11.F17)
- Scanned, set aside: [Akhtyrchenko, Katsnelson, and Ustyuzhanin, 2026](https://arxiv.org/abs/2606.17091); [Hamon, Nisioti, and Moulin-Frier, 2023](https://arxiv.org/abs/2302.09334); [Lu et al., JaxLife, 2024](https://arxiv.org/abs/2409.00853)
- Local: [brain evolvability audit](brain-evolvability-audit-2026-09-04.md); [mesh evolvability research note](mesh-evolvability-research-2026-09-06.md); [T11.F15 spec](../specs/roadmap/t11-f15-mesh-routing-connection-semantics.md) (founder and generation-22 rows); [T11.F04 spec](../specs/roadmap/t11-f04-mutation-supply-and-neutral-scaffold.md)

## Appendix A: reproducing the readings

The server readings need a running `v3-server` with a paused or running world:

```sh
curl -s http://localhost:3000/v3/simulation/status > status.json
curl -s http://localhost:3000/v3/simulation/config > config.json
curl -s "http://localhost:3000/v3/simulation/snapshot?x=0&y=0&width=1600&height=1600&canvas_width=1600&canvas_height=1600&zoom_tier=detail" > detail.json
# creature ids and generations are in detail.json view.creatures; sample 400 with seed 11, then
curl -s "http://localhost:3000/v3/simulation/creature/<id>?exclude=action_log" -o genomes/<id>.json
```

The probe below was run from `crates/v3-core/tests/zz_probe_live_census.rs` at 32468c20 and removed afterward, following the audit's pattern. Drop it back in and run:

```sh
PETRI_GENOME_DIR=<dir> PETRI_BIRTHS_N=60 PETRI_ARM=baseline \
  cargo test --release -p v3-core --test zz_probe_live_census probe_live_census -- --nocapture   # 4 s plus births
PETRI_ARM=bias1|exec1|scaled ...                                                                  # the Section 5.1 arms
cargo test --release -p v3-core --test zz_probe_live_census probe_drift_depth_births -- --nocapture   # Section 3.5, seconds
PETRI_ARMS=baseline,bias1,exec1,pressure,scaled_pressure,scaled \
  cargo test --release -p v3-core --test zz_probe_live_census probe_drift_policies -- --nocapture    # Section 5.2; scaled aborts at 400 nodes
```

All readings are deterministic (`SmallRng`; lineage seeds 90,000 and 95,000 upward; birth seeds as in the source; battery seeds 7 and 8 through `Battery::generate`). Section 3.5 and Section 5.2 baseline rows come from two runs with different lineage seeds and agree within their sampling error.

## Appendix B: the probe

```rust
//! TEMPORARY PROBE (not for commit): functional census of live genomes dumped
//! from a running server (`GET /v3/simulation/creature/:id`), mirroring the
//! 2026-09-06 research note's Probe F1 plus the T11.F14 battery readings and a
//! bounded per-birth neighborhood on a subset.
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{Hash, Hasher};
use v3_core::config::{FounderProfile, RuntimeConfig, SimulationConfig};
use v3_core::contracts::{NodeId, WorldAction};
use v3_core::creature::founder::founder_genome;
use v3_core::creature::genome::analysis::mesh_reachable_nodes;
use v3_core::creature::genome::cgp::OutputSinkKind;
use v3_core::creature::genome::{BackendDef, CreatureGenome, VmInstruction};
use v3_core::creature::state::GraphRuntimeState;
use v3_core::neighborhood::{evaluate_genome, Battery, EvalContext, Signature};
use v3_core::runtime::trace::domain::TerminationReason;
use v3_core::runtime::traced_mesh::execute_creature_mesh_traced;
use v3_core::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
use v3_core::sensors::static_inputs::StaticInputs;
use v3_core::sensors::typed_food::TypedFoodLocalSnapshot;

const AGE_CHOICES: [f32; 6] = [0.0, 5.0, 19.0, 20.0, 50.0, 200.0];
const ENERGY_CHOICES: [f32; 6] = [5.0, 15.0, 25.0, 31.0, 45.0, 80.0];
const RESERVE_CHOICES: [f32; 4] = [0.0, 2.0, 4.0, 8.0];

struct Scenario {
    sensors: SensorSnapshot,
    energy: f32,
    reserve: f32,
}

fn nonzero_or_zero(rng: &mut SmallRng, p_zero: f64) -> f32 {
    if rng.gen_bool(p_zero) { 0.0 } else { rng.gen_range(0.1f32..=1.0) }
}

fn draw_scenario(rng: &mut SmallRng, food_type_count: usize) -> Scenario {
    let food_here_by_type: Vec<f32> = (0..food_type_count)
        .map(|t| nonzero_or_zero(rng, if t == 0 { 0.5 } else { 0.6 }))
        .collect();
    let age = AGE_CHOICES[rng.gen_range(0..AGE_CHOICES.len())];
    let energy = ENERGY_CHOICES[rng.gen_range(0..ENERGY_CHOICES.len())];
    let reserve = RESERVE_CHOICES[rng.gen_range(0..RESERVE_CHOICES.len())];
    let neighbor_food_by_type: Vec<[f32; 8]> = (0..food_type_count)
        .map(|_| std::array::from_fn(|_| nonzero_or_zero(rng, 0.6)))
        .collect();
    let neighbor_occupied: [f32; 8] =
        std::array::from_fn(|_| if rng.gen_bool(0.2) { 1.0 } else { 0.0 });
    let food_here = food_here_by_type.first().copied().unwrap_or(0.0);
    let neighbor_food = neighbor_food_by_type.first().copied().unwrap_or([0.0; 8]);
    Scenario {
        sensors: SensorSnapshot {
            local: StaticInputs {
                food_here,
                neighbor_food,
                neighbor_barrier: [0.0; 8],
                neighbor_occupied,
                generation: 0.0,
                age_ticks: age,
            },
            typed_local_food: TypedFoodLocalSnapshot { food_here_by_type, neighbor_food_by_type },
            perception: PerceptionSnapshot::zeroed(food_type_count),
        },
        energy,
        reserve,
    }
}

fn scenarios(food_type_count: usize, count: usize) -> Vec<Scenario> {
    let mut rng = SmallRng::seed_from_u64(7);
    (0..count).map(|_| draw_scenario(&mut rng, food_type_count)).collect()
}

#[derive(Default)]
struct Census {
    executed: BTreeSet<NodeId>,
    chain_lens: Vec<usize>,
    choices: BTreeMap<NodeId, BTreeSet<usize>>,
    max_hops_hits: usize,
    executed_multi_target: usize,
    executed_gate_writer: bool,
    executed_memory_reader: bool,
    executed_memory_writer: bool,
    outputs: Vec<Vec<WorldAction>>,
}

fn node_writes_route_gate(genome: &CreatureGenome, id: NodeId) -> bool {
    let Some(node) = genome.nodes.iter().find(|n| n.node_id == id) else { return false };
    match &node.backend_def {
        BackendDef::Vm(vm) => vm.program.iter().any(|i| matches!(i, VmInstruction::WriteRouteGate { .. })),
        BackendDef::Graph(g) => g.output_sinks.iter().any(|s| matches!(s.kind, OutputSinkKind::RouterGate(_)) && !s.inputs.is_empty()),
    }
}

fn node_memory(genome: &CreatureGenome, id: NodeId) -> (bool, bool) {
    let Some(node) = genome.nodes.iter().find(|n| n.node_id == id) else { return (false, false) };
    match &node.backend_def {
        BackendDef::Vm(vm) => {
            let r = vm.program.iter().any(|i| matches!(i, VmInstruction::LoadSlot { .. } | VmInstruction::LoadSlotImm { .. } | VmInstruction::LoadSlotPrev { .. }));
            let w = vm.program.iter().any(|i| matches!(i, VmInstruction::StoreSlot { .. } | VmInstruction::StoreSlotImm { .. } | VmInstruction::ClearSlot { .. }));
            (r, w)
        }
        BackendDef::Graph(g) => {
            let w = g.output_sinks.iter().any(|s| matches!(s.kind, OutputSinkKind::WriteSlot(_) | OutputSinkKind::ClearSlot(_)) && !s.inputs.is_empty());
            (false, w)
        }
    }
}

fn census(genome: &CreatureGenome, runtime: &RuntimeConfig, scen: &[Scenario]) -> Census {
    let mut c = Census::default();
    for s in scen {
        let mut energy = s.energy;
        let mut shared = [0.0f32; 16];
        let prev = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();
        gr.begin_tick(&genome.nodes);
        let (out, hops, term) = execute_creature_mesh_traced(genome, &s.sensors, &mut energy, s.reserve, &mut shared, &prev, &mut gr, runtime);
        if matches!(term, TerminationReason::MaxHopsReached) { c.max_hops_hits += 1; }
        c.chain_lens.push(hops.len());
        for h in &hops {
            c.executed.insert(h.node_id);
            if let Some(r) = &h.route {
                c.choices.entry(h.node_id).or_default().insert(r.selected_target_idx);
            }
        }
        c.outputs.push(out.actions);
    }
    for id in &c.executed {
        let node = genome.nodes.iter().find(|n| n.node_id == *id);
        if node.is_some_and(|n| n.targets.len() >= 2) { c.executed_multi_target += 1; }
        if node_writes_route_gate(genome, *id) { c.executed_gate_writer = true; }
        let (r, w) = node_memory(genome, *id);
        c.executed_memory_reader |= r;
        c.executed_memory_writer |= w;
    }
    c
}

fn route_varies(c: &Census) -> bool { c.choices.values().any(|set| set.len() >= 2) }

fn pct(v: &mut Vec<usize>, p: f64) -> usize {
    v.sort_unstable();
    if v.is_empty() { return 0; }
    let i = ((v.len() as f64 - 1.0) * p).round() as usize;
    v[i]
}

fn sig_hash(sig: &Signature) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    format!("{sig:?}").hash(&mut h);
    h.finish()
}


fn executed_indices(genome: &CreatureGenome, runtime: &RuntimeConfig, scen: &[Scenario]) -> Vec<usize> {
    let c = census(genome, runtime, scen);
    let mut idx: Vec<usize> = c.executed.iter().filter_map(|id| genome.nodes.iter().position(|n| n.node_id == *id)).collect();
    idx.sort_unstable();
    idx
}

/// Production births whose target draw is biased (1.0) toward `targets`
/// (the executed set), classified like `neighborhood::births`: silent =
/// identical signature, dead = NoOp on every execution, else changed.
fn births_with_targets(genome: &CreatureGenome, targets: &[usize], battery: &Battery, base: &Signature, config: &SimulationConfig, births: u32, seed: u64) -> (u32, u32, u32, u32, u32) {
    use v3_core::mutation::MutationEngine;
    let mut mcfg = config.mutation.clone();
    mcfg.reachable_bias.vm = 1.0; mcfg.reachable_bias.graph = 1.0; mcfg.reachable_bias.input_ref = 1.0; mcfg.reachable_bias.topology = 1.0;
    let food_types = config.world.food.types.len();
    let decay = config.shared_memory.decay_rate;
    let (mut zero, mut silent, mut changed, mut dead) = (0u32, 0u32, 0u32, 0u32);
    for b in 0..births {
        let mut rng = SmallRng::seed_from_u64(seed + b as u64);
        let mut child = genome.clone();
        let summary = MutationEngine::apply_mutations_with_food_type_count(&mut child, &mcfg, targets, &mut rng, food_types);
        let _ = &summary;
        if child == *genome { zero += 1; continue; }
        let sig = battery.signature(&child, &config.runtime, decay);
        if sig == *base { silent += 1; }
        else if sig.snapshots.iter().all(|a| a.len() == 1 && matches!(a[0], WorldAction::NoOp)) && sig.sequences.iter().flatten().all(|a| a.len() == 1 && matches!(a[0], WorldAction::NoOp)) { dead += 1; }
        else { changed += 1; }
    }
    (births, zero, silent, changed, dead)
}

#[test]
fn probe_live_census() {
    let dir = std::env::var("PETRI_GENOME_DIR").expect("PETRI_GENOME_DIR");
    let births_n: usize = std::env::var("PETRI_BIRTHS_N").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    let config = SimulationConfig::default();
    let runtime = config.runtime.clone();
    let food_types = config.world.food.types.len();
    let scen = scenarios(food_types, 48);
    let battery = Battery::generate(food_types);
    let ctx = EvalContext::from_config(&config);
    let decay = config.shared_memory.decay_rate;

    let founder = founder_genome(FounderProfile::V3Alpha1);
    let fsig = battery.signature(&founder, &runtime, decay);
    let fc = census(&founder, &runtime, &scen);
    let fdistinct: BTreeSet<String> = fc.outputs.iter().map(|o| format!("{o:?}")).collect();
    println!("FOUNDER executed={} distinct_outputs={} sig={:016x}", fc.executed.len(), fdistinct.len(), sig_hash(&fsig));

    let mut files: Vec<_> = std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "json")).collect();
    files.sort();
    println!("ROWHEADER id gen total reachable executed knockout route_varies_probe route_varies_battery hopcap_probe hopcap_battery chain_med chain_max exec_multi exec_gate_writer exec_mem_reader exec_mem_writer distinct_outputs noop_snapshots same_as_founder sig exec_ids");
    let started = std::time::Instant::now();
    for (i, path) in files.iter().enumerate() {
        let text = std::fs::read_to_string(path).unwrap();
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        let id = v["id"].as_u64().unwrap_or(0);
        let gen = v["generation"].as_u64().unwrap_or(0);
        let genome: CreatureGenome = serde_json::from_value(v["genome"].clone()).expect("genome deserializes");
        let c = census(&genome, &runtime, &scen);
        let m = battery.mesh_execution(&genome, &runtime, decay);
        let sig = battery.signature(&genome, &runtime, decay);
        let distinct: BTreeSet<String> = c.outputs.iter().map(|o| format!("{o:?}")).collect();
        let noops = c.outputs.iter().filter(|o| o.len() == 1 && matches!(o[0], WorldAction::NoOp)).count();
        let mut lens = c.chain_lens.clone();
        let exec_ids: Vec<String> = c.executed.iter().map(|n| n.0.to_string()).collect();
        println!(
            "ROW {id} {gen} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {:016x} {}",
            genome.nodes.len(), mesh_reachable_nodes(&genome).len(), c.executed.len(), m.knockout_count,
            route_varies(&c) as u8, m.route_varies_with_input as u8, c.max_hops_hits, m.hop_cap_hits,
            pct(&mut lens, 0.5), pct(&mut lens, 1.0), c.executed_multi_target, c.executed_gate_writer as u8,
            c.executed_memory_reader as u8, c.executed_memory_writer as u8, distinct.len(), noops, (sig == fsig) as u8,
            sig_hash(&sig), exec_ids.join(";")
        );
        if i < births_n {
            let arm = std::env::var("PETRI_ARM").unwrap_or_else(|_| "baseline".to_string());
            let mut mcfg = config.mutation.clone();
            match arm.as_str() {
                "bias1" => {
                    mcfg.reachable_bias.vm = 1.0;
                    mcfg.reachable_bias.graph = 1.0;
                    mcfg.reachable_bias.input_ref = 1.0;
                    mcfg.reachable_bias.topology = 1.0;
                }
                "scaled" => {
                    // Per-node supply: mean requested events = 0.55 * nodes / 2 (founder-equivalent per node).
                    let mean = (0.55 * genome.nodes.len() as f64 / 2.0).max(0.55);
                    mcfg.mutation_probability = mean.min(1.0);
                    let tail_mean = (mean / mcfg.mutation_probability).max(1.0);
                    mcfg.per_birth_mutation_event_continuation_probability = 1.0 - 1.0 / tail_mean;
                    mcfg.per_birth_mutation_events_max = 10_000;
                }
                _ => {}
            }
            if arm == "exec1" {
                let targets = executed_indices(&genome, &runtime, &scen);
                let (t, z, s_, c_, d_) = births_with_targets(&genome, &targets, &battery, &sig, &config, 200, 1_000_000 * (i as u64 + 1));
                println!("BIRTHS {id} total={t} zero={z} mutated={} silent={s_} changed={c_} dead={d_} one_n=0 one_silent=0 one_changed=0 one_dead=0", t - z);
                continue;
            }
            let eval = evaluate_genome(&genome, &battery, &mcfg, &ctx, 1, 200, 1_000_000 * (i as u64 + 1));
            let b = &eval.births;
            let a = &b.any_events;
            let one = b.by_events.get(&1).copied().unwrap_or_default();
            println!(
                "BIRTHS {id} total={} zero={} mutated={} silent={} changed={} dead={} one_n={} one_silent={} one_changed={} one_dead={}",
                b.births_total, b.zero_event_births, a.trials - a.skipped, a.silent, a.changed, a.dead,
                one.trials - one.skipped, one.silent, one.changed, one.dead
            );
        }
        for nid in &c.executed {
            if let Some(node) = genome.nodes.iter().find(|n| n.node_id == *nid) {
                if let BackendDef::Vm(vm) = &node.backend_def {
                    let mut live = BTreeSet::new();
                    for (idx, instr) in vm.program.iter().enumerate() {
                        if v3_core::creature::genome::analysis::vm_is_output_instruction(instr) {
                            if let Some(gene) = v3_core::creature::genome::analysis::vm_backward_slice(&vm.program, idx) {
                                live.extend(gene.indices);
                            }
                        }
                    }
                    println!("LIVE {id} node={} len={} live={}", nid.0, vm.program.len(), live.len());
                }
            }
        }
        if (i + 1) % 50 == 0 { eprintln!("{} genomes in {:.0}s", i + 1, started.elapsed().as_secs_f64()); }
    }
    println!("DONE {} genomes in {:.1}s", files.len(), started.elapsed().as_secs_f64());
}

/// Drift walk (no selection) to depth, then the per-birth neighborhood at each
/// checkpoint under three supply policies. Answers: could a cheap depth probe
/// have predicted the live population's 97% silent reading?
#[test]
fn probe_drift_depth_births() {
    use v3_core::config::MutationConfig;
    use v3_core::mutation::MutationEngine;
    const LINEAGES: u64 = 50;
    let config = SimulationConfig::default();
    let runtime = config.runtime.clone();
    let food_types = config.world.food.types.len();
    let battery = Battery::generate(food_types);
    let ctx = EvalContext::from_config(&config);
    let decay = config.shared_memory.decay_rate;
    let founder = founder_genome(FounderProfile::V3Alpha1);
    let arms = ["baseline", "bias1", "scaled"];
    let mut genomes: Vec<CreatureGenome> = (0..LINEAGES).map(|_| founder.clone()).collect();
    let mut rngs: Vec<SmallRng> = (0..LINEAGES).map(|l| SmallRng::seed_from_u64(90_000 + l)).collect();
    let mut walked = 0u64;
    for &checkpoint in &[0u64, 22, 250, 1000, 2000] {
        while walked < checkpoint {
            for (g, rng) in genomes.iter_mut().zip(rngs.iter_mut()) {
                let reach = mesh_reachable_nodes(g);
                MutationEngine::apply_mutations_with_food_type_count(g, &config.mutation, &reach, rng, food_types);
            }
            walked += 1;
        }
        let mut total = 0usize; let mut reach = 0usize; let mut exec = 0usize; let mut varies = 0usize;
        for g in &genomes {
            let m = battery.mesh_execution(g, &runtime, decay);
            total += m.total_node_count; reach += m.reachable_node_count; exec += m.executed_node_count; varies += m.route_varies_with_input as usize;
        }
        println!("DRIFT gen={checkpoint} lineages={LINEAGES} mean_total={:.2} mean_reachable={:.2} mean_executed={:.2} route_varies={varies}", total as f64 / LINEAGES as f64, reach as f64 / LINEAGES as f64, exec as f64 / LINEAGES as f64);
        for arm in arms {
            let (mut t, mut z, mut mu, mut s, mut c, mut d) = (0u32, 0u32, 0u32, 0u32, 0u32, 0u32);
            for (i, g) in genomes.iter().enumerate() {
                let mut mcfg: MutationConfig = config.mutation.clone();
                match arm {
                    "bias1" => { mcfg.reachable_bias.vm = 1.0; mcfg.reachable_bias.graph = 1.0; mcfg.reachable_bias.input_ref = 1.0; mcfg.reachable_bias.topology = 1.0; }
                    "scaled" => {
                        let mean = (0.55 * g.nodes.len() as f64 / 2.0).max(0.55);
                        mcfg.mutation_probability = mean.min(1.0);
                        let tail_mean = (mean / mcfg.mutation_probability).max(1.0);
                        mcfg.per_birth_mutation_event_continuation_probability = 1.0 - 1.0 / tail_mean;
                        mcfg.per_birth_mutation_events_max = 10_000;
                    }
                    _ => {}
                }
                let eval = evaluate_genome(g, &battery, &mcfg, &ctx, 1, 100, 7_000_000 + 1000 * (i as u64 + 1) + checkpoint);
                let b = &eval.births; let a = &b.any_events;
                t += b.births_total; z += b.zero_event_births; mu += a.trials - a.skipped; s += a.silent; c += a.changed; d += a.dead;
            }
            println!("DRIFTBIRTHS gen={checkpoint} arm={arm} births={t} zero={z} mutated={mu} silent={s} changed={c} dead={d} changed_per_birth={:.4} dead_per_birth={:.4}", c as f64 / t as f64, d as f64 / t as f64);
        }
    }
}

/// Drift walk under each supply policy (the policy shapes the walk itself),
/// with per-birth readings under the same policy at each checkpoint.
#[test]
fn probe_drift_policies() {
    use v3_core::config::MutationConfig;
    use v3_core::mutation::MutationEngine;
    const LINEAGES: usize = 50;
    const BIRTH_LINEAGES: usize = 20;
    let config = SimulationConfig::default();
    let runtime = config.runtime.clone();
    let food_types = config.world.food.types.len();
    let battery = Battery::generate(food_types);
    let ctx = EvalContext::from_config(&config);
    let decay = config.shared_memory.decay_rate;
    let founder = founder_genome(FounderProfile::V3Alpha1);
    let arm_config = |arm: &str, g: &CreatureGenome| -> MutationConfig {
        let mut mcfg = config.mutation.clone();
        if arm.contains("scaled") {
            let mean = (0.55 * g.nodes.len() as f64 / 2.0).max(0.55);
            mcfg.mutation_probability = mean.min(1.0);
            let tail_mean = (mean / mcfg.mutation_probability).max(1.0);
            mcfg.per_birth_mutation_event_continuation_probability = 1.0 - 1.0 / tail_mean;
            mcfg.per_birth_mutation_events_max = 10_000;
        }
        if arm.contains("pressure") { mcfg.genome_size_pressure_enabled = true; }
        if arm.contains("bias1") || arm == "exec1" { mcfg.reachable_bias.vm = 1.0; mcfg.reachable_bias.graph = 1.0; mcfg.reachable_bias.input_ref = 1.0; mcfg.reachable_bias.topology = 1.0; }
        mcfg
    };
    for arm in std::env::var("PETRI_ARMS").unwrap_or_else(|_| "baseline,scaled,pressure,scaled_pressure,bias1".into()).split(',') {
        let mut genomes: Vec<CreatureGenome> = (0..LINEAGES).map(|_| founder.clone()).collect();
        let mut rngs: Vec<SmallRng> = (0..LINEAGES).map(|l| SmallRng::seed_from_u64(95_000 + l as u64)).collect();
        let mut walked = 0u64;
        let mut exec_sets: Vec<Vec<usize>> = Vec::new();
        let scen = scenarios(food_types, 48);
        let started = std::time::Instant::now();
        'arm: for &checkpoint in &[250u64, 1000, 2000] {
            while walked < checkpoint {
                if arm == "exec1" && walked % 10 == 0 {
                    exec_sets = genomes.iter().map(|g| executed_indices(g, &runtime, &scen)).collect();
                }
                for (li, (g, rng)) in genomes.iter_mut().zip(rngs.iter_mut()).enumerate() {
                    let reach = if arm == "exec1" { exec_sets[li].clone() } else { mesh_reachable_nodes(g) };
                    let mcfg = arm_config(arm, g);
                    MutationEngine::apply_mutations_with_food_type_count(g, &mcfg, &reach, rng, food_types);
                }
                walked += 1;
                let mean_total = genomes.iter().map(|g| g.nodes.len()).sum::<usize>() as f64 / LINEAGES as f64;
                if walked % 50 == 0 && arm.contains("scaled") { println!("DRIFTGROW arm={arm} gen={walked} mean_total={mean_total:.1}"); }
                if mean_total > 400.0 {
                    println!("DRIFTPOL arm={arm} ABORT gen={walked} mean_total={mean_total:.1} ({:.0}s)", started.elapsed().as_secs_f64());
                    break 'arm;
                }
            }
            let (mut total, mut reach, mut exec, mut varies, mut gsize) = (0usize, 0usize, 0usize, 0usize, 0u64);
            for g in &genomes {
                let m = battery.mesh_execution(g, &runtime, decay);
                total += m.total_node_count; reach += m.reachable_node_count; exec += m.executed_node_count; varies += m.route_varies_with_input as usize; gsize += g.genome_size() as u64;
            }
            let (mut t, mut z, mut mu, mut s, mut c, mut d) = (0u32, 0u32, 0u32, 0u32, 0u32, 0u32);
            for (i, g) in genomes.iter().take(BIRTH_LINEAGES).enumerate() {
                if arm == "exec1" {
                    let targets = executed_indices(g, &runtime, &scen);
                    let base = battery.signature(g, &runtime, decay);
                    let (t_, z_, s_, c_, d_) = births_with_targets(g, &targets, &battery, &base, &config, 100, 8_000_000 + 1000 * (i as u64 + 1) + checkpoint);
                    t += t_; z += z_; mu += t_ - z_; s += s_; c += c_; d += d_;
                    continue;
                }
                let mcfg = arm_config(arm, g);
                let eval = evaluate_genome(g, &battery, &mcfg, &ctx, 1, 100, 8_000_000 + 1000 * (i as u64 + 1) + checkpoint);
                let b = &eval.births; let a = &b.any_events;
                t += b.births_total; z += b.zero_event_births; mu += a.trials - a.skipped; s += a.silent; c += a.changed; d += a.dead;
            }
            println!("DRIFTPOL arm={arm} gen={checkpoint} mean_total={:.2} mean_genome_size={:.1} mean_reachable={:.2} mean_executed={:.2} route_varies={varies}/{LINEAGES} births={t} zero={z} mutated={mu} silent={s} changed={c} dead={d} changed_per_birth={:.4} dead_per_birth={:.4} ({:.0}s)",
                total as f64 / LINEAGES as f64, gsize as f64 / LINEAGES as f64, reach as f64 / LINEAGES as f64, exec as f64 / LINEAGES as f64, c as f64 / t.max(1) as f64, d as f64 / t.max(1) as f64, started.elapsed().as_secs_f64());
        }
    }
}
```
