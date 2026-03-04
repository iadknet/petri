# Feature Ideas

## Bug Fixes

- Fix creature inspector. Creature inspector breaks when creatures use newer attributes.

## Core Simulation

### Destructive-only complexity cap
Change complexity cap so it only does destructive changes. Creates room for new additive changes to fit within the complexity cap.

### Creature action log
Each creature maintains a log of all its actions and action metadata. Needs detail on storage limits, what metadata to track, and UI surface.

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

### General statistics and observability improvements
- Brainstorm other metrics and observability opportunities
- Refactor stats panels to be better organized and have better coverage

### Refactor creature inspector
Within the creature inspector have different tabs (creature summary, mesh viewer, sampler). Needs more brainstorming on tab organization and content.

## Architecture

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

The following ideas have been promoted to `docs/features/needs_refinement/`:

- **Storage slots** → `needs_refinement/storage-slots.md`
- **Age energy cost** → `needs_refinement/age-energy-cost.md`
- **Refactor movement** → `needs_refinement/refactor-movement.md`
- **Complexity energy cost** → `needs_refinement/complexity-energy-cost.md`
