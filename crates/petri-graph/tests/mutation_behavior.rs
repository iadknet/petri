use petri_graph::{
    ComputationGraph, ControllerPalette, Edge, MutationConfig, NodeKind, SensorInputs,
};
use rand::rngs::SmallRng;
use rand::SeedableRng;

fn hidden_node_count(graph: &ComputationGraph) -> usize {
    graph
        .nodes
        .iter()
        .filter(|node| {
            !matches!(
                node,
                NodeKind::InputFoodHere
                    | NodeKind::InputEnergy
                    | NodeKind::InputRandom
                    | NodeKind::InputFoodDirection
                    | NodeKind::InputFoodDistance
                    | NodeKind::InputCreatureDirection
                    | NodeKind::InputCreatureDistance
                    | NodeKind::InputLocalDensity
                    | NodeKind::InputBarrierDirection
                    | NodeKind::InputBarrierDistance
                    | NodeKind::InputMoveBlockedLastTick
                    | NodeKind::InputMemoryRead
                    | NodeKind::InputTouchExists(_)
                    | NodeKind::InputTouchFoodValue(_)
                    | NodeKind::InputTouchHasBarrier(_)
                    | NodeKind::InputTouchOccupied(_)
                    | NodeKind::InputSlotExists(_)
                    | NodeKind::InputSlotIsEmpty(_)
                    | NodeKind::InputSlotIsBarrier(_)
                    | NodeKind::InputSlotFoodValue(_)
                    | NodeKind::OutputMoveX
                    | NodeKind::OutputMoveY
                    | NodeKind::OutputEat
                    | NodeKind::OutputReproduce
                    | NodeKind::OutputMemoryWrite
                    | NodeKind::OutputInventoryPickup
                    | NodeKind::OutputInventoryPut
                    | NodeKind::OutputInventorySlotSelect
                    | NodeKind::OutputInventoryDirectionSelect
            )
        })
        .count()
}

fn parameter_checksum(graph: &ComputationGraph) -> f32 {
    let node_sum = graph
        .nodes
        .iter()
        .map(|node| match node {
            NodeKind::Constant(v) => *v,
            NodeKind::Threshold(t) => *t,
            _ => 0.0,
        })
        .sum::<f32>();
    let edge_sum = graph.edges.iter().map(|e| e.weight).sum::<f32>();
    node_sum + edge_sum
}

#[test]
fn mutation_changes_parameters_without_changing_topology() {
    let mut graph = ComputationGraph::founder(ControllerPalette::Hybrid);
    let before_nodes = graph.nodes.len();
    let before_edges = graph.edges.len();
    let before_checksum = parameter_checksum(&graph);

    let mut rng = SmallRng::seed_from_u64(7);
    graph.mutate_weights(&mut rng, 1.0, 0.2);

    let after_checksum = parameter_checksum(&graph);
    assert_eq!(graph.nodes.len(), before_nodes);
    assert_eq!(graph.edges.len(), before_edges);
    assert_ne!(before_checksum, after_checksum);

    let outputs = graph.evaluate(SensorInputs {
        food_here: 1.0,
        energy: 1.0,
        random: 0.25,
        food_direction: 0.0,
        food_distance: 1.0,
        creature_direction: 0.0,
        creature_distance: 1.0,
        local_density: 0.0,
        barrier_direction: 0.0,
        barrier_distance: 1.0,
        move_blocked_last_tick: 0.0,
        memory_read: 0.0,
        ..SensorInputs::default()
    });
    assert!((-1.0..=1.0).contains(&outputs.move_x));
    assert!((-1.0..=1.0).contains(&outputs.move_y));
    assert!((0.0..=1.0).contains(&outputs.eat));
    assert!((0.0..=1.0).contains(&outputs.reproduce));
    assert!((0.0..=1.0).contains(&outputs.memory_write));
}

#[test]
fn add_hidden_node_mutation_splices_existing_edge() {
    let mut graph = ComputationGraph::founder(ControllerPalette::Hybrid);
    let before_nodes = graph.nodes.len();
    let before_edges = graph.edges.len();
    let mut rng = SmallRng::seed_from_u64(13);

    let changed = graph.add_hidden_node_by_splicing_edge(&mut rng);
    assert!(changed);
    assert_eq!(graph.nodes.len(), before_nodes + 1);
    assert_eq!(graph.edges.len(), before_edges + 1);
    assert!(hidden_node_count(&graph) >= 1);
}

#[test]
fn add_edge_mutation_inserts_new_connection_when_possible() {
    let mut graph = ComputationGraph::founder(ControllerPalette::Hybrid);
    let before_edges = graph.edges.len();
    let mut rng = SmallRng::seed_from_u64(17);

    let mut changed = false;
    for _ in 0..32 {
        changed = graph.add_edge_mutation(&mut rng);
        if changed {
            break;
        }
    }

    assert!(changed);
    assert_eq!(graph.edges.len(), before_edges + 1);
}

#[test]
fn remove_edge_mutation_removes_one_edge() {
    let mut graph = ComputationGraph::founder(ControllerPalette::Hybrid);
    let before_edges = graph.edges.len();
    let mut rng = SmallRng::seed_from_u64(19);

    let changed = graph.remove_edge_mutation(&mut rng);
    assert!(changed);
    assert_eq!(graph.edges.len(), before_edges - 1);
}

#[test]
fn remove_disconnected_hidden_nodes_prunes_orphan_nodes() {
    let mut graph = ComputationGraph {
        palette: ControllerPalette::Hybrid,
        nodes: vec![NodeKind::InputFoodHere, NodeKind::OutputEat, NodeKind::Tanh],
        edges: vec![Edge {
            from: 0,
            to: 1,
            weight: 1.0,
        }],
    };

    let removed = graph.remove_disconnected_hidden_nodes();
    assert_eq!(removed, 1);
    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);
}

#[test]
fn change_hidden_node_type_mutation_retypes_hidden_node() {
    let mut graph = ComputationGraph {
        palette: ControllerPalette::Hybrid,
        nodes: vec![NodeKind::InputFoodHere, NodeKind::Add, NodeKind::OutputEat],
        edges: vec![
            Edge {
                from: 0,
                to: 1,
                weight: 1.0,
            },
            Edge {
                from: 1,
                to: 2,
                weight: 1.0,
            },
        ],
    };
    let mut rng = SmallRng::seed_from_u64(23);

    let changed = graph.change_hidden_node_type(&mut rng);
    assert!(changed);
    assert!(!matches!(graph.nodes[1], NodeKind::Add));
}

#[test]
fn mutate_with_config_runs_structural_and_logic_operators() {
    let mut graph = ComputationGraph::founder(ControllerPalette::Hybrid);
    let before_nodes = graph.nodes.len();
    let before_edges = graph.edges.len();
    let mut rng = SmallRng::seed_from_u64(29);

    let changed = graph.mutate_with_config(
        &mut rng,
        MutationConfig {
            weight_mutation_rate: 0.0,
            weight_mutation_magnitude: 0.0,
            structural_mutation_rate: 1.0,
            logic_node_mutation_rate: 1.0,
        },
    );

    assert!(changed);
    assert!(graph.nodes.len() != before_nodes || graph.edges.len() != before_edges);
}
