use crate::config::SimulationConfig;
use crate::contracts::{CreatureId, NodeId, Position};
use crate::creature::founder::v3alpha1_founder_genome;
use crate::creature::genome::{
    BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
};
use crate::creature::identity::CreatureIdentityState;
use crate::creature::state::CreatureState;
use crate::kernel::WorldState;
use crate::simulation::Simulation;
use rand::SeedableRng;
use slotmap::SlotMap;

pub(super) fn small_config() -> SimulationConfig {
    let mut cfg = SimulationConfig::default();
    cfg.world.width = 20;
    cfg.world.height = 20;
    cfg.population.initial_creatures = 5;
    cfg
}

pub(super) fn vm_program_genome(program: Vec<VmInstruction>) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program,
            }),
            targets: vec![],
        }],
    }
}

pub(super) fn make_sim_two_creatures(
    energy_a: f32,
    energy_b: f32,
) -> (Simulation, CreatureId, CreatureId) {
    let mut cfg = small_config();
    cfg.world.food.growth_rate = 0.0;
    cfg.world.food.initial_coverage = 0.0;

    let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
    world.apply_food_config(cfg.world.food.clone());
    let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();

    let a_id = creatures.insert_with_key(|id| {
        CreatureState::new(
            id,
            v3alpha1_founder_genome(),
            Position::new(5, 5),
            energy_a,
            0,
            [0, 0, 92, 92, 138, 138],
            0,
            [true; 6],
            CreatureIdentityState::default(),
            [0.0; 16],
        )
    });
    world.place_creature(Position::new(5, 5), a_id);

    let b_id = creatures.insert_with_key(|id| {
        CreatureState::new(
            id,
            v3alpha1_founder_genome(),
            Position::new(5, 4),
            energy_b,
            0,
            [0, 0, 92, 92, 138, 138],
            0,
            [true; 6],
            CreatureIdentityState::default(),
            [0.0; 16],
        )
    });
    world.place_creature(Position::new(5, 4), b_id);

    let sim = Simulation {
        world,
        creatures,
        action_logs: slotmap::SecondaryMap::new(),
        tick: 0,
        config: cfg,
        stats: crate::simulation::stats::SimStats::default(),
        rng: rand::rngs::SmallRng::seed_from_u64(42),
    };
    (sim, a_id, b_id)
}

pub(super) fn make_sim_with_one_creature(energy: f32) -> (Simulation, CreatureId) {
    let mut cfg = small_config();
    cfg.world.food.growth_rate = 0.0;
    cfg.world.food.initial_coverage = 0.0;

    let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
    world.apply_food_config(cfg.world.food.clone());
    let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
    let pos = Position::new(5, 5);
    let id = creatures.insert_with_key(|id| {
        CreatureState::new(
            id,
            v3alpha1_founder_genome(),
            pos,
            energy,
            0,
            [0, 0, 92, 92, 138, 138],
            0,
            [true; 6],
            CreatureIdentityState::default(),
            [0.0; 16],
        )
    });
    world.place_creature(pos, id);

    let sim = Simulation {
        world,
        creatures,
        action_logs: slotmap::SecondaryMap::new(),
        tick: 0,
        config: cfg,
        stats: crate::simulation::stats::SimStats::default(),
        rng: rand::rngs::SmallRng::seed_from_u64(42),
    };
    (sim, id)
}

pub(super) fn make_sim_with_custom_genome(
    energy: f32,
    genome: CreatureGenome,
) -> (Simulation, CreatureId) {
    let mut cfg = small_config();
    cfg.world.food.growth_rate = 0.0;
    cfg.world.food.initial_coverage = 0.0;

    let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
    world.apply_food_config(cfg.world.food.clone());
    let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
    let pos = Position::new(5, 5);
    let id = creatures.insert_with_key(|id| {
        CreatureState::new(
            id,
            genome,
            pos,
            energy,
            0,
            [0, 0, 92, 92, 138, 138],
            0,
            [true; 6],
            CreatureIdentityState::default(),
            [0.0; 16],
        )
    });
    world.place_creature(pos, id);

    let sim = Simulation {
        world,
        creatures,
        action_logs: slotmap::SecondaryMap::new(),
        tick: 0,
        config: cfg,
        stats: crate::simulation::stats::SimStats::default(),
        rng: rand::rngs::SmallRng::seed_from_u64(42),
    };
    (sim, id)
}
