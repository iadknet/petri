# Ring coordination, lifetime learning, and survival

Date: 2026-09-23. Source: `aef05f9077efffdaf620c077a98a1f377b29bc3e`.
Status: exploratory evidence for roadmap planning, not a feature specification
or a production-default decision.

## Recommendation

**Prioritize coordinated variation for useful starting behavior. Investigate
action-specific, error-correcting learning as a second mechanism.**

Three cheap experiments distinguish these directions:

1. Coordinated weight changes substantially improve discovery of a stable
   directional mapping over independent scalar changes.
2. A supplied learner that credits the action taken and corrects its payoff
   prediction adapts to changed sensor mappings. Simple group-shared settings
   for Petri's existing plasticity rules deliver much smaller benefits here.
3. Restoring normal survival costs sharply limits the learner's success.
   Learning an effective mapping with subsidized experience does not mean
   a newborn can survive long enough to learn it.

The strongest next hypothesis is **competent starting behavior plus adaptive
action credit**. That combination was not tested: the hybrid below uses the
existing native reward rule, not the new action-credit prototype. It needs
one bounded comparison before defining an implementation track around it.

Follow-up: that comparison is now recorded in
[Competent controllers and lifetime adaptation](ring-adaptation-experiment-2026-09-23.md).

The three measured panels took **55.75 seconds after compilation**. This is
resource accounting, not a comparative runtime benchmark. Production source,
founders, defaults, and roadmaps were not modified.

## What was measured

The isolated Rust harness calls Petri's native `Simulation` and `run_tick`.
Movement, eating, energy accounting, native Hebbian updates, and native reward
updates execute through production code. Every controller has eight sigmoid
compute nodes, a complete 64-weight food-ring projection to eight Move votes,
and a supplied FoodHere-to-Eat reflex. All have the same architecture.

Starting weights are zero except west-to-west = 0.25, with small independent
birth jitter in every treatment. The jitter breaks symmetry and means this
is not another zero-initialization test. Learner-only treatments cannot evolve
their starting matrix. Acquired weights reset between lifetimes.

A trial places food around a recentered creature. The first tick lets it move;
other food is removed, and the second tick lets it eat what it reached.
Training and experience alternate isolated food with a lower-valued distractor.
Directions are balanced in shuffled blocks. All treatments receive the same
20% exploratory movement intervention through shared-memory contributions to
the movement votes. The native plastic nodes do not receive a special
chosen-action signal.

There are two environments:

- **Stable:** food directions retain their usual motor interpretation.
- **Rotated between lifetimes:** sensor-channel indexing rotates, while physical
  movement stays unchanged. Training includes offsets 0, 2, 4, and 6; testing
  includes all eight, including the four unseen odd offsets.

The first two panels reset energy to 50 each trial while retaining learned
state. Metabolism and genome carrying are off; action, execution, and
age-adjusted costs remain native. Food growth, reproduction, and competition
are absent. The third panel removes the energy subsidy.

The main food metric is the fraction of the best available neighboring food
collected, averaged over trials. It is not a percentage of successful actions.
Endpoint accuracy freezes learning, removes exploratory movement, and asks
for the richest direction in 128 new scenes with food in all eight channels.
Separate one-hot tests measure coverage of all eight directions.

## Experiment 1: evolve weights, learning settings, or both

Each of six treatments ran 16 independent search seeds in each environment:
**192 bounded searches**. Each search evaluates its initial candidate and 128
proposals on four 64-trial lifetimes, accepting improvements and neutral
proposals. This is a restricted hill climb, not the production mutation engine
or population selection. Eight fresh 128-trial lifetimes evaluate each result.

Scalar mutation adds a signed step to one weight. Coordinated mutation replaces
25% of those steps with one cyclic-offset update across eight weights, using
the same maximum Euclidean step size. Edges can still vary independently.

Native learners select one group-shared rule from Classic, Oja, AntiHebb, and
Covariance, a learning rate, and, when applicable, trace retention. Reward
learners use native EnergyDelta. The hybrid divides its proposals between
coordinated weight mutation and these reward-learning settings; it does not
receive an additional search budget.

| Treatment | Stable: food collected | Rotated: food collected | Rotated: endpoint mixed accuracy |
| --- | ---: | ---: | ---: |
| Independent scalar weight mutation | 56.8% | 14.9% | 12.6% |
| Coordinated weight mutation | **77.1%** | 15.1% | 12.2% |
| Evolved native Hebbian settings, fixed birth matrix | 25.2% | 14.4% | 12.5% |
| Evolved native reward settings, fixed birth matrix | 23.9% | 15.8% | 12.8% |
| Coordinated weights plus native reward settings | 68.1% | 15.7% | 12.6% |
| Action-credit delta prototype, tuned rate, fixed birth matrix | 45.8% | **44.0%** | **54.7%** |

Coordinated versus scalar mutation gains **20.3 percentage points** in stable
food collection; the paired bootstrap interval across search seeds is
**[14.8, 25.9]**. Stable one-hot coverage averages 7.68/8 versus 5.38/8.
However, coordinated controllers reach only 29.3% mixed-scene endpoint
accuracy. Successful isolated directions and good performance on sparse food
scenes do not imply robust behavior on arbitrary food mixtures.

The native reward learner has a small rotated benefit over disabling its own
learning: **+1.8 points [0.4, 3.4]**. This is much smaller than the prototype's
**+30.0 points [26.0, 33.0]**. Native hybrid learning contributes approximately
zero to its stable food score compared with disabling learning in the same
genotype: **+0.02 points [-2.3, 1.7]**.

The prototype is hand-authored. It updates the selected action's row toward
the actual food payoff, rather than broadcasting a common reinforcement
update across every row. It receives no correct-direction label. Evolution
only selects its rate; this does not demonstrate evolution discovering its
update formula, circuit, or action-credit mechanism.

## Experiment 2: distinguish action credit from the formula

The first comparison changed several things at once. A predeclared follow-up
crosses two choices under otherwise matching prototype machinery:

- Credit the **chosen row** or **every row**.
- Use **Hebbian reinforcement** or **prediction-error correction**.

All four receive the same native food amount once after the two-tick trial.
Hebbian updates use first-tick pre/post activity. Delta updates use the
difference between food obtained and the row's linear payoff prediction.
Thus this comparison isolates credit scope and formula within the prototype;
its feedback timing and normalization still differ from native reward traces.

There are 16 fresh search seeds per method/environment, with seven candidate
learning rates selected only on four 64-trial training lifetimes. The same
selected settings are evaluated at 128 and 512 trials. These 128 tuning runs
have equal budgets within this panel, but not the first panel's budget.

| Prototype, rotated environment, 512 trials | Food collected over life | Food collected in final quarter | Endpoint mixed accuracy | Direction coverage |
| --- | ---: | ---: | ---: | ---: |
| Hebbian, every row | 13.7% | 12.7% | 12.4% | 0.72/8 |
| Hebbian, chosen row | 37.0% | 39.6% | 23.4% | 3.48/8 |
| Delta, every row | 15.3% | 15.1% | 12.6% | 1.02/8 |
| Delta, chosen row | **68.3%** | **80.0%** | **84.5%** | **7.74/8** |

Chosen-row Hebbian reinforcement gains **23.3 points [21.8, 24.9]** over
every-row reinforcement. Chosen-row delta correction gains another
**31.3 points [30.2, 32.5]** over chosen-row Hebbian reinforcement.
Both parts matter in this task.

The best prototype reaches all eight one-hot directions in 75% of the
128 held-out lifetimes. Unseen-rotation food collection is 68.2%, compared
with 68.3% for rotations seen during training.

Controls support genuine adaptation in this supplied circuit:

- Disabling learning drops lifetime food collection from 68.3% to 14.7%.
- Shuffling actually obtained rewards from the paired ordinary rollout drops
  it to 15.4%; reward occurrence alone is insufficient.
- Zeroing the sensor snapshot at frozen endpoint evaluation drops mixed
  accuracy from 84.5% to 12.9%.
- Removing explicit exploratory moves drops endpoint accuracy to 57.5% and
  coverage to 4.90/8. Exploration is useful over the longer horizon, although
  its short-term cost can outweigh its benefit in 128-trial evaluations.

These results concern a known projection with one row per action. General
credit assignment through a recurrent mesh, multiple modules, or VM code is
not established. Nor was preservation of earlier learning after a within-life
task switch measured.

## Experiment 3: can the controllers survive while learning?

This follow-up uses all selected genotypes from the named treatments, with
new scene seeds and **no retuning**. It restores production metabolism and
genome carrying and removes per-trial energy resets. Native action aging and
the energy ceiling remain active. Starting energy is either the production
founder value, 20, or an explicitly more generous 50.

The task still recenters creatures and supplies nearby food; it is not an
unrestricted world. Survival means reaching **1,024 ticks / 512 trials**.
Every row below contains 128 lifetimes clustered within 16 selected genotypes.
Primary and prototype panels use different seed cohorts, so comparisons
between those panels are descriptive, not paired.

| Treatment, starting energy 20 | Stable survivors | Rotated survivors |
| --- | ---: | ---: |
| Scalar mutation | 1/128 | 0/128 |
| Coordinated mutation | **83/128** | 0/128 |
| Native reward learner | 0/128 | 0/128 |
| Coordinated weights plus native reward learning | 29/128 | 0/128 |
| Chosen-row Hebbian prototype | 0/128 | 0/128 |
| Chosen-row delta prototype | 14/128 | **5/128** |

The delta learner still helps in the rotated condition: mean life rises from
56.5 ticks with learning disabled to 216.6 with learning. But only 3.9% reach
the horizon. Starting with 50 energy raises that to 10/128, or 7.8%; additional
energy alone does not close the gap.

Native hybrid learning can be harmful over a longer life: in the stable,
20-energy condition, **46/128** survive with its learning disabled, versus
**29/128** with it enabled. Short training-horizon performance did not expose
the full cost of continued plasticity.

This identifies a viability problem, not exclusively an initial learning-speed
problem. Longer exposure, increasing action costs, and persistent exploration
can also erode maintained performance. These factors need separate controls
before attributing all deaths to slow learning.

Correction, 2026-09-23: the reference's reward is **gross food energy before
the energy ceiling**, divided by five. `apply_typed_eat` records this intake
separately from maximum-energy clamp loss. The original explanation, and the
archived survival protocol, incorrectly said the ceiling reduced this learner's
reinforcement. The executed code and numerical results are unchanged. The
adaptation follow-up tests retained energy as a separate new condition.

## Interpretation for a future track

There is enough evidence to select research priorities, not enough to lock
in a production learning design:

1. **First candidate: coordinated sensor-group variation.** It has support
   from both the earlier wiring screen and this richer native-action assay,
   including the survival follow-up. Group construction alone was not varied
   here: every arm already had all 64 connections. This result favors how
   weights can change together, not a permanent all-to-all topology.
2. **Second candidate: action-specific learning with error correction.** The
   controlled factorial supports this mechanism. Compare a competent evolved
   starting controller with and without this learner under real costs before
   specifying production support. The successful formula was supplied, not
   discovered; richer evolved rules remain an alternative.
3. **Required next gate: preserve competence while adapting.** Test the new
   combination against coordinated mutation alone and the learner alone;
   use stable, changed, and irrelevant inputs, and separate initial learning
   from later forgetting or deterioration. Include a bounded VM representation
   and its actual cost before claiming a backend-neutral solution.

A potential track could focus on **sensorimotor coordination and adaptation**.
Its scope needs an explicit boundary with existing ownership: T11 owns
representation and variation; T13 owns recruitment; T09 owns learning
qualification. These results can inform that design without creating duplicate
implementations or parallel benchmark machinery. No track or feature IDs are
assigned by this report.

Do not infer that all sensors should be connected permanently, that deletion
should be disabled, that generic Hebbian learning is incapable in other
architectures, or that these controllers have evolved ecological cognition.
This assay supplies the projection, eating reflex, exploration access,
short feedback delay, and repeated nearby food opportunities.

## Costs, evidence, and reproduction

The primary, factorial, and survival panels ran in 34.50, 18.28, and 2.97
seconds respectively, after compilation. A disjoint timing pilot took 1.01
seconds. Neither measured panel hit its wall cap.

Native plastic treatments perform 128 update assignments per two-tick trial;
chosen-row prototypes perform eight. This does not establish an eight- or
sixteen-fold runtime advantage: their integration and bookkeeping differ.
Current default learning debits are zero. The prototype's extra arithmetic is
not billed by native runtime telemetry, so its reported native compute debit
is incomplete as a production cost estimate.

Seven harness tests passed, including two property tests. They check authored
solutions through actual movement/eating, channel rotation, mutation bounds,
native learning and birth reset, the factorial distinction, and survival with
restored costs. Replaying one complete paired seed block reproduced all
12 search rows exactly, including held-out controls. Documentation validation
uses `make check-docs`.

The [machine-readable evidence](ring-learning-experiment-2026-09-23.results.json)
contains seed-level summaries, paired bootstrap intervals, selected parameters,
raw artifact hashes, and a portable source snapshot with the lockfile and
predeclared protocols. Intervals resample independent search seeds, not the
correlated trials within a life. They describe exploratory comparisons and
are not multiplicity-adjusted confirmatory tests.

Full local artifacts are under
`.bench-artifacts/ring-learning-2026-09-23/`. At the recorded source commit,
run `sh .bench-artifacts/ring-learning-2026-09-23/reproduce.sh` to rebuild
offline, check the harness, and reproduce all three panels into a new temporary
directory. The JSON source snapshot preserves the harness if the ignored
local artifact directory is later removed.

Research context: [ALife literature review](alife-sensorimotor-learning-research-2026-09-23.md)
and [earlier ring-wiring experiment](ring-wiring-experiment-2026-09-23.md).
