# T13.F05 — Function-Preserving Module Recruitment readings

Tables and transcripts for the feature spec
[`docs/specs/roadmap/t13-f05-function-preserving-module-recruitment.md`](../../specs/roadmap/t13-f05-function-preserving-module-recruitment.md).
Every path is replayed by `crates/v3-core/src/neighborhood/recruitment_paths/qualification.rs`
(`qualified_paths()`), whose tests pin the operator/seed record below.

## Path table

Seeds are the first in `0..10_000` whose applied production event matches the
step's structural acceptance predicate; the search applies the operator only,
and the accepted genome is then evaluated. Start score is 4/8 on the active
task for every form. "Neutral mechanism" is why the steps before the last keep
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
| vm_blank | A | InputRef.Add:1, VmInstructionMutation:238, then exhausted at `skip_when_zero` | 7 planned | scaffold not dispatched until the swap | growth gap: planned path of 7; seed range exhausted (below) |
| vm_detour | A | InputRef.Add:1, VmInstructionMutation:238, VmInstructionMutation:800, then exhausted at `write_direction` | 6 planned | dispatched every tick; every inserted instruction is neutral until `PushAction` (the last event) | not qualified: seed range exhausted (below) |

Detour creation: production `Topology.AddNode` (`apply_splice_node`) on the
entry-to-incumbent edge of the Task A dead-incumbent base; seed 1 draws the
blank Graph backend, seed 0 the minimal VM backend. Both detours are
dispatched every tick and forward to the incumbent.

Six of nine forms qualify. No production operator was changed: every
transition the paths need is inside an existing operator's site set and draw
range.

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
| vm_detour | start (AddNode) | TopologyAddNode | 0 | 4/8 | 21 | 24 | 0 | 395.58 |
| vm_detour | cue_added | InputRefAdd | 1 | 4/8 | 22 | 24 | 0 | 395.58 |
| vm_detour | read_cue | VmInstructionMutation | 238 | 4/8 | 23 | 32 | 0 | 395.58 |
| vm_detour | double_to_east | VmInstructionMutation | 800 | 4/8 | 24 | 40 | 0 | 395.58 |

The graph relax iterations of the Graph detour rise from 8 to 16 once the
detour carries a compute node (a wired zero-compute visit pays one
node-equivalent, T13.F04); the VM detour pays one VM step per inserted
instruction per scene. No charge made a subject task-dead.

## Growth gaps and range exhaustion

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

The Graph blank path is complete and reaches 8/8 (a growth gap, not a
missing transition). The VM blank path and the six-event VM detour path
(`InputRef.Add`, `ReadInput`, `Add`, `WriteWorldActionMeta`, `JumpIfZero`,
`PushAction(2)` last, with the jump's offset drawn so that a zero cue lands on
the doubling, the meta write or the Halt at both lengths five and six) exhaust
the 10,000-seed range. Every needed instruction is inside
`random_vm_instruction`'s draw (41 kinds; `JumpIfZero` offset in `-16..=16`;
`WriteWorldActionMeta` slot in `0..8`; `PushAction` action type as a full
`u8`), so by the spec's definition no transition is missing; the draws are
rare. Enumerated per-seed odds, with three applicable VM nodes (entry,
incumbent, scaffold), the 1/3 insert choice and the uniform position draw:

| Step | Program before | Draw | Odds per seed | Expected seeds |
| --- | --- | --- | --- | --- |
| vm_blank `skip_when_zero` | `[ReadInput, Halt]` | node 1/3, insert 1/3, position 1/3, kind 1/41, offset 6/33 (3 mod 6) | 1/6,100 | 6,100 (exhausted at 10,000) |
| vm_detour `write_direction` | `[ReadInput, Add, Halt]` | node 1/3, insert 1/3, position 1/4, kind 1/41, slot 1/8 | 1/11,808 | 11,808 (exhausted) |
| vm_detour `skip_when_zero` | `[ReadInput, Add, WriteWorldActionMeta, Halt]` | node 1/3, insert 1/3, position 1/5, kind 1/41, offset 4/33 | 1/15,200 | 15,200 (not reached) |
| either `push_move` | five instructions | node 1/3, insert 1/3, position 1/6, kind 1/41, action type 1/256 | 1/566,784 | 566,784 (not reached) |

`PushAction { action_type }` is drawn as a full `u8` while only 0..=4 decode to
an action (`runtime/action_decode.rs`), so an insert event that lands on the
module draws a Move push once in 41 x 256 = 10,496 draws; on the copy forms
the push is inherited and the path is two events. Narrowing that
draw would be a production draw change, not a site or draw extension, and is
reported rather than built.

## Build-pass verification

| Command | Result |
| --- | --- |
| `cargo test -p v3-core recruitment_paths` | ok: 25 lib tests (5 new), 3 integration tests (1 new), 0 failed |
| `cargo test -p v3-core --test recruitment_paths` | ok, 3 passed |
| `cargo check --workspace --all-targets` | clean |
| `cargo clippy --workspace --all-targets` | clean, no warnings |
| `cargo fmt --all --check` | clean |
| `make roadmap-check` | pass |

Production code unchanged except visibility: `mutation/engine/mod.rs` makes
its four `*_operator_key` functions `pub(crate)` so the harness records the
same `MutationOperator` the engine records; no behavior, draw or RNG
consumption changed, so no viability run, re-pin or reference update applies.
