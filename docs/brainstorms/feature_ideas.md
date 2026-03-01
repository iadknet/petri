# Feature ideas

## Small Modifications

### Deduplicate graph relaxation loop via tracer trait

`graph.rs` and `traced_graph.rs` duplicate the entire relaxation loop (~100 lines each). The traced version adds per-pass `GraphPassTrace` recording but is otherwise identical. A tracer trait pattern (`NoopTracer` vs `RecordingTracer`) with a single generic `execute_graph_node_impl<T: GraphTracer>()` would eliminate the duplication, prevent future divergence, and ensure every graph feature only needs one implementation.

Design considerations:
- `graph.rs` uses scratch buffer reuse (`std::mem::take` from `GraphRuntimeState`); `traced_graph.rs` allocates fresh vecs. The generic implementation should support the optimized path.
- The tracer trait needs methods for: pass start (energy cost), per-node evaluation (weighted inputs, state transitions, output), pass end (max delta).
- `NoopTracer` methods should be `#[inline]` and zero-cost (no allocation, no recording).
- State backup differs: `graph.rs` uses `extend_from_slice` into a scratch buffer; `traced_graph.rs` uses `clone()`. The generic version should use the scratch-buffer approach.

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


### New internal graph nodes can be born with attached inputs

Right now a fresh internal graph node can be structurally valid but functionally useless for several mutation steps, because it may have no meaningful sensor signal attached yet. We could add a graph-specific mutation path where some node-add events create a **small input bundle** alongside the new internal node, so the new structure can start life with a readable signal instead of waiting for several follow-up mutations to line up.

Recommended shape:
- when adding a new internal compute node, optionally also create `1-2` `InputRef` leaves and wire them into the node immediately
- those `InputRef` leaves should point at valid entries in the owning `NodeGenome.input_refs`
- if the chosen input does not exist yet, append a new random `InputReference` first, then point the new leaf at that fresh `ref_idx`
- for compound inputs, either fan out all sub-values through the existing compound mechanism or pick one explicit `sub_idx`
- keep the current junk-DNA mutation path too; this should be an additional mutation mode, not a replacement

This would make graph evolution less brittle by reducing the number of independent mutations required before a new node can do anything adaptive. It also gives us a cleaner place to experiment with higher-level birth patterns later, like "add a comparator node already wired to food-gradient and occupancy inputs" without needing to change the runtime model.


### Generic outcome-modulated learning for open-ended creatures

Instead of teaching creatures that "food is good" or "movement was correct", we could add a **generic outcome signal bank** that exposes a small set of post-tick consequences, then let evolution decide which circuits use those signals for plasticity. This keeps learning open-ended and ecology-driven rather than baking a hand-written objective into the runtime.

Recommended shape:
- add a fixed-width `OutcomeSignalBank` with generic channels such as `energy_delta`, `action_success`, `damage_delta`, and `offspring_success`
- these signals should describe **what happened**, not **what should matter**
- graph internal nodes or edges with plasticity enabled can choose one `reward_source` from that bank
- plastic edges maintain an eligibility trace so that a later outcome can reinforce or weaken earlier activity
- delayed reward should work by updating traces when a circuit is active, then applying reward later with something like `dw = learning_rate * outcome_signal * eligibility_trace`
- this lets earlier sensing and routing decisions receive credit when the payoff only appears a tick or two later, such as spotting food, moving toward it, and only then eating
- keep the current unsupervised Hebbian rules too; reward-modulated plasticity should be an additional learning mode, not a replacement

This would fit Petri well because it preserves the existing three-loop model:
- evolution discovers which circuits are plastic and which outcome channels they listen to
- within-lifetime learning adapts those circuits based on actual consequences
- ecology determines which generic signals end up being useful in practice

The main tradeoff is complexity: we would be adding more per-creature runtime state, more delayed-credit machinery, and more room for strange learned strategies. But that is also the point. It gives creatures a path to develop local habits and niche-specific adaptations without forcing the engine to define one universal reward function.


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
