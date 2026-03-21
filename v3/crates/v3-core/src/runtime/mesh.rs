//! Mesh chain executor — evaluates a creature's genome mesh each tick.
//!
//! The mesh executor walks a chain of [`NodeGenome`] nodes starting from
//! `genome.entry_node_id`, dispatching each node to its backend (VM or Graph),
//! and routing to the next node via the returned internal routing decision until
//! a terminal condition or soft-default termination condition fires.

use crate::config::RuntimeConfig;
use crate::contracts::{NodeId, WorldAction};
use crate::creature::genome::{BackendDef, CreatureGenome, NodeGenome};
use crate::creature::state::GraphRuntimeState;
use crate::runtime::cgp::execute_graph_node;
use crate::runtime::routing::{resolve_route_index, RouteDecision};
use crate::runtime::types::{ComputeCostReport, MeshOutput, MeshSideOutputs, OUTPUT_SLOT_COUNT};
use crate::runtime::vm::execute_vm_node;
use crate::sensors::perception::SensorSnapshot;

/// Execute the creature's mesh chain for one tick, returning a [`MeshOutput`]
/// containing the queued actions, a [`ComputeCostReport`], and the priority bid.
///
/// The function walks the genome's node chain starting at `entry_node_id`,
/// dispatching each node to its VM or Graph backend, routing to subsequent
/// nodes via the `route` field of [`NodeResult`], and terminating
/// when a terminal instruction is reached or a soft-default condition fires.
///
/// All soft-default termination conditions return `vec![WorldAction::NoOp]`.
/// Energy exhaustion discards the queue and returns `vec![WorldAction::NoOp]`.
/// Halt without ExecuteActionQueue preserves the accumulated queue (forgiving).
///
/// # Arguments
/// - `genome`: the creature's node graph
/// - `sensors`: pre-assembled sensor snapshot (local + extended perception)
/// - `energy`: creature's mutable energy; decremented by node evaluation costs
/// - `shared_memory`: creature's shared f32 memory slots
/// - `prev_shared_memory`: snapshot of shared memory from previous tick
/// - `graph_runtime`: per-node persistent runtime state for Graph backends
/// - `config`: runtime limits (max_mesh_hops, max_vm_steps, etc.)
#[allow(clippy::too_many_arguments)]
pub fn execute_creature_mesh(
    genome: &CreatureGenome,
    sensors: &SensorSnapshot,
    energy: &mut f32,
    shared_memory: &mut [f32; 16],
    prev_shared_memory: &[f32; 16],
    graph_runtime: &mut GraphRuntimeState,
    config: &RuntimeConfig,
) -> MeshOutput {
    let mut current_node_id = genome.entry_node_id;
    let mut upstream_slots = [0.0f32; OUTPUT_SLOT_COUNT];
    let mut hops: usize = 0;
    let max_hops = config.max_mesh_hops.max(1) as usize;
    let start_energy = *energy;
    let mut report = ComputeCostReport::default();
    let mut side_outputs = MeshSideOutputs::new(config.max_actions_per_turn);

    // Soft default: entry_node_id missing from node set → return NoOp immediately.
    if find_node_index(&genome.nodes, current_node_id).is_none() {
        return MeshOutput {
            actions: vec![WorldAction::NoOp],
            cost_report: report,
            priority_bid: side_outputs.priority_bid,
        };
    }

    loop {
        if hops >= max_hops {
            return MeshOutput {
                actions: side_outputs.action_queue.into_actions_or_noop(),
                cost_report: report,
                priority_bid: side_outputs.priority_bid,
            };
        }

        // Invariant: verified present before the loop, and after every routing step.
        let current_idx =
            find_node_index(&genome.nodes, current_node_id).expect("node must exist in genome");
        let node = &genome.nodes[current_idx];

        let energy_consumed = (start_energy - *energy).max(0.0);

        // Snapshot energy before node dispatch to attribute cost to the correct backend.
        let node_energy_before = *energy;
        let result = match &node.backend_def {
            BackendDef::Vm(def) => execute_vm_node(
                def,
                &node.input_refs,
                &upstream_slots,
                energy,
                energy_consumed,
                shared_memory,
                prev_shared_memory,
                sensors,
                config,
                &mut side_outputs,
            ),
            BackendDef::Graph(def) => execute_graph_node(
                def,
                &node.input_refs,
                &upstream_slots,
                energy,
                energy_consumed,
                current_idx,
                graph_runtime,
                sensors,
                config,
                &mut side_outputs,
                shared_memory,
                prev_shared_memory,
            ),
        };

        // Attribute energy delta to the correct backend.
        let node_cost = (node_energy_before - *energy).max(0.0);
        match &node.backend_def {
            BackendDef::Vm(_) => report.vm_cost += node_cost,
            BackendDef::Graph(_) => report.graph_cost += node_cost,
        }

        // Check exhaustion first: NodeResult::exhausted() discards the action queue.
        if result.energy_exhausted {
            return MeshOutput {
                actions: vec![WorldAction::NoOp],
                cost_report: report,
                priority_bid: side_outputs.priority_bid,
            };
        }

        if result.terminal {
            return MeshOutput {
                actions: side_outputs.action_queue.into_actions_or_noop(),
                cost_report: report,
                priority_bid: side_outputs.priority_bid,
            };
        }

        // Routing: if no targets, the chain terminates — preserve accumulated queue.
        if node.targets.is_empty() {
            return MeshOutput {
                actions: side_outputs.action_queue.into_actions_or_noop(),
                cost_report: report,
                priority_bid: side_outputs.priority_bid,
            };
        }

        // Transitional: extract scores[0] for old resolve_route_index
        let transitional_route = match &node.backend_def {
            BackendDef::Vm(_) => RouteDecision::VmWrap {
                raw_value: result.route_gates.scores[0],
            },
            BackendDef::Graph(_) => RouteDecision::CgpNormalized {
                raw_value: result.route_gates.scores[0],
            },
        };
        let target_pos = resolve_route_index(node.targets.len(), transitional_route);
        let target_id = node.targets[target_pos];

        // Soft default: routed target id missing from node set.
        if find_node_index(&genome.nodes, target_id).is_none() {
            return MeshOutput {
                actions: side_outputs.action_queue.into_actions_or_noop(),
                cost_report: report,
                priority_bid: side_outputs.priority_bid,
            };
        }

        upstream_slots = result.output_slots;
        current_node_id = target_id;
        hops += 1;
    }
}

/// Find the index of a node by its `NodeId` via linear scan.
///
/// For typical genomes (2-10 nodes), linear scan is faster than HashMap
/// due to cache locality and zero heap allocation.
#[inline]
fn find_node_index(nodes: &[NodeGenome], id: NodeId) -> Option<usize> {
    nodes.iter().position(|n| n.node_id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RuntimeConfig;
    use crate::contracts::{NodeId, WorldAction};
    use crate::creature::genome::{
        BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
    };
    use crate::creature::state::GraphRuntimeState;
    use crate::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
    use crate::sensors::static_inputs::StaticInputs;

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn default_config() -> RuntimeConfig {
        RuntimeConfig::default()
    }

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
            perception: PerceptionSnapshot::zero(),
        }
    }

    /// Build a minimal VM node that pushes an action and executes the queue.
    /// The node has `register_count=1` so the VM will run.
    fn vm_emit_node(node_id: NodeId, action_type: u8, targets: Vec<NodeId>) -> NodeGenome {
        NodeGenome {
            node_id,
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![
                    VmInstruction::PushAction { action_type },
                    VmInstruction::ExecuteActionQueue,
                ],
            }),
            targets,
        }
    }

    /// Build a VM node that halts (no action) and routes with a given constant
    /// `route_value`.
    fn vm_halt_with_route(node_id: NodeId, route_value: f32, targets: Vec<NodeId>) -> NodeGenome {
        NodeGenome {
            node_id,
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![route_value],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::WriteRouteTarget { src: 0 },
                    VmInstruction::Halt,
                ],
            }),
            targets,
        }
    }

    // ── Test 1: missing_entry_node_returns_noop ───────────────────────────────

    /// A genome with no nodes — entry_node_id is not in the node set.
    #[test]
    fn missing_entry_node_returns_noop() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(99),
            nodes: vec![],
        };
        let ss = empty_sensor_snapshot();
        let mut energy = 100.0f32;
        let mut shared_mem = [0.0f32; 16];
        let prev_shared_mem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();
        let config = default_config();

        let output = execute_creature_mesh(
            &genome,
            &ss,
            &mut energy,
            &mut shared_mem,
            &prev_shared_mem,
            &mut gr,
            &config,
        );
        assert_eq!(output.actions, vec![WorldAction::NoOp]);
    }

    // ── Test 2: max_hops_exceeded_returns_noop ────────────────────────────────

    /// A VM node that halts (no action emitted) and routes to itself.
    /// With max_mesh_hops=3, after 3 hops the executor must return NoOp.
    #[test]
    fn max_hops_exceeded_returns_noop() {
        let id0 = NodeId::new(0);
        // Node routes to itself (self-loop); Halt emits no action.
        let node = NodeGenome {
            node_id: id0,
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![VmInstruction::Halt],
            }),
            targets: vec![id0], // self-loop
        };
        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![node],
        };
        let ss = empty_sensor_snapshot();
        let mut energy = 1000.0f32;
        let mut shared_mem = [0.0f32; 16];
        let prev_shared_mem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();
        let config = RuntimeConfig {
            max_mesh_hops: 3,
            ..RuntimeConfig::default()
        };

        let output = execute_creature_mesh(
            &genome,
            &ss,
            &mut energy,
            &mut shared_mem,
            &prev_shared_mem,
            &mut gr,
            &config,
        );
        assert_eq!(output.actions, vec![WorldAction::NoOp]);
    }

    // ── Test 3: empty_targets_returns_noop ───────────────────────────────────

    /// A VM node that halts with no targets — chain terminates with NoOp.
    #[test]
    fn empty_targets_returns_noop() {
        let id0 = NodeId::new(0);
        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![NodeGenome {
                node_id: id0,
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![], // no targets
            }],
        };
        let ss = empty_sensor_snapshot();
        let mut energy = 100.0f32;
        let mut shared_mem = [0.0f32; 16];
        let prev_shared_mem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();
        let config = default_config();

        let output = execute_creature_mesh(
            &genome,
            &ss,
            &mut energy,
            &mut shared_mem,
            &prev_shared_mem,
            &mut gr,
            &config,
        );
        assert_eq!(output.actions, vec![WorldAction::NoOp]);
    }

    // ── Test 4: energy_exhaustion_returns_noop ────────────────────────────────

    /// Energy is too low to run even one opcode — executor returns NoOp.
    #[test]
    fn energy_exhaustion_returns_noop() {
        let id0 = NodeId::new(0);
        // Noop costs 0.05; with energy=0.01 the first opcode exhausts energy.
        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![NodeGenome {
                node_id: id0,
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Noop, VmInstruction::Halt],
                }),
                targets: vec![],
            }],
        };
        let ss = empty_sensor_snapshot();
        let mut energy = 0.01f32; // way below the Noop cost of 0.05
        let mut shared_mem = [0.0f32; 16];
        let prev_shared_mem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();
        let config = default_config();

        let output = execute_creature_mesh(
            &genome,
            &ss,
            &mut energy,
            &mut shared_mem,
            &prev_shared_mem,
            &mut gr,
            &config,
        );
        assert_eq!(output.actions, vec![WorldAction::NoOp]);
    }

    // ── Test 5: vm_node_emits_eat_action ─────────────────────────────────────

    /// A VM node that pushes action_type 1 and executes queue → WorldAction::Eat.
    #[test]
    fn vm_node_emits_eat_action() {
        let id0 = NodeId::new(0);
        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![vm_emit_node(id0, 1, vec![])],
        };
        let ss = empty_sensor_snapshot();
        let mut energy = 100.0f32;
        let mut shared_mem = [0.0f32; 16];
        let prev_shared_mem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();
        let config = default_config();

        let output = execute_creature_mesh(
            &genome,
            &ss,
            &mut energy,
            &mut shared_mem,
            &prev_shared_mem,
            &mut gr,
            &config,
        );
        assert_eq!(output.actions, vec![WorldAction::Eat]);
    }

    // ── Test 7: route_wrapping_rem_euclid ────────────────────────────────────

    /// VM sets route=3.7; floor(3.7)=3; 3.rem_euclid(3 targets)=0.
    /// So the executor routes to targets[0] which emits Eat.
    #[test]
    fn route_wrapping_rem_euclid() {
        let id0 = NodeId::new(0);
        let id_a = NodeId::new(1);
        let id_b = NodeId::new(2);
        let id_c = NodeId::new(3);

        // Entry node: routes with value 3.7, has 3 targets [id_a, id_b, id_c]
        // floor(3.7)=3; 3 % 3 = 0 → targets[0] = id_a
        let entry = vm_halt_with_route(id0, 3.7, vec![id_a, id_b, id_c]);

        // targets[0] (id_a): emits Eat
        // targets[1] and [2]: emit NoOp (fallback, should not be chosen)
        let node_a = vm_emit_node(id_a, 1, vec![]); // Eat
        let node_b = vm_emit_node(id_b, 0, vec![]); // NoOp
        let node_c = vm_emit_node(id_c, 0, vec![]); // NoOp

        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![entry, node_a, node_b, node_c],
        };
        let ss = empty_sensor_snapshot();
        let mut energy = 1000.0f32;
        let mut shared_mem = [0.0f32; 16];
        let prev_shared_mem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();
        let config = default_config();

        let output = execute_creature_mesh(
            &genome,
            &ss,
            &mut energy,
            &mut shared_mem,
            &prev_shared_mem,
            &mut gr,
            &config,
        );
        assert_eq!(
            output.actions,
            vec![WorldAction::Eat],
            "route=3.7 should select targets[0]"
        );
    }

    // ── Test 8: negative_route_wraps_with_rem_euclid ─────────────────────────

    /// VM sets route=-1.0 → floor(-1.0)=-1 → (-1).rem_euclid(2)=1 → targets[1].
    /// targets[1] emits Eat, targets[0] emits NoOp.
    /// This verifies negative-index wrapping via rem_euclid in the mesh router.
    #[test]
    fn negative_route_wraps_with_rem_euclid() {
        let id0 = NodeId::new(0);
        let id_noop = NodeId::new(1); // targets[0]
        let id_eat = NodeId::new(2); // targets[1]

        // Entry node: sets route=-1.0 → floor(-1.0)=-1 → rem_euclid(2)=1 → targets[1]=id_eat
        // Note: the NaN→-1 code path in the mesh router is unreachable via current backends
        // because sanitize_f32 prevents NaN from appearing in any register or output slot.
        // This test verifies the rem_euclid wrapping behaviour with a directly injected
        // negative index (-1.0).
        let entry = vm_halt_with_route(id0, -1.0, vec![id_noop, id_eat]);
        let node_noop = vm_emit_node(id_noop, 0, vec![]); // NoOp
        let node_eat = vm_emit_node(id_eat, 1, vec![]); // Eat

        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![entry, node_noop, node_eat],
        };
        let ss = empty_sensor_snapshot();
        let mut energy = 1000.0f32;
        let mut shared_mem = [0.0f32; 16];
        let prev_shared_mem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();
        let config = default_config();

        let output = execute_creature_mesh(
            &genome,
            &ss,
            &mut energy,
            &mut shared_mem,
            &prev_shared_mem,
            &mut gr,
            &config,
        );
        assert_eq!(
            output.actions,
            vec![WorldAction::Eat],
            "route=-1.0 should wrap via rem_euclid and select targets[1]"
        );
    }

    // ── Priority bid tests ───────────────────────────────────────────────

    #[test]
    fn default_priority_bid_is_zero() {
        // A genome with no SetPriorityBid instruction should return priority_bid == 0.0.
        let id0 = NodeId::new(0);
        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![vm_emit_node(id0, 1, vec![])],
        };
        let ss = empty_sensor_snapshot();
        let mut energy = 100.0f32;
        let mut shared_mem = [0.0f32; 16];
        let prev_shared_mem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();
        let config = default_config();

        let output = execute_creature_mesh(
            &genome,
            &ss,
            &mut energy,
            &mut shared_mem,
            &prev_shared_mem,
            &mut gr,
            &config,
        );
        assert_eq!(output.priority_bid, 0.0);
    }

    #[test]
    fn priority_bid_propagates_to_mesh_output() {
        // A VM node loads 3.0 into r0, calls SetPriorityBid, then emits Eat.
        let id0 = NodeId::new(0);
        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![NodeGenome {
                node_id: id0,
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![3.0],
                    program: vec![
                        VmInstruction::LoadConst {
                            dst: 0,
                            const_idx: 0,
                        },
                        VmInstruction::SetPriorityBid { src: 0 },
                        VmInstruction::PushAction { action_type: 1 },
                        VmInstruction::ExecuteActionQueue,
                    ],
                }),
                targets: vec![],
            }],
        };
        let ss = empty_sensor_snapshot();
        let mut energy = 100.0f32;
        let mut shared_mem = [0.0f32; 16];
        let prev_shared_mem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();
        let config = default_config();

        let output = execute_creature_mesh(
            &genome,
            &ss,
            &mut energy,
            &mut shared_mem,
            &prev_shared_mem,
            &mut gr,
            &config,
        );
        assert_eq!(output.priority_bid, 3.0);
        assert_eq!(output.actions, vec![WorldAction::Eat]);
    }

    #[test]
    fn priority_bid_last_write_wins_across_hops() {
        // Two VM nodes both call SetPriorityBid. The second (downstream) value should win.
        let id0 = NodeId::new(0);
        let id1 = NodeId::new(1);

        // First node: bid 5.0, then halt and route to id1
        let node0 = NodeGenome {
            node_id: id0,
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![5.0],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::SetPriorityBid { src: 0 },
                    VmInstruction::Halt,
                ],
            }),
            targets: vec![id1],
        };

        // Second node: bid 2.0, then emit Eat
        let node1 = NodeGenome {
            node_id: id1,
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![2.0],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::SetPriorityBid { src: 0 },
                    VmInstruction::PushAction { action_type: 1 },
                    VmInstruction::ExecuteActionQueue,
                ],
            }),
            targets: vec![],
        };

        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![node0, node1],
        };
        let ss = empty_sensor_snapshot();
        let mut energy = 100.0f32;
        let mut shared_mem = [0.0f32; 16];
        let prev_shared_mem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();
        let config = default_config();

        let output = execute_creature_mesh(
            &genome,
            &ss,
            &mut energy,
            &mut shared_mem,
            &prev_shared_mem,
            &mut gr,
            &config,
        );
        assert_eq!(
            output.priority_bid, 2.0,
            "last-write-wins: second node's bid should be returned"
        );
    }
}
