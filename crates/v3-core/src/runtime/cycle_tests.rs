//! T19.F02 cycle fixtures: revisits are legal, the per-pass cap is the
//! only structural exit, and the three execution modes agree on a cycle.

use super::mesh::*;
use crate::config::RuntimeConfig;
use crate::contracts::{NodeId, RouteTarget};
use crate::creature::genome::cgp::{CgpGraphBackendDef, ComputeNode, ComputeNodeKind};
use crate::creature::genome::{
    BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
};
use crate::creature::state::GraphRuntimeState;
use crate::runtime::trace::domain::TerminationReason;
use crate::runtime::traced_mesh::execute_creature_mesh_traced;
use crate::runtime::types::MeshOutput;
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
                // A `Terminate` vote commits nothing, so every tick is one
                // pass and the cycle fixtures read the pass directly.
                VmInstruction::AddVote {
                    sink: crate::creature::genome::vote::VoteSink::Terminate.index() as u8,
                    src: 0,
                },
                VmInstruction::Halt,
            ],
        }),
        targets: self::targets(targets),
    }
}
fn targets(targets: &[u32]) -> Vec<RouteTarget> {
    targets
        .iter()
        .enumerate()
        .map(|(i, &id)| RouteTarget {
            target_id: NodeId::new(id),
            slot: i as u8,
            gate_bias: -(i as f32),
        })
        .collect()
}
/// A one-constant graph node with no sinks: it routes to its first target.
fn graph_node(id: u32, target_ids: &[u32]) -> NodeGenome {
    NodeGenome {
        node_id: NodeId::new(id),
        input_refs: vec![],
        backend_def: BackendDef::Graph(CgpGraphBackendDef {
            birth_weights: None,
            compute_nodes: vec![ComputeNode {
                kind: ComputeNodeKind::Constant(1.0),
                inputs: vec![],
                plasticity: None,
            }],
            output_sinks: vec![],
        }),
        targets: targets(target_ids),
    }
}
fn observe(g: &CreatureGenome, cap: u32) -> (MeshOutput, MeshObservation) {
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
}
#[test]
fn top_choice_revisits_and_a_cycle_runs_to_the_pass_cap() {
    let g = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![node(0, &[1]), node(1, &[0, 2]), node(2, &[1, 2])],
    };
    for _ in 0..2 {
        let (output, result) = observe(&g, 10);
        assert_eq!(result.hops.len(), 10);
        for (index, (id, route)) in result.hops.iter().enumerate() {
            let expected = NodeId::new((index % 2) as u32);
            assert_eq!(*id, expected, "hop {index}");
            assert_eq!(*route, Some((0, NodeId::new(((index + 1) % 2) as u32))));
        }
        assert_eq!(result.termination_reason, TerminationReason::NoDecision);
        assert_eq!(output.work_counters.pass_cap_hits, 1);
        assert_eq!(output.work_counters.passes, 1);
        assert_eq!(output.actions, vec![crate::contracts::WorldAction::NoOp]);
    }
}
#[test]
fn cap_is_a_per_pass_limit_and_the_final_dispatch_can_complete() {
    let g = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![node(0, &[1]), node(1, &[2]), node(2, &[])],
    };
    let (capped, result) = observe(&g, 2);
    assert_eq!(result.termination_reason, TerminationReason::NoDecision);
    assert_eq!(result.hops.len(), 2);
    assert_eq!(capped.work_counters.pass_cap_hits, 1);
    let (complete, result) = observe(&g, 3);
    assert_eq!(result.termination_reason, TerminationReason::NoDecision);
    assert_eq!(result.hops.len(), 3);
    assert_eq!(complete.work_counters.pass_cap_hits, 0);
}
/// Each backend's cost is the sum of its dispatches' debits: positive after
/// one dispatch, larger after three, and the whole energy delta is accounted
/// for by the backend costs, the hop ramp, and the settled bid.
#[test]
fn backend_costs_accumulate_every_dispatch_and_close_the_energy_account() {
    fn run(g: &CreatureGenome) -> (MeshOutput, f32) {
        // The default 1e-6 per opcode and 1e-5 per graph node are below the
        // f32 ulp at 1000 energy; visible rates make each debit measurable.
        let mut config = RuntimeConfig {
            max_mesh_hops: 3,
            graph_node_base_cost: 0.01,
            ..RuntimeConfig::default()
        };
        config.vm.opcode_cost_multiplier = 0.01;
        let mut energy = 1000.0;
        let (output, _) = execute_creature_mesh_impl(
            g,
            &sensors(),
            &mut energy,
            &mut [0.0; 16],
            &[0.0; 16],
            &mut GraphRuntimeState::new(),
            &config,
            ObservedMeshExecution::default(),
        );
        (output, 1000.0 - energy)
    }
    type Build = fn(u32, &[u32]) -> NodeGenome;
    type Cost = fn(&MeshOutput) -> f32;
    let vm: (Build, Cost) = (node, |o| o.cost_report.vm_cost);
    let graph: (Build, Cost) = (graph_node, |o| o.cost_report.graph_cost);
    for (build, cost) in [vm, graph] {
        let single = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![build(0, &[])],
        };
        let chain = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![build(0, &[1]), build(1, &[2]), build(2, &[])],
        };
        let (one, one_spent) = run(&single);
        let (three, three_spent) = run(&chain);
        assert!(cost(&one) > 0.0, "{:?}", one.cost_report);
        assert!(
            cost(&three) > cost(&one),
            "three dispatches debit more than one: {:?} vs {:?}",
            three.cost_report,
            one.cost_report
        );
        for (output, spent) in [(&one, one_spent), (&three, three_spent)] {
            let report = &output.cost_report;
            let accounted =
                report.vm_cost + report.graph_cost + report.mesh_ramp_cost + output.priority_bid;
            assert!(
                (spent - accounted).abs() < 1e-3,
                "spent {spent} but accounted {accounted}: {report:?}"
            );
        }
    }
}
#[test]
fn missing_winner_soft_terminates_without_fallback() {
    let g = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![node(0, &[99, 1]), node(1, &[])],
    };
    let (_, result) = observe(&g, 10);
    assert_eq!(
        result.hops,
        vec![(NodeId::new(0), Some((0, NodeId::new(99))))]
    );
    assert_eq!(result.termination_reason, TerminationReason::NoDecision);
}
#[test]
fn all_three_modes_match_on_a_revisiting_cycle_energy_memory_priority_and_cost() {
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
                let (traced, hops, _) = execute_creature_mesh_traced(
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
                    assert_eq!(plain.termination_reason, output.termination_reason);
                    assert_eq!(plain.energy_observation, output.energy_observation);
                    assert_eq!(plain.priority_bid, output.priority_bid);
                    assert_eq!(plain.cost_report.vm_cost, output.cost_report.vm_cost);
                    assert_eq!(plain.cost_report.graph_cost, output.cost_report.graph_cost);
                    assert_eq!(plain.work_counters, output.work_counters);
                }
                assert_eq!(energies, [energies[0]; 3]);
                assert_eq!(memories, [memories[0]; 3]);
                assert_eq!(traced.termination_reason, observation.termination_reason);
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
    /// Over arbitrary small meshes, a pass dispatches at most `cap` nodes,
    /// only the cap ends a pass at exactly `cap` dispatches, and the cap
    /// counter records exactly that.
    #[test]
    fn dispatches_are_bounded_by_the_cap_and_the_cap_alone_counts_a_capped_pass(edges in prop::collection::vec(prop::collection::vec(0u32..20,0..8),1..20), cap in 1u32..25) {
        let g=CreatureGenome { entry_node_id:NodeId::new(0),nodes:edges.iter().enumerate().map(|(i,e)|node(i as u32,e)).collect() };
        let (output, result)=observe(&g,cap);
        prop_assert!(result.hops.len()<=cap as usize);
        prop_assert!(result.hops.iter().all(|(id,_)| g.nodes.iter().any(|n| n.node_id==*id)));
        prop_assert_eq!(output.work_counters.passes, 1);
        let capped = output.work_counters.pass_cap_hits == 1;
        prop_assert!(output.work_counters.pass_cap_hits <= 1);
        if capped { prop_assert_eq!(result.hops.len(), cap as usize); }
        prop_assert_eq!(output.work_counters.mesh_hops as usize, result.hops.len());
    }
}
