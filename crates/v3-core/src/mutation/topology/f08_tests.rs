//! T11.F08 mesh qualification: `CopyNode` produces a dormant paralog that
//! shares its original's shared-memory addresses, output slots, and input
//! references, so activating it with `SwapRouteTargets` runs it in the
//! original's chain position and reproduces the original's behavior unless a
//! dormant mutation diverged it first. No topology code changes here; this
//! records the post-activation properties T11.F15 left to F08.

use super::*;
use crate::config::RuntimeConfig;
use crate::contracts::RouteTarget;
use crate::creature::genome::analysis::mesh_reachable_nodes;
use crate::creature::genome::{VmBackendDef, VmInstruction};
use crate::mutation::vm::mutate_one_instruction_field;
use crate::neighborhood::battery::{Battery, Signature};
use rand::{rngs::SmallRng, SeedableRng};

/// Entry node: halts without terminating the chain, so routing continues to
/// whichever target wins.
fn entry_node(target: u32) -> NodeGenome {
    NodeGenome {
        node_id: NodeId::new(0),
        input_refs: vec![],
        backend_def: birth::minimal_vm_backend(),
        targets: vec![RouteTarget {
            target_id: NodeId::new(target),
            slot: 0,
            gate_bias: 0.0,
        }],
    }
}

/// Worker node: writes a shared-memory slot and commits one action, so both
/// the action queue and shared memory carry its behavior.
fn worker_node() -> NodeGenome {
    NodeGenome {
        node_id: NodeId::new(1),
        input_refs: vec![],
        backend_def: BackendDef::Vm(VmBackendDef {
            register_count: 2,
            constants: vec![0.75],
            program: vec![
                VmInstruction::LoadConst {
                    dst: 0,
                    const_idx: 0,
                },
                VmInstruction::StoreSlotImm {
                    slot_idx: 2,
                    src: 0,
                },
                VmInstruction::AddVote { sink: 9, src: 0 },
                VmInstruction::Halt,
            ],
        }),
        targets: vec![],
    }
}

fn chain_genome() -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![entry_node(1), worker_node()],
    }
}

fn apply(
    genome: &mut CreatureGenome,
    op: TopologyOperator,
    seed: u64,
) -> Result<TargetReachability, MutationSkipReason> {
    let reachable = mesh_reachable_nodes(genome);
    TopologyMutator::apply(
        genome,
        op,
        &mut TargetSelector::reachable_only(&reachable, 0.0),
        &mut SmallRng::seed_from_u64(seed),
        &MutationConfig::default(),
    )
}

fn signature(genome: &CreatureGenome) -> Signature {
    Battery::generate(1).signature(genome, &RuntimeConfig::default(), 0.0)
}

/// Apply the production single-field step to the clone's first instruction.
fn diverge_clone(genome: &mut CreatureGenome, seed: u64) {
    let BackendDef::Vm(vm) = &mut genome.nodes[2].backend_def else {
        panic!("the clone carries the original's VM backend");
    };
    let index = vm
        .program
        .iter()
        .position(|instruction| matches!(instruction, VmInstruction::AddVote { .. }))
        .expect("the clone carries the copied AddVote");
    assert!(mutate_one_instruction_field(
        &mut vm.program[index],
        &mut SmallRng::seed_from_u64(seed)
    ));
}

#[test]
fn copy_node_clone_shares_its_original_addresses_slots_and_references() {
    let mut genome = chain_genome();
    apply(&mut genome, TopologyOperator::CopyNode, 7).unwrap();
    let (original, clone) = (&genome.nodes[1], &genome.nodes[2]);
    assert_eq!(clone.backend_def, original.backend_def);
    assert_eq!(clone.input_refs, original.input_refs);
    assert_ne!(clone.node_id, original.node_id);
    assert_eq!(
        genome.nodes[0].targets.len(),
        2,
        "the clone is attached behind the original's predecessor"
    );
    assert_eq!(genome.nodes[0].targets[0].target_id, original.node_id);
    assert_eq!(genome.nodes[0].targets[1].target_id, clone.node_id);
}

#[test]
fn mesh_copy_diverge_and_activate_trajectory() {
    let parent = chain_genome();
    let base = signature(&parent);

    let mut copied = parent.clone();
    apply(&mut copied, TopologyOperator::CopyNode, 7).unwrap();
    assert_eq!(
        signature(&copied),
        base,
        "the dormant clone loses every route tie to its original"
    );

    let mut diverged = copied.clone();
    diverge_clone(&mut diverged, 5);
    assert_eq!(
        signature(&diverged),
        base,
        "diverging the dormant clone is still silent"
    );

    for (label, genome, expect_equal) in [("exact", copied, true), ("diverged", diverged, false)] {
        let mut activated = genome;
        apply(&mut activated, TopologyOperator::SwapRouteTargets, 3).unwrap();
        assert_eq!(
            activated.nodes[0].targets[0].target_id, activated.nodes[2].node_id,
            "{label}: the swap runs the clone in the original's chain position"
        );
        let activated_signature = signature(&activated);
        if expect_equal {
            assert_eq!(
                activated_signature, base,
                "{label}: the activated clone reproduces the original"
            );
        } else {
            assert_ne!(
                activated_signature, base,
                "{label}: the activated diverged clone behaves differently"
            );
        }
    }
}
