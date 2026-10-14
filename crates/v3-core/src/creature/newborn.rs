//! The newborn's runtime weights from its draft's birth payload (T25.F01).
//!
//! The creature constructors take the [`BirthSeeds`] out of the genome they
//! receive and build `plasticity_weights` here, so reproduction writes no
//! runtime field and a built creature's genome carries no seeds.

use crate::creature::genome::cgp::{BirthSeeds, CgpGraphBackendDef};
use crate::creature::genome::{BackendDef, CreatureGenome};

/// Take every graph node's seeds out of `genome` and build the newborn's
/// `plasticity_weights`.
///
/// A genome with no seeds at all (seeding, the lab, instruments, observation
/// clones) gets the empty outer vector. A reproduction draft gets one row per
/// graph node (up to the last one, other nodes empty) and one slice per
/// compute node: empty unless the node is Lamarckian and inherited at least
/// one value, otherwise one value per edge, the genomic weight filling the
/// gaps.
pub(crate) fn take_plasticity_weights(genome: &mut CreatureGenome) -> Vec<Vec<Box<[f32]>>> {
    let birth = genome.nodes.iter().any(
        |node| matches!(&node.backend_def, BackendDef::Graph(def) if def.birth_seeds().is_some()),
    );
    if !birth {
        return Vec::new();
    }
    let mut result = Vec::new();
    for (idx, node) in genome.nodes.iter_mut().enumerate() {
        if let BackendDef::Graph(def) = &mut node.backend_def {
            result.resize_with(idx + 1, Vec::new);
            result[idx] = take_graph_plasticity_weights(def);
        }
    }
    result
}

/// Consume one graph's seeds into its per-compute-node weight slices.
pub(crate) fn take_graph_plasticity_weights(def: &mut CgpGraphBackendDef) -> Vec<Box<[f32]>> {
    let seeds = def.take_birth_seeds();
    let rows = seeds.as_ref().and_then(BirthSeeds::rows);
    def.compute_nodes()
        .iter()
        .enumerate()
        .map(|(idx, node)| {
            let values = rows.and_then(|rows| rows.get(idx));
            if !node.plasticity.as_ref().is_some_and(|cfg| cfg.lamarckian)
                || !values.is_some_and(|values| values.iter().any(Option::is_some))
            {
                return Box::default();
            }
            node.inputs
                .iter()
                .enumerate()
                .map(|(edge, genomic)| {
                    values
                        .and_then(|values| values.get(edge))
                        .copied()
                        .flatten()
                        .unwrap_or(genomic.weight())
                })
                .collect()
        })
        .collect()
}
