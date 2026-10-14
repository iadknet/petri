use rand::Rng;

use crate::contracts::{NodeId, RouteTarget};
use crate::creature::genome::cgp::{BirthSeeds, CgpGraphBackendDef};
use crate::creature::genome::{BackendDef, NodeGenome, VmBackendDef, VmInstruction};

pub(super) fn minimal_vm_backend() -> BackendDef {
    BackendDef::Vm(VmBackendDef {
        register_count: 1,
        constants: vec![],
        program: vec![VmInstruction::Halt],
    })
}

/// Carries the empty seed form, so a newborn of a parent with no Graph node
/// still gets the birth scaffold (T25.F01 I6).
pub(super) fn blank_graph_backend() -> BackendDef {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    def.seed_birth(BirthSeeds::Empty);
    BackendDef::Graph(def)
}

/// Either blank backend preserves the incoming bus and mesh side outputs.
pub(super) fn detour(node_id: NodeId, successor: NodeId, rng: &mut impl Rng) -> NodeGenome {
    NodeGenome {
        node_id,
        input_refs: vec![],
        backend_def: if rng.gen_bool(0.5) {
            blank_graph_backend()
        } else {
            minimal_vm_backend()
        },
        targets: vec![RouteTarget {
            target_id: successor,
            slot: 0,
            gate_bias: 0.0,
        }],
    }
}
