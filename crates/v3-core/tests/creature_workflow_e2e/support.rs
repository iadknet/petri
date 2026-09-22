use v3_core::contracts::NodeId;
use v3_core::creature::genome::{
    BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
};
use v3_core::runtime::trace::domain::{BackendTrace, TickTrace, VmTrace};

// Shared across test binaries; single source of truth in `tests/common/mod.rs`.
pub(crate) use crate::common::{graph_hop, insert_creature, run_one_traced_tick, test_config};

pub(crate) fn vm_emit_noop_genome() -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                // A `Terminate` vote commits nothing: the tick is `NoOp`.
                program: vec![
                    VmInstruction::AddVote { sink: 25, src: 0 },
                    VmInstruction::Halt,
                ],
            }),
            targets: vec![],
        }],
    }
}

pub(crate) fn vm_hop(tick: &TickTrace, hop_idx: usize) -> &VmTrace {
    match &tick.hops[hop_idx].backend_trace {
        BackendTrace::Vm(v) => v,
        BackendTrace::Graph(_) => panic!("expected vm hop"),
    }
}
