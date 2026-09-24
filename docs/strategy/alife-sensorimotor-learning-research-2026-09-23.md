# How ALife systems make sensors useful

Research date: 2026-09-23. Status: evidence review, not a feature specification
or a decision to change production defaults.

## Decision in brief

**Test group-level wiring together with coordinated mutation and evolved
plasticity. Do not assume that supplying neutral edges or enabling ordinary
Hebbian updates is sufficient.**

There are working precedents for organisms inheriting connection rules and
learning rules instead of every final connection weight. There are also
successful recent ALife ecosystems that retain evolved weights and do no
synaptic learning. The literature supports several alternatives, not one
settled solution.

The closest architectural precedent is older: Polyworld combines neural
groups, genetically specified connectivity, randomly initialized weights,
and lifetime Hebbian learning. Recent work from 2024–2026 provides more
controlled evidence about evolving learners, rewiring during life, and the
conditions under which plasticity helps.

**Correction to an overly strong reading of our earlier discussion:** explicit
reward during life is not universally necessary. Evolution can discover
specialized learning rules that extract useful information from observations
and activity. That does not mean an arbitrary correlation rule knows which
movement is useful. The learning machinery itself embodies information
discovered by evolution.

## What problem is actually being solved?

Petri's observation is that one dimension of a food ring becomes useful while
others remain unused. Three distinct obstacles could produce that result:

1. **Access:** a sensor channel has no effective route to an action.
2. **Coordination:** evolution must discover the same spatial relationship
   separately for several directions.
3. **Adaptation:** an organism cannot adjust a relationship through experience.

Adding a complete matrix directly addresses access. A spatial encoding or
coordinated mutation addresses coordination. Plasticity addresses adaptation
only when its activity, learning signals, and update rules are informative.
These mechanisms can be combined, but should be measured separately.

The current Petri Move group contains eight directional votes. A complete
food-ring-to-Move projection therefore contains **64 weights**. Providing
these together does not, by itself, make their later changes coordinated.
See the local [ring-wiring experiment](ring-wiring-experiment-2026-09-23.md).

## What evolving ecosystems actually do

### Polyworld: evolve connections between groups; learn individual weights

Yaeger's original Polyworld organizes sensory, internal, and behavioral
neurons into groups. Genes specify group properties and, between groups,
connection density, spatial ordering/distortion, and learning rates. Individual
weights start randomly; a period of noisy sensory stimulation precedes life,
and Hebbian adaptation continues during life. The system produced viable
ecological behavior, although some initialization procedures used explicit
fitness-based population support.

This is a close precedent for making a sensor group available as a unit while
leaving individual weights to develop. It is not an always-dense, zero-weight
network. Nor does the original demonstration isolate Hebbian learning's
benefit with the comparison Petri needs.

Sources: [original ALife III paper, published 1994, sections 6–7](https://shinyverse.org/larryy/Yaeger.ALife3.pdf)
and [author's project overview](https://shinyverse.org/larryy/polyworld.html).

### EmEvo, ALIFE 2024: evolve motivations; learn the controller during life

Kanagawa and Doya use a fixed neural controller connected to sensory inputs
and two motor forces. Every newborn receives randomized controller weights
and learns through PPO. What it inherits is a mutated reward function for
food consumption and motor action. Survival and reproduction select those
reward functions in a continuing birth-and-death simulation.

This directly demonstrates a route that does not require inheriting useful
controller weights. It supplies connectivity and a powerful learning algorithm
up front. Viability depends on allowing enough opportunity to learn; the
paper reports sensitivity to metabolism. It does not test adding new sensor
types to an already evolved controller.

Sources: [paper](https://arxiv.org/html/2406.15016v1),
[reproduction code](https://github.com/oist/emevo/tree/alife2024).

### JaxLife, 2024: connected controllers with evolved weights and recurrent state

JaxLife supplies encoders for observations, attention, an LSTM, and action
outputs. Offspring inherit mutated weights. Weights remain fixed within an
individual's life; recurrent state provides memory. Its evolving ecosystem
supports interactions involving other agents, terrain, and programmable
robots.

For Petri, the relevant distinction is architectural: input integration is
provided by the controller design. Evolution does not have to assemble every
sensor-to-action route through separate sparse-edge mutations. This does not
establish that newly evolved interfaces will become useful automatically, or
that synaptic plasticity is required.

Source: [Lu et al., JaxLife: An Open-Ended Agentic Simulator](https://arxiv.org/html/2409.00853v1).

### Continuous grid ecology, EvoApplications 2025: exploit spatial structure

Taylor-Davies and colleagues study kin-directed feeding in a persistent world
with births and deaths. Agents receive local grid observations through a
convolutional neural network. Weights are inherited with mutation and stay
fixed during life. Actions include moving forward and turning, rather than
eight independent world-direction outputs.

This offers a different answer to duplicated directional discovery: provide a
controller with spatial reuse and an embodied action interface. Convolution
does not imply rotational invariance, and relevant observation channels are
supplied by the researchers. The study demonstrates ecological behavior, not
a controlled comparison of ring wiring strategies.

Sources: [author preprint](https://arxiv.org/html/2411.10536v1),
[2025 publication](https://link.springer.com/chapter/10.1007/978-3-031-90062-4_31).

**Shared limitation:** these ecosystems mostly provide their sensory and motor
interfaces beforehand. Avoiding disconnected inputs is easier than evolving
the useful integration of a genuinely new modality. Their success does not
settle Petri's open-ended input-growth problem.

## Recent experiments that isolate learning mechanisms

### Group projections and evolved learners, ALIFE 2024 → Artificial Life 2026

Arnold, Suzuki, and Arita's 2024 study is particularly close to the group
proposal: projections between neural columns are complete weight matrices.
Evolution changes projections and neuromodulatory machinery. Agents learn
navigation despite randomized sensorimotor alignment.

However, its update mechanism is a neural network producing weight targets
and update strengths, considerably richer than ordinary Hebbian learning.
Initial weights also evolve, and guided mutations use parental experience
and optimization. This is not a demonstration that neutral matrices plus
unguided mutations suffice.

Source: [Breaching the Bottleneck, full author paper](https://arxiv.org/pdf/2404.12631).

The 2026 journal extension reports that reward-driven learning can serve as
an evolutionary starting point for learning that subsequently uses non-reward
information. It extends evaluation to quadrupeds with varying bodies. This
supports evolving the learning process and shows that explicit reward need
not drive every lifetime update. It does not eliminate selection for useful
learning across generations.

Sources: [2026 journal abstract and publication record](https://pubmed.ncbi.nlm.nih.gov/42518295/),
[authors' project](https://github.com/sovvel/hbl).
Access limitation: the 2026 full journal text was inaccessible; the 2024 full
paper was inspected. The project links 2D code but says 3D code is pending
permission as of this review. These are related results from one research
program, not independent replications.

### LNDP, ALIFE 2024: develop and rewire during life

Plantec and colleagues evolve a shared developmental program that changes
weights and creates or prunes connections using activity and experience.
Some conditions include learned spontaneous activity before environmental
interaction. Structural plasticity improves results on some tasks, including
changing food locations, compared with restricting development to weight
changes.

The foraging task is a five-position one-dimensional world, alongside classic
control benchmarks. This is not a self-sustaining ecology. Training uses a
substantial evolutionary search and shows variation across runs.

The useful implication is that connection removal can participate in successful
adaptation. These results argue against treating permanent connectivity as an
obvious requirement. Lifetime pruning and genetic edge deletion remain
different mechanisms, so this is not a direct verdict on Petri's mutation.

Sources: [Evolving Self-Assembling Neural Networks](https://arxiv.org/html/2406.09787v1),
[authors' implementation](https://github.com/erwanplantec/LNDP).

### Foraging agents, Artificial Life 2025: the interface shapes the learned rule

Giannakakis, Khajehabdollahi, and Levina evolve a motor controller together
with a plastic food-evaluation pathway. Consumption provides feedback about
food value. Different interfaces between valuation and action lead to
different evolved learning rules; a constrained binary interface can improve
compatibility between independently evolved components.

The experiments use a curriculum that initially supplies informative sensory
weights and progressively perturbs them. They therefore do not demonstrate
discovery of the whole learner from an uninformative start. Their relevance
is that architectural boundaries and available feedback can matter as much
as adding more parameters.

Sources: [full author preprint](https://arxiv.org/html/2403.13649v1),
[2025 publication record](https://pubmed.ncbi.nlm.nih.gov/39485363/).

### A useful negative result, GECCO companion/workshop 2026

Medvid and colleagues test Hebbian and anti-Hebbian updates on developmentally
grown recurrent controllers. Carefully chosen plasticity parameters can help
particular networks; indiscriminate choices can harm them. Crucially, their
evolutionary comparison does not establish an advantage over evolution
without plasticity in the tested conditions, despite benefits found by
searching plasticity settings afterward.

This separates two claims Petri must also separate: a useful learner exists,
and evolution can find that learner within the available budget. The tasks
are CartPole and Acrobot, so ecological conclusions would be premature.

Source: [Activity-Dependent Plasticity in Morphogenetically-Grown Recurrent Networks, camera-ready v2](https://arxiv.org/html/2604.03386v2).

## Foundational results that clarify the alternatives

**Weights need not be individually inherited.** Najarro and Risi's NeurIPS
2020 experiments evolve generalized Hebbian update rules while randomizing
plastic connection weights for each episode. Rules include pre-activity,
post-activity, their product, and a constant term. No explicit lifetime reward
is required for those updates; evolutionary fitness selects the rules. Their
quadruped controller makes a strong existence case. The visual driving
experiment also uses nonplastic evolved components, so the result should not
be generalized to every weight in every architecture. Per-connection learning
parameters can also make the genotype larger, not smaller.
[Paper](https://arxiv.org/pdf/2007.02686).

**Coordinated structure can reduce repeated discovery.** HyperNEAT generates
weights from neuron coordinates, allowing a genetic change to express a
repeated spatial relationship. Work on coordinated quadruped gaits is a useful
precedent for exploiting directional regularity. It is not evidence that a
complete matrix of independent zero weights provides the same benefit.
[Clune et al., 2009](https://www.cse.msu.edu/~ofria/pubs/2009CluneEtAl.pdf).

Petri inference: a small cyclic-offset representation or coordinated mutation
could test this principle without importing HyperNEAT. Its disadvantage is
an imposed geometric prior; exceptional directions need a way to diverge.

## What this means for VM organisms

Learning need not take the form of floating-point neural weights.

**Markov Brains** have evolved feedback gates that modify probabilistic
input/output tables during life. Feedback routing and learning characteristics
are genetically specified. This demonstrates lifetime adaptation in a
gate-based controller rather than a conventional neural network.
[Sheneman and Hintze, 2017](https://pubmed.ncbi.nlm.nih.gov/29196623/).

**SignalGP** uses tag-matched event handling so a sensory event can invoke a
function without first evolving a polling-and-dispatch sequence. This
addresses access to information, not automatic learning of a useful response.
[Lalejini and Ofria, 2018](https://lalejini.com/pubs/evolving-event-driven-programs-with-signalgp.pdf).

**A 2026 evolutionary-RL preprint** compares neural policies with differentiable
decision-list policies in a foraging survival model, including learning-only,
evolution-only, and combined conditions. It supplies a recent example of
lifetime learning in a program-like representation. It is not a bytecode VM,
and its bounded survival trials do not establish long-term open-endedness.
[Roupassov-Ruiz and Zuo, v2](https://arxiv.org/html/2601.04365v2).

Petri inference: the VM equivalent of a plastic projection could be a reusable
routine operating on an input vector and mutable parameter memory. Expanding
every possible connection into unrelated instructions is only one encoding.
Supporting an integrated routine would be a separate design decision, not a
capability demonstrated by the current graph/VM translation experiment.

## Options for Petri, ranked for the current question

| Option | What it directly addresses | Main tradeoff | Assessment |
| --- | --- | --- | --- |
| Group wiring plus coordinated spatial mutations | Access and repeated directional discovery | Introduces a geometry prior; can change several behaviors at once | Best low-cost continuation of the existing screen |
| Group wiring plus evolved plasticity | Within-life adaptation and potentially reusable learning | Needs informative experience, stable updates, and time to learn | Strong research candidate; compare with static controls |
| Fixed connected controller with a supplied RL algorithm | Reliable access plus an established optimization mechanism | Substantial compute and architecture commitments; less endogenous discovery | Useful reference baseline, not the first integration choice |
| Developmental growth and pruning | Creation and revision of connectivity | Larger design/search problem; difficult attribution | Later candidate if simpler mechanisms fail |
| Permanent dense neutral wiring alone | Makes every pair available to mutation | More parameters; no guaranteed coordination or learning | Insufficient evidence to choose as a default |

Recommendation: **extend Petri's existing research harness and plastic
compute-node mechanisms before adopting another simulator or learning
framework.** Polyworld is an architectural reference, while EmEvo and LNDP
are research implementations built around different computational assumptions.
Neither is a small drop-in solution to Petri's graph/VM representation problem.
No new dependency is justified by this review.

The local runtime currently applies learned weights to configured graph
compute-node inputs. Direct output-vote edges use their genomic weights.
Consequently, a learning experiment should use a genuinely plastic intermediate
circuit; drawing additional direct edges would not test the hypothesis.
Sources: [Hebbian implementation](../../crates/v3-core/src/runtime/plasticity/hebbian.rs),
[reward updates](../../crates/v3-core/src/runtime/plasticity/reward.rs),
[output effects](../../crates/v3-core/src/runtime/cgp/effects.rs).

## The smallest useful next experiment

This is a research comparison, not an implementation commitment.

1. **Reuse one small, connected sensor-group-to-motor-group circuit.** Compare
   independently evolved static weights, coordinated static mutations,
   evolved learning parameters with randomized starting weights, and the
   combination of evolved starting weights and learning. Begin with existing
   supported rules; a richer learned update function is a separate follow-up.
2. **Evaluate two demands.** First test all directions in a stable world.
   Then randomize sensorimotor alignment between lifetimes, or change food
   meaning, to test adaptation. The changing condition prevents a fixed
   correct mapping from answering the entire learning question.
3. **Measure actual behavior and learning cost.** Report food/energy gained,
   early versus late performance, time to competence, directional coverage,
   and computation. Test mixed scenes and sensor ablations, not only isolated
   one-hot food inputs. Preserve failed and extinct runs in outcomes.
4. **Use controls that identify the cause.** Disable plasticity in the same
   genotype; shuffle outcome feedback where applicable; reset learned state
   at birth; use held-out environments and independent evolutionary seeds.
   Match budgets and report parameter counts and wall time. Any inheritance
   of acquired weights must be a separately named condition.

Initialization and exploration need explicit treatment. Random weights,
spontaneous activity, or an existing active controller can break symmetry;
all-zero identical circuits often cannot. These mechanisms can disrupt
previous behavior, unlike a strictly neutral addition. A successful newborn
development scheme is not automatically safe for adding a sensor to an
already functioning adult circuit.

Do not bundle a ban on deletion into this comparison. Genetic deletion,
lifetime pruning, initialization, and the learning rule are independent
experimental choices.

## Confidence and remaining uncertainty

**Well supported:** evolution can discover learning rules; useful connection
weights can develop during life; structured encodings can exploit repeated
relationships; successful ALife systems use more than one approach.

**Plausible Petri inference:** combining a group projection with coordinated
changes or selected plasticity should be more promising than materializing
independent neutral edges alone. The local probe supports investigating
coordination but does not establish ecological benefit.

**Unresolved:** which mechanism survives Petri's actual energy, reproduction,
mutation, and compute constraints; whether it recruits new sensor groups;
whether learned directional behavior generalizes; and whether graph and VM
organisms receive comparable opportunities at comparable costs.

This is a targeted primary-source review with a 2024–2026 bias, not an
exhaustive systematic review. Older work is included where it is the closest
mechanistic precedent. Publication status and access limits are called out
above; reported successes from controlled tasks are not treated as proof of
self-sustaining ecological evolution. No direct controlled study was found
that settles Petri's exact neutral-wiring variants or justifies removing all
connection-deletion mutations.
