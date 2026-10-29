//! T25.F03: `Ring` edge bundles through the native effects pass. Members are
//! ordinary sink edges at their genomic weight, so equality with the
//! equivalent ungrouped edges is bitwise.
use super::*;
use crate::contracts::votes::DirectedVoteKind;
use crate::genome::cgp::{BundleId, EdgeSurface};
use crate::mutation::graph::bundle;
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
        InputReference::World(WorldInputKey::NearbyCreature),
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
    let (result, contribution) = apply_cgp_graph_effects(
        &mut crate::runtime::cgp::tracer::NoopTracer,
        def,
        &def.sink_weight_rows(refs),
        &[0.5; 3],
        refs,
        &ctx,
        &upstream,
        None,
        &mut Vec::new(),
        &mut side.work_counters.shared_memory_writes_changed,
        &mut side.action_params,
        &mut memory,
        &ramp(0.5),
    );
    side.stage_vote_contribution(&contribution);
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

/// Nonzero corrections, one per member.
fn corrections_strategy() -> impl Strategy<Value = Vec<f32>> {
    prop::collection::vec(prop_oneof![-4.0f32..-1e-3, 1e-3f32..4.0], 8)
}

/// Add a `Ring` bundle on `ring_ref` with `scale` and one correction per
/// member, in member order.
fn add_bundle(
    def: &mut CgpGraphBackendDef,
    ring_ref: u16,
    kind: DirectedVoteKind,
    offset: u8,
    scale: f32,
    corrections: &[f32],
) -> BundleId {
    let id = def.add_ring_bundle(ring_ref, kind, offset);
    def.bundle_mut(id).set_scale(scale);
    let members: Vec<_> = def.members(id).collect();
    for (occurrence, &own) in members.into_iter().zip(corrections) {
        def.edge_mut(occurrence).set_weight(own);
    }
    id
}

/// The eight ungrouped edges `grouped`'s bundle `id` is equivalent to: per
/// member, an edge from the same leaf into the same sink at weight
/// `scale + own_weight`, appended after the sink's incumbents (the member's
/// I10 position when the bundle is the sink's only one).
fn eight_edges(base: &mut CgpGraphBackendDef, grouped: &CgpGraphBackendDef, id: BundleId) {
    let scale = grouped.bundles()[id.index()].scale();
    for occurrence in grouped.members(id) {
        let member = grouped.edge_at(occurrence);
        let EdgeSurface::SinkInput(sink) = occurrence.surface else {
            unreachable!("members are sink edges")
        };
        wire(
            base,
            grouped.output_sinks()[sink].kind,
            edge(member.source(), scale + member.own_weight()),
        );
    }
}

proptest! {
    /// A new bundle (scale 0, corrections 0) on a valid ring is exactly
    /// neutral to every vote, result and memory.
    #[test]
    fn a_new_bundle_is_exactly_neutral(
        food in ring_strategy(),
        barrier in ring_strategy(),
        occupied in ring_strategy(),
        kind in kind_strategy(),
        offset in 0u8..8,
        ring_ref in 0u16..3,
        incumbents in prop::collection::vec((0usize..27, -4.0f32..4.0), 0..6),
    ) {
        let sensors = ring_sensors(food, barrier, occupied);
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        for (sink, weight) in incumbents {
            let vote = VoteSink::from_index(sink).unwrap();
            wire(&mut def, OutputSinkKind::ActionVote(vote), edge(memory(0, true), weight));
        }
        let before = effects_at(&def, &rings(), &sensors);
        def.add_ring_bundle(ring_ref, kind, offset);
        let after = effects_at(&def, &rings(), &sensors);
        prop_assert_eq!(after.side.dispatch_effects(), before.side.dispatch_effects());
        prop_assert_eq!(after.result, before.result);
        prop_assert_eq!(after.memory, before.memory);
        prop_assert_eq!(after.side.work_counters, before.side.work_counters);
    }

    /// The `eight_edges` oracle: a bundle (scale `s`, corrections `c_i` drawn
    /// nonzero) votes bitwise as its eight ungrouped edges of weight
    /// `s + c_i` on a kind without other edges.
    #[test]
    fn a_bundle_equals_its_eight_edges_on_a_kind_without_edges(
        food in ring_strategy(),
        barrier in ring_strategy(),
        kind in kind_strategy(),
        offset in 0u8..8,
        ring_ref in 0u16..3,
        scale in -4.0f32..4.0,
        corrections in corrections_strategy(),
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
        let mut grouped = base.clone();
        let id = add_bundle(&mut grouped, ring_ref, kind, offset, scale, &corrections);
        let mut wired = base;
        eight_edges(&mut wired, &grouped, id);
        let (a, b) = (votes(&grouped, &sensors), votes(&wired, &sensors));
        prop_assert_eq!(a.map(f32::to_bits), b.map(f32::to_bits));
    }

    /// With incumbent edges, a bundle's members sum after them like appended
    /// edges, nonfinite scales and ring values included.
    #[test]
    fn a_bundle_sums_after_incumbent_edges_like_appended_edges(
        mut food in ring_strategy(),
        nan_channel in prop::option::of(0usize..8),
        kind in kind_strategy(),
        offset in 0u8..8,
        scale in prop_oneof![
            -4.0f32..4.0,
            Just(f32::INFINITY),
            Just(f32::NEG_INFINITY),
            Just(f32::NAN),
            Just(3.0e38f32),
        ],
        corrections in corrections_strategy(),
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
        let mut grouped = base.clone();
        let id = add_bundle(&mut grouped, 0, kind, offset, scale, &corrections);
        let mut wired = base;
        eight_edges(&mut wired, &grouped, id);
        let (a, b) = (votes(&grouped, &sensors), votes(&wired, &sensors));
        for (x, y) in a.iter().zip(&b) {
            prop_assert!(x.to_bits() == y.to_bits() || x == y, "{a:?} {b:?}");
        }
    }

    #[test]
    fn shift_rotates_the_kind_vote_by_one_direction(
        food in ring_strategy(),
        kind in kind_strategy(),
        offset in 0u8..8,
        scale in -4.0f32..4.0,
        seed in any::<u64>(),
    ) {
        let sensors = ring_sensors(food, ramp(0.0), ramp(0.25));
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        add_bundle(&mut def, 0, kind, offset, scale, &[0.0; 8]);
        let before = votes(&def, &sensors);
        bundle::shift_offset(&mut def, &mut SmallRng::seed_from_u64(seed)).unwrap();
        let after = votes(&def, &sensors);
        let step = (def.bundles()[0].wiring().offset().expect("a geometric bundle") + 8 - offset) % 8;
        prop_assert!(step == 1 || step == 7);
        for d in 0..8u8 {
            let from = (d + 8 - step) % 8;
            prop_assert_eq!(after[kind.sink(d).index()], before[kind.sink(from).index()]);
        }
    }

    /// Redirecting a member's read out of range (the ablation rewrite) reads
    /// its ring channel as 0.0 at the member's unchanged genomic weight, and
    /// leaves every other channel's term unchanged.
    #[test]
    fn an_ablated_member_reads_zero(
        food in ring_strategy(),
        channel in 0u16..8,
        kind in kind_strategy(),
        offset in 0u8..8,
        scale in -4.0f32..4.0,
    ) {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        add_bundle(&mut def, 0, kind, offset, scale, &[0.0; 8]);
        let mut zeroed = food;
        zeroed[usize::from(channel)] = 0.0;
        let expected = votes(&def, &ring_sensors(zeroed, ramp(0.0), ramp(0.25)));
        def.redirect_leaf_reads(|ref_idx, sub_idx| (ref_idx == 0 && sub_idx == channel).then_some(u16::MAX));
        let masked = votes(&def, &ring_sensors(food, ramp(0.0), ramp(0.25)));
        prop_assert_eq!(masked, expected);
    }
}

/// Two bundles of one kind and offset and an incumbent edge sum sequentially
/// in the sink's I10 order: the incumbent, then the first bundle's member,
/// then the second's.
#[test]
fn duplicate_bundles_and_incumbent_edges_sum_in_sink_order() {
    let food = [0.5, -1.25, 3.0, 0.125, 7.5, -0.75, 2.0, 1e-3];
    let sensors = ring_sensors(food, ramp(0.0), ramp(0.25));
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let edge_weight = 0.3;
    wire(
        &mut def,
        OutputSinkKind::ActionVote(VoteSink::Directed(DirectedVoteKind::Move, 2)),
        edge(memory(1, false), edge_weight),
    );
    let (w1, k1, w2, k2) = (0.7f32, 3u8, -1.9f32, 3u8);
    add_bundle(&mut def, 0, DirectedVoteKind::Move, k1, w1, &[0.0; 8]);
    add_bundle(&mut def, 0, DirectedVoteKind::Move, k2, w2, &[0.0; 8]);
    let votes = votes(&def, &sensors);
    let memory_value = ramp::<16>(0.0)[1];
    for d in 0..8u8 {
        let c = usize::from((d + 8 - k1) % 8);
        let members = [(w1 + 0.0) * food[c], (w2 + 0.0) * food[c]];
        let terms: Vec<f32> = if d == 2 {
            std::iter::once(memory_value * edge_weight)
                .chain(members)
                .collect()
        } else {
            members.to_vec()
        };
        assert_eq!(
            votes[VoteSink::Directed(DirectedVoteKind::Move, d).index()].to_bits(),
            sanitize_f32(terms.iter().copied().sum::<f32>()).to_bits(),
            "d={d}"
        );
    }
}

/// An unresolvable `ring_ref` (out of range or not a ring) is a
/// `ShapeReference` violation: such a bundle never reaches a live genome,
/// so no reader weighs it (T25.F11 invariant 3 deleted the pattern-0
/// fallback).
#[test]
fn an_unresolvable_ring_ref_fails_validation() {
    for ring_ref in [3, 4, u16::MAX] {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        add_bundle(
            &mut def,
            ring_ref,
            DirectedVoteKind::Reproduce,
            1,
            2.0,
            &[0.0; 8],
        );
        let node = crate::genome::NodeGenome::new(
            crate::contracts::NodeId(0),
            rings(),
            crate::genome::BackendDef::Graph(def),
            Vec::new(),
        );
        assert_eq!(
            crate::genome::cgp::validate_graph(&node),
            Err(crate::genome::cgp::GraphInvariantViolation::ShapeReference),
            "{ring_ref}"
        );
    }
}

#[test]
fn one_scale_step_from_zero_moves_the_move_vote_toward_every_food_direction() {
    let mut food = [0.0; 8];
    for d in [1, 4, 6] {
        food[d] = 1.0;
    }
    let sensors = ring_sensors(food, ramp(0.0), ramp(0.25));
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    def.add_ring_bundle(0, DirectedVoteKind::Move, 0);
    assert!(votes(&def, &sensors).iter().all(|&v| v == 0.0));
    bundle::alter_scale(&mut def, &mut SmallRng::seed_from_u64(5)).unwrap();
    let scale = def.bundles()[0].scale();
    assert_ne!(scale, 0.0);
    let votes = votes(&def, &sensors);
    for (index, &vote) in votes.iter().enumerate() {
        let expected = match VoteSink::from_index(index) {
            Some(VoteSink::Directed(DirectedVoteKind::Move, d)) if food[usize::from(d)] != 0.0 => {
                scale
            }
            _ => 0.0,
        };
        assert_eq!(vote, expected, "{index}");
    }
}

fn visit(def: &CgpGraphBackendDef, energy: f32, traced: bool) -> (MeshSideOutputs, f32, Vec<f32>) {
    let mut energy = energy;
    let refs = rings();
    let genome = crate::genome::CreatureGenome::new(
        crate::contracts::NodeId(0),
        vec![crate::genome::NodeGenome::new(
            crate::contracts::NodeId(0),
            refs.clone(),
            crate::genome::BackendDef::Graph(def.clone()),
            Vec::new(),
        )],
        crate::creature::founder::default_founder_body(),
    );
    let mut state =
        GraphRuntimeState::for_genome(&genome, &crate::genome::learning::Seeds::default());
    let mut side = side();
    let mut memory = ramp(0.0);
    let config = RuntimeConfig {
        graph_node_base_cost: BASE_COST,
        ..RuntimeConfig::default()
    };
    let sensors = ring_sensors(ramp(0.0), ramp(0.5), ramp(0.25));
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
            .zip(def.output_sinks())
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

fn bundle_only() -> CgpGraphBackendDef {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    add_bundle(
        &mut def,
        1,
        DirectedVoteKind::StealEnergy,
        2,
        0.75,
        &[0.0; 8],
    );
    def
}

#[test]
fn production_and_traced_visits_agree_and_the_trace_carries_member_terms() {
    let def = bundle_only();
    assert!(def.enters_visit());
    let (production, energy_a, _) = visit(&def, 100.0, false);
    let (traced, energy_b, applied) = visit(&def, 100.0, true);
    assert_eq!(production.dispatch_effects(), traced.dispatch_effects());
    assert_eq!(energy_a, energy_b);
    let votes = production.dispatch_effects().0;
    assert_eq!(applied, votes.to_vec());
    assert!(
        (0..8).all(|d| votes[VoteSink::Directed(DirectedVoteKind::StealEnergy, d).index()] != 0.0)
    );
}

#[test]
fn a_bundle_on_a_dormant_graph_pays_as_its_first_zero_vote_edge_does() {
    let (_, untouched, _) = visit(&CgpGraphBackendDef::new_with_fixed_outputs(), 100.0, false);
    assert_eq!(untouched, 100.0, "a dormant graph's visit is free");
    let (side_a, grouped, _) = visit(&bundle_only(), 100.0, false);
    let mut zero_edge = CgpGraphBackendDef::new_with_fixed_outputs();
    wire(
        &mut zero_edge,
        OutputSinkKind::ActionVote(VoteSink::Eat),
        edge(memory(0, false), 0.0),
    );
    let (side_b, edged, _) = visit(&zero_edge, 100.0, false);
    assert!(grouped < 100.0);
    assert_eq!(grouped, edged);
    assert_eq!(
        side_a.work_counters.graph_relax_iters,
        side_b.work_counters.graph_relax_iters
    );
}

#[test]
fn an_exhausted_visit_stages_no_member_term() {
    let (side, _, _) = visit(&bundle_only(), 0.0, false);
    assert!(side.dispatch_effects().0.iter().all(|&v| v == 0.0));
}

// ── Direction-pair bundles (T25.F04) ─────────────────────────────────────────

mod pair {
    use super::*;
    use crate::config::{SimulationConfig, WorldEdgeMode};
    use crate::contracts::{CreatureId, Position};
    use crate::creature::founder::v3alpha1_founder_genome;
    use crate::creature::state::CreatureState;
    use crate::genome::body::BodyGenome;
    use crate::genome::body::Phenotype;
    use crate::kernel::WorldState;
    use crate::sensors::perception::PerceptionConfig;
    use crate::sensors::reducers::assemble_perception;
    use crate::sensors::visibility::{compute_visible_cells, get_visibility_table};
    use slotmap::SlotMap;

    /// What one fixture world holds besides the observer.
    #[derive(Clone, Copy, Debug)]
    enum Thing {
        Food,
        Barrier,
        Creature,
    }

    const ORIGIN: (u16, u16) = (10, 10);

    fn at(dx: i32, dy: i32) -> Position {
        Position::new(
            u16::try_from(i32::from(ORIGIN.0) + dx).unwrap(),
            u16::try_from(i32::from(ORIGIN.1) + dy).unwrap(),
        )
    }

    fn creature(id: CreatureId, position: Position) -> CreatureState {
        crate::creature::birth::Zygote::founder(
            v3alpha1_founder_genome().with_body(BodyGenome::new(
                Phenotype::new([0; 6], 0, [true; 6]).expect("an in-range active channel"),
                0,
                200.0,
                0.2,
                0.2,
            )),
            0,
        )
        .born_at(id, position, 50.0)
    }

    /// The perception of an observer at `ORIGIN` on a 20×20 wrapped world
    /// holding `thing` at each of `cells`.
    fn perceive(thing: Thing, cells: &[(i32, i32)]) -> SensorSnapshot {
        let mut world = WorldState::new(20, 20, WorldEdgeMode::Wrap);
        let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
        let origin = at(0, 0);
        let observer = creatures.insert_with_key(|id| creature(id, origin));
        world.place_creature(origin, observer);
        for &(dx, dy) in cells {
            let cell = at(dx, dy);
            match thing {
                Thing::Food => world.set_food(cell, 1.0),
                Thing::Barrier => world.set_barrier(cell, true),
                Thing::Creature => {
                    let other = creatures.insert_with_key(|id| creature(id, cell));
                    world.place_creature(cell, other);
                }
            }
        }
        let config = PerceptionConfig::default();
        let visible =
            compute_visible_cells(origin, &world, get_visibility_table(config.vision_radius));
        let mut sensors = sensors();
        let capacity = SimulationConfig::default().energy.capacity;
        let views: crate::sensors::static_inputs::BodyViews = creatures
            .iter()
            .map(|(id, c)| (id, c.sensed_body(&capacity)))
            .collect();
        sensors.perception =
            assemble_perception(observer, visible.cells(), &world, &views, &config);
        sensors
    }

    /// `kind`'s eight direction votes of a graph holding one pair bundle on
    /// `refs[0]`'s pair `pair` at `offset`, scale 1.
    fn direction_votes(
        reference: &InputReference,
        pair: u8,
        kind: DirectedVoteKind,
        offset: u8,
        sensors: &SensorSnapshot,
    ) -> [f32; 8] {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        let layout = crate::contracts::pair_layout(reference, pair);
        let id = def.add_pair_bundle(0, pair, layout, kind, offset);
        def.bundle_mut(id).set_scale(1.0);
        let votes = effects_at(&def, std::slice::from_ref(reference), sensors)
            .side
            .dispatch_effects()
            .0;
        std::array::from_fn(|d| votes[kind.sink(d as u8).index()])
    }

    fn argmax(votes: &[f32; 8]) -> usize {
        let best = votes.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let winners: Vec<_> = (0..8).filter(|&d| votes[d] == best).collect();
        assert_eq!(winners.len(), 1, "a unique argmax: {votes:?}");
        winners[0]
    }

    /// Each pair of each area family and slot 0 of `NearbyCreature`.
    fn cases() -> Vec<(Thing, InputReference, u8)> {
        let food =
            InputReference::World(WorldInputKey::area_food_summary(OrdinaryFoodTypeId::new(0)));
        let barrier = InputReference::World(WorldInputKey::AreaBarrierSummary);
        let crowd = InputReference::World(WorldInputKey::AreaOccupancySummary);
        let core = InputReference::World(WorldInputKey::NearbyCreature);
        vec![
            (Thing::Food, food.clone(), 0),
            (Thing::Food, food, 1),
            (Thing::Barrier, barrier.clone(), 0),
            (Thing::Barrier, barrier, 1),
            (Thing::Creature, crowd.clone(), 0),
            (Thing::Creature, crowd, 1),
            (Thing::Creature, core, 0),
        ]
    }

    fn check_each_direction(offset: u8) {
        for (thing, reference, pair) in cases() {
            for (d, direction) in Direction::ALL.into_iter().enumerate() {
                let sensors = perceive(thing, &[direction.delta()]);
                for kind in DirectedVoteKind::ALL {
                    let votes = direction_votes(&reference, pair, kind, offset, &sensors);
                    assert_eq!(
                        argmax(&votes),
                        (d + usize::from(offset)) % 8,
                        "{thing:?} {reference:?} pair {pair} {kind:?} toward {direction:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_pair_bundle_at_offset_0_votes_toward_a_thing_at_each_direction() {
        check_each_direction(0);
    }

    #[test]
    fn a_pair_bundle_at_offset_4_votes_away_from_a_thing_at_each_direction() {
        check_each_direction(4);
    }

    /// Slots 1–3: `k` nearer creatures off the tested direction rank the
    /// target `k`-th; only slot `k`'s pair is grouped, so the vote follows
    /// the target alone.
    #[test]
    fn a_nearby_creature_slot_pair_votes_toward_its_ranked_creature() {
        let core = InputReference::World(WorldInputKey::NearbyCreature);
        for k in 1..4u8 {
            for (d, direction) in Direction::ALL.into_iter().enumerate() {
                let (dx, dy) = direction.delta();
                // The target two steps out; `k` nearer ones on adjacent cells
                // at other directions.
                let mut cells = vec![(2 * dx, 2 * dy)];
                cells.extend(
                    Direction::ALL
                        .into_iter()
                        .filter(|&other| other != direction)
                        .take(usize::from(k))
                        .map(Direction::delta),
                );
                let sensors = perceive(Thing::Creature, &cells);
                for offset in [0u8, 4] {
                    let votes = direction_votes(&core, k, DirectedVoteKind::Move, offset, &sensors);
                    assert_eq!(
                        argmax(&votes),
                        (d + usize::from(offset)) % 8,
                        "slot {k} toward {direction:?} at offset {offset}"
                    );
                }
            }
        }
    }

    fn perception_strategy() -> impl Strategy<Value = [f32; 7]> {
        prop::array::uniform7(-1.0f32..1.0)
    }

    proptest! {
        /// A pair bundle (scale `s`, corrections `c_i`) votes bitwise as its
        /// twelve ungrouped edges of weight `compose(c_i, s, pattern_i)` in
        /// the I10 position, beside edges of other kinds.
        #[test]
        fn a_pair_bundle_equals_its_fifteen_edges(
            area in perception_strategy(),
            kind in kind_strategy(),
            offset in 0u8..8,
            pair in 0u8..2,
            scale in -4.0f32..4.0,
            corrections in prop::collection::vec(-4.0f32..4.0, 12),
            others in prop::collection::vec((0usize..27, -4.0f32..4.0), 0..4),
        ) {
            let mut sensors = sensors();
            sensors.perception.area_barrier = area;
            let refs = [InputReference::World(WorldInputKey::AreaBarrierSummary)];
            let mut base = CgpGraphBackendDef::new_with_fixed_outputs();
            for (sink, w) in others {
                let vote = VoteSink::from_index(sink).unwrap();
                if DirectedVoteKind::of_sink(vote).is_some_and(|(k, _)| k == kind) {
                    continue;
                }
                wire(&mut base, OutputSinkKind::ActionVote(vote), edge(memory(0, true), w));
            }
            let mut grouped = base.clone();
            let layout = crate::contracts::pair_layout(&refs[0], pair);
            let id = grouped.add_pair_bundle(0, pair, layout, kind, offset);
            grouped.bundle_mut(id).set_scale(scale);
            let members: Vec<_> = grouped.members(id).collect();
            prop_assert_eq!(members.len(), 12 + 3, "twelve compass members and three facts");
            for (occurrence, &own) in members.iter().zip(&corrections) {
                grouped.edge_mut(*occurrence).set_weight(own);
            }
            let mut wired = base;
            for occurrence in grouped.members(id) {
                let member = grouped.edge_at(occurrence);
                let EdgeSurface::SinkInput(sink) = occurrence.surface else {
                    unreachable!("members are sink edges")
                };
                wire(
                    &mut wired,
                    grouped.output_sinks()[sink].kind,
                    edge(member.source(), grouped.genomic_weight(occurrence, &refs)),
                );
            }
            let a = effects_at(&grouped, &refs, &sensors).side.dispatch_effects().0;
            let b = effects_at(&wired, &refs, &sensors).side.dispatch_effects().0;
            prop_assert_eq!(a.map(f32::to_bits), b.map(f32::to_bits));
        }
    }
}
