# Capability assays: a small, fast evolvability lab

Date: 2026-09-28. Status: research and planning note; no feature executed and
no production source, default or founder changed. It adds track T22, places
T22.F01–F03 at the head of the priority order, and adds T22.F01 as a
dependency of T20.F09 and T11.F10. Depth: targeted review of abstracts,
documentation pages and one prior full text already in the repository, plus
one local timing measurement; not a systematic review and not a measured
counterfactual. Reviewed adversarially by Codex Astra `high` on 2026-09-28;
the findings and their dispositions are recorded at the end.

## Decision in brief

**Build a thin, Petri-native lab crate (`crates/v3-lab`) that evolves small
populations of real Petri genomes on fixed, controlled arenas with the
production mutation engine and lab selection, scores one capability at a
time, and finishes in seconds to minutes.** Do not adopt a genetic-algorithm
framework, do not extend the whole-world benchmark, and do not put the lab
inside `v3-core`. The lab measures whether native variation can reach a
capability; it never becomes a production mechanism, objective or gate.

This implements a decision the master roadmap already records: "bounded
fixed environments as assays and endogenous feedback as the long-term
pressure" ([roadmap Notes](../roadmap.md#notes-for-ai-agents)). The lab is
the assay half of that sentence, made cheap enough to use every day.

## Problem, constraints and decision criteria

The program's evidence instrument is the goal profile: three 1600² worlds,
10,000 founders, 2,000 ticks, about eleven minutes per closure, read once per
feature. It answers "did the world change" and cannot answer "can creatures
evolve X": any single capability is confounded by ecology, and a null on the
goal profile does not separate "unreachable by variation" from "no
opportunity", "no selection", or "not enough exposure".

Constraints, all from existing project rules:

- The living world stays natural and open-ended. Nothing task-specific,
  no curriculum, no reward and no authored solution enters production
  ([natural-analog rule](../roadmap.md#notes-for-ai-agents); "no feature may
  optimize against an indicator").
- Determinism where assertions depend on it; POSIX `sh`; no new dependency
  merely to avoid a small stable piece of code (`AGENTS.md`, `CONTRIBUTING`).
- Verification proportional to the change; long runs stay outside
  `make check` (T10 notes).

Criteria material to the decision: fit with `v3-core`'s genome, mutation
engine and tick loop; wall time per experiment; separation from production
code; fidelity to native variation; maintenance and licence of anything
adopted; and the risk of repeating the T20.F09 failure mode.

## What exists locally

| Harness | What it does | Cost | Why it does not serve as the lab |
| --- | --- | --- | --- |
| `neighborhood::battery` (T11.F01) | 48 single-tick snapshots + 8 four-tick sequences; a genome signature | seconds | Reactive signature, no world, no selection, no lifetime |
| `neighborhood::steering` | Does a genome vote toward food on fixed scenes | seconds | Same: one decision, no lifetime |
| `neighborhood::recruitment_paths` (T13.F02/F06/F07) | 12×12 world, eight one-tick scenes, native-mutator lineage search with score-first selection | minutes | One-tick scoring, single-lineage hill climb, records tied to module recruitment, inside `v3-core` |
| `neighborhood::opportunity` (T20.F01) | Eight authored arms compete in the three goal worlds, 1,000 ticks, mutation off | hours | Reads opportunity, not evolvability; whole-world cost |
| Reverted `neighborhood::input_discovery` (T20.F09) | 720 lineages × 256 generations × 2 siblings, fresh one-tick scenes, native mutation | 41 minutes plus a 318 s release build | 0/32 discoveries in every arm including the baseline; see below |
| `.bench-artifacts/ring-*` probes (2026-09-23) | External crates depending on `v3-core`; authored 8×8 controllers; restricted mutation; JSON + Python analysis | 5–27 seconds | Authored controllers and a hand-rolled mutation rule, not Petri genomes under the native engine; scratch, not reusable |

The ring probes are the proof that the shape works: an external crate can
build a `WorldState`, place `CreatureState`s with arbitrary genomes, run
`run_tick`, and read energy, counters and node outputs through the public
API, in seconds. Every public seam the lab needs already exists:
`WorldState::{new, set_food, set_food_type, set_barrier, place_creature}`,
`CreatureState::new`, `Simulation::new` (which takes the starting world tick),
`run_tick`, `MutationEngine::apply_mutations_with_food_type_count`,
`ParentExecuted::Record`, `mesh_reachable_nodes`, `functional_complexity`,
`creature_sensor_census`, `SteeringBattery` and `Battery`. Reproduction is
suppressed by configuration alone, with test precedent
(`crates/v3-core/src/simulation/actions/reproduction.rs:578`:
`min_reproduce_energy = max_energy + 1.0`). The gate rejects before the
reproduce cost is charged (T16.F01), but `execute_reproduce` in
`crates/v3-core/src/simulation/tick.rs` then debits the failed-action penalty
like any rejected action, and that penalty follows the startup ramp whose
default target is world tick 62,680. The lab keeps the penalty, starts every
evaluation at world tick 62,680 so it runs at the production endpoint, and
records the charge; no core change is needed.

### What T20.F09 showed

The reverted reading (`git show 1c7ba514:docs/progress/readings/t20-f09.md`)
records separate verdicts of **inconclusive** for scalar access, vector access
and ring coordination: 0/32 discovery and 0/16 validation in every candidate
arm *and* in the baseline arm, with a conservative contrast interval of
±0.107. Across all arms 424,983 native events were applied over 720 lineages,
about 590 per lineage, and only 106 of them touched the focal input family.
Scoring was one tick per scene with a binary correct/incorrect per scene.

That is not evidence that T20 is ineffective. The reading supports a
hypothesis, not a verdict: a binary one-tick score gave selection little to
climb between "no use of the input" and "correct use", and exposure was
thin, with 106 of roughly 590 events per lineage touching the focal family and
no proposal holding an eligible focal structured group. Gradient,
exposure and applicability are separate explanations, and the reading cannot
rank them. The lab therefore records exposure and applicability counts and a
bounded native-mutant score-distribution check before it interprets any null.
The same design also produced a 534 MB raw file before its recording was
repaired, and the closure retired production candidates on an inconclusive
result. The lab must avoid all three faults: graded, many-tick scores; a
calibration gate before any campaign; and a hard byte cap on outputs from day
one.

### Timing evidence

Measured on 2026-09-28 with the prebuilt release CLI from 2026-09-19 (a stale
binary; approximate), production defaults with only world size and founder
count overridden:

| Arena | Founders | Ticks | Wall |
| --- | ---: | ---: | ---: |
| 64×64 | 100 | 1,000 | 0.33 s |
| 128×128 | 400 | 1,000 | 0.60 s |

The committed gate summary of the last real closure, T20.F05
(`docs/progress/features/t20-f05-structured-heritable-refinement.json`),
records 0.00185250333588498 ms per creature-tick on a populated gate world.
Extrapolating it to solo evaluations in freshly built arenas is unverified:
food ecology still traverses the grid each tick with growth off, controls and
calibration add work, and eight-thread scaling is assumed. On that basis a v1
campaign of 8 replicates × 100 generations × 64 individuals × 4 scenes × 400
ticks (82 million creature-ticks) projects to about 150 s single-threaded, and
a quick-iteration run (4 × 50 × 32 × 4 × 300) to about 15 s. Both are
projections; F01 runs an end-to-end solo-evaluation pilot before fixing the
quick-run sizes. The table above was measured with `target/release/v3-cli`
built on 2026-09-19 12:28 from an unrecorded commit.

## External options considered

Maintenance facts read from the crates.io API on 2026-09-28.

| Option | Facts | Fit | Decision |
| --- | --- | --- | --- |
| [`radiate`](https://github.com/pkalivas/radiate) 1.3.2 | Released 2026-09-27; MIT; public `Gene`, `Chromosome`, `Codec`, `Mutate`, `Crossover`, `Select` traits; eleven built-in selectors including tournament, elite, NSGA-II ([docs.rs](https://docs.rs/radiate/latest/radiate/)) | Maintained and extensible, but Petri's only legitimate variation operator is `MutationEngine`; wrapping a whole `CreatureGenome` as one gene buys a selection loop Petri needs about a hundred lines to write, at the price of a framework's population model and crossover/selection semantics that do not exist in Petri | Reject for v1; revisit only if multi-objective or quality-diversity selection becomes a requirement |
| [`genetic_algorithm`](https://github.com/basvanwesting/genetic-algorithm) 0.27.4 | Released 2026-09-26; MIT/Apache-2.0; genotypes are the crate's own Binary/List/Range/Unique/MultiRange vectors ([docs.rs](https://docs.rs/genetic_algorithm/latest/genetic_algorithm/genotype/index.html)) | No custom genotype; Petri's genome is a node mesh with typed backends | Reject |
| [`genevo`](https://github.com/innoave/genevo) 0.7.1 | Last release 2022-03-13; MIT/Apache-2.0 | Unmaintained for four years | Reject |
| [`oxigen`](https://github.com/Martin1887/oxigen) 2.2.2 | Last release 2021-02-28; MPL-2.0 | Unmaintained; copyleft | Reject |
| MABE2, Avida | Reviewed 2026-09-03 as platforms; rejected because neither hosts Petri's evolvable topology ([roadmap Notes](../roadmap.md#notes-for-ai-agents)) | Their *method* is the precedent: evaluate genomes in a controlled environment separate from the living population | Borrow the method, not the platform |
| Python quality-diversity tooling (pyribs, QDax) | Different language and runtime | The analysis layer can stay Python-optional, as the ring probes did; the evaluation must be Rust to run real Petri genomes | Not needed for v1 |
| Extend `recruitment_paths` or `input_discovery` | In `v3-core`; one-tick scoring; feature-specific records | Would inherit the failure mode and keep assay code inside production | Reject |
| Extend `v3-cli bench` | Whole-world profiles | Wrong instrument for capability questions | Reject |
| **Build `crates/v3-lab`** | Depends on `v3-core` only; ~1–2 k lines: arena builders, evaluation loop, selection, NDJSON writer, controls | Runs real genomes under the native engine; compile-time separation from production; seconds per run | **Adopt** |

## Method evidence

- **Separate lab from world.** Avida's analyze mode runs saved genotypes
  "through a test CPU and record[s] the measurements taken" outside the
  population run ([Avida wiki: Analyze File](https://github.com/devosoft/avida/wiki/Analyze-File)).
  MABE2 separates *Evaluators* ("the functions on which organisms get
  tested") from *Selectors* ([MABE2 module types](https://mabe2.readthedocs.io/en/latest/modules/01_module_types.html)).
  The lab copies this split: arenas and scorers are one module, selection
  another, and neither is reachable from `v3-core`.
- **Graded, many-case scoring beats a binary endpoint.** Lenski et al.
  (2003) found that complex functions "evolved by building on simpler
  functions that had evolved earlier, provided that these were also
  selectively favoured" ([Nature 423:139](https://doi.org/10.1038/nature01568),
  abstract; the per-treatment counts in the body were not verified here).
  Petri's ring wiring screen recorded the same trap at small scale: a 25-case
  binary selection set "creates broad fitness plateaus"
  ([ring wiring experiment](ring-wiring-experiment-2026-09-23.md)). Lab
  scores are therefore per-scene, continuous (energy gained, ticks to first
  food, distance closed), and reported as vectors.
- **Selection over many cases.** Lexicase selection chooses parents on
  training cases "considered one at a time, in random order" and outperforms
  aggregate fitness on uncompromising many-case problems
  ([Helmuth, Spector and Matheson 2015, IEEE TEVC 19(5):630–643](https://doi.org/10.1109/TEVC.2014.2362729));
  random subsampling of cases per generation improves it further at lower
  cost ([Hernandez, Lalejini, Dolson and Ofria 2019, GECCO Companion](https://doi.org/10.1145/3319619.3326900)).
  F01 uses only the user's truncation-plus-elites rule; lexicase is deferred
  to a later amendment, added only if F01's readings show the aggregate
  scalar plateauing, and the per-scene vectors are recorded from the start so
  that comparison needs no re-run.
- **Measure evolvability directly, cheaply.** An evolvability signature is
  "the post-mutation statistical distribution of both behavior diversity
  ... and fitness values" over a genotype's mutants and predicts adaptation
  speed ([Tarapore and Mouret 2015, Information Sciences 313:43–61](https://doi.org/10.1016/j.ins.2015.03.046));
  evolvability search measures "the behavioral diversity of its direct
  offspring" ([Mengistu, Lehman and Clune 2016, GECCO](https://doi.org/10.1145/2908812.2908838)).
  The lab reports this as a secondary reading on each generation's elite;
  the headline stays the user's numbers (score, generation to threshold,
  what changed in the brain).
- **Quality diversity as a later option.** MAP-Elites illuminates a search
  space by keeping the best solution per behavioral bin
  ([Mouret and Clune 2015](https://arxiv.org/abs/1504.04909)). Not in v1;
  it is the natural next step if assays show many distinct routes to a
  capability.

## Recommended design

One command: `v3-lab run --assay food-seeking --seeds 8 --generations 100`.

**Arena.** A small fixed world (48²–64², edge mode and every creature
policy at production defaults) whose only overrides are layout and initial
state: static food (growth and recovery zero), placed by a seeded pattern per
scene; barriers for the barrier assay. Several scenes per generation, seeds
fixed per replicate. No lab-only physics.

**Evaluation.** Each genome is evaluated alone per scene, from fresh state,
for a fixed lifetime (300–1,000 ticks), starting at world tick 62,680, with
reproduction suppressed by the `min_reproduce_energy` override and the
failed-action penalty retained and recorded. Because the evaluation is solo,
the simulation's own per-tick counters are the creature's: the harness reads
food intake by type, action results, energy and position between ticks and
accumulates them itself, recording the death tick when the creature dies. The
action log is not the evidence path: a tick can log several actions, the log
evicts at capacity, and it is deleted when the creature is removed. Scores are
graded per scene (energy gained, ticks to first food, distance closed,
blocked-move fraction) so partial progress is visible. Evaluating in cohorts is
a later option with its own justification, because it confounds a creature's
score with interference.

**Calibration gate (before any campaign).** The gate checks the instrument,
not the founder. On a predeclared density × lifetime grid it establishes
(1) exposure: every scene presents the opportunity the assay scores;
(2) comparator competence: the authored positive comparator scores above the
random-walk floor by a predeclared margin on every arena; and (3) score
sensitivity: the scorer separates floor from comparator and grades partial
progress between them. The founder is evaluated and its position recorded
wherever it falls, including at the floor: a capability the founder lacks
legitimately leaves it there, and the arena is never tuned until the founder
already has it. The spec predeclares the grid-selection rule, held-out
validation scenes drawn from a separate seed, and the disposition of an
`uncalibrated` result (the assay stops and reports; no campaign runs).

**Variation fidelity checklist.** Children are clones passed through
`MutationEngine::apply_mutations_with_food_type_count` with the production
`MutationConfig` and `parent_reachable_nodes = mesh_reachable_nodes(parent)`.
The executed set is frozen, not left to the implementer: `ParentExecuted::Record`
is the parent's dispatch record at the end of its last evaluated scene,
resolved at that scene's age with the production window; the harness snapshots
the record after every tick of that scene, so a parent that dies inside it
still supplies its last living record. A union across independently reset
scenes is not a production parent's record and is a separately labelled
diagnostic. The report separates requested, applied and skipped events; exact
genotype identity with the parent (which differs from zero applied events);
and elite carry-overs. Neutral offspring are preserved. Mutation, selection
and observational sampling draw from separate seeded RNG streams. At 0.005 per
unit on a 97-unit founder most children are identical to their parent, and
that rate is part of what is measured. A forced-at-least-one-event or ×k
supply arm is allowed as a labelled diagnostic and never reported as the
evolvability number.

**Selection.** F01 uses one selector: truncation with elites (the user's
sketch), keeping the top fraction and filling with mutants of the survivors,
ties broken by a seeded draw. The spec predeclares, per assay, the selection
scalar built from the per-scene vector, the reach threshold, and the
validation rule on held-out scenes so a memorized route or mere energy
conservation on the training scenes does not count as the capability. The
independent experimental unit is the replicate (one seed lineage-population);
population members and identical offspring are never confidence-interval
samples. Horizon-censored failures are reported separately from runs a cap
stopped. Lexicase over per-scene vectors is deferred to a later amendment,
added only if F01's readings show the aggregate scalar plateauing.

**Controls, every run.** Founder-only (no evolution); mutation-off GA
(exposes score noise and proves the loop does not drift); shuffled-score
selection (selection detached from performance, so improvement over it is
attributable to selection); authored ceiling instrument, labelled, never a
start; random-walk floor.

**Outputs.** One NDJSON row per replicate per generation (best, median,
mean, per-scene vectors, identical-offspring fraction, requested/applied/
skipped events by operator, elite genome size, reachable and executed node
counts) and one compact summary whose contents are a versioned keep-list
keyed to its named consumers (the assay report and research-note tables):
per-replicate reached/not, generation to threshold with censoring, Wilson
interval on the reached fraction, the calibration grid, provenance and
incomplete status. Elite genomes at the end only. A byte cap enforced before
every write covers raw, summary and genomes alike; a run that would exceed it
stops and marks itself incomplete. No per-tick traces, without exception in
T22. Raw and summary live under the ignored `.bench-artifacts/lab/`; numbers
that matter are cited in a research note or spec.

**Structure and sensor readings (F03).** Per elite per checkpoint: genome
size, `functional_complexity`, reachable and executed nodes, operator counts
since the founder, and three distinct sensor readings that F03 must not
conflate: structural reach from `creature_sensor_census` (a pure structural
read of which input keys a live reference can reach), executed use from the
tracer-backed machinery (`neighborhood::mesh_execution` hop records and
`input_use`), and causal influence from ablation on the same scenes; plus the
mutant signature (N mutants' score spread and silent/changed/dead fractions).
Any lab adapter over these seams is observation-only.

## Exploration and diagnosis

Most lab use is expected before a track exists, and the design above serves
that only if an experiment needs no Rust and no workflow:

- **Inputs are data.** Sizes are CLI parameters; an arm is a named
  `SimulationConfig` overlay on the arena config (mutation off, a supply
  multiplier, an operator-weight variant) plus an optional genome file as its
  start or comparator; F02 adds arena layouts from a JSON file. The controls
  are arms of this kind, so "what if the operator weights were X" or "start
  from this evolved genome" is a command line, not a feature. Every run
  carries the unchanged-policy reference arm, and any arm whose overlay
  changes mutation policy is classified `policy-deviation` in every verdict
  and summary: a pass under changed weights or targeting shows what a
  different policy could reach, never native reachability. The summary's
  provenance block (source revision and dirty state, resolved config digest,
  overlay contents and application order, genome and arena content hashes
  with schema versions, seeds) identifies the experiment after any file
  changes; an arm name or path never does.
- **Exploration is outside the feature workflow.** A session runs the assays,
  writes a dated research note under `docs/strategy/` in the existing form
  (options tried, measurements, compact results, a recommendation among
  roadmap feature, no action, more exploration), and lands any arena, arm or
  reading worth keeping inside `crates/v3-lab` by an ordinary commit gated by
  `make check`. No spec, Codex challenge, mutation gate or benchmark: the
  compile-time boundary is what makes that safe. A track or feature is created
  only when a note recommends one, which is how every recent track was born.
- **Why-not ladder (F04).** "Branching is not evolving" decomposes into five
  rungs, each an existing reading: exposure (does the arena present the
  opportunity and does the scorer respond to the authored comparator), supply
  (does the native engine propose changes touching the relevant nodes or
  families: requested, applied, skipped with reasons, by operator), viability
  (are those proposals silent, changed or dead), benefit (does a viable change
  move the score), and retention (does a selected change persist under
  continued native mutation and selection). F04's spec defines, per rung, the
  observable predicate, its conditional denominator (relevant sites exposed,
  not operator totals), the evidence threshold and the `inconclusive`
  disposition when exposure is insufficient, with fixtures that separate
  adjacent rungs. Retention is read over identified descendant lineages at
  declared mutation depths, never over an elite carried unchanged, and
  distinguishes deletion, lineage loss and censoring; F01's per-individual
  parent identity and applied-event counts in the NDJSON supply the ancestry
  bookkeeping. F04 reports the first failing rung with its counts and routes
  it to the owning track.

## Guardrails as written rules

1. `v3-lab` depends on `v3-core`; neither `v3-core`, `v3-cli` nor `v3-server`
   depends on `v3-lab`, and only the `v3-lab` binary consumes it. The reviewer
   checks that no lab arena, scorer, selected genome or calibration constant is
   imported into a production default, founder or recipe. If the lab needs a
   knob core lacks, that is a recorded finding, not a core patch.
2. The lab replaces births with lab selection. That is exactly the part that
   never enters production. A lab result may motivate a roadmap feature only
   through a mechanism that passes the existing natural-analog rule; a task
   reward, curriculum, scene-specific wiring or authored solution never does.
3. Lab scores are evidence for planning and research notes. A lab verdict may block or unblock roadmap rows, including mechanism rows that stay opt-in until qualified, and may retire opt-in, unqualified candidate code; it never sets a production default, founder, mutation policy or closure threshold. Production availability and any rate or targeting default still require goal-profile ecological evidence recorded in the changing feature's own spec; a lab result can suggest that change, never justify it alone.
   They never enter `make check` beyond a seconds-scale fixture, and the
   master rule "no feature may optimize against an indicator" applies to
   every lab assay by name. (User decision, 2026-09-28.)
4. Lab features are measurement tooling: the natural-analog rule does not
   apply to them, and they add no environmental pressure.
5. A lab pass shows a capability is reachable by native variation under lab
   selection. It does not show ecological retention or transfer, which stay
   with the goal profile and the owning tracks. Necessary, not sufficient.

## Placement

A new track, **T22 — Capability Assays and Evolvability Lab**
([draft](../roadmaps/t22-capability-assays-and-evolvability-lab.md)), with
four features: F01 harness plus the food-seeking assay and controls; F02 the
barrier-navigation assay and arena files; F03 brain and sensor change readings
on the same runs; F04 the why-not diagnostic ladder. T21 is left free for the observability track the 2026-09-27 revert rolled
back: the revert commit records the intent to restart it, and keeping the ID
free is a planning decision recorded here, not a rule the revert imposes. User decisions, 2026-09-28: T22.F01–F03 are the next new starts ahead of
T20.F09; lab features skip the gate and goal benchmark profiles at closure
(recorded in the workflow's Benchmark gate section); the lab is the framework
of record for capability, discovery and reachability questions, so the
restarted T20.F09 depends on T22.F01 and runs on it; and every
telemetry-producing command commits only the minimum summary that regenerates
its report, with raw archives capped at the producing command and no per-tick
traces (`AGENTS.md` and the workflow's telemetry commit rule).

## Remaining uncertainty and proof of concept

Answered from source during review: a gate-rejected Reproduce vote is charged
the failed-action penalty, so the lab keeps the penalty and starts evaluations
at the ramp's default target tick, 62,680. F01's first implementation brief
settles the rest with fixtures inside `crates/v3-lab`; none blocks the spec:

1. A founder lifetime and food density at which founders find food in a
   48²–64² sparse arena at all (the calibration grid's first point), and an
   end-to-end solo-evaluation pilot that replaces the projected runtimes.
2. Fixtures for the frozen executed-set rule (last scene, age-windowed,
   per-tick snapshot) and for the per-tick score accumulation path, including
   death inside a scene.
3. Whether the tracer-backed executed-use readings can be taken on a lab elite
   without the T11.F26 cohort machinery, or whether F03 needs a small
   observation-only adapter.

## Adversarial review record (Codex Astra `high`, 2026-09-28)

Job `task-mull2xlr-x9masf`, fresh read-only thread, verdict `not-ready`.
Every code claim it made was verified against the tree before disposition.

| # | Finding (condensed) | Disposition |
| --- | --- | --- |
| B1 | T20's evidence gates still let an F09 lab verdict govern downstream execution and candidate disposal, which T22's "never a closure gate" rule forbids. | **Applied (user approved 2026-09-28).** A lab verdict may block or unblock roadmap rows, including opt-in mechanism rows such as T20.F03, and retire opt-in, unqualified candidate code, never set a default, founder, mutation policy or closure threshold; F12's production availability still needs F10/F11 goal-world evidence in its own spec. Recorded in T22, the T20 F09 restart note and guardrail 3. |
| B2 | T11.F13 chooses production mutation rates and targeting from discovery/retention comparisons, a route from lab selection into production policy. | **Applied (user approved 2026-09-28).** T11.F10's lab readings are diagnostic inputs; T11.F13 changes a rate or targeting default only with goal-profile evidence recorded in its own spec. Recorded in the T11.F10 and T11.F13 notes. |
| B3 | Calibration required founder competence above the floor, which tunes the arena to the founder and excludes capabilities it lacks. | Fixed: calibration checks exposure, comparator competence and score sensitivity; founder recorded wherever it falls; grid rule, held-out scenes and `uncalibrated` disposition predeclared. |
| B4 | Executed-set choice (last scene or union) deferred to the brief although it changes the mutation treatment; `ParentExecuted::Record` is age-windowed. | Fixed: frozen to the last evaluated scene's record at its age with the production window, snapshotted per tick; union is a labelled diagnostic. |
| B5 | Action-log scoring loses evidence: several actions per tick, eviction at capacity, deletion on death. | Fixed: solo evaluation accumulates the simulation's per-tick counters; death tick recorded; the log is not the evidence path. |
| A1 | Rejected Reproduce is charged the failed-action penalty; startup ramp target is tick 62,680. | Fixed: claim corrected, penalty retained, evaluations start at tick 62,680. |
| A2 | "No gradient" stated as fact; exposure/applicability are rival explanations. | Fixed: labelled a hypothesis; exposure, applicability and a mutant score-distribution check required before reading a null. |
| A3 | Selection scalar, threshold, validation rule and independent unit undefined. | Fixed in the Selection paragraph and the T22 F01 contract. |
| A4 | Clone accounting conflates zero events, identical genotypes and carried elites; RNG streams unspecified. | Fixed. |
| A5 | Runtime projections extrapolate a populated-world average; stale-binary provenance missing. | Fixed: labelled unverified; provenance recorded; pilot required. |
| A6 | F01 overlaps F03 and carries two selectors and five controls. | Fixed: F01 keeps one selector and the validity controls; richer readings and mutant signatures are F03; lexicase deferred. |
| A7 | `creature_sensor_census` is structural, not executed use. | Fixed: three readings distinguished. |
| A8 | F02 calls an authored controller a "ceiling" and says lineages "learn". | Fixed: authored comparator; solvability demonstrated per arena; learning disabled and attributed. |
| A9 | Manifest/lock changes could alter production dependency resolution under the exemption. | Fixed in the workflow: existing versions, features and build settings must stay unchanged and the reviewer verifies it. |
| A10 | Blanket benchmark/specialist requirements elsewhere; how exempt closures appear in series and references. | Fixed: master criterion, AGENTS.md delegation bullets and the workflow now name the exemption; exempt closures add no series entry; features in other tracks that use the lab are not exempt. |
| A11 | Boundary rule covered only `v3-core`; no check against importing lab artifacts. | Fixed in guardrail 1 and the T22 Ownership note. |
| A12 | Existing uncapped writers versus an immediate universal cap. | Fixed: writers existing on 2026-09-28 keep their behavior until next touched. |
| A13 | Trace wording inconsistent across AGENTS.md, workflow and artifact contract. | Fixed: predeclared traces allowed generally; T22 forbids them outright. |
| A14 | "Regenerate the report" lacks the consumer-defined keep-list boundary. | Fixed: regeneration means rendering named consumers; each command declares a versioned keep-list. |
| A15 | T21 reservation attributed to the revert; opening claim of unchanged priority. | Fixed. |

### Round 2 (job `task-mulm0f5t-y06g2b`, verdict `not-ready`)

Twelve dispositions accepted; eight rejected for residual wording (B1 F03
listed as a measurement row, B3 the founder-above-floor criterion, B4/A1 the
superseded open questions, A6 lexicase as a v1 alternative, A8 "learn" and
"ceiling" in the F02 row, A10 the master benchmark bullet, A13 the master
T22 note's trace wording), all corrected. Five new blocking findings:

| # | Finding (condensed) | Disposition |
| --- | --- | --- |
| R2-1 | Exploration commits confined to `crates/v3-lab` could still change its manifest, build scripts or shared build settings. | Fixed: exploration commits touch only `crates/v3-lab/src`, `crates/v3-lab/tests` and `docs/strategy/`; any manifest, lock, build-script, `Makefile`, shared-setting or production-test change is a roadmap feature; the commit message records the scope check. |
| R2-2 | Arbitrary policy overlays labelled as native variation. | Fixed: unchanged-policy reference arm on every run; every mutation-policy overlay classified `policy-deviation`, never reported as native reachability. |
| R2-3 | Provenance undefined for mutable inputs. | Fixed: provenance block defined (revision and dirty state, resolved config digest, overlay contents and order, genome and arena hashes with schema versions, seeds). |
| R2-4 | Ladder rungs lack predicates, denominators, thresholds and an inconclusive outcome. | Fixed: F04's spec defines all four per rung with fixtures separating adjacent rungs. |
| R2-5 | Retention conflated with unchanged elite carry-over. | Fixed: retention over identified descendant lineages at declared depths; carry-overs excluded; deletion, lineage loss and censoring distinguished; ancestry bookkeeping assigned to F01's NDJSON. |

### Round 3 (job `task-mulm7pt0-x755vj`, verdict `ready`)

All seven round-2 corrections and all five R2 dispositions accepted; no new
findings. The loop ended at a `ready` verdict after three rounds.

## Sources

- [Petri master roadmap, Notes for AI Agents](../roadmap.md#notes-for-ai-agents): "bounded fixed environments as assays"; platform review of MABE2, Avida and JAX simulators (2026-09-03).
- [Ring wiring experiment, 2026-09-23](ring-wiring-experiment-2026-09-23.md); [ring group experiments](ring-group-experiments-2026-09-23.md); probe sources under `.bench-artifacts/ring-*`.
- Reverted T20.F09 reading, `git show 1c7ba514:docs/progress/readings/t20-f09.md`.
- crates.io API, read 2026-09-28: `radiate`, `genetic_algorithm`, `genevo`, `oxigen`.
- [radiate docs](https://docs.rs/radiate/latest/radiate/); [genetic_algorithm genotype docs](https://docs.rs/genetic_algorithm/latest/genetic_algorithm/genotype/index.html).
- [Avida wiki: Analyze File](https://github.com/devosoft/avida/wiki/Analyze-File); [MABE2 module types](https://mabe2.readthedocs.io/en/latest/modules/01_module_types.html).
- Lenski, Ofria, Pennock and Adami, [The evolutionary origin of complex features](https://doi.org/10.1038/nature01568), Nature 423:139–144 (2003), abstract.
- Helmuth, Spector and Matheson, [Solving uncompromising problems with lexicase selection](https://doi.org/10.1109/TEVC.2014.2362729), IEEE TEVC 19(5):630–643 (2015).
- Hernandez, Lalejini, Dolson and Ofria, [Random subsampling improves performance in lexicase selection](https://doi.org/10.1145/3319619.3326900), GECCO Companion (2019).
- Tarapore and Mouret, [Evolvability signatures of generative encodings](https://doi.org/10.1016/j.ins.2015.03.046), Information Sciences 313:43–61 (2015); [arXiv 1410.4985](https://arxiv.org/abs/1410.4985).
- Mengistu, Lehman and Clune, [Evolvability search](https://doi.org/10.1145/2908812.2908838), GECCO (2016).
- Mouret and Clune, [Illuminating search spaces by mapping elites](https://arxiv.org/abs/1504.04909) (2015).
