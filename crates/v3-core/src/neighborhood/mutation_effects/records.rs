//! The two named records a traced execution yields (T11.F26): what the mesh
//! computed (the computation record) and what it left behind (the
//! state-and-cost record). Floats are held as bit patterns so equality is
//! bit-exact: `-0.0` differs from `0.0` and equal NaN payloads agree.

use std::collections::{BTreeMap, BTreeSet};

use crate::config::RuntimeConfig;
use crate::contracts::{NodeId, WorldAction};
use crate::creature::genome::CreatureGenome;
use crate::creature::state::GraphRuntimeState;
use crate::runtime::mesh::UntracedMeshExecution;
use crate::runtime::trace::domain::{BackendTrace, MeshHopTrace, MeshPassTrace};
use crate::runtime::traced_mesh::RecordingMeshExecution;
use crate::runtime::types::{MeshOutput, WorkCounters};

use super::super::battery::{run_panel, PostState, Scenario};
use super::super::{Battery, Signature};

fn bits<const N: usize>(values: &[f32; N]) -> [u32; N] {
    values.map(f32::to_bits)
}

fn bits_of(values: &[f32]) -> Vec<u32> {
    values.iter().map(|value| value.to_bits()).collect()
}

/// One applied backend write: a Graph output sink (by catalog position) or a
/// VM shared-memory slot write. Only values are recorded; the genome content
/// that produced them is not.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Write {
    Sink { position: u16, value: u32 },
    Slot { slot: u8, value: u32 },
}

/// The route a hop applied: the selected target position and node, and every
/// gate's computed runtime and effective score in gate order. Gate biases,
/// slots and candidate target ids are genome content and are not recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RouteRecord {
    selected_index: usize,
    selected_id: NodeId,
    scores: Vec<(u32, u32)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HopRecord {
    node_id: NodeId,
    upstream: Vec<u32>,
    output: Vec<u32>,
    votes: Vec<u32>,
    route: Option<RouteRecord>,
    writes: Vec<Write>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PassRecord {
    votes: Vec<u32>,
    effective: Vec<u32>,
    /// The committed action's `Debug` rendering, which keeps `-0.0` apart.
    committed: Option<String>,
}

/// What one execution computed, hop by hop and pass by pass.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(super) struct ComputationRecord {
    hops: Vec<HopRecord>,
    passes: Vec<PassRecord>,
}

impl ComputationRecord {
    fn from_trace(hops: &[MeshHopTrace], passes: &[MeshPassTrace]) -> Self {
        Self {
            hops: hops.iter().map(hop_record).collect(),
            passes: passes
                .iter()
                .map(|pass| PassRecord {
                    votes: bits(&pass.votes).to_vec(),
                    effective: bits(&pass.effective_votes).to_vec(),
                    committed: pass.committed.map(|action| format!("{action:?}")),
                })
                .collect(),
        }
    }
}

fn hop_record(hop: &MeshHopTrace) -> HopRecord {
    let writes = match &hop.backend_trace {
        BackendTrace::Vm(trace) => trace
            .slot_writes
            .iter()
            .map(|write| Write::Slot {
                slot: write.slot_idx,
                value: write.new_value.to_bits(),
            })
            .collect(),
        BackendTrace::Graph(trace) => trace
            .output_sinks
            .iter()
            .enumerate()
            .filter(|(_, sink)| sink.applied)
            .map(|(position, sink)| Write::Sink {
                position: position as u16,
                value: sink.applied_value.to_bits(),
            })
            .collect(),
    };
    HopRecord {
        node_id: hop.node_id,
        upstream: bits(&hop.upstream_slots).to_vec(),
        output: bits(&hop.output_slots).to_vec(),
        votes: bits(&hop.vote_contribution).to_vec(),
        route: hop.route.as_ref().map(|route| RouteRecord {
            selected_index: route.selected_target_idx,
            selected_id: route.selected_target_id,
            scores: route
                .gate_scores
                .iter()
                .map(|gate| (gate.runtime_score.to_bits(), gate.effective_score.to_bits()))
                .collect(),
        }),
        writes,
    }
}

/// One node's graph runtime state: operator state, held outputs, learned
/// weights and eligibility traces, trailing empty placeholders trimmed.
type NodeState = [Vec<Vec<u32>>; 4];

fn trimmed(mut rows: Vec<Vec<u32>>) -> Vec<Vec<u32>> {
    while rows.last().is_some_and(Vec::is_empty) {
        rows.pop();
    }
    rows
}

/// What one execution left behind: shared memory, remaining energy, work
/// counters, the paid priority bid, and graph runtime state by node id.
/// A node whose state stores no scalar is omitted, so a node present on one
/// side only differs exactly when it stores some scalar, zeros included.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StateRecord {
    shared_memory: [u32; 16],
    energy: u32,
    work: WorkCounters,
    priority_bid: u32,
    graph: BTreeMap<NodeId, NodeState>,
}

impl StateRecord {
    fn read(genome: &CreatureGenome, output: &MeshOutput, state: &PostState<'_>) -> Self {
        Self {
            shared_memory: bits(state.shared_memory),
            energy: state.energy.to_bits(),
            work: output.work_counters,
            priority_bid: output.priority_bid.to_bits(),
            graph: graph_state(genome, state.graph_runtime),
        }
    }
}

/// Graph runtime state by node id, omitting nodes that store no scalar.
fn graph_state(
    genome: &CreatureGenome,
    runtime: &GraphRuntimeState,
) -> BTreeMap<NodeId, NodeState> {
    let row = |values: Option<&Vec<f32>>| trimmed(vec![bits_of(values.map_or(&[], Vec::as_slice))]);
    let rows = |values: Option<&Vec<Box<[f32]>>>| {
        trimmed(values.map_or_else(Vec::new, |rows| {
            rows.iter().map(|row| bits_of(row)).collect()
        }))
    };
    genome
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            let entry: NodeState = [
                row(runtime.node_state.get(index)),
                row(runtime.node_outputs.get(index)),
                rows(runtime.plasticity_weights.get(index)),
                rows(runtime.eligibility_traces.get(index)),
            ];
            entry
                .iter()
                .any(|field| !field.is_empty())
                .then_some((node.node_id, entry))
        })
        .collect()
}

/// One traced execution's actions and its two records.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct TracedExecution {
    pub(super) actions: Vec<WorldAction>,
    pub(super) computation: ComputationRecord,
    pub(super) state: StateRecord,
}

/// A genome's complete traced battery reading: every execution in signature
/// order and every node it dispatched anywhere.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct TracedBattery {
    pub(super) executions: Vec<TracedExecution>,
    pub(super) dispatched: BTreeSet<NodeId>,
}

impl TracedBattery {
    /// Execute the whole battery with trace recording.
    pub(super) fn run(
        battery: &Battery,
        genome: &CreatureGenome,
        runtime: &RuntimeConfig,
        decay_rate: f32,
    ) -> Self {
        let max_hops = runtime.max_mesh_hops.max(1) as usize;
        let mut dispatched = BTreeSet::new();
        let executions = battery.run_reading(
            genome,
            runtime,
            decay_rate,
            || RecordingMeshExecution::new(max_hops),
            |(output, hops, passes), state| {
                dispatched.extend(hops.iter().map(|hop| hop.node_id));
                TracedExecution {
                    computation: ComputationRecord::from_trace(&hops, &passes),
                    state: StateRecord::read(genome, &output, &state),
                    actions: output.actions,
                }
            },
        );
        Self {
            executions,
            dispatched,
        }
    }

    /// The action signature, in `Battery::signature`'s shape.
    pub(super) fn signature(&self, battery: &Battery) -> Signature {
        let snapshot_count = battery.snapshots().len();
        let mut actions = self
            .executions
            .iter()
            .map(|execution| execution.actions.clone());
        let snapshots = actions.by_ref().take(snapshot_count).collect();
        let sequences = battery
            .sequence_lengths()
            .map(|len| actions.by_ref().take(len).collect())
            .collect();
        Signature {
            snapshots,
            sequences,
        }
    }

    pub(super) fn computation_differs(&self, other: &Self) -> bool {
        self.executions
            .iter()
            .zip(&other.executions)
            .any(|(a, b)| a.computation != b.computation)
    }

    pub(super) fn state_differs(&self, other: &Self) -> bool {
        self.executions
            .iter()
            .zip(&other.executions)
            .any(|(a, b)| a.state != b.state)
    }
}

/// One untraced execution on a coverage panel: actions and the state-and-cost
/// record.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct PanelExecution {
    pub(super) actions: Vec<WorldAction>,
    pub(super) state: StateRecord,
}

/// Run `singles` then `sequences` untraced, reading actions and state.
pub(super) fn run_panel_executions(
    genome: &CreatureGenome,
    singles: &[Scenario],
    sequences: &[Vec<Scenario>],
    runtime: &RuntimeConfig,
    decay_rate: f32,
) -> Vec<PanelExecution> {
    run_panel(
        genome,
        singles,
        sequences,
        runtime,
        decay_rate,
        || UntracedMeshExecution,
        |output, state| PanelExecution {
            state: StateRecord::read(genome, &output, &state),
            actions: output.actions,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A genome whose nodes carry `ids`, in that order.
    fn genome(ids: &[u32]) -> CreatureGenome {
        let node = super::super::controls::graph_genome(Vec::new(), Vec::new(), Vec::new()).nodes
            [0]
        .clone();
        CreatureGenome {
            entry_node_id: NodeId::new(ids[0]),
            nodes: ids
                .iter()
                .map(|&id| crate::creature::genome::NodeGenome {
                    node_id: NodeId::new(id),
                    ..node.clone()
                })
                .collect(),
        }
    }

    fn runtime(node_state: Vec<Vec<f32>>) -> GraphRuntimeState {
        let mut runtime = GraphRuntimeState::new();
        runtime.node_state = node_state;
        runtime
    }

    #[test]
    fn empty_placeholders_hold_no_state_but_a_stored_zero_does() {
        let two = genome(&[0, 1]);
        let none = graph_state(&two, &runtime(Vec::new()));
        assert_eq!(
            none,
            graph_state(&two, &runtime(vec![Vec::new(), Vec::new()]))
        );
        let mut placeholders = GraphRuntimeState::new();
        placeholders.plasticity_weights = vec![vec![Box::from([])], Vec::new()];
        assert_eq!(none, graph_state(&two, &placeholders));
        assert_ne!(none, graph_state(&two, &runtime(vec![vec![0.0]])));
        assert_ne!(
            graph_state(&two, &runtime(vec![vec![0.0]])),
            graph_state(&two, &runtime(vec![vec![-0.0]]))
        );
    }

    #[test]
    fn a_node_present_on_one_side_differs_exactly_when_it_stores_a_scalar() {
        let parent = genome(&[0]);
        let child = genome(&[0, 7]);
        let base = graph_state(&parent, &runtime(vec![vec![1.0]]));
        assert_eq!(
            base,
            graph_state(&child, &runtime(vec![vec![1.0], Vec::new()]))
        );
        assert_ne!(
            base,
            graph_state(&child, &runtime(vec![vec![1.0], vec![0.0]]))
        );
    }

    #[test]
    fn graph_state_aligns_by_node_id_not_mesh_index() {
        let forward = graph_state(&genome(&[0, 1]), &runtime(vec![vec![1.0], vec![2.0]]));
        let reversed = graph_state(&genome(&[1, 0]), &runtime(vec![vec![2.0], vec![1.0]]));
        assert_eq!(forward, reversed);
    }
}
