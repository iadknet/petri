use super::mesh::*;
use crate::config::RuntimeConfig;
use crate::contracts::{NodeId, RouteTarget};
use crate::creature::genome::{
    BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
};
use crate::creature::state::GraphRuntimeState;
use crate::runtime::trace::domain::TerminationReason;
use crate::runtime::traced_mesh::execute_creature_mesh_traced;
use crate::sensors::{
    perception::{PerceptionSnapshot, SensorSnapshot},
    static_inputs::StaticInputs,
    typed_food::TypedFoodLocalSnapshot,
};
use proptest::prelude::*;

fn sensors() -> SensorSnapshot {
    SensorSnapshot {
        local: StaticInputs {
            food_here: 0.0,
            neighbor_food: [0.0; 8],
            neighbor_barrier: [0.0; 8],
            neighbor_occupied: [0.0; 8],
            max_energy: 200.0,
            age_ticks: 0.0,
        },
        typed_local_food: TypedFoodLocalSnapshot::zeroed(1),
        perception: PerceptionSnapshot::zeroed(1),
    }
}
fn node(id: u32, targets: &[u32]) -> NodeGenome {
    NodeGenome {
        node_id: NodeId::new(id),
        input_refs: vec![],
        backend_def: BackendDef::Vm(VmBackendDef {
            register_count: 1,
            constants: vec![2.0],
            program: vec![
                VmInstruction::LoadConst {
                    dst: 0,
                    const_idx: 0,
                },
                VmInstruction::StoreSlotImm {
                    slot_idx: 0,
                    src: 0,
                },
                VmInstruction::SetPriorityBid { src: 0 },
                VmInstruction::PushAction { action_type: 1 },
                VmInstruction::Halt,
            ],
        }),
        targets: targets
            .iter()
            .enumerate()
            .map(|(i, &id)| RouteTarget {
                target_id: NodeId::new(id),
                slot: i as u8,
                gate_bias: -(i as f32),
            })
            .collect(),
    }
}
fn observe(g: &CreatureGenome, cap: u32) -> MeshObservation {
    let config = RuntimeConfig {
        max_mesh_hops: cap,
        ..RuntimeConfig::default()
    };
    execute_creature_mesh_impl(
        g,
        &sensors(),
        &mut 1000.0,
        &mut [0.0; 16],
        &[0.0; 16],
        &mut GraphRuntimeState::new(),
        &config,
        ObservedMeshExecution::default(),
    )
    .1
}
#[test]
fn visited_top_choice_falls_through_and_visits_reset_each_tick() {
    let g = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![node(0, &[1]), node(1, &[0, 2]), node(2, &[1, 2])],
    };
    for _ in 0..2 {
        let result = observe(&g, 10);
        assert_eq!(
            result.hops,
            vec![
                (NodeId::new(0), Some((0, NodeId::new(1)))),
                (NodeId::new(1), Some((1, NodeId::new(2)))),
                (NodeId::new(2), None)
            ]
        );
        assert!(matches!(
            result.termination_reason,
            TerminationReason::NoTargets
        ));
    }
}
#[test]
fn cap_is_real_acyclic_limit_and_final_dispatch_can_complete() {
    let g = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![node(0, &[1]), node(1, &[2]), node(2, &[])],
    };
    assert!(matches!(
        observe(&g, 2).termination_reason,
        TerminationReason::MaxHopsReached
    ));
    assert_eq!(observe(&g, 2).hops.len(), 2);
    assert!(matches!(
        observe(&g, 3).termination_reason,
        TerminationReason::NoTargets
    ));
    assert_eq!(observe(&g, 3).hops.len(), 3);
}
#[test]
fn missing_winner_soft_terminates_without_fallback() {
    let g = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![node(0, &[99, 1]), node(1, &[])],
    };
    let result = observe(&g, 10);
    assert_eq!(
        result.hops,
        vec![(NodeId::new(0), Some((0, NodeId::new(99))))]
    );
    assert!(matches!(
        result.termination_reason,
        TerminationReason::MissingNode
    ));
}
#[test]
fn all_three_modes_match_cycle_fallback_energy_memory_priority_and_cost() {
    let g = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![node(0, &[1]), node(1, &[0, 2]), node(2, &[1])],
    };
    for cap in [1, 2, 3, 10] {
        for start in [0.0, 0.1, 1.0, 1000.0] {
            let config = RuntimeConfig {
                max_mesh_hops: cap,
                ..RuntimeConfig::default()
            };
            let mut energies = [start; 3];
            let mut memories = [[0.5; 16]; 3];
            let mut states = [
                GraphRuntimeState::new(),
                GraphRuntimeState::new(),
                GraphRuntimeState::new(),
            ];
            for tick in 0..2 {
                for state in &mut states {
                    state.begin_tick(&g.nodes, tick);
                }
                let plain = execute_creature_mesh_impl(
                    &g,
                    &sensors(),
                    &mut energies[0],
                    &mut memories[0],
                    &[0.25; 16],
                    &mut states[0],
                    &config,
                    UntracedMeshExecution,
                );
                let (observed, observation) = execute_creature_mesh_impl(
                    &g,
                    &sensors(),
                    &mut energies[1],
                    &mut memories[1],
                    &[0.25; 16],
                    &mut states[1],
                    &config,
                    ObservedMeshExecution::default(),
                );
                let (traced, hops, reason) = execute_creature_mesh_traced(
                    &g,
                    &sensors(),
                    &mut energies[2],
                    &mut memories[2],
                    &[0.25; 16],
                    &mut states[2],
                    &config,
                );
                for output in [&observed, &traced] {
                    assert_eq!(plain.actions, output.actions);
                    assert_eq!(plain.priority_bid, output.priority_bid);
                    assert_eq!(plain.cost_report.vm_cost, output.cost_report.vm_cost);
                    assert_eq!(plain.cost_report.graph_cost, output.cost_report.graph_cost);
                    assert_eq!(plain.work_counters, output.work_counters);
                }
                assert_eq!(energies, [energies[0]; 3]);
                assert_eq!(memories, [memories[0]; 3]);
                assert_eq!(
                    format!("{reason:?}"),
                    format!("{:?}", observation.termination_reason)
                );
                assert_eq!(
                    hops.iter().map(|h| h.node_id).collect::<Vec<_>>(),
                    observation
                        .hops
                        .iter()
                        .map(|(id, _)| *id)
                        .collect::<Vec<_>>()
                );
                for (hop, (_, route)) in hops.iter().zip(&observation.hops) {
                    if let Some((position, destination)) = route {
                        let decision = hop.route.as_ref().unwrap();
                        assert_eq!(decision.selected_target_idx, *position);
                        assert_eq!(decision.selected_target_id, *destination);
                    }
                }
            }
        }
    }
}
proptest! {
    #[test]
    fn dispatch_ids_are_unique_and_bounded_by_nodes_and_cap(edges in prop::collection::vec(prop::collection::vec(0u32..20,0..8),1..20), cap in 1u32..25) {
        let g=CreatureGenome { entry_node_id:NodeId::new(0),nodes:edges.iter().enumerate().map(|(i,e)|node(i as u32,e)).collect() };
        let result=observe(&g,cap);
        let ids:std::collections::HashSet<_>=result.hops.iter().map(|(id,_)|*id).collect();
        prop_assert_eq!(ids.len(),result.hops.len());
        prop_assert!(ids.len()<=g.nodes.len()); prop_assert!(ids.len()<=cap as usize);
    }
}
