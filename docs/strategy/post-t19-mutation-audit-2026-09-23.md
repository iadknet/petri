# Post-T19 mutation operation audit

Date: 2026-09-23. Audited checkout: `aef05f90` (T19.F06 closure).
Companion: [sensor integration audit](post-t19-sensor-integration-audit-2026-09-23.md).
This is an audit and decision record, not an executable feature spec.

## Conclusion

All **53 current genome mutation operations** are represented in the domain
catalogs, engine dispatch, and telemetry keys: 13 topology, 15 VM, 21 graph,
and 4 input-reference operations. Vote sinks and the new decision inputs are
connected to the general mutation machinery. T19 intentionally removed the
old graph action-behavior operation; an absent replacement with the same name
is not a dropped dispatch.

There are, however, concrete correctness and accessibility issues. Parent-based
target selection keeps positional indices after a child node is removed; fresh
VM reads sample only compound channel zero; five of eight action-parameter
slots are writable but ignored by action decoding; and applied-event telemetry
does not imply a changed genome or changed behavior. The latter distinction
also affects phenotype and kin-tag mutations. Several issues predate T19.

## Scope and selection pipeline

Reviewed every operator's selection, applicability, mutation implementation,
parseability rollback, output integration and recording; also phenotype,
identity, plasticity inheritance, sensor consumers and stored goal evidence.
Compared relevant source with pre-track `437e3358`. No production edits or
new mutation operators were made.

The production path is [reproduction.rs](../../crates/v3-core/src/simulation/actions/reproduction.rs)
→ [MutationEngine](../../crates/v3-core/src/mutation/engine/mod.rs) → domain
mutator → parseability gate → summary, followed by phenotype/identity updates
and plasticity inheritance.

- Supply is `Binomial(parent.genome_size(), per_unit_rate)`, drawn once before
  edits. Default rate is 0.005; the 97-unit founder requests 0.485 events per
  birth in expectation. The disabled legacy supply mode remains available to
  existing probes.
- Per requested event, topology gets default probability 0.2. VM, Graph and
  InputRef each get `(1 - 0.2)/3`, regardless of which backends exist. Thus a
  graph-only founder has no applicable VM targets for about 26.67% of requested
  domain draws, conditional on still being graph-only at that event. There is
  no cross-domain retry.
- Operators draw by weight. A `NoApplicableTarget` result discards that operator
  and retries another in the same domain; exhausted domains record a skip.
  VM/Graph/InputRef applicability generally filters nodes before selection.
- Default executed-node bias is 0.9, with a 100-tick window; remaining selection
  uses per-domain reachability bias. InputRef.RawFieldMutation and entry
  changes do not select a node through that mechanism.
- Optional size pressure uses `(size/cap)^2` and, when triggered, admits only
  operations classified **decreasing**, not all non-increasing operations.
  It is disabled by default. Its comments still say non-increasing.
- Each attempted operator has a genome snapshot. Errors or parseability
  rejection restore it. Parseability is not semantic validity, affordability,
  changed genotype, useful behavior or viability.

Sources: [engine](../../crates/v3-core/src/mutation/engine/mod.rs),
[target selection](../../crates/v3-core/src/mutation/reachability.rs),
[config defaults](../../crates/v3-core/src/config/simulation.rs),
[pressure](../../crates/v3-core/src/mutation/pressure.rs),
[telemetry types](../../crates/v3-core/src/mutation/types/mod.rs).

## Complete operation inventory

`w` is the domain's base weight, not a per-birth probability. `↑`, `↓`, `=`
are the implementation's size-pressure classifications, not a promise of
behavioral neutrality or an exact size delta. Topology CopyNode and the two
mesh-slice copies have their weights multiplied by the default 25% tuning.
The three other domains do not use that tuning. Every row has a live dispatch.

### Topology: 13 operations

Sources: [catalog and dispatch](../../crates/v3-core/src/mutation/topology/mod.rs),
[structural operators](../../crates/v3-core/src/mutation/topology/structural.rs),
[routing operators](../../crates/v3-core/src/mutation/topology/routing.rs),
[blank backends](../../crates/v3-core/src/mutation/topology/birth.rs).

| Operation | w / class | Actual prerequisites and effect | Integration status / limitation |
| --- | --- | --- | --- |
| AddNode | 10 ↑ | Needs an existing valid route; inserts a blank Graph or one-register Halt VM detour | Same implementation as SpliceNode; neither introduces sensors or votes |
| RemoveNode | 10 ↓ | Never removes entry/last node; prefers unreachable nodes, otherwise requires a usable non-self bypass successor | Repairs incoming routes; shifts vector indices, exposing M1 |
| RetargetNodeTarget | 20 = | Needs a different existing destination in current destination's targets or source's targets | Self-target legal if present in that neighborhood; not arbitrary global rewiring |
| AddRouteTarget | 20 ↑ | Source must have exactly one valid target and a free gate slot; adds gate writer plus blank detour rejoining old successor | Can operate on a self-loop after T19; does not independently connect to any arbitrary node or add a third branch |
| RemoveRouteTarget | 20 ↓ | At least two routes; removes a non-static-incumbent route | The dynamically selected route can differ from that static incumbent |
| ChangeEntryNode | 1 = | At least two nodes; chooses a different entry | Can bypass established behavior; deliberately rare and not target-biased |
| SwapNodeBackend | 10 ↑ | Needs a safe predecessor with tied unwritten alternative gate slot | Creates an attached copy with the opposite blank backend; original preserved; later routing activation required |
| CopyNode | 10 ↑ | Same safe-predecessor requirement | Copies refs/backend, remaps self-target, attaches losing tied alternative; does not immediately execute the copy |
| CopyMeshBackwardSlice | 10 ↑ | Nonempty mesh and attachment slot; ancestor slice up to 8 nodes | Remaps internal IDs, preserves external routes, attaches via a free slot; not guaranteed behavior-neutral |
| CopyMeshForwardSlice | 10 ↑ | Nonempty mesh and attachment slot; descendant slice up to 8 nodes | Same attachment caveat; can create recurrent structures |
| SpliceNode | 10 ↑ | Existing valid route | Blank detour as AddNode; extra dispatch/carrying/ramp cost can matter despite transparent data forwarding |
| SwapRouteTargets | 40 = | At least two routes | Swaps destinations while keeping gate slot/bias at each position; same destinations can make the edit a no-op |
| MutateGateBias | 40 = | At least one route | Adds a bounded perturbation and clamps to [-4,4]; saturation or irrelevant route can produce no behavioral change |

### VM: 15 operations

Sources: [catalog/applicability](../../crates/v3-core/src/mutation/vm/mod.rs),
[operator implementations](../../crates/v3-core/src/mutation/vm/operators.rs),
[VM runtime](../../crates/v3-core/src/runtime/vm.rs).

| Operation | w / class | Actual prerequisites and effect | Integration status / limitation |
| --- | --- | --- | --- |
| VmConstantMutation | 4 = | Any VM; seeds empty pool, otherwise relative ±10% scale nudge | Can grow an empty pool despite neutral classification; constants can be unreferenced |
| VmInstructionMutation | 2 = | Any VM; insert, replace, or delete; nonempty result retained | All 39 current opcodes drawable, including AddVote; not merely size-preserving; see M2/M3 |
| VmDeleteInstruction | 1 ↓ | Program length >1 | Removes one instruction and repairs jumps; can expose a formerly dormant tail |
| VmRegisterCountMutation | 2 = | Valid width 1..32 with feasible grow/shrink | Canonicalizes register operands; shrink only if disappearing register unused |
| VmInstructionRawFieldMutation | 4 = | An operand-bearing instruction | One integer field ±1, opcode retained; reaches other input channels, vote sinks and bus slots, but may leave contextual bounds |
| VmCopyInstructionBlock | 1 ↑ | Nonempty program | Copies 2..32 capped instructions to guarded dormant tail; later activation needed |
| VmCopyInstructionBlockRemapped | 1 ↑ | Nonempty program | Cyclic register remap and insertion at a random location; potentially active and disruptive, unlike guarded copies |
| VmCopyConstantBlock | 1 ↑ | Nonempty constants | Appends 1..16 capped constants; existing addresses preserved, new constants need readers |
| VmCopyGeneBackwardSlice | 1 ↑ | An output instruction anchoring a backward slice | Dormant tail copy with jump repair; AddVote is part of the current output analysis |
| VmCopyGeneForwardSlice | 1 ↑ | A register writer anchoring a forward slice | Dormant tail copy; no guarantee copied behavior becomes expressed |
| VmInsertReadStoreMotif | 2 ↑ | At least one declared input | Inserts adjacent ReadInput/StoreSlotImm at random position; channel zero, first 255 refs, may be beyond Halt |
| VmInsertReadBidMotif | 2 ↑ | At least one declared input | ReadInput/SetPriorityBid before a selected Halt, or random position if none; **priority auction bid**, not an action vote; channel zero, first 255 refs |
| VmInsertLoadCompareMotif | 2 ↑ | Any VM | Adjacent LoadSlotImm/CmpGt; supplies a comparison but no vote or branch consumer automatically |
| VmMutateSlotAddress | 4 = | A memory-slot instruction | Nudges immediate slot or register-indirect address operand; may become a soft-default access |
| VmMutatePairedSlotAddress | 4 = | Immediate-address group containing both load and store/clear | Moves all members to a different effective slot modulo 16; excludes register-indirect pairing |

Insertion/deletion/copy use the existing splice repair to preserve old jump
target identity. Preserving targets is not equivalent to preserving the executed
instruction sequence: newly inserted instructions, register writes, costs and
the remapped copy can still affect behavior. No ReadInput/AddVote motif exists;
the ordinary instruction operator can still build that chain.

### Graph: 21 operations

Sources: [catalog/applicability](../../crates/v3-core/src/mutation/graph/mod.rs),
[edge and compute operators](../../crates/v3-core/src/mutation/graph/operators.rs),
[plasticity operators](../../crates/v3-core/src/mutation/graph/hebbian.rs),
[effect application](../../crates/v3-core/src/runtime/cgp/effects.rs).

| Operation | w / class | Actual prerequisites and effect | Integration status / limitation |
| --- | --- | --- | --- |
| AlterGraphEdgeWeight | 4 = | Any compute/sink edge | Multiplicative or near-zero additive change; includes votes/parameters; clears inherited learned value for edited compute edge |
| SwapGraphOperator | 2 = | A compute node | Draws any of 19 compute kinds; can redraw the same kind; inputs retained even if new operator ignores them |
| MutateGraphOperatorParam | 4 = | Constant, Threshold, DecayIntegrator, Momentum or Oscillator | Bounded additive parameter step where applicable; AdaptiveGain has no encoded parameter |
| AddInternalGraphNode | 1 ↑ | Any graph | Disconnected, bootstrap, or eligible identity edge split; no arbitrary new sink connections; split excludes direct EnergyCurrent sink read on plastic graph |
| RemoveInternalGraphNode | 1 ↓ | A compute node | Removes and remaps sources; broken dependencies become soft-default/dangling sources |
| AddGraphEdge | 2 ↑ | A compute or fixed sink container | Uniform destination container, sampled source and signed weight; full input width, including every vote sink |
| RetargetGraphEdge | 2 = | An existing edge | Changes source, not destination container; can connect declared sensor to existing vote; can redraw same source |
| RemoveGraphEdge | 2 ↓ | An existing edge | Removes across compute and sink surfaces, including votes and parameters |
| GraphRawFieldMutation | 4 = | Parameterized compute node or edge with a valid unit move | Parameter nudge or one source field change; compound width-aware; never changes sink kind |
| CopyInternalNode | 1 ↑ | Compute node and index capacity | Faithful in-place-order duplication with remapping; copied output initially unconsumed, inputs/plasticity retained |
| CopySubgraph | 1 ↑ | At least two compute nodes and capacity | Random-walk cluster targeted at 2..4 nodes, may stop smaller; internal source remap; no new sink consumers |
| CopyEdgeBundle | 2 ↑ | Two compute nodes, one with inputs | Appends one compute node's inputs to another; does not copy a vote sink's edge bundle |
| EnableHebbian | 1 ↑ | Nonplastic compute node with inputs | Draws rule, rate, clamp and inheritance flag; direct sink edges cannot be plastic |
| DisableHebbian | 1 ↓ | Plastic compute node | Removes full plasticity config, including modulation |
| MutateHebbianRule | 2 = | Plastic compute node | Chooses a different Classic/Oja/AntiHebb/Covariance rule |
| MutateHebbianRate | 4 = | Plastic compute node | Perturbs and clamps rate to [0,1] |
| ToggleHebbianLamarckian | 2 = | Plastic compute node | Toggles learned-weight inheritance; inheritance follows birth-edge provenance |
| EnableRewardModulation | 1 ↑ | Plastic compute node without modulation | Adds outcome channel and trace decay |
| DisableRewardModulation | 1 ↓ | Modulated compute node | Removes modulation; pure Hebbian learning remains |
| MutateRewardSource | 2 = | Modulated compute node | Chooses different EnergyDelta/ActionSuccess/DamageDelta/OffspringSuccess channel |
| MutateTraceDecay | 4 = | Modulated compute node | Perturbs trace decay and clamps to [0,1] |

Graph source draws prefer ComputeNode (50%), InputLeaf (30%), SharedMemory
(20%) when both earlier categories exist; fallthrough changes those proportions
when compute nodes or refs are absent. All 99 fixed sink containers are offered
to edge addition. Most are not action votes. Neutral structural additions can
still incur compute, memory, carrying and hop costs, especially with live revisits.
The runtime evaluates compute nodes even when a structural census calls them
inactive. Plasticity parameters affect compute-input edges, not the fixed sinks.

### Input references: 4 operations

Source: [input operators](../../crates/v3-core/src/mutation/input_ref/mod.rs).

| Operation | w / class | Actual prerequisites and effect | Integration status / limitation |
| --- | --- | --- | --- |
| Add | 2 ↑ | Any mesh node | Appends one of all current input families, wires nothing; main introduction path for absent barrier/decision families |
| Prune | 2 ↓ | Entry with no backend consumer anywhere, including dormant code | Removes entry and repairs higher consumer indices; consumed but behaviorally irrelevant refs cannot be pruned until consumers are removed |
| Swap | 4 = | Entry with another member of same read class and width | Preserves modality/width; cannot turn a food or occupancy ring into a barrier ring |
| RawFieldMutation | 4 = | Any typed-food or upstream reference across genome | Changes food type or upstream slot; bypasses node targeting; one-food-type draw and upstream redraw can leave genome unchanged |

The [sensor audit](post-t19-sensor-integration-audit-2026-09-23.md) enumerates
every family, channel width and permitted Swap alternative.

## Birth changes outside the 53-operation catalog

| Mechanism | Trigger and semantics | Relationship to node behavior |
| --- | --- | --- |
| Phenotype active-channel switch | Once after a birth with at least one applied genome event; default chance 0.001, chooses another of six channels | Changes which inherited phenotype channel will step |
| Phenotype polarity flip | Same birth trigger; default chance 0.0002 on selected channel | Changes step direction |
| Phenotype channel step | Every such birth; default step 1 with u8 wrapping | Visible to NearbyCreatureIdentity.phenotype_similarity |
| Kin-tag mutation | Same birth trigger; flips exactly one of 32 bits | Visible to kin_affinity; lineage ID is inherited unchanged |
| Lamarckian learned-weight inheritance | Birth, through capture/remap/build of enabled edge provenance | Not another mutation event; new/changed edge handling and lamarckian flags govern copied learned values |
| Shared-memory inheritance | Parent's memory copied into child | Not a mutation operator; graph temporal state/outcome initialization is a separate runtime concern |

Sources: [reproduction](../../crates/v3-core/src/simulation/actions/reproduction.rs),
[phenotype](../../crates/v3-core/src/mutation/phenotype.rs),
[identity](../../crates/v3-core/src/creature/identity.rs), and
[plasticity companion data](../../crates/v3-core/src/creature/genome/companions.rs).
The mutation rate and perception radius are run configuration, not evolvable
per-creature genes. `weight_clamp` is sampled by EnableHebbian but has no
dedicated refinement operator; changing it requires disabling/re-enabling
plasticity or acquiring a copy. That is a tuning gap, not a missing dispatcher.

## Findings

### M1 — Parent membership is misapplied after node deletion

**Correctness defect established by source; predates T19.** The engine creates
`TargetSets` once from parent *indices*. `RemoveNode` removes an element of the
child's `nodes` vector. Subsequent events select current child indices against
the old parent sets. Rebuilding `node_ids` for event recording does not remap
those sets. Newborn cache reconstruction happens after all events.

Concrete counterexample: parent nodes `[A, B, C]`, with B unreachable and
parent reachable/executed indices `[0,2]`. Remove B: child is `[A,C]`. A later
event sees C at index 1 and classifies it as not parent-reachable/executed. If
another event appends D at index 2, D can incorrectly inherit C's membership.
This changes selection and telemetry. It is distinct from intentionally
retaining the parent's reachability instead of recomputing the child's graph.

Evidence: [engine lines 106–137](../../crates/v3-core/src/mutation/engine/mod.rs),
[RemoveNode](../../crates/v3-core/src/mutation/topology/structural.rs),
[TargetSets and select](../../crates/v3-core/src/mutation/reachability.rs).
**Recommendation:** preserve parent membership by stable NodeId and map to
current indices, or repair the parent-membership index sets after removal.
Stable identity is clearer than recomputing child reachability, which changes
the intended policy. Any identity-based fix must also distinguish newly created
nodes if a removed ID is reused: `next_node_id` derives IDs from the current
vector, not a monotonic birth-wide allocator. Before remediation, add a regression for removal followed
by a targeted event and removal/addition followed by a targeted event. This
audit did not execute that new counterexample as a Rust test.

### M2 — VM fresh-input and payload draws are narrower than the runtime

**Confirmed accessibility asymmetry; predates T19.** All three new-read
generators use compound channel zero and at most the first 255 refs. A u16
`ref_idx` can represent later entries, and raw nudges can reach them; fresh
draws cannot. `WriteInternalPayload` draws slots 0..8 although the bus has
24 slots. Graph edges draw full input widths and can write all 24 outputs.
See [VM operators](../../crates/v3-core/src/mutation/vm/operators.rs)
(`random_vm_instruction`, both read motifs),
[runtime widths](../../crates/v3-core/src/runtime/types.rs), and sensor finding S1.

**Recommendation:** share width-aware input drawing across VM creation paths,
use the real payload width, and explicitly decide limits for reference/constant
pools instead of relying on the historical 255 draw cap. This changes mutation
draws and needs new discovery readings; it is not proof these biases caused the
observed lack of avoidance. Keep contextual soft defaults for raw mutations.

### M3 — Five writable action-parameter slots have no action meaning

**Confirmed post-T19 surface mismatch.** The catalog has two parameters for
each of four kinds, and the VM draws all eight. `decode_commit` reads only
Eat[0] (food type), Reproduce[1] (transfer), and StealEnergy[1] (amount).
Eat[1], both Move slots, Reproduce[0], and StealEnergy[0] never affect the
decoded action. Thus 5/8 of *parameter-address* choices in a fresh
WriteActionParam target unused fields; this is not 5/8 of all mutation events.

These writes may still consume energy and count as wired structural outputs.
Sensor-to-unused-parameter paths can therefore inflate a structural census
without controlling an action. ClearSlot is another case where the edge's
value does not determine the applied effect: any wiring clears its slot.

Evidence: [fixed catalog](../../crates/v3-core/src/creature/genome/cgp.rs),
[decoder](../../crates/v3-core/src/runtime/action_decode.rs),
[VM draw](../../crates/v3-core/src/mutation/vm/operators.rs),
[graph effects](../../crates/v3-core/src/runtime/cgp/effects.rs).
**Recommendation:** make the active action-parameter catalog explicit and
derive creation draws from it. If rectangular storage is useful, retaining it
does not require drawing unused fields. Preserve explicit structural census
semantics and measure causal use separately. Re-encoding/removing storage is a
larger alternative and is unnecessary just to close the draw gap.

### M4 — Applied events can leave the genotype unchanged

**Confirmed accounting semantics; predates T19.** Single-food
InputRef.RawFieldMutation writes type 0 back to type 0 and returns success.
Upstream redraws, edge-source redraws, kind redraws and clamped parameters can
also succeed without changing the genome. Reproduction then mutates phenotype
and kin tag because `applied_events > 0`, even if the complete genome is equal.

Evidence: `sample_new_food_type_idx` and `apply_raw_field_mutation` in
[input_ref/mod.rs](../../crates/v3-core/src/mutation/input_ref/mod.rs), plus
[reproduction steps 11–11b](../../crates/v3-core/src/simulation/actions/reproduction.rs).
**Recommendation:** distinguish operator application, actual genotype change,
and measured behavioral change. First add explicit evidence for no-op events;
decide separately whether phenotype/identity should retain the current trigger.
Do not silently reinterpret historical applied-event counts or alter identity
evolution as an incidental accounting fix.

### M5 — Missing practical recruitment evidence is broader than dispatch coverage

**Measured gap, not proof of an unreachable operator.** Barrier avoidance works
in authored vote fixtures but is absent in the stored evolved steering samples
before and after T19. A new family generally needs declaration and connection;
new modules additionally need useful contents and routing. VM has a priority
bid motif but no action-vote motif. Graph has direct sensor-to-vote edges, but
edge additions also draw compute, memory, routing and unused parameter surfaces.

The generic neighborhood battery holds several inputs at zero. Mutation value
tables describe outcomes of carriers, potentially with multiple birth operators;
they are not isolated causal effects of each operator. **Recommendation:** use
the existing recruitment/steering probes to measure discovery, activation,
retention, cost and per-channel behavior. Consider a generic connection motif
only after a controlled comparison; no barrier-specific built-in behavior is
needed to make the current sensor executable.

### M6 — Queue addressing and documentation need an explicit contract decision

**Confirmed mismatch; exposed by T19.F06's fixed draw width.** Width helper says
12, actual ActionQueue resolution reads `sub_idx/3` without a four-slot cap,
and VM raw mutation can go beyond 11. Width-aware graph draws stay inside 12.
The sensor reference asserts a hard past-width zero result. Distinguish draw
width from runtime width and test the chosen contract. Also correct upstream
width (24), 19 compute kinds, two fieldless VM instructions, and pressure's
decreasing-only wording where stale. See sensor finding S4 and the inventory
sources. These documentation issues do not justify changing simulation semantics
without a feature decision.

## Research and recommended disposition

Primary sources checked 2026-09-23: the
[NEAT paper, §3.1](https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf)
supports separate connection and structural growth operations; it also explains
why new topology may need protection while it is optimized. The
[CGP-Library representation description](https://www.cgplibrary.co.uk/files2/CartesianGeneticProgramming-txt.html)
distinguishes inactive structure from active outputs. Neither source establishes
that Petri's present discovery rates or a proposed replacement policy are good.

| Option | Evidence, fit and tradeoff | Disposition |
| --- | --- | --- |
| Extend the current mutation engine and probes | All 53 operations dispatch; common source widths and stable node IDs already exist. Fits the live-state and mixed-backend contract with local changes. | **Recommended.** First resolve M1, VM draw asymmetries and active parameter targeting; make observation/accounting distinctions explicit. |
| Add generic connection/recruitment motifs or bias productive surfaces | Addresses a plausible multi-step discovery bottleneck; changes the search distribution and may reduce neutral exploration. Existing direct graph wiring is the control. | A measured experiment, not an assumed fix. Start with per-sensor/channel results and independent lineages. |
| Replace mutation with an external NEAT/CGP engine | Offers standard search representations, but replacing heterogeneous VM/graph state, ecological reproduction, costs and telemetry is a new architecture project. No missing dispatch requires it. | Reject for this audit's scope; adopt relevant techniques, not another engine or dependency. |

The order of follow-up should be correctness and honest observation, then
channel access and useful connection surfaces, then discovery experiments.
Retain existing roadmap workflow for any accepted feature; this audit adds no
parallel execution machinery. No default mutation-rate increase is supported:
more attempts cannot substitute for channel coverage or a sound target mapping.

## Verification and limits

Fresh existing tests: `cargo test -p v3-core --lib mutation` — **413 passed,
one ignored**. Shared sensor/runtime/census/steering runs add 106 passing tests,
for **519 passing filtered test executions** in six runs. These runs do not
assert every finding above; M1's counterexample and the mismatched draw/decoder
contracts were established by code inspection, and need focused regression
fixtures when remediated. Catalog coverage was checked against the four enum
lists, per-domain matches, engine key mapping and stable MutationOperator keys.

Stored T19.F06 records were inspected without rerunning benchmarks or changing
their epochs. The comparison with pre-T19 identifies unchanged source paths
and earlier zero-avoidance evidence; it is not a causal experiment isolating
T19's effect. A complete ecological efficacy claim for every operator remains
outside what these tests or three stored world runs can establish.

Documentation validation: `make check-docs` passed (exit 0). The sandbox
prevented Aqua's optional last-used timestamp updates; those warnings did not
prevent policy or quality validation. Local report links and the inventory's
53 operator rows were also checked against the current files and enum lists.
