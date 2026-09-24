//! Backend-neutral growth: attachment properties, live activation, and reproduction evidence.
use super::*;
use crate::creature::state::GraphRuntimeState;
use crate::neighborhood::{mesh_execution::static_successor_bypass, Battery};
use crate::runtime::trace::domain::BackendTrace;

const GROWTH: [TopologyOperator; 3] = [
    TopologyOperator::AddNode,
    TopologyOperator::SpliceNode,
    TopologyOperator::AddRouteTarget,
];

fn blank(graph: bool) -> BackendDef {
    if graph {
        birth::blank_graph_backend()
    } else {
        birth::minimal_vm_backend()
    }
}

fn controlled_growth(g: &mut CreatureGenome, op: TopologyOperator, graph: bool) {
    let mut probe = g.clone();
    let mut count = BackendDraw {
        draw: 0,
        calls: 0,
        at: usize::MAX,
    };
    TopologyMutator::apply(
        &mut probe,
        op,
        &mut TargetSelector::reachable_only(&[0], 1.0),
        &mut count,
        &MutationConfig::default(),
    )
    .unwrap();
    let mut rng = BackendDraw {
        draw: if graph { 0 } else { u64::MAX },
        calls: 0,
        at: count.calls - 1,
    };
    TopologyMutator::apply(
        g,
        op,
        &mut TargetSelector::reachable_only(&[0], 1.0),
        &mut rng,
        &MutationConfig::default(),
    )
    .unwrap();
    assert_eq!(g.nodes.last().unwrap().backend_def, blank(graph));
}

proptest! {
    #[test]
    fn growth_preserves_survivors_routes_aliases_and_pass_through(bias in -10f32..10f32, slot in 0u8..7, value in -8f32..8f32) {
        for source_graph in [false, true] {
            for graph in [false, true] {
                let mut base = conditional_fixture(source_graph);
                base.nodes[0].targets[0].slot = slot;
                base.nodes[0].targets[0].gate_bias = bias;
                for op in GROWTH {
                    let mut grown = base.clone();
                    controlled_growth(&mut grown, op, graph);
                    prop_assert_eq!(grown.nodes.len(), base.nodes.len()+1);
                    prop_assert_eq!(&grown.nodes[1], &base.nodes[1]);
                    prop_assert_eq!(grown.entry_node_id,base.entry_node_id);
                    prop_assert_eq!(&grown.nodes[0].input_refs,&base.nodes[0].input_refs);
                    let new = grown.nodes.last().unwrap();
                    prop_assert!(!base.nodes.iter().any(|n| n.node_id == new.node_id));
                    prop_assert!(new.input_refs.is_empty());
                    prop_assert_eq!(new.targets.as_slice(), &[RouteTarget {target_id: NodeId::new(1),slot:0,gate_bias:0.0}]);
                    if op == TopologyOperator::AddRouteTarget {
                        prop_assert_eq!(grown.nodes[0].targets[0],base.nodes[0].targets[0]);
                        prop_assert_eq!(grown.nodes[0].targets.len(),2);
                        prop_assert_eq!(grown.nodes[0].targets[1].gate_bias,bias);
                        prop_assert_eq!(grown.nodes[0].targets[1].target_id,new.node_id);
                        prop_assert_ne!(grown.nodes[0].targets[1].slot,slot);
                        let added_slot=grown.nodes[0].targets[1].slot;
                        let mut restored=grown.nodes[0].backend_def.clone();
                        match &mut restored {
                            BackendDef::Vm(vm) => {
                                let at=vm.program.iter().position(|i| matches!(i,VmInstruction::WriteRouteGate {slot,..} if *slot==added_slot)).unwrap();
                                vm.program.remove(at);
                            }
                            BackendDef::Graph(g) => {g.output_sinks.iter_mut().find(|s| s.kind==OutputSinkKind::RouterGate(added_slot)).unwrap().inputs.pop();}
                        }
                        prop_assert_eq!(&restored,&base.nodes[0].backend_def);
                    } else {
                        prop_assert_eq!(&grown.nodes[0].backend_def,&base.nodes[0].backend_def);
                        prop_assert_eq!(grown.nodes[0].targets.len(),1);
                        prop_assert_eq!(grown.nodes[0].targets[0],RouteTarget {target_id:new.node_id, ..base.nodes[0].targets[0]});
                        let mut alias=base.clone();
                        controlled_growth(&mut alias, if op==TopologyOperator::AddNode {TopologyOperator::SpliceNode} else {TopologyOperator::AddNode},graph);
                        prop_assert_eq!(&grown,&alias);
                    }
                    let before=execute(&base,value,1000.0); let after=execute(&grown,value,1000.0);
                    prop_assert_eq!(before.0.actions,after.0.actions);
                    prop_assert_eq!(before.0.priority_bid,after.0.priority_bid);
                    prop_assert_eq!(before.2,after.2);
                }
            }
        }
    }
}

fn stateful_fixture(source_graph: bool) -> CreatureGenome {
    use crate::creature::genome::cgp::{ComputeNode, ComputeNodeKind};
    let mut base = conditional_fixture(source_graph);
    // Keep a stateful surviving graph downstream of the attachment.
    let mut temporal = node(9, &[1]);
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    def.compute_nodes.push(ComputeNode {
        kind: ComputeNodeKind::DecayIntegrator(0.5),
        inputs: vec![GraphEdge {
            source: GraphSource::SharedMemory {
                slot: 3,
                previous: false,
            },
            weight: 1.0,
        }],
        plasticity: None,
    });
    def.output_sinks
        .iter_mut()
        .find(|s| s.kind == OutputSinkKind::WriteSlot(4))
        .unwrap()
        .inputs
        .push(GraphEdge {
            source: GraphSource::ComputeNode(0),
            weight: 1.0,
        });
    temporal.backend_def = BackendDef::Graph(def);
    base.nodes[0].targets[0].target_id = temporal.node_id;
    base.nodes.push(temporal);
    // Prelude ensures both source kinds receive a real queue, bid,
    // and nonzero bus. Its metadata remains local to that VM.
    let mut prelude = node(8, &[0]);
    prelude.backend_def = BackendDef::Vm(VmBackendDef {
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
            VmInstruction::WriteActionParam {
                field_idx: 0,
                src: 0,
            },
            VmInstruction::AddVote { sink: 0, src: 0 },
            VmInstruction::SetPriorityBid { src: 0 },
            VmInstruction::Halt,
        ],
    });
    base.entry_node_id = prelude.node_id;
    base.nodes.push(prelude);
    base
}

#[test]
fn both_detours_preserve_nonzero_bus_queue_local_metadata_and_temporal_state() {
    for source_graph in [false, true] {
        for graph in [false, true] {
            for op in GROWTH {
                let base = stateful_fixture(source_graph);
                let mut grown = base.clone();
                controlled_growth(&mut grown, op, graph);
                let mut runtime = crate::config::RuntimeConfig::default();
                runtime.vm.opcode_cost_multiplier = 1.0;
                let mut states = [GraphRuntimeState::new(), GraphRuntimeState::new()];
                let mut memory = [[3.0; 16]; 2];
                for (tick, input) in [0.0, 1.0, 2.0, 0.0].into_iter().enumerate() {
                    let mut results = Vec::new();
                    for (i, g) in [&base, &grown].into_iter().enumerate() {
                        states[i].begin_tick(&g.nodes, tick as u64);
                        let prev = memory[i];
                        results.push(crate::runtime::traced_mesh::execute_creature_mesh_traced(
                            g,
                            &sensors(input),
                            &mut 1000.0,
                            &mut memory[i],
                            &prev,
                            &mut states[i],
                            &runtime,
                        ));
                    }
                    let (before, old_hops, _) = &results[0];
                    let (after, new_hops, _) = &results[1];
                    assert!(before.actions.len() >= 2);
                    assert_eq!(before.actions, after.actions);
                    assert_eq!(before.priority_bid, after.priority_bid);
                    assert_eq!(memory[0], memory[1]);
                    for old in old_hops {
                        let new = new_hops
                            .iter()
                            .find(|n| n.node_id == old.node_id && n.pass_index == old.pass_index)
                            .unwrap();
                        assert_eq!(old.upstream_slots, new.upstream_slots);
                        assert_eq!(old.output_slots, new.output_slots);
                        if let (BackendTrace::Vm(a), BackendTrace::Vm(b)) =
                            (&old.backend_trace, &new.backend_trace)
                        {
                            assert_eq!(a.final_payload, b.final_payload);
                        }
                    }
                    for (old_idx, node) in base.nodes.iter().enumerate() {
                        let new_idx = grown
                            .nodes
                            .iter()
                            .position(|n| n.node_id == node.node_id)
                            .unwrap();
                        assert_eq!(
                            states[0].node_state.get(old_idx),
                            states[1].node_state.get(new_idx)
                        );
                        assert_eq!(
                            states[0].node_outputs.get(old_idx),
                            states[1].node_outputs.get(new_idx)
                        );
                        assert_eq!(
                            states[0].plasticity_weights.get(old_idx),
                            states[1].plasticity_weights.get(new_idx)
                        );
                    }
                    let visited = op != TopologyOperator::AddRouteTarget || input > 0.0;
                    assert_eq!(
                        new_hops
                            .iter()
                            .any(|h| h.node_id == grown.nodes.last().unwrap().node_id),
                        visited
                    );
                    let source = new_hops
                        .iter()
                        .find(|h| h.node_id == NodeId::new(0))
                        .unwrap();
                    if op == TopologyOperator::AddRouteTarget {
                        assert_eq!(
                            source.route.as_ref().unwrap().selected_target_idx,
                            usize::from(input > 0.0)
                        );
                    }
                    // Every pass pays the growth (T19.F04).
                    let passes = before.work_counters.passes;
                    assert_eq!(after.work_counters.passes, passes);
                    assert_eq!(
                        after.work_counters.mesh_hops,
                        before.work_counters.mesh_hops + passes * u32::from(visited)
                    );
                    assert_eq!(
                        after.work_counters.graph_relax_iters,
                        before.work_counters.graph_relax_iters
                    );
                    assert_eq!(
                        after.work_counters.vm_steps,
                        before.work_counters.vm_steps
                            + passes
                                * (u32::from(
                                    op == TopologyOperator::AddRouteTarget && !source_graph
                                ) + u32::from(visited && !graph))
                    );
                    if let Some(detour) = new_hops
                        .iter()
                        .find(|h| h.node_id == grown.nodes.last().unwrap().node_id)
                    {
                        if graph {
                            assert_eq!(detour.energy_before, detour.energy_after);
                        } else {
                            assert!(detour.energy_before > detour.energy_after);
                        }
                    }
                }
                assert_eq!(
                    genome(vec![grown.nodes.last().unwrap().clone()]).genome_size(),
                    if graph { 2 } else { 3 }
                );
                let mut paired = grown.clone();
                paired.nodes.last_mut().unwrap().backend_def = blank(!graph);
                assert_eq!(
                    if graph {
                        paired.genome_size() - grown.genome_size()
                    } else {
                        grown.genome_size() - paired.genome_size()
                    },
                    1
                );
            }
        }
    }
}

#[test]
fn extra_hop_exhausts_budget_for_both_backends() {
    for graph in [false, true] {
        let base = conditional_fixture(false);
        let mut grown = base.clone();
        controlled_growth(&mut grown, TopologyOperator::AddNode, graph);
        let config = crate::config::RuntimeConfig {
            max_mesh_hops: 2,
            ..Default::default()
        };
        let before = execute_with_config(&base, 1.0, 1000.0, &config);
        let after = execute_with_config(&grown, 1.0, 1000.0, &config);
        assert_ne!(before.0.actions, after.0.actions);
        assert!(after.0.work_counters.pass_cap_hits > 0);
    }
}

#[test]
#[ignore = "T11.F18 paired release characterization; run through scripts/bench-wait"]
fn t11_f18_paired_growth_characterization() {
    let started = std::time::Instant::now();
    let config = crate::config::SimulationConfig::default();
    let battery = Battery::generate(7);
    let founder = crate::creature::founder::v3alpha1_founder_genome();
    let reachable = mesh_reachable_nodes(&founder);
    let executed =
        battery.executed_indices(&founder, &config.runtime, config.shared_memory.decay_rate);
    let baseline = battery.signature(&founder, &config.runtime, config.shared_memory.decay_rate);
    for op in GROWTH {
        let mut applied = 0;
        let mut skipped = 0;
        let mut created = [0; 2];
        let mut visits = [0; 2];
        let mut contributes = [0; 2];
        let mut equal = [0; 2];
        for trial in 0..200 {
            let mut child = founder.clone();
            let mut targets = crate::mutation::reachability::TargetSets::new(&reachable, &executed)
                .selector(
                    config.mutation.reachable_bias.topology,
                    config.mutation.executed_bias,
                );
            if TopologyMutator::apply(
                &mut child,
                op,
                &mut targets,
                &mut SmallRng::seed_from_u64(20260908 + trial),
                &config.mutation,
            )
            .is_err()
            {
                skipped += 1;
                continue;
            }
            applied += 1;
            created[usize::from(matches!(
                child.nodes.last().unwrap().backend_def,
                BackendDef::Vm(_)
            ))] += 1;
            let id = child.nodes.last().unwrap().node_id;
            for (i, graph) in [true, false].into_iter().enumerate() {
                let mut pair = child.clone();
                pair.nodes.last_mut().unwrap().backend_def = blank(graph);
                let signature =
                    battery.signature(&pair, &config.runtime, config.shared_memory.decay_rate);
                equal[i] += usize::from(signature == baseline);
                let executed = battery
                    .executed_node_ids(&pair, &config.runtime, config.shared_memory.decay_rate)
                    .contains(&id);
                visits[i] += usize::from(executed);
                contributes[i] += usize::from(
                    executed
                        && battery.signature(
                            &static_successor_bypass(&pair, id),
                            &config.runtime,
                            config.shared_memory.decay_rate,
                        ) != signature,
                );
            }
        }
        println!("{op:?}: applied={applied} skipped={skipped} created_graph={} created_vm={} graph_executed={} vm_executed={} graph_contributing={} vm_contributing={} graph_parent_equal={} vm_parent_equal={}",created[0],created[1],visits[0],visits[1],contributes[0],contributes[1],equal[0],equal[1]);
        assert_eq!(equal, [applied; 2]);
        assert_eq!(contributes, [0; 2]);
        assert!(applied > 0);
    }
    println!("elapsed_ms={}", started.elapsed().as_millis());
    assert!(started.elapsed().as_secs_f64() < 30.0);
}

#[test]
fn both_backend_choices_keep_skips_atomic_and_branch_energy_boundary_real() {
    for graph in [false, true] {
        for source_graph in [false, true] {
            for op in GROWTH {
                let mut g = conditional_fixture(source_graph);
                g.nodes[0].targets[0].target_id = NodeId::new(99);
                let before = g.clone();
                let mut rng = BackendDraw {
                    draw: if graph { 0 } else { u64::MAX },
                    calls: 0,
                    at: 0,
                };
                assert_eq!(
                    TopologyMutator::apply(
                        &mut g,
                        op,
                        &mut TargetSelector::reachable_only(&[], 0.0),
                        &mut rng,
                        &MutationConfig::default()
                    ),
                    Err(MutationSkipReason::NoApplicableTarget)
                );
                assert_eq!(g, before);
                assert_eq!(rng.calls, 0);
            }
        }
        let base = conditional_fixture(false);
        let mut grown = base.clone();
        controlled_growth(&mut grown, TopologyOperator::AddRouteTarget, graph);
        // The fixture's 3.0 bid settles after the chain (T19.F02), outside `vm_cost`.
        let cost = execute(&base, 1.0, 1000.0).0.cost_report.vm_cost + 3.0;
        let before = execute(&base, 1.0, cost + 0.025);
        let after = execute(&grown, 1.0, cost + 0.025);
        assert!(matches!(
            before.1.termination_reason,
            crate::runtime::trace::domain::TerminationReason::NoDecision
        ));
        assert!(matches!(
            after.1.termination_reason,
            crate::runtime::trace::domain::TerminationReason::EnergyExhausted
        ));
    }
}
