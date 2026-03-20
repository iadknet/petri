# Feature ideas

## Ideas to improve evolvability of better cognition

Ranked Changes

1. Add better routing primitives.
This is the highest-leverage change. Right now routing is a single scalar, and one tiny weight can silently kill an entire branch, which is exactly what happened with 15 -> 19. I’d prioritize either multi-bit routing, per-target gating, or a “stay / branch / fanout” style router over the current single normalized scalar in routing.rs (line 15). Expected upside: more modular circuits, less dead latent subgraphs. Risk: more branching can explode search if mutation operators aren’t adjusted.

2. Introduce typed intermediate channels instead of anonymous payload slots.
The graph-to-VM handoff is currently just numbered floats in output_slots, and this creature effectively used only one of them. That makes coordinated evolution expensive. I’d add a small fixed semantic bank like dir_hint, threat, food_score, goal_score, memory_key, while keeping some free-form slots. This would sit on top of the existing types.rs (line 5) mechanism. Upside: easier graph/VM cooperation. Risk: too much hand-design can overconstrain emergence.

3. Make memory easier to use and harder to accidentally erase.
The architecture has memory, but it’s 16 untyped scalar slots, and graph nodes can write and clear them in one hop. That favors accidental scratch use over stable working memory. I’d add either protected memory classes like sticky, decaying, tick-local, or explicit write modes. See state.rs (line 71) and cgp.rs (line 111). Upside: much easier emergence of multi-tick state. Risk: stronger memory can let brittle loops dominate unless selection rewards real usefulness.

4. Increase representational richness of perception before increasing radius.
The current perception is smartly compressed, but it is still summary-heavy: area aggregates plus only 4 nearby creature slots in perception.rs (line 74). I would not increase vision_radius first. I’d first add a few more “decision-ready” summaries such as obstacle corridor openness, food-behind-barrier cues, and path asymmetry signals. Upside: better planning signals without huge sensory dimensionality. Risk: more designer bias.

5. Create environments that actually require cognition.
This is probably as important as any runtime change. If local greedy foraging works, evolution will keep rediscovering scripts like Move, Eat, fallback Move. To get richer cognition, you need tasks that reward detours, obstacle memory, delayed payoff, or social inference. Without that pressure, smarter architectures may still collapse into cheap heuristics.

6. Add mutation operators biased toward preserving working subcircuits.
The trace showed live structure and dead structure coexisting. If useful modules are easy to break, evolution stays in shallow reactive basins. I’d add operators that duplicate live branches, retarget inputs while preserving topology, or perturb weights locally around reachable nodes. That fits the existing reachable-bias direction in config and genome analysis. Upside: better hill-climbing on cognition. Risk: too much preservation can reduce novelty.





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
