//! Mesh chain executor — evaluates a creature's genome mesh each tick.
//!
//! The mesh executor walks a chain of [`NodeGenome`] nodes starting from
//! `genome.entry_node_id`, dispatching each node to its backend (VM or Graph),
//! and routing to the next node via the returned `route_target_idx` until a
//! [`WorldAction`] is emitted or a soft-default termination condition fires.

use std::collections::HashMap;

use crate::config::RuntimeConfig;
use crate::contracts::{NodeId, WorldAction};
use crate::creature::genome::{BackendDef, CreatureGenome};
use crate::runtime::graph::execute_graph_node;
use crate::runtime::vm::execute_vm_node;
use crate::sensors::static_inputs::StaticInputs;

/// Execute the creature's mesh chain for one tick, returning the chosen [`WorldAction`].
///
/// The function walks the genome's node chain starting at `entry_node_id`,
/// dispatching each node to its VM or Graph backend, routing to subsequent
/// nodes via the `route_target_idx` field of [`NodeResult`], and terminating
/// when a `WorldAction` is emitted or a soft-default condition fires.
///
/// All soft-default termination conditions return [`WorldAction::NoOp`].
///
/// # Arguments
/// - `genome`: the creature's node graph
/// - `static_inputs`: pre-assembled sensor snapshot for this tick
/// - `energy`: creature's mutable energy; decremented by node evaluation costs
/// - `memory`: creature's 1024-byte persistent memory
/// - `graph_state`: per-node persistent state for Graph backends
/// - `config`: runtime limits (max_mesh_hops, max_vm_steps, etc.)
#[allow(clippy::too_many_arguments)]
pub fn execute_creature_mesh(
    genome: &CreatureGenome,
    static_inputs: &StaticInputs,
    energy: &mut f32,
    memory: &mut [u8; 1024],
    graph_state: &mut HashMap<NodeId, Vec<f32>>,
    config: &RuntimeConfig,
) -> WorldAction {
    let mut current_node_id = genome.entry_node_id;
    let mut upstream_slots = [0.0f32; 12];
    let mut hops: usize = 0;
    let max_hops = config.max_mesh_hops.max(1) as usize;
    let start_energy = *energy;

    // Soft default: entry_node_id missing from node set → return NoOp immediately.
    if genome.find_node(current_node_id).is_none() {
        return WorldAction::NoOp;
    }

    loop {
        if hops >= max_hops {
            return WorldAction::NoOp;
        }

        // Invariant: verified present before the loop, and after every routing step.
        let node = genome
            .find_node(current_node_id)
            .expect("node must exist: checked before loop and after routing");

        let energy_consumed = (start_energy - *energy).max(0.0);

        let result = match &node.backend_def {
            BackendDef::Vm(def) => execute_vm_node(
                def,
                &node.input_refs,
                &upstream_slots,
                energy,
                energy_consumed,
                memory,
                static_inputs,
                config,
            ),
            BackendDef::Graph(def) => execute_graph_node(
                def,
                &node.input_refs,
                &upstream_slots,
                energy,
                energy_consumed,
                node.node_id,
                graph_state,
                static_inputs,
                config,
            ),
        };

        // Check exhaustion before world_action: NodeResult::exhausted() never carries a valid action.
        if result.energy_exhausted {
            return WorldAction::NoOp;
        }

        if let Some(action) = result.world_action {
            return action;
        }

        // Routing: if no targets, the chain terminates with NoOp.
        if node.targets.is_empty() {
            return WorldAction::NoOp;
        }

        // Convert route_target_idx (f32) to i64 with special-case handling for
        // NaN and infinities to avoid undefined behaviour from out-of-range casts.
        let route_target_idx = result.route_target_idx;
        let route_idx_i64: i64 = if route_target_idx.is_nan() {
            -1
        } else if route_target_idx == f32::INFINITY {
            i64::MAX
        } else if route_target_idx == f32::NEG_INFINITY {
            i64::MIN
        } else {
            // Clamp to the representable i64 range before casting to prevent UB.
            route_target_idx
                .clamp(i64::MIN as f32, i64::MAX as f32)
                .floor() as i64
        };

        let target_pos = route_idx_i64.rem_euclid(node.targets.len() as i64) as usize;
        let target_id = node.targets[target_pos];

        // Soft default: routed target id missing from node set → return NoOp.
        if genome.find_node(target_id).is_none() {
            return WorldAction::NoOp;
        }

        upstream_slots = result.output_slots;
        current_node_id = target_id;
        hops += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RuntimeConfig;
    use crate::contracts::{InputReference, NodeId, WorldAction};
    use crate::creature::genome::{
        BackendDef, CreatureGenome, GraphBackendDef, GraphInput, GraphInternalNode, GraphNodeKind,
        NodeGenome, VmBackendDef, VmInstruction,
    };
    use crate::sensors::static_inputs::StaticInputs;
    use std::collections::HashMap;

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn default_config() -> RuntimeConfig {
        RuntimeConfig::default()
    }

    fn empty_static_inputs() -> StaticInputs {
        StaticInputs {
            food_here: 0.0,
            neighbor_food: [0.0; 8],
            neighbor_barrier: [0.0; 8],
            neighbor_occupied: [0.0; 8],
            generation: 0.0,
            age_ticks: 0.0,
        }
    }

    /// Build a minimal VM node that emits `EmitWorldAction { action_type }` and
    /// then halts. The node has `register_count=1` so the VM will run.
    fn vm_emit_node(node_id: NodeId, action_type: u8, targets: Vec<NodeId>) -> NodeGenome {
        NodeGenome {
            node_id,
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![VmInstruction::EmitWorldAction { action_type }],
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
        let si = empty_static_inputs();
        let mut energy = 100.0f32;
        let mut memory = [0u8; 1024];
        let mut graph_state = HashMap::new();
        let config = default_config();

        let action = execute_creature_mesh(
            &genome,
            &si,
            &mut energy,
            &mut memory,
            &mut graph_state,
            &config,
        );
        assert_eq!(action, WorldAction::NoOp);
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
        let si = empty_static_inputs();
        let mut energy = 1000.0f32;
        let mut memory = [0u8; 1024];
        let mut graph_state = HashMap::new();
        let config = RuntimeConfig {
            max_mesh_hops: 3,
            ..RuntimeConfig::default()
        };

        let action = execute_creature_mesh(
            &genome,
            &si,
            &mut energy,
            &mut memory,
            &mut graph_state,
            &config,
        );
        assert_eq!(action, WorldAction::NoOp);
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
        let si = empty_static_inputs();
        let mut energy = 100.0f32;
        let mut memory = [0u8; 1024];
        let mut graph_state = HashMap::new();
        let config = default_config();

        let action = execute_creature_mesh(
            &genome,
            &si,
            &mut energy,
            &mut memory,
            &mut graph_state,
            &config,
        );
        assert_eq!(action, WorldAction::NoOp);
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
        let si = empty_static_inputs();
        let mut energy = 0.01f32; // way below the Noop cost of 0.05
        let mut memory = [0u8; 1024];
        let mut graph_state = HashMap::new();
        let config = default_config();

        let action = execute_creature_mesh(
            &genome,
            &si,
            &mut energy,
            &mut memory,
            &mut graph_state,
            &config,
        );
        assert_eq!(action, WorldAction::NoOp);
    }

    // ── Test 5: vm_node_emits_eat_action ─────────────────────────────────────

    /// A VM node that emits EmitWorldAction{action_type: 1} → WorldAction::Eat.
    #[test]
    fn vm_node_emits_eat_action() {
        let id0 = NodeId::new(0);
        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![vm_emit_node(id0, 1, vec![])],
        };
        let si = empty_static_inputs();
        let mut energy = 100.0f32;
        let mut memory = [0u8; 1024];
        let mut graph_state = HashMap::new();
        let config = default_config();

        let action = execute_creature_mesh(
            &genome,
            &si,
            &mut energy,
            &mut memory,
            &mut graph_state,
            &config,
        );
        assert_eq!(action, WorldAction::Eat);
    }

    // ── Test 6: graph_node_routes_to_vm_node_which_emits_action ──────────────

    /// A Graph node with RouterOutput(0.0) routes to targets[0], which is a VM
    /// node that emits Eat.
    #[test]
    fn graph_node_routes_to_vm_node_which_emits_action() {
        let id_graph = NodeId::new(0);
        let id_vm = NodeId::new(1);

        // Graph node: single Constant(0.0) → RouterOutput → routes to targets[0]
        let graph_node = NodeGenome {
            node_id: id_graph,
            input_refs: vec![],
            backend_def: BackendDef::Graph(GraphBackendDef {
                internal_nodes: vec![
                    GraphInternalNode {
                        kind: GraphNodeKind::Constant(0.0),
                        inputs: vec![],
                    },
                    GraphInternalNode {
                        kind: GraphNodeKind::RouterOutput,
                        inputs: vec![GraphInput {
                            source_idx: 0,
                            weight: 1.0,
                        }],
                    },
                ],
            }),
            targets: vec![id_vm],
        };

        // VM node: emits Eat (action_type=1)
        let vm_node = vm_emit_node(id_vm, 1, vec![]);

        let genome = CreatureGenome {
            entry_node_id: id_graph,
            nodes: vec![graph_node, vm_node],
        };
        let si = empty_static_inputs();
        let mut energy = 100.0f32;
        let mut memory = [0u8; 1024];
        let mut graph_state = HashMap::new();
        let config = default_config();

        let action = execute_creature_mesh(
            &genome,
            &si,
            &mut energy,
            &mut memory,
            &mut graph_state,
            &config,
        );
        assert_eq!(action, WorldAction::Eat);
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
        let si = empty_static_inputs();
        let mut energy = 1000.0f32;
        let mut memory = [0u8; 1024];
        let mut graph_state = HashMap::new();
        let config = default_config();

        let action = execute_creature_mesh(
            &genome,
            &si,
            &mut energy,
            &mut memory,
            &mut graph_state,
            &config,
        );
        assert_eq!(
            action,
            WorldAction::Eat,
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
        let si = empty_static_inputs();
        let mut energy = 1000.0f32;
        let mut memory = [0u8; 1024];
        let mut graph_state = HashMap::new();
        let config = default_config();

        let action = execute_creature_mesh(
            &genome,
            &si,
            &mut energy,
            &mut memory,
            &mut graph_state,
            &config,
        );
        assert_eq!(
            action,
            WorldAction::Eat,
            "route=-1.0 should wrap via rem_euclid and select targets[1]"
        );
    }

    // ── Test 9: output_slots_passed_as_upstream ───────────────────────────────

    /// A Graph node writes 9.0 to slot 5 via CustomOutput(5).
    /// A downstream VM node reads UpstreamSlot(5) via ReadInput — should get 9.0,
    /// then emits Eat after reading a truthy value (9.0 >= 0.5).
    #[test]
    fn output_slots_passed_as_upstream() {
        let id_graph = NodeId::new(0);
        let id_vm = NodeId::new(1);

        // Graph node: Constant(9.0) → CustomOutput(5)
        // RouterOutput is 0.0 (default) so it routes to targets[0] = id_vm
        let graph_node = NodeGenome {
            node_id: id_graph,
            input_refs: vec![],
            backend_def: BackendDef::Graph(GraphBackendDef {
                internal_nodes: vec![
                    GraphInternalNode {
                        kind: GraphNodeKind::Constant(9.0),
                        inputs: vec![],
                    },
                    GraphInternalNode {
                        kind: GraphNodeKind::CustomOutput(5),
                        inputs: vec![GraphInput {
                            source_idx: 0,
                            weight: 1.0,
                        }],
                    },
                    // RouterOutput: 0.0 → targets[0]
                    GraphInternalNode {
                        kind: GraphNodeKind::RouterOutput,
                        inputs: vec![],
                    },
                ],
            }),
            targets: vec![id_vm],
        };

        // VM node: ReadInput(0) → UpstreamSlot(5) → r0 = 9.0
        // ToBool(r0) → 1.0 → EmitWorldAction(1)=Eat
        let vm_node = NodeGenome {
            node_id: id_vm,
            input_refs: vec![InputReference::UpstreamSlot(5)],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 2,
                constants: vec![],
                program: vec![
                    VmInstruction::ReadInput {
                        dst: 0,
                        input_idx: 0,
                    }, // r0 = upstream_slots[5] = 9.0
                    VmInstruction::ToBool { dst: 1, src: 0 }, // r1 = 1.0
                    VmInstruction::JumpIfZero { cond: 1, offset: 1 }, // skip Eat if r1==0
                    VmInstruction::EmitWorldAction { action_type: 1 }, // Eat
                    VmInstruction::EmitWorldAction { action_type: 0 }, // NoOp fallback
                ],
            }),
            targets: vec![],
        };

        let genome = CreatureGenome {
            entry_node_id: id_graph,
            nodes: vec![graph_node, vm_node],
        };
        let si = empty_static_inputs();
        let mut energy = 1000.0f32;
        let mut memory = [0u8; 1024];
        let mut graph_state = HashMap::new();
        let config = default_config();

        let action = execute_creature_mesh(
            &genome,
            &si,
            &mut energy,
            &mut memory,
            &mut graph_state,
            &config,
        );
        assert_eq!(
            action,
            WorldAction::Eat,
            "slot 5 should carry 9.0 from graph to VM node"
        );
    }
}
