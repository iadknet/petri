use std::collections::{BTreeMap, BTreeSet};

use proptest::prelude::*;
use prost::Message as _;
use v3_core::config::OrdinaryFoodTypeId;
use v3_core::creature::sensor_census::world_input_key_universe;
use v3_core::mutation::{
    MutationAddedNodeInputClass, MutationDomain, MutationOperator, MutationOperatorFunnel,
    MutationSkipReason,
};
use v3_core::simulation::actions::{
    BarrierReaderState, MoveBlockedCause, PredationActionResult, ReproductionActionResult,
    ReproductionInvalidTargetCause,
};

use super::*;
use crate::testing::{decode_metrics, MetricKind, PointValue, ReceivedSnapshot};

const MS: Duration = Duration::from_millis(1);

#[test]
fn an_interval_snapshot_waits_for_the_interval_since_the_last_one_or_the_start() {
    let start = Instant::now();
    let interval = 100 * MS;
    let due_at = |last, tick, after| {
        due(
            Trigger::Interval,
            last,
            start,
            tick,
            start + after,
            interval,
        )
    };
    assert!(!due_at(None, 1, 99 * MS));
    assert!(due_at(None, 1, 100 * MS));
    let last = Some(Taken {
        tick: 1,
        at: start + 100 * MS,
    });
    assert!(!due_at(last, 2, 199 * MS));
    assert!(due_at(last, 2, 200 * MS));
    assert!(!due_at(last, 1, 900 * MS), "one snapshot per tick");
}

#[test]
fn a_transition_snapshot_needs_a_new_tick_and_10_ms_since_the_last_one() {
    let start = Instant::now();
    let interval = 1_000 * MS;
    let due_at = |last, tick, after| {
        due(
            Trigger::Transition,
            last,
            start,
            tick,
            start + after,
            interval,
        )
    };
    assert!(due_at(None, 0, Duration::ZERO), "a run's first snapshot");
    let last = Some(Taken {
        tick: 4,
        at: start + 50 * MS,
    });
    assert!(
        !due_at(last, 4, 500 * MS),
        "a second transition at one tick"
    );
    assert!(!due_at(last, 5, 59 * MS));
    assert!(due_at(last, 5, 60 * MS));
}

#[test]
fn a_run_end_snapshot_is_taken_unless_its_tick_has_one() {
    let start = Instant::now();
    let last = Some(Taken { tick: 9, at: start });
    assert!(due(Trigger::RunEnd, last, start, 10, start, 1_000 * MS));
    assert!(!due(Trigger::RunEnd, last, start, 9, start, 1_000 * MS));
    assert!(due(Trigger::RunEnd, None, start, 0, start, 1_000 * MS));
}

proptest! {
    #[test]
    fn a_stamp_is_the_later_of_capture_and_previous_plus_1_ms(
        now in 0_u64..u64::MAX / 2,
        previous in proptest::option::of(0_u64..u64::MAX / 2),
    ) {
        let stamped = stamp(now, previous);
        prop_assert!(stamped >= now);
        if let Some(previous) = previous {
            prop_assert!(stamped >= previous + STAMP_STEP_NS);
            prop_assert!(stamped == now || stamped == previous + STAMP_STEP_NS);
        } else {
            prop_assert_eq!(stamped, now);
        }
    }
}

/// A `SimStats` with every map key populated: two food types, every enum
/// key once, each `WorldInputKey` of both food types once.
fn populated() -> SimStats {
    let mut s = SimStats::default();
    let types = [OrdinaryFoodTypeId::new(0), OrdinaryFoodTypeId::new(1)];
    let skip = [
        MutationSkipReason::ParseabilityViolation,
        MutationSkipReason::NumericProposalRejected,
        MutationSkipReason::NoApplicableTarget,
    ];
    let classes = [
        MutationAddedNodeInputClass::None,
        MutationAddedNodeInputClass::Food,
        MutationAddedNodeInputClass::Neighbor,
        MutationAddedNodeInputClass::Barrier,
        MutationAddedNodeInputClass::Occupancy,
        MutationAddedNodeInputClass::Introspection,
        MutationAddedNodeInputClass::Upstream,
        MutationAddedNodeInputClass::ActionQueue,
        MutationAddedNodeInputClass::Decision,
    ];
    let readers = [
        BarrierReaderState::HasBarrierReader,
        BarrierReaderState::NoBarrierReader,
    ];
    for cause in DeathCause::ALL {
        s.mortality.record(cause);
    }
    for class in CognitiveClass::ALL {
        s.reproductive_success_by_cognitive_class
            .record(class, 2, 30);
    }
    s.energy_flows.food_intake_by_type = vec![1.5, 2.5];
    s.energy_flows.action_charges.r#move = -3.25;
    s.phase_wall_clock.cognition = Duration::from_millis(1_500);
    for domain in MutationDomain::all() {
        s.mutation_events_attempted_total_by_domain
            .insert(domain, 1);
        s.mutation_events_applied_total_by_domain.insert(domain, 1);
    }
    for reason in skip {
        s.mutation_events_skipped_by_reason.insert(reason, 1);
    }
    let inputs = world_input_key_universe(types);
    for operator in MutationOperator::all() {
        s.mutation_events_attempted_total_by_operator
            .insert(operator, 1);
        s.mutation_events_applied_total_by_operator
            .insert(operator, 1);
        s.mutation_events_skipped_total_by_operator
            .insert(operator, 1);
        s.mutation_operator_funnel_total_by_operator
            .insert(operator, MutationOperatorFunnel::default());
        s.mutation_skip_reasons_total_by_operator
            .insert(operator, skip.iter().map(|reason| (*reason, 1)).collect());
        s.mutation_added_node_input_classes_total_by_operator
            .insert(operator, classes.iter().map(|class| (*class, 1)).collect());
        s.mutation_added_node_world_inputs_total_by_operator
            .insert(operator, inputs.iter().map(|input| (*input, 1)).collect());
        s.mutation_value_totals_by_operator
            .insert(operator, MutationValueTotals::default());
    }
    for result in [
        ReproductionActionResult::Spawned,
        ReproductionActionResult::RejectedInvalidTarget,
        ReproductionActionResult::RejectedAgeConstraints,
        ReproductionActionResult::RejectedEnergyConstraints,
        ReproductionActionResult::RejectedPopulationCap,
    ] {
        s.reproduction_actions_rejected_by_reason.insert(result, 1);
    }
    for cause in [
        ReproductionInvalidTargetCause::Barrier,
        ReproductionInvalidTargetCause::Occupied,
        ReproductionInvalidTargetCause::OutOfBounds,
        ReproductionInvalidTargetCause::Contention,
    ] {
        s.reproduction_actions_rejected_invalid_target_total_by_cause
            .insert(cause, 1);
    }
    for reader in readers {
        for map in [
            &mut s.reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state,
            &mut s.move_actions_blocked_avoidable_total_by_reader_state,
            &mut s.move_attempts_with_barrier_neighbor_total_by_reader_state,
            &mut s.move_blocked_barrier_with_barrier_neighbor_total_by_reader_state,
            &mut s.reproduction_attempts_with_barrier_neighbor_total_by_reader_state,
            &mut s.reproduction_invalid_target_barrier_with_barrier_neighbor_total_by_reader_state,
        ] {
            map.insert(reader, 1);
        }
    }
    for food in types {
        s.eat_actions_applied_total_by_type.insert(food, 1);
        s.eat_actions_failed_total_by_type.insert(food, 1);
    }
    for cause in [
        MoveBlockedCause::Barrier,
        MoveBlockedCause::Occupied,
        MoveBlockedCause::OutOfBounds,
    ] {
        s.move_actions_blocked_total_by_cause.insert(cause, 1);
    }
    for result in [
        PredationActionResult::Transferred,
        PredationActionResult::TransferredAndKilled,
        PredationActionResult::RejectedNoVictim,
    ] {
        s.predation_actions_by_result.insert(result, 1);
    }
    s.last_tick_food_total_density_by_type = vec![0.5, 0.25];
    s.last_tick_food_grazing_modifier_mean_by_type = vec![1.0, 1.0];
    s.last_tick_food_grazed_cell_share_by_type = vec![0.0, 0.0];
    s
}

fn strings<T>(keys: impl IntoIterator<Item = T>, key: impl Fn(T) -> String) -> Vec<String> {
    keys.into_iter().map(key).collect()
}
type Attribute = (&'static str, Vec<String>);
/// A family: OTel name, kind, and each attribute's expected values.
type Family = (String, MetricKind, Vec<Attribute>);

fn op() -> Attribute {
    (
        "petri.operator",
        strings(MutationOperator::all(), |k| k.as_key().to_owned()),
    )
}

fn food() -> Attribute {
    ("petri.food_type", vec!["0".to_owned(), "1".to_owned()])
}

fn skip_reasons() -> Attribute {
    (
        "petri.skip_reason",
        strings(
            [
                "ParseabilityViolation",
                "NumericProposalRejected",
                "NoApplicableTarget",
            ],
            str::to_owned,
        ),
    )
}

fn reader() -> Attribute {
    (
        "petri.reader_state",
        strings(
            [
                BarrierReaderState::HasBarrierReader,
                BarrierReaderState::NoBarrierReader,
            ],
            |k| k.as_key().to_owned(),
        ),
    )
}

/// The table's families for [`populated`].
fn expected() -> Vec<Family> {
    let mut families = Vec::new();
    run_families(&mut families);
    mutation_families(&mut families);
    outcome_families(&mut families);
    tick_families(&mut families);
    families
}

fn run_families(families: &mut Vec<Family>) {
    use MetricKind::{Gauge, MonotonicSum, Sum};
    families.extend([
        ("petri.run.tick".to_owned(), Gauge, vec![]),
        ("petri.run.elapsed_seconds".to_owned(), Gauge, vec![]),
    ]);
    for field in [
        "reproduction_actions_attempted",
        "reproduction_actions_spawned",
        "reproduction_actions_rejected",
        "mutation_events_attempted",
        "mutation_events_applied",
        "mutation_events_skipped",
        "move_actions_attempted",
        "mutation_reachable_target",
        "mutation_unreachable_target",
        "mutation_executed_target",
        "mutation_not_applicable_target",
        "mesh_hops",
        "vm_steps",
        "graph_relax_iters",
        "plasticity_updates",
        "plasticity_changes",
        "hebbian_updates",
        "hebbian_changes",
        "reward_modulated_updates",
        "reward_modulated_changes",
        "shared_memory_writes_changed",
        "creature_ticks",
        "mesh_dispatches_energy_exhausted",
        "pass_cap_hits",
        "passes",
        "decided_passes",
        "actions_applied",
        "predation_actions_attempted",
        "predation_actions_transferred",
        "predation_actions_rejected",
        "predation_kills",
        "deaths",
        "genome_size_creature_ticks",
    ] {
        families.push((format!("petri.run.{field}"), MonotonicSum, vec![]));
    }
    families.push((
        "petri.run.deaths_by_cause".to_owned(),
        MonotonicSum,
        vec![(
            "petri.cause",
            strings(DeathCause::ALL, |k| k.as_key().to_owned()),
        )],
    ));
    for field in ["creatures_observed", "offspring_spawned", "survival_ticks"] {
        families.push((
            format!("petri.run.reproductive_success.{field}"),
            MonotonicSum,
            vec![(
                "petri.cognitive_class",
                strings(CognitiveClass::ALL, |k| k.as_key().to_owned()),
            )],
        ));
    }
    let mut flows = strings(
        [
            "failed_action_penalty",
            "vm_compute",
            "priority_bid",
            "graph_compute",
            "mesh_ramp",
            "hebbian_learning",
            "reward_learning",
            "lifecycle_decay",
            "genome_carrying",
            "parental_transfer_debit",
            "offspring_energy_credit",
            "predation_victim_debit",
            "predation_attacker_credit",
            "predation_kill_bonus_credit",
            "maximum_energy_clamp_loss",
            "zero_floor_credit",
            "external_removal_loss",
            "action_charge.noop",
            "action_charge.eat",
            "action_charge.move",
            "action_charge.reproduce",
            "action_charge.steal_energy",
            "food_intake.0",
            "food_intake.1",
        ],
        str::to_owned,
    );
    flows.sort();
    families.push((
        "petri.run.energy_flow".to_owned(),
        Sum,
        vec![("petri.flow", flows)],
    ));
    families.push((
        "petri.run.phase_wall_clock".to_owned(),
        MonotonicSum,
        vec![(
            "petri.phase",
            strings(
                [
                    "world_update",
                    "sensor_assembly",
                    "cognition",
                    "actions",
                    "reward_learning",
                ],
                str::to_owned,
            ),
        )],
    ));
}

fn mutation_families(families: &mut Vec<Family>) {
    use MetricKind::{MonotonicSum, Sum};
    for verb in ["attempted", "applied"] {
        families.push((
            format!("petri.run.mutation_events_{verb}_by_domain"),
            MonotonicSum,
            vec![(
                "petri.domain",
                strings(MutationDomain::all(), |k| k.as_key().to_owned()),
            )],
        ));
    }
    for verb in ["attempted", "applied", "skipped"] {
        families.push((
            format!("petri.run.mutation_events_{verb}_by_operator"),
            MonotonicSum,
            vec![op()],
        ));
    }
    families.push((
        "petri.run.mutation_events_skipped_by_reason".to_owned(),
        MonotonicSum,
        vec![skip_reasons()],
    ));
    families.push((
        "petri.run.mutation_operator_funnel".to_owned(),
        MonotonicSum,
        vec![
            op(),
            (
                "petri.stage",
                strings(
                    [
                        "attempted",
                        "applicable",
                        "structurally_valid",
                        "applied",
                        "skipped",
                    ],
                    str::to_owned,
                ),
            ),
        ],
    ));
    families.push((
        "petri.run.mutation_skip_reasons_by_operator".to_owned(),
        MonotonicSum,
        vec![op(), skip_reasons()],
    ));
    families.push((
        "petri.run.mutation_added_node_input_classes_by_operator".to_owned(),
        MonotonicSum,
        vec![
            op(),
            (
                "petri.input_class",
                strings(
                    [
                        "none",
                        "food",
                        "neighbor",
                        "barrier",
                        "occupancy",
                        "introspection",
                        "upstream",
                        "action_queue",
                        "decision",
                    ],
                    str::to_owned,
                ),
            ),
        ],
    ));
    families.push((
        "petri.run.mutation_added_node_world_inputs_by_operator".to_owned(),
        MonotonicSum,
        vec![
            op(),
            (
                "petri.world_input",
                strings(
                    [
                        "FoodHere",
                        "NeighborFoodRing",
                        "NeighborBarrierRing",
                        "NeighborOccupiedRing",
                        "AreaFoodSummary",
                        "AreaBarrierSummary",
                        "AreaOccupancySummary",
                        "NearbyCreatureCore",
                        "NearbyCreatureVitals",
                        "NearbyCreatureIdentity",
                    ],
                    str::to_owned,
                ),
            ),
        ],
    ));
    for (field, kind) in [
        ("carriers_observed", MonotonicSum),
        ("survival_ticks_sum", MonotonicSum),
        ("offspring_spawned_sum", MonotonicSum),
        ("final_energy_sum", Sum),
        ("helpful", MonotonicSum),
        ("neutral", MonotonicSum),
        ("detrimental", MonotonicSum),
        ("confidence_low", MonotonicSum),
        ("confidence_medium", MonotonicSum),
        ("confidence_high", MonotonicSum),
        ("viability_score_sum", Sum),
        ("viability_score_delta_sum", Sum),
        ("survived_short_horizon", MonotonicSum),
        ("survived_long_horizon", MonotonicSum),
        ("reproduced_once", MonotonicSum),
        ("mean_lifetime_energy_sum", Sum),
        ("action_attempted", MonotonicSum),
        ("blocked_move", MonotonicSum),
        ("invalid_reproduce", MonotonicSum),
        ("invalid_action", MonotonicSum),
    ] {
        families.push((
            format!("petri.run.mutation_value.{field}"),
            kind,
            vec![op()],
        ));
        families.push((
            format!("petri.run.mutation_outcome_summary.{field}"),
            kind,
            vec![],
        ));
    }
}

fn outcome_families(families: &mut Vec<Family>) {
    use MetricKind::{Gauge, MonotonicSum};
    families.push((
        "petri.run.reproduction_actions_rejected_by_reason".to_owned(),
        MonotonicSum,
        vec![(
            "petri.reason",
            strings(
                [
                    "Spawned",
                    "RejectedInvalidTarget",
                    "RejectedAgeConstraints",
                    "RejectedEnergyConstraints",
                    "RejectedPopulationCap",
                ],
                str::to_owned,
            ),
        )],
    ));
    families.push((
        "petri.run.reproduction_actions_rejected_invalid_target_by_cause".to_owned(),
        MonotonicSum,
        vec![(
            "petri.cause",
            strings(
                ["barrier", "occupied", "out_of_bounds", "contention"],
                str::to_owned,
            ),
        )],
    ));
    for field in [
        "reproduction_actions_rejected_invalid_target_avoidable_by_reader_state",
        "move_actions_blocked_avoidable_by_reader_state",
        "move_attempts_with_barrier_neighbor_by_reader_state",
        "move_blocked_barrier_with_barrier_neighbor_by_reader_state",
        "reproduction_attempts_with_barrier_neighbor_by_reader_state",
        "reproduction_invalid_target_barrier_with_barrier_neighbor_by_reader_state",
    ] {
        families.push((format!("petri.run.{field}"), MonotonicSum, vec![reader()]));
    }
    for verb in ["applied", "failed"] {
        families.push((
            format!("petri.run.eat_actions_{verb}_by_type"),
            MonotonicSum,
            vec![food()],
        ));
    }
    families.push((
        "petri.run.move_actions_blocked_by_cause".to_owned(),
        MonotonicSum,
        vec![(
            "petri.cause",
            strings(["barrier", "occupied", "out_of_bounds"], str::to_owned),
        )],
    ));
    families.push((
        "petri.run.predation_actions_by_result".to_owned(),
        MonotonicSum,
        vec![(
            "petri.result",
            strings(
                ["Transferred", "TransferredAndKilled", "RejectedNoVictim"],
                str::to_owned,
            ),
        )],
    ));
    families.push((
        "petri.tick.actions".to_owned(),
        Gauge,
        vec![(
            "petri.action",
            strings(["move", "eat", "noop", "reproduce", "steal"], str::to_owned),
        )],
    ));
}

fn tick_families(families: &mut Vec<Family>) {
    use MetricKind::Gauge;
    for field in [
        "predation_kills",
        "compute_total_mean",
        "compute_total_min",
        "compute_total_max",
        "compute_vm_mean",
        "compute_graph_mean",
        "priority_bid_mean",
        "priority_bidders_count",
        "food_occupancy_depletion_mean",
        "food_occupancy_depletion_occupied_cells",
        "food_growth_suppressed_by_occupancy_depletion",
        "food_cells_with_type_inhibition",
        "food_growth_suppressed_by_type_inhibition",
        "population",
        "mean_energy",
        "mean_genome_size",
        "mean_mesh_nodes",
        "mean_generation",
    ] {
        families.push((format!("petri.tick.{field}"), Gauge, vec![]));
    }
    for field in [
        "food_total_density",
        "food_grazing_modifier_mean",
        "food_grazed_cell_share",
    ] {
        families.push((format!("petri.tick.{field}"), Gauge, vec![food()]));
    }
}

/// Every combination of the attributes' values.
fn label_sets(attributes: &[(&'static str, Vec<String>)]) -> BTreeSet<BTreeMap<String, String>> {
    let mut sets = BTreeSet::from([BTreeMap::new()]);
    for (key, values) in attributes {
        sets = sets
            .into_iter()
            .flat_map(|set| {
                values.iter().map(move |value| {
                    let mut set = set.clone();
                    set.insert((*key).to_owned(), value.clone());
                    set
                })
            })
            .collect();
    }
    sets
}

fn encoded(stats: &SimStats) -> ReceivedSnapshot {
    let resource = [string_value("service.name", "v3-cli".to_owned())];
    let request = encode(
        &resource,
        "0123456789abcdef0123456789abcdef",
        stats,
        &Census::default(),
        Moment {
            tick: 42,
            elapsed: Duration::from_millis(2_500),
            time_ns: 2_000,
            start_ns: 1_000,
        },
    );
    let mut snapshots = decode_metrics(&request.encode_to_vec());
    assert_eq!(snapshots.len(), 1);
    snapshots.remove(0)
}

#[test]
fn every_family_is_encoded_with_its_name_kind_attributes_and_keys_and_nothing_else() {
    let snapshot = encoded(&populated());
    let mut received: BTreeMap<String, (MetricKind, BTreeSet<BTreeMap<String, String>>)> =
        BTreeMap::new();
    for point in &snapshot.points {
        assert_eq!(
            point.attribute("petri.run_id"),
            Some("0123456789abcdef0123456789abcdef")
        );
        assert_eq!(point.time_unix_nano, 2_000);
        let mut labels = point.attributes.clone();
        labels.remove("petri.run_id");
        let entry = received
            .entry(point.name.clone())
            .or_insert_with(|| (point.kind, BTreeSet::new()));
        assert_eq!(entry.0, point.kind, "{}", point.name);
        assert!(entry.1.insert(labels), "duplicate series in {}", point.name);
    }
    let expected: BTreeMap<String, (MetricKind, BTreeSet<BTreeMap<String, String>>)> = expected()
        .into_iter()
        .map(|(name, kind, attributes)| (name, (kind, label_sets(&attributes))))
        .collect();
    let received_names: BTreeSet<&String> = received.keys().collect();
    let expected_names: BTreeSet<&String> = expected.keys().collect();
    assert_eq!(received_names, expected_names);
    for (name, family) in &expected {
        assert_eq!(&received[name], family, "{name}");
    }
}

#[test]
fn encoded_values_units_and_descriptions_follow_the_contract() {
    let snapshot = encoded(&populated());
    let value = |name: &str, labels: &[(&str, &str)]| snapshot.value(name, labels).unwrap();
    assert_eq!(value("petri.run.tick", &[]), PointValue::Int(42));
    assert_eq!(
        value("petri.run.elapsed_seconds", &[]),
        PointValue::Double(2.5)
    );
    assert_eq!(
        value("petri.run.deaths", &[]),
        PointValue::Int(DeathCause::ALL.len() as i64)
    );
    assert_eq!(
        value(
            "petri.run.energy_flow",
            &[("petri.flow", "action_charge.move")]
        ),
        PointValue::Double(-3.25)
    );
    assert_eq!(
        value("petri.run.energy_flow", &[("petri.flow", "food_intake.1")]),
        PointValue::Double(2.5)
    );
    assert_eq!(
        value(
            "petri.run.phase_wall_clock",
            &[("petri.phase", "cognition")]
        ),
        PointValue::Double(1.5)
    );
    // Both food types' `FoodHere` keys share one `as_key()` value and sum.
    assert_eq!(
        value(
            "petri.run.mutation_added_node_world_inputs_by_operator",
            &[
                ("petri.operator", "Topology.AddNode"),
                ("petri.world_input", "FoodHere")
            ]
        ),
        PointValue::Int(2)
    );
    assert_eq!(
        value(
            "petri.run.mutation_added_node_world_inputs_by_operator",
            &[
                ("petri.operator", "Topology.AddNode"),
                ("petri.world_input", "NeighborBarrierRing")
            ]
        ),
        PointValue::Int(1)
    );
    assert_eq!(
        value("petri.tick.food_total_density", &[("petri.food_type", "1")]),
        PointValue::Double(0.25)
    );
    for point in &snapshot.points {
        let unit = if point.name == "petri.run.phase_wall_clock" {
            "s"
        } else {
            ""
        };
        assert_eq!(point.unit, unit, "{}", point.name);
        let sampled = point.name.starts_with("petri.tick.");
        let description = if sampled { SAMPLED } else { "" };
        assert_eq!(point.description, description, "{}", point.name);
        let start = if point.kind == MetricKind::Gauge {
            0
        } else {
            1_000
        };
        assert_eq!(point.start_time_unix_nano, start, "{}", point.name);
    }
}

#[test]
fn an_empty_map_exports_no_family_and_a_default_snapshot_is_bounded() {
    let snapshot = encoded(&SimStats::default());
    assert_eq!(
        snapshot
            .family("petri.run.mutation_operator_funnel")
            .count(),
        0
    );
    assert_eq!(snapshot.family("petri.tick.food_total_density").count(), 0);
    assert_eq!(
        snapshot.family("petri.run.deaths_by_cause").count(),
        DeathCause::ALL.len()
    );
}
