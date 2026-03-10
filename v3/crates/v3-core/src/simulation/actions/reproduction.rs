use rand::Rng;

use super::cgp_reproduction;
use crate::contracts::{CreatureId, Direction};
use crate::creature::action_log::ActionLog;
use crate::creature::genome::{BackendDef, CreatureGenome};
use crate::creature::identity::CreatureIdentityState;
use crate::creature::state::CreatureState;
use crate::mutation::phenotype::mutate_phenotype;
use crate::mutation::MutationEngine;
use crate::simulation::simulation::Simulation;

/// Build child plasticity weights from parent state, respecting Lamarckian/Darwinian inheritance.
///
/// For each mesh node with a Graph backend, inspects each internal node:
/// - `plasticity.lamarckian == true`: copies parent's learned weights if available
/// - `plasticity.lamarckian == false` or `plasticity == None`: empty `Box<[f32]>` (reinit from genome on first tick)
fn build_child_plasticity_weights(
    child_genome: &CreatureGenome,
    parent_plasticity: &[Vec<Box<[f32]>>],
) -> Vec<Vec<Box<[f32]>>> {
    let mut result: Vec<Vec<Box<[f32]>>> = Vec::new();

    for (mesh_idx, mesh_node) in child_genome.nodes.iter().enumerate() {
        let cgp_def = match &mesh_node.backend_def {
            BackendDef::Graph(g) => g,
            _ => continue, // VM nodes have no plasticity weights
        };

        // Ensure result covers this mesh node index.
        if result.len() <= mesh_idx {
            result.resize_with(mesh_idx + 1, Vec::new);
        }

        let parent_weights_for_node = parent_plasticity
            .get(mesh_idx)
            .map_or(&[] as &[_], |v| v.as_slice());
        let inner =
            cgp_reproduction::build_cgp_child_plasticity_weights(cgp_def, parent_weights_for_node);

        result[mesh_idx] = inner;
    }

    result
}

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

    // Step 4: Deduct reproduce_cost from parent (scaled by genome complexity and age).
    sim.creatures[parent_id].energy -= sim.config.energy.adjusted_action_cost(
        sim.config.energy.costs.reproduce_cost,
        sim.creatures[parent_id].cached_complexity,
        sim.creatures[parent_id].age,
    );

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

    // Step 9: Apply genome mutations.
    let mut child_genome = child_genome;
    let summary = MutationEngine::apply_mutations(
        &mut child_genome,
        &sim.config.mutation,
        &parent_cached_reachable,
        rng,
    );

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
    sim.stats.mutation_reachable_target_total += summary.reachable_target_events as u64;
    sim.stats.mutation_unreachable_target_total += summary.unreachable_target_events as u64;
    sim.stats.mutation_not_applicable_target_total += summary.not_applicable_events as u64;

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

    // Step 10b: Identity inheritance — child inherits parent identity, kin-tag
    // mutates only when genome mutation applied events.
    let parent_identity = sim.creatures[parent_id].identity;
    let mut child_identity = CreatureIdentityState::inherit(&parent_identity);
    if summary.applied_events > 0 {
        child_identity.mutate_kin_tag(rng);
    }

    // Step 11: Build child's plasticity weights (Lamarckian inheritance).
    let child_plasticity = build_child_plasticity_weights(&child_genome, &parent_plasticity);

    // Step 12–13: Spawn child in slotmap + world.
    // No-mutation fast path: if no genome mutations were applied, the offspring's
    // genome is identical to the parent's — copy cached_complexity to avoid
    // expensive recomputation.
    let parent_cached_complexity = sim.creatures[parent_id].cached_complexity;
    let child_id = sim.creatures.insert_with_key(|id| {
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
        child
    });
    sim.action_logs
        .insert(child_id, ActionLog::new(sim.config.action_log.capacity));
    sim.world.place_creature(target, child_id);

    sim.stats.reproduction_actions_spawned_total += 1;
    ReproductionActionResult::Spawned
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MutationConfig;
    use crate::creature::genome::cgp::{ComputeNode, ComputeNodeKind, GraphEdge, GraphSource};
    use crate::creature::genome::{HebbianRule, PlasticityConfig};

    /// Helper: builds a genome with a single Graph mesh node containing the given compute nodes.
    fn genome_with_cgp_compute_nodes(compute_nodes: Vec<ComputeNode>) -> CreatureGenome {
        use crate::contracts::NodeId;
        use crate::creature::genome::NodeGenome;

        let config = MutationConfig::default();
        let mut def =
            crate::creature::genome::cgp::CgpGraphBackendDef::new_with_fixed_outputs(&config);
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
}
