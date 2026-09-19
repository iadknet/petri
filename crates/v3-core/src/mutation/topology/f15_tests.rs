//! Connection invariants and nonvacuous production operator fixtures.
use super::*;
use crate::contracts::{InputReference, RouteTarget, MAX_GATE_SLOTS};
use crate::creature::genome::analysis::mesh_reachable_nodes;
use crate::creature::genome::cgp::{CgpGraphBackendDef, GraphEdge, GraphSource, OutputSinkKind};
use crate::creature::genome::{VmBackendDef, VmInstruction};
use proptest::prelude::*;
use rand::{rngs::SmallRng, SeedableRng};

fn node(id: u32, targets: &[u32]) -> NodeGenome {
    NodeGenome {
        node_id: NodeId::new(id),
        input_refs: vec![],
        backend_def: birth::minimal_vm_backend(),
        targets: targets
            .iter()
            .enumerate()
            .map(|(i, &target)| RouteTarget {
                target_id: NodeId::new(target),
                slot: i as u8,
                gate_bias: 0.0,
            })
            .collect(),
    }
}
fn genome(nodes: Vec<NodeGenome>) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes,
    }
}
fn apply(
    g: &mut CreatureGenome,
    op: TopologyOperator,
    seed: u64,
) -> Result<TargetReachability, MutationSkipReason> {
    let reachable = mesh_reachable_nodes(g);
    TopologyMutator::apply(
        g,
        op,
        &mut TargetSelector::reachable_only(&reachable, 0.0),
        &mut SmallRng::seed_from_u64(seed),
        &MutationConfig::default(),
    )
}
fn gate(node: &mut NodeGenome, slot: u8) {
    if let BackendDef::Vm(vm) = &mut node.backend_def {
        vm.program
            .insert(0, VmInstruction::WriteRouteGate { slot, src: 0 });
    }
}

#[test]
fn copy_faithfully_preserves_references_and_uses_safe_later_slot() {
    for op in [
        TopologyOperator::CopyNode,
        TopologyOperator::SwapNodeBackend,
    ] {
        let mut g = genome(vec![node(0, &[1]), node(1, &[1, 2]), node(2, &[])]);
        g.nodes[1].input_refs = vec![InputReference::UpstreamSlot(3)];
        gate(&mut g.nodes[0], 1); // orphan write must not make slot 2 unavailable
        gate(&mut g.nodes[1], 0); // prevents node 2 being another attachment candidate
        let old = g.clone();
        apply(&mut g, op, 7).unwrap();
        let clone = &g.nodes[3];
        assert_eq!(g.nodes[0].targets[1].slot, 2);
        assert_eq!(g.nodes[0].targets[0], old.nodes[0].targets[0]);
        assert_eq!(&g.nodes[1..3], &old.nodes[1..3]);
        assert_eq!(clone.input_refs, old.nodes[1].input_refs);
        assert_eq!(clone.targets[0].target_id, clone.node_id);
        assert_eq!(clone.targets[1], old.nodes[1].targets[1]);
        if op == TopologyOperator::CopyNode {
            assert_eq!(clone.backend_def, old.nodes[1].backend_def);
        } else {
            assert!(matches!(clone.backend_def, BackendDef::Graph(_)));
        }
    }
}

#[test]
fn copy_and_backend_growth_skip_unsafe_predecessors_atomically() {
    for op in [
        TopologyOperator::CopyNode,
        TopologyOperator::SwapNodeBackend,
    ] {
        for case in 0..5 {
            let mut g = genome(vec![node(0, &[1]), node(1, &[])]);
            match case {
                0 => gate(&mut g.nodes[0], 0),
                1 => {
                    for slot in 1..MAX_GATE_SLOTS as u8 {
                        gate(&mut g.nodes[0], slot);
                    }
                }
                2 => g.nodes[1].targets = node(1, &[0]).targets,
                3 => g.nodes[0].targets[0].gate_bias = f32::INFINITY,
                _ => g.nodes[0].targets.clear(),
            }
            let before = g.clone();
            assert_eq!(
                apply(&mut g, op, 7),
                Err(MutationSkipReason::NoApplicableTarget)
            );
            assert_eq!(g, before);
        }
    }
}

#[test]
fn branch_failure_is_atomic_and_orphan_writes_are_preserved() {
    for case in 0..5 {
        let mut g = genome(vec![node(0, &[1]), node(1, &[])]);
        match case {
            0 => g.nodes[0].targets[0].target_id = NodeId::new(99),
            1 => g.nodes[0].targets[0].target_id = NodeId::new(0),
            2 => {
                let target = g.nodes[0].targets[0];
                g.nodes[0].targets.push(target);
            }
            3 => {
                if let BackendDef::Vm(vm) = &mut g.nodes[0].backend_def {
                    vm.register_count = 0;
                }
            }
            _ => {
                let mut graph =
                    CgpGraphBackendDef::new_with_fixed_outputs(&MutationConfig::default());
                graph
                    .output_sinks
                    .retain(|s| !matches!(s.kind, OutputSinkKind::RouterGate(1)));
                g.nodes[0].backend_def = BackendDef::Graph(graph);
            }
        }
        let old = g.clone();
        assert_eq!(
            apply(&mut g, TopologyOperator::AddRouteTarget, 7),
            Err(MutationSkipReason::NoApplicableTarget)
        );
        assert_eq!(g, old);
    }
    let mut g = genome(vec![node(0, &[1]), node(1, &[])]);
    gate(&mut g.nodes[0], 1);
    apply(&mut g, TopologyOperator::AddRouteTarget, 7).unwrap();
    let BackendDef::Vm(vm) = &g.nodes[0].backend_def else {
        panic!()
    };
    assert_eq!(
        vm.program
            .iter()
            .filter(|i| matches!(i, VmInstruction::WriteRouteGate { slot: 1, .. }))
            .count(),
        2
    );
}

#[test]
fn paired_vm_gate_insertion_repairs_jumps_and_precedes_first_terminal() {
    let mut g = genome(vec![node(0, &[1]), node(1, &[])]);
    let BackendDef::Vm(vm) = &mut g.nodes[0].backend_def else {
        panic!()
    };
    vm.program = vec![
        VmInstruction::Jump { offset: 2 },
        VmInstruction::Halt,
        VmInstruction::Noop,
    ];
    apply(&mut g, TopologyOperator::AddRouteTarget, 7).unwrap();
    let BackendDef::Vm(vm) = &g.nodes[0].backend_def else {
        panic!()
    };
    let VmInstruction::Jump { offset } = vm.program[0] else {
        panic!()
    };
    assert_eq!(
        crate::runtime::vm::jump_target(0, offset, vm.program.len()),
        0
    );
    assert!(matches!(
        vm.program[1],
        VmInstruction::WriteRouteGate { slot: 1, src: 0 }
    ));
    assert_eq!(vm.program[2], VmInstruction::Halt);
}

#[test]
fn node_removal_bypasses_all_incoming_edges_and_protects_terminal() {
    let mut g = genome(vec![node(0, &[1, 2]), node(1, &[2]), node(2, &[])]);
    g.nodes[0].targets[0].gate_bias = 1.5;
    let old = g.nodes[0].targets.clone();
    apply(&mut g, TopologyOperator::RemoveNode, 7).unwrap();
    assert_eq!(g.nodes.len(), 2);
    assert_eq!(g.nodes[0].targets[0].target_id, NodeId::new(2));
    assert_eq!(g.nodes[0].targets[0].slot, old[0].slot);
    assert_eq!(g.nodes[0].targets[0].gate_bias, old[0].gate_bias);
    assert_eq!(g.nodes[0].targets[1], old[1]);
    let before = g.clone();
    assert_eq!(
        apply(&mut g, TopologyOperator::RemoveNode, 7),
        Err(MutationSkipReason::NoApplicableTarget)
    );
    assert_eq!(g, before);
}

#[test]
fn unreachable_terminal_removal_cleans_incoming_references() {
    let mut g = genome(vec![node(0, &[]), node(1, &[2]), node(2, &[])]);
    // Either unreachable node can be selected. No removed reference may remain.
    for seed in 0..20 {
        let mut copy = g.clone();
        apply(&mut copy, TopologyOperator::RemoveNode, seed).unwrap();
        for target in copy.nodes.iter().flat_map(|n| &n.targets) {
            assert!(copy.nodes.iter().any(|n| n.node_id == target.target_id));
        }
    }
    g.nodes[1].targets.clear();
}

proptest! {
    #[test]
    fn growth_preserves_unique_ids_at_wrap(seed in any::<u64>(), extra in 2u32..1000) {
        let base = genome(vec![node(0, &[1]), node(1, &[]), node(u32::MAX, &[]), node(extra, &[])]);
        for op in [TopologyOperator::AddNode, TopologyOperator::AddRouteTarget, TopologyOperator::CopyNode, TopologyOperator::SwapNodeBackend, TopologyOperator::SpliceNode, TopologyOperator::CopyMeshBackwardSlice, TopologyOperator::CopyMeshForwardSlice] {
            let mut g = base.clone();
            apply(&mut g, op, seed).unwrap();
            let ids: std::collections::HashSet<_> = g.nodes.iter().map(|n| n.node_id).collect();
            prop_assert_eq!(ids.len(), g.nodes.len());
            for n in &g.nodes { for target in &n.targets { prop_assert!(ids.contains(&target.target_id)); } }
        }
    }

    #[test]
    fn full_slice_attachment_skips_without_partial_growth(seed in any::<u64>()) {
        let mut g = genome(vec![node(0, &[0,0,0,0,0,0,0,0])]);
        for op in [TopologyOperator::CopyMeshBackwardSlice, TopologyOperator::CopyMeshForwardSlice, TopologyOperator::CopyNode, TopologyOperator::SwapNodeBackend] {
            let before = g.clone();
            prop_assert_eq!(apply(&mut g, op, seed), Err(MutationSkipReason::NoApplicableTarget));
            prop_assert_eq!(&g, &before);
        }
    }

    #[test]
    fn local_retarget_preserves_route_fields_and_uses_neighborhood(seed in any::<u64>(), bias in -4f32..4f32) {
        let mut g = genome(vec![node(0, &[1, 2]), node(1, &[2, 3, 3, 99]), node(2, &[]), node(3, &[]), node(4, &[])]);
        g.nodes[0].targets[0].gate_bias = bias;
        let before = g.clone();
        apply(&mut g, TopologyOperator::RetargetNodeTarget, seed).unwrap();
        let mut changed = 0;
        for (index, (old, new)) in before.nodes.iter().zip(&g.nodes).enumerate() {
            prop_assert_eq!(old.targets.len(), new.targets.len());
            for (a, b) in old.targets.iter().zip(&new.targets) {
                prop_assert_eq!(a.slot, b.slot); prop_assert_eq!(a.gate_bias, b.gate_bias);
                if a.target_id != b.target_id {
                    changed += 1;
                    prop_assert_ne!(b.target_id, old.node_id);
                    let mut local: Vec<_> = old.targets.iter().map(|t| t.target_id).collect();
                    if let Some(next) = before.nodes.iter().find(|n| n.node_id == a.target_id) { local.extend(next.targets.iter().map(|t| t.target_id)); }
                    prop_assert!(local.contains(&b.target_id));
                    prop_assert!(g.nodes.iter().any(|n| n.node_id == b.target_id), "node {index}");
                }
            }
        }
        prop_assert_eq!(changed, 1);
    }

    #[test]
    fn route_removal_keeps_incumbent_and_every_surviving_field(seed in any::<u64>(), biases in prop::collection::vec(-4f32..4f32, 2..9)) {
        let mut g = genome(vec![node(0, &vec![1; biases.len()]), node(1, &[])]);
        for (target, bias) in g.nodes[0].targets.iter_mut().zip(biases) { target.gate_bias = bias; }
        let old = g.nodes[0].targets.clone();
        let incumbent = old[routing::static_incumbent(&old).unwrap()];
        apply(&mut g, TopologyOperator::RemoveRouteTarget, seed).unwrap();
        prop_assert_eq!(g.nodes[0].targets.len(), old.len() - 1);
        prop_assert!(g.nodes[0].targets.contains(&incumbent));
        for target in &g.nodes[0].targets { prop_assert!(old.contains(target)); }
    }
}

#[test]
fn removal_uses_current_topology_when_parent_reachability_cache_is_stale() {
    let mut g = genome(vec![node(0, &[1]), node(1, &[])]);
    let before = g.clone();
    assert_eq!(
        TopologyMutator::apply(
            &mut g,
            TopologyOperator::RemoveNode,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut SmallRng::seed_from_u64(7),
            &MutationConfig::default()
        ),
        Err(MutationSkipReason::NoApplicableTarget)
    );
    assert_eq!(g, before);
    // Earlier event joined a formerly unreachable terminal into the live pathway.
    g.nodes.push(node(2, &[]));
    g.nodes[0].targets[0].target_id = NodeId::new(2);
    TopologyMutator::apply(
        &mut g,
        TopologyOperator::RemoveNode,
        &mut TargetSelector::reachable_only(&[0, 1], 1.0),
        &mut SmallRng::seed_from_u64(7),
        &MutationConfig::default(),
    )
    .unwrap();
    assert!(g.nodes.iter().any(|n| n.node_id == NodeId::new(2)));
    assert!(!g.nodes.iter().any(|n| n.node_id == NodeId::new(1)));
}

fn conditional_fixture(graph: bool) -> CreatureGenome {
    use crate::contracts::WorldInputKey;
    let mut g = genome(vec![node(0, &[1]), node(1, &[])]);
    g.nodes[0].input_refs = vec![InputReference::World(WorldInputKey::FoodHere {
        type_idx: Default::default(),
    })];
    if graph {
        let mut backend = CgpGraphBackendDef::new_with_fixed_outputs(&MutationConfig::default());
        backend
            .compute_nodes
            .push(crate::creature::genome::cgp::ComputeNode {
                kind: crate::creature::genome::cgp::ComputeNodeKind::WeightedSum,
                inputs: vec![GraphEdge {
                    source: GraphSource::InputLeaf {
                        ref_idx: 0,
                        sub_idx: 0,
                    },
                    weight: 1.0,
                }],
                plasticity: None,
            });
        backend
            .output_sinks
            .iter_mut()
            .find(|s| matches!(s.kind, OutputSinkKind::CustomOutput(0)))
            .unwrap()
            .inputs
            .push(GraphEdge {
                source: GraphSource::SharedMemory {
                    slot: 3,
                    previous: false,
                },
                weight: 1.0,
            });
        g.nodes[0].backend_def = BackendDef::Graph(backend);
    } else {
        g.nodes[0].backend_def = BackendDef::Vm(VmBackendDef {
            register_count: 1,
            constants: vec![3.0],
            program: vec![
                VmInstruction::LoadConst {
                    dst: 0,
                    const_idx: 0,
                },
                VmInstruction::WriteInternalPayload {
                    slot_idx: 0,
                    src: 0,
                },
                VmInstruction::StoreSlotImm {
                    slot_idx: 3,
                    src: 0,
                },
                VmInstruction::SetPriorityBid { src: 0 },
                VmInstruction::PushAction { action_type: 1 },
                VmInstruction::ReadInput {
                    dst: 0,
                    ref_idx: 0,
                    sub_idx: 0,
                },
                VmInstruction::Halt,
            ],
        });
    }
    g.nodes[1].input_refs = vec![InputReference::UpstreamSlot(0)];
    g.nodes[1].backend_def = BackendDef::Vm(VmBackendDef {
        register_count: 1,
        constants: vec![],
        program: vec![
            VmInstruction::ReadInput {
                dst: 0,
                ref_idx: 0,
                sub_idx: 0,
            },
            VmInstruction::WriteWorldActionMeta {
                slot_idx: 0,
                src: 0,
            },
            VmInstruction::PushAction { action_type: 2 },
            VmInstruction::ExecuteActionQueue,
        ],
    });
    g
}
fn execute_with_config(
    g: &CreatureGenome,
    input: f32,
    energy: f32,
    config: &crate::config::RuntimeConfig,
) -> (
    crate::runtime::types::MeshOutput,
    crate::runtime::mesh::MeshObservation,
    [f32; 16],
) {
    let sensors = sensors(input);
    let mut energy = energy;
    let mut memory = [3.0; 16];
    let (output, observation) = crate::runtime::mesh::execute_creature_mesh_impl(
        g,
        &sensors,
        &mut energy,
        &mut memory,
        &[1.0; 16],
        &mut crate::creature::state::GraphRuntimeState::new(),
        config,
        crate::runtime::mesh::ObservedMeshExecution::default(),
    );
    (output, observation, memory)
}

fn execute(
    g: &CreatureGenome,
    input: f32,
    energy: f32,
) -> (
    crate::runtime::types::MeshOutput,
    crate::runtime::mesh::MeshObservation,
    [f32; 16],
) {
    let mut config = crate::config::RuntimeConfig::default();
    config.vm.opcode_cost_multiplier = 1.0;
    execute_with_config(g, input, energy, &config)
}

#[test]
fn added_work_retains_default_behavior_but_can_exhaust_at_the_boundary() {
    let base = conditional_fixture(false);
    let mut grown = base.clone();
    apply(&mut grown, TopologyOperator::AddRouteTarget, 7).unwrap();
    grown.nodes.last_mut().unwrap().backend_def = birth::minimal_vm_backend();
    let config = crate::config::RuntimeConfig::default();
    for input in [0.0, 1.0] {
        let before = execute_with_config(&base, input, 80.0, &config);
        let after = execute_with_config(&grown, input, 80.0, &config);
        assert_eq!(before.0.actions, after.0.actions);
        assert_eq!(before.2, after.2);
        assert_eq!(
            after.0.work_counters.vm_steps,
            before.0.work_counters.vm_steps + 1 + u32::from(input > 0.0)
        );
        assert_eq!(
            after.0.work_counters.mesh_hops,
            before.0.work_counters.mesh_hops + u32::from(input > 0.0)
        );
    }
    let base_cost = execute(&base, 1.0, 1000.0).0.cost_report.vm_cost;
    let budget = base_cost + 0.025;
    let before = execute(&base, 1.0, budget);
    let after = execute(&grown, 1.0, budget);
    assert!(matches!(
        before.1.termination_reason,
        crate::runtime::trace::domain::TerminationReason::ActionEmitted
    ));
    assert!(matches!(
        after.1.termination_reason,
        crate::runtime::trace::domain::TerminationReason::EnergyExhausted
    ));
    assert_ne!(before.0.actions, after.0.actions);
    assert_eq!(after.0.actions, vec![crate::contracts::WorldAction::NoOp]);
}

#[test]
fn one_production_addition_is_conditional_silent_and_can_diverge_on_both_backends() {
    use crate::mutation::vm::{VmMutator, VmOperator};
    for graph in [false, true] {
        let base = conditional_fixture(graph);
        let mut grown = base.clone();
        apply(&mut grown, TopologyOperator::AddRouteTarget, 7).unwrap();
        grown.nodes.last_mut().unwrap().backend_def = birth::minimal_vm_backend();
        if let BackendDef::Graph(g) = &grown.nodes[0].backend_def {
            let sink = g
                .output_sinks
                .iter()
                .find(|s| matches!(s.kind, OutputSinkKind::RouterGate(1)))
                .unwrap();
            assert!(
                matches!(
                    sink.inputs.last().unwrap().source,
                    GraphSource::ComputeNode(0)
                ),
                "seed 7 graph gate {:?}",
                sink.inputs
            );
        }
        for input in [0.0, 1.0] {
            let old = execute(&base, input, 1000.0);
            let new = execute(&grown, input, 1000.0);
            assert_eq!(old.0.actions, new.0.actions);
            assert_eq!(old.0.priority_bid, new.0.priority_bid);
            assert_eq!(old.2, new.2);
            assert_eq!(new.1.hops[0].1, Some(usize::from(input > 0.0)));
            if !graph || input > 0.0 {
                assert!(
                    new.0.cost_report.vm_cost + new.0.cost_report.graph_cost
                        > old.0.cost_report.vm_cost + old.0.cost_report.graph_cost
                );
            }
        }
        // A fixed, bounded ordinary mutation search demonstrates copy-and-diverge supply.
        let mut divergent = None;
        for seed in 0..256 {
            let mut candidate = grown.clone();
            VmMutator::apply(
                &mut candidate,
                VmOperator::VmInstructionMutation,
                &mut TargetSelector::reachable_only(&[2], 1.0),
                &mut SmallRng::seed_from_u64(seed),
                &MutationConfig::default(),
            )
            .unwrap();
            if execute(&candidate, 1.0, 1000.0).0.actions != execute(&grown, 1.0, 1000.0).0.actions
            {
                divergent = Some(candidate);
                break;
            }
        }
        let divergent = divergent
            .expect("fixed mutation trials must contain an action-changing detour insertion");
        assert_eq!(
            execute(&divergent, 0.0, 1000.0).0.actions,
            execute(&grown, 0.0, 1000.0).0.actions
        );
    }
}

#[test]
fn dormant_alternative_cannot_win_even_when_other_routes_have_dynamic_bids() {
    for op in [
        TopologyOperator::CopyNode,
        TopologyOperator::SwapNodeBackend,
    ] {
        let mut g = conditional_fixture(false);
        let mut other = g.nodes[1].clone();
        other.node_id = NodeId::new(2);
        g.nodes.push(other);
        g.nodes[0].targets.push(RouteTarget {
            target_id: NodeId::new(2),
            slot: 1,
            gate_bias: 0.0,
        });
        gate(&mut g.nodes[0], 1);
        // Place a varying bid after the ReadInput, before Halt.
        if let BackendDef::Vm(vm) = &mut g.nodes[0].backend_def {
            vm.program.remove(0);
            let at = vm.program.len() - 1;
            vm.program
                .insert(at, VmInstruction::WriteRouteGate { slot: 1, src: 0 });
        }
        let old = g.clone();
        apply(&mut g, op, 7).unwrap();
        for input in [0.0, 1.0] {
            let before = execute(&old, input, 1000.0);
            let after = execute(&g, input, 1000.0);
            assert_eq!(before.0.actions, after.0.actions);
            assert_eq!(before.2, after.2);
            assert_eq!(before.1.hops, after.1.hops);
            assert_eq!(after.1.hops[0].1, Some(usize::from(input > 0.0)));
        }
    }
}

#[test]
fn graph_alternative_can_grow_a_vm_without_erasing_original() {
    let mut g = genome(vec![node(0, &[1]), node(1, &[])]);
    g.nodes[1].backend_def = BackendDef::Graph(CgpGraphBackendDef::new_with_fixed_outputs(
        &MutationConfig::default(),
    ));
    let old = g.nodes[1].clone();
    apply(&mut g, TopologyOperator::SwapNodeBackend, 7).unwrap();
    assert_eq!(g.nodes[1], old);
    assert!(
        matches!(&g.nodes[2].backend_def,BackendDef::Vm(vm) if vm.program==vec![VmInstruction::Halt])
    );
    apply(&mut g, TopologyOperator::SwapRouteTargets, 7).unwrap();
    assert_eq!(g.nodes[0].targets[0].target_id, g.nodes[2].node_id);
}

#[test]
fn paired_graph_gates_reach_all_sensor_subvalues_without_resampling() {
    use crate::contracts::WorldInputKey;
    let mut base = conditional_fixture(true);
    base.nodes[0].input_refs = vec![InputReference::World(WorldInputKey::NeighborBarrierRing)];
    let mut seen = std::collections::BTreeSet::new();
    let mut memory_sources = 0;
    for seed in 0..256 {
        let mut g = base.clone();
        apply(&mut g, TopologyOperator::AddRouteTarget, seed).unwrap();
        let BackendDef::Graph(graph) = &g.nodes[0].backend_def else {
            panic!()
        };
        let edge = graph
            .output_sinks
            .iter()
            .find(|s| matches!(s.kind, OutputSinkKind::RouterGate(1)))
            .unwrap()
            .inputs
            .last()
            .unwrap();
        assert_eq!(edge.weight, 1.0);
        match edge.source {
            GraphSource::InputLeaf {
                ref_idx: 0,
                sub_idx,
            } => {
                seen.insert(sub_idx);
            }
            GraphSource::SharedMemory { .. } => memory_sources += 1,
            GraphSource::ComputeNode(0) => {}
            _ => panic!("invalid source"),
        }
    }
    assert_eq!(seen, (0..8).collect());
    assert!(memory_sources > 0);
}

#[test]
fn vm_gate_can_append_without_terminal_and_preserve_shifted_jump_target() {
    for terminal in [VmInstruction::Halt, VmInstruction::ExecuteActionQueue] {
        let mut g = genome(vec![node(0, &[1]), node(1, &[])]);
        let BackendDef::Vm(vm) = &mut g.nodes[0].backend_def else {
            panic!()
        };
        vm.program = vec![
            VmInstruction::Jump { offset: 1 },
            terminal.clone(),
            VmInstruction::Noop,
        ];
        apply(&mut g, TopologyOperator::AddRouteTarget, 7).unwrap();
        let BackendDef::Vm(vm) = &g.nodes[0].backend_def else {
            panic!()
        };
        let VmInstruction::Jump { offset } = vm.program[0] else {
            panic!()
        };
        assert_eq!(
            crate::runtime::vm::jump_target(0, offset, vm.program.len()),
            3
        );
        assert_eq!(vm.program[2], terminal);
    }
    let mut g = genome(vec![node(0, &[1]), node(1, &[])]);
    if let BackendDef::Vm(vm) = &mut g.nodes[0].backend_def {
        vm.program = vec![VmInstruction::Noop];
    }
    apply(&mut g, TopologyOperator::AddRouteTarget, 7).unwrap();
    let BackendDef::Vm(vm) = &g.nodes[0].backend_def else {
        panic!()
    };
    assert!(matches!(
        vm.program[1],
        VmInstruction::WriteRouteGate { slot: 1, src: 0 }
    ));
}

#[test]
fn inline_growth_preserves_bus_queue_priority_memory_and_exposes_real_energy_cost() {
    for op in [TopologyOperator::AddNode, TopologyOperator::SpliceNode] {
        let base = conditional_fixture(false);
        let mut grown = base.clone();
        apply(&mut grown, op, 7).unwrap();
        for input in [0.0, 1.0] {
            let before = execute(&base, input, 1000.0);
            let after = execute(&grown, input, 1000.0);
            assert_eq!(before.0.actions, after.0.actions);
            assert_eq!(before.0.priority_bid, after.0.priority_bid);
            assert_eq!(before.2, after.2);
            assert_eq!(
                after.0.work_counters.mesh_hops,
                before.0.work_counters.mesh_hops + 1
            );
        }
    }
    let mut base = genome(vec![node(0, &[1]), node(1, &[])]);
    base.nodes[1].input_refs = vec![InputReference::DynamicIntrospection(
        crate::contracts::DynamicIntrospectionKey::EnergyConsumedThisTick,
    )];
    base.nodes[1].backend_def = BackendDef::Vm(VmBackendDef {
        register_count: 1,
        constants: vec![],
        program: vec![
            VmInstruction::ReadInput {
                dst: 0,
                ref_idx: 0,
                sub_idx: 0,
            },
            VmInstruction::SetPriorityBid { src: 0 },
            VmInstruction::Halt,
        ],
    });
    let mut grown = base.clone();
    apply(&mut grown, TopologyOperator::AddNode, 7).unwrap();
    grown.nodes.last_mut().unwrap().backend_def = birth::minimal_vm_backend();
    let before = execute(&base, 0.0, 1000.0);
    let after = execute(&grown, 0.0, 1000.0);
    assert!(
        before.0.priority_bid < after.0.priority_bid,
        "live energy reads observe the extra Halt charge"
    );
}

proptest! {
    #[test]
    fn node_bypass_preserves_surviving_routes_and_leaves_no_dangling_reference(seed in any::<u64>(), length in 3u32..20) {
        let mut g=genome((0..length).map(|id| node(id,&if id+1<length {vec![id+1]} else {vec![]})).collect());
        let before=g.clone();apply(&mut g,TopologyOperator::RemoveNode,seed).unwrap();
        prop_assert_eq!(g.nodes.len()+1,before.nodes.len());
        prop_assert_eq!(g.entry_node_id,before.entry_node_id);
        for node in &g.nodes {
            let original=before.nodes.iter().find(|n|n.node_id==node.node_id).unwrap();
            prop_assert_eq!(node.targets.len(),original.targets.len());
            for (new,old) in node.targets.iter().zip(&original.targets) {
                prop_assert_eq!(new.slot,old.slot);prop_assert_eq!(new.gate_bias,old.gate_bias);
                prop_assert!(g.nodes.iter().any(|n|n.node_id==new.target_id));
            }
        }
    }
}

proptest! {
    #[test]
    fn static_incumbent_uses_highest_bias_and_earliest_tie(bias in -1000f32..1000f32) {
        let mut n = node(0, &[1, 2, 3]);
        n.targets[0].gate_bias = bias - 1.0;
        n.targets[1].gate_bias = bias;
        n.targets[2].gate_bias = bias;
        prop_assert_eq!(routing::static_incumbent(&n.targets), Some(1));
        prop_assert_eq!(routing::static_incumbent(&[]), None);
    }

    #[test]
    fn removal_rejects_reachable_self_or_missing_successor(seed in any::<u64>()) {
        for successor in [1, 99] {
            let mut g = genome(vec![node(0, &[1]), node(1, &[successor])]);
            let before = g.clone();
            prop_assert_eq!(apply(&mut g, TopologyOperator::RemoveNode, seed), Err(MutationSkipReason::NoApplicableTarget));
            prop_assert_eq!(g, before);
        }
    }

    #[test]
    fn splice_rejects_missing_successor_without_partial_growth(seed in any::<u64>()) {
        for op in [TopologyOperator::AddNode, TopologyOperator::SpliceNode] {
            let mut g = genome(vec![node(0, &[99]), node(1, &[])]);
            let before = g.clone();
            prop_assert_eq!(apply(&mut g, op, seed), Err(MutationSkipReason::NoApplicableTarget));
            prop_assert_eq!(g, before);
        }
    }
}

#[test]
fn f18_growth_backend_choice_is_equal_and_source_independent() {
    for source_graph in [false, true] {
        for op in [
            TopologyOperator::AddNode,
            TopologyOperator::SpliceNode,
            TopologyOperator::AddRouteTarget,
        ] {
            let mut probe = conditional_fixture(source_graph);
            let mut count = BackendDraw {
                draw: 0,
                calls: 0,
                at: usize::MAX,
            };
            TopologyMutator::apply(
                &mut probe,
                op,
                &mut TargetSelector::reachable_only(&[0, 1], 0.0),
                &mut count,
                &MutationConfig::default(),
            )
            .unwrap();
            // Replay the zero-valued preceding draws, varying only the final
            // backend Bernoulli draw at and immediately below its half boundary.
            for (draw, graph) in [
                (0, true),
                ((1u64 << 63) - 1, true),
                (1u64 << 63, false),
                (u64::MAX, false),
            ] {
                let mut g = conditional_fixture(source_graph);
                let mut rng = BackendDraw {
                    draw,
                    calls: 0,
                    at: count.calls - 1,
                };
                TopologyMutator::apply(
                    &mut g,
                    op,
                    &mut TargetSelector::reachable_only(&[0, 1], 0.0),
                    &mut rng,
                    &MutationConfig::default(),
                )
                .unwrap();
                assert_eq!(
                    matches!(g.nodes[2].backend_def, BackendDef::Graph(_)),
                    graph,
                    "{op:?} source_graph={source_graph} draw={draw}"
                );
            }
        }
    }
}

// Keep all range sampling at zero and control only the final u64 draw.
struct BackendDraw {
    draw: u64,
    calls: usize,
    at: usize,
}
impl rand::RngCore for BackendDraw {
    fn next_u32(&mut self) -> u32 {
        0
    }
    fn next_u64(&mut self) -> u64 {
        let value = if self.calls == self.at { self.draw } else { 0 };
        self.calls += 1;
        assert!(self.calls <= 32, "unexpected RNG rejection loop");
        value
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        dest.fill(0);
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

fn sensors(input: f32) -> crate::sensors::perception::SensorSnapshot {
    use crate::sensors::{
        perception::{PerceptionSnapshot, SensorSnapshot},
        static_inputs::StaticInputs,
        typed_food::TypedFoodLocalSnapshot,
    };
    SensorSnapshot {
        local: StaticInputs {
            food_here: input,
            neighbor_food: [0.0; 8],
            neighbor_barrier: [0.0; 8],
            neighbor_occupied: [0.0; 8],
            age_ticks: 0.0,
            max_energy: 200.0,
        },
        typed_local_food: TypedFoodLocalSnapshot {
            food_here_by_type: vec![input],
            neighbor_food_by_type: vec![[0.0; 8]],
        },
        perception: PerceptionSnapshot::zeroed(1),
    }
}

#[path = "f18_tests.rs"]
mod f18_tests;
