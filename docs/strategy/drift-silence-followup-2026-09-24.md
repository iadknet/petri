# Drift silence: coverage, early ecological genomes, and two-step follow-up

Date: 2026-09-24. Source: `df35975f805e11324b196535d8d391569270f07b`.
Exploratory experiments; no production or roadmap changes.
[Prior pilot](drift-silence-experiment-2026-09-24.md),
[research grounding](drift-silence-alife-research-2026-09-24.md),
[structured results](drift-silence-followup-2026-09-24.results.json),
[replay bundle](drift-silence-followup-2026-09-24.replay.tar.gz).

## What changed our understanding

**The deep-drift silence survives these additional probes.** Broader observation
found no action differences in 200 sampled silent pairs. A second, independent
proposal panel did reveal two changes missed by the original battery, so coverage
has real value, but these results do not support coverage as the main explanation.
Twenty sampled silent first steps did not improve the next sampled mutation
neighborhood on the directional task.

Early ecological genomes were much more responsive to mutation. However, the
planned longer population samples failed: both small populations went extinct.
Their earlier survivors were only generations 1–4, versus 2,000 unconditional
birth steps for the drift panel. This is descriptive evidence about different
starting genomes, not an isolated effect of selection or evidence of retained
cognition. None of these experiments compares T11 revisions, so none establishes
that those features caused harm.

## 1. Broader observation of silent mutations

Reuse the exact 20 depth-2,000 parents from the prior pilot. For each parent, retain
the first ten genuinely genome-changing, applied single-event mutations that are
silent on the original battery, within a fixed stream of 100 proposals. All 20
parents supplied ten pairs; 200 pairs were retained out of 1,974 eligible pairs.
This is a conditional sample of single-event silence, not all production births.

Each pair was evaluated on:

- 32 sensor contexts captured with native assemblers from the early ecological
  runs, including extended perception and previous action outcomes;
- 24 authored food, barrier, and nearby-creature contexts;
- four 32-tick input sequences cycling recorded contexts, with fresh initial state,
  native memory decay, and scenario energy reset each tick.

These are diagnostic executions. The sequences are not closed-loop ecological
trajectories; supplied previous outcomes do not come from the controller being
assessed. Authored contexts are not guaranteed realizable world snapshots.

| Observation | Pairs differing, out of 200 |
| --- | ---: |
| Actions on recorded contexts | 0 |
| Actions on authored contexts | 0 |
| Actions in first four sequence ticks | 0 |
| Actions in later sequence ticks | 0 |
| Work, energy, or shared-memory state | 27 |

All ten parents that were all-NoOp on the original battery remained so on the
extension. Authored positive controls passed: a barrier-dependent action and a
slow integrator threshold were silent on the original battery and detected by the
extension. A same-genome control matched exactly. The temporal control crosses its
threshold after the original four-tick horizon; longer sequences alone do not
identify memory as a cause in natural genomes.

Thus, action silence can hide state or cost changes. It is not evidence of total
functional equivalence, fitness neutrality, or absence of all future potential.
The zero action result applies to this sample and these contexts.

## 2. Ecological sampling and mutation supply

Two runs used the current Orchards recipe, 128×128 cells, 256 founders, seeds 11
and 22, and other native defaults. The predeclared endpoint was tick 1,500, with
sensor captures at 750 and 1,500. Both populations went extinct before sampling:

| Seed | Extinction tick | Total births |
| --- | ---: | ---: |
| 11 | 220 | 1,400 |
| 22 | 209 | 1,413 |

These failures remain part of the result. A documented amendment, frozen before
any mutation assays, reran the same seeds and configuration to tick 100, with
sensor captures at ticks 50 and 100. Twelve survivors per seed were sampled using
the existing uniform rank sampler. No seeds, costs, or food parameters were tuned.

At tick 100 the populations held 495 and 528 creatures, with median generation 2
and maximum generation 5. The 24 sampled genomes were generations 1–4; none was a
founder. They came from two populations on one fixed terrain, not 24 independent
selection experiments. The small world's spatial resource geometry also differs
from the goal worlds. The extinction observation cannot be extrapolated to those
larger worlds or attributed to any particular mechanism here.

The founder, 20 drift parents, and 24 early survivors each received 100 proposals
under three mutation-supply settings. The operator weights and executed preference
were unchanged. The exactly-one-request arm provides the simplest comparison:

| Cohort | Parents | Mean genome units | Proposals | Applied births | Changed | Silent | Dead |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Canonical founder | 1 | 97.00 | 100 | 59 | 30 | 29 | 0 |
| Early ecological survivors | 24 | 97.46 | 2,400 | 1,799 | 772 | 1,027 | 0 |
| Depth-2,000 drift parents | 20 | 1,380.05 | 2,000 | 2,000 | 13 | 1,986 | 1 |

Changed shares among applied births were 50.8%, 42.9%, and 0.65%, respectively.
The skipped requests remain in proposal denominators. All 24 early survivors
acted on both batteries; ten drift parents did not. Size, history, depth, and
survival conditioning differ greatly, so this is not a causal selection contrast.
“Changed” also does not mean improved.

Production birth supply scales with genome size, whereas the progress chart's
walk pins supply to the founder's 97 units. On the same drift parents:

| Supply arm | Proposals | Requested / applied events | Zero-applied births | Silent births | Changed births | Dead births |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Chart-style fixed 97 units | 2,000 | 967 / 967 | 1,234 | 764 | 2 | 0 |
| Own genome size, production rate | 2,000 | 13,676 / 13,676 | 8 | 1,919 | 62 | 11 |

More exposure produces more changes and more all-NoOp transitions. Even with
own-size supply, 1,919 of 1,992 event-bearing births remain silent. The experiment
does not justify raising mutation rates: it measures immediate variation, with
no discovery, retention, or ecological benefit test. Shared RNG seeds across
arms do not imply identical events after different draws. New streams also mean
these counts need not equal the prior pilot's checkpoint counts.

## 3. Does a silent step prepare useful second steps?

A second protocol was frozen after the preceding results and before generating
second-step proposals. For every drift parent, choose the first retained silent
child matching actions on the original battery, the extension, and the existing
48-scenario `steering-v1` seeking task. All 20 parents qualified with their trial-0
child. Every selected child genuinely differed genetically; three also differed
in work, energy, or shared-memory diagnostics.

The first-step edits spanned input swaps/prunes/raw-field changes, topology
swaps/splicing/gate changes, graph weights/copies/plasticity fields, and VM address
changes. They were not chosen for a known preparatory mechanism. Full edit records
and per-pair counts are in the structured results.

From each pair member, make 100 independent-copy proposals requesting exactly one
mutation, using fresh paired seeds and each member's own executed set: 2,000
proposals per arm. The task requires the first action to move toward one-hot food.
A directional improvement must preserve **every previously successful scene**,
add a success, and correctly respond to at least two food directions within one
base scenario. Merely acquiring a constant move does not meet that threshold.

| Outcome | Original-parent arm | Silent-child arm |
| --- | ---: | ---: |
| Requested / applied events | 2,000 / 2,000 | 2,000 / 2,000 |
| Applied but genome-identical proposals | 16 | 16 |
| Original battery: changed / dead / silent | 9 / 1 / 1,990 | 9 / 1 / 1,990 |
| Any action difference on extension | 11 | 11 |
| Any action difference on seeking task | 7 | 7 |
| Distinct nonbaseline combined signatures, summed per parent | 11 | 11 |
| Incumbent-preserving seeking gains | 0 | 0 |
| Qualifying directional improvements | 0 | 0 |

Both arms produced identical action signatures in all 2,000 paired trials. Their
mutation-event records differed in 49 trials, so this is not simply reuse of the
same event output. No child-only sampled signature occurred. The ten all-NoOp
parents produced no novel action signatures in either arm. Six parents initially
hit six seeking scenes each; the other fourteen hit none.

The extension detected two original-battery-silent mutations from drift parent 12
in this new stream, in both arms. One other original-battery change was absent on
the extension. These panels are complementary, and the earlier 0/200 result must
not be read as proof that the original battery misses nothing.

An authored positive control passed: a fixed east mover gains an unused food-ring
declaration silently, then a connection from that new input to a north move adds
six successes while preserving its original six. The connection requires the
previously missing declaration. This validates detection of a known preparatory
path; it does not estimate the probability of that path under random mutation.

The null result concerns these 20 first steps and 100 proposals per member. It
neither excludes rare or longer preparatory paths nor proves silent structure
useless. Diagnostic cost flags for child-arm proposals compare the combined path
to the original parent; they do not isolate the second mutation's cost effect.

## Roadmap implication

Retain the prior proposal for a small **T11 mutation-effect attribution and
observation coverage addition**, provisionally F26, after F25 and before T20's
baseline. Make parent action capability, mutation exposure, applied-but-identical
edits, and action-versus-state effects visible. Keep the original progress series
intact and name additional observation panels separately. This remains a proposal;
no priority order or feature checklist has changed.

These results do not support choosing a new mutation mechanism yet. F25 remains
the owner of inactive action-parameter targeting. A further focused diagnostic
could trace sampled applied changes through module execution, vote contributions,
and action thresholds to distinguish disconnected edits from masked effects.
That would narrow the repair decision more directly than indiscriminately widening
mutation or deleting silent structure.

Keep multistep neutral potential with T11.F12, and selected mutation-policy
comparisons with T11.F13. The failed long sample means this work does not fulfill
a sustained selected-population comparison. T20 should qualify input recruitment
through directional discovery, preservation, and ecological transfer; neither
arbitrary action changes nor this pilot's nulls establish increased cognition.

## Verification and replay

Native simulation, mutation, and execution code were used without modification.
The standalone harness is under `.bench-artifacts/drift-followup-2026-09-24/`.
Coverage controls and the two-step positive control pass. Because native seeking
per-scene outputs are private, the harness mirrors its generator, including RNG
draw order, for exactly two food types. Source inspection and aggregate native
move/exact-hit equivalence on all 45 parents plus three fixtures validate that
mirror; aggregate equality alone is not a proof of per-scene equivalence.

The analysis validates proposal partitions, exactly-one-request arms, per-scene
success preservation, 200 retained pairs, 20 second-step pairs, and all 4,000
second-step records. A read-only research-agent audit found no blocking flaw and
identified the combined-path cost interpretation noted above.

First-run wall times were 0.61 seconds for the two extinction runs, 0.48 seconds
for early sampling, 4.84 seconds for the first assays, and 22.98 seconds for the
second-step assay including output overhead. These are resource measurements,
not performance comparisons. The structured results record final-replay timings;
`first-run-timings.json` in the bundle preserves the first-run measurements above.
Offline incremental builds took roughly 3–6 seconds
each. All stages stayed within their frozen caps. A final replay matched SHA-256 hashes exactly for the configuration, parents,
contexts, retained pairs, all first-assay records, all compact second-step records,
and protocol. The sampled outcomes are not selected from repeated runs.

Extract the bundle under `.bench-artifacts/` at the repository root and run its
`reproduce.sh` there, using the recorded source revision. It contains source,
lockfile, frozen protocols/amendments, the 20 drift input genomes, sampled parents,
contexts and silent children, analysis, compact trial records, and verification
logs. The complete second-step raw output, including every candidate genome, is
retained locally as `second.ndjson.gz`; it is omitted from the compact bundle and
regenerated by replay. SHA-256 digests tie raw and compact records to the summary.
The standalone tests, exact replay comparison, and `make check-docs` passed.
No production defaults, baseline records, roadmap files, or commits were changed.
