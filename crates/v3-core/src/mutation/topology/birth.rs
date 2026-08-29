use crate::config::MutationConfig;
use crate::contracts::{NodeId, RouteTarget};
use crate::creature::genome::cgp::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource, OutputSinkKind,
};
use crate::creature::genome::{BackendDef, NodeGenome, VmBackendDef, VmInstruction};
use crate::mutation::sampling::{
    random_input_reference_for_food_types, sample_sub_idx_for_input_ref,
};
use rand::Rng;

/// Current minimal VM backend used by newborn topology nodes.
pub(super) fn minimal_vm_backend() -> BackendDef {
    BackendDef::Vm(VmBackendDef {
        register_count: 1,
        constants: vec![],
        program: vec![VmInstruction::Halt],
    })
}

/// Current blank graph backend used when topology swaps VM->Graph.
pub(super) fn blank_graph_backend(config: &MutationConfig) -> BackendDef {
    BackendDef::Graph(CgpGraphBackendDef::new_with_fixed_outputs(config))
}

fn curated_birth_compute_kind(rng: &mut impl Rng) -> ComputeNodeKind {
    match rng.gen_range(0..6) {
        0 => ComputeNodeKind::Sigmoid,
        1 => ComputeNodeKind::Tanh,
        2 => ComputeNodeKind::Relu,
        3 => ComputeNodeKind::Clamp01,
        4 => ComputeNodeKind::Abs,
        _ => ComputeNodeKind::Threshold(rng.gen_range(-1.0f32..=1.0)),
    }
}

fn try_random_custom_output_sink_idx(
    graph: &CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Option<usize> {
    let custom_sink_indices: Vec<usize> = graph
        .output_sinks
        .iter()
        .enumerate()
        .filter(|(_, sink)| matches!(sink.kind, OutputSinkKind::CustomOutput(_)))
        .map(|(idx, _)| idx)
        .collect();
    if custom_sink_indices.is_empty() {
        return None;
    }
    Some(custom_sink_indices[rng.gen_range(0..custom_sink_indices.len())])
}

fn wire_initialized_newborn_graph(
    graph: &mut CgpGraphBackendDef,
    sub_idx: u16,
    graph_compute_gate_chance: f64,
    rng: &mut impl Rng,
) -> bool {
    let Some(sink_idx) = try_random_custom_output_sink_idx(graph, rng) else {
        return false;
    };

    let input_source = GraphSource::InputLeaf {
        ref_idx: 0,
        sub_idx,
    };
    let wire_via_compute = rng.gen_bool(graph_compute_gate_chance);
    let sink_edge = if wire_via_compute {
        let compute_idx = graph.compute_nodes.len() as u16;
        graph.compute_nodes.push(ComputeNode {
            kind: curated_birth_compute_kind(rng),
            inputs: vec![GraphEdge {
                source: input_source,
                weight: 1.0,
            }],
            plasticity: None,
        });
        GraphEdge {
            source: GraphSource::ComputeNode(compute_idx),
            weight: 1.0,
        }
    } else {
        GraphEdge {
            source: input_source,
            weight: 1.0,
        }
    };
    graph.output_sinks[sink_idx].inputs.push(sink_edge);
    true
}

fn initialized_graph_backend_with_single_input(
    config: &MutationConfig,
    food_type_count: usize,
    rng: &mut impl Rng,
) -> (Vec<crate::contracts::InputReference>, BackendDef) {
    let input_ref = random_input_reference_for_food_types(rng, food_type_count);
    let sub_idx = sample_sub_idx_for_input_ref(&input_ref, config, rng);
    let mut graph = CgpGraphBackendDef::new_with_fixed_outputs(config);
    let wired = wire_initialized_newborn_graph(
        &mut graph,
        sub_idx,
        config.topology_new_node_birth.graph_compute_gate_chance as f64,
        rng,
    );
    if wired {
        (vec![input_ref], BackendDef::Graph(graph))
    } else {
        (Vec::new(), BackendDef::Graph(graph))
    }
}

fn new_graph_or_vm_birth(
    config: &MutationConfig,
    food_type_count: usize,
    rng: &mut impl Rng,
) -> (Vec<crate::contracts::InputReference>, BackendDef) {
    if !rng.gen_bool(config.topology_new_node_birth.graph_backend_chance as f64) {
        return (Vec::new(), minimal_vm_backend());
    }

    if !rng.gen_bool(config.topology_new_node_birth.graph_initialized_chance as f64) {
        return (Vec::new(), blank_graph_backend(config));
    }

    initialized_graph_backend_with_single_input(config, food_type_count, rng)
}

/// Construct a newborn topology node from configured birth policy.
pub(super) fn new_topology_birth_node(
    node_id: NodeId,
    targets: Vec<RouteTarget>,
    config: &MutationConfig,
    food_type_count: usize,
    rng: &mut impl Rng,
) -> NodeGenome {
    let (input_refs, backend_def) = new_graph_or_vm_birth(config, food_type_count, rng);
    NodeGenome {
        node_id,
        input_refs,
        backend_def,
        targets,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    #[test]
    fn initialized_wiring_returns_false_when_graph_has_no_custom_output_sinks() {
        let config = MutationConfig::default();
        let mut graph = CgpGraphBackendDef::new_with_fixed_outputs(&config);
        graph
            .output_sinks
            .retain(|sink| !matches!(sink.kind, OutputSinkKind::CustomOutput(_)));

        let mut rng = SmallRng::seed_from_u64(1);
        let wired = wire_initialized_newborn_graph(&mut graph, 0, 1.0, &mut rng);

        assert!(!wired);
        assert!(graph.compute_nodes.is_empty());
        assert!(graph.output_sinks.iter().all(|sink| sink.inputs.is_empty()));
    }
}
