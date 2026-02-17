use crate::config::SimulationConfig;
use crate::creature::state::CreatureState;
use crate::kernel::types::Position;
use rand::Rng;

/// Build offspring creature state from a parent at a resolved spawn position.
///
/// Stage 3A semantics: deterministic clone for phenotype + memory.
/// Mutation is deferred to Stage 3B.
pub fn create_offspring(
    parent: &CreatureState,
    spawn_position: Position,
    initial_energy: u32,
    _config: &SimulationConfig,
    _rng: &mut impl Rng,
) -> CreatureState {
    let mut child = CreatureState::new(
        spawn_position,
        initial_energy,
        parent.generation + 1,
        [parent.phenotype_r, parent.phenotype_g, parent.phenotype_b],
    );
    child.memory = parent.memory;
    child
}
