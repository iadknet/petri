use crate::config::runtime::mutation::MutationConfig;
use crate::creature::genome::{BackendDef, CreatureGenome};
use rand::Rng;

/// Apply mutation to a genome according to config.
///
/// Rolls against `mutation_probability` to determine if any mutation occurs.
/// If triggered, applies `per_birth_mutation_events_min..=max` jitter events.
/// Each event picks a random node with a non-empty constants pool and jitters
/// one constant by ±`constant_jitter_magnitude`.
pub fn mutate_genome(genome: &mut CreatureGenome, config: &MutationConfig, rng: &mut impl Rng) {
    if rng.gen::<f64>() >= config.mutation_probability {
        return;
    }

    // Collect indices of nodes that have mutable constants.
    let mutable_node_indices: Vec<usize> = genome
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| match &node.backend_def {
            BackendDef::Vm(vm_def) => !vm_def.constants.is_empty(),
        })
        .map(|(i, _)| i)
        .collect();

    if mutable_node_indices.is_empty() {
        return;
    }

    let event_count =
        rng.gen_range(config.per_birth_mutation_events_min..=config.per_birth_mutation_events_max);

    for _ in 0..event_count {
        let node_idx = mutable_node_indices[rng.gen_range(0..mutable_node_indices.len())];
        let node = &mut genome.nodes[node_idx];
        match &mut node.backend_def {
            BackendDef::Vm(vm_def) => {
                let const_idx = rng.gen_range(0..vm_def.constants.len());
                let jitter = rng.gen_range(
                    -config.constant_jitter_magnitude..=config.constant_jitter_magnitude,
                );
                vm_def.constants[const_idx] += jitter;
                // Clamp to avoid NaN/Inf drift.
                vm_def.constants[const_idx] = vm_def.constants[const_idx].clamp(-1e6, 1e6);
            }
        }
    }
}
