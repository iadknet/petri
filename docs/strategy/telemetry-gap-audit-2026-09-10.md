# Telemetry Gap Audit — 2026-09-10

**Status**: Evidence base for [T14](../roadmaps/t14-runtime-telemetry-and-report-integrity.md)
**Machine-readable companion**: [telemetry-gap-audit-2026-09-10.results.json](telemetry-gap-audit-2026-09-10.results.json)

This note records what the simulator records, what it does not, and which
already-made claims cannot be re-read from stored telemetry. It is the evidence
base for the T14 track. It asserts no new mechanism and proposes no assay.

## 1. Provenance and method

Eight parallel finders were run over the telemetry surface, one per dimension:
demography, energy accounting, genome and brain structure, production mutation,
ecology and diversity, cognition, layer-transfer, and a top-down audit of stated
criteria. Each finder was required to verify absence with its own greps before
reporting. Every candidate was then put to three adversarial verifiers with
distinct lenses — is it actually absent, is it already owned by a planned
feature, does the program actually need it — with refutation as the default on
uncertainty. A candidate survived with fewer than two refutals of three.

Fifty candidates were produced. Thirty-one survived, six were refuted, and
thirteen never received verifiers: a session usage limit killed 47 verifier
agents and the synthesis agent mid-run. **The tiering and synthesis below are
the orchestrator's, not a verified agent's.** Every claim carries its status:

- **[3-lens]** — survived three adversarial verifiers.
- **[spot-checked]** — verifiers died; re-verified directly by the orchestrator
  against the source or the stored report, as cited.
- **[unverified]** — reported by a finder with its own evidence, never
  independently checked. Treat as a lead, not a finding.

Raw candidate records, verifier verdicts, and verifier corrections are in the
JSON companion. Where a verifier narrowed a finder's claim, the narrowed form is
what appears below.

## 2. The three telemetry layers

A gap in one layer is not a gap in another, and the distinction decides cost.

| Layer | Where | What it is |
| --- | --- | --- |
| (a) runtime | `SimStats`, `crates/v3-core/src/simulation/stats.rs:191` | what the simulation counts at all |
| (b) stored closure report | `crates/v3-cli/src/bench.rs` → `docs/progress/features/*.json` | what a closure preserves and `docs/progress.md` reads |
| (c) live per-tick | `last_tick_*` in `SimStats`, consumed by `v3-server` | what the running app shows, reset every tick |

Layer (b) is what the program's criteria are read against. A signal present in
(a) or (c) but absent from (b) is a real gap whose fix is a transfer, not a new
measurement.

## 3. Tier 1 — an already-made claim cannot be re-read

### 3.1 No death telemetry [3-lens]

`SimStats` has no death field of any kind. `Simulation::remove_creature`
(`crates/v3-core/src/simulation/simulation.rs:86-109`) is the single funnel every
death passes through, from both the Phase 0 bulk removal and the Phase 2
`remove_creature_if_dead` (`crates/v3-core/src/simulation/tick/helpers.rs:9-15`,
`crates/v3-core/src/simulation/tick.rs:184-189`), and it increments no counter.
Nothing records which energy sink took a creature to zero.

The consequence is not abstract. T03.F10 made runaway computation lethal, and its
stated mechanism is a causal chain: ramp charge → energy exhaustion → death
removes the looper. Its closure measured the first link only — the goal profile's
`vm_steps` was predeclared down
(`docs/specs/roadmap/t03-f10-activity-ramped-compute-cost.md:350-357`) — and read
population alongside it. `vm_steps` falling is consistent with loopers dying, and
equally consistent with loopers being selected away before they loop, or with the
charge binding somewhere the feature did not intend.

One partial exception, which a verifier supplied: predation deaths *are* counted
at runtime (`predation_kills_total`, `stats.rs:303`, incremented at
`crates/v3-core/src/simulation/actions/predation.rs:128`). That counter reaches
`v3-server` and neither report half, so it is a layer-transfer gap rather than a
runtime one.

Lifespan is likewise captured and discarded: `survival_ticks: creature.age` is
read at every death (`simulation.rs:150`), but only into
`MutationOutcomeObservation`, and only when the creature carries at least one
birth mutation operator — roughly the mutated minority, not the population.

### 3.2 The quantity T03.F08 charges is in no report [3-lens, spot-checked]

The Phase 0 charge is `genome_carry_cost_per_unit * cached_genome_size`
(`crates/v3-core/src/simulation/tick.rs:162-166`). The string `genome_size`
appears zero times in every stored closure report; only
`reachable_structure_size_distribution` is stored, and the carrying cost does not
target reachable structure. The feature's own predeclared criterion had to be
restated after implementation because the charged quantity could not be read
(`docs/specs/roadmap/t03-f08-genome-size-maintenance-cost.md:195-199`).

Spot-checked: the accessor already exists. `structure_means`
(`crates/v3-cli/src/lib.rs:165-185`) returns mean genome size, mean mesh nodes and
mean generation over the living population, and is already called by
`build_tick_sample` on the `v3-cli run` streaming path. `bench.rs` never calls
it. This is a one-call transfer, not a measurement to build.

A verifier narrowed the finder's stronger claim: the junk fraction is not wholly
unreadable, since drift-depth and neighborhood blocks carry structural readings
on sampled genomes. What is unreadable is total carried structure over the
*living population* at any closure.

### 3.3 The comparison chain can reference the report itself [spot-checked]

Verified directly against `docs/progress/features/t12-f04-baseline-world-set-goal.json`:
`comparison.references[0].path` is that report's own filename. All 32 readings
compare a value to itself — `final_population` reads current 8818, reference
8818, delta 0 — so that closure's no-regression verdict is vacuous.

The chain is not systemically broken: T13.F01's goal report correctly references
T12.F04, the previous closure. This is a series-boundary defect. It matters
because two Final Success Criteria — that indicators rise across the series, and
that no closure lowers a component below the previous closure's reading — are
read by walking exactly this chain, and a self-reference is indistinguishable
from a clean pass.

### 3.4 A wired cognition indicator is exempt from the no-regression rule [spot-checked]

Verified directly: the 32 comparison reading names in T13.F01's goal report
include `memory_different_from_either_fraction` and no `temporal_memory_*` field.
`temporal_memory_sensitivity` is wired into the report and omitted from the
comparison block, so the no-regression rule cannot see it. A closure could lower
all three of its substrates — `previous_slots`, `operator_state`,
`persisted_outputs` — with no stored signal.

Related and weaker: `lineage_diversity`, `memory_sensitivity` and
`reachable_structure_size_distribution` carry no version or definition token,
while `temporal_memory_sensitivity` carries `version: temporal-memory-v1` and
`drift_depth` carries `drift-depth-v3` **[unverified]**. The no-regression rule
compares a component against the previous closure; if a definition moved between
them, nothing stored says so.

## 4. Tier 2 — a stored indicator is ambiguous without it

### 4.1 No energy flow decomposition [3-lens]

`mean_energy` is a stock. None of the flows is counted:

- **Intake.** `apply_typed_eat` (`crates/v3-core/src/simulation/actions/mod.rs:71-82`)
  credits energy and records only the event count, never units consumed or energy
  credited. A verifier narrowed the finder's "eating is the world's only inflow":
  predation awards a created `kill_complexity_bonus`, non-default.
- **Action charges.** Six charge sites, all scaled by a complexity × age
  multiplier, none summed. The one published claim about action cost is a nominal
  rate times an event count that ignores the multiplier
  (`docs/strategy/mesh-depth-research-2026-09-07.md:66`).
- **Compute charge.** `TickComputeStats` accumulates VM, graph and priority-bid
  energy every tick and keeps only per-tick means. The program's one realized
  cognition-cost reading came from a hand-run server endpoint, not from telemetry
  (`docs/strategy/mesh-depth-research-2026-09-07.md:68`).
- **Parental investment.** The energy a parent hands each offspring is
  creature-chosen and never summed **[unverified]**.

Together these mean no cost in the simulator can be stated as a share of what the
population earns, which is how every cost feature wants to be judged, and
`mean_energy` cannot distinguish a rich population eating little from a poor one
eating constantly.

### 4.2 Lineage diversity is a terminal scalar pair [3-lens]

`surviving_founder_clade_count` and `shannon_entropy_nats` are computed once after
the run ends. `PersistenceSample` carries 20 checkpoints with population, mean
energy and births, and no lineage field. The clade distribution itself is
discarded: `bench.rs:1979-2003` builds the counts map and returns only its length
and the entropy.

Roughly 98.6% of founder clades are lost over a goal run with no recorded timing.
If that loss lands in the tick-150 peak-and-crash, the indicator is reading the
founding lottery rather than anything a closed feature changed — and the first
Final Success Criterion is that this indicator rises across closures.

### 4.3 Generation depth is one terminal point [3-lens]

A correction to the surface this audit began with: generation depth *is*
population-wide, not battery-sampled — `bench.rs:2230-2233` builds it from
`sim.creatures.values().map(|c| c.generation)`. What is stored is `{median, max}`
at the final tick, currently about 49/60. There is no minimum, no quartiles and
no trajectory, so whether depth is still advancing at the horizon or has stalled
cannot be read, and ticks-per-generation within a run — the demographic signature
of a population leaving its boom phase — is invisible.

The "22 generations" figure in `docs/roadmap.md` is a dated historical reading and
stands as such; a new spec should cite the current terminal depth instead.

### 4.4 Plasticity counts visits, not changes [3-lens]

Both update loops increment unconditionally
(`crates/v3-core/src/runtime/plasticity/hebbian.rs:144-146`,
`crates/v3-core/src/runtime/plasticity/reward.rs:77-80`) with no comparison of old
weight to new. An update that writes an unchanged weight — a saturated edge at the
clamp boundary, or a zero outcome signal — is indistinguishable from one that
moves a weight. Reward-modulated updates are summed into the same counter, so that
pathway has no separate reading at all.

The roadmap states that plasticity updates are not cognition by themselves and are
supporting structural observations only. Today they cannot even distinguish live
learning from churn.

### 4.5 Memory sensitivity has no exposure denominator [3-lens]

`memory_sensitivity` divides by `final_creature_count`, every living creature,
including those whose reachable structure cannot read the intervened substrate at
all. The carrier census already exists and is pure and RNG-free
(`structural_companions`, `crates/v3-core/src/neighborhood/`). A verifier confirmed
the sample-size complaint is not the issue — count and denominator are both stored,
so an interval is computable downstream — and that the real gap is exposure.

## 5. Tier 3 — counted already, stored nowhere

Each of these exists in layer (a) or (c) and reaches no report. The fix is a
transfer in `bench.rs`.

| Signal | Status | Note |
| --- | --- | --- |
| Production mutation counters: applied-by-operator, and the executed / reachable / unreachable / not-applicable target split | [3-lens] | T11.F17's own delivery instrument. Every mutation number in the stored report comes from an offline battery or an unselected drift walk, never from the production run. |
| Mutation lifecycle outcomes on real carriers: `survival_ticks_sum`, `offspring_spawned_sum`, `reproduced_once_total`, survival horizons | [3-lens] | Recorded at every qualifying death across ~1.16M births per goal run and read by no consumer. Exclude `final_energy_sum` — see §6. |
| Predation counters | [3-lens] | Reach `v3-server`, neither report half. |
| Failed eats (`ActionResult::NoFood`) | [3-lens] | Blocked moves are instrumented exhaustively by cause; the eating analogue has no counter. Relevant to the food-type niches the baseline worlds exist to create. |
| Dispatches ending in `TerminationReason::EnergyExhausted` | [3-lens] | The applied consequence of every compute cost: the queue is discarded and the creature loses its turn. Production discards the reason. |
| Reproduction outcome funnel, including population-cap saturation | [unverified] | Present in `SimStats`, absent from the report. |
| Per-creature perception branch | [unverified] | The sim branches perception per creature per tick on referenced world inputs; the branch is uncounted. |

## 6. Defects, not gaps

Both were found during the audit and are recorded here so the transfers above do
not propagate them. Neither is fixed by this note.

- **`mutation_events_applied_total_semantic_noop` is provably always zero.**
  `MutationOperator::semantic_category` (`crates/v3-core/src/mutation/types/mod.rs:321-324`)
  has no match arm and returns `SemanticChange` unconditionally, so the noop
  counter never increments and `semantic_change` always equals
  `mutation_events_applied_total`. The one runtime counter that reads like a
  production silence measure is a constant, and it is exposed over the server
  payload. **[3-lens]**
- **`final_energy_sum` is structurally always zero.** Spot-checked: it is built as
  `f64::from(creature.energy.max(0.0))` (`simulation.rs:152`) and only at death,
  when energy is at or below zero. Transferring `MutationValueTotals` wholesale
  would add a permanent zero column to every future report and to
  `docs/progress.md`. **[spot-checked]**

## 7. Refuted — considered and rejected

| Candidate | Why it failed |
| --- | --- |
| Per-lineage behavioral telemetry | `WorldTracking::observe` sources every field from world totals by deliberate design; the proposal was a re-architecture, not a gap. |
| Predation kills as a program-level need | No criterion cites predation; the finder itself cited NONE. |
| Production zero-event birth fraction | Already computed and stored in the closure report. |
| Mesh node creation by backend | Already stored; T11.F18 wired it. |
| Age at death over the whole population | No criterion; overlaps §3.1, which carries the criterion. |
| Mutation cohort vital rates as a Tier 1 item | Real but criterion-free; retained in §5 as a transfer. |

## 8. Determinism constraint on every proposal here

Seeded runs must reproduce byte-for-byte across processes and thread counts at
every world size the goal profile uses — a Final Success Criterion, and the
subject of the T10.F11 repair. The simulator runs multi-threaded, so any float
accumulation added by this track must reduce in a fixed order or it breaks that
criterion.

The rule this track adopts: prefer integer counters; accumulate floats only
inside the tick's existing deterministic reduction, which already documents
queue-order summation (`crates/v3-core/src/simulation/tick.rs:541-543`). The
genome-size proposal is the model — a `u64` incremented by `cached_genome_size`
inside the existing Phase 0 loop is order-independent by construction.

The standing observation contract from T13.F01 also applies unchanged: additive
observation must not consume production RNG, select survivors, or change
execution; and an unobserved field is unmeasured, never zero.

## 9. What becomes a track

The measures above split cleanly by whether a mechanism owns them.

Telemetry introduced *by* a mechanism feature stays with that feature — the
roadmap already requires that any feature introducing a diversity or cognition
measure wires its indicator into the goal profile as part of the same feature.
What has no home today is the residue: recording that spans several features, and
recording whose need surfaced after its owning feature closed. Sections 3 through
6 are almost entirely that residue — T03.F10 and T03.F08 are closed, and the
report-integrity defects belong to no mechanism at all.

That is the durable area T14 owns. It records what the simulation already does.
It adds no mechanism, no assay, no campaign apparatus, and no dashboard,
database or composite score.

## 10. Beyond repair: what the north star needs that nothing measures

Added 2026-09-11 at the user's direction. Sections 3–6 make existing readings
trustworthy; they do not add readings the success definition needs. The stored
indicators measure **presence, not payoff**. Clade count and entropy say which
identities survived, not whether they live differently. Memory sensitivity says
behavior changes when shared memory is scrambled, not whether the memory-using
behavior helps its carrier. The success definition names four things no stored
reading touches:

| North-star phrase | What it needs | Reading | T14 row |
| --- | --- | --- | --- |
| "many … ways of making a living" | differentiation | per-surviving-clade profile: eats by type, actions by type, size, energy, age, generation, genome size | F07 |
| "coexisting" | duration | per-clade peak and extinction tick — the loss curve, not the terminal count | F09 |
| "coexisting" (spatially) | partitioning | coarse-grid population and clade count per checkpoint on the T12 worlds | F10 |
| "rather than reacting alone" | selection for cognition | offspring and lifespan by cognitive class at death, classes from `structural_companions` | F06 |
| all of the above | perceptual prerequisite | living-population census of referenced `WorldInputKey`s and stateful sources, from `cached_live_vm_world_inputs` | F08 |
| "predicting" | a cycle to be ahead of | phase relationship between clade intake and the food cycle — **ships with T02.F01**, not here | — |

What makes each cheap is already on the creature: `structural_companions` is
computed per creature and RNG-free; `cached_live_vm_world_inputs` lists every
world input a brain references; `offspring_spawned_count`, `age`, `generation`
and `cached_genome_size` are per-creature fields; every death passes one funnel.
F07 needs two new per-creature lifetime counters (eats by type, actions by type)
beside the lifetime counters that already exist. Everything else is a bucket or
a sum over fields that are already there.

Two boundaries, stated so the rows stay small. The audit refuted per-lineage
*cumulative* telemetry inside `WorldTracking` because that struct sources world
totals by design; F07's terminal table bounded by surviving clades (22–142 today)
is a different artifact and leaves that design alone. And founder `lineage_id`
is clade identity, not ecotype — two clades can converge on one way of living —
so F07 shows what each clade did and names no strategy. Descriptors, strategy
counts and overlap measures derived from these rows remain T01.F04 and T01.F06.
