# T22.F04 — Why-Not Diagnostic Ladder

**Status**: In Progress
**Last updated**: 2026-09-29
**Feature**: T22.F04
**Track**: [T22 — Capability Assays and Evolvability Lab](../../roadmaps/t22-capability-assays-and-evolvability-lab.md)

## Goal

Measurement tooling. `v3-lab why-not --assay <capability> [--genome
start.json] [run flags]` calibrates and runs the campaign exactly as `run`
does and prints the ladder: for the
reference arm, and labelled for every other evolving arm, the five rungs
exposure, supply, viability, benefit and retention, each `pass`, `fail` or
`inconclusive` with the counts behind it, then one verdict line naming the
first rung that is not `pass` and the track that owns it. The NDJSON rows
and the summary carry the ladder's bookkeeping; `v3-lab report` renders the
same ladder from the summary alone. Scores, selection, calibration and every
existing RNG stream are untouched: at T22.F03's settings the seed-1 quick
runs reproduce its calibration, reach, fidelity and readings blocks exactly,
and the ladder draws no random number.

## Non-Goals

- No reading beyond the ladder's bookkeeping: no read-level executed use, no
  action trace, no per-tick record, no new battery.
- No change to scoring, selection, calibration, the mutant signature or the
  reach test; no `v3-core` change (a knob the lab needs and core lacks is a
  recorded finding for the owning track).
- No automatic routing: the report names the owning track; a research note
  carries the finding there.
- No ladder for scripted or fixed arms (no births). A `policy-deviation`
  arm's ladder is printed under a `diagnostic` label, never as the
  capability's verdict.

## Inputs and Invariants

**Contract.** The T22.F04 row and the track's "Why-not ladder (F04)" note
are the source of truth; F01's spec fixes the harness, F03's the readings;
this spec is a delta on them. Diff scope (lab exemption): `crates/v3-lab/**`,
this spec, the readings file, the track checkbox and the committed lab
summaries under `docs/progress/lab/` this feature regenerates.
`Cargo.toml`, `Cargo.lock` and the `Makefile` are unchanged (`make lab
LAB_ARGS="why-not …"` already reaches the new subcommand), and
`crates/v3-lab/Cargo.toml` adds no dependency and changes no existing
dependency version, feature or shared build setting; a diff outside this
scope loses the exemption.

**Design evidence (2026-09-29).**

| Decision | Options | Choice and reason |
| --- | --- | --- |
| Where the ladder runs | (a) a `why-not` subcommand sharing `RunArgs`, bookkeeping always on in the campaign so `run` and `why-not` write the same rows and summary; (b) a post-processor over an existing run directory; (c) a separate campaign loop | (a): the rows do not carry per-event targets or change signatures, so (b) cannot read supply or retention; (c) duplicates the loop the track forbids changing casually |
| Proposals read for viability and benefit | (a) the population's non-identical children against their carried parent on the same scenes; (b) F03's fresh mutants of the elite | (a): every child's parent is a carried survivor scored on the same generation's scenes, so scalar deltas are same-scene, the sample is every proposal selection saw, and a child that improves and survives is the very object retention follows; (b) stays F03's secondary signature, untouched |
| Viability class | `neighborhood::classify` on the arm's `Batteries` (F03 seam) plus F03's scene-difference rule | Both existing; the battery answers `dead`, the scenes catch a change the 80 executions miss |
| Retention identity | (a) node-id change signature: the child's nodes that differ from or are absent in the parent, the parent's nodes absent in the child, and `entry_node_id` when moved, each with its content; (b) score above the start each generation; (c) re-scoring the pre-change parent | (a): `NodeGenome: PartialEq`, O(nodes), names deletion structurally like `recruitment_paths::classify_retention`'s `Deleted`; ids are not fresh (`next_node_id` allocates max + 1, so a removed highest id is reused), hence the predicate compares `(id, content)` pairs and never an id alone; (b) confounds prior gains; (c) is a new evaluation per tracked change per generation |
| Evidence threshold | (a) a descriptive rate against a predeclared stall rate with a rule-of-three adequacy minimum (`n ≥ ⌈3 / ρ⌉`) within a replicate, and `stats::wilson_95` only over replicate outcomes; (b) Wilson bounds on births and children within a replicate | (a): individuals and identical offspring are never interval samples (F01), the replicate stays the sole independent unit, and the adequacy minimum is the count at which zero successes bound the rate below `ρ` |

**Seams verified in code (2026-09-29).**

| Need | Seam |
| --- | --- |
| Per-event target | `MutationSummary::events: Vec<MutationEventRecord>` — `operator: Option<MutationOperator>` (`None` when no operator was accepted), `target: Option<NodeId>` (the first node the operator picked, recorded for skipped events too), `outcome: Applied(TargetReachability) \| Skipped(MutationSkipReason)`, `discarded: Vec<(MutationOperator, Option<NodeId>)>` |
| Relevant sites | `readings::consumers(genome)` (node indices by `Family`), `nodes[i].node_id`, `mesh_reachable_nodes`; graph `OutputSink { kind: ActionVote(VoteSink::Eat \| Move(_)), inputs }` on `CgpGraphBackendDef::output_sinks` — every graph node holds the full fixed sink catalog with empty `inputs` (`new_with_fixed_outputs`), so only a sink with `inputs` non-empty votes; VM `VmInstruction::AddVote { sink, .. }` with `VoteSink::from_index(sink)` `Eat` or `Move(_)` |
| Provenance | `run.rs::genome_records` names genomes `start` and `arm:<name>`; `summary::GenomeRecord` has `name`, SHA-256, `genome_format`, `v3_core_version` and no origin |
| Class and scene difference | `Batteries` per arm, `readings::signature`, `classify(&parent, &child).class`; `Scored::{scenes, sequences}` of every member are kept until ranking (F03) |
| Ancestry | `Individual::{id, parent, birth, carried, ancestry}`; `Ancestry::{births, applied}` sum along the path (`readings.rs:82`: every applied event counts, identity aside); `breed` has the `MutationSummary` of every birth |
| Survivors | `advance` ranks by the true order and draws the tie keys; `breed` carries `order[..survivors]`, drawing the `shuffled-score` permutation only when a next generation is bred (not at reach, the last generation or a refused row) |
| Interval | `stats::wilson_95(successes, trials) -> Option<[f64; 2]>` (successes ≤ trials) |
| No campaign | `run.rs:449–482`: an uncalibrated or `--calibrate-only` run writes a summary with empty `arms` |

**Relevant sites.** A capability names its relevant families; the relevant
sites of a genome are its reachable nodes that consume one of them
(`sensor`) or that vote `Eat` or `Move(_)` through a graph sink with at
least one input edge or a VM `AddVote` (`motor`), keyed by node id; a node
whose vote sinks are all unconnected is not a motor site.

| Assay | Relevant families |
| --- | --- |
| `food-seeking` | `FoodHere(*)`, `NeighborFoodRing(*)`, `AreaFoodSummary(*)` |
| `barrier-navigation` | the food-seeking set plus `NeighborBarrierRing`, `AreaBarrierSummary` |

**Bookkeeping per birth** (`breed`, from the birth's `MutationSummary`).
`targeted` events are those whose `target` is a relevant site of the parent
(`target: None` never targets): requested, applied and skipped, each by
operator (an event with `operator: None` under the key `none`), skipped
also by skip reason; `discarded` picks on a relevant site are counted apart
and enter no denominator. `created` and `removed` are the relevant sites
present in the child and not the parent, and the reverse, by node id — this
is how a start with no relevant site, or a swap or retarget on an existing
node, touches the capability. A birth is *touching* when its targeted
applied count, `created` or `removed` is ≥ 1; a child is touching when its
birth is. The birth also records the parent's relevant-site and reachable
counts and its change signature (design table).

**Per-generation bookkeeping** (`advance`, after ranking, before the row):
for every non-identical child `c` with carried parent `p` in the same
generation, `scene_changed` (any `SceneScore` field or the per-tick
`(position, energy)` sequence differs from `p`'s on a training scene, F03's
rule), `improved` (`scalar_c > scalar_p`), `progress_improved` (mean
`progress` over the scenes strictly greater) and `delta = scalar_c −
scalar_p`. Touching children are also classed on the battery against their
parent's signature (one signature per parent, computed once); `viable` =
class ≠ `dead` and (class = `changed` or `scene_changed`). Non-touching
children carry the scene fields only, as context.

**Retention.** The *survivors* of a generation are the order `breed`
carries, computed before the row from the same draws in the same order (the
permutation still drawn only when a next generation is bred); a generation
with no breeding has no survivors. A *selected improvement* is a
non-identical child with `improved` that is a survivor of its birth
generation. Its *descendants* are found by walking parent identity; a
descendant's *depth* is the applied events on the path from the change
(`Ancestry::applied` difference; applied events that cancel still count,
as F01 keeps identity distinct from zero applied events); a carried elite
adds none and is never evidence. A descendant *carries* the change when
every changed or added `(id, content)` pair is present, no removed `(id,
content)` pair is present (an unrelated node reusing the id is not the
removed node) and `entry_node_id` matches when the change moved it. After
ranking in every later generation, per depth `d ∈ 1..=D` still open: a
survivor descended from the change at depth ≥ `d` carrying it resolves
`retained`; no member of the evaluated population descended from the change
(its own carried copy included) resolves `lineage_loss`; descendants
present but none carrying the change resolves `deleted`; otherwise the
depth stays open (a non-carrying branch at depth ≥ `d` beside a carrying
branch below `d` decides nothing). An open depth at the arm's stop
(`horizon`, `reached`, `byte_cap`, or a generation without survivors) is
`censored`; a generation without survivors can still resolve `deleted` or
`lineage_loss` from the evaluated population, never `retained`. The rung
reads depth `D`; at `--quick` sizes it is expected `inconclusive` (4–6
selected improvements per replicate against the minimum of 15), and
campaign sizes are where it reads. A later applied event on a changed
node reads as `deleted`; the depth-1 counts show whether re-targeting
dominates.

**Rungs.** Every rung's counts are computed and printed; a replicate's
verdict is its first rung whose status is not `pass`.

| Rung | Predicate (`s` / `n`) | `ρ` | Route on `fail` |
| --- | --- | --- | --- |
| exposure | calibration verdict `calibrated` (`pass`/`fail` only; `--calibrate-only` stops here with `no campaign`) | — | the assay (T22): instrument |
| supply | touching births / births; context: targeted requested/applied/skipped by operator, skipped by reason, `discarded`, `created`, `removed`, mean relevant sites and reachable, and `uniform_reference = Σ applied_b × sites_b / reachable_b` (a uniform-targeting reference, not the native expectation — targeting is executed-biased; `null` at zero reachable) | 0.01 | T11 / T13 |
| viability | `viable` / touching non-identical children; context: `silent`, `changed`, `dead`, `scene_changed` | 0.05 | T11 / T17 |
| benefit | `improved` viable touching children / viable touching children; context: `progress_improved`, `worse`, `delta_mean`, `delta_max` over improved, the non-touching `improved` fraction | 0.05 | the scorer (T22) or, in the world, the pressure tracks |
| retention | `retained` / (`retained` + `deleted` + `lineage_loss`) at depth `D`, all selected improvements; context: the touching subset, `censored`, every depth `1..=D` | 0.20 | T13 / T14 |

Status within a replicate from `s`, `n` and the rung's `ρ`: `inconclusive`
when `n < ⌈3 / ρ⌉` (300, 60, 60 and 15 trials at the defaults — the count
at which zero successes bound the rate below `ρ`); otherwise `pass` when
`s / n ≥ ρ` and `fail` when `s / n < ρ`. Births and children are counted,
never interval samples; the stall rates are predeclared judgment values,
recorded in `provenance.sizes` and overridable by `--stall-rates`. Per arm
and rung, the report prints the replicate-status tally `pass a / fail b /
inconclusive c` over completed replicates, with `wilson_95(b, a + b + c)`
on the `fail` share (the replicate is the interval sample, as in F01). The
arm verdict: with no completed replicate, `inconclusive, partial` and no
stall claim; otherwise the verdict rung is the modal first non-`pass` rung
over the completed replicates that have one (a replicate passing all five
casts no vote; ties to the earliest rung), and among the
replicates naming it the verdict reads `stalls at <rung>` only when more
than half are `fail`, else `inconclusive at <rung>`, always with the tally
(`fail b / inconclusive c` of those replicates, e.g. `stalls at benefit:
fail 3 / inconclusive 1 of 4`); `no stall` only when every completed
replicate passes all five (then `reached k/n`, censored at the horizon
when `k = 0`). Pooled counts and fractions over completed
replicates are printed as description without an interval; an incomplete
replicate is excluded and labels the verdict `partial (m incomplete)`.

**Determinism.** The ladder draws no RNG: touching is read from the
recorded events, classes from the deterministic battery, differences from
the sequences already kept, retention from ids and node equality. Counts
fold in index order; same-seed rows stay byte-identical across thread counts
and processes. Bookkeeping is transactional with the row: a refused row
records no counts and no resolution, and its open changes are censored.

**Outputs.**

| Artifact | Content |
| --- | --- |
| `row_version: 3` | `ladder` beside `readings` (null for scripted and fixed arms): `supply` (`births`, `touching_births`, `sites`, `reachable`, `uniform_reference`, `targeted` requested/applied/skipped by operator and skipped by reason, `discarded`, `created`, `removed`); `children` (`touching` and `other` blocks with the per-generation counts above; classes in `touching` only); `retention` (`selected`, `selected_touching`, per depth `[retained, deleted, lineage_loss]` resolved in this generation, and `depths_touching` for the touching subset) |
| `summary_version: 4` | Top-level `ladder`: `exposure` from the calibration block alone (verdict, the selected point or the failing checks with their counts: exposure scenes, comparator wins, sensitivity gaps), so an uncalibrated or `--calibrate-only` summary still renders the instrument finding; per evolving arm, per replicate (pooled counts, `censored` and `censored_touching` per depth, per-rung status, verdict) and per arm (per-rung tallies with the `fail` interval, each replicate's status, the tally verdict, pooled `s / n` and context counts, `incomplete_replicates`); `uniform_reference` is null for a row and for the replicate once any parent in it has zero reachable nodes |
| Provenance | `sizes` gains `stall_rates`, `retention_depth`, `relevant_families`; each `genomes` entry gains `source` (`founder`, `file` or `builtin-comparator`) and a file's basename, so the start label is reproducible from the summary |
| `report` | Renders v2–v4 (v1 stays refused); from a v4 summary the ladder section: exposure, then per evolving arm one line per rung with status, counts and tally, and the verdict line (assay, arena, start — `founder` or the file's basename, each with its SHA-256 — arm, role, policy, verdict, route) |
| `why-not` | Prints the ladder section and the run directory; exits as `run` does (0; 2 uncalibrated; 3 byte cap: a stall is not an error). `run` prints the report as before |
| Committed summaries | `docs/progress/lab/t22-f02-*.json` regenerated at v4 |
| Sizes | By-operator maps hold only the keys that fired: `ladder` ≈ 1 KB per row in the scenario of ≤ 5 touching events per generation, ≤ 6 KB at the bound of all 55 `MutationOperator::all()` keys in three maps plus three skip reasons and `D ≤ 8`; rows ≤ 12 KB / ≤ 17 KB at population 64; campaign ≈ 29 MB / ≤ 41 MB under the 64 MiB cap; summary: the pooled maps at the bound (three maps × 55 names plus reasons and tallies) are ≤ 8 KB per evolving arm and replicate and ≤ 8 KB per arm, so a campaign summary (3 evolving arms × 8 replicates) grows ≤ 220 KB over F03's ≤ 200 KB, inside the 1 MiB reserve whose exit-1 oversize disposition is unchanged |

| CLI parameter (`run` and `why-not`) | Default |
| --- | --- |
| `--retention-depth` | 2 (1..=8): applied events after the change |
| `--stall-rates supply,viability,benefit,retention` | 0.01,0.05,0.05,0.20, each in (0, 1) |
| `--mutants` under `--quick` (F03) | 8; the pilot lowers it to 4, then 2, then `--signature-arms native`, only if the quick food-seeking run exceeds 60 s, and records the value here |

**Cost.** Touching share of non-identical births: assumed 20–40 %, measured
85 % (food-seeking) and 84 % (wall-v1) from the founder start, whose nodes
are all relevant sites, so the touching filter separates little for that
start and more for a larger `--genome`; the measured ladder cost (≈ 0.4 s
per quick run) is in the Timing item.

| Item | Quick (population 16, 4 survivors) | Campaign (64, 16 survivors) |
| --- | --- | --- |
| Battery signatures per arm-generation (80 executions each): touching non-identical children plus their distinct parents | 12 slots × ≈ 0.35 × ≤ 0.4 ≈ 2 children + ≤ 2 parents; bound (12 + 4) × 80 | ≈ 7 + ≤ 7; bound (48 + 16) × 80 |
| Creature-ticks per arm-generation for comparison | 12,800 | 51,200 |
| Node comparisons | O(nodes) per birth and per open change per descendant; open changes ≤ survivors × generations; descendant walk ≤ population × path length | same |
| Projected addition to the quick food-seeking run | under 1 s against T22.F03's 59.0 s pilot | — |

## Implementation Tasks

- [x] Relevant sites and touching: per-assay family set, sensor and motor
      sites of a genome (connected graph sinks and VM `AddVote`), targeted
      / discarded / created / removed classification of a birth; founder,
      zero-site-start and unconnected-sink fixtures.
- [x] Birth bookkeeping in `breed`: touching counts, site and reachable
      counts, change signature on the individual; carry-test fixtures
      including a removed highest id reused by an unrelated node.
- [x] Generation bookkeeping in `advance`: survivors computed before the
      row from the same draws; scene difference, improvement and deltas for
      every non-identical child; battery class and `viable` for touching
      ones.
- [x] Retention: selected improvements, descendant walk, depth, resolution
      per depth, censoring at stop, refused row and no-survivor generation.
- [x] Row v3 and summary v4 blocks (exposure independent of arms),
      per-replicate statuses, arm tally verdict, `report` rendering, the
      `why-not` subcommand, `provenance.sizes`.
- [x] Fixtures that separate adjacent rungs: synthetic counts placing the
      stall at each rung and at `inconclusive`; an uncalibrated run
      stopping at exposure with a rendered exposure block; a `mutation-off`
      arm stalling at supply under a raised supply rate whose adequacy
      minimum the fixture's births meet; all-`inconclusive` and mixed
      `fail`/`inconclusive` replicate tallies; retention lineages
      resolving `retained`, `deleted`, `lineage_loss` and `censored`,
      including the mixed carrying/non-carrying sibling case.
- [ ] Regenerate the three committed summaries at v4 from a clean tree
      after review remediation, as F03 did (provenance `dirty: false`);
      pilot the quick run time and the touching share; record both in the
      readings file.

## Verification

- [x] Focused tests: `cargo test -p v3-lab` -> 173 unit and 19
      integration tests pass (1 ignored child helper); names in
      [`docs/progress/readings/t22-f04.md`](../../progress/readings/t22-f04.md).
- [x] Determinism: the same-seed byte-identity test (1 thread in-process, 2
      threads in a child) passes on v3 rows -> readings file.
- [x] Compatibility: at T22.F03's settings (`--quick --mutants 8
      --signature-arms changing`, seed 1) the food-seeking, wall-v1 and
      ring-v1 runs reproduce the committed v3 summaries' calibration,
      reach, fidelity and readings blocks exactly -> readings file.
- [x] Timing: measured 2026-09-29 on a host running other work: the
      unchanged 02aa7851 reference binary 63.5–72.4 s (T22.F03 recorded
      57.5–59.0 s); paired runs this build vs reference 65.8/65.3,
      70.5/68.6, 62.8/63.5 s, so the ladder adds −0.7 to +1.9 s
      (instrumented ≈ 0.4 s), within the predeclaration. No fallback
      ships and `QUICK_MUTANTS` stays 8; the 60 s bound is waived for this
      closure by the Exception below. Fallback data (`--mutants` 4:
      67.7 s, 2: 62.6 s, 2 + `native`: 54.3 s) -> readings file.
- [ ] Quick runs (seed 1, food-seeking, wall-v1, ring-v1): ladder verdicts
      and the regenerated `docs/progress/lab/t22-f02-*.json` at v4 ->
      readings file.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      and every survivor resolved as killed, equivalent, or deferred.
- [ ] Benchmark summary: `Not applicable: lab feature` (see below).

## Performance and Goal Impact

**Predeclaration — written before the run.** Not applicable: lab feature.
The diff is confined to `crates/v3-lab/**`, this spec, the readings file,
the track checkbox and the regenerated committed lab summaries; `Cargo.toml`,
`Cargo.lock`, the `Makefile`, `v3-core`, `v3-cli`, `v3-server` and the
frontend are unchanged, and no existing dependency version, feature or
shared build setting changes anywhere, including `crates/v3-lab/Cargo.toml`;
a diff beyond that loses the exemption and the profiles run. No trajectory,
default, founder, recipe or mutation policy changes, so no profile runs and
no series entry. The natural-analog and environmental-pressure rules do not
apply. Lab cost is in the Cost paragraph above.

**Measured verdict.** Not applicable: lab feature; scope checked in review.

- Full readings: [`docs/progress/readings/t22-f04.md`](../../progress/readings/t22-f04.md).

## Success Criteria

- [ ] `v3-lab why-not --assay food-seeking --quick` and the same on
      `barrier-navigation` print the exposure block, the rungs with counts
      per evolving arm and one verdict line per arm naming the first
      non-`pass` rung, its replicate tally and its route.
- [ ] Each rung has an observable predicate, a conditional denominator, a
      predeclared stall rate and an `inconclusive` disposition; fixtures
      place the stall at each rung.
- [ ] Retention is read over descendant lineages at declared applied-event
      depths, never over a carried elite, and distinguishes `retained`,
      `deleted`, `lineage_loss` and `censored`.
- [ ] Same-seed rows are byte-identical across thread counts; at F03's
      settings the seed-1 quick runs reproduce the existing blocks exactly.
- [ ] The quick food-seeking run completes under 60 s at the shipped
      defaults.
- [ ] The diff stays inside the lab exemption scope.

## Notes for AI Agents

- Decision: lab exemption (user, 2026-09-28) — no gate or goal profile, no
  benchmark specialist; the mutation gate, the Codex review and `make check`
  apply.
- Decision: the user authorized one confirm-only fourth Codex challenge
  round beyond the workflow's three-round cap (2026-09-29), limited to the
  two items left open after round 3; recorded as round 4 in the readings
  file.
- Exception: 60 s quick food-seeking bound (user, 2026-09-29) — ignored
  for this closure because other work was running on the host: the
  unchanged 02aa7851 reference took 63.5–72.4 s there, and paired runs
  show the ladder adds −0.7 to +1.9 s against it (65.8/65.3, 70.5/68.6,
  62.8/63.5 s). No fallback ships; `QUICK_MUTANTS` stays 8.
