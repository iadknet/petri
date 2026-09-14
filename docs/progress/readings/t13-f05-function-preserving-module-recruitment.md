# T13.F05 — Function-Preserving Module Recruitment readings

Tables and transcripts for the feature spec
[`docs/specs/roadmap/t13-f05-function-preserving-module-recruitment.md`](../../specs/roadmap/t13-f05-function-preserving-module-recruitment.md).
Every path is replayed by `crates/v3-core/src/neighborhood/recruitment_paths/qualification.rs`
(`qualified_paths()`), whose tests pin the operator/seed record below.

## Path table

Seeds are the first in `0..1_000_000` whose applied production event matches
the step's structural acceptance predicate (the one-off search under "Seed
search" below); the maintained tests replay the pinned seed only. Start score
is 4/8 on the active task for every form. "Neutral mechanism" is why the steps before the last keep
the incumbent's battery signature and its per-scene actions, shared memory
and routing.

| Form | Task | Events (operator:seed) | Length | Neutral mechanism | Outcome |
| --- | --- | --- | --- | --- | --- |
| graph_copy | A | AddGraphEdge:102, SwapRouteTargets:0 | 2 | scaffold not dispatched until the swap | qualified, 8/8, no operator changed |
| graph_split | A | AddGraphEdge:1762, SwapRouteTargets:0 | 2 | scaffold not dispatched until the swap | qualified, 8/8, no operator changed |
| vm_copy | A | VmDeleteInstruction:1, SwapRouteTargets:0 | 2 | scaffold not dispatched until the swap | qualified, 8/8, no operator changed |
| graph_unprepared | B | InputRef.Swap:25, RetargetGraphEdge:32, AddInternalGraphNode:64, AddGraphEdge:6718, RetargetGraphEdge:4, SwapRouteTargets:0 | 6 | scaffold not dispatched until the swap | qualified, 8/8, no operator changed |
| vm_unprepared | B | InputRef.Swap:25, VmInstructionRawFieldMutation:72, VmInstructionRawFieldMutation:223, VmConstantMutation:13, VmConstantMutation:13, SwapRouteTargets:0 | 6 | scaffold not dispatched until the swap | qualified, 8/8, no operator changed |
| graph_detour | A | InputRef.Add:1, MutateActionSlotBehavior:25, AddInternalGraphNode:1020, AddGraphEdge:1650, AddGraphEdge:3612, AddGraphEdge:102 | 6 | dispatched every tick; action slot 0 has no gate edge until the last event, so it never fires | qualified, 8/8, no operator changed |
| graph_blank | A | InputRef.Add:1, MutateActionSlotBehavior:25, AddInternalGraphNode:1020, AddGraphEdge:1650, AddGraphEdge:3612, AddGraphEdge:102, SwapRouteTargets:0 | 7 | scaffold not dispatched until the swap | growth gap: complete path of 7, 8/8 |
| vm_blank | A | InputRef.Add:1, VmInstructionMutation:238, VmInstructionMutation:9940, VmInstructionMutation:800, VmInstructionMutation:41854, VmInstructionMutation:4126, SwapRouteTargets:0 | 7 | scaffold not dispatched until the swap | growth gap: complete path of 7, 8/8 |
| vm_detour | A | InputRef.Add:1, VmInstructionMutation:238, VmInstructionMutation:800, VmInstructionMutation:21017, VmInstructionMutation:3709, VmInstructionMutation:4126 | 6 | dispatched every tick; the jump lands on the Halt, so every inserted instruction is neutral until `PushAction(Move)` (the last event) | qualified, 8/8, `PushAction` draw repaired |

Detour creation: production `Topology.AddNode` (`apply_splice_node`) on the
entry-to-incumbent edge of the Task A dead-incumbent base; seed 1 draws the
blank Graph backend, seed 0 the minimal VM backend. Both detours are
dispatched every tick and forward to the incumbent.

Seven of nine forms qualify; the two blanks are complete seven-event growth
gaps. One production draw changed: `random_vm_instruction` draws
`PushAction.action_type` in `0..=4` (`MAX_DECODED_ACTION_TYPE`), the range
`decode_world_action` admits, instead of the full `u8` ("Seed search" below).

## Per-step readings

Columns: active-task score, genome size, VM steps and graph relax iterations
summed over the eight scenes, ending energy summed over the eight scenes.
Every step before the last is task-live with `incumbent_actions_unchanged`
and unchanged per-scene actions, shared memory and routing; the last step of
each complete path dispatches node 2, is useful by `static_successor_bypass`
(bypass 4/8 against 8/8) and is one edit on one node.

| Form | Step | Operator | Seed | Score | Size | VM steps | Graph iters | Energy |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| graph_copy | start (F02 dormant copy) | | 7 | 4/8 | 22 | 8 | 8 | 395.58 |
| graph_copy | gate_edge_added | GraphAddGraphEdge | 102 | 4/8 | 23 | 8 | 8 | 395.58 |
| graph_copy | activated | TopologySwapRouteTargets | 0 | 8/8 | 23 | 8 | 8 | 394.98 |
| graph_split | start (F02 neutral split) | | 7 | 4/8 | 24 | 8 | 8 | 395.58 |
| graph_split | gate_edge_added | GraphAddGraphEdge | 1762 | 4/8 | 25 | 8 | 8 | 395.58 |
| graph_split | activated | TopologySwapRouteTargets | 0 | 8/8 | 25 | 8 | 8 | 394.98 |
| vm_copy | start (F02 dormant copy) | | 7 | 4/8 | 34 | 16 | 0 | 395.57 |
| vm_copy | leading_halt_removed | VmDeleteInstruction | 1 | 4/8 | 33 | 16 | 0 | 395.57 |
| vm_copy | activated | TopologySwapRouteTargets | 0 | 8/8 | 33 | 64 | 0 | 394.97 |
| graph_unprepared | start (F02 unprepared) | | | 4/8 | 25 | 8 | 8 | 394.98 |
| graph_unprepared | cue_swapped | InputRefSwap | 25 | 4/8 | 25 | 8 | 8 | 394.98 |
| graph_unprepared | cue_edge_retargeted | GraphRetargetGraphEdge | 32 | 4/8 | 25 | 8 | 8 | 394.98 |
| graph_unprepared | direction_node | GraphAddInternalGraphNode | 64 | 4/8 | 27 | 8 | 8 | 394.98 |
| graph_unprepared | direction_doubled | GraphAddGraphEdge | 6718 | 4/8 | 28 | 8 | 8 | 394.98 |
| graph_unprepared | direction_read | GraphRetargetGraphEdge | 4 | 4/8 | 28 | 8 | 8 | 394.98 |
| graph_unprepared | activated | TopologySwapRouteTargets | 0 | 8/8 | 28 | 8 | 8 | 394.98 |
| vm_unprepared | start (F02 unprepared) | | | 4/8 | 33 | 64 | 0 | 394.97 |
| vm_unprepared | cue_swapped | InputRefSwap | 25 | 4/8 | 33 | 64 | 0 | 394.97 |
| vm_unprepared | read_sub_idx_1 | VmInstructionRawFieldMutation | 72 | 4/8 | 33 | 64 | 0 | 394.97 |
| vm_unprepared | read_sub_idx_2 | VmInstructionRawFieldMutation | 223 | 4/8 | 33 | 64 | 0 | 394.97 |
| vm_unprepared | direction_half | VmConstantMutation | 13 | 4/8 | 33 | 64 | 0 | 394.97 |
| vm_unprepared | direction_east | VmConstantMutation | 13 | 4/8 | 33 | 64 | 0 | 394.97 |
| vm_unprepared | activated | TopologySwapRouteTargets | 0 | 8/8 | 33 | 64 | 0 | 394.97 |
| graph_detour | start (AddNode) | TopologyAddNode | 1 | 4/8 | 14 | 8 | 8 | 395.59 |
| graph_detour | cue_added | InputRefAdd | 1 | 4/8 | 15 | 8 | 8 | 395.59 |
| graph_detour | slot_emits_move | GraphMutateActionSlotBehavior | 25 | 4/8 | 15 | 8 | 8 | 395.59 |
| graph_detour | direction_node | GraphAddInternalGraphNode | 1020 | 4/8 | 17 | 8 | 16 | 395.59 |
| graph_detour | direction_doubled | GraphAddGraphEdge | 1650 | 4/8 | 18 | 8 | 16 | 395.59 |
| graph_detour | direction_read | GraphAddGraphEdge | 3612 | 4/8 | 20 | 8 | 16 | 395.58 |
| graph_detour | gate_edge_added | GraphAddGraphEdge | 102 | 8/8 | 21 | 8 | 16 | 394.98 |
| graph_blank | start (F02 blank) | | | 4/8 | 14 | 8 | 8 | 395.59 |
| graph_blank | cue_added .. gate_edge_added | as graph_detour | 1, 25, 1020, 1650, 3612, 102 | 4/8 | 15..21 | 8 | 8 | 395.59..395.58 |
| graph_blank | activated | TopologySwapRouteTargets | 0 | 8/8 | 21 | 8 | 8 | 394.98 |
| vm_blank | start (F02 blank) | | | 4/8 | 21 | 16 | 0 | 395.58 |
| vm_blank | cue_added | InputRefAdd | 1 | 4/8 | 22 | 16 | 0 | 395.58 |
| vm_blank | read_cue | VmInstructionMutation | 238 | 4/8 | 23 | 16 | 0 | 395.58 |
| vm_blank | skip_when_zero | VmInstructionMutation | 9940 | 4/8 | 24 | 16 | 0 | 395.58 |
| vm_blank | double_to_east | VmInstructionMutation | 800 | 4/8 | 25 | 16 | 0 | 395.58 |
| vm_blank | write_direction | VmInstructionMutation | 41854 | 4/8 | 26 | 16 | 0 | 395.58 |
| vm_blank | push_move | VmInstructionMutation | 4126 | 4/8 | 27 | 16 | 0 | 395.58 |
| vm_blank | activated | TopologySwapRouteTargets | 0 | 8/8 | 27 | 44 | 0 | 394.98 |
| vm_detour | start (AddNode) | TopologyAddNode | 0 | 4/8 | 21 | 24 | 0 | 395.58 |
| vm_detour | cue_added | InputRefAdd | 1 | 4/8 | 22 | 24 | 0 | 395.58 |
| vm_detour | read_cue | VmInstructionMutation | 238 | 4/8 | 23 | 32 | 0 | 395.58 |
| vm_detour | double_to_east | VmInstructionMutation | 800 | 4/8 | 24 | 40 | 0 | 395.58 |
| vm_detour | write_direction | VmInstructionMutation | 21017 | 4/8 | 25 | 48 | 0 | 395.58 |
| vm_detour | skip_when_zero | VmInstructionMutation | 3709 | 4/8 | 26 | 48 | 0 | 395.58 |
| vm_detour | push_move | VmInstructionMutation | 4126 | 8/8 | 27 | 52 | 0 | 394.98 |

The graph relax iterations of the Graph detour rise from 8 to 16 once the
detour carries a compute node (a wired zero-compute visit pays one
node-equivalent, T13.F04); the VM detour pays one VM step per inserted
instruction per scene until the jump, after which the four zero-cue scenes
halt at the jump. No charge made a subject task-dead.

## Growth gaps

**Direction on the Graph backend.** A Graph `Emit(Move)` slot decodes its
direction from `param_inputs[0]` alone (`runtime/cgp/effects.rs`,
`decode_action_from_kind`: parameters are positional, not summed), `AddGraphEdge`
draws its weight in `[-1, 1]`, and every input leaf and compute kind yields at
most the cue value 1.0 from one input, so east (code 2, decoded from
`[1.5, 2.5)`) needs a compute node summing two cue edges: `AddInternalGraphNode`
bootstrap form (one cue edge, weight >= 0.9), `AddGraphEdge` onto that node
(sum >= 1.7), then the parameter edge reading the node (product in
`[1.5, 2.5)`) or, on the unprepared copy, `RetargetGraphEdge` of the existing
parameter edge onto it. `MutateGraphOperatorParam` moves a `Constant` by at most
0.1 per event, so nudging the copy's zeroed constant back to 2 would take at
least fifteen events; the three-event node is the shortest route found.

| Form | Shortest path | Length | Lengthening step |
| --- | --- | --- | --- |
| graph_blank | the six graph_detour events plus `SwapRouteTargets` | 7 | activation: undispatched tissue needs the route swap on top of the sensor, the Move behavior, the three-event direction node and the gate edge |
| vm_blank | `InputRef.Add`, five `VmInstructionMutation` inserts (`ReadInput`, `JumpIfZero`, `Add`, `WriteWorldActionMeta`, `PushAction(2)`), `SwapRouteTargets` | 7 | activation: the one-register Task A program needs five instructions and each VM event inserts at most one of them (the read/store, read/bid and load/compare motifs carry none of the jump, meta write or push) |

Both blank paths are complete and reach 8/8: growth gaps, not missing
transitions.

## Seed search

The one-off search is the ignored test
`recruitment_paths_seed_search_finds_the_pinned_seeds`
(`cargo test --release -p v3-core --lib -- --ignored --nocapture
recruitment_paths_seed_search`, about 3 minutes including the build): it walks
every form's plan taking the first accepted seed per step in `0..1_000_000`
and asserts the result equals the pinned record. The seven Graph, copy, split
and unprepared forms keep the seeds the 10,000-seed range found. The VM
insert steps need the draws below; odds are enumerated per seed with three
applicable VM nodes (entry, incumbent, scaffold), the 1/3 insert choice, the
uniform position draw over `len + 1` slots, the 1/41 kind draw and the
kind's own field draws (register count 1, so every register field is 0).

| Step | Program before | Draw | Odds per seed | Expected seeds | First seed |
| --- | --- | --- | --- | --- | --- |
| vm_blank `skip_when_zero` | `[ReadInput, Halt]` | position 1/3, `JumpIfZero` offset landing on the Halt: 0 mod 3, 11/33 | 1/3,321 | 3,321 | 9,940 |
| vm_blank `double_to_east` | `[ReadInput, JumpIfZero, Halt]` | position 1/4, `Add` | 1/1,476 | 1,476 | 800 |
| vm_blank `write_direction` | four instructions | position 1/5, `WriteWorldActionMeta` slot 1/8 | 1/14,760 | 14,760 | 41,854 |
| vm_blank `push_move` | five instructions | position 1/6, `PushAction` action type 1/5 | 1/11,070 | 11,070 | 4,126 |
| vm_detour `write_direction` | `[ReadInput, Add, Halt]` | position 1/4, slot 1/8 | 1/11,808 | 11,808 | 21,017 |
| vm_detour `skip_when_zero` | `[ReadInput, Add, WriteWorldActionMeta, Halt]` | position 1/5, offset landing on the Halt: 2 mod 5, 6/33 | 1/10,148 | 10,148 | 3,709 |
| vm_detour `push_move` | five instructions | position 1/6, action type 1/5 | 1/11,070 | 11,070 | 4,126 |

**Repaired draw.** `random_vm_instruction` drew `PushAction { action_type }`
over the full `u8` while `decode_world_action` (`runtime/action_decode.rs`)
admits only 0..=4 and treats the rest as a soft `NoOp`, so a Move push had
odds 1/256 per `PushAction` draw (1/566,784 per seed on the push step): a
transition missing in practice under the spec's "Missing transition". The
draw is now `rng.gen_range(0..=MAX_DECODED_ACTION_TYPE)`
(`mutation/vm/operators.rs`, constant in `runtime/action_decode.rs`), pinned by
the property test `random_vm_instruction_never_draws_an_undecodable_action_type`
and the coverage test `random_vm_instruction_reaches_every_decodable_action_type`
(which re-pins the former `random_vm_instruction_widens_push_action_range`,
whose "above 3" expectation encoded the full-`u8` draw). Existing genome
values above 4 and `VmInstructionRawFieldMutation`'s unit step are unchanged;
the reference paragraph is `docs/reference/v3-mutation-spec.md` §3, VM domain.
RNG consumption changes on every opcode-27 draw; no other pinned expectation
in `v3-core` or `v3-cli` moved.

**Jump acceptance.** The build pass expected a drawn jump offset to survive
later inserts, but `VmInstructionMutation` inserts go through
`splice_program_with_reference_repair` (T11.F02), which rewrites every
surviving jump's offset so it keeps its resolved target; a path step's
acceptance is therefore "the `JumpIfZero` resolves to the closing Halt"
(`Expect::JumpToHalt`, via `runtime::vm::jump_target`) at every length, and
the pinned VM detour's last event repairs the jump from offset 2 to 3 while
inserting the push. That repair is what keeps the detour neutral: a zero cue
halts at the jump both before and after the push.

## Verification

| Command | Result |
| --- | --- |
| `cargo test -p v3-core --test viability` (before and after the draw change) | ok, 24 passed |
| `cargo test -p v3-core` | ok: 1436 lib tests passed (3 ignored), every integration binary passed, 0 failed |
| `cargo test -p v3-core --test reproducibility` | ok, 3 passed |
| `cargo test -p v3-cli` | ok: 101 + 11 + 20 + 20 + 11 passed, 0 failed |
| `cargo test -p v3-core recruitment_paths` | ok: 25 lib tests (1 ignored search), 3 integration tests, 0 failed |
| `cargo test --release -p v3-core --lib -- --ignored recruitment_paths_seed_search` | ok, 1 passed: the search reproduces every pinned seed. Run before the harness simplification; it still holds because the acceptance predicates are unchanged in meaning (the derived expected programs equal the former literals, as the pinned record test replaying every seed against the current predicates shows), so the first accepted seed per step cannot have moved |
| `cargo check --workspace --all-targets` | clean |
| `cargo clippy --workspace --all-targets` | clean, no warnings |
| `cargo fmt --all --check` | clean |
| `make roadmap-check` | pass |

The table reflects the current harness, seed-search row excepted.

Production changes: the `PushAction.action_type` draw above and the
`mutation/engine/mod.rs` visibility change (four `*_operator_key` functions
`pub(crate)` so the harness records the same `MutationOperator` the engine
records). Weights, biases, selectors and the F02 experiment are unchanged.

## Benchmark readings (2026-09-14, tested commit `ba3267d1`)

**Gate** — `make bench PROFILE=gate FEATURE=t13-f05-function-preserving-module-recruitment`,
exit 0, `severe=false`. Raw `.bench-artifacts/t13-f05-function-preserving-module-recruitment/gate.json`
(94,071 bytes), summary `docs/progress/features/t13-f05-function-preserving-module-recruitment.json`
(96,932 bytes). Vs epoch `remove-complementary-nutrition.json`: all six counters `level=ok`
(largest move plasticity_updates −27.1%, a decrease, no flag). Vs previous
`t13-f04-direct-graph-effect-activation.json`: all six counters byte-identical
(delta% 0.000000), `level=ok`.

**Goal** — `make bench PROFILE=goal FEATURE=t13-f05-function-preserving-module-recruitment`,
exit 0, `severe=false`. Raw `.bench-artifacts/t13-f05-function-preserving-module-recruitment/goal.json`
(350,790,069 bytes, sha256 `2b5be7b4…`), summary
`docs/progress/features/t13-f05-function-preserving-module-recruitment-goal.json`
(4,161,635 bytes). Wall caps: founder 128.8 ms (<10 s), evolved-neighborhood
614.0 ms total across 3 seeds (<180 s), goal total 706,145.8 ms ≈ 11.77 min
(<15 min). Vs epoch `t13-f03-mutation-target-applicability-goal.json`: all six
counters `level=ok`. Vs previous `t13-f04-direct-graph-effect-activation-goal.json`:
five counters `level=ok`; `plasticity_updates` +22.75% (current 0.097385 vs
0.079336), `level=flag` — inside the +10%/+50% work band (not severe), and
attributable to the repaired opcode-27 draw per the predeclaration's "with a
repair, no predeclared direction, inside the flags."

Founder-neighborhood predeclaration check (all 3 seeds, founder side of
`mutational_neighborhood` byte-identical to T13.F04's report):
- `VmInstructionMutation` silent share: 0.420 (seed 11), 0.400 (seed 22),
  0.420 (seed 33) — unchanged from T13.F04 (fall = 0, within "at most ~1/41").
- Per-birth (`any_events`) dead fraction: 0.000 all seeds — within the ≤5%
  floor (d).
- Per-birth (`any_events`) silent fraction: 0.543269 (seed 11), 0.576923
  (seed 22), 0.543269 (seed 33) — below the predeclared ≥60% floor (e). This
  reading is byte-identical to T13.F04's own report (same field, same
  values), so it is an inherited condition, not a regression introduced by
  this feature; flagged here because the spec's own floor language names 60%
  and the reading does not fit it. Escalate to the orchestrator rather than
  resolving here.

Drift-depth predeclaration check (`changed_per_all_births`, all 3 seeds,
`drift_depth` block byte-identical to T13.F04's report at every checkpoint):
- Depth 1,000: 0.0045 / 0.0085 / 0.0045 — all above the 0.0015 floor.
- Depth 2,000: 0.0035 / 0.0045 / 0.0035 (7/9/7 per 2,000) — all below the
  0.005 floor. This is exactly the predeclaration's "without a repair"
  branch ("equal T13.F04's (7/9/7 per 2,000, below the floor) and are
  escalated as such"), even though a repair did land elsewhere: the
  drift-depth battery's draws were not touched by the `PushAction` fix, so
  the reading did not move. Per the spec's own wording this is escalated,
  not silently accepted; T13.F03/F04's acceptances of sub-0.005 readings do
  not extend to this closure.

Other predeclared items, checked byte-for-byte against T13.F04's goal report:
T13.F01 rungs/backends (`graph`/`vm` contributing, executed, total per
checkpoint) identical; T13.F02 `recruitment_paths.total_proposals` (55,296)
and `opportunities.attempted` (30,726) identical; per-arm/per-pair detail and
the `mutational_neighborhood.evolved` block differ from T13.F04 (expected:
evolved trajectories consume RNG differently after the opcode-27 repair, "no
floor" per the predeclaration).

**Measured verdict.** Gate: pass, no flags, no severe. Goal: pass, no severe;
one work-band flag (`plasticity_updates`, expected under the repair). Two
predeclared items do not fit their stated bound: the founder per-birth silent
fraction (~54–58% vs ≥60%) and the depth-2,000 drift readings (below 0.005),
both inherited unchanged from T13.F04 and both explicitly named for
escalation by the predeclaration's own text — reported to the orchestrator,
not resolved here.
