# T11.F27 — Compact Action-Parameter Storage

**Status**: Complete
**Last updated**: 2026-09-24
**Feature**: T11.F27
**Track**: [T11 — Brain Genotype-Phenotype Map](../../roadmaps/t11-brain-genotype-phenotype-map.md)

## Goal

A brain's action-parameter outputs exist only for the three fields the body's
commit decoder (`decode_commit`) reads: `Eat` food type, `Reproduce` transfer
fraction and `StealEnergy` amount. The Graph sink catalog, the VM
`WriteActionParam` address space and the runtime parameter surface all hold
exactly those three fields, so no sink, VM address or inherited wiring can
name one of the five undecoded fields (`Eat[1]`, `Move[0]`, `Move[1]`,
`Reproduce[0]`, `StealEnergy[0]`). Fresh `ReadActionQueueParam` draws name
only the queue-parameter slots that can carry a value (`0..2`). A pre-F27
serialized genome that carries an old parameter sink or `WriteActionParam`
is rejected, not converted.

Natural analog: a motor neuron can only synapse on muscle that contracts.
Motor end plates exist where there is contractile muscle, so a motor
connection has nothing else to land on. The change reaches creatures only
through the parameter wiring their offspring inherit and the fields their
mutations can name, with no sensor.

## Non-Goals

- Other inactive wiring: unwired sinks of any kind, `CustomOutput`,
  `WriteSlot`, `ClearSlot`, `RouterGate`, vote sinks (track note).
- The VM's tolerant invalid-address behavior. A `WriteActionParam` whose field
  index is 3 or more is ignored and still pays its opcode charge, and a
  raw-field nudge may still produce one (track note).
- `ReadActionQueueParam` beyond the fresh draw: `WorldAction::param`, the
  queue's parameter layout, the runtime read, the raw-field nudge and
  out-of-range reads (still `0.0`) are unchanged.
- Conversion of old artifacts, a genome format-version field, and the server
  `PROTOCOL_VERSION`.
- Mutation supply, operator weights, the opcode draw (uniform over 39) and
  any config value.
- The structural census and T20 causal-use measurement, T17.F03/F04, and new
  telemetry fields or instrument version strings.

## Inputs and Invariants

- Sources: the T11.F27 track row and success line, the track note
  "Action-parameter storage cleanup, 2026-09-24" (scope, `FIXED_SINK_COUNT`
  99 → 94, `WriteActionParam` addressing the three decoded fields, explicit
  reject-or-convert boundary, current-format round-trip and replay, and the
  `ReadActionQueueParam.param_slot` draw), and the T11.F25 spec (decoded
  catalog `(Eat, 0)`, `(Reproduce, 1)`, `(StealEnergy, 1)`, flat VM slots
  0/5/7, its decoder-agreement test and draws). The owning track row lists
  the dependencies.
- Current code (verified 2026-09-24 at `2a346774`):
  - `creature/genome/vote.rs` holds `VOTE_PARAM_SLOTS` = 2 and T11.F25's
    `DECODED_ACTION_PARAMS`, `action_param_flat_index`,
    `DECODED_ACTION_PARAM_FLAT_SLOTS` and `is_decoded_action_param`.
  - `creature/genome/cgp.rs`: `OutputSinkKind::ActionParam(VoteKind, u8)`;
    `new_with_fixed_outputs` appends 8 parameter sinks kind-major at 91..99;
    `FIXED_SINK_COUNT` = 99 with a const assert.
  - Runtime: `MeshSideOutputs.action_params: [ActionParams; 4]` with
    `ActionParams = [f32; 2]` (`runtime/types.rs`); the graph sink write
    (`runtime/cgp/effects.rs`); the VM write addresses
    `params[slot_idx / 2][slot_idx % 2]` and ignores an invalid slot
    (`runtime/vm.rs`); the commit passes the kind's row to `decode_commit`
    (`runtime/mesh.rs`), which reads `params[0]` for `Eat`, nothing for
    `Move` and `params[1]` for `Reproduce` and `StealEnergy`.
  - Draws (`mutation/`): `pick_random_surface` skips undecoded sinks through
    `is_fresh_edge_sink` (also used by `can_add_edge`) and makes one
    `gen_range(0..c + 94)` on a fixed-catalog def; opcode 25 draws
    `DECODED_ACTION_PARAM_FLAT_SLOTS[gen_range(0..3)]`; opcode 29 draws
    `param_slot: gen_range(0..8)`. `mutate_one_instruction_field` nudges one
    of `WriteActionParam`'s two operands by ±1. No operator moves an existing
    edge to another sink.
  - `WorldAction::param` returns non-constant values only for slots 0 and 1.
  - Production founder: a 97-unit vote graph with no VM node that wires only
    `ActionParam(Reproduce, 1)`; `genome_size` counts only wired sinks. Every
    `WriteActionParam` literal outside `mutation/vm/operators.rs` is in test
    code.
  - Serialization: genomes are JSON through serde (externally tagged enums).
    Stored genomes appear in recruitment raw records (`v3-cli/src/recruitment.rs`
    `read_raw`, read back only by the same run's replay check), bench
    internals and the server's creature inspector payload, which the frontend
    renders (`frontend/src/types/genome.ts`, `inspector/graphNodeFormatters.ts`,
    `inspector/vmInstructionFormat.ts`, `inspector/mesh/meshSemantics.ts`).
    No committed file is loaded as a genome by code.
  - Reference docs naming the old layout: `v3-genome-spec.md`,
    `v3-graph-backend-spec.md`, `v3-vm-isa-spec.md`, `v3-mutation-spec.md`,
    `v3-startup-seeding-spec.md`.
- Concurrent main changes: two non-roadmap cleanup branches (dead-code
  removal, removal of the shared food-config copy) are merging into main
  during this feature. If main advances, the feature rebases onto it before
  the benchmark runs.
- Research decision, 2026-09-24:

| Option | Evidence and fit | Disposition |
| --- | --- | --- |
| Typed field catalog: `ActionParam(ActionParamField)` sinks, VM field index over the same catalog, runtime surface of three | Makes an undecoded Graph sink unrepresentable and removes the five cells from storage. CGP fixes output genes to the outputs the phenotype reads ([CGP-Library](https://www.cgplibrary.co.uk/files2/CartesianGeneticProgramming-txt.html)) | Adopted |
| Keep `ActionParam(VoteKind, u8)` and build only three sinks | Undecoded pairs stay representable, and old sinks deserialize silently | Rejected |
| Convert old artifacts (flat 0/5/7 → new indices, other writes → `Noop` to keep jump offsets) | No code path loads a pre-F27 artifact, so conversion code would have no consumer; backward compatibility is not a goal (track note, `AGENTS.md`) | Rejected |
| Add a format-version field to genomes | New machinery; the serialized shape change already rejects every affected genome ([serde enum representations](https://serde.rs/enum-representations.html); a field without `#[serde(default)]` is required, [serde field attributes](https://serde.rs/field-attrs.html)) | Rejected |

Fixed design:

| Decision | Value |
| --- | --- |
| Catalog | `creature/genome/vote.rs` defines `ActionParamField` with variants `EatFoodType`, `ReproduceTransferFraction`, `StealEnergyAmount`, in that order (T11.F25's kind-major catalog order). `ALL` gives the order, `index()` is the position in `0..3` and `kind()` gives the `VoteKind`. The T11.F25 catalog items and `VOTE_PARAM_SLOTS` are removed or replaced when nothing else needs them. |
| Graph sinks | `OutputSinkKind::ActionParam(ActionParamField)`. `new_with_fixed_outputs` appends the three sinks in `ALL` order after the 27 vote sinks, at indices 91..94. `FIXED_SINK_COUNT` = 94, const-asserted. |
| VM | `WriteActionParam { field_idx: u8, src: u8 }`. It writes `action_params[field_idx]` when `field_idx < 3`, and an invalid index is ignored. Cost stays 0.14. The rename from `slot_idx` is the serialization boundary for VM genomes. |
| Runtime | `action_params` is `[f32; 3]`, indexed by `ActionParamField::index()`, local to one mesh run and zero at its start, last write wins. `decode_commit` receives the surface and reads only its kind's field: `EatFoodType` for `Eat`, `ReproduceTransferFraction` for `Reproduce`, `StealEnergyAmount` for `StealEnergy`, nothing for `Move`. Clamping and sanitizing are unchanged. |
| Decoder agreement | Tests tie the catalog to the decoder. Sensitivity: for every field, some pair of finite values in that field alone, with the others fixed, changes the action `decode_commit` returns for its kind's sinks. Independence, as a property over arbitrary finite values of every field: a field never changes the action for a sink of another kind, and no field changes `Move`, `Terminate` or `Decide`. This replaces T11.F25's `undecoded_action_params_never_change_the_commit`. |
| Graph draw | `pick_random_surface` makes one uniform `gen_range` over compute nodes then every sink in vector order. The T11.F25 filter goes, because every sink is drawable, and `can_add_edge` is true when the def has any compute node or sink. On a fixed-catalog def this is the same `gen_range(0..c + 94)`, which picks the same sink kind as T11.F25. |
| VM draws | Opcode 25: `field_idx = ActionParamField::ALL[rng.gen_range(0..ActionParamField::ALL.len())].index()`, the same single `usize` `gen_range(0..3)` as T11.F25, naming the same field. Opcode 29: `param_slot: rng.gen_range(0..2)`. Other operands and opcodes are unchanged. |
| Boundary | Reject the affected old shapes. A pre-F27 genome that contains an old `ActionParam` sink (`{"ActionParam":["Eat",0]}`) or an old `WriteActionParam` (`{"slot_idx":…}`) fails to deserialize with serde's error. Every graph built by `new_with_fixed_outputs` carries the old parameter sinks, so every pre-F27 production graph genome is rejected. A pre-F27 genome that contains neither, such as a VM-only genome without `WriteActionParam`, deserializes unchanged, because its meaning did not change. No conversion path exists, so no conversion has to preserve old slots 0/5/7 or VM jump targets. |
| Frontend | The TypeScript genome types and inspector labels follow the new shapes: three named parameter sinks and three `WriteActionParam` field labels, with an invalid index rendered as `param[i] ← rN`. Frontend tests use fixtures in the serialized shape (the three Graph parameter sinks, VM `field_idx` 0, 1, 2 and 3) and assert the distinct labels, so a missed rename cannot label every write as `Eat`. |
| Determinism and equivalence | Draws remain pure functions of the seeded RNG. Two draws keep T11.F25's RNG use, and focused tests pin it against a cloned RNG. On a fixed-catalog def, `pick_random_surface` makes exactly one `gen_range(0..c + 94)` and returns `ComputeInput(k)` or `SinkInput(k - c)`. A def with neither compute nodes nor sinks returns `None` without consuming RNG. Opcode 25's field is the one `gen_range(0..3)` names, with the same RNG state afterward. Expected consequence, read through the re-pins and not claimed as a proof: a run from the production founder follows T11.F25 until the first birth that makes a fresh opcode-29 draw or nudges a `WriteActionParam`'s field operand, under the representation map (sink indices 96 → 92 and 98 → 93, field indices 0/5/7 → 0/1/2). Values derived from serialized genomes, such as payload hashes and fingerprints, change without a trajectory change. Every re-pinned value is attributed to one of these causes or to the trajectory downstream of such a birth. |
| Cost | Negligible: three floats instead of eight per mesh evaluation, and 94 sinks instead of 99 per graph scan. |

## Implementation Tasks

- [x] Write failing tests first: decoder agreement; the fixed catalog (94
      sinks, the three parameter sinks at 91..94 in `ALL` order); VM write
      addressing (field `i` writes surface `i`, `field_idx` ≥ 3 ignored and
      charged); the Determinism row's RNG-use tests; opcode-25 draws cover
      exactly the three fields and opcode-29 draws cover exactly `0..2`, both
      slots drawn; and the
      boundary (pre-F27 `ActionParam` sink JSON and
      `WriteActionParam { slot_idx, .. }` JSON fail to deserialize, an
      unaffected pre-F27 VM genome deserializes, and a current-format genome
      with all three fields wired by both backends round-trips equal).
- [x] Cover the commit path end to end, with new or existing runtime tests
      (`runtime/vote_surface_tests.rs`, `runtime/pass_loop_tests.rs`). Each
      of the three fields, written by a Graph sink and by a VM write, must
      reach the committed `WorldAction`. The last write across Graph and VM
      nodes must win, and the surface must persist across passes within a
      tick. The surface is local to one mesh run (`MeshSideOutputs::new` in
      `runtime/mesh.rs`), so a commit that reads a field no node wrote in that
      run decodes it as zero.
- [x] Implement the fixed design in core (catalog, sinks, runtime surface,
      decoder, graph effects, VM execution, draws, raw-field nudge on the
      renamed operand), then the frontend types and labels. Update the doc
      comments that describe the eight-slot surface.
- [x] Update the five reference docs listed in Inputs to the three-field
      layout, the `0..2` queue-parameter draw and the reject boundary.
- [x] Re-pin every trajectory, replay, drift, recruitment-paths, hash or
      fingerprint test value that changes. List each old and new value in the
      readings with its attribution (Determinism row). No predicate may be
      weakened, and a changed founder-behavior pin (a run without mutation)
      is a defect, not a re-pin.
- [x] Record gate and goal readings as Performance requires.

## Verification

- [x] `cargo test -p v3-core --test viability` first, then
      `cargo test -p v3-core --lib` and the new tests, with the red run and
      green run in [readings](../../progress/readings/t11-f27.md).
- [x] The focused tests from the first two Implementation Tasks pass, named
      in readings, including the mesh-run-local zero decode and the exact
      `param[3] ← r1` frontend label.
- [x] `cargo clippy --workspace --all-targets -- -D warnings`,
      `cargo fmt --all --check`, `cargo test --workspace --no-fail-fast`, and
      from `frontend/` `npx tsc -b`, `npx biome check src/` and
      `npx vitest run` pass; results in readings.
- [x] Current-format replay: the recruitment test
      `a_complete_run_writes_the_record_summary_and_replay_check`
      (`v3-cli/src/recruitment.rs`) matches every proposal, and
      `cargo test -p v3-core --test reproducibility` passes; results in
      readings.
- [x] `make check` exits 0 in the worktree, frontend included (`869d8339`).
- [x] Fresh mutants run (`bab88f34`): `54 mutants tested in 9m: 23
      caught, 31 unviable`; no survivors. Output
      `~/.local/share/petri-tools/mutants/t11-f27/mutants.out`.
- [x] Gate and goal summaries stored at
      `docs/progress/features/t11-f27-compact-action-parameter-storage.json`
      and `...-goal.json`. Local raw hash, byte count and verification time
      are checked, series entries point to the summaries, and no new full
      report is staged. Gate not severe; goal severe against the T19.F04
      epoch only, not severe against the latest closure; severe accepted
      and goal epoch re-pinned to T11.F27 (Measured verdict); readings in
      [`docs/progress/readings/t11-f27.md`](../../progress/readings/t11-f27.md#benchmark-gate-and-goal-benchmark-specialist-2026-09-24).

## Performance and Goal Impact

**Predeclaration — written before the run.** The natural analog and the path
to creatures are in the Goal. The feature adds no environmental pressure, so
the three-world rule adds nothing beyond the ordinary goal run. Expected
compute cost: none measurable. From the founder, trajectories match T11.F25
until the first birth that makes a fresh opcode-29 draw or nudges a
`WriteActionParam` field operand. Opcode 29 is 1 in 39 fresh VM
instructions, and VM operators touched 9–16% of mutation carriers at T11.F25.
So divergence is expected early in each world wherever such draws occur,
though it is not guaranteed, and the readings report where it was observed.
After divergence every counter can move. The track
note expects the goal trajectories to move and an epoch re-pin.

References. Gate: epoch and latest closure
`t11-f25-meaningful-action-parameter-targets.json`. Goal (goal-worlds-v1):
epoch `t19-f04-vote-based-action-selection-goal.json`, latest closure
`t11-f25-meaningful-action-parameter-targets-goal.json`. Standard thresholds
apply: +10%/+50% for work and +25%/+100% for wall time. The expected re-pin
and any severe work counter on either profile, or an extinction in any goal
world, are user decisions under the blocker rule. Wall-time moves are
flag-only, and the observation caps are unchanged.

| Indicator | Predeclared direction |
| --- | --- |
| `config_digest`, `FOUNDER_GENOME_SIZE_UNITS` (97) | Unchanged |
| Founder behavior: founder-only runs, and founder half `mesh_execution`, `steering`, `reachable_node_count` (no mutation) | Unchanged |
| Founder half `operator_rows` and `births`; goal `founder_changed_per_all_births`, `founder_dead_per_all_births` per world | Move only through births that make a fresh opcode-29 draw or nudge a `WriteActionParam` field operand drawn earlier in the same birth; no sign |
| Gate and goal work counters (all nine bench `COUNTER_NAMES`) | No direction; standard thresholds |
| Gate per-seed `final_population`; goal `final_population`, `plateau_population`, births per creature-tick per world | Move; no sign; an extinction is a blocker |
| Goal evolved half, drift depth rows, recruitment paths, lineage diversity, memory and temporal memory sensitivity, reachable structure size, structural companions, `mutation_supply` operator mix | No direction; recorded. Instrument versions stay, because production changed and the instruments did not |

No goal indicator counts undecoded parameter fields. The catalog, boundary
and draw tests are the evidence that the Goal holds.

**Measured verdict.** Both profiles ran on `07d34258`, which is rebased onto main `a9f59893` and so includes the dead-code and shared-food-config cleanups.

Gate: `make bench PROFILE=gate FEATURE=t11-f27-compact-action-parameter-storage`, `v3-cli` exit 0; outer `make` status not captured (`outer_exit` null). Against `t11-f25-meaningful-action-parameter-targets.json`, the epoch and latest closure, it is not severe and every counter is at 0.000% delta.

Goal: `make bench PROFILE=goal FEATURE=t11-f27-compact-action-parameter-storage`, `v3-cli` exit 3; `make: *** [bench] Error 3` is the recipe status, and the outer `make` status was not captured (`outer_exit` null). Exit 3 is v3-cli's "severe work-counter regression against a stored reference" (`v3-cli/src/main.rs:573-575`), and both artifacts were written.
- Against the epoch `t19-f04-vote-based-action-selection-goal.json`: severe. `vm_steps` is +134.82% and `decided_passes` +76.09%, both severe. `plasticity_updates` +42.32% and `mesh_hops` +10.51% are flagged.
- Against the latest closure `t11-f25-meaningful-action-parameter-targets-goal.json`: not severe, and every counter is `ok`. `decided_passes` is +9.46%, just under the flag, and `actions_applied` −0.44%. The other seven are within ±0.34%.
- No extinction. Final population is 4088/3221/1219 for seeds 11/22/33.

Observed outcome against the predeclaration: the expected early divergence in every world did not occur. Canyon (22) and Confluence (33) reproduce T11.F25's per-seed counters and populations exactly. Orchards (11) diverges late: final population 4877 → 4088, plateau 3873 → 3707, `decided_passes` 2339 → 2844, and its minimum population (18) is unchanged. This fits the design; it is not a failure to reach runtime. For founder-derived genomes, the representation map is behavior-preserving (Determinism row), and a changed opcode-29 `param_slot` alters behavior only when that read feeds an output. A field-operand nudge alters behavior only when the nudged write executes and is committed, and `vm_steps` counts instructions whatever their operands. The commit-path tests are the runtime evidence.

The Orchards divergence is not attributed between this feature and the rebased cleanups. The food-config cleanup declares and tests no trajectory change, and the gate plus two worlds are identical, so this feature is the likelier cause. No attribution run was made; none is required. Case `config_digest`s moved through cleanup `4e1dfbb5` (removed recipe keys), not this feature.

Decision (user decision 2026-09-24, "Accept, re-pin epoch"): the goal severe against the T19.F04 epoch is accepted, and the goal epoch is re-pinned to the T11.F27 goal summary (`epoch_baseline` in `docs/progress/benchmark-series.json`). The gate epoch stays at T11.F25. The severe is inherited from T11.F25 (`vm_steps` +134.55%, `decided_passes` +60.87% against the same epoch, accepted); this feature adds `vm_steps` +0.11% and `decided_passes` +9.46% against that closure, both under the flag threshold.

- Summaries: [gate](../../progress/features/t11-f27-compact-action-parameter-storage.json),
  [goal](../../progress/features/t11-f27-compact-action-parameter-storage-goal.json).
- Full readings: [`docs/progress/readings/t11-f27.md`](../../progress/readings/t11-f27.md).

## Success Criteria

- [x] The Graph sink catalog, the VM `WriteActionParam` address space and the
      runtime surface hold exactly the three decoded fields, tested against
      `decode_commit`, and fresh `ReadActionQueueParam` draws stay in `0..2`.
- [x] Pre-F27 serialized genomes carrying parameter structure are rejected,
      current-format genomes round-trip, replay matches, and founder behavior
      is unchanged with every re-pinned value attributed.
- [x] `make check` and the mutation gate pass with every survivor resolved,
      and the gate and goal summaries are stored and read against the
      predeclaration.

## Notes for AI Agents

- Decision: Fable credits are exhausted, so this feature's spec owner runs on Opus (Agent model `opus`, high-effort intent) instead of Fable 5.1 `high` and is resumed with `SendMessage`; no Fable advisor is used anywhere in the run, implementers included (workflow launch step 3 is skipped).
