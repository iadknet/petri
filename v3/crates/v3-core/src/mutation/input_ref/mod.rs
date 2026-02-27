use rand::Rng;

use crate::contracts::{
    Direction, DynamicIntrospectionKey, InputReference, StaticIntrospectionKey, WorldInputKey,
};
use crate::creature::genome::CreatureGenome;
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
        let mut sum = 0u16;
        let mut i = 0;
        while i < Self::ALL.len() {
            sum += Self::ALL[i].weight() as u16;
            i += 1;
        }
        sum
    };

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
    ) -> Result<(), MutationSkipReason> {
        if genome.nodes.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }

        match op {
            InputRefOperator::Add => apply_add(genome, rng),
            InputRefOperator::Remove => apply_remove(genome, rng),
            InputRefOperator::Swap => apply_swap(genome, rng),
            InputRefOperator::RawFieldMutation => apply_raw_field_mutation(genome, rng),
        }
    }
}

fn apply_add(genome: &mut CreatureGenome, rng: &mut impl Rng) -> Result<(), MutationSkipReason> {
    let node_idx = rng.gen_range(0..genome.nodes.len());
    genome.nodes[node_idx]
        .input_refs
        .push(random_input_reference(rng));
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
    Ok(())
}

fn apply_swap(genome: &mut CreatureGenome, rng: &mut impl Rng) -> Result<(), MutationSkipReason> {
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
    genome.nodes[node_idx].input_refs[ref_idx] = random_input_reference(rng);
    Ok(())
}

/// Generate a random input reference from the full set of 37 possible values.
///
/// Distribution: FoodHere (1) + NeighborCellFood (8) + NeighborCellBarrier (8) +
/// NeighborCellOccupied (8) + StaticIntrospection (2) + DynamicIntrospection (2) +
/// UpstreamSlot(usize) raw values (8 weighted slots) = 37 total.
fn random_input_reference(rng: &mut impl Rng) -> InputReference {
    let idx = rng.gen_range(0u8..37);
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
        _ => InputReference::UpstreamSlot(rng.gen::<u8>() as usize),
    }
}

fn apply_raw_field_mutation(
    genome: &mut CreatureGenome,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let mut eligible_count: usize = 0;
    for node in &genome.nodes {
        eligible_count += node
            .input_refs
            .iter()
            .filter(|r| matches!(r, InputReference::UpstreamSlot(_)))
            .count();
    }
    if eligible_count == 0 {
        return Err(MutationSkipReason::NoApplicableTarget);
    }

    let mut pick = rng.gen_range(0..eligible_count);
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
    Ok(())
}

/// Map an index 0..8 to a Direction (canonical order).
fn direction_from_idx(idx: u8) -> Direction {
    Direction::ALL[idx as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::NodeId;
    use crate::creature::founder::v3alpha1_founder_genome;
    use crate::creature::genome::{
        BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
    };
    use crate::creature::parseability::ParseabilityGate;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    fn rng(seed: u64) -> SmallRng {
        SmallRng::seed_from_u64(seed)
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
        InputRefMutator::apply(&mut genome, InputRefOperator::Add, &mut r).unwrap();
        let after: usize = genome.nodes.iter().map(|n| n.input_refs.len()).sum();
        assert_eq!(after, before + 1);
    }

    #[test]
    fn remove_input_ref_decreases_count() {
        let mut genome = v3alpha1_founder_genome();
        let before: usize = genome.nodes.iter().map(|n| n.input_refs.len()).sum();
        assert!(before > 0, "founder must have input_refs");
        let mut r = rng(0);
        InputRefMutator::apply(&mut genome, InputRefOperator::Remove, &mut r).unwrap();
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
        let result = InputRefMutator::apply(&mut genome, InputRefOperator::Remove, &mut r);
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
            if InputRefMutator::apply(&mut g, InputRefOperator::Swap, &mut r).is_ok() {
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
                InputReference::StaticIntrospection(_) => "StaticIntrospection".to_string(),
                InputReference::DynamicIntrospection(_) => "DynamicIntrospection".to_string(),
                InputReference::UpstreamSlot(_) => "UpstreamSlot".to_string(),
            };
            categories.insert(cat);
        }
        assert_eq!(
            categories.len(),
            7,
            "all 7 input reference categories must be reachable; got {:?}",
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
            InputRefMutator::apply(&mut genome, InputRefOperator::RawFieldMutation, &mut r)
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
            let _ = InputRefMutator::apply(&mut genome, op, &mut r);
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
}
