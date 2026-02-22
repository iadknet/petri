use rand::rngs::SmallRng;
use slotmap::SlotMap;

use crate::config::SimulationConfig;
use crate::contracts::CreatureId;
use crate::creature::state::CreatureState;
use crate::kernel::WorldState;

/// Central simulation container.
///
/// Holds world state, creature slotmap, tick counter, config, and the simulation RNG.
/// The RNG is private to maintain determinism — only `seeding` and `tick` modules
/// drive it directly.
pub struct Simulation {
    pub world: WorldState,
    pub creatures: SlotMap<CreatureId, CreatureState>,
    pub tick: u64,
    pub config: SimulationConfig,
    /// Private RNG seeded at startup; used for food growth, turn-queue shuffling, etc.
    pub(crate) rng: SmallRng,
}

impl Simulation {
    /// Create a `Simulation` from pre-built components, seeding the internal RNG from `seed`.
    ///
    /// Prefer [`crate::simulation::seed_simulation`] for normal simulation startup.
    /// This constructor exists for tests that need fine-grained control over world state.
    pub fn new(
        world: WorldState,
        creatures: SlotMap<CreatureId, CreatureState>,
        tick: u64,
        config: SimulationConfig,
        seed: u64,
    ) -> Self {
        use rand::SeedableRng;
        Self {
            world,
            creatures,
            tick,
            config,
            rng: SmallRng::seed_from_u64(seed),
        }
    }

    /// Number of living creatures in this simulation.
    pub fn creature_count(&self) -> usize {
        self.creatures.len()
    }

    /// Current tick number (incremented by `run_tick`).
    pub fn tick_number(&self) -> u64 {
        self.tick
    }
}
