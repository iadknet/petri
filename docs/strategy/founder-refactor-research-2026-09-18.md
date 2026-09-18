# Founder Refactor: A Specialized-Node Founder (research note, 2026-09-18)

**Status**: research; track T18 created from it on 2026-09-18 (`docs/roadmaps/t18-founder-architecture.md`); probe measured in `.worktrees/founder-serial-probe` (not for merge). Branches off the
[reproduction-collapse research note](reproduction-collapse-research-2026-09-18.md)
(Section 4 of its handoff: founder hazards). Code read at `main` 7f5d8994.

## Question

The user's proposal: split the founder from its two-node mesh (graph sensor
aggregator → monolithic VM decision program) into specialized nodes from
birth: router nodes, nodes that read world inputs and write them to shared
memory, one node per action kind (move, eat, reproduce), and one terminal
node that executes the action queue. Is this the right refactor, what does
the engine already offer for it, what does prior art say, and what should
the track contain?

## Verdict

**Adopt the shape, as a new founder profile, all-graph, serial by default.**
The engine already has every primitive the proposal needs; no new node kind,
sink, operator, or sensor is required (Section 2). Prior art in evolutionary
robotics measured exactly this decomposition, one output module per effector
with its own selector gate, and found it beats a monolithic controller on
rate and level of adaptation, with the caveat that the *arbitration* among
modules must be left to evolution rather than designed (Section 3). Petri's
own evidence is that the monolithic VM program is the block: 26 relative
jumps make one insertion break behavior 71% of the time, the reproduce
branch is terminal so a founder forages *or* breeds, 3.5% of founder births
are sterile, and after 400 generations the population still runs on founder
node 1 plus one evolved node with routing load-bearing in 3.3% of creatures
(Section 1).

Three forks are the user's to decide before a spec is written (Section 5):
serial chain versus router-selected action (the chain executor allows one
successor per hop, so a router that picks the action node re-creates the
reproduce-or-forage hazard); new profile versus replacing V3Alpha1 (a new
profile keeps every indicator baseline as the control); and ordering
against T17.F02 (the new founder should be born on unit-scale introspection
or its spec absorbs the re-expression).

## 1. What the current founder is and why it blocks

Read from `creature/founder.rs`, `creature/cgp_founder.rs`, and the
strategy notes cited.

| Fact | Where | Consequence |
| --- | --- | --- |
| Node 0 (graph): 3 compute nodes (energy `Threshold(30)`, age threshold, `Multiply`), 6 `CustomOutput` sinks carrying food-here, can-reproduce, and N/E/S/W primary food onto the slot bus. Action bank, execute gate, router gates, memory sinks all unwired. | `cgp_founder.rs` | The graph node is a pure sensor aggregator; the evolvable action bank and direction bank (T11.F21) start empty on every founder. |
| Node 1 (VM): ~70 instructions, 26 jumps, all 16 registers live, argmax over the ring by compare-and-jump, `PushAction` + `ExecuteActionQueue` on each branch. | `founder.rs` | Reproduce branch ends in the terminal `ExecuteActionQueue`: a tick is reproduce *or* eat+move, never both. A refused attempt is an idle tick, so a gate below physiology's acceptance pins the founder at its threshold (handoff §4). |
| Single mutable reproduce path: energy gate edge → CN2 → CustomOutput(1) → VM `CmpGt` → jump. | both | 3.5% of founder births are sterile; the [Orchards collapse](orchards-collapse-2026-09-17.md) is famine selection for these lesions. |
| Jump offsets not remapped; one no-op insertion breaks behavior 71% of the time on the founder. | [brain evolvability audit](brain-evolvability-audit-2026-09-04.md) §"Where the brittleness comes from" | Every lesion is pleiotropic: an eat-branch edit can break the reproduce branch and vice versa. |
| Evolved populations keep founder node 1 as the core: contributing set `{1, 4}` in 252 of 400 creatures; conditional routing load-bearing in 13 of 400; one seeking rule in the whole population. | [live survey](live-survey-2026-09-16.md) §2.2 | Evolution has not replaced the VM program in 400 generations; the decision core is still the hand-written jump table. |
| Founder `genome_size()` is 111 units, most of it the VM program; per-unit supply is 0.005 events per unit per birth. | `founder.rs`, `genome/mod.rs` | The VM program is also where most of the founder's mutation supply lands. |

What the refactor can and cannot fix. It removes the reproduce-or-forage
tick, the jump-table pleiotropy, and the empty action bank at birth, and
it puts routing, memory, and per-action gates on the executed chain from
tick 0. It does **not** fix the famine-suicide rule (fixed transfer at 15%
condition) or the raw-unit inputs: those are T17.F01/F02 and stay there. A
per-action reproduce node with its own gate is still one edge removal from
sterile; the difference is that the lesion no longer takes eating with it.

## 2. What the engine already provides (local inspection)

Every element of the proposal is an existing primitive of the graph backend
or the mesh executor. Read at `main` 7f5d8994.

| Proposal element | Existing primitive | Where |
| --- | --- | --- |
| Router node | `OutputSinkKind::RouterGate(slot)` writes `route_gates.scores[slot]`; `RouteTarget { slot, gate_bias }` per successor; argmax with first-wins ties; single visit per tick (T11.F15). | `genome/cgp.rs`, `runtime/routing.rs`, `runtime/mesh.rs` |
| Sensor → memory node | `InputReference::World/…` on the node's `input_refs`; `OutputSinkKind::WriteSlot(s)` / `ClearSlot(s)` write the 16-slot blackboard; downstream nodes read `GraphSource::SharedMemory { slot, previous }` (current tick, or last tick's snapshot). | `cgp.rs`, `runtime/cgp/sources.rs`, `effects.rs` |
| One node per action kind | Action bank `ActionSlot { behavior: Emit(Eat|Move|Reproduce|StealEnergy), gate_inputs, param_inputs, direction_bids }`; fires when the gate's weighted sum > 0; a movement kind commits the argmax of the eight-slot direction bank (T11.F21). | `cgp.rs`, `effects.rs` Phase 2 |
| Terminal executor node | `ExecuteGate { inputs }`: `terminal = wired && wsum > 0 && queue non-empty`. A graph node with only a `Constant → execute_gate` edge enters the visit (`enters_visit()` checks the execute gate). If the queue is empty it falls through to routing; `NoTargets` returns the queue-or-NoOp. | `cgp.rs:346`, `effects.rs` Phase 3, `mesh.rs` |
| Actions accumulate across hops | `MeshSideOutputs.action_queue` is shared by every node in the chain; Phase 2 of the tick applies them sequentially (eat, then move, then reproduce is legal). `max_actions_per_turn` = 10. | `mesh.rs`, `simulation/tick.rs:1026` |
| Cost | `graph_node_base_cost` 1e-5 per compute node per visit; a seven-node founder pays under 1e-4 per tick against a 0.02 decay. | `config/simulation.rs` |
| Mutation reaches all of it | Graph operators spray edges onto sinks, gates, and the execute gate; T11.F08 `CopyNode`, T11.F15 branch-with-bid, T11.F17 executed-biased targeting, T11.F18 backend-neutral growth. Graph family reads 0.326 changed / 0.000 dead against VM 0.306 / 0.004. | T11 track |

Two constraints the chain executor imposes on the layout:

1. **One successor per hop.** A node's route is an argmax over its targets;
   there is no fan-out. A router whose targets are the three action nodes
   therefore selects one action per tick. That is the current founder's
   reproduce-or-forage hazard in a new coat. The layout that removes the
   hazard is serial: each action node is visited every tick and its own
   action-bank gate decides whether it *queues*; a router, if present,
   selects between behavior programs (sub-chains), not between actions.
   Sub-chains may converge: a "fed" branch can route Reproduce → Eat while
   the "hungry" branch routes straight to Eat, since single-visit only
   forbids revisiting.
2. **Single visit per tick.** Fine for a DAG-shaped founder; it means a
   loop-shaped founder is impossible, which is not what is proposed.

A serial founder's `genome_size()` is roughly 70 to 80 units (estimated
from the unit rules in `genome/mod.rs`: one sensor node with five refs and
six sinks, a router, three action nodes with gate, param, and four direction
bids, one executor) against V3Alpha1's 111, so per-unit supply per birth
falls by about a third. `FOUNDER_GENOME_SIZE_UNITS` anchors the replication
cost to the *canonical* founder; a new profile needs its own anchor or the
anchor becomes per-profile.

## 3. Prior art

Read in full unless marked otherwise. Research date 2026-09-18.

| Source | What it did | What it says for this refactor |
| --- | --- | --- |
| [Nolfi 1997, *Using Emergent Modularity to Develop Control Systems for Mobile Robots*](https://doi.org/10.1177/105971239700500306), Adaptive Behavior 5:343–363 (PDF read in full, http://laral.istc.cnr.it/nolfi/papers/nolfi.emodular.pdf) | Five architectures for a trash-collecting Khepera, weights evolved, no learning: (A) feed-forward, (B) one hidden layer, (C) Elman recurrent, (D) **hand-crafted modular**: two modules whose expected behavior the designer fixed in advance (gripper empty → find and pick up; gripper full → find wall and release), (E) **emergent modular**: two competing output modules per motor, each with a selector neuron; the more active selector's module controls that motor that step, and which module does what is left to evolution. E was significantly better than all others at generation 199 (p < 0.05), found correct solutions earlier than D, and on the real robot 7 of 10 E controllers cleaned the arena against 1 or 2 of 10 for every other architecture, "even more meaningful" because E's genotype was the longest. All but A reached the same plateau by generation 999. Module use did not map onto the designer's sub-behaviors ("distal description"); it correlated with low-level sensory-motor mappings. | Two distinct lessons. (1) One gated module per effector, arbitration by competition, is the architecture that won. (2) The hand-crafted state router (D) is exactly a "fed / hungry" behavior-program router seeded by the designer: it reached the plateau but later than E and transferred worse. So seed the per-action gates and do not seed a behavior partition above them; if a router is added, add it as a measured variant, not in the base profile. |
| [Calabretta, Nolfi, Parisi, Wagner 2000, *Duplication of Modules Facilitates the Evolution of Functional Specialization*](https://direct.mit.edu/artl/article-abstract/6/1/69/2340), Artificial Life 6:69–84 (PDF read in full) | Three architectures: non-modular feed-forward; hardwired modular (two modules with selectors per effector from generation 0); duplication-based modular (**one module per effector at start**, a duplication operator adds a competitor whose selector begins "completely nonfunctional"). Both modular beat non-modular on rate and level of adaptation; duplication-based reached functional specialization, hardwired stayed distributed. | One module per action at birth plus a neutral duplication operator is the measured recipe, and Petri already has both halves: the action bank per node and T11.F08 `CopyNode`. Starting with two hand-wired competitors per action is *not* better on performance and yields less specialization. |
| [Stanley and Miikkulainen 2002, *Evolving Neural Networks through Augmenting Topologies*](https://gwern.net/doc/reinforcement-learning/exploration/2002-stanley.pdf), Evolutionary Computation 10(2) (ablation section, verified 2026-09-18) | Starting NEAT from *random* initial topologies (1 to 10 hidden neurons, random connectivity) was 7× slower and failed 5% of runs, against starting minimal. | The opposing school, correctly scoped: the penalty is for *random, non-functional* initial structure that inflates the search space. A founder whose every node is live and load-bearing at birth is not that. It is the reason not to pad the founder with speculative nodes (a second router, an unused memory node, a steal node with no prey). |
| Polyworld, Yaeger 1994 (primary read for the [reproduction-collapse note](reproduction-collapse-research-2026-09-18.md); behavior list confirmed against Yaeger and Sporns's later summaries) | Seven fixed output neurons, one per behavior (move, turn, eat, mate, attack, light, focus); input groups → internal groups → output groups, group counts and densities in the genome. | The per-action output is the standard shape in ecology ALife; Petri's action bank already is that, unused by the founder. |
| The Bibites ([wiki: input and output neurons](https://the-bibites.fandom.com/wiki/Input_and_Output_Neurons), [FAQ](https://the-bibites.fandom.com/wiki/FAQ)) | Fixed output layer with one neuron per act (Accelerate, Rotate, Want2Lay, Want2Eat, Want2Attack, …); seed "virgin" bibites are hand-wired templates per scenario because the random-brain default was "problematic". | A hand-authored seed with per-action outputs is normal practice; the seed is a template, and different worlds get different templates (Petri's `FounderProfile`). |
| Genetic Network Programming (Mabu, Hirasawa, Hu 2007; in the [brain audit's mesh table](brain-evolvability-audit-2026-09-04.md)) | Judgment nodes read sensors and pick the next node; processing nodes emit actions; fixed node pool, evolvable links. | The router/action split by another name, on a nodes-route-to-nodes graph like the mesh. Supports specializing node *roles* while keeping one node type contract. |
| Avida default ancestor (read for the audit) | A hand-written copy loop padded with `nop-C` so most of the genome is neutral slack. | Neutral scaffold at birth matters (T11.F04 owns it); the refactor should leave unwired sinks and bank slots as they are, not fill them. |

Not consulted, deliberately: subsumption-architecture literature (Brooks) and
behavior-network action selection (Maes, Tyrrell). They describe designed
arbitration hierarchies, which is Nolfi's architecture D.

## 4. Options

| Option | What changes | Fit with the evidence | Cost and risk |
| --- | --- | --- | --- |
| A. Repair in place | Keep two nodes. Founder VM queues eat + move + reproduce before one `ExecuteActionQueue`; reproduce gate set at or above physiology's acceptance. | Fixes the reproduce-or-forage tick and the pinning hazard only. Leaves the jump table, the empty action bank, and the two-node core. Does not test whether routing and memory become load-bearing when seeded. | Smallest diff; T17.F01 already plans the gate half. Nothing learned about the mesh. |
| **B. Specialized-node founder, serial, all-graph (recommended)** | New `FounderProfile`: sensor→memory node → [router] → eat → move → reproduce → executor, every node a graph backend, one action per node in the bank with its own gate, direction bids from the neighbor ring, reproduce param a constant. Node 0's argmax-by-jumps becomes four direction-bid edges (the T11.F21 tie rule, lowest tied index with an unwired scalar decoding N, reproduces the VM's first-argmax exactly: 0 mismatches in 2,000 random rings, Section 8). | Matches Calabretta 2000's winning start (one module per effector, gate per module) and Nolfi's warning (no designed arbitration above the gates). Uses only existing primitives; graph family is the most evolvable backend. Founder writes the action bank and direction bank that T11.F21 built. | Medium: a builder in `founder.rs`, profile row in the seeding spec, per-profile size anchor, founder tests re-derived (argmax parity, gate boundaries), viability and indicator baselines for the new profile. Risk: the population might still collapse to two contributing nodes; that is the measurement. |
| C. Minimal three-node | Sensor → one graph action node (all three actions in one bank) → executor. | The smallest all-graph founder; fixes the tick and the jump table but keeps all actions in one node, so `CopyNode` duplicates all of them together and a lesion can still take eat with reproduce. | Cheaper than B; less specialization to measure. A fallback if B's per-node overhead shows in the indicators. |
| D. Router-selected action (the literal reading) | Router node targets the three action nodes; each ends in the executor. | Re-creates one-action-per-tick under the single-successor rule; would need fan-out in the mesh executor, which the brain audit deferred as a mesh feature and nothing measured asks for. | Not recommended as a founder feature. If the user wants fan-out, that is a T11 executor feature with its own note. |

**Recommendation: B**, with C as the recorded fallback. B beats A because A
learns nothing about the mesh and the live survey's core finding (the VM
node is the population's decision core after 400 generations) is exactly
what A preserves. B beats D because D is the hazard the refactor exists to
remove. The remaining tradeoff: B is a new founder, so every founder-anchored
indicator (battery, births, drift, steering) gets a second baseline row and
the viability test gets a second profile; that is the price of keeping
V3Alpha1 as the control.

## 5. Decisions for the user before a spec

1. **Serial chain, or router between behavior programs?** Serial (no
   router) is the minimal B and matches Calabretta's start. A router that
   picks "hungry" (Eat → Move) versus "fed" (Reproduce → Eat → Move) seeds
   one real conditional route, which the [mesh note](mesh-evolvability-research-2026-09-06.md)
   found is what route variation needs to become load-bearing. Both are
   expressible today. Recommendation: serial in F01, the router as a
   measured follow-up (F02) so the router's effect is attributable.
2. **New profile or replace V3Alpha1?** New profile (`SpecializedSerial`
   or similar); the default switch is a later decision with its epoch
   re-pin. Genome consumers (30 files) do not change; the exhaustive
   `match FounderProfile` sites do: `config/simulation.rs` (enum, wire name,
   validation), `creature/state.rs`, `neighborhood/{battery,births,drift,
   mesh_execution,operators}.rs`, `simulation/seeding.rs`,
   `v3-cli/bench/indicators.rs`, and the profile table in
   `v3-runtime-config-spec.md` §5. The frontend has no profile picker.
3. **Order against T17.F02.** If T17.F02 (unit-scale introspection) lands
   first, the new founder is born with a `Threshold(0.15)` energy gate and
   never needs re-expression. If the founder lands first, T17.F02
   re-expresses two founders. Recommendation: T17.F02 first, or the founder
   spec absorbs F02's re-expression for its own profile only.
4. **Chain order is action order.** Phase 2 applies the queue in order, so
   the node order fixes whether the child is placed relative to the pre-move
   or post-move cell. Eat → Reproduce(toward food) → Move(toward food) makes
   the parent's move collide with its own child every breeding tick (a
   failed move, plus the penalty while T16.F02 is open). The probe used
   Eat → Move → Reproduce *behind* (bids reversed, scalar S so a full tie
   still places the child on the vacated cell): the vacated cell is free by
   construction after a successful move. Name the order in the spec.
5. **Memory or slot bus for the sensor handoff?** The proposal says memory.
   Writing sensors to shared memory gives every downstream node a
   `SharedMemory { previous: true }` handle to *last tick's* reading from
   birth, which is the temporal seed T11.F10 asks about, at no cost. The
   slot bus is what node 0 uses today. Recommendation: memory for the
   values the founder re-reads (food here, energy), direct `input_refs` on
   the action nodes for the ring, so a sensor-node lesion does not blind
   every action.

## 6. Track shape as placed (2026-09-18)

Track **T18 — Founder Architecture** (`../roadmaps/t18-founder-architecture.md`).
The user's decisions after this note: a new profile first with V3Alpha1 as
the control; the specialized profile becomes the default when done; the
founder should carry a router that makes a real decision. The owned-route
shape below is this note's design for that (the route is the reproduce
decision, the reproduce gate a constant), unmeasured at track creation.

- **T18.F01 — Specialized-Node Founder Profile**: `sense → eat → move → router → { reproduce → execute | execute }`, all graph backends, the router after the motor nodes so a broken route still leaves a queued action; the spec reads it beside the serial-gated variant probed here (Section 8) and records which ships.
- **T18.F02 — Specialized Founder as the Default**: after T17.F02, both epochs re-pinned, size anchor and every founder row re-baselined, V3Alpha1 selectable.

Not in the track: transfer fraction and unit-scale inputs (T17), collapse
protection (separate discussion), fan-out in the mesh executor (T11, if
ever), any new sensor or operator. The topology weight rescale that made
`ChangeEntryNode` 1 draw in 211 (Section 8) was made ahead of the track, in
the working tree with the gate decision pending.

For T17: the serial profile discharges the hazard T17 carries ("the founder
must queue reproduce and forage in one tick, or keep its gate at or above
acceptance") for itself, because a refused reproduce no longer costs the
eat and move queued before it. T17.F01's founder constraint then applies to
V3Alpha1 only, or falls away once T18.F02 switches the default.

## 7. Remaining uncertainty

- Whether a population seeded with per-action nodes keeps them as separate
  contributing nodes, and whether the router's route stays load-bearing, is
  unmeasured beyond tick 2,000 on one world; F01's battery reading and the
  survey counters answer it. Calabretta's result is on four effectors and
  seven sensors with a GA, not an open ecology, and the live survey says
  nothing in the current worlds pays for a second decision.
- The routed variant's Orchards reading is one seed (Section 8); seed 12
  and the goal worlds are F01's. Route edits read
  0% dead on evolved genomes since T11.F15 (T16.F01 goal summary: retarget
  24 changed / 216 silent / 0 dead of 240; remove-target 0 / 160 / 0); on
  the routed founder they are not dead but they are *sterilizing*
  (`SwapRouteTargets` inverts the decision), which is the measured price of
  a decision that lives in a route.

## 8. Measurements (scratch probe, 2026-09-18)

`crates/v3-core/tests/zz_probe_founder_serial.rs` on branch
`scratch/founder-serial-probe` at 7f5d8994, production defaults, seeds as
written in the file. Serial-gated layout: sense→memory, eat (gate food
here), move (constant gate, four ring bids), reproduce (gate can_reproduce
from memory, bids reversed, scalar S, transfer 20), execute (constant gate).

Routed layout (T18.F01's shape): sense→memory, eat, move, router
(`RouterGate(0)` ← can_reproduce; targets reproduce at bias 0, execute at
bias 0.5), reproduce with a constant gate, execute.

| Reading | V3Alpha1 | Serial-gated | Routed |
| --- | ---: | ---: | ---: |
| `genome_size()` (nodes) | 111 (2) | 55 (5) | 61 (6) |
| Move direction against the founder's first-argmax, 2,000 random cardinal rings | — | 0 mismatches | same bids |
| Nine founder-test scenarios | reproduce *or* forage | `[Eat, Move W, Reproduce E]` (child behind) | identical to serial on all nine |
| Births probe, 4,000 births: mutated | 1,701 | 711 | 800 |
| Changed per birth / per mutated birth | 0.200 / 0.469 | 0.059 / 0.332 | 0.083 / 0.415 |
| Silent per mutated birth | 0.530 | 0.660 | 0.584 |
| Dead per mutated birth | 0.0006 | 0.0084 (`ChangeEntryNode` to the executor, retarget before a motor node) | 0.0013 |
| Sterile per birth / per mutated birth (fed, energy 150, age 100) | 4.6% / 10.5% | 2.7% / 14.0% (76 of 108 `InputRefSwap` on the sense node) | 4.3% / 21.1% (77 `InputRefSwap`, 40 `SwapRouteTargets`, 24 `RemoveRouteTarget`) |

Reading the two layouts: the router puts the reproduce decision on the
executed route, so routing operators change behavior (changed per mutated
birth 0.415 against 0.332, dead down to 0.13%) and the same operators can
invert or drop the decision (`SwapRouteTargets` breeds when hungry and
never when fed; sterile per mutated birth 21% against 14%). Both are the
same fact: a decision in a route is reachable by route edits. The
probe's `ChangeEntryNode` counts predate the weight rescale.

Orchards goal case (1600², 10,000 founders, 2,000 ticks; founders rebuilt
with the serial genome at tick 0, V3Alpha1 control from the same seed):

| Seed 11 | Control | Serial-gated | Routed |
| --- | ---: | ---: | ---: |
| Peak | 96,879 | 100,000 (cap) | 99,473 |
| First trough | 59 at tick 400, fertile 10% | 26 at tick 400, fertile 70% | 77 at tick 300, fertile 4% |
| Rebuild | none; 0 births after tick 250 | 956 at tick 600, fertile 95% | 2,123 at tick 700, fertile 82% |
| Second famine | — | ticks 700–1,100, to 5 sterile | none: 1,100–1,900 through tick 1,600, then growth |
| Tick 2,000 | 7, fertile 14% | 5, fertile 0% | **5,110, fertile 60%, 9,053 births in the last 100 ticks** |

The routed founder is the only one of the three alive at tick 2,000 on
seed 11. One seed, one world, and its trough fertile share (4%) was the
lowest of the three, so the rebuild came from a handful of fertile
lineages; whether it holds on seed 12 and on the goal worlds is F01's
reading, not this note's claim.

Seed 12 serial: same shape (rebuild after the first trough, second famine
fatal, 5 sterile at tick 2,000). The layout keeps fertile lineages through
the first trough; the breed-at-15%-condition rule then produces the second
famine (the [collapse note](orchards-collapse-2026-09-17.md)'s
recurring-famine shape), which is T17's and the deferred collapse
discussion's, not the layout's.

Topology weight rescale (working tree, 2026-09-18, ahead of the track): every
topology operator weight ×10 with `ChangeEntryNode` held at 1, so the
whole-brain macro is 1 draw in 211 instead of 1 in 22; four seeded
trajectory pins re-pinned; the gate benchmark flags `plasticity_updates`
+80% against T16.F01 as draw remapping, awaiting the user's epoch decision.

## Sources

- Nolfi, S. 1997. Using emergent modularity to develop control systems for mobile robots. *Adaptive Behavior* 5(3–4):343–363. https://doi.org/10.1177/105971239700500306
- Calabretta, R., Nolfi, S., Parisi, D., Wagner, G. P. 2000. Duplication of modules facilitates the evolution of functional specialization. *Artificial Life* 6(1):69–84. https://direct.mit.edu/artl/article-abstract/6/1/69/2340 (PDF: http://gral.ip.rm.cnr.it/rcalabretta/calabretta.duplication.pdf)
- Stanley, K. O., Miikkulainen, R. 2002. Evolving neural networks through augmenting topologies. *Evolutionary Computation* 10(2):99–127.
- Yaeger, L. 1994. Computational genetics, physiology, metabolism, neural systems, learning, vision, and behavior or PolyWorld: Life in a new context. *Artificial Life III*.
- The Bibites wiki: Input and Output Neurons; FAQ. https://the-bibites.fandom.com/wiki/Input_and_Output_Neurons
- Mabu, S., Hirasawa, K., Hu, J. 2007. A graph-based evolutionary algorithm: Genetic Network Programming. *Evolutionary Computation* 15(3):369–398.
- Petri: `crates/v3-core/src/creature/{founder,cgp_founder}.rs`, `creature/genome/{cgp,mod}.rs`, `runtime/{mesh,routing}.rs`, `runtime/cgp/{execute,effects,sources}.rs`, `simulation/tick.rs`, `config/simulation.rs`; notes cited inline.
