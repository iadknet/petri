# Competent controllers and lifetime adaptation

Date: 2026-09-23. Source: `aef05f9077efffdaf620c077a98a1f377b29bc3e`.
Status: exploratory experiment, not a production feature or roadmap commitment.

Follow-up: [four experiments on shared learning and new sensors](ring-group-experiments-2026-09-23.md)
now test the proposed group-sharing mechanism, exceptions, recruitment, and costs.

## Decision

**Competent evolved starting weights plus action-specific learning improve
stable performance. Adaptation to an abrupt sensor remapping is still too slow
to survive normal costs.**

The next mechanism worth testing is **sharing a learned spatial relationship
across a sensor group**. This directly addresses the need to rediscover useful
behavior separately for each direction. Keep the current independent learner
as the comparison and test whether the improvement survives actual costs.
This is a hypothesis, not an established result from this panel.

The measured primary and follow-up panels took **7.81 seconds after compilation**.
Application source, defaults, and roadmap files were not changed.

## Primary results

Each cell represents 128 lifetimes: eight fresh episodes for each of 16 source
controllers. All deaths count. Survival means reaching 1,024 native ticks.

| Starting controller and lifetime rule | Stable: survivors | Stable: food collected | After remapping: survivors |
| --- | ---: | ---: | ---: |
| Competent evolved, learning off | 91/128 (71.1%) | 70.8% | 0/128 |
| Competent evolved, action-specific delta learning | **120/128 (93.8%)** | **80.3%** | 0/128 |
| Weak starting weights, learning off | 0/128 | 2.6% | 0/128 |
| Weak starting weights, action-specific delta learning | 19/128 (14.8%) | 24.3% | 0/128 |

Food collected is physical food obtained divided by the best food available
over the entire scheduled lifetime, including periods after death. It measures
both foraging and persistence, not just accuracy while alive.

For competent starts, learning improved stable survival by **22.7 percentage
points**, with a paired 95% bootstrap interval of **9.4 to 37.5 points**. Stable
food capture rose by **9.5 points** (interval **5.0 to 14.8**). These are paired
comparisons within the same starting controller, not comparisons of separately
evolved winners. The stable-preservation gate passed.

All 128 competent controllers in each treatment reached the remapping at tick
256. Mean stored energy was 167.3 with learning and 158.1 without, against a
maximum of 200. After the change, every one died before tick 512. Learning
extended mean life from **325.4 to 336.2 ticks**: a **10.8-tick** gain (interval
**8.0 to 13.5**), insufficient for viable adaptation. The adaptation gate failed.

The switchback was scheduled for tick 512. No creature reached it under normal
costs, so the primary panel cannot establish recovery or forgetting on return.

## Does more experience allow recovery?

A post-result diagnostic retained the selected weights and learning rates,
but supplied energy from the first remapping onward: energy reset to 50 before
each two-tick trial. Native charges still execute, but this external support
removes the survival constraint. The same episodes and food sequences are
used; no parameters are retuned.

| Mapping and controller | First 128 trials after change | Next 128 trials | Final 128 trials |
| --- | ---: | ---: | ---: |
| Remains changed, learning off | 6.3% | 5.9% | 5.6% |
| Remains changed, learning on | **27.0%** | **50.0%** | **60.6%** |
| Returns to original after first period, learning off | 6.3% | 77.9% | 77.7% |
| Returns to original after first period, learning on | 27.0% | 59.3% | **78.0%** |

Values are food capture fractions within each period. All 512 diagnostic
lifetimes reach the horizon because of the subsidy; that survival is not an
adaptation success. The learner's final 32-trial food capture reaches 64.8%
when the mapping stays changed, and 80.5% after return.

This supports **insufficient adaptation speed/opportunity** as a bottleneck.
It does not isolate whether the main cause is exploration, interference from
old weights, the learning-rate search, or learning each direction separately.
Recovery remains incomplete in the changed condition even after the subsidy.

Returning to the original mapping imposes a temporary relearning cost. These
maps are uncued and require different actions for identical input vectors;
the single memoryless weight matrix cannot represent both responses at once.
Immediate recall of both maps was not a realistic success criterion.

## Energy-feedback correction

The [previous learning report](ring-learning-experiment-2026-09-23.md) incorrectly
described the prototype's reward as energy retained after the ceiling.
Production `apply_typed_eat` actually records **gross nutritional intake** in
`food_intake_by_type`, with overflow recorded separately. Previous numerical
results remain valid; their cap-related explanation has been corrected.

This panel compares the original gross-food signal with a new retained-energy
signal: subtract the eating tick's maximum-energy clamp loss, then divide by
five. Neither includes metabolic, movement, or compute costs in its feedback.

Both feedback variants select the same rates, produce the same stable food
capture and survivor counts, and fail the remapping survival gate. Mean life
in the changed condition differs by less than 0.1 tick. This contrast does not
explain the main failure in this assay; it does not establish equivalence in
other energy regimes or tasks.

## Experimental design

The probe extends the existing native-action harness; it does not introduce
another benchmark framework. All 16 coordinated/stable controllers from the
previous experiment are reused, including weak performers. There is no
selection on prior survival and no further evolution of starting weights.
The weak-start control retains the earlier mostly-zero asymmetric matrix.

Every arm has an eight-channel food ring, eight Move votes, 64 independently
modifiable weights, eight sigmoid nodes, and a supplied eating reflex.
The lifetime learner updates only the chosen action's row using prediction
error against observed food intake. The formula is supplied, not evolved.
All arms receive identical birth jitter, food sequences and 20% exploration.

Each lifetime starts at 20 energy. Metabolism, genome carrying, age-adjusted
actions, execution costs, and the energy ceiling are native. The primary panel
never resets energy. Food and position are reset each trial to isolate the
sensor-to-action relationship; competition, reproduction and free navigation
are absent. Prototype learning bookkeeping itself is not charged energy.

The three regimes last up to 512 two-tick trials:

1. Stable mapping throughout.
2. Original mapping for 128 trials, changed mapping for the remaining 384.
3. Original for 128 trials, changed for 128, original for the final 256.

Changes rotate channel references while preserving learned weights, runtime
state and energy. They change the interpretation of existing inputs; **this
does not test adding a new sensor group, irrelevant inputs, or new dimensions**.

For each source seed, start type and feedback type, seven learning rates
(0, .003, .01, .03, .1, .3, 1) are compared on four training lifetimes: two stable,
one changed by two bins, and one changed by four bins then returned. Selection
maximizes ticks survived, breaking ties by food obtained, then smaller rate.
This gives **64 rate searches and 1,792 training lifetimes**. Eleven of the
16 competent learners choose rate 1; the others choose .03, .1, or .3.

The **2,304 primary holdout lifetimes** use eight new paired episode seeds per
arm and regime, with unseen odd rotations (1, 3, 5, 7). The additional diagnostic
uses **512 lifetimes**. Exact protocols, seeds, tuning scores and outcomes are
retained in the data artifact.

Intervals resample the 16 source-seed blocks, preserving within-seed pairing,
with 10,000 percentile bootstrap draws. They are exploratory, without adjustment
for multiple comparisons. Zero observed survival gives a degenerate bootstrap
interval; it is not proof the population success probability is exactly zero.

## Implications for a roadmap

The [research review](alife-sensorimotor-learning-research-2026-09-23.md) separates
inherited structure, useful initial behavior, and lifetime learning. The local
experiments now support that separation:

1. **Coordinated variation remains the best-supported starting point.** The
   [wiring experiment](ring-wiring-experiment-2026-09-23.md) and subsequent native
   assays support coordinated changes more strongly than dense wiring alone.
2. **Action-specific delta learning is a useful complement.** It improves
   competent controllers under stable conditions; weak starts still struggle.
   Plain correlation-based Hebbian updates and the existing native reward
   rule should not be treated as equivalent to this prototype.
3. **Faster group learning is the next bounded hypothesis.** Compare learning
   one shared relative-direction relationship across the ring against eight
   independent action rows, using the same experience and survival budget.
   Credit remains attached to the executed action; transfer evidence by rotating
   the input/action relationship together.
   Spatial symmetry is an additional inductive bias: test broken-symmetry
   controls so a result is not attributed to general learning when geometry
   was supplied. Keep independent weights available for exceptions.

Before specifying a production track, transfer the successful mechanism to an
assay where an existing sensor remains useful while a new sensor group appears.
Include irrelevant-input controls and an explicitly costed VM representation.
The current graph-only probe establishes neither ecological usefulness nor a
backend-neutral design. Existing ownership remains T11 for representation and
variation, T13 for recruitment, and T09 for learning qualification.

There is still no evidence here to require permanent all-to-all connectivity
or to remove edge-deletion mutations.

## Verification and reproduction

- Twelve targeted probe tests pass, including three property tests. They cover
  native eating at the energy ceiling, remapping in an already-running creature,
  preservation of weights under remapping, learning reset, and support timing.
- A complete seed block was replayed after the diagnostic was added: all
  **24 tuning/result rows match exactly**.
- Primary runtime: 5.904 seconds. Diagnostic runtime: 1.906 seconds. These are
  resource costs, not comparative performance measurements.
- Project documentation validation: `make check-docs` passed.

The results and complete source snapshot (local only; see the Evidence line
below) contain all primary and diagnostic observations, input controllers,
protocols, Cargo lockfile, analysis scripts, property-test regression seed, and hashes.
The working harness is under `.bench-artifacts/ring-adaptation-2026-09-23/`;
run `sh .bench-artifacts/ring-adaptation-2026-09-23/reproduce.sh` from the repository.
For reconstruction, restore each `source_snapshot` entry's `text` under that
directory, using the recorded repository revision, then run the same command.

Evidence (relocated 2026-09-24, local only): `.bench-artifacts/research/ring-adaptation-experiment-2026-09-23/ring-adaptation-experiment-2026-09-23.results.json`,
sha256 `6118edbb0f685f3cc7fe0702516da79c5ed0e3d92e92ad65b9ec158bd921dd2d`, 7,758,320 bytes.
