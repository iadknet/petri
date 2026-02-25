# Feature ideas

## Bug fixes
  - Fix "blurring" when dragging full world in canvas, does not blur when more zoomed in. 

## Small Modifications

### Add "complexity" score metadata attribute to creatures
  - Need to figure out how to compute compute creature "complexity"
    - Number of nodes?
    - Total genome size?
    - Any other ideas?

### Change phenotype drift
  - instead of weighted random channel have a set channel that increments/decrements depending on polarity
  - Random chance the channel target changes
  - Make this random channel change probability configurable
  - This makes it so creature's appearance drifts apart more subtly.
  
### Allow creatures to move more than one space
  - add metadata to move action for number of spaces.
  - Extra energy penalty for each additional space

## UI Cleanup
  - Add "reset" button for each confuration option, that resets to default. This conditionally appears if value has changed from default.
  - Change "restart" behavior to use current runtime values.
  - Limit "startup" config options to only those values that apply to word initialization and cannot be changed at runtime.
    - Remove any runtime config options that cannot actually be adjusted at runtime.
    - In the end, there should be no overlap between "startup" and "runtime" config options.
  - Add an informative tooltip
  - provide zoom in / zoom out buttons (translucent buttons in upper-right of canvas)

## Major New Features

### Creature inspector
  - click on creature to open new modal or window with creature inspector
  - a fun whimsical illustration representation of node graph
  - Pan with additional creature metadata
  - Live updating up creature stats
  - have the ability to see creature memory somehow
  
### Creature inspector execution sampler
  - In creature inspector have an "execution sampler" button.
  - captures full execution over a few ticks.
  - Pauses world after capturing.
  - Have the ability to visualize the execution in slow motion or step-by-step
    - "zoom in" on each node as it is evaluated
    - show all inputs used by node
    - for graph nodes show a visualization of the graph
    - for vm nodes, show the code
  - Show all inputs that are used for that node on the left, show outputs on the right
  - As the node "executes" highlight that particular part of the node, highlight all values changed

### Drawing of barriers and food
  - Add the a ability to draw barriers

### Storage slots
  - add the ability for creatures to pick up and place food and barriers
  - This was implemented in v1, we can refrence v1 implementation
  
### Rayon parallelism
  - refactor tick processor to allow for paralell execution
    - snapshot world state for tick
    - loop through cognition, collect actions for later processing
    - randomize order of action execution
  - implement rayon

### Predation
  - Creature can "steal" energy from neighbors
  - steal action has target square, energy amount
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
