use super::*;
use crate::config::{MutationConfig, OrdinaryFoodTypeId, RuntimeConfig};
use crate::contracts::votes::{ActionParamField, VoteSink, VOTE_SINK_COUNT};
use crate::contracts::OUTPUT_SLOT_COUNT;
use crate::contracts::{DynamicIntrospectionKey, NodeId, StaticIntrospectionKey, WorldInputKey};
use crate::creature::founder::v3alpha1_founder_genome;
use crate::creature::parseability::ParseabilityGate;
use crate::genome::cgp::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource, OutputSink,
    OutputSinkKind,
};
use crate::genome::mesh_annotations::MeshReadClass;
use crate::genome::{input_ref_kind, InputRefKind};
use crate::genome::{BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction};
use crate::mutation::applicability_tests::{genome as generated_genome, module_size};
use crate::mutation::sampling::{random_input_reference, random_input_reference_for_food_types};
use crate::neighborhood::{Battery, Signature};
use proptest::prelude::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

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
        InputRefOperator::AddSensorLive,
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
    assert!(swap_alternatives(&food_here(0), &crate::config::test_sensor_context(1)).is_empty());
    assert_eq!(
        swap_alternatives(&food_here(0), &crate::config::test_sensor_context(2)),
        vec![food_here(1)]
    );
    assert_eq!(
        swap_alternatives(
            &InputReference::World(WorldInputKey::neighbor_food_ring(OrdinaryFoodTypeId::new(
                1
            ))),
            &crate::config::test_sensor_context(3)
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
    let others = swap_alternatives(&energy, &crate::config::test_sensor_context(1));
    assert_eq!(others.len(), 6);
    assert!(others.contains(&InputReference::DynamicIntrospection(
        DynamicIntrospectionKey::HopsThisTick
    )));
    assert!(!others.contains(&energy));
    assert!(others.iter().all(
        |r| input_ref_kind(r, &crate::config::test_sensor_context(1))
            == input_ref_kind(&energy, &crate::config::test_sensor_context(1))
    ));
    // Upstream slots form one kind.
    let upstream = swap_alternatives(
        &InputReference::UpstreamSlot(2),
        &crate::config::test_sensor_context(1),
    );
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
            swap_alternatives(&lone, &crate::config::test_sensor_context(4)).is_empty(),
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
        input_ref_kind(&ring, &crate::config::test_sensor_context(1)),
        InputRefKind {
            class: MeshReadClass::Food,
            width: 8
        }
    );
    assert_eq!(
        input_ref_kind(&summary, &crate::config::test_sensor_context(1)),
        InputRefKind {
            class: MeshReadClass::Food,
            width: 7
        }
    );
    assert_ne!(
        input_ref_kind(&ring, &crate::config::test_sensor_context(1)),
        input_ref_kind(&summary, &crate::config::test_sensor_context(1))
    );
    assert_eq!(
        input_ref_kind(
            &InputReference::ActionQueue,
            &crate::config::test_sensor_context(1)
        ),
        InputRefKind {
            class: MeshReadClass::ActionQueue,
            width: 30
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
        InputRefMutator::applicable_indices(
            &genome,
            InputRefOperator::Swap,
            &crate::config::test_sensor_context(1)
        ),
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
        InputRefMutator::applicable_indices(
            &genome,
            InputRefOperator::Prune,
            &crate::config::test_sensor_context(1)
        ),
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
    InputRefMutator::apply_to_node(
        &mut pruned,
        InputRefOperator::Prune,
        1,
        &mut r,
        &crate::config::test_sensor_context(1),
    )
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
        InputRefMutator::apply_in(
            &mut genome,
            InputRefOperator::Swap,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
            &config,
            &crate::config::test_sensor_context(2),
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
    let kind = |reference: &InputReference| {
        input_ref_kind(reference, &crate::config::test_sensor_context(1))
    };
    let hops = InputReference::DynamicIntrospection(DynamicIntrospectionKey::HopsThisTick);
    for (reference, class, width) in [
        (InputReference::ActionVotes, MeshReadClass::Decision, 39),
        (
            InputReference::PreviousPassVotes,
            MeshReadClass::Decision,
            39,
        ),
        (InputReference::CommitCounts, MeshReadClass::Decision, 5),
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
        swap_alternatives(
            &InputReference::ActionVotes,
            &crate::config::test_sensor_context(3)
        ),
        vec![InputReference::PreviousPassVotes]
    );
    assert_eq!(
        swap_alternatives(
            &InputReference::PreviousPassVotes,
            &crate::config::test_sensor_context(3)
        ),
        vec![InputReference::ActionVotes]
    );
    let mut hops_partners = swap_alternatives(&hops, &crate::config::test_sensor_context(1));
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
            &base, InputRefOperator::Swap, &crate::config::test_sensor_context(food_type_count),
        );
        for node_idx in applicable {
            for apply_seed in 0u64..4 {
                let mut genome = base.clone();
                let mut r = rng(seed ^ apply_seed);
                InputRefMutator::apply_to_node(
                    &mut genome, InputRefOperator::Swap, node_idx, &mut r, &crate::config::test_sensor_context(food_type_count),
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
                    swap_alternatives(&before.input_refs[i], &crate::config::test_sensor_context(food_type_count))
                        .contains(&after.input_refs[i]),
                    "{:?} -> {:?} is not a within-kind step",
                    before.input_refs[i],
                    after.input_refs[i]
                );
                prop_assert_eq!(
                    input_ref_kind(&after.input_refs[i], &crate::config::test_sensor_context(1)),
                    input_ref_kind(&before.input_refs[i], &crate::config::test_sensor_context(1))
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
        let applicable = InputRefMutator::applicable_indices(&base, InputRefOperator::Prune, &crate::config::test_sensor_context(1));
        for node_idx in applicable {
            for apply_seed in 0u64..4 {
                let mut genome = base.clone();
                let mut r = rng(seed ^ apply_seed);
                InputRefMutator::apply_to_node(
                    &mut genome, InputRefOperator::Prune, node_idx, &mut r, &crate::config::test_sensor_context(1),
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
        let ir = random_input_reference(&mut r, &crate::config::test_sensor_context(1));
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
            InputReference::UpstreamBus => "UpstreamBus".to_string(),
            InputReference::ActionQueue => "ActionQueue".to_string(),
            InputReference::ActionVotes => "ActionVotes".to_string(),
            InputReference::PreviousPassVotes => "PreviousPassVotes".to_string(),
            InputReference::CommitCounts => "CommitCounts".to_string(),
            InputReference::PreviousOutcome => "PreviousOutcome".to_string(),
        };
        categories.insert(cat);
    }
    // 17 categories: 8 original + 4 extended perception families (one
    // `NearbyCreature`, T25.F05) + the four decision-state compounds
    // (T19.F05; `HopsThisTick` is a dynamic key) + `UpstreamBus` (T25.F11).
    assert_eq!(
        categories.len(),
        17,
        "all 17 input reference categories must be reachable; got {:?}",
        categories
    );
}

#[test]
fn random_input_reference_upstream_slot_bounded() {
    for seed in 0u64..20_000 {
        let mut r = rng(seed);
        if let InputReference::UpstreamSlot(slot) =
            random_input_reference(&mut r, &crate::config::test_sensor_context(1))
        {
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
        InputRefOperator::AddSensorLive,
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
fn input_ref_operator_weights_are_positive() {
    let all = InputRefOperator::ALL;
    assert_eq!(
        all.len(),
        5,
        "ALL must cover every InputRefOperator variant"
    );
    for &op in &all {
        assert!(op.weight() > 0, "weight must be positive for {:?}", op);
    }
}

#[test]
fn complexity_effect_consistent_with_types() {
    use crate::mutation::event::ComplexityEffect;
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

// ── AddSensorLive / AddSensorSilent wire every field set (T25.F11) ─────────

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

const ADD_SENSOR: [InputRefOperator; 2] = [
    InputRefOperator::AddSensorLive,
    InputRefOperator::AddSensorSilent,
];

fn apply_add_to_node(
    genome: &mut CreatureGenome,
    op: InputRefOperator,
    node_idx: usize,
    rng: &mut SmallRng,
    food_type_count: usize,
) -> Result<crate::mutation::event::EventReport, MutationSkipReason> {
    InputRefMutator::apply_to_node(
        genome,
        op,
        node_idx,
        rng,
        &crate::config::test_sensor_context(food_type_count),
    )
}

/// The whole-set postcondition of a creation on `before` → `after` (T25.F11
/// invariants 5–7): one declaration appended; on a Graph node one bundle
/// per field set of the new sensor, in set order, each with every
/// coordinate its wiring materializes present, own weight 0, and a scale
/// in `[-1, 1]` (live) or exactly 0 (silent); nothing else changes.
fn assert_whole_sensor(before: &NodeGenome, after: &NodeGenome, op: InputRefOperator) {
    assert_eq!(after.input_refs.len(), before.input_refs.len() + 1);
    assert_eq!(
        after.input_refs[..before.input_refs.len()],
        before.input_refs[..]
    );
    let ref_idx = u16::try_from(before.input_refs.len()).unwrap();
    let reference = &after.input_refs[usize::from(ref_idx)];
    let (BackendDef::Graph(b), BackendDef::Graph(a)) = (&before.backend_def, &after.backend_def)
    else {
        assert_eq!(
            after.backend_def, before.backend_def,
            "a VM node is unchanged"
        );
        return;
    };
    assert_eq!(a.compute_nodes(), b.compute_nodes());
    assert_eq!(a.bundles()[..b.bundles().len()], b.bundles()[..]);
    let sets: Vec<_> =
        crate::contracts::SensorDescriptor::of(reference, &crate::config::test_sensor_context(1))
            .field_sets()
            .collect();
    let new: Vec<_> = (b.bundles().len()..a.bundles().len())
        .map(|index| crate::genome::cgp::BundleId::new(u16::try_from(index).unwrap()))
        .collect();
    assert_eq!(
        new.len(),
        sets.len(),
        "{reference:?}: one bundle per field set"
    );
    let mut members = 0;
    for (field_set, (&id, set)) in new.iter().zip(&sets).enumerate() {
        let record = &a.bundles()[id.index()];
        assert_eq!(
            record.source(),
            crate::genome::cgp::BundleSource::Sensor {
                ref_idx,
                field_set: u8::try_from(field_set).unwrap()
            }
        );
        assert!(
            record.wiring().fits(set),
            "{reference:?}: {:?}",
            record.wiring()
        );
        // Every coordinate of the set under the run's context, each once.
        let mut want = record.wiring().coordinates(set);
        let mut got: Vec<_> = a
            .members(id)
            .map(|m| {
                let GraphSource::InputLeaf { sub_idx, .. } = a.edge_at(m).source() else {
                    panic!("a sensor member reads a leaf");
                };
                (sub_idx, a.site_of(m).expect("a member on its wiring"))
            })
            .collect();
        want.sort_unstable();
        got.sort_unstable();
        assert_eq!(got, want, "{reference:?}: the whole set");
        assert!(a.members(id).all(|m| a.edge_at(m).own_weight() == 0.0));
        members += want.len();
        match op {
            InputRefOperator::AddSensorSilent => assert_eq!(record.scale(), 0.0),
            _ => assert!((-1.0..=1.0).contains(&record.scale())),
        }
    }
    assert_eq!(edge_count(after), edge_count(before) + members);
}

/// Both creation operators on a bare full-catalog Graph node declare one
/// sensor and wire every field set of it, live or silent by the operator.
#[test]
fn add_sensor_on_a_graph_node_wires_the_whole_sensor() {
    for op in ADD_SENSOR {
        for seed in 0u64..200 {
            let base = graph_node_genome_full_catalog();
            let mut genome = base.clone();
            InputRefMutator::apply(
                &mut genome,
                op,
                &mut TargetSelector::reachable_only(&[], 0.0),
                &mut rng(seed),
                &default_config(),
            )
            .unwrap();
            assert_whole_sensor(&base.nodes[0], &genome.nodes[0], op);
        }
    }
}

/// A populated founder decision node with plasticity and birth seeds:
/// every incumbent compute node, seed, sink edge and weight is kept, and the
/// new members follow each sink's incumbents.
#[test]
fn add_sensor_to_graph_node_leaves_existing_edges_untouched() {
    let mut base = v3alpha1_founder_genome();
    {
        let BackendDef::Graph(def) = &mut base.nodes[1].backend_def else {
            panic!("the founder decision node is a Graph node");
        };
        crate::genome::cgp::test_support::set_surface_plasticity(
            def,
            crate::genome::cgp::EdgeSurface::ComputeInput(0),
            Some(crate::genome::PlasticityConfig {
                rule: crate::genome::HebbianRule::Classic,
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
    assert_eq!(before_def.output_sinks().len(), 108, "a full sink catalog");
    for op in ADD_SENSOR {
        for seed in 0u64..32 {
            let mut genome = base.clone();
            apply_add_to_node(&mut genome, op, 1, &mut rng(seed), 2).expect("a full catalog");
            assert_eq!(genome.nodes[0], base.nodes[0], "seed {seed}: other node");
            assert_whole_sensor(&base.nodes[1], &genome.nodes[1], op);
            let after_def = graph_def(&genome.nodes[1]);
            assert_eq!(
                after_def.birth_seeds().map(|seeds| seeds.compute_rows()),
                before_def.birth_seeds().map(|seeds| seeds.compute_rows()),
                "seed {seed}"
            );
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
            }
        }
    }
}

/// The draws after the sensor are, per field set in order, the plan (a
/// `Plain` set's sink over the present vote sinks in catalog order; a
/// geometric set's kind over the hostable kinds, then the offset) and, for
/// a live bundle, its scale; replaying them by hand reproduces every
/// wiring and scale (T25.F11 invariant 6).
#[test]
fn add_sensor_draws_the_sensor_then_each_sets_plan_and_scale() {
    use crate::contracts::votes::DirectedVoteKind;
    use crate::contracts::{FieldSet, SensorDescriptor};
    use crate::genome::cgp::Wiring;
    let context = crate::config::test_sensor_context(1);
    for op in ADD_SENSOR {
        for seed in 0u64..64 {
            let mut genome = graph_node_genome_full_catalog();
            apply_add_to_node(&mut genome, op, 0, &mut rng(seed), 1).unwrap();
            let mut replay = rng(seed);
            let reference = random_input_reference(&mut replay, &context);
            assert_eq!(genome.nodes[0].input_refs, vec![reference.clone()]);
            let expected: Vec<(Wiring, f32)> = SensorDescriptor::of(&reference, &context)
                .field_sets()
                .map(|set| {
                    let wiring = match set {
                        FieldSet::Plain { .. } => Wiring::Plain {
                            sink: VoteSink::from_index(replay.gen_range(0..VOTE_SINK_COUNT))
                                .unwrap(),
                        },
                        FieldSet::Ring | FieldSet::Pair(_) => {
                            let kind = DirectedVoteKind::ALL
                                [replay.gen_range(0..DirectedVoteKind::ALL.len())];
                            let offset = replay.gen_range(0..8u8);
                            if set == FieldSet::Ring {
                                Wiring::Ring { kind, offset }
                            } else {
                                Wiring::DirectionPair { kind, offset }
                            }
                        }
                    };
                    let scale = match op {
                        InputRefOperator::AddSensorLive => replay.gen_range(-1.0f32..=1.0),
                        _ => 0.0,
                    };
                    (wiring, scale)
                })
                .collect();
            let made: Vec<(Wiring, f32)> = graph_def(&genome.nodes[0])
                .bundles()
                .iter()
                .map(|bundle| (bundle.wiring(), bundle.scale()))
                .collect();
            assert_eq!(made, expected, "{op:?} seed {seed}");
        }
    }
}

/// On a VM node a creation is the declaration alone: one reference drawn,
/// no further draw and no backend change.
#[test]
fn add_sensor_on_vm_draws_only_the_reference() {
    for op in ADD_SENSOR {
        for food_type_count in [1usize, 3] {
            for seed in 0u64..64 {
                let mut genome = single_node_genome_with_input_ref(InputReference::ActionQueue);
                let before = genome.clone();
                let mut r = rng(seed);
                apply_add_to_node(&mut genome, op, 0, &mut r, food_type_count).unwrap();
                let mut replay = rng(seed);
                let reference = if food_type_count <= 1 {
                    random_input_reference(&mut replay, &crate::config::test_sensor_context(1))
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
}

/// A Graph node without a vote sink admits no plan: the event skips with
/// `NoApplicableTarget` before the first set's draws and writes nothing, so
/// no unwired declaration survives.
#[test]
fn add_sensor_without_an_admissible_plan_skips_and_writes_nothing() {
    for op in ADD_SENSOR {
        for seed in 0u64..64 {
            let mut genome = graph_node_genome_full_catalog();
            let BackendDef::Graph(def) = &mut genome.nodes[0].backend_def else {
                unreachable!()
            };
            crate::genome::cgp::test_support::edit_sinks(def, |sinks| {
                sinks.retain(|sink| !matches!(sink.kind, OutputSinkKind::ActionVote(_)));
            });
            let before = genome.clone();
            let mut r = rng(seed);
            assert_eq!(
                apply_add_to_node(&mut genome, op, 0, &mut r, 1),
                Err(MutationSkipReason::NoApplicableTarget)
            );
            assert_eq!(genome, before, "seed {seed}");
            let mut replay = rng(seed);
            random_input_reference(&mut replay, &crate::config::test_sensor_context(1));
            assert_eq!(
                r.gen::<u64>(),
                replay.gen::<u64>(),
                "seed {seed}: the sensor only"
            );
        }
    }
}

/// The appended index 65,535 still fits `u16` and is wired; 65,536 does
/// not, so `add_sensor` refuses it and nothing is written.
#[test]
fn add_sensor_declares_only_while_the_index_fits_u16() {
    let filler = InputReference::ActionQueue;
    let mut fits = graph_node_genome_full_catalog();
    fits.nodes[0].input_refs = vec![filler.clone(); 65_535];
    let mut overflows = graph_node_genome_full_catalog();
    overflows.nodes[0].input_refs = vec![filler; 65_536];

    apply_add_to_node(
        &mut fits,
        InputRefOperator::AddSensorLive,
        0,
        &mut rng(7),
        1,
    )
    .unwrap();
    let before = overflows.clone();
    assert_eq!(
        apply_add_to_node(
            &mut overflows,
            InputRefOperator::AddSensorLive,
            0,
            &mut rng(7),
            1
        ),
        Err(MutationSkipReason::NoApplicableTarget),
        "a declaration past the u16 index space is refused (T25.F11 `add_sensor`)"
    );
    assert_eq!(fits.nodes[0].input_refs.len(), 65_536);
    let edges = vote_edges(&fits.nodes[0]);
    assert!(!edges.is_empty());
    assert!(edges.iter().all(|edge| matches!(
        edge.1,
        GraphSource::InputLeaf {
            ref_idx: 65_535,
            ..
        }
    )));
    assert_eq!(overflows, before, "nothing written");
}

/// A node at the 65,536-declaration limit, its last sensor wired, still
/// mutates without panicking: the Graph source draw and the raw-field moves
/// count declarations past `u16` and leave a valid graph.
#[test]
fn graph_mutation_after_the_boundary_addition_stays_within_u16() {
    use crate::mutation::graph::{GraphMutator, GraphOperator};
    let mut boundary = graph_node_genome_full_catalog();
    boundary.nodes[0].input_refs = vec![InputReference::ActionQueue; 65_535];
    apply_add_to_node(
        &mut boundary,
        InputRefOperator::AddSensorLive,
        0,
        &mut rng(7),
        1,
    )
    .unwrap();
    let context = crate::config::test_sensor_context(1);
    for op in [
        GraphOperator::AddGraphEdge,
        GraphOperator::RetargetGraphEdge,
        GraphOperator::GraphRawFieldMutation,
    ] {
        for seed in 0u64..16 {
            let mut genome = boundary.clone();
            let _ = GraphMutator::apply_to_node(
                &mut genome,
                op,
                0,
                &mut rng(seed),
                &MutationConfig::default(),
                &context,
            );
            assert_eq!(genome.nodes[0].input_refs.len(), 65_536);
            assert_eq!(
                crate::genome::cgp::validate_graph(&genome.nodes[0]),
                Ok(()),
                "{op:?} seed {seed}"
            );
        }
    }
}

/// The members consume the new reference, so `Prune` cannot take it.
#[test]
fn add_sensor_on_graph_leaves_the_new_reference_unprunable() {
    for op in ADD_SENSOR {
        for seed in 0u64..32 {
            let mut genome = graph_node_genome_full_catalog();
            apply_add_to_node(&mut genome, op, 0, &mut rng(seed), 1).unwrap();
            assert!(prunable_indices(&genome.nodes[0]).is_empty(), "seed {seed}");
        }
    }
}

/// Over many seeds a one-field sensor's `Plain` bundle reaches every vote
/// sink, and a live scale takes both signs; a ring's bundle reaches every
/// kind and offset and always wires all eight cells.
#[test]
fn creations_reach_every_plan_and_both_signs() {
    use crate::genome::cgp::Wiring;
    use crate::mutation::graph::bundle::{add_sensor_by_law, BirthLaw};
    let context = crate::config::test_sensor_context(1);
    let mut sinks = std::collections::BTreeSet::new();
    let mut rings = std::collections::BTreeSet::new();
    let (mut negative, mut positive) = (0u32, 0u32);
    for seed in 0u64..4096 {
        for reference in [
            InputReference::UpstreamSlot(7),
            InputReference::World(WorldInputKey::neighbor_food_ring(OrdinaryFoodTypeId::new(
                0,
            ))),
        ] {
            let mut genome = graph_node_genome_full_catalog();
            let node = &mut genome.nodes[0];
            let before = node.clone();
            add_sensor_by_law(node, reference, BirthLaw::Live, &context, &mut rng(seed))
                .expect("a full catalog");
            assert_whole_sensor(&before, node, InputRefOperator::AddSensorLive);
            let bundle = graph_def(node).bundles()[0];
            match bundle.wiring() {
                Wiring::Plain { sink } => {
                    sinks.insert(sink.index());
                }
                Wiring::Ring { kind, offset } => {
                    rings.insert((kind, offset));
                }
                Wiring::DirectionPair { .. } => panic!("no pair set"),
            }
            if bundle.scale() < 0.0 {
                negative += 1;
            } else {
                positive += 1;
            }
        }
    }
    assert_eq!(sinks.len(), VOTE_SINK_COUNT);
    assert_eq!(rings.len(), 4 * 8);
    assert!(
        negative > 3000 && positive > 3000,
        "{negative} negative, {positive} positive"
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// Over generated genomes, a creation either skips writing nothing or
    /// appends exactly one reference to the target node and wires every
    /// field set of it on a Graph node (a VM node's backend unchanged);
    /// every other node is unchanged.
    #[test]
    fn add_sensor_declares_one_reference_and_wires_it_whole(
        seed in any::<u64>(),
        sizes in prop::collection::vec(module_size(), 1..4),
        food_type_count in 1usize..4,
    ) {
        let base = generated_genome(seed, &sizes);
        for op in ADD_SENSOR {
            for node_idx in 0..base.nodes.len() {
                let mut genome = base.clone();
                let result = apply_add_to_node(
                    &mut genome, op, node_idx, &mut rng(seed ^ node_idx as u64), food_type_count,
                );
                if result.is_err() {
                    prop_assert_eq!(&genome, &base, "a skip writes nothing");
                    continue;
                }
                for (idx, node) in genome.nodes.iter().enumerate() {
                    if idx != node_idx {
                        prop_assert_eq!(node, &base.nodes[idx], "an untargeted node changed");
                    }
                }
                assert_whole_sensor(&base.nodes[node_idx], &genome.nodes[node_idx], op);
            }
        }
    }
}

/// The VM `ReadInput` auto-insertion is removed: a creation on a VM node
/// leaves the program byte-identical.
#[test]
fn add_sensor_to_vm_node_leaves_program_unchanged() {
    for op in ADD_SENSOR {
        let mut genome = single_node_genome_with_input_ref(InputReference::World(
            WorldInputKey::food_here(OrdinaryFoodTypeId::default()),
        ));
        genome.nodes[0].input_refs.clear(); // start with 0 refs
        let program_before = match &genome.nodes[0].backend_def {
            BackendDef::Vm(vm) => vm.program.clone(),
            _ => panic!("expected VM backend"),
        };
        InputRefMutator::apply(
            &mut genome,
            op,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut rng(42),
            &default_config(),
        )
        .unwrap();
        assert_eq!(genome.nodes[0].input_refs.len(), 1);
        let BackendDef::Vm(ref vm) = genome.nodes[0].backend_def else {
            panic!("expected VM backend");
        };
        assert_eq!(
            vm.program, program_before,
            "{op:?} changes no VM instruction"
        );
    }
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
        match random_input_reference(&mut r, &crate::config::test_sensor_context(1)) {
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
        if random_input_reference(&mut r, &crate::config::test_sensor_context(1))
            == InputReference::ActionQueue
        {
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
        let _ = InputRefMutator::apply_in(
            &mut genome,
            InputRefOperator::AddSensorLive,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
            &default_config(),
            &crate::config::test_sensor_context(3),
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
        let _ = InputRefMutator::apply_in(
            &mut genome,
            InputRefOperator::Swap,
            &mut TargetSelector::reachable_only(&[], 0.0),
            &mut r,
            &default_config(),
            &crate::config::test_sensor_context(4),
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
    let result = InputRefMutator::apply_in(
        &mut genome,
        InputRefOperator::RawFieldMutation,
        &mut TargetSelector::reachable_only(&[], 0.0),
        &mut r,
        &default_config(),
        &crate::config::test_sensor_context(3),
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

    let at_cap = input_reference_universe(&crate::config::test_sensor_context(full_range));

    assert!(
        at_cap.contains(&last_food_type),
        "food type {} must be enumerated at {full_range} food types",
        u16::MAX
    );
    assert_eq!(
        input_reference_universe(&crate::config::test_sensor_context(full_range + 7)),
        at_cap,
        "food type counts above the cap add nothing to the universe"
    );
}
