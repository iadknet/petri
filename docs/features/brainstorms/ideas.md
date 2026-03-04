# Feature Ideas

## Bug Fixes

## Core Simulation


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

### Food growth layer
Add a new layer overlay to the map with variable growth rates for food. New brush for painting on this layer with configurable size, growth rate modifier, and gaussian blur. Can paint positive and negative growth rate modifiers.

### World startup config for food growth layer
Add config section for configuring food growth layer generation on new map. Can set different noise patterns. Needs refinement.

### Time variation for food growth layer
Allow food growth layer to drift over time. Options for seasonality? Needs further thought on configuration model.

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

## Frontend & UI

### Death statistics and lifespan reporting
Capture and report on creature death statistics. Track causes of death (starvation, predation, etc.) and report aggregate lifespan statistics — mean, median, min, max age at death across the population and over time. Include age-at-death distributions.

Brainstorm how to distinguish "old age" deaths from regular starvation. Currently creatures that die old likely just run out of energy, but it would be valuable to identify whether a creature lived a "full life" vs dying young. Possible approaches: track age relative to some expected lifespan threshold, flag deaths where age exceeds a configurable percentile, or introduce an explicit aging/senescence mechanic that makes old age a distinct death cause.

Surface as a stats panel with historical trends.

### General statistics and observability improvements
- Brainstorm other metrics and observability opportunities
- Refactor stats panels to be better organized and have better coverage

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
- Memory usage — what % of the 1024-byte memory buffer is written to (does the creature use memory at all?)
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

### Eventual multi-crate v3-server split
If the internal command/query/transport boundaries stabilize, a later follow-up could split `v3-server` into multiple crates. Intentionally deferred to avoid increasing migration scope while boundaries are settling.

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
