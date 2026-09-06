# Mesh Evolvability Research Note

**Status**: Research (companion to the [T11 track](../roadmaps/t11-brain-genotype-phenotype-map.md) and the [brain evolvability audit](brain-evolvability-audit-2026-09-04.md); not executable roadmap guidance)
**Date**: 2026-09-06
**Code**: baseline probes on `main` at c387e423 (T11.F06 closed); counterfactual probes in a scratch worktree at 43d966c1; V3Alpha1 founder, production defaults
**Method**: code reading; re-analysis of the stored goal reports; a functional census probe run on mutation-only walks and on one whole goal-profile population; three cumulative counterfactual patches to the mesh operators and executor, each re-read with the same probes; primary literature read in full text (fourteen papers, listed in Section 6)
**Prompted by**: an observed `serve` run after T11.F02 through T11.F06 in which VM and graph structure visibly evolved while the mesh did not
**Addendum**: Section 10 records the arXiv scan run later on 2026-09-06 and the roadmap amendments it produced; Sections 1 to 9 are unchanged from the morning draft
**Prior draft**: an Opus session earlier the same day wrote a structure-only note and two roadmap edits; the user discarded that draft on 2026-09-06 once this note superseded it. Its four structural probes are reproduced here (Section 3.4), the two numbers taken from it that were not re-run are quoted where used (Sections 4 and 9), and its diagnosis is corrected by the functional census and the counterfactuals

## Question

Does the roadmap already address the mesh's failure to evolve complexity, and if not, which evolution mechanisms should be added or tuned, and which mesh properties, topology rules, or execution-contract terms should change?

## Verdict

Partly addressed, and the missing part is now measured and its repair demonstrated in a counterfactual.

No roadmap feature owns the mesh routing operators. Nothing scheduled measures what the mesh executes, only what is structurally reachable. Under selection at the goal horizon, not one of 11,024 evolved creatures routes conditionally on its input, although a quarter of them execute a node carrying a second branch. Under mutation alone to generation 1,000, the same holds for 200 of 200 lineages.

The cause is not the single-successor chain, which Tangled Program Graphs show can grow deep modular controllers. It is that a Petri route branch is two halves in two genome domains: the route target lives in the topology and the gate write that could select it lives inside the node's backend, and no operator creates them together. Making a branch neutral to activate is not enough; the counterfactual shows conditional routing appears only when a branch is born with its own bid, as a TPG edge is. Adding TPG's single-visit rule then removes the hop-cap loops that today turn a fraction of every population into `NoOp` creatures. With both in place, one goal-profile seed goes from zero creatures routing conditionally to one in six, at the same population and horizon.

| Reading | Baseline | After counterfactuals 1 to 3 |
| --- | --- | --- |
| Drift lineages at generation 1,000 whose route varies with input (48 scenarios) | 0 of 200 | 11 of 200 (5.5%) |
| Drift lineages at generation 50, same | 0 of 200 | 16 of 200 (8.0%) |
| Executed mesh nodes at generation 1,000, mean (reachable mean) | 2.14 (8.72) | 3.19 (8.96) |
| Drift lineages at generation 1,000 hitting the 1,024-hop cap on some scenario | 10.0% | 0.0% |
| Founder mutated births silent / changed / dead (of 208) | 83 / 116 / 9 | 79 / 123 / 6 |
| Evolved creatures at the goal horizon (seed 11, 2,000 ticks, median generation 22 to 23) whose route varies with input | 0 of 11,024 | 1,832 of 11,242 (16.3%) |
| Evolved creatures executing a node that writes a route gate | 1.3% | 29.6% |
| Final population at that horizon | 11,024 | 11,242 |
| Evolved topology events, changed and dead (T11.F06 goal report) | 0.110 / 0.182 | not re-read under selection (needs the indicator's evolved half; T11.F14) |

## 1. Problem, constraints, decision criteria

**Problem.** A creature's mesh is a set of nodes (VM programs or CGP graphs) with route targets. Each tick the chain runs from `entry_node_id`; each node's gate scores pick one successor; the chain ends at a terminal instruction, a missing target, or the hop cap ([`runtime/mesh.rs`](../../crates/v3-core/src/runtime/mesh.rs)). The founder is two nodes. Evolved creatures still look like two nodes with decoration.

**Constraints carried from the program** ([roadmap notes](../roadmap.md#notes-for-ai-agents)): ecological selection only, no global fitness or speciation; every mechanism names a natural analog and reaches creatures through world or body; seeded runs stay byte-for-byte reproducible; the neutral scaffold delivered by T11.F04 stays; the single-successor chain stays unless a measured gap implicates it; brain execution is on the hot path at about ten thousand creatures.

**Decision criteria.** Whether an option supplies a small step where the code has none; whether growth is neutral both when it fires and when it activates; whether activation can make behavior depend on the input; hot-path cost; compatibility with the node-type contract and the no-regression rule; whether it is a bounded T11 feature rather than a substrate change.

## 2. What the roadmap covers today

| Item | Covers | Does not cover |
| --- | --- | --- |
| Node-type contract ([`v3-mutation-spec.md`](../reference/v3-mutation-spec.md), "Node-type evolvability contract") | Growth class names the topology growth operators (`AddNode`, `CopyNode`, mesh slices, `SpliceNode`) and requires them neutral when they fire | The connection class lists graph and InputRef operators only; `AddRouteTarget`, `MutateGateBias`, `RetargetNodeTarget`, `RemoveRouteTarget`, `RemoveNode`, `SwapNodeBackend`, `ChangeEntryNode`, `SwapRouteTargets` are in neither class and have no owner |
| T11.F08 duplication and module growth | Qualifies the existing copy operators, including mesh copies, for reference preservation and copy-and-divergence | Branch activation; the copies already read 0.89 to 1.00 silent |
| T11.F11 label-addressed control flow | Nearest-match addressing inside the VM ISA, conditional on VM readings | Mesh route targets, which are `NodeId`s; its trigger cannot fire on mesh evidence |
| T11.F10 memory-motif evolvability | Discovery and retention of remembered decisions from founders | Runs on whatever mesh substrate exists when it starts |
| T11.F12 neutral-network characterization | Neutral structure along attributed lineages | Unscheduled until T08.F02 |
| T09.F08, T09.F01 | Diagnostic checkpoint and substrate qualification; a substrate gap is handed to T11 as one bounded feature | Deferred proof phase; no mesh measure of their own |
| T03.F08 | Maintenance cost for larger or more active controllers | The only place a per-hop or per-node cost could live; explicitly after measured costs |
| T02.F01, T02.F03 | Seasons and regional offsets: the world's own modularly varying pressure (Section 6, Kashtan and Alon) | Nothing in T02 reads the mesh |
| Audit reconciliation ([companion](brain-evolvability-audit-2026-09-04.md#the-mesh-vision-and-the-node-type-contract)) | Keeps the single-successor chain; defers fan-out to a measured gap | How a route branch turns on; self-loops and the hop cap |
| Mesh execution spec ([`v3-mesh-execution-spec.md`](../reference/v3-mesh-execution-spec.md)) | "No visited set is used; self-loops are legal"; cap at 1,024 hops returns `NoOp` | Any statement of what a legal self-loop is for |
| Goal-profile indicators (T01.F12, T11.F01) | `functional_complexity`, `reachable_node_count` per sampled genome | Executed nodes, route variation, hop-cap hits, total node count, generation depth |

The repairs landed for VM and graph edits in T11.F02 and T11.F03 were never mirrored for the mesh layer, and no scheduled feature will notice, because no indicator reads mesh execution and no floor names a topology operator.

## 3. Local evidence

### 3.1 The goal series never moved at the mesh layer

`reachable_node_count` is the number of mesh nodes reachable by breadth-first search over `targets` from the entry node ([`genome/analysis.rs`](../../crates/v3-core/src/creature/genome/analysis.rs), `mesh_reachable_nodes`). On the twelve sampled genomes per seed in each goal report:

| Closure | Reachable mesh nodes across 36 sampled genomes | Mesh hops per creature-tick, seeds 11 / 22 / 33 |
| --- | --- | --- |
| T11.F01 (unrepaired baseline) | 2: 27, 3: 4, 4: 5 | 3.09 / 3.27 / 3.10 |
| T11.F02 | 1: 1, 2: 20, 3: 11, 4: 4 | 2.97 / 2.97 / 3.20 |
| T11.F03 | 2: 29, 3: 4, 4: 3 | 2.94 / 2.95 / 3.11 |
| T11.F04 | 2: 25, 3: 6, 4: 4, 5: 1 | 3.62 / 3.10 / 3.12 |
| T11.F06 | 2: 26, 3: 6, 4: 3, 6: 1 | 3.42 / 3.37 / 3.21 |

The founder executes exactly two hops. The population mean of about three hops per creature-tick is not evidence of longer working chains: Section 3.5 shows the median evolved creature still runs two hops and the mean is lifted by the fraction whose chain loops until the 1,024-hop cap and returns `NoOp`.

The composite `reachable_structure_size_distribution` (median about 100, founder 96) is `functional_complexity`: one per reachable node plus its targets, live VM instructions, referenced constants, and consumed input references. With two reachable nodes almost all of it is inside those nodes. It measures VM and graph growth and says nothing about the mesh.

### 3.2 The mesh operator family is inverted relative to VM and graph

Evolved-genome half of the T11.F06 goal report, three seeds pooled, 240 trials per operator per seed:

| Family | Applied | Silent | Changed | Dead |
| --- | ---: | ---: | ---: | ---: |
| vm | 9,729 | 0.690 | 0.306 | 0.004 |
| graph | 9,913 | 0.674 | 0.326 | 0.000 |
| input_ref | 2,880 | 0.473 | 0.527 | 0.000 |
| **topology** | 9,560 | 0.707 | **0.110** | **0.182** |

Per topology operator (same report; `SwapRouteTargets` skipped 520 of 720 draws because it needs two targets on a node):

| Operator | Weight of 27 | Silent | Changed | Dead | Reading |
| --- | ---: | ---: | ---: | ---: | --- |
| `AddNode` | 1 | 1.000 | 0.000 | 0.000 | disconnected `Halt` node; never activates |
| `AddRouteTarget` | 2 | 1.000 | 0.000 | 0.000 | branch at `gate_bias -1.0` on an unwritten slot; never activates |
| `CopyNode` | 1 | 0.893 | 0.042 | 0.065 | backlink hangs off the copied node itself |
| `CopyMeshBackwardSlice`, `CopyMeshForwardSlice` | 1 + 1 | 0.992 | 0.003 | 0.005 | copies land on a losing slot |
| `SpliceNode` | 1 | 0.956 | 0.044 | 0.000 | the one activation-neutral growth operator |
| `MutateGateBias` | 4 | 0.992 | 0.008 | 0.000 | the intended small step; cannot cross a 1.0 handicap in one draw |
| `SwapRouteTargets` | 4 | 0.690 | 0.110 | 0.200 | the only one-step activator; usually skipped |
| `RetargetNodeTarget` | 2 | 0.599 | 0.017 | 0.385 | destination drawn uniformly from all nodes, self included |
| `RemoveRouteTarget` | 2 | 0.335 | 0.008 | 0.657 | removes a sole target as readily as a losing one |
| `RemoveNode` | 1 | 0.179 | 0.000 | 0.821 | removes live nodes as readily as junk |
| `SwapNodeBackend` | 1 | 0.197 | 0.411 | 0.392 | replaces a live backend with a blank one |
| `ChangeEntryNode` | 2 | 0.064 | 0.899 | 0.037 | whole-brain jump |
| `RewriteNodeId` | 4 | 1.000 | 0.000 | 0.000 | renames a node and remaps references; semantic no-op |

### 3.3 Supply arithmetic

Production defaults ([`config/simulation.rs`](../../crates/v3-core/src/config/simulation.rs)): 0.44 trigger with a 0.2 geometric tail, so 0.55 requested events per birth; `mesh_layer_probability` 0.2, so 0.11 topology events per birth; `reachable_bias.topology` 0.0, so targets are drawn uniformly over all nodes, live or junk. Topology weights sum to 27 ([`mutation/topology/mod.rs`](../../crates/v3-core/src/mutation/topology/mod.rs)):

| Class | Operators | Share of mesh events |
| --- | --- | ---: |
| Neutral growth | `AddNode`, `CopyNode`, both slices, `SpliceNode` | 18.5% |
| Neutral branch | `AddRouteTarget` | 7.4% |
| Refinement, silent unless a node has two targets | `MutateGateBias`, `SwapRouteTargets` | 29.6% |
| Semantic no-op | `RewriteNodeId` | 14.8% |
| Destructive macro | `RemoveNode`, `RemoveRouteTarget`, `ChangeEntryNode`, `SwapNodeBackend`, `RetargetNodeTarget` | 29.6% |

The engine redraws when an operator reports no applicable target ([`mutation/engine/mod.rs`](../../crates/v3-core/src/mutation/engine/mod.rs)), so `SwapRouteTargets` skips are redistributed rather than lost; `RewriteNodeId` applies and does nothing, so its share is lost outright.

### 3.4 Structure and function under mutation alone (Probe F2)

Two hundred independent lineages walked from the founder with the production mutation engine and no selection. Structure columns agree with the discarded draft's Probe C to within the difference in food-type count; the functional columns are new. "Executed" is the union of node ids visited over 48 fixed sensor scenarios drawn the way the T11.F01 battery draws them, with fresh memory and graph state per scenario; "route varies" means some node selected two different target positions across those scenarios.

| Generation | Total nodes, mean | Reachable, median / mean | Executed, median / mean | Reachable but never executed | Executed node with 2+ targets | Route varies with input | Any scenario hits the hop cap |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 50 | 3.16 | 2 / 2.00 | 1 / 1.56 | 26.5% | 28.0% | 0.0% | 12.5% |
| 250 | 8.37 | 2 / 3.05 | 1 / 1.69 | 46.5% | 46.0% | 0.5% | 18.0% |
| 1,000 | 36.98 | 4 / 8.72 | 1 / 2.14 | 55.5% | 53.0% | 0.0% | 10.0% |

By generation 1,000 the median lineage executes one node: the founder's two-node chain has been cut to a single node acting alone, junk has piled up around it, and no lineage routes conditionally. `AddNode` and `AddRouteTarget` are working exactly as written, and what they produce is a reachable graph that is not a working graph.

### 3.5 Function under selection at the goal horizon (Probe F1)

Goal seed 11 (1600 by 1600, 10,000 founders, 2,000 ticks, production defaults), the whole final population of 11,024 creatures (the count the T11.F06 goal report records for this seed), each read on the same 48 scenarios:

| Quantity | min | p25 | median | p75 | max | mean |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| generation | 1 | 17 | 22 | 27 | 45 | 22.0 |
| age, ticks | 0 | 31 | 102 | 255 | 1,969 | 202.5 |
| total mesh nodes | 1 | 2 | 2 | 3 | 11 | 2.51 |
| reachable mesh nodes | 1 | 2 | 2 | 3 | 10 | 2.37 |
| executed mesh nodes | 1 | 2 | 2 | 2 | 4 | 2.08 |
| chain length, per-creature median over scenarios | 1 | 2 | 2 | 2 | 1,024 | 2.73 |

| Fraction of the 11,024 creatures | Value |
| --- | ---: |
| reachable node that never executes | 20.8% |
| executed node with two or more route targets | 23.2% |
| executed node that writes a route gate | 1.3% |
| executed node that reads or writes shared memory | 48.2% |
| any scenario hits the 1,024-hop cap | 0.12% |
| **route varies with input** | **0.0% (0 of 11,024; zero in every reachable-count bucket from 1 to 10)** |

The goal profile is 22 generations deep at the median and 45 at the maximum, so mesh growth of any kind has barely started at closure time. Branches do form under selection, in nearly a quarter of creatures, but none is ever taken, and only 1.3% of creatures execute a node that writes a route gate at all.

## 4. Where the brittleness comes from

Each item was read in code and, where a probe reaches it, confirmed above or in Section 5. Paths are relative to `crates/v3-core/src`.

1. **Activation is two events that must co-occur.** `AddRouteTarget` appends at `gate_bias -1.0` on the lowest unused gate slot ([`mutation/topology/routing.rs`](../../crates/v3-core/src/mutation/topology/routing.rs)). Routing is argmax over `gate_bias + score[slot]` with ties to the earlier position ([`runtime/routing.rs`](../../crates/v3-core/src/runtime/routing.rs)), and a slot's score is nonzero only when the node's own backend writes it: `WriteRouteGate` in the VM ([`runtime/vm.rs`](../../crates/v3-core/src/runtime/vm.rs)), a `RouterGate` sink in the graph ([`runtime/cgp/effects.rs`](../../crates/v3-core/src/runtime/cgp/effects.rs)). `MutateGateBias` at 0.992 silent is that arithmetic.
2. **Activation is not neutral.** When a branch does win, its destination is a random existing node or a blank `AddNode` node. A blank node is `Halt` with no targets, so the chain ends there and returns `NoOp` when the queue is empty; a random node is the entry or the terminal, so the chain loops or re-runs; a self-target loops to the 1,024-hop cap, pays graph node cost on every visit, and returns `NoOp`. NEAT's add-node preserves function at the moment it fires; a TPG pointer targets a whole working team. Only `SpliceNode` is neutral at activation, and it is 3.7% of mesh events.
3. **A branch has no bid of its own.** This is the defect the counterfactuals isolate (Section 5). In TPG the program that computes the bid *is* the edge; creating an edge creates its bid. In Petri the branch is a `RouteTarget` in the topology domain and the bid is a gate write inside the backend, in the VM or graph domain, and the two are mutated by different operator families with no operator that produces both. A branch that is neutral to activate but has no bid can only ever be switched *on*, by bias or swap, never made *conditional*; the route then changes but does not vary with the input. Repairing items 1 and 2 alone left route variation at zero (Section 5.2).
4. **Copies hang off the wrong node.** `CopyNode` adds the backlink from the copied node to its copy ([`mutation/topology/structural.rs`](../../crates/v3-core/src/mutation/topology/structural.rs)). A copy of the terminal node is never executed, and a copy of the entry becomes entry, copy, terminal in series. A paralog that can take over must hang off the original's predecessor as an alternative successor.
5. **`RewriteNodeId` spends 14.8% of mesh supply doing nothing.**
6. **Five macro operators are destructive by construction.** `ChangeEntryNode`, `SwapNodeBackend`, `RemoveNode`, `RemoveRouteTarget`, and `RetargetNodeTarget` have no small form: none prefers junk, none bypasses, none draws a nearby destination, and two allow self-targets.
7. **Self-loops are legal and free.** The mesh spec keeps them legal; a looping chain runs to the cap and returns `NoOp`, costing a graph node `1e-5` per visit and a VM opcode a millionth of its table cost, so the creature does nothing that tick and dies slowly rather than at once. Under drift 10 to 18% of lineages carry such a loop on some scenario; under selection 0.12% of the final population still does.
8. **Reachability is the wrong mesh indicator.** `reachable_node_count` follows every target whether or not it ever wins its gate; Section 3.4 shows executed nodes at a quarter of reachable ones at depth. A mesh indicator has to count executed nodes and route variation or it will report growth that does not exist.
9. **Routing has no memory of its own.** The chain restarts from `entry_node_id` every tick. That is a design choice consistent with memory living in shared memory and graph state, not a defect; Section 7 records the alternative that Genetic Network Programming and PADO use.
10. **Junk is free by design.** No per-hop or per-node maintenance cost exists; T11.F04 relies on that for the neutral scaffold. The discarded draft's Probe C (a 200-lineage mutation-only walk, same seeds as Section 3.4) showed the consequence at depth: because targets are drawn uniformly, the live core's share of mutation decays from 0.90 at generation 10 to 0.18 at generation 1,000. A known trade, not a T11 repair target.

## 5. Counterfactuals

Three cumulative patches in a scratch worktree (Appendix B), each re-read with Probe F2 on the same 200 lineages and Probe F3 (founder operator rows and 500 production births, the gate-profile sizes). None is proposed as written; they isolate which change moves which reading.

- **CF1, activation-neutral connection semantics.** `AddRouteTarget` seeds the new branch at the incumbent's bias in a later position (tied, so it loses) and points it at a destination that already works: a pass-through detour node forwarding to the incumbent successor, or a paralog copy of that successor. `CopyNode` hangs the copy off a predecessor as a tied-losing alternative. `RetargetNodeTarget` draws from the successor's own targets or the node's other targets and never from itself. `RemoveNode` prefers unreachable nodes and otherwise bypasses (the inverse of splice). `RemoveRouteTarget` removes losing branches only. `RewriteNodeId` weight zero.
- **CF2, the branch is born with its own bid.** On top of CF1, `AddRouteTarget` also wires the new branch's gate slot: a graph node gets one weight-1 edge from a random source (compute node, sensor sub-value, or shared memory slot, drawn by the production sampler) into that slot's `RouterGate` sink; a VM node gets `WriteRouteGate` from a random register inserted with reference repair before its first terminal instruction. Because the destination is behaviorally identical to the incumbent, the branch is still silent when it fires and when its bid first wins.
- **CF3, single-visit routing.** On top of CF2, the executor keeps a per-tick visited set: a route to an already-visited node falls through to the next-best unvisited target, and the chain ends when none remains. This is TPG's rule verbatim (Section 6).

### 5.1 Drift walk, 200 lineages (Probe F2), by stage

| Generation | Stage | Total nodes, mean | Reachable, mean | Executed, mean | Executed node with 2+ targets | Executed node writes a route gate | **Route varies with input** | Hop-cap hit |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 50 | baseline | 3.16 | 2.00 | 1.56 | 28.0% | 2.5% | 0.0% | 12.5% |
| 50 | CF1 | 4.42 | 2.48 | 1.83 | 31.0% | 3.5% | 0.0% | 0.5% |
| 50 | CF2 | 4.45 | 2.50 | 1.91 | 31.0% | 26.5% | **8.0%** | 1.0% |
| 50 | CF3 | 4.45 | 2.50 | 1.91 | 31.0% | 26.5% | **8.0%** | 0.0% |
| 250 | baseline | 8.37 | 3.05 | 1.69 | 46.0% | 11.5% | 0.5% | 18.0% |
| 250 | CF1 | 14.20 | 4.02 | 2.01 | 42.0% | 6.0% | 0.0% | 9.0% |
| 250 | CF2 | 14.51 | 3.85 | 2.05 | 37.5% | 33.5% | **5.5%** | 7.5% |
| 250 | CF3 | 14.51 | 3.85 | 2.09 | 37.5% | 34.0% | **4.5%** | 0.0% |
| 1,000 | baseline | 36.98 | 8.72 | 2.14 | 53.0% | 11.0% | 0.0% | 10.0% |
| 1,000 | CF1 | 56.92 | 7.67 | 2.60 | 49.0% | 14.5% | 0.0% | 12.5% |
| 1,000 | CF2 | 56.43 | 8.96 | 3.02 | 49.0% | 50.0% | **6.0%** | 14.0% |
| 1,000 | CF3 | 56.43 | 8.96 | 3.19 | 49.0% | 50.0% | **5.5%** | 0.0% |

CF3's structure columns equal CF2's because the visited rule changes execution, not mutation; the identical values are a consistency check on the probe. Executed median rises from 1 (baseline) to 2 (every counterfactual) at all three depths.

### 5.2 What each stage shows

- **CF1 keeps the chain alive but cannot make it conditional.** The founder's two-node chain survives drift (executed median 2 instead of 1), total structure grows faster because every branch now creates a working node, and route variation stays at exactly zero through generation 1,000. A branch that is neutral to activate can be switched on by a bias step or a swap, and then it is on forever. Neutral activation is necessary and insufficient.
- **CF2 is the step that matters.** Giving the branch its own bid moves route variation from 0 to 8.0% of lineages at generation 50 and holds it at 5 to 6% through generation 1,000, raises the share of lineages executing a gate-writing node from 11% to 50%, and raises executed nodes to a mean of 3.0 at depth. The founder rows are unchanged (`AddRouteTarget` still 50 of 50 silent), as designed: the branch fires silently and its bid, when it wins, selects a detour or paralog that behaves like the incumbent, so behavior stays identical until later mutation differentiates the new node. The bid is sensor-driven only when the sampler draws a sensor sub-value or a compute node fed by one; shared-memory sources read zero on a fresh battery and never vary, which bounds the fraction.
- **CF3 removes the loop failure completely.** Hop-cap hits fall to zero at every depth with no other reading harmed; executed nodes rise slightly more because a chain that used to loop now continues to an unvisited node. The spec sentence "no visited set is used; self-loops are legal" has no measured benefit to trade against this.

### 5.3 Founder battery (Probe F3), by stage

| Stage | Mutated births of 500 | Silent / changed / dead | Single-event births: silent / changed / dead |
| --- | ---: | --- | --- |
| baseline (T11.F06 gate report) | 208 | 83 / 116 / 9 | 72 / 86 / 6 (of 164) |
| CF1 | 208 | 81 / 121 / 6 | 71 / 89 / 4 |
| CF2 and CF3 | 208 | 79 / 123 / 6 | 71 / 89 / 4 |

Founder-level operator rows barely move, which is the expected reading on a two-node mesh: `AddRouteTarget` and `CopyNode` stay 50 of 50 silent (both were already silent, now for the right reason); `RetargetNodeTarget`, `RemoveRouteTarget`, and `SwapRouteTargets` skip, because a two-node mesh has no local alternative, no losing branch, and no second target; `RemoveNode` stays 50 of 50 dead, because removing the founder's only actor cannot be bypassed into anything. Dead births fall from 9 to 6 of 208. The counterfactuals are for evolved meshes, and the founder battery is the wrong instrument to grade them, which is itself a finding for T11.F14: the evolved half must carry the mesh components.

### 5.4 Selection at the goal horizon under CF3 (Probe F1)

The same goal seed, ticks, founders, and defaults as Section 3.5, run on the cumulative CF3 build; the whole final population of 11,242 creatures read on the same 48 scenarios.

| Quantity | Baseline (Section 3.5) | CF3 |
| --- | ---: | ---: |
| final population | 11,024 | 11,242 |
| generation, median / max | 22 / 45 | 23 / 49 |
| total mesh nodes, median / mean | 2 / 2.51 | 3 / 3.31 |
| reachable mesh nodes, median / mean | 2 / 2.37 | 3 / 2.81 |
| executed mesh nodes, median / mean | 2 / 2.08 | 2 / 2.29 |
| chain length, per-creature median over scenarios, max / mean | 1,024 / 2.73 | 5 / 2.14 |
| reachable node that never executes | 20.8% | 34.3% |
| executed node with two or more route targets | 23.2% | 43.5% |
| executed node that writes a route gate | 1.3% | 29.6% |
| executed node that reads or writes shared memory | 48.2% | 40.9% |
| any scenario hits the hop cap | 0.12% | 0.0% |
| **route varies with input** | **0.0%** | **16.3% (1,832 of 11,242)** |

Route variation by reachable node count under CF3: 2 nodes 0.6% (34 of 5,514), 3 nodes 26.8% (902 of 3,371), 4 nodes 30.9% (518 of 1,674), 5 nodes 59.0% (206 of 349), 6 nodes 51.2% (124 of 242), 10 nodes 100% (37 of 37). Every bucket was zero at baseline.

Three things follow. Persistence is unchanged at this horizon (11,242 against 11,024 final creatures, births not separately recorded), so the repairs cost nothing the goal profile can see. Conditional routing is three times as common under selection (16.3%) as under drift at comparable depth (8.0% at generation 50), which is the first evidence that a branch with its own bid is kept once it exists, not merely tolerated; whether it is *used* is T11.F10's question. And the fraction of creatures whose reachable nodes include one that never executes rises (20.8% to 34.3%), because paralog and detour nodes are born reachable and losing, which is the neutral scaffold working as designed and one more reason the indicator must count execution rather than reachability.

## 6. What established systems do

Every row is from full text read on 2026-09-06 unless marked; quotations are verbatim.

| System | Design choice | Evidence | Petri today |
| --- | --- | --- | --- |
| **Tangled Program Graphs**, [Kelly and Heywood, EuroGP 2017](https://web.cs.dal.ca/~mheywood/OpenAccess/open-kelly17a.pdf); [IJCAI 2018](https://www.ijcai.org/proceedings/2018/0740.pdf); [Kelly, Smith, Heywood, Banzhaf, ACM TELO 2021](https://dl.acm.org/doi/10.1145/3468857); [Gegelati, Desnos et al. 2021](https://arxiv.org/pdf/2012.08296) | A team is a vertex; each program is an edge with a bid and an action that is atomic or "a pointer to another team." "The modified action has an equal probability of referencing either an atomic action or another team." Decision making "follows one path through the network until an atomic action is selected." "Cycles may exist in the graph, but they are never followed during execution. That is, a team is never visited twice per decision ... If so, the next highest bid is selected." "Only root teams are subject to modification by the variation operators." Offspring "are created by cloning the team along with all its programs." Team operators: `pmd`, `pma` 0.7; `pmm` 0.2; `pmn` 0.1 (change a program's action). | Champion policies grow from one team to about 60 over 2,000 generations; "policies are initialized in their simplest form and only complexify when/if simpler solutions are outperformed." Multi-task Atari agents visit 3 to 6 teams and execute 588 to 1,244 instructions per decision. With indexed memory, graphs "may subsume up to 60 teams per graph" while "typically only one to nine teams are visited per timestep." | The same one-path argmax chain, so the chain is not the block. But a TPG edge is created *with* its bid; Petri's branch is created without one (Section 4, item 3). TPG's single-visit rule is the CF3 patch. TPG never mutates subsumed interior teams; Petri mutates uniformly. |
| **NEAT**, [Stanley and Miikkulainen, Evolutionary Computation 2002](https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf) | Each connection gene carries "whether or not the connection gene is expressed (an enable bit)." "In the add node mutation, an existing connection is split and the new node placed where the old connection used to be. The old connection is disabled and two new connections are added." Rates: add node 0.03, add link 0.05 (0.3 in the large population). "There was a 75% chance that an inherited gene was disabled if it was disabled in either parent." | "Adding nodes and connections usually initially decreases the fitness of the network," so "it is necessary to somehow protect networks with structural innovations," which NEAT does by speciation. On the alternative: "The GNARL system addresses the problem of protecting innovation by adding nonfunctional structure. A node is added to a genome without any connections, in the hopes that in the future some useful connections will develop. However, nonfunctional structures may never end up connecting to the functional network, adding extraneous parameters to the search." | Petri has no speciation, so structure must be neutral at activation, not merely at birth. `AddNode` is GNARL's operator, and Section 3.4 is GNARL's failure mode measured. |
| **Markov Brains**, [Hintze et al. 2017](https://arxiv.org/pdf/1709.05601) | A genome of up to 20,000 sites, "most sites are non-coding," gates read from start codons; "we allow for point mutations, insertions, and gene duplications"; "a 20% chance for a section of 256 to 512 sites to be copied and randomly inserted anywhere in the genome." All gates read the state buffer at *t* and write *t + 1* in parallel. Gate kinds include timer gates and feedback gates whose history window "is an evolvable property of the gate." | Position-independent gates make large duplications routine; the blackboard updated once per step is the memory. | Petri's graph clock (T11.F06) now matches the once-per-step rule; the mesh's slot baton is per hop. Gate kinds are a candidate node kind under the contract, not a mesh fix. |
| **SignalGP**, [Lalejini and Ofria, GECCO 2018](https://arxiv.org/pdf/1804.05445); [SignalGP-Lite, 2021](https://arxiv.org/pdf/2108.00382) | "Both events and functions are labeled with evolvable tags; when an event occurs, the function with the closest matching tag is triggered," above a similarity threshold, similarity being "the proportion of matching bits." "The SignalGP virtual hardware supports an arbitrary number of execution threads that run concurrently." Mutation: "whole-function duplication and deletion operators (applied at a per-function rate of 0.05)," tags "at a per-bit mutation rate (0.05)," instructions at 0.005. | Tags let architecture change "while still guaranteeing syntactic correctness." SignalGP-Lite reports "an 8x to 30x speedup" by "reducing control flow overhead and trading run-time flexibility for better performance," which is what large-population fan-out costs. | Tag-addressed route targets remain the durable answer to uniform-random retargeting, with nothing to sort until meshes carry nodes; fan-out has a measured cost and no measured gap. |
| **Genetic Network Programming**, [Mabu, Hirasawa, and Hu, Evolutionary Computation 2007](https://direct.mit.edu/evco/article-abstract/15/3/369/1274/A-Graph-Based-Evolutionary-Algorithm-Genetic) (abstract, via [science.gov](https://www.science.gov/topicpages/g/genetic+network+programming)) | "The node transition of GNP is executed according to its node connections without any terminal nodes, thus the past history of the node transition affects the current node to be used and this characteristic works as an implicit memory function." Fixed node pool; evolution changes links. | Routing position persisting across steps is memory without a memory slot. | Petri restarts from the entry every tick (Section 7, C3). |
| **PADO**, [Teller and Veloso 1995](https://www.cs.cmu.edu/~mmv/papers/Teller-ESJ.pdf) | "Each node has two parts: an action and a branch-decision." The branch-decision "may use the top of the stack, the previous state number, the memory, and constants to pick an arc." Programs share "150 Library programs ... available to all programs in the population." | The branch decision reads *the previous node visited*: routing history as an input. A population-level module library. | A Petri node's gate cannot see where the chain came from; only the slot baton carries it. |
| **Evolving Virtual Creatures**, [Sims, SIGGRAPH 1994](https://www.karlsims.com/papers/siggraph94.pdf) | Neural node functions include "integrate, differentiate, smooth, memory, oscillate-wave, and oscillate-saw." "A new node normally has no effect on the phenotype unless a connection also mutates a pointer to it. Therefore a new node is always initially added, but then garbage collected later ... if it does not become connected." "Although leaving the disconnected nodes for possible reconnection might be advantageous, and is probably biologically analogous, at least the unconnected newly added ones are removed to prevent unnecessary growth in graph size." Mutation frequencies are "scaled by an amount inversely proportional to the size of the current graph." | The earliest heterogeneous stateful node graph; junk was pruned and per-genome mutation held constant against size. | Petri keeps junk deliberately (the scaffold); Sims' size-scaled mutation is the alternative to a cost for the dilution in item 10. |
| **Connection cost and modularity**, [Clune, Mouret, and Lipson, Proc. R. Soc. B 2013](https://arxiv.org/abs/1207.2743) | Cost is "the summed squared length of all connections, assuming nodes are optimally located"; "a second measure of costs as solely the number of connections yields qualitatively similar results." Applied as an NSGA-II objective that "affects selection probabilistically only 25 per cent of the time." | "Direct selection pressure to reduce the cost of connections between network nodes causes the emergence of modular networks," which are "significantly more modular and more evolvable" than performance-only controls. | A per-connection cost is the count of route targets and edges. It is T03.F08's decision and would tax the scaffold today. |
| **Hierarchy from connection cost**, [Mengistu, Huizinga, Mouret, and Clune, PLOS Comput. Biol. 2016](https://journals.plos.org/ploscompbiol/article?id=10.1371/journal.pcbi.1004829) (fetched summary) | Same cost regime; hierarchy measured as recursive module composition. | Cost networks "took significantly fewer generations to adapt to the new environment" and hierarchy improved evolvability "independently of modularity." | Same as above. |
| **Modularly varying goals**, [Kashtan and Alon, PNAS 2005](https://pmc.ncbi.nlm.nih.gov/articles/PMC1236541/) | "Repeatedly switch between several goals, each made of a different combination of subgoals," every 20 generations. | Fixed goals gave "very low modularity: Qm = 0.12 ± 0.02"; varying goals gave "Qm = 0.54 ± 0.02," and after a switch "the population was able to reach a perfect solution to the new goal within about five generations." | The ecological side of the same question: T02.F01 seasons and T02.F03 regional offsets are Petri's modularly varying pressure. Without it no mesh mechanism will be selected for, whatever the operators do. |
| **Module duplication and specialization**, [Calabretta, Nolfi, Parisi, and Wagner, Artificial Life 2000](https://direct.mit.edu/artl/article-abstract/6/1/69/2340/Duplication-of-Modules-Facilitates-the-Evolution) (abstract) | Robot controllers whose modules arise by duplication, versus hardwired modules and a non-modular net. | "Both modular architectures outperform the non-modular architecture, both in terms of rate of adaptation as well as the level of adaptation achieved"; duplication-based modules specialize more, and "functional specialization may be an evolutionary absorption state." | The paralog branch in CF1 and CF2 is this operator at the mesh level. |
| **Duplication in a genotype-phenotype map**, [2021 study of a non-deterministic Polyomino model](https://pmc.ncbi.nlm.nih.gov/articles/PMC8220273/) | Gene duplication in a self-assembly genotype-phenotype map. | "Duplication increases robustness and reduces evolvability initially, but ... the subsequent diversification that duplication enables has a stronger, inverse effect, greatly increasing evolvability." | Expect a paralog to read silent at birth and to pay off only after divergence; the indicator must not penalize the first half. |
| **Architecture-altering operations**, Koza (via [genetic-programming.com](http://www.genetic-programming.com/jkpubs94.html), site summary) | Subroutine duplication, creation, and deletion "patterned after the naturally occurring operations of gene duplication and gene deletion." | "The offspring produced by a subroutine duplication is semantically equivalent to its parent." | The property the detour and paralog branches have at birth. |
| **Options**, [Sutton, Precup, and Singh, Artificial Intelligence 1999](https://people.cs.umass.edu/~barto/courses/cs687/Sutton-Precup-Singh-AIJ99.pdf) | "Options consist of three components: a policy, a termination condition, and an initiation set." "If the option is taken, then actions are selected according to π until the option terminates." Options may also be interrupted before their termination condition. | The standard formalism for a behavior that runs for several steps until its own condition ends it. | The formal shape of Section 7, C3: a mesh node that stays the entry until its own termination fires. |
| **Evolutionary programming of finite-state machines**, Fogel (audit's 2026-09-05 review; not re-read here) | Add state, delete state, change transition. | The audit records that deleting a state remaps the transitions into it. | `RemoveNode` bypass in CF1 is that rule. |

## 7. Options: what to implement, adjust, or tune, and what to change in the contract

### 7.1 Evolution mechanisms (operators and supply)

| # | Option | Kind | Evidence and fit | Verdict |
| --- | --- | --- | --- | --- |
| A1 | Branch born with its own bid: `AddRouteTarget` wires the branch's gate slot in the same event (graph edge or VM `WriteRouteGate`) | Operator; touches topology and backend domains together | CF2 is the only stage that moves route variation off zero (Section 5.1). TPG: the bid program is the edge. | **Adopt.** The centerpiece of the repair. |
| A2 | Activation-neutral destinations: detour or paralog, seeded tied-but-losing; `CopyNode` hangs off a predecessor | Operator | CF1 keeps the chain alive (executed median 1 to 2) and makes A1 silent at birth. NEAT split-edge; Koza's semantic equivalence; Calabretta's duplication. | **Adopt**, as the substrate A1 needs. |
| A3 | Local retargeting: successor's targets or the node's other targets, never self | Operator | `RetargetNodeTarget` 0.385 dead on evolved genomes; a two-node mesh has no local candidate, which is the right skip. | **Adopt.** |
| A4 | Non-lethal removal: junk first, else bypass; losing branch only | Operator | `RemoveNode` 0.821 and `RemoveRouteTarget` 0.657 dead on evolved genomes. Fogel's state deletion. On the founder the only actor stays unremovable, correctly. | **Adopt.** |
| A5 | Retire `RewriteNodeId`; split `SwapNodeBackend` into a neutral add of a blank alternative on a losing branch plus a small-step swap; keep `ChangeEntryNode` as an explicit macro at weight 1 | Operator weights | 14.8% of mesh supply is a no-op; `SwapNodeBackend` 0.392 dead. No floor forbids retiring a no-op. | **Adopt.** |
| A6 | Rebalance weights toward growth with a bid (A1, splice, copies) and raise `mesh_layer_probability` | Tuning | Weight changes alone do not change the outcome mix (Section 3.2 is per operator). | Fold into A1 to A5; do not tune alone. |
| A7 | Size-scaled mutation supply (Sims) as an answer to dilution (item 10) | Supply policy | Holds per-genome events constant against junk growth; interacts with T11.F04's provisional 0.55 and T11.F13's characterization. | **Defer** to T11.F13 as a treatment arm. |
| A8 | Mutation opportunity by depth: protect interior, mutate near the root (TPG) | Supply policy | TPG's "only root teams are subject to modification" avoids unlearning; in Petri the analog is a reachable-depth weighting, the inverse of uniform targeting. | **Defer**; a T11.F13 or T11.F12 characterization knob, not a repair. |

### 7.2 Mesh properties, topology, and execution contract

| # | Option | What changes | Precedent and natural analog | Cost and risk | Evidence needed | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| C1 | **Single-visit routing** | A node runs at most once per tick; a route to a visited node falls through to the next-best unvisited target; the chain ends when none remains. Replaces "self-loops are legal" in the mesh spec. | TPG ("a team is never visited twice per decision"). Analog: a reflex arc does not re-fire within its own refractory period. | One bitmap per chain on the hot path; behavior changes for every genome that loops, which today is the `NoOp` class. Re-run at max hops becomes unreachable. | CF3: hop-cap hits 0 at every depth, nothing else harmed (Section 5.1). | **Adopt** with A1 to A5. |
| C2 | **Executed-node and route-variation indicator components** | Battery-based executed node count, route-variation flag, hop-cap hits, total node count, generation depth of the final population. | The audit's own rule that the indicator gates any mesh change. | None on the simulation; seconds in the gate profile, part of the evolved half in the goal profile. | Section 3.4 and 3.5: reachability overstates function 4 to 1 at depth. | **Adopt first.** |
| C3 | **Persistent routing position** (temporally extended nodes) | A node may declare itself sticky: while its own termination condition has not fired, next tick's chain starts at it rather than at `entry_node_id`; interruption when the node is missing or its energy exhausts. | GNP's implicit memory; PADO's "previous state number"; options' policy, termination, initiation triple; behavior trees' running state. Analog: a foraging bout continues until interrupted. | Small: one `Option<NodeId>` per creature; deterministic. Interacts with the memory-sensitivity indicators (a new perturbation component) and with T11.F10's question about whether memory is reachable at all. | T11.F10's reading on the existing memory paths; a delayed-cue fixture where a sticky node solves what shared memory did not. | **Defer** until T11.F10 reports; then a bounded T11 feature with its own indicator component. |
| C4 | **Tag-addressed route targets** with nearest match | `target_id` becomes an evolvable tag resolved to the nearest node tag. | SignalGP; Analog Genetic Encoding. | Touches genome, mutation, runtime, frontend; must share one scheme with T11.F11. | Meshes with enough nodes that "nearest" sorts anything; A3 first. | **Contingent** on C2 readings after A1 to A5. |
| C5 | **Per-hop or per-node metabolic cost** | Every hop or every live node costs energy each tick. | Clune 2013; Mengistu 2016. Analog: neurons are expensive. | Would tax the neutral scaffold T11.F04 relies on; T03.F08 owns brain cost after measured realized costs. | T03.F08's measurements. | **Not now**; T03.F08. |
| C6 | **Fan-out** (several nodes active per tick under a hop budget) | Replaces the single path. | SignalGP threads. | SignalGP-Lite's 8 to 30 times speedup came from removing exactly this flexibility; no measured gap. TPG shows one path suffices for deep graphs. | A task where a creature needed two nodes in one tick and could not have it. | **Reject** for now. |
| C7 | **Gate reads the slot bus directly** | A route target scored by an output slot instead of a gate slot. | None. | Conflates data and control; A1 makes it unnecessary. | | **Reject.** |
| C8 | **Population-level module library** | Shared modules any creature can route into. | PADO's 150 library programs; TPG's subsumed root teams. | Needs T08 homology and lineage attribution; ecological selection has no population-level bookkeeping. | T08.F06 evidence. | **Defer** to T08. |
| C9 | **New node kinds** (Markov gate bank, Sims' oscillator or memory node) | Another backend under the contract. | Markov Brains; Sims. | Welcome by the contract; orthogonal to routing. | Contract compliance and floors only. | Not a mesh fix; unchanged. |
| C10 | **Modularly varying pressure from the world** | Nothing in the mesh; T02.F01 seasons and T02.F03 offsets stay where they are in the priority order. | Kashtan and Alon; Clune's remark that bacterial network modularity tracks environmental change frequency. | | | Keep T02 after the mesh repairs so their readings land on a substrate that can route. |

## 8. Recommendation

Two bounded T11 features, in this order, added to the roadmap on 2026-09-06. The discarded Opus draft had proposed the same pair as T11.F14 and T11.F15 with the same priority placement (F14 after T11.F07, F15 before T11.F10); the scope below differs in what F14 measures and in what F15 makes neutral and conditional.

### T11.F14, mesh execution observability (C2)

Goal line: *Generations, not ticks; executed, not reachable.* Record what a lineage's mesh actually runs and how deep the profile reaches, so mesh-layer claims are read at a depth and in a unit that can support them; observation only.

- Per sampled genome, alongside the T11.F01 companions: total node count, reachable count, executed node count over the battery, route-variation flag, hop-cap hits; founder half in the gate profile, evolved half in the goal profile.
- Generation distribution of the final population (median and maximum), so every mesh claim states its depth.
- Keep T01.F12's goal parameters; if the recorded depth argues for a deeper or cheaper profile, hand that to T01 as a finding.
- Serde-default fields; readings in the `deterministic` block.

### T11.F15, mesh routing connection semantics (A1 to A5, C1)

Goal line: *A synapse forms with its own trigger.* A new route branch is born pointing at something that already works and carrying its own bid, so it is silent at birth and conditional from its first win; retargeting lands near, removal is not usually lethal, and a node runs at most once per tick.

1. Branch born with a bid (A1) onto an activation-neutral destination (A2), seeded tied-but-losing; `CopyNode` as paralog off a predecessor.
2. Local retargeting (A3); junk-first removal with bypass and losing-branch-only removal (A4); retire `RewriteNodeId`, split `SwapNodeBackend`, demote `ChangeEntryNode` (A5).
3. Single-visit routing (C1), recorded in the mesh execution spec in place of the self-loop sentence.
4. Taxonomy: place all eight connection operators in the growth-versus-connection classes in `v3-mutation-spec.md` with this feature as owner.
5. Predeclared indicator directions, from Section 5.1: route-variation fraction above zero on the evolved half; executed count up; hop-cap hits to zero; `RemoveNode`, `RemoveRouteTarget`, and `RetargetNodeTarget` dead fractions down on the evolved half; founder rows unchanged except `RewriteNodeId` retired; per-birth dead fraction not up.

No floor is retro-fitted; T11.F15 predeclares its own targets, which then fall under the no-regression rule.

### Recorded but not scheduled

C3 (persistent routing position) after T11.F10; C4 (tags) contingent on C2 readings; C5 (cost) to T03.F08; A7 and A8 to T11.F13; C6, C7 rejected; C8 to T08; C10 a sequencing note for T02. Section 10, added after the arXiv scan later the same day, folds `AddNode` into the detour form (A9) and adds a knockout reading (C11), a silent-fraction predeclaration, a T11.F13 reference arm, and a T02.F01 closure reading.

## 9. Remaining uncertainty and the cheap proofs

- **A conditional route is necessary, not sufficient.** Five to eight percent of drift lineages routing conditionally is a ratchet, not cognition. Whether selection keeps and uses such branches is the question the T11.F14 evolved half and T11.F10 answer; Section 5.4 is the first reading.
- **The bid's source distribution bounds the effect.** The production sampler draws shared memory 20% of the time, which reads zero on a fresh battery; a sensor-biased draw for new bids would raise the fraction and is a one-line decision for the feature spec.
- **Depth.** Every selection reading here is 22 generations deep. A run of a hundred or more generations under selection is a sweep-profile characterization nobody has budgeted; T11.F14 records depth so the gap stays visible.
- **The founder battery cannot grade mesh repairs.** Section 5.3 shows why; the feature's acceptance test has to be the evolved half plus a drift-walk census, both seconds to minutes.
- **The discarded draft's Probe D** found that the pre-T11.F04 reachable bias of 0.7 roughly doubles reachable mesh at generation 1,000 (12.09 against 6.91 reachable nodes) without changing the dilution trend (0.643 to 0.300 in reachable-over-total). That is a T11.F13 knob, not part of this repair.
- **C1 changes semantics for every looping genome.** Today those are `NoOp` creatures; a genome that used a bounded loop as iteration would lose it. Nothing measured shows one exists.
- **GNP** was read at abstract level only; the fixed-pool design is rejected on the roadmap's own terms and nothing here depends on the rest of the paper.

## 10. Addendum: arXiv scan of newer prior art (2026-09-06)

**Method**: the `neuroarxiv` skill, run later the same day. Nine queries to the arXiv export API over cs.NE, cs.AI, cs.MA, nlin.AO, q-bio.PE, cs.LG, and cs.RO with terms for tangled program graphs, neuroevolution neutrality, developmental encodings, open-ended evolution, Cartesian genetic programming, gene duplication and mutation rate, connection cost, tag matching, and evolved behavior trees and state machines; about forty entries returned; entries already in Section 6 and off-target hits (HyperNEAT GPU ports, LLM workflow evolution, behavior-tree tooling, level generators, biology-specific regulatory papers) dropped; seventeen abstracts read in isolation, one reader per paper with no sight of the others; three papers then read in full text and quoted below. Abstract-level reads are marked as such and are not cited for anything their abstract does not state.

**Result**: nothing found supersedes Section 8. Tangled Program Graphs have one arXiv paper since Gegelati ([Bayer, Smith, and Heywood 2024](https://arxiv.org/abs/2404.06529), abstract), which confirms sparse indexing under explicit reward and changes nothing about operators. The scan adds one operator decision, one predeclaration, one indicator component, one characterization arm, and one closure reading, all recorded in Section 10.3 and applied to the roadmap on 2026-09-06.

### 10.1 Full-text reads

| Paper | Finding (verbatim) | Petri reading |
| --- | --- | --- |
| **Runtime analysis of CGP**, [Dang, Kalkreuth, and Opris, 2026](https://arxiv.org/abs/2606.15923) (full text) | "a non-terminal node of V is called active if there is a path from this specific node to one of the output nodes, otherwise it is called inactive." Bounds of O(nD^5) for strict and O(nD^4) for non-strict selection on an n-input conjunction with D gates: "The faster performance of (1+1) CGP over (1+1) CGP* stems from its non strict selection, which accepts equally fit solutions with redundant active gates, a beneficial effect previously observed only empirically." The proof's staging sublevel is a program with "at least one input edge of an active node that can be redirected to an unused input." The standard mutation operator, single active-gene mutation, "repeatedly modifies genes chosen uniformly at random until an active node is modified." Experiments: "increasing the number of function nodes generally improves the search performance of CGP while using strict selection deteriorates its performance"; "for 1n, CGP failed to evolve solutions for n > 9 with strict selection." Negative result: "(1+1) CGP requires exponential time with exponentially high probability, to construct an exclusive disjunction of n inputs." | The material the proof credits is connected and executed and one mutation from contributing: the CF1 detour and paralog (Section 5), not `AddNode`'s disconnected `Halt` node, which reads 1.000 silent and never activates (Section 3.2). This is NEAT's GNARL critique (Section 6) with a proof behind it, and it argues for folding `AddNode` into the detour form (A9 below). Single active-gene mutation is a reachable bias of 1.0 on the target draw; Petri's topology bias has been 0.0 since T11.F04 (Section 3.3), and the discarded draft's Probe D read 0.7 as doubling reachable mesh (Section 9). The XOR result says some functions are out of reach of point mutation regardless of neutrality; the copy and paralog operators are the macro moves that cover that class and stay. |
| **Evolving complexity is hard**, [Wright and Laue, 2022](https://arxiv.org/abs/2209.13013) (full text) | "The robustness of a genotype is defined as the fraction of mutations of the genotype that don't change the mapped-to phenotype." "complex phenotypes tend to have both low redundancy and low robustness." "Miller and Smith (2006) show that the most evolvable CGP representation is extremely large where over 95% of gates are inactive." Two population effects named from the literature: the "survival of the flattest" phenomenon "where genotypes with high robustness (i. e., low mutational load) will out-compete genotypes with higher fitness but lower robustness," and the "arrival of the frequent" phenomenon "where infrequent (rare) phenotypes are not discovered in time to compete with frequent (common) phenotypes." "as complexity evolves, a greater number of new phenotypes are nearby, and thus there is a positive feedback enabling the discovery of further complexity." | Robustness is the silent fraction the gate and goal reports already record. A population that begins routing conditionally should therefore read a lower evolved-half silent fraction, and the T11.F15 predeclaration has to say so before the reading exists or the no-regression rule will grade success as failure. Survival of the flattest names the counter-force to Section 5.4: at 0.55 requested events per birth, the robust two-node creature may outcompete a router whose robustness is lower, and the 22-generation reading cannot tell which force wins. Arrival of the frequent is the mesh-layer statement of Section 3.4's junk accumulation. |
| **Genesis**, [Sharma, 2026](https://arxiv.org/abs/2607.21630) (full text; a three-page GECCO Companion abstract) | 512 agents on a 128 by 128 Gray-Scott grid, actions Move, Secrete, Idle; a metabolic cost that "is a hard constraint, not a fitness signal." Fitness removed over 2,000 generations: 7 of 12 runs sustained activity; "Ablation studies show that both CARP and the AIS are necessary: removing either raises failure rates from 41.7% to above 90%"; "EPC plateaued at 140–155 across all successful runs, confirming a structural ceiling rather than an implementation artifact"; three failure modes, "metabolic overload, dominance monopolisation, and neutral drift saturation." Sham control: "In the sham condition, the secretion code executes and the metabolic cost is deducted, but the write to field S is suppressed"; niche construction then shows zero complexity growth in both arms over a million agent-generations. The ceiling breaks only in Version 4, a CPPN encoding where "fitness sharing shields novel CPPN topologies from elimination by fitter incumbents," on "Medium-scale tests (5,000 generations, 4 seeds)." Persistence metric: "GAC: fraction of genome edits persisting beyond a 500-generation horizon." | The closest published regime to Petri's, and its protection for novelty is a novelty archive (the AIS) plus speciation with fitness sharing, both outside the program's constraints (Section 1). Petri's substitute is neutrality at birth and at activation (A1, A2), so the Genesis ceiling is the expected default and T11.F14's generation depth is what shows whether Petri reaches it. Reusable: the sham design, which CF1 against CF2 already is (same connected structure, bid absent against bid present, Section 5.2); the failure names, of which dominance monopolisation is what T02.F03's regional offsets exist to counter; and the persistence metric, which needs the lineage attribution T08.F02 provides and so belongs to T11.F12 when it is scheduled. |

### 10.2 Abstract-level reads

| Paper | Finding (verbatim from the abstract) | Petri reading |
| --- | --- | --- |
| [Milano and Nolfi, 2018](https://arxiv.org/abs/1810.09485) | CGP scales up "through the preferential selection of larger solutions among equally good solutions," validated on parity, a dynamically varying classification, and regression. | Supports keeping the scaffold; the tie-break needs an equal-fitness comparison Petri does not have, so the mutation-side analog is the reachable-bias arm, not a selection rule. |
| [Cui, Margraf, and Hähner, 2024](https://arxiv.org/abs/2410.00518) | CGP positional bias; reorder operators "shuffle the current genotype without changing its corresponding phenotype"; "there is no consistently best performing reorder operator." | Mesh node ids are not positional (retiring `RewriteNodeId` as a no-op is that fact), so no mesh operator follows; whether the graph backend's index order carries a positional bias is a T11.F12 question. |
| [Galván, 2021](https://arxiv.org/abs/2102.08475) | Position paper: "neutrality, given certain conditions, can help to speed up the training/design of deep neural networks." | Motivation only. |
| [Langdon, 2022](https://arxiv.org/abs/2204.13997) | 9.7 million random-value injections into evolved integer GP trees: "only errors near the root node have impact and disruption falls exponentially with depth at between exp(-depth/3) and exp(-depth/5)." | Newer support for depth-weighted targeting (A8), and a reason it is premature: the median executed chain is two nodes (Section 3.5). Stays a T11.F13 arm. |
| [Friedlander, Mayo, Tlusty, and Alon, 2013](https://arxiv.org/abs/1302.4267) | "Product-rule mutations generate sparseness and modularity because they tend to reduce interactions, and to keep small interaction terms small." | A trap under today's operators: a multiplicative `MutateGateBias` would keep a branch at gate bias -1.0 from ever crossing the incumbent. Only meaningful after branches are born tied (A2); recorded, not adopted. |
| [Kumar, Liu, Miikkulainen, and Stone, 2022](https://arxiv.org/abs/2204.04817) | Self-adaptive mutation rates "Sometimes they decay the MR to zero, thus halting evolution"; group elite selection avoids it. | Needs a fitness score per group; the T08.F05 inherited-rate discussion, not a mesh item. |
| [Lalejini, Moreno, and Ofria, 2020](https://arxiv.org/abs/2012.09229) | Tag-based promote and repress regulation: "the system could not evolve solutions to some context-dependent problems until regulation was added"; but "We identify scenarios where the correct response to a particular input never changes, rendering tag-based regulation an unneeded functionality that can sometimes impede adaptive evolution." | The strongest reason to keep C3 (persistent routing position) and C4 (tags) behind T02: until the world rewards context dependence, regulation machinery slows adaptation. Triggers in Section 8 unchanged. |
| [Moreno, Lalejini, and Ofria, 2021](https://arxiv.org/abs/2108.04507) | "tag-matching criteria can influence the rate of adaptive evolution and the quality of evolved solutions." | When C4 becomes live, the matching criterion is a design decision shared with T11.F11, not a detail. |
| [Najarro, Sudhakaran, Glanois, and Risi, 2022](https://arxiv.org/abs/2204.11674) | HyperNCA: a neural cellular automaton grows the weights of a network that can "grow neural networks capable of solving common reinforcement learning tasks." | Grows parameters of a fixed architecture; no discrete topology edit, so not a mesh fix. A new node kind under the contract if anyone wants it. |
| [Zhang and Yoder, 2024](https://arxiv.org/abs/2407.10359) | CGP-evolved developmental programs with "Activity Dependence (AD) into the model such that environmental feedback can help to regulate the behavior of each type of unit." | Activity-dependent pruning contradicts the deliberate scaffold (item 10 of Section 4) and belongs to T03.F08 if anywhere. |
| [Ikeda, Kaneko, and Hatakeyama, 2026](https://arxiv.org/abs/2608.24704) | Exhaustive genotype-phenotype map: "stronger phenotypic noise in a fixed environment increases the selective advantage of reliable expression, thereby favoring high penetrance and mutational robustness. Frequent environmental change instead favors mutational accessibility at the expense of penetrance." | A mechanistic reading of Section 3.5's zero: a fixed world selects the robust single path. Kashtan and Alon's modularly varying goals (Section 6) said what to build; this says what to read when T02.F01 lands. |
| [Estrella Dzib and Holehouse, 2026](https://arxiv.org/abs/2604.26082) | "cell-to-cell variability exerts strong selective pressure, driving the evolution of aligned, robust, and motif-enriched GRN architectures." | Sensory noise is T02.F02's off-by-default treatment; nothing for the mesh. |
| [Sharma, 2026, Genesis platform](https://arxiv.org/abs/2607.21631) | Companion to the full-text read: "speciation-protected niche construction initiates structural diversification that unprotected secretion cannot." | Same reading as Section 10.1. |
| [Bayer, Smith, and Heywood, 2024](https://arxiv.org/abs/2404.06529) | TPG navigates a ViZDoom labyrinth with "a modular indexing scheme that only employs 0.8% of the state space," under RL reward. | Confirms the reference design under explicit reward; no operator change. |

### 10.3 Decisions applied to the roadmap on 2026-09-06

| # | Decision | Ground | Where |
| --- | --- | --- | --- |
| A9 | `AddNode` folds into the detour form: every new node is born on the executed chain as a pass-through to the incumbent successor, seeded tied-but-losing like A2. No disconnected node is created by any operator. | Dang et al. 2026: the credited staging material is active, connected, and executed; NEAT's GNARL critique. CF1 left `AddNode` untouched (Appendix B). | T11.F15 |
| Predeclaration | The evolved-half silent fraction is expected to fall as route variation rises; that fall is a predeclared move under the no-regression rule, not a regression. | Wright and Laue 2022: robustness is the silent fraction, and complex phenotypes have low robustness. | T11.F15 |
| C11 | Knockout count per sampled genome: bypass each executed node on the battery (successor takes its place) and count the nodes whose removal changes no scenario output. This is the executed-but-non-contributing material of the proof's staging sublevel, and it should rise before route variation does. | Dang et al. 2026. | T11.F14 |
| Arm | Single active-gene mutation, a reachable bias of 1.0 on the target draw, as the reference arm for the reachable-bias knob, beside Probe D's 0.7 and T11.F04's 0.0. | Dang et al. 2026; Section 9 (Probe D). | T11.F13 |
| Reading | At T02.F01 closure, read the T11.F14 evolved-half route-variation fraction against the prediction that a static world holds it near its T11.F15 closure value and seasons raise it; if the season period is a config field, read two periods. | Ikeda et al. 2026; Kashtan and Alon 2005 (Section 6). | T02.F01 |
| Later | Persistence as the fraction of genome edits surviving a 500-generation horizon, when lineage attribution exists. | Sharma 2026 (GAC). | T11.F12 |

Rejected on the scan's evidence: speciation, a novelty archive, or protected niches (Genesis; excluded by Section 1); persistent routing position or tags before T02 (Lalejini et al. 2020, beyond the triggers Section 8 already sets); a multiplicative gate-bias step (Friedlander et al. 2013); depth-weighted supply now (Langdon 2022; chain length two); CGP reorder operators at the mesh layer (ids are not positional); developmental encodings or activity-dependent pruning as mesh fixes (Najarro et al. 2022; Zhang and Yoder 2024).

### 10.4 The cheap proof

Probe F2 on the same 200 lineages with A9 on top of CF3 in the scratch worktree; the readings to watch are route variation at generation 1,000 (5.5% under CF3, Section 5.1) and the executed mean (3.19), with the founder rows unchanged. If A9 does not move either reading, it still stands on Section 3.2's 1.000 silent row, since a node that can never execute costs supply for nothing.

## Sources

- [Kelly and Heywood, Emergent Tangled Graph Representations for Atari Game Playing Agents, EuroGP 2017](https://web.cs.dal.ca/~mheywood/OpenAccess/open-kelly17a.pdf)
- [Kelly and Heywood, Emergent Tangled Program Graphs in Multi-Task Learning, IJCAI 2018](https://www.ijcai.org/proceedings/2018/0740.pdf)
- [Kelly, Smith, Heywood, and Banzhaf, Emergent Tangled Program Graphs in Partially Observable Recursive Forecasting and ViZDoom Navigation Tasks, ACM TELO 2021](https://dl.acm.org/doi/10.1145/3468857)
- [Desnos et al., Gegelati: Lightweight Artificial Intelligence through Generic and Evolvable Tangled Program Graphs, DASIP 2021](https://arxiv.org/pdf/2012.08296)
- [Stanley and Miikkulainen, Evolving Neural Networks through Augmenting Topologies, Evolutionary Computation 2002](https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf)
- [Hintze et al., Markov Brains: A Technical Introduction, 2017](https://arxiv.org/pdf/1709.05601)
- [Lalejini and Ofria, Evolving Event-driven Programs with SignalGP, GECCO 2018](https://arxiv.org/pdf/1804.05445)
- [Moreno, Rodriguez Papa, Lalejini, and Ofria, SignalGP-Lite, 2021](https://arxiv.org/pdf/2108.00382)
- [Mabu, Hirasawa, and Hu, A Graph-Based Evolutionary Algorithm: Genetic Network Programming and Its Extension Using Reinforcement Learning, Evolutionary Computation 2007](https://direct.mit.edu/evco/article-abstract/15/3/369/1274/A-Graph-Based-Evolutionary-Algorithm-Genetic)
- [Teller and Veloso, Program Evolution for Data Mining, International Journal of Expert Systems 1995](https://www.cs.cmu.edu/~mmv/papers/Teller-ESJ.pdf)
- [Sims, Evolving Virtual Creatures, SIGGRAPH 1994](https://www.karlsims.com/papers/siggraph94.pdf)
- [Clune, Mouret, and Lipson, The evolutionary origins of modularity, Proc. R. Soc. B 2013](https://arxiv.org/abs/1207.2743)
- [Mengistu, Huizinga, Mouret, and Clune, The Evolutionary Origins of Hierarchy, PLOS Computational Biology 2016](https://journals.plos.org/ploscompbiol/article?id=10.1371/journal.pcbi.1004829)
- [Kashtan and Alon, Spontaneous evolution of modularity and network motifs, PNAS 2005](https://pmc.ncbi.nlm.nih.gov/articles/PMC1236541/)
- [Calabretta, Nolfi, Parisi, and Wagner, Duplication of Modules Facilitates the Evolution of Functional Specialization, Artificial Life 2000](https://direct.mit.edu/artl/article-abstract/6/1/69/2340/Duplication-of-Modules-Facilitates-the-Evolution)
- [Gene duplication and subsequent diversification strongly affect phenotypic evolvability and robustness, 2021](https://pmc.ncbi.nlm.nih.gov/articles/PMC8220273/)
- [Koza, publications 1994, architecture-altering operations](http://www.genetic-programming.com/jkpubs94.html)
- [Sutton, Precup, and Singh, Between MDPs and semi-MDPs: A framework for temporal abstraction in reinforcement learning, Artificial Intelligence 1999](https://people.cs.umass.edu/~barto/courses/cs687/Sutton-Precup-Singh-AIJ99.pdf)
- [neat-python, `genes.py`](https://neat-python.readthedocs.io/en/latest/_modules/genes.html)
- Local: [brain evolvability audit](brain-evolvability-audit-2026-09-04.md); goal reports under `docs/progress/features/`; the discarded 2026-09-06 Opus draft's Probes C and D, quoted above where used
- Added by the arXiv scan (Section 10), full text: [Dang, Kalkreuth, and Opris, Runtime Analysis of Cartesian Genetic Programming in Evolving Boolean Functions, 2026](https://arxiv.org/abs/2606.15923); [Wright and Laue, Evolving Complexity is Hard, 2022](https://arxiv.org/abs/2209.13013); [Sharma, Evolving Self-Organising Agents Without Fitness, GECCO Companion 2026](https://arxiv.org/abs/2607.21630)
- Added by the arXiv scan (Section 10), abstracts: [Milano and Nolfi 2018](https://arxiv.org/abs/1810.09485); [Cui, Margraf, and Hähner 2024](https://arxiv.org/abs/2410.00518); [Galván 2021](https://arxiv.org/abs/2102.08475); [Langdon 2022](https://arxiv.org/abs/2204.13997); [Friedlander, Mayo, Tlusty, and Alon 2013](https://arxiv.org/abs/1302.4267); [Kumar, Liu, Miikkulainen, and Stone 2022](https://arxiv.org/abs/2204.04817); [Lalejini, Moreno, and Ofria 2020](https://arxiv.org/abs/2012.09229); [Moreno, Lalejini, and Ofria 2021](https://arxiv.org/abs/2108.04507); [Najarro, Sudhakaran, Glanois, and Risi 2022](https://arxiv.org/abs/2204.11674); [Zhang and Yoder 2024](https://arxiv.org/abs/2407.10359); [Ikeda, Kaneko, and Hatakeyama 2026](https://arxiv.org/abs/2608.24704); [Estrella Dzib and Holehouse 2026](https://arxiv.org/abs/2604.26082); [Sharma, Genesis platform, 2026](https://arxiv.org/abs/2607.21631); [Bayer, Smith, and Heywood 2024](https://arxiv.org/abs/2404.06529)

## Appendix A: the probes

`crates/v3-core/tests/zz_probe_mesh_function.rs`, run on `main` at c387e423 for the baseline and in the worktree at 43d966c1 with Appendix B applied for the counterfactuals, removed afterward, following the audit's pattern. Drop it back in and run:

```sh
cargo test --release -p v3-core --test zz_probe_mesh_function probe_f -- --nocapture                  # F2 and F3, seconds
cargo test --release -p v3-core --test zz_probe_mesh_function probe_f1 -- --ignored --nocapture       # F1, about 4 minutes
```

All are deterministic (`SmallRng`; lineage seeds 90000 and up; scenario seed 7). Probe F1 uses the goal profile's predeclared parameters (1600 by 1600, 10,000 founders, seed 11, 2,000 ticks) and production defaults otherwise.

```rust
//! TEMPORARY PROBE (not for commit): functional mesh census.
//!
//! The 2026-09-06 stashed note measured mesh *structure* (total and reachable
//! node counts). This probe measures mesh *function*: on a fixed battery of
//! random sensor scenarios, which mesh nodes actually execute, how long the
//! executed chain is, and whether any node's route choice ever varies with
//! the input. A mesh that never varies its route is a chain with junk on it,
//! whatever its reachable count says.

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::collections::{BTreeMap, BTreeSet};
use v3_core::config::{FounderProfile, MutationConfig, RuntimeConfig, SimulationConfig};
use v3_core::contracts::NodeId;
use v3_core::creature::founder::founder_genome;
use v3_core::creature::genome::analysis::mesh_reachable_nodes;
use v3_core::creature::genome::cgp::OutputSinkKind;
use v3_core::creature::genome::{BackendDef, CreatureGenome, VmInstruction};
use v3_core::creature::state::GraphRuntimeState;
use v3_core::mutation::MutationEngine;
use v3_core::runtime::trace::domain::TerminationReason;
use v3_core::runtime::traced_mesh::execute_creature_mesh_traced;
use v3_core::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
use v3_core::sensors::static_inputs::StaticInputs;
use v3_core::sensors::typed_food::TypedFoodLocalSnapshot;
use v3_core::simulation::{run_tick, seed_simulation};

const AGE_CHOICES: [f32; 6] = [0.0, 5.0, 19.0, 20.0, 50.0, 200.0];
const ENERGY_CHOICES: [f32; 6] = [5.0, 15.0, 25.0, 31.0, 45.0, 80.0];
const RESERVE_CHOICES: [f32; 4] = [0.0, 2.0, 4.0, 8.0];

struct Scenario {
    sensors: SensorSnapshot,
    energy: f32,
    reserve: f32,
}

fn nonzero_or_zero(rng: &mut SmallRng, p_zero: f64) -> f32 {
    if rng.gen_bool(p_zero) {
        0.0
    } else {
        rng.gen_range(0.1f32..=1.0)
    }
}

/// Mirrors `neighborhood::battery::draw_scenario` (private there).
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
            typed_local_food: TypedFoodLocalSnapshot {
                food_here_by_type,
                neighbor_food_by_type,
            },
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

#[derive(Default, Debug)]
struct Census {
    executed: BTreeSet<NodeId>,
    chain_lens: Vec<usize>,
    /// node -> distinct selected target positions across scenarios
    choices: BTreeMap<NodeId, BTreeSet<usize>>,
    max_hops_hits: usize,
    /// executed nodes carrying >= 2 route targets
    executed_multi_target: usize,
    /// any executed node structurally writes a route gate
    executed_gate_writer: bool,
    /// any executed VM node reads/writes shared memory or any executed graph node does
    executed_memory_user: bool,
}

fn node_writes_route_gate(genome: &CreatureGenome, id: NodeId) -> bool {
    let Some(node) = genome.nodes.iter().find(|n| n.node_id == id) else {
        return false;
    };
    match &node.backend_def {
        BackendDef::Vm(vm) => vm
            .program
            .iter()
            .any(|i| matches!(i, VmInstruction::WriteRouteGate { .. })),
        BackendDef::Graph(g) => g
            .output_sinks
            .iter()
            .any(|s| matches!(s.kind, OutputSinkKind::RouterGate(_)) && !s.inputs.is_empty()),
    }
}

fn node_uses_memory(genome: &CreatureGenome, id: NodeId) -> bool {
    let Some(node) = genome.nodes.iter().find(|n| n.node_id == id) else {
        return false;
    };
    match &node.backend_def {
        BackendDef::Vm(vm) => vm.program.iter().any(|i| {
            matches!(
                i,
                VmInstruction::LoadSlot { .. }
                    | VmInstruction::LoadSlotImm { .. }
                    | VmInstruction::LoadSlotPrev { .. }
                    | VmInstruction::StoreSlot { .. }
                    | VmInstruction::StoreSlotImm { .. }
                    | VmInstruction::ClearSlot { .. }
            )
        }),
        BackendDef::Graph(g) => g.output_sinks.iter().any(|s| {
            matches!(
                s.kind,
                OutputSinkKind::WriteSlot(_) | OutputSinkKind::ClearSlot(_)
            ) && !s.inputs.is_empty()
        }),
    }
}

fn census(genome: &CreatureGenome, runtime: &RuntimeConfig, scen: &[Scenario]) -> Census {
    let mut c = Census::default();
    for s in scen {
        let mut energy = s.energy;
        let mut shared = [0.0f32; 16];
        let prev = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();
        gr.begin_tick();
        let (_out, hops, term) = execute_creature_mesh_traced(
            genome,
            &s.sensors,
            &mut energy,
            s.reserve,
            &mut shared,
            &prev,
            &mut gr,
            runtime,
        );
        if matches!(term, TerminationReason::MaxHopsReached) {
            c.max_hops_hits += 1;
        }
        c.chain_lens.push(hops.len());
        for h in &hops {
            c.executed.insert(h.node_id);
            if let Some(r) = &h.route {
                c.choices
                    .entry(h.node_id)
                    .or_default()
                    .insert(r.selected_target_idx);
            }
        }
    }
    for id in &c.executed {
        let node = genome.nodes.iter().find(|n| n.node_id == *id);
        if node.is_some_and(|n| n.targets.len() >= 2) {
            c.executed_multi_target += 1;
        }
        if node_writes_route_gate(genome, *id) {
            c.executed_gate_writer = true;
        }
        if node_uses_memory(genome, *id) {
            c.executed_memory_user = true;
        }
    }
    c
}

fn route_varies(c: &Census) -> bool {
    c.choices.values().any(|set| set.len() >= 2)
}

fn pct(v: &mut Vec<usize>, p: f64) -> usize {
    v.sort_unstable();
    if v.is_empty() {
        return 0;
    }
    let i = ((v.len() as f64 - 1.0) * p).round() as usize;
    v[i]
}

fn summarize(label: &str, mut v: Vec<usize>) {
    let n = v.len();
    let mean = v.iter().sum::<usize>() as f64 / n.max(1) as f64;
    println!(
        "  {label:28} n={n:6} min={:3} p25={:3} median={:3} p75={:3} max={:5} mean={mean:7.3}",
        pct(&mut v, 0.0),
        pct(&mut v, 0.25),
        pct(&mut v, 0.5),
        pct(&mut v, 0.75),
        pct(&mut v, 1.0)
    );
}

struct PopSummary {
    total: Vec<usize>,
    reachable: Vec<usize>,
    executed: Vec<usize>,
    chain_median: Vec<usize>,
    chain_max: Vec<usize>,
    varies: usize,
    dead_reachable: usize,
    multi_target: usize,
    gate_writer: usize,
    memory_user: usize,
    max_hops_any: usize,
    n: usize,
}

impl PopSummary {
    fn new() -> Self {
        Self {
            total: vec![],
            reachable: vec![],
            executed: vec![],
            chain_median: vec![],
            chain_max: vec![],
            varies: 0,
            dead_reachable: 0,
            multi_target: 0,
            gate_writer: 0,
            memory_user: 0,
            max_hops_any: 0,
            n: 0,
        }
    }
    fn add(&mut self, g: &CreatureGenome, c: &Census) {
        self.n += 1;
        let reach = mesh_reachable_nodes(g).len();
        self.total.push(g.nodes.len());
        self.reachable.push(reach);
        self.executed.push(c.executed.len());
        let mut lens = c.chain_lens.clone();
        self.chain_median.push(pct(&mut lens, 0.5));
        self.chain_max.push(pct(&mut lens, 1.0));
        if route_varies(c) {
            self.varies += 1;
        }
        if c.executed.len() < reach {
            self.dead_reachable += 1;
        }
        if c.executed_multi_target > 0 {
            self.multi_target += 1;
        }
        if c.executed_gate_writer {
            self.gate_writer += 1;
        }
        if c.executed_memory_user {
            self.memory_user += 1;
        }
        if c.max_hops_hits > 0 {
            self.max_hops_any += 1;
        }
    }
    fn print(self, label: &str) {
        let n = self.n as f64;
        println!("{label}: n={}", self.n);
        summarize("total mesh nodes", self.total);
        summarize("reachable mesh nodes", self.reachable);
        summarize("EXECUTED mesh nodes", self.executed);
        summarize("chain length (median/scn)", self.chain_median);
        summarize("chain length (max/scn)", self.chain_max);
        println!(
            "  executed<reachable (dead-reachable): {:.4}   executed node with >=2 targets: {:.4}   route VARIES with input: {:.4}",
            self.dead_reachable as f64 / n,
            self.multi_target as f64 / n,
            self.varies as f64 / n
        );
        println!(
            "  executed node writes a route gate: {:.4}   executed node uses shared memory: {:.4}   any scenario hit max_mesh_hops: {:.4}\n",
            self.gate_writer as f64 / n,
            self.memory_user as f64 / n,
            self.max_hops_any as f64 / n
        );
    }
}

/// F2: mutation-only walk, no selection. Does drift ever produce conditional routing?
#[test]
fn probe_f2_walk_function() {
    const LINEAGES: u64 = 200;
    let sim_config = SimulationConfig::default();
    let food_types = sim_config.world.food.types.len();
    let scen = scenarios(food_types, 48);
    let mutation = MutationConfig::default();
    let founder = founder_genome(FounderProfile::V3Alpha1);
    let fc = census(&founder, &sim_config.runtime, &scen);
    println!("\n=== F2: functional census on mutation-only walks (no selection), {LINEAGES} lineages ===");
    println!(
        "founder: executed={} varies={} multi_target={} gate_writer={}\n",
        fc.executed.len(),
        route_varies(&fc),
        fc.executed_multi_target,
        fc.executed_gate_writer
    );
    for &checkpoint in &[50u64, 250, 1000] {
        let mut pop = PopSummary::new();
        for lineage in 0..LINEAGES {
            let mut g = founder.clone();
            let mut rng = SmallRng::seed_from_u64(90_000 + lineage);
            for _ in 0..checkpoint {
                let reach = mesh_reachable_nodes(&g);
                MutationEngine::apply_mutations_with_food_type_count(
                    &mut g, &mutation, &reach, &mut rng, food_types,
                );
            }
            let c = census(&g, &sim_config.runtime, &scen);
            pop.add(&g, &c);
        }
        pop.print(&format!("generation {checkpoint}"));
    }
}

/// F1: one real goal-profile seed with selection; whole final population.
#[test]
#[ignore = "slow: ~5 min release; run explicitly"]
fn probe_f1_goal_seed_function() {
    const SEED: u64 = 11;
    const TICKS: u64 = 2_000;
    let mut config = SimulationConfig::default();
    config.world.width = 1600;
    config.world.height = 1600;
    config.population.initial_creatures = 10_000;
    let runtime = config.runtime.clone();
    let food_types = config.world.food.types.len();
    let scen = scenarios(food_types, 48);

    println!("\n=== F1: goal profile seed {SEED}, {TICKS} ticks, with selection ===");
    let mut sim = seed_simulation(config, SEED);
    let started = std::time::Instant::now();
    for t in 0..TICKS {
        run_tick(&mut sim, &mut None);
        if sim.creatures.is_empty() {
            panic!("population collapsed at tick {t}");
        }
        if (t + 1) % 500 == 0 {
            eprintln!("tick {} pop {} ({:.0}s)", t + 1, sim.creatures.len(), started.elapsed().as_secs_f64());
        }
    }
    let mut pop = PopSummary::new();
    let mut generations = Vec::new();
    let mut ages = Vec::new();
    let mut by_reach: BTreeMap<usize, (usize, usize)> = BTreeMap::new(); // reach -> (count, varies)
    for c in sim.creatures.values() {
        let cen = census(&c.genome, &runtime, &scen);
        let reach = mesh_reachable_nodes(&c.genome).len();
        let e = by_reach.entry(reach).or_default();
        e.0 += 1;
        if route_varies(&cen) {
            e.1 += 1;
        }
        pop.add(&c.genome, &cen);
        generations.push(c.generation as usize);
        ages.push(c.age as usize);
    }
    println!("final population {} after {:.0}s", sim.creatures.len(), started.elapsed().as_secs_f64());
    summarize("generation", generations);
    summarize("age (ticks)", ages);
    pop.print("final population");
    println!("route-varies by reachable count (reach: n, varies):");
    for (r, (n, v)) in by_reach {
        println!("  {r}: {n} creatures, {v} vary ({:.3})", v as f64 / n as f64);
    }
}

/// F3: founder per-operator neighborhood rows and per-birth result through
/// the production engine (same battery and sizes as the gate profile).
#[test]
fn probe_f3_founder_rows() {
    use v3_core::neighborhood::{evaluate_genome, Battery, EvalContext};
    let config = SimulationConfig::default();
    let battery = Battery::generate(config.world.food.types.len());
    let ctx = EvalContext::from_config(&config);
    let founder = founder_genome(FounderProfile::V3Alpha1);
    let eval = evaluate_genome(&founder, &battery, &config.mutation, &ctx, 50, 500, 0);
    println!("\n=== F3: founder topology operator rows (50 trials each) ===");
    for row in &eval.operator_rows {
        if row.family == "topology" {
            let t = &row.tally;
            println!(
                "  {:24} applied={:3} skipped={:3} silent={:3} changed={:3} dead={:3}",
                row.operator,
                t.trials - t.skipped,
                t.skipped,
                t.silent,
                t.changed,
                t.dead
            );
        }
    }
    let b = &eval.births;
    let a = &b.any_events;
    println!(
        "  births total={} zero_event={} mutated={} silent={} changed={} dead={}",
        b.births_total,
        b.zero_event_births,
        a.trials - a.skipped,
        a.silent,
        a.changed,
        a.dead
    );
    for (k, t) in &b.by_events {
        println!(
            "    events={k}: n={} silent={} changed={} dead={}",
            t.trials - t.skipped,
            t.silent,
            t.changed,
            t.dead
        );
    }
}
```

## Appendix B: the counterfactual patch

Applied cumulatively to a detached worktree at 43d966c1 and never committed. `PROBE` comments mark every change. Not a proposed implementation: the feature spec owns the real design, including the bid's source distribution and the spec sentences C1 replaces.

```diff
diff --git a/crates/v3-core/src/mutation/topology/mod.rs b/crates/v3-core/src/mutation/topology/mod.rs
index 5014113e..615ed244 100644
--- a/crates/v3-core/src/mutation/topology/mod.rs
+++ b/crates/v3-core/src/mutation/topology/mod.rs
@@ -63,7 +63,7 @@ impl TopologyOperator {
             Self::RemoveRouteTarget => 2,
             Self::ChangeEntryNode => 2,
             Self::SwapNodeBackend => 1,
-            Self::RewriteNodeId => 4,
+            Self::RewriteNodeId => 0, // PROBE: retired no-op
             Self::CopyNode => 1,
             Self::CopyMeshBackwardSlice => 1,
             Self::CopyMeshForwardSlice => 1,
@@ -196,7 +196,7 @@ impl TopologyMutator {
                 routing::apply_retarget_node_target(genome, reachable_nodes, bias, rng)
             }
             TopologyOperator::AddRouteTarget => {
-                routing::apply_add_route_target(genome, reachable_nodes, bias, rng)
+                routing::apply_add_route_target(genome, reachable_nodes, bias, rng, config)
             }
             TopologyOperator::RemoveRouteTarget => {
                 routing::apply_remove_route_target(genome, reachable_nodes, bias, rng)
diff --git a/crates/v3-core/src/mutation/topology/routing.rs b/crates/v3-core/src/mutation/topology/routing.rs
index fbeb9114..86960441 100644
--- a/crates/v3-core/src/mutation/topology/routing.rs
+++ b/crates/v3-core/src/mutation/topology/routing.rs
@@ -1,6 +1,11 @@
 use rand::Rng;

-use crate::contracts::{RouteTarget, MAX_GATE_SLOTS};
+use crate::contracts::{NodeId, RouteTarget, MAX_GATE_SLOTS};
+use crate::config::MutationConfig;
+use crate::creature::genome::cgp::{GraphEdge, OutputSinkKind};
+use crate::creature::genome::{BackendDef, NodeGenome, VmInstruction};
+use crate::mutation::graph::operators::random_graph_source;
+use crate::mutation::vm::operators::insert_new_instruction_with_reference_repair;
 use crate::creature::genome::CreatureGenome;
 use crate::mutation::reachability::biased_select_from;
 use crate::mutation::types::{MutationSkipReason, TargetReachability};
@@ -17,13 +22,29 @@ pub(super) fn lowest_unused_slot(targets: &[RouteTarget]) -> Option<u8> {
     (0..MAX_GATE_SLOTS as u8).find(|&s| used & (1 << s) == 0)
 }

+/// PROBE: position of the static winner among `targets` (highest `gate_bias`,
+/// earliest position on ties), mirroring `resolve_gated_route` with zero
+/// runtime gate scores.
+pub(super) fn static_winner_position(targets: &[RouteTarget]) -> Option<usize> {
+    let mut best: Option<(usize, f32)> = None;
+    for (i, t) in targets.iter().enumerate() {
+        match best {
+            Some((_, b)) if !(t.gate_bias > b) => {}
+            _ => best = Some((i, t.gate_bias)),
+        }
+    }
+    best.map(|(i, _)| i)
+}
+
 pub(super) fn apply_retarget_node_target(
     genome: &mut CreatureGenome,
     reachable_nodes: &[usize],
     bias: f64,
     rng: &mut impl Rng,
 ) -> Result<TargetReachability, MutationSkipReason> {
-    // Find nodes with non-empty targets.
+    // PROBE: local destination rule. Candidates are the current successor's
+    // own targets and this node's other targets; never the node itself. Only
+    // when no local candidate exists fall back to any other node.
     let eligible: Vec<usize> = genome
         .nodes
         .iter()
@@ -34,7 +55,36 @@ pub(super) fn apply_retarget_node_target(
     let (node_idx, reachability) = biased_select_from(&eligible, reachable_nodes, bias, rng)
         .ok_or(MutationSkipReason::NoApplicableTarget)?;
     let target_slot = rng.gen_range(0..genome.nodes[node_idx].targets.len());
-    let new_target = genome.nodes[rng.gen_range(0..genome.nodes.len())].node_id;
+    let self_id = genome.nodes[node_idx].node_id;
+    let current = genome.nodes[node_idx].targets[target_slot].target_id;
+    let mut candidates: Vec<NodeId> = Vec::new();
+    if let Some(succ) = genome.nodes.iter().find(|n| n.node_id == current) {
+        candidates.extend(
+            succ.targets
+                .iter()
+                .map(|t| t.target_id)
+                .filter(|&id| id != self_id && id != current),
+        );
+    }
+    candidates.extend(
+        genome.nodes[node_idx]
+            .targets
+            .iter()
+            .map(|t| t.target_id)
+            .filter(|&id| id != self_id && id != current),
+    );
+    if candidates.is_empty() {
+        candidates = genome
+            .nodes
+            .iter()
+            .map(|n| n.node_id)
+            .filter(|&id| id != self_id && id != current)
+            .collect();
+    }
+    if candidates.is_empty() {
+        return Err(MutationSkipReason::NoApplicableTarget);
+    }
+    let new_target = candidates[rng.gen_range(0..candidates.len())];
     genome.nodes[node_idx].targets[target_slot].target_id = new_target;
     Ok(reachability)
 }
@@ -44,25 +94,89 @@ pub(super) fn apply_add_route_target(
     reachable_nodes: &[usize],
     bias: f64,
     rng: &mut impl Rng,
+    config: &MutationConfig,
 ) -> Result<TargetReachability, MutationSkipReason> {
-    // Pre-filter for nodes with free routing slots (< MAX_GATE_SLOTS targets).
+    // PROBE: activation-neutral branch. The new branch is seeded at the
+    // incumbent's bias in a later position (tied, so it loses), and its
+    // destination already works: either a pass-through detour that forwards
+    // to the incumbent successor, or a paralog copy of that successor.
     let eligible: Vec<usize> = genome
         .nodes
         .iter()
         .enumerate()
-        .filter(|(_, n)| n.targets.len() < MAX_GATE_SLOTS)
+        .filter(|(_, n)| n.targets.len() < MAX_GATE_SLOTS && !n.targets.is_empty())
         .map(|(i, _)| i)
         .collect();
     let (node_idx, reachability) = biased_select_from(&eligible, reachable_nodes, bias, rng)
         .ok_or(MutationSkipReason::NoApplicableTarget)?;
     let slot =
         lowest_unused_slot(&genome.nodes[node_idx].targets).expect("pre-filtered for free slots");
-    let target_id = genome.nodes[rng.gen_range(0..genome.nodes.len())].node_id;
+    let winner = static_winner_position(&genome.nodes[node_idx].targets).expect("non-empty");
+    let incumbent = genome.nodes[node_idx].targets[winner];
+    let new_id = super::structural::next_node_id(genome);
+    let incumbent_node = genome
+        .nodes
+        .iter()
+        .find(|n| n.node_id == incumbent.target_id)
+        .cloned();
+    let new_node = match incumbent_node {
+        Some(b) if rng.gen_bool(0.5) => NodeGenome {
+            node_id: new_id,
+            input_refs: b.input_refs.clone(),
+            backend_def: b.backend_def.clone(),
+            targets: b.targets.clone(),
+        },
+        _ => NodeGenome {
+            node_id: new_id,
+            input_refs: Vec::new(),
+            backend_def: super::birth::minimal_vm_backend(),
+            targets: vec![RouteTarget {
+                target_id: incumbent.target_id,
+                slot: 0,
+                gate_bias: 0.0,
+            }],
+        },
+    };
+    genome.nodes.push(new_node);
     genome.nodes[node_idx].targets.push(RouteTarget {
-        target_id,
+        target_id: new_id,
         slot,
-        gate_bias: -1.0,
+        gate_bias: incumbent.gate_bias,
     });
+    // PROBE (counterfactual 2): the branch is born with its own bid. A graph
+    // node gets one edge from a random source into the branch's gate slot; a
+    // VM node gets a `WriteRouteGate` from a random register inserted (with
+    // reference repair) just before its first terminal instruction.
+    let node = &mut genome.nodes[node_idx];
+    match &mut node.backend_def {
+        BackendDef::Graph(g) => {
+            let source =
+                random_graph_source(g.compute_nodes.len() as u16, &node.input_refs, config, rng);
+            if let Some(sink) = g
+                .output_sinks
+                .iter_mut()
+                .find(|s| s.kind == OutputSinkKind::RouterGate(slot))
+            {
+                sink.inputs.push(GraphEdge {
+                    source,
+                    weight: 1.0,
+                });
+            }
+        }
+        BackendDef::Vm(vm) => {
+            let src = rng.gen_range(0..vm.register_count.max(1));
+            let at = vm
+                .program
+                .iter()
+                .position(|i| matches!(i, VmInstruction::ExecuteActionQueue | VmInstruction::Halt))
+                .unwrap_or(vm.program.len());
+            let _ = insert_new_instruction_with_reference_repair(
+                &mut vm.program,
+                at,
+                VmInstruction::WriteRouteGate { slot, src },
+            );
+        }
+    }
     Ok(reachability)
 }

@@ -72,17 +186,22 @@ pub(super) fn apply_remove_route_target(
     bias: f64,
     rng: &mut impl Rng,
 ) -> Result<TargetReachability, MutationSkipReason> {
+    // PROBE: remove a losing branch only; a sole target or the static winner
+    // is never removed.
     let eligible: Vec<usize> = genome
         .nodes
         .iter()
         .enumerate()
-        .filter(|(_, n)| !n.targets.is_empty())
+        .filter(|(_, n)| n.targets.len() >= 2)
         .map(|(i, _)| i)
         .collect();
     let (node_idx, reachability) = biased_select_from(&eligible, reachable_nodes, bias, rng)
         .ok_or(MutationSkipReason::NoApplicableTarget)?;
-    let target_slot = rng.gen_range(0..genome.nodes[node_idx].targets.len());
-    genome.nodes[node_idx].targets.remove(target_slot);
+    let len = genome.nodes[node_idx].targets.len();
+    let winner = static_winner_position(&genome.nodes[node_idx].targets).expect("non-empty");
+    let losers: Vec<usize> = (0..len).filter(|&i| i != winner).collect();
+    let pos = losers[rng.gen_range(0..losers.len())];
+    genome.nodes[node_idx].targets.remove(pos);
     Ok(reachability)
 }

diff --git a/crates/v3-core/src/mutation/topology/structural.rs b/crates/v3-core/src/mutation/topology/structural.rs
index bfa00260..50885d16 100644
--- a/crates/v3-core/src/mutation/topology/structural.rs
+++ b/crates/v3-core/src/mutation/topology/structural.rs
@@ -3,7 +3,7 @@ use std::collections::HashMap;
 use rand::Rng;

 use crate::config::MutationConfig;
-use crate::contracts::{NodeId, RouteTarget};
+use crate::contracts::{NodeId, RouteTarget, MAX_GATE_SLOTS};
 use crate::creature::genome::analysis::{mesh_backward_slice, mesh_forward_slice};
 use crate::creature::genome::{BackendDef, CreatureGenome, NodeGenome};
 use crate::mutation::reachability::biased_select_from;
@@ -41,10 +41,12 @@ pub(super) fn apply_remove_node(
     bias: f64,
     rng: &mut impl Rng,
 ) -> Result<TargetReachability, MutationSkipReason> {
+    // PROBE: prefer an unreachable node; otherwise bypass the removed node
+    // (every reference to it is rerouted to its static successor), the
+    // inverse of splice.
     if genome.nodes.len() <= 1 {
         return Err(MutationSkipReason::NoApplicableTarget);
     }
-    // Find non-entry nodes to avoid removing the entry.
     let entry_id = genome.entry_node_id;
     let removable: Vec<usize> = genome
         .nodes
@@ -53,9 +55,37 @@ pub(super) fn apply_remove_node(
         .filter(|(_, n)| n.node_id != entry_id)
         .map(|(i, _)| i)
         .collect();
-    let (idx, reachability) = biased_select_from(&removable, reachable_nodes, bias, rng)
-        .ok_or(MutationSkipReason::NoApplicableTarget)?;
+    let unreachable: Vec<usize> = removable
+        .iter()
+        .copied()
+        .filter(|i| reachable_nodes.binary_search(i).is_err())
+        .collect();
+    let (idx, reachability) = if unreachable.is_empty() {
+        biased_select_from(&removable, reachable_nodes, bias, rng)
+            .ok_or(MutationSkipReason::NoApplicableTarget)?
+    } else {
+        (
+            unreachable[rng.gen_range(0..unreachable.len())],
+            TargetReachability::Unreachable,
+        )
+    };
+    let removed_id = genome.nodes[idx].node_id;
+    let bypass = super::routing::static_winner_position(&genome.nodes[idx].targets)
+        .map(|p| genome.nodes[idx].targets[p].target_id)
+        .filter(|&id| id != removed_id);
     genome.nodes.remove(idx);
+    for node in &mut genome.nodes {
+        match bypass {
+            Some(b) => {
+                for t in &mut node.targets {
+                    if t.target_id == removed_id {
+                        t.target_id = b;
+                    }
+                }
+            }
+            None => node.targets.retain(|t| t.target_id != removed_id),
+        }
+    }
     Ok(reachability)
 }

@@ -132,42 +162,55 @@ pub(super) fn apply_copy_node(
     bias: f64,
     rng: &mut impl Rng,
 ) -> Result<TargetReachability, MutationSkipReason> {
+    // PROBE: paralog. The copy keeps the original's inputs, backend, and
+    // targets, and hangs off one of the original's predecessors as a
+    // tied-losing alternative successor. With no predecessor it stays
+    // disconnected.
     let all_indices: Vec<usize> = (0..genome.nodes.len()).collect();
     let (source_idx, reachability) = biased_select_from(&all_indices, reachable_nodes, bias, rng)
         .ok_or(MutationSkipReason::NoApplicableTarget)?;
+    let source_id = genome.nodes[source_idx].node_id;
     let backend_def = genome.nodes[source_idx].backend_def.clone();
+    let input_refs = genome.nodes[source_idx].input_refs.clone();
+    let targets = genome.nodes[source_idx].targets.clone();
     let new_id = next_node_id(genome);
-
-    let targets = if rng.gen_bool(0.5) {
-        genome.nodes[source_idx].targets.clone()
-    } else {
-        vec![]
-    };
-    let input_refs = if rng.gen_bool(0.5) {
-        genome.nodes[source_idx].input_refs.clone()
-    } else {
-        vec![]
-    };
     genome.nodes.push(NodeGenome {
         node_id: new_id,
         input_refs,
         backend_def,
         targets,
     });
-
-    // Add backlink to ensure the copied node is reachable, using the lowest
-    // unused slot. Skip if all slots are already occupied.
-    if let Some(slot) = lowest_unused_slot(&genome.nodes[source_idx].targets) {
-        genome.nodes[source_idx].targets.push(RouteTarget {
-            target_id: new_id,
-            slot,
-            gate_bias: 0.0,
-        });
+    let preds: Vec<usize> = genome
+        .nodes
+        .iter()
+        .enumerate()
+        .filter(|(i, n)| {
+            *i != source_idx
+                && n.node_id != new_id
+                && n.targets.len() < MAX_GATE_SLOTS
+                && n.targets.iter().any(|t| t.target_id == source_id)
+        })
+        .map(|(i, _)| i)
+        .collect();
+    if !preds.is_empty() {
+        let p = preds[rng.gen_range(0..preds.len())];
+        let bias_of_original = genome.nodes[p]
+            .targets
+            .iter()
+            .filter(|t| t.target_id == source_id)
+            .map(|t| t.gate_bias)
+            .fold(f32::NEG_INFINITY, f32::max);
+        if let Some(slot) = lowest_unused_slot(&genome.nodes[p].targets) {
+            genome.nodes[p].targets.push(RouteTarget {
+                target_id: new_id,
+                slot,
+                gate_bias: bias_of_original,
+            });
+        }
     }
     Ok(reachability)
 }

-/// Maximum number of nodes in a mesh slice for copy operators.
 const MESH_SLICE_MAX_SIZE: usize = 8;

 /// Clone a set of nodes identified by `gene_indices`, remapping internal
diff --git a/crates/v3-core/src/mutation/vm/mod.rs b/crates/v3-core/src/mutation/vm/mod.rs
index 09abe4a8..809f9485 100644
--- a/crates/v3-core/src/mutation/vm/mod.rs
+++ b/crates/v3-core/src/mutation/vm/mod.rs
@@ -5,7 +5,7 @@ use crate::creature::genome::{BackendDef, CreatureGenome};
 use crate::mutation::reachability::biased_select_from;
 use crate::mutation::types::{MutationSkipReason, TargetReachability};

-mod operators;
+pub(crate) mod operators; // PROBE: exposed for the gate-wired branch
 use operators::*;
 #[cfg(test)]
 pub(crate) use operators::{
diff --git a/crates/v3-core/src/runtime/mesh.rs b/crates/v3-core/src/runtime/mesh.rs
index bdbd866b..839d98c6 100644
--- a/crates/v3-core/src/runtime/mesh.rs
+++ b/crates/v3-core/src/runtime/mesh.rs
@@ -215,6 +215,10 @@ pub(crate) fn execute_creature_mesh_impl<M: MeshExecutionMode>(
     let mut current_node_id = genome.entry_node_id;
     let mut upstream_slots = [0.0f32; OUTPUT_SLOT_COUNT];
     let mut hops: usize = 0;
+    // PROBE (counterfactual 3): single-visit rule. A node runs at most once
+    // per tick; a route to an already-visited node falls through to the next
+    // best unvisited target, and the chain ends when none remains.
+    let mut visited = vec![false; genome.nodes.len()];
     let max_hops = config.max_mesh_hops.max(1) as usize;
     let start_energy = *energy;
     let mut report = ComputeCostReport::default();
@@ -246,6 +250,7 @@ pub(crate) fn execute_creature_mesh_impl<M: MeshExecutionMode>(
         let current_idx =
             find_node_index(&genome.nodes, current_node_id).expect("node must exist in genome");
         let node = &genome.nodes[current_idx];
+        visited[current_idx] = true;

         let energy_consumed = (start_energy - *energy).max(0.0);

@@ -277,7 +282,9 @@ pub(crate) fn execute_creature_mesh_impl<M: MeshExecutionMode>(
         }

         let route_result = if M::RECORDS_HOPS || (!result.energy_exhausted && !result.terminal) {
-            resolve_gated_route(&node.targets, &result.route_gates)
+            resolve_gated_route_unvisited(&node.targets, &result.route_gates, |id| {
+                find_node_index(&genome.nodes, id).is_none_or(|i| !visited[i])
+            })
         } else {
             None
         };
@@ -343,6 +350,28 @@ pub(crate) fn execute_creature_mesh_impl<M: MeshExecutionMode>(
     }
 }

+/// PROBE (counterfactual 3): argmax over `gate_bias + score[slot]` restricted
+/// to targets that pass `allowed`; ties by position; `None` if no target is
+/// allowed.
+fn resolve_gated_route_unvisited(
+    targets: &[crate::contracts::RouteTarget],
+    gates: &crate::runtime::routing::RouteGateMap,
+    allowed: impl Fn(NodeId) -> bool,
+) -> Option<(usize, NodeId)> {
+    let mut best: Option<(usize, NodeId, f32)> = None;
+    for (i, target) in targets.iter().enumerate() {
+        if !allowed(target.target_id) {
+            continue;
+        }
+        let effective = target.gate_bias + gates.score_for_slot(target.slot);
+        match best {
+            Some((_, _, b)) if !(effective > b) => {}
+            _ => best = Some((i, target.target_id, effective)),
+        }
+    }
+    best.map(|(i, id, _)| (i, id))
+}
+
 /// Find the index of a node by its `NodeId` via linear scan.
 ///
 /// For typical genomes (2-10 nodes), linear scan is faster than HashMap
```
