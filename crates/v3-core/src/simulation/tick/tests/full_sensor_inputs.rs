//! `assemble_full_sensor_inputs` (T11.F26 recorded contexts) assembles typed
//! local food and extended perception for every creature, whatever its
//! genome reads, exactly as production does for a genome that reads them.

use super::super::{assemble_full_sensor_inputs, assemble_sensor_inputs};
use super::support::{small_config, vm_raw_program_genome};
use crate::config::OrdinaryFoodTypeId;
use crate::contracts::{InputReference, WorldInputKey};
use crate::creature::genome::CreatureGenome;
use crate::sensors::perception::genome_uses_extended_perception;
use crate::sensors::typed_food::genome_uses_typed_local_food;
use crate::simulation::{seed_simulation, Simulation};

fn world_with(genome: &CreatureGenome) -> Simulation {
    let mut sim = seed_simulation(small_config(), 5);
    for creature in sim.creatures.values_mut() {
        creature.genome = genome.clone();
    }
    sim
}

#[test]
fn full_assembly_matches_production_for_a_genome_reading_every_input() {
    // The donor reads neither typed local food nor extended perception.
    let donor = vm_raw_program_genome(Vec::new());
    assert!(!genome_uses_extended_perception(&donor));
    assert!(!genome_uses_typed_local_food(&donor));
    let mut reader = vm_raw_program_genome(Vec::new());
    reader.nodes[0].input_refs = vec![
        InputReference::World(WorldInputKey::AreaBarrierSummary),
        InputReference::World(WorldInputKey::NeighborFoodRing {
            type_idx: OrdinaryFoodTypeId::new(1),
        }),
    ];
    assert!(genome_uses_extended_perception(&reader));
    assert!(genome_uses_typed_local_food(&reader));

    let donors = world_with(&donor);
    let mut ids: Vec<_> = donors.creatures.keys().collect();
    ids.sort();
    let full = assemble_full_sensor_inputs(&donors, &ids);
    assert_eq!(full, assemble_sensor_inputs(&world_with(&reader), &ids));
    assert_ne!(full, assemble_sensor_inputs(&donors, &ids));
}
