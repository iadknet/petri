# Feature ideas

## Small Modifications

### Tick phase system

The tick loop now has 5 phases (0, 1, 2, 2.5, 3). A phase-based system where phases are registered handlers would improve extensibility and make it easier to add future tick-level passes without growing the monolithic `run_tick` function.

### Graph evaluation → plasticity decoupling

Currently `graph.rs` calls directly into plasticity modules for post-convergence updates (Hebbian weight updates, eligibility trace updates). A more extensible design would have graph evaluation produce "learning events" dispatched to registered plasticity backends, decoupling the graph relaxation loop from the specifics of any learning algorithm.

## Major New Features

### New internal graph nodes can be born with attached inputs

Right now a fresh internal graph node can be structurally valid but functionally useless for several mutation steps, because it may have no meaningful sensor signal attached yet. We could add a graph-specific mutation path where some node-add events create a **small input bundle** alongside the new internal node, so the new structure can start life with a readable signal instead of waiting for several follow-up mutations to line up.

Recommended shape:
- when adding a new internal compute node, optionally also create `1-2` `InputRef` leaves and wire them into the node immediately
- those `InputRef` leaves should point at valid entries in the owning `NodeGenome.input_refs`
- if the chosen input does not exist yet, append a new random `InputReference` first, then point the new leaf at that fresh `ref_idx`
- for compound inputs, either fan out all sub-values through the existing compound mechanism or pick one explicit `sub_idx`
- keep the current junk-DNA mutation path too; this should be an additional mutation mode, not a replacement

This would make graph evolution less brittle by reducing the number of independent mutations required before a new node can do anything adaptive. It also gives us a cleaner place to experiment with higher-level birth patterns later, like "add a comparator node already wired to food-gradient and occupancy inputs" without needing to change the runtime model.


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

### Creatures can spend energy to "cut in line" during action execution
  - When selecting an action, a creature can dedicate "extra energy" to an action
  - When determining action order, the actions with the most extra energy are evaluated first
  - The order of ties should be randomized

### Refactor movement
  - This is going to be a big change that we will have to execute carefully.  I want to refactor the movement actions. Instead of being able to move in different cardinal directions, I want there to be a sense of "Forward" for creatures and  
  then have a two turn actions (turn left, turn right). And "move foward". The turn actions should have small default energy cost, but should still cost something.
  - The zoomed in view of creatures can have a pointy tip for the direction they are facing.
  - Predation should also be forward-facing.

### More advanced sensors
  - area sensor (provides a "map" of surrounding area in radius)

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
