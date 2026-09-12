//! Mesh chain executor — evaluates a creature's genome mesh each tick.
//!
//! The mesh executor walks a chain of [`NodeGenome`] nodes starting from
//! `genome.entry_node_id`, dispatching each node to its backend (VM or Graph),
//! and routing to the next node via the returned internal routing decision until
//! a terminal condition or soft-default termination condition fires.

use crate::config::RuntimeConfig;
use crate::contracts::{NodeId, WorldAction};
use crate::creature::genome::{BackendDef, CreatureGenome, NodeGenome};
use crate::creature::state::GraphRuntimeState;
use crate::runtime::cgp::execute_graph_node;
use crate::runtime::routing::resolve_gated_route_where;
use crate::runtime::trace::domain::TerminationReason;
use crate::runtime::types::{ComputeCostReport, MeshOutput, MeshSideOutputs, OUTPUT_SLOT_COUNT};
use crate::runtime::vm::execute_vm_node;
use crate::sensors::perception::SensorSnapshot;

/// Execute the creature's mesh chain within the current tick, returning a [`MeshOutput`]
/// containing the queued actions, a [`ComputeCostReport`], and the priority bid.
///
/// Before the first mesh execution of each new world tick, the caller must call
/// `graph_runtime.begin_tick(&genome.nodes, age)`. Mesh execution does not
/// advance the graph clock. Each mesh node dispatches at most once per tick.
///
/// The function walks the genome's node chain starting at `entry_node_id`,
/// dispatching each node to its VM or Graph backend, routing to subsequent
/// nodes by resolving each internal node result's route-gate scores, and terminating
/// when a terminal instruction is reached or a soft-default condition fires.
///
/// All soft-default termination conditions return `vec![WorldAction::NoOp]`.
/// Energy exhaustion discards the queue and returns `vec![WorldAction::NoOp]`.
/// Halt without ExecuteActionQueue preserves the accumulated queue (forgiving).
///
/// # Arguments
/// - `genome`: the creature's node graph
/// - `sensors`: pre-assembled sensor snapshot (local + extended perception)
/// - `energy`: creature's mutable energy; decremented by node evaluation costs
/// - `shared_memory`: creature's shared f32 memory slots
/// - `prev_shared_memory`: snapshot of shared memory from previous tick
/// - `graph_runtime`: per-node persistent runtime state for Graph backends
/// - `config`: runtime limits (max_mesh_hops, max_vm_steps, etc.)
#[allow(clippy::too_many_arguments)]
pub fn execute_creature_mesh(
    genome: &CreatureGenome,
    sensors: &SensorSnapshot,
    energy: &mut f32,
    shared_memory: &mut [f32; 16],
    prev_shared_memory: &[f32; 16],
    graph_runtime: &mut GraphRuntimeState,
    config: &RuntimeConfig,
) -> MeshOutput {
    execute_creature_mesh_impl(
        genome,
        sensors,
        energy,
        shared_memory,
        prev_shared_memory,
        graph_runtime,
        config,
        UntracedMeshExecution,
    )
}

/// Compile-time seam between normal and trace-recording mesh execution.
pub(crate) trait MeshExecutionMode {
    type BackendTrace;
    type Output;

    const RECORDS_HOPS: bool;

    #[allow(clippy::too_many_arguments)]
    fn execute_node(
        &mut self,
        node: &NodeGenome,
        node_idx: usize,
        upstream_slots: &[f32; OUTPUT_SLOT_COUNT],
        energy: &mut f32,
        energy_consumed: f32,
        shared_memory: &mut [f32; 16],
        prev_shared_memory: &[f32; 16],
        graph_runtime: &mut GraphRuntimeState,
        sensors: &SensorSnapshot,
        config: &RuntimeConfig,
        side_outputs: &mut MeshSideOutputs,
    ) -> (crate::runtime::types::NodeResult, Self::BackendTrace);

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn record_hop(
        &mut self,
        _hop_index: usize,
        _node: &NodeGenome,
        _upstream_slots: [f32; OUTPUT_SLOT_COUNT],
        _energy_before: f32,
        _energy_after: f32,
        _result: &crate::runtime::types::NodeResult,
        _route_result: Option<(usize, NodeId)>,
        _backend_trace: Self::BackendTrace,
    ) {
    }

    /// Consume the finished evaluation. The termination reason travels on
    /// [`MeshOutput::termination_reason`], so every mode reads the same value.
    fn finish(self, output: MeshOutput) -> Self::Output;
}

pub(crate) struct UntracedMeshExecution;

impl MeshExecutionMode for UntracedMeshExecution {
    type BackendTrace = ();
    type Output = MeshOutput;

    const RECORDS_HOPS: bool = false;

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn execute_node(
        &mut self,
        node: &NodeGenome,
        node_idx: usize,
        upstream_slots: &[f32; OUTPUT_SLOT_COUNT],
        energy: &mut f32,
        energy_consumed: f32,
        shared_memory: &mut [f32; 16],
        prev_shared_memory: &[f32; 16],
        graph_runtime: &mut GraphRuntimeState,
        sensors: &SensorSnapshot,
        config: &RuntimeConfig,
        side_outputs: &mut MeshSideOutputs,
    ) -> (crate::runtime::types::NodeResult, ()) {
        let result = match &node.backend_def {
            BackendDef::Vm(def) => execute_vm_node(
                def,
                &node.input_refs,
                upstream_slots,
                energy,
                energy_consumed,
                shared_memory,
                prev_shared_memory,
                sensors,
                config,
                side_outputs,
            ),
            BackendDef::Graph(def) => execute_graph_node(
                def,
                &node.input_refs,
                upstream_slots,
                energy,
                energy_consumed,
                node_idx,
                graph_runtime,
                sensors,
                config,
                side_outputs,
                shared_memory,
                prev_shared_memory,
            ),
        };
        (result, ())
    }

    #[inline]
    fn finish(self, output: MeshOutput) -> MeshOutput {
        output
    }
}

/// Compact observations of actual dispatches and applied routing, without backend traces.
#[derive(Debug, Clone)]
pub(crate) struct MeshObservation {
    pub hops: Vec<(NodeId, Option<usize>)>,
    pub termination_reason: TerminationReason,
}

#[derive(Default)]
pub(crate) struct ObservedMeshExecution {
    hops: Vec<(NodeId, Option<usize>)>,
}

impl MeshExecutionMode for ObservedMeshExecution {
    type BackendTrace = ();
    type Output = (MeshOutput, MeshObservation);
    // Diagnostic route winners on terminal/exhausted nodes are not applied routes.
    const RECORDS_HOPS: bool = false;

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn execute_node(
        &mut self,
        node: &NodeGenome,
        node_idx: usize,
        upstream_slots: &[f32; OUTPUT_SLOT_COUNT],
        energy: &mut f32,
        energy_consumed: f32,
        shared_memory: &mut [f32; 16],
        prev_shared_memory: &[f32; 16],
        graph_runtime: &mut GraphRuntimeState,
        sensors: &SensorSnapshot,
        config: &RuntimeConfig,
        side_outputs: &mut MeshSideOutputs,
    ) -> (crate::runtime::types::NodeResult, ()) {
        UntracedMeshExecution.execute_node(
            node,
            node_idx,
            upstream_slots,
            energy,
            energy_consumed,
            shared_memory,
            prev_shared_memory,
            graph_runtime,
            sensors,
            config,
            side_outputs,
        )
    }

    fn record_hop(
        &mut self,
        _hop_index: usize,
        node: &NodeGenome,
        _upstream_slots: [f32; OUTPUT_SLOT_COUNT],
        _energy_before: f32,
        _energy_after: f32,
        result: &crate::runtime::types::NodeResult,
        route_result: Option<(usize, NodeId)>,
        _backend_trace: (),
    ) {
        let route = if result.terminal || result.energy_exhausted {
            None
        } else {
            route_result.map(|(position, _)| position)
        };
        self.hops.push((node.node_id, route));
    }

    fn finish(self, output: MeshOutput) -> Self::Output {
        let termination_reason = output.termination_reason;
        (
            output,
            MeshObservation {
                hops: self.hops,
                termination_reason,
            },
        )
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn execute_creature_mesh_impl<M: MeshExecutionMode>(
    genome: &CreatureGenome,
    sensors: &SensorSnapshot,
    energy: &mut f32,
    shared_memory: &mut [f32; 16],
    prev_shared_memory: &[f32; 16],
    graph_runtime: &mut GraphRuntimeState,
    config: &RuntimeConfig,
    mut mode: M,
) -> M::Output {
    let mut current_node_id = genome.entry_node_id;
    let mut upstream_slots = [0.0f32; OUTPUT_SLOT_COUNT];
    let mut hops: usize = 0;
    let max_hops = config.max_mesh_hops.max(1) as usize;
    let mut visited = std::collections::HashSet::with_capacity(max_hops.min(genome.nodes.len()));
    let start_energy = *energy;
    let mut report = ComputeCostReport::default();
    let mut side_outputs = MeshSideOutputs::new(config.max_actions_per_turn);

    // Soft default: entry_node_id missing from node set → return NoOp immediately.
    if find_node_index(&genome.nodes, current_node_id).is_none() {
        let output = MeshOutput {
            actions: vec![WorldAction::NoOp],
            cost_report: report,
            priority_bid: side_outputs.priority_bid,
            work_counters: side_outputs.work_counters,
            energy_observation: side_outputs.energy_observation,
            termination_reason: TerminationReason::MissingNode,
        };
        return mode.finish(output);
    }

    loop {
        if hops >= max_hops {
            let output = MeshOutput {
                actions: side_outputs.action_queue.into_actions_or_noop(),
                cost_report: report,
                priority_bid: side_outputs.priority_bid,
                work_counters: side_outputs.work_counters,
                energy_observation: side_outputs.energy_observation,
                termination_reason: TerminationReason::MaxHopsReached,
            };
            return mode.finish(output);
        }

        // Invariant: verified present before the loop, and after every routing step.
        let current_idx =
            find_node_index(&genome.nodes, current_node_id).expect("node must exist in genome");
        let node = &genome.nodes[current_idx];
        visited.insert(node.node_id);

        let energy_consumed = (start_energy - *energy).max(0.0);

        // One mesh hop is one node dispatch, counted regardless of outcome.
        side_outputs.work_counters.mesh_hops += 1;
        // Same event, recorded for the offspring's mutation targeting (T11.F17).
        // Written from the shared loop, so every execution mode agrees.
        graph_runtime.dispatch_record.record_dispatch(current_idx);

        // Snapshot energy before node dispatch to attribute cost to the correct backend.
        let node_energy_before = *energy;
        let (result, backend_trace) = mode.execute_node(
            node,
            current_idx,
            &upstream_slots,
            energy,
            energy_consumed,
            shared_memory,
            prev_shared_memory,
            graph_runtime,
            sensors,
            config,
            &mut side_outputs,
        );

        // Attribute energy delta to the correct backend.
        let node_cost = (node_energy_before - *energy).max(0.0);
        match &node.backend_def {
            BackendDef::Vm(_) => report.vm_cost += node_cost,
            BackendDef::Graph(_) => report.graph_cost += node_cost,
        }

        let route_result = if M::RECORDS_HOPS || (!result.energy_exhausted && !result.terminal) {
            resolve_gated_route_where(&node.targets, &result.route_gates, |id| {
                !visited.contains(&id)
            })
        } else {
            None
        };

        mode.record_hop(
            hops,
            node,
            upstream_slots,
            node_energy_before,
            *energy,
            &result,
            route_result,
            backend_trace,
        );

        // Check exhaustion first: NodeResult::exhausted() discards the action queue.
        if result.energy_exhausted {
            let output = MeshOutput {
                actions: vec![WorldAction::NoOp],
                cost_report: report,
                priority_bid: side_outputs.priority_bid,
                work_counters: side_outputs.work_counters,
                energy_observation: side_outputs.energy_observation,
                termination_reason: TerminationReason::EnergyExhausted,
            };
            return mode.finish(output);
        }

        if result.terminal {
            let output = MeshOutput {
                actions: side_outputs.action_queue.into_actions_or_noop(),
                cost_report: report,
                priority_bid: side_outputs.priority_bid,
                work_counters: side_outputs.work_counters,
                energy_observation: side_outputs.energy_observation,
                termination_reason: TerminationReason::ActionEmitted,
            };
            return mode.finish(output);
        }

        // Routing via per-target gate scoring.
        match route_result {
            Some((_idx, id)) => {
                if find_node_index(&genome.nodes, id).is_none() {
                    let output = MeshOutput {
                        actions: side_outputs.action_queue.into_actions_or_noop(),
                        cost_report: report,
                        priority_bid: side_outputs.priority_bid,
                        work_counters: side_outputs.work_counters,
                        energy_observation: side_outputs.energy_observation,
                        termination_reason: TerminationReason::MissingNode,
                    };
                    return mode.finish(output);
                }
                upstream_slots = result.output_slots;
                current_node_id = id;
                hops += 1;
            }
            None => {
                let output = MeshOutput {
                    actions: side_outputs.action_queue.into_actions_or_noop(),
                    cost_report: report,
                    priority_bid: side_outputs.priority_bid,
                    work_counters: side_outputs.work_counters,
                    energy_observation: side_outputs.energy_observation,
                    termination_reason: TerminationReason::NoTargets,
                };
                return mode.finish(output);
            }
        }
    }
}

/// Find the index of a node by its `NodeId` via linear scan.
///
/// For typical genomes (2-10 nodes), linear scan is faster than HashMap
/// due to cache locality and zero heap allocation.
#[inline]
fn find_node_index(nodes: &[NodeGenome], id: NodeId) -> Option<usize> {
    nodes.iter().position(|n| n.node_id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RuntimeConfig;
    use crate::contracts::{NodeId, RouteTarget, WorldAction};
    use crate::creature::genome::{
        BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
    };
    use crate::creature::state::GraphRuntimeState;
    use crate::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
    use crate::sensors::static_inputs::StaticInputs;
    use crate::sensors::typed_food::TypedFoodLocalSnapshot;

    fn wrap_targets(ids: Vec<NodeId>) -> Vec<RouteTarget> {
        ids.into_iter()
            .enumerate()
            .map(|(i, id)| RouteTarget {
                target_id: id,
                slot: i as u8,
                gate_bias: 0.0,
            })
            .collect()
    }

    #[test]
    fn compact_observation_records_only_applied_routes_and_real_cap_termination() {
        let id = NodeId::new(0);
        let mut genome = CreatureGenome {
            entry_node_id: id,
            nodes: vec![vm_halt_with_route(id, 1.0, vec![id])],
        };
        let config = RuntimeConfig {
            max_mesh_hops: 2,
            ..default_config()
        };
        let observe = |genome: &CreatureGenome, mut energy| {
            execute_creature_mesh_impl(
                genome,
                &empty_sensor_snapshot(),
                &mut energy,
                &mut [0.0; 16],
                &[0.0; 16],
                &mut GraphRuntimeState::new(),
                &config,
                ObservedMeshExecution::default(),
            )
        };
        let (_, observed) = observe(&genome, 100.0);
        assert_eq!(observed.hops, vec![(id, None)]);
        assert!(matches!(
            observed.termination_reason,
            TerminationReason::NoTargets
        ));
        genome.nodes[0] = vm_emit_node(id, 0, vec![id]);
        let (_, observed) = observe(&genome, 100.0);
        assert_eq!(observed.hops, vec![(id, None)]);
        assert!(matches!(
            observed.termination_reason,
            TerminationReason::ActionEmitted
        ));
        let (_, observed) = observe(&genome, 0.0);
        assert_eq!(observed.hops, vec![(id, None)]);
        assert!(matches!(
            observed.termination_reason,
            TerminationReason::EnergyExhausted
        ));
    }

    #[test]
    fn applied_energy_observations_survive_all_mesh_exits_in_every_mode() {
        let id = NodeId::new(0);
        let next = NodeId::new(1);
        let config = RuntimeConfig {
            max_mesh_hops: 1,
            ..default_config()
        };
        let cases = [
            (vec![], 100.0, TerminationReason::MissingNode),
            (
                vec![vm_halt_with_route(id, 1.0, vec![])],
                100.0,
                TerminationReason::NoTargets,
            ),
            (
                vec![vm_emit_node(id, 0, vec![])],
                100.0,
                TerminationReason::ActionEmitted,
            ),
            (
                vec![vm_emit_node(id, 0, vec![])],
                0.0,
                TerminationReason::EnergyExhausted,
            ),
            (
                vec![vm_halt_with_route(id, 1.0, vec![next])],
                100.0,
                TerminationReason::MissingNode,
            ),
            (
                vec![
                    vm_halt_with_route(id, 1.0, vec![next]),
                    vm_emit_node(next, 0, vec![]),
                ],
                100.0,
                TerminationReason::MaxHopsReached,
            ),
        ];
        for (nodes, start, expected) in cases {
            let genome = CreatureGenome {
                entry_node_id: id,
                nodes,
            };
            let mut energy = start;
            let plain = execute_creature_mesh(
                &genome,
                &empty_sensor_snapshot(),
                &mut energy,
                &mut [0.0; 16],
                &[0.0; 16],
                &mut GraphRuntimeState::new(),
                &config,
            );
            let (observed, _) = execute_creature_mesh_impl(
                &genome,
                &empty_sensor_snapshot(),
                &mut { start },
                &mut [0.0; 16],
                &[0.0; 16],
                &mut GraphRuntimeState::new(),
                &config,
                ObservedMeshExecution::default(),
            );
            let (traced, _, _) = crate::runtime::traced_mesh::execute_creature_mesh_traced(
                &genome,
                &empty_sensor_snapshot(),
                &mut { start },
                &mut [0.0; 16],
                &[0.0; 16],
                &mut GraphRuntimeState::new(),
                &config,
            );
            assert_eq!(plain.termination_reason, expected);
            assert_eq!(plain.energy_observation, observed.energy_observation);
            assert_eq!(plain.energy_observation, traced.energy_observation);
            assert_eq!(
                plain.energy_observation.vm_compute,
                f64::from(start) - f64::from(energy)
            );
        }
    }

    #[test]
    fn compact_observation_preserves_complete_applied_outcomes_and_graph_state() {
        use crate::creature::genome::cgp::{
            CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource,
        };
        let mut graph =
            CgpGraphBackendDef::new_with_fixed_outputs(&crate::config::MutationConfig::default());
        graph.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::DecayIntegrator(0.5),
            inputs: vec![GraphEdge {
                source: GraphSource::SharedMemory {
                    slot: 0,
                    previous: false,
                },
                weight: 1.0,
            }],
            plasticity: Some(crate::creature::genome::PlasticityConfig {
                rule: crate::creature::genome::HebbianRule::Classic,
                learning_rate: 0.1,
                weight_clamp: 2.0,
                lamarckian: false,
                modulation: Some(crate::creature::genome::RewardModulationConfig {
                    reward_source: crate::creature::genome::OutcomeChannel::EnergyDelta,
                    trace_decay: 0.9,
                }),
            }),
        });
        let mut vm = vm_halt_with_route(NodeId::new(0), 1.0, vec![NodeId::new(1)]);
        if let BackendDef::Vm(def) = &mut vm.backend_def {
            def.program.splice(
                2..2,
                [
                    VmInstruction::StoreSlotImm {
                        slot_idx: 0,
                        src: 0,
                    },
                    VmInstruction::SetPriorityBid { src: 0 },
                    VmInstruction::PushAction { action_type: 1 },
                ],
            );
        }
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![
                vm,
                NodeGenome {
                    node_id: NodeId::new(1),
                    input_refs: vec![],
                    backend_def: BackendDef::Graph(graph),
                    targets: wrap_targets(vec![NodeId::new(1)]),
                },
            ],
        };
        for starting_energy in [0.0, 0.01, 2.0, 100.0] {
            let mut energy = starting_energy;
            let mut observed_energy = energy;
            let mut traced_energy = energy;
            let mut memory = [0.5; 16];
            let mut observed_memory = memory;
            let mut traced_memory = memory;
            let previous = [0.25; 16];
            let mut state = GraphRuntimeState::new();
            let mut observed_state = state.clone();
            let mut traced_state = state.clone();
            let config = RuntimeConfig {
                max_mesh_hops: 4,
                ..default_config()
            };
            for tick in 0..2 {
                state.begin_tick(&genome.nodes, tick);
                observed_state.begin_tick(&genome.nodes, tick);
                traced_state.begin_tick(&genome.nodes, tick);
                let plain = execute_creature_mesh_impl(
                    &genome,
                    &empty_sensor_snapshot(),
                    &mut energy,
                    &mut memory,
                    &previous,
                    &mut state,
                    &config,
                    UntracedMeshExecution,
                );
                let (observed, _) = execute_creature_mesh_impl(
                    &genome,
                    &empty_sensor_snapshot(),
                    &mut observed_energy,
                    &mut observed_memory,
                    &previous,
                    &mut observed_state,
                    &config,
                    ObservedMeshExecution::default(),
                );
                let (traced, _, _) = crate::runtime::traced_mesh::execute_creature_mesh_traced(
                    &genome,
                    &empty_sensor_snapshot(),
                    &mut traced_energy,
                    &mut traced_memory,
                    &previous,
                    &mut traced_state,
                    &config,
                );
                assert_eq!(plain.actions, traced.actions);
                assert_eq!(plain.priority_bid, traced.priority_bid);
                assert_eq!(plain.cost_report.vm_cost, traced.cost_report.vm_cost);
                assert_eq!(plain.cost_report.graph_cost, traced.cost_report.graph_cost);
                assert_eq!(plain.work_counters, traced.work_counters);
                assert_eq!(plain.energy_observation, traced.energy_observation);
                assert_eq!(energy, traced_energy);
                assert_eq!(memory, traced_memory);
                assert_eq!(plain.actions, observed.actions);
                assert_eq!(plain.priority_bid, observed.priority_bid);
                assert_eq!(plain.cost_report.vm_cost, observed.cost_report.vm_cost);
                assert_eq!(
                    plain.cost_report.graph_cost,
                    observed.cost_report.graph_cost
                );
                assert_eq!(plain.work_counters, observed.work_counters);
                assert_eq!(plain.energy_observation, observed.energy_observation);
                assert_eq!(energy, observed_energy);
                assert_eq!(memory, observed_memory);
                assert_eq!(state.node_state, observed_state.node_state);
                assert_eq!(state.node_outputs, observed_state.node_outputs);
                assert_eq!(state.tick_start_state, observed_state.tick_start_state);
                assert_eq!(state.tick_start_outputs, observed_state.tick_start_outputs);
                assert_eq!(state.plasticity_weights, observed_state.plasticity_weights);
                assert_eq!(state.eligibility_traces, observed_state.eligibility_traces);
                assert_eq!(
                    state.tick_start_eligibility_traces,
                    observed_state.tick_start_eligibility_traces
                );
                // T11.F17: the dispatch record is mode-independent, and a
                // second tick at a later age refreshes the same entries.
                let executed = state.dispatch_record.executed_indices(tick, 1);
                assert_eq!(
                    executed.first(),
                    Some(&0),
                    "the entry node dispatches every tick at every energy level"
                );
                assert_eq!(
                    executed,
                    observed_state.dispatch_record.executed_indices(tick, 1)
                );
                assert_eq!(
                    executed,
                    traced_state.dispatch_record.executed_indices(tick, 1)
                );
                assert_eq!(
                    executed.len(),
                    (plain.work_counters.mesh_hops as usize).min(genome.nodes.len()),
                    "one record entry per dispatched node"
                );
                assert_eq!(state.scratch_prev, observed_state.scratch_prev);
                assert_eq!(state.scratch_curr, observed_state.scratch_curr);
                assert_eq!(state.scratch_backup, observed_state.scratch_backup);
                assert_eq!(state.scratch_w_inputs, observed_state.scratch_w_inputs);
                assert_eq!(state.node_state, traced_state.node_state);
                assert_eq!(state.node_outputs, traced_state.node_outputs);
                assert_eq!(state.tick_start_state, traced_state.tick_start_state);
                assert_eq!(state.tick_start_outputs, traced_state.tick_start_outputs);
                assert_eq!(state.plasticity_weights, traced_state.plasticity_weights);
                assert_eq!(state.eligibility_traces, traced_state.eligibility_traces);
                assert_eq!(
                    state.tick_start_eligibility_traces,
                    traced_state.tick_start_eligibility_traces
                );
                assert_eq!(state.scratch_prev, traced_state.scratch_prev);
                assert_eq!(state.scratch_curr, traced_state.scratch_curr);
                assert_eq!(state.scratch_backup, traced_state.scratch_backup);
                assert_eq!(state.scratch_w_inputs, traced_state.scratch_w_inputs);
            }
        }
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn default_config() -> RuntimeConfig {
        RuntimeConfig::default()
    }

    fn empty_sensor_snapshot() -> SensorSnapshot {
        SensorSnapshot {
            local: StaticInputs {
                food_here: 0.0,
                neighbor_food: [0.0; 8],
                neighbor_barrier: [0.0; 8],
                neighbor_occupied: [0.0; 8],
                generation: 0.0,
                age_ticks: 0.0,
            },
            typed_local_food: TypedFoodLocalSnapshot::zeroed(1),
            perception: PerceptionSnapshot::zeroed(1),
        }
    }

    /// Build a minimal VM node that pushes an action and executes the queue.
    /// The node has `register_count=1` so the VM will run.
    fn vm_emit_node(node_id: NodeId, action_type: u8, targets: Vec<NodeId>) -> NodeGenome {
        NodeGenome {
            node_id,
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![
                    VmInstruction::PushAction { action_type },
                    VmInstruction::ExecuteActionQueue,
                ],
            }),
            targets: wrap_targets(targets),
        }
    }

    /// Build a VM node that halts (no action) and routes with a given constant
    /// `route_value`.
    fn vm_halt_with_route(node_id: NodeId, route_value: f32, targets: Vec<NodeId>) -> NodeGenome {
        NodeGenome {
            node_id,
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![route_value],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::WriteRouteGate { slot: 0, src: 0 },
                    VmInstruction::Halt,
                ],
            }),
            targets: wrap_targets(targets),
        }
    }

    // ── Test 1: missing_entry_node_returns_noop ───────────────────────────────

    /// A genome with no nodes — entry_node_id is not in the node set.
    #[test]
    fn missing_entry_node_returns_noop() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(99),
            nodes: vec![],
        };
        let ss = empty_sensor_snapshot();
        let mut energy = 100.0f32;
        let mut shared_mem = [0.0f32; 16];
        let prev_shared_mem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();
        let config = default_config();

        let output = execute_creature_mesh(
            &genome,
            &ss,
            &mut energy,
            &mut shared_mem,
            &prev_shared_mem,
            &mut gr,
            &config,
        );
        assert_eq!(output.actions, vec![WorldAction::NoOp]);
    }

    // ── Test 2: max_hops_exceeded_returns_noop ────────────────────────────────

    /// A VM node that halts (no action emitted) and routes to itself.
    /// With max_mesh_hops=3, after 3 hops the executor must return NoOp.
    #[test]
    fn f15_self_loop_dispatches_once() {
        let id0 = NodeId::new(0);
        // Node routes to itself (self-loop); Halt emits no action.
        let node = NodeGenome {
            node_id: id0,
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![VmInstruction::Halt],
            }),
            targets: wrap_targets(vec![id0]), // self-loop
        };
        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![node],
        };
        let ss = empty_sensor_snapshot();
        let mut energy = 1000.0f32;
        let mut shared_mem = [0.0f32; 16];
        let prev_shared_mem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();
        let config = RuntimeConfig {
            max_mesh_hops: 3,
            ..RuntimeConfig::default()
        };

        let output = execute_creature_mesh(
            &genome,
            &ss,
            &mut energy,
            &mut shared_mem,
            &prev_shared_mem,
            &mut gr,
            &config,
        );
        assert_eq!(output.actions, vec![WorldAction::NoOp]);
        assert_eq!(output.work_counters.mesh_hops, 1);
    }

    // ── Test 3: empty_targets_returns_noop ───────────────────────────────────

    /// A VM node that halts with no targets — chain terminates with NoOp.
    #[test]
    fn empty_targets_returns_noop() {
        let id0 = NodeId::new(0);
        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![NodeGenome {
                node_id: id0,
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![], // no targets
            }],
        };
        let ss = empty_sensor_snapshot();
        let mut energy = 100.0f32;
        let mut shared_mem = [0.0f32; 16];
        let prev_shared_mem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();
        let config = default_config();

        let output = execute_creature_mesh(
            &genome,
            &ss,
            &mut energy,
            &mut shared_mem,
            &prev_shared_mem,
            &mut gr,
            &config,
        );
        assert_eq!(output.actions, vec![WorldAction::NoOp]);
    }

    // ── Test 4: energy_exhaustion_returns_noop ────────────────────────────────

    /// Energy is too low to run even one opcode — executor returns NoOp.
    #[test]
    fn energy_exhaustion_returns_noop() {
        let id0 = NodeId::new(0);
        // Noop costs 0.05; with energy=0.01 the first opcode exhausts energy.
        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![NodeGenome {
                node_id: id0,
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Noop, VmInstruction::Halt],
                }),
                targets: vec![],
            }],
        };
        let ss = empty_sensor_snapshot();
        let mut energy = 0.01f32; // way below the Noop cost of 0.05
        let mut shared_mem = [0.0f32; 16];
        let prev_shared_mem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();
        let config = default_config();

        let output = execute_creature_mesh(
            &genome,
            &ss,
            &mut energy,
            &mut shared_mem,
            &prev_shared_mem,
            &mut gr,
            &config,
        );
        assert_eq!(output.actions, vec![WorldAction::NoOp]);
    }

    // ── Test 5: vm_node_emits_eat_action ─────────────────────────────────────

    /// A VM node that pushes action_type 1 and executes queue → WorldAction::Eat { type_idx: crate::config::OrdinaryFoodTypeId::default() }.
    #[test]
    fn vm_node_emits_eat_action() {
        let id0 = NodeId::new(0);
        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![vm_emit_node(id0, 1, vec![])],
        };
        let ss = empty_sensor_snapshot();
        let mut energy = 100.0f32;
        let mut shared_mem = [0.0f32; 16];
        let prev_shared_mem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();
        let config = default_config();

        let output = execute_creature_mesh(
            &genome,
            &ss,
            &mut energy,
            &mut shared_mem,
            &prev_shared_mem,
            &mut gr,
            &config,
        );
        assert_eq!(
            output.actions,
            vec![WorldAction::Eat {
                type_idx: crate::config::OrdinaryFoodTypeId::default()
            }]
        );
    }

    // ── Test 7: single_slot_gate_routes_to_target ──────────────────────────────

    /// VM writes 3.7 to gate slot 0; single-slot targets all on slot 0,
    /// first target wins (all same effective score, tie-break by position).
    #[test]
    fn single_slot_gate_routes_to_target() {
        let id0 = NodeId::new(0);
        let id_a = NodeId::new(1);
        let id_b = NodeId::new(2);
        let id_c = NodeId::new(3);

        // Entry node: writes 3.7 to gate slot 0. All 3 targets share slot 0,
        // so all have the same effective score → first target (id_a) wins tie-break.
        let entry = vm_halt_with_route(id0, 3.7, vec![id_a, id_b, id_c]);

        // targets[0] (id_a): emits Eat
        // targets[1] and [2]: emit NoOp (fallback, should not be chosen)
        let node_a = vm_emit_node(id_a, 1, vec![]); // Eat
        let node_b = vm_emit_node(id_b, 0, vec![]); // NoOp
        let node_c = vm_emit_node(id_c, 0, vec![]); // NoOp

        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![entry, node_a, node_b, node_c],
        };
        let ss = empty_sensor_snapshot();
        let mut energy = 1000.0f32;
        let mut shared_mem = [0.0f32; 16];
        let prev_shared_mem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();
        let config = default_config();

        let output = execute_creature_mesh(
            &genome,
            &ss,
            &mut energy,
            &mut shared_mem,
            &prev_shared_mem,
            &mut gr,
            &config,
        );
        assert_eq!(
            output.actions,
            vec![WorldAction::Eat {
                type_idx: crate::config::OrdinaryFoodTypeId::default()
            }],
            "route=3.7 should select targets[0]"
        );
    }

    // ── Test 8: negative_gate_score_loses_to_zero ───────────────────────────

    /// VM writes -1.0 to gate slot 0; targets[0] on slot 0 (effective -1.0),
    /// targets[1] on slot 1 (effective 0.0). Slot 1 wins → routes to id_eat.
    #[test]
    fn negative_gate_score_loses_to_zero() {
        let id0 = NodeId::new(0);
        let id_noop = NodeId::new(1); // targets[0]
        let id_eat = NodeId::new(2); // targets[1]

        // Entry node: writes -1.0 to gate slot 0.
        // targets[0] (id_noop) on slot 0: effective = 0.0 + (-1.0) = -1.0
        // targets[1] (id_eat) on slot 1: effective = 0.0 + 0.0 = 0.0 (wins)
        let entry = vm_halt_with_route(id0, -1.0, vec![id_noop, id_eat]);
        let node_noop = vm_emit_node(id_noop, 0, vec![]); // NoOp
        let node_eat = vm_emit_node(id_eat, 1, vec![]); // Eat

        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![entry, node_noop, node_eat],
        };
        let ss = empty_sensor_snapshot();
        let mut energy = 1000.0f32;
        let mut shared_mem = [0.0f32; 16];
        let prev_shared_mem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();
        let config = default_config();

        let output = execute_creature_mesh(
            &genome,
            &ss,
            &mut energy,
            &mut shared_mem,
            &prev_shared_mem,
            &mut gr,
            &config,
        );
        assert_eq!(
            output.actions,
            vec![WorldAction::Eat {
                type_idx: crate::config::OrdinaryFoodTypeId::default()
            }],
            "route=-1.0 should wrap via rem_euclid and select targets[1]"
        );
    }

    // ── Priority bid tests ───────────────────────────────────────────────

    #[test]
    fn default_priority_bid_is_zero() {
        // A genome with no SetPriorityBid instruction should return priority_bid == 0.0.
        let id0 = NodeId::new(0);
        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![vm_emit_node(id0, 1, vec![])],
        };
        let ss = empty_sensor_snapshot();
        let mut energy = 100.0f32;
        let mut shared_mem = [0.0f32; 16];
        let prev_shared_mem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();
        let config = default_config();

        let output = execute_creature_mesh(
            &genome,
            &ss,
            &mut energy,
            &mut shared_mem,
            &prev_shared_mem,
            &mut gr,
            &config,
        );
        assert_eq!(output.priority_bid, 0.0);
    }

    #[test]
    fn priority_bid_propagates_to_mesh_output() {
        // A VM node loads 3.0 into r0, calls SetPriorityBid, then emits Eat.
        let id0 = NodeId::new(0);
        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![NodeGenome {
                node_id: id0,
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![3.0],
                    program: vec![
                        VmInstruction::LoadConst {
                            dst: 0,
                            const_idx: 0,
                        },
                        VmInstruction::SetPriorityBid { src: 0 },
                        VmInstruction::PushAction { action_type: 1 },
                        VmInstruction::ExecuteActionQueue,
                    ],
                }),
                targets: vec![],
            }],
        };
        let ss = empty_sensor_snapshot();
        let mut energy = 100.0f32;
        let mut shared_mem = [0.0f32; 16];
        let prev_shared_mem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();
        let config = default_config();

        let output = execute_creature_mesh(
            &genome,
            &ss,
            &mut energy,
            &mut shared_mem,
            &prev_shared_mem,
            &mut gr,
            &config,
        );
        assert_eq!(output.priority_bid, 3.0);
        assert_eq!(
            output.actions,
            vec![WorldAction::Eat {
                type_idx: crate::config::OrdinaryFoodTypeId::default()
            }]
        );
    }

    #[test]
    fn priority_bid_last_write_wins_across_hops() {
        // Two VM nodes both call SetPriorityBid. The second (downstream) value should win.
        let id0 = NodeId::new(0);
        let id1 = NodeId::new(1);

        // First node: bid 5.0, then halt and route to id1
        let node0 = NodeGenome {
            node_id: id0,
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![5.0],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::SetPriorityBid { src: 0 },
                    VmInstruction::Halt,
                ],
            }),
            targets: wrap_targets(vec![id1]),
        };

        // Second node: bid 2.0, then emit Eat
        let node1 = NodeGenome {
            node_id: id1,
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![2.0],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::SetPriorityBid { src: 0 },
                    VmInstruction::PushAction { action_type: 1 },
                    VmInstruction::ExecuteActionQueue,
                ],
            }),
            targets: vec![],
        };

        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![node0, node1],
        };
        let ss = empty_sensor_snapshot();
        let mut energy = 100.0f32;
        let mut shared_mem = [0.0f32; 16];
        let prev_shared_mem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();
        let config = default_config();

        let output = execute_creature_mesh(
            &genome,
            &ss,
            &mut energy,
            &mut shared_mem,
            &prev_shared_mem,
            &mut gr,
            &config,
        );
        assert_eq!(
            output.priority_bid, 2.0,
            "last-write-wins: second node's bid should be returned"
        );
    }

    // ── Work counter tests ───────────────────────────────────────────────

    #[test]
    fn single_hop_vm_node_reports_known_mesh_hops_and_vm_steps() {
        // A single VM node (PushAction, ExecuteActionQueue = 2 opcodes) with
        // no routing: exactly one mesh hop and two VM steps.
        let id0 = NodeId::new(0);
        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![vm_emit_node(id0, 1, vec![])],
        };
        let ss = empty_sensor_snapshot();
        let mut energy = 100.0f32;
        let mut shared_mem = [0.0f32; 16];
        let prev_shared_mem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();
        let config = default_config();

        let output = execute_creature_mesh(
            &genome,
            &ss,
            &mut energy,
            &mut shared_mem,
            &prev_shared_mem,
            &mut gr,
            &config,
        );
        assert_eq!(output.work_counters.mesh_hops, 1);
        assert_eq!(output.work_counters.vm_steps, 2);
        assert_eq!(output.work_counters.graph_relax_iters, 0);
        assert_eq!(output.work_counters.plasticity_updates, 0);
    }

    #[test]
    fn two_hop_chain_sums_vm_steps_across_both_nodes() {
        // node0: LoadConst, SetPriorityBid, Halt = 3 opcodes, routes to node1.
        // node1: LoadConst, SetPriorityBid, PushAction, ExecuteActionQueue = 4 opcodes.
        // Two mesh hops (one dispatch per node), seven VM steps total.
        let id0 = NodeId::new(0);
        let id1 = NodeId::new(1);

        let node0 = NodeGenome {
            node_id: id0,
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![5.0],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::SetPriorityBid { src: 0 },
                    VmInstruction::Halt,
                ],
            }),
            targets: wrap_targets(vec![id1]),
        };

        let node1 = NodeGenome {
            node_id: id1,
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![2.0],
                program: vec![
                    VmInstruction::LoadConst {
                        dst: 0,
                        const_idx: 0,
                    },
                    VmInstruction::SetPriorityBid { src: 0 },
                    VmInstruction::PushAction { action_type: 1 },
                    VmInstruction::ExecuteActionQueue,
                ],
            }),
            targets: vec![],
        };

        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![node0, node1],
        };
        let ss = empty_sensor_snapshot();
        let mut energy = 100.0f32;
        let mut shared_mem = [0.0f32; 16];
        let prev_shared_mem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();
        let config = default_config();

        let output = execute_creature_mesh(
            &genome,
            &ss,
            &mut energy,
            &mut shared_mem,
            &prev_shared_mem,
            &mut gr,
            &config,
        );
        assert_eq!(output.work_counters.mesh_hops, 2);
        assert_eq!(output.work_counters.vm_steps, 7);
    }

    #[test]
    fn missing_entry_node_reports_zero_work() {
        // A genome with no reachable entry node performs no mesh, VM, or
        // graph work at all.
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(99),
            nodes: vec![],
        };
        let ss = empty_sensor_snapshot();
        let mut energy = 100.0f32;
        let mut shared_mem = [0.0f32; 16];
        let prev_shared_mem = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();
        let config = default_config();

        let output = execute_creature_mesh(
            &genome,
            &ss,
            &mut energy,
            &mut shared_mem,
            &prev_shared_mem,
            &mut gr,
            &config,
        );
        assert_eq!(
            output.work_counters,
            crate::runtime::types::WorkCounters::default()
        );
    }
}
