//! T13.F03 applicability properties: a node-internal mutation operator
//! selects its target from the modules on which it can actually apply.
//!
//! The five invariants of the feature spec, over generated genomes:
//! 1. every node the predicate accepts applies without skipping,
//! 2. a predicate that accepts no node skips before the draw and leaves the
//!    genome untouched,
//! 3. padding with inapplicable nodes never removes an applicable one,
//! 4. growth and connection operators still reach blank and dormant modules,
//! 5. the executed/reachable draw stays inside the applicable set and
//!    `first_pick` records the node it selected.
//!
//! Genomes are generated from a proptest seed plus structural sizes: the
//! seed drives the same `random_*` samplers the operators use, so the cases
//! are drawn from the shapes mutation actually produces. No assertion below
//! depends on which case was drawn.

use proptest::prelude::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::config::MutationConfig;
use crate::contracts::{InputReference, NodeId};
use crate::creature::genome::cgp::{CgpGraphBackendDef, ComputeNode, GraphEdge};
use crate::creature::genome::{
    BackendDef, CreatureGenome, HebbianRule, NodeGenome, OutcomeChannel, PlasticityConfig,
    RewardModulationConfig, VmBackendDef, VmInstruction,
};
use crate::mutation::graph::operators::{random_compute_node_kind, random_graph_source};
use crate::mutation::graph::{GraphMutator, GraphOperator};
use crate::mutation::input_ref::{InputRefMutator, InputRefOperator};
use crate::mutation::reachability::TargetSelector;
use crate::mutation::sampling::random_input_reference;
use crate::mutation::types::MutationSkipReason;
use crate::mutation::vm::operators::random_vm_instruction;
use crate::mutation::vm::{VmMutator, VmOperator};

// ─── Generators ─────────────────────────────────────────────────────────────

/// How large a generated module is. Zero sizes are in range on purpose:
/// blank modules are exactly the case the applicability filter exists for.
#[derive(Debug, Clone, Copy)]
struct ModuleSize {
    compute_nodes: usize,
    edges_per_node: usize,
    sinks: usize,
    action_slots: usize,
    gate_edges: usize,
    input_refs: usize,
    instructions: usize,
    constants: usize,
}

fn module_size() -> impl Strategy<Value = ModuleSize> {
    (
        0usize..4,
        0usize..3,
        0usize..3,
        0usize..3,
        0usize..3,
        0usize..3,
        0usize..6,
        0usize..3,
    )
        .prop_map(
            |(
                compute_nodes,
                edges_per_node,
                sinks,
                action_slots,
                gate_edges,
                input_refs,
                instructions,
                constants,
            )| {
                ModuleSize {
                    compute_nodes,
                    edges_per_node,
                    sinks,
                    action_slots,
                    gate_edges,
                    input_refs,
                    instructions,
                    constants,
                }
            },
        )
}

fn input_refs(rng: &mut SmallRng, count: usize) -> Vec<InputReference> {
    (0..count).map(|_| random_input_reference(rng)).collect()
}

fn plasticity(rng: &mut SmallRng) -> Option<PlasticityConfig> {
    if !rng.gen_bool(0.4) {
        return None;
    }
    Some(PlasticityConfig {
        rule: HebbianRule::Classic,
        learning_rate: 0.1,
        weight_clamp: 5.0,
        lamarckian: false,
        modulation: rng.gen_bool(0.5).then_some(RewardModulationConfig {
            reward_source: OutcomeChannel::EnergyDelta,
            trace_decay: 0.9,
        }),
    })
}

fn graph_def(
    rng: &mut SmallRng,
    size: ModuleSize,
    refs: &[InputReference],
    config: &MutationConfig,
) -> CgpGraphBackendDef {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs(config);
    def.output_sinks.truncate(size.sinks);
    def.action_bank.truncate(size.action_slots);
    for _ in 0..size.compute_nodes {
        def.compute_nodes.push(ComputeNode {
            kind: random_compute_node_kind(rng),
            inputs: Vec::new(),
            plasticity: plasticity(rng),
        });
    }
    // Edges are wired after every node exists so a `ComputeNode` source can
    // name any index, including a dangling one.
    let compute_count = def.compute_nodes.len() as u16;
    let edge = |rng: &mut SmallRng| GraphEdge {
        source: random_graph_source(compute_count, refs, config, rng),
        weight: rng.gen_range(-1.0f32..=1.0),
    };
    for i in 0..def.compute_nodes.len() {
        for _ in 0..size.edges_per_node {
            let e = edge(rng);
            def.compute_nodes[i].inputs.push(e);
        }
    }
    for i in 0..def.output_sinks.len() {
        for _ in 0..size.edges_per_node {
            let e = edge(rng);
            def.output_sinks[i].inputs.push(e);
        }
    }
    for i in 0..def.action_bank.len() {
        for _ in 0..size.edges_per_node {
            let e = edge(rng);
            def.action_bank[i].gate_inputs.push(e);
        }
    }
    for _ in 0..size.gate_edges {
        let e = edge(rng);
        def.execute_gate.inputs.push(e);
    }
    def
}

fn vm_def(rng: &mut SmallRng, size: ModuleSize) -> VmBackendDef {
    let register_count = rng.gen_range(1u8..=8);
    let constants: Vec<f32> = (0..size.constants)
        .map(|_| rng.gen_range(-1.0f32..=1.0))
        .collect();
    let program = (0..size.instructions)
        .map(|_| random_vm_instruction(rng, register_count, constants.len(), size.input_refs))
        .collect();
    VmBackendDef {
        register_count,
        constants,
        program,
    }
}

/// A genome of `sizes.len()` nodes, alternating Graph and VM backends.
fn genome(seed: u64, sizes: &[ModuleSize], config: &MutationConfig) -> CreatureGenome {
    let mut rng = SmallRng::seed_from_u64(seed);
    let nodes = sizes
        .iter()
        .enumerate()
        .map(|(i, &size)| {
            let refs = input_refs(&mut rng, size.input_refs);
            let backend_def = if i % 2 == 0 {
                BackendDef::Graph(graph_def(&mut rng, size, &refs, config))
            } else {
                BackendDef::Vm(vm_def(&mut rng, size))
            };
            NodeGenome {
                node_id: NodeId::new(i as u32),
                input_refs: refs,
                backend_def,
                targets: Vec::new(),
            }
        })
        .collect();
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes,
    }
}

/// The blank Graph module the mesh birth operators create.
fn blank_graph_node(id: u32, config: &MutationConfig) -> NodeGenome {
    NodeGenome {
        node_id: NodeId::new(id),
        input_refs: Vec::new(),
        backend_def: BackendDef::Graph(CgpGraphBackendDef::new_with_fixed_outputs(config)),
        targets: Vec::new(),
    }
}

/// The minimal VM module the mesh birth operators create.
fn blank_vm_node(id: u32) -> NodeGenome {
    NodeGenome {
        node_id: NodeId::new(id),
        input_refs: Vec::new(),
        backend_def: BackendDef::Vm(VmBackendDef {
            register_count: 1,
            constants: Vec::new(),
            program: vec![VmInstruction::Halt],
        }),
        targets: Vec::new(),
    }
}

fn selector<'a>() -> TargetSelector<'a> {
    TargetSelector::reachable_only(&[], 0.0)
}

// ─── Invariants 1, 2 and 5 ──────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// Invariant 1: every node a Graph operator's predicate accepts applies
    /// without skipping, over every seed the dispatch draws with.
    #[test]
    fn accepted_graph_node_always_applies(
        seed in any::<u64>(),
        sizes in prop::collection::vec(module_size(), 1..4),
    ) {
        let config = MutationConfig::default();
        let base = genome(seed, &sizes, &config);
        for &op in &GraphOperator::ALL {
            for node_idx in GraphMutator::applicable_indices(&base, op, &config) {
                for apply_seed in 0u64..8 {
                    let mut genome = base.clone();
                    let mut rng = SmallRng::seed_from_u64(seed ^ apply_seed);
                    let result =
                        GraphMutator::apply_to_node(&mut genome, op, node_idx, &mut rng, &config);
                    prop_assert!(
                        result.is_ok(),
                        "{op:?} accepted node {node_idx} but skipped: {result:?}"
                    );
                }
            }
        }
    }

    /// Invariant 1 for the VM domain.
    #[test]
    fn accepted_vm_node_always_applies(
        seed in any::<u64>(),
        sizes in prop::collection::vec(module_size(), 1..4),
    ) {
        let config = MutationConfig::default();
        let base = genome(seed, &sizes, &config);
        for &op in &VmOperator::ALL {
            for node_idx in VmMutator::applicable_indices(&base, op) {
                for apply_seed in 0u64..8 {
                    let mut genome = base.clone();
                    let mut rng = SmallRng::seed_from_u64(seed ^ apply_seed);
                    let result =
                        VmMutator::apply_to_node(&mut genome, op, node_idx, &mut rng, &config);
                    prop_assert!(
                        result.is_ok(),
                        "{op:?} accepted node {node_idx} but skipped: {result:?}"
                    );
                }
            }
        }
    }

    /// Invariants 2 and 5: an empty applicable set skips before the draw and
    /// leaves the genome untouched; a non-empty one applies and records a
    /// `first_pick` inside the set.
    #[test]
    fn empty_applicable_set_skips_and_pick_stays_inside_it(
        seed in any::<u64>(),
        sizes in prop::collection::vec(module_size(), 1..4),
    ) {
        let config = MutationConfig::default();
        let base = genome(seed, &sizes, &config);
        for &op in &GraphOperator::ALL {
            let applicable = GraphMutator::applicable_indices(&base, op, &config);
            let mut genome = base.clone();
            let mut rng = SmallRng::seed_from_u64(seed);
            let mut targets = selector();
            let result = GraphMutator::apply(&mut genome, op, &mut targets, &mut rng, &config);
            if applicable.is_empty() {
                prop_assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
                prop_assert_eq!(&genome, &base, "{:?} mutated on a skip", op);
                prop_assert_eq!(targets.first_pick(), None, "{:?} drew before skipping", op);
            } else {
                prop_assert!(result.is_ok(), "{op:?} had targets {applicable:?} but skipped");
                let pick = targets.first_pick();
                prop_assert!(
                    pick.is_some_and(|i| applicable.contains(&i)),
                    "{op:?} picked {pick:?} outside {applicable:?}"
                );
            }
        }
        for &op in &VmOperator::ALL {
            let applicable = VmMutator::applicable_indices(&base, op);
            let mut genome = base.clone();
            let mut rng = SmallRng::seed_from_u64(seed);
            let mut targets = selector();
            let result = VmMutator::apply(&mut genome, op, &mut targets, &mut rng, &config);
            if applicable.is_empty() {
                prop_assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
                prop_assert_eq!(&genome, &base, "{:?} mutated on a skip", op);
                prop_assert_eq!(targets.first_pick(), None, "{:?} drew before skipping", op);
            } else {
                prop_assert!(result.is_ok(), "{op:?} had targets {applicable:?} but skipped");
                let pick = targets.first_pick();
                prop_assert!(
                    pick.is_some_and(|i| applicable.contains(&i)),
                    "{op:?} picked {pick:?} outside {applicable:?}"
                );
            }
        }
    }

    /// Invariant 3: appending blank modules never removes an applicable node
    /// from an operator's eligible set, and an operator that had a target
    /// still applies.
    #[test]
    fn padding_never_shrinks_the_applicable_set(
        seed in any::<u64>(),
        sizes in prop::collection::vec(module_size(), 1..4),
        padding in 1usize..4,
    ) {
        let config = MutationConfig::default();
        let base = genome(seed, &sizes, &config);
        let mut padded = base.clone();
        for i in 0..padding {
            let id = (base.nodes.len() + i) as u32;
            padded.nodes.push(if i % 2 == 0 {
                blank_graph_node(id, &config)
            } else {
                blank_vm_node(id)
            });
        }
        for &op in &GraphOperator::ALL {
            let before = GraphMutator::applicable_indices(&base, op, &config);
            let after = GraphMutator::applicable_indices(&padded, op, &config);
            for idx in &before {
                prop_assert!(after.contains(idx), "{op:?} lost node {idx} to padding");
            }
            if !before.is_empty() {
                let mut genome = padded.clone();
                let mut rng = SmallRng::seed_from_u64(seed);
                let mut targets = selector();
                prop_assert!(
                    GraphMutator::apply(&mut genome, op, &mut targets, &mut rng, &config).is_ok(),
                    "{op:?} skipped on the padded genome"
                );
            }
        }
        for &op in &VmOperator::ALL {
            let before = VmMutator::applicable_indices(&base, op);
            let after = VmMutator::applicable_indices(&padded, op);
            for idx in &before {
                prop_assert!(after.contains(idx), "{op:?} lost node {idx} to padding");
            }
            if !before.is_empty() {
                let mut genome = padded.clone();
                let mut rng = SmallRng::seed_from_u64(seed);
                let mut targets = selector();
                prop_assert!(
                    VmMutator::apply(&mut genome, op, &mut targets, &mut rng, &config).is_ok(),
                    "{op:?} skipped on the padded genome"
                );
            }
        }
    }
}

// ─── Invariant 4: growth still reaches silent tissue ────────────────────────

/// A blank Graph module keeps its growth and connection reach: both add
/// operators target it, and a copy operator does once it has a compute node.
#[test]
fn growth_operators_reach_a_blank_graph_module() {
    let config = MutationConfig::default();
    let mut genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![blank_graph_node(0, &config)],
    };
    for op in [
        GraphOperator::AddInternalGraphNode,
        GraphOperator::AddGraphEdge,
    ] {
        assert_eq!(
            GraphMutator::applicable_indices(&genome, op, &config),
            vec![0],
            "{op:?} must reach a blank Graph module"
        );
    }
    assert!(
        GraphMutator::applicable_indices(&genome, GraphOperator::CopyInternalNode, &config)
            .is_empty(),
        "CopyInternalNode has nothing to copy on a blank module"
    );

    // Grow one compute node the way the engine would, then the copy
    // operators reach the module too.
    let mut rng = SmallRng::seed_from_u64(7);
    GraphMutator::apply(
        &mut genome,
        GraphOperator::AddInternalGraphNode,
        &mut selector(),
        &mut rng,
        &config,
    )
    .expect("growth applies to a blank module");
    assert_eq!(
        GraphMutator::applicable_indices(&genome, GraphOperator::CopyInternalNode, &config),
        vec![0],
        "CopyInternalNode must reach a module that has a compute node"
    );
}

/// A dormant VM module keeps its insert and copy reach; the input-reading
/// motifs wait for an input reference, which `InputRefOperator::Add`
/// supplies and which always has a target.
#[test]
fn growth_operators_reach_a_dormant_vm_module() {
    let config = MutationConfig::default();
    let mut genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![blank_vm_node(0)],
    };
    for op in [
        VmOperator::VmInstructionMutation,
        VmOperator::VmInsertLoadCompareMotif,
        VmOperator::VmCopyInstructionBlock,
        VmOperator::VmCopyInstructionBlockRemapped,
    ] {
        assert_eq!(
            VmMutator::applicable_indices(&genome, op),
            vec![0],
            "{op:?} must reach a dormant VM module"
        );
    }
    for op in [
        VmOperator::VmInsertReadStoreMotif,
        VmOperator::VmInsertReadBidMotif,
    ] {
        assert!(
            VmMutator::applicable_indices(&genome, op).is_empty(),
            "{op:?} has no input reference to read on a dormant module"
        );
    }

    let mut rng = SmallRng::seed_from_u64(11);
    InputRefMutator::apply(
        &mut genome,
        InputRefOperator::Add,
        &mut selector(),
        &mut rng,
        &config,
    )
    .expect("InputRef Add always has a target");
    for op in [
        VmOperator::VmInsertReadStoreMotif,
        VmOperator::VmInsertReadBidMotif,
    ] {
        assert_eq!(
            VmMutator::applicable_indices(&genome, op),
            vec![0],
            "{op:?} must reach a module once it has an input reference"
        );
    }
}
