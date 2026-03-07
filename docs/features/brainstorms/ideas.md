# Feature Ideas

## Bug Fixes

### Action timeline: manual refresh button

Add a refresh button/icon to the action timeline header (far right of the header row). Clicking it refreshes the timeline data on demand.

### Action timeline: improve failure indicator visibility

The current failure indicator (a red bar) is easy to miss and may be too bright/saturated. Consider adding a red "x" icon above the failed action segment for clearer identification, and tone down the red to a less saturated shade.

### Action timeline: only first action per tick is clickable

In the creature inspector's action timeline, if multiple actions happened in a single tick, only the first action's segment is clickable for inspecting metadata. Subsequent actions in the same tick cannot be selected.

## Core Simulation

### Ring-based vision sensors (per-distance-ring splitting)

Split vision into smaller per-ring sensors — one sensor for the ring 1 space away, another for 2 spaces away, another for 3, etc. Each existing area sensor type (food, creatures, barriers, etc.) would be decomposed this way, giving creatures the ability to evolve sensitivity to specific distance ranges rather than a single aggregate over the full radius.

The goal is to lower the evolutionary barrier for creatures to start using sensors. Currently, each sensor covers the full radius, producing a large input array that is hard for random mutations to wire up usefully. Smaller per-ring sensors mean fewer inputs per sensor, making it easier for a creature to stumble into a useful connection through mutation and build on it incrementally.

Key challenge: the perception radius is dynamically configurable at runtime, so the number of sensor inputs per ring must also be dynamic. This affects genome input slot allocation, sensor key enumeration, and potentially the VM input vector sizing. Needs design work on how dynamic sensor counts interact with fixed genome structure — possibly a max-ring cap with unused slots when radius is smaller, or a registry that re-maps slots when radius changes.

### Unified shared memory and VM memory evolvability
Current behavior appears to have drifted away from the original intent: VM nodes have explicit creature-level byte memory, while graph nodes get a separate persistence path through graph-local stateful operators and plasticity state. That split likely makes "memory" feel underused, because graph already has a much easier evolutionary path to temporal behavior than VM memory does.

Recommended direction:
- introduce a single creature-level shared memory surface that both VM and graph can access
- keep graph-local runtime state for integrators, oscillators, plasticity weights, and eligibility traces as backend-internal mechanics rather than the public memory model
- do **not** repurpose graph internal `node_state` as shared memory, because it is indexed by graph topology and would change meaning when internal nodes are inserted, deleted, or reordered

Recommended first implementation shape:
- replace or de-emphasize the current `1024`-byte `u8` memory model in favor of a small shared bank of scalar `f32` memory slots
- start small, likely `8`, `16`, or `32` slots, so useful addressing patterns are easy to discover through mutation
- let VM read and write slots directly
- let graph read a committed snapshot of the shared slots during evaluation, but stage graph-originated writes until after convergence so recurrence and memory writes do not interfere with each other inside the same relaxation loop
- keep reproduction semantics explicit: shared creature memory is inherited, graph-local runtime state is not

Evolvability improvements that should ship with the memory redesign:
- add easier temporal primitives such as one-tick delay, latch, clear/reset, and optionally leaky/decaying memory behavior
- reduce search friction by avoiding `f32 -> u8 -> f32` quantization for the primary memory path
- add mutation motifs that create small useful memory circuits rather than relying only on random independent opcode/node sampling
- examples: insert a `read input -> store slot` motif, insert a `load slot -> compare/use` motif, mutate paired load/store addresses together, or duplicate a small memory-using slice with remapped slots
- consider separating short-term working memory from any future long-term or more persistent memory bank only if a single small shared bank proves insufficient; do not start with multiple exotic memory classes

Expected benefits:
- VM and graph creatures compete on the same memory surface instead of graph getting a separate hidden advantage
- memory use becomes easier to evolve because address space, value representation, and write timing are all simpler
- observability becomes clearer because "uses memory" can mean one thing across both backends

Good validation targets:
- partial observability / delayed-cue tasks where memory should matter
- mutation telemetry showing whether memory-capable motifs are actually entering the population
- inspector/debug views that show which shared slots are read, written, non-zero, or stable over time

### Communication
- Creature can modify metadata fields about itself that are visible to other creatures that "see" it.
- Creature can write to communication channels:
  - Public channel readable by everyone
  - Channel readable only by creatures of same phenotype
  - Channel addressed to a particular creature ID

### Creatures can "infect" other creatures
- Creatures have an output ability to copy parts of their genome into other creatures
- Can take source node ID, target node ID
- Open question: can it copy whole parts of the mesh?
  - Maybe have a branch-depth value. Starts with node id and copies all nodes to specified depth?
  - This whole copying of a "unit" of the mesh feels like it would be best abstracted out into its own thing so that it can be used by multiple features.

### Additional asexual reproduction options
- Creatures can choose to produce offspring with only certain sections of their mesh

### Sexual reproduction
- Creatures can reproduce sexually
- Needs significant design work

## World & Environment

### Complex barrier painting tools
Set of new barrier painting tools that create sections of the map with interesting unique environments:
- Paint a maze
- Paint a spiral
- Paint random noise
- Paint parallel squiggly/jagged lines
- Paint random star patterns

Each needs sub-configuration (spacing, density, etc.). Tool allows drawing a box on the map and filling it with the selected pattern. UI organization needed.

### Configurable barrier topology generation
In the startup config, add a section for configuring initial barrier topology. Way to "add" features (mazes, open spaces, noise) with size/configuration. Algorithm needs to handle competing spatial requirements proportionally.

## Ecology & Diversity

### Food growth-rate layer
Add a world overlay that modifies base food growth rate per cell. New brush for painting positive and negative growth-rate modifiers with configurable size and blur.

This is the slow, world-level habitat layer: some areas become naturally rich, poor, or patchy even before creature behavior feeds back on them.

Likely shape:
- persistent scalar mask over the whole map
- neutral default means normal food growth
- positive values create fertile zones; negative values create sparse or stressed zones
- UI tools could paint local modifiers directly, blur them, or stamp procedural patterns

Main value:
- creates macro-scale ecological structure without requiring creature behavior to generate it
- gives the world a concept of "habitat quality"
- should pair well with other local feedback systems rather than replace them

This idea is more about long-lived environmental heterogeneity than immediate anti-monoculture pressure.

### World startup config for food growth-rate layer
Add startup controls for generating the food growth-rate layer on new maps. Could support noise-driven presets, gradients, or mixed habitat features.

This should stay conceptually separate from short-term local depletion so world fertility and recent creature pressure can be tuned independently.

Possible startup shapes:
- low-frequency noise fields
- a few fertile / infertile bands or blobs
- edge-biased fertility around barriers
- hand-authored mixes of several generators with weights

Important property:
- startup generation should be deterministic from the startup seed
- fertility topology should be visible in debugging/overlay tools so ecology can be interpreted after long runs

This entry is mainly about giving experiments a controllable initial ecology rather than forcing every run to use hand-painted maps.

### Time variation for food growth-rate layer
Allow the growth-rate layer to drift or evolve over time. Options could include slow drift, seasonal oscillation, or moving fertility fronts.

This is aimed more at habitat specialization than immediate local coexistence, so it likely belongs later in the rollout order than more local niche pressure.

Good constraints:
- changes should be slow relative to short-term action loops, so creatures can exploit a patch before it fully moves away
- drift should preserve recognizable structure rather than pure frame-to-frame noise
- the layer should probably have its own timescale controls separate from ordinary food regrowth

Potential effects:
- encourages migration and mode switching
- destabilizes static territorial lock-in
- may reward controllers that track larger-scale world context instead of relying only on local reflexes

Likely downside:
- if this changes too quickly, it may favor only the most generalist strategy and suppress local coexistence rather than improve it

### Occupancy depletion mask
Add a separate per-cell occupancy/depletion mask that reduces food regrowth in places that have been occupied recently. Occupancy deposits local depletion; the mask then recovers over time back toward neutral.

This should be separate from the exogenous growth-rate layer. Effective food growth for a cell would come from combining base growth with both the long-lived fertility layer and the recovering occupancy-depletion layer.

Primary goal: create local negative feedback so one successful strategy degrades its own patch and opens space for different strategies in the same area.

Likely shape:
- each occupied tick deposits depletion into that cell, or into a small radius around it
- depletion recovers smoothly over time, creating a "recently exploited" memory in the terrain
- effective growth rate could be a combination of:
  `base_growth * fertility_modifier * occupancy_modifier`

Why this is attractive:
- it creates anti-monoculture pressure without hard-coding any species logic
- it should encourage patch turnover, movement, and local coexistence
- it gives non-dominant strategies a chance to exploit areas that the dominant strategy has temporarily exhausted

Open questions:
- whether only the current occupied cell is affected or whether occupancy should leave a short trail
- whether all creatures deposit the same depletion or whether larger/older/more numerous creatures should deplete more strongly
- whether this should affect only ordinary food or other renewable resources too

### Carrion resource and scavenging action
When creatures die, leave behind a separate temporary food resource with different dynamics from base food. Carrion should be high-value, decay over time, and likely require separate sensors and a dedicated action to harvest.

This creates a transient scavenger niche that can overlap spatially with grazers, predators, and opportunists instead of forcing species separation by biome.

Likely shape:
- creature death spawns a local carrion deposit
- carrion decays over time and disappears if ignored
- carrion uses its own sensor family and a separate action rather than piggybacking entirely on ordinary `Eat`
- carrion amount could plausibly depend on death cause, stored energy, body complexity, or some capped combination

Main ecological role:
- creates short-lived resource pulses inside already-populated areas
- rewards creatures that can react quickly to local events
- gives predators and nearby opportunists a follow-on niche rather than making kills only reduce competition

Interesting interaction potential:
- local carrion patches may attract both scavengers and predators
- occupancy depletion could make an area bad for grazing while carrion makes it temporarily good for scavenging
- this should increase local ecological layering without requiring geographic separation

### Barrier-surface food and harvest action
Add a third resource channel associated with barriers. This food should be distinct from base food, have a higher energy value, and require a separate harvest-style action rather than normal `Eat`.

Open design questions:
- how far it should extend from barriers before fading to zero
- how quickly it replenishes relative to base food

Primary goal: create edge-specialist niches inside the same map region, encouraging mixed local coexistence rather than only large-scale habitat partitioning.

Likely shape:
- barrier-associated resource forms a stable edge ecology
- barrier food does **not** exist on barrier cells themselves; it exists on passable cells
- a cell is a source cell only if it is orthogonally adjacent to at least one barrier cell
- diagonal-only adjacency does not count as a source
- when a source cell has no barrier food, it can respawn with some probability at density `1`
- barrier food then spreads outward into nearby passable cells while rapidly losing density with distance
- default behavior should fade to zero within only a few cells, with the fading rate configurable
- consumption still happens from the occupied cell, similar to ordinary food, but should remain a separate resource/action channel from base food
- energy value should be higher than ordinary food, but abundance or replenishment should be lower so it stays niche-specific

Why this feels promising:
- barriers already structure movement and visibility, so adding a barrier-linked resource creates meaningfully different microhabitats
- edge specialists can coexist spatially with open-field grazers and scavengers
- a dedicated harvest action makes the niche behaviorally distinct instead of just being "normal food with a different texture," even though the creature still harvests it from its own occupied cell

Good follow-on questions:
- whether some barrier topologies create especially valuable edge patterns
- whether barrier food should be affected by occupancy depletion, or remain a partially independent channel so edge niches do not collapse into ordinary grazing dynamics

## Frontend & UI

### Death statistics and lifespan reporting
Capture and report on creature death statistics. Track causes of death (starvation, predation, etc.) and report aggregate lifespan statistics — mean, median, min, max age at death across the population and over time. Include age-at-death distributions.

Brainstorm how to distinguish "old age" deaths from regular starvation. Currently creatures that die old likely just run out of energy, but it would be valuable to identify whether a creature lived a "full life" vs dying young. Possible approaches: track age relative to some expected lifespan threshold, flag deaths where age exceeds a configurable percentile, or introduce an explicit aging/senescence mechanic that makes old age a distinct death cause.

Surface as a stats panel with historical trends.

### General statistics and observability improvements
- Brainstorm other metrics and observability opportunities
- Refactor stats panels to be better organized and have better coverage
- Active mesh observability report + live panel:
  - Population report focused on "functional mesh" rather than raw genome size. Use reachability from the entry node as the canonical definition of live mesh vs junk DNA.
  - Report distributions over the living population: total mesh nodes, reachable mesh nodes, unreachable mesh nodes, reachable complexity share, top-1 reachable-node complexity share, top-2 reachable-node complexity share.
  - Track a few headline ratios that quickly answer "is cognition still concentrated in 1-2 nodes?": percent of creatures with <=2 reachable nodes, percent with <=5 reachable nodes, percent where top 2 reachable nodes account for >=80% of reachable complexity.
  - Include backend mix in the live mesh: reachable VM node count vs reachable graph node count, plus common reachable topology motifs/signatures so it is obvious whether the population is converging on the same active scaffold or diversifying.
  - Show trend lines over time rather than only current values. This should make it possible to answer whether evolution is actually distributing function across more of the mesh or just accumulating more junk DNA.
  - Provide "interesting specimen" callouts in the report: sparsest live mesh, densest live mesh, highest reachable-complexity share, lowest reachable-complexity share, and maybe representative specimens for the most common live topology signatures.
  - Live metrics panel should show population medians/p90s plus the currently selected creature's values side-by-side, so the inspector can answer "is this creature typical?" without leaving the UI.
  - In the creature inspector, add a compact functional-mesh card: reachable nodes / total nodes, reachable complexity / total complexity, top-1 and top-2 reachable concentration, reachable backend mix, and maybe a small label like "highly concentrated", "moderately distributed", or "diffuse".
  - For performance, compute the heavy population-wide metrics on a sample or periodic snapshot cadence (for example every N ticks) rather than on every frame publish. The panel can show the sample size / freshness timestamp so the user knows how current the report is.
  - Good follow-on UX: clicking a report bucket or exemplar filters/highlights matching creatures on the map or opens the relevant creature in the inspector.

### SelectRow shared component for enum config fields
Uncovered during reconcile-config-panel: the `edge_mode` select was implemented inline in `WorldTopologySection.tsx`. If a second string-enum config field is added to either panel, extract a reusable `SelectRow` component in `config-panel/shared/` following the same data-driven pattern as `FieldRow` and `ToggleRow`.

### FieldLabel primitive extraction
Uncovered during reconcile-config-panel: the label+tooltip+lock-icon layout is duplicated across `FieldRow.tsx`, `ToggleRow.tsx`, and the inline edge_mode select in `WorldTopologySection.tsx`. Extract a `FieldLabel` subcomponent to eliminate the three-site JSX duplication.

### Refactor creature inspector
Within the creature inspector have different tabs (creature summary, mesh viewer, sampler). Needs more brainstorming on tab organization and content.

### Lineage ancestry tracking
Each creature stores its parent_id (some of this exists via CreatureIdentityState). The server maintains a compressed ancestry graph — not every individual, but enough to answer "trace this creature back to its founder" and "find the common ancestor of creatures A and B." Pruned over time: drop branches where all descendants are dead. Pure data infrastructure — no UI, but everything in the species/phylogeny features builds on it.

### Species classification system
Algorithm that groups living creatures into "species" using a multi-signal fingerprint:
- Phenotype similarity (color distance)
- Behavioral profile (action distribution over recent ticks — % move/eat/reproduce/steal)
- Sensor usage (which WorldInputKey values their genome actually reads)
- Lineage proximity (share a recent common ancestor)

Species are dynamic — they form, split, merge, go extinct. Each gets an auto-assigned label and representative color (average phenotype). Weighting of signals could be configurable. For efficiency: assign offspring to parent's species by default, re-cluster periodically (every 100-500 ticks) to detect splits/merges.

### Species stats dashboard
New stats panel tab showing:
- Active species list sorted by population: name/color, population count, mean energy, dominant action profile, dominant sensors, age of oldest member
- Historical species: extinct species with peak population, lifespan (tick born → tick extinct), cause of extinction heuristic (starved? out-competed? predated?)
- Species population chart: stacked area chart over time showing relative population of top N species

### Phylogenetic tree visualization
Interactive tree/graph view:
- Nodes = species (not individual creatures)
- Node size = current population (0 for extinct, but still shown)
- Node color = representative phenotype color
- Edges = lineage descent (species A split into A and B)
- Active species glow, extinct species fade
- Click a species → show stats + highlight members on world map
- Pan/zoom navigation (tree can get large)
- Layout: force-directed graph or top-down timeline tree (time on Y, branching on X)

### Map species overlay
Toggle-able overlay on the world viewport that colors creatures by species rather than individual phenotype. When a species is selected (from tree or dashboard), its members pulse/glow on the map. Could also show species "territory" as a translucent hull around clusters of same-species creatures.

### Genome behavioral summary (stat card)
Compact card in the creature inspector showing derived stats at a glance:
- Sensor coverage — which sensor categories the genome actually reads (local food, neighbors, area summaries, creature detection, introspection) shown as lit/unlit icons
- Action profile — pie chart or bar of action distribution from recent ticks (or static analysis of which actions the genome can produce)
- Memory usage — what % of the shared creature memory bank is read or written to (does the creature use memory at all?)
- Complexity breakdown — total complexity score split by node count, edge count, VM instruction count, graph node count
- Node type distribution — VM nodes vs graph nodes; within graph: math ops vs stateful ops vs plasticity-enabled edges

Relatively cheap — mostly static genome analysis plus recent action history.

### Fade junk DNA in genome viewer
In the genome/mesh viewer, display "junk" DNA as slightly faded — junk mesh nodes, junk VM instructions, etc. Visually distinguishes functional genome elements from non-functional ones, making it easier to see the creature's actual working circuitry at a glance.

### Annotated mesh diagram
Enhanced NodeGraph in the creature inspector:
- Each node gets a human-readable label derived from its function (e.g., "Food Scanner", "Move Decider", "Memory Writer") based on sensors read and actions/targets written
- Data flow arrows colored by signal type (sensor input = green, action output = red, internal routing = grey)
- Sensor inputs labeled at entry points ("FoodHere", "NeighborOccupied[N]", etc.)
- Action outputs labeled at exit points ("Move(N)", "Eat", "Reproduce")
- Active path highlighting — when combined with execution sampler, the path taken through the mesh is highlighted

Key challenge is auto-labeling heuristics for nodes from bytecode/graph structure. Even rough labels ("reads 3 food sensors → outputs to action queue") are far more useful than raw node IDs.

### Genome viewer connection directionality and downstream tree highlighting

In the genome/mesh viewer, better distinguish the directionality of connections (e.g., arrowheads, gradient coloring, or animated flow indicators). When clicking on a node, highlight the full "tree" of interconnected downstream nodes and the connections between them — making it easy to trace signal flow from any point in the mesh.

### Genome diff / comparison tool
Select two creatures (or a parent-offspring pair) and see:
- Structural diff — nodes added/removed/rewired
- Behavioral diff — action profile differences, sensor coverage changes
- Mutation history — what mutations were applied at birth (may need to store mutation events at reproduction time)

Accessible from creature inspector ("Compare with...") or species dashboard ("Compare species representatives").

### Genome search / filter
Find creatures by genome characteristics and highlight results on the map:
- "Show me all creatures that use memory"
- "Show me all creatures with predation actions"
- "Show me creatures with >5 nodes"
- "Show me creatures that read area food sensors"

Needs a query UI and efficient filtering over potentially 100K creatures. Powerful for finding interesting specimens.

### Reconcile config panel with all backend options
Audit the frontend config panel against all backend configuration options. Ensure every config option has a corresponding frontend control, verify all existing controls are functional, and reassess the logical groupings of config elements in the UI. This subsumes the "Expose predation config in UI" idea below — predation config is one example of a missing control, but the full reconciliation should catch all gaps.

### Expose predation config in UI
PredationConfig (steal_cost_rate, kill_complexity_bonus_multiplier) exists in the backend but has no controls in the runtime config panel. Add it.

### Replay / time travel
Record simulation history to allow rewinding, replaying from checkpoints, or exporting timelapse. Could range from simple periodic snapshots to full tick-level recording. Large design space — needs significant refinement.

### Save / load world config
UI ability to save the current simulation config state to a file and load a previously saved config file. On load, apply config values that match current options and silently ignore any that no longer exist or have changed shape — graceful handling of config drift over time.

### Save / load world state
Save and load complete world state (all creatures, map, world state) to/from a compact binary file. Must handle the full simulation snapshot — potentially large data. On load, best-effort restoration: apply what can be mapped to the current schema and gracefully skip or default anything that has changed. Strict backward compatibility is explicitly not a goal given the rapid pace of changes. Lower priority than save/load config.

## Architecture

### Creature endpoint sparse field selection and compression
The `GET /v3/simulation/creature/:id` endpoint returns the full payload (genome, memory, action_log) on every poll at up to 10Hz. With a full 500-entry action_log (~75KB JSON) and no server-side compression, this is ~750KB/s per inspected creature. Three improvements:
1. **`since_tick` query parameter** — return only action_log entries newer than the specified tick. Reduces action_log transfer by ~98% in steady-state (only ~0-10 new entries per poll vs 500).
2. **`tower-http` CompressionLayer** — add gzip/brotli middleware to the Axum server. JSON action logs are highly compressible (repeated field names, similar float patterns). Would reduce payload 80-90%.
3. **Sparse field selection** — query parameter to include/exclude heavy fields (action_log, genome, memory). The frontend already caches genome (set-once) so it doesn't need it after first fetch.

Discovered during creature-action-timeline planning (data transfer analysis).

### Frontend type file domain cleanup
`CreatureDetail` (the HTTP response type for `GET /creature/:id`) lives in `src/types/genome.ts` but is not a genome type — it's an HTTP response envelope. Should be moved to `src/types/http.ts` alongside `LifecycleResponse`, `ConfigResponse`, etc. Touches many import sites so should be a standalone cleanup task.

Discovered during creature-action-timeline planning (type decomposition review).

### Split CreatureInspector into data display and sampler sections
`CreatureInspector.tsx` (220+ lines) mixes two distinct concerns: data display orchestration (stats, phenotype, genome, memory, action timeline) and execution sampler controls/playback (own store, hooks, complex transport state machine). As the inspector grows, splitting into `InspectorDataSections` and `SamplerSection` sub-orchestrators would improve separation of concerns.

Discovered during creature-action-timeline planning (component decomposition review).

### Action cost helper refactor
The complexity multiplier pattern `config.energy.complexity_cost.multiplier(creature.genome.complexity())` is repeated ~10 times across action cost deduction sites in `actions/mod.rs`, `actions/reproduction.rs`, `actions/predation.rs`, and `tick.rs`. A helper method on `Simulation` or a utility function could centralize the cost-deduction-with-multiplier logic and reduce duplication.

### Shared incremental query/projection platform
The viewport transport refactor will add a server-local projection layer, but a larger follow-up still exists: a shared incremental query/projection platform for server, CLI, inspector, metrics, and future replay consumers.

Potential shape:
- Shared projection types outside the current server-local module tree
- Reusable spatial indexes
- Common query surfaces for viewport transport and inspector reads
- Eventual compatibility with replay/export consumers

### Event-driven projection invalidation
Replace repeated projection rescans with event-driven invalidation and incremental updates.

Potential shape:
- Topology-dirty tracking
- Food-region dirty tracking
- Creature-visualization dirty tracking
- Incremental cache updates instead of whole-snapshot rebuilds

### Tick phase system
The tick loop has 5 phases (0, 1, 2, 2.5, 3). A phase-based system where phases are registered handlers would improve extensibility and make it easier to add future tick-level passes without growing the monolithic `run_tick` function.

### Graph evaluation / plasticity decoupling
Currently `graph.rs` calls directly into plasticity modules for post-convergence updates. A more extensible design would have graph evaluation produce "learning events" dispatched to registered plasticity backends, decoupling the graph relaxation loop from the specifics of any learning algorithm.

### Action timeline segment virtualization
At 500 entries × 10Hz, the ActionTimeline re-renders 500 DOM segments each cycle. If profiling shows jank, a windowed/virtualized renderer (only rendering visible segments) could reduce DOM work significantly. Currently acceptable.

Discovered during creature-action-timeline code review (performance assessment).

### Creature detail API spec documentation
The `GET /v3/simulation/creature/:id` endpoint is not documented in `docs/reference/v3-server-api-protocol-spec.md`. The `action_log` field (added by the creature-action-log feature) has no reference spec entry. Should be documented alongside the existing sample sub-endpoint docs.

Discovered during creature-action-timeline architecture review.

### Eventual multi-crate v3-server split
If the internal command/query/transport boundaries stabilize, a later follow-up could split `v3-server` into multiple crates. Intentionally deferred to avoid increasing migration scope while boundaries are settling.

### Reachability-based complexity: separate genome_size from functional complexity

Refactor the current complexity metric into two distinct metrics:

1. **`genome_size`** — matches the current complexity algorithm (total node count, edge count, VM instruction count, etc. across the entire genome). Used for `max_genome_size` cap (renamed from `max_complexity`) to bound total genetic material a creature can carry.

2. **`complexity`** (refactored) — only counts nodes/edges/instructions reachable from the root node. Excludes junk DNA, unreachable VM operations (dead code within VM nodes), and unused graph node internals (dormant input slots, inert sub-expressions). This becomes the metric used for complexity-scaled costs (energy upkeep, action costs, etc.).

This ensures creatures aren't penalized via complexity-scaled costs for carrying inert genetic material, while still capping total genome size to prevent unbounded growth. Most systems that currently reference "complexity" actually care about functional complexity and should use the reachability-based metric. The genome size cap is the main (possibly only) consumer of the total-genome metric.

### Reachability-aware mutation pathways

Mutation currently appears to choose domains and operators generically, while live-vs-junk structure is only visible through separate reachability analysis. A promising direction is to make mutation more aware of whether it is acting on reachable functional mesh or unreachable exploratory reserve.

Primary goal:
- make it easier for evolution to grow and reorganize active mesh structure rather than mostly adding junk or repeatedly polishing the same tiny live core
- preserve some neutral drift and dormant structure so search does not collapse into only mutating currently-live nodes

Recommended first version:
- keep a single mutation engine and current operator families
- when an operator needs a structural target, bias target selection toward reachable nodes/slices/edges rather than sampling the full genome uniformly
- keep a non-trivial probability of choosing unreachable structure so dormant material can still drift, simplify, or become useful later
- add reachability-aware observability so mutation events can be categorized as primarily `reachable`, `unreachable`, or `cross-boundary`

Candidate operator behavior to bias first:
- topology edits like retarget, splice, route swaps, and mesh-slice copy
- graph-internal copy/subgraph/edge mutations on reachable graph nodes
- VM block/gene copy mutations on reachable VM nodes
- input-ref edits on reachable nodes so active controllers can more easily acquire or shed sensors

Potential new operators worth exploring after the biasing layer exists:
- `SplitLiveEdge` — choose a reachable `A -> B` route and insert a new node or short motif between them
- `DuplicateLiveSliceWithReconnect` — copy a reachable slice and force at least one live route through the copied structure
- `ActivateDormantSlice` — reconnect an unreachable slice back into a reachable path
- `PruneLiveBranch` — deliberately remove a reachable side branch to counterbalance growth and create cleaner structural alternatives

Important design caution:
- a strict two-lane system (`functional lane` vs `junk lane`) sounds simple, but may be too rigid as a first implementation
- unreachable structure is often only one reconnection away from usefulness
- many valuable mutations are hybrid by nature: reconnecting dormant structure, cloning live structure and then letting it drift, or routing a live path through partly inactive components
- over-targeting the currently-live core may improve short-term hill-climbing while reducing discovery of genuinely new organization

If a two-lane system is explored later, it should probably remain soft rather than absolute:
- lane probabilities should be configurable rather than hard-coded
- hybrid operators should explicitly exist for crossing between live and dormant structure
- mutation telemetry should show whether the system is merely deepening the current dominant scaffold or actually increasing active-mesh diversity

## Far Future

### Massive introduction of neural nets
The Grand Unified Architecture — mapping biological intelligence to CPU-optimized data structures:

| Node Archetype | Biological Equivalent | ML Equivalent | Core Function |
|---|---|---|---|
| VisionKernel | Retina / Visual Cortex | CNN | Compress dense 2D spatial grids |
| AttentionHead | Parietal Lobe | Transformer | Track sparse, moving neighbors |
| GradientTracker | Olfactory Bulb | Spatial Derivative | Follow chemical/pheromone slopes |
| LiquidReservoir | Auditory Cortex | Echo State | Detect short-term temporal rhythms |
| AssociativeMemory | Hippocampus | Neural Turing Machine | Bridge fuzzy NNs to rigid memory |
| NeuralMatrix | Prefrontal Cortex | Dense FF / RNN | Mix inputs, apply fuzzy logic, learn |
| Neuromodulator | Endocrine System | FiLM / Hypernetwork | Global state shifts |
| SpikingAccumulator | Action Potentials | Integrate-and-Fire | Halt execution to save energy |

## Promoted

- **Fix creature inspector** → `docs/features/needs_refinement/fix-creature-inspector.md`
- **Reconcile config panel** → `docs/features/needs_refinement/reconcile-config-panel.md`
- **Fix passive event listener warnings** → `docs/features/needs_refinement/fix-passive-event-listeners.md`
- **Creature action log** → `docs/features/needs_refinement/creature-action-log.md`
- **Creature action timeline** → `docs/features/needs_refinement/creature-action-timeline.md`
- **Frontend type file domain cleanup** → `docs/features/needs_refinement/frontend-type-domain-cleanup.md`
- **Split CreatureInspector** → `docs/features/needs_refinement/split-creature-inspector.md`
- **Action cost helper refactor** → `docs/features/needs_refinement/action-cost-helper-refactor.md`
- **Shared incremental query/projection platform + Event-driven projection invalidation** → `docs/features/needs_refinement/incremental-projection-platform.md`
- **Tick phase system** → `docs/features/needs_refinement/tick-phase-system.md`
- **Graph evaluation / plasticity decoupling** → `docs/features/needs_refinement/graph-plasticity-decoupling.md`
- **Action timeline segment virtualization** → `docs/features/needs_refinement/action-timeline-virtualization.md`
- **Creature endpoint sparse field selection & compression + Creature detail API spec documentation** → `docs/features/needs_refinement/creature-detail-api-optimization.md`
- **Eventual multi-crate v3-server split** → `docs/features/needs_refinement/v3-server-crate-split.md`
- **Ring-based vision sensors** → `docs/features/needs_refinement/ring-based-sensors.md`
- **Reachability-aware mutation pathways** → `docs/features/needs_refinement/reachability-aware-mutation.md`
- **Reachability-based complexity** → `docs/features/needs_refinement/reachability-based-complexity.md`
