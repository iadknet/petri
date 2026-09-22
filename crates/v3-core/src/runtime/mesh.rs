//! Mesh chain executor — evaluates a creature's genome mesh each tick.
//!
//! The mesh executor walks a chain of [`NodeGenome`] nodes starting from
//! `genome.entry_node_id`, dispatching each node to its backend (VM or Graph),
//! and routing to the next node via the returned internal routing decision until
//! a terminal condition or soft-default termination condition fires.

use crate::config::RuntimeConfig;
use crate::contracts::{NodeId, WorldAction};
use crate::creature::genome::vote::VoteVector;
use crate::creature::genome::{BackendDef, CreatureGenome, NodeGenome};
use crate::creature::state::GraphRuntimeState;
use crate::runtime::cgp::execute_graph_node;
use crate::runtime::routing::resolve_gated_route;
use crate::runtime::trace::domain::TerminationReason;
use crate::runtime::types::{ComputeCostReport, MeshOutput, MeshSideOutputs, OUTPUT_SLOT_COUNT};
use crate::runtime::vm::execute_vm_node;
use crate::sensors::perception::SensorSnapshot;
use crate::simulation::energy_accounting::{applied_debit, observe_energy_change, DeathCause};

/// Energy charged for the `k`-th mesh hop of a world tick (`k` from 1): zero
/// while the tick stays within `allowance`, then rising linearly with the hop
/// index. Over `n` hops the ramp totals `cost * m * (m + 1) / 2` with
/// `m = max(0, n - allowance)`.
///
/// The index never resets within a tick and starts at 1 at every tick, which is
/// what separates this charge from the VM's per-dispatch step ramp (T03.F10).
#[inline]
#[must_use]
fn hop_charge(k: u32, allowance: u32, cost: f32) -> f32 {
    cost * k.saturating_sub(allowance) as f32
}

/// Execute the creature's mesh chain within the current tick, returning a [`MeshOutput`]
/// containing the queued actions, a [`ComputeCostReport`], and the priority bid.
///
/// Before the first mesh execution of each new world tick, the caller must call
/// `graph_runtime.begin_tick(&genome.nodes, age)`, which decays eligibility
/// once per world tick. A node may be dispatched any number of times within
/// the tick on its live state (T19.F02); the per-pass cap `max_mesh_hops`
/// bounds the dispatches and the per-tick hop ramp prices them.
///
/// The function walks the genome's node chain starting at `entry_node_id`,
/// dispatching each node to its VM or Graph backend, routing to subsequent
/// nodes by resolving each internal node result's route-gate scores, and terminating
/// when a terminal instruction is reached, the cap is hit, or a soft-default
/// condition fires. The recorded priority bid is settled once at the end.
///
/// Soft-default and cap terminations keep the accumulated queue (or return
/// `vec![WorldAction::NoOp]` when it is empty). Energy exhaustion, including
/// an all-in bid, discards the queue and returns `vec![WorldAction::NoOp]`.
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
        _vote_contribution: VoteVector,
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

/// One applied route: the winning target position and the node it named.
pub(crate) type AppliedRoute = (usize, NodeId);

/// Compact observations of actual dispatches and applied routing, without backend traces.
#[derive(Debug, Clone)]
pub(crate) struct MeshObservation {
    pub hops: Vec<(NodeId, Option<AppliedRoute>)>,
    pub termination_reason: TerminationReason,
}

#[derive(Default)]
pub(crate) struct ObservedMeshExecution {
    hops: Vec<(NodeId, Option<AppliedRoute>)>,
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
        _vote_contribution: VoteVector,
        _backend_trace: (),
    ) {
        let route = if result.terminal || result.energy_exhausted {
            None
        } else {
            route_result
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
    let mut upstream_slots = [0.0f32; OUTPUT_SLOT_COUNT];
    let mut hops: usize = 0;
    let max_hops = config.max_mesh_hops.max(1) as usize;
    let start_energy = *energy;
    let mut report = ComputeCostReport::default();
    let mut side_outputs = MeshSideOutputs::new(config.max_actions_per_turn);

    // Soft default: entry_node_id missing from node set → NoOp immediately.
    let termination_reason = match find_node_index(&genome.nodes, genome.entry_node_id) {
        None => TerminationReason::MissingNode,
        Some(mut current_idx) => loop {
            // Per-pass cap (T19.F02): a pass dispatches at most `max_hops`
            // nodes; reaching the cap ends the evaluation with the queue kept.
            if hops >= max_hops {
                side_outputs.work_counters.pass_cap_hits += 1;
                break TerminationReason::MaxHopsReached;
            }

            let node = &genome.nodes[current_idx];

            // One mesh hop is one node dispatch, counted regardless of outcome.
            side_outputs.work_counters.mesh_hops += 1;
            // Same event, recorded for the offspring's mutation targeting (T11.F17).
            // Written from the shared loop, so every execution mode agrees.
            graph_runtime.dispatch_record.record_dispatch(current_idx);

            // Per-tick hop ramp (T19.F01): sustained neural activity costs
            // metabolism. Charged before the dispatch, so an unaffordable hop is
            // counted and recorded but never executed.
            let ramp_charge = hop_charge(
                side_outputs.work_counters.mesh_hops,
                config.hop_ramp_allowance,
                config.hop_ramp_cost,
            );
            if ramp_charge > 0.0 {
                let before = *energy;
                *energy -= ramp_charge;
                side_outputs.energy_observation.mesh_ramp += applied_debit(before, *energy);
                report.mesh_ramp_cost += (before - *energy).max(0.0);
                observe_energy_change(
                    &mut side_outputs.energy_observation.pending_cause,
                    f64::from(before),
                    f64::from(*energy),
                    DeathCause::MeshRamp,
                );
                if *energy <= 0.0 {
                    break TerminationReason::EnergyExhausted;
                }
            }

            let energy_consumed = (start_energy - *energy).max(0.0);

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

            // Take the dispatch's vote contribution as this node's latest
            // (T19.F03). A dispatch that ended exhausted staged nothing, so
            // the node keeps whatever it last committed and the hop reports
            // zeros.
            let vote_contribution = side_outputs.commit_vote_contribution(current_idx);

            // Attribute energy delta to the correct backend.
            let node_cost = (node_energy_before - *energy).max(0.0);
            match &node.backend_def {
                BackendDef::Vm(_) => report.vm_cost += node_cost,
                BackendDef::Graph(_) => report.graph_cost += node_cost,
            }

            // Every existing target is eligible (T19.F02): a node may route to
            // itself or to any node this tick already dispatched.
            let route_result = if M::RECORDS_HOPS || (!result.energy_exhausted && !result.terminal)
            {
                resolve_gated_route(&node.targets, &result.route_gates)
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
                vote_contribution,
                backend_trace,
            );

            // Check exhaustion first: NodeResult::exhausted() discards the action queue.
            if result.energy_exhausted {
                break TerminationReason::EnergyExhausted;
            }

            if result.terminal {
                break TerminationReason::ActionEmitted;
            }

            // Routing via per-target gate scoring.
            match route_result.map(|(_, id)| find_node_index(&genome.nodes, id)) {
                Some(Some(next_idx)) => {
                    upstream_slots = result.output_slots;
                    current_idx = next_idx;
                    hops += 1;
                }
                Some(None) => break TerminationReason::MissingNode,
                None => break TerminationReason::NoTargets,
            }
        },
    };

    // Single bid settlement (T19.F02): the recorded bid is paid once, on every
    // exit that is not already an exhaustion; a bid the creature cannot cover
    // is an all-in that ends the evaluation exhausted. A creature that
    // exhausts on compute pays no bid.
    let (termination_reason, priority_bid) =
        if termination_reason == TerminationReason::EnergyExhausted {
            (termination_reason, 0.0)
        } else {
            match settle_priority_bid(side_outputs.priority_bid, energy, &mut side_outputs) {
                Some(paid) => (termination_reason, paid),
                None => (TerminationReason::EnergyExhausted, 0.0),
            }
        };
    // Exhaustion discards the queue; every other exit keeps it (a missing
    // entry node has an empty queue, so it reads `NoOp` as before).
    let actions = if termination_reason == TerminationReason::EnergyExhausted {
        vec![WorldAction::NoOp]
    } else {
        side_outputs.action_queue.into_actions_or_noop()
    };

    let output = MeshOutput {
        actions,
        cost_report: report,
        priority_bid,
        work_counters: side_outputs.work_counters,
        votes: side_outputs.votes,
        commit_counts: side_outputs.commit_counts,
        energy_observation: side_outputs.energy_observation,
        termination_reason,
    };
    mode.finish(output)
}

/// Pay the recorded `bid` against `energy` exactly once: `paid = min(bid,
/// energy)`. A bid at or above the creature's energy is an all-in that pins
/// energy to `0.0` and returns `None`; otherwise the paid amount is returned.
/// A zero bid pays nothing and never exhausts.
fn settle_priority_bid(
    bid: f32,
    energy: &mut f32,
    side_outputs: &mut MeshSideOutputs,
) -> Option<f32> {
    if bid <= 0.0 {
        return Some(0.0);
    }
    let before = *energy;
    let all_in = bid >= before;
    *energy = if all_in { 0.0 } else { before - bid };
    side_outputs.energy_observation.priority_bid += applied_debit(before, *energy);
    observe_energy_change(
        &mut side_outputs.energy_observation.pending_cause,
        f64::from(before),
        f64::from(*energy),
        DeathCause::PriorityBid,
    );
    (!all_in).then_some(bid)
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
        // A self-loop is legal (T19.F02): both capped dispatches applied
        // the self route, and the cap is the real termination.
        let (_, observed) = observe(&genome, 100.0);
        assert_eq!(
            observed.hops,
            vec![(id, Some((0, id))), (id, Some((0, id)))]
        );
        assert!(matches!(
            observed.termination_reason,
            TerminationReason::MaxHopsReached
        ));
        genome.nodes[0] = vm_halt_with_route(id, 1.0, vec![]);
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
                assert_eq!(state.plasticity_weights, observed_state.plasticity_weights);
                assert_eq!(state.eligibility_traces, observed_state.eligibility_traces);
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
                assert_eq!(state.plasticity_weights, traced_state.plasticity_weights);
                assert_eq!(state.eligibility_traces, traced_state.eligibility_traces);
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
                max_energy: 200.0,
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

    // ── Cycle fixtures (T19.F02) ─────────────────────────────────────────────

    fn run(genome: &CreatureGenome, config: &RuntimeConfig, energy: &mut f32) -> MeshOutput {
        let mut memory = [0.0f32; 16];
        run_with_memory(genome, config, energy, &mut memory)
    }

    fn run_with_memory(
        genome: &CreatureGenome,
        config: &RuntimeConfig,
        energy: &mut f32,
        memory: &mut [f32; 16],
    ) -> MeshOutput {
        execute_creature_mesh(
            genome,
            &empty_sensor_snapshot(),
            energy,
            memory,
            &[0.0; 16],
            &mut GraphRuntimeState::new(),
            config,
        )
    }

    /// W11: a Halt-only self-loop is legal and runs to the per-pass cap, 64
    /// hops at the defaults, paying the T19.F01 ramp
    /// (`1e-4 * 32 * 33 / 2 = 0.0528`) and counting one capped pass.
    #[test]
    fn w11_halt_only_self_loop_runs_to_the_pass_cap() {
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
                targets: wrap_targets(vec![id0]),
            }],
        };
        let config = default_config();
        assert_eq!(config.max_mesh_hops, 64);
        let mut energy = 10.0f32;
        let output = run(&genome, &config, &mut energy);
        assert_eq!(output.actions, vec![WorldAction::NoOp]);
        assert_eq!(output.termination_reason, TerminationReason::MaxHopsReached);
        assert_eq!(output.work_counters.mesh_hops, 64);
        assert_eq!(output.work_counters.pass_cap_hits, 1);
        assert!(
            (output.cost_report.mesh_ramp_cost - 0.0528).abs() < 1e-5,
            "ramp {}",
            output.cost_report.mesh_ramp_cost
        );
        assert!(energy > 0.0, "one capped pass is survivable");
    }

    /// W12: a self-looping non-terminal push node fills the queue to
    /// `max_actions_per_turn` and the cap keeps it there; the capped pass
    /// still returns the queue.
    #[test]
    fn w12_self_looping_push_node_fills_the_queue_and_keeps_it_at_the_cap() {
        let id0 = NodeId::new(0);
        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![NodeGenome {
                node_id: id0,
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![
                        VmInstruction::PushAction { action_type: 1 },
                        VmInstruction::Halt,
                    ],
                }),
                targets: wrap_targets(vec![id0]),
            }],
        };
        let config = default_config();
        let output = run(&genome, &config, &mut 10.0);
        assert_eq!(output.termination_reason, TerminationReason::MaxHopsReached);
        assert_eq!(output.work_counters.mesh_hops, 64);
        assert_eq!(output.work_counters.pass_cap_hits, 1);
        assert_eq!(output.actions.len(), config.max_actions_per_turn);
        assert!(output
            .actions
            .iter()
            .all(|action| matches!(action, WorldAction::Eat { .. })));
    }

    /// The W14 entry node: count visits in shared slot 0, push one action per
    /// visit, and route back to itself while the count is below `limit`
    /// (gate 1.0 on slot 0 against a constant 0.5 on the exit's slot 1).
    fn counted_cycle_node(limit: f32, exit: NodeId) -> NodeGenome {
        let id0 = NodeId::new(0);
        NodeGenome {
            node_id: id0,
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 3,
                constants: vec![1.0, limit, 0.5],
                program: vec![
                    VmInstruction::LoadSlotImm {
                        dst: 0,
                        slot_idx: 0,
                    },
                    VmInstruction::LoadConst {
                        dst: 1,
                        const_idx: 0,
                    },
                    VmInstruction::Add { dst: 0, a: 0, b: 1 },
                    VmInstruction::StoreSlotImm {
                        slot_idx: 0,
                        src: 0,
                    },
                    VmInstruction::PushAction { action_type: 1 },
                    VmInstruction::LoadConst {
                        dst: 1,
                        const_idx: 1,
                    },
                    VmInstruction::CmpLt { dst: 2, a: 0, b: 1 },
                    VmInstruction::WriteRouteGate { slot: 0, src: 2 },
                    VmInstruction::LoadConst {
                        dst: 2,
                        const_idx: 2,
                    },
                    VmInstruction::WriteRouteGate { slot: 1, src: 2 },
                    VmInstruction::Halt,
                ],
            }),
            targets: wrap_targets(vec![id0, exit]),
        }
    }

    /// W14: a two-target cycle gated on a memory counter exits after the
    /// counted visits (three dispatches of the entry, then the terminal exit)
    /// and keeps the actions every visit pushed.
    #[test]
    fn w14_counted_cycle_exits_on_its_gate_and_keeps_its_actions() {
        let exit = NodeId::new(1);
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![
                counted_cycle_node(3.0, exit),
                NodeGenome {
                    node_id: exit,
                    input_refs: vec![],
                    backend_def: BackendDef::Vm(VmBackendDef {
                        register_count: 1,
                        constants: vec![],
                        program: vec![VmInstruction::ExecuteActionQueue],
                    }),
                    targets: vec![],
                },
            ],
        };
        let config = default_config();
        let mut memory = [0.0f32; 16];
        let output = run_with_memory(&genome, &config, &mut 10.0, &mut memory);
        assert_eq!(output.termination_reason, TerminationReason::ActionEmitted);
        assert_eq!(output.work_counters.mesh_hops, 4);
        assert_eq!(output.work_counters.pass_cap_hits, 0);
        assert_eq!(output.actions.len(), 3);
        assert_eq!(
            memory[0], 3.0,
            "each visit read the previous visit's commit"
        );
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
        assert!(
            (energy - 97.0).abs() < 1e-3,
            "the bid is settled once, after the opcode costs: {energy}"
        );
        assert!((output.energy_observation.priority_bid - 3.0).abs() < 1e-3);
    }

    /// A VM node that bids `bid` (from constant 0) and then runs `tail`.
    fn bidding_node(
        node_id: NodeId,
        bid: f32,
        tail: Vec<VmInstruction>,
        targets: Vec<NodeId>,
    ) -> NodeGenome {
        let mut program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            VmInstruction::SetPriorityBid { src: 0 },
        ];
        program.extend(tail);
        NodeGenome {
            node_id,
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![bid],
                program,
            }),
            targets: wrap_targets(targets),
        }
    }

    /// A bid at or above the creature's energy is an all-in (T19.F02): energy
    /// lands on exactly `0.0`, the evaluation ends `EnergyExhausted` with
    /// `NoOp`, and the pending death cause is the bid.
    #[test]
    fn priority_bid_all_in_exhausts_at_settlement() {
        let id0 = NodeId::new(0);
        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![bidding_node(
                id0,
                5.0,
                vec![
                    VmInstruction::PushAction { action_type: 1 },
                    VmInstruction::ExecuteActionQueue,
                ],
                vec![],
            )],
        };
        let mut energy = 2.0f32;
        let output = run(&genome, &default_config(), &mut energy);
        assert_eq!(energy, 0.0);
        assert_eq!(output.actions, vec![WorldAction::NoOp]);
        assert_eq!(
            output.termination_reason,
            TerminationReason::EnergyExhausted
        );
        assert_eq!(output.priority_bid, 0.0);
        assert_eq!(
            output.energy_observation.pending_cause,
            Some(DeathCause::PriorityBid)
        );
        assert!((output.energy_observation.priority_bid - 2.0).abs() < 1e-5);
    }

    /// A creature that exhausts on compute pays no bid: the recorded bid is
    /// dropped with the queue and energy is untouched by it.
    #[test]
    fn priority_bid_is_unpaid_when_compute_exhausts_the_creature() {
        let id0 = NodeId::new(0);
        let genome = CreatureGenome {
            entry_node_id: id0,
            nodes: vec![bidding_node(
                id0,
                0.5,
                vec![VmInstruction::PushAction { action_type: 1 }; 10_000],
                vec![],
            )],
        };
        let config = RuntimeConfig {
            vm: crate::config::VmRuntimeConfig {
                opcode_cost_multiplier: 1.0,
                ..RuntimeConfig::default().vm
            },
            ..default_config()
        };
        let mut energy = 1.0f32;
        let output = run(&genome, &config, &mut energy);
        assert_eq!(
            output.termination_reason,
            TerminationReason::EnergyExhausted
        );
        assert_eq!(output.priority_bid, 0.0);
        assert_eq!(output.energy_observation.priority_bid, 0.0);
        assert_eq!(
            output.energy_observation.pending_cause,
            Some(DeathCause::VmCompute)
        );
    }

    /// Across a revisit the last write wins and the bid is settled once:
    /// three visits bidding 1, 2, and 3 pay 3, not 6.
    #[test]
    fn priority_bid_last_write_wins_across_a_revisit_and_settles_once() {
        let exit = NodeId::new(1);
        let mut entry = counted_cycle_node(3.0, exit);
        if let BackendDef::Vm(def) = &mut entry.backend_def {
            // After the counter store, bid the visit count held in r0.
            def.program
                .insert(4, VmInstruction::SetPriorityBid { src: 0 });
        }
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![
                entry,
                NodeGenome {
                    node_id: exit,
                    input_refs: vec![],
                    backend_def: BackendDef::Vm(VmBackendDef {
                        register_count: 1,
                        constants: vec![],
                        program: vec![VmInstruction::ExecuteActionQueue],
                    }),
                    targets: vec![],
                },
            ],
        };
        let mut energy = 10.0f32;
        let output = run(&genome, &default_config(), &mut energy);
        assert_eq!(output.termination_reason, TerminationReason::ActionEmitted);
        assert_eq!(output.priority_bid, 3.0);
        assert!((energy - 7.0).abs() < 1e-3, "energy {energy}");
    }

    /// Non-finite and negative reads record no bid.
    #[test]
    fn priority_bid_ignores_negative_and_non_finite_reads() {
        for raw in [-1.0, f32::NAN, f32::NEG_INFINITY] {
            let id0 = NodeId::new(0);
            let genome = CreatureGenome {
                entry_node_id: id0,
                nodes: vec![bidding_node(
                    id0,
                    raw,
                    vec![
                        VmInstruction::PushAction { action_type: 1 },
                        VmInstruction::ExecuteActionQueue,
                    ],
                    vec![],
                )],
            };
            let mut energy = 10.0f32;
            let output = run(&genome, &default_config(), &mut energy);
            assert_eq!(output.priority_bid, 0.0);
            assert_eq!(output.termination_reason, TerminationReason::ActionEmitted);
            assert!((energy - 10.0).abs() < 1e-3);
        }
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

    // ── T19.F01: per-tick hop ramp ───────────────────────────────────────────

    /// A forward chain of halting nodes: node `i` routes to node `i + 1`, and
    /// the last node has no target, so the evaluation runs exactly `len` hops.
    fn halting_chain(len: usize) -> CreatureGenome {
        let nodes = (0..len)
            .map(|i| {
                let targets = if i + 1 < len {
                    vec![NodeId::new(i as u32 + 1)]
                } else {
                    vec![]
                };
                vm_halt_with_route(NodeId::new(i as u32), 1.0, targets)
            })
            .collect();
        CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes,
        }
    }

    fn run_chain(genome: &CreatureGenome, config: &RuntimeConfig, energy: &mut f32) -> MeshOutput {
        execute_creature_mesh(
            genome,
            &empty_sensor_snapshot(),
            energy,
            &mut [0.0; 16],
            &[0.0; 16],
            &mut GraphRuntimeState::new(),
            config,
        )
    }

    #[test]
    fn pass_cap_hits_counts_only_capped_passes() {
        let config = RuntimeConfig {
            max_mesh_hops: 2,
            ..default_config()
        };
        let capped = run_chain(&halting_chain(3), &config, &mut 100.0);
        assert_eq!(capped.termination_reason, TerminationReason::MaxHopsReached);
        assert_eq!(capped.work_counters.pass_cap_hits, 1);
        assert_eq!(capped.work_counters.mesh_hops, 2);
        let uncapped = run_chain(&halting_chain(2), &config, &mut 100.0);
        assert_eq!(uncapped.termination_reason, TerminationReason::NoTargets);
        assert_eq!(uncapped.work_counters.pass_cap_hits, 0);
    }

    proptest::proptest! {
        /// Hop `k` pays nothing inside the allowance and `cost * (k - allowance)`
        /// past it, with the saturation at the boundary derived independently.
        #[test]
        fn hop_charge_is_zero_inside_the_allowance_and_linear_past_it(
            k in 1u32..4096,
            allowance in 0u32..4096,
            cost in 0.0f32..1.0,
        ) {
            let excess = (k.max(allowance) - allowance) as f32;
            proptest::prop_assert_eq!(hop_charge(k, allowance, cost), cost * excess);
            if k <= allowance {
                proptest::prop_assert_eq!(hop_charge(k, allowance, cost), 0.0);
            }
            proptest::prop_assert!(hop_charge(k + 1, allowance, cost) >= hop_charge(k, allowance, cost));
        }

        /// A tick of `n` hops owes the closed form `cost * m * (m + 1) / 2`
        /// with `m = max(0, n - allowance)`. Asserted at unit cost, where every
        /// term and the sum are exact in `f32`.
        #[test]
        fn hop_ramp_total_over_a_tick_is_the_closed_form(
            n in 0u32..2048,
            allowance in 0u32..2048,
        ) {
            let summed: f32 = (1..=n).map(|k| hop_charge(k, allowance, 1.0)).sum();
            let m = f64::from(n.saturating_sub(allowance));
            proptest::prop_assert_eq!(f64::from(summed), m * (m + 1.0) / 2.0);
        }
    }

    /// A chain no longer than the allowance pays nothing, and the charge starts
    /// on the first hop past it.
    #[test]
    fn hop_ramp_charges_only_past_the_allowance() {
        let genome = halting_chain(4);
        for (allowance, expected_hops_charged) in [(4u32, 0u32), (3, 1), (2, 3), (0, 10)] {
            let config = RuntimeConfig {
                hop_ramp_allowance: allowance,
                hop_ramp_cost: 0.25,
                ..default_config()
            };
            let mut energy = 100.0f32;
            let output = run_chain(&genome, &config, &mut energy);
            assert_eq!(output.work_counters.mesh_hops, 4, "allowance {allowance}");
            let expected = 0.25 * f64::from(expected_hops_charged);
            assert_eq!(
                output.energy_observation.mesh_ramp, expected,
                "allowance {allowance}"
            );
            assert_eq!(
                f64::from(output.cost_report.mesh_ramp_cost),
                expected,
                "allowance {allowance}"
            );
        }
    }

    /// The production defaults leave the founder cost-free: it runs two hops,
    /// far inside the 32-hop allowance.
    #[test]
    fn founder_pays_no_hop_ramp_at_the_production_defaults() {
        let genome = crate::creature::founder::v3alpha1_founder_genome();
        let config = default_config();
        assert_eq!(config.hop_ramp_allowance, 32);
        let mut energy = 100.0f32;
        let output = run_chain(&genome, &config, &mut energy);
        assert_eq!(
            output.work_counters.mesh_hops, 2,
            "the founder chain runs exactly two hops, far inside the allowance",
        );
        assert_eq!(output.energy_observation.mesh_ramp, 0.0);
        assert_eq!(output.cost_report.mesh_ramp_cost, 0.0);
    }

    /// A node reading `EnergyConsumedThisTick` sees the hop's own ramp charge:
    /// the debit lands before the dispatch, so the tick's consumption at hop 2
    /// includes hop 1's charge and hop 2's.
    ///
    /// Measured differentially against a zero-cost run of the same genome, so
    /// node 0's VM cost and the reading dispatch's own in-flight step costs
    /// (which the read also includes) cancel and only the ramp remains.
    #[test]
    fn a_node_reads_its_own_hop_ramp_charge_as_consumed_this_tick() {
        fn consumption_read(hop_ramp_cost: f32) -> f32 {
            let genome = CreatureGenome {
                entry_node_id: NodeId::new(0),
                nodes: vec![
                    vm_halt_with_route(NodeId::new(0), 1.0, vec![NodeId::new(1)]),
                    NodeGenome {
                        node_id: NodeId::new(1),
                        input_refs: vec![crate::contracts::InputReference::DynamicIntrospection(
                            crate::contracts::DynamicIntrospectionKey::EnergyConsumedThisTick,
                        )],
                        backend_def: BackendDef::Vm(VmBackendDef {
                            register_count: 1,
                            constants: vec![],
                            program: vec![
                                VmInstruction::ReadInput {
                                    dst: 0,
                                    ref_idx: 0,
                                    sub_idx: 0,
                                },
                                VmInstruction::StoreSlotImm {
                                    slot_idx: 0,
                                    src: 0,
                                },
                                VmInstruction::Halt,
                            ],
                        }),
                        targets: wrap_targets(vec![]),
                    },
                ],
            };
            let config = RuntimeConfig {
                hop_ramp_allowance: 0,
                hop_ramp_cost,
                ..default_config()
            };
            let mut energy = 100.0f32;
            let mut shared_mem = [0.0f32; 16];
            let output = execute_creature_mesh(
                &genome,
                &empty_sensor_snapshot(),
                &mut energy,
                &mut shared_mem,
                &[0.0; 16],
                &mut GraphRuntimeState::new(),
                &config,
            );
            assert_eq!(output.work_counters.mesh_hops, 2);
            // The read is a fraction of `max_energy`, which is 200.0 here.
            shared_mem[0] * 200.0
        }

        let cost = 1.0f32;
        let seen = consumption_read(cost) - consumption_read(0.0);
        // Hop 1 owes `cost`, hop 2 owes `2 * cost`, and hop 2's charge is
        // debited before the node that reads it runs.
        let expected = 3.0 * cost;
        assert!(
            (seen - expected).abs() < 1e-4,
            "the node read {seen} more consumed under the ramp, expected {expected}",
        );
    }

    /// An unaffordable ramp charge ends the tick before the node runs: the hop
    /// is counted, nothing is dispatched, and the creature acts `NoOp`.
    #[test]
    fn hop_ramp_exhaustion_ends_the_tick_as_noop_before_dispatch() {
        let genome = halting_chain(2);
        let config = RuntimeConfig {
            hop_ramp_allowance: 0,
            hop_ramp_cost: 10.0,
            ..default_config()
        };
        let mut energy = 5.0f32;
        let mut memory = [0.5f32; 16];
        let mut state = GraphRuntimeState::new();
        let output = execute_creature_mesh(
            &genome,
            &empty_sensor_snapshot(),
            &mut energy,
            &mut memory,
            &[0.0; 16],
            &mut state,
            &config,
        );
        assert_eq!(output.actions, vec![WorldAction::NoOp]);
        assert!(matches!(
            output.termination_reason,
            TerminationReason::EnergyExhausted
        ));
        assert_eq!(
            output.energy_observation.pending_cause,
            Some(crate::simulation::energy_accounting::DeathCause::MeshRamp)
        );
        assert_eq!(
            output.work_counters.mesh_hops, 1,
            "the hop is still counted"
        );
        assert_eq!(
            output.work_counters.vm_steps, 0,
            "the node never dispatched"
        );
        assert_eq!(memory, [0.5f32; 16], "no node ran, so memory is untouched");
        assert_eq!(energy, -5.0);
        assert_eq!(output.energy_observation.mesh_ramp, 10.0);
        assert_eq!(output.cost_report.mesh_ramp_cost, 10.0);
    }

    /// Production, observed, and traced execution charge the ramp identically,
    /// because the charge sits in the shared loop.
    #[test]
    fn hop_ramp_agrees_across_execution_modes() {
        let genome = halting_chain(4);
        let config = RuntimeConfig {
            hop_ramp_allowance: 1,
            hop_ramp_cost: 0.5,
            ..default_config()
        };
        for starting_energy in [0.4f32, 1.6, 100.0] {
            let mut plain_energy = starting_energy;
            let mut observed_energy = starting_energy;
            let mut traced_energy = starting_energy;
            let plain = run_chain(&genome, &config, &mut plain_energy);
            let (observed, _) = execute_creature_mesh_impl(
                &genome,
                &empty_sensor_snapshot(),
                &mut observed_energy,
                &mut [0.0; 16],
                &[0.0; 16],
                &mut GraphRuntimeState::new(),
                &config,
                ObservedMeshExecution::default(),
            );
            let (traced, _, _) = crate::runtime::traced_mesh::execute_creature_mesh_traced(
                &genome,
                &empty_sensor_snapshot(),
                &mut traced_energy,
                &mut [0.0; 16],
                &[0.0; 16],
                &mut GraphRuntimeState::new(),
                &config,
            );
            for other in [&observed, &traced] {
                assert_eq!(plain.actions, other.actions);
                assert_eq!(plain.work_counters, other.work_counters);
                assert_eq!(plain.energy_observation, other.energy_observation);
                assert_eq!(
                    plain.cost_report.mesh_ramp_cost,
                    other.cost_report.mesh_ramp_cost
                );
                assert_eq!(plain.termination_reason, other.termination_reason);
            }
            assert_eq!(plain_energy, observed_energy);
            assert_eq!(plain_energy, traced_energy);
            assert!(plain.cost_report.mesh_ramp_cost > 0.0);
        }
    }
}
