//! A plain vote-edge graph survives native offspring construction: the
//! child at rate 0 carries the parent's edges, deleted positions included,
//! and resolves the same inputs into votes.
use super::*;
use crate::mutation::graph::operators::remove_edge;
use rand::{rngs::SmallRng, SeedableRng};

/// The four ranked `present` slots of the nearby-creature core into `Eat`.
fn fixture() -> CreatureGenome {
    let key = WorldInputKey::NearbyCreature;
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    for i in 0..4u16 {
        wire(
            &mut def,
            OutputSinkKind::ActionVote(VoteSink::Eat),
            edge(leaf(0, i * 9), if i == 0 { 0.025 } else { 0.0 }),
        );
    }
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![graph_node(def, vec![InputReference::World(key)])],
    }
}

fn native_run(genome: &CreatureGenome) {
    let node = &genome.nodes[0];
    let mut energy = 10.0;
    let mut state = GraphRuntimeState::for_genome(genome, Vec::new());
    let mut side = MeshSideOutputs::new(8);
    let mut memory = [0.0; 16];
    let snapshot = sensors();
    let result = execute_graph_node(
        graph(node),
        &node.input_refs,
        &[0.0; OUTPUT_SLOT_COUNT],
        &mut energy,
        0.0,
        0,
        &mut state,
        &snapshot,
        &RuntimeConfig::default(),
        &mut side,
        &mut memory,
        &[0.0; 16],
    );
    assert!(!result.energy_exhausted);
    let staged = side.dispatch_effects().0;
    side.commit_vote_contribution(0);
    assert_eq!(side.votes, staged);
    let mut expected = [0.0; VOTE_SINK_COUNT];
    for sink in graph(node).output_sinks() {
        let OutputSinkKind::ActionVote(vote) = sink.kind else {
            continue;
        };
        for edge in &sink.inputs {
            let GraphSource::InputLeaf { sub_idx, .. } = edge.source() else {
                panic!("fixture leaf")
            };
            let input = snapshot.perception.nearby[usize::from(sub_idx)];
            assert!(input > 0.0);
            expected[vote.index()] += edge.own_weight() * input;
        }
    }
    assert_eq!(
        side.votes, expected,
        "native input resolution and applied vote contribution"
    );
    assert_eq!(side.work_counters.plasticity_updates, 0);
}

#[test]
fn deleted_positions_survive_native_offspring_construction() {
    use crate::simulation::actions::{apply_reproduce, ReproductionActionResult};
    use crate::simulation::seeding::seed_simulation_with_founder_genomes;
    let mut genome = fixture();
    let mut rng = SmallRng::seed_from_u64(29_005_002);
    remove_edge(graph_mut(&mut genome.nodes[0]), &mut rng).unwrap();
    let mut config = crate::config::SimulationConfig::default();
    config.population.initial_creatures = 1;
    config.mutation.per_unit_rate = 0.0;
    // Rate 0.0 delivers at conception (T03.F12): the child exists at return.
    config.energy.lifecycle.gestation_ticks_per_founder_genome = 0.0;
    let mut sim = seed_simulation_with_founder_genomes(config, 83, |_, founder| {
        *founder = genome.clone();
    });
    let parent_id = sim.creatures.keys().next().unwrap();
    let parent = &mut sim.creatures[parent_id];
    parent.age = sim.config.energy.lifecycle.min_reproduce_age;
    parent.energy = 10000.0;
    let direction = Direction::ALL
        .into_iter()
        .find(|&d| {
            sim.world
                .resolve_neighbor(sim.creatures[parent_id].position, d)
                .is_some_and(|p| sim.world.is_valid_target_cell(p))
        })
        .unwrap();
    assert_eq!(
        apply_reproduce(parent_id, &mut sim, direction, 1.0, &mut rng),
        ReproductionActionResult::Conceived
    );
    let child = sim
        .creatures
        .iter()
        .find(|(id, _)| *id != parent_id)
        .unwrap()
        .1;
    assert_eq!(*child.genome(), genome);
    assert_eq!(child.genome_size(), genome.genome_size());
    assert_eq!(*sim.creatures[parent_id].genome(), genome);
    native_run(child.genome());
}
