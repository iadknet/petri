# Petri Brain Evolvability Audit

**Status**: Reference (companion to the [T11 track](../roadmaps/t11-brain-genotype-phenotype-map.md); not executable roadmap guidance)
**Date**: 2026-09-04
**Code**: `main` at commit 4b8650ef, V3Alpha1 founder, production defaults
**Method**: code reading, two in-process probes (400 trials per operator, 6,000 births through the engine), primary literature

## Question

Is the brain's genotype-to-phenotype map, rather than the environment, what
keeps complex cognition from evolving in Petri? If so, what should be fixed
first? This audit was requested as a fresh, research-grounded perspective
alongside the same-day roadmap revision (commit 941c498e) that had already
moved brain execution and evolvability foundations ahead of environmental
mechanisms. Its recommendation was adopted on 2026-09-04 as the T11 track.

## Verdict

Yes. The brain's mutation map is a first-order problem. It is measurable in
two minutes, it explains why evolution in Petri can only tweak parameters,
and it sits upstream of every environmental feature on the roadmap.

At production settings, 90% of births are exact clones. Of the births that do
mutate, one in seventeen behaves like its parent, three in four behave
differently on most of a reactive test battery, and one in five emits no
action at all. The operators that would let a lineage start using memory
change reactive behavior 76 to 86% of the time. The graph operator that adds
a compute node changes behavior 99% of the time because it sprays about thirty
random edges into the live circuit. None of this is inherent to a VM plus
graph design. Each case traces to a specific operator or runtime decision,
and the two largest can be repaired with small, testable changes.

| Headline reading | Value |
| --- | --- |
| Mutated births that act exactly like the parent on the 48-scenario battery | 5.9% (32.4% when a birth gets a single event) |
| Mutated births that are behaviorally dead at birth (NoOp on every scenario) | 18.7% |
| Memory-motif insertions that leave the founder's reactive behavior intact | 1 in 4 today; 7 in 8 with jump offsets repaired |

## What was measured

Two throwaway integration tests were run against the crate through its
public API and then removed from the tree. Their sources are Appendices A and
B; they are the seed of the T11.F01 neighborhood indicator.

- **Subject.** The V3Alpha1 founder: a two-node mesh whose graph node
  aggregates sensors (6 compute nodes, 22 edges, 8 input references) and
  whose VM node is a hand-written decision program (105 instructions, 26
  jumps, all 16 registers in use). Every one of the 10,000 founders in the
  default world is this genome, so it is the starting point of every lineage.
- **Battery.** 48 seeded sensor snapshots varying local food of both types,
  the two neighbor food rings, occupancy, age, energy, and reproductive
  reserve. The founder yields 7 distinct action outputs across them. Each
  snapshot is run for one tick with shared memory zeroed and fresh graph
  state.
- **Classes.** *Silent*: identical actions on all 48 scenarios. *Changed*: at
  least one scenario differs and the creature still acts somewhere. *Dead*:
  NoOp on every scenario, which under production economics is starvation
  within about 40 ticks.
- **Treatments.** Each operator applied once to a fresh founder copy, 400
  seeded trials, with the production reachability bias. Then the full engine
  with the production mutation config, 6,000 births, bucketed by applied
  event count.

## Results

### Full engine, production config, per birth

Mutation triggers on 10% of births and then draws 1 to 10 events uniformly.
The bursts do the damage: single-event births are silent a third of the time,
four or more events almost never, and above four events roughly a quarter of
offspring are dead. "Scenarios changed" is the mean fraction of the 48
scenarios on which a changed or dead offspring's action differs from the
parent's.

| Births | n | silent | changed | dead | scenarios changed |
| --- | ---: | ---: | ---: | ---: | ---: |
| zero events (clones) | 5,411 | 100% | – | – | 0 |
| any events | 589 | 5.9% | 75.4% | 18.7% | 55% |
| 1 event | 74 | 32.4% | 63.5% | 4.1% | 27% |
| 2 events | 50 | 8.0% | 82.0% | 10.0% | 37% |
| 3 events | 64 | 6.2% | 84.4% | 9.4% | 35% |
| 4 events | 69 | 1.4% | 72.5% | 26.1% | 58% |
| 5 events | 64 | 3.1% | 76.6% | 20.3% | 59% |
| 6 events | 53 | 0.0% | 86.8% | 13.2% | 54% |
| 7 events | 63 | 0.0% | 71.4% | 28.6% | 64% |
| 8 events | 49 | 0.0% | 69.4% | 30.6% | 74% |
| 9 events | 41 | 0.0% | 73.2% | 26.8% | 65% |
| 10 events | 62 | 0.0% | 77.4% | 22.6% | 74% |

### Single operator events on the founder

Rows are the engine's operator names with their selection weight. Operators
that could not apply to the founder (no shared-memory instructions, no
plasticity nodes, one route target) are omitted. The only structural
operators that are reliably silent are the ones that create disconnected
copies.

| Operator | weight | silent | changed | dead | Note |
| --- | ---: | ---: | ---: | ---: | --- |
| **VM domain** | | | | | |
| VmConstantMutation | 4 | 56.8% | 43.2% | 0.0% | ±1.0 on a constant; the founder's thresholds are 30 and 20 |
| VmInstructionRawFieldMutation | 4 | 15.8% | 84.2% | 0.0% | redraws every operand at once; jump offsets become a random i32 |
| VmInstructionMutation | 2 | 14.5% | 84.0% | 1.5% | insert, replace, or delete one instruction; no jump remap |
| VmRegisterCountMutation | 2 | 49.5% | 50.5% | 0.0% | +1 is always silent; −1 aliases register 15 onto register 0 |
| VmInsertReadStoreMotif | 2 | 24.0% | 76.0% | 0.0% | the operator meant to seed memory use |
| VmInsertLoadCompareMotif | 2 | 17.8% | 82.2% | 0.0% | the operator meant to seed memory reads |
| VmInsertReadBidMotif | 2 | 12.8% | 86.0% | 1.2% | |
| VmDeleteInstruction | 1 | 7.2% | 92.8% | 0.0% | the founder has no introns to delete |
| VmCopyInstructionBlock | 1 | 9.0% | 90.2% | 0.8% | duplication inserts mid-program and shifts every crossing jump |
| VmCopyInstructionBlockRemapped | 1 | 4.8% | 92.8% | 2.5% | adds the positional delta to internal jumps that should not move |
| VmCopyGeneBackwardSlice | 1 | 9.2% | 84.0% | 6.8% | |
| VmCopyGeneForwardSlice | 1 | 8.5% | 91.5% | 0.0% | |
| VmCopyConstantBlock | 1 | 100% | 0.0% | 0.0% | append-only, so references survive |
| **Graph domain** | | | | | |
| AlterGraphEdgeWeight | 4 | 85.8% | 14.2% | 0.0% | ±20% multiplicative; the healthy kind of step |
| MutateGraphOperatorParam | 4 | 80.5% | 19.5% | 0.0% | |
| GraphRawFieldMutation | 4 | 14.2% | 85.8% | 0.0% | mostly retargets an edge source at random |
| SwapGraphOperator | 2 | 16.5% | 83.5% | 0.0% | |
| AddGraphEdge | 2 | 88.2% | 11.8% | 0.0% | most surfaces are inert sinks, so most new edges are silent |
| RetargetGraphEdge | 2 | 4.5% | 95.5% | 0.0% | every founder edge is live |
| RemoveGraphEdge | 2 | 5.5% | 94.5% | 0.0% | |
| CopyEdgeBundle | 2 | 44.2% | 55.8% | 0.0% | |
| AddInternalGraphNode | 1 | 1.0% | 99.0% | 0.0% | adds 29.5 random edges per event; 87% of events wire an action or execute gate |
| RemoveInternalGraphNode | 1 | 0.0% | 100% | 0.0% | no inactive node to remove |
| CopyInternalNode | 1 | 60.0% | 40.0% | 0.0% | always adds a backlink into a live node |
| CopySubgraph | 1 | 100% | 0.0% | 0.0% | copies land disconnected: the one neutral growth operator |
| EnableHebbian | 1 | 100% | 0.0% | 0.0% | invisible on a one-tick battery |
| **Topology domain** | | | | | |
| AddNode, CopyNode, mesh slices, RewriteNodeId, AddRouteTarget, MutateGateBias | 1–4 | 100% | 0.0% | 0.0% | neutral by construction; new nodes pass slots through |
| SpliceNode | 1 | 92.5% | 7.5% | 0.0% | |
| RetargetNodeTarget | 2 | 51.2% | 0.0% | 48.8% | |
| SwapNodeBackend | 1 | 0.0% | 48.2% | 51.8% | replaces a live node's whole backend with a blank one |
| ChangeEntryNode | 2 | 0.0% | 100% | 0.0% | |
| RemoveNode, RemoveRouteTarget | 1–2 | 0.0% | 0.0% | 100% | lethal on a two-node mesh |
| **Input reference domain** | | | | | |
| Add | 2 | 50.5% | 49.5% | 0.0% | also wires the new input somewhere at random |
| Swap | 4 | 13.5% | 86.5% | 0.0% | |
| RawFieldMutation | 4 | 7.0% | 93.0% | 0.0% | |
| Remove | 2 | 11.8% | 88.2% | 0.0% | |

### Cause isolation

The second probe re-applies the same edits with one thing changed. The
headline defects are separable and each accounts for most of its operator's
brittleness.

| Edit on the founder | silent today | silent after one fix | What the fix was |
| --- | ---: | ---: | --- |
| Insert one `Noop` at a random position | 28.8% | 100% | Recompute the offset of every jump whose source and target straddle the insertion point |
| Insert the read-store memory motif at a random position | 22.8% | 86.8% | Same jump repair; the remaining 13% is the motif clobbering one of the founder's 16 fully used registers |
| Same motif, jump repair, written to a fresh 17th register | 22.8% | 100% | Register slack |
| Register count +1 / −1 | silent / changed | – | Operands resolve modulo the register count; −1 aliases the top register onto register 0 |
| Add a graph compute node with no inputs, or with one bootstrap edge | 100% | – | Compared with 1.0% for the production operator, which also sprays one random edge per input sub-index (29.5 on the founder) |

## Where the brittleness comes from

Each item was read in the code and, where the probe can reach it, confirmed
by the numbers above. Paths are relative to `crates/v3-core/src`.

1. **Relative jump offsets are never remapped.** The VM encodes control flow
   as signed offsets from the jump instruction. Insert, delete, replace, the
   three motif inserts, and the block and slice copies in
   `mutation/vm/operators.rs` splice into the program without touching
   offsets, so every jump that spans the edit lands one instruction off. The
   founder's decision program has 26 jumps whose spans cover most of it,
   which is why a single no-op insertion breaks behavior 71% of the time.
   The "remapped" block copy adds the positional delta to every jump in the
   block, which is wrong for internal targets (they should keep their
   offset) and has the wrong sign for external ones.
2. **Raw-field mutation redraws whole instructions.** The highest-weighted VM
   operator redraws every operand of one instruction at once: register fields
   become uniform over 0 to 255 and wrap, and a jump's offset becomes a
   uniform random i32 wrapped by program length, so a nudged jump becomes a
   jump to anywhere. It also rewrites `Halt`, `ExecuteActionQueue`,
   `PopAction`, and `Noop` into a random `PushAction`. In linear GP terms
   these are not micro mutations; they are several macro mutations at once.
3. **Register count decrement aliases registers.** Operands resolve as
   `index mod register_count`, so shrinking the count by one silently merges
   the top register with register 0 across the whole program. Growing it is
   always silent and creates slack.
4. **Graph growth is a rewiring event.** `add_compute_node` in
   `mutation/graph/operators.rs` pushes the node and then adds one
   random-weight edge per sub-value of every input reference to a random
   surface, including action gates and the execute gate. On the founder that
   is about 30 edges, and 87% of events wire a gate that was deliberately
   left blank. NEAT's add-node mutation is designed to be function-preserving;
   Petri's is close to function-destroying.
5. **There is no neutral scaffold to mutate into.** The founder has zero
   introns: every instruction and every compute node is live, all 16
   registers are used, and every edge feeds an output. Removal, retarget, and
   swap operators therefore always hit live structure. The reachability bias
   (0.7) reinforces this by steering events at reachable nodes. New edges
   default to sub-index 0, so the ring directions are only reachable through
   raw-field redraws. Brain compute is nearly free (opcode cost multiplier
   1e-6, size pressure off), so junk would cost nothing, but the operators
   rarely produce it: only `CopySubgraph`, `VmCopyConstantBlock`, and the
   mesh-level copies are reliably neutral.
6. **Mutation arrives in bursts.** A 10% trigger followed by a uniform draw
   of 1 to 10 events gives an expected 0.55 events per birth, which is in
   Avida's range, but concentrated: 90% clones, and the mutated tenth
   averages 5.5 simultaneous edits. Even with every operator repaired,
   births with four or more of today's structural events would rarely be
   viable. The per-birth distribution, not only the rate, decides how many
   stepping stones exist.
7. **Runtime timing makes found memory unstable.** Confirmed in code, not
   measurable by the one-tick probe: stateful graph operators advance once
   per relaxation pass (at least three per tick with the default two stable
   passes, up to fifteen when a disconnected node keeps the graph from
   converging), eligibility traces decay only when their node executes while
   rewards are applied every tick, and the learning rate enters both the
   trace and the reward update. The graph also zeroes every compute output at
   the start of each dispatch, so a backward edge reads the previous
   relaxation pass rather than the previous tick: the graph's recurrence is
   a within-tick fixed point, not temporal memory. These are second-order
   until a lineage can reach memory at all, and they belong to T11.F06 and
   T11.F07.

## What established systems do

Every mature digital-evolution platform faced this problem and left a record.
The pattern is consistent: evolvability tracks mutational robustness,
robustness comes from position-independent references and neutral structure,
and mutation is delivered in small, mostly single-edit steps.

| System | Design choice | Evidence | Petri today |
| --- | --- | --- | --- |
| Avida (Ofria, Adami, Collier 2002) | Jump targets are nop-instruction templates found by complement matching, never distances; the genome is circular. | Five instruction sets compared. The set that replaced templates with an explicit jump distance in a register was the least evolvable, with rare length changes and poor adaptation. In a Redcode-style language over 99.7% of nontrivial mutations were deleterious and no trial adapted in 2,000 generations. The differences among sets were attributable mainly to mutational robustness and how length changes happen. | Explicit relative distances, unremapped on edit: the 2002 negative control. |
| Linear GP (Brameier and Banzhaf 2007) | A false branch skips exactly one instruction; nested branches AND together. Variation step size is limited to one instruction; introns are treated as protective. | Non-effective code "may act as a protection that reduces the effect of variation" on effective code and lets variation stay neutral. Effective step size falls over generations as effective code becomes robust. | Arbitrary jumps; one operator edits several fields at once; no introns in the founder. |
| Cartesian GP (Miller and Smith 2006; Goldman and Punch 2015) | Most of the genotype is inactive; mutation lands there and is neutral; active changes are rare and small. | The most evolvable genotypes are very large with over 95% inactive genes, at low mutation rates. Skipping neutral offspring evaluation and mutating until exactly one active gene changes reduces effort and sensitivity to mutation rate. | All compute nodes live; the reachability bias steers 70% of events at live structure, the inverse of the CGP regime. |
| NEAT (Stanley and Miikkulainen 2002) | Add-node splits an existing connection: the new incoming weight is 1, the outgoing keeps the old weight, so function is preserved at the moment of growth. | The canonical "grow without breaking" operator in neuroevolution. | Add-node sprays about 30 random edges and is 99% behavior-changing. |
| Markov Brains (Hintze et al. 2017) | Gates are read from start codons in a mostly non-coding genome; position carries no meaning. | Insertions, deletions, and duplications of 256 to 512 sites are routine operators because the encoding tolerates them. | Position is meaning in the VM; duplication breaks references. |
| SignalGP (Lalejini and Ofria 2018, 2021) | Modules and events are referenced by evolvable tags with closest-match resolution. | Inexact tag matching tolerates drift in references; tag-based regulation improved context-dependent problem solving. | Shared-memory slots are already immediate-addressed; jumps are not. |
| Quasispecies theory (Wilke et al. 2001; Wagner 2008) | Mutation supply must stay below the error threshold; robustness at the phenotype level is what makes evolvability possible. | At high mutation rates selection favors mutationally robust genotypes over faster replicators. Phenotype robustness (large neutral networks) promotes evolvability. | Bursts of up to ten structural edits push mutated lineages toward whatever is smallest and most robust, not toward more computation. |
| Evolved mutation rates (Clune et al. 2008) | Let lineages inherit their rate. | In digital organisms evolved rates land far below the rate that maximizes long-term adaptation. | Heritable mutation policy (T08.F05) should not be expected to repair supply on its own. |

Avida's operating point is a useful reference for supply: a per-instruction
copy mutation rate of 0.0075 by default (0.0025 in most published work) plus
a 5% chance of one insertion or deletion per gestation, so a 100-instruction
ancestor sees on the order of one small edit per offspring, never ten.

## The mesh vision and the node-type contract

Petri's design is a heterogeneous mesh: nodes of any compute kind (today a
VM program or a CGP graph; a recurrent network or a gate bank would fit) that
read sensors, shared memory, and an upstream slot bus, steer which node runs
next, may write memory as a side effect, and commit one action or a queue
each tick. The three systems above are not alternatives to that design. Each
is a single homogeneous representation that answers, inside itself, what a
unit computes, how state crosses a tick, and how the genome encodes structure
so that mutation is gentle. Replacing the mesh with one of them (option C
below) was therefore the wrong framing; they are candidate node kinds and
sources of contract rules, not replacements.

Most of the vision is already in the code. Every node declares its own
sensor, introspection, and upstream-slot inputs; nodes write shared memory,
push and pop the action queue, and bid for turn order; the slot vector is the
bus and unwritten slots pass through; each node's gate scores pick the next
node; and a tick ends with a committed queue. Two parts are narrower than the
vision. A node reaches exactly one successor per hop, so the mesh is a chain
with branching rather than fan-out, and nodes can only talk sideways through
shared memory. And the graph backend cannot be recurrent across ticks,
because it relaxes from zero every dispatch.

A heterogeneous mesh cannot inherit evolvability from any one node type, so
it has to guarantee in its own contract what NEAT, Markov brains, and
SignalGP each guarantee inside theirs:

1. every internal reference is stable by id or remapped on every edit;
2. every growth operator preserves function at the moment it fires;
3. every piece of persistent state advances once per world tick;
4. mutation supply arrives as small steps.

This audit is the record that those four were never written down as the
node-type contract: the VM violates the first two, the graph violates the
second and third, and the engine violates the fourth. Once they are
invariants, a new node kind is a bounded feature that inherits evolvability
from the mesh rather than re-deriving it. Fan-out (several modules active per
tick under a hop budget, SignalGP's shape) is left alone, because nothing
measured here says the chain is what blocks cognition. Recurrence inside
graph nodes does matter, and the T11 graph memory clock (T11.F06) decides it.

Prior art for the mesh itself, not only for its node kinds, reviewed
2026-09-05: each of these evolved a graph of heterogeneous decision-making
units in which nodes choose what runs next and the agent commits actions per
step.

| System | What it shares with the mesh | What it teaches Petri |
| --- | --- | --- |
| [Genetic Network Programming](https://direct.mit.edu/evco/article-abstract/15/3/369/1274/A-Graph-Based-Evolutionary-Algorithm-Genetic) (Mabu, Hirasawa, Hu 2007) | Judgment nodes read sensors and pick the next node; processing nodes emit actions; a fixed node pool with evolvable links; node-transition history acts as implicit memory | The closest match to the routing layer. Stable node identity plus evolvable links sidesteps reference breakage |
| [PADO](https://www.cs.cmu.edu/~mmv/papers/Teller-ESJ.pdf) (Teller and Veloso 1995) | Each node executes an action, then a branch decision selects the outgoing arc; programs act on an indexed memory | Shared memory plus an action-then-route node is PADO's node. Teller saw no obvious crossover analogue for such networks |
| [Evolving Virtual Creatures](https://www.karlsims.com/evolved-virtual-creatures.html) (Sims 1994) | A directed graph of heterogeneous node functions (sum, threshold, if, integrate, memory, oscillators) driving effectors | Heterogeneous node kinds, including stateful ones, evolve when unconnected nodes may persist as junk |
| [Markov Brains](https://arxiv.org/abs/1709.05601) (Hintze et al. 2017) | Heterogeneous gates (deterministic, probabilistic, ANN, threshold, timer, feedback) over one shared state buffer updated once per step | A blackboard plus mixed compute kinds is a mainstream design; the authors report that gate choice shapes the adaptive trajectory |
| Evolutionary programming (Fogel, 1960s) | Finite-state machines evolved by adding or deleting states and rerouting transitions | The oldest nodes-route-to-nodes representation; deleting a state remaps references into it |
| [SignalGP](https://arxiv.org/abs/1804.05445) (Lalejini and Ofria 2018) | Modules invoked by inexact tag, global memory, event-driven handlers | Tag references survive edits and duplication |
| Koza's [architecture-altering operations](https://www.researchgate.net/publication/2655092_Evolution_of_Both_the_Architecture_and_the_Sequence_of_Work-Performing_Steps_of_a_Computer_Program_Using_Genetic_Programming_with_Architecture-Altering_Operations) (1994, 1999) | Programs grow subroutines by duplication, creation, and deletion | Duplication yields an offspring semantically equivalent to its parent, the property T11.F08 needs |
| [Semantic neutral drift](https://eprints.whiterose.ac.uk/id/eprint/154735/1/Atkinson2019_Article_EvolvingGraphsWithSemanticNeut.pdf) (Atkinson, Plump, Stepney 2021) | Graph programs mutated by equivalence laws that preserve function | Function-preserving mutations on live structure measurably improve evolutionary performance |
| [Evolving Neural Turing Machines](https://dl.acm.org/doi/10.1145/2908812.2908930) (Greve, Jacobsen, Risi 2016) | NEAT-evolved controllers with an external memory they read and write | Side-effect memory attached to an evolved controller retains information longer than plasticity alone |

The price of heterogeneity is real but manageable. A richer alphabet lowers
the rate of advantageous substitutions (Ofria's 84-instruction set lagged the
standard set), every indirection layer (per-node sensor lists, the slot bus,
gated routing) is a mutation surface, and homology across mixed node kinds is
hard, which constrains sexual recombination (T08) rather than asexual
evolution. The remedy in every case is the contract: a small kind set, one
reference discipline, and the indicator as the gate for adding a kind.

| Vision element | Petri today | What the three systems do |
| --- | --- | --- |
| Node kinds | VM program, CGP graph | Any of them is a valid node kind; a temporally recurrent net and a gate bank are the missing flavors |
| Routing to downstream nodes | gated argmax, one successor per hop | SignalGP invokes modules by tag and runs several per step under a thread cap |
| Bus and memory | slot baton along the chain, 16-slot blackboard across ticks | Markov brains treat the blackboard as the whole state, updated once per world step |
| Time | shared memory ticks; graph state advances per relaxation pass | All three advance state exactly once per step |
| References inside a node | mesh node ids are stable; VM jumps and graph indices are positional | Tags, start codons, and innovation numbers make references survive edits |
| Growth | mesh add is neutral and splice nearly so; graph add and VM insert are not | NEAT splits an edge and preserves function; SignalGP and Markov brains duplicate modules freely |

## Options

| Option | What changes | Evidence and fit | Cost and risk | Call |
| --- | --- | --- | --- | --- |
| A. Repair in place (extend) | Remap jump offsets on every VM insert, delete, and copy; single-field operand steps with bounded jump nudges; register count grows or remaps, never aliases; add-node without the spray plus a NEAT-style split-edge variant; copy operators never backlink into live nodes; supply delivered as mostly single events. | Direct fixes for the measured causes. Matches linear GP, NEAT, and CGP practice. Keeps the ISA, specs, founder, and frontend. | Days. Relative jumps remain, so each future operator must keep the remap invariant; property tests cover it. The persistence baseline must be re-read after supply changes. | Do now. |
| B. Position-independent references (extend the ISA) | Replace offset jumps with label-addressed jumps (evolvable tag, nearest match, fall through on no match) and add a one-instruction skip branch. Shared memory already works this way. | The Avida, SignalGP, and Markov-brain lesson. Removes the whole reference-integrity class and makes block duplication plus divergence work by construction. | One to two weeks: ISA spec, VM, founder builder, VM operators, slice analysis, frontend VM view. Behavior change for all genomes, which the repository allows. | Schedule after A as the durable fix, gated on what the probe still shows. |
| C. Replace the controller (adopt) | SignalGP-lite, Markov brains, or a NEAT-style recurrent network in place of the mesh. | Most external evidence, but it discards the mesh, VM, graph, plasticity, tracing, and determinism work. | Months. Unknown throughput at 10,000 creatures. Ecology and observability rewire. | Only if A and B fail to reach the floors. |
| D. Tune rates only | Lower events per birth, raise trigger probability. | Necessary but not sufficient: single-event births are still 63.5% changed and 4.1% dead today. | Hours. | Fold into A. |

## Recommendation, as adopted in T11

Treat the brain's mutation map as a foundation with a standing invariant,
and compress the path: one cheap probe, then repairs, with the probe as the
acceptance test for each.

1. Make the probe a gate-profile indicator with predeclared floors
   (T11.F01). A no-op insertion is always silent; memory-motif inserts are
   at least 80% silent on the founder; add-node and copy-node are at least
   95% silent; mutated births are at most 5% dead; single-event births are
   at least 60% silent. These are floors that say structural growth is
   usually neutral, not a score to maximize by removing behavior-changing
   operators. The track treats them as targets met by the close of T11.F10,
   with a per-feature no-regression rule in between, since the dead and
   single-event floors cannot be reached before the supply change.
2. Broaden the VM repair from reference integrity to VM structural-mutation
   semantics (T11.F02): jump remap on every splice, single-field operand
   perturbation with a small bounded jump nudge, no terminal-to-push
   rewrites, register count that grows or remaps rather than aliases, and a
   founder with register slack.
3. Make graph growth NEAT-style (T11.F03): a node with zero or one bootstrap
   edge or a function-preserving split of an existing edge, no
   input-reference spray, no backlinks into live nodes, and new edges that
   sample sub-indices.
4. Add one bounded mutation-supply feature (T11.F04): the same expected
   supply as mostly single events, a lower reachability bias, and a
   re-read persistence baseline.
5. Keep the graph memory clock and reward trace clock (T11.F06, T11.F07)
   after the accessibility repairs, with the temporal fixtures (T11.F05)
   placed just ahead of them rather than ahead of the repairs. The graph
   clock decides whether backward edges read last tick's outputs, which
   would make every recurrent edge a one-tick memory element.
6. Then memory-motif evolvability from founders (T11.F10), then seasons
   (T02.F01). Label-addressed control flow (T11.F11) is scheduled
   conditionally after the repairs, gated on what the indicator still shows.

A proof of concept before committing to the whole track: implement the jump
remap, the no-spray add-node, and single-field operand steps behind a config
flag, rerun the two probes and `cargo test -p v3-core --test viability`. If
the founder's single-event silent rate does not at least double, this
analysis is wrong and option B should move up.

### Where the same-day roadmap revision landed

Right: the four defects it named (jump remapping, broad graph growth,
relaxation-dependent memory timing, stale reward traces) are all real, and
foundations-first is the correct order. Short in four places. It scoped the VM
repair to reference integrity, while operand redraws, register aliasing, and
terminal rewrites are as large. It treated supply and bias as tuning to defer,
when the per-birth distribution is what turns individual defects into a 19%
dead rate. It sequenced diagnostics ahead of cheap repairs whose test already
exists. And it did not weigh position-independent jump addressing as an
extension of the ISA, which the 2002 evidence identifies as the design choice
that matters most.

## Limits of this evidence

- The battery is reactive and single-tick with zeroed memory. It cannot see
  changes that only show up through memory or plasticity, so it under-counts
  change for those operators and says nothing about whether memory, once
  found, is useful.
- "Changed" is not "worse". Some changed offspring are the adaptive ones.
  The claim is about the shape of the neighborhood (too few neutral and
  near-neutral neighbors, too many dead ones), not that change is bad.
- Only the founder was probed. Lineages 2,000 ticks into a run may have
  evolved toward more robust genotypes, as quasispecies theory predicts; the
  T11.F01 sample from an evolved population will show it. That would confirm
  the mechanism rather than excuse it.
- Repairing the map does not guarantee cognition. It removes a sufficient
  reason for its absence. Whether the ecology then rewards memory is what
  T11.F10, T09.F08, and T02 test.

## Reproducing the probes

The two test files in the appendices were run from `crates/v3-core/tests/`
at commit 4b8650ef and removed afterward. Drop them back in and run:

```sh
cargo test -p v3-core --test zz_probe_neighborhood -- --nocapture   # ~130 s, debug build
cargo test -p v3-core --test zz_probe_causes -- --nocapture         # ~2 s
```

Both are deterministic (`SmallRng`, fixed seeds). The battery seed is 7;
operator trials use seeds 1000 to 4399; engine births use seeds 9000 to
14999. The probes use only the crate's public API and will need small edits
if the mutator or mesh signatures change.

## Sources

- Ofria, Adami, Collier. [Design of Evolvable Computer Languages](https://langev.com/pdf/ofria02ieee.pdf). IEEE Trans. Evolutionary Computation 6(4), 2002. Read in full.
- Brameier, Banzhaf. [Linear Genetic Programming](https://link.springer.com/book/10.1007/978-0-387-31030-5). Springer, 2007. Sections 2.1.3 (branching) and 3.2 (introns) read from the publisher preview.
- Miller, Smith. [Redundancy and computational efficiency in Cartesian genetic programming](https://pure.york.ac.uk/portal/en/publications/redundancy-and-computational-efficiency-in-cartesian-genetic-prog). IEEE Trans. Evolutionary Computation 10(2), 2006. Abstract.
- Goldman, Punch. [Analysis of Cartesian Genetic Programming's Evolutionary Mechanisms](https://dl.acm.org/doi/abs/10.1109/TEVC.2014.2324539). IEEE Trans. Evolutionary Computation 19(3), 2015. Abstract and secondary summaries.
- Stanley, Miikkulainen. [Evolving Neural Networks through Augmenting Topologies](https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf). Evolutionary Computation 10(2), 2002.
- Hintze et al. [Markov Brains: A Technical Introduction](https://arxiv.org/abs/1709.05601). arXiv:1709.05601, 2017. Encoding and mutation sections read in full.
- Lalejini, Ofria. [Evolving Event-driven Programs with SignalGP](https://arxiv.org/abs/1804.05445). GECCO 2018. Lalejini, Moreno, Ofria. [Tag-based regulation of modules in genetic programming](https://link.springer.com/article/10.1007/s10710-021-09406-8). GPEM 2021.
- Wilke, Wang, Ofria, Lenski, Adami. [Evolution of digital organisms at high mutation rates leads to survival of the flattest](https://www.nature.com/articles/35085569). Nature 412, 2001.
- Wagner. [Robustness and evolvability: a paradox resolved](https://royalsocietypublishing.org/doi/10.1098/rspb.2007.1137). Proc. R. Soc. B 275, 2008.
- Clune et al. [Natural Selection Fails to Optimize Mutation Rates for Long-Term Adaptation on Rugged Fitness Landscapes](https://journals.plos.org/ploscompbiol/article?id=10.1371/journal.pcbi.1000187). PLoS Comput. Biol. 4(9), 2008.
- Avida wiki. [Mutation settings](https://github.com/devosoft/avida/wiki/Mutation-settings) and [Instruction Set](https://github.com/devosoft/avida/wiki/Instruction-Set).
- Petri code read: `runtime/vm.rs`, `runtime/cgp/execute.rs`, `runtime/plasticity/{traces,reward}.rs`, `mutation/engine/mod.rs`, `mutation/vm/operators.rs`, `mutation/graph/operators.rs`, `mutation/topology/{structural,birth}.rs`, `creature/{founder,cgp_founder}.rs`, `config/simulation.rs`; roadmap commit 941c498e.

## Appendix A: neighborhood probe

`crates/v3-core/tests/zz_probe_neighborhood.rs` as run:

```rust
//! TEMPORARY PROBE (not for commit): single-event mutational neighborhood of the founder.
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use v3_core::config::{FounderProfile, MutationConfig, RuntimeConfig};
use v3_core::contracts::WorldAction;
use v3_core::creature::founder::founder_genome;
use v3_core::creature::genome::analysis::mesh_reachable_nodes;
use v3_core::creature::genome::CreatureGenome;
use v3_core::creature::state::GraphRuntimeState;
use v3_core::mutation::graph::{GraphMutator, GraphOperator};
use v3_core::mutation::input_ref::{InputRefMutator, InputRefOperator};
use v3_core::mutation::topology::{TopologyMutator, TopologyOperator};
use v3_core::mutation::vm::{VmMutator, VmOperator};
use v3_core::mutation::{MutationEngine, MutationSkipReason, TargetReachability};
use v3_core::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
use v3_core::sensors::static_inputs::StaticInputs;
use v3_core::sensors::typed_food::TypedFoodLocalSnapshot;

struct Scenario {
    food: [f32; 2],
    nfood: [[f32; 8]; 2],
    occ: [f32; 8],
    age: f32,
    energy: f32,
    reserve: f32,
}

fn scenarios(n: usize) -> Vec<Scenario> {
    let mut rng = SmallRng::seed_from_u64(7);
    let mut v = Vec::new();
    for _ in 0..n {
        let f = |rng: &mut SmallRng, p0: f64| -> f32 {
            if rng.gen_bool(p0) { 0.0 } else { rng.gen_range(0.1f32..=1.0) }
        };
        let mut s = Scenario {
            food: [f(&mut rng, 0.5), f(&mut rng, 0.6)],
            nfood: [[0.0; 8]; 2],
            occ: [0.0; 8],
            age: *[0.0f32, 5.0, 19.0, 20.0, 50.0, 200.0].iter().nth(rng.gen_range(0..6)).unwrap(),
            energy: *[5.0f32, 15.0, 25.0, 31.0, 45.0, 80.0].iter().nth(rng.gen_range(0..6)).unwrap(),
            reserve: *[0.0f32, 2.0, 4.0, 8.0].iter().nth(rng.gen_range(0..4)).unwrap(),
        };
        for t in 0..2 {
            for i in 0..8 {
                s.nfood[t][i] = f(&mut rng, 0.6);
            }
        }
        for i in 0..8 {
            s.occ[i] = if rng.gen_bool(0.2) { 1.0 } else { 0.0 };
        }
        v.push(s);
    }
    v
}

fn signature(genome: &CreatureGenome, scen: &[Scenario], cfg: &RuntimeConfig) -> Vec<Vec<WorldAction>> {
    scen.iter()
        .map(|s| {
            let ss = SensorSnapshot {
                local: StaticInputs {
                    food_here: s.food[0],
                    neighbor_food: s.nfood[0],
                    neighbor_barrier: [0.0; 8],
                    neighbor_occupied: s.occ,
                    generation: 0.0,
                    age_ticks: s.age,
                },
                typed_local_food: TypedFoodLocalSnapshot {
                    food_here_by_type: vec![s.food[0], s.food[1]],
                    neighbor_food_by_type: vec![s.nfood[0], s.nfood[1]],
                },
                perception: PerceptionSnapshot::zeroed(2),
            };
            let mut energy = s.energy;
            let mut sm = [0.0f32; 16];
            let prev = [0.0f32; 16];
            let mut gr = GraphRuntimeState::new();
            let out = execute_creature_mesh_pub(genome, &ss, &mut energy, s.reserve, &mut sm, &prev, &mut gr, cfg);
            out
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn execute_creature_mesh_pub(
    genome: &CreatureGenome,
    ss: &SensorSnapshot,
    energy: &mut f32,
    reserve: f32,
    sm: &mut [f32; 16],
    prev: &[f32; 16],
    gr: &mut GraphRuntimeState,
    cfg: &RuntimeConfig,
) -> Vec<WorldAction> {
    v3_core::runtime::execute_creature_mesh(genome, ss, energy, reserve, sm, prev, gr, cfg).actions
}

#[derive(Default, Debug)]
struct Tally {
    trials: u32,
    skipped: u32,
    silent: u32,
    changed: u32,
    dead: u32,
    frac_changed_sum: f64,
}

fn classify(base: &[Vec<WorldAction>], sig: &[Vec<WorldAction>], t: &mut Tally) {
    let n = base.len();
    let diff = base.iter().zip(sig.iter()).filter(|(a, b)| a != b).count();
    if diff == 0 {
        t.silent += 1;
    } else {
        let all_noop = sig.iter().all(|acts| acts.iter().all(|a| matches!(a, WorldAction::NoOp)));
        if all_noop { t.dead += 1 } else { t.changed += 1 }
        t.frac_changed_sum += diff as f64 / n as f64;
    }
}

fn report(name: &str, t: &Tally) {
    let applied = t.trials - t.skipped;
    let pct = |x: u32| if applied == 0 { 0.0 } else { 100.0 * x as f64 / applied as f64 };
    let mean_frac = if t.changed + t.dead == 0 { 0.0 } else { t.frac_changed_sum / (t.changed + t.dead) as f64 };
    println!(
        "{:<36} applied={:>4} skipped={:>4} silent={:>5.1}% changed={:>5.1}% dead={:>5.1}% mean_scen_frac_when_changed={:.2}",
        name, applied, t.skipped, pct(t.silent), pct(t.changed), pct(t.dead), mean_frac
    );
}

#[test]
fn probe_founder_neighborhood() {
    const TRIALS: u32 = 400;
    let scen = scenarios(48);
    let rt = RuntimeConfig::default();
    let mcfg = MutationConfig::default();
    let founder = founder_genome(FounderProfile::V3Alpha1);
    let reachable = mesh_reachable_nodes(&founder);
    let base = signature(&founder, &scen, &rt);
    let distinct: std::collections::HashSet<String> = base.iter().map(|a| format!("{a:?}")).collect();
    println!("founder: {} nodes, reachable={:?}, distinct action outputs over {} scenarios = {}", founder.nodes.len(), reachable, scen.len(), distinct.len());

    println!("\n== VM operators (single event) ==");
    for op in VmOperator::ALL {
        let mut t = Tally::default();
        for trial in 0..TRIALS {
            let mut g = founder.clone();
            let mut rng = SmallRng::seed_from_u64(1_000 + trial as u64);
            t.trials += 1;
            match VmMutator::apply(&mut g, op, &reachable, 0.7, &mut rng, &mcfg) {
                Err(_) => t.skipped += 1,
                Ok(_) => classify(&base, &signature(&g, &scen, &rt), &mut t),
            }
        }
        report(&format!("{op:?}"), &t);
    }

    println!("\n== Graph operators (single event) ==");
    for op in GraphOperator::ALL {
        let mut t = Tally::default();
        for trial in 0..TRIALS {
            let mut g = founder.clone();
            let mut rng = SmallRng::seed_from_u64(2_000 + trial as u64);
            t.trials += 1;
            match GraphMutator::apply(&mut g, op, &reachable, 0.7, &mut rng) {
                Err(_) => t.skipped += 1,
                Ok(_) => classify(&base, &signature(&g, &scen, &rt), &mut t),
            }
        }
        report(&format!("{op:?}"), &t);
    }

    println!("\n== Topology operators (single event) ==");
    for op in TopologyOperator::ALL {
        let mut t = Tally::default();
        for trial in 0..TRIALS {
            let mut g = founder.clone();
            let mut rng = SmallRng::seed_from_u64(3_000 + trial as u64);
            t.trials += 1;
            match TopologyMutator::apply_with_food_type_count(&mut g, op, &reachable, 0.7, &mut rng, &mcfg, 2) {
                Err(_) => t.skipped += 1,
                Ok(_) => classify(&base, &signature(&g, &scen, &rt), &mut t),
            }
        }
        report(&format!("{op:?}"), &t);
    }

    println!("\n== InputRef operators (single event) ==");
    for op in InputRefOperator::ALL {
        let mut t = Tally::default();
        for trial in 0..TRIALS {
            let mut g = founder.clone();
            let mut rng = SmallRng::seed_from_u64(4_000 + trial as u64);
            t.trials += 1;
            match InputRefMutator::apply_with_food_type_count(&mut g, op, &reachable, 0.7, &mut rng, &mcfg, 2) {
                Err(_) => t.skipped += 1,
                Ok(_) => classify(&base, &signature(&g, &scen, &rt), &mut t),
            }
        }
        report(&format!("{op:?}"), &t);
    }

    println!("\n== Full engine, production MutationConfig, per birth ==");
    let mut by_events: std::collections::BTreeMap<u32, Tally> = Default::default();
    let mut overall = Tally::default();
    let mut zero_event_births = 0u32;
    const BIRTHS: u32 = 6000;
    for b in 0..BIRTHS {
        let mut g = founder.clone();
        let mut rng = SmallRng::seed_from_u64(9_000 + b as u64);
        let summary = MutationEngine::apply_mutations_with_food_type_count(&mut g, &mcfg, &reachable, &mut rng, 2);
        if summary.applied_events == 0 {
            zero_event_births += 1;
            continue;
        }
        overall.trials += 1;
        let t = by_events.entry(summary.applied_events).or_default();
        t.trials += 1;
        let sig = signature(&g, &scen, &rt);
        classify(&base, &sig, &mut overall);
        classify(&base, &sig, t);
    }
    println!("births={} zero-event={} ({:.1}%)", BIRTHS, zero_event_births, 100.0 * zero_event_births as f64 / BIRTHS as f64);
    report("mutated births (any events)", &overall);
    for (k, t) in &by_events {
        report(&format!("  applied_events={k}"), t);
    }
    let _ = (MutationSkipReason::NoApplicableTarget, TargetReachability::Reachable);
}
```

## Appendix B: cause-isolation probe

`crates/v3-core/tests/zz_probe_causes.rs` as run:

```rust
//! TEMPORARY PROBE (not for commit): isolate causes of founder-neighborhood brittleness.
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use v3_core::config::{FounderProfile, RuntimeConfig};
use v3_core::contracts::WorldAction;
use v3_core::creature::founder::founder_genome;
use v3_core::creature::genome::cgp::{ComputeNode, ComputeNodeKind, GraphEdge, GraphSource};
use v3_core::creature::genome::{BackendDef, CreatureGenome, VmInstruction};
use v3_core::creature::state::GraphRuntimeState;
use v3_core::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
use v3_core::sensors::static_inputs::StaticInputs;
use v3_core::sensors::typed_food::TypedFoodLocalSnapshot;

struct Scenario { food: [f32; 2], nfood: [[f32; 8]; 2], occ: [f32; 8], age: f32, energy: f32, reserve: f32 }

fn scenarios(n: usize) -> Vec<Scenario> {
    let mut rng = SmallRng::seed_from_u64(7);
    let mut v = Vec::new();
    for _ in 0..n {
        let f = |rng: &mut SmallRng, p0: f64| -> f32 { if rng.gen_bool(p0) { 0.0 } else { rng.gen_range(0.1f32..=1.0) } };
        let mut s = Scenario {
            food: [f(&mut rng, 0.5), f(&mut rng, 0.6)], nfood: [[0.0; 8]; 2], occ: [0.0; 8],
            age: [0.0f32, 5.0, 19.0, 20.0, 50.0, 200.0][rng.gen_range(0..6)],
            energy: [5.0f32, 15.0, 25.0, 31.0, 45.0, 80.0][rng.gen_range(0..6)],
            reserve: [0.0f32, 2.0, 4.0, 8.0][rng.gen_range(0..4)],
        };
        for t in 0..2 { for i in 0..8 { s.nfood[t][i] = f(&mut rng, 0.6); } }
        for i in 0..8 { s.occ[i] = if rng.gen_bool(0.2) { 1.0 } else { 0.0 }; }
        v.push(s);
    }
    v
}

fn signature(genome: &CreatureGenome, scen: &[Scenario], cfg: &RuntimeConfig) -> Vec<Vec<WorldAction>> {
    scen.iter().map(|s| {
        let ss = SensorSnapshot {
            local: StaticInputs { food_here: s.food[0], neighbor_food: s.nfood[0], neighbor_barrier: [0.0; 8], neighbor_occupied: s.occ, generation: 0.0, age_ticks: s.age },
            typed_local_food: TypedFoodLocalSnapshot { food_here_by_type: vec![s.food[0], s.food[1]], neighbor_food_by_type: vec![s.nfood[0], s.nfood[1]] },
            perception: PerceptionSnapshot::zeroed(2),
        };
        let mut energy = s.energy; let mut sm = [0.0f32; 16]; let prev = [0.0f32; 16]; let mut gr = GraphRuntimeState::new();
        v3_core::runtime::execute_creature_mesh(genome, &ss, &mut energy, s.reserve, &mut sm, &prev, &mut gr, cfg).actions
    }).collect()
}

fn silent_pct(base: &[Vec<WorldAction>], sigs: &[Vec<Vec<WorldAction>>]) -> f64 {
    100.0 * sigs.iter().filter(|s| s.as_slice() == base).count() as f64 / sigs.len() as f64
}

/// Insert `block` at `pos`, adjusting every jump whose source/target straddle `pos`.
fn insert_with_fixup(program: &mut Vec<VmInstruction>, pos: usize, block: Vec<VmInstruction>) {
    let len = program.len() as i64;
    let k = block.len() as i64;
    let shift = |i: i64| if i >= pos as i64 { i + k } else { i };
    for (j, instr) in program.iter_mut().enumerate() {
        if let VmInstruction::Jump { offset } | VmInstruction::JumpIfZero { offset, .. } = instr {
            let t = (j as i64 + 1 + *offset as i64).rem_euclid(len);
            let j2 = shift(j as i64);
            let t2 = shift(t);
            *offset = (t2 - j2 - 1) as i32;
        }
    }
    program.splice(pos..pos, block);
}

fn vm_program_mut(g: &mut CreatureGenome) -> &mut v3_core::creature::genome::VmBackendDef {
    match &mut g.nodes[1].backend_def { BackendDef::Vm(vm) => vm, _ => panic!("node1 is VM") }
}

#[test]
fn probe_causes() {
    const TRIALS: usize = 400;
    let scen = scenarios(48);
    let rt = RuntimeConfig::default();
    let founder = founder_genome(FounderProfile::V3Alpha1);
    let base = signature(&founder, &scen, &rt);
    let (plen, jumps, regs) = match &founder.nodes[1].backend_def {
        BackendDef::Vm(vm) => (vm.program.len(), vm.program.iter().filter(|i| matches!(i, VmInstruction::Jump{..}|VmInstruction::JumpIfZero{..})).count(), vm.register_count),
        _ => unreachable!(),
    };
    println!("founder VM: {plen} instructions, {jumps} jumps, {regs} registers");
    if let BackendDef::Graph(gd) = &founder.nodes[0].backend_def {
        let edges: usize = gd.compute_nodes.iter().map(|n| n.inputs.len()).sum::<usize>() + gd.output_sinks.iter().map(|s| s.inputs.len()).sum::<usize>();
        println!("founder graph: {} compute nodes, {} edges, {} input_refs", gd.compute_nodes.len(), edges, founder.nodes[0].input_refs.len());
    }

    // 1. Insert a single Noop at a random position: current semantics vs jump fixup.
    let mut sig_raw = Vec::new(); let mut sig_fix = Vec::new();
    for t in 0..TRIALS {
        let mut rng = SmallRng::seed_from_u64(100 + t as u64);
        let pos = rng.gen_range(0..=plen);
        let mut g1 = founder.clone(); vm_program_mut(&mut g1).program.insert(pos, VmInstruction::Noop); sig_raw.push(signature(&g1, &scen, &rt));
        let mut g2 = founder.clone(); insert_with_fixup(&mut vm_program_mut(&mut g2).program, pos, vec![VmInstruction::Noop]); sig_fix.push(signature(&g2, &scen, &rt));
    }
    println!("insert 1 Noop at random pos: silent current={:.1}%  with-jump-fixup={:.1}%", silent_pct(&base, &sig_raw), silent_pct(&base, &sig_fix));

    // 2. Insert the read-store memory motif: current vs fixup, and fixup + fresh (unused) register.
    let mut s_raw = Vec::new(); let mut s_fix = Vec::new(); let mut s_fix_fresh = Vec::new();
    for t in 0..TRIALS {
        let mut rng = SmallRng::seed_from_u64(200 + t as u64);
        let pos = rng.gen_range(0..=plen);
        let dst = rng.gen_range(0..regs);
        let slot = rng.gen_range(0u8..16);
        let refi = rng.gen_range(0u16..13);
        let motif = |d: u8| vec![VmInstruction::ReadInput { dst: d, ref_idx: refi, sub_idx: 0 }, VmInstruction::StoreSlotImm { slot_idx: slot, src: d }];
        let mut g1 = founder.clone(); vm_program_mut(&mut g1).program.splice(pos..pos, motif(dst)); s_raw.push(signature(&g1, &scen, &rt));
        let mut g2 = founder.clone(); insert_with_fixup(&mut vm_program_mut(&mut g2).program, pos, motif(dst)); s_fix.push(signature(&g2, &scen, &rt));
        let mut g3 = founder.clone(); { let vm = vm_program_mut(&mut g3); vm.register_count = regs + 1; insert_with_fixup(&mut vm.program, pos, motif(regs)); } s_fix_fresh.push(signature(&g3, &scen, &rt));
    }
    println!("insert read-store motif: silent current={:.1}%  with-jump-fixup={:.1}%  fixup+fresh-register={:.1}%", silent_pct(&base, &s_raw), silent_pct(&base, &s_fix), silent_pct(&base, &s_fix_fresh));

    // 3. Register count: increment vs decrement.
    let mut inc = founder.clone(); vm_program_mut(&mut inc).register_count = regs + 1;
    let mut dec = founder.clone(); vm_program_mut(&mut dec).register_count = regs - 1;
    println!("register_count +1 silent={}  -1 silent={}", signature(&inc, &scen, &rt) == base, signature(&dec, &scen, &rt) == base);

    // 4. Graph: add a compute node WITHOUT the input-ref edge spray (disconnected or one bootstrap edge).
    let mut s_none = Vec::new(); let mut s_boot = Vec::new();
    for t in 0..TRIALS {
        let mut rng = SmallRng::seed_from_u64(300 + t as u64);
        let kinds = [ComputeNodeKind::Add, ComputeNodeKind::Sigmoid, ComputeNodeKind::Threshold(rng.gen_range(-1.0..=1.0)), ComputeNodeKind::DecayIntegrator(rng.gen_range(0.0..=1.0)), ComputeNodeKind::Max];
        let kind = kinds[rng.gen_range(0..kinds.len())];
        let mut g1 = founder.clone();
        if let BackendDef::Graph(gd) = &mut g1.nodes[0].backend_def { gd.compute_nodes.push(ComputeNode { kind, inputs: vec![], plasticity: None }); }
        s_none.push(signature(&g1, &scen, &rt));
        let mut g2 = founder.clone();
        if let BackendDef::Graph(gd) = &mut g2.nodes[0].backend_def {
            let src = if rng.gen_bool(0.5) { GraphSource::ComputeNode(rng.gen_range(0..6)) } else { GraphSource::InputLeaf { ref_idx: rng.gen_range(0..8), sub_idx: 0 } };
            gd.compute_nodes.push(ComputeNode { kind, inputs: vec![GraphEdge { source: src, weight: rng.gen_range(-1.0..=1.0) }], plasticity: None });
        }
        s_boot.push(signature(&g2, &scen, &rt));
    }
    println!("graph add node w/o spray: silent disconnected={:.1}%  one-bootstrap-edge={:.1}%", silent_pct(&base, &s_none), silent_pct(&base, &s_boot));

    // 5. How many edges does the production AddInternalGraphNode add on the founder?
    use v3_core::mutation::graph::{GraphMutator, GraphOperator};
    use v3_core::creature::genome::analysis::mesh_reachable_nodes;
    let reachable = mesh_reachable_nodes(&founder);
    let count_edges = |g: &CreatureGenome| -> usize { match &g.nodes[0].backend_def { BackendDef::Graph(gd) => gd.compute_nodes.iter().map(|n| n.inputs.len()).sum::<usize>() + gd.output_sinks.iter().map(|s| s.inputs.len()).sum::<usize>() + gd.action_bank.iter().map(|a| a.gate_inputs.len()+a.param_inputs.len()).sum::<usize>() + gd.execute_gate.inputs.len(), _ => 0 } };
    let e0 = count_edges(&founder);
    let mut added = 0usize; let mut gate_hits = 0usize;
    for t in 0..TRIALS {
        let mut g = founder.clone(); let mut rng = SmallRng::seed_from_u64(400 + t as u64);
        GraphMutator::apply(&mut g, GraphOperator::AddInternalGraphNode, &reachable, 0.7, &mut rng).unwrap();
        added += count_edges(&g) - e0;
        if let BackendDef::Graph(gd) = &g.nodes[0].backend_def { if gd.action_bank.iter().any(|a| !a.gate_inputs.is_empty()) || !gd.execute_gate.inputs.is_empty() { gate_hits += 1; } }
    }
    println!("production AddInternalGraphNode on founder: mean edges added per event = {:.1}; events that wired an action gate or execute gate = {:.1}%", added as f64 / TRIALS as f64, 100.0 * gate_hits as f64 / TRIALS as f64);
}
```
