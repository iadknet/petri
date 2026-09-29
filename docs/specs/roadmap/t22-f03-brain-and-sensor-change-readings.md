# T22.F03 — Brain and Sensor Change Readings

**Status**: Complete
**Last updated**: 2026-09-29
**Feature**: T22.F03
**Track**: [T22 — Capability Assays and Evolvability Lab](../../roadmaps/t22-capability-assays-and-evolvability-lab.md)

## Goal

Measurement tooling. Every `v3-lab run` records, in each generation's NDJSON
row, what changed in that generation's elite: genome size and functional
complexity, reachable and executed node counts, the mutation events applied
along its ancestry since the arm's start by operator, its input use by
catalog family as three distinct readings (structural, executed at node
level, causal under ablation on the same scenes), its `steering-v1`
reading, and the score spread and silent/changed/dead fractions of `n`
fresh mutants scored on the same scenes. The summary keeps a compact first
and last projection per arm and replicate so `v3-lab report` shows what
changed between the start and the end. Scores, selection, calibration and
every existing RNG stream are untouched: the seed-1 quick runs reproduce
T22.F02's calibration tables, reach results and fidelity blocks exactly.

## Non-Goals

- No change outside `crates/v3-lab` and docs. Read-level executed use and
  channel-level ablation need `v3-core` exports (`input_use::reads`,
  `consumers::ablated`, `cgp_analysis` are crate-private and `run_tick`
  admits no observed mesh mode); F03 reads at node and family level and
  records the export as a finding (Notes), per the track's ownership rule.
- No new scoring, selection, arena, control, gate, per-tick trace output, or
  T11.F14 knockout count; no why-not ladder (T22.F04).

## Inputs and Invariants

**Contract.** The T22.F03 row, the track's "F03 adds per-elite readings"
note and the "Answered from source" note are the source of truth; F01's spec
fixes the harness and F02's the mask; this spec is a delta on them. Diff
scope (lab exemption): `crates/v3-lab/**`, this spec, the readings file, the
track checkbox and the committed lab summaries under `docs/progress/lab/`
this feature regenerates. `Cargo.toml`, `Cargo.lock` and the `Makefile` are
unchanged.

**Design evidence (2026-09-29).**

| Decision | Options | Choice and reason |
| --- | --- | --- |
| Executed input use | (a) node level: a consumer of the family on a node the elite dispatched on its own scenes (`DispatchRecord`, public); (b) read level: `input_use::reads::ReadRecording` (`pub(super)`; the world tick executes untraced and `ActiveTrace` hops carry node ids, not reads); (c) `Battery::executed_node_ids` (public, battery scenarios, not the assay's scenes) | (a), named `executed_node` so the granularity is explicit; (b) is the recorded T20 finding; (c) reads the wrong scenes |
| Family inventory | (a) `creature_sensor_census` alone (live walk, world and decision keys only; the founder's `EnergyCurrent` and `AgeTicks` reads would be invisible); (b) a lab walk over every `InputLeaf` edge, `ReadInput` instruction, `SharedMemory` source and `LoadSlot*` instruction on public genome types, keyed by `catalog::Family::of` (all 22 families), without the crate-private liveness filter | (b) for the inventory, with the census's live verdict added where it applies |
| Causal influence | (a) family-level ablation rewrite in the lab: every Graph `InputLeaf` edge and VM `ReadInput` whose `input_refs[ref_idx]` is of the family gets an out-of-range `ref_idx`, which both backends resolve to 0.0 (`runtime/cgp/sources.rs:24`, `runtime/vm.rs:382`); each distinct ablated index of a node gets its own sentinel, counting down from `u16::MAX` past indices the node already uses, because `functional_complexity` counts distinct consumed indices per node (`cgp_analysis::collect_consumed_input_refs`, a `HashSet<u16>`; the VM branch likewise) and feeds `cached_complexity` and the action cost — the single-sentinel rule of `consumers::ablated` would collapse the founder's six `UpstreamSlot` reads and change its charges. `genome_size()` and complexity stay equal, pinned by test; (b) export `ablated` (loses the exemption); (c) skip causal (contradicts the note) | (a), ~40 lines. Behaviour change is read on the observables F01 already records, never an action trace (F01: "the action log is never read") |
| Silent / changed / dead | (a) `neighborhood-v1` signature and `neighborhood::classify` (public, T11.F01 semantics, what the F04 note calls "on the battery"); (b) per-tick action comparison on the scenes (needs an action trace) | (a); score spread on the scenes, class on the battery, kept distinct |
| Steering | `neighborhood::steering::SteeringBattery::{generate, read}` (public, T11.F21) | Adopted unchanged; `executed` is the elite's scene-executed node ids |
| Mutant signature (Tarapore and Mouret 2015, research note) | `n` fresh mutants on the `observation` stream F01 reserved vs. reusing the children | Fresh: the children's count varies with the population and their scalars drive selection; a fixed-`n` sample is comparable across generations and arms |
| Signature arms | Every genome arm each generation vs. the arms whose elite can change | Default `changing` (reference, user, `shuffled-score`): founder-only, `mutation-off` and the comparator carry the same genome every generation, so their signature is the reference arm's generation-0 block (the founder) or identical-by-construction mutants; `--signature-arms all` reads them, keeping the quick run inside F01's minute |

**Seams verified in code (2026-09-29).**

| Need | Seam |
| --- | --- |
| Size, complexity, reach | `CreatureGenome::genome_size`, `analysis::{functional_complexity, mesh_reachable_nodes}` |
| Executed nodes | `DispatchRecord::executed_indices(age, u64::MAX)` on the per-scene `Frozen` `evaluate_genome` returns: every node dispatched from fresh state through the last living tick boundary (`eval.rs:499–506` breaks on death before that tick's record; a first-tick death leaves it empty) |
| Live census | `creature::sensor_census::creature_sensor_census(genome, indices)` → `world_inputs`, `decision_inputs`, `reads_shared_memory`, `holds_stateful_node`; labels via `neighborhood::input_use::catalog::Family::{of, label}` (`pub mod catalog`); decision keys map to the same-named variants |
| Lab walk and ablation | `NodeGenome::input_refs`, `BackendDef::{Graph, Vm}`, `CgpGraphBackendDef::{compute_nodes, output_sinks}`, `GraphEdge::source`, `GraphSource::{InputLeaf, SharedMemory}`, `VmInstruction::{ReadInput, LoadSlot, LoadSlotImm, LoadSlotPrev}` are public; the engine itself indexes `input_refs` as `u16` (`mutation/graph/operators.rs:842`) |
| Battery class | `neighborhood::{Battery::generate(food_type_count), Battery::signature(genome, &config.runtime, config.shared_memory.decay_rate), classify(&base, &candidate).class}`; a signature is 80 executions (48 snapshots, 8 × 4 sequence ticks) |
| Steering | `SteeringBattery::generate(food_type_count)`, `read(genome, &config.runtime, &BTreeSet<NodeId>)` → `SteeringReading`: 48 (a) executions plus one (b) execution per move |
| Mutants | `MutationEngine::apply_mutations_with_food_type_count` as `campaign::breed` calls it; `MutationSummary::{applied_by_operator, attempted_by_operator}` |
| Mask | `eval::expressed` (F02): every battery and steering execution and every evaluated copy is masked like the evaluation; structural readings are of the stored genotype |

**Read arms and the elite.** The elite of a row is F01's validation
candidate, the top-ranked individual by true scalar. Every genome arm's row
(comparator included) carries `shape`; `signature` is carried by the
`--signature-arms` set; scripted arms carry `readings: null`. Readings are
computed after ranking and before the row is written, for every written row
including the reach generation; a row the byte cap refuses contributes no
reading anywhere, though the creature-ticks it spent still count in
`timing`. They never read the `scenes`, `mutation` or `selection`
streams and never feed the population.

**`shape` block.**

| Field | Definition |
| --- | --- |
| `genome_size`, `functional_complexity`, `nodes`, `reachable` | `genome_size()`, `functional_complexity()`, `nodes.len()`, `mesh_reachable_nodes().len()` of the elite |
| `executed`, `deaths` | Count of the union over the generation's training scenes of the nodes dispatched through each scene's last living boundary (a death tick's dispatches are unobservable, as F01 records); `deaths` = scenes the elite died in, so the censoring is visible. The row's existing `elite.executed_nodes` (production window at the frozen point) is unchanged |
| `ancestry` | `births` (engine passes on the path from the arm's start genome, applied events or not, so a `mutation-off` elite shows births with zero applied), `requested`, `applied` totals and `applied_by_operator` (sorted keys) summed along the path: a start has zeros, a child has its parent's plus its own birth events, a carried elite keeps its own |
| `families[]` | One entry per catalog family with a consumer on a reachable node, in catalog order: `family` (label), `structural` (true by construction), `executed_node` (a consumer on a node in the executed union), `live` (the census's verdict for world and decision families; `null` for the others, which it does not cover). A consumer is an `InputLeaf` edge or `ReadInput` instruction whose in-range `ref_idx` names a reference of the family; `SharedMemory` sources and `LoadSlot*` instructions are consumers of the two shared-memory families |
| `stateful_node` | The census's `holds_stateful_node` |
| `steering` | `SteeringReading` of the masked elite with the executed node ids |

**`signature` block.** `null` where not computed.

| Field | Definition |
| --- | --- |
| `causal[]` | One entry per `families[]` entry except the shared-memory families (no reference to rewrite): `family`, `causal`, `score_delta`. The ablated copy is evaluated on the generation's training scenes; `causal` is true when, on some scene, any `SceneScore` field or the per-tick `(position, energy)` sequence differs from the elite's, so a score change implies `causal`; `score_delta` is its scalar minus the elite's. The scoring pass keeps every individual's per-tick sequence until ranking (≤ scenes × lifetime × 16 B each), so the elite is not re-run. The rewrite needs, per node, one unused out-of-range index per distinct ablated index: a genome file with a node of ≥ 65,535 references is refused at load, and an evolved elite whose node lacks that headroom gets `causal: null` with `unsupported: "input_refs"` |
| `mutants` | `n`; for `k` in `0..n` the elite's clone passed once through the engine with the arm's `MutationConfig`, `mesh_reachable_nodes(elite)`, `ParentExecuted::Record` of the elite's frozen record and age (as breeding uses them), RNG `hash(observation, arm, generation, k)` with `observation = hash(r_i, "observation")`, and the arm's food-type count. Per mutant: `identical` (genome `==`); class on `neighborhood-v1` against the masked elite's signature (`silent` without execution when identical); scalar over the training scenes (the elite's by copy when identical: solo evaluation is deterministic) |
| `mutants` fields | `n`, `identical`, `silent`, `changed`, `dead` (the three sum to `n`), `scores` (the `n` scalars sorted ascending), `improved`, `equal`, `worse` against the elite's scalar, `mean_delta` |

**Determinism.** Mutant seeds are fixed by construction; ablated copies and
mutants evaluate in parallel and are collected in index order; family lists
follow catalog order, maps sorted keys, scores are sorted; every count is an
integer folded in a fixed order. Same-seed rows stay byte-identical across
thread counts and processes.

**Outputs.** `row_version: 2` adds `readings: {shape, signature}` beside
`elite`. `summary_version: 3` adds, per arm and replicate,
`readings: {first, last}` — projections of the replicate's first and last
written rows for that arm, `null` when none was written — and
`timing.readings_creature_ticks` (production ticks on ablated copies and
mutants, also in the total); `provenance.sizes` gains the resolved
`mutants` and `signature_arms`, present in every summary, rows written or
not. The projection is keyed to its two named consumers and carries
nothing else: `first` holds the shape scalars (`genome_size`,
`functional_complexity`, `nodes`, `reachable`, `executed`, which the report
deltas, plus `deaths`, `births`, `requested`, `applied`, which only a
research-note delta reads); `last` holds the shape fields the report renders
(the `first` scalars less `requested`, the `families` table, steering
`moves`, `exact_hits`, `avoidance_trials`, `avoided`; `stateful_node`,
`requested`, per-operator counts and the other steering counters stay in
the rows) and, when computed, the signature aggregates: `causal[]`, `n`, `identical`, `silent`, `changed`,
`dead`, `improved`, `equal`, `worse`, `score_min`, `score_median`,
`score_max`, `mean_delta`. The sorted score vectors stay in the ignored
NDJSON. `report` renders v2 and v3 summaries (v2 without the readings
section; v1 stays refused) and, from a v3 summary, one row per arm and
replicate: last shape with the delta from first, the family table
(`structural` / `executed_node` / `live` / `causal` with `score_delta`),
steering `exact_hits / moves` and `avoided / avoidance_trials` (`null` at a
zero denominator), and the signature aggregates or `signature: not
computed`. The committed `docs/progress/lab/t22-f02-*.json` summaries are
regenerated at `summary_version: 3`; v3 first reaches `main` with this
feature, so its `last` shape needs no further version.

Projected sizes: `shape` ≤ 1.5 KB (22 families with the lab's one food
type; three families gain an entry per further represented type), `signature` ≤
1.5 KB at `n = 8`, so a row stays ≤ 11 KB at population 64 and a campaign
of three evolving arms × 8 × 100 rows ≈ 27 MB under the 64 MiB cap; the
summary gains ≈ 2.75 KB per genome arm and replicate as stored (≤ 200 KB at campaign
sizes) inside the 1 MiB reserve, whose oversize disposition (exit 1, no growth) is
unchanged. Nothing per tick is written: sequences are compared in memory
and discarded.

| CLI parameter | Default (campaign / `--quick`) |
| --- | --- |
| `--mutants` | 8 / 8 (1..=64); the pilot lowers the `--quick` value, never below 2, if the quick food-seeking run exceeds 60 s, and records it here. Pilot: 8 kept (59.0 s, repeat 57.6 s) |
| `--signature-arms` | `changing` (reference, user, `shuffled-score`) / same; `native` (reference and user only, the pilot's second step if 60 s is still exceeded at `--mutants 2`); `all` (every genome arm, comparator included). A quick run over 60 s at `native` and 2 mutants is a user decision |

## Implementation Tasks

- [x] `readings` module: ancestry bookkeeping on `Individual`, per-scene
      frozen records and per-tick sequences kept by `score_individual`, the
      family walk, the `shape` block, the family-level ablation rewrite with
      its precondition, the `signature` block on the `observation` stream,
      and the row and summary fields (`row_version: 2`, `summary_version:
      3`, `readings_creature_ticks`).
- [x] `report` renders v2 and v3 summaries; the readings section.
- [x] CLI: `--mutants`, `--signature-arms`; `--quick` sizes carry `mutants`.
- [x] Pilot: seed-1 `--quick` food-seeking and wall runs; fix the `--quick`
      values; regenerate the three committed F02 summaries.

## Verification

- [x] `cargo test -p v3-lab`: founder shape (families are `FoodHere:0`,
      `NeighborFoodRing:0`, `AgeTicks`, `EnergyCurrent`, `ActionQueue` and
      `UpstreamSlot`; executed union ⊆ reachable; ancestry zeros); the
      ablated founder keeps its `genome_size` and `functional_complexity`
      for every family, and a Graph and VM proptest with repeated indices
      and already-occupied sentinels gives each distinct target its own
      free out-of-range sentinel, keeps every other index, `genome_size`
      and `functional_complexity`; a child's ancestry is its parent's plus its
      birth and a carried elite's is unchanged; first-tick death gives an
      empty executed union and `deaths` counts it; an unread family ablates
      to `causal: false`, `score_delta: 0`; a family with `score_delta ≠ 0`
      is `causal: true`; the founder's `NeighborFoodRing:0` on a scene it
      eats in is `causal: true`; a node with 65,535 references is refused at
      load and a node without sentinel headroom yields `causal: null` on an
      evolved elite, for a Graph and a VM node; a
      `mutation-off` elite under `--signature-arms all` yields `n` identical
      mutants classed `silent` and scored by copy; classes sum to `n`,
      scores sorted; same-seed rows byte-identical across threads and
      processes with `mutants: 4`, `all`; a byte-cap-refused row contributes
      no `first`/`last` and an arm with no written row has `null`; the
      committed v2 summaries render and a v3 summary renders the readings
      section; `readings_creature_ticks` counted -> results in the readings
      file. 126 unit (135 after the mutation gate) and 18 integration
      tests pass; `AreaFoodSummary:0` is
      the comparator's, pinned there.
- [x] Seed-1 `--quick` food-seeking and wall runs: calibration tables,
      per-arm reach results and fidelity blocks equal the
      `docs/progress/lab/t22-f02-{food-seeking,wall}.json` values committed
      at df2ddbe3 (the comparison target, since the files are regenerated in
      place); wall time, creature-ticks and `readings_creature_ticks`
      recorded in the readings file. Run from the clean tree at 2dd9aa20
      (summaries `dirty: false`): equal for all three arenas (ring
      included); creature-ticks less `readings_creature_ticks` equal F02's;
      rows sha256 food `c46e4af9…`, wall `408ab735…`, unchanged;
      food-seeking 57.5 s committed (five same-row runs at 57.5–60.5 s, one
      over the bound by 0.5 s; the ladder's pilot trigger did not fire and
      the rows are identical, so the excess is host load, not cost — ruling
      in the readings file), wall 32.7 s, ring
      0.2 s (exit 2); summaries 77,502 / 77,634 / 10,028 B -> readings
      file, Quick runs.
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants` at 88d66a0d (feature code
      2dd9aa20; diff base df2ddbe3; run mode `fresh`): `136 mutants
      tested in 6m: 23 missed, 78 caught, 35 unviable`, 0 timeouts; output
      `~/.local/share/petri-tools/mutants/t22-f03/mutants.out`. All 23
      killed by tests only (table below), so one fresh run; a
      `MUTANTS_ITERATE=1` pass in the same directory caught all 23
      (feedback only). 135 unit, 18
      integration tests pass.

| Survivor (all killed) | Test |
| --- | --- |
| `readings.rs:350` `breeding_frozen` → `Default` | `breeding_uses_the_last_training_scene_record` |
| `readings.rs:420/425/428` delete VM arms `ReadInput`, `LoadSlot \| LoadSlotImm`, `LoadSlotPrev` in `consumers` | `a_vm_node_consumes_its_read_input_and_shared_memory_families` |
| `readings.rs:522` `==` → `!=` (world `live`) | `a_dead_world_read_is_structural_but_not_live` |
| `readings.rs:526` `==` → `!=` (decision `live`) | `a_wired_decision_read_is_live` |
| `readings.rs:673` `\|\|` → `&&` in `run_copies` | `a_copy_differs_when_either_its_score_or_its_sequence_differs` |
| `readings.rs:740` `/` → `*`, `/` → `%` in `fold_mutants` | `folded_mutants_average_their_deltas`; `mean_delta` in the fold proptest |
| `readings.rs:850` `+` → `-` in `read` | `read_charges_the_ablated_copies_and_the_mutants` |
| `summary.rs:521` `with_delta` → `String::new()`, `"xyzzy"` | `readings_cells_print_changes_ratios_and_nulls` |
| `summary.rs:527` `ratio` → `String::new()`, `"xyzzy"`; `==` → `!=`; `:531` `/` → `%`, `*` | same |
| `summary.rs:537` `opt_bool` → `String::new()`, `"xyzzy"` | same |
| `summary.rs:541` `usize_i64` → `0`, `1`, `-1` | same |
| `summary.rs:611` `==` → `!=` (causal lookup) | `readings_section_prints_the_last_elite_and_each_family_its_own_causal_reading` |

- [x] Benchmark: `Not applicable: lab feature` (below).
- [x] `cargo test -p v3-lab` (126 unit, 18 integration pass), `cargo
      clippy -p v3-lab --all-targets` (0 warnings), `cargo check
      --workspace --all-targets` and `make roadmap-check` clean at
      2dd9aa20 plus the regenerated summaries and documents.

## Performance and Goal Impact

**Predeclaration — written before the run.** Not applicable: lab feature.
The diff is confined to `crates/v3-lab/**`, this spec, the readings file,
the track checkbox and the regenerated committed lab summaries; no
trajectory, default, founder, recipe or mutation policy changes, so no
profile runs and no series entry. The natural-analog and
environmental-pressure rules do not apply. Lab cost per generation: every
shape arm pays ≤ 96 single-tick steering executions; every signature arm
pays `(novel + F) × scenes × lifetime` production creature-ticks, where
`novel` is the non-identical mutants (≈ 0.35 × `n` at F02's identical
fraction 0.62–0.69) and `F` the elite's ablated families, plus 80 battery
executions for the elite and 80 per non-identical mutant. At quick sizes
(4 × 40 generations, `changing` = 2 signature arms, `n = 8`, founder `F` =
6: `FoodHere:0`, `NeighborFoodRing:0`, `AgeTicks`, `EnergyCurrent`,
`ActionQueue`, `UpstreamSlot`) this is ≈ 2 × 128k × 8.8 ≈ 2.25M
creature-ticks, about 46% of F02's 4.94M, so the quick food-seeking run is
projected at ≈ 46 × 1.46 ≈ 67 s at 8 threads against F01's 60 s criterion; the pilot lowers `--quick` `--mutants`
(to 4: ≈ 55 s), then the arm set, as the CLI table says. At campaign sizes
(population 64) the addition is ≈ 2 × 7.8 evaluations per generation over
3 × 64 + 2, about 8% of the creature-ticks before the battery and steering
work, under 10% with it.

**Measured verdict.** Not applicable: lab feature. `git diff --stat
main...HEAD` plus the working tree touches only `crates/v3-lab/**`
(`campaign.rs`, `cli.rs`, `eval.rs`, `lib.rs`, `readings.rs`,
`readings/tests.rs`, `run.rs`, `summary.rs`,
`tests/fixtures/summary-v2-wall.json`, `tests/lab_run.rs`), this spec, the
readings file and `docs/progress/lab/t22-f02-{food-seeking,wall,ring}.json`,
within the predeclared scope (the track checkbox lands at closure).

- Full readings: [`docs/progress/readings/t22-f03.md`](../../progress/readings/t22-f03.md).

## Success Criteria

- [x] Every genome arm's rows carry `readings.shape`, the `changing` arms'
      rows carry `readings.signature`, the summary carries the first and
      last projections per arm and replicate, and `v3-lab report` renders
      the readings section from the summary alone.
- [x] The seed-1 quick food-seeking and wall runs reproduce T22.F02's
      calibration, reach and fidelity numbers exactly and the food-seeking
      run finishes under 60 s.
- [x] Same-seed rows are byte-identical across thread counts and processes.
- [x] Nothing outside `crates/v3-lab` and the documentation changes.

## Notes for AI Agents

- Decision: lab exemption (user, 2026-09-28) — no gate or goal profile, no
  benchmark specialist; the mutation gate, the Codex review and `make check`
  apply.
- Deferred: read-level executed use and channel-level ablation are not
  reachable from the lab (`input_use::reads` and `consumers::ablated` are
  crate-private, `cgp_analysis` is `pub(crate)`, `run_tick` executes the
  mesh untraced); F03 reads executed use at node level (`executed_node`)
  and ablates by family. A `v3-core` export is a T20 finding, not a lab
  patch.
- Deferred: the elite's `applied_by_operator` since the start, `requested`,
  `stateful_node` and the steering `scenarios`, `within_45`, `move_voted`
  counters live in the ignored rows only (review P2 #1 remediation kept the
  summary to what the report renders); a research note that needs them from
  the committed summary adds them to the keep-list with their consumer, at
  the earliest in T22.F04's summary change.
- Cost: `/usage` totals at closure await the user; implementer passes 3
  (build, self-review, review remediation) with advisor consults 3, 2, 2;
  spec-owner resumes after Plan 2; Codex challenge rounds 2, final verdict
  `ready`; Codex reviewer findings P1 0, P2 2 (both remediated), P3 0.
