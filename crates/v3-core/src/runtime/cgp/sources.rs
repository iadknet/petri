use crate::contracts::InputReference;
use crate::creature::genome::cgp::{GraphEdge, GraphSource};
use crate::runtime::inputs::{resolve_input, ResolveCtx};

/// Resolve a `GraphSource` to its scalar value during ordered graph evaluation.
///
/// For `ComputeNode` sources, uses index order: sources already
/// evaluated this visit (`idx < current_idx`) read from `curr_outputs`;
/// self/higher-index sources read frozen tick-start `prev_outputs`.
#[inline]
#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_source(
    source: &GraphSource,
    current_idx: usize,
    node_count: usize,
    prev_outputs: &[f32],
    curr_outputs: &[f32],
    input_refs: &[InputReference],
    resolve_ctx: &ResolveCtx<'_>,
    shared_memory: &[f32; 16],
    prev_shared_memory: &[f32; 16],
) -> f32 {
    match source {
        GraphSource::InputLeaf { ref_idx, sub_idx } => {
            let ri = *ref_idx as usize;
            if ri < input_refs.len() {
                resolve_input(&input_refs[ri], *sub_idx, resolve_ctx)
            } else {
                0.0
            }
        }
        GraphSource::SharedMemory { slot, previous } => {
            let idx = *slot as usize;
            if idx >= 16 {
                return 0.0;
            }
            if *previous {
                prev_shared_memory[idx]
            } else {
                shared_memory[idx]
            }
        }
        GraphSource::ComputeNode(idx) => {
            let i = *idx as usize;
            if i >= node_count {
                0.0
            } else if i < current_idx {
                curr_outputs[i]
            } else {
                prev_outputs[i]
            }
        }
    }
}

/// Resolve a source after evaluation where all compute outputs are finalized.
#[inline]
#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_source_post_convergence(
    source: &GraphSource,
    compute_count: usize,
    final_outputs: &[f32],
    input_refs: &[InputReference],
    resolve_ctx: &ResolveCtx<'_>,
    shared_memory: &[f32; 16],
    prev_shared_memory: &[f32; 16],
) -> f32 {
    resolve_source(
        source,
        compute_count,
        compute_count,
        final_outputs,
        final_outputs,
        input_refs,
        resolve_ctx,
        shared_memory,
        prev_shared_memory,
    )
}

/// Collect weighted input values for a compute node into `buf`.
///
/// Clears `buf` and fills it with one entry per edge: `resolve(source) * weight`.
/// The caller allocates `buf` once and reuses it across nodes/visits.
#[inline]
#[allow(clippy::too_many_arguments)]
pub(crate) fn collect_cgp_weighted_inputs(
    edges: &[GraphEdge],
    current_idx: usize,
    node_count: usize,
    prev_outputs: &[f32],
    curr_outputs: &[f32],
    input_refs: &[InputReference],
    resolve_ctx: &ResolveCtx<'_>,
    shared_memory: &[f32; 16],
    prev_shared_memory: &[f32; 16],
    buf: &mut Vec<f32>,
) {
    buf.clear();
    buf.extend(edges.iter().map(|edge| {
        let source_value = resolve_source(
            &edge.source,
            current_idx,
            node_count,
            prev_outputs,
            curr_outputs,
            input_refs,
            resolve_ctx,
            shared_memory,
            prev_shared_memory,
        );
        source_value * edge.weight
    }));
}
