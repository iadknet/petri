//! The newborn's birth seeds from its draft's payload (T25.F01, T25.F06).
//!
//! The creature constructors take the [`BirthSeeds`] out of the genome they
//! receive and build the learned state from them
//! (`runtime::plasticity::LearnedState`), so reproduction writes no runtime
//! field and a built creature's genome carries no seeds.

use crate::creature::genome::cgp::BirthSeeds;
use crate::creature::genome::{BackendDef, CreatureGenome};

/// Take every graph node's seeds out of `genome`, indexed by mesh node.
/// A genome with no seeds at all (seeding, the lab, instruments, observation
/// clones, a parent with nothing initialized) gives the empty vector.
pub(crate) fn take_birth_seeds(genome: &mut CreatureGenome) -> Vec<Option<BirthSeeds>> {
    let seeded = genome.nodes.iter().any(
        |node| matches!(&node.backend_def, BackendDef::Graph(def) if def.birth_seeds().is_some()),
    );
    if !seeded {
        return Vec::new();
    }
    genome
        .nodes
        .iter_mut()
        .map(|node| match &mut node.backend_def {
            BackendDef::Graph(def) => def.take_birth_seeds(),
            BackendDef::Vm(_) => None,
        })
        .collect()
}
