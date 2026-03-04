# Feature ideas

## Bug fixes
 - Fix creature inspector. Creature is inspector when creatures use newer attributes.



## Small tasks
 - Change complexity cap so it only does destructive changes. We want to create room for new addative changes to fit in the complexity cap.


## Deferred Architecture Follow-Ups

### Shared incremental query/projection platform

The viewport transport refactor will add a server-local projection layer, but a
larger follow-up still exists: a shared incremental query/projection platform
for server, CLI, inspector, metrics, and future replay consumers.

Potential shape:
- shared projection types outside the current server-local module tree
- reusable spatial indexes
- common query surfaces for viewport transport and inspector reads
- eventual compatibility with replay/export consumers

### Event-driven projection invalidation

The first viewport transport refactor will likely publish projections on a
scheduled cadence plus synchronous mutation hooks. A cleaner follow-up would
replace repeated projection rescans with event-driven invalidation and
incremental updates.

Potential shape:
- topology-dirty tracking
- food-region dirty tracking
- creature-visualization dirty tracking
- incremental cache updates instead of whole-snapshot rebuilds

### Tick phase system

The tick loop now has 5 phases (0, 1, 2, 2.5, 3). A phase-based system where phases are registered handlers would improve extensibility and make it easier to add future tick-level passes without growing the monolithic `run_tick` function.

### Graph evaluation → plasticity decoupling

Currently `graph.rs` calls directly into plasticity modules for post-convergence updates (Hebbian weight updates, eligibility trace updates). A more extensible design would have graph evaluation produce "learning events" dispatched to registered plasticity backends, decoupling the graph relaxation loop from the specifics of any learning algorithm.

### Eventual multi-crate `v3-server` split

If the internal command/query/transport boundaries stabilize, a later follow-up
could split `v3-server` into multiple crates. That is intentionally deferred
from the current refactor to avoid increasing migration scope while the new
boundaries are still settling.


## Feature ideas

### General statistics and observability improvements
  - Brainstorm other metrics and observability opportunities
  - Refactor stats panels to be better organized and have better coverage


### Complexity energy cost
 - companion to complexity cap that magnifies the energy cost of all actions based on complexity

### Age energy cost
 - Set an "age cap"
 - Energy cost for actions increaseses when you get close to age cape
 - Gradual increase, then bigger increase closer to cap
 - Configureable age and max energy penalty multiplier.
 - Sane defaults - something like age 500, multiplier maxes out at x10

### Add a action log for  creature

Each creature will maintain an log of all its actions and action metadata.


### Ability to paint complex barriers

I'm imaginging this as a set of new barrier painting tools. That can create sections of the map with interesting unqiue "environments"


  - Paint a maze
  - Paint a spiral
  - Paint random noise
  - Paint a parralell squiggly/jagged lines
  - Paint random star patterns
  
  Are there any other interesting ideas I'm missing?

This will be a more complicated tool than the other painting tools. Each of these probably needs some sub-configuration... like width of spaces between lines in the maze and spiral, density of random noise, etc.

I'm imaging the tool will allow drawing a box on the map and then it will fill that area with the selected items.

We need to think about how to organize the UI.

### Ability to generate map with configurable barrier topology

In the startup config, add a section for configuring inital barrier topology.

This will be a completely different type of configuration that will need a lot of thought on how to design it.

I'm thinking there will be a way to "add" features to be generated.

For example, you can "add" a maze and configure its size and other maze configuration values.

There should be options for adding all the different barrier configurations.

There should also be an option for "empty space" which defines a large/medium/small area of space with no barriers.


The algorithm for generating the map might get complex with competing requirements. If there are 5 large open spaces, 5 large mazes, and 5 large areas of random noise... how does it fit them all?   I'm thinking it takes all the sizes of the different features and adds them all up, then divides it proportionally based on all the demands.


### Food growth layer

Add a new layer overalay to the map that has variable growth rates for food.

There would be a new brush for painting on this layer.  The brush would need to be able to be resized with configurable size, growth rate modifier, and guassian blur.

Could paint on positive and negative growth rate modifiers.


### World startup config for food growth layer.

Add config section for configuring food growth layer generation on new map.

Can set different noise patterns?  This needs refinement.

### Add time variation for food growth barrier

This needs further thought and refinement.  Allow food growth barrier to "drift" over time? Should there be an option for seasonality?  What kind of configuration can we do?

## Big refactors / changes

### Refactor creature inspector

Within the creature inspector have different tabs (creatur summary, mesh viewer, sampler).

Need to brainstorm more on this.


### Storage slots

- Add creature inventory with a fixed slot count, controlled by a new config option `inventory.slot_count` with default `5`.
- Each slot stores one of:
  - `Empty`
  - `Barrier`
  - `Food { density }`
- Inventory is creature-local runtime state. Offspring spawn with empty inventory; slot count comes from config rather than inheritance.

- Add two new actions:
  - `Pickup { direction, slot }`
  - `Place { direction, slot }`
- Both actions use the existing 8-neighbor `Direction` model only. There is no self-cell pickup/place behavior.
- Current-cell food remains `Eat`-only.

- `Pickup(direction, slot)` semantics:
  - Resolve the target neighbor using existing world edge rules.
  - Fail if the target is unresolved, occupied by a creature, or the slot is invalid/full.
  - If the target cell has a barrier, remove the barrier and store `Barrier` in the slot.
  - Otherwise, if the target cell has food density `> 0.0`, clear that food from the cell and store `Food { density: previous_density }`.
  - If the target has neither barrier nor food, the action fails.
  - If a cell somehow has both barrier and food, barrier takes priority.

- `Place(direction, slot)` semantics:
  - Resolve the target neighbor using existing world edge rules.
  - Fail if the target is unresolved or the slot is invalid/empty.
  - If the slot contains `Barrier`, placement succeeds only if the target cell is unoccupied and barrier-free. On success, clear any food on that cell and place the barrier.
  - If the slot contains `Food { density }`, placement succeeds only if the target cell is not a barrier and adding the stored density would not exceed `world.food.max_density`.
  - Food placement should fail on overflow rather than clamp, so stored food density is conserved.

- Add creature introspection inputs for storage slots.
- Initial slot introspection should expose slot occupancy and item kind only, not stored food density.
- Stored food density must still be preserved internally and should be visible in inspector/debug surfaces, just not exposed to creature cognition initially.

### Refactor movement
  - This is going to be a big change that we will have to execute carefully.  I want to refactor the movement actions. Instead of being able to move in different cardinal directions, I want there to be a sense of "Forward" for creatures and  
  then have a two turn actions (turn left, turn right). And "move foward". The turn actions should have small default energy cost, but should still cost something.
  - The zoomed in view of creatures can have a pointy tip for the direction they are facing.
  - Predation should also be forward-facing.

### Communication
  - creature can modify some metadata fields about itself that are visible to other creatures that "see" it.
  - Creature can write to "comunication" channels
    - Public channel that can be read by everyone
    - Channel that can only be read by creatures of same phenotype
    - Channel that can be addressed to a particular creature id

### Creatures can "infect" other creatures
  - creatures have an output ability to copy parts of their genome into other creatures
  - can take source node id, target node id
  - can it be more complicated than that? Copy whole parts of the mesh?

### Additional asexual reproduction options 
  - Creatures can choose to produce offspring with only certain sections of their mesh

### Sexual reproduction
  - creatures can reproduce sexually

### Massive introduction of Neural nets

The Grand Unified Architecture
If we step back and look at the V3 BackendDef enum now, we have perfectly mapped the entire spectrum of Biological Intelligence to CPU-optimized Data Structures:
Node Archetype	Biological Equivalent	Machine Learning Equivalent	Core Function in V3
VisionKernel	Retina / Visual Cortex	Convolution (CNN)	Compress dense 2D spatial grids.
AttentionHead	Parietal Lobe	Transformer / Self-Attention	Track sparse, moving neighbors/swarms.
GradientTracker	Olfactory Bulb (Nose)	Spatial Derivative	Follow chemical/pheromone slopes.
LiquidReservoir	Auditory Cortex	Echo State / Reservoir	Detect short-term temporal rhythms.
AssociativeMemory	Hippocampus	Neural Turing Machine	Bridge fuzzy NNs to rigid 1024b memory.
NeuralMatrix	Prefrontal Cortex	Dense Feedforward / RNN	Mix inputs, apply fuzzy logic, learn.
Neuromodulator	Endocrine System	FiLM / Hypernetwork	Global state shifts (Adrenaline/Sleep).
SpikingAccumulator	Action Potentials	Integrate-and-Fire (SNN)	Halt execution to save energy (Sleep).
VM / Graph	Spinal Cord / Reflexes	Assembly Code / Logic Gates	Hard math limits and physical actions.
