use crate::config::SimulationConfig;
use crate::creature::mutation::mutate_genome;
use crate::creature::state::CreatureState;
use crate::kernel::types::Position;
use rand::Rng;

/// Build offspring creature state from a parent at a resolved spawn position.
///
/// Copies parent genome, phenotype, and memory. Applies mutation to the
/// offspring's genome according to `config.runtime.mutation`.
pub fn create_offspring(
    parent: &CreatureState,
    spawn_position: Position,
    initial_energy: u32,
    config: &SimulationConfig,
    rng: &mut impl Rng,
) -> CreatureState {
    let mut offspring_genome = parent.genome.clone();
    mutate_genome(&mut offspring_genome, &config.runtime.mutation, rng);

    let mut child = CreatureState::new(
        spawn_position,
        initial_energy,
        parent.generation + 1,
        [parent.phenotype_r, parent.phenotype_g, parent.phenotype_b],
        offspring_genome,
    );
    child.memory = parent.memory;
    child
}
