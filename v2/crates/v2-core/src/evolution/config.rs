#[derive(Clone, Debug, PartialEq)]
pub struct MutationWeights {
    pub add_node: f32,
    pub remove_node: f32,
    pub retarget_node: f32,
    pub add_output: f32,
    pub remove_output: f32,
    pub retarget_output: f32,
    pub graph_local_mutation: f32,
    pub vm_instruction_mutation: f32,
    pub node_duplication: f32,
    pub subgraph_duplication: f32,
}

impl Default for MutationWeights {
    fn default() -> Self {
        Self {
            add_node: 0.10,
            remove_node: 0.06,
            retarget_node: 0.06,
            add_output: 0.10,
            remove_output: 0.08,
            retarget_output: 0.12,
            graph_local_mutation: 0.12,
            vm_instruction_mutation: 0.20,
            node_duplication: 0.10,
            subgraph_duplication: 0.06,
        }
    }
}

impl MutationWeights {
    #[must_use]
    pub fn total(&self) -> f32 {
        self.add_node
            + self.remove_node
            + self.retarget_node
            + self.add_output
            + self.remove_output
            + self.retarget_output
            + self.graph_local_mutation
            + self.vm_instruction_mutation
            + self.node_duplication
            + self.subgraph_duplication
    }

    fn entries(&self) -> [(&'static str, f32); 10] {
        [
            ("add_node", self.add_node),
            ("remove_node", self.remove_node),
            ("retarget_node", self.retarget_node),
            ("add_output", self.add_output),
            ("remove_output", self.remove_output),
            ("retarget_output", self.retarget_output),
            ("graph_local_mutation", self.graph_local_mutation),
            ("vm_instruction_mutation", self.vm_instruction_mutation),
            ("node_duplication", self.node_duplication),
            ("subgraph_duplication", self.subgraph_duplication),
        ]
    }

    pub fn validate(&self) -> Result<(), MutationConfigError> {
        for (name, value) in self.entries() {
            if value.is_nan() || value.is_sign_negative() {
                return Err(MutationConfigError::InvalidWeight(name));
            }
        }

        let total = self.total();
        if (total - 1.0).abs() > 1e-6 {
            return Err(MutationConfigError::InvalidWeightTotal(total));
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MutationConfig {
    pub per_birth_mutation_events_min: u8,
    pub per_birth_mutation_events_max: u8,
    pub max_nodes: usize,
    pub min_nodes: usize,
    pub max_outputs_per_node: usize,
    pub max_subgraph_duplication_nodes: usize,
    pub max_vm_program_len: usize,
    pub node_duplication_clone_outputs_probability: f32,
    pub node_duplication_target_remap_probability: f32,
    pub subgraph_external_edge_retarget_probability: f32,
    pub weights: MutationWeights,
}

impl Default for MutationConfig {
    fn default() -> Self {
        Self {
            per_birth_mutation_events_min: 2,
            per_birth_mutation_events_max: 8,
            max_nodes: 64,
            min_nodes: 1,
            max_outputs_per_node: 8,
            max_subgraph_duplication_nodes: 6,
            max_vm_program_len: 128,
            node_duplication_clone_outputs_probability: 1.0,
            node_duplication_target_remap_probability: 0.35,
            subgraph_external_edge_retarget_probability: 0.0,
            weights: MutationWeights::default(),
        }
    }
}

impl MutationConfig {
    pub fn validate(&self) -> Result<(), MutationConfigError> {
        if self.per_birth_mutation_events_min == 0
            || self.per_birth_mutation_events_min > self.per_birth_mutation_events_max
        {
            return Err(MutationConfigError::InvalidMutationEventBounds);
        }

        if self.min_nodes == 0 || self.min_nodes > self.max_nodes {
            return Err(MutationConfigError::InvalidNodeBounds);
        }

        if self.max_outputs_per_node == 0 {
            return Err(MutationConfigError::InvalidOutputBound);
        }

        if self.max_subgraph_duplication_nodes == 0 {
            return Err(MutationConfigError::InvalidSubgraphBound);
        }

        if self.max_vm_program_len == 0 || self.max_vm_program_len > 128 {
            return Err(MutationConfigError::InvalidVmProgramBound);
        }

        validate_probability(
            self.node_duplication_clone_outputs_probability,
            "node_duplication_clone_outputs_probability",
        )?;
        validate_probability(
            self.node_duplication_target_remap_probability,
            "node_duplication_target_remap_probability",
        )?;
        validate_probability(
            self.subgraph_external_edge_retarget_probability,
            "subgraph_external_edge_retarget_probability",
        )?;

        self.weights.validate()
    }
}

fn validate_probability(value: f32, field: &'static str) -> Result<(), MutationConfigError> {
    if value.is_nan() || !(0.0..=1.0).contains(&value) {
        return Err(MutationConfigError::InvalidProbability(field));
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq)]
pub enum MutationConfigError {
    InvalidMutationEventBounds,
    InvalidNodeBounds,
    InvalidOutputBound,
    InvalidSubgraphBound,
    InvalidVmProgramBound,
    InvalidProbability(&'static str),
    InvalidWeight(&'static str),
    InvalidWeightTotal(f32),
}
