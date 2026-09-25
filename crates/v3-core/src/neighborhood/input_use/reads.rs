//! A mesh execution mode that records which input consumers actually
//! resolved a read: every step a VM dispatch executed past its charge, every
//! compute node a Graph visit evaluated, and a Graph visit's sink edges when
//! its effects pass ran. Execution is the production executor unchanged; the
//! mode only observes through the existing tracer hooks.

use std::collections::BTreeSet;

use crate::config::RuntimeConfig;
use crate::creature::genome::cgp::ComputeNodeKind;
use crate::creature::genome::{BackendDef, NodeGenome, VmBackendDef, VmInstruction};
use crate::creature::state::GraphRuntimeState;
use crate::runtime::cgp::effects::CgpEffectsTrace;
use crate::runtime::cgp::execute::{execute_graph_impl, GraphTracer};
use crate::runtime::mesh::MeshExecutionMode;
use crate::runtime::types::{MeshOutput, MeshSideOutputs, NodeResult, OUTPUT_SLOT_COUNT};
use crate::runtime::vm::{execute_vm_node_impl, nr, VmTraceSink};
use crate::sensors::perception::SensorSnapshot;

/// One recorded read site.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum ReadEvent {
    /// A VM step at `pc` of node `node` that executed; `slot` is the slot a
    /// dynamic `LoadSlot` addressed.
    Vm {
        node: usize,
        pc: usize,
        slot: Option<u8>,
    },
    /// Every edge of compute node `index` of Graph node `node`.
    Compute { node: usize, index: usize },
    /// Every sink edge of Graph node `node`.
    Sinks { node: usize },
}

/// Records the read events of one execution.
#[derive(Default)]
pub(super) struct ReadRecording {
    reads: BTreeSet<ReadEvent>,
}

struct VmReads<'a> {
    node: usize,
    reads: &'a mut BTreeSet<ReadEvent>,
    pending: Option<ReadEvent>,
}

impl VmTraceSink for VmReads<'_> {
    type Output = ();

    fn finish_empty(self, _: &VmBackendDef, _: &[f32; OUTPUT_SLOT_COUNT]) {}

    fn before_instruction(&mut self, pc: usize, instruction: &VmInstruction, registers: &[f32]) {
        let slot = match *instruction {
            VmInstruction::LoadSlot { slot_reg, .. } => {
                let value = registers[nr(slot_reg, registers.len())];
                Some((value as i64).rem_euclid(16) as u8)
            }
            VmInstruction::ReadInput { .. }
            | VmInstruction::LoadSlotImm { .. }
            | VmInstruction::LoadSlotPrev { .. } => None,
            _ => return,
        };
        self.pending = Some(ReadEvent::Vm {
            node: self.node,
            pc,
            slot,
        });
    }

    fn after_instruction(
        &mut self,
        _: usize,
        _: &VmInstruction,
        _: f32,
        energy_after: f32,
        _: &[f32],
    ) {
        // The exhaustion break reports a non-positive energy and executes
        // nothing; every executed step leaves positive energy.
        if let Some(event) = self.pending.take() {
            if energy_after > 0.0 {
                self.reads.insert(event);
            }
        }
    }

    fn finish(self, _: &VmBackendDef, _: &[f32], _: [f32; OUTPUT_SLOT_COUNT]) {}
}

struct GraphReads<'a> {
    node: usize,
    reads: &'a mut BTreeSet<ReadEvent>,
}

impl GraphTracer for GraphReads<'_> {
    fn on_pass_start(&mut self, _: u32, _: f32, _: f32) {}

    fn on_node_eval(
        &mut self,
        node_index: usize,
        _: &ComputeNodeKind,
        _: &[f32],
        _: f32,
        _: f32,
        _: f32,
        _: f32,
    ) {
        self.reads.insert(ReadEvent::Compute {
            node: self.node,
            index: node_index,
        });
    }

    fn on_pass_end(&mut self, _: f32) {}

    fn on_finish(&mut self, _: &[f32], _: bool) {}

    fn on_effects(&mut self, _: CgpEffectsTrace) {
        self.reads.insert(ReadEvent::Sinks { node: self.node });
    }
}

impl MeshExecutionMode for ReadRecording {
    type BackendTrace = ();
    type Output = (MeshOutput, BTreeSet<ReadEvent>);

    const RECORDS_HOPS: bool = false;

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
    ) -> (NodeResult, ()) {
        let result = match &node.backend_def {
            BackendDef::Vm(def) => {
                execute_vm_node_impl(
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
                    VmReads {
                        node: node_idx,
                        reads: &mut self.reads,
                        pending: None,
                    },
                )
                .0
            }
            BackendDef::Graph(def) => execute_graph_impl(
                &mut GraphReads {
                    node: node_idx,
                    reads: &mut self.reads,
                },
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

    fn finish(self, output: MeshOutput) -> Self::Output {
        (output, self.reads)
    }
}
