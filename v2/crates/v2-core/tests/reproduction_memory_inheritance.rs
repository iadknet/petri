use v2_core::evolution::{MutationConfig, reproduce_asexual};
use v2_core::mesh::{
    BackendDef, CreatureGenome, GraphBackendDef, GraphOperator, NodeGenome, NodeType,
};

fn parent_genome() -> CreatureGenome {
    CreatureGenome {
        entry_node_id: 7,
        nodes: vec![NodeGenome {
            node_id: 7,
            node_type: NodeType::Graph,
            backend_def: BackendDef::Graph(GraphBackendDef {
                operator: GraphOperator::Passthrough,
                inputs: Vec::new(),
                coefficients: Vec::new(),
                bias: 0.0,
                state_slot_count: 0,
            }),
            output_definitions: Vec::new(),
            local_state_init: vec![1, 2, 3],
        }],
        evolution_params: None,
    }
}

#[test]
fn offspring_memory_is_byte_for_byte_parent_copy() {
    let config = MutationConfig::default();
    let parent = parent_genome();
    let mut parent_memory = [0_u8; 1024];
    for (index, slot) in parent_memory.iter_mut().enumerate() {
        *slot = u8::try_from(index % 251).expect("index modulo is always in-range");
    }

    let offspring =
        reproduce_asexual(&parent, &parent_memory, &config, 42).expect("offspring should exist");

    assert_eq!(offspring.memory_bytes, parent_memory);
    assert!(offspring.genome.validate().is_ok());
}

#[test]
fn offspring_memory_snapshot_is_taken_at_commit_time() {
    let config = MutationConfig::default();
    let parent = parent_genome();
    let mut parent_memory = [0_u8; 1024];
    parent_memory[5] = 17;

    let offspring =
        reproduce_asexual(&parent, &parent_memory, &config, 7).expect("offspring should exist");

    parent_memory[5] = 222;
    assert_eq!(parent_memory[5], 222);
    assert_eq!(offspring.memory_bytes[5], 17);
}
