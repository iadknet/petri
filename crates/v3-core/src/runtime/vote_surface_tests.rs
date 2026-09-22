//! T19.F03 vote surface: accumulation, commit boundaries, and inertness.
//!
//! The surface is wired everywhere a genome and a trace are represented while
//! nothing reads it, so every assertion here is about what is recorded, never
//! about a decision the recorded values caused.

use crate::config::RuntimeConfig;
use crate::contracts::{NodeId, RouteTarget};
use crate::creature::genome::cgp::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, ExecuteGate, GraphEdge, GraphSource,
    OutputSink, OutputSinkKind,
};
use crate::creature::genome::vote::{VoteKind, VoteSink, VOTE_SINK_COUNT};
use crate::creature::genome::{
    BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
};
use crate::creature::state::GraphRuntimeState;
use crate::runtime::mesh::{execute_creature_mesh_impl, ObservedMeshExecution};
use crate::runtime::traced_mesh::execute_creature_mesh_traced;
use crate::runtime::types::MeshOutput;
use crate::sensors::{
    perception::{PerceptionSnapshot, SensorSnapshot},
    static_inputs::StaticInputs,
    typed_food::TypedFoodLocalSnapshot,
};

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

fn targets(ids: &[u32]) -> Vec<RouteTarget> {
    ids.iter()
        .enumerate()
        .map(|(i, &id)| RouteTarget {
            target_id: NodeId::new(id),
            slot: i as u8,
            gate_bias: -(i as f32),
        })
        .collect()
}

/// A VM node that loads `value` and votes it into `sink`, then halts.
fn voting_vm_node(id: u32, value: f32, sink: u8, target_ids: &[u32]) -> NodeGenome {
    NodeGenome {
        node_id: NodeId::new(id),
        input_refs: vec![],
        backend_def: BackendDef::Vm(VmBackendDef {
            register_count: 1,
            constants: vec![value],
            program: vec![
                VmInstruction::LoadConst {
                    dst: 0,
                    const_idx: 0,
                },
                VmInstruction::AddVote { sink, src: 0 },
                VmInstruction::Halt,
            ],
        }),
        targets: targets(target_ids),
    }
}

/// A graph node whose wired sinks are `sinks`, each fed by one constant
/// compute node of value `value`.
fn voting_graph_node(
    id: u32,
    value: f32,
    sinks: &[OutputSinkKind],
    target_ids: &[u32],
) -> NodeGenome {
    NodeGenome {
        node_id: NodeId::new(id),
        input_refs: vec![],
        backend_def: BackendDef::Graph(CgpGraphBackendDef {
            birth_weights: None,
            compute_nodes: vec![ComputeNode {
                kind: ComputeNodeKind::Constant(value),
                inputs: vec![],
                plasticity: None,
            }],
            output_sinks: sinks
                .iter()
                .map(|kind| OutputSink {
                    kind: *kind,
                    inputs: vec![GraphEdge {
                        source: GraphSource::ComputeNode(0),
                        weight: 1.0,
                    }],
                })
                .collect(),
            action_bank: vec![],
            execute_gate: ExecuteGate { inputs: vec![] },
        }),
        targets: targets(target_ids),
    }
}

fn run(genome: &CreatureGenome, energy: f32, cap: u32) -> MeshOutput {
    run_with(
        genome,
        energy,
        RuntimeConfig {
            max_mesh_hops: cap,
            ..RuntimeConfig::default()
        },
    )
}

/// Production opcode costs are around `1e-7`, so the exhaustion cases price
/// compute at its nominal base cost and drop both ramps to put the boundary
/// where the test can name it.
fn costly_config() -> RuntimeConfig {
    RuntimeConfig {
        max_mesh_hops: 4,
        hop_ramp_cost: 0.0,
        graph_node_base_cost: 0.5,
        vm: crate::config::VmRuntimeConfig {
            opcode_cost_multiplier: 1.0,
            step_ramp_cost: 0.0,
            ..crate::config::VmRuntimeConfig::default()
        },
        ..RuntimeConfig::default()
    }
}

fn run_with(genome: &CreatureGenome, energy: f32, config: RuntimeConfig) -> MeshOutput {
    let mut energy = energy;
    let mut memory = [0.0; 16];
    let mut state = GraphRuntimeState::new();
    state.begin_tick(&genome.nodes, 0);
    execute_creature_mesh_impl(
        genome,
        &sensors(),
        &mut energy,
        &mut memory,
        &[0.0; 16],
        &mut state,
        &config,
        crate::runtime::mesh::UntracedMeshExecution,
    )
}

#[test]
fn a_vm_dispatch_sums_its_own_votes_and_commits_them() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![1.5],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::AddVote {
                        sink: VoteSink::Move(2).index() as u8,
                        src: 0,
                    },
                    VmInstruction::AddVote {
                        sink: VoteSink::Move(2).index() as u8,
                        src: 0,
                    },
                    VmInstruction::Halt,
                ],
            }),
            targets: vec![],
        }],
    };
    let output = run(&genome, 100.0, 4);
    assert_eq!(output.votes[VoteSink::Move(2).index()], 3.0);
    assert_eq!(output.votes.iter().filter(|v| **v != 0.0).count(), 1);
    assert_eq!(output.commit_counts, [0; 4]);
}

#[test]
fn a_revisit_replaces_that_node_s_contribution_rather_than_adding_one() {
    // Node 0 votes 1.0 and routes to node 1, which routes back to node 0.
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            voting_vm_node(0, 1.0, VoteSink::Eat.index() as u8, &[1]),
            voting_vm_node(1, 4.0, VoteSink::Decide.index() as u8, &[0]),
        ],
    };
    let one_pass = run(&genome, 1000.0, 1);
    assert_eq!(one_pass.votes[VoteSink::Eat.index()], 1.0);

    // Three hops: node 0, node 1, node 0 again. The revisit replaces node 0's
    // contribution with the same value instead of doubling it.
    let revisited = run(&genome, 1000.0, 3);
    assert_eq!(revisited.work_counters.mesh_hops, 3);
    assert_eq!(revisited.votes[VoteSink::Eat.index()], 1.0);
    assert_eq!(revisited.votes[VoteSink::Decide.index()], 4.0);
}

#[test]
fn contributions_from_two_nodes_sum_with_their_signs() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            voting_vm_node(0, 2.5, VoteSink::Terminate.index() as u8, &[1]),
            voting_vm_node(1, -4.0, VoteSink::Terminate.index() as u8, &[]),
        ],
    };
    let output = run(&genome, 1000.0, 4);
    assert_eq!(output.votes[VoteSink::Terminate.index()], -1.5);
}

#[test]
fn a_non_finite_contribution_is_sanitized_at_commit_and_so_is_the_sum() {
    let mut side_outputs = crate::runtime::types::MeshSideOutputs::new(4);
    let mut staged = [0.0f32; VOTE_SINK_COUNT];
    staged[0] = f32::INFINITY;
    staged[1] = f32::NAN;
    staged[2] = f32::NEG_INFINITY;
    side_outputs.stage_vote_contribution(&staged);
    let committed = side_outputs.commit_vote_contribution(0);
    assert_eq!(committed[0], 1e9);
    assert_eq!(committed[1], 0.0);
    assert_eq!(committed[2], -1e9);
    assert_eq!(side_outputs.votes[0], 1e9);

    // Two nodes at the clamp sum past it, and the sum is sanitized again.
    let mut second = [0.0f32; VOTE_SINK_COUNT];
    second[0] = 1e9;
    side_outputs.stage_vote_contribution(&second);
    side_outputs.commit_vote_contribution(1);
    assert_eq!(side_outputs.votes[0], 1e9);
    assert!(side_outputs.votes.iter().all(|v| v.is_finite()));
}

#[test]
fn a_saturating_vm_vote_is_clamped_when_the_dispatch_commits() {
    // 1e30 * 1e30 overflows to +inf, which sanitizes to the clamp.
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 2,
                constants: vec![f32::MAX],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::Mul { dst: 1, a: 0, b: 0 },
                    VmInstruction::AddVote {
                        sink: VoteSink::Reproduce(0).index() as u8,
                        src: 1,
                    },
                    VmInstruction::Halt,
                ],
            }),
            targets: vec![],
        }],
    };
    let output = run(&genome, 100.0, 4);
    assert_eq!(output.votes[VoteSink::Reproduce(0).index()], 1e9);
    assert!(output.votes.iter().all(|v| v.is_finite()));
}

#[test]
fn an_invalid_add_vote_sink_writes_nothing_and_still_costs() {
    let program = |sink: u8| {
        vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            VmInstruction::AddVote { sink, src: 0 },
            VmInstruction::Halt,
        ]
    };
    let make = |sink: u8| CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![3.0],
                program: program(sink),
            }),
            targets: vec![],
        }],
    };
    let valid = run(&make(VOTE_SINK_COUNT as u8 - 1), 100.0, 4);
    let invalid = run(&make(VOTE_SINK_COUNT as u8), 100.0, 4);
    assert_eq!(invalid.votes, [0.0; VOTE_SINK_COUNT]);
    assert_eq!(invalid.work_counters.vm_steps, valid.work_counters.vm_steps);
    assert_eq!(invalid.cost_report.vm_cost, valid.cost_report.vm_cost);
}

#[test]
fn an_exhausted_vm_dispatch_commits_nothing() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            voting_vm_node(0, 7.0, VoteSink::Eat.index() as u8, &[1]),
            // Node 1 votes, then runs out of energy before it can halt.
            NodeGenome {
                node_id: NodeId::new(1),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![9.0],
                    program: vec![
                        VmInstruction::LoadConst {
                            dst: 0,
                            const_idx: 0,
                        },
                        VmInstruction::AddVote {
                            sink: VoteSink::Decide.index() as u8,
                            src: 0,
                        },
                        VmInstruction::Halt,
                    ],
                }),
                targets: vec![],
            },
        ],
    };
    // Node 0 spends 0.27; node 1 executes its vote and then exhausts on the
    // charge for its `Halt`, so it never reaches the commit boundary.
    let exhausted = run_with(&genome, 0.52, costly_config());
    assert_eq!(
        exhausted.termination_reason,
        crate::runtime::trace::domain::TerminationReason::EnergyExhausted
    );
    assert_eq!(exhausted.votes[VoteSink::Eat.index()], 7.0);
    assert_eq!(exhausted.votes[VoteSink::Decide.index()], 0.0);
}

#[test]
fn a_graph_visit_commits_its_wired_sinks_and_writes_the_parameter_surface() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![voting_graph_node(
            0,
            2.0,
            &[
                OutputSinkKind::ActionVote(VoteSink::StealEnergy(5)),
                OutputSinkKind::ActionParam(VoteKind::Eat, 1),
            ],
            &[],
        )],
    };
    let output = run(&genome, 100.0, 4);
    assert_eq!(output.votes[VoteSink::StealEnergy(5).index()], 2.0);
    // The parameter surface is not carried on `MeshOutput`: nothing reads it.
    // Its write is asserted through the effects pass below.
    assert_eq!(output.commit_counts, [0; 4]);
}

#[test]
fn the_parameter_surface_takes_the_last_visit_s_value_and_an_unwired_sink_leaves_it() {
    let mut side_outputs = crate::runtime::types::MeshSideOutputs::new(4);
    assert_eq!(side_outputs.action_params, [[0.0; 2]; 4]);

    let param_sink = OutputSinkKind::ActionParam(VoteKind::Move, 1);
    let mut visit = |value: f32, sinks: &[OutputSinkKind], node_idx: usize| {
        let BackendDef::Graph(def) = voting_graph_node(0, value, sinks, &[]).backend_def else {
            unreachable!("voting_graph_node builds a graph backend")
        };
        let mut energy = 100.0;
        let mut memory = [0.0; 16];
        let mut state = GraphRuntimeState::new();
        let _ = crate::runtime::cgp::execute_graph_node(
            &def,
            &[],
            &[0.0; crate::runtime::OUTPUT_SLOT_COUNT],
            &mut energy,
            0.0,
            node_idx,
            &mut state,
            &sensors(),
            &RuntimeConfig::default(),
            &mut side_outputs,
            &mut memory,
            &[0.0; 16],
        );
    };
    visit(3.0, &[param_sink], 0);
    visit(-1.0, &[param_sink], 1);
    // A visit with no parameter sink leaves the last written value standing.
    visit(9.0, &[OutputSinkKind::ActionVote(VoteSink::Eat)], 2);

    assert_eq!(
        side_outputs.action_params[VoteKind::Move.index()],
        [0.0, -1.0]
    );
    assert_eq!(
        side_outputs.action_params[VoteKind::Eat.index()],
        [0.0, 0.0]
    );
}

#[test]
fn an_exhausted_graph_visit_leaves_the_vote_vector_untouched() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            voting_vm_node(0, 6.0, VoteSink::Eat.index() as u8, &[1]),
            voting_graph_node(
                1,
                3.0,
                &[OutputSinkKind::ActionVote(VoteSink::Move(0))],
                &[],
            ),
        ],
    };
    let affordable = run(&genome, 100.0, 4);
    assert_eq!(affordable.votes[VoteSink::Move(0).index()], 3.0);

    // Same genome, only enough energy for the VM node: the graph visit is
    // entered, charged, and exhausted before its effects pass.
    let exhausted = run_with(&genome, 0.5, costly_config());
    assert_eq!(
        exhausted.termination_reason,
        crate::runtime::trace::domain::TerminationReason::EnergyExhausted
    );
    assert_eq!(exhausted.votes[VoteSink::Eat.index()], 6.0);
    assert_eq!(exhausted.votes[VoteSink::Move(0).index()], 0.0);
}

#[test]
fn the_three_execution_modes_agree_on_a_voting_genome() {
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![
            voting_vm_node(0, 1.25, VoteSink::Eat.index() as u8, &[1]),
            voting_graph_node(
                1,
                -0.5,
                &[
                    OutputSinkKind::ActionVote(VoteSink::Reproduce(3)),
                    OutputSinkKind::ActionParam(VoteKind::Reproduce, 0),
                ],
                &[0],
            ),
        ],
    };
    for cap in [1, 2, 3] {
        for start in [0.2, 5.0, 100.0] {
            let config = RuntimeConfig {
                max_mesh_hops: cap,
                ..RuntimeConfig::default()
            };
            let mut energies = [start; 3];
            let mut memories = [[0.0; 16]; 3];
            let mut states = [
                GraphRuntimeState::new(),
                GraphRuntimeState::new(),
                GraphRuntimeState::new(),
            ];
            for state in &mut states {
                state.begin_tick(&genome.nodes, 0);
            }
            let plain = execute_creature_mesh_impl(
                &genome,
                &sensors(),
                &mut energies[0],
                &mut memories[0],
                &[0.0; 16],
                &mut states[0],
                &config,
                crate::runtime::mesh::UntracedMeshExecution,
            );
            let (observed, _) = execute_creature_mesh_impl(
                &genome,
                &sensors(),
                &mut energies[1],
                &mut memories[1],
                &[0.0; 16],
                &mut states[1],
                &config,
                ObservedMeshExecution::default(),
            );
            let (traced, hops, _) = execute_creature_mesh_traced(
                &genome,
                &sensors(),
                &mut energies[2],
                &mut memories[2],
                &[0.0; 16],
                &mut states[2],
                &config,
            );
            for output in [&observed, &traced] {
                assert_eq!(plain.votes, output.votes, "cap {cap} start {start}");
                assert_eq!(plain.commit_counts, output.commit_counts);
                assert_eq!(plain.work_counters, output.work_counters);
                assert_eq!(plain.actions, output.actions);
            }
            assert_eq!(energies, [energies[0]; 3]);
            // The traced hops carry each hop's committed contribution, and the
            // last commit of every node sums to the evaluation's vote vector.
            let mut latest: Vec<(NodeId, [f32; VOTE_SINK_COUNT])> = Vec::new();
            for hop in &hops {
                match latest.iter_mut().find(|(id, _)| *id == hop.node_id) {
                    Some((_, entry)) => *entry = hop.vote_contribution,
                    None => latest.push((hop.node_id, hop.vote_contribution)),
                }
            }
            let mut summed = [0.0f32; VOTE_SINK_COUNT];
            for (_, entry) in &latest {
                for (sum, value) in summed.iter_mut().zip(entry) {
                    *sum += *value;
                }
            }
            assert_eq!(summed, traced.votes, "cap {cap} start {start}");
        }
    }
}

/// The sum rule as a property: for any sequence of stages and commits, the
/// mesh vote vector is the sanitized sum, in first-commit order, of every
/// node's latest sanitized contribution.
mod sum_rule_property {
    use super::VOTE_SINK_COUNT;
    use crate::creature::genome::vote::VoteVector;
    use crate::runtime::types::{sanitize_f32, MeshSideOutputs};
    use proptest::prelude::*;

    const NO_VOTES: VoteVector = [0.0; VOTE_SINK_COUNT];

    #[derive(Debug)]
    enum Step {
        Stage(VoteVector),
        Commit(usize),
    }

    fn step_strategy() -> impl Strategy<Value = Step> {
        prop_oneof![
            prop::array::uniform(any::<f32>()).prop_map(Step::Stage),
            (0usize..6).prop_map(Step::Commit),
        ]
    }

    /// The reference model: the same per-node-latest list, in the same
    /// first-commit order, so its summation order matches the implementation
    /// and the comparison is bit-exact rather than approximate.
    #[derive(Default)]
    struct Model(Vec<(usize, VoteVector)>);

    impl Model {
        fn record(&mut self, node_idx: usize, contribution: VoteVector) {
            match self.0.iter_mut().find(|(idx, _)| *idx == node_idx) {
                Some((_, entry)) => *entry = contribution,
                None => self.0.push((node_idx, contribution)),
            }
        }

        fn votes(&self) -> VoteVector {
            let mut votes = NO_VOTES;
            for (_, entry) in &self.0 {
                for (sum, value) in votes.iter_mut().zip(entry) {
                    *sum += *value;
                }
            }
            votes.map(sanitize_f32)
        }
    }

    proptest! {
        #[test]
        fn votes_are_the_sanitized_sum_of_each_node_s_latest_contribution(
            steps in prop::collection::vec(step_strategy(), 0..32),
        ) {
            let mut side_outputs = MeshSideOutputs::new(4);
            let mut model = Model::default();
            let mut staged: Option<VoteVector> = None;

            for step in steps {
                match step {
                    Step::Stage(contribution) => {
                        side_outputs.stage_vote_contribution(&contribution);
                        staged = Some(contribution.map(sanitize_f32));
                    }
                    Step::Commit(node_idx) => {
                        let expected = staged.take();
                        if let Some(contribution) = expected {
                            model.record(node_idx, contribution);
                        }
                        prop_assert_eq!(
                            side_outputs.commit_vote_contribution(node_idx),
                            expected.unwrap_or(NO_VOTES)
                        );
                    }
                }
                prop_assert_eq!(side_outputs.votes, model.votes());
                prop_assert!(side_outputs.votes.iter().all(|v| v.is_finite()));
            }
        }
    }
}
