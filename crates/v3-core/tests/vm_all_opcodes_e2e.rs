//! End-to-end VM opcode coverage test.
//!
//! Uses one sample VM program containing all 43 opcodes. The program's final
//! branch reads `FoodHere`: with food it executes `PushAction` + `ExecuteActionQueue`,
//! without food it executes `Halt`. Running both scenarios yields full opcode coverage
//! through the simulation tick path.

use std::collections::HashSet;
use std::mem::{discriminant, Discriminant};

use slotmap::SlotMap;
use v3_core::config::SimulationConfig;
use v3_core::contracts::{
    CreatureId, InputReference, NodeId, Position, WorldAction, WorldInputKey,
};
use v3_core::creature::genome::{
    BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
};
use v3_core::creature::identity::CreatureIdentityState;
use v3_core::creature::state::CreatureState;
use v3_core::kernel::WorldState;
use v3_core::runtime::trace::domain::{BackendTrace, TickTrace, VmTrace};
use v3_core::runtime::trace::recording::ActiveTrace;
use v3_core::simulation::{run_tick, Simulation};

const FOUNDER_CHANNELS: [u8; 6] = [0, 0, 92, 92, 138, 138];
const FOUNDER_ACTIVE_CHANNEL: usize = 0;
const FOUNDER_POLARITY: [bool; 6] = [true; 6];

fn sample_vm_program() -> Vec<VmInstruction> {
    vec![
        // ── Shared memory slots + arithmetic + logical opcodes ─────────────
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        }, // r0 = 5.0 (slot addr)
        VmInstruction::LoadConst {
            dst: 1,
            const_idx: 1,
        }, // r1 = 77.0 (value)
        VmInstruction::StoreSlot {
            slot_reg: 0,
            src: 1,
        },
        VmInstruction::LoadSlot {
            dst: 2,
            slot_reg: 0,
        },
        VmInstruction::StoreSlotImm {
            slot_idx: 9,
            src: 1,
        },
        VmInstruction::LoadSlotImm {
            dst: 3,
            slot_idx: 9,
        },
        VmInstruction::LoadSlotPrev {
            dst: 3,
            slot_idx: 9,
        },
        VmInstruction::ClearSlot { slot_idx: 9 },
        VmInstruction::Move { dst: 4, src: 2 },
        VmInstruction::Add { dst: 5, a: 2, b: 3 },
        VmInstruction::Sub { dst: 6, a: 5, b: 3 },
        VmInstruction::Mul { dst: 7, a: 6, b: 1 },
        VmInstruction::Div { dst: 8, a: 7, b: 1 },
        VmInstruction::Min { dst: 9, a: 8, b: 1 },
        VmInstruction::Max {
            dst: 10,
            a: 8,
            b: 1,
        },
        VmInstruction::Abs { dst: 11, src: 10 },
        VmInstruction::Neg { dst: 11, src: 11 },
        VmInstruction::Clamp01 { dst: 11, src: 11 },
        VmInstruction::CmpGt {
            dst: 12,
            a: 10,
            b: 9,
        },
        VmInstruction::CmpLt {
            dst: 13,
            a: 9,
            b: 10,
        },
        VmInstruction::LoadConst {
            dst: 14,
            const_idx: 2,
        }, // r14 = eps = 0.1
        VmInstruction::CmpEq {
            dst: 15,
            a: 9,
            b: 9,
            eps: 14,
        },
        VmInstruction::And {
            dst: 12,
            a: 12,
            b: 13,
        },
        VmInstruction::Or {
            dst: 13,
            a: 12,
            b: 15,
        },
        VmInstruction::Not { dst: 13, src: 13 },
        VmInstruction::ToI32 { dst: 14, src: 10 },
        VmInstruction::ToU8 { dst: 14, src: 14 },
        VmInstruction::ToBool { dst: 15, src: 13 },
        VmInstruction::Noop,
        VmInstruction::Jump { offset: 1 }, // skip next LoadConst
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 1,
        }, // r0 would become 77.0 if Jump failed
        // ── Output/routing writes ───────────────────────────────────────────
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 10,
        },
        VmInstruction::WriteWorldActionMeta {
            slot_idx: 0,
            src: 15,
        },
        VmInstruction::WriteDirectionBid {
            direction: 0,
            src: 15,
        },
        // Inert vote surface (T19.F03): executed and costed, read by nothing.
        VmInstruction::AddVote { sink: 0, src: 15 },
        VmInstruction::WriteRouteGate { slot: 0, src: 0 },
        // ── Priority bid ────────────────────────────────────────────────────
        VmInstruction::SetPriorityBid { src: 13 }, // r13 = 0.0, so no energy deducted
        // ── Action queue introspection opcodes ──────────────────────────────
        VmInstruction::PushAction { action_type: 0 }, // push test NoOp action
        VmInstruction::ReadActionQueueLength { dst: 14 }, // r14 = 1.0
        VmInstruction::ReadActionQueueType {
            index_src: 14,
            dst: 14,
        },
        VmInstruction::ReadActionQueueParam {
            index_src: 14,
            param_slot: 0,
            dst: 14,
        },
        VmInstruction::PopAction, // remove the test action
        // ── Branch to terminal opcode based on FoodHere ─────────────────────
        VmInstruction::ReadInput {
            dst: 15,
            ref_idx: 0,
            sub_idx: 0,
        }, // FoodHere
        VmInstruction::ToBool { dst: 15, src: 15 },
        VmInstruction::JumpIfZero {
            cond: 15,
            offset: 2,
        }, // jump to Halt when no food
        VmInstruction::PushAction { action_type: 1 }, // Eat when food exists
        VmInstruction::ExecuteActionQueue,            // terminal: return queue
        VmInstruction::Noop,
        VmInstruction::Halt,
    ]
}

fn sample_vm_genome() -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![InputReference::World(WorldInputKey::FoodHere {
                type_idx: v3_core::config::OrdinaryFoodTypeId::default(),
            })],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 16,
                constants: vec![5.0, 77.0, 0.1],
                program: sample_vm_program(),
            }),
            targets: vec![],
        }],
    }
}

fn expected_all_opcode_discriminants() -> HashSet<Discriminant<VmInstruction>> {
    let instructions = vec![
        VmInstruction::Noop,
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        },
        VmInstruction::Move { dst: 0, src: 1 },
        VmInstruction::Add { dst: 0, a: 1, b: 2 },
        VmInstruction::Sub { dst: 0, a: 1, b: 2 },
        VmInstruction::Mul { dst: 0, a: 1, b: 2 },
        VmInstruction::Div { dst: 0, a: 1, b: 2 },
        VmInstruction::Min { dst: 0, a: 1, b: 2 },
        VmInstruction::Max { dst: 0, a: 1, b: 2 },
        VmInstruction::Abs { dst: 0, src: 1 },
        VmInstruction::Neg { dst: 0, src: 1 },
        VmInstruction::Clamp01 { dst: 0, src: 1 },
        VmInstruction::CmpGt { dst: 0, a: 1, b: 2 },
        VmInstruction::CmpLt { dst: 0, a: 1, b: 2 },
        VmInstruction::CmpEq {
            dst: 0,
            a: 1,
            b: 2,
            eps: 3,
        },
        VmInstruction::And { dst: 0, a: 1, b: 2 },
        VmInstruction::Or { dst: 0, a: 1, b: 2 },
        VmInstruction::Not { dst: 0, src: 1 },
        VmInstruction::ToI32 { dst: 0, src: 1 },
        VmInstruction::ToU8 { dst: 0, src: 1 },
        VmInstruction::ToBool { dst: 0, src: 1 },
        VmInstruction::JumpIfZero { cond: 0, offset: 1 },
        VmInstruction::Jump { offset: 0 },
        VmInstruction::ReadInput {
            dst: 0,
            ref_idx: 0,
            sub_idx: 0,
        },
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 0,
        },
        VmInstruction::WriteWorldActionMeta {
            slot_idx: 0,
            src: 0,
        },
        VmInstruction::WriteDirectionBid {
            direction: 0,
            src: 0,
        },
        VmInstruction::WriteRouteGate { slot: 0, src: 0 },
        VmInstruction::PushAction { action_type: 0 },
        VmInstruction::PopAction,
        VmInstruction::ReadActionQueueLength { dst: 0 },
        VmInstruction::ReadActionQueueType {
            index_src: 0,
            dst: 0,
        },
        VmInstruction::ReadActionQueueParam {
            index_src: 0,
            param_slot: 0,
            dst: 0,
        },
        VmInstruction::SetPriorityBid { src: 0 },
        VmInstruction::ExecuteActionQueue,
        VmInstruction::Halt,
        VmInstruction::LoadSlot {
            dst: 0,
            slot_reg: 1,
        },
        VmInstruction::StoreSlot {
            slot_reg: 0,
            src: 1,
        },
        VmInstruction::LoadSlotImm {
            dst: 0,
            slot_idx: 0,
        },
        VmInstruction::StoreSlotImm {
            slot_idx: 0,
            src: 0,
        },
        VmInstruction::LoadSlotPrev {
            dst: 0,
            slot_idx: 0,
        },
        VmInstruction::ClearSlot { slot_idx: 0 },
        VmInstruction::AddVote { sink: 0, src: 0 },
    ];

    let set: HashSet<Discriminant<VmInstruction>> = instructions.iter().map(discriminant).collect();
    assert_eq!(set.len(), 43, "expected 43 unique VM opcode discriminants");
    set
}

fn build_simulation(food_here: f32) -> (Simulation, CreatureId, Position) {
    let mut cfg = SimulationConfig::default();
    cfg.world.width = 8;
    cfg.world.height = 8;
    cfg.world.food.initial_coverage = 0.0;
    cfg.world.food.growth_rate = 0.0;
    cfg.energy.lifecycle.energy_decay_per_tick = 0.0;

    let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
    world.reconfigure_food(cfg.world.food.clone());
    let pos = Position::new(3, 3);
    world.set_food(pos, food_here);

    let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
    let genome = sample_vm_genome();
    let id = creatures.insert_with_key(|cid| {
        CreatureState::new(
            cid,
            genome.clone(),
            pos,
            100.0,
            0,
            FOUNDER_CHANNELS,
            FOUNDER_ACTIVE_CHANNEL,
            FOUNDER_POLARITY,
            CreatureIdentityState::default(),
            [0.0; 16],
        )
    });
    world.place_creature(pos, id);

    (Simulation::new(world, creatures, 0, cfg, 42), id, pos)
}

fn collect_vm_discriminants(tick: &TickTrace) -> HashSet<Discriminant<VmInstruction>> {
    assert_eq!(
        tick.hops.len(),
        1,
        "single-node sample genome should execute exactly one hop"
    );
    let vm_trace = vm_trace(tick);
    vm_trace
        .steps
        .iter()
        .map(|step| discriminant(&step.instruction))
        .collect()
}

fn vm_trace(tick: &TickTrace) -> &VmTrace {
    match &tick.hops[0].backend_trace {
        BackendTrace::Vm(vm) => vm,
        BackendTrace::Graph(_) => panic!("sample genome backend must be VM"),
    }
}

fn step_pair_after<F>(trace: &VmTrace, mut pred: F) -> (usize, usize)
where
    F: FnMut(&VmInstruction) -> bool,
{
    let idx = trace
        .steps
        .iter()
        .position(|step| pred(&step.instruction))
        .expect("expected opcode in trace");
    let curr_pc = trace.steps[idx].pc;
    let next_pc = trace
        .steps
        .get(idx + 1)
        .expect("opcode should not be the final trace step")
        .pc;
    (curr_pc, next_pc)
}

#[test]
fn sample_program_exercises_all_vm_opcodes_e2e() {
    let expected = expected_all_opcode_discriminants();

    // Scenario A: food present -> PushAction+ExecuteActionQueue path (Eat).
    let (mut sim_emit, emit_id, emit_pos) = build_simulation(1.0);
    let mut emit_trace = Some(ActiveTrace::new(emit_id, 1));
    run_tick(&mut sim_emit, &mut emit_trace);
    let emit_trace = emit_trace.expect("trace should remain available");
    assert!(emit_trace.is_complete());
    assert_eq!(emit_trace.ticks.len(), 1);
    assert_eq!(
        emit_trace.ticks[0].final_actions[0],
        WorldAction::Eat {
            type_idx: v3_core::config::OrdinaryFoodTypeId::default()
        }
    );
    assert!(
        (sim_emit.world.food_at(emit_pos) - 0.0).abs() < 1e-6,
        "Eat path should consume food"
    );
    // No targets on this single-node genome, so route is None.
    assert!(emit_trace.ticks[0].hops[0].route.is_none());
    let emit_seen = collect_vm_discriminants(&emit_trace.ticks[0]);
    let emit_vm_trace = vm_trace(&emit_trace.ticks[0]);

    // Scenario B: no food -> Halt path.
    let (mut sim_halt, halt_id, halt_pos) = build_simulation(0.0);
    let mut halt_trace = Some(ActiveTrace::new(halt_id, 1));
    run_tick(&mut sim_halt, &mut halt_trace);
    let halt_trace = halt_trace.expect("trace should remain available");
    assert!(halt_trace.is_complete());
    assert_eq!(halt_trace.ticks.len(), 1);
    assert_eq!(halt_trace.ticks[0].final_actions[0], WorldAction::NoOp);
    assert!(
        (sim_halt.world.food_at(halt_pos) - 0.0).abs() < 1e-6,
        "Halt path should leave no food on an empty cell"
    );
    // No targets on this single-node genome, so route is None.
    assert!(halt_trace.ticks[0].hops[0].route.is_none());
    let halt_seen = collect_vm_discriminants(&halt_trace.ticks[0]);
    let halt_vm_trace = vm_trace(&halt_trace.ticks[0]);

    let observed: HashSet<Discriminant<VmInstruction>> =
        emit_seen.union(&halt_seen).copied().collect();
    assert_eq!(
        observed, expected,
        "sample VM program should cover all 43 opcodes across emit/halt runs"
    );

    // Verify unconditional Jump was actually taken (pc + 2 because offset=1).
    let (jump_pc_emit, after_jump_emit) =
        step_pair_after(emit_vm_trace, |i| matches!(i, VmInstruction::Jump { .. }));
    assert_eq!(
        after_jump_emit,
        jump_pc_emit + 2,
        "unconditional Jump should skip exactly one instruction"
    );

    // Verify conditional JumpIfZero branch behavior differs by scenario.
    let (jiz_pc_emit, after_jiz_emit) = step_pair_after(emit_vm_trace, |i| {
        matches!(i, VmInstruction::JumpIfZero { .. })
    });
    assert_eq!(
        after_jiz_emit,
        jiz_pc_emit + 1,
        "JumpIfZero should not jump on truthy FoodHere (emit path)"
    );

    let (jump_pc_halt, after_jump_halt) =
        step_pair_after(halt_vm_trace, |i| matches!(i, VmInstruction::Jump { .. }));
    assert_eq!(
        after_jump_halt,
        jump_pc_halt + 2,
        "unconditional Jump should skip exactly one instruction"
    );

    let (jiz_pc_halt, after_jiz_halt) = step_pair_after(halt_vm_trace, |i| {
        matches!(i, VmInstruction::JumpIfZero { .. })
    });
    assert_eq!(
        after_jiz_halt,
        jiz_pc_halt + 3,
        "JumpIfZero should jump over PushAction + ExecuteActionQueue on zero FoodHere (halt path)"
    );

    assert!(
        emit_seen.contains(&discriminant(&VmInstruction::ExecuteActionQueue)),
        "emit scenario must execute ExecuteActionQueue"
    );
    assert!(
        !emit_seen.contains(&discriminant(&VmInstruction::Halt)),
        "emit scenario should terminate at ExecuteActionQueue before Halt"
    );
    assert!(
        halt_seen.contains(&discriminant(&VmInstruction::Halt)),
        "halt scenario must execute Halt"
    );
    assert!(
        !halt_seen.contains(&discriminant(&VmInstruction::ExecuteActionQueue)),
        "halt scenario should skip ExecuteActionQueue via JumpIfZero"
    );
}
