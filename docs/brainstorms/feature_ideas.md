# Feature ideas

## Small Modifications

### Allow creatures to move more than one space
  - add metadata to move action for number of spaces.
  - Extra energy penalty for each additional space

### Add a failed action penalty configuration
  - Is there any existing penalty for failed actions other than the action cost?
  - Add a configuration for an extra energy penalty on failed actions

## Major New Features

### Storage slots
  - add the ability for creatures to pick up and place food and barriers
  - This was implemented in v1, we can refrence v1 implementation
  
### Creatures can spend energy to "cut in line" during action execution
  - When selecting an action, a creature can dedicate "extra energy" to an action
  - When determining action order, the actions with the most extra energy are evaluated first
  - The order of ties should be randomized

### Multiple action queue
  - give the creature an option to perform multiple actions in one turn
  - actions can stack (so it can move n, move e, eat) all in one turn
  - There should be an additional energy penalty for each action after the first
  
### Predation
  - Creature can "steal" energy from neighbors
  - steal action has target direction, energy amount
  - steal action costs energy proportional to energy amount attempted to be stolen
  - this percentage should be configuration (maybe 10% is a good default?)
  - significant energy bonus for killing a creature (based on creature genome complexity)
  
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
