use rand::Rng;

use super::cgp_reproduction;
use crate::contracts::{CreatureId, Direction};
use crate::creature::action_log::ActionLog;
use crate::creature::founder::FOUNDER_GENOME_SIZE_UNITS;
use crate::creature::genome::{BackendDef, CreatureGenome};
use crate::creature::identity::CreatureIdentityState;
use crate::creature::state::CreatureState;
use crate::mutation::phenotype::mutate_phenotype;
use crate::mutation::reachability::ParentExecuted;
use crate::mutation::MutationEngine;
use crate::runtime::action_decode::clamp_unit_interval;
use crate::simulation::simulation::Simulation;

/// Capture birth-local values before any structural mutation.
fn capture_birth_weights(genome: &mut CreatureGenome, parent: &[Vec<Box<[f32]>>]) {
    for (idx, node) in genome.nodes.iter_mut().enumerate() {
        if let BackendDef::Graph(def) = &mut node.backend_def {
            cgp_reproduction::capture_birth_weights(
                def,
                parent.get(idx).map_or(&[], Vec::as_slice),
            );
        }
    }
}

/// Consume every backend's birth correspondence before constructing the newborn.
fn build_child_plasticity_weights(child_genome: &mut CreatureGenome) -> Vec<Vec<Box<[f32]>>> {
    let mut result = Vec::new();
    for (idx, node) in child_genome.nodes.iter_mut().enumerate() {
        if let BackendDef::Graph(def) = &mut node.backend_def {
            result.resize_with(idx + 1, Vec::new);
            result[idx] = cgp_reproduction::build_cgp_child_plasticity_weights(def);
        }
    }
    result
}

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

/// Result of an attempted reproduction action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReproductionActionResult {
    Spawned,
    RejectedInvalidTarget,
    RejectedAgeConstraints,
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

/// Apply a Reproduce action per v3-reproduction-spec.md Section 6 unified sequence.
///
/// `energy_transfer_fraction` is the share of the parent's post-cost energy
/// the child starts with, in [0, 1] (T17.F01); the litter is capped at
/// `default_offspring_energy` and refused under `initial_energy`.
///
/// Returns the outcome indicating whether offspring was spawned or why it was rejected.
#[must_use]
#[allow(
    clippy::too_many_lines,
    reason = "the unified reproduction sequence in v3-reproduction-spec.md \
              Section 6 is ordered end to end; splitting it would obscure the \
              spec-mandated gate order"
)]
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
    // parent's energy after it. Nothing is deducted until both gates pass
    // (T16.F01): a rejected attempt leaves the parent untouched.
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
    // conceived, and the refusal is free.
    let fraction = clamp_unit_interval(energy_transfer_fraction);
    let transfer =
        (fraction * after_cost).min(sim.config.energy.lifecycle.default_offspring_energy);

    if transfer <= 0.0 || transfer < sim.config.energy.lifecycle.initial_energy {
        sim.stats.reproduction_actions_rejected_total += 1;
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
    sim.stats.energy_flows.action_charges.reproduce += sim.creatures[parent_id].observe_energy(
        energy_before_cost,
        crate::simulation::energy_accounting::DeathCause::ActionReproduce,
    );
    sim.creatures[parent_id].energy = after_cost - transfer;
    sim.stats.energy_flows.parental_transfer_debit += sim.creatures[parent_id].observe_energy(
        after_cost,
        crate::simulation::energy_accounting::DeathCause::ParentalTransfer,
    );

    // Step 9: Build offspring draft (clone parent genome + state).
    let child_genome = sim.creatures[parent_id].genome.clone();
    let child_shared_memory = sim.creatures[parent_id].shared_memory;
    let child_generation = sim.creatures[parent_id].generation + 1;
    let child_channels = sim.creatures[parent_id].phenotype_channels;
    let child_active_channel = sim.creatures[parent_id].phenotype_active_channel;
    let child_polarity = sim.creatures[parent_id].phenotype_channel_polarity;
    // Snapshot parent's learned plasticity weights before genome mutation.
    let parent_plasticity = sim.creatures[parent_id]
        .graph_runtime
        .plasticity_weights
        .clone();
    let parent_cached_reachable = sim.creatures[parent_id].cached_reachable_nodes.clone();
    // T11.F17: the parent's own recent dispatches bias the offspring's targets.
    // Passed unresolved, so a zero-event birth derives nothing.
    let parent = &sim.creatures[parent_id];
    let parent_executed = ParentExecuted::Record(&parent.graph_runtime.dispatch_record, parent.age);

    // Step 10: Apply genome mutations.
    let mut child_genome = child_genome;
    capture_birth_weights(&mut child_genome, &parent_plasticity);
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
    let mut child_birth_mutation_operators: Vec<_> =
        summary.applied_by_operator.keys().copied().collect();
    child_birth_mutation_operators.sort_by_key(|operator| operator.as_key());
    let child_birth_mutation_operators = child_birth_mutation_operators.into_boxed_slice();

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

    // Step 12: Build child's plasticity weights (Lamarckian inheritance).
    let child_plasticity = build_child_plasticity_weights(&mut child_genome);

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
        child.graph_runtime.plasticity_weights = child_plasticity;
        child.birth_mutation_operators = child_birth_mutation_operators;
        child
    });
    sim.stats.energy_flows.offspring_energy_credit += f64::from(sim.creatures[child_id].energy);
    sim.action_logs
        .insert(child_id, ActionLog::new(sim.config.action_log.capacity));
    sim.world.place_creature(target, child_id);
    if let Some(parent) = sim.creatures.get_mut(parent_id) {
        parent.offspring_spawned_count += 1;
    }

    sim.stats.reproduction_actions_spawned_total += 1;
    ReproductionActionResult::Spawned
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MutationConfig;
    use crate::creature::genome::cgp::{ComputeNode, ComputeNodeKind, GraphEdge, GraphSource};
    use crate::creature::genome::{HebbianRule, PlasticityConfig};

    fn build_child_plasticity_weights(
        genome: &CreatureGenome,
        parent: &[Vec<Box<[f32]>>],
    ) -> Vec<Vec<Box<[f32]>>> {
        let mut child = genome.clone();
        capture_birth_weights(&mut child, parent);
        super::build_child_plasticity_weights(&mut child)
    }

    /// Helper: builds a genome with a single Graph mesh node containing the given compute nodes.
    fn genome_with_cgp_compute_nodes(compute_nodes: Vec<ComputeNode>) -> CreatureGenome {
        use crate::contracts::NodeId;
        use crate::creature::genome::NodeGenome;

        let mut def = crate::creature::genome::cgp::CgpGraphBackendDef::new_with_fixed_outputs();
        def.compute_nodes = compute_nodes;

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
        for size in [0, 1, 7, 96] {
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
    fn replication_multiplier_charges_the_reference_genome_29_9_times() {
        // The T03.F08 cost arm's 386-unit genome: 289 units above the 97-unit
        // founder (T19.F04).
        let factor = genome_replication_cost_multiplier(0.1, 386);
        assert!((factor - 29.9).abs() < 1e-4, "factor {factor}");
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

    /// T16.F01 invariant 4: `action_charges.reproduce` accumulates only over
    /// births. With the energy gate above `max_energy`, every attempt is an
    /// energy rejection and the flow stays at zero across the tick loop.
    #[test]
    fn energy_rejected_attempts_leave_the_reproduce_flow_at_zero() {
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
        // Exact zero is the claim: no observe_energy crossing was recorded.
        assert_eq!(
            sim.stats.energy_flows.action_charges.reproduce, 0.0,
            "rejected attempts must not contribute to action_charges.reproduce"
        );
    }

    #[test]
    fn lamarckian_copies_parent_learned_weights() {
        let genome = genome_with_cgp_compute_nodes(vec![
            ComputeNode {
                kind: ComputeNodeKind::Constant(1.0),
                inputs: vec![],
                plasticity: None,
            },
            ComputeNode {
                kind: ComputeNodeKind::Add,
                inputs: vec![GraphEdge {
                    source: GraphSource::ComputeNode(0),
                    weight: 0.5,
                }],
                plasticity: Some(PlasticityConfig {
                    rule: HebbianRule::Classic,
                    learning_rate: 0.1,
                    weight_clamp: 5.0,
                    lamarckian: true,
                    modulation: None,
                }),
            },
        ]);

        // Parent has learned weight 0.9 (drifted from genome 0.5).
        let parent_plasticity: Vec<Vec<Box<[f32]>>> = vec![vec![
            Box::new([]) as Box<[f32]>, // CN0: no Hebbian
            Box::new([0.9]),            // CN1: learned weight
        ]];

        let child_hw = build_child_plasticity_weights(&genome, &parent_plasticity);

        assert_eq!(child_hw.len(), 1);
        assert_eq!(child_hw[0].len(), 2);
        // CN0 (no Hebbian): empty
        assert!(child_hw[0][0].is_empty());
        // CN1 (Lamarckian): copied from parent
        assert_eq!(child_hw[0][1].len(), 1);
        assert!((child_hw[0][1][0] - 0.9).abs() < 1e-6);
    }

    #[test]
    fn darwinian_resets_to_empty() {
        let genome = genome_with_cgp_compute_nodes(vec![
            ComputeNode {
                kind: ComputeNodeKind::Constant(1.0),
                inputs: vec![],
                plasticity: None,
            },
            ComputeNode {
                kind: ComputeNodeKind::Add,
                inputs: vec![GraphEdge {
                    source: GraphSource::ComputeNode(0),
                    weight: 0.5,
                }],
                plasticity: Some(PlasticityConfig {
                    rule: HebbianRule::Classic,
                    learning_rate: 0.1,
                    weight_clamp: 5.0,
                    lamarckian: false, // Darwinian
                    modulation: None,
                }),
            },
        ]);

        // Parent has learned weight 0.9.
        let parent_plasticity: Vec<Vec<Box<[f32]>>> =
            vec![vec![Box::new([]) as Box<[f32]>, Box::new([0.9])]];

        let child_hw = build_child_plasticity_weights(&genome, &parent_plasticity);

        assert_eq!(child_hw.len(), 1);
        assert_eq!(child_hw[0].len(), 2);
        // CN0 (no Hebbian): empty
        assert!(child_hw[0][0].is_empty());
        // CN1 (Darwinian): empty — will lazy-init from genome weights on first tick
        assert!(child_hw[0][1].is_empty());
    }

    #[test]
    fn lamarckian_with_no_parent_weights_returns_empty() {
        let genome = genome_with_cgp_compute_nodes(vec![ComputeNode {
            kind: ComputeNodeKind::Add,
            inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 0.5,
            }],
            plasticity: Some(PlasticityConfig {
                rule: HebbianRule::Oja,
                learning_rate: 0.1,
                weight_clamp: 5.0,
                lamarckian: true,
                modulation: None,
            }),
        }]);

        // Parent never ran graph execution — no Hebbian weights.
        let parent_plasticity: Vec<Vec<Box<[f32]>>> = Vec::new();

        let child_hw = build_child_plasticity_weights(&genome, &parent_plasticity);

        // Result should have entry for mesh node 0 with empty inner (lazy init)
        assert_eq!(child_hw.len(), 1);
        assert_eq!(child_hw[0].len(), 1);
        assert!(child_hw[0][0].is_empty());
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

        let parent_plasticity: Vec<Vec<Box<[f32]>>> = Vec::new();
        let child_hw = build_child_plasticity_weights(&genome, &parent_plasticity);

        // VM nodes produce no Hebbian weight entries
        assert!(child_hw.is_empty());
    }
    #[test]
    fn birth_mesh_copies_removal_and_backend_replacement_keep_their_own_values() {
        use crate::contracts::{NodeId, RouteTarget};
        use crate::mutation::reachability::TargetSelector;
        use crate::mutation::topology::{TopologyMutator, TopologyOperator};
        use rand::SeedableRng;
        let compute = ComputeNode {
            kind: ComputeNodeKind::Add,
            inputs: vec![GraphEdge {
                source: GraphSource::SharedMemory {
                    slot: 0,
                    previous: false,
                },
                weight: 0.1,
            }],
            plasticity: Some(PlasticityConfig {
                rule: HebbianRule::Classic,
                learning_rate: 0.1,
                weight_clamp: 5.0,
                lamarckian: true,
                modulation: None,
            }),
        };
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
            def.compute_nodes[0].inputs[0].weight = idx as f32 + 1.0;
        }
        let parent: Vec<Vec<Box<[f32]>>> = vec![
            vec![Box::new([11.0])],
            vec![Box::new([12.0])],
            vec![Box::new([13.0])],
        ];
        for op in [
            TopologyOperator::CopyNode,
            TopologyOperator::CopyMeshBackwardSlice,
            TopologyOperator::CopyMeshForwardSlice,
            TopologyOperator::RemoveNode,
        ] {
            let mut child = original.clone();
            capture_birth_weights(&mut child, &parent);
            TopologyMutator::apply(
                &mut child,
                op,
                &mut TargetSelector::reachable_only(&[0, 1, 2], 0.0),
                &mut rand::rngs::SmallRng::seed_from_u64(73),
                &MutationConfig::default(),
            )
            .unwrap();
            let result = super::build_child_plasticity_weights(&mut child);
            for (idx, node) in child.nodes.iter().enumerate() {
                let BackendDef::Graph(def) = &node.backend_def else {
                    unreachable!()
                };
                assert_eq!(
                    result[idx][0][0],
                    def.compute_nodes[0].inputs[0].weight + 10.0
                );
                assert!(def.birth_weights.is_none());
            }
        }
        let mut child = original.clone();
        capture_birth_weights(&mut child, &parent);
        // Replacing a backend at the same mesh index/ID gives it no correspondence.
        child.nodes[0].backend_def = original.nodes[0].backend_def.clone();
        let result = super::build_child_plasticity_weights(&mut child);
        assert!(result[0][0].is_empty());
        assert_eq!(result[1][0][0], 12.0);
    }

    #[test]
    fn birth_applied_reproduction_keeps_parent_and_resets_all_newborn_credit() {
        use crate::config::SimulationConfig;
        use crate::simulation::seed_simulation;
        use rand::SeedableRng;
        let mut cfg = SimulationConfig::default();
        cfg.population.initial_creatures = 1;
        cfg.mutation.per_unit_rate = 0.0;
        let mut sim = seed_simulation(cfg, 83);
        let parent_id = sim.creatures.keys().next().unwrap();
        let genome = genome_with_cgp_compute_nodes(vec![ComputeNode {
            kind: ComputeNodeKind::Add,
            inputs: vec![GraphEdge {
                source: GraphSource::SharedMemory {
                    slot: 0,
                    previous: false,
                },
                weight: 0.5,
            }],
            plasticity: Some(PlasticityConfig {
                rule: HebbianRule::Classic,
                learning_rate: 0.1,
                weight_clamp: 5.0,
                lamarckian: true,
                modulation: None,
            }),
        }]);
        let parent = &mut sim.creatures[parent_id];
        parent.genome = genome.clone();
        parent.age = sim.config.energy.lifecycle.min_reproduce_age;
        parent.energy = 10000.0;
        parent.graph_runtime.plasticity_weights = vec![vec![Box::new([0.9])]];
        parent.graph_runtime.eligibility_traces = vec![vec![Box::new([99.0])]];
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
                &mut rand::rngs::SmallRng::seed_from_u64(2)
            ),
            ReproductionActionResult::Spawned
        );
        let child = sim
            .creatures
            .iter()
            .find(|(id, _)| *id != parent_id)
            .unwrap()
            .1;
        assert_eq!(child.graph_runtime.plasticity_weights[0][0][0], 0.9);
        assert!(child.graph_runtime.eligibility_traces.is_empty());
        assert!(child.graph_runtime.node_state.is_empty());
        assert!(child.graph_runtime.node_outputs.is_empty());
        assert!(child.graph_runtime.dispatch_record.is_empty());
        let BackendDef::Graph(def) = &child.genome.nodes[0].backend_def else {
            unreachable!()
        };
        assert!(def.birth_weights.is_none());
        assert_eq!(sim.creatures[parent_id].genome, genome);
        assert_eq!(
            sim.creatures[parent_id].graph_runtime.eligibility_traces[0][0][0],
            99.0
        );
        assert_eq!(
            sim.creatures[parent_id].graph_runtime.plasticity_weights[0][0][0],
            0.9
        );
    }
}
