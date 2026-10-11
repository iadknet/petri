//! T11.F31: ring projections through the native effects pass. Equality is
//! exact numeric f32 equality (signed zero allowed), as for zero vote edges.
use super::*;
use crate::creature::genome::cgp::RingProjection;
use crate::creature::genome::vote::DirectedVoteKind;
use crate::mutation::graph::projection;
use crate::runtime::cgp::execute_graph_node_traced;
use crate::runtime::types::sanitize_f32;
use rand::{rngs::SmallRng, SeedableRng};

fn food_ring() -> InputReference {
    InputReference::World(WorldInputKey::neighbor_food_ring(OrdinaryFoodTypeId::new(
        0,
    )))
}

fn rings() -> Vec<InputReference> {
    vec![
        food_ring(),
        InputReference::World(WorldInputKey::NeighborBarrierRing),
        InputReference::World(WorldInputKey::NeighborOccupiedRing),
        InputReference::World(WorldInputKey::NearbyCreatureVitals),
    ]
}

/// `sensors()` with the three rings replaced.
fn ring_sensors(food: [f32; 8], barrier: [f32; 8], occupied: [f32; 8]) -> SensorSnapshot {
    let mut sensors = sensors();
    sensors.local.neighbor_food = food;
    sensors.typed_local_food.neighbor_food_by_type[0] = food;
    sensors.local.neighbor_barrier = barrier;
    sensors.local.neighbor_occupied = occupied;
    sensors
}

fn effects_at(
    def: &CgpGraphBackendDef,
    refs: &[InputReference],
    sensors: &SensorSnapshot,
) -> Effects {
    let upstream = ramp(0.25);
    let input_side = side();
    let ctx = ResolveCtx {
        sensors,
        upstream_slots: &upstream,
        energy: 100.0,
        energy_consumed: 25.0,
        action_queue: &input_side.action_queue,
        votes: &input_side.votes,
        previous_pass_votes: &input_side.previous_pass_votes,
        commit_counts: &input_side.commit_counts,
        mesh_hops: 6,
    };
    let mut memory = ramp(0.0);
    let mut side = side();
    let (result, _) = apply_cgp_graph_effects(
        def,
        &[0.5; 3],
        refs,
        &ctx,
        &upstream,
        &mut side,
        &mut memory,
        &ramp(0.5),
    );
    Effects {
        result,
        side,
        memory,
    }
}

fn votes(def: &CgpGraphBackendDef, sensors: &SensorSnapshot) -> [f32; VOTE_SINK_COUNT] {
    effects_at(def, &rings(), sensors).side.dispatch_effects().0
}

fn kind_strategy() -> impl Strategy<Value = DirectedVoteKind> {
    (0usize..3).prop_map(|i| DirectedVoteKind::ALL[i])
}

fn ring_strategy() -> impl Strategy<Value = [f32; 8]> {
    prop::array::uniform8(-10.0f32..10.0)
}

/// The eight edges a projection `(w, k)` of `kind` over `ref_idx` is
/// equivalent to.
fn eight_edges(def: &mut CgpGraphBackendDef, p: RingProjection) {
    for d in 0..8 {
        wire(
            def,
            OutputSinkKind::ActionVote(p.kind.sink(d)),
            edge(leaf(p.ref_idx, u16::from(p.channel_for(d))), p.weight),
        );
    }
}

proptest! {
    #[test]
    fn zero_weight_projection_is_exactly_neutral(
        food in ring_strategy(),
        barrier in ring_strategy(),
        occupied in ring_strategy(),
        kind in kind_strategy(),
        offset in 0u8..16,
        ref_idx in 0u16..5,
        negative_zero in any::<bool>(),
        incumbents in prop::collection::vec((0usize..27, -4.0f32..4.0), 0..6),
    ) {
        let sensors = ring_sensors(food, barrier, occupied);
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        for (sink, weight) in incumbents {
            let vote = VoteSink::from_index(sink).unwrap();
            wire(&mut def, OutputSinkKind::ActionVote(vote), edge(memory(0, true), weight));
        }
        let before = effects_at(&def, &rings(), &sensors);
        let weight = if negative_zero { -0.0 } else { 0.0 };
        def.projections.push(RingProjection::new(ref_idx, kind, offset, weight));
        let after = effects_at(&def, &rings(), &sensors);
        prop_assert_eq!(after.side.dispatch_effects(), before.side.dispatch_effects());
        prop_assert_eq!(after.result, before.result);
        prop_assert_eq!(after.memory, before.memory);
        prop_assert_eq!(after.side.work_counters, before.side.work_counters);
    }

    #[test]
    fn a_projection_equals_its_eight_edges_on_a_kind_without_edges(
        food in ring_strategy(),
        barrier in ring_strategy(),
        kind in kind_strategy(),
        offset in 0u8..8,
        ref_idx in 0u16..3,
        weight in -4.0f32..4.0,
        others in prop::collection::vec((0usize..27, -4.0f32..4.0), 0..4),
    ) {
        let sensors = ring_sensors(food, barrier, ramp(0.25));
        let mut base = CgpGraphBackendDef::new_with_fixed_outputs();
        for (sink, w) in others {
            let vote = VoteSink::from_index(sink).unwrap();
            if DirectedVoteKind::of_sink(vote).is_some_and(|(k, _)| k == kind) {
                continue;
            }
            wire(&mut base, OutputSinkKind::ActionVote(vote), edge(memory(0, true), w));
        }
        let p = RingProjection::new(ref_idx, kind, offset, weight);
        let mut projected = base.clone();
        projected.projections.push(p);
        let mut wired = base;
        eight_edges(&mut wired, p);
        prop_assert_eq!(votes(&projected, &sensors), votes(&wired, &sensors));
    }

    /// With incumbent edges, one projection is the same eight terms appended
    /// after them, nonfinite weights and ring values included.
    #[test]
    fn one_projection_sums_after_incumbent_edges_like_appended_edges(
        mut food in ring_strategy(),
        nan_channel in prop::option::of(0usize..8),
        kind in kind_strategy(),
        offset in 0u8..8,
        weight in prop_oneof![
            -4.0f32..4.0,
            Just(f32::INFINITY),
            Just(f32::NEG_INFINITY),
            Just(f32::NAN),
            Just(3.0e38f32),
        ],
        incumbents in prop::collection::vec((0u8..8, -4.0f32..4.0), 1..6),
    ) {
        if let Some(channel) = nan_channel {
            food[channel] = f32::NAN;
        }
        let sensors = ring_sensors(food, ramp(0.0), ramp(0.25));
        let mut base = CgpGraphBackendDef::new_with_fixed_outputs();
        for (d, w) in incumbents {
            wire(&mut base, OutputSinkKind::ActionVote(kind.sink(d)), edge(memory(1, false), w));
        }
        let p = RingProjection::new(0, kind, offset, weight);
        let mut projected = base.clone();
        projected.projections.push(p);
        let mut wired = base;
        eight_edges(&mut wired, p);
        let (a, b) = (votes(&projected, &sensors), votes(&wired, &sensors));
        for (x, y) in a.iter().zip(&b) {
            prop_assert!(x.to_bits() == y.to_bits() || x == y, "{a:?} {b:?}");
        }
    }

    #[test]
    fn shift_rotates_the_kind_vote_by_one_direction(
        food in ring_strategy(),
        kind in kind_strategy(),
        offset in 0u8..8,
        weight in -4.0f32..4.0,
        seed in any::<u64>(),
    ) {
        let sensors = ring_sensors(food, ramp(0.0), ramp(0.25));
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        def.projections.push(RingProjection::new(0, kind, offset, weight));
        let before = votes(&def, &sensors);
        projection::shift_offset(&mut def, &mut SmallRng::seed_from_u64(seed)).unwrap();
        let after = votes(&def, &sensors);
        let step = (def.projections[0].offset + 8 - offset) % 8;
        prop_assert!(step == 1 || step == 7);
        for d in 0..8u8 {
            let from = (d + 8 - step) % 8;
            prop_assert_eq!(after[kind.sink(d).index()], before[kind.sink(from).index()]);
        }
    }
}

#[test]
fn duplicate_projections_and_incumbent_edges_sum_in_the_stated_order() {
    let food = [0.5, -1.25, 3.0, 0.125, 7.5, -0.75, 2.0, 1e-3];
    let sensors = ring_sensors(food, ramp(0.0), ramp(0.25));
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let edge_weight = 0.3;
    wire(
        &mut def,
        OutputSinkKind::ActionVote(VoteSink::Move(2)),
        edge(memory(1, false), edge_weight),
    );
    let (w1, k1, w2, k2) = (0.7f32, 3u8, -1.9f32, 3u8);
    def.projections
        .push(RingProjection::new(0, DirectedVoteKind::Move, k1, w1));
    def.projections
        .push(RingProjection::new(0, DirectedVoteKind::Move, k2, w2));
    let votes = votes(&def, &sensors);
    let memory_value = ramp::<16>(0.0)[1];
    for d in 0..8u8 {
        let c = usize::from((d + 8 - k1) % 8);
        let p = (0.0f32 + w1 * food[c]) + w2 * food[c];
        let s = if d == 2 {
            [memory_value * edge_weight].iter().copied().sum::<f32>()
        } else {
            0.0
        };
        assert_eq!(
            votes[VoteSink::Move(d).index()],
            sanitize_f32(s + p),
            "d={d}"
        );
    }
}

#[test]
fn a_projection_off_range_or_off_ring_contributes_zero() {
    let sensors = ring_sensors(ramp(0.0), ramp(0.0), ramp(0.25));
    for ref_idx in [3, 4, u16::MAX] {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        def.projections.push(RingProjection::new(
            ref_idx,
            DirectedVoteKind::Reproduce,
            1,
            2.0,
        ));
        assert!(votes(&def, &sensors).iter().all(|&v| v == 0.0), "{ref_idx}");
    }
}

/// A nonfinite weight on an off-range or off-ring projection leaves the
/// kind's incumbent edge votes unchanged: the projection contributes 0, not
/// `weight · 0.0`.
#[test]
fn a_nonfinite_weight_off_range_or_off_ring_leaves_incumbent_votes_unchanged() {
    let sensors = ring_sensors(ramp(0.0), ramp(0.0), ramp(0.25));
    let mut base = CgpGraphBackendDef::new_with_fixed_outputs();
    for d in 0..8u8 {
        wire(
            &mut base,
            OutputSinkKind::ActionVote(DirectedVoteKind::Move.sink(d)),
            edge(leaf(0, u16::from(d)), 1.5),
        );
    }
    let incumbent = votes(&base, &sensors);
    assert!((0..8u8).all(|d| incumbent[DirectedVoteKind::Move.sink(d).index()] != 0.0));
    for ref_idx in [3, 4, u16::MAX] {
        for weight in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let mut def = base.clone();
            def.projections.push(RingProjection::new(
                ref_idx,
                DirectedVoteKind::Move,
                1,
                weight,
            ));
            assert_eq!(votes(&def, &sensors), incumbent, "{ref_idx} {weight}");
        }
    }
}

#[test]
fn one_weight_step_from_zero_moves_the_move_vote_toward_every_food_direction() {
    let mut food = [0.0; 8];
    for d in [1, 4, 6] {
        food[d] = 1.0;
    }
    let sensors = ring_sensors(food, ramp(0.0), ramp(0.25));
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    def.projections
        .push(RingProjection::new(0, DirectedVoteKind::Move, 0, 0.0));
    assert!(votes(&def, &sensors).iter().all(|&v| v == 0.0));
    projection::alter_weight(&mut def, &mut SmallRng::seed_from_u64(5)).unwrap();
    let weight = def.projections[0].weight;
    assert_ne!(weight, 0.0);
    let votes = votes(&def, &sensors);
    for (index, &vote) in votes.iter().enumerate() {
        let expected = match VoteSink::from_index(index) {
            Some(VoteSink::Move(d)) if food[usize::from(d)] != 0.0 => weight,
            _ => 0.0,
        };
        assert_eq!(vote, expected, "{index}");
    }
}

fn visit(def: &CgpGraphBackendDef, energy: f32, traced: bool) -> (MeshSideOutputs, f32, Vec<f32>) {
    let mut energy = energy;
    let mut state = GraphRuntimeState::new();
    let mut side = side();
    let mut memory = ramp(0.0);
    let config = RuntimeConfig {
        graph_node_base_cost: BASE_COST,
        ..RuntimeConfig::default()
    };
    let sensors = ring_sensors(ramp(0.0), ramp(0.5), ramp(0.25));
    let refs = rings();
    let mut applied = Vec::new();
    if traced {
        let (_, trace) = execute_graph_node_traced(
            def,
            &refs,
            &ramp(0.25),
            &mut energy,
            25.0,
            0,
            &mut state,
            &sensors,
            &config,
            &mut side,
            &mut memory,
            &ramp(0.5),
        );
        applied = trace
            .output_sinks
            .iter()
            .zip(&def.output_sinks)
            .filter(|(_, sink)| matches!(sink.kind, OutputSinkKind::ActionVote(_)))
            .map(|(record, _)| record.applied_value)
            .collect();
    } else {
        let _ = execute_graph_node(
            def,
            &refs,
            &ramp(0.25),
            &mut energy,
            25.0,
            0,
            &mut state,
            &sensors,
            &config,
            &mut side,
            &mut memory,
            &ramp(0.5),
        );
    }
    (side, energy, applied)
}

fn projection_only() -> CgpGraphBackendDef {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    def.projections.push(RingProjection::new(
        1,
        DirectedVoteKind::StealEnergy,
        2,
        0.75,
    ));
    def
}

#[test]
fn production_and_traced_visits_agree_and_the_trace_carries_projection_terms() {
    let def = projection_only();
    assert!(def.enters_visit());
    let (production, energy_a, _) = visit(&def, 100.0, false);
    let (traced, energy_b, applied) = visit(&def, 100.0, true);
    assert_eq!(production.dispatch_effects(), traced.dispatch_effects());
    assert_eq!(energy_a, energy_b);
    let votes = production.dispatch_effects().0;
    assert_eq!(applied, votes.to_vec());
    assert!((0..8).all(|d| votes[VoteSink::StealEnergy(d).index()] != 0.0));
}

#[test]
fn a_projection_on_a_dormant_graph_pays_as_its_first_zero_vote_edge_does() {
    let (_, untouched, _) = visit(&CgpGraphBackendDef::new_with_fixed_outputs(), 100.0, false);
    assert_eq!(untouched, 100.0, "a dormant graph's visit is free");
    let (side_a, projected, _) = visit(&projection_only(), 100.0, false);
    let mut zero_edge = CgpGraphBackendDef::new_with_fixed_outputs();
    wire(
        &mut zero_edge,
        OutputSinkKind::ActionVote(VoteSink::Eat),
        edge(memory(0, false), 0.0),
    );
    let (side_b, edged, _) = visit(&zero_edge, 100.0, false);
    assert!(projected < 100.0);
    assert_eq!(projected, edged);
    assert_eq!(
        side_a.work_counters.graph_relax_iters,
        side_b.work_counters.graph_relax_iters
    );
}

#[test]
fn an_exhausted_visit_stages_no_projection_term() {
    let (side, _, _) = visit(&projection_only(), 0.0, false);
    assert!(side.dispatch_effects().0.iter().all(|&v| v == 0.0));
}

proptest! {
    /// An ablation-only mask bit reads its ring channel as 0.0 and leaves
    /// every other channel's term unchanged.
    #[test]
    fn an_ablated_channel_term_reads_zero(
        food in ring_strategy(),
        channel in 0usize..8,
        kind in kind_strategy(),
        offset in 0u8..8,
        weight in -4.0f32..4.0,
    ) {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        let mut p = RingProjection::new(0, kind, offset, weight);
        p.ablated_channels = 1 << channel;
        def.projections.push(p);
        let mut zeroed = food;
        zeroed[channel] = 0.0;
        let masked = votes(&def, &ring_sensors(food, ramp(0.0), ramp(0.25)));
        def.projections[0].ablated_channels = 0;
        let expected = votes(&def, &ring_sensors(zeroed, ramp(0.0), ramp(0.25)));
        prop_assert_eq!(masked, expected);
    }
}
