# Mesh Backend Bias: One Graph and a VM Chain

**Status**: Research (evidence for [T11.F18](../roadmaps/t11-brain-genotype-phenotype-map.md); not an implementation spec)
**Date**: 2026-09-08
**Code**: analysis probe linked against `main` at `beee4c05`
**Subject**: paused local server at tick 73,035, population 49,545; 200 creatures sampled without replacement using Python `random.Random(20260908)`
**Results**: [machine-readable readings](mesh-backend-bias-research-2026-09-08.results.json)

## Question and decision

Why do controllers appear to retain one Graph node while adding a chain of
VM nodes? Separate what mutation supplies, what executes, and what affects
behavior. Preserve working controllers and the existing mesh while making
both backend kinds available through the same safe growth path.

The code imposes a direct VM creation bias. The user requested a roadmap
feature to remove it and made that feature the next implementation priority.
T11.F18 extends the existing detour constructor to choose either backend with
equal probability. This is an opportunity correction; it does not establish
that selection should retain equal numbers of Graph and VM nodes.

## What this run shows

The applied configuration differs from production defaults: mutation trigger
0.1, one or two requested events, continuation probability 0.2, executed bias
0.9, base decay 0.1, and genome carry cost 0.0001 per unit per tick. The
requested mutation mean is therefore 0.12 events per birth, versus the
current default's approximately 0.55. Size-pressure restriction is off.
The older depth note's free-junk, uniform-target regime is not this run's
current configuration; configuration history was not recovered.

Replay used the existing `neighborhood-v1` battery: 48 individual snapshots
and eight four-tick sequences, 80 executions per genome, fresh initial
state, and the run's applied runtime and memory-decay settings. The server
binary's source revision was not independently established; these are
replays of its serialized genomes under the named analysis revision.

| Reading across 200 sampled creatures | Result |
| --- | ---: |
| Generation, median | 768.5 |
| Total mesh nodes, median | 13 |
| Total Graph / VM nodes, separate medians | 2 / 11 |
| Executed Graph / VM nodes, separate medians | 1 / 5 |
| Contributing Graph / VM nodes, separate medians | 1 / 2 |
| Exactly one executed Graph | 193 / 200 |
| Exactly one contributing Graph | 200 / 200 |
| Input-dependent route variation | 200 / 200 |
| Any hop-cap hit | 0 / 200 |

“Executed” is the union across the battery, not simultaneous execution or a
single tick's path length. “Contributing” means that removing the node and
redirecting through its static winning successor changes at least one
action queue. The probe's backend-specific bypass counts were checked
against the existing `Battery::mesh_execution` knockout total for every
genome. This assay does not prove indispensability in every environment.

The observation is mostly about active function: 160 creatures already
carry more than one Graph node, but additional Graph nodes contribute no
action differences in this battery. Of 1,111 executed VM node occurrences,
715 can be bypassed without changing the action signature. The median
executed Graph has 15 internal compute nodes; the outer count of one does
not imply one primitive operation. The population already routes
conditionally, so a visually linear tick trace is not evidence of a
controller that never branches.

## The asymmetry in the code

- All founder profiles start with a Graph sensor node followed by a VM
  decision node: [`creature/founder.rs`](../../crates/v3-core/src/creature/founder.rs).
- `AddNode` aliases `SpliceNode`. Both insert `birth::detour` between a
  predecessor and its successor. `AddRouteTarget` uses the same constructor
  for its alternative route. That constructor always creates a one-register,
  Halt-only VM with no input references:
  [`topology/birth.rs`](../../crates/v3-core/src/mutation/topology/birth.rs),
  [`topology/mod.rs`](../../crates/v3-core/src/mutation/topology/mod.rs), and
  [`topology/routing.rs`](../../crates/v3-core/src/mutation/topology/routing.rs).
- Graph nodes can arise through backend swaps and mesh copies, but those
  are different routes into a controller. `CopyNode` needs a safe predecessor;
  the founder's entry Graph has none, so the eligible source is its VM.
  `SwapNodeBackend` creates a dormant alternative with the source node's
  targets. On the founder this is an empty terminal Graph, with no onward
  route to the working VM:
  [`topology/structural.rs`](../../crates/v3-core/src/mutation/topology/structural.rs).
- Ordinary node-internal events choose VM, Graph, or input-reference domains
  with equal probability. More VM nodes do not themselves give the VM domain
  a larger mutation share:
  [`mutation/engine/mod.rs`](../../crates/v3-core/src/mutation/engine/mod.rs).

The run's cumulative operator counters include 358,320 `AddNode`, 358,359
`SpliceNode`, and 690,242 `AddRouteTarget` events: 1,406,921 applications of
the three detour-producing operators. The current implementation makes
every such detour a VM. These counters are not counts of retained nodes,
and do not establish the producing binary's entire configuration history.

## Paired probes

Each listed operator received 200 independent applications to the founder,
with seeds `20260908 + trial`. All applied. New-node execution was read
over the same battery.

| Operator | New Graph / VM nodes, totals | New Graph / VM nodes executed, totals |
| --- | ---: | ---: |
| AddNode | 0 / 200 | 0 / 200 |
| SpliceNode | 0 / 200 | 0 / 200 |
| AddRouteTarget | 0 / 200 | 0 / 163 |
| CopyNode | 0 / 200 | 0 / 0 |
| SwapNodeBackend | 200 / 0 | 0 / 0 |
| CopyMeshBackwardSlice | 200 / 97 | 0 / 0 |
| CopyMeshForwardSlice | 103 / 200 | 0 / 0 |

All original offspring had the founder's action signature. For the first
three operators, replacing just the new VM backend with the existing empty
Graph backend preserved all 200 signatures per operator. On the same 200
live genomes, one paired `AddNode` insertion per genome, seeded
`30260908 + sample_index`, was silent for both backend choices in 200/200
cases. No simulation or production source was modified.

Conversely, forcing the founder's dormant backend-swap alternative to run
produced all-NoOp behavior in 200/200 cases. This diagnoses an activation
barrier in that specific path; it is not a claim that Graph backends cannot
emit actions. Giving an empty Graph the original successor avoids that
terminal dead end, exactly as the existing VM detour does.

These measurements establish action-level feasibility. They do not cover
arbitrary queue, payload, bid, or memory states; energy exhaustion; the hop
limit boundary; runtime performance; or descendants adapting under ecological
selection. Those are implementation verification and follow-up readings,
not conclusions of the probe.

## Existing options and research

Primary sources checked on 2026-09-08:

- Schaper and Louis, [The Arrival of the Frequent](https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0086635),
  2014, abstract and introduction: modeled differences in how often mutations
  generate phenotypes can steer evolutionary outcomes even when rarer
  alternatives have higher fitness. This supports separating mutation supply
  from selection; it does not quantify Petri's bias.
- Kelly and Heywood, [Emergent Tangled Graph Representations](https://web.cs.dal.ca/~mheywood/OpenAccess/open-kelly17a.pdf),
  2017, Section 4.2: modular controllers grow while a decision follows one
  path and never revisits a team. This supplies a counterexample to the claim
  that single-path execution inherently prevents modularity; its search and
  selection system differs from Petri's ecology.

| Option | Material finding | Decision |
| --- | --- | --- |
| Extend the existing detour constructor to choose Graph or VM equally | Same successor and attachment semantics; existing blank Graph passes the paired action probe; small change without new dependencies | Adopt as T11.F18 |
| Match the predecessor's backend, or duplicate its working code | Would retain founder/composition biases; duplication has different side effects and already has its own operators | Do not use as the replacement for neutral detour creation |
| Raise Graph-domain mutation rates or change costs | Cannot make the three VM-only constructors produce Graph nodes; confounds creation with refinement and selection | Leave rate characterization to T11.F13 |
| Replace the mesh or introduce parallel node execution | Current sample already routes conditionally; the measured defect is in constructors | No demonstrated need |

Equal creation odds are a scoped engineering choice, not an optimum derived
from the papers. The residual question is whether newly accessible Graph
detours become useful and persist. Read creation, execution, and contribution
separately at closure; do not turn a desired visual topology into a fitness
reward or a population quota.

The standalone probe and raw samples were kept temporarily under
`/tmp/petri-node-probe`; the linked results preserve per-creature count
readings and operator outcomes. The implementation session should use the
repository's existing battery and property-test infrastructure rather than
promote this throwaway probe into another workflow.
