use rand::Rng;

use super::cgp_reproduction;
use crate::contracts::{CreatureId, Direction, Position};
use crate::creature::action_log::ActionLog;
use crate::creature::founder::FOUNDER_GENOME_SIZE_UNITS;
use crate::creature::identity::CreatureIdentityState;
use crate::creature::state::{CreatureState, PendingBirth};
use crate::mutation::phenotype::mutate_phenotype;
use crate::mutation::reachability::ParentExecuted;
use crate::mutation::MutationEngine;
use crate::runtime::action_decode::clamp_unit_interval;
use crate::simulation::birth_age::BirthOrder;
use crate::simulation::energy_accounting::DeathCause;
use crate::simulation::simulation::Simulation;

/// Genome replication cost multiplier on the parent's reproduce charge:
/// `1 + rate * max(genome_size - FOUNDER_GENOME_SIZE_UNITS, 0)`.
///
/// Exactly `1.0` for every genome at or below the founder's size and for
/// `rate == 0.0`, so the founder's charge is bit-identical to the base charge;
/// non-decreasing in `genome_size`. Composes multiplicatively with the
/// complexity and age multipliers of `adjusted_action_cost` (T03.F11).
#[inline]
#[must_use]
pub(crate) fn genome_replication_cost_multiplier(rate: f32, genome_size: u32) -> f32 {
    let units_above_founder = genome_size.saturating_sub(FOUNDER_GENOME_SIZE_UNITS);
    // u32 -> f32 is exact for every genome size that fits in memory.
    1.0 + rate * units_above_founder as f32
}

/// Gestation span in ticks for a parent of `genome_size` units (T03.F12):
/// `ceil(genome_size * rate / FOUNDER_GENOME_SIZE_UNITS)`.
///
/// At rate `1.0` the founder gestates one tick and a genome twice its size
/// two; `rate == 0.0` gives `0` (delivery at conception). Non-decreasing in
/// `genome_size` at a fixed rate. A rate large enough to exceed `u64`
/// saturates the cast.
#[inline]
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the operand is a finite non-negative ceiling; `as` saturates \
              past u64::MAX, which holds the parent for the rest of its life"
)]
pub(crate) fn gestation_ticks(genome_size: u32, rate: f32) -> u64 {
    ((f64::from(genome_size) * f64::from(rate)) / f64::from(FOUNDER_GENOME_SIZE_UNITS)).ceil()
        as u64
}

/// Result of an attempted reproduction action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReproductionActionResult {
    /// Every gate passed: the parent paid the charge and the litter and
    /// carries a pending birth, or (at gestation rate `0.0`) the child was
    /// born at once (T03.F12).
    Conceived,
    RejectedInvalidTarget,
    RejectedAgeConstraints,
    RejectedEnergyConstraints,
    RejectedPopulationCap,
}

impl ReproductionActionResult {
    /// Every rejection result, in declaration order.
    pub const REJECTIONS: [Self; 4] = [
        Self::RejectedInvalidTarget,
        Self::RejectedAgeConstraints,
        Self::RejectedEnergyConstraints,
        Self::RejectedPopulationCap,
    ];

    /// Stable string key used by transport/API boundaries.
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Conceived => "Conceived",
            Self::RejectedInvalidTarget => "RejectedInvalidTarget",
            Self::RejectedAgeConstraints => "RejectedAgeConstraints",
            Self::RejectedEnergyConstraints => "RejectedEnergyConstraints",
            Self::RejectedPopulationCap => "RejectedPopulationCap",
        }
    }
}

/// Fine-grained cause for invalid reproduction targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReproductionInvalidTargetCause {
    Barrier,
    Occupied,
    OutOfBounds,
    Contention,
}

impl ReproductionInvalidTargetCause {
    /// Stable string key used by transport/API boundaries.
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Barrier => "barrier",
            Self::Occupied => "occupied",
            Self::OutOfBounds => "out_of_bounds",
            Self::Contention => "contention",
        }
    }
}

/// The inheritance witness (T25.F04): the child's committed genome holds a
/// `DirectionPair` group with `|scale| > 1e-3` and no event of the birth
/// could have created a group (no committed `Graph.AddDirectionPairGroup`
/// or `Graph.RecruitNeutralInput`), so every pair group it carries came from
/// the parent's genome.
pub(crate) fn inherits_nonzero_pair_group(
    child: &crate::creature::genome::CreatureGenome,
    summary: &crate::mutation::types::MutationSummary,
) -> bool {
    use crate::creature::genome::cgp::Shape;
    use crate::creature::genome::BackendDef;
    use crate::mutation::types::MutationOperator as Op;
    let created = [Op::GraphAddDirectionPairGroup, Op::GraphRecruitNeutralInput]
        .iter()
        .any(|op| summary.applied_by_operator.get(op).is_some_and(|&n| n > 0));
    !created
        && child.nodes.iter().any(|node| match &node.backend_def {
            BackendDef::Graph(def) => def.groups().iter().any(|group| {
                matches!(group.shape(), Shape::DirectionPair { .. })
                    && group.scale().abs() > crate::creature::drives::PATH_WEIGHT_FLOOR
            }),
            BackendDef::Vm(_) => false,
        })
}

/// Apply a Reproduce action per v3-reproduction-spec.md Section 6 unified sequence.
///
/// `energy_transfer_fraction` is the share of the parent's post-cost energy
/// the child starts with, in [0, 1] (T17.F01); the litter is capped at
/// `default_offspring_energy` and refused under `initial_energy`.
///
/// Step 0 (T16.F02) debits the attempt effort,
/// `action_effort(reproduce_attempt_cost)`, before any gate; a rejected
/// attempt pays that effort and nothing more. Steps 1 to 8 then run at
/// conception on the post-effort energy. When every gate passes the parent
/// carries a pending birth delivered after `gestation_ticks` (T03.F12) by
/// [`deliver_pending_birth`]; a span of `0` (rate `0.0`) runs Steps 9 to 14
/// at once, the pre-feature engine.
///
/// Returns `Conceived` or why the attempt was rejected.
#[must_use]
pub fn apply_reproduce(
    parent_id: CreatureId,
    sim: &mut Simulation,
    dir: Direction,
    energy_transfer_fraction: f32,
    rng: &mut impl Rng,
) -> ReproductionActionResult {
    // Stats: always count attempt and per-tick reproduce regardless of outcome.
    sim.stats.reproduction_actions_attempted_total += 1;
    sim.stats.last_tick_reproduce += 1;

    // Step 0 (T16.F02): the attempt effort, paid whatever the outcome.
    let effort = super::pay_effort(
        &mut sim.creatures[parent_id],
        &sim.config.energy,
        sim.config.energy.costs.reproduce_attempt_cost,
        DeathCause::ActionReproduce,
    );
    sim.stats.energy_flows.action_charges.reproduce += effort;

    let result = conceive(parent_id, sim, dir, energy_transfer_fraction, rng);
    if result != ReproductionActionResult::Conceived {
        sim.stats.energy_flows.failed_action_effort.reproduce += effort;
    }
    result
}

/// Steps 1 to 8 of the unified sequence on the post-effort energy (T16.F01
/// order): gates first, then the replication cost and the litter.
fn conceive(
    parent_id: CreatureId,
    sim: &mut Simulation,
    dir: Direction,
    energy_transfer_fraction: f32,
    rng: &mut impl Rng,
) -> ReproductionActionResult {
    let parent_pos = sim.creatures[parent_id].position;

    // Step 1: Resolve target cell.
    let target = match sim.world.resolve_neighbor(parent_pos, dir) {
        Some(p) => p,
        None => {
            sim.stats.reproduction_actions_rejected_total += 1;
            *sim.stats
                .reproduction_actions_rejected_by_reason
                .entry(ReproductionActionResult::RejectedInvalidTarget)
                .or_insert(0) += 1;
            return ReproductionActionResult::RejectedInvalidTarget;
        }
    };

    // Step 2: Validate target cell (no barrier, not occupied).
    if !sim.world.is_valid_target_cell(target) {
        sim.stats.reproduction_actions_rejected_total += 1;
        *sim.stats
            .reproduction_actions_rejected_by_reason
            .entry(ReproductionActionResult::RejectedInvalidTarget)
            .or_insert(0) += 1;
        return ReproductionActionResult::RejectedInvalidTarget;
    }

    // Step 3: Check population cap.
    if sim.creatures.len() >= sim.config.population.max_creatures as usize {
        sim.stats.reproduction_actions_rejected_total += 1;
        *sim.stats
            .reproduction_actions_rejected_by_reason
            .entry(ReproductionActionResult::RejectedPopulationCap)
            .or_insert(0) += 1;
        return ReproductionActionResult::RejectedPopulationCap;
    }

    // Step 4: Enforce minimum reproduction age gate.
    if sim.creatures[parent_id].age < sim.config.energy.lifecycle.min_reproduce_age {
        sim.stats.reproduction_actions_rejected_total += 1;
        *sim.stats
            .reproduction_actions_rejected_by_reason
            .entry(ReproductionActionResult::RejectedAgeConstraints)
            .or_insert(0) += 1;
        return ReproductionActionResult::RejectedAgeConstraints;
    }

    // Step 5: Compute reproduce_cost (scaled by genome complexity, age, and
    // the genome replication cost on units above the founder's size) and the
    // parent's post-effort energy after it. Nothing more is deducted until
    // both gates pass (T16.F01): a rejected attempt pays only Step 0.
    let cost = sim.config.energy.adjusted_action_cost(
        sim.config.energy.costs.reproduce_cost,
        sim.creatures[parent_id].cached_complexity,
        sim.creatures[parent_id].age,
    ) * genome_replication_cost_multiplier(
        sim.config.energy.lifecycle.genome_replication_cost_per_unit,
        sim.creatures[parent_id].cached_genome_size,
    );
    let energy_before_cost = sim.creatures[parent_id].energy;
    let after_cost = energy_before_cost - cost;

    // Step 6: Check parent would have sufficient energy after the charge.
    if after_cost < sim.config.energy.lifecycle.min_reproduce_energy {
        sim.stats.reproduction_actions_rejected_total += 1;
        *sim.stats
            .reproduction_actions_rejected_by_reason
            .entry(ReproductionActionResult::RejectedEnergyConstraints)
            .or_insert(0) += 1;
        return ReproductionActionResult::RejectedEnergyConstraints;
    }

    // Step 7 (T17.F01): the litter is a fraction of the parent's post-cost
    // energy, capped at `default_offspring_energy`. Decode already sanitizes
    // the fraction to [0, 1]; the clamp here keeps `after_cost >= transfer`
    // true for every caller. A litter under `initial_energy` is not
    // conceived, and the refusal pays only Step 0.
    let fraction = clamp_unit_interval(energy_transfer_fraction);
    let transfer =
        (fraction * after_cost).min(sim.config.energy.lifecycle.default_offspring_energy);

    if transfer <= 0.0 || transfer < sim.config.energy.lifecycle.initial_energy {
        sim.stats.reproduction_actions_rejected_total += 1;
        sim.stats.reproduction_actions_rejected_litter_total += 1;
        *sim.stats
            .reproduction_actions_rejected_by_reason
            .entry(ReproductionActionResult::RejectedEnergyConstraints)
            .or_insert(0) += 1;
        return ReproductionActionResult::RejectedEnergyConstraints;
    }

    // Step 8: Both gates passed; deduct cost, then transfer, as two
    // successive f32 subtractions so the parent lands on the same value the
    // charge-first order produced.
    sim.creatures[parent_id].energy = after_cost;
    sim.stats.energy_flows.action_charges.reproduce +=
        sim.creatures[parent_id].observe_energy(energy_before_cost, DeathCause::ActionReproduce);
    sim.creatures[parent_id].energy = after_cost - transfer;
    sim.stats.energy_flows.parental_transfer_debit +=
        sim.creatures[parent_id].observe_energy(after_cost, DeathCause::ParentalTransfer);

    // Conception (T03.F12): the litter is carried for the gestation span and
    // born at the parent's Phase 2 slot on its last tick. A span of 0 is
    // delivered here, at the validated target, exactly as before the feature.
    sim.stats.gestations_started_total += 1;
    let span = gestation_ticks(
        sim.creatures[parent_id].cached_genome_size,
        sim.config
            .energy
            .lifecycle
            .gestation_ticks_per_founder_genome,
    );
    if span == 0 {
        spawn_child(parent_id, sim, target, dir, transfer, rng);
    } else {
        sim.creatures[parent_id].pending_birth = Some(PendingBirth {
            direction: dir,
            litter_energy: transfer,
            deliver_tick: sim.tick.saturating_add(span),
        });
    }
    ReproductionActionResult::Conceived
}

/// Outcome of delivering a pending birth (T03.F12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BirthDelivery {
    /// The child was born on this cell.
    Delivered(Position),
    /// No valid cell adjacent to the parent: the litter is lost.
    LostNoCell,
    /// The population was at its cap: the litter is lost.
    LostPopulationCap,
}

/// Deliver `parent_id`'s pending birth at its Phase 2 slot (T03.F12).
///
/// At the population cap the litter is lost; otherwise the stored direction
/// is resolved from the parent's position, falling back to the first valid
/// cell in `Direction::ALL` order, and no valid cell loses the litter. A
/// delivered birth runs Steps 9 to 14 with the stored litter energy, the
/// child heading the direction of the cell it is placed on (T11.F30). The
/// pending birth is cleared in every case. Callers check the parent holds a
/// pending birth.
pub(crate) fn deliver_pending_birth(
    parent_id: CreatureId,
    sim: &mut Simulation,
    rng: &mut impl Rng,
) -> BirthDelivery {
    let pending = sim.creatures[parent_id]
        .pending_birth
        .take()
        .expect("delivery requires a pending birth");
    if sim.creatures.len() >= sim.config.population.max_creatures as usize {
        sim.stats.energy_flows.gestation_lost += f64::from(pending.litter_energy);
        sim.stats.gestations_lost_population_cap_total += 1;
        return BirthDelivery::LostPopulationCap;
    }
    let parent_pos = sim.creatures[parent_id].position;
    let valid_cell = |dir: Direction| {
        sim.world
            .resolve_neighbor(parent_pos, dir)
            .filter(|&cell| sim.world.is_valid_target_cell(cell))
    };
    let valid_step = |dir: Direction| valid_cell(dir).map(|cell| (dir, cell));
    let Some((direction, target)) =
        valid_step(pending.direction).or_else(|| Direction::ALL.into_iter().find_map(valid_step))
    else {
        sim.stats.energy_flows.gestation_lost += f64::from(pending.litter_energy);
        sim.stats.gestations_lost_no_cell_total += 1;
        return BirthDelivery::LostNoCell;
    };
    spawn_child(
        parent_id,
        sim,
        target,
        direction,
        pending.litter_energy,
        rng,
    );
    BirthDelivery::Delivered(target)
}

/// Steps 9 to 14: clone and mutate the parent's genome and state at this
/// moment and place the child on `target` with `litter_energy`. `direction`
/// is the step from the parent's cell to `target`, the child's heading
/// (T11.F30).
#[allow(
    clippy::too_many_lines,
    reason = "Steps 9 to 14 of the unified reproduction sequence in \
              v3-reproduction-spec.md Section 6 are ordered end to end"
)]
fn spawn_child(
    parent_id: CreatureId,
    sim: &mut Simulation,
    target: Position,
    direction: Direction,
    transfer: f32,
    rng: &mut impl Rng,
) {
    // Step 9: Build offspring draft (clone parent genome + state).
    let child_genome = sim.creatures[parent_id].genome.clone();
    let child_shared_memory = sim.creatures[parent_id].shared_memory;
    let child_generation = sim.creatures[parent_id].generation + 1;
    let child_channels = sim.creatures[parent_id].phenotype_channels;
    let child_active_channel = sim.creatures[parent_id].phenotype_active_channel;
    let child_polarity = sim.creatures[parent_id].phenotype_channel_polarity;
    let parent_cached_reachable = sim.creatures[parent_id].cached_reachable_nodes.clone();
    // T11.F17: the parent's own recent dispatches bias the offspring's targets.
    // Passed unresolved, so a zero-event birth derives nothing.
    let parent = &sim.creatures[parent_id];
    let parent_executed = ParentExecuted::Record(&parent.graph_runtime.dispatch_record, parent.age);

    // Step 10: Apply genome mutations to the draft, which carries the
    // parent's learned weights as birth seeds (T25.F01).
    let mut child_genome = child_genome;
    cgp_reproduction::capture_birth_seeds(
        &mut child_genome,
        sim.creatures[parent_id].graph_runtime.learned(),
    );
    let summary = MutationEngine::apply_mutations_with_food_type_count(
        &mut child_genome,
        &sim.config.mutation,
        &parent_cached_reachable,
        parent_executed,
        rng,
        sim.config.world.food.types.len(),
    );

    // Update mutation stats.
    sim.stats.mutation_events_attempted_total += summary.attempted_events as u64;
    sim.stats.mutation_events_applied_total += summary.applied_events as u64;
    sim.stats.mutation_events_skipped_total += summary.skipped_events as u64;
    for (domain, count) in &summary.attempted_by_domain {
        *sim.stats
            .mutation_events_attempted_total_by_domain
            .entry(*domain)
            .or_insert(0) += *count as u64;
    }
    for (domain, count) in &summary.applied_by_domain {
        *sim.stats
            .mutation_events_applied_total_by_domain
            .entry(*domain)
            .or_insert(0) += *count as u64;
    }
    for (operator, count) in &summary.attempted_by_operator {
        *sim.stats
            .mutation_events_attempted_total_by_operator
            .entry(*operator)
            .or_insert(0) += *count as u64;
    }
    for (operator, count) in &summary.applied_by_operator {
        *sim.stats
            .mutation_events_applied_total_by_operator
            .entry(*operator)
            .or_insert(0) += *count as u64;
    }
    for (reason, count) in &summary.skip_reasons {
        *sim.stats
            .mutation_events_skipped_by_reason
            .entry(*reason)
            .or_insert(0) += *count as u64;
    }
    for (operator, count) in &summary.skipped_by_operator {
        *sim.stats
            .mutation_events_skipped_total_by_operator
            .entry(*operator)
            .or_insert(0) += *count as u64;
    }
    for (operator, funnel) in &summary.operator_funnel_by_operator {
        let entry = sim
            .stats
            .mutation_operator_funnel_total_by_operator
            .entry(*operator)
            .or_default();
        entry.attempted += funnel.attempted;
        entry.applicable += funnel.applicable;
        entry.structurally_valid += funnel.structurally_valid;
        entry.applied += funnel.applied;
        entry.skipped += funnel.skipped;
    }
    for (operator, by_reason) in &summary.skip_reasons_by_operator {
        let entry = sim
            .stats
            .mutation_skip_reasons_total_by_operator
            .entry(*operator)
            .or_default();
        for (reason, count) in by_reason {
            *entry.entry(*reason).or_insert(0) += *count as u64;
        }
    }
    for (operator, by_class) in &summary.added_node_input_classes_by_operator {
        let entry = sim
            .stats
            .mutation_added_node_input_classes_total_by_operator
            .entry(*operator)
            .or_default();
        for (class, count) in by_class {
            *entry.entry(*class).or_insert(0) += *count as u64;
        }
    }
    for (operator, by_key) in &summary.added_node_world_inputs_by_operator {
        let entry = sim
            .stats
            .mutation_added_node_world_inputs_total_by_operator
            .entry(*operator)
            .or_default();
        for (key, count) in by_key {
            *entry.entry(*key).or_insert(0) += *count as u64;
        }
    }
    sim.stats.mutation_reachable_target_total += summary.reachable_target_events as u64;
    sim.stats.mutation_executed_target_total += summary.executed_target_events as u64;
    sim.stats.mutation_unreachable_target_total += summary.unreachable_target_events as u64;
    sim.stats.mutation_not_applicable_target_total += summary.not_applicable_events as u64;
    sim.stats.graph_edges_detached_total += u64::from(summary.graph_edges_detached);
    sim.stats.groups_removed_total += u64::from(summary.groups_removed);
    for (total, count) in sim
        .stats
        .mutation_attribute_changes_total_by_scope
        .iter_mut()
        .zip(summary.attribute_changes_by_scope)
    {
        *total += u64::from(count);
    }
    sim.stats.pair_group_inherited_nonzero_births_total +=
        u64::from(inherits_nonzero_pair_group(&child_genome, &summary));
    let mut child_birth_mutation_operators: Vec<_> =
        summary.applied_by_operator.keys().copied().collect();
    child_birth_mutation_operators.sort_by_key(|operator| operator.as_key());
    let child_birth_mutation_operators = child_birth_mutation_operators.into_boxed_slice();
    let child_birth_edge_class = summary.birth_edge_class();

    // Step 11: Phenotype mutation — triggered only when at least one genome event was applied.
    let (child_channels, child_active_channel, child_polarity) = if summary.applied_events > 0 {
        mutate_phenotype(
            child_channels,
            child_active_channel,
            child_polarity,
            &sim.config.mutation.phenotype,
            rng,
        )
    } else {
        (child_channels, child_active_channel, child_polarity)
    };

    // Step 11b: Identity inheritance — child inherits parent identity, kin-tag
    // mutates only when genome mutation applied events.
    let parent_identity = sim.creatures[parent_id].identity;
    let mut child_identity = CreatureIdentityState::inherit(&parent_identity);
    if summary.applied_events > 0 {
        child_identity.mutate_kin_tag(rng);
    }

    // Step 12: Lamarckian inheritance happens in the constructors below,
    // which build the newborn's weights from the draft's seeds.

    // Step 13–14: Spawn child in slotmap + world.
    // No-mutation fast path: if no genome mutations were applied, the offspring's
    // genome is identical to the parent's — copy cached_complexity and
    // cached_genome_size to avoid expensive recomputation.
    let parent_cached_complexity = sim.creatures[parent_id].cached_complexity;
    let parent_cached_genome_size = sim.creatures[parent_id].cached_genome_size;
    let child_id = sim.creatures.insert_with_key(move |id| {
        let mut child = if summary.applied_events == 0 {
            CreatureState::new_with_cached_fields(
                id,
                child_genome,
                target,
                transfer,
                child_generation,
                child_channels,
                child_active_channel,
                child_polarity,
                child_identity,
                child_shared_memory,
                parent_cached_complexity,
                parent_cached_genome_size,
                parent_cached_reachable,
            )
        } else {
            CreatureState::new(
                id,
                child_genome,
                target,
                transfer,
                child_generation,
                child_channels,
                child_active_channel,
                child_polarity,
                child_identity,
                child_shared_memory,
            )
        };
        child.birth_mutation_operators = child_birth_mutation_operators;
        child.birth_edge_class = child_birth_edge_class;
        child.heading = direction;
        child
    });
    sim.stats.energy_flows.offspring_energy_credit += f64::from(sim.creatures[child_id].energy);
    sim.action_logs
        .insert(child_id, ActionLog::new(sim.config.action_log.capacity));
    sim.world.place_creature(target, child_id);
    if let Some(parent) = sim.creatures.get_mut(parent_id) {
        sim.stats
            .birth_parent_age
            .record(BirthOrder::of(parent.offspring_spawned_count), parent.age);
        parent.offspring_spawned_count += 1;
    }

    sim.stats.reproduction_actions_spawned_total += 1;
    sim.stats.gestations_delivered_total += 1;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MutationConfig;
    use crate::creature::genome::cgp::{
        ComputeNode, ComputeNodeKind, EdgeOccurrence, EdgeSurface, GraphEdge, GraphSource,
    };
    use crate::creature::genome::{BackendDef, CreatureGenome};
    use crate::creature::genome::{HebbianRule, PlasticityConfig};

    /// A parent state on `genome` whose plastic slots of mesh node `n`
    /// learned `parent[n]`, in canonical order (each backend visited).
    fn parent_state(
        genome: &CreatureGenome,
        parent: &[Vec<f32>],
    ) -> crate::creature::state::GraphRuntimeState {
        let mut state = crate::creature::state::GraphRuntimeState::for_genome(genome, Vec::new());
        for (node, values) in parent.iter().enumerate() {
            for (slot, &value) in values.iter().enumerate() {
                state.learned.set_slot(node, slot, value, 0.0);
            }
        }
        state
    }

    /// The child of a parent on `genome` holding `parent` (see
    /// [`parent_state`]) after `mutate` edits the seeded draft: each mesh
    /// node's learned values per plastic slot, `None` where uninitialized.
    fn birth_learned(
        genome: &CreatureGenome,
        parent: &[Vec<f32>],
        mutate: impl FnOnce(&mut CreatureGenome),
    ) -> (CreatureGenome, Vec<Vec<Option<f32>>>) {
        let state = parent_state(genome, parent);
        let mut child = genome.clone();
        cgp_reproduction::capture_birth_seeds(&mut child, state.learned());
        mutate(&mut child);
        let seeds = crate::creature::newborn::take_birth_seeds(&mut child);
        let built = crate::creature::state::GraphRuntimeState::for_genome(&child, seeds);
        let learned = (0..child.nodes.len())
            .map(|node| {
                built
                    .learned()
                    .edges(node)
                    .map(|edge| edge.learned)
                    .collect()
            })
            .collect();
        (child, learned)
    }

    fn build_child_learned(genome: &CreatureGenome, parent: &[Vec<f32>]) -> Vec<Vec<Option<f32>>> {
        birth_learned(genome, parent, |_| {}).1
    }

    /// Helper: builds a genome with a single Graph mesh node containing the given compute nodes.
    fn genome_with_cgp_compute_nodes(compute_nodes: Vec<ComputeNode>) -> CreatureGenome {
        use crate::contracts::NodeId;
        use crate::creature::genome::NodeGenome;

        let mut def = crate::creature::genome::cgp::CgpGraphBackendDef::new_with_fixed_outputs();
        def = def.with_compute_nodes(compute_nodes);

        CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Graph(def),
                targets: vec![],
            }],
        }
    }

    /// Newborns on both reproduction paths carry a genome size equal to a
    /// fresh `genome_size()`: the mutated path recomputes it, the no-mutation
    /// fast path copies the parent's.
    fn assert_every_creature_caches_its_own_genome_size(per_unit_rate: f64) {
        use crate::config::SimulationConfig;
        use crate::simulation::{run_tick, seed_simulation};

        let mut cfg = SimulationConfig::default();
        cfg.world.width = 48;
        cfg.world.height = 48;
        cfg.population.initial_creatures = 200;
        cfg.mutation.per_unit_rate = per_unit_rate;
        let mut sim = seed_simulation(cfg, 2026);
        for _ in 0..40 {
            run_tick(&mut sim, &mut None);
        }

        assert!(
            sim.stats.reproduction_actions_spawned_total > 0,
            "the fixture must actually reproduce"
        );
        for (_, creature) in sim.creatures.iter() {
            assert_eq!(
                creature.cached_genome_size,
                creature.genome.genome_size(),
                "generation {} caches a stale genome size",
                creature.generation
            );
        }
    }

    // The multiplier's identity cases are exact products of `1.0 + rate * 0.0`
    // (or `0.0 * n`), so `==` is the intended bit-exact check there.
    #[test]
    fn replication_multiplier_is_exactly_one_for_the_founder() {
        assert_eq!(
            genome_replication_cost_multiplier(0.1, FOUNDER_GENOME_SIZE_UNITS),
            1.0
        );
    }

    #[test]
    fn replication_multiplier_is_exactly_one_below_the_founder_anchor() {
        // T25.F02: the anchor is 80 (was 97); the old rule and 97 restore 96.
        for size in [0, 1, 7, 79] {
            assert_eq!(genome_replication_cost_multiplier(0.1, size), 1.0);
        }
    }

    #[test]
    fn replication_multiplier_is_exactly_one_at_rate_zero() {
        for size in [0, 111, 386, 10_000] {
            assert_eq!(genome_replication_cost_multiplier(0.0, size), 1.0);
        }
    }

    #[test]
    fn replication_multiplier_charges_the_reference_genome_31_6_times() {
        // The T03.F08 cost arm's 386-unit genome: 306 units above the 80-unit
        // founder (T25.F02 unit rule; 289 above the 97-unit anchor gave 29.9,
        // which the old rule and anchor 97 restore).
        let factor = genome_replication_cost_multiplier(0.1, 386);
        assert!((factor - 31.6).abs() < 1e-4, "factor {factor}");
    }

    proptest::proptest! {
        #[test]
        fn replication_multiplier_is_at_least_one(
            rate in 0.0f32..10.0,
            size in proptest::prelude::any::<u32>(),
        ) {
            proptest::prop_assert!(genome_replication_cost_multiplier(rate, size) >= 1.0);
        }

        #[test]
        fn replication_multiplier_is_non_decreasing_in_size(
            rate in 0.0f32..10.0,
            a in proptest::prelude::any::<u32>(),
            b in proptest::prelude::any::<u32>(),
        ) {
            let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
            proptest::prop_assert!(
                genome_replication_cost_multiplier(rate, lo)
                    <= genome_replication_cost_multiplier(rate, hi)
            );
        }

        #[test]
        fn replication_multiplier_is_exactly_one_at_or_below_the_anchor(
            rate in 0.0f32..10.0,
            size in 0u32..=FOUNDER_GENOME_SIZE_UNITS,
        ) {
            proptest::prop_assert_eq!(genome_replication_cost_multiplier(rate, size), 1.0);
        }

        #[test]
        fn gestation_ticks_at_rate_one_is_size_div_ceil_founder(
            size in proptest::prelude::any::<u32>(),
        ) {
            proptest::prop_assert_eq!(
                gestation_ticks(size, 1.0),
                u64::from(size.div_ceil(FOUNDER_GENOME_SIZE_UNITS))
            );
        }

        #[test]
        fn gestation_ticks_is_non_decreasing_in_size(
            rate in 0.0f32..10.0,
            a in proptest::prelude::any::<u32>(),
            b in proptest::prelude::any::<u32>(),
        ) {
            let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
            proptest::prop_assert!(gestation_ticks(lo, rate) <= gestation_ticks(hi, rate));
        }

        #[test]
        fn gestation_ticks_is_zero_at_rate_zero(size in proptest::prelude::any::<u32>()) {
            proptest::prop_assert_eq!(gestation_ticks(size, 0.0), 0);
        }
    }

    #[test]
    fn gestation_ticks_hold_the_founder_one_tick_and_twice_its_size_two() {
        assert_eq!(gestation_ticks(FOUNDER_GENOME_SIZE_UNITS, 1.0), 1);
        assert_eq!(gestation_ticks(1, 1.0), 1);
        assert_eq!(gestation_ticks(0, 1.0), 0);
        // T25.F02 anchor 80 (97 restores 98 / 194 / 195).
        assert_eq!(gestation_ticks(81, 1.0), 2);
        assert_eq!(gestation_ticks(160, 1.0), 2);
        assert_eq!(gestation_ticks(161, 1.0), 3);
        assert_eq!(gestation_ticks(FOUNDER_GENOME_SIZE_UNITS, 2.0), 2);
        assert_eq!(gestation_ticks(FOUNDER_GENOME_SIZE_UNITS, 0.5), 1);
        assert_eq!(gestation_ticks(u32::MAX, f32::MAX), u64::MAX);
    }

    #[test]
    fn mutated_newborns_cache_their_own_genome_size() {
        // About five requested events per founder birth.
        assert_every_creature_caches_its_own_genome_size(0.05);
    }

    #[test]
    fn fast_path_newborns_cache_the_parents_genome_size() {
        assert_every_creature_caches_its_own_genome_size(0.0);
    }

    /// T16.F01 invariant 4 as T16.F02 amends it: the replication charge
    /// accumulates only over births. With the energy gate above
    /// `max_energy`, every attempt is an energy rejection, so the whole
    /// reproduce flow is attempt effort, all of it failed.
    #[test]
    fn energy_rejected_attempts_pay_only_their_attempt_effort() {
        use crate::config::SimulationConfig;
        use crate::simulation::{run_tick, seed_simulation};

        let mut cfg = SimulationConfig::default();
        cfg.world.width = 48;
        cfg.world.height = 48;
        cfg.population.initial_creatures = 200;
        cfg.energy.lifecycle.min_reproduce_energy = cfg.energy.lifecycle.max_energy + 1.0;
        let mut sim = seed_simulation(cfg, 2026);
        for _ in 0..40 {
            run_tick(&mut sim, &mut None);
        }

        let energy_rejections = sim
            .stats
            .reproduction_actions_rejected_by_reason
            .get(&ReproductionActionResult::RejectedEnergyConstraints)
            .copied()
            .unwrap_or(0);
        assert!(
            energy_rejections > 0,
            "the fixture must actually reject attempts on energy"
        );
        assert_eq!(sim.stats.reproduction_actions_spawned_total, 0);
        let flows = &sim.stats.energy_flows;
        assert!(flows.action_charges.reproduce > 0.0);
        // Exact equality is the claim: nothing but failed attempt effort.
        assert_eq!(
            flows.action_charges.reproduce, flows.failed_action_effort.reproduce,
            "rejected attempts contribute only their effort to action_charges.reproduce"
        );
    }

    #[test]
    fn lamarckian_copies_parent_learned_weights() {
        let genome = genome_with_cgp_compute_nodes(vec![
            ComputeNode::new(ComputeNodeKind::Constant(1.0), vec![]),
            crate::creature::genome::cgp::test_support::plastic_node(
                ComputeNodeKind::Add,
                vec![GraphEdge::new(GraphSource::ComputeNode(0), 0.5)],
                Some(PlasticityConfig {
                    rule: HebbianRule::Classic,
                    learning_rate: 0.1,
                    weight_clamp: 5.0,
                    lamarckian: true,
                    modulation: None,
                }),
            ),
        ]);

        // Parent has learned weight 0.9 (drifted from genome 0.5) on CN1's
        // one plastic edge; CN0 has none.
        let child = build_child_learned(&genome, &[vec![0.9]]);
        assert_eq!(child, vec![vec![Some(0.9)]]);
    }

    #[test]
    fn darwinian_resets_to_empty() {
        let genome = genome_with_cgp_compute_nodes(vec![
            ComputeNode::new(ComputeNodeKind::Constant(1.0), vec![]),
            crate::creature::genome::cgp::test_support::plastic_node(
                ComputeNodeKind::Add,
                vec![GraphEdge::new(GraphSource::ComputeNode(0), 0.5)],
                Some(PlasticityConfig {
                    rule: HebbianRule::Classic,
                    learning_rate: 0.1,
                    weight_clamp: 5.0,
                    lamarckian: false,
                    modulation: None,
                }),
            ),
        ]);

        // Parent has learned weight 0.9; a Darwinian edge ignores its seed and
        // starts at its genomic weight, uninitialized until its first visit.
        let child = build_child_learned(&genome, &[vec![0.9]]);
        assert_eq!(child, vec![vec![None]]);
    }

    #[test]
    fn lamarckian_with_no_parent_weights_returns_empty() {
        let genome = genome_with_cgp_compute_nodes(vec![
            crate::creature::genome::cgp::test_support::plastic_node(
                ComputeNodeKind::Add,
                vec![GraphEdge::new(GraphSource::ComputeNode(0), 0.5)],
                Some(PlasticityConfig {
                    rule: HebbianRule::Oja,
                    learning_rate: 0.1,
                    weight_clamp: 5.0,
                    lamarckian: true,
                    modulation: None,
                }),
            ),
        ]);

        // Parent never ran graph execution: nothing initialized, no seeds.
        let child = build_child_learned(&genome, &[]);
        assert_eq!(child, vec![vec![None]]);
    }

    #[test]
    fn vm_nodes_skipped() {
        use crate::contracts::NodeId;
        use crate::creature::genome::{NodeGenome, VmBackendDef, VmInstruction};

        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![],
            }],
        };

        // VM nodes carry no plastic edge.
        let child = build_child_learned(&genome, &[]);
        assert_eq!(child, vec![Vec::new()]);
    }

    /// A Graph node the batch creates on a VM-only parent's draft carries no
    /// seeds and no plastic edge, so the newborn has no learned state.
    #[test]
    fn birth_graph_created_on_a_vm_only_draft_has_no_learned_state() {
        use crate::contracts::{NodeId, RouteTarget};
        use crate::creature::genome::{NodeGenome, VmBackendDef, VmInstruction};
        use crate::mutation::reachability::TargetSelector;
        use crate::mutation::topology::{TopologyMutator, TopologyOperator};
        use rand::SeedableRng;

        let vm = |id: u32, targets: Vec<RouteTarget>| NodeGenome {
            node_id: NodeId::new(id),
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![VmInstruction::Halt],
            }),
            targets,
        };
        let mut child = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![
                vm(
                    0,
                    vec![RouteTarget {
                        target_id: NodeId::new(1),
                        slot: 0,
                        gate_bias: 0.0,
                    }],
                ),
                vm(1, vec![]),
            ],
        };
        let parent = crate::creature::state::GraphRuntimeState::for_genome(&child, Vec::new());
        cgp_reproduction::capture_birth_seeds(&mut child, parent.learned());
        TopologyMutator::apply(
            &mut child,
            TopologyOperator::SwapNodeBackend,
            &mut TargetSelector::reachable_only(&[0, 1], 0.0),
            &mut rand::rngs::SmallRng::seed_from_u64(5),
            &MutationConfig::default(),
        )
        .unwrap();
        let BackendDef::Graph(created) = &child.nodes[2].backend_def else {
            panic!("a swapped VM copy is a Graph node");
        };
        assert!(created.birth_seeds().is_none());
        assert!(crate::creature::newborn::take_birth_seeds(&mut child).is_empty());
        assert!(
            !crate::creature::state::GraphRuntimeState::for_genome(&child, Vec::new())
                .learned()
                .has_learning()
        );
    }
    #[test]
    fn birth_mesh_copies_removal_and_backend_replacement_keep_their_own_values() {
        use crate::contracts::{NodeId, RouteTarget};
        use crate::mutation::reachability::TargetSelector;
        use crate::mutation::topology::{TopologyMutator, TopologyOperator};
        use rand::SeedableRng;
        let compute = crate::creature::genome::cgp::test_support::plastic_node(
            ComputeNodeKind::Add,
            vec![GraphEdge::new(
                GraphSource::SharedMemory {
                    slot: 0,
                    previous: false,
                },
                0.1,
            )],
            Some(PlasticityConfig {
                rule: HebbianRule::Classic,
                learning_rate: 0.1,
                weight_clamp: 5.0,
                lamarckian: true,
                modulation: None,
            }),
        );
        let mut original = genome_with_cgp_compute_nodes(vec![compute]);
        original.nodes = vec![original.nodes[0].clone(); 3];
        for (idx, node) in original.nodes.iter_mut().enumerate() {
            node.node_id = NodeId::new(idx as u32);
            if idx < 2 {
                node.targets = vec![RouteTarget {
                    target_id: NodeId::new(idx as u32 + 1),
                    slot: 0,
                    gate_bias: 0.0,
                }];
            }
            let BackendDef::Graph(def) = &mut node.backend_def else {
                unreachable!()
            };
            def.edge_mut(EdgeOccurrence::new(EdgeSurface::ComputeInput(0), 0))
                .set_weight(idx as f32 + 1.0);
        }
        let parent = vec![vec![2.0], vec![3.0], vec![4.0]];
        for op in [
            TopologyOperator::CopyNode,
            TopologyOperator::CopyMeshBackwardSlice,
            TopologyOperator::CopyMeshForwardSlice,
            TopologyOperator::RemoveNode,
        ] {
            let (child, result) = birth_learned(&original, &parent, |child| {
                TopologyMutator::apply(
                    child,
                    op,
                    &mut TargetSelector::reachable_only(&[0, 1, 2], 0.0),
                    &mut rand::rngs::SmallRng::seed_from_u64(73),
                    &MutationConfig::default(),
                )
                .unwrap();
            });
            for (idx, node) in child.nodes.iter().enumerate() {
                let BackendDef::Graph(def) = &node.backend_def else {
                    unreachable!()
                };
                assert_eq!(
                    result[idx][0],
                    Some(def.compute_nodes()[0].inputs[0].own_weight() + 1.0)
                );
                assert!(def.birth_seeds().is_none());
            }
        }
        // Replacing a backend at the same mesh index/ID gives it no correspondence.
        let (_, result) = birth_learned(&original, &parent, |child| {
            child.nodes[0].backend_def = original.nodes[0].backend_def.clone();
        });
        assert_eq!(result[0][0], None);
        assert_eq!(result[1][0], Some(3.0));
    }

    /// One live reproduction of a single Lamarckian parent holding `learned`
    /// on its one edge, at mutation rate `rate` and reproduction seed `seed`.
    /// Returns the simulation, the parent's id and genome, the child's id and
    /// the genome events the birth applied.
    fn live_birth(
        rate: f64,
        seed: u64,
        learned: Option<f32>,
    ) -> (
        Simulation,
        crate::contracts::CreatureId,
        CreatureGenome,
        crate::contracts::CreatureId,
        u64,
    ) {
        use crate::config::SimulationConfig;
        use crate::simulation::seed_simulation;
        use rand::SeedableRng;
        let mut cfg = SimulationConfig::default();
        cfg.population.initial_creatures = 1;
        cfg.mutation.per_unit_rate = rate;
        // Rate 0.0 delivers at conception (T03.F12): the child exists at return.
        cfg.energy.lifecycle.gestation_ticks_per_founder_genome = 0.0;
        let mut sim = seed_simulation(cfg, 83);
        let parent_id = sim.creatures.keys().next().unwrap();
        // A seeded founder carries no seeds and no runtime weights.
        assert!(!sim.creatures[parent_id]
            .graph_runtime
            .learned()
            .has_learning());
        let genome = genome_with_cgp_compute_nodes(vec![
            crate::creature::genome::cgp::test_support::plastic_node(
                ComputeNodeKind::Add,
                vec![GraphEdge::new(
                    GraphSource::SharedMemory {
                        slot: 0,
                        previous: false,
                    },
                    0.5,
                )],
                Some(PlasticityConfig {
                    rule: HebbianRule::Classic,
                    learning_rate: 0.1,
                    weight_clamp: 5.0,
                    lamarckian: true,
                    modulation: None,
                }),
            ),
        ]);
        let parent = &mut sim.creatures[parent_id];
        parent.genome = genome.clone();
        parent.age = sim.config.energy.lifecycle.min_reproduce_age;
        parent.energy = 10000.0;
        parent.graph_runtime =
            crate::creature::state::GraphRuntimeState::for_genome(&genome, Vec::new());
        if let Some(learned) = learned {
            parent.graph_runtime.learned.set_slot(0, 0, learned, 99.0);
        }
        parent.graph_runtime.node_state = vec![vec![7.0]];
        parent.graph_runtime.node_outputs = vec![vec![8.0]];
        parent.graph_runtime.dispatch_record.record_dispatch(0);
        let direction = Direction::ALL
            .into_iter()
            .find(|&direction| {
                sim.world
                    .resolve_neighbor(sim.creatures[parent_id].position, direction)
                    .is_some_and(|p| sim.world.is_valid_target_cell(p))
            })
            .unwrap();
        assert_eq!(
            apply_reproduce(
                parent_id,
                &mut sim,
                direction,
                10.0,
                &mut rand::rngs::SmallRng::seed_from_u64(seed)
            ),
            ReproductionActionResult::Conceived
        );
        let child_id = sim.creatures.keys().find(|&id| id != parent_id).unwrap();
        let applied = sim.stats.mutation_events_applied_total;
        (sim, parent_id, genome, child_id, applied)
    }

    /// The newborn holds the inherited value in its own storage, carries no
    /// seeds, starts with empty temporal state, and the parent is unchanged.
    fn assert_seeded_birth(
        sim: &Simulation,
        parent_id: crate::contracts::CreatureId,
        genome: &CreatureGenome,
        child_id: crate::contracts::CreatureId,
    ) {
        let child = &sim.creatures[child_id];
        let edge = child.graph_runtime.learned().edges(0).next().unwrap();
        assert_eq!(edge.learned, Some(0.9));
        // A pure-Hebbian slot carries no trace.
        assert_eq!(edge.trace, None);
        assert!(child.graph_runtime.node_state.is_empty());
        assert!(child.graph_runtime.node_outputs.is_empty());
        assert!(child.graph_runtime.dispatch_record.is_empty());
        for node in &child.genome.nodes {
            if let BackendDef::Graph(def) = &node.backend_def {
                assert!(def.birth_seeds().is_none());
            }
        }
        let parent = &sim.creatures[parent_id];
        assert_eq!(&parent.genome, genome);
        let edge = parent.graph_runtime.learned().edges(0).next().unwrap();
        assert_eq!(edge.learned, Some(0.9));
    }

    /// Fast path (`new_with_cached_fields`, no genome event applied).
    #[test]
    fn birth_applied_reproduction_keeps_parent_and_resets_all_newborn_credit() {
        let (sim, parent_id, genome, child_id, applied) = live_birth(0.0, 2, Some(0.9));
        assert_eq!(applied, 0);
        assert_seeded_birth(&sim, parent_id, &genome, child_id);
    }

    /// Mutated branch (`CreatureState::new`, at least one genome event
    /// applied): the first seed whose events leave the learned edge in place.
    #[test]
    fn mutated_birth_keeps_parent_and_resets_all_newborn_credit() {
        let (sim, parent_id, genome, child_id) = (0..64)
            .find_map(|seed| {
                let (sim, parent_id, genome, child_id, applied) = live_birth(1.0, seed, Some(0.9));
                let kept = sim.creatures[child_id]
                    .graph_runtime
                    .learned()
                    .edges(0)
                    .next()
                    .is_some_and(|edge| edge.learned == Some(0.9));
                (applied > 0 && kept).then_some((sim, parent_id, genome, child_id))
            })
            .expect("a mutated birth keeps the learned edge");
        assert_seeded_birth(&sim, parent_id, &genome, child_id);
    }

    /// A parent with nothing initialized gives a newborn an uninitialized
    /// layout: every plastic slot at its genomic weight until its first visit.
    #[test]
    fn birth_from_a_parent_with_nothing_learned_starts_uninitialized() {
        let (sim, _, _, child_id, applied) = live_birth(0.0, 2, None);
        assert_eq!(applied, 0);
        let learned = sim.creatures[child_id].graph_runtime.learned();
        let edges: Vec<_> = learned.edges(0).collect();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].learned, None);
        assert_eq!(edges[0].genomic, 0.5);
    }

    /// One live birth at mutation rate 1.0 and reproduction seed `seed` from
    /// a founder parent whose Graph node carries two barrier-ring groups.
    /// Returns the run's `(graph_edges_detached_total, groups_removed_total)`.
    fn grouped_birth_totals(seed: u64) -> (u64, u64) {
        use crate::config::SimulationConfig;
        use crate::contracts::{InputReference, WorldInputKey};
        use crate::creature::founder::v3alpha1_founder_genome;
        use crate::creature::genome::cgp::Shape;
        use crate::creature::genome::vote::DirectedVoteKind;
        use crate::simulation::seed_simulation;
        use rand::SeedableRng;

        let mut cfg = SimulationConfig::default();
        cfg.population.initial_creatures = 1;
        cfg.mutation.per_unit_rate = 1.0;
        cfg.energy.lifecycle.gestation_ticks_per_founder_genome = 0.0;
        let mut sim = seed_simulation(cfg, 83);
        let parent_id = sim.creatures.keys().next().unwrap();
        let mut genome = v3alpha1_founder_genome();
        let node = genome
            .nodes
            .iter_mut()
            .find(|node| matches!(node.backend_def, BackendDef::Graph(_)))
            .expect("the founder carries a Graph node");
        let ring = u16::try_from(node.input_refs.len()).unwrap();
        node.input_refs
            .push(InputReference::World(WorldInputKey::NeighborBarrierRing));
        let BackendDef::Graph(def) = &mut node.backend_def else {
            unreachable!("a Graph node")
        };
        def.add_group(Shape::Ring { ring_ref: ring }, DirectedVoteKind::Move, 0);
        def.add_group(Shape::Ring { ring_ref: ring }, DirectedVoteKind::Move, 4);
        let parent = &mut sim.creatures[parent_id];
        parent.genome = genome;
        parent.age = sim.config.energy.lifecycle.min_reproduce_age;
        parent.energy = 10000.0;
        let direction = Direction::ALL
            .into_iter()
            .find(|&direction| {
                sim.world
                    .resolve_neighbor(sim.creatures[parent_id].position, direction)
                    .is_some_and(|p| sim.world.is_valid_target_cell(p))
            })
            .unwrap();
        assert_eq!(
            apply_reproduce(
                parent_id,
                &mut sim,
                direction,
                10.0,
                &mut rand::rngs::SmallRng::seed_from_u64(seed)
            ),
            ReproductionActionResult::Conceived
        );
        (
            sim.stats.graph_edges_detached_total,
            sim.stats.groups_removed_total,
        )
    }

    /// A birth's member detaches and group removals reach the run totals:
    /// some seed's mutated birth detaches a member and some seed's removes a
    /// group (T25.F03).
    #[test]
    fn a_births_detaches_and_group_removals_reach_the_run_totals() {
        let (mut detached, mut removed) = (None, None);
        for seed in 0..64 {
            let (d, r) = grouped_birth_totals(seed);
            detached = detached.or((d > 0).then_some(seed));
            removed = removed.or((r > 0).then_some(seed));
            if detached.is_some() && removed.is_some() {
                break;
            }
        }
        assert!(detached.is_some(), "no birth detached a member");
        assert!(removed.is_some(), "no birth removed a group");
    }

    /// The founder with an area-barrier reference and one `DirectionPair`
    /// group at `scale` on its Graph node.
    fn pair_grouped_founder(scale: f32) -> crate::creature::genome::CreatureGenome {
        use crate::contracts::{InputReference, WorldInputKey};
        use crate::creature::founder::v3alpha1_founder_genome;
        use crate::creature::genome::vote::DirectedVoteKind;

        let mut genome = v3alpha1_founder_genome();
        let node = genome
            .nodes
            .iter_mut()
            .find(|node| matches!(node.backend_def, BackendDef::Graph(_)))
            .expect("the founder carries a Graph node");
        let reference = InputReference::World(WorldInputKey::AreaBarrierSummary);
        let ref_idx = u16::try_from(node.input_refs.len()).unwrap();
        let layout = reference.direction_pairs()[0];
        node.input_refs.push(reference);
        let BackendDef::Graph(def) = &mut node.backend_def else {
            unreachable!("a Graph node")
        };
        let id = def.add_pair_group(ref_idx, 0, layout, DirectedVoteKind::Move, 3);
        def.group_mut(id).set_scale(scale);
        genome
    }

    fn applied(
        ops: &[crate::mutation::types::MutationOperator],
    ) -> crate::mutation::types::MutationSummary {
        let mut summary = crate::mutation::types::MutationSummary::zero();
        for &op in ops {
            summary.attempted_by_operator.insert(op, 1);
            summary.applied_by_operator.insert(op, 1);
        }
        summary
    }

    #[test]
    fn the_inheritance_witness_counts_an_inherited_nonzero_pair_group_only() {
        use crate::mutation::types::MutationOperator as Op;
        let nonzero = pair_grouped_founder(0.5);
        let zero = pair_grouped_founder(1e-3);
        // No creation event and a nonzero inherited group: counted.
        assert!(inherits_nonzero_pair_group(&nonzero, &applied(&[])));
        assert!(inherits_nonzero_pair_group(
            &nonzero,
            &applied(&[Op::GraphAlterGroupScale])
        ));
        // A committed creation event: not counted, even with a nonzero group.
        for op in [Op::GraphAddDirectionPairGroup, Op::GraphRecruitNeutralInput] {
            assert!(!inherits_nonzero_pair_group(&nonzero, &applied(&[op])));
        }
        // A rolled-back creation event (attempted, skipped) does not suppress.
        let mut rolled_back = applied(&[]);
        rolled_back
            .attempted_by_operator
            .insert(Op::GraphAddDirectionPairGroup, 1);
        rolled_back
            .skipped_by_operator
            .insert(Op::GraphAddDirectionPairGroup, 1);
        assert!(inherits_nonzero_pair_group(&nonzero, &rolled_back));
        // A scale at or below the floor, and no pair group at all: not counted.
        assert!(!inherits_nonzero_pair_group(&zero, &applied(&[])));
        assert!(!inherits_nonzero_pair_group(
            &crate::creature::founder::v3alpha1_founder_genome(),
            &applied(&[])
        ));
    }

    /// A mutation-free birth from a parent carrying a nonzero pair group
    /// reaches the run's witness total.
    #[test]
    fn an_unmutated_birth_of_a_pair_group_carrier_counts_once() {
        use crate::config::SimulationConfig;
        use crate::simulation::seed_simulation;
        use rand::SeedableRng;

        let mut cfg = SimulationConfig::default();
        cfg.population.initial_creatures = 1;
        cfg.mutation.per_unit_rate = 0.0;
        cfg.energy.lifecycle.gestation_ticks_per_founder_genome = 0.0;
        let mut sim = seed_simulation(cfg, 83);
        let parent_id = sim.creatures.keys().next().unwrap();
        let parent = &mut sim.creatures[parent_id];
        parent.genome = pair_grouped_founder(-0.25);
        parent.age = sim.config.energy.lifecycle.min_reproduce_age;
        parent.energy = 10000.0;
        let direction = Direction::ALL
            .into_iter()
            .find(|&direction| {
                sim.world
                    .resolve_neighbor(sim.creatures[parent_id].position, direction)
                    .is_some_and(|p| sim.world.is_valid_target_cell(p))
            })
            .unwrap();
        assert_eq!(
            apply_reproduce(
                parent_id,
                &mut sim,
                direction,
                10.0,
                &mut rand::rngs::SmallRng::seed_from_u64(1)
            ),
            ReproductionActionResult::Conceived
        );
        assert_eq!(sim.stats.pair_group_inherited_nonzero_births_total, 1);
    }

    /// A birth's committed attribute changes reach the run's totals by the
    /// scope they landed on (T25.F08 invariant 8): over a pair-grouped
    /// parent's births, the two totals sum to the applied events of the ten
    /// attribute operators, and both scopes are reached.
    #[test]
    fn a_births_attribute_changes_reach_the_run_totals_by_scope() {
        use crate::config::SimulationConfig;
        use crate::mutation::types::MutationOperator as Op;
        use crate::simulation::seed_simulation;
        use rand::SeedableRng;

        const ATTRIBUTE: [Op; 10] = [
            Op::GraphEnableHebbian,
            Op::GraphDisableHebbian,
            Op::GraphMutateHebbianRule,
            Op::GraphMutateHebbianRate,
            Op::GraphToggleHebbianLamarckian,
            Op::GraphEnableRewardModulation,
            Op::GraphDisableRewardModulation,
            Op::GraphMutateRewardSource,
            Op::GraphMutateTraceDecay,
            Op::GraphSetScope,
        ];
        let mut reached = [false; 2];
        for seed in 0..64 {
            let mut cfg = SimulationConfig::default();
            cfg.population.initial_creatures = 1;
            cfg.mutation.per_unit_rate = 1.0;
            cfg.energy.lifecycle.gestation_ticks_per_founder_genome = 0.0;
            let mut sim = seed_simulation(cfg, 83);
            let parent_id = sim.creatures.keys().next().unwrap();
            let parent = &mut sim.creatures[parent_id];
            parent.genome = pair_grouped_founder(-0.25);
            parent.age = sim.config.energy.lifecycle.min_reproduce_age;
            parent.energy = 10000.0;
            let direction = Direction::ALL
                .into_iter()
                .find(|&direction| {
                    sim.world
                        .resolve_neighbor(sim.creatures[parent_id].position, direction)
                        .is_some_and(|p| sim.world.is_valid_target_cell(p))
                })
                .unwrap();
            assert_eq!(
                apply_reproduce(
                    parent_id,
                    &mut sim,
                    direction,
                    10.0,
                    &mut rand::rngs::SmallRng::seed_from_u64(seed)
                ),
                ReproductionActionResult::Conceived
            );
            let applied: u64 = ATTRIBUTE
                .iter()
                .filter_map(|op| sim.stats.mutation_events_applied_total_by_operator.get(op))
                .sum();
            let totals = sim.stats.mutation_attribute_changes_total_by_scope;
            assert_eq!(totals.iter().sum::<u64>(), applied, "seed {seed}");
            for (seen, total) in reached.iter_mut().zip(totals) {
                *seen |= total > 0;
            }
            if reached == [true; 2] {
                break;
            }
        }
        assert_eq!(reached, [true; 2], "both scopes reached");
    }
}
