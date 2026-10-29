//! T11.F29: `AddFeedbackRoute` destinations, born-losing margin, gate write,
//! atomic skips, founder eligibility, loop formation and the battery fixture.
use super::*;
use crate::config::{RuntimeConfig, SimulationConfig};
use crate::creature::founder::v3alpha1_founder_genome;
use crate::genome::analysis::mesh_cycle_nodes;
use crate::mutation::reachability::TargetSets;
use crate::neighborhood::Battery;
use crate::runtime::routing::{resolve_gated_route, RouteGateMap};
use rand::Rng;
use routing::{feedback_destinations, losing_bias};

fn ids(values: &[u32]) -> Vec<NodeId> {
    values.iter().copied().map(NodeId::new).collect()
}

/// Apply `AddFeedbackRoute` with `executed` as the parent's executed set.
fn feedback(
    g: &mut CreatureGenome,
    executed: &[usize],
    rng: &mut SmallRng,
) -> Result<TargetReachability, MutationSkipReason> {
    let reachable = mesh_reachable_nodes(g);
    TopologyMutator::apply(
        g,
        TopologyOperator::AddFeedbackRoute,
        &mut TargetSets::new(&reachable, executed).selector(0.0, 0.0),
        rng,
        &MutationConfig::default(),
    )
}

fn feedback_seeded(
    g: &mut CreatureGenome,
    executed: &[usize],
    seed: u64,
) -> Result<TargetReachability, MutationSkipReason> {
    feedback(g, executed, &mut SmallRng::seed_from_u64(seed))
}

fn chain() -> CreatureGenome {
    genome(vec![
        node(0, &[1]),
        node(1, &[2]),
        node(2, &[3]),
        node(3, &[]),
    ])
}

#[test]
fn destinations_without_an_executed_record_are_the_source_alone() {
    let g = chain();
    for source in 0..4 {
        assert_eq!(
            feedback_destinations(&g, &[], source),
            vec![g.nodes[source].node_id]
        );
    }
}

#[test]
fn destinations_are_executed_predecessors_within_two_hops_and_the_source() {
    let g = chain();
    // Node 0 is three hops upstream of node 3 and is excluded.
    assert_eq!(feedback_destinations(&g, &[0, 1, 2, 3], 3), ids(&[1, 2, 3]));
    // An unexecuted predecessor neither qualifies nor bridges to P2.
    assert_eq!(feedback_destinations(&g, &[0, 2, 3], 3), ids(&[2, 3]));
    assert_eq!(feedback_destinations(&g, &[0, 1, 3], 3), ids(&[3]));
    // A source outside the executed set self-loops only, even when executed
    // nodes target it.
    assert_eq!(feedback_destinations(&g, &[0, 1, 2], 3), ids(&[3]));
}

#[test]
fn destinations_exclude_current_targets_and_admit_a_source_on_a_cycle() {
    // 0 -> 1 -> 2 <-> 3: node 2 is both predecessor and successor of node 3.
    let g = genome(vec![
        node(0, &[1]),
        node(1, &[2]),
        node(2, &[3]),
        node(3, &[2]),
    ]);
    assert_eq!(feedback_destinations(&g, &[0, 1, 2, 3], 3), ids(&[1, 3]));
    // Node 2's predecessors are 1 and 3, and 3 is its current target.
    assert_eq!(feedback_destinations(&g, &[0, 1, 2, 3], 2), ids(&[0, 1, 2]));
    // A self-targeting source is not its own candidate.
    let looped = genome(vec![node(0, &[0])]);
    assert!(feedback_destinations(&looped, &[0], 0).is_empty());
}

#[test]
fn destinations_follow_node_index_order_with_sparse_ids_and_duplicate_edges() {
    // Index order 0..4 carries ids 7, 3, 42, 5; node 42 names node 5 twice.
    let g = genome(vec![
        node(7, &[3]),
        node(3, &[42]),
        node(42, &[5, 5]),
        node(5, &[]),
    ]);
    assert_eq!(
        feedback_destinations(&g, &[0, 1, 2, 3], 3),
        ids(&[3, 42, 5])
    );
}

fn small_genome() -> impl Strategy<Value = CreatureGenome> {
    (1usize..=6).prop_flat_map(|n| {
        proptest::collection::vec(proptest::collection::vec(0u32..=(n as u32), 0..3), n).prop_map(
            move |targets| {
                // Ids are sparse (index * 3 + 1) and id `n * 3 + 1` is missing.
                let id = |i: u32| i * 3 + 1;
                genome(
                    targets
                        .iter()
                        .enumerate()
                        .map(|(i, t)| {
                            let named: Vec<u32> = t.iter().map(|&j| id(j)).collect();
                            node(id(i as u32), &named)
                        })
                        .collect(),
                )
            },
        )
    })
}

proptest! {
    #[test]
    fn destinations_match_the_two_hop_upstream_definition(
        g in small_genome(),
        executed_mask in 0u8..64,
        source_pick in 0usize..6,
    ) {
        let n = g.nodes.len();
        let source = source_pick % n;
        let executed: Vec<usize> = (0..n).filter(|i| executed_mask & (1 << i) != 0).collect();
        let found = feedback_destinations(&g, &executed, source);
        let source_id = g.nodes[source].node_id;
        let targets_of = |i: usize| g.nodes[i].targets.iter().map(|t| t.target_id).collect::<Vec<_>>();
        let current = targets_of(source);
        let in_e = |i: usize| executed.contains(&i);
        let p1: Vec<usize> = (0..n)
            .filter(|&i| in_e(source) && in_e(i) && targets_of(i).contains(&source_id))
            .collect();
        let p2: Vec<usize> = (0..n)
            .filter(|&i| in_e(i) && p1.iter().any(|&j| targets_of(i).contains(&g.nodes[j].node_id)))
            .collect();
        let mut expected: Vec<usize> = std::iter::once(source).chain(p1).chain(p2).collect();
        expected.sort_unstable();
        expected.dedup();
        let expected: Vec<NodeId> = expected
            .into_iter()
            .map(|i| g.nodes[i].node_id)
            .filter(|id| !current.contains(id))
            .collect();
        prop_assert_eq!(&found, &expected);
        for id in &found {
            prop_assert!(g.nodes.iter().any(|node| node.node_id == *id));
            prop_assert!(!current.contains(id));
        }
        if !in_e(source) {
            prop_assert!(found.iter().all(|id| *id == source_id));
        }
    }

    #[test]
    fn a_unit_range_gate_never_selects_the_branch_and_a_larger_one_can(
        incumbent in prop_oneof![Just(0.1f32), -1.0e3f32..1.0e3f32],
        gate_value in -1.0f32..=1.0f32,
    ) {
        let bias = losing_bias(incumbent);
        prop_assert!(bias + 1.0 <= incumbent);
        let targets = [
            RouteTarget { target_id: NodeId::new(1), slot: 0, gate_bias: incumbent },
            RouteTarget { target_id: NodeId::new(0), slot: 1, gate_bias: bias },
        ];
        let mut gates = RouteGateMap::default();
        gates.scores[1] = gate_value;
        prop_assert_eq!(resolve_gated_route(&targets, &gates).map(|(i, _)| i), Some(0));
        gates.scores[1] = 2.0;
        prop_assert_eq!(resolve_gated_route(&targets, &gates).map(|(i, _)| i), Some(1));
    }
}

#[test]
fn the_margin_steps_below_a_rounded_difference() {
    // `0.1 - 1.0 + 1.0` rounds above `0.1` in f32, so the ulp step is needed.
    let incumbent = std::hint::black_box(0.1f32);
    assert!(incumbent - 1.0 + 1.0 > incumbent);
    let bias = losing_bias(incumbent);
    assert!(bias < incumbent - 1.0);
    assert!(bias + 1.0 <= incumbent);
    assert_eq!(losing_bias(0.0), -1.0);
}

#[test]
fn the_margin_starts_one_unit_below_where_f32_spacing_reaches_two() {
    // At 2^24 the spacing above is 2 and below is 1: `incumbent - 1.0` is
    // exact, while a start at or above the incumbent rounds back onto it.
    let incumbent = 16_777_216.0f32;
    assert_eq!(losing_bias(incumbent), 16_777_215.0);
    assert_eq!(losing_bias(-incumbent), -16_777_218.0);
}

#[test]
fn a_zero_register_vm_node_is_never_the_selected_source() {
    let mut base = genome(vec![node(0, &[1]), node(1, &[2]), node(2, &[])]);
    if let BackendDef::Vm(vm) = base.nodes[0].backend_edit().1 {
        vm.register_count = 0;
    }
    for seed in 0..32 {
        let mut g = base.clone();
        assert!(feedback_seeded(&mut g, &[], seed).is_ok(), "seed {seed}");
        assert_eq!(g.nodes[0], base.nodes[0], "seed {seed}");
        assert_eq!(g.nodes[1].targets.len(), 2, "seed {seed}");
    }
}

#[test]
fn the_event_appends_a_losing_branch_with_its_own_gate_write_on_both_backends() {
    for graph in [false, true] {
        let base = conditional_fixture(graph);
        let mut g = base.clone();
        feedback_seeded(&mut g, &[], 7).unwrap();
        let source = &g.nodes[0];
        assert_eq!(source.targets.len(), 2, "graph={graph}");
        assert_eq!(source.targets[0], base.nodes[0].targets[0]);
        let branch = source.targets[1];
        assert_eq!(branch.target_id, NodeId::new(0));
        assert_eq!(branch.slot, 1);
        assert_eq!(
            branch.gate_bias,
            losing_bias(base.nodes[0].targets[0].gate_bias)
        );
        assert_eq!(g.nodes[1], base.nodes[1]);
        assert_eq!(g.nodes.len(), base.nodes.len());
        match source.backend_def() {
            BackendDef::Vm(vm) => {
                let BackendDef::Vm(old) = base.nodes[0].backend_def() else {
                    unreachable!()
                };
                assert_eq!(vm.program.len(), old.program.len() + 1);
                let halt = vm
                    .program
                    .iter()
                    .position(|i| matches!(i, VmInstruction::Halt))
                    .unwrap();
                assert!(matches!(
                    vm.program[halt - 1],
                    VmInstruction::WriteRouteGate { slot: 1, src: 0 }
                ));
            }
            BackendDef::Graph(backend) => {
                let sink = backend
                    .output_sinks()
                    .iter()
                    .find(|s| matches!(s.kind, OutputSinkKind::RouterGate(1)))
                    .unwrap();
                assert_eq!(sink.inputs.len(), 1);
                assert_eq!(sink.inputs[0].own_weight(), 1.0);
            }
        }
        // T25.F02: +2 on both backends (Graph was +3, the newly wired sink's
        // unit); the old size rule restores +3.
        assert_eq!(
            g.genome_size(),
            base.genome_size() + 2,
            "one target unit plus one instruction, or one target unit plus one edge (T25.F02: a wired sink costs no unit)"
        );
    }
}

#[test]
fn the_slot_skips_an_orphan_gate_write() {
    let mut g = genome(vec![node(0, &[1]), node(1, &[])]);
    gate(&mut g.nodes[0], 1);
    feedback_seeded(&mut g, &[], 3).unwrap();
    assert_eq!(g.nodes[0].targets[1].slot, 2);
    let BackendDef::Vm(vm) = g.nodes[0].backend_def() else {
        unreachable!()
    };
    assert!(vm
        .program
        .iter()
        .any(|i| matches!(i, VmInstruction::WriteRouteGate { slot: 2, .. })));
}

#[test]
fn ineligible_sources_skip_atomically() {
    let base = || genome(vec![node(0, &[1]), node(1, &[])]);
    let mut cases: Vec<(&str, CreatureGenome)> = Vec::new();
    let mut zero_registers = base();
    if let BackendDef::Vm(vm) = zero_registers.nodes[0].backend_edit().1 {
        vm.register_count = 0;
    }
    cases.push(("zero-register VM", zero_registers));
    let mut full = base();
    full.nodes[0].targets = (0..MAX_GATE_SLOTS as u8)
        .map(|slot| RouteTarget {
            target_id: NodeId::new(1),
            slot,
            gate_bias: 0.0,
        })
        .collect();
    cases.push(("no free slot", full));
    let mut no_candidates = base();
    no_candidates.nodes[0].targets.push(RouteTarget {
        target_id: NodeId::new(0),
        slot: 1,
        gate_bias: -1.0,
    });
    cases.push(("no candidates", no_candidates));
    for bias in [f32::NAN, f32::INFINITY] {
        let mut non_finite = base();
        non_finite.nodes[0].targets[0].gate_bias = bias;
        cases.push(("non-finite incumbent bias", non_finite));
    }
    let mut written = base();
    gate(&mut written.nodes[0], 0);
    cases.push(("written incumbent slot", written));
    let mut dangling = base();
    dangling.nodes[0].targets[0].target_id = NodeId::new(99);
    cases.push(("incumbent names a missing node", dangling));
    for (label, case) in cases {
        let mut g = case.clone();
        assert_eq!(
            feedback_seeded(&mut g, &[0, 1], 7),
            Err(MutationSkipReason::NoApplicableTarget),
            "{label}"
        );
        // Debug text compares NaN biases, which `==` cannot.
        assert_eq!(format!("{g:?}"), format!("{case:?}"), "{label}");
    }
}

#[test]
fn the_founder_source_is_node_zero_with_itself_as_the_only_destination() {
    let founder = v3alpha1_founder_genome();
    for executed in [&[][..], &[0, 1][..]] {
        assert_eq!(feedback_destinations(&founder, executed, 0), ids(&[0]));
        for seed in 0..16 {
            let mut g = founder.clone();
            feedback_seeded(&mut g, executed, seed).unwrap();
            assert_eq!(g.nodes[1], founder.nodes[1], "node 1 is never a source");
            assert_eq!(g.nodes[0].targets[0], founder.nodes[0].targets[0]);
            assert_eq!(g.nodes[0].targets[1].target_id, NodeId::new(0));
        }
    }
}

#[test]
fn one_event_closes_a_loop_and_replays_identically() {
    let founder = v3alpha1_founder_genome();
    assert!(mesh_cycle_nodes(&founder).is_empty());
    let mut first = founder.clone();
    let mut second = founder.clone();
    let mut rng_a = SmallRng::seed_from_u64(11);
    let mut rng_b = SmallRng::seed_from_u64(11);
    feedback(&mut first, &[0, 1], &mut rng_a).unwrap();
    feedback(&mut second, &[0, 1], &mut rng_b).unwrap();
    assert_eq!(first, second);
    assert_eq!(rng_a.gen::<u64>(), rng_b.gen::<u64>());
    assert_eq!(
        mesh_cycle_nodes(&first).into_iter().collect::<Vec<_>>(),
        ids(&[0])
    );
}

/// The battery fixture: one production event on T11.F15's conditional fixture
/// is silent and cycle-carrying without a revisit; production `MutateGateBias`
/// events then activate the loop. Prints the event count for the readings.
#[test]
fn a_feedback_branch_is_silent_at_birth_and_ordinary_bias_steps_activate_it() {
    let battery = Battery::generate(1);
    let runtime = RuntimeConfig::default();
    let decay = SimulationConfig::default().shared_memory.decay_rate;
    for graph in [false, true] {
        let base = conditional_fixture(graph);
        // The Graph gate-source draw can land on shared memory, which can
        // exceed 1.0; seed 7 draws a source in the unit range.
        let mut grown = base.clone();
        feedback_seeded(&mut grown, &[0, 1], 7).unwrap();
        if let BackendDef::Graph(backend) = grown.nodes[0].backend_def() {
            let sink = backend
                .output_sinks()
                .iter()
                .find(|s| matches!(s.kind, OutputSinkKind::RouterGate(1)))
                .unwrap();
            assert!(
                !matches!(sink.inputs[0].source(), GraphSource::SharedMemory { .. }),
                "seed 7 graph gate {:?}",
                sink.inputs
            );
        }
        assert_eq!(
            battery.signature(&grown, &runtime, decay),
            battery.signature(&base, &runtime, decay),
            "graph={graph}"
        );
        let reading = battery.mesh_execution(&grown, &runtime, decay);
        assert!(
            reading.cycle_carrying && !reading.revisiting,
            "graph={graph}"
        );

        let mut activated = grown.clone();
        let mut rng = SmallRng::seed_from_u64(29);
        let events = (1..=64)
            .find(|_| {
                TopologyMutator::apply(
                    &mut activated,
                    TopologyOperator::MutateGateBias,
                    &mut TargetSelector::reachable_only(&[0], 1.0),
                    &mut rng,
                    &MutationConfig::default(),
                )
                .unwrap();
                battery
                    .mesh_execution(&activated, &runtime, decay)
                    .revisiting
            })
            .expect("bounded MutateGateBias walk activates the loop");
        assert_eq!(activated.nodes[0].targets[1].target_id, NodeId::new(0));
        println!("T11.F29 activation graph={graph}: {events} MutateGateBias events");
    }
}
