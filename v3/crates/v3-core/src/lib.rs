pub mod config;
pub mod contracts;
pub mod creature;
pub mod kernel;
pub mod runtime;
pub mod seed;
pub mod sensors;
pub mod tick;

use creature::state::CreatureState;
use kernel::types::CreatureId;
use kernel::world_state::WorldState;
use slotmap::SlotMap;

/// Coordination shell for all mutable simulation state.
/// No business logic — modules own behavior, this owns the data bundle.
///
/// Fields are `pub` for borrow splitting: the tick orchestrator needs
/// simultaneous `&mut creatures` and `&mut world`. Rust allows this via
/// direct field access but not through `&mut self` accessor methods.
pub struct SimulationState {
    pub creatures: SlotMap<CreatureId, CreatureState>,
    pub world: WorldState,
    pub tick_number: u64,
}

impl SimulationState {
    pub fn new(world: WorldState) -> Self {
        Self {
            creatures: SlotMap::with_key(),
            world,
            tick_number: 0,
        }
    }

    /// Insert a creature and register it in the spatial index.
    pub fn spawn_creature(&mut self, creature: CreatureState) -> CreatureId {
        let pos = creature.position;
        let id = self.creatures.insert(creature);
        self.world.place_creature(pos, id);
        id
    }

    pub fn tick(
        &mut self,
        config: &config::SimulationConfig,
        rng: &mut impl rand::Rng,
    ) -> tick::orchestrator::TickStats {
        tick::orchestrator::tick(self, config, rng)
    }
}
