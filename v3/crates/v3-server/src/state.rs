use rand::rngs::SmallRng;
use rand::SeedableRng;
use v3_core::kernel::world_state::WorldState;
use v3_core::seed::seed_creatures;
use v3_core::SimulationState;

pub struct ServerState {
    pub sim: SimulationState,
    pub running: bool,
    pub rng: SmallRng,
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerState {
    pub fn new() -> Self {
        let world = WorldState::new(400, 400, true);
        let mut sim = SimulationState::new(world);
        let mut rng = SmallRng::seed_from_u64(42);

        seed_creatures(&mut sim, 50, 20, &mut rng);

        Self {
            sim,
            running: false,
            rng,
        }
    }

    pub fn tick(&mut self) {
        if !self.running {
            return;
        }
        let _stats = self.sim.tick(&mut self.rng);
    }
}
