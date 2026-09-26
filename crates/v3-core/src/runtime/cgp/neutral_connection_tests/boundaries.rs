//! Address, destination and native execution boundaries of the qualified contract.
use super::*;

struct Visit {
    result: NodeResult,
    state: GraphRuntimeState,
    side: MeshSideOutputs,
    memory: [f32; 16],
    energy: f32,
}

fn visit(def: &CgpGraphBackendDef, mut energy: f32, mut state: GraphRuntimeState) -> Visit {
    let mut memory = ramp(0.5);
    let mut side = side();
    let result = execute_graph_node(
        def,
        &[],
        &ramp(0.25),
        &mut energy,
        0.0,
        0,
        &mut state,
        &sensors(),
        &RuntimeConfig {
            graph_node_base_cost: BASE_COST,
            ..RuntimeConfig::default()
        },
        &mut side,
        &mut memory,
        &ramp(0.0),
    );
    Visit {
        result,
        state,
        side,
        memory,
        energy,
    }
}

#[test]
fn resolver_boundaries_preserve_wrap_scalar_raw_queue_and_soft_zero_semantics() {
    let sensors = sensors();
    let upstream = ramp(0.25);
    let side = side();
    let ctx = ResolveCtx {
        sensors: &sensors,
        upstream_slots: &upstream,
        energy: 100.0,
        energy_consumed: 25.0,
        action_queue: &side.action_queue,
        votes: &side.votes,
        previous_pass_votes: &side.previous_pass_votes,
        commit_counts: &side.commit_counts,
        mesh_hops: 6,
    };
    let resolve = |refs: &[InputReference], source| {
        resolve_source_post_convergence(&source, 0, &[], refs, &ctx, &ramp(0.0), &ramp(0.5))
    };
    for (reference, values) in reference_cases() {
        let width = values.len();
        let expected = match &reference {
            InputReference::World(key) if key.compound_width() > 1 => {
                values[usize::from(u16::MAX) % width]
            }
            InputReference::ActionQueue
            | InputReference::ActionVotes
            | InputReference::PreviousPassVotes
            | InputReference::CommitCounts
            | InputReference::PreviousOutcome => 0.0,
            _ => values[0],
        };
        assert_eq!(
            resolve(std::slice::from_ref(&reference), leaf(0, u16::MAX)),
            expected,
            "{reference:?}"
        );
        if matches!(
            reference,
            InputReference::ActionVotes
                | InputReference::PreviousPassVotes
                | InputReference::CommitCounts
                | InputReference::PreviousOutcome
                | InputReference::ActionQueue
        ) {
            assert_eq!(resolve(&[reference], leaf(0, width as u16)), 0.0);
        }
    }
    assert_eq!(
        resolve(&[InputReference::ActionQueue], leaf(0, 14)),
        0.75,
        "beyond draw width 12 remains readable"
    );
    assert_eq!(resolve(&[], leaf(0, 0)), 0.0);
    assert_eq!(
        resolve(&[InputReference::UpstreamSlot(24)], leaf(0, 0)),
        0.0
    );
    assert_eq!(
        resolve(
            &[InputReference::World(WorldInputKey::food_here(
                OrdinaryFoodTypeId::new(2)
            ))],
            leaf(0, 0)
        ),
        0.0
    );
    for previous in [false, true] {
        assert_eq!(resolve(&[], memory(16, previous)), 0.0);
    }
    let mut refs = vec![InputReference::UpstreamSlot(0); usize::from(u16::MAX) + 1];
    refs[usize::from(u16::MAX)] = InputReference::UpstreamSlot(23);
    assert_eq!(resolve(&refs, leaf(u16::MAX, u16::MAX)), upstream[23]);
    assert!(
        u16::try_from(refs.len()).is_err(),
        "a next index cannot be represented; no truncating admission"
    );
    let empty = CgpGraphBackendDef {
        compute_nodes: vec![],
        output_sinks: vec![],
        birth_weights: None,
    };
    assert!(empty
        .sink(OutputSinkKind::ActionVote(VoteSink::Eat))
        .is_none());
}

#[test]
fn dormant_zero_edge_charges_one_visit_and_exhaustion_commits_no_effects() {
    for initial_energy in [10.0, BASE_COST] {
        for active in [false, true] {
            let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
            if active {
                wire(
                    &mut def,
                    OutputSinkKind::ActionVote(VoteSink::Eat),
                    edge(memory(0, false), 0.0),
                );
            }
            let mut energy = initial_energy;
            let mut state = GraphRuntimeState::new();
            let mut memory = ramp(0.5);
            let mut side = side();
            // Distinguish no stage on exhaustion/skipping from a committed zero.
            side.stage_vote_contribution(&[0.25; VOTE_SINK_COUNT]);
            let result = execute_graph_node(
                &def,
                &[],
                &ramp(0.25),
                &mut energy,
                0.0,
                0,
                &mut state,
                &sensors(),
                &RuntimeConfig {
                    graph_node_base_cost: BASE_COST,
                    ..RuntimeConfig::default()
                },
                &mut side,
                &mut memory,
                &ramp(0.0),
            );
            assert_eq!(
                energy,
                initial_energy - if active { BASE_COST } else { 0.0 }
            );
            assert_eq!(side.work_counters.graph_relax_iters, u32::from(active));
            assert_eq!(
                side.energy_observation.graph_compute,
                if active { f64::from(BASE_COST) } else { 0.0 }
            );
            assert_eq!(
                result.energy_exhausted,
                active && initial_energy == BASE_COST
            );
            assert_eq!(memory, ramp(0.5));
            assert_eq!(
                side.dispatch_effects().0,
                if active && initial_energy > BASE_COST {
                    [0.0; VOTE_SINK_COUNT]
                } else {
                    [0.25; VOTE_SINK_COUNT]
                }
            );
            assert!(state.node_state.iter().all(Vec::is_empty));
            assert!(state.node_outputs.iter().all(Vec::is_empty));
        }
    }
}

#[test]
fn zero_edges_can_clear_overwrite_and_change_nonadditive_compute_outputs() {
    for kind in [
        OutputSinkKind::ClearSlot(3),
        OutputSinkKind::WriteSlot(3),
        OutputSinkKind::CustomOutput(3),
        OutputSinkKind::ActionParam(ActionParamField::StealEnergyAmount),
    ] {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        let before = effects(&def, &[], 0.75);
        wire(&mut def, kind, edge(memory(15, false), 0.0));
        let after = effects(&def, &[], 0.75);
        match kind {
            OutputSinkKind::ClearSlot(_) | OutputSinkKind::WriteSlot(_) => {
                assert_ne!(before.memory[3], after.memory[3]);
                assert_eq!(after.memory[3], 0.0);
            }
            OutputSinkKind::CustomOutput(_) => {
                assert_ne!(before.result.output_slots[3], after.result.output_slots[3]);
                assert_eq!(after.result.output_slots[3], 0.0);
            }
            OutputSinkKind::ActionParam(field) => {
                assert_ne!(
                    before.side.action_params[field.index()],
                    after.side.action_params[field.index()]
                );
                assert_eq!(after.side.action_params[field.index()], 0.0);
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn native_nonadditive_compute_inputs_are_not_qualified_zero_destinations() {
    for kind in [ComputeNodeKind::Multiply, ComputeNodeKind::Min] {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        def.compute_nodes.push(ComputeNode {
            kind,
            inputs: vec![edge(memory(0, false), 1.0)],
            plasticity: None,
        });
        wire(
            &mut def,
            OutputSinkKind::CustomOutput(0),
            edge(GraphSource::ComputeNode(0), 1.0),
        );
        let before = visit(&def, 10.0, GraphRuntimeState::new());
        def.compute_nodes[0]
            .inputs
            .push(edge(memory(0, false), 0.0));
        let after = visit(&def, 10.0, GraphRuntimeState::new());
        assert_eq!(before.result.output_slots[0], ramp::<16>(0.5)[0]);
        assert_eq!(after.result.output_slots[0], 0.0);
    }
}

#[test]
fn nonfinite_zero_source_can_poison_an_incumbent_vote_before_sanitization() {
    for source in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let kind = OutputSinkKind::ActionVote(VoteSink::Eat);
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        wire(&mut def, kind, edge(memory(0, true), 1.0));
        assert_ne!(effects(&def, &[], source).side.dispatch_effects().0[0], 0.0);
        wire(&mut def, kind, edge(memory(15, false), 0.0));
        assert_eq!(effects(&def, &[], source).side.dispatch_effects().0[0], 0.0);
    }
}

#[test]
fn unaffordable_active_graph_keeps_incumbent_carried_state_and_commits_no_effects() {
    let mut def = active_graph();
    let mut initial = GraphRuntimeState::new();
    initial.node_state = vec![vec![0.25, 0.75]];
    initial.node_outputs = vec![vec![0.5, 0.125]];
    initial.plasticity_weights = vec![vec![
        vec![0.375].into_boxed_slice(),
        vec![0.625].into_boxed_slice(),
    ]];
    initial.eligibility_traces = vec![vec![
        vec![].into_boxed_slice(),
        vec![0.75].into_boxed_slice(),
    ]];
    let before = visit(&def, 2.0 * BASE_COST, initial.clone());
    wire(
        &mut def,
        OutputSinkKind::ActionVote(VoteSink::Eat),
        edge(memory(0, false), 0.0),
    );
    let after = visit(&def, 2.0 * BASE_COST, initial.clone());
    for run in [before, after] {
        assert!(run.result.energy_exhausted);
        assert_eq!(run.energy, 0.0);
        assert_eq!(
            run.side.energy_observation.graph_compute,
            f64::from(2.0 * BASE_COST)
        );
        assert_eq!(run.side.work_counters.graph_relax_iters, 1);
        assert_eq!(run.side.work_counters.plasticity_updates, 0);
        assert_eq!(
            run.side.dispatch_effects(),
            ([0.0; VOTE_SINK_COUNT], side().action_params)
        );
        assert_eq!(
            run.side.action_queue.into_actions(),
            side().action_queue.into_actions()
        );
        assert_eq!(run.memory, ramp(0.5));
        assert_eq!(run.state.node_state, initial.node_state);
        assert_eq!(run.state.node_outputs, initial.node_outputs);
        assert_eq!(run.state.plasticity_weights, initial.plasticity_weights);
        assert_eq!(run.state.eligibility_traces, initial.eligibility_traces);
    }
}

#[test]
fn dormant_activation_changes_a_later_live_energy_read() {
    let mut entry = graph_node(CgpGraphBackendDef::new_with_fixed_outputs(), vec![]);
    entry.targets.push(RouteTarget {
        target_id: NodeId::new(1),
        slot: 0,
        gate_bias: 1.0,
    });
    let mut reader = graph_node(
        CgpGraphBackendDef::new_with_fixed_outputs(),
        vec![InputReference::DynamicIntrospection(
            DynamicIntrospectionKey::EnergyCurrent,
        )],
    );
    reader.node_id = NodeId::new(1);
    wire(
        graph_mut(&mut reader),
        OutputSinkKind::WriteSlot(0),
        edge(leaf(0, 0), 1.0),
    );
    let base = CreatureGenome {
        entry_node_id: entry.node_id,
        nodes: vec![entry, reader],
    };
    let mut silent = base.clone();
    wire(
        graph_mut(&mut silent.nodes[0]),
        OutputSinkKind::ActionVote(VoteSink::Eat),
        edge(memory(0, false), 0.0),
    );
    let mut results = Vec::new();
    for genome in [base, silent] {
        let mut energy = 10.0;
        let mut memory = ramp(0.5);
        let mut state = GraphRuntimeState::new();
        state.begin_tick(&genome.nodes, 0);
        let result = execute_creature_mesh(
            &genome,
            &sensors(),
            &mut energy,
            &mut memory,
            &ramp(0.0),
            &mut state,
            &RuntimeConfig {
                graph_node_base_cost: BASE_COST,
                max_actions_per_turn: 1,
                ..RuntimeConfig::default()
            },
        );
        assert_eq!(memory[0], energy / 200.0);
        results.push((energy, memory[0], result));
    }
    assert_eq!(results[0].0 - results[1].0, BASE_COST);
    assert_ne!(
        results[0].1, results[1].1,
        "zero vote signal does not imply physiological neutrality"
    );
    assert_eq!(
        results[1].2.work_counters.graph_relax_iters,
        results[0].2.work_counters.graph_relax_iters + 1
    );
}

#[test]
fn already_wired_zero_compute_graph_keeps_its_native_charge_and_incumbent_effects() {
    for vote_already_wired in [false, true] {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        wire(
            &mut def,
            OutputSinkKind::WriteSlot(3),
            edge(memory(0, false), 0.5),
        );
        let kind = OutputSinkKind::ActionVote(VoteSink::Eat);
        if vote_already_wired {
            wire(&mut def, kind, edge(memory(0, false), 0.25));
        }
        let before = visit(&def, 10.0, GraphRuntimeState::new());
        wire(&mut def, kind, edge(memory(1, false), 0.0));
        let after = visit(&def, 10.0, GraphRuntimeState::new());
        assert_eq!(before.energy, 10.0 - BASE_COST);
        assert_eq!(after.energy, before.energy);
        assert_eq!(before.result, after.result);
        assert_eq!(before.memory, after.memory);
        assert_ne!(before.memory[3], ramp::<16>(0.5)[3]);
        assert_eq!(
            before.side.dispatch_effects(),
            after.side.dispatch_effects()
        );
        assert_eq!(before.side.work_counters, after.side.work_counters);
        assert_eq!(
            before.side.energy_observation,
            after.side.energy_observation
        );
    }
}
