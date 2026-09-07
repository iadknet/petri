use crate::config::MutationConfig;
use crate::contracts::{NodeId, RouteTarget};
use crate::creature::genome::cgp::CgpGraphBackendDef;
use crate::creature::genome::{BackendDef, NodeGenome, VmBackendDef, VmInstruction};

pub(super) fn minimal_vm_backend() -> BackendDef {
    BackendDef::Vm(VmBackendDef {
        register_count: 1,
        constants: vec![],
        program: vec![VmInstruction::Halt],
    })
}

pub(super) fn blank_graph_backend(config: &MutationConfig) -> BackendDef {
    BackendDef::Graph(CgpGraphBackendDef::new_with_fixed_outputs(config))
}

/// Halt preserves the incoming bus and all mesh side outputs while continuing the chain.
pub(super) fn detour(node_id: NodeId, successor: NodeId) -> NodeGenome {
    NodeGenome {
        node_id,
        input_refs: vec![],
        backend_def: minimal_vm_backend(),
        targets: vec![RouteTarget {
            target_id: successor,
            slot: 0,
            gate_bias: 0.0,
        }],
    }
}
