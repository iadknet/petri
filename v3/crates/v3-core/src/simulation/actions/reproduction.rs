use rand::Rng;

use crate::contracts::{CreatureId, Direction};
use crate::creature::state::CreatureState;
use crate::mutation::phenotype::mutate_phenotype;
use crate::mutation::MutationEngine;
use crate::simulation::simulation::Simulation;

/// Result of an attempted reproduction action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReproductionActionResult {
    Spawned,
    RejectedInvalidTarget,
    RejectedEnergyConstraints,
    RejectedPopulationCap,
}

impl ReproductionActionResult {
    /// Stable string key used by transport/API boundaries.
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Spawned => "Spawned",
            Self::RejectedInvalidTarget => "RejectedInvalidTarget",
            Self::RejectedEnergyConstraints => "RejectedEnergyConstraints",
            Self::RejectedPopulationCap => "RejectedPopulationCap",
        }
    }
}

/// Apply a Reproduce action per v3-reproduction-spec.md Section 6 unified sequence.
///
/// Returns the outcome indicating whether offspring was spawned or why it was rejected.
#[must_use]
pub fn apply_reproduce(
    parent_id: CreatureId,
    sim: &mut Simulation,
    dir: Direction,
    energy_transfer_request: f32,
    rng: &mut impl Rng,
) -> ReproductionActionResult {
    // Stats: always count attempt and per-tick reproduce regardless of outcome.
    sim.stats.reproduction_actions_attempted_total += 1;
    sim.stats.last_tick_reproduce += 1;

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

    // Step 4: Deduct reproduce_cost from parent.
    sim.creatures[parent_id].energy -= sim.config.energy.costs.reproduce_cost;

    // Step 5: Check parent has sufficient energy after cost deduction.
    if sim.creatures[parent_id].energy < sim.config.energy.lifecycle.min_reproduce_energy {
        sim.stats.reproduction_actions_rejected_total += 1;
        *sim.stats
            .reproduction_actions_rejected_by_reason
            .entry(ReproductionActionResult::RejectedEnergyConstraints)
            .or_insert(0) += 1;
        return ReproductionActionResult::RejectedEnergyConstraints;
    }

    // Step 6: Compute energy transfer (clamped to [0, default_offspring_energy]).
    let max_transfer = sim.config.energy.lifecycle.default_offspring_energy;
    let transfer = if energy_transfer_request.is_finite() && energy_transfer_request > 0.0 {
        energy_transfer_request.min(max_transfer)
    } else {
        0.0
    };

    if transfer <= 0.0 || sim.creatures[parent_id].energy < transfer {
        sim.stats.reproduction_actions_rejected_total += 1;
        *sim.stats
            .reproduction_actions_rejected_by_reason
            .entry(ReproductionActionResult::RejectedEnergyConstraints)
            .or_insert(0) += 1;
        return ReproductionActionResult::RejectedEnergyConstraints;
    }

    // Step 7: Deduct transfer from parent.
    sim.creatures[parent_id].energy -= transfer;

    // Step 8: Build offspring draft (clone parent genome + state).
    let child_genome = sim.creatures[parent_id].genome.clone();
    let child_memory = sim.creatures[parent_id].memory;
    let child_generation = sim.creatures[parent_id].generation + 1;
    let child_channels = sim.creatures[parent_id].phenotype_channels;
    let child_active_channel = sim.creatures[parent_id].phenotype_active_channel;
    let child_polarity = sim.creatures[parent_id].phenotype_channel_polarity;

    // Step 9: Apply genome mutations.
    let mut child_genome = child_genome;
    let summary = MutationEngine::apply_mutations(&mut child_genome, &sim.config.mutation, rng);

    // Update mutation stats.
    sim.stats.mutation_events_attempted_total += summary.attempted_events as u64;
    sim.stats.mutation_events_applied_total += summary.applied_events as u64;
    sim.stats.mutation_events_skipped_total += summary.skipped_events as u64;
    sim.stats.mutation_events_applied_total_semantic_noop +=
        summary.applied_semantic_noop_events as u64;
    sim.stats.mutation_events_applied_total_semantic_change +=
        summary.applied_semantic_change_events as u64;
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

    // Step 10: Phenotype mutation — triggered only when at least one genome event was applied.
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

    // Step 11–12: Spawn child in slotmap + world.
    let child_id = sim.creatures.insert_with_key(|id| {
        let mut child = CreatureState::new(
            id,
            child_genome,
            target,
            transfer,
            child_generation,
            child_channels,
            child_active_channel,
            child_polarity,
        );
        child.memory = child_memory;
        child
    });
    sim.world.place_creature(target, child_id);

    sim.stats.reproduction_actions_spawned_total += 1;
    ReproductionActionResult::Spawned
}
