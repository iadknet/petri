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

#[test]
fn neutral_recruitment_triggers_native_declaration_driven_sensor_assembly() {
    use crate::config::NeutralInputRecruitment as Arm;
    use crate::contracts::NodeId;
    use crate::creature::genome::cgp::CgpGraphBackendDef;
    use crate::creature::genome::vote::VoteSink;
    use crate::creature::genome::{BackendDef, NodeGenome};
    use crate::mutation::graph::recruitment::{recruit_source, SourceFamily};
    let parent = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Graph(CgpGraphBackendDef::new_with_fixed_outputs()),
            targets: vec![],
        }],
    };
    for (key, extended, typed) in [
        (
            WorldInputKey::area_food_summary(OrdinaryFoodTypeId::new(0)),
            true,
            false,
        ),
        (
            WorldInputKey::food_here(OrdinaryFoodTypeId::new(1)),
            false,
            true,
        ),
        (
            WorldInputKey::neighbor_food_ring(OrdinaryFoodTypeId::new(0)),
            false,
            true,
        ),
    ] {
        let mut child = parent.clone();
        recruit_source(
            &mut child.nodes[0],
            &SourceFamily::Input(InputReference::World(key)),
            Arm::WholeFamily,
            0,
            VoteSink::Eat,
        )
        .unwrap();
        assert_eq!(genome_uses_extended_perception(&child), extended);
        assert_eq!(genome_uses_typed_local_food(&child), typed);
        let donors = world_with(&parent);
        let readers = world_with(&child);
        let mut ids: Vec<_> = donors.creatures.keys().collect();
        ids.sort();
        let before = assemble_sensor_inputs(&donors, &ids);
        let after = assemble_sensor_inputs(&readers, &ids);
        let full = assemble_full_sensor_inputs(&readers, &ids);
        assert_eq!(after.len(), ids.len());
        if extended {
            assert!(after
                .iter()
                .zip(&full)
                .all(|(a, b)| a.1.perception == b.1.perception));
            assert!(after
                .iter()
                .zip(&before)
                .any(|(a, b)| a.1.perception != b.1.perception));
        }
        if typed {
            assert!(after
                .iter()
                .zip(&full)
                .all(|(a, b)| a.1.typed_local_food == b.1.typed_local_food));
            assert!(after
                .iter()
                .zip(&before)
                .any(|(a, b)| a.1.typed_local_food != b.1.typed_local_food));
        }
        println!("RECRUIT_ASSEMBLY {key:?}: creatures={} extended_assemblies={} typed_local_assemblies={}", ids.len(), usize::from(extended)*ids.len(), usize::from(typed)*ids.len());
    }
}
