# T14.F08 — Sensor Usage Census

**Status**: In Progress
**Last updated**: 2026-09-12
**Feature**: T14.F08
**Track**: [T14 — Runtime Telemetry and Report Integrity](../../roadmaps/t14-runtime-telemetry-and-report-integrity.md)

## Goal

Every persistence checkpoint the benchmark already samples carries a census of
what the living population can perceive: how many living creatures hold a live
reference to each world input key, and how many read any stateful source —
shared memory or a persisted compute-node state. A stored report shows a food
type nobody senses, or a population no brain of which can remember, as a time
series rather than as an absence nobody looked for.

## Non-Goals

- Any new mechanism, charge, rate, default, threshold or RNG draw. This feature
  reads structure the simulation already carries.
- New checkpoints or a new cadence. `SAMPLE_EVERY_TICKS` and the sampled-tick
  rule are T14.F04's and are untouched.
- New goal indicators, thresholds, comparison-block entries, indicator version
  or definition tokens, and the no-regression rule's coverage set. This is a
  checkpoint reading inside the stored `population_persistence` block, not an
  indicator; T14.F01's token rule applies to indicators.
- Any change to `cached_live_vm_world_inputs`, to `structural_companions`, or to
  the server's `vm_live_read_world_inputs_current` payload and its semantics.
- Plasticity resolution, changed-write detection and the shared-memory carrier
  census beside `memory_sensitivity` — T14.F05.
- The clade persistence timeline (T14.F09) and the occupancy grid (T14.F10),
  which reuse these same checkpoints.
- The `v3-cli run` tick sample, and progress-page presentation of the new series
  (T14.F11).
- Any claim that a referenced input is used, matters, or was selected for. A
  reference is exposure, not capability.
- Rewriting historical reports. A report stored before this block existed reads
  it as absent, never as zero.

## Inputs and Invariants

Sources of truth: `crates/v3-core/src/creature/state.rs`
(`compute_live_vm_world_inputs`, `cached_reachable_nodes`),
`crates/v3-core/src/creature/genome/mesh_annotations.rs`
(`collect_live_vm_instruction_indices`),
`crates/v3-core/src/creature/genome/cgp_mesh_annotations.rs`
(`derive_cgp_annotations`, `cgp_live_compute_indices`),
`crates/v3-core/src/contracts/inputs.rs` (`WorldInputKey`, `as_key`,
`food_type_idx`, `InputReference`), `crates/v3-core/src/creature/genome/cgp.rs`
(`GraphSource`, `NodeClass`), `crates/v3-core/src/simulation/tick.rs`
(`advance_shared_memory`), `crates/v3-cli/src/bench.rs` (`PersistenceSample`,
`PopulationReadings`, `PersistenceAccumulator::observe`, `run_one_seed`), and
the track's F08 note.

**The track note's cache claim is wrong, re-verified in planning.** The note
says graph input references are cached beside `cached_live_vm_world_inputs`.
They are not: `compute_live_vm_world_inputs` skips every node whose backend is
not `BackendDef::Vm`, and no graph-input cache exists. Founders are
Graph-backend (`crates/v3-core/src/creature/founder.rs:106`), so a census built
on that cache alone would report the founder population as referencing no world
input at all — precisely the false closed door this feature exists to prevent.
The census covers both backends and is computed at checkpoint time.

**A live reference, in both backends.** A creature is counted for a key when a
mesh-reachable node holds a live reference to it. The two liveness rules are the
ones already in the tree, applied at exact-key resolution:

| Backend | Liveness | Key extraction |
| --- | --- | --- |
| `BackendDef::Vm` | `collect_live_vm_instruction_indices` (backward slice from output instructions) | `VmInstruction::ReadInput { ref_idx }` -> `node.input_refs[ref_idx]` -> `InputReference::World(key)` |
| `BackendDef::Graph` | `cgp_live_compute_indices`, plus every wired output surface — output sink, action-bank slot, execute gate | each `GraphSource::InputLeaf { ref_idx, .. }` edge on a live compute node or on a wired output surface -> `node.input_refs[ref_idx]` -> `InputReference::World(key)` |

The VM rule is `compute_live_vm_world_inputs`'s, unchanged. The graph rule is
the one `derive_cgp_annotations` already uses to derive `MeshReadClass`, read at
key resolution instead of class resolution — which means every wired surface,
not compute nodes alone. A compute-node-only rule reports the founder population
as blind: `crates/v3-core/src/creature/cgp_founder.rs:71-95` wires `FoodHere`
and `NeighborFoodRing` as `InputLeaf` edges straight onto `output_sinks`, and
the founder's compute nodes reference no world input at all.

**Creatures, not references.** A creature contributes at most `1` to a key's
count however many instructions or edges reference it. The server's
`vm_live_read_world_inputs_current` sums reference counts instead; it is a
different reading and stays as it is.

**Stateful reach.** `advance_shared_memory` snapshots `shared_memory` into
`prev_shared_memory` and then decays `shared_memory` in place — a no-op multiply
at the production default `0.0`. It never clears it, so a same-tick `LoadSlot`
reads a value an earlier tick wrote. Every shared-memory read is therefore a
stateful read, not only the explicitly previous ones:

| Content of a live, reachable node | Counts as a stateful read |
| --- | --- |
| `VmInstruction::LoadSlot`, `LoadSlotImm`, `LoadSlotPrev` | yes |
| `VmInstruction::StoreSlot`, `StoreSlotImm`, `ClearSlot` | no — a write, not a read |
| `GraphSource::SharedMemory { .. }` edge on a live compute node or a wired output surface, `previous` either way | yes |
| compute node whose `kind.class()` is `NodeClass::Stateful` | yes — persisted node state |
| compute node carrying a plasticity rule | no — T14.F05 owns plasticity |

Three counts per checkpoint: creatures reading shared memory, creatures holding
a stateful compute node, and creatures with either. The split is what makes the
combined count readable; the combined count is what the track note asks for.

**The whole key universe, not the observed subset.** Rows are emitted for every
key the run's world can present — the seven unparameterized keys plus the three
food-parameterized families once per ordinary food type the world is configured
with, read from the same `food_types()` accessor
(`crates/v3-core/src/kernel/ordinary_food/mod.rs:190`) the rest of the report
uses — so a `0`
distinguishes "no living creature references this" from "this key does not exist
in this world". Row order is `BTreeMap<WorldInputKey, _>` order; no `HashMap`
iteration order reaches the report, per T14.F02's constraint. A row's label is
`as_key()` with `:<food_type_idx>` appended when `food_type_idx()` is `Some`, so
the food-parameterized families stay distinct instead of colliding as they do in
the server's `HashMap<String, u64>` payload.

**Determinism is the binding constraint.** Seeded runs reproduce byte-for-byte
across processes and thread counts, and the gate profile's two-run
byte-identical check inside `make check` exercises these fields:

- Every value is an integer count. No float is accumulated.
- Every value is read after `run_tick` on an already-sampled tick, from state
  the tick produced. No production RNG is consumed, no survivor is selected, and
  no execution path changes.
- The census is pure structure: it inspects genomes and cached reachability, and
  executes no brain.

**Empty population.** Every count is a true `0` and the key-universe rows are
still emitted. A census of an empty population is zero, not unmeasured, so this
follows `surviving_founder_clade_count` rather than the `Option` means beside
it.

**Placement.** The per-creature predicates are pure functions in `v3-core`,
taking a genome and the creature's `cached_reachable_nodes` so no second
reachability walk is paid, which also puts the pure invariants where `proptest`
is already a dev-dependency. Aggregation lives in `bench.rs` inside the existing
sampled-tick `readings()` closure, so it fires at most 21 times per seed.
`CreatureState`, the birth path and the existing caches are untouched: no new
cached field is added to the hot path.

**Serialization.** The census is one structured block of its own type on
`PersistenceSample`, `Option` and `#[serde(default)]`. This is the shape
T14.F04's deferred note proposed for this feature: a structured block, not a
sixth flat scalar, and no `#[serde(flatten)]` change to a determinism-critical
stored-report struct.

**Test-helper fallout.** `PopulationReadings` derives `PartialEq` and is
constructed literally by the `bench.rs` test helpers `empty_readings` and
`readings_for`; extending the checkpoint readings updates those helpers.

## Implementation Tasks

- [x] Pure per-creature predicates in `v3-core`: the live world-input key set
      across both backends, and the stateful-read predicate of the table above,
      each from a genome and its cached reachable indices.
      `compute_live_vm_world_inputs`, `structural_companions`,
      `derive_cgp_annotations` and the server payload keep their current
      behavior and output.
- [x] Carry the census on `PersistenceSample` as a structured optional block,
      aggregated only on sampled ticks and only from post-tick state, emitting
      the full key universe including zeros, under the empty-population and
      historical-report rules above.
- [x] Tests: the Graph-backend founder case a VM-only census misses; a key
      referenced several times counted once; an unreferenced key present with
      `0`; unreachable and dead references uncounted; each row of the stateful
      table; the census at every checkpoint of the real `run_one_seed` loop,
      matching an independently computed value; zeros at extinction; a stored
      report predating the block still loading, plus `v3-core` property tests
      for the pure invariants.

## Verification

- [ ] `make check` -> exit 0, run once on the final feature code; record the
      tested commit.
- [x] Focused tests at `6112143f`: `cargo test -p v3-core -p v3-cli`,
      `cargo clippy -p v3-core -p v3-cli --all-targets` and
      `cargo fmt --all -- --check` all exit 0 and clean. 12 unit
      tests, 2 proptests in `creature::sensor_census::tests` and four
      `bench::tests` arms; named in the readings file.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      and every survivor resolved as killed, equivalent, or deferred. The full
      survivor list stays here.
- [x] Checkpoint samples in the stored goal report carry the census at every
      checkpoint of all three world cases, with the key universe complete and
      the founder population's graph-backend references present rather than
      zero. Key diff in the readings file.
- [x] Benchmark reports stored at
      `docs/progress/features/t14-f08-sensor-usage-census.json` and its `-goal`
      companion.

## Performance and Goal Impact

**Predeclaration — written before the run.** Measurement feature; the track's
observation contract exempts it from the natural-analog rule and it adds no
mechanism. References: the gate and goal profiles compare against the epoch
baselines the series index names, under the existing thresholds.

Expected compute cost: small and larger than T14.F04's. T14.F04 reads cached
values; this feature recomputes VM and graph liveness for each living creature
on each sampled tick — at most 21 sampled ticks per seed, against a population
in the thousands, inside the timed loop. The walk is the same liveness analysis
birth already performs once per creature, and no new work enters the hot path.
The measured wall-clock delta is reported in the readings file. A severe compute
regression is a blocker to report, not a cost to justify here, and no epoch
re-pin is expected or authorized.

Predeclared direction for every goal indicator: **none**. This feature changes
nothing the simulation applies, so lineage diversity, memory sensitivity,
temporal memory sensitivity, mutational neighborhood, drift depth, population
persistence and births per 100 ticks are all expected to be unchanged, and any
movement in them is a defect rather than a result. The stored reports grow by
one census block per checkpoint sample.

Goal impact: the perceptual prerequisite the north-star rows depend on becomes
readable before capability is looked for. T14.F06 and T14.F07 ask whether
creatures live differently; this says whether they can perceive the difference
at all.

**Measured verdict.**

- Gate profile: exit 0 at `6112143f`, `severe=false` on both references, every
  compute metric `level=ok`, epoch not re-pinned, founder neighborhood 76.62 ms
  against the 10,000 ms cap, no evolved neighborhood in this profile.
  `wall_clock` crosses to `level=flag` (+51.3 %, +69.7 %): mostly host load.
- Goal profile: exit 0 at `6112143f`, `severe=false` on both references, epoch
  not re-pinned, evolved neighborhood 659.74 ms against 180,000 ms, founder
  230.65 ms against 10,000 ms, total 636,820.52 ms (10.61 min) against the
  15-minute budget. Direction **none** holds on the indicators: with
  `sensor_census` deleted, `deterministic.goal_indicators` diffs empty against
  T14.F04's. `wall_clock` crosses to `level=flag` (+41.0 %, +29.2 %), 4–5 % of
  it the census; `plasticity_updates` flags T12.F04's inherited 0.066183.

- Reports: [gate](../../progress/features/t14-f08-sensor-usage-census.json),
  [goal](../../progress/features/t14-f08-sensor-usage-census-goal.json).
- Full readings: [`docs/progress/readings/t14-f08.md`](../../progress/readings/t14-f08.md).

## Deviations

Authorized by the user in this feature's goal command, for this feature only,
because the Fable 5.1 budget is exhausted. No model configuration reaches
`main`: the merged range touches no file under `.claude/`.

- The orchestrator is Opus 5 at effort `high` in place of Fable 5.1 at effort
  `medium`; the contract's model check passes on that basis. The effort is
  raised from the contract's `medium` to `high` to offset the loss of the Fable
  spec-owner judgment this substitution removes.
- `roadmap-reviewer` is spawned with the Agent tool's `model` parameter set to
  `opus`, which overrides the agent definition's `fable` frontmatter while its
  `effort: high` frontmatter stays in force, so the reviewer is Opus 5 at effort
  `high`. `.claude/agents/roadmap-reviewer.md` is not edited.
- The implementer's advisor is Opus 5, set by a worktree-local
  `.claude/settings.local.json` containing `{"advisorModel": "opus"}`. That path
  is ignored by git and cannot be committed; the tracked
  `.claude/settings.json` is not edited.

None of the three is a precedent for later features.

## Success Criteria

- [ ] Every checkpoint sample of a stored benchmark report carries the world
      input key census and the three stateful-reach counts, on all three goal
      world cases.
- [ ] The census counts Graph-backend references, so the founder population's
      perceptual reach is visible rather than reading as zero.
- [ ] The readings are reproducible byte-for-byte across processes and thread
      counts, consume no production RNG and change no execution.
- [ ] Reports stored before this feature still load with the census absent, and
      the existing caches, the server payload and the `v3-cli run` tick sample
      are unchanged.

## Notes for AI Agents

- Decision: The census counts living creatures per key, not references per key.
  The server's `vm_live_read_world_inputs_current` sums references and remains a
  different reading with its own semantics.
- Decision: Row labels are `as_key()` with `:<food_type_idx>` appended for the
  food-parameterized families, and the full key universe is emitted with zeros,
  so an unread key is distinguishable from an absent one.
- Exception: The three model substitutions in this spec's Deviations section
  were authorized by the user on 2026-09-12 for T14.F08 only, because the
  Fable 5.1 budget is exhausted. They are not a precedent for later features and
  reach no file on `main`.
