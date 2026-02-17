use rand::rngs::SmallRng;
use rand::SeedableRng;
use v3_core::config::SimulationConfig;
use v3_core::kernel::world_state::WorldState;
use v3_core::seed::seed_creatures;
use v3_core::SimulationState;

pub struct ServerState {
    pub sim: SimulationState,
    pub running: bool,
    pub rng: SmallRng,
    pub config: SimulationConfig,
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerState {
    pub fn new() -> Self {
        let config = SimulationConfig::default();
        let world = WorldState::new(400, 400, true);
        let mut sim = SimulationState::new(world);
        let mut rng = SmallRng::seed_from_u64(42);

        seed_creatures(&mut sim, 50, &config, &mut rng);

        Self {
            sim,
            running: false,
            rng,
            config,
        }
    }

    pub fn tick(&mut self) {
        if !self.running {
            return;
        }
        let _stats = self.sim.tick(&self.config, &mut self.rng);
    }
}
