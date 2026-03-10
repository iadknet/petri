use rand::Rng;

use crate::creature::genome::{BackendDef, CreatureGenome};
use crate::mutation::types::MutationSkipReason;

// --- Old graph operators: stubbed as no-ops for Graph backend ---
// CGP replacement lives in cgp_operators.rs. These stubs exist only to keep
// the dispatch in graph/mod.rs compiling until it is rewired to CGP operators.

pub(super) fn alter_edge_weight(
    genome: &mut CreatureGenome,
    node_idx: usize,
    _rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(_) = genome.nodes[node_idx].backend_def {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    Ok(())
}

pub(super) fn swap_operator(
    genome: &mut CreatureGenome,
    node_idx: usize,
    _rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(_) = genome.nodes[node_idx].backend_def {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    Ok(())
}

pub(super) fn mutate_operator_param(
    genome: &mut CreatureGenome,
    node_idx: usize,
    _rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(_) = genome.nodes[node_idx].backend_def {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    Ok(())
}

pub(super) fn add_internal_node(
    genome: &mut CreatureGenome,
    node_idx: usize,
    _rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(_) = genome.nodes[node_idx].backend_def {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    Ok(())
}

pub(super) fn add_graph_edge(
    genome: &mut CreatureGenome,
    node_idx: usize,
    _rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(_) = genome.nodes[node_idx].backend_def {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    Ok(())
}

pub(super) fn remove_internal_node(
    genome: &mut CreatureGenome,
    node_idx: usize,
    _rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(_) = genome.nodes[node_idx].backend_def {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    Ok(())
}

pub(super) fn retarget_graph_edge(
    genome: &mut CreatureGenome,
    node_idx: usize,
    _rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(_) = genome.nodes[node_idx].backend_def {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    Ok(())
}

pub(super) fn remove_graph_edge(
    genome: &mut CreatureGenome,
    node_idx: usize,
    _rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(_) = genome.nodes[node_idx].backend_def {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    Ok(())
}

pub(super) fn apply_graph_raw_field_mutation(
    genome: &mut CreatureGenome,
    node_idx: usize,
    _rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(_) = genome.nodes[node_idx].backend_def {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    Ok(())
}

pub(super) fn apply_copy_internal_node(
    genome: &mut CreatureGenome,
    node_idx: usize,
    _rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(_) = genome.nodes[node_idx].backend_def {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    Ok(())
}

pub(super) fn apply_copy_subgraph(
    genome: &mut CreatureGenome,
    node_idx: usize,
    _rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(_) = genome.nodes[node_idx].backend_def {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    Ok(())
}

pub(super) fn apply_copy_edge_bundle(
    genome: &mut CreatureGenome,
    node_idx: usize,
    _rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(_) = genome.nodes[node_idx].backend_def {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    Ok(())
}
