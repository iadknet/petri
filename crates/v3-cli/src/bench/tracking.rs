//! Per-tick observers and persistence sampling.

use super::indicators::clade_diversity;
use super::profiles::SAMPLE_EVERY_TICKS;
use super::schema::{
    ratio, ByReaderState, MovesBlockedByCause, MutationSupply, PopulationPersistenceSeed,
    TrackedFractions,
};
use crate::{fraction_or_undefined, six};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use v3_core::contracts::WorldInputKey;
use v3_core::creature::action_log::{ActionType, ACTION_TYPE_COUNT};
use v3_core::creature::sensor_census::{
    creature_sensor_census, world_input_key_label, world_input_key_universe,
};
use v3_core::kernel::occupancy_grid::{occupancy_grid, OCCUPANCY_CELLS_PER_AXIS};

/// The applied-behavior integer sums of
/// `v3_core::simulation::stats::MutationValueTotals`: what the carriers of an
/// applied birth mutation did while they lived. The score sums are the
/// composite the observation contract leaves out, the six classification
/// counters are bucketings of `viability_score` and go with it, and
/// `final_energy_sum` is zero on every benchmark path. The three value
/// counters (T11.F22) classify each carrier's outcome so a per-operator
/// helpful share, helpful / (helpful + detrimental), is readable from the
/// summary; they are optional so a summary written before them reads as
/// absent, never as zero.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MutationOutcomeTotals {
    pub carriers_observed_total: u64,
    pub survival_ticks_sum: u64,
    pub offspring_spawned_sum: u64,
    pub survived_short_horizon_total: u64,
    pub survived_long_horizon_total: u64,
    pub reproduced_once_total: u64,
    pub action_attempted_total: u64,
    pub blocked_move_total: u64,
    pub invalid_reproduce_total: u64,
    pub invalid_action_total: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub helpful_total: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub neutral_total: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detrimental_total: Option<u64>,
}

impl From<&v3_core::simulation::stats::MutationValueTotals> for MutationOutcomeTotals {
    fn from(totals: &v3_core::simulation::stats::MutationValueTotals) -> Self {
        Self {
            carriers_observed_total: totals.carriers_observed_total,
            survival_ticks_sum: totals.survival_ticks_sum,
            offspring_spawned_sum: totals.offspring_spawned_sum,
            survived_short_horizon_total: totals.survived_short_horizon_total,
            survived_long_horizon_total: totals.survived_long_horizon_total,
            reproduced_once_total: totals.reproduced_once_total,
            action_attempted_total: totals.action_attempted_total,
            blocked_move_total: totals.blocked_move_total,
            invalid_reproduce_total: totals.invalid_reproduce_total,
            invalid_action_total: totals.invalid_action_total,
            helpful_total: Some(totals.helpful_total),
            neutral_total: Some(totals.neutral_total),
            detrimental_total: Some(totals.detrimental_total),
        }
    }
}

/// What the population's predation attempts did (T14.F02), with the per-result
/// breakdown keyed by `PredationActionResult::as_key` so its order is fixed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PredationTracking {
    pub actions_attempted_total: u64,
    pub actions_transferred_total: u64,
    pub actions_rejected_total: u64,
    pub kills_total: u64,
    pub actions_by_result: BTreeMap<String, u64>,
}

/// Every successful removal, partitioned by the first applied exhausting sink.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MortalityTracking {
    pub definition: String,
    pub deaths_total: u64,
    pub by_cause: BTreeMap<String, u64>,
}

/// Correlated lifetime outcomes of removed creatures carrying each structure class.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReproductiveSuccessByCognitiveClassTracking {
    pub definition: String,
    pub by_class: BTreeMap<String, ReproductiveSuccessTotalsTracking>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReproductiveSuccessTotalsTracking {
    pub creatures_observed_total: u64,
    pub offspring_spawned_sum: u64,
    pub survival_ticks_sum: u64,
}

impl From<v3_core::simulation::reproductive_success::ReproductiveSuccessTotals>
    for ReproductiveSuccessTotalsTracking
{
    fn from(totals: v3_core::simulation::reproductive_success::ReproductiveSuccessTotals) -> Self {
        Self {
            creatures_observed_total: totals.creatures_observed_total,
            offspring_spawned_sum: totals.offspring_spawned_sum,
            survival_ticks_sum: totals.survival_ticks_sum,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionChargeTracking {
    pub noop: String,
    pub eat: String,
    pub r#move: String,
    pub reproduce: String,
    pub steal_energy: String,
}

/// Applied signed energy changes, rounded only at the report boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnergyFlowTracking {
    pub definition: String,
    pub food_intake_by_type: Vec<String>,
    pub action_charges: ActionChargeTracking,
    pub failed_action_penalty: String,
    pub vm_compute: String,
    pub priority_bid: String,
    pub graph_compute: String,
    pub hebbian_learning: String,
    pub reward_learning: String,
    pub lifecycle_decay: String,
    pub genome_carrying: String,
    pub genome_size_creature_ticks: u64,
    pub parental_transfer_debit: String,
    pub offspring_energy_credit: String,
    pub predation_victim_debit: String,
    pub predation_attacker_credit: String,
    pub predation_kill_bonus_credit: String,
    pub maximum_energy_clamp_loss: String,
    pub zero_floor_credit: String,
    pub external_removal_loss: String,
}

impl From<&v3_core::simulation::energy_accounting::EnergyFlows> for EnergyFlowTracking {
    fn from(flows: &v3_core::simulation::energy_accounting::EnergyFlows) -> Self {
        Self {
            definition: "applied-energy-flows-v1".into(),
            food_intake_by_type: flows.food_intake_by_type.iter().copied().map(six).collect(),
            action_charges: ActionChargeTracking {
                noop: six(flows.action_charges.noop),
                eat: six(flows.action_charges.eat),
                r#move: six(flows.action_charges.r#move),
                reproduce: six(flows.action_charges.reproduce),
                steal_energy: six(flows.action_charges.steal_energy),
            },
            failed_action_penalty: six(flows.failed_action_penalty),
            vm_compute: six(flows.vm_compute),
            priority_bid: six(flows.priority_bid),
            graph_compute: six(flows.graph_compute),
            hebbian_learning: six(flows.hebbian_learning),
            reward_learning: six(flows.reward_learning),
            lifecycle_decay: six(flows.lifecycle_decay),
            genome_carrying: six(flows.genome_carrying),
            genome_size_creature_ticks: flows.genome_size_creature_ticks,
            parental_transfer_debit: six(flows.parental_transfer_debit),
            offspring_energy_credit: six(flows.offspring_energy_credit),
            predation_victim_debit: six(flows.predation_victim_debit),
            predation_attacker_credit: six(flows.predation_attacker_credit),
            predation_kill_bonus_credit: six(flows.predation_kill_bonus_credit),
            maximum_energy_clamp_loss: six(flows.maximum_energy_clamp_loss),
            zero_floor_credit: six(flows.zero_floor_credit),
            external_removal_loss: six(flows.external_removal_loss),
        }
    }
}

/// Cumulative per-world behavior a baseline world is followed by (T12.F04):
/// what the population ate, how much food stood, and how often it walked into
/// something. Every field comes from applied simulation behavior, and every one
/// is serde-defaulted so reports stored before T12.F04 still parse.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldTracking {
    /// Applied Eat actions by the food type they consumed.
    #[serde(default)]
    pub typed_eats_total: Vec<u64>,
    /// Standing density per food type after the last executed tick's growth.
    #[serde(default)]
    pub food_density_total: Vec<String>,
    /// Mean grazing fertility modifier per food type over passable cells
    /// after the last executed tick's recovery step (T02.F04); 1.0 for a type
    /// nothing grazed and everywhere while grazing is disabled. Empty in
    /// reports stored before T02.F04.
    #[serde(default)]
    pub grazing_modifier_mean: Vec<String>,
    /// Share of passable cells whose grazing modifier is below 1.0 per food
    /// type, read at the same tick. Empty in reports stored before T02.F04.
    #[serde(default)]
    pub grazed_cell_share: Vec<String>,
    /// Move actions executed, blocked or not.
    #[serde(default)]
    pub moves_attempted_total: u64,
    /// Move actions a barrier blocked. The `barrier` entry of
    /// [`WorldTracking::moves_blocked_total_by_cause`], kept under its own key
    /// because reports stored before that map existed carry only this one.
    #[serde(default)]
    pub moves_blocked_barrier_total: u64,
    /// Every blocked move by what blocked it, straight from
    /// `SimStats::move_actions_blocked_total_by_cause`: barriers, an occupied
    /// target, and the world edge. Their sum is every move the population
    /// attempted and did not make. Absent — not zeroed — in a report stored
    /// before these totals were carried.
    #[serde(default)]
    pub moves_blocked_total_by_cause: Option<MovesBlockedByCause>,
    /// Blocked moves of any cause — barrier, occupancy, or edge — that had a
    /// valid alternative target, by reader state. Counted over every move the
    /// population made, not only the ones made beside a barrier.
    #[serde(default)]
    pub moves_blocked_avoidable_by_reader_state: ByReaderState<u64>,
    /// Move attempts made from a cell that had at least one neighboring
    /// barrier, by reader state: the only moves at which reading the barrier
    /// ring could have changed anything, and the denominator of
    /// [`TrackedFractions::barrier_blocked_fraction_by_reader_state`].
    #[serde(default)]
    pub move_attempts_with_barrier_neighbor_by_reader_state: ByReaderState<u64>,
    /// Of those attempts, the ones a barrier blocked: the numerator of the
    /// same fraction.
    #[serde(default)]
    pub moves_blocked_barrier_with_barrier_neighbor_by_reader_state: ByReaderState<u64>,
    /// Eat actions that found no food, indexed like
    /// [`WorldTracking::typed_eats_total`] by the food type the action named.
    /// Absent — not zeroed — in a report stored before T14.F02, and absent
    /// from every checkpoint sample, which carries only the fields above.
    /// The same holds for each transferred block that follows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub typed_eats_failed_total: Option<Vec<u64>>,
    /// Mesh dispatches that stopped because the creature ran out of energy
    /// mid-chain.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mesh_dispatches_energy_exhausted_total: Option<u64>,
    /// The mutation supply this case produced and where its events aimed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mutation_supply: Option<MutationSupply>,
    /// What the carriers of an applied birth mutation did while they lived,
    /// pooled across every operator.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mutation_outcome_summary: Option<MutationOutcomeTotals>,
    /// The same lifetime totals split by the operator that produced them,
    /// keyed by `MutationOperator::as_key`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mutation_value_totals_by_operator: Option<BTreeMap<String, MutationOutcomeTotals>>,
    /// What the population's predation attempts did.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub predation: Option<PredationTracking>,
    /// Terminal-only applied death attribution; absent means unmeasured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mortality: Option<MortalityTracking>,
    /// Terminal-only removed-creature cohorts; absence is unmeasured, zeros are empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reproductive_success_by_cognitive_class:
        Option<ReproductiveSuccessByCognitiveClassTracking>,
    /// Terminal-only applied energy flows; absent means unmeasured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub energy_flows: Option<EnergyFlowTracking>,
    /// Terminal-only applied learning and changed-memory-write events.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cognition: Option<CognitionTracking>,
    /// Terminal-only per-clade profile of the living population (T14.F07);
    /// absent means unmeasured, an empty `rows` means extinction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surviving_clade_profiles: Option<SurvivingCladeProfiles>,
}

/// How each founder clade with at least one living creature made its living,
/// one row per surviving `lineage_id` in ascending order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurvivingCladeProfiles {
    pub definition: String,
    pub rows: Vec<SurvivingCladeProfileRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurvivingCladeProfileRow {
    pub lineage_id: u32,
    pub size: u64,
    pub mean_energy: String,
    pub mean_age: String,
    pub mean_generation: String,
    pub mean_genome_size: String,
    /// Applied eats indexed by food type like
    /// [`WorldTracking::typed_eats_total`].
    pub eats_by_type: Vec<u64>,
    /// Attempts keyed by `ActionType::as_key`, all five keys always present.
    pub actions_by_type: BTreeMap<String, u64>,
    pub predation_kills: u64,
    pub predation_hits_taken: u64,
}

/// One living creature's contribution to its clade's row: its lineage and
/// the lifetime counters `CreatureState` carries.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct CladeMemberReading {
    pub(super) lineage_id: u32,
    pub(super) energy: f32,
    pub(super) age: u64,
    pub(super) generation: u64,
    pub(super) genome_size: u32,
    pub(super) eats_by_type: Vec<u64>,
    pub(super) actions_by_type: [u64; ACTION_TYPE_COUNT as usize],
    pub(super) predation_kills: u64,
    pub(super) predation_hits_taken: u64,
}

impl CladeMemberReading {
    fn observe(creature: &v3_core::creature::state::CreatureState) -> Self {
        Self {
            lineage_id: creature.identity.lineage_id,
            energy: creature.energy,
            age: creature.age,
            generation: creature.generation,
            genome_size: creature.cached_genome_size,
            eats_by_type: creature.lifetime_eats_applied_by_type.clone(),
            actions_by_type: creature.lifetime_actions_attempted_by_type,
            predation_kills: creature.lifetime_predation_kills_count,
            predation_hits_taken: creature.lifetime_predation_hits_taken_count,
        }
    }
}

#[derive(Default)]
struct CladeAccumulator {
    size: u64,
    energy_sum: f64,
    age_sum: u64,
    generation_sum: u64,
    genome_size_sum: u64,
    eats_by_type: Vec<u64>,
    actions_by_type: [u64; ACTION_TYPE_COUNT as usize],
    predation_kills: u64,
    predation_hits_taken: u64,
}

/// Bucket living creatures by `lineage_id` into rows ascending by lineage.
/// Pure over its input: sizes sum to the member count, every integer column
/// is the sum over the row's members, and `eats_by_type` is exactly
/// `food_type_count` long (a member's lazily grown vector may be shorter, and
/// a type index past the configured types is not a food type of this run).
/// Energy is summed in `f64` in input order, so feeding `creatures.values()`
/// keeps the order the checkpoint `mean_energy` uses.
pub(super) fn bucket_surviving_clades(
    food_type_count: usize,
    members: impl IntoIterator<Item = CladeMemberReading>,
) -> Vec<SurvivingCladeProfileRow> {
    let mut clades: BTreeMap<u32, CladeAccumulator> = BTreeMap::new();
    for member in members {
        let clade = clades
            .entry(member.lineage_id)
            .or_insert_with(|| CladeAccumulator {
                eats_by_type: vec![0; food_type_count],
                ..CladeAccumulator::default()
            });
        clade.size += 1;
        clade.energy_sum += f64::from(member.energy);
        clade.age_sum += member.age;
        clade.generation_sum += member.generation;
        clade.genome_size_sum += u64::from(member.genome_size);
        for (total, eaten) in clade.eats_by_type.iter_mut().zip(&member.eats_by_type) {
            *total += eaten;
        }
        for (total, attempted) in clade.actions_by_type.iter_mut().zip(member.actions_by_type) {
            *total += attempted;
        }
        clade.predation_kills += member.predation_kills;
        clade.predation_hits_taken += member.predation_hits_taken;
    }
    clades
        .into_iter()
        .map(|(lineage_id, clade)| {
            let size = clade.size as f64;
            SurvivingCladeProfileRow {
                lineage_id,
                size: clade.size,
                mean_energy: six(clade.energy_sum / size),
                mean_age: six(clade.age_sum as f64 / size),
                mean_generation: six(clade.generation_sum as f64 / size),
                mean_genome_size: six(clade.genome_size_sum as f64 / size),
                eats_by_type: clade.eats_by_type,
                actions_by_type: ActionType::ALL
                    .into_iter()
                    .zip(clade.actions_by_type)
                    .map(|(action, count)| (action.as_key().to_string(), count))
                    .collect(),
                predation_kills: clade.predation_kills,
                predation_hits_taken: clade.predation_hits_taken,
            }
        })
        .collect()
}

/// Applied assignments and changes, without claiming useful learning or memory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CognitionTracking {
    pub plasticity_updates_total: u64,
    pub plasticity_changes_total: u64,
    pub hebbian_updates_total: u64,
    pub hebbian_changes_total: u64,
    pub reward_modulated_updates_total: u64,
    pub reward_modulated_changes_total: u64,
    pub shared_memory_writes_changed_total: u64,
}

/// Counts keyed by ordinary food type, as a `Vec` indexed by type id: a dense
/// order fixed by the configured food types, so no `HashMap` iteration order
/// reaches the report.
fn by_food_type(
    sim: &v3_core::simulation::Simulation,
    counts: &std::collections::HashMap<v3_core::config::OrdinaryFoodTypeId, u64>,
) -> Vec<u64> {
    use v3_core::config::OrdinaryFoodTypeId;
    (0..sim.config.world.food.types.len())
        .map(|index| {
            counts
                .get(&OrdinaryFoodTypeId::new(index as u16))
                .copied()
                .unwrap_or(0)
        })
        .collect()
}

impl WorldTracking {
    /// Read the cumulative per-world counters out of a running simulation.
    /// The T14.F02 transferred blocks are left absent here; only the
    /// end-of-run per-case block carries them, via
    /// [`WorldTracking::with_transferred_counters`].
    pub(super) fn observe(sim: &v3_core::simulation::Simulation) -> Self {
        use std::collections::HashMap;
        use v3_core::simulation::actions::{BarrierReaderState, MoveBlockedCause};
        let stats = &sim.stats;
        let by_reader_state = |counts: &HashMap<BarrierReaderState, u64>| {
            let count = |state: BarrierReaderState| counts.get(&state).copied().unwrap_or_default();
            ByReaderState {
                has_barrier_reader: count(BarrierReaderState::HasBarrierReader),
                no_barrier_reader: count(BarrierReaderState::NoBarrierReader),
            }
        };
        let blocked = |cause: MoveBlockedCause| {
            stats
                .move_actions_blocked_total_by_cause
                .get(&cause)
                .copied()
                .unwrap_or_default()
        };
        let by_cause = MovesBlockedByCause {
            barrier: blocked(MoveBlockedCause::Barrier),
            occupied: blocked(MoveBlockedCause::Occupied),
            out_of_bounds: blocked(MoveBlockedCause::OutOfBounds),
        };
        Self {
            typed_eats_total: by_food_type(sim, &stats.eat_actions_applied_total_by_type),
            food_density_total: stats
                .last_tick_food_total_density_by_type
                .iter()
                .map(|density| six(f64::from(*density)))
                .collect(),
            grazing_modifier_mean: stats
                .last_tick_food_grazing_modifier_mean_by_type
                .iter()
                .map(|modifier| six(f64::from(*modifier)))
                .collect(),
            grazed_cell_share: stats
                .last_tick_food_grazed_cell_share_by_type
                .iter()
                .map(|share| six(f64::from(*share)))
                .collect(),
            moves_attempted_total: stats.move_actions_attempted_total,
            moves_blocked_barrier_total: by_cause.barrier,
            moves_blocked_total_by_cause: Some(by_cause),
            moves_blocked_avoidable_by_reader_state: by_reader_state(
                &stats.move_actions_blocked_avoidable_total_by_reader_state,
            ),
            move_attempts_with_barrier_neighbor_by_reader_state: by_reader_state(
                &stats.move_attempts_with_barrier_neighbor_total_by_reader_state,
            ),
            moves_blocked_barrier_with_barrier_neighbor_by_reader_state: by_reader_state(
                &stats.move_blocked_barrier_with_barrier_neighbor_total_by_reader_state,
            ),
            typed_eats_failed_total: None,
            mesh_dispatches_energy_exhausted_total: None,
            mutation_supply: None,
            mutation_outcome_summary: None,
            mutation_value_totals_by_operator: None,
            predation: None,
            mortality: None,
            reproductive_success_by_cognitive_class: None,
            energy_flows: None,
            cognition: None,
            surviving_clade_profiles: None,
        }
    }

    /// Add the T14.F02 transferred counters, which only the end-of-run
    /// per-case block carries. Checkpoint samples stay at the shape they had
    /// before T14.F02, so a stored report keeps one copy of these totals per
    /// case rather than one per sampled tick.
    pub(super) fn with_transferred_counters(self, sim: &v3_core::simulation::Simulation) -> Self {
        let stats = &sim.stats;
        Self {
            cognition: Some(CognitionTracking {
                plasticity_updates_total: stats.plasticity_updates_total,
                plasticity_changes_total: stats.plasticity_changes_total,
                hebbian_updates_total: stats.hebbian_updates_total,
                hebbian_changes_total: stats.hebbian_changes_total,
                reward_modulated_updates_total: stats.reward_modulated_updates_total,
                reward_modulated_changes_total: stats.reward_modulated_changes_total,
                shared_memory_writes_changed_total: stats.shared_memory_writes_changed_total,
            }),
            mortality: Some(MortalityTracking {
                definition: "applied-mortality-v1".into(),
                deaths_total: stats.mortality.deaths_total,
                by_cause: v3_core::simulation::energy_accounting::DeathCause::ALL
                    .into_iter()
                    .map(|cause| (cause.as_key().to_string(), stats.mortality.count(cause)))
                    .collect(),
            }),
            reproductive_success_by_cognitive_class: Some(
                ReproductiveSuccessByCognitiveClassTracking {
                    definition: "reproductive-success-by-cognitive-class-v1".into(),
                    by_class: v3_core::simulation::reproductive_success::CognitiveClass::ALL
                        .into_iter()
                        .map(|class| {
                            (
                                class.as_key().to_string(),
                                stats
                                    .reproductive_success_by_cognitive_class
                                    .totals(class)
                                    .into(),
                            )
                        })
                        .collect(),
                },
            ),
            energy_flows: Some((&stats.energy_flows).into()),
            typed_eats_failed_total: Some(by_food_type(
                sim,
                &stats.eat_actions_failed_total_by_type,
            )),
            mesh_dispatches_energy_exhausted_total: Some(
                stats.mesh_dispatches_energy_exhausted_total,
            ),
            mutation_supply: Some(MutationSupply {
                events_attempted_total: stats.mutation_events_attempted_total,
                events_applied_total: stats.mutation_events_applied_total,
                events_skipped_total: stats.mutation_events_skipped_total,
                executed_target_total: stats.mutation_executed_target_total,
                reachable_target_total: stats.mutation_reachable_target_total,
                unreachable_target_total: stats.mutation_unreachable_target_total,
                not_applicable_target_total: stats.mutation_not_applicable_target_total,
            }),
            mutation_outcome_summary: Some((&stats.mutation_outcome_summary).into()),
            mutation_value_totals_by_operator: Some(
                stats
                    .mutation_value_totals_by_operator
                    .iter()
                    .map(|(operator, totals)| (operator.as_key().to_string(), totals.into()))
                    .collect(),
            ),
            predation: Some(PredationTracking {
                actions_attempted_total: stats.predation_actions_attempted_total,
                actions_transferred_total: stats.predation_actions_transferred_total,
                actions_rejected_total: stats.predation_actions_rejected_total,
                kills_total: stats.predation_kills_total,
                actions_by_result: stats
                    .predation_actions_by_result
                    .iter()
                    .map(|(result, count)| (result.as_key().to_string(), *count))
                    .collect(),
            }),
            surviving_clade_profiles: Some(SurvivingCladeProfiles {
                definition: "surviving-clade-profile-v1".into(),
                rows: bucket_surviving_clades(
                    sim.config.world.food.types.len(),
                    sim.creatures.values().map(CladeMemberReading::observe),
                ),
            }),
            ..self
        }
    }

    /// The rates these totals imply, each against its own denominator: a
    /// type's eat share against every applied Eat, `blocked_move_fraction` and
    /// the avoidable share against every move attempted, and the
    /// barrier-blocked fraction against that reader state's own attempts made
    /// beside a barrier. Only the last is a per-state rate.
    pub(super) fn fractions(&self) -> TrackedFractions {
        let eats: u64 = self.typed_eats_total.iter().sum();
        let attempted = self.moves_attempted_total;
        let avoidable = &self.moves_blocked_avoidable_by_reader_state;
        let beside = &self.move_attempts_with_barrier_neighbor_by_reader_state;
        let blocked_beside = &self.moves_blocked_barrier_with_barrier_neighbor_by_reader_state;
        TrackedFractions {
            typed_eat_share: self
                .typed_eats_total
                .iter()
                .map(|typed| fraction_or_undefined(*typed, eats))
                .collect(),
            blocked_move_fraction: fraction_or_undefined(
                self.moves_blocked_barrier_total,
                attempted,
            ),
            barrier_blocked_fraction_by_reader_state: ByReaderState {
                has_barrier_reader: fraction_or_undefined(
                    blocked_beside.has_barrier_reader,
                    beside.has_barrier_reader,
                ),
                no_barrier_reader: fraction_or_undefined(
                    blocked_beside.no_barrier_reader,
                    beside.no_barrier_reader,
                ),
            },
            avoidable_blocked_share_of_all_moves_by_reader_state: ByReaderState {
                has_barrier_reader: fraction_or_undefined(avoidable.has_barrier_reader, attempted),
                no_barrier_reader: fraction_or_undefined(avoidable.no_barrier_reader, attempted),
            },
        }
    }
}

/// One persistence sample, taken after `run_tick` on every executed tick that
/// is a multiple of [`SAMPLE_EVERY_TICKS`] and on the last executed tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceSample {
    pub tick: u64,
    pub population: u64,
    pub mean_energy: Option<String>,
    /// Cumulative `reproduction_actions_spawned_total` at this tick.
    pub births_total: u64,
    /// Mean genome size of the living population, junk included. `null` at
    /// extinction and absent in reports stored before T14.F04.
    #[serde(default)]
    pub mean_genome_size: Option<String>,
    /// Mean mesh node count of the living population. `null` at extinction and
    /// absent in reports stored before T14.F04.
    #[serde(default)]
    pub mean_mesh_nodes: Option<String>,
    /// Mean generation of the living population. `null` at extinction and
    /// absent in reports stored before T14.F04.
    #[serde(default)]
    pub mean_generation: Option<String>,
    /// Founder clades with at least one living creature — a true `0` at
    /// extinction, absent in reports stored before T14.F04.
    #[serde(default)]
    pub surviving_founder_clade_count: Option<u64>,
    /// Shannon entropy of the living clade distribution, in nats. `UNDEFINED`
    /// at extinction and absent in reports stored before T14.F04.
    #[serde(default)]
    pub shannon_entropy_nats: Option<String>,
    /// What the living population can perceive and remember at this tick. A
    /// true all-zero census at extinction, absent in reports stored before
    /// T14.F08.
    #[serde(default)]
    pub sensor_census: Option<SensorCensus>,
    /// Where the living population stands, on a fixed 16x16 grid over the
    /// world. A true all-zero grid at extinction, absent in reports stored
    /// before T14.F10.
    #[serde(default)]
    pub occupancy_grid: Option<OccupancyGrid>,
    #[serde(flatten)]
    pub tracking: WorldTracking,
}

/// The occupancy grid of one checkpoint: two parallel row-major arrays of
/// length `cells_x * cells_y`, always emitted in full.
///
/// An unoccupied cell is a `0` in both arrays, never an absence, so a reader
/// never reconstructs the grid shape from which cells appear.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OccupancyGrid {
    pub cells_x: u16,
    pub cells_y: u16,
    /// Living creatures per cell, row-major.
    pub population: Vec<u64>,
    /// Distinct founder clades per cell, row-major. A clade counts once per
    /// cell however many of its creatures stand there.
    pub distinct_clades: Vec<u64>,
}

impl OccupancyGrid {
    /// Aggregate the living population's positions from post-tick state. Pure
    /// integer counting over an ordered structure: no RNG is consumed, and the
    /// creature iteration order does not reach the result.
    pub(super) fn observe(sim: &v3_core::simulation::Simulation) -> Self {
        let grid = occupancy_grid(
            sim.world.width,
            sim.world.height,
            sim.creatures
                .values()
                .map(|creature| (creature.position, creature.identity.lineage_id)),
        );
        Self {
            cells_x: OCCUPANCY_CELLS_PER_AXIS,
            cells_y: OCCUPANCY_CELLS_PER_AXIS,
            population: grid.population.to_vec(),
            distinct_clades: grid.distinct_clades.to_vec(),
        }
    }
}

/// One world input key's row in a checkpoint census.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldInputCensusRow {
    /// `WorldInputKey::as_key()`, with `:<food_type_idx>` appended for the
    /// food-parameterized families.
    pub key: String,
    /// Living creatures holding at least one live reference to this key. A
    /// creature counts once however many instructions or edges reference it.
    pub creatures: u64,
}

/// The sensor usage census of one checkpoint: living creatures per world input
/// key, and the three stateful-reach counts.
///
/// Every key the run's world can present has a row, so a `0` reads as "no
/// living creature references this" rather than as an absent key. Rows are in
/// `WorldInputKey` order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SensorCensus {
    pub world_inputs: Vec<WorldInputCensusRow>,
    /// Living creatures a live reference of which reads a shared-memory slot.
    /// `shared_memory` persists across ticks, so a same-tick read counts.
    pub creatures_reading_shared_memory: u64,
    /// Living creatures holding a live compute node with persisted state.
    pub creatures_with_stateful_node: u64,
    /// Living creatures reading either stateful source.
    pub creatures_with_any_stateful_read: u64,
}

impl SensorCensus {
    /// Census the living population from post-tick state. Pure structure: it
    /// reads genomes and cached reachability and executes no brain.
    pub(super) fn observe(sim: &v3_core::simulation::Simulation) -> Self {
        let mut world_inputs: BTreeMap<WorldInputKey, u64> = world_input_key_universe(
            sim.world
                .food()
                .food_types()
                .iter()
                .map(|food_type| food_type.id),
        )
        .into_iter()
        .map(|key| (key, 0))
        .collect();
        let mut creatures_reading_shared_memory = 0;
        let mut creatures_with_stateful_node = 0;
        let mut creatures_with_any_stateful_read = 0;

        for creature in sim.creatures.values() {
            let census = creature_sensor_census(&creature.genome, &creature.cached_reachable_nodes);
            creatures_reading_shared_memory += u64::from(census.reads_shared_memory);
            creatures_with_stateful_node += u64::from(census.holds_stateful_node);
            creatures_with_any_stateful_read += u64::from(census.reads_any_stateful());
            for key in census.world_inputs {
                // The row set is the same at every checkpoint: mutation draws
                // food type ids from `config.world.food.types.len()`, clamped
                // to at least one (`mutation::sampling::sample_food_type_id`),
                // and the universe above is the runtime catalog — that same
                // list, or the one synthesized default when the config carries
                // none. So no key can land outside the declared universe.
                *world_inputs.entry(key).or_insert(0) += 1;
            }
        }

        Self {
            world_inputs: world_inputs
                .into_iter()
                .map(|(key, creatures)| WorldInputCensusRow {
                    key: world_input_key_label(key),
                    creatures,
                })
                .collect(),
            creatures_reading_shared_memory,
            creatures_with_stateful_node,
            creatures_with_any_stateful_read,
        }
    }
}

/// The population readings a checkpoint sample carries, read from post-tick
/// state through the same accessors the horizon readings use.
#[derive(Debug, PartialEq)]
pub(super) struct PopulationReadings {
    pub(super) mean_genome_size: f64,
    pub(super) mean_mesh_nodes: f64,
    pub(super) mean_generation: f64,
    pub(super) surviving_founder_clade_count: u64,
    pub(super) shannon_entropy_nats: String,
    sensor_census: SensorCensus,
    occupancy_grid: OccupancyGrid,
}

impl PopulationReadings {
    pub(super) fn observe(sim: &v3_core::simulation::Simulation) -> Self {
        let (mean_genome_size, mean_mesh_nodes, mean_generation) = crate::structure_means(sim);
        let (surviving_founder_clade_count, shannon_entropy_nats) =
            clade_diversity(sim.creatures.values().map(|c| c.identity.lineage_id));
        Self {
            mean_genome_size,
            mean_mesh_nodes,
            mean_generation,
            surviving_founder_clade_count,
            shannon_entropy_nats,
            sensor_census: SensorCensus::observe(sim),
            occupancy_grid: OccupancyGrid::observe(sim),
        }
    }
}

/// Accumulates the per-seed persistence observations defined by the T01.F11
/// spec. It never touches the simulation, so it is unit-testable from
/// synthetic observations alone.
#[derive(Debug)]
pub(super) struct PersistenceAccumulator {
    horizon: u64,
    population: u64,
    minimum_population: u64,
    peak_population: u64,
    peak_tick: u64,
    extinction_tick: Option<u64>,
    plateau_sum: u64,
    plateau_ticks: u64,
    samples: Vec<PersistenceSample>,
}

impl PersistenceAccumulator {
    /// `seeded_population` is the founder count observed at tick 0, before
    /// any tick has run: the first peak and minimum candidate.
    pub(super) fn new(horizon: u64, seeded_population: u64) -> Self {
        Self {
            horizon,
            population: seeded_population,
            minimum_population: seeded_population,
            peak_population: seeded_population,
            peak_tick: 0,
            extinction_tick: None,
            plateau_sum: 0,
            plateau_ticks: 0,
            samples: Vec::new(),
        }
    }

    /// The plateau window is the ticks strictly after `0.75 x horizon`,
    /// compared in exact integer arithmetic.
    fn in_plateau_window(&self, tick: u64) -> bool {
        tick * 4 > self.horizon * 3
    }

    /// A tick is sampled when it is a multiple of [`SAMPLE_EVERY_TICKS`] or is
    /// the last executed tick — the horizon, or the extinction tick that ends
    /// the run early. One condition, so the final tick is never duplicated.
    fn is_sampled(&self, tick: u64, population: u64) -> bool {
        tick.is_multiple_of(SAMPLE_EVERY_TICKS) || tick == self.horizon || population == 0
    }

    /// Record one executed tick. `mean_energy`, `readings` and `tracking` are
    /// evaluated only on sampled ticks — the first keeps the `O(population)`
    /// energy sum to the predeclared cadence and leaves `null` at extinction,
    /// the second keeps the structure and clade passes to that cadence, and the
    /// third keeps the per-world counter reads there too.
    pub(super) fn observe(
        &mut self,
        tick: u64,
        population: u64,
        births_total: u64,
        mean_energy: impl FnOnce() -> f64,
        readings: impl FnOnce() -> PopulationReadings,
        tracking: impl FnOnce() -> WorldTracking,
    ) {
        self.population = population;
        self.minimum_population = self.minimum_population.min(population);
        if population > self.peak_population {
            self.peak_population = population;
            self.peak_tick = tick;
        }
        if population == 0 && self.extinction_tick.is_none() {
            self.extinction_tick = Some(tick);
        }
        if self.in_plateau_window(tick) {
            self.plateau_sum += population;
            self.plateau_ticks += 1;
        }
        if self.is_sampled(tick, population) {
            let alive = population > 0;
            let readings = readings();
            self.samples.push(PersistenceSample {
                tick,
                population,
                mean_energy: alive.then(|| six(mean_energy())),
                births_total,
                mean_genome_size: alive.then(|| six(readings.mean_genome_size)),
                mean_mesh_nodes: alive.then(|| six(readings.mean_mesh_nodes)),
                mean_generation: alive.then(|| six(readings.mean_generation)),
                surviving_founder_clade_count: Some(readings.surviving_founder_clade_count),
                shannon_entropy_nats: Some(readings.shannon_entropy_nats),
                sensor_census: Some(readings.sensor_census),
                occupancy_grid: Some(readings.occupancy_grid),
                tracking: tracking(),
            });
        }
    }

    pub(super) fn finish(self, seed: u64) -> PopulationPersistenceSeed {
        PopulationPersistenceSeed {
            seed,
            extinction_tick: self.extinction_tick,
            minimum_population: self.minimum_population,
            final_population: self.population,
            peak_population: self.peak_population,
            peak_tick: self.peak_tick,
            plateau_population: (self.plateau_ticks > 0)
                .then(|| ratio(self.plateau_sum, self.plateau_ticks)),
            // The last executed tick is always sampled, so the run's final
            // mean energy is the last sample's.
            mean_energy: self.samples.last().and_then(|s| s.mean_energy.clone()),
            samples: self.samples,
        }
    }
}

// ── Seed execution ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests;
