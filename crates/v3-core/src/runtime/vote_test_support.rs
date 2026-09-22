//! Test builders for vote genomes (T19.F04): VM nodes that vote constants,
//! a small graph builder, sensor snapshots, and a runner that executes one
//! tick in all three execution modes and asserts they agree.

use crate::config::RuntimeConfig;
use crate::contracts::{InputReference, NodeId, RouteTarget, WorldAction};
use crate::creature::genome::cgp::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource, OutputSinkKind,
};
use crate::creature::genome::vote::{VoteKind, VoteSink};
use crate::creature::genome::{
    BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
};
use crate::creature::state::GraphRuntimeState;
use crate::runtime::mesh::{
    execute_creature_mesh_impl, ObservedMeshExecution, UntracedMeshExecution,
};
use crate::runtime::trace::domain::{MeshHopTrace, MeshPassTrace};
use crate::runtime::traced_mesh::execute_creature_mesh_traced;
use crate::runtime::types::MeshOutput;
use crate::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
use crate::sensors::static_inputs::StaticInputs;
use crate::sensors::typed_food::TypedFoodLocalSnapshot;

/// Route targets on slots `0..`, all at bias 0.
pub(crate) fn targets(ids: &[u32]) -> Vec<RouteTarget> {
    ids.iter()
        .enumerate()
        .map(|(slot, &id)| RouteTarget {
            target_id: NodeId::new(id),
            slot: slot as u8,
            gate_bias: 0.0,
        })
        .collect()
}

/// A VM node that votes each constant into its sink, then halts.
pub(crate) fn vm_voter(id: u32, votes: &[(VoteSink, f32)], target_ids: &[u32]) -> NodeGenome {
    let mut program = Vec::with_capacity(votes.len() * 2 + 1);
    for (index, (sink, _)) in votes.iter().enumerate() {
        program.push(VmInstruction::LoadConst {
            dst: 0,
            const_idx: index as u8,
        });
        program.push(VmInstruction::AddVote {
            sink: sink.index() as u8,
            src: 0,
        });
    }
    program.push(VmInstruction::Halt);
    NodeGenome {
        node_id: NodeId::new(id),
        input_refs: vec![],
        backend_def: BackendDef::Vm(VmBackendDef {
            register_count: 1,
            constants: votes.iter().map(|(_, value)| *value).collect(),
            program,
        }),
        targets: targets(target_ids),
    }
}

/// A VM node running `program` over `constants` with `registers` registers.
pub(crate) fn vm_node(
    id: u32,
    registers: u8,
    constants: Vec<f32>,
    program: Vec<VmInstruction>,
    input_refs: Vec<InputReference>,
    target_ids: &[u32],
) -> NodeGenome {
    NodeGenome {
        node_id: NodeId::new(id),
        input_refs,
        backend_def: BackendDef::Vm(VmBackendDef {
            register_count: registers,
            constants,
            program,
        }),
        targets: targets(target_ids),
    }
}

/// A graph backend under construction on the full fixed catalog.
pub(crate) struct GraphBuilder(pub(crate) CgpGraphBackendDef);

impl GraphBuilder {
    pub(crate) fn new() -> Self {
        Self(CgpGraphBackendDef::new_with_fixed_outputs())
    }

    /// Append a compute node and return its source.
    pub(crate) fn node(
        &mut self,
        kind: ComputeNodeKind,
        inputs: &[(GraphSource, f32)],
    ) -> GraphSource {
        let index = self.0.compute_nodes.len() as u16;
        self.0.compute_nodes.push(ComputeNode {
            kind,
            inputs: inputs
                .iter()
                .map(|&(source, weight)| GraphEdge { source, weight })
                .collect(),
            plasticity: None,
        });
        GraphSource::ComputeNode(index)
    }

    /// A constant node.
    pub(crate) fn constant(&mut self, value: f32) -> GraphSource {
        self.node(ComputeNodeKind::Constant(value), &[])
    }

    /// Append edges to the catalog sink of `kind`.
    pub(crate) fn sink(
        &mut self,
        kind: OutputSinkKind,
        inputs: &[(GraphSource, f32)],
    ) -> &mut Self {
        self.0
            .output_sinks
            .iter_mut()
            .find(|sink| sink.kind == kind)
            .expect("catalog sink")
            .inputs
            .extend(
                inputs
                    .iter()
                    .map(|&(source, weight)| GraphEdge { source, weight }),
            );
        self
    }

    /// Append edges to the vote sink `sink`.
    pub(crate) fn vote(&mut self, sink: VoteSink, inputs: &[(GraphSource, f32)]) -> &mut Self {
        self.sink(OutputSinkKind::ActionVote(sink), inputs)
    }

    /// Append edges to the parameter sink `(kind, slot)`.
    pub(crate) fn param(
        &mut self,
        kind: VoteKind,
        slot: u8,
        inputs: &[(GraphSource, f32)],
    ) -> &mut Self {
        self.sink(OutputSinkKind::ActionParam(kind, slot), inputs)
    }

    pub(crate) fn build(
        self,
        id: u32,
        input_refs: Vec<InputReference>,
        target_ids: &[u32],
    ) -> NodeGenome {
        NodeGenome {
            node_id: NodeId::new(id),
            input_refs,
            backend_def: BackendDef::Graph(self.0),
            targets: targets(target_ids),
        }
    }
}

pub(crate) fn leaf(ref_idx: u16, sub_idx: u16) -> GraphSource {
    GraphSource::InputLeaf { ref_idx, sub_idx }
}

pub(crate) fn genome(nodes: Vec<NodeGenome>) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: nodes.first().map_or(NodeId::new(0), |node| node.node_id),
        nodes,
    }
}

/// Unit-scale local sensors on one food type.
#[derive(Clone, Copy, Default)]
pub(crate) struct Senses {
    pub(crate) food_here: f32,
    pub(crate) food: [f32; 8],
    pub(crate) barrier: [f32; 8],
    pub(crate) occupied: [f32; 8],
    /// `AgeTicks` on its unit scale.
    pub(crate) age: f32,
}

impl Senses {
    pub(crate) fn snapshot(self) -> SensorSnapshot {
        SensorSnapshot {
            local: StaticInputs {
                food_here: self.food_here,
                neighbor_food: self.food,
                neighbor_barrier: self.barrier,
                neighbor_occupied: self.occupied,
                max_energy: 200.0,
                age_ticks: self.age,
            },
            typed_local_food: TypedFoodLocalSnapshot {
                food_here_by_type: vec![self.food_here],
                neighbor_food_by_type: vec![self.food],
            },
            perception: PerceptionSnapshot::zeroed(1),
        }
    }
}

/// One tick of every execution mode, asserted to agree.
pub(crate) struct Tick {
    pub(crate) output: MeshOutput,
    pub(crate) hops: Vec<MeshHopTrace>,
    pub(crate) passes: Vec<MeshPassTrace>,
    pub(crate) energy: f32,
}

impl Tick {
    pub(crate) fn actions(&self) -> &[WorldAction] {
        &self.output.actions
    }

    pub(crate) fn committed(&self) -> Vec<Option<WorldAction>> {
        self.passes.iter().map(|pass| pass.committed).collect()
    }
}

/// Run one tick of `genome` in the plain, observed, and traced modes from
/// the same state, assert that they agree on every applied outcome, and
/// return the traced run.
pub(crate) fn run_tick(
    genome: &CreatureGenome,
    senses: Senses,
    config: &RuntimeConfig,
    energy: f32,
    memory: [f32; 16],
) -> Tick {
    let sensors = senses.snapshot();
    let mut energies = [energy; 3];
    let mut memories = [memory; 3];
    let mut states: [GraphRuntimeState; 3] = std::array::from_fn(|_| GraphRuntimeState::new());
    for state in &mut states {
        state.begin_tick(&genome.nodes, 0);
    }
    let [plain_state, observed_state, traced_state] = &mut states;
    let [plain_energy, observed_energy, traced_energy] = &mut energies;
    let [plain_memory, observed_memory, traced_memory] = &mut memories;
    let plain = execute_creature_mesh_impl(
        genome,
        &sensors,
        plain_energy,
        plain_memory,
        &[0.0; 16],
        plain_state,
        config,
        UntracedMeshExecution,
    );
    let (observed, observation) = execute_creature_mesh_impl(
        genome,
        &sensors,
        observed_energy,
        observed_memory,
        &[0.0; 16],
        observed_state,
        config,
        ObservedMeshExecution::default(),
    );
    let (traced, hops, passes) = execute_creature_mesh_traced(
        genome,
        &sensors,
        traced_energy,
        traced_memory,
        &[0.0; 16],
        traced_state,
        config,
    );
    for other in [&observed, &traced] {
        assert_eq!(plain.actions, other.actions);
        assert_eq!(plain.termination_reason, other.termination_reason);
        assert_eq!(plain.work_counters, other.work_counters);
        assert_eq!(plain.commit_counts, other.commit_counts);
        assert_eq!(plain.priority_bid.to_bits(), other.priority_bid.to_bits());
        assert_eq!(plain.energy_observation, other.energy_observation);
    }
    assert_eq!(energies.map(f32::to_bits), [energies[0].to_bits(); 3]);
    assert_eq!(memories, [memories[0]; 3]);
    assert_eq!(states[0].node_state, states[1].node_state);
    assert_eq!(states[0].node_state, states[2].node_state);
    assert_eq!(passes.len(), plain.work_counters.passes as usize);
    assert_eq!(hops.len(), plain.work_counters.mesh_hops as usize);
    assert_eq!(observation.hops.len(), hops.len());
    Tick {
        output: traced,
        hops,
        passes,
        energy: energies[0],
    }
}
