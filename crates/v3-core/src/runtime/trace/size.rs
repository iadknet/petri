//! In-memory sizes of trace records, for recording budgets (T21.F04).
//!
//! A record's bytes are its `size_of` plus, for every vector it owns,
//! recursively, `capacity × size_of` of the element type: what the record
//! keeps allocated, spare capacity included.

use std::mem::size_of;

use crate::config::SimulationConfig;
use crate::contracts::WorldAction;
use crate::runtime::trace::domain::{
    AppliedAction, BackendTrace, GraphTrace, MeshHopTrace, MeshPassTrace, TickOutcome, TickTrace,
    VmTrace,
};

/// `capacity × size_of::<T>()` of `v`'s buffer.
fn buffer<T>(v: &Vec<T>) -> u64 {
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
        size_of::<Self>() as u64
            + buffer(&self.input_refs)
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

impl TickOutcome {
    /// Bytes the outcome owns beyond its own `size_of`.
    #[must_use]
    pub fn heap_bytes(&self) -> u64 {
        buffer(&self.typed_local_food.food_here_by_type)
            + buffer(&self.typed_local_food.neighbor_food_by_type)
            + buffer(&self.typed_area_food)
            + buffer(&self.applied)
    }
}

impl TickTrace {
    /// The tick record's bytes: its fixed part, its selected actions, every
    /// hop and pass record and its outcome.
    #[must_use]
    pub fn retained_bytes(&self) -> u64 {
        size_of::<Self>() as u64
            + buffer(&self.final_actions)
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
            + self.outcome.as_ref().map_or(0, TickOutcome::heap_bytes)
    }
}

/// What a recorded tick reserves against the caps before its first hop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickReserve {
    /// `max_actions_per_turn` applied-action events.
    pub events: u32,
    /// The tick record's fixed part, `max_actions_per_turn` selected and
    /// applied action records, the outcome's food banks at the configured
    /// type count, and the recorder's hop preallocation (`max_mesh_hops`).
    pub bytes: u64,
}

impl TickReserve {
    #[must_use]
    pub fn for_config(config: &SimulationConfig) -> Self {
        let actions = config.runtime.max_actions_per_turn;
        let types = config.world.food.types.len();
        let hops = config.runtime.max_mesh_hops.max(1) as usize;
        let bytes = size_of::<TickTrace>()
            + actions * (size_of::<WorldAction>() + size_of::<AppliedAction>())
            + types * (size_of::<f32>() + size_of::<[f32; 8]>() + size_of::<[f32; 7]>())
            + hops * size_of::<MeshHopTrace>();
        Self {
            events: u32::try_from(actions).unwrap_or(u32::MAX),
            bytes: bytes as u64,
        }
    }
}
