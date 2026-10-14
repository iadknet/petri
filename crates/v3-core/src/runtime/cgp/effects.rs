//! Post-convergence effect pass for CGP-style graph backend evaluation.

use crate::contracts::{InputReference, MAX_GATE_SLOTS};
use crate::creature::genome::cgp::{CgpGraphBackendDef, GraphEdge, GraphSource, OutputSinkKind};
use crate::creature::genome::vote::{
    DirectedVoteKind, VoteVector, VOTE_DIRECTION_COUNT, VOTE_SINK_COUNT,
};
use crate::runtime::cgp::sources::resolve_source_post_convergence;
use crate::runtime::inputs::ResolveCtx;
use crate::runtime::routing::RouteGateMap;
use crate::runtime::trace::domain::GraphOutputSinkTrace;
use crate::runtime::types::{
    sanitize_f32, shared_memory_write_changed, MeshSideOutputs, NodeResult, OUTPUT_SLOT_COUNT,
};

pub(crate) struct CgpEffectsTrace {
    pub(crate) output_sinks: Vec<GraphOutputSinkTrace>,
}

#[inline]
#[allow(clippy::too_many_arguments)]
fn edges_wsum(
    edges: &[GraphEdge],
    compute_count: usize,
    curr_outputs: &[f32],
    input_refs: &[InputReference],
    resolve_ctx: &ResolveCtx<'_>,
    shared_memory: &[f32; 16],
    prev_shared_memory: &[f32; 16],
    buf: &mut Vec<f32>,
) -> f32 {
    buf.clear();
    buf.extend(edges.iter().map(|edge| {
        let v = resolve_source_post_convergence(
            &edge.source(),
            compute_count,
            curr_outputs,
            input_refs,
            resolve_ctx,
            shared_memory,
            prev_shared_memory,
        );
        v * edge.weight()
    }));
    buf.iter().copied().sum()
}

/// Per directed kind, the projection term of each direction's vote, or `None`
/// when the kind has no projection (T11.F31).
type ProjectionTerms = [Option<DirectionTerms>; DirectedVoteKind::ALL.len()];
type DirectionTerms = [f32; VOTE_DIRECTION_COUNT as usize];

/// `P_K[d] = Σ w_p · r_p((d − k_p) mod 8)` over each kind's projections in
/// genome order, accumulated from 0.0. A projection whose `ref_idx` is out of
/// range or not a ring contributes nothing, whatever its weight; an ablated
/// channel reads 0.0, as it does for an edge.
fn projection_terms(
    def: &CgpGraphBackendDef,
    compute_count: usize,
    curr_outputs: &[f32],
    input_refs: &[InputReference],
    resolve_ctx: &ResolveCtx<'_>,
    shared_memory: &[f32; 16],
    prev_shared_memory: &[f32; 16],
) -> ProjectionTerms {
    let mut terms: ProjectionTerms = [None; DirectedVoteKind::ALL.len()];
    for projection in &def.projections {
        let entry = terms[projection.kind.index()].get_or_insert([0.0; _]);
        if !input_refs
            .get(usize::from(projection.ref_idx))
            .is_some_and(InputReference::is_ring)
        {
            continue;
        }
        let mut ring: DirectionTerms = [0.0; _];
        for (channel, value) in ring.iter_mut().enumerate() {
            if projection.ablated_channels & (1 << channel) != 0 {
                continue;
            }
            *value = resolve_source_post_convergence(
                &GraphSource::InputLeaf {
                    ref_idx: projection.ref_idx,
                    sub_idx: channel as u16,
                },
                compute_count,
                curr_outputs,
                input_refs,
                resolve_ctx,
                shared_memory,
                prev_shared_memory,
            );
        }
        for (d, term) in entry.iter_mut().enumerate() {
            let channel = projection.channel_for(d as u8);
            *term += projection.weight * ring[usize::from(channel)];
        }
    }
    terms
}

/// Apply post-convergence effects from CGP graph evaluation.
#[inline]
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_cgp_graph_effects(
    def: &CgpGraphBackendDef,
    curr_outputs: &[f32],
    input_refs: &[InputReference],
    resolve_ctx: &ResolveCtx<'_>,
    upstream_slots: &[f32; OUTPUT_SLOT_COUNT],
    side_outputs: &mut MeshSideOutputs,
    shared_memory: &mut [f32; 16],
    prev_shared_memory: &[f32; 16],
) -> (NodeResult, CgpEffectsTrace) {
    let mut output_slots = *upstream_slots;
    let mut route_gates = RouteGateMap::default();
    let compute_count = def.compute_nodes().len();
    let mut buf = Vec::with_capacity(8);
    let mut output_sink_traces = Vec::with_capacity(def.output_sinks().len());
    // This visit's vote contribution (T19.F03): the sanitized weighted sum of
    // each wired `ActionVote` sink, 0 for an unwired one. Staged at the end of
    // the effects, which only run when the visit commits.
    let mut contribution: VoteVector = [0.0; VOTE_SINK_COUNT];
    let projections = projection_terms(
        def,
        compute_count,
        curr_outputs,
        input_refs,
        resolve_ctx,
        shared_memory,
        prev_shared_memory,
    );

    for sink in def.output_sinks() {
        // A directed vote sink's projection term (T11.F31), added to its edge
        // sum; the sink is applied when it has edges or a projection.
        let projection = match sink.kind {
            OutputSinkKind::ActionVote(vote) => {
                DirectedVoteKind::of_sink(vote).and_then(|(kind, d)| {
                    projections[kind.index()].and_then(|terms| terms.get(usize::from(d)).copied())
                })
            }
            _ => None,
        };
        if sink.inputs.is_empty() && projection.is_none() {
            output_sink_traces.push(GraphOutputSinkTrace {
                wired: false,
                weighted_sum: 0.0,
                applied: false,
                applied_value: 0.0,
            });
            continue;
        }

        let edge_sum = if sink.inputs.is_empty() {
            0.0
        } else {
            edges_wsum(
                &sink.inputs,
                compute_count,
                curr_outputs,
                input_refs,
                resolve_ctx,
                shared_memory,
                prev_shared_memory,
                &mut buf,
            )
        };
        let wsum = match projection {
            Some(term) => edge_sum + term,
            None => edge_sum,
        };
        let mut applied = false;
        let mut applied_value = 0.0;

        match sink.kind {
            OutputSinkKind::CustomOutput(s) => {
                if (s as usize) < OUTPUT_SLOT_COUNT {
                    applied_value = sanitize_f32(wsum);
                    output_slots[s as usize] = applied_value;
                    applied = true;
                }
            }
            OutputSinkKind::RouterGate(slot) => {
                let s = slot as usize;
                if s < MAX_GATE_SLOTS {
                    let val = sanitize_f32(wsum);
                    route_gates.scores[s] = val;
                    applied_value = val;
                    applied = true;
                }
            }
            OutputSinkKind::WriteSlot(s) => {
                if (s as usize) < 16 {
                    applied_value = sanitize_f32(wsum);
                    side_outputs.work_counters.shared_memory_writes_changed += u32::from(
                        shared_memory_write_changed(shared_memory[s as usize], applied_value),
                    );
                    shared_memory[s as usize] = applied_value;
                    applied = true;
                }
            }
            OutputSinkKind::ClearSlot(s) => {
                if (s as usize) < 16 {
                    side_outputs.work_counters.shared_memory_writes_changed +=
                        u32::from(shared_memory_write_changed(shared_memory[s as usize], 0.0));
                    shared_memory[s as usize] = 0.0;
                    applied_value = 0.0;
                    applied = true;
                }
            }
            // The vote surface: the visit's contribution and the parameter
            // surface, which the pass end reads (T19.F04). Neither counts work.
            OutputSinkKind::ActionVote(sink) => {
                if let Some(entry) = contribution.get_mut(sink.index()) {
                    applied_value = sanitize_f32(wsum);
                    *entry = applied_value;
                    applied = true;
                }
            }
            OutputSinkKind::ActionParam(field) => {
                applied_value = sanitize_f32(wsum);
                side_outputs.action_params[field.index()] = applied_value;
                applied = true;
            }
        }

        output_sink_traces.push(GraphOutputSinkTrace {
            wired: true,
            weighted_sum: sanitize_f32(wsum),
            applied,
            applied_value,
        });
    }

    // The visit reached its effects, so it commits (T19.F03): an exhausted
    // visit returns before this pass and stages nothing.
    side_outputs.stage_vote_contribution(&contribution);

    (
        NodeResult::halted(output_slots, route_gates),
        CgpEffectsTrace {
            output_sinks: output_sink_traces,
        },
    )
}
