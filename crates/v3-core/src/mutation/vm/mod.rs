use rand::Rng;

use crate::config::MutationConfig;
use crate::contracts::InputReference;
use crate::creature::genome::analysis::{vm_is_output_instruction, vm_register_write};
use crate::creature::genome::{BackendDef, CreatureGenome, VmBackendDef};
use crate::mutation::reachability::TargetSelector;
use crate::mutation::types::{MutationSkipReason, TargetReachability};

pub(crate) mod operators;
use operators::*;
#[cfg(test)]
pub(crate) use operators::{
    insert_new_instruction_with_reference_repair, mutate_one_instruction_field,
    splice_program_with_reference_repair, SpliceInstruction,
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

impl VmOperator {
    /// Whether this operator has a site to apply to on one VM-backend node.
    /// Each arm delegates to the same enumeration the operator draws its
    /// target from, so a node this accepts never skips at application.
    fn applies_to(self, vm: &VmBackendDef, input_refs: &[InputReference]) -> bool {
        match self {
            // Infallible on any VM def: the constant pool is seeded when
            // empty, instruction mutation inserts into an empty program, the
            // load/compare motif needs no existing site, and an insert-only
            // splice always repairs.
            Self::VmConstantMutation
            | Self::VmInstructionMutation
            | Self::VmInsertLoadCompareMotif => true,
            Self::VmDeleteInstruction => vm.program.len() > 1,
            Self::VmRegisterCountMutation => {
                let (grow, shrink) = register_count_moves(vm);
                grow || shrink
            }
            Self::VmInstructionRawFieldMutation => vm.program.iter().any(has_mutable_field),
            Self::VmCopyInstructionBlock | Self::VmCopyInstructionBlockRemapped => {
                !vm.program.is_empty()
            }
            Self::VmCopyConstantBlock => !vm.constants.is_empty(),
            Self::VmCopyGeneBackwardSlice => vm.program.iter().any(vm_is_output_instruction),
            Self::VmCopyGeneForwardSlice => vm
                .program
                .iter()
                .any(|instr| vm_register_write(instr).is_some()),
            Self::VmInsertReadStoreMotif | Self::VmInsertReadBidMotif => !input_refs.is_empty(),
            Self::VmMutateSlotAddress => vm.program.iter().any(is_slot_instruction),
            Self::VmMutatePairedSlotAddress => !paired_slot_groups(&vm.program).is_empty(),
        }
    }
}

/// VM domain mutator.
pub struct VmMutator;

impl VmMutator {
    /// The VM-backend node indices `op` can apply to, ascending.
    ///
    /// Selection draws from this set rather than from every VM-backend node,
    /// so a refinement operator reaches a module that has a suitable site
    /// instead of landing on one that does not (T13.F03).
    pub(crate) fn applicable_indices(genome: &CreatureGenome, op: VmOperator) -> Vec<usize> {
        genome
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| match &n.backend_def {
                BackendDef::Vm(vm) => op.applies_to(vm, &n.input_refs),
                BackendDef::Graph(_) => false,
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// Apply a VM operator to the genome.
    ///
    /// Returns `Ok(TargetReachability)` on success, or `Err(MutationSkipReason::NoApplicableTarget)`
    /// if no VM-backend node has a site this operator can apply to. The skip
    /// happens before the draw, so the engine records it as no-eligible-node
    /// with no selected target.
    pub fn apply(
        genome: &mut CreatureGenome,
        op: VmOperator,
        targets: &mut TargetSelector<'_>,
        rng: &mut impl Rng,
        config: &MutationConfig,
    ) -> Result<TargetReachability, MutationSkipReason> {
        let applicable = Self::applicable_indices(genome, op);
        if applicable.is_empty() {
            return Err(MutationSkipReason::NoApplicableTarget);
        }

        let (node_idx, reachability) = targets
            .select(&applicable, rng)
            .ok_or(MutationSkipReason::NoApplicableTarget)?;
        Self::apply_to_node(genome, op, node_idx, rng, config).map(|()| reachability)
    }

    /// Dispatch `op` onto one already-selected node.
    pub(crate) fn apply_to_node(
        genome: &mut CreatureGenome,
        op: VmOperator,
        node_idx: usize,
        rng: &mut impl Rng,
        config: &MutationConfig,
    ) -> Result<(), MutationSkipReason> {
        match op {
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
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod f08_tests;
