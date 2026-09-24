# Post-T19 sensor integration audit

Date: 2026-09-23. Audited checkout: `aef05f90` (T19.F06 closure).
Companion: [mutation audit](post-t19-mutation-audit-2026-09-23.md).
This is an audit and decision record, not an executable feature spec.

## Conclusion

Barrier detection **can be introduced by mutation and can affect action votes**.
Both barrier inputs are sampled, resolved by both backends, and addressable by
connection mutations. An existing post-cutover fixture demonstrates avoidance
after adding a negative barrier edge to the founder's move vote. The concern
is nevertheless supported at the evolutionary level: barrier readers are rare,
and the stored evolved steering samples show no avoidance. That was also true
before T19; the audit does not establish a T19-caused loss of this capability.

Every current sensor family has a runtime path and a mutation path. The material
gaps are the VM's channel-zero creation bias, the preparation needed before an
unrepresented sensor can be connected, incomplete behavioral coverage in the
generic observation battery, and several mismatches between the reference
contract and actual addressing. Preserve the common input resolver and extend
the existing integration and observation machinery.

## Scope and evidence standard

The question is whether each signal can be assembled, declared on a mesh node,
read by a graph edge or VM instruction, connected to an applied effect, and
discovered and retained by evolution. Those are five different claims.

Inspected the sensor contracts and assemblers, runtime resolver and both
backends, mutation samplers and operators, founders, structural census,
steering and neighborhood batteries, inspector labels, and stored goal records.
Compared relevant code with pre-track `437e3358`, the revision identified in
the [T19.F01 readings](../progress/readings/t19-f01.md). Historical records are
evidence, not instructions. No production code, defaults, or roadmap changed.

Constraints: external perception stays frozen for the tick; internal state stays
live; legal cycles remain legal; input growth remains separate from connection
under the current node-type contract. Recommendations are judged on complete
channel access, local mutation reach, measured behavioral influence, ecological
retention, and cost. The existence of a source path is not evidence of a useful
evolved behavior.

## Complete input inventory

All rows use the same node-local `input_refs` vector. Graph access is
`GraphSource::InputLeaf { ref_idx, sub_idx }`; VM access is
`ReadInput { dst, ref_idx, sub_idx }`. The following table covers all 19
non-upstream families plus the upstream family. Food rows exist per configured
ordinary food type. The sampler has 27 equally likely *buckets*, not 27 unique
concrete references: 19 named buckets and eight upstream buckets, each of the
latter drawing one of 24 slots.

| Input | Channels and meaning | Timing | Concrete integration into a node |
| --- | --- | --- | --- |
| `FoodHere(type)` | 1; local food clamped to [0,1] | Frozen | Feed an Eat vote; select the food type separately through `ActionParam(Eat,0)` |
| `NeighborFoodRing(type)` | 8; N, NE, E, SE, S, SW, W, NW | Frozen | Positive input to the corresponding Move vote; compare directions or combine with barriers |
| `NeighborBarrierRing` | 8; barrier presence 0/1 in the same direction order | Frozen | Negative edge to the corresponding Move or Reproduce vote; also usable for route gates |
| `NeighborOccupiedRing` | 8; occupancy 0/1 | Frozen | Suppress occupied destinations, choose social approach or a StealEnergy direction |
| `AreaFoodSummary(type)` | 7; total, gradient x/y, nearest dx/dy/distance, maximum | Frozen, LOS | Combine signed gradient/direction channels with weighted directional votes; gate search on total food |
| `AreaBarrierSummary` | 7; density, blocked-adjacent ratio, gradient x/y, nearest dx/dy/distance | Frozen, LOS | Suppress approach toward nearby barriers or change routing in crowded terrain; density alone has no direction |
| `AreaOccupancySummary` | 7; count, center x/y, nearest dx/dy/distance, crowding | Frozen, LOS | Approach or avoid groups using directional compute nodes and vote weights |
| `NearbyCreatureCore` | 4 ranked creatures × present, relative x/y, distance = 16 | Frozen, LOS | Gate on present, derive direction from relative position, then use Move/StealEnergy votes |
| `NearbyCreatureVitals` | Same 4 slots × energy ratio, reproduce-ready = 8 | Frozen, LOS | Combine with Core for target choice; this row alone supplies no target direction |
| `NearbyCreatureIdentity` | Same 4 slots × kin affinity, lineage match, phenotype similarity = 12 | Frozen, LOS | Combine with Core/Vitals for discrimination; no action accepts a creature-slot ID directly |
| `AgeTicks` | 1; age/reference span, saturating at 1 | Frozen self | Threshold into reproduction or a life-stage route |
| `EnergyCurrent` | 1; current energy/max energy, clamped | Live | Gate expenditure or reproduction; changes during cognition |
| `EnergyConsumedThisTick` | 1; expenditure/max energy, clamped | Live | Compare with a budget and feed Decide/Terminate or route gates |
| `HopsThisTick` | 1; raw dispatched hop count, including current dispatch | Live | Combine with a constant/memory threshold to end deliberation |
| `ActionQueue` | Mutation width 12; four triples of action type, param0, param1 | Live | Condition the next vote on an earlier queued action; addressing discrepancy below |
| `ActionVotes` | 27; current committed pass vote vector | Live at dispatch boundaries | Read competition so far; a visit sees its earlier contribution, not its staged replacement |
| `PreviousPassVotes` | 27; previous pass's final vector | Live between passes | Compare last pass with current evidence; first pass reads zeros |
| `CommitCounts` | 4; Eat, Move, Reproduce, StealEnergy counts | Live between commits | Adapt kind votes to the rising per-kind bar or limit repetition |
| `PreviousOutcome` | 4; energy delta/max energy, success fraction, negative damage/max energy, offspring count | Frozen previous tick | Feed memory, a compute node, routing, or votes; not a within-tick action-result sensor |
| `UpstreamSlot(slot)` | 1 per reference; 24 possible slots | Live carried bus | Read a prior dispatched node's output; the bus persists across passes and resets each tick |

Sources: [input enums and widths](../../crates/v3-core/src/contracts/inputs.rs),
[assembler and compound layout](../../crates/v3-core/src/sensors/perception.rs),
[reducers](../../crates/v3-core/src/sensors/reducers.rs),
[local/self snapshot](../../crates/v3-core/src/sensors/static_inputs.rs),
[runtime resolver](../../crates/v3-core/src/runtime/inputs.rs),
[input sampler](../../crates/v3-core/src/mutation/sampling.rs), and
[action decoder](../../crates/v3-core/src/runtime/action_decode.rs).

The three nearby-creature banks share ranking. Missing slots are all zero, so
Core.present should disambiguate absence from a zero vital or identity value.
Area summaries have different layouts despite sharing width 7. Compound world
indices wrap modulo width; decision compounds do not. Invalid food types read
zero. Bounded-world out-of-bounds neighbors read zero, including barriers:
barrier absence does not guarantee a legal move. Frozen perception describes
the tick-start position, so a second queued move cannot simply reinterpret the
same ring as a newly sensed neighborhood.

Other readable state is deliberately outside `InputReference`:

| Surface | Node access | Mutation route and timing |
| --- | --- | --- |
| Current/previous shared memory, 16 slots each | Graph `SharedMemory { slot, previous }`; VM LoadSlot, LoadSlotImm, LoadSlotPrev | Graph source drawing, VM instruction drawing and slot-address operators. Current memory is live; previous memory is the retained delay line. |
| Queue length and indexed queue fields | VM ReadActionQueueLength, ReadActionQueueType, ReadActionQueueParam | Ordinary VM instruction draw; indexed opcodes can inspect later queued actions without a declared ActionQueue reference. Graphs use InputLeaf queue fields. |
| Graph operator state, previous committed outputs and learned weights | Stateful compute operators, recurrent ComputeNode edges and plasticity | Compute-kind/edge/plasticity mutations. These are live per-visit mechanisms, not extra world sensors or independently named input families. |

Sources: [graph sources](../../crates/v3-core/src/runtime/cgp/sources.rs),
[VM execution](../../crates/v3-core/src/runtime/vm.rs), and
[graph execution](../../crates/v3-core/src/runtime/cgp/execute.rs).

## How evolution connects these inputs today

1. **Declare.** `InputRef.Add` samples any input family and appends it to one
   node. Each named family has probability 1/27 conditional on this operator;
   the three food families additionally sample a food type. It wires nothing.
   The reference consumes genome space and can be pruned while unreferenced.
2. **Read and connect.** On a graph, `AddGraphEdge` can connect that reference
   directly to a vote, parameter, bus, memory, or route sink, without a compute
   node. `RetargetGraphEdge` can redirect an existing edge's source. Both draw
   the chosen input's entire channel width. On a VM, instruction mutation can
   create a read and an `AddVote`; existing live reads can instead be redirected
   by field mutation. Data must reach a register consumed by an applied effect.
3. **Refine.** Graph weights set excitation/inhibition; compute nodes add
   comparisons or state. VM arithmetic and field mutations tune the read and
   its consumer. The graph's fixed sink identity is not itself retargetable;
   selecting another output normally requires adding a connection there.
4. **Recruit a module when needed.** New mesh nodes are blank detours; they
   have no automatically supplied sensor. Copies inherit references and
   structure but may need routing activation. An existing executed node can
   acquire a barrier input without adding a mesh node.

Sources: [input operators](../../crates/v3-core/src/mutation/input_ref/mod.rs)
(`apply_add`, `apply_swap`), [graph operators](../../crates/v3-core/src/mutation/graph/operators.rs)
(`random_graph_source`, `pick_random_surface`, `add_edge`),
[VM operators](../../crates/v3-core/src/mutation/vm/operators.rs), and
[node birth](../../crates/v3-core/src/mutation/topology/birth.rs).

`Swap` cannot turn the founder's food or occupancy ring into a barrier ring.
It preserves **read class and width**, not width alone. With one food type,
each world input is a singleton in its kind; with multiple types, food inputs
can switch type within a family. Introspection scalars can swap among Age,
EnergyCurrent, EnergyConsumedThisTick, and HopsThisTick. Vote vectors can swap
ActionVotes/PreviousPassVotes. Upstream slots can swap. Queue, CommitCounts,
PreviousOutcome, each barrier family, each occupancy family, and each nearby
family have no same-kind alternative. Consumer retargeting remains free to
select a different declared family.

### Barrier example and the evidence it supplies

For direction `d`, add `NeighborBarrierRing` to the founder decision node,
then add a negative edge from that reference's channel `d` to
`ActionVote(Move(d))`. Its contribution is zero without the barrier and
inhibitory when the barrier is present. A sufficiently negative weight changes
the winning direction or prevents that move. Inhibiting a lone vote need not
create an alternative move; other positive evidence determines the outcome.

The test
`founder_one_negative_barrier_edge_avoids_where_the_founder_led_and_changes_nothing_else`
in [steering.rs](../../crates/v3-core/src/neighborhood/steering.rs) exercises all
eight directions. It passed in this audit. This is an **authored connection**
fixture; the sampler and edge operators independently establish a nonzero
mutation path. It does not measure the probability of discovering and retaining
the connection. Nor does one edge wire all eight directions automatically.

## Stored evolutionary evidence

Read the [T19.F06 goal summary](../progress/features/t19-f06-retirement-and-observability-goal.json),
not a new benchmark. World order below is the summary's case order. Census
counts are structural readers at tick 2,000; they do not establish execution
or causal influence. Steering measures a sample of 12 evolved genomes per
world on `steering-v1`, not the entire population.

| World / seed | Final population | Barrier-ring readers | Area-barrier readers | Evolved steering avoided / trials |
| --- | ---: | ---: | ---: | ---: |
| Orchards / 11 | 2,924 | 0 | 2 | 0 / 482 |
| Canyon / 22 | 3,482 | 3 | 0 | 0 / 534 |
| Confluence / 33 | 4,261 | 2 | 0 | 0 / 517 |

The [pre-T19 T13.F07 summary](../progress/features/t13-f07-current-policy-recruitment-transitions-goal.json)
also reported zero sampled steering avoidance: 0/392, 0/480, 0/340. Its final
ring/area reader counts were 35/0, 8/4, and 2/63 respectively, with different
final populations. These trajectories differ in execution semantics and mutation
draws, so raw reader-count changes are not an isolated causal comparison.

Post-T19 final decision-input census is also sparse: ActionVotes readers are
2/3/0 across the worlds; PreviousPassVotes 0/1/0; CommitCounts 0/0/0;
PreviousOutcome 0/4/0. HopsThisTick is much more common at 119/316/124; unlike
the compound decision inputs it can enter existing introspection wiring by
Swap. That is a plausible accessibility explanation, not a measured cause.

For reproducibility, the JSON paths are:

- `deterministic.goal_indicators.population_persistence.per_seed[*].samples[-1].sensor_census`
- `deterministic.goal_indicators.cases[*].mutational_neighborhood.evolved.per_seed[*].steering_pooled`

## Findings and integration priorities

| Finding | Evidence and impact | Disposition |
| --- | --- | --- |
| **S1: Fresh VM compound reads are restricted to channel 0** | `random_vm_instruction`, `apply_insert_read_store_motif`, and `apply_insert_read_bid_motif` hard-code `sub_idx: 0`. Other channels need successive raw-field nudges or an existing copied read. For ActionVotes, channel 0 is Eat; reaching Decide at 26 from that fresh read requires 26 upward sub-index edits if no suitable read already exists. This predates T19, which made the bias more consequential by adding wider inputs. | Extend the existing generators to draw from `sub_value_count`, as graphs already do. Validate every family/channel and keep field nudges for refinement. No new sensor is required. |
| **S2: Main neighborhood battery cannot expose several sensor effects** | `draw_scenario` sets barriers, extended perception, and PreviousOutcome to zero. A correct barrier-dependent change can be classified Silent there; the steering fixture explicitly demonstrates this. Separate steering covers a barred leading move, but not all area/identity/outcome behaviors or later queued actions. | Extend/version the existing observation battery with controlled nonzero channels and behavioral contrasts. Keep old records interpretable. Zero in an unexcited battery is not a disconnected runtime. |
| **S3: Structural census is not causal use** | Census follows reachable structure and wired surfaces, not actual channel reads or changes to applied choices. Zero-weight edges, overwritten effects, dormant control paths, and ignored parameter values can all weaken its meaning. It records family keys, not compound channels. | Retain its structural meaning; add channel-level execution and controlled-ablation evidence in the existing probes before claiming sensor adoption or usefulness. |
| **S4: Queue width and upstream reference prose disagree with runtime** | Mutation limits queue channel draws to 0..12, but `resolve_input(ActionQueue,12,...)` reads a real fifth queued action if present; VM raw-field nudges can reach it. Sensor spec says past-width reads are zero. The spec also says upstream slots >=12 are zero, while runtime and sampling have 24 slots. | Decide whether four queue slots is only a draw policy or a runtime boundary, then align contract and tests. Correct the upstream documentation to 24. Do not silently change behavior to satisfy stale prose. |
| **S5: Unwired extended inputs still cause perception assembly** | `genome_uses_extended_perception` scans every declared reference on every node, including unreachable/unconsumed declarations. Adding an unused area input can increase host work immediately. | If optimizing, extend the existing detection with a conservative, cached consumption analysis covering cycles, control flow and both backends; first measure cost. Preserve frozen snapshot timing. |
| **S6: Energy expenditure has backend-specific read timing** | Mesh computes expenditure at dispatch entry. VM reads add current instruction debt, while graph evaluation and effects reuse dispatch-entry `energy_consumed`, even after graph/plasticity charges; EnergyCurrent does reflect those charges. | Document and test this distinction before treating the two inputs as complementary values at every read. If the intended contract is fully current expenditure, extend graph context construction and recheck identity-edge split assumptions. This is a source-observed timing discrepancy, not a demonstrated cause of weak sensor evolution. |

Sources: [VM generation and mutation](../../crates/v3-core/src/mutation/vm/operators.rs),
[battery](../../crates/v3-core/src/neighborhood/battery.rs),
[census](../../crates/v3-core/src/creature/sensor_census.rs),
[graph slicing](../../crates/v3-core/src/creature/genome/cgp_analysis.rs),
[compound draw widths](../../crates/v3-core/src/mutation/compound.rs),
[runtime resolver](../../crates/v3-core/src/runtime/inputs.rs),
[runtime slot constant](../../crates/v3-core/src/runtime/types.rs),
[sensor spec §§3.4–3.5, 8](../reference/v3-sensor-spec.md),
[perception detector](../../crates/v3-core/src/sensors/perception.rs), and
[tick snapshot assembly](../../crates/v3-core/src/simulation/tick.rs).
For S6, see expenditure construction in
[mesh.rs](../../crates/v3-core/src/runtime/mesh.rs), ReadInput in
[vm.rs](../../crates/v3-core/src/runtime/vm.rs), and the evaluation, plasticity
and effects contexts in [cgp/execute.rs](../../crates/v3-core/src/runtime/cgp/execute.rs).

Additional interpretation cautions: the sensor spec's §6 absence prose says
out-of-range compound indices return zero, conflicting with its explicit world
wrapping rule and implementation. `reproduce_ready` for a nearby creature is
an energy/age threshold, not proof it has a legal destination or will reproduce.
Current [inspector labels](../../frontend/src/components/inspector/inputRefUtils.ts)
show ring directions, but most other compound fields are numeric indices;
field names would make this audit's distinctions easier to inspect.

## Research and recommendation

External primary sources checked 2026-09-23:

- Stanley and Miikkulainen, [NEAT, §3.1](https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf):
  separate connection addition and connection-splitting node addition support
  incremental topology growth. Petri already has both mechanisms; the paper
  does not validate Petri's ecological discovery rate.
- [CGP-Library's representation documentation](https://www.cgplibrary.co.uk/files2/CartesianGeneticProgramming-txt.html):
  connections and outputs can address inputs directly, and inactive structure
  can become active through connection mutation. Petri's node-local declaration
  step adds another prerequisite to this general pattern.

| Option | Material fit and tradeoff | Decision |
| --- | --- | --- |
| Extend Petri's common resolver, full-width graph sampling pattern, steering and recruitment probes | Already respects mixed backends, frozen perception, live internal state and neutral declaration; smallest semantic change. Sparse declaration still requires later connection. | **Recommended**: fix VM access asymmetry and observation coverage first. |
| Add a deliberate sensor-connection mutation that can declare and wire a sensor in one event | Shortens discovery from an unrepresented family; changes mutation probabilities and the current growth/connection boundary. A generic operation is preferable to a barrier-specific circuit. | Experimental alternative only. Compare discovery and retention with the current two-step route before proposing a contract change. Do not silently make `InputRef.Add` active. |
| Give every node a global, directly addressable sensor catalog, following ordinary CGP input addressing | Removes declaration entirely, but rewrites encoding, mutation locality, costs and introspection tooling after an already invasive refactor. | Not justified by the evidence: no runtime sensor family is missing. |

The recommendation is to extend existing components, with no new dependency.
Useful measurements are per-family/channel declaration → consumption → executed
read → changed vote/queue → retained lineage transitions, using existing
recruitment and steering infrastructure. Preserve the distinction between an
authored proof of reach and spontaneous discovery. A future connection-policy
comparison needs independent evolutionary replicates and cost measurements;
this audit does not claim which policy will win.

## Verification

Fresh focused runs on the audited checkout: `cargo test -p v3-core --lib sensors`
(53 passed), `runtime::inputs` (21), `creature::sensor_census` (16),
`neighborhood::steering` (11), `runtime::decision_input` (5), and `mutation`
(413 passed, one ignored). These cover existing behavior; they are not new
end-to-end discovery experiments. The mutation report records the shared
documentation check. No benchmark rerun or production remediation was performed.
