use rand::Rng;

use crate::config::MutationConfig;
use crate::contracts::{
    Direction, DynamicIntrospectionKey, InputReference, StaticIntrospectionKey, WorldInputKey,
};
use crate::creature::genome::{BackendDef, CreatureGenome, GraphNodeKind, VmInstruction};
use crate::mutation::compound;
use crate::mutation::types::MutationSkipReason;

/// Input reference mutation operator variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputRefOperator {
    Add,
    Remove,
    Swap,
    RawFieldMutation,
}

impl InputRefOperator {
    pub const ALL: [Self; 4] = [Self::Add, Self::Remove, Self::Swap, Self::RawFieldMutation];

    /// Per-operator weight reflecting impact tier.
    /// 4 = refinement, 2 = moderate, 1 = structural.
    #[must_use]
    pub const fn weight(self) -> u8 {
        match self {
            Self::Add => 2,
            Self::Remove => 2,
            Self::Swap => 4,
            Self::RawFieldMutation => 4,
        }
    }

    const TOTAL_WEIGHT: u16 = {
        assert!(
            Self::ALL.len() == 4,
            "ALL must cover every InputRefOperator variant"
        );
        let mut sum = 0u16;
        let mut i = 0;
        while i < Self::ALL.len() {
            sum += Self::ALL[i].weight() as u16;
            i += 1;
        }
        sum
    };

    /// Whether this operator increases, decreases, or preserves genome complexity.
    #[must_use]
    pub const fn complexity_effect(self) -> crate::mutation::types::ComplexityEffect {
        use crate::mutation::types::ComplexityEffect;
        match self {
            Self::Add => ComplexityEffect::Increasing,
            Self::Remove => ComplexityEffect::Decreasing,
            Self::Swap | Self::RawFieldMutation => ComplexityEffect::Neutral,
        }
    }

    const NON_INCREASING_WEIGHT: u16 = {
        let mut sum = 0u16;
        let mut i = 0;
        while i < Self::ALL.len() {
            if !Self::ALL[i].complexity_effect().is_increasing() {
                sum += Self::ALL[i].weight() as u16;
            }
            i += 1;
        }
        sum
    };

    const DECREASING_WEIGHT: u16 = {
        let mut sum = 0u16;
        let mut i = 0;
        while i < Self::ALL.len() {
            if Self::ALL[i].complexity_effect().is_decreasing() {
                sum += Self::ALL[i].weight() as u16;
            }
            i += 1;
        }
        sum
    };

    /// Pick a random Decreasing-only operator weighted by impact tier.
    pub fn random_decreasing(rng: &mut impl Rng) -> Option<Self> {
        if Self::DECREASING_WEIGHT == 0 {
            return None;
        }
        let mut r = rng.gen_range(0..Self::DECREASING_WEIGHT);
        for &op in &Self::ALL {
            if !op.complexity_effect().is_decreasing() {
                continue;
            }
            let w = op.weight() as u16;
            if r < w {
                return Some(op);
            }
            r -= w;
        }
        unreachable!()
    }

    /// Pick a random non-increasing operator (Neutral or Decreasing) weighted by impact tier.
    pub fn random_non_increasing(rng: &mut impl Rng) -> Option<Self> {
        if Self::NON_INCREASING_WEIGHT == 0 {
            return None;
        }
        let mut r = rng.gen_range(0..Self::NON_INCREASING_WEIGHT);
        for &op in &Self::ALL {
            if op.complexity_effect().is_increasing() {
                continue;
            }
            let w = op.weight() as u16;
            if r < w {
                return Some(op);
            }
            r -= w;
        }
        unreachable!()
    }

    /// Pick a random input ref operator weighted by impact tier.
    pub fn random(rng: &mut impl Rng) -> Self {
        let mut r = rng.gen_range(0..Self::TOTAL_WEIGHT);
        for &op in &Self::ALL {
            let w = op.weight() as u16;
            if r < w {
                return op;
            }
            r -= w;
        }
        unreachable!()
    }
}

/// Input reference domain mutator.
pub struct InputRefMutator;

impl InputRefMutator {
    /// Apply an input ref operator to the genome.
    pub fn apply(
        genome: &mut CreatureGenome,
        op: InputRefOperator,
        rng: &mut impl Rng,
        config: &MutationConfig,
    ) -> Result<(), MutationSkipReason> {
        if genome.nodes.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }

        match op {
            InputRefOperator::Add => apply_add(genome, rng, config),
            InputRefOperator::Remove => apply_remove(genome, rng),
            InputRefOperator::Swap => apply_swap(genome, rng, config),
            InputRefOperator::RawFieldMutation => apply_raw_field_mutation(genome, rng),
        }
    }
}

/// After removing `input_refs[removed_ref_idx]` from `genome.nodes[node_idx]`,
/// update all internal references (graph InputRef nodes and VM ReadInput instructions)
/// so they stay consistent:
/// - `ref_idx == removed_ref_idx` → set to `u16::MAX` (invalidated → resolves to 0.0)
/// - `ref_idx > removed_ref_idx` → decrement by 1
/// - `ref_idx < removed_ref_idx` → unchanged
///
/// `sub_idx` is unaffected — it indexes within a compound input, not across `input_refs`.
pub(crate) fn reindex_after_removal(
    genome: &mut CreatureGenome,
    node_idx: usize,
    removed_ref_idx: u16,
) {
    debug_assert!(node_idx < genome.nodes.len(), "node_idx out of bounds");
    let node = &mut genome.nodes[node_idx];
    match &mut node.backend_def {
        BackendDef::Graph(gd) => {
            for internal in &mut gd.internal_nodes {
                if let GraphNodeKind::InputRef { ref_idx, .. } = &mut internal.kind {
                    if *ref_idx == removed_ref_idx {
                        *ref_idx = u16::MAX;
                    } else if *ref_idx > removed_ref_idx {
                        *ref_idx -= 1;
                    }
                }
            }
        }
        BackendDef::Vm(vm) => {
            for instr in &mut vm.program {
                if let VmInstruction::ReadInput { ref_idx, .. } = instr {
                    if *ref_idx == removed_ref_idx {
                        *ref_idx = u16::MAX;
                    } else if *ref_idx > removed_ref_idx {
                        *ref_idx -= 1;
                    }
                }
            }
        }
    }
}

fn apply_add(
    genome: &mut CreatureGenome,
    rng: &mut impl Rng,
    config: &MutationConfig,
) -> Result<(), MutationSkipReason> {
    let node_idx = rng.gen_range(0..genome.nodes.len());
    let new_ref = random_input_reference(rng);
    let count = compound::sub_value_count(&new_ref, config);
    let ref_idx = genome.nodes[node_idx].input_refs.len() as u16;
    genome.nodes[node_idx].input_refs.push(new_ref);
    if count > 1 {
        compound::create_fan_out_nodes(&mut genome.nodes[node_idx], ref_idx, count);
    }
    Ok(())
}

fn apply_remove(genome: &mut CreatureGenome, rng: &mut impl Rng) -> Result<(), MutationSkipReason> {
    let eligible: Vec<usize> = genome
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| !n.input_refs.is_empty())
        .map(|(i, _)| i)
        .collect();
    if eligible.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let node_idx = eligible[rng.gen_range(0..eligible.len())];
    let ref_idx = rng.gen_range(0..genome.nodes[node_idx].input_refs.len());
    genome.nodes[node_idx].input_refs.remove(ref_idx);
    debug_assert!(ref_idx <= u16::MAX as usize, "input_refs index exceeds u16");
    reindex_after_removal(genome, node_idx, ref_idx as u16);
    Ok(())
}

fn apply_swap(
    genome: &mut CreatureGenome,
    rng: &mut impl Rng,
    config: &MutationConfig,
) -> Result<(), MutationSkipReason> {
    let eligible: Vec<usize> = genome
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| !n.input_refs.is_empty())
        .map(|(i, _)| i)
        .collect();
    if eligible.is_empty() {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let node_idx = eligible[rng.gen_range(0..eligible.len())];
    let ref_idx = rng.gen_range(0..genome.nodes[node_idx].input_refs.len());
    let new_ref = random_input_reference(rng);
    let count = compound::sub_value_count(&new_ref, config);
    genome.nodes[node_idx].input_refs[ref_idx] = new_ref;
    if count > 1 {
        compound::create_fan_out_nodes(&mut genome.nodes[node_idx], ref_idx as u16, count);
    }
    Ok(())
}

/// Generate a random input reference from the full set of 44 possible values.
///
/// Distribution: FoodHere (1) + NeighborCellFood (8) + NeighborCellBarrier (8) +
/// NeighborCellOccupied (8) + StaticIntrospection (2) + DynamicIntrospection (2) +
/// ActionQueue (1) + AreaFoodSummary (1) + AreaBarrierSummary (1) +
/// AreaOccupancySummary (1) + NearbyCreatureCore (1) + NearbyCreatureVitals (1) +
/// NearbyCreatureIdentity (1) + UpstreamSlot(usize) raw values (8 weighted slots) = 44 total.
fn random_input_reference(rng: &mut impl Rng) -> InputReference {
    let idx = rng.gen_range(0u8..44);
    match idx {
        0 => InputReference::World(WorldInputKey::FoodHere),
        1..=8 => {
            InputReference::World(WorldInputKey::NeighborCellFood(direction_from_idx(idx - 1)))
        }
        9..=16 => InputReference::World(WorldInputKey::NeighborCellBarrier(direction_from_idx(
            idx - 9,
        ))),
        17..=24 => InputReference::World(WorldInputKey::NeighborCellOccupied(direction_from_idx(
            idx - 17,
        ))),
        25 => InputReference::StaticIntrospection(StaticIntrospectionKey::Generation),
        26 => InputReference::StaticIntrospection(StaticIntrospectionKey::AgeTicks),
        27 => InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
        28 => InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyConsumedThisTick),
        29 => InputReference::ActionQueue,
        30 => InputReference::World(WorldInputKey::AreaFoodSummary),
        31 => InputReference::World(WorldInputKey::AreaBarrierSummary),
        32 => InputReference::World(WorldInputKey::AreaOccupancySummary),
        33 => InputReference::World(WorldInputKey::NearbyCreatureCore),
        34 => InputReference::World(WorldInputKey::NearbyCreatureVitals),
        35 => InputReference::World(WorldInputKey::NearbyCreatureIdentity),
        _ => InputReference::UpstreamSlot(rng.gen::<u8>() as usize),
    }
}

fn apply_raw_field_mutation(
    genome: &mut CreatureGenome,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    // Count eligible targets: UpstreamSlot input_refs + InputRef graph nodes (for sub_idx mutation).
    let mut upstream_count: usize = 0;
    let mut graph_input_ref_count: usize = 0;
    for node in &genome.nodes {
        upstream_count += node
            .input_refs
            .iter()
            .filter(|r| matches!(r, InputReference::UpstreamSlot(_)))
            .count();
        if let BackendDef::Graph(ref gd) = node.backend_def {
            graph_input_ref_count += gd
                .internal_nodes
                .iter()
                .filter(|n| matches!(n.kind, GraphNodeKind::InputRef { .. }))
                .count();
        }
    }
    let total = upstream_count + graph_input_ref_count;
    if total == 0 {
        return Err(MutationSkipReason::NoApplicableTarget);
    }

    let mut pick = rng.gen_range(0..total);

    // First pool: UpstreamSlot input_refs.
    if pick < upstream_count {
        for node in &mut genome.nodes {
            for input_ref in &mut node.input_refs {
                if matches!(input_ref, InputReference::UpstreamSlot(_)) {
                    if pick == 0 {
                        *input_ref = InputReference::UpstreamSlot(rng.gen::<u16>() as usize);
                        return Ok(());
                    }
                    pick -= 1;
                }
            }
        }
    }

    // Second pool: InputRef graph node sub_idx mutation.
    pick -= upstream_count;
    for node in &mut genome.nodes {
        if let BackendDef::Graph(ref mut gd) = node.backend_def {
            for internal in &mut gd.internal_nodes {
                if let GraphNodeKind::InputRef { sub_idx, .. } = &mut internal.kind {
                    if pick == 0 {
                        *sub_idx = rng.gen::<u16>();
                        return Ok(());
                    }
                    pick -= 1;
                }
            }
        }
    }
    Ok(())
}

/// Map an index 0..8 to a Direction (canonical order).
fn direction_from_idx(idx: u8) -> Direction {
    Direction::ALL[idx as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MutationConfig;
    use crate::contracts::NodeId;
    use crate::creature::founder::v3alpha1_founder_genome;
    use crate::creature::genome::{
        BackendDef, CreatureGenome, GraphBackendDef, GraphInternalNode, GraphNodeKind, NodeGenome,
        VmBackendDef, VmInstruction,
    };
    use crate::creature::parseability::ParseabilityGate;
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
            &mut r,
            &default_config(),
        )
        .unwrap();
        let after: usize = genome.nodes.iter().map(|n| n.input_refs.len()).sum();
        assert_eq!(after, before + 1);
    }

    #[test]
    fn remove_input_ref_decreases_count() {
        let mut genome = v3alpha1_founder_genome();
        let before: usize = genome.nodes.iter().map(|n| n.input_refs.len()).sum();
        assert!(before > 0, "founder must have input_refs");
        let mut r = rng(0);
        InputRefMutator::apply(
            &mut genome,
            InputRefOperator::Remove,
            &mut r,
            &default_config(),
        )
        .unwrap();
        let after: usize = genome.nodes.iter().map(|n| n.input_refs.len()).sum();
        assert_eq!(after, before - 1);
    }

    #[test]
    fn remove_input_ref_on_empty_returns_no_applicable_target() {
        let mut genome = v3alpha1_founder_genome();
        for node in &mut genome.nodes {
            node.input_refs.clear();
        }
        let mut r = rng(0);
        let result = InputRefMutator::apply(
            &mut genome,
            InputRefOperator::Remove,
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
            if InputRefMutator::apply(&mut g, InputRefOperator::Swap, &mut r, &default_config())
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

    #[test]
    fn random_input_reference_covers_all_categories() {
        use std::collections::HashSet;
        let mut categories: HashSet<String> = HashSet::new();
        for seed in 0u64..1000 {
            let mut r = rng(seed);
            let ir = random_input_reference(&mut r);
            let cat = match ir {
                InputReference::World(WorldInputKey::FoodHere) => "FoodHere".to_string(),
                InputReference::World(WorldInputKey::NeighborCellFood(_)) => {
                    "NeighborCellFood".to_string()
                }
                InputReference::World(WorldInputKey::NeighborCellBarrier(_)) => {
                    "NeighborCellBarrier".to_string()
                }
                InputReference::World(WorldInputKey::NeighborCellOccupied(_)) => {
                    "NeighborCellOccupied".to_string()
                }
                InputReference::World(WorldInputKey::AreaFoodSummary) => {
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
            };
            categories.insert(cat);
        }
        // 14 categories: 8 original + 6 extended perception families
        assert_eq!(
            categories.len(),
            14,
            "all 14 input reference categories must be reachable; got {:?}",
            categories
        );
    }

    #[test]
    fn random_input_reference_reaches_out_of_range_upstream_slot() {
        let mut saw_out_of_range_upstream_slot = false;
        for seed in 0u64..20_000 {
            let mut r = rng(seed);
            if let InputReference::UpstreamSlot(slot) = random_input_reference(&mut r) {
                if slot > 11 {
                    saw_out_of_range_upstream_slot = true;
                    break;
                }
            }
        }
        assert!(
            saw_out_of_range_upstream_slot,
            "input ref mutation surface must include out-of-range upstream slots"
        );
    }

    #[test]
    fn raw_field_mutation_can_set_out_of_range_upstream_slot() {
        let mut found_out_of_range = false;
        for seed in 0u64..512 {
            let mut genome = single_node_genome_with_input_ref(InputReference::UpstreamSlot(0));
            let mut r = rng(seed);
            InputRefMutator::apply(
                &mut genome,
                InputRefOperator::RawFieldMutation,
                &mut r,
                &default_config(),
            )
            .unwrap();
            if let InputReference::UpstreamSlot(slot) = genome.nodes[0].input_refs[0] {
                if slot > 11 {
                    found_out_of_range = true;
                    break;
                }
            }
        }
        assert!(
            found_out_of_range,
            "raw input-ref mutation must reach out-of-range upstream slots"
        );
    }

    #[test]
    fn input_ref_after_mutation_passes_parseability_gate() {
        let operators = [
            InputRefOperator::Add,
            InputRefOperator::Remove,
            InputRefOperator::Swap,
            InputRefOperator::RawFieldMutation,
        ];
        for (i, &op) in operators.iter().enumerate() {
            let mut genome = v3alpha1_founder_genome();
            let mut r = rng(i as u64 + 400);
            let _ = InputRefMutator::apply(&mut genome, op, &mut r, &default_config());
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

    #[test]
    fn random_non_increasing_never_returns_increasing() {
        use crate::mutation::types::ComplexityEffect;
        for seed in 0u64..200 {
            let mut r = rng(seed);
            if let Some(op) = InputRefOperator::random_non_increasing(&mut r) {
                assert_ne!(
                    op.complexity_effect(),
                    ComplexityEffect::Increasing,
                    "random_non_increasing returned Increasing operator {:?} at seed {}",
                    op,
                    seed
                );
            }
        }
    }

    #[test]
    fn random_non_increasing_covers_neutral_and_decreasing() {
        use crate::mutation::types::ComplexityEffect;
        let mut saw_neutral = false;
        let mut saw_decreasing = false;
        for seed in 0u64..1000 {
            let mut r = rng(seed);
            if let Some(op) = InputRefOperator::random_non_increasing(&mut r) {
                match op.complexity_effect() {
                    ComplexityEffect::Neutral => saw_neutral = true,
                    ComplexityEffect::Decreasing => saw_decreasing = true,
                    ComplexityEffect::Increasing => unreachable!(),
                }
            }
            if saw_neutral && saw_decreasing {
                break;
            }
        }
        assert!(saw_neutral, "must produce at least one neutral operator");
        assert!(
            saw_decreasing,
            "must produce at least one decreasing operator"
        );
    }

    #[test]
    fn random_decreasing_never_returns_non_decreasing() {
        use crate::mutation::types::ComplexityEffect;
        for seed in 0u64..200 {
            let mut r = rng(seed);
            if let Some(op) = InputRefOperator::random_decreasing(&mut r) {
                assert_eq!(
                    op.complexity_effect(),
                    ComplexityEffect::Decreasing,
                    "random_decreasing returned non-Decreasing operator {:?} at seed {}",
                    op,
                    seed
                );
            }
        }
    }

    #[test]
    fn random_decreasing_covers_all_decreasing_operators() {
        use std::collections::HashSet;
        let expected: HashSet<InputRefOperator> = InputRefOperator::ALL
            .iter()
            .copied()
            .filter(|op| op.complexity_effect().is_decreasing())
            .collect();
        let mut seen = HashSet::new();
        for seed in 0u64..2000 {
            let mut r = rng(seed);
            if let Some(op) = InputRefOperator::random_decreasing(&mut r) {
                seen.insert(op);
            }
            if seen == expected {
                break;
            }
        }
        assert_eq!(
            seen, expected,
            "random_decreasing must cover all Decreasing operators"
        );
    }

    // ── reindex_after_removal tests ──────────────────────────────────────

    fn graph_genome_with_input_refs(
        input_refs: Vec<InputReference>,
        nodes: Vec<GraphInternalNode>,
    ) -> CreatureGenome {
        CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs,
                backend_def: BackendDef::Graph(GraphBackendDef {
                    internal_nodes: nodes,
                }),
                targets: vec![],
            }],
        }
    }

    #[test]
    fn reindex_graph_decrements_refs_above_removed() {
        let genome = graph_genome_with_input_refs(
            vec![
                InputReference::World(WorldInputKey::FoodHere),
                InputReference::UpstreamSlot(0),
                InputReference::UpstreamSlot(1),
            ],
            vec![
                GraphInternalNode {
                    kind: GraphNodeKind::InputRef {
                        ref_idx: 0,
                        sub_idx: 0,
                    },
                    inputs: vec![],
                    plasticity: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::InputRef {
                        ref_idx: 2,
                        sub_idx: 0,
                    },
                    inputs: vec![],
                    plasticity: None,
                },
            ],
        );
        let mut g = genome;
        // Remove input ref at index 1 → ref_idx 0 stays, ref_idx 2 → 1
        reindex_after_removal(&mut g, 0, 1);
        if let BackendDef::Graph(ref gd) = g.nodes[0].backend_def {
            // ref_idx 0 < removed(1) → unchanged
            assert_eq!(
                gd.internal_nodes[0].kind,
                GraphNodeKind::InputRef {
                    ref_idx: 0,
                    sub_idx: 0
                }
            );
            // ref_idx 2 > removed(1) → decremented to 1
            assert_eq!(
                gd.internal_nodes[1].kind,
                GraphNodeKind::InputRef {
                    ref_idx: 1,
                    sub_idx: 0
                }
            );
        } else {
            panic!("expected Graph backend");
        }
    }

    #[test]
    fn reindex_graph_invalidates_removed_ref() {
        let genome = graph_genome_with_input_refs(
            vec![
                InputReference::World(WorldInputKey::FoodHere),
                InputReference::UpstreamSlot(0),
            ],
            vec![GraphInternalNode {
                kind: GraphNodeKind::InputRef {
                    ref_idx: 1,
                    sub_idx: 3,
                },
                inputs: vec![],
                plasticity: None,
            }],
        );
        let mut g = genome;
        // Remove the ref at index 1 → ref_idx 1 == removed → invalidate
        reindex_after_removal(&mut g, 0, 1);
        if let BackendDef::Graph(ref gd) = g.nodes[0].backend_def {
            assert_eq!(
                gd.internal_nodes[0].kind,
                GraphNodeKind::InputRef {
                    ref_idx: u16::MAX,
                    sub_idx: 3
                }
            );
        } else {
            panic!("expected Graph backend");
        }
    }

    #[test]
    fn reindex_vm_decrements_and_invalidates() {
        let mut genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![
                    InputReference::World(WorldInputKey::FoodHere),
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
        reindex_after_removal(&mut genome, 0, 1);
        if let BackendDef::Vm(ref vm) = genome.nodes[0].backend_def {
            // ref_idx 0 < 1 → unchanged
            assert!(matches!(
                vm.program[0],
                VmInstruction::ReadInput {
                    ref_idx: 0,
                    sub_idx: 0,
                    ..
                }
            ));
            // ref_idx 1 == removed → invalidated to u16::MAX
            assert!(matches!(
                vm.program[1],
                VmInstruction::ReadInput {
                    ref_idx: u16::MAX,
                    sub_idx: 0,
                    ..
                }
            ));
            // ref_idx 2 > 1 → decremented to 1
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
    fn remove_operator_calls_reindex() {
        // Build a genome with 2 input refs and a graph node referencing ref_idx 1.
        // After Remove removes ref at index 0, the graph node's ref_idx should be 0 (decremented).
        let genome = graph_genome_with_input_refs(
            vec![
                InputReference::World(WorldInputKey::FoodHere),
                InputReference::UpstreamSlot(0),
            ],
            vec![GraphInternalNode {
                kind: GraphNodeKind::InputRef {
                    ref_idx: 1,
                    sub_idx: 0,
                },
                inputs: vec![],
                plasticity: None,
            }],
        );
        // Force rng to pick node 0 and ref_idx 0 for removal.
        // With seed search, find one that removes index 0.
        let mut found = false;
        for seed in 0u64..200 {
            let mut g = genome.clone();
            let mut r = rng(seed);
            if InputRefMutator::apply(&mut g, InputRefOperator::Remove, &mut r, &default_config())
                .is_ok()
                && g.nodes[0].input_refs.len() == 1
            {
                // Removed one ref. Check if the remaining is UpstreamSlot(0)
                // meaning we removed index 0 (FoodHere).
                if g.nodes[0].input_refs[0] == InputReference::UpstreamSlot(0) {
                    // Ref at index 0 was removed → old ref_idx 1 should be 0 now.
                    if let BackendDef::Graph(ref gd) = g.nodes[0].backend_def {
                        assert_eq!(
                            gd.internal_nodes[0].kind,
                            GraphNodeKind::InputRef {
                                ref_idx: 0,
                                sub_idx: 0,
                            },
                            "Remove must call reindex to decrement ref_idx above removed"
                        );
                        found = true;
                        break;
                    }
                }
            }
        }
        assert!(found, "must find a seed that removes index 0");
    }

    #[test]
    fn reindex_noop_on_empty_graph_backend() {
        let mut genome = graph_genome_with_input_refs(
            vec![InputReference::World(WorldInputKey::FoodHere)],
            vec![], // no internal nodes
        );
        reindex_after_removal(&mut genome, 0, 0); // should not panic
    }

    #[test]
    fn reindex_noop_on_empty_vm_program() {
        let mut genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![InputReference::World(WorldInputKey::FoodHere)],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![], // empty program
                }),
                targets: vec![],
            }],
        };
        reindex_after_removal(&mut genome, 0, 0); // should not panic
    }

    #[test]
    fn reindex_graph_invalidates_all_matching_refs() {
        let mut genome = graph_genome_with_input_refs(
            vec![InputReference::World(WorldInputKey::FoodHere)],
            vec![
                GraphInternalNode {
                    kind: GraphNodeKind::InputRef {
                        ref_idx: 0,
                        sub_idx: 0,
                    },
                    inputs: vec![],
                    plasticity: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::InputRef {
                        ref_idx: 0,
                        sub_idx: 1,
                    },
                    inputs: vec![],
                    plasticity: None,
                },
            ],
        );
        reindex_after_removal(&mut genome, 0, 0);
        if let BackendDef::Graph(ref gd) = genome.nodes[0].backend_def {
            for (i, node) in gd.internal_nodes.iter().enumerate() {
                if let GraphNodeKind::InputRef { ref_idx, .. } = node.kind {
                    assert_eq!(
                        ref_idx,
                        u16::MAX,
                        "internal node {} should be invalidated",
                        i
                    );
                }
            }
        } else {
            panic!("expected Graph backend");
        }
    }

    // ── Phase 2b compound-aware tests ──────────────────────────────────

    #[test]
    fn add_compound_input_creates_fan_out_nodes() {
        // Build a graph genome with no input refs. Force Add to pick ActionQueue.
        // Since random_input_reference picks ActionQueue at idx 29, we search seeds.
        let mut found = false;
        for seed in 0u64..2000 {
            let mut genome = graph_genome_with_input_refs(vec![], vec![]);
            let mut r = rng(seed);
            InputRefMutator::apply(
                &mut genome,
                InputRefOperator::Add,
                &mut r,
                &default_config(),
            )
            .unwrap();
            if genome.nodes[0].input_refs.last() == Some(&InputReference::ActionQueue) {
                // ActionQueue was added — should have fan-out nodes
                if let BackendDef::Graph(ref gd) = genome.nodes[0].backend_def {
                    // default config: action_queue_cap=4 → 12 fan-out nodes
                    assert_eq!(
                        gd.internal_nodes.len(),
                        12,
                        "compound Add must create 4*3=12 fan-out InputRef nodes"
                    );
                    for (i, n) in gd.internal_nodes.iter().enumerate() {
                        assert_eq!(
                            n.kind,
                            GraphNodeKind::InputRef {
                                ref_idx: 0,
                                sub_idx: i as u16,
                            }
                        );
                    }
                    found = true;
                    break;
                }
            }
        }
        assert!(found, "must find a seed producing ActionQueue input ref");
    }

    #[test]
    fn swap_to_compound_creates_fan_out_nodes() {
        // Start with a scalar input ref, swap to ActionQueue.
        let mut found = false;
        for seed in 0u64..2000 {
            let mut genome = graph_genome_with_input_refs(
                vec![InputReference::World(WorldInputKey::FoodHere)],
                vec![],
            );
            let mut r = rng(seed);
            InputRefMutator::apply(
                &mut genome,
                InputRefOperator::Swap,
                &mut r,
                &default_config(),
            )
            .unwrap();
            if genome.nodes[0].input_refs[0] == InputReference::ActionQueue {
                if let BackendDef::Graph(ref gd) = genome.nodes[0].backend_def {
                    assert_eq!(
                        gd.internal_nodes.len(),
                        12,
                        "compound Swap must create fan-out nodes"
                    );
                    found = true;
                    break;
                }
            }
        }
        assert!(found, "must find a seed swapping to ActionQueue");
    }

    #[test]
    fn raw_field_mutation_mutates_sub_idx_on_graph_input_ref() {
        // Create a genome with a graph InputRef node — RawFieldMutation should
        // be able to mutate its sub_idx.
        let mut found_changed = false;
        for seed in 0u64..500 {
            let mut genome = graph_genome_with_input_refs(
                vec![InputReference::ActionQueue],
                vec![GraphInternalNode {
                    kind: GraphNodeKind::InputRef {
                        ref_idx: 0,
                        sub_idx: 5,
                    },
                    inputs: vec![],
                    plasticity: None,
                }],
            );
            let mut r = rng(seed);
            let _ = InputRefMutator::apply(
                &mut genome,
                InputRefOperator::RawFieldMutation,
                &mut r,
                &default_config(),
            );
            if let BackendDef::Graph(ref gd) = genome.nodes[0].backend_def {
                if let GraphNodeKind::InputRef { sub_idx, .. } = gd.internal_nodes[0].kind {
                    if sub_idx != 5 {
                        found_changed = true;
                        break;
                    }
                }
            }
        }
        assert!(
            found_changed,
            "RawFieldMutation must be able to mutate sub_idx on graph InputRef nodes"
        );
    }

    #[test]
    fn extended_perception_families_reachable_in_pool() {
        let mut found = [false; 6]; // Food, Barrier, Occupancy, Core, Vitals, Identity
        for seed in 0u64..5000 {
            let mut r = rng(seed);
            match random_input_reference(&mut r) {
                InputReference::World(WorldInputKey::AreaFoodSummary) => found[0] = true,
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
    fn add_extended_perception_compound_creates_correct_fan_out() {
        // Verify that adding a NearbyCreatureCore input creates 16 fan-out nodes
        let mut found = false;
        for seed in 0u64..5000 {
            let mut genome = graph_genome_with_input_refs(vec![], vec![]);
            let mut r = rng(seed);
            InputRefMutator::apply(
                &mut genome,
                InputRefOperator::Add,
                &mut r,
                &default_config(),
            )
            .unwrap();
            if genome.nodes[0].input_refs.last()
                == Some(&InputReference::World(WorldInputKey::NearbyCreatureCore))
            {
                if let BackendDef::Graph(ref gd) = genome.nodes[0].backend_def {
                    assert_eq!(
                        gd.internal_nodes.len(),
                        16,
                        "NearbyCreatureCore compound Add must create 16 fan-out nodes"
                    );
                    found = true;
                    break;
                }
            }
        }
        assert!(found, "must find a seed producing NearbyCreatureCore");
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
}
