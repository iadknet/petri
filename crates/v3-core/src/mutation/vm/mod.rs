use rand::Rng;

use crate::config::MutationConfig;
use crate::creature::genome::{BackendDef, CreatureGenome};
use crate::mutation::reachability::biased_select_from;
use crate::mutation::types::{MutationSkipReason, TargetReachability};

pub(crate) mod operators;
use operators::*;
#[cfg(test)]
pub(crate) use operators::{
    insert_new_instruction_with_reference_repair, splice_program_with_reference_repair,
    SpliceInstruction,
};

/// VM mutation operator variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VmOperator {
    VmConstantMutation,
    VmInstructionMutation,
    VmDeleteInstruction,
    VmRegisterCountMutation,
    VmInstructionRawFieldMutation,
    VmCopyInstructionBlock,
    VmCopyInstructionBlockRemapped,
    VmCopyConstantBlock,
    VmCopyGeneBackwardSlice,
    VmCopyGeneForwardSlice,
    VmInsertReadStoreMotif,
    VmInsertReadBidMotif,
    VmInsertLoadCompareMotif,
    VmMutateSlotAddress,
    VmMutatePairedSlotAddress,
}

impl VmOperator {
    pub const ALL: [Self; 15] = [
        Self::VmConstantMutation,
        Self::VmInstructionMutation,
        Self::VmDeleteInstruction,
        Self::VmRegisterCountMutation,
        Self::VmInstructionRawFieldMutation,
        Self::VmCopyInstructionBlock,
        Self::VmCopyInstructionBlockRemapped,
        Self::VmCopyConstantBlock,
        Self::VmCopyGeneBackwardSlice,
        Self::VmCopyGeneForwardSlice,
        Self::VmInsertReadStoreMotif,
        Self::VmInsertReadBidMotif,
        Self::VmInsertLoadCompareMotif,
        Self::VmMutateSlotAddress,
        Self::VmMutatePairedSlotAddress,
    ];

    /// Per-operator weight reflecting impact tier.
    /// 4 = refinement, 2 = moderate, 1 = structural.
    #[must_use]
    pub const fn weight(self) -> u8 {
        match self {
            Self::VmConstantMutation => 4,
            Self::VmInstructionMutation => 2,
            Self::VmDeleteInstruction => 1,
            Self::VmRegisterCountMutation => 2,
            Self::VmInstructionRawFieldMutation => 4,
            Self::VmCopyInstructionBlock => 1,
            Self::VmCopyInstructionBlockRemapped => 1,
            Self::VmCopyConstantBlock => 1,
            Self::VmCopyGeneBackwardSlice => 1,
            Self::VmCopyGeneForwardSlice => 1,
            Self::VmInsertReadStoreMotif => 2,
            Self::VmInsertReadBidMotif => 2,
            Self::VmInsertLoadCompareMotif => 2,
            Self::VmMutateSlotAddress => 4,
            Self::VmMutatePairedSlotAddress => 4,
        }
    }

    const TOTAL_WEIGHT: u16 = {
        assert!(
            Self::ALL.len() == 15,
            "ALL must cover every VmOperator variant"
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
            Self::VmCopyInstructionBlock
            | Self::VmCopyInstructionBlockRemapped
            | Self::VmCopyConstantBlock
            | Self::VmCopyGeneBackwardSlice
            | Self::VmCopyGeneForwardSlice
            | Self::VmInsertReadStoreMotif
            | Self::VmInsertReadBidMotif
            | Self::VmInsertLoadCompareMotif => ComplexityEffect::Increasing,
            Self::VmDeleteInstruction => ComplexityEffect::Decreasing,
            Self::VmConstantMutation
            | Self::VmInstructionMutation
            | Self::VmRegisterCountMutation
            | Self::VmInstructionRawFieldMutation
            | Self::VmMutateSlotAddress
            | Self::VmMutatePairedSlotAddress => ComplexityEffect::Neutral,
        }
    }

    /// Pick a random VM operator weighted by impact tier.
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

/// VM domain mutator.
pub struct VmMutator;

impl VmMutator {
    /// Apply a VM operator to the genome.
    ///
    /// Returns `Ok(TargetReachability)` on success, or `Err(MutationSkipReason::NoApplicableTarget)`
    /// if the genome contains no VM-backend nodes.
    pub fn apply(
        genome: &mut CreatureGenome,
        op: VmOperator,
        reachable_nodes: &[usize],
        bias: f64,
        rng: &mut impl Rng,
        config: &MutationConfig,
    ) -> Result<TargetReachability, MutationSkipReason> {
        // Pre-guard: must have at least one VM-backend node.
        let vm_indices: Vec<usize> = genome
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| matches!(n.backend_def, BackendDef::Vm(_)))
            .map(|(i, _)| i)
            .collect();
        if vm_indices.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }

        let (node_idx, reachability) = biased_select_from(&vm_indices, reachable_nodes, bias, rng)
            .ok_or(MutationSkipReason::NoApplicableTarget)?;
        let result = match op {
            VmOperator::VmConstantMutation => apply_constant_mutation(genome, node_idx, rng),
            VmOperator::VmInstructionMutation => apply_instruction_mutation(genome, node_idx, rng),
            VmOperator::VmDeleteInstruction => apply_delete_instruction(genome, node_idx, rng),
            VmOperator::VmRegisterCountMutation => {
                apply_register_count_mutation(genome, node_idx, rng)
            }
            VmOperator::VmInstructionRawFieldMutation => {
                apply_instruction_raw_field_mutation(genome, node_idx, rng, config)
            }
            VmOperator::VmCopyInstructionBlock => {
                apply_copy_instruction_block(genome, node_idx, rng)
            }
            VmOperator::VmCopyInstructionBlockRemapped => {
                apply_copy_instruction_block_remapped(genome, node_idx, rng)
            }
            VmOperator::VmCopyConstantBlock => apply_copy_constant_block(genome, node_idx, rng),
            VmOperator::VmCopyGeneBackwardSlice => {
                apply_copy_gene_backward_slice(genome, node_idx, rng)
            }
            VmOperator::VmCopyGeneForwardSlice => {
                apply_copy_gene_forward_slice(genome, node_idx, rng)
            }
            VmOperator::VmInsertReadStoreMotif => {
                apply_insert_read_store_motif(genome, node_idx, rng)
            }
            VmOperator::VmInsertReadBidMotif => apply_insert_read_bid_motif(genome, node_idx, rng),
            VmOperator::VmInsertLoadCompareMotif => {
                apply_insert_load_compare_motif(genome, node_idx, rng)
            }
            VmOperator::VmMutateSlotAddress => apply_mutate_slot_address(genome, node_idx, rng),
            VmOperator::VmMutatePairedSlotAddress => {
                apply_mutate_paired_slot_address(genome, node_idx, rng)
            }
        };
        result.map(|()| reachability)
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod f08_tests;
