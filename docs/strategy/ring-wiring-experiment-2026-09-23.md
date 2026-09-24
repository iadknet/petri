# Ring wiring and learning: a bounded experiment

Date: 2026-09-23. Source: `aef05f9077efffdaf620c077a98a1f377b29bc3e`.
Status: exploratory research, not a feature spec or default-change decision.

## Findings

**Providing all connections did little by itself in this screen. Coordinated
mutations helped substantially. Hebbian learning did not automatically turn
the proposed silent connections into directional behavior.**

The current Move group has eight directional votes. A food ring connected to
the entire group therefore has 8 × 8 = 64 connections, not 32.

This is a deliberately cheap test of a local discovery problem: starting with
only a west-food-to-west-movement connection, can a controller develop useful
responses in the other directions? It uses Petri's actual graph/VM execution,
but a restricted mutation experiment and artificial task selection. It does
not measure reproduction, ecological adaptation, or the full mutation engine.

The six-arm panel ran in 5.51 seconds after compilation; the fresh-seed
mechanism control ran in 2.86 seconds. These are resource-accounting times,
not comparative runtime benchmarks. No production source, founder, default,
roadmap, or existing observation battery changed.

## Wiring comparison

Each arm has 32 independent seeded search runs, each with 2,048 proposals.
Every start has the same one working west connection, weight 0.25. Growth
occurs with probability 1/3; ordinary refinement uses the production
`AlterGraphEdgeWeight` operator with probability 2/3. Existing weights are
preserved when completing a bundle. A selected pair already present is not
duplicated; a growth proposal with nothing to add still consumes its budget.

| Wiring rule | New connections per growth event | Runs reaching all 8 directions | Mean accuracy on held-out mixed-food scenes |
| --- | --- | ---: | ---: |
| Single active edge | One randomly selected pair, weight drawn from [-1,1] | 0/32 | 21.2% |
| Single neutral edge | One randomly selected pair, weight zero | 0/32 | 21.5% |
| Whole input ring to one output | All missing channels to one randomly selected Move direction, zero weights | 0/32 | 22.5% |
| Whole input ring to whole Move group | All missing pairs in the 64-edge matrix, zero weights | 0/32 | 22.6% |
| Matched directions | Eight pairs: food N → Move N, etc., zero weights | 3/32 | 39.1% |

The sixth arm used the same full 64-edge matrix but replaced one quarter of
its refinement proposals with a coordinated update. It reached all eight
directions in **28/32 runs**, with **42.4%** held-out accuracy. Its median first
all-eight response was 726 proposals. Failures remain in latency summaries,
censored at proposal 2,049; the other five arms have a censored median.

A coordinated update picks one of eight cyclic input/output offsets uniformly
and adds the same signed amount to all eight pairs at that offset. For offset
zero these are N→N, NE→NE, etc.; other offsets connect rotated directions.
The rule does not choose the offset using the task's answer. Each edge receives
`U(-0.1,0.1)/sqrt(8)`, bounding the eight-coordinate Euclidean step by 0.1.
Most refinements remain independent production weight mutations, so weights
are not permanently tied.

**Interpretation:** creating the wiring together and changing its behavior
together are different mechanisms. This experiment favors testing a mutation
that can express a repeated directional relationship, not merely materializing
more independently tuned edges. The matched-direction arm has an explicit
spatial prior; its smaller search space is not a neutral comparison of storage.

The input-bundle versus single-neutral held-out difference was +1.01 percentage
points, paired bootstrap interval [-1.32, +3.26]. Full matrix versus input
bundle was +0.10 points [-2.39, +2.59]. These small panels do not establish
equivalence, but provide no clear improvement from broader neutral wiring.
Zero successes in 32 trials is not proof of impossibility; its Wilson 95%
upper bound is about 10.7% under this assay.

## Follow-up: distinguish coordination from the weight-update rule

The first result contained an important confound. Production weight mutation
multiplies by a factor in [0.8,1.2] when the magnitude exceeds 0.01, preserving
the sign in that branch. Near zero it adds a step in [-0.1,0.1]. The coordinated
arm instead sometimes adds a signed amount regardless of current magnitude.
A wrong initial sign can therefore persist under ordinary refinement; it can
change once a weight returns to the near-zero branch, or through other
operators that this local screen excludes.

Before running additional outcomes, a separate control was specified: the same
full matrix and the same 25% replacement probability, but add `U(-0.1,0.1)` to
one uniformly selected weight. This matches additive-event frequency and the
maximum vector step magnitude. It does not match the number of coefficients
touched. All three treatments were rerun with 32 fresh seeds each.

| Full-matrix refinement | All-eight runs | Held-out accuracy |
| --- | ---: | ---: |
| Ordinary production scalar steps | 0/32 | 23.0% |
| Occasional additive single-weight steps | 1/32 | 25.4% |
| Occasional coordinated steps | 27/32 | 39.0% |

The coordinated-minus-additive difference was +13.59 percentage points,
paired bootstrap interval [+9.96, +17.24]. This supports a contribution from
coordinated changes beyond simply permitting additive steps, within this
particular representation, task, horizon, and proposal budget. It does not
show that coordinated mutation is universally superior or optimal.

Even the coordinated arm makes the wrong choice on most held-out mixed rings.
An all-eight response to isolated food is not robust general food seeking.
The 25-case selection set is small and creates broad fitness plateaus; neither
training success nor this held-out result qualifies ecological deployment.

## Could Hebbian learning supply the weights instead?

Yes in principle: evolution can supply a learner rather than separately tune
every final weight. Three current implementation details prevent that from
happening automatically for these proposed connections:

1. **Direct vote edges are not plastic.** Learned weights and plasticity
   configuration belong to graph compute-node inputs. The output-effect pass
   reads the genome weights of vote edges directly. VM constants do not have
   automatic Hebbian updates either.
2. **Classic learning needs receiving-node activity.** Petri's update is
   `eta × pre × post`. With all-zero weights into an unbiased WeightedSum,
   post remains zero, so Classic learning cannot bootstrap. Oja and AntiHebb
   also contain a post factor. Covariance has a different zero-start behavior;
   silence is not an absorbing condition for every supported rule.
3. **Correlation is not behavioral credit.** Learning needs a mechanism that
   distinguishes helpful action relationships. Identical output circuits with
   identical activity can learn identical weights and remain directionally
   indistinguishable. A preexisting active west pathway may just strengthen.

Sources: [Hebbian runtime](../../crates/v3-core/src/runtime/plasticity/hebbian.rs),
[output effects](../../crates/v3-core/src/runtime/cgp/effects.rs), and
[reward updates](../../crates/v3-core/src/runtime/plasticity/reward.rs).

A small mechanism check used five authored circuits, without mutations or
reward, for 128 ticks of equally frequent one-hot food in all directions.
Learned state persisted; energy was reset to 50 each tick. For compute-node
conditions, eight nodes each received all eight food channels and drove one
Move vote through a fixed unit-weight edge. Learned weights were then frozen
for endpoint evaluation.

| Circuit | Actual weight-changing assignments | Correct isolated-food directions after exposure |
| --- | ---: | ---: |
| Direct zero-weight vote edges | 0; no plastic updates scheduled | 0/8 |
| Zero-weight WeightedSum nodes, Classic | 0, despite 8,192 update assignments | 0/8 |
| Same, with one initial west weight | 16 | 1/8, west |
| Zero-weight Sigmoid nodes, Classic | 1,024 | 1/8, north |
| Zero-weight WeightedSum nodes, Covariance | 8,192 | 1/8, north |

In the last two circuits, all eight learned rows remained identical. North is
the runtime's lowest-index tie winner, not a learned north preference. These
are bootstrap/symmetry demonstrations, not a comparison of optimized learning
architectures. They do not show that Hebbian learning cannot solve the task.

Petri already has reward-modulated compute-node plasticity and eligibility
traces. A meaningful next learning comparison would hold the plastic circuit
architecture constant and compare mutation-only, learning-only, and both,
including exploration, actual outcome signals and costs. It must retain
within-life state; the wiring screen's fresh-state evaluations cannot answer
that question. Extending plasticity to output sinks is a separate runtime
change, not an existing feature of the wiring proposals.

## What the costs show

The fixed proposal budget isolates local accessibility; it does not give larger
genomes the extra birth mutations that production would supply. At the current
per-unit rate 0.005, a complete direct graph matrix has genome size 74 and an
expected 0.37 requested events per birth. The matched graph has size 18 and
0.09 expected events. These small authored genomes are not production founders.

The single-edge and one-output-bundle arms also accumulate almost all 64 edges
by the endpoint because this screen accepts neutral additions and excludes
deletion and size selection. Thus it compares their discovery paths; it does
not demonstrate that single-edge growth must produce a dense ecological genome.

Every final graph was also translated into an equivalent VM instruction
sequence, preserving even zero-weight edges. All held-out first actions matched
between backends. The dense VM executed 201 instructions per decision versus
33 for the matched-direction VM. Its measured compute debit was approximately
0.00517 energy versus 0.00000381. The dense program crosses the default VM step
ramp allowance. The graph debit was approximately 0.00001144 in both layouts;
the current graph charge does not price each output edge separately. These
values exclude action, carrying, reproduction, and lifetime costs. The
translation is one straightforward encoding, not an optimal VM compiler and
not a test of evolution discovering those instruction sequences.

## Research grounding and recommendation

Primary research inspected during this discussion:

- [Atkinson, Plump and Stepney, semantic neutral drift](https://www-users.york.ac.uk/~ss44/bib/ss/nonstd/naco19-snd.htm)
  supports testing behavior-preserving changes that alter future mutation
  opportunities. It does not establish an advantage for this exact bundle.
- [Clune et al., coordinated gaits with HyperNEAT](https://www.cse.msu.edu/~ofria/pubs/2009CluneEtAl.pdf)
  supports considering encodings that exploit repeated spatial relationships.
  The coordinated arm here is a small explicit mutation prior, not HyperNEAT.
- [Frémaux and Gerstner, three-factor learning rules](https://www.frontiersin.org/journals/neural-circuits/articles/10.3389/fncir.2015.00085/full)
  explains why outcome modulation and eligibility matter for action learning
  beyond local coactivation. Petri's existing reward rule is not thereby
  guaranteed to solve directional credit assignment.

The options considered were single-edge growth, neutral sensor bundles, full
group wiring, matched-direction wiring, coordinated updates, and learning.
The material criteria were directional acquisition, transfer to mixed inputs,
preservation of the starting competence, mutation accessibility, and costs.

**Recommendation:** retain the existing graph/VM runtime and observation
machinery. Do not adopt permanent all-to-all wiring from these results. The
stronger follow-up hypotheses are mutations that reuse directional structure
and circuits that can learn useful directional distinctions. Qualify them on
richer fixed tasks and under real lifetime costs before any ecology or default
claim. T11 owns representation/mutation semantics; T13 supplies recruitment
evaluation. No new roadmap or external framework is needed for this screen.

## Reproduction and limits

The [compact result record](ring-wiring-experiment-2026-09-23.results.json)
contains every replicate's scalar measurements, configurations, intervals,
follow-up results, plasticity readings, and source/raw hashes. Full genomes,
the frozen primary protocol, the separately declared follow-up, probe source,
lockfile and analysis scripts are in the ignored local archive
`.bench-artifacts/ring-wiring-2026-09-23/`. The archive must be copied separately
when sharing reproduction material. Run its POSIX `reproduce.sh` from this
checkout to rebuild and write fresh results into a temporary directory.

Training uses eight one-hot scenes, eight adjacent-distractor scenes, eight
opposite-distractor scenes, and an empty ring. Selection accepts nondecreasing
correct-first-action counts with west competence retained, including ties.
Held-out evaluation uses 256 fixed random mixed rings that never influence
selection. Every run starts from fresh controller state for each scene and
uses a one-action cap; remaining runtime settings are production defaults.
Existing `steering-v1` is read only at the endpoint and is not used as fitness.

This is conditional on already possessing a food reference and an executed
module. It omits node recruitment, competing action kinds, hidden computation,
deletion, input changes, the full operator distribution, per-birth supply,
population drift, reproduction and natural selection. Equal seed labels do
not synchronize semantic edits across representations. Replicate-level
bootstrap intervals are conditional on this task suite and do not include
uncertainty from other environments; exploratory contrasts are not corrected
for multiple comparisons. The follow-up was adaptively motivated, explicitly
separate, and used fresh mutation seeds but the same scenes.

Verification: four probe tests passed, including property tests for neutral
growth and graph/VM parity, and an authored identity controller solving the
training and mixed-food scenes. Every sampled endpoint also asserted graph/VM
parity. A complete reproduction matched all 288 search endpoints and their
scalar measurements, plus the plasticity readings. `make check-docs` and POSIX
shell validation passed. No application/runtime source or build configuration
changed.
