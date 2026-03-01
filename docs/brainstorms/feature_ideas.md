# Feature ideas

## Small Modifications

## Major New Features

### Graph nodes can execute actions through deferred queue outputs

Instead of emitting a `WorldAction` directly, graph backends would gain a small set of **deferred action-effect outputs** that mirror the VM’s action-queue behavior. During graph relaxation, these nodes behave like normal graph nodes and only produce scalar values. After the graph converges, the runtime performs a single post-convergence effect pass that interprets specific graph node kinds as queue operations. This keeps graph evaluation deterministic, avoids repeated side effects across relaxation passes, and fits the `multi-action-queue` model where the action queue is owned by the mesh executor rather than by an individual node result.

Proposed graph-side effects:
- `WriteActionMeta(slot)`: write a value into a staged action metadata buffer
- `PushAction(action_type)`: decode an action from the staged metadata and push it onto the mesh-owned action queue
- `PopAction`: remove the most recently queued action
- `ExecuteActionQueue`: mark the current mesh hop as terminal and return the queued actions for Phase 2 execution

Ordering must be explicit and deterministic. Recommended rule:
1. Run graph relaxation to convergence
2. Apply staged-value writes first (`CustomOutput`, `RouterOutput`, `WriteActionMeta`)
3. Apply queue mutations next (`PushAction`, `PopAction`)
4. Apply terminal check last (`ExecuteActionQueue`)

Within each phase, process internal graph nodes in internal-node-index order. This means conflicts like “push and pop are both active” are resolved by node order, not by ambiguous graph topology. If energy is exhausted before convergence completes, none of the deferred action effects are committed.


### Storage slots
  - add the ability for creatures to pick up and place food and barriers
  - This was implemented in v1, we can refrence v1 implementation
  
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
