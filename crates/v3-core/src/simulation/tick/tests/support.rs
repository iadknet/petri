use crate::config::SimulationConfig;
use crate::contracts::{CreatureId, NodeId, Position, WorldAction};
use crate::creature::body::HeritableBody;
use crate::creature::founder::v3alpha1_founder_genome;
use crate::creature::identity::CreatureIdentityState;
use crate::creature::phenotype::Phenotype;
use crate::creature::state::CreatureState;
use crate::genome::{BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction};
use crate::kernel::WorldState;
use crate::runtime::trace::domain::TerminationReason;
use crate::runtime::types::{ComputeCostReport, MeshOutput};
use crate::simulation::outcomes::OutcomeAccumulator;
use crate::simulation::Simulation;
use rand::SeedableRng;
use slotmap::SlotMap;

/// A founder-phenotype body with the default identity.
fn founder_body() -> HeritableBody {
    HeritableBody {
        phenotype: Phenotype::FOUNDER,
        identity: CreatureIdentityState::default(),
        capacity: 200.0,
        steal_share: 0.2,
        give_share: 0.2,
    }
}

/// A cognition result committing `actions` with no compute charge.
pub(super) fn decision(actions: Vec<WorldAction>) -> MeshOutput {
    MeshOutput {
        actions,
        cost_report: ComputeCostReport::default(),
        priority_bid: 0.0,
        work_counters: Default::default(),
        energy_observation: Default::default(),
        termination_reason: TerminationReason::NoDecision,
        heading: crate::contracts::Direction::N,
        commit_counts: Default::default(),
    }
}

/// Run Phase 2 on creature `id` committing `actions`, on a fixed RNG.
pub(super) fn run_actions(sim: &mut Simulation, id: CreatureId, actions: Vec<WorldAction>) {
    super::super::run_phase_2(
        sim,
        vec![(id, decision(actions))],
        &mut OutcomeAccumulator::default(),
        &mut rand::rngs::SmallRng::seed_from_u64(11),
    );
}

/// Run Phase 2 on creature `id` committing the single `action`.
pub(super) fn execute(sim: &mut Simulation, id: CreatureId, action: WorldAction) {
    run_actions(sim, id, vec![action]);
}

pub(super) fn small_config() -> SimulationConfig {
    let mut cfg = SimulationConfig::default();
    cfg.world.width = 20;
    cfg.world.height = 20;
    cfg.population.initial_creatures = 5;
    // Fixtures here read a birth on the tick of its `Reproduce`; rate 0.0
    // delivers at conception (T03.F12). Gestation tests set the rate.
    cfg.energy.lifecycle.gestation_ticks_per_founder_genome = 0.0;
    cfg
}

/// A one-node VM genome running `program` exactly as given, with no
/// constants and one register.
pub(super) fn vm_raw_program_genome(program: Vec<VmInstruction>) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome::new(
            NodeId::new(0),
            vec![],
            BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program,
            }),
            vec![],
        )],
    }
}

/// A one-node VM genome of exactly `units` genome units (`units >= 1`):
/// the node and `units - 1` `Halt` instructions.
pub(super) fn sized_genome(units: u32) -> CreatureGenome {
    let genome = vm_raw_program_genome(vec![VmInstruction::Halt; units as usize - 1]);
    assert_eq!(genome.genome_size(), units);
    genome
}

/// A one-node VM genome of functional complexity `complexity`
/// (`complexity >= 1`): the node and `complexity - 1` live payload writes.
pub(super) fn genome_of_complexity(complexity: u32) -> CreatureGenome {
    let write = VmInstruction::WriteInternalPayload {
        slot_idx: 0,
        src: 0,
    };
    let genome = vm_raw_program_genome(vec![write; complexity as usize - 1]);
    assert_eq!(genome.complexity(), complexity);
    genome
}

/// A one-node VM genome running `program` after `r0 = 1.0`, so an
/// `AddVote { src: 0, .. }` votes one unit and commits its action once.
pub(super) fn vm_program_genome(program: Vec<VmInstruction>) -> CreatureGenome {
    let mut with_unit = vec![VmInstruction::LoadConst {
        dst: 0,
        const_idx: 0,
    }];
    with_unit.extend(program);
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome::new(
            NodeId::new(0),
            vec![],
            BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![1.0],
                program: with_unit,
            }),
            vec![],
        )],
    }
}

pub(super) fn make_sim_two_creatures(
    energy_a: f32,
    energy_b: f32,
) -> (Simulation, CreatureId, CreatureId) {
    make_sim_two_with_genomes(
        energy_a,
        energy_b,
        v3alpha1_founder_genome(),
        v3alpha1_founder_genome(),
    )
}

/// [`make_sim_two_creatures`] with creature `a` built from `genome_a` and
/// `b` (one cell north of `a`) from `genome_b`.
pub(super) fn make_sim_two_with_genomes(
    energy_a: f32,
    energy_b: f32,
    genome_a: CreatureGenome,
    genome_b: CreatureGenome,
) -> (Simulation, CreatureId, CreatureId) {
    let mut cfg = small_config();
    cfg.world.food.growth_rate = 0.0;

    let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
    world.reconfigure_food(cfg.world.food.clone());
    let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();

    let a_id = creatures.insert_with_key(|id| {
        {
            let mut founder = crate::creature::birth::Zygote::founder(genome_a, founder_body())
                .finalize(
                    id,
                    crate::creature::birth::BirthPlace {
                        position: Position::new(5, 5),
                        heading: crate::contracts::Direction::N,
                        energy: energy_a,
                    },
                )
                .creature;
            // A fixture keeps its requested energy past the body cap.
            founder.energy = energy_a;
            founder
        }
    });
    world.place_creature(Position::new(5, 5), a_id);

    let b_id = creatures.insert_with_key(|id| {
        {
            let mut founder = crate::creature::birth::Zygote::founder(genome_b, founder_body())
                .finalize(
                    id,
                    crate::creature::birth::BirthPlace {
                        position: Position::new(5, 4),
                        heading: crate::contracts::Direction::N,
                        energy: energy_b,
                    },
                )
                .creature;
            // A fixture keeps its requested energy past the body cap.
            founder.energy = energy_b;
            founder
        }
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
        food_stream_key: crate::kernel::FoodStreamKey::from_run_seed(42),
    };
    (sim, a_id, b_id)
}

pub(super) fn make_sim_with_one_creature(energy: f32) -> (Simulation, CreatureId) {
    let mut cfg = small_config();
    cfg.world.food.growth_rate = 0.0;

    let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
    world.reconfigure_food(cfg.world.food.clone());
    let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
    let pos = Position::new(5, 5);
    let id = creatures.insert_with_key(|id| {
        {
            let mut founder =
                crate::creature::birth::Zygote::founder(v3alpha1_founder_genome(), founder_body())
                    .finalize(
                        id,
                        crate::creature::birth::BirthPlace {
                            position: pos,
                            heading: crate::contracts::Direction::N,
                            energy,
                        },
                    )
                    .creature;
            // A fixture keeps its requested energy past the body cap.
            founder.energy = energy;
            founder
        }
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
        food_stream_key: crate::kernel::FoodStreamKey::from_run_seed(42),
    };
    (sim, id)
}

pub(super) fn make_sim_with_custom_genome(
    energy: f32,
    genome: CreatureGenome,
) -> (Simulation, CreatureId) {
    let mut cfg = small_config();
    cfg.world.food.growth_rate = 0.0;

    let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
    world.reconfigure_food(cfg.world.food.clone());
    let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
    let pos = Position::new(5, 5);
    let id = creatures.insert_with_key(|id| {
        {
            let mut founder = crate::creature::birth::Zygote::founder(genome, founder_body())
                .finalize(
                    id,
                    crate::creature::birth::BirthPlace {
                        position: pos,
                        heading: crate::contracts::Direction::N,
                        energy,
                    },
                )
                .creature;
            // A fixture keeps its requested energy past the body cap.
            founder.energy = energy;
            founder
        }
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
        food_stream_key: crate::kernel::FoodStreamKey::from_run_seed(42),
    };
    (sim, id)
}
