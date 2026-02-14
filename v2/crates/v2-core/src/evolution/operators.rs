use std::collections::{HashMap, HashSet, VecDeque};

use super::config::{MutationConfig, MutationWeights};
use crate::mesh::{
    ActionMetadataField, BackendDef, CreatureGenome, GraphBackendDef, GraphOperator,
    InputReference, InternalTargetDef, NodeGenome, NodeId, NodeType, OutputDefinition,
    PayloadField, VmBackendDef, VmInstruction, WorldActionDef, WorldActionKind,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MutationOperatorKind {
    AddNode,
    RemoveNode,
    RetargetNode,
    AddOutput,
    RemoveOutput,
    RetargetOutput,
    GraphLocalMutation,
    VmInstructionMutation,
    NodeDuplication,
    SubgraphDuplication,
}

impl MutationOperatorKind {
    #[must_use]
    pub const fn all() -> [Self; 10] {
        [
            Self::AddNode,
            Self::RemoveNode,
            Self::RetargetNode,
            Self::AddOutput,
            Self::RemoveOutput,
            Self::RetargetOutput,
            Self::GraphLocalMutation,
            Self::VmInstructionMutation,
            Self::NodeDuplication,
            Self::SubgraphDuplication,
        ]
    }
}

#[must_use]
pub fn apply_birth_mutations(
    parent_genome: &CreatureGenome,
    config: &MutationConfig,
    seed: u64,
) -> CreatureGenome {
    if config.validate().is_err() {
        return parent_genome.clone();
    }

    let mut rng = Lcg64::new(seed);
    let mut genome = parent_genome.clone();

    let min = usize::from(config.per_birth_mutation_events_min);
    let max = usize::from(config.per_birth_mutation_events_max);
    let event_count = min + (usize::try_from(rng.next_u32()).unwrap_or(0) % (max - min + 1));

    for _ in 0..event_count {
        let operator = sample_operator(&config.weights, rng.next_f32());
        let _ = apply_operator(&mut genome, config, operator, rng.next_u64());
    }

    genome
}

#[must_use]
pub fn apply_operator(
    genome: &mut CreatureGenome,
    config: &MutationConfig,
    operator: MutationOperatorKind,
    seed: u64,
) -> bool {
    if config.validate().is_err() {
        return false;
    }

    let mut rng = Lcg64::new(seed);
    match operator {
        MutationOperatorKind::AddNode => add_node(genome, config, &mut rng),
        MutationOperatorKind::RemoveNode => remove_node(genome, config, &mut rng),
        MutationOperatorKind::RetargetNode => retarget_node(genome, &mut rng),
        MutationOperatorKind::AddOutput => add_output(genome, config, &mut rng),
        MutationOperatorKind::RemoveOutput => remove_output(genome, &mut rng),
        MutationOperatorKind::RetargetOutput => retarget_output(genome, &mut rng),
        MutationOperatorKind::GraphLocalMutation => graph_local_mutation(genome, &mut rng),
        MutationOperatorKind::VmInstructionMutation => {
            vm_instruction_mutation(genome, config, &mut rng)
        }
        MutationOperatorKind::NodeDuplication => node_duplication(genome, config, &mut rng),
        MutationOperatorKind::SubgraphDuplication => subgraph_duplication(genome, config, &mut rng),
    }
}

fn sample_operator(weights: &MutationWeights, roll: f32) -> MutationOperatorKind {
    let mut remaining = roll.clamp(0.0, 1.0);
    for (operator, weight) in weight_table(weights) {
        if remaining <= weight {
            return operator;
        }
        remaining -= weight;
    }

    MutationOperatorKind::SubgraphDuplication
}

fn weight_table(weights: &MutationWeights) -> [(MutationOperatorKind, f32); 10] {
    [
        (MutationOperatorKind::AddNode, weights.add_node),
        (MutationOperatorKind::RemoveNode, weights.remove_node),
        (MutationOperatorKind::RetargetNode, weights.retarget_node),
        (MutationOperatorKind::AddOutput, weights.add_output),
        (MutationOperatorKind::RemoveOutput, weights.remove_output),
        (
            MutationOperatorKind::RetargetOutput,
            weights.retarget_output,
        ),
        (
            MutationOperatorKind::GraphLocalMutation,
            weights.graph_local_mutation,
        ),
        (
            MutationOperatorKind::VmInstructionMutation,
            weights.vm_instruction_mutation,
        ),
        (
            MutationOperatorKind::NodeDuplication,
            weights.node_duplication,
        ),
        (
            MutationOperatorKind::SubgraphDuplication,
            weights.subgraph_duplication,
        ),
    ]
}

fn add_node(genome: &mut CreatureGenome, config: &MutationConfig, rng: &mut Lcg64) -> bool {
    if genome.nodes.len() >= config.max_nodes {
        return false;
    }

    let node_id = next_node_id(&genome.nodes);
    let node = if rng.chance(0.5) {
        default_graph_node(node_id)
    } else {
        default_vm_node(node_id)
    };

    genome.nodes.push(node);
    if let Some(source) = genome
        .nodes
        .iter_mut()
        .find(|candidate| candidate.output_definitions.len() < config.max_outputs_per_node)
    {
        source
            .output_definitions
            .push(OutputDefinition::InternalTarget(InternalTargetDef {
                target_node_id: node_id,
                input_refs: Vec::new(),
                payload_fields: Vec::new(),
            }));
    }

    true
}

fn remove_node(genome: &mut CreatureGenome, config: &MutationConfig, rng: &mut Lcg64) -> bool {
    if genome.nodes.len() <= config.min_nodes {
        return false;
    }

    let candidate_indices = genome
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            if node.node_id != genome.entry_node_id {
                Some(index)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let remove_index = if candidate_indices.is_empty() {
        rng.choose_index(genome.nodes.len())
    } else {
        candidate_indices[rng.choose_index(candidate_indices.len())]
    };

    let removed_id = genome.nodes.remove(remove_index).node_id;
    if !genome
        .nodes
        .iter()
        .any(|node| node.node_id == genome.entry_node_id)
    {
        genome.entry_node_id = genome
            .nodes
            .first()
            .map(|node| node.node_id)
            .unwrap_or(removed_id);
    }

    let fallback_target = genome
        .nodes
        .first()
        .map(|node| node.node_id)
        .unwrap_or(genome.entry_node_id);

    for node in &mut genome.nodes {
        for output in &mut node.output_definitions {
            if let OutputDefinition::InternalTarget(target) = output
                && target.target_node_id == removed_id
            {
                target.target_node_id = fallback_target;
            }
        }
    }

    true
}

fn retarget_node(genome: &mut CreatureGenome, rng: &mut Lcg64) -> bool {
    if genome.nodes.len() < 2 {
        return false;
    }

    let ids = genome
        .nodes
        .iter()
        .map(|node| node.node_id)
        .collect::<Vec<_>>();
    let entry_retarget = rng.chance(0.5);

    if entry_retarget
        && let Some(new_entry) = choose_different_id(&ids, genome.entry_node_id, rng)
    {
        genome.entry_node_id = new_entry;
        return true;
    }

    for node in &mut genome.nodes {
        for output in &mut node.output_definitions {
            if let OutputDefinition::InternalTarget(target) = output
                && let Some(new_target) = choose_different_id(&ids, target.target_node_id, rng)
            {
                target.target_node_id = new_target;
                return true;
            }
        }
    }

    if let Some(new_entry) = choose_different_id(&ids, genome.entry_node_id, rng) {
        genome.entry_node_id = new_entry;
        return true;
    }

    false
}

fn add_output(genome: &mut CreatureGenome, config: &MutationConfig, rng: &mut Lcg64) -> bool {
    let node_ids = genome
        .nodes
        .iter()
        .map(|node| node.node_id)
        .collect::<Vec<_>>();
    let candidate_indices = genome
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            if node.output_definitions.len() < config.max_outputs_per_node {
                Some(index)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if candidate_indices.is_empty() {
        return false;
    }

    let target_index = candidate_indices[rng.choose_index(candidate_indices.len())];
    let node = &mut genome.nodes[target_index];

    let output = if rng.chance(0.5) {
        let target_node_id = node_ids[rng.choose_index(node_ids.len())];
        OutputDefinition::InternalTarget(InternalTargetDef {
            target_node_id,
            input_refs: vec![InputReference::World(crate::mesh::WorldInputKey::FoodHere)],
            payload_fields: vec![PayloadField::Scalar {
                key: format!("signal_{}", node.output_definitions.len()),
                value: 1,
            }],
        })
    } else {
        OutputDefinition::WorldAction(WorldActionDef {
            action_kind: WorldActionKind::Move,
            action_metadata_fields: vec![ActionMetadataField::Direction(0)],
        })
    };

    node.output_definitions.push(output);
    true
}

fn remove_output(genome: &mut CreatureGenome, rng: &mut Lcg64) -> bool {
    let candidate_indices = genome
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            if node.output_definitions.is_empty() {
                None
            } else {
                Some(index)
            }
        })
        .collect::<Vec<_>>();

    if candidate_indices.is_empty() {
        return false;
    }

    let node_index = candidate_indices[rng.choose_index(candidate_indices.len())];
    let node = &mut genome.nodes[node_index];
    let output_index = rng.choose_index(node.output_definitions.len());
    node.output_definitions.remove(output_index);
    true
}

fn retarget_output(genome: &mut CreatureGenome, rng: &mut Lcg64) -> bool {
    let node_ids = genome
        .nodes
        .iter()
        .map(|node| node.node_id)
        .collect::<Vec<_>>();
    for node in &mut genome.nodes {
        for output in &mut node.output_definitions {
            if let OutputDefinition::InternalTarget(target) = output
                && let Some(new_target) = choose_different_id(&node_ids, target.target_node_id, rng)
            {
                target.target_node_id = new_target;
                return true;
            }
        }
    }

    false
}

fn graph_local_mutation(genome: &mut CreatureGenome, rng: &mut Lcg64) -> bool {
    for node in &mut genome.nodes {
        if let BackendDef::Graph(graph) = &mut node.backend_def {
            graph.bias = clamp_finite(graph.bias + rng.signed_unit() * 0.25);
            if !graph.coefficients.is_empty() {
                let index = rng.choose_index(graph.coefficients.len());
                graph.coefficients[index] =
                    clamp_finite(graph.coefficients[index] + rng.signed_unit() * 0.25);
            }

            if let Some(slot) = node.local_state_init.first_mut() {
                *slot = slot.wrapping_add(1);
            } else {
                node.local_state_init.push(rng.next_u32().to_le_bytes()[0]);
            }
            return true;
        }
    }

    false
}

fn vm_instruction_mutation(
    genome: &mut CreatureGenome,
    config: &MutationConfig,
    rng: &mut Lcg64,
) -> bool {
    let mut vm_node = genome
        .nodes
        .iter_mut()
        .find(|node| matches!(node.backend_def, BackendDef::Vm(_)));
    let Some(node) = vm_node.as_mut() else {
        return false;
    };

    let BackendDef::Vm(vm) = &mut node.backend_def else {
        return false;
    };

    let max_len = config.max_vm_program_len.min(128);
    match rng.next_u32() % 3 {
        0 => {
            let index = rng.choose_index(vm.program.len());
            vm.program[index] =
                random_vm_instruction(rng, vm.register_count.max(1), vm.constants.len());
        }
        1 => {
            if vm.program.len() < max_len {
                let index = rng.choose_index(vm.program.len() + 1);
                vm.program.insert(index, VmInstruction::Noop);
            } else {
                let index = rng.choose_index(vm.program.len());
                vm.program[index] = VmInstruction::Noop;
            }
        }
        _ => {
            if vm.program.len() > 1 {
                let index = rng.choose_index(vm.program.len());
                vm.program.remove(index);
            } else {
                vm.program[0] = VmInstruction::Halt;
            }
        }
    }

    if vm.program.is_empty() {
        vm.program.push(VmInstruction::Halt);
    }

    if vm.program.len() > max_len {
        vm.program.truncate(max_len);
    }

    true
}

fn node_duplication(genome: &mut CreatureGenome, config: &MutationConfig, rng: &mut Lcg64) -> bool {
    if genome.nodes.len() >= config.max_nodes {
        return false;
    }

    let Some(source) = genome
        .nodes
        .iter()
        .find(|node| !node.output_definitions.is_empty())
        .cloned()
        .or_else(|| genome.nodes.first().cloned())
    else {
        return false;
    };

    let source_id = source.node_id;
    let new_id = next_node_id(&genome.nodes);

    let mut duplicated = source;
    duplicated.node_id = new_id;

    if !rng.chance(config.node_duplication_clone_outputs_probability) {
        duplicated.output_definitions.clear();
    } else {
        for output in &mut duplicated.output_definitions {
            if let OutputDefinition::InternalTarget(target) = output
                && target.target_node_id == source_id
                && rng.chance(config.node_duplication_target_remap_probability)
            {
                target.target_node_id = new_id;
            }
        }
    }

    genome.nodes.push(duplicated);
    true
}

fn subgraph_duplication(
    genome: &mut CreatureGenome,
    config: &MutationConfig,
    rng: &mut Lcg64,
) -> bool {
    if genome.nodes.len() >= config.max_nodes {
        return false;
    }

    let budget = config
        .max_subgraph_duplication_nodes
        .min(config.max_nodes.saturating_sub(genome.nodes.len()));
    if budget == 0 {
        return false;
    }

    let by_id = genome
        .nodes
        .iter()
        .map(|node| (node.node_id, node))
        .collect::<HashMap<_, _>>();

    let mut selected = Vec::new();
    let mut seen = HashSet::new();
    let mut queue = VecDeque::from([genome.entry_node_id]);

    while let Some(node_id) = queue.pop_front() {
        if !seen.insert(node_id) {
            continue;
        }

        selected.push(node_id);
        if selected.len() >= budget {
            break;
        }

        if let Some(node) = by_id.get(&node_id) {
            for output in &node.output_definitions {
                if let OutputDefinition::InternalTarget(target) = output {
                    queue.push_back(target.target_node_id);
                }
            }
        }
    }

    if selected.is_empty() {
        return false;
    }

    let selected_set = selected.iter().copied().collect::<HashSet<_>>();
    let mut mapping = HashMap::with_capacity(selected.len());
    let mut next_id = next_node_id(&genome.nodes);
    for source_id in &selected {
        mapping.insert(*source_id, next_id);
        next_id = next_id.saturating_add(1);
    }

    let mapped_targets = mapping.values().copied().collect::<Vec<_>>();
    let mut clones = Vec::with_capacity(selected.len());
    for source_id in &selected {
        let Some(source) = by_id.get(source_id) else {
            continue;
        };

        let mut clone = (*source).clone();
        clone.node_id = *mapping
            .get(source_id)
            .expect("mapping contains every selected source");

        for output in &mut clone.output_definitions {
            if let OutputDefinition::InternalTarget(target) = output {
                if let Some(mapped) = mapping.get(&target.target_node_id) {
                    target.target_node_id = *mapped;
                } else if rng.chance(config.subgraph_external_edge_retarget_probability) {
                    target.target_node_id = mapped_targets[rng.choose_index(mapped_targets.len())];
                }
            }
        }

        clones.push(clone);
    }

    if rng.chance(config.subgraph_external_edge_retarget_probability) {
        for node in &mut genome.nodes {
            if selected_set.contains(&node.node_id) {
                continue;
            }
            for output in &mut node.output_definitions {
                if let OutputDefinition::InternalTarget(target) = output
                    && let Some(mapped) = mapping.get(&target.target_node_id)
                {
                    target.target_node_id = *mapped;
                }
            }
        }
    }

    genome.nodes.extend(clones);
    true
}

fn random_vm_instruction(rng: &mut Lcg64, register_count: u8, const_count: usize) -> VmInstruction {
    let register = |rng: &mut Lcg64| -> u8 {
        let count = usize::from(register_count.max(1));
        u8::try_from(rng.choose_index(count)).expect("register index is bounded by u8")
    };

    match rng.next_u32() % 5 {
        0 => VmInstruction::Noop,
        1 => VmInstruction::Halt,
        2 => VmInstruction::Add {
            dst: register(rng),
            a: register(rng),
            b: register(rng),
        },
        3 => VmInstruction::Sub {
            dst: register(rng),
            a: register(rng),
            b: register(rng),
        },
        _ => VmInstruction::LoadConst {
            dst: register(rng),
            const_idx: u8::try_from(if const_count == 0 {
                0
            } else {
                rng.choose_index(const_count)
            })
            .expect("const index is bounded by u8"),
        },
    }
}

fn choose_different_id(ids: &[NodeId], current: NodeId, rng: &mut Lcg64) -> Option<NodeId> {
    let candidates = ids
        .iter()
        .copied()
        .filter(|candidate| *candidate != current)
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        None
    } else {
        Some(candidates[rng.choose_index(candidates.len())])
    }
}

fn default_graph_node(node_id: NodeId) -> NodeGenome {
    NodeGenome {
        node_id,
        node_type: NodeType::Graph,
        backend_def: BackendDef::Graph(GraphBackendDef {
            operator: GraphOperator::Passthrough,
            inputs: Vec::new(),
            coefficients: Vec::new(),
            bias: 0.0,
            state_slot_count: 0,
        }),
        output_definitions: Vec::new(),
        local_state_init: Vec::new(),
    }
}

fn default_vm_node(node_id: NodeId) -> NodeGenome {
    NodeGenome {
        node_id,
        node_type: NodeType::Vm,
        backend_def: BackendDef::Vm(VmBackendDef {
            register_count: 4,
            program: vec![VmInstruction::Noop, VmInstruction::Halt],
            constants: vec![0.0],
            max_input_slots: 8,
        }),
        output_definitions: Vec::new(),
        local_state_init: Vec::new(),
    }
}

fn next_node_id(nodes: &[NodeGenome]) -> NodeId {
    nodes
        .iter()
        .map(|node| node.node_id)
        .max()
        .unwrap_or(0)
        .saturating_add(1)
}

fn clamp_finite(value: f32) -> f32 {
    if !value.is_finite() {
        0.0
    } else {
        value.clamp(-1_000.0, 1_000.0)
    }
}

#[derive(Clone, Debug)]
struct Lcg64 {
    state: u64,
}

impl Lcg64 {
    fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(0x9E37_79B9_7F4A_7C15),
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.state
    }

    fn next_u32(&mut self) -> u32 {
        let bytes = self.next_u64().to_le_bytes();
        u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]])
    }

    fn next_f32(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }

    fn signed_unit(&mut self) -> f32 {
        self.next_f32() * 2.0 - 1.0
    }

    fn choose_index(&mut self, len: usize) -> usize {
        if len == 0 {
            return 0;
        }

        let sample = usize::try_from(self.next_u32()).unwrap_or(0);
        sample % len
    }

    fn chance(&mut self, probability: f32) -> bool {
        self.next_f32() <= probability
    }
}
