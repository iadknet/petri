use super::*;
use crate::config::{MutationConfig, OrdinaryFoodTypeId, RuntimeConfig};
use crate::contracts::{DynamicIntrospectionKey, NodeId, StaticIntrospectionKey, WorldInputKey};
use crate::creature::founder::v3alpha1_founder_genome;
use crate::creature::genome::cgp::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource, OutputSink,
    OutputSinkKind,
};
use crate::creature::genome::vote::{ActionParamField, VoteSink, VOTE_SINK_COUNT};
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
            .filter_map(|edge| match edge.source() {
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
    let food_here =
        |t: u16| InputReference::World(WorldInputKey::food_here(OrdinaryFoodTypeId::new(t)));
    // Typed food is swappable only with more than one food type.
    assert!(swap_alternatives(&food_here(0), 1).is_empty());
    assert_eq!(swap_alternatives(&food_here(0), 2), vec![food_here(1)]);
    assert_eq!(
        swap_alternatives(
            &InputReference::World(WorldInputKey::neighbor_food_ring(OrdinaryFoodTypeId::new(
                1
            ))),
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
    // The seven introspection scalars form one kind (T19.F05 adds
    // `HopsThisTick`, T27.F01 the three own-colour keys).
    let energy = InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent);
    let others = swap_alternatives(&energy, 1);
    assert_eq!(others.len(), 6);
    assert!(others.contains(&InputReference::DynamicIntrospection(
        DynamicIntrospectionKey::HopsThisTick
    )));
    assert!(!others.contains(&energy));
    assert!(others
        .iter()
        .all(|r| input_ref_kind(r) == input_ref_kind(&energy)));
    // Upstream slots form one kind.
    let upstream = swap_alternatives(&InputReference::UpstreamSlot(2), 1);
    assert_eq!(upstream.len(), OUTPUT_SLOT_COUNT - 1);
    assert!(!upstream.contains(&InputReference::UpstreamSlot(2)));
    // Single-member kinds are never swappable.
    for lone in [
        InputReference::World(WorldInputKey::NeighborBarrierRing),
        InputReference::World(WorldInputKey::AreaBarrierSummary),
        InputReference::World(WorldInputKey::NeighborOccupiedRing),
        InputReference::World(WorldInputKey::AreaOccupancySummary),
        InputReference::World(WorldInputKey::NearbyCreature),
        InputReference::ActionQueue,
        InputReference::CommitCounts,
        InputReference::PreviousOutcome,
    ] {
        assert!(
            swap_alternatives(&lone, 4).is_empty(),
            "{lone:?} must not be swappable"
        );
    }
}

/// Every reference in the sampling pool has the kind the spec table gives
/// it: a swap can never cross a class or a width.
#[test]
fn kind_is_class_and_width() {
    let ring = InputReference::World(WorldInputKey::neighbor_food_ring(
        OrdinaryFoodTypeId::default(),
    ));
    let summary = InputReference::World(WorldInputKey::area_food_summary(
        OrdinaryFoodTypeId::default(),
    ));
    assert_eq!(
        input_ref_kind(&ring),
        InputRefKind {
            class: MeshReadClass::Food,
            width: 8
        }
    );
    assert_eq!(
        input_ref_kind(&summary),
        InputRefKind {
            class: MeshReadClass::Food,
            width: 7
        }
    );
    assert_ne!(input_ref_kind(&ring), input_ref_kind(&summary));
    assert_eq!(
        input_ref_kind(&InputReference::ActionQueue),
        InputRefKind {
            class: MeshReadClass::ActionQueue,
            width: 12
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
        InputRefMutator::applicable_indices(&genome, InputRefOperator::Swap, 1),
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
        InputRefMutator::applicable_indices(&genome, InputRefOperator::Prune, 1),
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
    let mut genome = crate::creature::founder::vm_decision_founder_genome();
    genome.nodes[1].input_refs.push(InputReference::ActionQueue);
    assert_eq!(prunable_indices(&genome.nodes[1]), vec![7]);
    let base = battery_signature(&genome);

    let mut pruned = genome.clone();
    let mut r = rng(11);
    InputRefMutator::apply_to_node(&mut pruned, InputRefOperator::Prune, 1, &mut r, 1).unwrap();
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
    let leaf = |sub_idx: u16| {
        GraphEdge::new(
            GraphSource::InputLeaf {
                ref_idx: 0,
                sub_idx,
            },
            1.0,
        )
    };
    let def = CgpGraphBackendDef::new_with_fixed_outputs()
        .with_compute_nodes(vec![ComputeNode::new(
            ComputeNodeKind::Add,
            vec![leaf(3), leaf(0)],
        )])
        .with_output_sinks(vec![
            OutputSink::new(OutputSinkKind::CustomOutput(0), vec![leaf(2)]),
            OutputSink::new(OutputSinkKind::ActionVote(VoteSink::Eat), vec![leaf(4)]),
            OutputSink::new(
                OutputSinkKind::ActionParam(ActionParamField::EatFoodType),
                vec![leaf(0)],
            ),
            OutputSink::new(
                OutputSinkKind::ActionVote(VoteSink::Terminate),
                vec![leaf(7)],
            ),
        ]);
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
    let kind = |reference: &InputReference| input_ref_kind(reference);
    let hops = InputReference::DynamicIntrospection(DynamicIntrospectionKey::HopsThisTick);
    for (reference, class, width) in [
        (InputReference::ActionVotes, MeshReadClass::Decision, 30),
        (
            InputReference::PreviousPassVotes,
            MeshReadClass::Decision,
            30,
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
        swap_alternatives(&InputReference::ActionVotes, 3),
        vec![InputReference::PreviousPassVotes]
    );
    assert_eq!(
        swap_alternatives(&InputReference::PreviousPassVotes, 3),
        vec![InputReference::ActionVotes]
    );
    let mut hops_partners = swap_alternatives(&hops, 1);
    hops_partners.sort_by_key(|reference| format!("{reference:?}"));
    assert_eq!(
        hops_partners,
        vec![
            InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyConsumedThisTick),
            InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
            InputReference::StaticIntrospection(StaticIntrospectionKey::AgeTicks),
            InputReference::StaticIntrospection(StaticIntrospectionKey::OwnBlue),
            InputReference::StaticIntrospection(StaticIntrospectionKey::OwnGreen),
            InputReference::StaticIntrospection(StaticIntrospectionKey::OwnRed),
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
        let base = generated_genome(seed, &sizes);
        let applicable = InputRefMutator::applicable_indices(
            &base, InputRefOperator::Swap, food_type_count,
        );
        for node_idx in applicable {
            for apply_seed in 0u64..4 {
                let mut genome = base.clone();
                let mut r = rng(seed ^ apply_seed);
                InputRefMutator::apply_to_node(
                    &mut genome, InputRefOperator::Swap, node_idx, &mut r, food_type_count,
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
                    swap_alternatives(&before.input_refs[i], food_type_count)
                        .contains(&after.input_refs[i]),
                    "{:?} -> {:?} is not a within-kind step",
                    before.input_refs[i],
                    after.input_refs[i]
                );
                prop_assert_eq!(
                    input_ref_kind(&after.input_refs[i]),
                    input_ref_kind(&before.input_refs[i])
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
        let base = generated_genome(seed, &sizes);
        let applicable = InputRefMutator::applicable_indices(&base, InputRefOperator::Prune, 1);
        for node_idx in applicable {
            for apply_seed in 0u64..4 {
                let mut genome = base.clone();
                let mut r = rng(seed ^ apply_seed);
                InputRefMutator::apply_to_node(
                    &mut genome, InputRefOperator::Prune, node_idx, &mut r, 1,
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
            InputReference::World(WorldInputKey::NearbyCreature) => "NearbyCreature".to_string(),
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
    // 16 categories: 8 original + 4 extended perception families (one
    // `NearbyCreature`, T25.F05) + the four decision-state compounds
    // (T19.F05; `HopsThisTick` is a dynamic key).
    assert_eq!(
        categories.len(),
        16,
        "all 16 input reference categories must be reachable; got {:?}",
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

// ── InputRef::Add declares a born-connected input on Graph (T20.F16) ───────

/// One Graph node with the full fixed sink catalog and no references.
fn graph_node_genome_full_catalog() -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Graph(CgpGraphBackendDef::new_with_fixed_outputs()),
            targets: vec![],
        }],
    }
}

fn graph_def(node: &NodeGenome) -> &CgpGraphBackendDef {
    let BackendDef::Graph(def) = &node.backend_def else {
        panic!("expected graph backend");
    };
    def
}

/// Every edge on the node's `ActionVote` sinks as (sink, source, weight).
fn vote_edges(node: &NodeGenome) -> Vec<(VoteSink, GraphSource, f32)> {
    let BackendDef::Graph(def) = &node.backend_def else {
        return Vec::new();
    };
    def.output_sinks()
        .iter()
        .filter_map(|sink| match sink.kind {
            OutputSinkKind::ActionVote(vote) => Some((vote, sink)),
            _ => None,
        })
        .flat_map(|(vote, sink)| {
            sink.inputs
                .iter()
                .map(move |edge| (vote, edge.source(), edge.own_weight()))
        })
        .collect()
}

fn edge_count(node: &NodeGenome) -> usize {
    match &node.backend_def {
        BackendDef::Graph(def) => def.edges().count(),
        BackendDef::Vm(_) => 0,
    }
}

/// The vote edges `after` carries beyond `before`'s, as a multiset
/// difference; panics when an old vote edge is gone.
fn new_vote_edges(before: &NodeGenome, after: &NodeGenome) -> Vec<(VoteSink, GraphSource, f32)> {
    let mut new = vote_edges(after);
    for edge in vote_edges(before) {
        let pos = new
            .iter()
            .position(|candidate| *candidate == edge)
            .expect("every old vote edge kept");
        new.remove(pos);
    }
    new
}

fn apply_add_to_node(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut SmallRng,
    food_type_count: usize,
) {
    InputRefMutator::apply_to_node(
        genome,
        InputRefOperator::Add,
        node_idx,
        rng,
        food_type_count,
    )
    .expect("Add always applies");
}

/// `InputRef::Add` on a Graph node is a connection event: it appends one
/// reference and one edge from it into one `ActionVote` sink of the same
/// node, at a sub-value under the reference's width and a weight in [-1, 1].
#[test]
fn add_input_ref_to_graph_node_wires_one_vote_edge() {
    for seed in 0u64..200 {
        let base = graph_node_genome_full_catalog();
        let mut genome = base.clone();
        let mut r = rng(seed);
        InputRefMutator::apply(
            &mut genome,
            InputRefOperator::Add,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
            &default_config(),
        )
        .unwrap();
        let after = &genome.nodes[0];
        assert_eq!(after.input_refs.len(), 1, "seed {seed}");
        assert_eq!(edge_count(after), 1, "seed {seed}: exactly one edge");
        let new = new_vote_edges(&base.nodes[0], after);
        assert_eq!(new.len(), 1, "seed {seed}: the edge is on a vote sink");
        let (_, source, weight) = new[0];
        let GraphSource::InputLeaf { ref_idx, sub_idx } = source else {
            panic!("seed {seed}: the born edge reads the new reference, not {source:?}");
        };
        assert_eq!(ref_idx, 0, "seed {seed}");
        assert!(
            sub_idx < sub_value_count(&after.input_refs[0]),
            "seed {seed}"
        );
        assert!(
            (-1.0..=1.0).contains(&weight),
            "seed {seed}: weight {weight}"
        );
    }
}

/// A populated founder decision node with plasticity and birth seeds:
/// every incumbent compute node, plasticity entry, sink, edge and weight is
/// kept, and the one new edge is the last input of one vote sink.
#[test]
fn add_input_ref_to_graph_node_leaves_existing_edges_untouched() {
    let mut base = v3alpha1_founder_genome();
    {
        let BackendDef::Graph(def) = &mut base.nodes[1].backend_def else {
            panic!("the founder decision node is a Graph node");
        };
        crate::creature::genome::cgp::test_support::set_surface_plasticity(
            def,
            crate::creature::genome::cgp::EdgeSurface::ComputeInput(0),
            Some(crate::creature::genome::PlasticityConfig {
                rule: crate::creature::genome::HebbianRule::Classic,
                learning_rate: 0.1,
                weight_clamp: 5.0,
                lamarckian: false,
                modulation: None,
                retention: 1.0,
            }),
        );
        let rows = def
            .compute_nodes()
            .iter()
            .map(|node| vec![Some(0.25); node.inputs.len()])
            .collect();
        def.seed_compute_rows(rows);
    }
    let before_def = graph_def(&base.nodes[1]).clone();
    assert_eq!(before_def.output_sinks().len(), 97, "a full sink catalog");
    assert!(
        before_def
            .compute_nodes()
            .iter()
            .any(|node| node.inputs.iter().any(|edge| edge.own_weight() != 0.0)),
        "the fixture carries weighted compute edges"
    );
    assert!(
        vote_edges(&base.nodes[1]).iter().any(|edge| edge.2 != 0.0),
        "the fixture carries weighted vote edges"
    );
    for seed in 0u64..32 {
        let mut genome = base.clone();
        let mut r = rng(seed);
        apply_add_to_node(&mut genome, 1, &mut r, 2);
        assert_eq!(genome.nodes[0], base.nodes[0], "seed {seed}: other node");
        let after_def = graph_def(&genome.nodes[1]);
        assert_eq!(
            after_def.compute_nodes(),
            before_def.compute_nodes(),
            "seed {seed}"
        );
        assert_eq!(
            after_def.birth_seeds().map(|seeds| seeds.compute_rows()),
            before_def.birth_seeds().map(|seeds| seeds.compute_rows()),
            "seed {seed}"
        );
        assert_eq!(
            after_def.output_sinks().len(),
            before_def.output_sinks().len()
        );
        let mut grown = 0;
        for (after_sink, before_sink) in after_def
            .output_sinks()
            .iter()
            .zip(before_def.output_sinks())
        {
            assert_eq!(after_sink.kind, before_sink.kind, "seed {seed}");
            let kept = before_sink.inputs.len();
            assert_eq!(
                after_sink.inputs[..kept],
                before_sink.inputs[..],
                "seed {seed}"
            );
            match after_sink.inputs.len() - kept {
                0 => {}
                1 => {
                    assert!(matches!(after_sink.kind, OutputSinkKind::ActionVote(_)));
                    grown += 1;
                }
                extra => panic!("seed {seed}: {extra} edges appended to one sink"),
            }
        }
        assert_eq!(grown, 1, "seed {seed}: exactly one vote sink grew");
        let len = base.nodes[1].input_refs.len();
        assert_eq!(
            genome.nodes[1].input_refs[..len],
            base.nodes[1].input_refs[..]
        );
        assert_eq!(genome.nodes[1].input_refs.len(), len + 1);
    }
}

/// The three draws after the reference are, in order: the sub-value under
/// the width, the vote sink by index over all 27, the weight on [-1, 1].
/// Replaying them by hand reproduces the edge and leaves the stream at the
/// same position.
#[test]
fn add_input_ref_on_graph_draws_sub_value_then_sink_then_weight() {
    for seed in 0u64..64 {
        let mut genome = graph_node_genome_full_catalog();
        let mut r = rng(seed);
        apply_add_to_node(&mut genome, 0, &mut r, 1);
        let mut replay = rng(seed);
        let reference = random_input_reference(&mut replay);
        let sub_idx = replay.gen_range(0..sub_value_count(&reference));
        let sink = VoteSink::from_index(replay.gen_range(0..VOTE_SINK_COUNT)).unwrap();
        let weight = replay.gen_range(-1.0f32..=1.0);
        assert_eq!(genome.nodes[0].input_refs, vec![reference], "seed {seed}");
        assert_eq!(
            vote_edges(&genome.nodes[0]),
            vec![(
                sink,
                GraphSource::InputLeaf {
                    ref_idx: 0,
                    sub_idx
                },
                weight
            )],
            "seed {seed}"
        );
        assert_eq!(
            r.gen::<u64>(),
            replay.gen::<u64>(),
            "seed {seed}: nothing else drawn"
        );
    }
}

/// On a VM node `Add` is the old path draw for draw: one reference, no
/// extra random number.
#[test]
fn add_input_ref_on_vm_draws_only_the_reference() {
    for food_type_count in [1usize, 3] {
        for seed in 0u64..64 {
            let mut genome = single_node_genome_with_input_ref(InputReference::ActionQueue);
            let before = genome.clone();
            let mut r = rng(seed);
            apply_add_to_node(&mut genome, 0, &mut r, food_type_count);
            let mut replay = rng(seed);
            let reference = if food_type_count <= 1 {
                random_input_reference(&mut replay)
            } else {
                random_input_reference_for_food_types(&mut replay, food_type_count)
            };
            assert_eq!(
                genome.nodes[0].input_refs,
                vec![InputReference::ActionQueue, reference]
            );
            assert_eq!(genome.nodes[0].backend_def, before.nodes[0].backend_def);
            assert_eq!(r.gen::<u64>(), replay.gen::<u64>(), "seed {seed}");
        }
    }
}

/// A drawn sink the node lacks leaves the event a plain declaration, with
/// the three draws still taken.
#[test]
fn add_input_ref_with_a_truncated_catalog_stays_a_declaration() {
    for seed in 0u64..64 {
        let mut genome = graph_node_genome_full_catalog();
        let BackendDef::Graph(def) = &mut genome.nodes[0].backend_def else {
            unreachable!()
        };
        crate::creature::genome::cgp::test_support::edit_sinks(def, |sinks| {
            sinks.retain(|sink| !matches!(sink.kind, OutputSinkKind::ActionVote(_)));
        });
        let before = genome.clone();
        let mut r = rng(seed);
        apply_add_to_node(&mut genome, 0, &mut r, 1);
        assert_eq!(genome.nodes[0].input_refs.len(), 1, "seed {seed}");
        assert_eq!(genome.nodes[0].backend_def, before.nodes[0].backend_def);
        let mut full = graph_node_genome_full_catalog();
        let mut replay = rng(seed);
        apply_add_to_node(&mut full, 0, &mut replay, 1);
        assert_eq!(
            r.gen::<u64>(),
            replay.gen::<u64>(),
            "seed {seed}: same draws"
        );
    }
}

/// The appended index 65,535 still fits `u16` and connects; 65,536 does not,
/// so the event stays a declaration, and both take the same three draws.
#[test]
fn add_input_ref_connects_only_while_the_index_fits_u16() {
    let filler = InputReference::ActionQueue;
    let mut fits = graph_node_genome_full_catalog();
    fits.nodes[0].input_refs = vec![filler.clone(); 65_535];
    let mut overflows = graph_node_genome_full_catalog();
    overflows.nodes[0].input_refs = vec![filler; 65_536];

    let mut r_fits = rng(7);
    apply_add_to_node(&mut fits, 0, &mut r_fits, 1);
    let mut r_overflows = rng(7);
    apply_add_to_node(&mut overflows, 0, &mut r_overflows, 1);

    assert_eq!(fits.nodes[0].input_refs.len(), 65_536);
    let edges = vote_edges(&fits.nodes[0]);
    assert_eq!(edges.len(), 1);
    assert!(matches!(
        edges[0].1,
        GraphSource::InputLeaf {
            ref_idx: 65_535,
            ..
        }
    ));
    assert_eq!(overflows.nodes[0].input_refs.len(), 65_537);
    assert_eq!(edge_count(&overflows.nodes[0]), 0, "a plain declaration");
    assert_eq!(r_fits.gen::<u64>(), r_overflows.gen::<u64>(), "same draws");
}

/// The born edge consumes the new reference, so `Prune` cannot take it.
#[test]
fn add_input_ref_on_graph_leaves_the_new_reference_unprunable() {
    for seed in 0u64..32 {
        let mut genome = graph_node_genome_full_catalog();
        let mut r = rng(seed);
        apply_add_to_node(&mut genome, 0, &mut r, 1);
        assert!(prunable_indices(&genome.nodes[0]).is_empty(), "seed {seed}");
    }
}

/// Over many seeds the born edge reaches every one of the 27 vote sinks and
/// all 8 sub-values of a ring, with both signs: every draw is uniform over
/// its whole range, named by index, never by meaning.
#[test]
fn born_edge_reaches_every_vote_sink_and_ring_sub_value() {
    let ring = InputReference::World(WorldInputKey::neighbor_food_ring(OrdinaryFoodTypeId::new(
        0,
    )));
    let width = sub_value_count(&ring);
    assert_eq!(width, 8);
    let mut sinks = std::collections::BTreeSet::new();
    let mut subs = std::collections::BTreeSet::new();
    let (mut negative, mut positive) = (0u32, 0u32);
    for seed in 0u64..4096 {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        let mut r = rng(seed);
        connect_declared_input(&mut def, Some(7), width, &mut r);
        let mut edges = def
            .output_sinks()
            .iter()
            .flat_map(|sink| sink.inputs.iter().map(move |edge| (sink.kind, *edge)));
        let (kind, edge) = edges.next().expect("one edge");
        assert!(edges.next().is_none(), "seed {seed}: only one edge");
        let OutputSinkKind::ActionVote(vote) = kind else {
            panic!("seed {seed}: {kind:?} is not a vote sink");
        };
        sinks.insert(vote.index());
        let GraphSource::InputLeaf {
            ref_idx: 7,
            sub_idx,
        } = edge.source()
        else {
            panic!("seed {seed}: {:?}", edge.source());
        };
        subs.insert(sub_idx);
        if edge.own_weight() < 0.0 {
            negative += 1;
        } else {
            positive += 1;
        }
    }
    assert_eq!(sinks.len(), VOTE_SINK_COUNT);
    assert_eq!(subs.len(), usize::from(width));
    assert!(
        negative > 1500 && positive > 1500,
        "{negative} negative, {positive} positive"
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// Over generated genomes, `Add` appends exactly one reference to the
    /// target node and keeps every earlier one; on a VM node the backend is
    /// unchanged; on a Graph node at most one edge is added (exactly one with
    /// the full vote catalog), on a vote sink, sourced from the appended index
    /// at a sub-value under its width, with a weight in [-1, 1], and the
    /// compute nodes are unchanged; every other node is unchanged.
    #[test]
    fn add_declares_one_reference_and_at_most_one_vote_edge(
        seed in any::<u64>(),
        sizes in prop::collection::vec(module_size(), 1..4),
        food_type_count in 1usize..4,
    ) {
        let base = generated_genome(seed, &sizes);
        for node_idx in 0..base.nodes.len() {
            let mut genome = base.clone();
            let mut r = rng(seed ^ node_idx as u64);
            InputRefMutator::apply_to_node(
                &mut genome, InputRefOperator::Add, node_idx, &mut r, food_type_count,
            )
            .expect("Add always applies");
            for (idx, node) in genome.nodes.iter().enumerate() {
                if idx != node_idx {
                    prop_assert_eq!(node, &base.nodes[idx], "an untargeted node changed");
                }
            }
            let before = &base.nodes[node_idx];
            let after = &genome.nodes[node_idx];
            prop_assert_eq!(after.input_refs.len(), before.input_refs.len() + 1);
            prop_assert_eq!(&after.input_refs[..before.input_refs.len()], &before.input_refs[..]);
            let width = sub_value_count(&after.input_refs[before.input_refs.len()]);
            match (&before.backend_def, &after.backend_def) {
                (BackendDef::Vm(b), BackendDef::Vm(a)) => prop_assert_eq!(a, b),
                (BackendDef::Graph(b), BackendDef::Graph(a)) => {
                    prop_assert_eq!(a.compute_nodes(), b.compute_nodes());
                    let vote_sinks = b
                        .output_sinks()
                        .iter()
                        .filter(|sink| matches!(sink.kind, OutputSinkKind::ActionVote(_)))
                        .count();
                    let new = new_vote_edges(before, after);
                    if vote_sinks == VOTE_SINK_COUNT {
                        prop_assert_eq!(new.len(), 1, "a full catalog receives the edge");
                    } else {
                        prop_assert!(new.len() <= 1);
                    }
                    prop_assert_eq!(edge_count(after), edge_count(before) + new.len());
                    if let Some(&(_, source, weight)) = new.first() {
                        let GraphSource::InputLeaf { ref_idx, sub_idx } = source else {
                            return Err(TestCaseError::fail(format!("born edge source {source:?}")));
                        };
                        prop_assert_eq!(usize::from(ref_idx), before.input_refs.len());
                        prop_assert!(sub_idx < width);
                        prop_assert!((-1.0..=1.0).contains(&weight));
                    }
                }
                _ => prop_assert!(false, "the backend kind changed"),
            }
        }
    }
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
    let mut found = [false; 4]; // Food, Barrier, Occupancy, Nearby
    for seed in 0u64..5000 {
        let mut r = rng(seed);
        match random_input_reference(&mut r) {
            InputReference::World(WorldInputKey::AreaFoodSummary { .. }) => found[0] = true,
            InputReference::World(WorldInputKey::AreaBarrierSummary) => found[1] = true,
            InputReference::World(WorldInputKey::AreaOccupancySummary) => found[2] = true,
            InputReference::World(WorldInputKey::NearbyCreature) => found[3] = true,
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
        "NearbyCreature",
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
