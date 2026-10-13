# Program 1 Roadmap Archive (2026-09-02 to 2026-10-12)

> Historical record only. Nothing here is executable roadmap guidance, and
> nothing here is maintained. The live roadmap contract is in
> [`docs/roadmaps/README.md`](../../roadmaps/README.md). Closure evidence stays
> live in [`docs/progress.md`](../../progress.md), the committed summaries under
> [`docs/progress/features/`](../../progress/features/) and the readings under
> [`docs/progress/readings/`](../../progress/readings/).

This directory holds the first Petri roadmap program as it stood when the user
reset it on 2026-10-12: the [master roadmap](roadmap.md), its 24
[track roadmaps](roadmaps/) and 93 [feature specs](specs/roadmap/). Links inside
the program resolve within this directory. The success definition, "Diverse
Ecosystems and Complex Cognition", carries forward unchanged to the next
program; the open rows do not.

| Count | Value |
| --- | --- |
| Tracks | 24 (3 complete) |
| Feature rows | 199 |
| Closed | 91 |
| Open when archived, not carried forward | 108 |

## Why the program was reset

Most of the program went into the brain's genotype-phenotype map: how mutation
reaches the parts of a brain that run, how new structure joins a circuit
without breaking it, and how a brain chooses actions. By 2026-10-12 the user
judged that core problem solved well enough for evolution to proceed, and the
remaining rows were either held, lab-bound, or written against problems that
no longer described the live world. The next program starts from what the live
world shows now rather than from those rows.

## What made evolution start working

The user's judgment on 2026-10-12, with the evidence each claim rests on.

1. **Separate drive and direction sinks.** [T11.F31 — Ring Projection
   Motif](specs/roadmap/t11-f31-ring-projection-motif.md) lets a neighbor ring
   connect to a kind's eight direction votes as one gene, born at zero weight.
   [T11.F30 — Kind Drives](specs/roadmap/t11-f30-kind-drives.md) gives each
   directed action kind one drive vote, so whether to act is apart from where.
   [T11.F21](specs/roadmap/t11-f21-per-direction-motor-output.md) built the
   per-direction bank they factor. The user called this the final missing
   piece.
   - The [factored motor output note](../../strategy/factored-motor-output-research-2026-10-11.md)
     found the founder's four-direction food taxis eroded to a median of one
     direction in the live world before these features.
   - The [barrier awareness probe](../../strategy/barrier-awareness-probe-2026-10-12.md)
     found barrier avoidance evolving through ring projections: projection
     avoiders rose from 22 to 121 of the sample, with a live blocked fraction
     of 0.12 against about 0.30 for others.
   - Three 50,000-tick Canyon runs at T11.F30's closure recovered to about
     1,400 to 1,500 creatures, with barrier readers making 93 to 99.9% of
     barrier-adjacent moves and their blocked fraction falling to 12 to 23%.
2. **Neutral input recruitment.** [T20.F01](specs/roadmap/t20-f01-input-use-baseline-and-ecological-opportunity.md)
   measured where input use stalls. [T20.F02](specs/roadmap/t20-f02-graph-neutral-connection-contract.md)
   set the contract for a silent, refinable input connection.
   [T20.F04](specs/roadmap/t20-f04-general-neutral-input-recruitment.md) added
   a bounded recruitment operator. [T20.F16](specs/roadmap/t20-f16-born-connected-input-declaration.md)
   made every new Graph input declaration arrive already connected.
   - Production runs T20.F02 and T20.F16. T20.F04's operator is `Off` by
     default and in every committed world file.
   - The [born-connected run](../../strategy/born-connected-run-2026-10.md) and
     the [output-wiring run](../../strategy/output-wiring-run-2026-10.md)
     record the born edge's effect on Canyon seeds 1022 and 2022.
   - The [vote-margin read](../../strategy/vote-margin-read-run-2026-10.md)
     found 38 of 89 executed input channels vote-causal on Confluence at tick
     50,000, every consumer a vote sink, the born-edge shape.
3. **Structured refinement, possibly.** [T20.F05](specs/roadmap/t20-f05-structured-heritable-refinement.md)
   adds bounded semantic input-to-vote refinement. It is off by default and in
   every committed world file, and no reading attributes a live result to it.
   The user marked it "maybe".

The 2,000-tick goal profile does not register these changes. Its share of
behavior-changing births stayed between 0.11 and 0.24 across the last 16
closures. The evidence above comes from long runs on the live server and the
lab.

The foundations these stand on, closed earlier in the program:

- **Mutation reaches what runs.** Executed-biased targeting (T11.F17),
  per-unit supply (T11.F19, T11.F20) and stable parent membership (T11.F24).
- **Growth that preserves function.** VM and Graph structural semantics
  (T11.F02, T11.F03), duplication (T11.F08), backend-neutral node growth
  (T11.F18) and routing semantics (T11.F15, T11.F29).
- **Votes instead of an action bank.** T19.F01 to T19.F06 replaced push, pop
  and execute machinery with competing votes and live internal state.
- **Size and compute pay their way.** Carrying cost (T03.F08), ramped compute
  cost (T03.F10), replication cost (T03.F11) and reproductive time (T03.F12).
- **Instruments.** The neighborhood indicator (T11.F01, T14.F12), drift walk
  (T11.F16), capability lab and why-not ladder (T22), run telemetry (T21) and
  creature windows (T23.F10).

## Findings carried into the next program

Open problems measured on the live world, not yet owned by any row.

1. **Genome bloat at depth.** Live run 6d4ea0c8 held about 1,300 creatures to
   tick 37,000, then went extinct by tick 84,000 while food rose. Mean genome
   size grew from 121 to about 3,000 units, and three size-scaled costs choked
   births. The genome size cap is inert while `genome_size_pressure_enabled`
   is false, the default.
2. **Reproduction decoupled from condition.** The [fruit episodes
   note](../../strategy/fruit-episodes-2026-10-12.md) found only 12% of
   creatures still gate reproduction on energy and age. Rich food cannot become
   offspring, so a food niche cannot pay off.
3. **Edge and VM barrier avoiders fail live.** They are perfect on the lab's
   wall arena but show no live benefit. The cause is not known.
4. **Memory appears only at long horizons.** The output-wiring run read memory
   sensitivity at 0 of 18,053 creatures at tick 2,000 and 6 to 12 of about
   3,000 at tick 20,000. A 2,000-tick reading cannot see it.
5. **Steal requests are unbounded.** A VM `WriteActionParam` is unclamped, so a
   creature once requested about 1e9 energy. Energy stayed conserved, but
   telemetry ratios broke. The user's decision was pending at archive.

## Dropped at reset

None of these carries forward as a row. A new program may re-derive any of
them from current evidence.

- **Held rows at the head of the queue.** T11.F10 memory-motif evolvability
  (held because lab runs are expensive), T19.F07 failed step ends the plan
  (held after the effort-cost defaults landed; a partial spec draft was left in
  the `worktree-t19-f07` worktree), T18.F01 and T18.F02 specialized founder
  (on hold since T19).
- **Lab qualification chains.** T20.F09 to T20.F15 and T20.F03, T13.F08 to
  T13.F10, T11.F12 and T11.F13, T08.F02 and T08.F05.
- **Deletion-rate feature.** On 2026-10-09 the user chose to build it without
  supporting evidence; no row was written. The prototype is on branch
  `worktree-deletion-rate` and the evidence in the
  [deletion-rate run note](../../strategy/deletion-rate-run-2026-10.md). The
  related lead "slower loss of silent structure" stayed a lead.
- **Deferred causal-proof phase.** T01.F01 to T01.F10, T10.F01 to T10.F08 and
  the campaign rows in the domain tracks, which existed to prove diversity and
  cognition to an outside reviewer.
- **Never-started ecology tracks.** T04 to T07, most of T02, T03 and T09. Their
  themes (seasons, occlusion, body traits, predation, niche construction,
  communication) remain the natural material for "Diverse Ecosystems" in the
  next program.
- **Sexual reproduction.** T08.F06 to T08.F09, homology, mating and
  recombination.
- **Smaller rows.** T01.F13, T12.F05, T14.F09, T16.F03, T17.F03, T17.F04,
  T11.F11, and T23.F01 to T23.F09.

State the archive leaves untidy, recorded rather than fixed: T22's four
features are closed while its success criteria and master row are unchecked;
T11's criteria are unchecked; a Codex review of the progress page, T11.F30's
deferred Codex P2 finding and T11.F31's cost line were still owed.

## Closed features

Dates come from the progress table, or from the spec's last update for features
with no benchmark row. Goal lines are the track rows as written, cut at 200
characters.

### T01 — Experimental Science and Causal Evaluation

| Feature | Closed | Goal line (first sentence) |
| --- | --- | --- |
| [T01.F11 — Baseline Persistence Characterization](specs/roadmap/t01-f11-baseline-persistence-characterization.md) | 2026-09-04 | Measure how long populations persist at each world size under production defaults, so every later reading has a baseline. |
| [T01.F12 — Goal Profile, Basic Indicators, and Progress Table](specs/roadmap/t01-f12-goal-profile-basic-indicators-and-progress-table.md) | 2026-09-05 | Read diversity and cognition once per closed feature: a minutes-scale goal profile, a lineage-diversity indicator, a memory-sensitivity indicator, and `docs/progress.md` with one row per feature. |

### T02 — Environmental Dynamics

| Feature | Closed | Goal line (first sentence) |
| --- | --- | --- |
| [T02.F04 — Grazing Recovery and Overuse](specs/roadmap/t02-f04-grazing-recovery-and-overuse.md) | 2026-09-15 | Overgrazing. Every bite halves the grazed cell's fertility for that food type, repeated bites compound down to a floor, and a bitten cell takes about 500 ticks to recover, so a patch grazed too rec… |
| [T02.F08 — Partial Bites](specs/roadmap/t02-f08-partial-bites.md) | 2026-10-12 | Partial grazing. Eat gains an evolvable share of the cell's food to leave behind, so a creature can stop short of energy it cannot store and a lightly grazed cell keeps growing in place. |

### T03 — Functional Traits and Metabolism

| Feature | Closed | Goal line (first sentence) |
| --- | --- | --- |
| [T03.F08 — Genome Size Maintenance Cost](specs/roadmap/t03-f08-genome-size-maintenance-cost.md) | 2026-09-07 | Carrying tissue costs, even idle tissue. Every unit of brain structure a creature carries costs energy each tick whether or not it ever runs, so unused structure is shed by selection and genome siz… |
| [T03.F10 — Activity-Ramped Compute Cost](specs/roadmap/t03-f10-activity-ramped-compute-cost.md) | 2026-09-07 | Thinking burns fuel and fatigue sets in. A brain pays for each instruction it runs, cheaply for ordinary programs and at a rising rate for sustained execution within one dispatch, so a runaway loop… |
| [T03.F11 — Genome Replication Cost](specs/roadmap/t03-f11-genome-replication-cost.md) | 2026-09-15 | Copying a longer genome costs more, as in Avida and Tierra. Every unit of brain structure beyond the founder's raises the parent's energy cost to reproduce, so a large genome pays at every birth as… |
| [T03.F12 — Genome-Length-Dependent Reproductive Time](specs/roadmap/t03-f12-genome-length-dependent-reproductive-time.md) | 2026-10-08 | Copying a longer genome takes longer, as in Avida's replication time. A parent that has just reproduced is occupied for a span that grows with its genome's length, so a large genome pays in reprodu… |

### T10 — Evolutionary Scale and Experiment Infrastructure

| Feature | Closed | Goal line (first sentence) |
| --- | --- | --- |
| [T10.F09 — Throughput Baseline and Profiling Budget](specs/roadmap/t10-f09-throughput-baseline-and-profiling-budget.md) | 2026-09-04 | Measure how fast the world runs and where the time goes, so later features have a compute budget. |
| [T10.F10 — Deterministic Benchmark Harness and Per-Feature Report](specs/roadmap/t10-f10-deterministic-benchmark-harness.md) | 2026-09-03 | One command that measures every feature the same way and fails on a severe compute regression. |
| [T10.F11 — Cross-Process Reproducibility of Seeded Runs](specs/roadmap/t10-f11-cross-process-reproducibility-of-seeded-runs.md) | 2026-09-04 | The same seed produces the same world in any process at any thread count. |

### T11 — Brain Genotype-Phenotype Map

| Feature | Closed | Goal line (first sentence) |
| --- | --- | --- |
| [T11.F01 — Mutational Neighborhood Indicator](specs/roadmap/t11-f01-mutational-neighborhood-indicator.md) | 2026-09-05 | One step away. At every closure, read how often a single mutation leaves a brain acting like its parent, acting differently, or not acting at all, on the founder and on evolved genomes, with floors… |
| [T11.F02 — VM Structural Mutation Semantics](specs/roadmap/t11-f02-vm-structural-mutation-semantics.md) | 2026-09-05 | Gene insertion, deletion, and duplication. Adding, removing, replacing, or copying instructions keeps every surviving jump pointing where it did, an operand mutation changes one field by a small st… |
| [T11.F03 — Function-Preserving Graph Growth](specs/roadmap/t11-f03-function-preserving-graph-growth.md) | 2026-09-05 | New neural connections. A compute node joins the circuit disconnected or by splitting an existing edge with its function preserved, copies never rewire live nodes, and new edges can reach every sen… |
| [T11.F04 — Mutation Supply and Neutral Scaffold](specs/roadmap/t11-f04-mutation-supply-and-neutral-scaffold.md) | 2026-09-06 | Point mutations one at a time. Mutation arrives mostly as single events at a provisional mean of about 0.55 requested events per birth, inactive structure receives the same target opportunity as li… |
| [T11.F05 — Temporal Controller Fixtures](specs/roadmap/t11-f05-temporal-controller-fixtures.md) | 2026-09-05 | Check the brain's building blocks with small delayed-cue, memory-retention, and delayed-reward fixtures on constructed controllers, recording capability and measured gaps before interpreting evolut… |
| [T11.F06 — Graph Memory Clock](specs/roadmap/t11-f06-graph-memory-clock.md) | 2026-09-06 | Neural timescales. Graph state has an explicit committed-state clock; T19.F02 supersedes this feature's original once-per-world-tick update with live per-visit updates, while eligibility decay foll… |
| [T11.F07 — Reward Trace Clock](specs/roadmap/t11-f07-reward-trace-clock.md) | 2026-09-06 | Synaptic eligibility. Recent activity fades with elapsed world time and influences later learning through one explicit, calibrated reward-update rule. |
| [T11.F08 — Function-Preserving Duplication and Module Growth](specs/roadmap/t11-f08-function-preserving-duplication-and-module-growth.md) | 2026-09-06 | Gene duplication. Copying a working module keeps it working, so complexity can grow by copy and divergence as in real genomes. |
| [T11.F09 — Learned-State Inheritance Integrity](specs/roadmap/t11-f09-learned-state-inheritance-integrity.md) | 2026-09-10 | Inherited neural adaptation. When learned weights are heritable, structural mutation preserves their association with surviving connections and initializes new connections explicitly. |
| [T11.F14 — Mesh Execution Observability](specs/roadmap/t11-f14-mesh-execution-observability.md) | 2026-09-06 | Generations, not ticks; executed, not reachable. Record what a lineage's mesh actually runs, whether its route ever varies with the input, how often it loops to the hop cap, and how many generation… |
| [T11.F15 — Mesh Routing Connection Semantics](specs/roadmap/t11-f15-mesh-routing-connection-semantics.md) | 2026-09-06 | A synapse forms with its own trigger. A new route branch carries its own bid and points at working structure, retargeting lands near, and removal is not usually lethal; the original single-visit re… |
| [T11.F16 — Drift-Depth Indicator](specs/roadmap/t11-f16-drift-depth-indicator.md) | 2026-09-07 | Generations, not ticks, read at depth. Walk lineages from the founder under the production mutation engine to generations 250, 1,000, and 2,000 and read what their meshes execute and how often a bi… |
| [T11.F17 — Executed-Biased Mutation Targeting](specs/roadmap/t11-f17-executed-biased-mutation-targeting.md) | 2026-09-07 | Expressed genes mutate more. Node-internal mutation events land mostly on the nodes a creature's brain actually ran in its recent ticks, so the functional core keeps receiving mutations however muc… |
| [T11.F18 — Backend-Neutral Mesh Node Growth](specs/roadmap/t11-f18-backend-neutral-mesh-node-growth.md) | 2026-09-08 | Developmental variation in new neural tissue. A creature can grow either kind of brain module along a working decision path without losing its current behavior, so graph computation has the same di… |
| [T11.F19 — Per-Unit Mutation Supply](specs/roadmap/t11-f19-per-unit-mutation-supply.md) | 2026-09-14 | Copy errors per base. Every `genome_size()` unit a parent carries is an independent chance of one mutation event at birth, so the expected number of events grows with the genome instead of being a… |
| [T11.F20 — Per-Birth Supply Rule Retirement](specs/roadmap/t11-f20-per-birth-supply-rule-retirement.md) | 2026-09-23 | One way to mutate. The per-unit draw is the only mutation-count rule in the engine, the config, the panel, and the reference specs; the drift walk and the recruitment-paths experiment hold their fi… |
| [T11.F21 — Per-Direction Motor Output](specs/roadmap/t11-f21-per-direction-motor-output.md) | 2026-09-16 | A motor map with winner-take-all selection. A movement action (Move, Reproduce, StealEnergy) may carry an eight-slot direction bank, one bid per neighbor cell, and the body commits the direction wi… |
| [T11.F22 — Meaning-Stable Input References](specs/roadmap/t11-f22-meaning-stable-input-references.md) | 2026-09-16 | A neuron keeps the afferents it was wired with; a receptive field may drift within its modality but never changes modality. A node's sensor references keep their kind for the node's life and its de… |
| [T11.F23 — Scale-Relative VM Constant Steps](specs/roadmap/t11-f23-scale-relative-vm-constant-steps.md) | 2026-09-18 | A mutation nudges a number by a share of what it is, not by a fixed amount. `VmConstantMutation` today adds a uniform draw from [−1, 1] to one constant of a VM program's pool regardless of the cons… |
| [T11.F24 — Stable Parent Membership During Mutation](specs/roadmap/t11-f24-stable-parent-membership-during-mutation.md) | 2026-09-24 | Mutations follow the tissue that existed in the parent: later events in one birth retain correct parent-reachable and parent-executed membership after deletion or addition changes the child's node… |
| [T11.F25 — Meaningful Action-Parameter Targets](specs/roadmap/t11-f25-meaningful-action-parameter-targets.md) | 2026-09-24 | Variation reaches expressed action traits: new Graph and VM parameter connections target fields the body's current action decoder actually consumes. |
| [T11.F27 — Compact Action-Parameter Storage](specs/roadmap/t11-f27-compact-action-parameter-storage.md) | 2026-09-24 | Like a motor neuron that can only synapse on muscle that contracts, a brain's action-parameter outputs exist only for the three parameters the body reads (Eat food type, Reproduce transfer fraction… |
| [T11.F26 — Mutation-Effect Attribution and Observation Coverage](specs/roadmap/t11-f26-mutation-effect-attribution-and-observation-coverage.md) | 2026-09-24 | Show in the Petri progress report whether applied mutations change a creature's computation, state or actions, separating observed barriers to behavioral change from changes missed by the observati… |
| [T11.F29 — Feedback Route Formation](specs/roadmap/t11-f29-feedback-route-formation.md) | 2026-10-09 | Feedback projections, a later stage of a nervous system wiring back to an earlier one. A mesh can grow a route branch back to a node on its own working path, itself included, born silent like any n… |
| [T11.F31 — Ring Projection Motif](specs/roadmap/t11-f31-ring-projection-motif.md) | 2026-10-11 | A topographic map from one developmental program, as central-complex columns repeat one circuit around the heading ring and molecular gradients wire a whole retinotopic map. A neighbor ring connect… |
| [T11.F30 — Kind Drives](specs/roadmap/t11-f30-kind-drives.md) | 2026-10-12 | Motor gating apart from aiming, as a collicular gate and burst vigor start a saccade whose direction the map sets, and as a bacterium's run keeps its heading until something turns it. Each directed… |

### T12 — World Composition and Baseline Worlds

| Feature | Closed | Goal line (first sentence) |
| --- | --- | --- |
| [T12.F01 — Seeded Terrain in the World Config](specs/roadmap/t12-f01-seeded-terrain-in-the-world-config.md) | 2026-09-08 | Bedrock and water. The land has a shape before anything lives on it: barrier terrain is generated from the world config and seed at startup, before food and founders are placed, so where a creature… |
| [T12.F02 — World Recipe Save and Load](specs/roadmap/t12-f02-world-recipe-save-and-load.md) | 2026-09-08 | A world is a file. The effective world config, terrain and seeds included, saves from the app or the CLI and loads back to regenerate the same world, so a baseline is a small recipe in Git rather t… |
| [T12.F04 — Baseline World Set](specs/roadmap/t12-f04-baseline-world-set.md) | 2026-09-09 | Plains, orchards in grassland, canyon country, and one world where everything meets. Food types first gain their own energy per bite, coverage, regrowth rate, and recovery rate in the world config,… |
| [T12.F06 — Fertility Channels](specs/roadmap/t12-f06-fertility-channels.md) | 2026-10-10 | Streams through dry country. A fertility layer can draw narrow, wavy channels of fertile ground from ridged fBm noise, a few cells wide and set by the recipe, and a layer can blend by maximum inste… |

### T13 — Neutral Module Recruitment

| Feature | Closed | Goal line (first sentence) |
| --- | --- | --- |
| [T13.F01 — Module Recruitment Observability](specs/roadmap/t13-f01-module-recruitment-observability.md) | 2026-09-10 | Observe a new brain module from its creation through mutation, expression, contribution and loss, so each closure shows where recruitment advances or stalls. |
| [T13.F02 — Recruitment Paths and Replicated Baseline](specs/roadmap/t13-f02-recruitment-paths-and-replicated-baseline.md) | 2026-09-13 | Measure how often new Graph and VM modules acquire useful behavior through viable steps, separating lineage variation, dormant preparation and selection from mere growth. |
| [T13.F03 — Mutation Target Applicability](specs/roadmap/t13-f03-mutation-target-applicability.md) | 2026-09-13 | Point mutations alter existing molecular sites: refinement reaches a module with a suitable site while silent tissue keeps its own opportunities to grow and connect. |
| [T13.F04 — Direct Graph Effect Activation](specs/roadmap/t13-f04-direct-graph-effect-activation.md) | 2026-09-14 | Direct synaptic transmission: a wired signal can affect a controller without first growing an unrelated processing cell, while unwired tissue preserves the incumbent decision. |
| [T13.F05 — Function-Preserving Module Recruitment](specs/roadmap/t13-f05-function-preserving-module-recruitment.md) | 2026-09-14 | Gene duplication and circuit growth: new Graph and VM tissue can extend working computation through short viable mutation paths and then diverge into a useful contribution. |
| [T13.F06 — Recruitment and Retention Qualification](specs/roadmap/t13-f06-recruitment-and-retention-qualification.md) | 2026-09-14 | Measure whether newly recruited tissue is discovered and retained because it helps a creature, including the cost of carrying it and its response to a changed task. |
| [T13.F07 — Current-Policy Recruitment Transitions](specs/roadmap/t13-f07-current-policy-recruitment-transitions.md) | 2026-09-20 | Observe where newly created brain tissue stops progressing from mutation opportunity to expression, useful specialization and retention under the current mutation policy. |

### T14 — Runtime Telemetry and Report Integrity

| Feature | Closed | Goal line (first sentence) |
| --- | --- | --- |
| [T14.F01 — Closure Comparison and Indicator Coverage Integrity](specs/roadmap/t14-f01-closure-comparison-and-indicator-coverage-integrity.md) | 2026-09-11 | A closure's verdict means what it says: the report compares against an earlier closure or records that it has none, and every wired indicator is visible to the no-regression rule. |
| [T14.F02 — Existing Counter Transfer](specs/roadmap/t14-f02-existing-counter-transfer.md) | 2026-09-11 | Counters the simulation already maintains — production mutation supply and targeting, mutation lifecycle outcomes, predation, failed eats, energy-exhausted dispatches — become readable at closure i… |
| [T14.F03 — Applied Mortality and Energy Accounting](specs/roadmap/t14-f03-applied-mortality-and-energy-accounting.md) | 2026-09-12 | Death is counted and attributed to the sink that caused it, and every energy flow the world applies is totalled, so a cost feature's effect is read from what it did rather than from what it charged. |
| [T14.F04 — Population Readings on the Persistence Checkpoints](specs/roadmap/t14-f04-population-readings-on-the-persistence-checkpoints.md) | 2026-09-11 | Lineage diversity, carried structure and generation depth are read at the checkpoints the run already samples, so a closure shows when a change took hold and not only where it ended. |
| [T14.F05 — Cognition Telemetry Resolution](specs/roadmap/t14-f05-cognition-telemetry-resolution.md) | 2026-09-12 | A plasticity update that moves a weight is distinguishable from one that does not, reward-modulated learning is counted separately from Hebbian, a shared-memory write that changes a slot is counted… |
| [T14.F06 — Reproductive Success by Cognitive Class](specs/roadmap/t14-f06-reproductive-success-by-cognitive-class.md) | 2026-09-12 | Whether cognition pays is readable: creatures that read memory, hold state or learn are counted at death with their offspring and lifespan beside those that do not, so selection for remembering is… |
| [T14.F07 — Surviving-Clade Behavioral Profile](specs/roadmap/t14-f07-surviving-clade-behavioral-profile.md) | 2026-09-15 | Each clade that survives to closure shows how it made its living — what it ate, what it did, how large and how old it grew — so a count of survivors becomes a reading of how many of them live diffe… |
| [T14.F08 — Sensor Usage Census](specs/roadmap/t14-f08-sensor-usage-census.md) | 2026-09-12 | What the population can perceive is read at the checkpoints — which world inputs living brains reference, and how many read anything stateful — so a food niche nobody senses or a memory no brain re… |
| [T14.F10 — Spatial Occupancy by Clade](specs/roadmap/t14-f10-spatial-occupancy-by-clade.md) | 2026-09-12 | Where life is becomes readable on the worlds built to shape it — a coarse grid of population and clade count at each checkpoint — so habitat partitioning shows as a way of coexisting rather than st… |
| [T14.F11 — Progress Page Information Design](specs/roadmap/t14-f11-progress-page-information-design.md) | 2026-09-15 | `docs/progress/index.html` reads as answers rather than a spray of charts: it leads with the questions the success definition asks, shows each indicator across the three worlds side by side against… |
| [T14.F12 — Neighborhood Read of Selected Genomes](specs/roadmap/t14-f12-neighborhood-read-of-selected-genomes.md) | 2026-09-14 | Whether evolution is still finding behavior at depth is read on the genomes the world actually kept: at closure, fresh births from a sample of each world's surviving population are classified as si… |

### T15 — Benchmark Evidence Storage

| Feature | Closed | Goal line (first sentence) |
| --- | --- | --- |
| [T15.F01 — Local Raw Artifacts and Committed Benchmark Summaries](specs/roadmap/t15-f01-local-raw-artifacts-and-committed-benchmark-summaries.md) | 2026-09-13 | The world's measured outcomes remain readable and comparable across closures from concise committed summaries, while full experiment records stay local and their availability is stated honestly. |
| [T15.F02 — Historical Report Migration and Main-History Rewrite](specs/roadmap/t15-f02-historical-report-migration-and-main-history-rewrite.md) | 2026-09-13 | Every earlier benchmark report is reduced to concise committed evidence and preserved as a local ignored raw artifact, then the full reports are removed from `main` history and the rewritten branch… |

### T16 — Cost and Cue Fidelity

| Feature | Closed | Goal line (first sentence) |
| --- | --- | --- |
| [T16.F01 — Gate Before Charge in Reproduction](specs/roadmap/t16-f01-gate-before-charge-in-reproduction.md) | 2026-09-17 | An animal that cannot afford a litter does not conceive. The reproduce energy gate (`energy - cost >= min_reproduce_energy`) and the transfer feasibility check are evaluated before any charge is ta… |
| [T16.F02 — Effort-Priced Actions](specs/roadmap/t16-f02-effort-priced-actions.md) | 2026-10-10 | Every act costs its effort, and failing costs nothing extra. Analog: a muscle burns energy whether or not the act succeeds (Polyworld charges activation, not outcome), and a courtship is spent whet… |

### T17 — Brain Boundary Evolvability

| Feature | Closed | Goal line (first sentence) |
| --- | --- | --- |
| [T17.F01 — Offspring Investment as a Fraction of Parent Energy](specs/roadmap/t17-f01-offspring-investment-fraction.md) | 2026-09-18 | A parent gives its young a share of what it has, not a fixed ration. The reproduce action's energy-transfer meta becomes a fraction in [0, 1] of the parent's post-cost energy (Polyworld's `MateEner… |
| [T17.F02 — Unit-Scale Introspection](specs/roadmap/t17-f02-unit-scale-introspection.md) | 2026-09-19 | A creature feels how full, how tired, and how old it is as a share of itself, not in joules or ticks. `EnergyCurrent` and `EnergyConsumedThisTick` reach the brain as fractions of `max_energy`; `Age… |

### T19 — Mesh Action Selection and Live State

| Feature | Closed | Goal line (first sentence) |
| --- | --- | --- |
| [T19.F01 — Per-Tick Hop Ramp](specs/roadmap/t19-f01-per-tick-hop-ramp.md) | 2026-09-21 | Sustained neural activity costs metabolism, so a creature that deliberates past a per-tick allowance of mesh hops pays a charge that rises with every further hop, summed over the tick, beside T03.F… |
| [T19.F02 — Live Internal State and Legal Cycles](specs/roadmap/t19-f02-live-internal-state-and-legal-cycles.md) | 2026-09-22 | A nervous system's dynamics run faster than its behavior and a circuit may reverberate, so a mesh node may run any number of times in a tick on its live state, a cycle ends at a survivable per-pass… |
| [T19.F03 — Vote Surface as Inert Data](specs/roadmap/t19-f03-vote-surface-as-inert-data.md) | 2026-09-22 | A motor pool is wired before it is ever driven, like a silent synapse, so the vote sinks (the four kinds, Terminate, and Decide), the `AddVote` opcode, the vote vector, and the per-kind counters ex… |
| [T19.F04 — Vote-Based Action Selection](specs/roadmap/t19-f04-vote-based-action-selection.md) | 2026-09-22 | Motor programs compete and the winner habituates, as in basal ganglia selection and competitive queuing of serial order, so a tick becomes a sequence of passes, each ended by the genome's `Decide`… |
| [T19.F05 — Decision-State Inputs](specs/roadmap/t19-f05-decision-state-inputs.md) | 2026-09-22 | Corollary discharge, a brain sensing what it just decided and how tired it is, so the current and previous pass's votes, the per-kind commit counts, hops this tick, and the previous tick's outcome… |
| [T19.F06 — Retirement and Observability](specs/roadmap/t19-f06-retirement-and-observability.md) | 2026-09-23 | Removal of a rule with no natural analog, so one execution model remains, the bank's last config field and readings are retired or marked historical, and the inspector shows passes. |

### T20 — Input Evolvability and Structured Variation

| Feature | Closed | Goal line (first sentence) |
| --- | --- | --- |
| [T20.F01 — Input-Use Baseline and Ecological Opportunity](specs/roadmap/t20-f01-input-use-baseline-and-ecological-opportunity.md) | 2026-09-25 | Measure where input use stops between declaration and retained behavior, and whether competent use of differently shaped inputs can improve reproduction in the existing worlds at native costs. |
| [T20.F02 — Graph Neutral-Connection Contract](specs/roadmap/t20-f02-graph-neutral-connection-contract.md) | 2026-09-26 | Initially silent afferents: a Graph node can acquire a refinable input connection without changing its incumbent signal or overwriting state, using existing edges wherever they suffice. |
| [T20.F04 — General Neutral Input Recruitment](specs/roadmap/t20-f04-general-neutral-input-recruitment.md) | 2026-09-26 | Afferent circuit growth: one bounded Graph mutation can declare an input and connect it neutrally to an eligible consumer while preserving existing coefficients and ordinary separate growth paths. |
| [T20.F05 — Structured Heritable Refinement](specs/roadmap/t20-f05-structured-heritable-refinement.md) | 2026-09-26 | Repeated circuit motifs: inherited variation can change homologous relationships across directions or repeated entity slots while independent mutations preserve local exceptions. |
| [T20.F16 — Born-Connected Input Declaration](specs/roadmap/t20-f16-born-connected-input-declaration.md) | 2026-10-08 | A new sensory input is expressed into an existing pathway rather than into nothing, as a receptor gene duplicated with its parent's expression enters the parent's cascade. When ordinary variation d… |

### T21 — Run Observability and Execution Tracing

| Feature | Closed | Goal line (first sentence) |
| --- | --- | --- |
| [T21.F01 — Local Telemetry Stack and Run Identity](specs/roadmap/t21-f01-local-telemetry-stack-and-run-identity.md) | 2026-09-29 | Measurement tooling. One command starts a local stack that keeps its data until the user retires it, and a run started with the telemetry flag appears in it with its identity, full config and start… |
| [T21.F02 — Run Metrics and Dashboards](specs/roadmap/t21-f02-run-metrics-and-dashboards.md) | 2026-09-30 | Measurement tooling. The cumulative counters a run already keeps are exported per run as complete counts and its per-tick values as sampled gauges, and provisioned dashboards list runs and show one… |
| [T21.F03 — Tick and Phase Tracing](specs/roadmap/t21-f03-tick-and-phase-tracing.md) | 2026-09-29 | Measurement tooling. A sampled tick becomes one short trace whose spans show how long each phase took and the config and world state in force, reachable from the run's dashboard. |
| [T21.F04 — Creature Cognition Windows](specs/roadmap/t21-f04-creature-cognition-windows.md) | 2026-09-30 | Measurement tooling. A selected creature is recorded for a short run of consecutive ticks, showing what it sensed, how its brain routed and voted, what it chose, what the world actually applied and… |
| [T21.F05 — Qualified Sampling Defaults](specs/roadmap/t21-f05-qualified-sampling-defaults.md) | 2026-09-30 | Measurement tooling. The cost of each level of detail is measured, the user sets the default's ceiling from it with 5% as the target, the default export cadences and sampling caps are chosen to sta… |
| [T21.F06 — Benchmark, Assay and Lab Run Records](specs/roadmap/t21-f06-benchmark-assay-and-lab-run-records.md) | 2026-09-30 | Measurement tooling. Benchmark, assay and lab commands appear in the stack with identity, lifecycle and end-of-run totals, held in memory and exported only after every timed region of the invocatio… |
| [T21.F07 — Telemetry Conventions in the Feature Contract](specs/roadmap/t21-f07-telemetry-conventions-in-the-feature-contract.md) | 2026-09-30 | Measurement tooling. The feature spec template's telemetry section names the delivered conventions, and a feature that adds a mechanism or counter ships its exported signals with it. |

### T22 — Capability Assays and Evolvability Lab

| Feature | Closed | Goal line (first sentence) |
| --- | --- | --- |
| [T22.F01 — Lab Harness and Food-Seeking Assay](specs/roadmap/t22-f01-lab-harness-and-food-seeking-assay.md) | 2026-09-28 | Measurement tooling. From one command, a small population of founders evolves on a fixed sparse-food arena under the production mutation engine and lab selection, and a seconds-scale report says wh… |
| [T22.F02 — Barrier-Navigation Assay](specs/roadmap/t22-f02-barrier-navigation-assay.md) | 2026-09-28 | Measurement tooling. The same harness scores whether lineages come to reach food behind barrier walls and rings without wasting moves, against an authored ring-inhibition comparator, the random-wal… |
| [T22.F03 — Brain and Sensor Change Readings](specs/roadmap/t22-f03-brain-and-sensor-change-readings.md) | 2026-09-29 | Measurement tooling. Every lab run records what changed in each generation's elite: genome size, reachable and executed nodes, operator counts since the founder, sensor use by family, steering, and… |
| [T22.F04 — Why-Not Diagnostic Ladder](specs/roadmap/t22-f04-why-not-diagnostic-ladder.md) | 2026-09-29 | Measurement tooling. One command takes a capability and a start and reports the first rung at which evolution stalls, exposure, supply, viability, benefit or retention, with the counts behind it, s… |

### T23 — Behavior and Cognition Readings

| Feature | Closed | Goal line (first sentence) |
| --- | --- | --- |
| [T23.F10 — Thinned Creature Window Store](specs/roadmap/t23-f10-thinned-creature-window-store.md) | 2026-10-01 | Measurement tooling. When a creature window closes, a compact record of it goes to Loki, and a scheduled job in the telemetry stack thins each run's records by age, keeping all of the newest 300, e… |

### T24 — Simulation Throughput

| Feature | Closed | Goal line (first sentence) |
| --- | --- | --- |
| [T24.F03 — Food Growth Phase Timing](specs/roadmap/t24-f03-food-growth-phase-timing.md) | 2026-10-06 | The microscope sees inside world update: food growth is timed apart from creature aging, energy decay and death removal in the phase metric and the sampled tick traces, each of its grid passes is a… |
| [T24.F01 — Order-Free Food Spread](specs/roadmap/t24-f01-order-free-food-spread.md) | 2026-10-07 | Seeds disperse evenly in every direction: food spreading from a dense cell keeps its share whichever neighbour it lands on, so a cell that already holds food no longer drops spread from its north a… |
| [T24.F02 — Parallel Food Regrowth](specs/roadmap/t24-f02-parallel-food-regrowth.md) | 2026-10-07 | Food regrows by the repaired rules on every core at once: `World::grow_food` runs on the rayon pool with its own seeded per-cell stream, so a goal run spends at most half as long per tick on regrow… |

## Open rows when archived

Listed by track for reference; none carries forward.

- **T01 — Experimental Science and Causal Evaluation**: T01.F01, T01.F02, T01.F03, T01.F04, T01.F05, T01.F06, T01.F07, T01.F08, T01.F09, T01.F10, T01.F13.
- **T02 — Environmental Dynamics**: T02.F01, T02.F02, T02.F03, T02.F05, T02.F06, T02.F07.
- **T03 — Functional Traits and Metabolism**: T03.F01, T03.F02, T03.F03, T03.F04, T03.F05, T03.F06, T03.F07, T03.F09.
- **T04 — Population Structure and Diversification**: T04.F01, T04.F03, T04.F04, T04.F05, T04.F06, T04.F07.
- **T05 — Biotic Interactions and Coevolution**: T05.F01, T05.F02, T05.F03, T05.F04, T05.F05, T05.F06, T05.F07.
- **T06 — Niche Construction and Ecological Inheritance**: T06.F01, T06.F02, T06.F03, T06.F04, T06.F05, T06.F06, T06.F07.
- **T07 — Communication and Social Evolution**: T07.F01, T07.F02, T07.F03, T07.F04, T07.F05, T07.F06, T07.F07.
- **T08 — Heredity and Recombination**: T08.F02, T08.F05, T08.F06, T08.F07, T08.F08, T08.F09.
- **T09 — Cognition and Learning**: T09.F01, T09.F03, T09.F04, T09.F05, T09.F06, T09.F07, T09.F08.
- **T10 — Evolutionary Scale and Experiment Infrastructure**: T10.F01, T10.F02, T10.F03, T10.F04, T10.F05, T10.F06, T10.F07, T10.F08.
- **T11 — Brain Genotype-Phenotype Map**: T11.F10, T11.F11, T11.F12, T11.F13.
- **T12 — World Composition and Baseline Worlds**: T12.F05.
- **T13 — Neutral Module Recruitment**: T13.F08, T13.F09, T13.F10.
- **T14 — Runtime Telemetry and Report Integrity**: T14.F09.
- **T16 — Cost and Cue Fidelity**: T16.F03.
- **T17 — Brain Boundary Evolvability**: T17.F03, T17.F04.
- **T18 — Founder Architecture**: T18.F01, T18.F02.
- **T19 — Mesh Action Selection and Live State**: T19.F07.
- **T20 — Input Evolvability and Structured Variation**: T20.F03, T20.F06, T20.F07, T20.F08, T20.F09, T20.F10, T20.F11, T20.F12, T20.F13, T20.F14, T20.F15.
- **T23 — Behavior and Cognition Readings**: T23.F01, T23.F02, T23.F03, T23.F04, T23.F05, T23.F06, T23.F07, T23.F08, T23.F09.
