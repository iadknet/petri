use super::*;
use crate::config::{MutationConfig, OrdinaryFoodTypeId, RuntimeConfig};
use crate::contracts::{NodeId, WorldInputKey};
use crate::creature::founder::v3alpha1_founder_genome;
use crate::creature::genome::cgp::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource, OutputSink,
    OutputSinkKind,
};
use crate::creature::genome::vote::{VoteKind, VoteSink};
use crate::creature::genome::{
    BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
};
use crate::creature::parseability::ParseabilityGate;
use crate::mutation::applicability_tests::{genome as generated_genome, module_size};
use crate::mutation::sampling::{random_input_reference, random_input_reference_for_food_types};
use crate::neighborhood::{Battery, Signature};
use crate::runtime::OUTPUT_SLOT_COUNT;
use proptest::prelude::*;
use rand::rngs::SmallRng;
use rand::SeedableRng;

fn rng(seed: u64) -> SmallRng {
    SmallRng::seed_from_u64(seed)
}

fn default_config() -> MutationConfig {
    MutationConfig::default()
}

fn single_node_genome_with_input_ref(input_ref: InputReference) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![input_ref],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![VmInstruction::Halt],
            }),
            targets: vec![],
        }],
    }
}

#[test]
fn add_input_ref_increases_count() {
    let mut genome = v3alpha1_founder_genome();
    let before: usize = genome.nodes.iter().map(|n| n.input_refs.len()).sum();
    let mut r = rng(0);
    InputRefMutator::apply(
        &mut genome,
        InputRefOperator::Add,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
        &default_config(),
    )
    .unwrap();
    let after: usize = genome.nodes.iter().map(|n| n.input_refs.len()).sum();
    assert_eq!(after, before + 1);
}

fn battery_signature(genome: &CreatureGenome) -> Signature {
    Battery::generate(1).signature(genome, &RuntimeConfig::default(), 0.0)
}

/// Every consumer on the node, in container order, resolved through the
/// node's table: `None` for a dangling index. A prune must leave this
/// sequence identical.
fn resolved_consumers(node: &NodeGenome) -> Vec<Option<InputReference>> {
    let resolve = |ref_idx: u16| node.input_refs.get(usize::from(ref_idx)).cloned();
    match &node.backend_def {
        BackendDef::Graph(def) => def
            .edges()
            .filter_map(|edge| match edge.source {
                GraphSource::InputLeaf { ref_idx, .. } => Some(resolve(ref_idx)),
                _ => None,
            })
            .collect(),
        BackendDef::Vm(vm) => vm
            .program
            .iter()
            .filter_map(|instruction| match instruction {
                VmInstruction::ReadInput { ref_idx, .. } => Some(resolve(*ref_idx)),
                _ => None,
            })
            .collect(),
    }
}

fn consumer_count(node: &NodeGenome) -> usize {
    resolved_consumers(node).len()
}

fn noop_count(node: &NodeGenome) -> usize {
    match &node.backend_def {
        BackendDef::Vm(vm) => vm
            .program
            .iter()
            .filter(|instruction| matches!(instruction, VmInstruction::Noop))
            .count(),
        BackendDef::Graph(_) => 0,
    }
}

// ─── Kind partition (T11.F22) ───────────────────────────────────────────────

#[test]
fn swap_alternatives_follow_the_kind_table() {
    let config = default_config();
    let food_here =
        |t: u16| InputReference::World(WorldInputKey::food_here(OrdinaryFoodTypeId::new(t)));
    // Typed food is swappable only with more than one food type.
    assert!(swap_alternatives(&food_here(0), &config, 1).is_empty());
    assert_eq!(
        swap_alternatives(&food_here(0), &config, 2),
        vec![food_here(1)]
    );
    assert_eq!(
        swap_alternatives(
            &InputReference::World(WorldInputKey::neighbor_food_ring(OrdinaryFoodTypeId::new(
                1
            ))),
            &config,
            3
        ),
        vec![
            InputReference::World(WorldInputKey::neighbor_food_ring(OrdinaryFoodTypeId::new(
                0
            ))),
            InputReference::World(WorldInputKey::neighbor_food_ring(OrdinaryFoodTypeId::new(
                2
            ))),
        ]
    );
    // The four introspection scalars form one kind (T19.F05 adds
    // `HopsThisTick`).
    let energy = InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent);
    let others = swap_alternatives(&energy, &config, 1);
    assert_eq!(others.len(), 3);
    assert!(others.contains(&InputReference::DynamicIntrospection(
        DynamicIntrospectionKey::HopsThisTick
    )));
    assert!(!others.contains(&energy));
    assert!(others
        .iter()
        .all(|r| input_ref_kind(r, &config) == input_ref_kind(&energy, &config)));
    // Upstream slots form one kind.
    let upstream = swap_alternatives(&InputReference::UpstreamSlot(2), &config, 1);
    assert_eq!(upstream.len(), OUTPUT_SLOT_COUNT - 1);
    assert!(!upstream.contains(&InputReference::UpstreamSlot(2)));
    // Single-member kinds are never swappable.
    for lone in [
        InputReference::World(WorldInputKey::NeighborBarrierRing),
        InputReference::World(WorldInputKey::AreaBarrierSummary),
        InputReference::World(WorldInputKey::NeighborOccupiedRing),
        InputReference::World(WorldInputKey::AreaOccupancySummary),
        InputReference::World(WorldInputKey::NearbyCreatureCore),
        InputReference::World(WorldInputKey::NearbyCreatureVitals),
        InputReference::World(WorldInputKey::NearbyCreatureIdentity),
        InputReference::ActionQueue,
        InputReference::CommitCounts,
        InputReference::PreviousOutcome,
    ] {
        assert!(
            swap_alternatives(&lone, &config, 4).is_empty(),
            "{lone:?} must not be swappable"
        );
    }
}

/// Every reference in the sampling pool has the kind the spec table gives
/// it: a swap can never cross a class or a width.
#[test]
fn kind_is_class_and_width() {
    let config = default_config();
    let ring = InputReference::World(WorldInputKey::neighbor_food_ring(
        OrdinaryFoodTypeId::default(),
    ));
    let summary = InputReference::World(WorldInputKey::area_food_summary(
        OrdinaryFoodTypeId::default(),
    ));
    assert_eq!(
        input_ref_kind(&ring, &config),
        InputRefKind {
            class: MeshReadClass::Food,
            width: 8
        }
    );
    assert_eq!(
        input_ref_kind(&summary, &config),
        InputRefKind {
            class: MeshReadClass::Food,
            width: 7
        }
    );
    assert_ne!(
        input_ref_kind(&ring, &config),
        input_ref_kind(&summary, &config)
    );
    assert_eq!(
        input_ref_kind(&InputReference::ActionQueue, &config),
        InputRefKind {
            class: MeshReadClass::ActionQueue,
            width: config.action_queue_cap as u16 * 3
        }
    );
}

// ─── Founder fixture (T11.F22 Verification) ─────────────────────────────────

/// `Swap` applies on both founder nodes: node 0 carries two introspection
/// scalars, node 1 six upstream slots.
#[test]
fn swap_applies_on_both_founder_nodes() {
    let genome = v3alpha1_founder_genome();
    assert_eq!(
        InputRefMutator::applicable_indices(&genome, InputRefOperator::Swap, &default_config(), 1),
        vec![0, 1]
    );
}

/// `Prune` applies to founder node 0 only (`NeighborOccupiedRing` at ref 4
/// has no consumer; node 1 reads all six slots), and the pruned genome's
/// battery signature is identical.
#[test]
fn prune_applies_to_founder_node_zero_only_and_is_silent() {
    let genome = v3alpha1_founder_genome();
    let config = default_config();
    assert_eq!(
        InputRefMutator::applicable_indices(&genome, InputRefOperator::Prune, &config, 1),
        vec![0]
    );
    assert_eq!(prunable_indices(&genome.nodes[0]), vec![4]);
    assert_eq!(
        genome.nodes[0].input_refs[4],
        InputReference::World(WorldInputKey::NeighborOccupiedRing)
    );
    assert!(prunable_indices(&genome.nodes[1]).is_empty());

    let base = battery_signature(&genome);
    let mut pruned = genome.clone();
    let mut r = rng(7);
    InputRefMutator::apply(
        &mut pruned,
        InputRefOperator::Prune,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
        &config,
    )
    .unwrap();
    assert_eq!(pruned.nodes[0].input_refs.len(), 4);
    assert_eq!(pruned.nodes[0].backend_def, genome.nodes[0].backend_def);
    assert_eq!(pruned.nodes[1], genome.nodes[1]);
    assert_eq!(battery_signature(&pruned), base);
}

/// The VM half of the fixture: an unread entry added to the founder's VM
/// node is the only prunable one; pruning it leaves the program and the
/// battery signature identical.
#[test]
fn prune_on_the_founder_vm_node_is_silent() {
    let config = default_config();
    let mut genome = crate::creature::founder::vm_decision_founder_genome();
    genome.nodes[1].input_refs.push(InputReference::ActionQueue);
    assert_eq!(prunable_indices(&genome.nodes[1]), vec![7]);
    let base = battery_signature(&genome);

    let mut pruned = genome.clone();
    let mut r = rng(11);
    InputRefMutator::apply_to_node(&mut pruned, InputRefOperator::Prune, 1, &mut r, &config, 1)
        .unwrap();
    assert_eq!(
        pruned.nodes[1].input_refs,
        crate::creature::founder::vm_decision_founder_genome().nodes[1].input_refs
    );
    assert_eq!(pruned.nodes[1].backend_def, genome.nodes[1].backend_def);
    assert_eq!(battery_signature(&pruned), base);
}

#[test]
fn prune_input_ref_decreases_count() {
    let mut genome = v3alpha1_founder_genome();
    let before: usize = genome.nodes.iter().map(|n| n.input_refs.len()).sum();
    assert!(before > 0, "founder must have input_refs");
    let mut r = rng(0);
    InputRefMutator::apply(
        &mut genome,
        InputRefOperator::Prune,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
        &default_config(),
    )
    .unwrap();
    let after: usize = genome.nodes.iter().map(|n| n.input_refs.len()).sum();
    assert_eq!(after, before - 1);
}

#[test]
fn prune_input_ref_on_empty_returns_no_applicable_target() {
    let mut genome = v3alpha1_founder_genome();
    for node in &mut genome.nodes {
        node.input_refs.clear();
    }
    let mut r = rng(0);
    let result = InputRefMutator::apply(
        &mut genome,
        InputRefOperator::Prune,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
        &default_config(),
    );
    assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
}

#[test]
fn swap_input_ref_changes_value() {
    let genome = v3alpha1_founder_genome();
    let original_refs: Vec<InputReference> = genome
        .nodes
        .iter()
        .flat_map(|n| n.input_refs.iter().cloned())
        .collect();
    let mut changed = false;
    for seed in 0u64..100 {
        let mut g = genome.clone();
        let mut r = rng(seed);
        if InputRefMutator::apply(
            &mut g,
            InputRefOperator::Swap,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
            &default_config(),
        )
        .is_ok()
        {
            let new_refs: Vec<InputReference> = g
                .nodes
                .iter()
                .flat_map(|n| n.input_refs.iter().cloned())
                .collect();
            if new_refs != original_refs {
                changed = true;
                break;
            }
        }
    }
    assert!(changed, "swap must change at least one input ref");
}

/// A within-kind swap keeps the width, so every edge on every container
/// survives: the T11.F22 replacement for the pre-feature clamp test, whose
/// premise (a swap could shrink the width) no longer holds.
#[test]
fn swap_on_the_graph_backend_keeps_every_edge() {
    let config = default_config();
    let leaf = |sub_idx: u16| GraphEdge {
        source: GraphSource::InputLeaf {
            ref_idx: 0,
            sub_idx,
        },
        weight: 1.0,
    };
    let def = CgpGraphBackendDef {
        birth_weights: None,
        compute_nodes: vec![ComputeNode {
            kind: ComputeNodeKind::Add,
            inputs: vec![leaf(3), leaf(0)],
            plasticity: None,
        }],
        output_sinks: vec![
            OutputSink {
                kind: OutputSinkKind::CustomOutput(0),
                inputs: vec![leaf(2)],
            },
            OutputSink {
                kind: OutputSinkKind::ActionVote(VoteSink::Eat),
                inputs: vec![leaf(4)],
            },
            OutputSink {
                kind: OutputSinkKind::ActionParam(VoteKind::Eat, 0),
                inputs: vec![leaf(0)],
            },
            OutputSink {
                kind: OutputSinkKind::ActionVote(VoteSink::Terminate),
                inputs: vec![leaf(7)],
            },
        ],
    };
    let base_genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![InputReference::World(WorldInputKey::neighbor_food_ring(
                OrdinaryFoodTypeId::new(0),
            ))],
            backend_def: BackendDef::Graph(def),
            targets: vec![],
        }],
    };

    for seed in 0u64..64 {
        let mut genome = base_genome.clone();
        let mut r = rng(seed);
        InputRefMutator::apply_with_food_type_count(
            &mut genome,
            InputRefOperator::Swap,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
            &config,
            2,
        )
        .unwrap();
        assert_eq!(
            genome.nodes[0].input_refs,
            vec![InputReference::World(WorldInputKey::neighbor_food_ring(
                OrdinaryFoodTypeId::new(1)
            ))],
            "the only other member of (Food, 8) at two food types"
        );
        assert_eq!(
            genome.nodes[0].backend_def,
            base_genome.nodes[0].backend_def
        );
    }
}

/// T19.F05 invariant 5: the two vote vectors swap with each other only;
/// the decision compounds never share a kind with the introspection ones.
#[test]
fn decision_state_kinds_follow_the_swap_table() {
    let config = default_config();
    let kind = |reference: &InputReference| input_ref_kind(reference, &config);
    let hops = InputReference::DynamicIntrospection(DynamicIntrospectionKey::HopsThisTick);
    for (reference, class, width) in [
        (InputReference::ActionVotes, MeshReadClass::Decision, 27),
        (
            InputReference::PreviousPassVotes,
            MeshReadClass::Decision,
            27,
        ),
        (InputReference::CommitCounts, MeshReadClass::Decision, 4),
        (hops.clone(), MeshReadClass::Introspection, 1),
        (
            InputReference::PreviousOutcome,
            MeshReadClass::Introspection,
            4,
        ),
    ] {
        assert_eq!(
            kind(&reference),
            InputRefKind { class, width },
            "{reference:?}"
        );
    }
    assert_eq!(
        swap_alternatives(&InputReference::ActionVotes, &config, 3),
        vec![InputReference::PreviousPassVotes]
    );
    assert_eq!(
        swap_alternatives(&InputReference::PreviousPassVotes, &config, 3),
        vec![InputReference::ActionVotes]
    );
    let mut hops_partners = swap_alternatives(&hops, &config, 1);
    hops_partners.sort_by_key(|reference| format!("{reference:?}"));
    assert_eq!(
        hops_partners,
        vec![
            InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyConsumedThisTick),
            InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
            InputReference::StaticIntrospection(StaticIntrospectionKey::AgeTicks),
        ]
    );
}

// ─── Invariants 1 and 2 over generated genomes (T11.F22) ────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// Invariant 1: an applied `Swap` changes exactly one entry, to a
    /// different member of the same kind; the table length, every other
    /// entry, every edge and every instruction are unchanged.
    #[test]
    fn swap_changes_one_entry_within_its_kind(
        seed in any::<u64>(),
        sizes in prop::collection::vec(module_size(), 1..4),
        food_type_count in 1usize..4,
    ) {
        let config = default_config();
        let base = generated_genome(seed, &sizes, &config);
        let applicable = InputRefMutator::applicable_indices(
            &base, InputRefOperator::Swap, &config, food_type_count,
        );
        for node_idx in applicable {
            for apply_seed in 0u64..4 {
                let mut genome = base.clone();
                let mut r = rng(seed ^ apply_seed);
                InputRefMutator::apply_to_node(
                    &mut genome, InputRefOperator::Swap, node_idx, &mut r, &config, food_type_count,
                )
                .expect("an accepted node applies");
                for (idx, node) in genome.nodes.iter().enumerate() {
                    if idx != node_idx {
                        prop_assert_eq!(node, &base.nodes[idx], "an untargeted node changed");
                    }
                }
                let before = &base.nodes[node_idx];
                let after = &genome.nodes[node_idx];
                prop_assert_eq!(&after.backend_def, &before.backend_def, "a swap touched an edge or instruction");
                prop_assert_eq!(after.input_refs.len(), before.input_refs.len());
                let changed: Vec<usize> = (0..before.input_refs.len())
                    .filter(|&i| before.input_refs[i] != after.input_refs[i])
                    .collect();
                prop_assert_eq!(changed.len(), 1, "exactly one entry changes");
                let i = changed[0];
                prop_assert!(
                    swap_alternatives(&before.input_refs[i], &config, food_type_count)
                        .contains(&after.input_refs[i]),
                    "{:?} -> {:?} is not a within-kind step",
                    before.input_refs[i],
                    after.input_refs[i]
                );
                prop_assert_eq!(
                    input_ref_kind(&after.input_refs[i], &config),
                    input_ref_kind(&before.input_refs[i], &config)
                );
            }
        }
    }

    /// Invariant 2: an applied `Prune` deletes one entry no consumer
    /// addresses and renumbers; every consumer resolves to the same reference
    /// before and after, and no edge or instruction is removed or rewritten.
    #[test]
    fn prune_deletes_an_unreferenced_entry_and_keeps_every_consumer(
        seed in any::<u64>(),
        sizes in prop::collection::vec(module_size(), 1..4),
    ) {
        let config = default_config();
        let base = generated_genome(seed, &sizes, &config);
        let applicable = InputRefMutator::applicable_indices(&base, InputRefOperator::Prune, &config, 1);
        for node_idx in applicable {
            for apply_seed in 0u64..4 {
                let mut genome = base.clone();
                let mut r = rng(seed ^ apply_seed);
                InputRefMutator::apply_to_node(
                    &mut genome, InputRefOperator::Prune, node_idx, &mut r, &config, 1,
                )
                .expect("an accepted node applies");
                for (idx, node) in genome.nodes.iter().enumerate() {
                    if idx != node_idx {
                        prop_assert_eq!(node, &base.nodes[idx], "an untargeted node changed");
                    }
                }
                let before = &base.nodes[node_idx];
                let after = &genome.nodes[node_idx];
                prop_assert_eq!(after.input_refs.len() + 1, before.input_refs.len());
                prop_assert_eq!(consumer_count(after), consumer_count(before), "a consumer was removed");
                prop_assert_eq!(noop_count(after), noop_count(before), "a ReadInput became Noop");
                prop_assert_eq!(resolved_consumers(after), resolved_consumers(before));
                // Duplicate entries make the deleted index ambiguous by
                // position, so ask whether deleting any unreferenced entry
                // reproduces the table instead of inferring one index.
                let deleted_an_unreferenced_entry = prunable_indices(before).into_iter().any(|i| {
                    let mut expected = before.input_refs.clone();
                    expected.remove(i);
                    expected == after.input_refs
                });
                prop_assert!(
                    deleted_an_unreferenced_entry,
                    "the deleted entry had a consumer: {:?} -> {:?}",
                    before.input_refs,
                    after.input_refs
                );
            }
        }
    }
}

#[test]
fn random_input_reference_covers_all_categories() {
    use std::collections::HashSet;
    let mut categories: HashSet<String> = HashSet::new();
    for seed in 0u64..1000 {
        let mut r = rng(seed);
        let ir = random_input_reference(&mut r);
        let cat = match ir {
            InputReference::World(WorldInputKey::FoodHere { .. }) => "FoodHere".to_string(),
            InputReference::World(WorldInputKey::NeighborFoodRing { .. }) => {
                "NeighborFoodRing".to_string()
            }
            InputReference::World(WorldInputKey::NeighborBarrierRing) => {
                "NeighborBarrierRing".to_string()
            }
            InputReference::World(WorldInputKey::NeighborOccupiedRing) => {
                "NeighborOccupiedRing".to_string()
            }
            InputReference::World(WorldInputKey::AreaFoodSummary { .. }) => {
                "AreaFoodSummary".to_string()
            }
            InputReference::World(WorldInputKey::AreaBarrierSummary) => {
                "AreaBarrierSummary".to_string()
            }
            InputReference::World(WorldInputKey::AreaOccupancySummary) => {
                "AreaOccupancySummary".to_string()
            }
            InputReference::World(WorldInputKey::NearbyCreatureCore) => {
                "NearbyCreatureCore".to_string()
            }
            InputReference::World(WorldInputKey::NearbyCreatureVitals) => {
                "NearbyCreatureVitals".to_string()
            }
            InputReference::World(WorldInputKey::NearbyCreatureIdentity) => {
                "NearbyCreatureIdentity".to_string()
            }
            InputReference::StaticIntrospection(_) => "StaticIntrospection".to_string(),
            InputReference::DynamicIntrospection(_) => "DynamicIntrospection".to_string(),
            InputReference::UpstreamSlot(_) => "UpstreamSlot".to_string(),
            InputReference::ActionQueue => "ActionQueue".to_string(),
            InputReference::ActionVotes => "ActionVotes".to_string(),
            InputReference::PreviousPassVotes => "PreviousPassVotes".to_string(),
            InputReference::CommitCounts => "CommitCounts".to_string(),
            InputReference::PreviousOutcome => "PreviousOutcome".to_string(),
        };
        categories.insert(cat);
    }
    // 18 categories: 8 original + 6 extended perception families + the four
    // decision-state compounds (T19.F05; `HopsThisTick` is a dynamic key).
    assert_eq!(
        categories.len(),
        18,
        "all 18 input reference categories must be reachable; got {:?}",
        categories
    );
}

#[test]
fn random_input_reference_upstream_slot_bounded() {
    for seed in 0u64..20_000 {
        let mut r = rng(seed);
        if let InputReference::UpstreamSlot(slot) = random_input_reference(&mut r) {
            assert!(
                slot < OUTPUT_SLOT_COUNT,
                "upstream slot must be bounded < {OUTPUT_SLOT_COUNT}, got {slot}"
            );
        }
    }
}

#[test]
fn raw_field_mutation_upstream_slot_bounded() {
    for seed in 0u64..512 {
        let mut genome = single_node_genome_with_input_ref(InputReference::UpstreamSlot(0));
        let mut r = rng(seed);
        InputRefMutator::apply(
            &mut genome,
            InputRefOperator::RawFieldMutation,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
            &default_config(),
        )
        .unwrap();
        if let InputReference::UpstreamSlot(slot) = genome.nodes[0].input_refs[0] {
            assert!(
                slot < OUTPUT_SLOT_COUNT,
                "mutated upstream slot must be bounded < {OUTPUT_SLOT_COUNT}, got {slot}"
            );
        }
    }
}

#[test]
fn input_ref_after_mutation_passes_parseability_gate() {
    let operators = [
        InputRefOperator::Add,
        InputRefOperator::Prune,
        InputRefOperator::Swap,
        InputRefOperator::RawFieldMutation,
    ];
    for (i, &op) in operators.iter().enumerate() {
        let mut genome = v3alpha1_founder_genome();
        let mut r = rng(i as u64 + 400);
        let _ = InputRefMutator::apply(
            &mut genome,
            op,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
            &default_config(),
        );
        assert!(
            ParseabilityGate::validate(&genome).is_ok(),
            "parseability failed after {:?}",
            op
        );
    }
}

#[test]
fn input_ref_weighted_random_favors_refinement() {
    let mut counts = std::collections::HashMap::new();
    let mut r = rng(42);
    for _ in 0..10_000 {
        let op = InputRefOperator::random(&mut r);
        *counts.entry(op).or_insert(0u32) += 1;
    }
    let swap = counts.get(&InputRefOperator::Swap).copied().unwrap_or(0);
    let add = counts.get(&InputRefOperator::Add).copied().unwrap_or(0);
    assert!(
        swap > add + (add / 2),
        "Swap (weight 4) must appear >1.5x Add (weight 2); got {} vs {}",
        swap,
        add,
    );
}

#[test]
fn input_ref_operator_weights_are_positive() {
    let all = InputRefOperator::ALL;
    assert_eq!(
        all.len(),
        4,
        "ALL must cover every InputRefOperator variant"
    );
    for &op in &all {
        assert!(op.weight() > 0, "weight must be positive for {:?}", op);
    }
}

#[test]
fn complexity_effect_consistent_with_types() {
    use crate::mutation::types::ComplexityEffect;
    for &op in &InputRefOperator::ALL {
        let effect = op.complexity_effect();
        assert!(
            matches!(
                effect,
                ComplexityEffect::Increasing
                    | ComplexityEffect::Decreasing
                    | ComplexityEffect::Neutral
            ),
            "complexity_effect must return valid effect for {:?}",
            op
        );
    }
}

// ── InputRef::Add wires nothing (T11.F03) ───────────────────────────────────

fn graph_node_genome_zero_refs() -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Graph(CgpGraphBackendDef {
                birth_weights: None,
                compute_nodes: vec![ComputeNode {
                    kind: ComputeNodeKind::Add,
                    inputs: vec![],
                    plasticity: None,
                }],
                output_sinks: vec![
                    OutputSink {
                        kind: OutputSinkKind::CustomOutput(0),
                        inputs: vec![],
                    },
                    OutputSink {
                        kind: OutputSinkKind::ActionVote(VoteSink::Eat),
                        inputs: vec![],
                    },
                ],
            }),
            targets: vec![],
        }],
    }
}

fn count_input_leaf_edges_for_ref(genome: &CreatureGenome, node_idx: usize, ref_idx: u16) -> usize {
    let BackendDef::Graph(ref def) = genome.nodes[node_idx].backend_def else {
        return 0;
    };
    let mut count = 0;
    for node in &def.compute_nodes {
        count += node
            .inputs
            .iter()
            .filter(
                |e| matches!(e.source, GraphSource::InputLeaf { ref_idx: r, .. } if r == ref_idx),
            )
            .count();
    }
    for sink in &def.output_sinks {
        count += sink
            .inputs
            .iter()
            .filter(
                |e| matches!(e.source, GraphSource::InputLeaf { ref_idx: r, .. } if r == ref_idx),
            )
            .count();
    }
    count
}

/// `InputRef::Add` is a growth operator: it must be neutral at fire time
/// (`docs/reference/v3-mutation-spec.md`'s node-type contract), so it wires
/// the new reference into nothing on the graph backend, no matter how many
/// sub-values the sampled reference has.
#[test]
fn add_input_ref_to_graph_node_wires_nothing() {
    for seed in 0u64..200 {
        let mut genome = graph_node_genome_zero_refs();
        let mut r = rng(seed);
        InputRefMutator::apply(
            &mut genome,
            InputRefOperator::Add,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
            &default_config(),
        )
        .unwrap();
        assert_eq!(genome.nodes[0].input_refs.len(), 1, "seed {seed}");
        assert_eq!(
            count_input_leaf_edges_for_ref(&genome, 0, 0),
            0,
            "seed {seed}: InputRef::Add must not wire the new reference into any edge"
        );
    }
}

#[test]
fn add_input_ref_to_graph_node_leaves_existing_edges_untouched() {
    let mut genome = graph_node_genome_zero_refs();
    genome.nodes[0]
        .input_refs
        .push(InputReference::World(WorldInputKey::food_here(
            OrdinaryFoodTypeId::default(),
        )));
    let before = genome.clone();
    let mut r = rng(99);
    InputRefMutator::apply(
        &mut genome,
        InputRefOperator::Add,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
        &default_config(),
    )
    .unwrap();
    assert_eq!(genome.nodes[0].input_refs.len(), 2);
    let BackendDef::Graph(ref before_def) = before.nodes[0].backend_def else {
        panic!("expected graph backend");
    };
    let BackendDef::Graph(ref after_def) = genome.nodes[0].backend_def else {
        panic!("expected graph backend");
    };
    assert_eq!(
        before_def, after_def,
        "InputRef::Add must not touch any existing edge"
    );
}

/// The VM `ReadInput` auto-insertion is removed: `InputRef::Add` on a VM
/// node leaves the program byte-identical.
#[test]
fn add_input_ref_to_vm_node_leaves_program_unchanged() {
    let mut genome = single_node_genome_with_input_ref(InputReference::World(
        WorldInputKey::food_here(OrdinaryFoodTypeId::default()),
    ));
    genome.nodes[0].input_refs.clear(); // start with 0 refs
    let program_before = match &genome.nodes[0].backend_def {
        BackendDef::Vm(vm) => vm.program.clone(),
        _ => panic!("expected VM backend"),
    };
    let mut r = rng(42);
    InputRefMutator::apply(
        &mut genome,
        InputRefOperator::Add,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
        &default_config(),
    )
    .unwrap();
    assert_eq!(genome.nodes[0].input_refs.len(), 1);
    let BackendDef::Vm(ref vm) = genome.nodes[0].backend_def else {
        panic!("expected VM backend");
    };
    assert_eq!(
        vm.program, program_before,
        "InputRef::Add must not insert or otherwise change any VM instruction"
    );
}

// Legacy flat-graph lifecycle tests were removed during the CGP migration.
// The remaining Graph coverage lives in genome/mod.rs tests.

#[test]
fn reindex_vm_decrements_and_noops_orphans() {
    let mut genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![
                InputReference::World(WorldInputKey::food_here(OrdinaryFoodTypeId::default())),
                InputReference::UpstreamSlot(0),
                InputReference::UpstreamSlot(1),
            ],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 2,
                constants: vec![],
                program: vec![
                    VmInstruction::ReadInput {
                        dst: 0,
                        ref_idx: 0,
                        sub_idx: 0,
                    },
                    VmInstruction::ReadInput {
                        dst: 1,
                        ref_idx: 1,
                        sub_idx: 0,
                    },
                    VmInstruction::ReadInput {
                        dst: 0,
                        ref_idx: 2,
                        sub_idx: 5,
                    },
                    VmInstruction::Halt,
                ],
            }),
            targets: vec![],
        }],
    };
    // Remove input ref at index 1
    genome.nodes[0]
        .backend_def
        .reindex_input_refs_after_removal(1);
    if let BackendDef::Vm(ref vm) = genome.nodes[0].backend_def {
        // ref_idx 0 < 1 -> unchanged
        assert!(matches!(
            vm.program[0],
            VmInstruction::ReadInput {
                ref_idx: 0,
                sub_idx: 0,
                ..
            }
        ));
        // ref_idx 1 == removed -> converted to Noop (not u16::MAX zombie)
        assert!(
            matches!(vm.program[1], VmInstruction::Noop),
            "orphaned ReadInput must become Noop, got {:?}",
            vm.program[1]
        );
        // ref_idx 2 > 1 -> decremented to 1
        assert!(matches!(
            vm.program[2],
            VmInstruction::ReadInput {
                ref_idx: 1,
                sub_idx: 5,
                ..
            }
        ));
    } else {
        panic!("expected VM backend");
    }
}

#[test]
fn reindex_noop_on_empty_vm_program() {
    let mut genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![InputReference::World(WorldInputKey::food_here(
                OrdinaryFoodTypeId::default(),
            ))],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![], // empty program
            }),
            targets: vec![],
        }],
    };
    genome.nodes[0]
        .backend_def
        .reindex_input_refs_after_removal(0); // should not panic
}

#[test]
fn extended_perception_families_reachable_in_pool() {
    let mut found = [false; 6]; // Food, Barrier, Occupancy, Core, Vitals, Identity
    for seed in 0u64..5000 {
        let mut r = rng(seed);
        match random_input_reference(&mut r) {
            InputReference::World(WorldInputKey::AreaFoodSummary { .. }) => found[0] = true,
            InputReference::World(WorldInputKey::AreaBarrierSummary) => found[1] = true,
            InputReference::World(WorldInputKey::AreaOccupancySummary) => found[2] = true,
            InputReference::World(WorldInputKey::NearbyCreatureCore) => found[3] = true,
            InputReference::World(WorldInputKey::NearbyCreatureVitals) => found[4] = true,
            InputReference::World(WorldInputKey::NearbyCreatureIdentity) => found[5] = true,
            _ => {}
        }
        if found.iter().all(|&f| f) {
            break;
        }
    }
    let names = [
        "AreaFoodSummary",
        "AreaBarrierSummary",
        "AreaOccupancySummary",
        "NearbyCreatureCore",
        "NearbyCreatureVitals",
        "NearbyCreatureIdentity",
    ];
    for (i, &f) in found.iter().enumerate() {
        assert!(
            f,
            "{} must be reachable from random_input_reference",
            names[i]
        );
    }
}

#[test]
fn action_queue_appears_in_random_input_reference_pool() {
    let mut found_aq = false;
    for seed in 0u64..500 {
        let mut r = rng(seed);
        if random_input_reference(&mut r) == InputReference::ActionQueue {
            found_aq = true;
            break;
        }
    }
    assert!(
        found_aq,
        "ActionQueue must be reachable from random_input_reference"
    );
}

fn any_non_default_food_type(input_refs: &[InputReference]) -> bool {
    input_refs.iter().any(|input_ref| match input_ref {
        InputReference::World(WorldInputKey::FoodHere { type_idx })
        | InputReference::World(WorldInputKey::NeighborFoodRing { type_idx })
        | InputReference::World(WorldInputKey::AreaFoodSummary { type_idx }) => {
            *type_idx != OrdinaryFoodTypeId::default()
        }
        _ => false,
    })
}

#[test]
fn random_input_reference_for_food_types_can_sample_non_default_food_type() {
    let mut r = rng(0xC0FFEE);
    let saw_non_default = (0..2048)
        .map(|_| random_input_reference_for_food_types(&mut r, 3))
        .any(|input_ref| any_non_default_food_type(&[input_ref]));

    assert!(
        saw_non_default,
        "typed sampler should produce food refs with non-default type_idx when 3 food types are available"
    );
}

#[test]
fn input_ref_add_can_introduce_non_default_food_type() {
    let mut r = rng(0xA11D);
    let mut saw_non_default = false;
    for _ in 0..1024 {
        let mut genome = v3alpha1_founder_genome();
        let _ = InputRefMutator::apply_with_food_type_count(
            &mut genome,
            InputRefOperator::Add,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
            &default_config(),
            3,
        );
        if genome
            .nodes
            .iter()
            .any(|node| any_non_default_food_type(&node.input_refs))
        {
            saw_non_default = true;
            break;
        }
    }

    assert!(
        saw_non_default,
        "InputRef.Add should be able to introduce food sensors for non-default food types"
    );
}

#[test]
fn input_ref_swap_can_introduce_non_default_food_type() {
    let mut r = rng(0x5A9);
    let mut saw_non_default = false;
    for _ in 0..1024 {
        let mut genome = v3alpha1_founder_genome();
        let _ = InputRefMutator::apply_with_food_type_count(
            &mut genome,
            InputRefOperator::Swap,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
            &default_config(),
            4,
        );
        if genome
            .nodes
            .iter()
            .any(|node| any_non_default_food_type(&node.input_refs))
        {
            saw_non_default = true;
            break;
        }
    }

    assert!(
        saw_non_default,
        "InputRef.Swap should be able to produce food refs with non-default type_idx"
    );
}

#[test]
fn input_ref_raw_field_mutation_can_mutate_food_type_idx() {
    let mut genome = single_node_genome_with_input_ref(InputReference::World(
        WorldInputKey::food_here(OrdinaryFoodTypeId::default()),
    ));
    let mut r = rng(0xFACE);
    let result = InputRefMutator::apply_with_food_type_count(
        &mut genome,
        InputRefOperator::RawFieldMutation,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
        &default_config(),
        3,
    );

    assert!(
        result.is_ok() && any_non_default_food_type(&genome.nodes[0].input_refs),
        "InputRef.RawFieldMutation should be able to mutate food sensor type_idx when multiple food types exist"
    );
}

#[test]
fn input_reference_universe_caps_food_types_at_the_full_u16_range() {
    // The cap is one past `u16::MAX`: every representable food type id,
    // including the last one, has its keys in the universe.
    let full_range = usize::from(u16::MAX) + 1;
    let last_food_type =
        InputReference::World(WorldInputKey::food_here(OrdinaryFoodTypeId::new(u16::MAX)));

    let at_cap = input_reference_universe(full_range);

    assert!(
        at_cap.contains(&last_food_type),
        "food type {} must be enumerated at {full_range} food types",
        u16::MAX
    );
    assert_eq!(
        input_reference_universe(full_range + 7),
        at_cap,
        "food type counts above the cap add nothing to the universe"
    );
}
