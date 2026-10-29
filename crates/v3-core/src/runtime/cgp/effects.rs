//! Post-convergence effect pass for CGP-style graph backend evaluation.

use crate::contracts::votes::{VoteVector, VOTE_SINK_COUNT};
use crate::contracts::OUTPUT_SLOT_COUNT;
use crate::contracts::{InputReference, SHARED_MEMORY_SLOTS};
use crate::genome::cgp::{CgpGraphBackendDef, EdgeSurface, GraphEdge, OutputSinkKind};
use crate::runtime::action_decode::ActionParams;
use crate::runtime::cgp::sources::resolve_source_post_convergence;
use crate::runtime::cgp::tracer::GraphTracer;
use crate::runtime::inputs::ResolveCtx;
use crate::runtime::plasticity::Learner;
use crate::runtime::routing::RouteGateMap;
use crate::runtime::types::{sanitize_f32, shared_memory_write_changed, NodeResult};

/// One sink's weighted inputs into `buf` and their sum: each edge's value
/// as the sink sum reads it, with shared memory as earlier sinks wrote it,
/// times its applied weight. On a learning graph (sink `sink`) the value is
/// a learner's pre sample and the sink's sanitized sum its post sample
/// (T25.F06).
#[inline]
#[allow(clippy::too_many_arguments)]
fn edges_wsum(
    sink: usize,
    edges: &[GraphEdge],
    weights: &[f32],
    compute_count: usize,
    curr_outputs: &[f32],
    input_refs: &[InputReference],
    resolve_ctx: &ResolveCtx<'_>,
    shared_memory: &[f32; SHARED_MEMORY_SLOTS],
    prev_shared_memory: &[f32; SHARED_MEMORY_SLOTS],
    mut learner: Option<&mut Learner<'_>>,
    buf: &mut Vec<f32>,
) -> f32 {
    buf.clear();
    if let Some(learner) = learner.as_deref_mut() {
        learner.begin_row(EdgeSurface::SinkInput(sink));
    }
    buf.extend(edges.iter().zip(weights).map(|(edge, &weight)| {
        let value = resolve_source_post_convergence(
            &edge.source(),
            compute_count,
            curr_outputs,
            input_refs,
            resolve_ctx,
            shared_memory,
            prev_shared_memory,
        );
        let weight = match learner.as_deref_mut() {
            Some(learner) => learner.weigh(value, weight),
            None => weight,
        };
        value * weight
    }));
    let wsum = buf.iter().copied().sum();
    if let Some(learner) = learner {
        learner.end_row(sanitize_f32(wsum));
    }
    wsum
}

/// Apply post-convergence effects from CGP graph evaluation: the sinks in
/// catalog order, each reported to `tracer`. Returns the visit's result and
/// its vote contribution, which the caller stages.
#[inline]
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_cgp_graph_effects<T: GraphTracer>(
    tracer: &mut T,
    def: &CgpGraphBackendDef,
    sink_weights: &[Box<[f32]>],
    curr_outputs: &[f32],
    input_refs: &[InputReference],
    resolve_ctx: &ResolveCtx<'_>,
    upstream_slots: &[f32; OUTPUT_SLOT_COUNT],
    mut learner: Option<&mut Learner<'_>>,
    buf: &mut Vec<f32>,
    shared_memory_writes_changed: &mut u32,
    action_params: &mut ActionParams,
    shared_memory: &mut [f32; SHARED_MEMORY_SLOTS],
    prev_shared_memory: &[f32; SHARED_MEMORY_SLOTS],
) -> (NodeResult, VoteVector) {
    let mut output_slots = *upstream_slots;
    let mut route_gates = RouteGateMap::default();
    let compute_count = def.compute_nodes().len();
    // This visit's vote contribution (T19.F03): the sanitized weighted sum of
    // each wired `ActionVote` sink, 0 for an unwired one. Staged by the
    // caller once the effects ran, which they only do when the visit commits.
    let mut contribution: VoteVector = [0.0; VOTE_SINK_COUNT];
    debug_assert_eq!(
        sink_weights.len(),
        def.output_sinks().len(),
        "the weight table has one row per sink"
    );
    for (sink_idx, (sink, weights)) in def.output_sinks().iter().zip(sink_weights).enumerate() {
        debug_assert!(
            weights.len() == sink.inputs.len()
                && weights
                    .iter()
                    .zip(sink.inputs.iter())
                    .all(|(weight, edge)| {
                        weight.to_bits() == def.edge_weight(edge, sink.kind, input_refs).to_bits()
                    }),
            "the weight table is the genome's: rebuild runtime state after editing the genome"
        );
        if sink.inputs.is_empty() {
            tracer.on_sink(false, &[], 0.0, 0.0);
            continue;
        }

        // Members are ordinary sink edges at their applied weights (T25.F03,
        // T25.F07), summed in the sink's I10 order.
        let wsum = edges_wsum(
            sink_idx,
            &sink.inputs,
            weights,
            compute_count,
            curr_outputs,
            input_refs,
            resolve_ctx,
            shared_memory,
            prev_shared_memory,
            learner.as_deref_mut(),
            buf,
        );
        let value = sanitize_f32(wsum);
        let applied_value = match sink.kind {
            OutputSinkKind::CustomOutput(s) => {
                output_slots[s as usize] = value;
                value
            }
            OutputSinkKind::RouterGate(slot) => {
                route_gates.scores[slot as usize] = value;
                value
            }
            OutputSinkKind::WriteSlot(s) => {
                *shared_memory_writes_changed += u32::from(shared_memory_write_changed(
                    shared_memory[s as usize],
                    value,
                ));
                shared_memory[s as usize] = value;
                value
            }
            OutputSinkKind::ClearSlot(s) => {
                *shared_memory_writes_changed +=
                    u32::from(shared_memory_write_changed(shared_memory[s as usize], 0.0));
                shared_memory[s as usize] = 0.0;
                0.0
            }
            // The vote surface: the visit's contribution and the parameter
            // surface, which the pass end reads (T19.F04). Neither counts work.
            OutputSinkKind::ActionVote(sink) => {
                contribution[sink.index()] = value;
                value
            }
            OutputSinkKind::ActionParam(field) => {
                action_params[field.index()] = value;
                value
            }
        };

        tracer.on_sink(true, buf, value, applied_value);
    }

    (NodeResult::halted(output_slots, route_gates), contribution)
}
