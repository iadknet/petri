# Why complex behavior is not evolving: an adversarial diagnosis and a quick-experiment plan

Date: 2026-10-05. Source read at `main` `18c796ae` (run 6 still in flight in its own
worktree; nothing here touches it). Status: research note and experiment plan under the
[T22 exploration contract](../roadmaps/t22-capability-assays-and-evolvability-lab.md);
no production source, default, founder or roadmap row changed. Primary texts read in
full by four research readers on 2026-10-05 (Sources). Reviewed by a Fable advisor
before and after writing; not yet reviewed by Codex.

## Answer first

1. **The six exploration runs could not have found the answer, whatever it is.** They
   measured reach under truncation selection in populations of 16 to 32 with 4 survivors
   and 4 scenes per generation, for 100 to 1,000 generations, about 3 × 10⁴ births per
   replicate. Wagner 2008 shows the neutral-drift benefit a small population gets
   appears only over about 10⁴ generations and only when intermediates are strictly
   neutral; Petri's intermediates are not (Section 3.2). Bejjani et al. 2025's own
   control shows the same compute split into small worlds fails where one large world
   succeeds. Lenski et al. 2003's failing runs tested 2 × 10⁷ genotypes and still scored
   0 of 50. A null from the 32-individual lab carries no information about the
   1600² world with 60,000 creatures and 3 million births an hour. Five outcome-B and
   outcome-C verdicts are verdicts on the instrument.
2. **The one number that matters was never measured.** The production worlds reward a
   finished hand-wired gradient follower (T20.F01: about 1.4 × the founder's births in
   Canyon and Confluence, inconclusive in Orchards). Lenski 2003 (p. 143) is explicit
   that what decides whether a complex feature evolves is whether the *first step* pays:
   with only the finished function rewarded, 0 of 50 populations evolved it although
   they had more mutational throughput than the ones that did. Nobody has measured the
   birth ratio of a founder carrying *one* new sensor-to-motor edge. If that is 1.0 or
   below, Petri is in the EQU-only regime and no operator change can help; the world
   has to change first.
3. **The premise is unmeasured on the current substrate.** The live survey that found
   "nothing a third node could do is rewarded" read generation 410 on 2026-09-16, before
   T11.F21, T11.F22, T19 and T20. Every reading since is at tick 2,000 (generation
   about 22, inside the founder boom-and-crash). No long production run has been
   censused with the input-use instrument T20.F01 built. Experiment E2 does that
   first.
4. **Where the framework is wrong, it is wrong in four identifiable places**, each with
   a natural analog it violates (Section 3): intermediates are pruned by designed
   cleanup operators about 10⁴ times faster than they are completed; function needs a
   sign-consistent *field* of edges but variation supplies one random edge at a time;
   the minimal criterion for reproduction is satisfiable blind, so sensing is never the
   floor; and mutation supply plus copy operators have a positive feedback that costs do
   not bound.
5. **Next action:** run E1 (one-edge birth ratio at production scale, about an hour) and
   E2 (a 50,000-tick production census, about 90 minutes per world) before any
   mechanism prototype. They decide between "the world must change" and "the encoding
   must change". Everything else in the ledger is ordered behind them.

## 1. What the six runs established, read adversarially

| Run | What it claims | What it actually shows | What it could not show |
| --- | --- | --- | --- |
| 1 | Elites plateau; no arm moves reach | A lab artifact: the plateau was 92 % sterility shortcut (run 2, P1), because the lab refuses reproduction and charges the refused attempt. Four of the six cycles measured that artifact | Anything about mechanisms: every arm ran on the confounded instrument |
| 2 | Outcome C, no admissible repair | Correct and valuable: a solo fixed-lifetime scorer cannot be made indifferent to reproduction. It also shows the lab's scoring boundary is the wrong one for an organism whose fitness *is* reproduction | A mechanism reading; nothing ran |
| 3 | Outcome C by exhaustion; census says selection re-wires the incumbent's reads; witness path 13 edits, 9 neutral | The census and witness path are the run's real findings. The six arms at 16 pairs could neither show nor exclude anything (its own words) | A reach effect under 0.25 |
| 4 | Outcome B: no arm raises reach by ≥ 0.25 over an A/A control at 48 pairs | Base-rate reach is 0 on fresh seeds, so the bound excludes only a large rise. The A/A control departs the wall rung in 56 % of pairs: the instrument's own noise is as large as the effects sought | Small effects; anything at ecological scale |
| 5 | Outcome B: heavy-tailed jumps are not twice as helpful | Sound, and the twin screen is the one instrument from these runs worth keeping. Its other finding is the important one: M4 gives 3 × more confirmed-helpful one-step children and campaigns saw nothing, which is exactly what Wagner 2008 predicts when the limit is retention of neutral spread, not supply | Why one-step gains do not accumulate |
| 6 (in flight) | 1,000-generation horizons; silent-afferent start | M4 ran away to 291,145 units and the unselected control to 535,000: a real supply-feedback defect (Section 3.4), amplified by a lab with no replication cost that carries dead elites | Its ALT reading is the closest existing test of "born connected"; read it when it lands |

The frontier tables are honest. The problem is upstream of them: the question "which
operator change lets a 32-individual truncation GA reach a calibrated threshold in 100
generations" was never the question the program needs answered.

## 2. Local evidence the diagnosis rests on

- **Production throughput** ([T10.F09 sweeps](../progress/sweeps/t10-f09/)): the 1600²
  default world runs 9.4 to 11.7 ticks/s on 8 threads, 3.1 to 3.3 million births per
  hour. The lab campaign of run 4 ran 48 pairs × 32 × 100 generations in about 9 hours
  for roughly 1.8 million births, under truncation.
- **Ecological demand exists but was measured only for the finished circuit**
  ([T20.F01 readings](../progress/readings/t20-f01.md)): `A_vector` (gated
  `AreaFoodSummary` follower, 4 compute nodes, 12 edges) positive in Canyon (pooled A/Z
  1.42, 8 of 8 above one) and Confluence (1.12, 6 of 8), inconclusive in Orchards;
  `A_scalar` (eat fruit) positive in Orchards (16.6) and Confluence; `A_ring` (barrier
  avoid) not positive anywhere.
- **The economy at tick 2,000 is a crowded equilibrium**
  ([goal summary](../progress/features/t20-f01-input-use-baseline-and-ecological-opportunity-goal.json),
  `energy_flows`, `mortality`, `reproductive_success_by_cognitive_class`): in Orchards
  food intake is 0.91 energy per creature-tick against 0.50 lifecycle decay and 0.36 in
  move charges; 59 % of deaths are decay and 37 % move charges; a creature lives about
  57 ticks and leaves 0.97 offspring. In the user's long-running world (live survey,
  tick 28,315) 85.8 % of reproduction attempts were rejected, 56.5 million of them for
  an occupied cell. At that equilibrium the marginal value of finding food is capped by
  space, not energy.
- **Retention of a causal channel across one native birth is high** (T20.F01
  `retained_share`: 0.95 to 0.98 in the selected cohorts, 1.0 in drift). The loss is not
  in copying a working circuit; it is in building one.
- **Discovery baseline** (T20.F01, 10,000 proposals per step on the founder): vector
  family declared on the vote node 4 in 10,000, connected 1 in 10,000, implied births
  25,773 for the declare-then-connect pair of *any* channel to *any* surface.
- **The genome bloats under weak selection** (run 6 memory; [T11.F16
  drift walk](../progress/features/t11-f20-per-birth-supply-rule-retirement-goal.json)):
  behavior-changing births fall from 15 % at depth 0 to 0.25 % at depth 2,000 as junk
  dilutes the executed core; live-survey median genome 475 units at generation 410
  against the founder's 97.

### 2.1 The needle, in the engine's own numbers

Per-birth odds of the smallest useful afferent on the founder (one declaration of
`AreaFoodSummary` on the vote node, then one `AddGraphEdge` from a gradient or nearest
sub-value to a cardinal `Move` sink with the right sign), from the operator weights in
`mutation/engine/mod.rs`, `mutation/input_ref/mod.rs` and `mutation/graph/mod.rs` at
the production defaults (mesh layer 0.2; node-internal domains equal; InputRef weights
Add 2, Prune 2, Swap 4, RawField 4; Graph `AddGraphEdge` 2 of 44, `RemoveGraphEdge` 2,
`AlterGraphEdgeWeight` 4; `random_graph_source` draws an input leaf with probability
0.3, uniform over references and sub-values; executed bias puts the event on node 0 or
1). Estimates, not measurements; E4 measures them.

| Step | Per birth | Note |
| --- | ---: | --- |
| Declare the family on the vote node | 3.2 × 10⁻⁴ | T20.F01 measured 4 per 10,000 proposals on the founder: the same event |
| Prune that declaration while it is unconnected | 8.6 × 10⁻³ | `Prune` is 1 of 6 InputRef events and the new entry is the only prunable one; half-life about 80 births |
| Connect any channel of the family to any surface, given declared | 8.8 × 10⁻⁵ | T20.F01 measured 1 per 10,000 for an authored channel on any surface; this row and the next differ by what counts as a connection, not by a disagreement |
| Connect a useful sub-value to a cardinal Move sink with the right sign, given declared | 4.8 × 10⁻⁷ | 4 of 104 surfaces, 2 of 7 sub-values, 1 of 8 references, sign 1 of 2 |
| Connect before prune | 5.6 × 10⁻⁵ | |
| Useful one-edge afferent, population rate | 1.8 × 10⁻⁸ per birth | one every 18 hours at 3.1 million births per hour, with a random magnitude in [−1, 1] |
| A fresh edge is removed vs re-weighted | 2.3 × 10⁻³ vs 4.7 × 10⁻³ | removal runs at half the rate of refinement |

Two things in this table are design choices with no natural analog at these rates. A
receptor expressed without a downstream partner is neutral in biology and is lost at a
mutation rate per site of about 10⁻⁸; Petri removes it at 10⁻² per birth. An edge with
the wrong weight is in biology tuned by activity or drift; Petri deletes it at half the
rate it adjusts it. These are the cleanup operators of a programmer, and they shrink the
neutral network (Wagner 2008, Section 2c: what buys access to new phenotypes is the
*size* of the neutral set, not per-genotype openness).

## 3. Diagnosis: four defects, ranked, each with its natural analog

### 3.1 The minimal criterion is satisfiable blind (world)

Soros and Stanley 2014 argue (Condition 1, p. 2) that open-ended systems need a
nontrivial minimal criterion for reproduction that involves interacting with the world,
and the seed must already meet it. Petri's founder meets its criterion (energy ≥ 30 and
age ≥ 20) by walking and eating what is under it, in worlds that start at 65 % food
coverage, boom to the 100,000 cap in 100 ticks and settle into a crowded equilibrium
where reproduction fails for lack of space. Sensing is the ceiling, never the floor.

Two readings in Section 2 pull against each other and the tension is the point: 59 %
of deaths are starvation, yet 85.8 % of reproduction attempts fail for want of a cell.
The reconciliation this note proposes, and E6 tests, is that at the crowded equilibrium
starvation is competition for grazed-out ground: a creature inside a meadow sees food
everywhere and one on bare ground sees none, so the food gradient carries information
only at patch edges, and a better sense of where food is buys little that a free cell
to breed into would not buy more. If E6 finds the gradient informative for most living
creatures, this paragraph is wrong and the world is not the block.

The 1.4 × advantage of the finished follower is the right sign and the wrong step: Lenski
2003 (p. 143) found a complex feature never evolved when its simpler steps were not
rewarded, with more throughput than the successful runs. Bejjani 2025 evolved foraging
only where a sensor (compass) changed the payoff of a single motor action from the
first mutation, and only at 256² and above.

Natural analog Petri is missing: resources that cannot be found by a random walk at the
density the population lives at (patchy food with spacing beyond the founder's ring,
regrowth that moves, a cost of blind movement above its expected return), and
organism-made opportunity (grazing fronts, deposited bodies, nests) so that the
criterion keeps moving (Condition 2).

A second world-side lever, from the sibling
[recent-research scan](alife-recent-research-scan-2026-10-05.md) written the same day:
the world is also static. Kumawat et al. 2025 found in Avida that a task environment
switching every about 30 generations evolved neighborhoods with a median 45,802
alternate-phenotype single and double mutants against 1 in a constant environment, and
Ikeda, Kaneko and Hatakeyama 2026 show from an exhaustive genotype-phenotype map why: a
fixed environment selects robust interior genotypes, away from the boundary where
alternative phenotypes are one mutation away. Petri's goal worlds are fixed maps, both
lab instruments draw every generation from one layout distribution, and T02.F01 Seasons
is unbuilt. The two levers are not the same: sparsity makes the first sensor edge pay;
fluctuation makes lineages sit where a second rule is reachable. E11 tests the second.

### 3.2 Intermediates are not neutral, and they are pruned by design (variation)

Section 2.1. The declare-then-connect path is two rare events whose first state has a
half-life of 80 births against a completion time of 10⁶ births. Wagner 2008 Section 2e:
at low N·μ a population is monomorphic and walks its neutral network slowly; at high
N·μ it settles in the robust interior of the network, away from the edge where new
phenotypes sit. Either way the quantity that governs reaching a specific rare phenotype
is its frequency in genotype space (Section 2f), which per-synapse encoding makes tiny.
Hinton and Nowlan 1987 describe the same shape: a plateau with a needle, where
recombination and mutation have nothing to climb.

Natural analog: silent receptors and silent synapses persist; nothing in a cell
removes an unused gene at 1 % per generation.

### 3.3 Function needs a sign-consistent field; variation supplies one random edge (encoding)

The witness path (run 3, C2) is 13 edits with 9 neutral because a steering circuit is a
*pattern* (every direction's sensor to that direction's motor, same sign) and Petri's
operators draw source, sink, sub-value and sign independently. Gaier and Ha 2019 (p. 6,
Table 1) found random per-edge weights fail outright while a topology with one shared
weight carries function: what matters is consistency of sign across the field, not the
individual magnitudes. Stanley et al. 2009 (Section 7.2) express the whole
sensor-to-matching-effector map as one Gaussian-of-distance motif. Kirschner and
Gerhart 2007 call the biological version weak regulatory linkage: the output is
pre-built in the core process, and a signal only hooks into it (their example is adding
a receptor to a neuron without rewiring its output machinery). The ring experiments of
2026-09-23 already found coordinated variation helps substantially and a dense zero
matrix alone does nothing.

Natural analog: topographic maps formed by developmental gradients (retinotopy,
somatotopy): one developmental rule aligns a sensory sheet with a motor sheet; sign and
gain are then the evolvable genes. The lab contract's not-forcing check 3 forbids
"pairing a source with the sink that solves a scene"; an index-aligned projection over
every 8-wide family and every 8-wide vote group pairs index with index, never food with
move, and its sign is unset. Both readings are recorded in the options table for the
user's ruling.

### 3.4 Supply and copy operators have a positive feedback that costs do not bound (supply)

Per-unit supply (0.005 per unit per birth) times copy operators that add units gives
growth proportional to size. In the lab, with no replication cost and dead elites
carried, that is runaway (run 6). In production the replication surcharge (0.1 per unit
above founder on a 0.1 reproduce charge) and carry cost (10⁻⁴ per unit per tick, 0.006
energy over a 57-tick life at founder size) are both far below the energy scale of a
birth (about 40 energy transferred), so growth is bounded only by dilution of the
executed core: the drift walk's 15 % to 0.25 % behavior-changing births by depth 2,000
is the mutational-load signature. Avida bounds genome size because copying a longer
genome takes proportionally longer (Lenski 2003, Methods): size costs reproduction
*time*, not a token charge.

Natural analog: replication time and the metabolic cost of neural tissue.

### 3.5 What is not a defect

Lifetime plasticity as a fix is not licensed by the sources until an outcome signal
exists. Hinton and Nowlan's effect needs an organism that can recognise success within
its life (footnote 2); Najarro and Risi 2020 got robustness, not faster search (App.
6.2 shows parity), and pure-correlation Hebbian (their A-only ablation) fails. Petri's
Hebbian rules are correlation rules and reward modulation defaults off. The
action-credit features T20.F06 and T20.F07 are the right shape and are gated on
lab evidence the lab cannot produce (T20.F13 depends on T22 reach), which is circular.
That gate was the user's decision on 2026-09-23; this note proposes re-basing it on
E2-style production readings rather than removing it.

Mesh routing, the vote surface, VM versus Graph, cross-node edges and weight jump laws
were all tested and none is implicated. H8 behaved as the prior art predicted. The
T19 refactor was predicted neutral on discovery by its own review and was.

## 4. What the primary texts license and do not

| Source | Licenses for Petri | Does not license | Does nothing for Petri when |
| --- | --- | --- | --- |
| Lenski, Ofria, Pennock, Adami 2003 | The first step toward a function must pay; throughput cannot substitute for a gradient | Any particular stepping stone being necessary (34 % of one- or two-reward-removed environments still evolved EQU) | The one-edge mutant has no birth advantage |
| Soros and Stanley 2014 | A reproduction rule that cannot be met blind is a necessary condition; organisms must make new opportunities for each other | That novelty arises without a seed found by a separate search; only Condition 2 was tested | Reproduction stays satisfiable by walking and eating |
| Bejjani et al. 2025 | A small lab is a different dynamical system from a large world (F.3 ablation at fixed compute); a sensor whose single input changes one action's payoff is what evolves first | Transfer of its Gaussian all-weight mutation to a discrete 0.5-event-per-birth graph | The world is below the scale at which policy diversity persists |
| Wagner 2008 | Evolvability is a property of the neutral network's size and the population spread over it; per-genotype probes measure the wrong thing | That a truncation GA traverses neutral steps; that robustness reaches a specific target (Section 2f) | Intermediates carry a cost or the horizon is under 10⁴ generations at N_e about 4 |
| Hinton and Nowlan 1987 | One gene may influence many connections (p. 500); a lifetime search converts a needle to a zone | Learning speeding search in general; assimilation without a learning cost | No in-life signal distinguishes a right from a wrong weight |
| Najarro and Risi 2020 | A field born with random weights reaches function in tens of steps if every synapse carries an inherited working rule | Faster evolutionary search; pure Hebbian co-activity | Rule coefficients are themselves random at birth, or lifetimes are under about 50 ticks |
| Gaier and Ha 2019 | Topology plus activation with one shared weight carries sensorimotor function; sign consistency is what the field needs | Random per-edge weights; one weight draw per lifetime without robustness averaging | Each edge keeps an independent sign gene |
| Kirschner and Gerhart 2007 | A projection as one regulated unit with a pre-built output and a one-switch hookup; exploratory growth stabilized by a target-emitted signal | Building a new core process by regulation; activity alone as the stabilizer | The regulatory switch is itself several independent genes |
| Stanley, D'Ambrosio, Gauci 2009 | An index-aligned sensor-to-effector map as one motif that generalizes across directions | Beating direct encoding on a small sensorimotor task (no such control in the paper); irregular exceptions (Clune et al. 2011, HybrID) | The 8-direction map is the only regularity creatures ever need |
| Taylor et al. 2016 | The consensus necessary conditions are population, duration, search-space size, weak selection and organism-made pressure; operator design is not on the list | | |

## 5. Options considered

| Option | Verdict | Why |
| --- | --- | --- |
| Continue the exploration series (run 7 of the same shape: more arms, more pairs, longer horizons) | **Rejected** | Section 1. The instrument cannot see the effect class; Bejjani F.3 and Wagner 2e say why |
| Adopt a GA framework or another ALife platform (JaxLife, Neural MMO, Avida) | Rejected (unchanged from 2026-09-03 and 2026-09-28) | None has an evolvable graph topology; the reasons to borrow methods not platforms stand |
| Make the production world the instrument of record for mechanism questions | **Adopted; this reverses part of the 2026-09-28 decision** that made the lab the framework of record for capability, discovery and reachability questions in every track | The only scale at which the question is well posed; the T20.F01 input-use census, the server creature dump and `v3-cli run` samples already exist. The lab stays the framework of record for scale-independent readings (calibration, enumeration, twin screens); reach and retention verdicts move to production scale |
| Keep the lab for calibration, enumeration and twin screens only | Adopted | Run 5's screen and run 3's census are valid at any scale; reach campaigns are not |
| Change the world so the minimal criterion needs sensing (experiments first, then a production world under the shared baseline contract) | **Adopted as the first lever**, gated on E1 and E6 | Soros Condition 1; Lenski's first-step gradient; natural analogs exist |
| Retire or slow the cleanup operators (`Prune` at 1 of 6 InputRef events; `RemoveGraphEdge` at half the refinement rate) | Adopted as a candidate, gated on E4 | Section 2.1; Wagner 2c. Analog: unused genes are lost at mutation rates, not by a janitor |
| Projection as one unit of variation: one event wires a family to a vote kind with a shared sign and gain, index-aligned for 8-wide families, per-edge exceptions evolvable afterwards | Adopted as a candidate, gated on E5; **the check-3 ruling is the user's** | Gaier and Ha, Kirschner and Gerhart, Stanley 2009 Section 7.2, the ring experiments; HybrID says keep per-edge exceptions |
| A world that changes: food rules that are periodically wrong (seasons, moving regrowth), so selection favours genotypes one step from the alternative rule | Adopted as a candidate beside the static-sparsity world, gated on E11 | The sibling [recent-research scan](alife-recent-research-scan-2026-10-05.md) of the same day: Kumawat et al. 2025 (Avida, switching every about 30 generations gave a median 45,802 alternate-phenotype mutants against 1 in a constant environment) and Ikeda, Kaneko and Hatakeyama 2026 (static environments select interior, robust genotypes away from the boundary where new phenotypes sit). Petri's worlds and both lab instruments are stationary; T02.F01 is unbuilt. Analog: seasons |
| Dense exploratory afferents at birth, pruned by use (the run 6 ALT direction) | Deferred to run 6's reading | Kirschner and Gerhart: exploratory processes need a target-side stabilizer; activity alone keeps whatever is used |
| Lifetime learning with outcome credit (T20.F06, F07) | Candidate behind the projection unit, not before it | Section 3.5: licensed only with an outcome signal and an inherited working rule |
| Replication cost as time (a birth takes ticks proportional to genome size) in place of the token surcharge | Candidate, gated on E8 | Lenski 2003 Methods; bounds the Section 3.4 feedback where it operates |
| Topographic projection with food-to-move hard-wired | Rejected | Pre-bakes the behavior: fails the natural-analog rule and the evolvability-not-abilities rule |

## 6. Quick experiments

Ordered by what each decides; each has a wall budget, a prediction and a falsifier.
E1, E2 and E6 need no production code. Prototypes stay on an experiment branch as
default-off `proto:` commits, never merged. Nothing seeds a lineage with an authored
solution except the explicitly labelled diagnostic arms (E1, E7), which measure the
world, not discovery.

| ID | Question it decides | Method and instrument | Wall | Prediction | Falsifier / decision |
| --- | --- | --- | --- | --- | --- |
| **E1** | Does the first step pay? (Lenski) | `v3-cli input-opportunity` machinery: diagnostic arms seeded in the three goal worlds beside the founder, 8 replicates, 1,000-tick horizon as T20.F01: founder + one `AreaFoodSummary` gradient edge to one cardinal Move at +0.5 and at −0.5; founder + one barrier ring edge to its Move at −0.5; founder + the same edges at a random sink (pairing control); zero-weight twins. Read A/Z and A/F birth ratios | about 1 h (T20.F01's full run took 1,692 s at 3 threads) | One correctly paired edge gives 1.0 to 1.1; a mispaired edge below 1.0 | If no one-edge arm beats 1.05 where `A_vector` gave 1.4, the world is EQU-only: do E6 and E7 before any encoding work. If a one-edge arm pays, the block is variation (Sections 3.2, 3.3): do E4, E5 |
| **E2** | Is the premise true on the current substrate? Where does the funnel stop over time? | An ignored in-process probe, one per positive-demand world (Canyon, Confluence) at 1600²: run `Simulation` to ticks 10k, 20k and 50k and at each checkpoint call the T20.F01 funnel exactly as the goal bench does (`funnel::observe` in `neighborhood/input_use/mod.rs` takes the live simulation and the T11.F26 cohorts; the bench caller in `v3-cli/src/bench/input_use.rs` is the template, about 100 lines of adapter), writing the family rows and the declared → connected → executed → causal → retained stops per checkpoint. The server's creature dump is the fallback for looking, not for the reading | about 90 min per world to 50k ticks, plus the census at each checkpoint (the goal bench's census runs inside its 11-minute budget at 1600²) | Declared and connected `AreaFoodSummary` and barrier reads appear; causal use stays rare; behavior programs stay fixed | If causal use rises and is retained by 50k ticks, nothing is broken but patience: the program's horizon, not its mechanism, is wrong. If reads connect but never become causal, Section 3.3 is confirmed |
| **E3** | Are intermediates neutral in the production economy? | Analytic plus the run 5 twin screen: selection coefficient of (a) a declared unconnected reference, (b) a zero-weight edge, (c) a wrong-sign edge, from carry cost, perception assembly cost and the measured per-birth removal rates; twin-screen harmful share for each | minutes to 1 h | (a) and (b) are neutral to 10⁻³; (c) is harmful at 10 to 20 % | If (a) or (b) carry a measurable cost, that cost is the first repair |
| **E4** | What are the real transition rates? | A lab probe: 10⁶ founder births through the production engine, counting every declare, prune, connect, remove and re-weight transition on the vote node; the Section 2.1 table measured, not estimated; repeated on three run 4 elites | seconds to minutes | Within a factor 3 of Section 2.1; prune-to-connect ratio above 10³ | If the ratio is under 10, Section 3.2 is wrong and cleanup is not the block |
| **E5** | Does a projection unit change the helpful rate per birth? | `proto:` operator `AddProjection` (default off): one event declares a family if absent and wires every channel to every sink of one vote kind; three variants: random per-edge weights, one shared weight with random sign, index-aligned (channel *i* to sink *i*) with one shared weight and sign. Measured by run 5's twin screen (confirmed-helpful and harmful shares per fired birth, A/A check) on the founder and run 4 elites, both assays | 1 to 3 h | Random per-edge weights no better than native (Gaier and Ha Table 1); index-aligned shared sign several times more often helpful and more often harmful | If index-aligned is not above native, the encoding is not the block and the projection candidate is dropped. If it is, the check-3 question goes to the user with numbers |
| **E6** | Does the equilibrium world carry a usable gradient? | From an E2 snapshot at 20k ticks: for every living creature, `AreaFoodSummary` nearest distance, gradient magnitude and the expected energy of one step along the gradient against a random step; the share of creatures for which sensing changes the best move | minutes | Under 20 % of creatures at equilibrium have an informative gradient; most sit inside a meadow or on bare ground | If the share is high, the world is not the block and E7 is skipped |
| **E7** | Can a world make sensing the floor without naming it? | Experiment worlds (not production): patch spacing beyond ring reach, regrowth that moves, move cost above a blind walker's expected return, offspring placed within a radius so space is not the binding constraint. Three reads at 800² (persistence first): founder alone; founder with 1 % `A_vector`; founder alone for 50k ticks with the E2 census. Analogs: patchy resources, seasonal depletion | 30 min per read at 800² (throughput about 4 × the 1600² rate) | Founder alone goes extinct or persists at low density; followers persist and sweep; founder alone evolves causal food reads where the default world did not | If followers do not sweep, demand is still too weak at that geometry; iterate the world, not the brain |
| **E8** | Is the size feedback bounded in production, and by what? | `v3-cli run` 20k ticks (run 6 part P's command) on Canyon at default and at ×4 supply, plus a `proto:` arm where the reproduce action takes `1 + size / founder` ticks; read genome size, births per creature-tick, persistence | 2 h | Default bounded near 5 × founder by dilution; ×4 grows; time-cost arm holds near founder | If the default already runs away past 10 × by 20k ticks, Section 3.4 moves ahead of 3.3 |
| **E9** | Can the lab reproduce a known production positive? | The one evolved rule the live survey found (fruit-W to move W, swept the population) as a reach target in the lab from the founder at pop 32 × 100 generations | 1 h | The lab does not find it | If it does not, no lab negative is admissible as evidence about production; written into the T22 contract |
| **E10** | Run 6's ALT | Read when it lands | — | No detectable excess at 1,000 generations (Kirschner and Gerhart: no stabilizer) | A positive would reorder E5 behind a dense-afferent variant |
| **E11** | Does a changing world widen the useful neighborhood? (Kumawat, Ikeda) | The sibling scan's instrument: `food-seeking-hunger` with two mirror-image frozen layouts A and B switched every k generations (k in 3, 30, 300), a random-layout arm as the predictability control, the stationary V2 as the constant-environment reference. Read with the twin screen, not reach: the share of one-step children of each arm's elites that are confirmed-helpful on the *other* layout (the alternate-neighborhood analog); reach and lag after a switch descriptive only | 3 h | The switching arm's elites have several times the alternate-layout helpful share of the stationary arm's | A positive is a lead for T02.F01's priority and for a non-stationary production world, never a stepped season; a supported negative closes the fluctuation lever for this instrument class |

Order: E1, E2 and E4 run first and in parallel on separate cores (E4 is seconds). E6
reads E2's snapshot. E3 and E5 follow if E1 says variation is the block; E7 follows
if E1 says the world is; E11 runs on either branch because it reads neighborhoods, not
reach. E8 and E9 run whenever a core is free. Every row writes a
compact summary beside this note under `evolvability-diagnosis-2026-10-05/`; raw
output stays under `.bench-artifacts/`.

## 7. Provisional solution direction, gated on E1 and E2

Stated now so the experiments have a target to falsify. Nothing here is a feature until
a row above says so.

1. **World first** (3.1). A production world under the shared baseline contract in
   which a blind walker's expected intake is below its decay at equilibrium and food
   returns somewhere else than where it was eaten, so the minimal criterion needs
   sensing and keeps moving. Through the world only: no reward, no sensor flag.
2. **Stop deleting the neutral network** (3.2). Retire `Prune` and `RemoveGraphEdge` as
   weighted operators, or weight them at the rate an unused gene decays, and let carry
   cost be the only pressure against silent structure.
3. **A projection is one gene** (3.3). One event declares a family and wires it to a
   vote kind with a shared sign and gain, index-aligned where both sides are 8-wide,
   with every edge afterwards an ordinary refinable exception. The index alignment is
   the contested part; the user rules on it with E5's numbers.
4. **Size costs time** (3.4). Replication time proportional to genome size, read in
   production.
5. **Learning last, and only with credit** (3.5). T20.F06 action credit, with the rule
   template inherited working, after 1 to 4 have been read.
6. **The lab measures mechanisms, not reach.** T22's reach ladder stays for calibration;
   mechanism verdicts come from twin screens, enumeration and production censuses.

## 8. Rules for these experiments

- The not-forcing checks of the run 1 plan apply to every mechanism arm; diagnostic
  arms (E1, E7, E9) are labelled and never recommended. Experiment worlds are
  experiments; a production world change goes through T12 and the shared baseline
  contract.
- No production default, founder or mutation policy changes on `main` from this note.
- Each row is predeclared in its summary before it runs; `incomplete` rows are kept.
- Codex reviews this note before any row runs and after E1, E2 and E4 are read.

## 9. Remaining uncertainty

- Section 2.1 is arithmetic from the operator weights, not a measurement (E4).
- The 1.4 × demand figure is at generation 22 in the boom; it may be larger or smaller
  at equilibrium (E2, E6).
- E7 may find no 800² geometry that persists; persistence below 1600² has been
  seed-dependent since T01.F11.
- The projection unit may help the two assays and nothing else; HybrID's lesson is that
  regular encodings lose on irregular tasks, and ecology rewards exceptions.
- Run 6 may contradict Section 3.3 if ALT reaches; this note then reorders E5.

## Sources

Read in full on 2026-10-05 (local extracts in the session scratchpad):

- Hinton, G. E. and Nowlan, S. J. 1987. How learning can guide evolution. *Complex
  Systems* 1:495–502. https://content.wolfram.com/sites/13/2018/02/01-3-6.pdf
- Najarro, E. and Risi, S. 2020. Meta-learning through Hebbian plasticity in random
  networks. NeurIPS 2020. https://arxiv.org/pdf/2007.02686 (v5)
- Gaier, A. and Ha, D. 2019. Weight agnostic neural networks. NeurIPS 2019.
  https://arxiv.org/pdf/1906.04358 (v2)
- Kirschner, M. W. and Gerhart, J. C. 2007. The theory of facilitated variation. *PNAS*
  104 (suppl. 1):8582–8589. https://pmc.ncbi.nlm.nih.gov/articles/PMC1876433/
- Wagner, A. 2008. Robustness and evolvability: a paradox resolved. *Proc. R. Soc. B*
  275:91–100. https://www.ncbi.nlm.nih.gov/pmc/articles/PMC2562401/
- Stanley, K. O., D'Ambrosio, D. B. and Gauci, J. 2009. A hypercube-based encoding for
  evolving large-scale neural networks. *Artificial Life* 15(2):185–212 (authors'
  manuscript). https://web.archive.org/web/2015id_/http://eplex.cs.ucf.edu/papers/stanley_alife09.pdf
- Lenski, R. E., Ofria, C., Pennock, R. T. and Adami, C. 2003. The evolutionary origin
  of complex features. *Nature* 423:139–144.
  https://directory.natsci.msu.edu/media/Directory/Profiles/2003,%20Natu20260121115450.pdf
- Soros, L. B. and Stanley, K. O. 2014. Identifying necessary conditions for open-ended
  evolution through the artificial life world of Chromaria. ALIFE 14.
  https://web.archive.org/web/2016id_/http://eplex.cs.ucf.edu/papers/soros_alife14.pdf
- Bejjani, N. et al. 2025. The emergence of complex behavior in large-scale ecological
  environments. arXiv:2510.18221v3. https://arxiv.org/pdf/2510.18221v3
- Taylor, T. et al. 2016. Open-ended evolution: perspectives from the OEE workshop in
  York. *Artificial Life* 22:408–423. https://publications.aston.ac.uk/id/eprint/44173/1/artl_a_00210.pdf

Recalled, not re-read this session: Clune, Stanley, Pennock and Ofria 2011 (IEEE TEC,
regularity continuum); Clune et al. 2009 (HybrID); Mayley 1996 (cost of learning);
Pedersen and Risi 2021 (merged Hebbian rules).

Local notes this builds on: the five exploration results notes and the run 6 plan,
[T20.F01 readings](../progress/readings/t20-f01.md), [live
survey](live-survey-2026-09-16.md), [ring experiments](ring-group-experiments-2026-09-23.md),
[mesh action-selection review](mesh-action-selection-review-2026-09-20.md) Section 1.1,
[capability assay research](capability-assay-research-2026-09-28.md).
