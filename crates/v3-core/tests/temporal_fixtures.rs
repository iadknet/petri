//! T11.F05 — Temporal Controller Fixtures.
//!
//! A maintained fixture suite of small constructed controllers, run through
//! the production tick path (`Simulation` + `run_tick`) at production runtime
//! settings, that records what the brain's temporal building blocks are
//! expected to do and what they observably do. Every fixture is `#[test]`
//! green on the current code: it pins the *observed* behavior, not an
//! aspirational one. Where the observed behavior is a gap against the
//! node-type contract (every piece of persistent state advances once per
//! world tick — `docs/roadmaps/t11-brain-genotype-phenotype-map.md`), the
//! fixture's doc comment and the spec catalogue at
//! `docs/specs/roadmap/t11-f05-temporal-controller-fixtures.md` name the
//! repair feature (T11.F06 graph memory clock, T11.F07 reward trace clock)
//! that is expected to flip the assertion.
//!
//! Fixture IDs: A1 (reactive control), B1 (shared-memory delayed cue), B2
//! (previous-slot one-tick cue), C1 (retention), C2 (retention under decay),
//! C3 (graph slot write and previous read), D1 (integrator clock), D2
//! (disconnected-node perturbation), D3 (backward-edge recurrence), E1 (exact
//! one-edge update, immediate reward), E2 (delayed reward, visited every
//! tick), E3 (skipped module visits), plus one proptest for the stateful
//! compute-node convex-combination invariant.
//!
//! Settings held at production values throughout (`RuntimeConfig::default()`
//! and `shared_memory.decay_rate == 0.0`), per the spec's Inputs and
//! Invariants section. The only three permitted deviations, each labeled at
//! its use site:
//! 1. `test_config()` (shared with `creature_workflow_e2e`): tiny 12x12
//!    world, zero food coverage/growth, zero energy decay, zero mutation
//!    probability — this is world/mutation scaffolding, not a runtime or
//!    tick-loop setting.
//! 2. Fixture C2 sets `shared_memory.decay_rate = 0.1` to characterize
//!    retention under decay (contrasted with C1's production decay of 0.0).
//! 3. The proptest sets `max_graph_relax_iters = 1` to force exactly one
//!    relaxation pass per `execute_creature_mesh` call, isolating a single
//!    stateful-node step (the invariant under test) from the pass-count gap
//!    D1/D2 record.
//!
//! No production source file is edited by this feature.

mod common;

use common::{graph_hop, insert_creature, run_one_traced_tick, test_config};

use proptest::prelude::*;
use slotmap::SlotMap;
use v3_core::config::{MutationConfig, OrdinaryFoodTypeId};
use v3_core::contracts::{
    CreatureId, Direction, InputReference, NodeId, Position, RouteTarget, StaticIntrospectionKey,
    WorldAction, WorldInputKey,
};
use v3_core::creature::genome::cgp::{
    ActionSlotBehavior, CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource,
    OutputSinkKind, WorldActionKind,
};
use v3_core::creature::genome::{
    BackendDef, CreatureGenome, HebbianRule, NodeGenome, OutcomeChannel, PlasticityConfig,
    RewardModulationConfig, VmBackendDef, VmInstruction,
};
use v3_core::creature::state::{CreatureState, GraphRuntimeState};
use v3_core::kernel::WorldState;
use v3_core::runtime::mesh::execute_creature_mesh;
use v3_core::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
use v3_core::sensors::static_inputs::StaticInputs;
use v3_core::sensors::typed_food::TypedFoodLocalSnapshot;
use v3_core::simulation::Simulation;

const DIR_E: f32 = 2.0; // Direction::ALL[2] == Direction::E
const ACTION_NOOP: u8 = 0;
const ACTION_MOVE: u8 = 2;

fn food_here_ref() -> InputReference {
    InputReference::World(WorldInputKey::FoodHere {
        type_idx: OrdinaryFoodTypeId::default(),
    })
}

fn age_ticks_ref() -> InputReference {
    InputReference::StaticIntrospection(StaticIntrospectionKey::AgeTicks)
}

/// A1's genome, also used as B1's matched reactive control: read `FoodHere`
/// this tick; push `Move(E)` if it is positive, else `NoOp`. No memory.
fn reactive_control_vm_genome() -> CreatureGenome {
    let program = vec![
        VmInstruction::ReadInput {
            dst: 0,
            ref_idx: 0,
            sub_idx: 0,
        }, // r0 = FoodHere
        VmInstruction::LoadConst {
            dst: 1,
            const_idx: 0,
        }, // r1 = 0.0
        VmInstruction::CmpGt { dst: 2, a: 0, b: 1 }, // r2 = food > 0
        VmInstruction::JumpIfZero { cond: 2, offset: 4 }, // -> idx 8 (NoOp) if food <= 0
        VmInstruction::LoadConst {
            dst: 3,
            const_idx: 1,
        }, // r3 = DIR_E
        VmInstruction::WriteWorldActionMeta {
            slot_idx: 0,
            src: 3,
        },
        VmInstruction::PushAction {
            action_type: ACTION_MOVE,
        },
        VmInstruction::ExecuteActionQueue,
        VmInstruction::PushAction {
            action_type: ACTION_NOOP,
        },
        VmInstruction::ExecuteActionQueue,
    ];
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![food_here_ref()],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 4,
                constants: vec![0.0, DIR_E],
                program,
            }),
            targets: vec![],
        }],
    }
}

/// B1's genome: on any tick with `FoodHere > 0`, latch `shared_memory[0] =
/// 1.0`; every tick, read the latched slot and push `Move(E)` if positive,
/// else `NoOp`. Never eats, so food presence is purely a cue signal.
fn delayed_cue_vm_genome() -> CreatureGenome {
    let program = vec![
        VmInstruction::ReadInput {
            dst: 0,
            ref_idx: 0,
            sub_idx: 0,
        }, // idx0: r0 = FoodHere
        VmInstruction::LoadConst {
            dst: 1,
            const_idx: 0,
        }, // idx1: r1 = 0.0
        VmInstruction::CmpGt { dst: 2, a: 0, b: 1 }, // idx2: r2 = food > 0
        VmInstruction::JumpIfZero { cond: 2, offset: 2 }, // idx3: -> idx6 if food <= 0
        VmInstruction::LoadConst {
            dst: 6,
            const_idx: 1,
        }, // idx4: r6 = 1.0
        VmInstruction::StoreSlotImm {
            slot_idx: 0,
            src: 6,
        }, // idx5: slot0 = 1.0
        VmInstruction::LoadSlotImm {
            dst: 3,
            slot_idx: 0,
        }, // idx6: r3 = slot0
        VmInstruction::LoadConst {
            dst: 4,
            const_idx: 0,
        }, // idx7: r4 = 0.0
        VmInstruction::CmpGt { dst: 5, a: 3, b: 4 }, // idx8: r5 = slot0 > 0
        VmInstruction::JumpIfZero { cond: 5, offset: 4 }, // idx9: -> idx14 if slot0 <= 0
        VmInstruction::LoadConst {
            dst: 7,
            const_idx: 2,
        }, // idx10: r7 = DIR_E
        VmInstruction::WriteWorldActionMeta {
            slot_idx: 0,
            src: 7,
        }, // idx11
        VmInstruction::PushAction {
            action_type: ACTION_MOVE,
        }, // idx12
        VmInstruction::ExecuteActionQueue,           // idx13
        VmInstruction::PushAction {
            action_type: ACTION_NOOP,
        }, // idx14
        VmInstruction::ExecuteActionQueue,           // idx15
    ];
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![food_here_ref()],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 8,
                constants: vec![0.0, 1.0, DIR_E],
                program,
            }),
            targets: vec![],
        }],
    }
}

/// B2's genome: at `AgeTicks == 1`, latch `shared_memory[0] = 1.0`; at
/// `AgeTicks == 2`, clear it (so it does not propagate past one tick of
/// `LoadSlotPrev` visibility); every tick, read the *previous* tick's slot 0
/// via `LoadSlotPrev` and push `Move(E)` if positive, else `NoOp`.
fn previous_slot_cue_vm_genome() -> CreatureGenome {
    let program = vec![
        VmInstruction::ReadInput {
            dst: 0,
            ref_idx: 0,
            sub_idx: 0,
        }, // idx0: r0 = AgeTicks
        VmInstruction::LoadConst {
            dst: 1,
            const_idx: 0,
        }, // idx1: r1 = 1.0 (age target 1)
        VmInstruction::LoadConst {
            dst: 2,
            const_idx: 2,
        }, // idx2: r2 = eps 0.5
        VmInstruction::CmpEq {
            dst: 3,
            a: 0,
            b: 1,
            eps: 2,
        }, // idx3: r3 = (age == 1)
        VmInstruction::JumpIfZero { cond: 3, offset: 2 }, // idx4: -> idx7 if age != 1
        VmInstruction::LoadConst {
            dst: 4,
            const_idx: 0,
        }, // idx5: r4 = 1.0
        VmInstruction::StoreSlotImm {
            slot_idx: 0,
            src: 4,
        }, // idx6: slot0 = 1.0
        VmInstruction::LoadConst {
            dst: 5,
            const_idx: 1,
        }, // idx7: r5 = 2.0 (age target 2)
        VmInstruction::CmpEq {
            dst: 6,
            a: 0,
            b: 5,
            eps: 2,
        }, // idx8: r6 = (age == 2)
        VmInstruction::JumpIfZero { cond: 6, offset: 1 }, // idx9: -> idx11 if age != 2
        VmInstruction::ClearSlot { slot_idx: 0 },         // idx10: slot0 = 0
        VmInstruction::LoadSlotPrev {
            dst: 7,
            slot_idx: 0,
        }, // idx11: r7 = prev_slot0
        VmInstruction::LoadConst {
            dst: 8,
            const_idx: 3,
        }, // idx12: r8 = 0.0
        VmInstruction::CmpGt { dst: 9, a: 7, b: 8 },      // idx13: r9 = prev_slot0 > 0
        VmInstruction::JumpIfZero { cond: 9, offset: 4 }, // idx14: -> idx19 if prev_slot0 <= 0
        VmInstruction::LoadConst {
            dst: 10,
            const_idx: 1,
        }, // idx15: r10 = 2.0 (DIR_E)
        VmInstruction::WriteWorldActionMeta {
            slot_idx: 0,
            src: 10,
        }, // idx16
        VmInstruction::PushAction {
            action_type: ACTION_MOVE,
        }, // idx17
        VmInstruction::ExecuteActionQueue,                // idx18
        VmInstruction::PushAction {
            action_type: ACTION_NOOP,
        }, // idx19
        VmInstruction::ExecuteActionQueue,                // idx20
    ];
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![age_ticks_ref()],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 11,
                constants: vec![1.0, 2.0, 0.5, 0.0],
                program,
            }),
            targets: vec![],
        }],
    }
}

/// C1/C2's genome: at `AgeTicks == 1`, latch `shared_memory[slot] = value`;
/// every tick otherwise a no-op action. Used to characterize retention with
/// and without shared-memory decay.
fn slot_write_once_vm_genome(slot: u8, value: f32) -> CreatureGenome {
    let program = vec![
        VmInstruction::ReadInput {
            dst: 0,
            ref_idx: 0,
            sub_idx: 0,
        }, // idx0: r0 = AgeTicks
        VmInstruction::LoadConst {
            dst: 1,
            const_idx: 0,
        }, // idx1: r1 = 1.0
        VmInstruction::LoadConst {
            dst: 2,
            const_idx: 1,
        }, // idx2: r2 = eps 0.5
        VmInstruction::CmpEq {
            dst: 3,
            a: 0,
            b: 1,
            eps: 2,
        }, // idx3: r3 = (age == 1)
        VmInstruction::JumpIfZero { cond: 3, offset: 2 }, // idx4: -> idx7 if age != 1
        VmInstruction::LoadConst {
            dst: 4,
            const_idx: 2,
        }, // idx5: r4 = value
        VmInstruction::StoreSlotImm {
            slot_idx: slot,
            src: 4,
        }, // idx6: slot[slot] = value
        VmInstruction::PushAction {
            action_type: ACTION_NOOP,
        }, // idx7
        VmInstruction::ExecuteActionQueue,                // idx8
    ];
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![age_ticks_ref()],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 5,
                constants: vec![1.0, 0.5, value],
                program,
            }),
            targets: vec![],
        }],
    }
}

/// C3's genome: `Constant(0.6)` wired to `WriteSlot(0)`; a second node reads
/// `SharedMemory { slot: 0, previous: true }` and is wired to `WriteSlot(1)`.
fn slot_write_and_previous_read_graph_genome() -> CreatureGenome {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs(&MutationConfig::default());
    def.compute_nodes.push(ComputeNode {
        kind: ComputeNodeKind::Constant(0.6),
        inputs: Vec::new(),
        plasticity: None,
    });
    def.compute_nodes.push(ComputeNode {
        kind: ComputeNodeKind::Add,
        inputs: vec![GraphEdge {
            source: GraphSource::SharedMemory {
                slot: 0,
                previous: true,
            },
            weight: 1.0,
        }],
        plasticity: None,
    });
    wire_custom_write_slot(&mut def, OutputSinkKind::WriteSlot(0), 0);
    wire_custom_write_slot(&mut def, OutputSinkKind::WriteSlot(1), 1);
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Graph(def),
            targets: vec![],
        }],
    }
}

fn wire_custom_write_slot(def: &mut CgpGraphBackendDef, kind: OutputSinkKind, source_node: u16) {
    let sink = def
        .output_sinks
        .iter_mut()
        .find(|s| s.kind == kind)
        .expect("sink kind must exist in the fixed catalog");
    sink.inputs.push(GraphEdge {
        source: GraphSource::ComputeNode(source_node),
        weight: 1.0,
    });
}

/// D1's genome: `Constant(1.0)` feeding a `DecayIntegrator(0.5)`.
fn decay_integrator_graph_genome() -> CreatureGenome {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs(&MutationConfig::default());
    def.compute_nodes.push(ComputeNode {
        kind: ComputeNodeKind::Constant(1.0),
        inputs: Vec::new(),
        plasticity: None,
    });
    def.compute_nodes.push(ComputeNode {
        kind: ComputeNodeKind::DecayIntegrator(0.5),
        inputs: vec![GraphEdge {
            source: GraphSource::ComputeNode(0),
            weight: 1.0,
        }],
        plasticity: None,
    });
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Graph(def),
            targets: vec![],
        }],
    }
}

/// D2's genome: D1 plus an `Oscillator(0.25)` node with no inputs and no
/// consumers (a structurally disconnected perturbation).
fn decay_integrator_with_disconnected_oscillator_graph_genome() -> CreatureGenome {
    let mut genome = decay_integrator_graph_genome();
    let BackendDef::Graph(def) = &mut genome.nodes[0].backend_def else {
        unreachable!("constructed as a graph backend");
    };
    def.compute_nodes.push(ComputeNode {
        kind: ComputeNodeKind::Oscillator(0.25),
        inputs: Vec::new(),
        plasticity: None,
    });
    genome
}

/// D3's genome: `Constant(1.0)` feeding an `Add` node with a self-loop
/// (edges from `ComputeNode(0)` and from itself, both weight 1), wired to
/// `WriteSlot(0)`.
fn backward_edge_recurrence_graph_genome() -> CreatureGenome {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs(&MutationConfig::default());
    def.compute_nodes.push(ComputeNode {
        kind: ComputeNodeKind::Constant(1.0),
        inputs: Vec::new(),
        plasticity: None,
    });
    def.compute_nodes.push(ComputeNode {
        kind: ComputeNodeKind::Add,
        inputs: vec![
            GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            },
            GraphEdge {
                source: GraphSource::ComputeNode(1),
                weight: 1.0,
            },
        ],
        plasticity: None,
    });
    wire_custom_write_slot(&mut def, OutputSinkKind::WriteSlot(0), 1);
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Graph(def),
            targets: vec![],
        }],
    }
}

/// E1/E2's reward-modulated graph node: `Constant(1.0)` feeding an `Add`
/// node with one reward-modulated Hebbian edge (`Classic`, `learning_rate =
/// 0.5`, `trace_decay = 0.5`), wired to fire an `Eat` action and the execute
/// gate every tick.
fn reward_modulated_node_genome(reward_source: OutcomeChannel) -> CreatureGenome {
    reward_modulated_node_genome_impl(reward_source, true)
}

/// E3's reward-modulated graph node: identical compute nodes to
/// [`reward_modulated_node_genome`], but with no action bank or execute-gate
/// wiring and no route targets of its own. With no route target, the mesh
/// soft-default terminates the tick with `vec![WorldAction::NoOp]`, and
/// Phase 2 charges `noop_cost` (0.05, `crates/v3-core/src/config/
/// simulation.rs:390`) — an order of magnitude larger than the VM/graph
/// opcode costs it sits on top of (1e-6 to 1e-5 scale), so it dominates the
/// tick's `EnergyDelta` and gives a clean, always-nonzero (negative) reward
/// signal (about -0.05 per tick) independent of food placement.
fn reward_modulated_node_genome_inert(reward_source: OutcomeChannel) -> CreatureGenome {
    reward_modulated_node_genome_impl(reward_source, false)
}

fn reward_modulated_node_genome_impl(
    reward_source: OutcomeChannel,
    wire_action: bool,
) -> CreatureGenome {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs(&MutationConfig::default());
    def.compute_nodes.push(ComputeNode {
        kind: ComputeNodeKind::Constant(1.0),
        inputs: Vec::new(),
        plasticity: None,
    });
    def.compute_nodes.push(ComputeNode {
        kind: ComputeNodeKind::Add,
        inputs: vec![GraphEdge {
            source: GraphSource::ComputeNode(0),
            weight: 1.0,
        }],
        plasticity: Some(PlasticityConfig {
            rule: HebbianRule::Classic,
            learning_rate: 0.5,
            weight_clamp: 10.0,
            lamarckian: false,
            modulation: Some(RewardModulationConfig {
                reward_source,
                trace_decay: 0.5,
            }),
        }),
    });
    if wire_action {
        def.action_bank[0].behavior = ActionSlotBehavior::Emit(WorldActionKind::Eat);
        def.action_bank[0].gate_inputs.push(GraphEdge {
            source: GraphSource::ComputeNode(1),
            weight: 1.0,
        });
        def.execute_gate.inputs.push(GraphEdge {
            source: GraphSource::ComputeNode(1),
            weight: 1.0,
        });
    }
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Graph(def),
            targets: vec![],
        }],
    }
}

/// E3's entry VM node: routes to `graph_target` only on ticks where
/// `FoodHere > 0` (`WriteRouteGate` toward the graph target's slot);
/// otherwise pushes `NoOp` and executes immediately, so the mesh never
/// reaches the graph node this tick (a skipped module visit).
fn conditional_route_vm_node(
    node_id: NodeId,
    graph_target: NodeId,
    alt_target: NodeId,
) -> NodeGenome {
    let program = vec![
        VmInstruction::ReadInput {
            dst: 0,
            ref_idx: 0,
            sub_idx: 0,
        }, // idx0: r0 = FoodHere
        VmInstruction::LoadConst {
            dst: 1,
            const_idx: 0,
        }, // idx1: r1 = 0.0
        VmInstruction::CmpGt { dst: 2, a: 0, b: 1 }, // idx2: r2 = food > 0
        VmInstruction::JumpIfZero { cond: 2, offset: 3 }, // idx3: -> idx7 if food <= 0 (skip)
        VmInstruction::LoadConst {
            dst: 3,
            const_idx: 1,
        }, // idx4: r3 = 1000.0 (large gate boost)
        VmInstruction::WriteRouteGate { slot: 0, src: 3 }, // idx5: boost graph target's gate
        VmInstruction::Halt,                         // idx6: non-terminal, routes via targets
        VmInstruction::PushAction {
            action_type: ACTION_NOOP,
        }, // idx7: skip branch
        VmInstruction::ExecuteActionQueue, // idx8: terminal, mesh ends without visiting the graph node
    ];
    NodeGenome {
        node_id,
        input_refs: vec![food_here_ref()],
        backend_def: BackendDef::Vm(VmBackendDef {
            register_count: 4,
            constants: vec![0.0, 1000.0],
            program,
        }),
        targets: vec![
            RouteTarget {
                target_id: graph_target,
                slot: 0,
                gate_bias: -100.0,
            },
            RouteTarget {
                target_id: alt_target,
                slot: 1,
                gate_bias: 0.0,
            },
        ],
    }
}

fn noop_vm_node(node_id: NodeId) -> NodeGenome {
    NodeGenome {
        node_id,
        input_refs: vec![],
        backend_def: BackendDef::Vm(VmBackendDef {
            register_count: 1,
            constants: vec![],
            program: vec![
                VmInstruction::PushAction {
                    action_type: ACTION_NOOP,
                },
                VmInstruction::ExecuteActionQueue,
            ],
        }),
        targets: vec![],
    }
}

/// Build a one-creature `Simulation` at production runtime settings (plus
/// the labeled `test_config` world deviations) with `genome` at `pos`.
fn one_creature_sim(
    genome: CreatureGenome,
    pos: Position,
    energy: f32,
) -> (Simulation, CreatureId) {
    sim_with_config(genome, pos, energy, test_config())
}

/// Build a one-creature `Simulation` from an explicit `cfg`, for the one
/// fixture (C2) that needs a labeled non-default runtime setting.
fn sim_with_config(
    genome: CreatureGenome,
    pos: Position,
    energy: f32,
    cfg: v3_core::config::SimulationConfig,
) -> (Simulation, CreatureId) {
    let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
    world.reconfigure_food(cfg.world.food.clone());
    let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
    let target = insert_creature(&mut creatures, &mut world, genome, pos, energy, 0);
    let sim = Simulation::new(world, creatures, 0, cfg, 41);
    (sim, target)
}

/// Place a barrier on the cell immediately east of `pos`, so a successful
/// `Move(E)` action can never relocate the creature off `pos`. Several
/// fixtures assert on the *decided* action at a fixed position across
/// several ticks; without this, a genome that once decides `Move(E)` would
/// walk the creature away from the cell the test keeps observing.
fn block_east_neighbor(sim: &mut Simulation, pos: Position) {
    let east = sim
        .world
        .resolve_neighbor(pos, Direction::E)
        .expect("center position has an east neighbor");
    sim.world.set_barrier(east, true);
}

// ── A1: reactive control ────────────────────────────────────────────────────

/// A1 reactive control: the action tracks the current observation every
/// tick; identical observations give identical actions. Food is toggled on
/// the creature's cell across 4 ticks: present, absent, present, absent.
///
/// Expected and observed agree: this fixture has no persistent state to
/// desynchronize from the tick clock, so there is no gap to assign.
#[test]
fn a1_reactive_control() {
    let pos = Position::new(5, 5);
    let (mut sim, target) = one_creature_sim(reactive_control_vm_genome(), pos, 100.0);
    // The fixture is about the *decided* action tracking the current
    // observation, not about locomotion.
    block_east_neighbor(&mut sim, pos);

    let toggles = [true, false, true, false];
    let expected = [
        WorldAction::Move(Direction::E),
        WorldAction::NoOp,
        WorldAction::Move(Direction::E),
        WorldAction::NoOp,
    ];
    for (tick_idx, (&food_present, &want)) in toggles.iter().zip(expected.iter()).enumerate() {
        sim.world
            .set_food(pos, if food_present { 1.0 } else { 0.0 });
        let tick = run_one_traced_tick(&mut sim, target);
        assert_eq!(
            tick.final_actions[0], want,
            "tick {tick_idx}: action should track the current FoodHere observation"
        );
    }
}

// ── B1: shared-memory delayed cue ───────────────────────────────────────────

/// B1 shared-memory delayed cue: a cue presented for one tick and latched
/// into shared memory is acted on at every tested delay (1, 2, 4, 8, 16
/// ticks later), while the matched reactive control (A1) — which shares the
/// same current-tick observation at the decision tick — emits `NoOp` in both
/// trials. Status: meets. Shared-memory latching is not one of the flagged
/// clocks (graph relaxation, eligibility trace); it is a plain memory write
/// read back unconditionally, so this fixture shows no gap.
#[test]
fn b1_shared_memory_delayed_cue() {
    for delay in [1u64, 2, 4, 8, 16] {
        let decision_tick = 1 + delay;
        for (genome_ctor, is_reactive_control) in [
            (delayed_cue_vm_genome as fn() -> CreatureGenome, false),
            (reactive_control_vm_genome as fn() -> CreatureGenome, true),
        ] {
            let mut decision_snapshots = Vec::with_capacity(2);
            for cue_present in [true, false] {
                let pos = Position::new(6, 6);
                let (mut sim, target) = one_creature_sim(genome_ctor(), pos, 100.0);
                // Both genomes may emit Move(E) (the cue trial's latch never
                // clears, so it re-fires every tick after the cue). Block
                // the destination in both trials identically so the
                // creature — and thus every subsequent observation — stays
                // on `pos` regardless of trial.
                block_east_neighbor(&mut sim, pos);

                let mut decision_snapshot = None;
                for tick_num in 1..=decision_tick {
                    let food_now = cue_present && tick_num == 1;
                    sim.world.set_food(pos, if food_now { 1.0 } else { 0.0 });
                    let tick = run_one_traced_tick(&mut sim, target);
                    if tick_num == decision_tick {
                        decision_snapshot = Some(tick.static_inputs.clone());
                        let expected = if is_reactive_control {
                            // The matched reactive control reads the same
                            // current (zero) observation at the decision
                            // tick in both trials: always NoOp.
                            WorldAction::NoOp
                        } else if cue_present {
                            WorldAction::Move(Direction::E)
                        } else {
                            WorldAction::NoOp
                        };
                        assert_eq!(
                            tick.final_actions[0], expected,
                            "delay {delay}, cue_present {cue_present}, reactive_control \
                             {is_reactive_control}: decision-tick action"
                        );
                    }
                }
                assert_eq!(
                    sim.creatures.get(target).expect("creature alive").position,
                    pos,
                    "delay {delay}, cue_present {cue_present}: the creature must stay on \
                     `pos` for the decision-tick observation to be comparable across trials"
                );
                decision_snapshots.push(decision_snapshot.expect("decision tick ran"));
            }
            // The full static-input snapshot at the decision tick is
            // identical whether or not the cue trial happened: the memory
            // effect on `final_actions` above is not a reactive artifact of
            // some other observation differing between trials.
            // `StaticInputsSnapshot` derives no `PartialEq` (production
            // type), so compare it field by field.
            let (cue, no_cue) = (&decision_snapshots[0], &decision_snapshots[1]);
            assert_eq!(cue.food_here, no_cue.food_here);
            assert_eq!(cue.neighbor_food, no_cue.neighbor_food);
            assert_eq!(cue.neighbor_barrier, no_cue.neighbor_barrier);
            assert_eq!(cue.neighbor_occupied, no_cue.neighbor_occupied);
            assert_eq!(cue.generation, no_cue.generation);
            assert_eq!(
                cue.age_ticks, no_cue.age_ticks,
                "delay {delay}, reactive_control {is_reactive_control}: static_inputs at the \
                 decision tick must be identical across the cue and no-cue trials"
            );
        }
    }
}

// ── B2: previous-slot one-tick cue ──────────────────────────────────────────

/// B2 previous-slot one-tick cue: `LoadSlotPrev` is a genuine one-tick memory
/// element. Tick 2 sees the tick-1 latched value; tick 3, after the latch is
/// cleared at tick 2, sees zero. Status: meets.
#[test]
fn b2_previous_slot_one_tick_cue() {
    let pos = Position::new(4, 4);
    let (mut sim, target) = one_creature_sim(previous_slot_cue_vm_genome(), pos, 100.0);

    let tick1 = run_one_traced_tick(&mut sim, target);
    assert_eq!(
        tick1.final_actions[0],
        WorldAction::NoOp,
        "tick1: no prior cue yet"
    );

    let tick2 = run_one_traced_tick(&mut sim, target);
    assert_eq!(
        tick2.final_actions[0],
        WorldAction::Move(Direction::E),
        "tick2: LoadSlotPrev sees the tick-1 latched value"
    );

    let tick3 = run_one_traced_tick(&mut sim, target);
    assert_eq!(
        tick3.final_actions[0],
        WorldAction::NoOp,
        "tick3: the latch was cleared at tick2, so the previous-tick slot reads zero"
    );
}

// ── C1: retention ────────────────────────────────────────────────────────────

/// C1 retention: a value latched once into shared memory at production decay
/// (`shared_memory.decay_rate == 0.0`) is exactly retained at every tested
/// delay (1, 2, 4, 8, 16 ticks). Status: meets.
#[test]
fn c1_retention() {
    let pos = Position::new(3, 3);
    let (mut sim, target) = one_creature_sim(slot_write_once_vm_genome(3, 0.75), pos, 100.0);

    let mut tick_num = 0u64;
    for delay in [1u64, 2, 4, 8, 16] {
        let decision_tick = 1 + delay;
        while tick_num < decision_tick {
            run_one_traced_tick(&mut sim, target);
            tick_num += 1;
        }
        let creature = sim.creatures.get(target).expect("creature alive");
        assert!(
            (creature.shared_memory[3] - 0.75).abs() < 1e-6,
            "delay {delay}: shared_memory[3] should retain 0.75 exactly at zero decay"
        );
        assert!(
            (creature.prev_shared_memory[3] - 0.75).abs() < 1e-6,
            "delay {delay}: prev_shared_memory[3] should also read 0.75 from tick2 onward"
        );
    }
}

// ── C2: retention under decay (labeled treatment) ───────────────────────────

/// C2 retention under decay: C1 with `shared_memory.decay_rate = 0.1`, the
/// one non-production runtime deviation this feature permits. After `d`
/// ticks the retained value is `0.75 * 0.9^d` within `1e-5`. Status: meets
/// (decay is applied exactly as documented; this is not one of the flagged
/// clocks).
#[test]
fn c2_retention_under_decay() {
    let pos = Position::new(3, 3);
    let cfg = {
        let mut cfg = test_config();
        cfg.shared_memory.decay_rate = 0.1; // labeled deviation
        cfg
    };
    let (mut sim, target) = sim_with_config(slot_write_once_vm_genome(3, 0.75), pos, 100.0, cfg);

    let mut tick_num = 0u64;
    for delay in [1u64, 2, 4, 8, 16] {
        let decision_tick = 1 + delay;
        while tick_num < decision_tick {
            run_one_traced_tick(&mut sim, target);
            tick_num += 1;
        }
        let expected = 0.75 * 0.9f32.powi(delay as i32);
        let creature = sim.creatures.get(target).expect("creature alive");
        assert!(
            (creature.shared_memory[3] - expected).abs() < 1e-5,
            "delay {delay}: shared_memory[3] should be 0.75 * 0.9^{delay} = {expected}, \
             got {}",
            creature.shared_memory[3]
        );
    }
}

// ── C3: graph slot write and previous read ──────────────────────────────────

/// C3 graph slot write and previous read: a `Constant` node writes shared
/// memory slot 0 every tick; a second node reads slot 0's *previous-tick*
/// value and writes it to slot 1. Tick 1: slot0 = 0.6, slot1 = 0 (no prior
/// tick yet). Tick 2: slot1 = 0.6 (sees tick 1's write). Status: meets.
#[test]
fn c3_graph_slot_write_and_previous_read() {
    let pos = Position::new(2, 2);
    let (mut sim, target) =
        one_creature_sim(slot_write_and_previous_read_graph_genome(), pos, 100.0);

    run_one_traced_tick(&mut sim, target);
    {
        let creature = sim.creatures.get(target).expect("creature alive");
        assert!(
            (creature.shared_memory[0] - 0.6).abs() < 1e-6,
            "tick1 slot0"
        );
        assert!(
            (creature.shared_memory[1] - 0.0).abs() < 1e-6,
            "tick1 slot1"
        );
    }

    run_one_traced_tick(&mut sim, target);
    {
        let creature = sim.creatures.get(target).expect("creature alive");
        assert!(
            (creature.shared_memory[1] - 0.6).abs() < 1e-6,
            "tick2 slot1 sees tick1's slot0"
        );
    }
}

// ── D helpers and literals ───────────────────────────────────────────────────
//
// D1_STATES/D1_PASSES and D2_STATES/D2_PASSES are measured from the
// production tick path (recorded in the spec catalogue's Observed column);
// the fixtures assert against these literals directly rather than against a
// duplicate of the production relaxation recurrence, so a repair to that
// recurrence (T11.F06) needs only one line flipped per fixture instead of
// two divergent implementations kept in sync.
//
// Why the values land where they do (mechanism, not a re-derivable
// formula): each tick's relaxation loop compares the persisted `node_state`
// against `prev_outputs`, which resets to zero at the start of every tick;
// it runs at least one pass and at most `max_graph_relax_iters` (15),
// stopping once `graph_convergence_stable_passes` (2) consecutive passes
// have `delta <= graph_convergence_epsilon` (1e-3). D1's lone
// `DecayIntegrator(0.5)` fed a constant input of 1.0 races toward its fixed
// point during tick 1's artificial zero-baseline comparison (11 passes),
// then settles quickly in later ticks (3, 3, 3) once `node_state` already
// sits near the fixed point. D2 adds a companion `Oscillator` node with no
// inputs and no consumers; its own output cycles 1, 0, -1, 0, ... every
// pass and never satisfies the epsilon, so the stable-pass counter never
// reaches 2 and every tick instead runs the full 15-pass cap.
const D1_PASSES: [u32; 4] = [11, 3, 3, 3];
const D1_STATES: [f32; 4] = [0.9995117, 0.99993896, 0.9999924, 0.99999905];
const D2_PASSES: [u32; 4] = [15, 15, 15, 15];
const D2_STATES: [f32; 4] = [0.9999695, 1.0, 1.0, 1.0];

/// Run `ticks` traced ticks against the entry node's single graph hop,
/// recording the number of relaxation passes and the persisted
/// `DecayIntegrator` state (`node_state[0][1]`) after each tick. Shared by
/// D1 and D2, whose only difference is the genome under test.
fn run_and_observe_integrator_clock(
    sim: &mut Simulation,
    target: CreatureId,
    ticks: usize,
) -> (Vec<f32>, Vec<u32>) {
    let mut observed_states = Vec::with_capacity(ticks);
    let mut observed_passes = Vec::with_capacity(ticks);
    for _ in 0..ticks {
        let tick = run_one_traced_tick(sim, target);
        let gtrace = graph_hop(&tick, 0);
        observed_passes.push(gtrace.passes.len() as u32);
        let creature = sim.creatures.get(target).expect("creature alive");
        observed_states.push(creature.graph_runtime.node_state[0][1]);
    }
    (observed_states, observed_passes)
}

// ── D1: integrator clock ────────────────────────────────────────────────────

/// D1 integrator clock: at production settings (`max_graph_relax_iters = 15`,
/// `graph_convergence_epsilon = 1e-3`, `graph_convergence_stable_passes =
/// 2`), a `DecayIntegrator` advances on every relaxation *pass*, not once per
/// world tick. Status: gap, assigned to T11.F06. The idealized one-pass-per-
/// tick clock would give states 0.5, 0.75, 0.875, 0.9375; the observed
/// per-tick pass counts and states are the `D1_PASSES`/`D1_STATES` literals
/// (see the D helpers comment for their derivation).
#[test]
fn d1_integrator_clock() {
    let pos = Position::new(1, 1);
    let (mut sim, target) = one_creature_sim(decay_integrator_graph_genome(), pos, 100.0);

    let (observed_states, observed_passes) = run_and_observe_integrator_clock(&mut sim, target, 4);

    assert_eq!(
        observed_passes, D1_PASSES,
        "passes per tick should match the catalogue's recorded observation"
    );
    for (i, (&obs, &exp)) in observed_states.iter().zip(D1_STATES.iter()).enumerate() {
        assert!(
            (obs - exp).abs() < 1e-5,
            "tick {}: integrator state {obs} should match the catalogue's recorded {exp}",
            i + 1
        );
    }
    // Gap against the idealized one-pass-per-tick clock: tick 1 alone
    // already advances far past the idealized 0.5.
    assert!(
        observed_states[0] > 0.9,
        "tick1 state {} should already be far past the idealized single-pass value 0.5, \
         evidencing the per-pass (not per-tick) advance",
        observed_states[0]
    );
}

// ── D2: disconnected-node perturbation ──────────────────────────────────────

/// D2 disconnected-node perturbation: adding an `Oscillator` node with no
/// inputs and no consumers should leave the integrator's trajectory and pass
/// count identical to D1 (node-type contract property 3). Status: gap,
/// assigned to T11.F06. The oscillator's own output never satisfies the
/// convergence epsilon (it moves by a fixed nonzero step every pass), which
/// forces every tick to run the full `max_graph_relax_iters = 15` passes and
/// gives the integrator a different trajectory than D1's — the observed
/// per-tick pass counts and states are the `D2_PASSES`/`D2_STATES` literals
/// (see the D helpers comment for their derivation).
#[test]
fn d2_disconnected_node_perturbation() {
    let pos = Position::new(1, 1);
    let (mut sim, target) = one_creature_sim(
        decay_integrator_with_disconnected_oscillator_graph_genome(),
        pos,
        100.0,
    );

    let (observed_states, observed_passes) = run_and_observe_integrator_clock(&mut sim, target, 4);

    assert_eq!(
        observed_passes, D2_PASSES,
        "every tick should hit the max_graph_relax_iters cap because the disconnected \
         oscillator never lets the graph converge"
    );
    for (i, (&obs, &exp)) in observed_states.iter().zip(D2_STATES.iter()).enumerate() {
        assert!(
            (obs - exp).abs() < 1e-4,
            "tick {}: integrator state {obs} should match the catalogue's recorded {exp}",
            i + 1
        );
    }

    // Contract violation: the disconnected, unconsumed oscillator changed
    // both the pass count and the integrator's trajectory relative to D1.
    assert_ne!(
        observed_passes, D1_PASSES,
        "gap: a disconnected, unconsumed node changed the pass count per tick"
    );
    assert!(
        (observed_states[0] - D1_STATES[0]).abs() > 1e-6,
        "gap: a disconnected, unconsumed node changed the integrator's tick-1 trajectory"
    );
}

// ── D3: backward-edge recurrence ────────────────────────────────────────────

/// D3 backward-edge recurrence: a self-referential `Add` node (edges from a
/// `Constant(1.0)` node and from itself) is read out through `WriteSlot(0)`
/// every tick. The node-type contract's idealized one-tick memory element
/// would give slot0 = 1, 2, 3 across three ticks (each tick's self-edge
/// reading the previous *tick's* output). Status: gap, assigned to T11.F06.
/// Because `prev_outputs` resets to zero at the start of every tick and the
/// self-edge only ever reads within-tick `prev_outputs` (never the previous
/// tick's converged value, since `Add` has no persistent `state`), the
/// self-loop instead accumulates once per relaxation pass and is capped by
/// `max_graph_relax_iters`, giving the same value (15.0) every tick.
#[test]
fn d3_backward_edge_recurrence() {
    let pos = Position::new(1, 1);
    let (mut sim, target) = one_creature_sim(backward_edge_recurrence_graph_genome(), pos, 100.0);

    let mut observed = Vec::with_capacity(3);
    for _ in 0..3 {
        run_one_traced_tick(&mut sim, target);
        let creature = sim.creatures.get(target).expect("creature alive");
        observed.push(creature.shared_memory[0]);
    }

    for (i, &v) in observed.iter().enumerate() {
        assert!(
            (v - 15.0).abs() < 1e-4,
            "tick {}: slot0 should be capped at max_graph_relax_iters (15.0), got {v}",
            i + 1
        );
    }
    // Gap: the idealized one-tick memory element (1, 2, 3) does not appear;
    // the value is identical every tick instead of incrementing.
    assert!(
        (observed[0] - observed[2]).abs() < 1e-6,
        "gap: the backward edge should read a genuinely advancing one-tick memory (1,2,3), \
         but observed the same pass-capped value every tick: {observed:?}"
    );
}

// ── E1: exact one-edge update, immediate reward ─────────────────────────────

/// E1 exact one-edge update, immediate reward: a single reward-modulated
/// Hebbian edge (`Classic`, `learning_rate = eta = 0.5`, `trace_decay =
/// decay = 0.5`) fed by a `Constant(1.0)` pre-synaptic node. The node fires
/// `Eat` and the execute gate every tick; food is present, so `ActionSuccess
/// == 1.0` exactly (the only action attempted, and it succeeds). Status:
/// gap, assigned to T11.F07. The documented rule applies the learning rate
/// twice — once folded into the eligibility trace (`trace = decay * trace +
/// eta * pre * post`) and again at the reward update (`dw = eta * signal *
/// trace`) — so the effective one-tick gain is `eta^2 * signal * pre * post`
/// rather than a single `eta` application.
#[test]
fn e1_exact_one_edge_update_immediate_reward() {
    let pos = Position::new(5, 5);
    let (mut sim, target) = one_creature_sim(
        reward_modulated_node_genome(OutcomeChannel::ActionSuccess),
        pos,
        100.0,
    );
    sim.world.set_food(pos, 1.0);

    let tick = run_one_traced_tick(&mut sim, target);
    assert_eq!(
        tick.final_actions[0],
        WorldAction::Eat {
            type_idx: OrdinaryFoodTypeId::default()
        },
        "the wired action bank should fire Eat every tick"
    );

    let creature = sim.creatures.get(target).expect("creature alive");
    let eta = 0.5f32;
    let decay = 0.5f32;
    let pre = 1.0f32;
    let post = 1.0f32; // node1's own output: wsum = weight(1.0) * pre(1.0)
    let signal = 1.0f32; // ActionSuccess: the only attempted action succeeded

    let expected_trace = decay * 0.0 + eta * pre * post; // 0.5
    let expected_gain = eta * eta * signal * pre * post; // eta^2 * signal * pre * post = 0.25
    let expected_weight = 1.0 + expected_gain; // genome weight 1.0 + gain

    let trace = creature.graph_runtime.eligibility_traces[0][1][0];
    let weight = creature.graph_runtime.plasticity_weights[0][1][0];
    assert!(
        (trace - expected_trace).abs() < 1e-6,
        "eligibility trace after tick1 should be eta*pre*post = {expected_trace}, got {trace}"
    );
    assert!(
        (weight - expected_weight).abs() < 1e-6,
        "plasticity weight after tick1 should show the double learning-rate application \
         (eta^2 * signal * pre * post = {expected_gain} added to the genome weight), got {weight}"
    );
}

// ── E2: delayed reward, visited every tick ──────────────────────────────────

/// E2 delayed reward, visited every tick: E1's genome with the reward signal
/// held at zero (no food, so `Eat` fails) until tick `d + 1`, for `d` in {1,
/// 2, 4}. Because the module runs every tick, the eligibility trace
/// correctly accumulates decayed credit across the elapsed ticks
/// (`trace_n = eta * sum_{k=0}^{n-1} decay^k`), and the credit delivered at
/// the reward tick is discounted by exactly that elapsed-tick trace. Status:
/// meets on elapsed-time discounting — this is the well-behaved contrast
/// case against E3's skipped visits — but the weight update still carries
/// E1's `eta^2` double learning-rate application (`expected_weight` below
/// folds `eta` into `expected_trace` and again into the reward update), so
/// this fixture inherits E1's gap (assigned to T11.F07) on gain, not timing.
#[test]
fn e2_delayed_reward_visited_every_tick() {
    for delay in [1u64, 2, 4] {
        let reward_tick = delay + 1;
        let pos = Position::new(5, 5);
        let (mut sim, target) = one_creature_sim(
            reward_modulated_node_genome(OutcomeChannel::ActionSuccess),
            pos,
            100.0,
        );

        for tick_num in 1..=reward_tick {
            sim.world
                .set_food(pos, if tick_num == reward_tick { 1.0 } else { 0.0 });
            run_one_traced_tick(&mut sim, target);
        }

        let eta = 0.5f32;
        let decay = 0.5f32;
        let mut expected_trace = 0.0f32;
        for _ in 0..reward_tick {
            expected_trace = decay * expected_trace + eta * 1.0 * 1.0;
        }
        let expected_weight = 1.0 + eta * 1.0 * expected_trace; // signal == 1.0 at the reward tick

        let creature = sim.creatures.get(target).expect("creature alive");
        let trace = creature.graph_runtime.eligibility_traces[0][1][0];
        let weight = creature.graph_runtime.plasticity_weights[0][1][0];
        assert!(
            (trace - expected_trace).abs() < 1e-5,
            "delay {delay}: trace at the reward tick should match the elapsed-tick \
             recurrence {expected_trace}, got {trace}"
        );
        assert!(
            (weight - expected_weight).abs() < 1e-5,
            "delay {delay}: weight should reflect eta*signal*trace credit discounted by \
             {delay} elapsed ticks ({expected_weight}), got {weight}"
        );
    }
}

// ── E3: skipped module visits ────────────────────────────────────────────────

/// E3 skipped module visits: the entry VM node routes to the reward-modulated
/// graph node only on ticks with `FoodHere > 0` (tick 1); ticks 2 and 3 skip
/// the module entirely (`NoOp` + `ExecuteActionQueue` terminates the mesh
/// before any routing decision). The reward-source channel is `EnergyDelta`
/// (nonzero and independent of whether the module ran) rather than
/// `ActionSuccess`, so a nonzero signal is available on skipped ticks too:
/// on the visit tick, the graph node has no route targets of its own, so the
/// mesh soft-default terminates with `NoOp`; on a skipped tick, the entry VM
/// explicitly pushes `NoOp`. Both paths land on the same Phase 2 charge,
/// `noop_cost` (0.05), which dominates the VM/graph opcode costs it sits on
/// top of (1e-6 to 1e-5 scale) and gives a measured signal of about -0.05 per
/// tick (signal_1 ≈ -0.050079, skipped ≈ -0.050018). Status:
/// gap, assigned to T11.F07. The eligibility trace is frozen at its tick-1
/// value on skipped ticks (the module never re-evaluates it), but the
/// reward-learning pass still applies `dw = eta * signal * trace` every tick
/// to every creature with a reward-modulated node — using that stale,
/// frozen trace, so the weight still moves by about -0.0125 per skipped
/// tick — because it does not check whether the node's mesh hop executed
/// this tick.
///
/// The test measures the `EnergyDelta` signal itself by reading
/// `creature.energy` around each `run_tick` call, rather than hand-deriving
/// VM/graph opcode costs; this equivalence to production's own signal holds
/// only because `test_config()` sets `energy_decay_per_tick == 0.0` and
/// production's `reward_learning_cost` defaults to `0.0` — if either
/// defaults away from zero, this fixture's arithmetic (not just its
/// assertions) needs revisiting.
#[test]
fn e3_skipped_module_visits() {
    let pos = Position::new(5, 5);
    let id_vm = NodeId::new(0);
    let id_graph = NodeId::new(1);
    let id_alt = NodeId::new(2);

    let BackendDef::Graph(graph_def) =
        reward_modulated_node_genome_inert(OutcomeChannel::EnergyDelta)
            .nodes
            .remove(0)
            .backend_def
    else {
        unreachable!("constructed as a graph backend");
    };
    let genome = CreatureGenome {
        entry_node_id: id_vm,
        nodes: vec![
            conditional_route_vm_node(id_vm, id_graph, id_alt),
            NodeGenome {
                node_id: id_graph,
                input_refs: vec![],
                backend_def: BackendDef::Graph(graph_def),
                targets: vec![],
            },
            noop_vm_node(id_alt),
        ],
    };

    let (mut sim, target) = one_creature_sim(genome, pos, 500.0);

    // Tick 1: visit (food present).
    sim.world.set_food(pos, 1.0);
    let energy_before_1 = sim.creatures.get(target).expect("alive").energy;
    let tick1 = run_one_traced_tick(&mut sim, target);
    assert_eq!(
        tick1.hops.len(),
        2,
        "tick1 should route into the graph node"
    );
    let energy_after_1 = sim.creatures.get(target).expect("alive").energy;
    let signal_1 = energy_after_1 - energy_before_1;
    assert!(
        signal_1.abs() > 1e-3,
        "tick1 signal should be nonzero (the mesh soft-default NoOp's noop_cost, about \
         -0.05) so the weight-moved-by-eta*signal*trace claim below is not vacuous; got \
         {signal_1}"
    );

    let creature = sim.creatures.get(target).expect("alive");
    let trace_1 = creature.graph_runtime.eligibility_traces[1][1][0];
    let eta = 0.5f32;
    let expected_trace_1 = eta; // decay*0 + eta*pre(1.0)*post(1.0)
    assert!(
        (trace_1 - expected_trace_1).abs() < 1e-5,
        "tick1 trace should be eta*pre*post = {expected_trace_1}, got {trace_1}"
    );
    let mut expected_weight = 1.0 + eta * signal_1 * trace_1;
    let weight_1 = creature.graph_runtime.plasticity_weights[1][1][0];
    assert!(
        (weight_1 - expected_weight).abs() < 1e-4,
        "tick1 weight should be genome weight + eta*signal*trace = {expected_weight}, \
         got {weight_1}"
    );

    // Ticks 2 and 3: skipped visits (no food; entry VM terminates before
    // reaching the graph node), but reward learning still fires.
    for skipped_tick in [2u64, 3] {
        sim.world.set_food(pos, 0.0);
        let energy_before = sim.creatures.get(target).expect("alive").energy;
        let tick = run_one_traced_tick(&mut sim, target);
        assert_eq!(
            tick.hops.len(),
            1,
            "skipped tick {skipped_tick} should terminate at the entry VM node, \
             never reaching the graph node"
        );
        let energy_after = sim.creatures.get(target).expect("alive").energy;
        let signal = energy_after - energy_before;
        assert!(
            signal.abs() > 1e-3,
            "skipped tick {skipped_tick} signal should be nonzero (the entry VM's explicit \
             NoOp noop_cost, about -0.05) so the weight-still-moved claim below is not \
             vacuous; got {signal}"
        );

        let creature = sim.creatures.get(target).expect("alive");
        let trace_now = creature.graph_runtime.eligibility_traces[1][1][0];
        assert!(
            (trace_now - trace_1).abs() < 1e-9,
            "gap: eligibility trace should stay frozen across a skipped tick \
             (module never re-evaluated it); tick {skipped_tick} trace {trace_now} \
             should equal tick1 trace {trace_1}"
        );

        expected_weight += eta * signal * trace_1;
        let weight_now = creature.graph_runtime.plasticity_weights[1][1][0];
        assert!(
            (weight_now - expected_weight).abs() < 1e-4,
            "gap: reward learning should still move the weight on a skipped tick \
             using the stale, frozen trace (eta*signal*trace_1 = {}), \
             expected cumulative weight {expected_weight}, got {weight_now}",
            eta * signal * trace_1
        );
    }
}

// ── Proptest: stateful compute-node convex-combination invariant ───────────

fn empty_sensor_snapshot() -> SensorSnapshot {
    SensorSnapshot {
        local: StaticInputs {
            food_here: 0.0,
            neighbor_food: [0.0; 8],
            neighbor_barrier: [0.0; 8],
            neighbor_occupied: [0.0; 8],
            generation: 0.0,
            age_ticks: 0.0,
        },
        typed_local_food: TypedFoodLocalSnapshot::zeroed(1),
        perception: PerceptionSnapshot::zeroed(1),
    }
}

/// Build a genome with `Constant(input)` feeding `node1_kind` (one edge,
/// weight 1.0), for use with the fully public `execute_creature_mesh` path
/// (no `Simulation` needed — this is a pure, in-process mesh invocation).
fn one_step_stateful_genome(node1_kind: ComputeNodeKind, input: f32) -> CreatureGenome {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs(&MutationConfig::default());
    def.compute_nodes.push(ComputeNode {
        kind: ComputeNodeKind::Constant(input),
        inputs: Vec::new(),
        plasticity: None,
    });
    def.compute_nodes.push(ComputeNode {
        kind: node1_kind,
        inputs: vec![GraphEdge {
            source: GraphSource::ComputeNode(0),
            weight: 1.0,
        }],
        plasticity: None,
    });
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Graph(def),
            targets: vec![],
        }],
    }
}

proptest! {
    /// Pure invariant: one `DecayIntegrator(a)` step and one `Momentum(b)`
    /// step, for any `a`, `b` in `[0, 1]` and any finite `state`, `input` in
    /// `[-1e6, 1e6]`, return a value between `state` and `input` inclusive —
    /// both are convex combinations of the prior state and the current
    /// input. Exercised through the fully public `execute_creature_mesh`
    /// path (`max_graph_relax_iters` forced to 1 so exactly one pass runs),
    /// with the prior state seeded directly into the public
    /// `GraphRuntimeState::node_state` field. Assertions do not depend on
    /// which cases are drawn.
    #[test]
    fn stateful_node_step_stays_between_state_and_input(
        a in 0.0f32..=1.0,
        b in 0.0f32..=1.0,
        state in -1.0e6f32..=1.0e6,
        input in -1.0e6f32..=1.0e6,
    ) {
        for kind in [ComputeNodeKind::DecayIntegrator(a), ComputeNodeKind::Momentum(b)] {
            let genome = one_step_stateful_genome(kind, input);
            let sensors = empty_sensor_snapshot();
            let cfg = v3_core::config::RuntimeConfig {
                max_graph_relax_iters: 1, // force exactly one relaxation pass
                ..v3_core::config::RuntimeConfig::default()
            };
            let mut energy = 1.0e9f32;
            let mut shared_memory = [0.0f32; 16];
            let prev_shared_memory = [0.0f32; 16];
            let mut graph_runtime = GraphRuntimeState::new();
            graph_runtime.node_state.push(vec![0.0, state]);

            let _ = execute_creature_mesh(
                &genome,
                &sensors,
                &mut energy,
                0.0,
                &mut shared_memory,
                &prev_shared_memory,
                &mut graph_runtime,
                &cfg,
            );

            let after = graph_runtime.node_state[0][1];
            let lo = state.min(input);
            let hi = state.max(input);
            prop_assert!(
                after >= lo - 1e-3 && after <= hi + 1e-3,
                "kind {:?}: state {after} should lie within [{lo}, {hi}] (state {state}, input {input})",
                kind
            );
        }
    }
}
