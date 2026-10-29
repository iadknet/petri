//! World-input wire keys (T30.F16 invariant 3): with two food types present,
//! both health maps keyed by world input carry each food type's count under
//! its own `WorldInputKey::as_key`.

use super::census_pins::frame_of;
use super::*;
use v3_core::config::OrdinaryFoodTypeId;
use v3_core::contracts::{CreatureId, Direction, InputReference, NodeId, Position, WorldInputKey};
use v3_core::creature::birth::{BirthPlace, Zygote};
use v3_core::creature::state::CreatureState;
use v3_core::genome::{BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction};
use v3_core::mutation::topology::TopologyOperator;
use v3_core::mutation::MutationOperator;

const ADD_NODE: MutationOperator = MutationOperator::Topology(TopologyOperator::AddNode);

const FOOD_0: WorldInputKey = WorldInputKey::food_here(OrdinaryFoodTypeId::new(0));
const FOOD_1: WorldInputKey = WorldInputKey::food_here(OrdinaryFoodTypeId::new(1));

fn empty() -> Simulation {
    let mut config = SimulationConfig::default();
    config.world.width = 16;
    config.world.height = 16;
    config.population.initial_creatures = 0;
    seed_simulation(config, 29)
}

/// One entry node reading `FoodHere` of type 0 once and of type 1 twice.
fn two_food_reader(id: CreatureId, config: &SimulationConfig) -> CreatureState {
    let read = |dst, ref_idx| VmInstruction::ReadInput {
        dst,
        ref_idx,
        sub_idx: 0,
    };
    let genome = CreatureGenome::new(
        NodeId::new(0),
        vec![NodeGenome::new(
            NodeId::new(0),
            vec![InputReference::World(FOOD_0), InputReference::World(FOOD_1)],
            BackendDef::Vm(VmBackendDef {
                register_count: 4,
                constants: vec![],
                program: vec![
                    read(0, 0),
                    read(1, 1),
                    read(2, 1),
                    VmInstruction::WriteInternalPayload {
                        slot_idx: 0,
                        src: 0,
                    },
                    VmInstruction::WriteInternalPayload {
                        slot_idx: 1,
                        src: 1,
                    },
                    VmInstruction::WriteInternalPayload {
                        slot_idx: 2,
                        src: 2,
                    },
                ],
            }),
            vec![],
        )],
        v3_core::creature::founder::founder_body(0, 1, config),
    );
    let place = BirthPlace {
        position: Position::new(0, 0),
        heading: Direction::N,
        energy: 10.0,
    };
    Zygote::founder(genome, 0).finalize(id, place).creature
}

#[test]
fn the_mutation_world_input_map_keys_each_food_type_apart() {
    let mut sim = empty();
    sim.stats
        .mutation_added_node_world_inputs_total_by_operator
        .insert(ADD_NODE, [(FOOD_0, 3), (FOOD_1, 5)].into_iter().collect());
    let health = frame_of(sim).health;
    let inputs = &health.mutation_added_node_world_inputs_total_by_operator[ADD_NODE.as_key()];
    let expected: HashMap<String, u64> =
        HashMap::from([("FoodHere:0".to_owned(), 3), ("FoodHere:1".to_owned(), 5)]);
    assert_eq!(inputs, &expected);
}

#[test]
fn the_live_read_world_input_map_keys_each_food_type_apart() {
    let mut sim = empty();
    let config = sim.config.clone();
    sim.creatures
        .insert_with_key(|id| two_food_reader(id, &config));
    let reads = frame_of(sim).health.vm_live_read_world_inputs_current;
    let expected: HashMap<String, u64> =
        HashMap::from([("FoodHere:0".to_owned(), 1), ("FoodHere:1".to_owned(), 2)]);
    assert_eq!(reads, expected);
}
