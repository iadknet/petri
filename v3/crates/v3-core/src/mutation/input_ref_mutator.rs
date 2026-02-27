use rand::Rng;

use crate::contracts::{
    Direction, DynamicIntrospectionKey, InputReference, StaticIntrospectionKey, WorldInputKey,
};
use crate::creature::genome::CreatureGenome;
use crate::mutation::types::MutationSkipReason;

/// Input reference mutation operator variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputRefOperator {
    Add,
    Remove,
    Swap,
}

impl InputRefOperator {
    /// Pick a random input ref operator uniformly.
    pub fn random(rng: &mut impl Rng) -> Self {
        match rng.gen_range(0u8..3) {
            0 => Self::Add,
            1 => Self::Remove,
            _ => Self::Swap,
        }
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
            InputRefOperator::Add => {
                let node_idx = rng.gen_range(0..genome.nodes.len());
                genome.nodes[node_idx]
                    .input_refs
                    .push(random_input_reference(rng));
                Ok(())
            }
            InputRefOperator::Remove => {
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
            InputRefOperator::Swap => {
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
        }
    }
}

/// Generate a random input reference from the full set of 37 possible values.
///
/// Distribution: FoodHere (1) + NeighborCellFood (8) + NeighborCellBarrier (8) +
/// NeighborCellOccupied (8) + StaticIntrospection (2) + DynamicIntrospection (2) +
/// UpstreamSlot(0..8) (8) = 37 total.
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
        _ => InputReference::UpstreamSlot((idx - 29) as usize),
    }
}

/// Map an index 0..8 to a Direction (canonical order).
fn direction_from_idx(idx: u8) -> Direction {
    Direction::ALL[idx as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creature::founder::v3alpha1_founder_genome;
    use crate::creature::parseability::ParseabilityGate;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    fn rng(seed: u64) -> SmallRng {
        SmallRng::seed_from_u64(seed)
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
    fn input_ref_after_mutation_passes_parseability_gate() {
        let operators = [
            InputRefOperator::Add,
            InputRefOperator::Remove,
            InputRefOperator::Swap,
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
}
