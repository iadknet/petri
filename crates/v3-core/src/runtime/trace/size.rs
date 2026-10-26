//! In-memory sizes of trace records, for recording budgets (T21.F04).
//!
//! A record's bytes are its `size_of` plus, for every vector it owns,
//! recursively, `capacity × size_of` of the element type: what the record
//! keeps allocated, spare capacity included.

use std::mem::size_of;

use crate::runtime::trace::domain::{
    BackendTrace, GraphTrace, MeshHopTrace, MeshPassTrace, TickTrace, VmTrace,
};

/// `capacity × size_of::<T>()` of `v`'s buffer.
pub(crate) fn buffer<T>(v: &Vec<T>) -> u64 {
    (v.capacity() * size_of::<T>()) as u64
}

/// The buffer slots of `v` beyond its length, whose elements count themselves.
fn spare<T>(v: &Vec<T>) -> u64 {
    ((v.capacity() - v.len()) * size_of::<T>()) as u64
}

fn vm_heap(trace: &VmTrace) -> u64 {
    buffer(&trace.constants)
        + buffer(&trace.steps)
        + trace
            .steps
            .iter()
            .map(|step| buffer(&step.register_changes))
            .sum::<u64>()
        + buffer(&trace.final_registers)
        + buffer(&trace.slot_writes)
}

fn graph_heap(trace: &GraphTrace) -> u64 {
    buffer(&trace.passes)
        + trace
            .passes
            .iter()
            .map(|pass| {
                buffer(&pass.node_evaluations)
                    + pass
                        .node_evaluations
                        .iter()
                        .map(|eval| buffer(&eval.weighted_inputs))
                        .sum::<u64>()
            })
            .sum::<u64>()
        + buffer(&trace.final_outputs)
        + buffer(&trace.output_sinks)
        + trace
            .output_sinks
            .iter()
            .map(|sink| buffer(&sink.weighted_inputs))
            .sum::<u64>()
}

impl BackendTrace {
    /// Bytes the backend trace owns beyond its own `size_of`.
    #[must_use]
    pub fn heap_bytes(&self) -> u64 {
        match self {
            Self::Vm(trace) => vm_heap(trace),
            Self::Graph(trace) => graph_heap(trace),
        }
    }
}

impl MeshHopTrace {
    /// The hop record's bytes, backend trace included.
    #[must_use]
    pub fn retained_bytes(&self) -> u64 {
        size_of::<Self>() as u64 + self.heap_bytes()
    }

    /// Bytes the hop record owns beyond its own `size_of`.
    #[must_use]
    pub fn heap_bytes(&self) -> u64 {
        buffer(&self.input_refs)
            + self
                .route
                .as_ref()
                .map_or(0, |route| buffer(&route.gate_scores))
            + self.backend_trace.heap_bytes()
    }
}

impl MeshPassTrace {
    /// The pass record's bytes.
    #[must_use]
    pub fn retained_bytes(&self) -> u64 {
        size_of::<Self>() as u64
    }
}

impl TickTrace {
    /// The tick record's bytes: its fixed part, its selected actions and
    /// every hop and pass record.
    #[must_use]
    pub fn retained_bytes(&self) -> u64 {
        size_of::<Self>() as u64 + self.heap_bytes()
    }

    /// Bytes the tick record owns beyond its own `size_of`.
    #[must_use]
    pub fn heap_bytes(&self) -> u64 {
        buffer(&self.final_actions)
            + spare(&self.hops)
            + self
                .hops
                .iter()
                .map(MeshHopTrace::retained_bytes)
                .sum::<u64>()
            + spare(&self.passes)
            + self
                .passes
                .iter()
                .map(MeshPassTrace::retained_bytes)
                .sum::<u64>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::trace::domain::GraphOutputSinkTrace;

    fn sink(capacity: usize, len: usize) -> GraphOutputSinkTrace {
        let mut weighted_inputs = Vec::with_capacity(capacity);
        weighted_inputs.extend(std::iter::repeat_n(0.5_f32, len));
        GraphOutputSinkTrace {
            wired: len > 0,
            weighted_inputs,
            weighted_sum: 0.0,
            applied: false,
            applied_value: 0.0,
        }
    }

    /// Each sink's weighted-input buffer (T25.F06) counts at its capacity,
    /// spare slots included, on top of the sink records themselves.
    #[test]
    fn graph_heap_counts_every_sink_weighted_input_buffer() {
        let output_sinks = vec![sink(8, 3), sink(0, 0), sink(5, 5)];
        let sinks_bytes = (output_sinks.capacity() * size_of::<GraphOutputSinkTrace>()) as u64;
        let inputs_bytes: u64 = output_sinks
            .iter()
            .map(|sink| (sink.weighted_inputs.capacity() * size_of::<f32>()) as u64)
            .sum();
        let trace = BackendTrace::Graph(GraphTrace {
            passes: Vec::new(),
            temporal_committed: false,
            final_outputs: Vec::new(),
            output_sinks,
        });

        assert_eq!(trace.heap_bytes(), sinks_bytes + inputs_bytes);
    }
}
