//! The tracing contract of the ordered graph evaluation and its effects
//! pass: the callbacks a tracer receives, and the zero-cost production one.

use crate::creature::genome::cgp::ComputeNodeKind;

/// Callback trait for instrumenting the ordered graph evaluation.
#[allow(clippy::too_many_arguments)]
pub(crate) trait GraphTracer {
    fn on_pass_start(&mut self, pass: u32, pass_cost: f32, energy_after: f32);
    fn on_node_eval(
        &mut self,
        node_index: usize,
        kind: &ComputeNodeKind,
        weighted_inputs: &[f32],
        weighted_sum: f32,
        state_before: f32,
        state_after: f32,
        output: f32,
    );
    fn on_pass_end(&mut self, delta: f32);
    fn on_finish(&mut self, curr_outputs: &[f32], temporal_committed: bool);
    /// One sink of the effects pass, in catalog order: whether it is wired,
    /// its weighted inputs and sum, and the value it applied.
    fn on_sink(
        &mut self,
        wired: bool,
        weighted_inputs: &[f32],
        weighted_sum: f32,
        applied: bool,
        applied_value: f32,
    );
    /// The effects pass ran: the visit committed.
    fn on_effects(&mut self);
}

/// Zero-cost tracer used by the production path.
pub(crate) struct NoopTracer;

impl GraphTracer for NoopTracer {
    #[inline]
    fn on_pass_start(&mut self, _: u32, _: f32, _: f32) {}
    #[inline]
    fn on_node_eval(
        &mut self,
        _: usize,
        _: &ComputeNodeKind,
        _: &[f32],
        _: f32,
        _: f32,
        _: f32,
        _: f32,
    ) {
    }
    #[inline]
    fn on_pass_end(&mut self, _: f32) {}
    #[inline]
    fn on_finish(&mut self, _: &[f32], _: bool) {}
    #[inline]
    fn on_sink(&mut self, _: bool, _: &[f32], _: f32, _: bool, _: f32) {}
    #[inline]
    fn on_effects(&mut self) {}
}
