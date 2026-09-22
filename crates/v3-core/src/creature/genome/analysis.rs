//! Genome analysis utilities: gene detection via backward/forward slicing.
//!
//! Pure analysis functions that identify functional gene boundaries within
//! VM programs and graph backends. Used by mutation operators, and available
//! for sexual reproduction, visualization, and selective reproduction.

use rand::Rng;

use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

use crate::contracts::NodeId;

use super::cgp_analysis;
use super::{BackendDef, CreatureGenome, VmInstruction};

/// A detected functional gene: a set of indices within a backend array.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DetectedGene {
    /// Indices within the backend array (VM program or graph internal_nodes),
    /// in ascending order.
    pub indices: Vec<usize>,
}

// ── VM analysis helpers ─────────────────────────────────────────────────────

/// Returns the `dst` register if the instruction writes one, `None` otherwise.
pub fn vm_register_write(instr: &VmInstruction) -> Option<u8> {
    match instr {
        VmInstruction::LoadConst { dst, .. }
        | VmInstruction::Move { dst, .. }
        | VmInstruction::Add { dst, .. }
        | VmInstruction::Sub { dst, .. }
        | VmInstruction::Mul { dst, .. }
        | VmInstruction::Div { dst, .. }
        | VmInstruction::Min { dst, .. }
        | VmInstruction::Max { dst, .. }
        | VmInstruction::Abs { dst, .. }
        | VmInstruction::Neg { dst, .. }
        | VmInstruction::Clamp01 { dst, .. }
        | VmInstruction::CmpGt { dst, .. }
        | VmInstruction::CmpLt { dst, .. }
        | VmInstruction::CmpEq { dst, .. }
        | VmInstruction::And { dst, .. }
        | VmInstruction::Or { dst, .. }
        | VmInstruction::Not { dst, .. }
        | VmInstruction::ToI32 { dst, .. }
        | VmInstruction::ToU8 { dst, .. }
        | VmInstruction::ToBool { dst, .. }
        | VmInstruction::ReadInput { dst, .. }
        | VmInstruction::ReadActionQueueLength { dst, .. }
        | VmInstruction::ReadActionQueueType { dst, .. }
        | VmInstruction::ReadActionQueueParam { dst, .. }
        | VmInstruction::LoadSlot { dst, .. }
        | VmInstruction::LoadSlotImm { dst, .. }
        | VmInstruction::LoadSlotPrev { dst, .. } => Some(*dst),
        VmInstruction::Noop
        | VmInstruction::Halt
        | VmInstruction::Jump { .. }
        | VmInstruction::JumpIfZero { .. }
        | VmInstruction::WriteInternalPayload { .. }
        | VmInstruction::WriteWorldActionMeta { .. }
        | VmInstruction::WriteDirectionBid { .. }
        | VmInstruction::AddVote { .. }
        | VmInstruction::PushAction { .. }
        | VmInstruction::PopAction
        | VmInstruction::ExecuteActionQueue
        | VmInstruction::WriteRouteGate { .. }
        | VmInstruction::SetPriorityBid { .. }
        | VmInstruction::StoreSlot { .. }
        | VmInstruction::StoreSlotImm { .. }
        | VmInstruction::ClearSlot { .. } => None,
    }
}

/// Safe bit-shift: returns `1u32 << reg` if `reg < 32`, else `0`.
/// Prevents panics when raw field mutations produce out-of-range register indices.
#[inline]
pub fn vm_reg_bit(reg: u8) -> u32 {
    if reg < 32 {
        1u32 << reg
    } else {
        0
    }
}

/// Returns a `u32` bitmask of registers read by the instruction.
/// Bit `i` set means register `i` is read.
pub fn vm_register_read_mask(instr: &VmInstruction) -> u32 {
    match instr {
        VmInstruction::Noop
        | VmInstruction::Halt
        | VmInstruction::Jump { .. }
        | VmInstruction::PopAction
        | VmInstruction::ExecuteActionQueue => 0,
        VmInstruction::LoadConst { .. }
        | VmInstruction::LoadSlotImm { .. }
        | VmInstruction::LoadSlotPrev { .. }
        | VmInstruction::ClearSlot { .. }
        | VmInstruction::ReadActionQueueLength { .. } => 0,
        VmInstruction::PushAction { .. } => 0,
        VmInstruction::Move { src, .. }
        | VmInstruction::Abs { src, .. }
        | VmInstruction::Neg { src, .. }
        | VmInstruction::Clamp01 { src, .. }
        | VmInstruction::Not { src, .. }
        | VmInstruction::ToI32 { src, .. }
        | VmInstruction::ToU8 { src, .. }
        | VmInstruction::ToBool { src, .. } => vm_reg_bit(*src),
        VmInstruction::ReadInput { .. } => 0,
        VmInstruction::Add { a, b, .. }
        | VmInstruction::Sub { a, b, .. }
        | VmInstruction::Mul { a, b, .. }
        | VmInstruction::Div { a, b, .. }
        | VmInstruction::Min { a, b, .. }
        | VmInstruction::Max { a, b, .. }
        | VmInstruction::CmpGt { a, b, .. }
        | VmInstruction::CmpLt { a, b, .. }
        | VmInstruction::And { a, b, .. }
        | VmInstruction::Or { a, b, .. } => vm_reg_bit(*a) | vm_reg_bit(*b),
        VmInstruction::CmpEq { a, b, eps, .. } => {
            vm_reg_bit(*a) | vm_reg_bit(*b) | vm_reg_bit(*eps)
        }
        VmInstruction::JumpIfZero { cond, .. } => vm_reg_bit(*cond),
        VmInstruction::WriteInternalPayload { src, .. }
        | VmInstruction::WriteWorldActionMeta { src, .. }
        | VmInstruction::WriteDirectionBid { src, .. }
        | VmInstruction::AddVote { src, .. }
        | VmInstruction::WriteRouteGate { src, .. }
        | VmInstruction::SetPriorityBid { src }
        | VmInstruction::StoreSlotImm { src, .. } => vm_reg_bit(*src),
        VmInstruction::LoadSlot { slot_reg, .. } => vm_reg_bit(*slot_reg),
        VmInstruction::StoreSlot { slot_reg, src } => vm_reg_bit(*slot_reg) | vm_reg_bit(*src),
        VmInstruction::ReadActionQueueType { index_src, .. } => vm_reg_bit(*index_src),
        VmInstruction::ReadActionQueueParam { index_src, .. } => vm_reg_bit(*index_src),
    }
}

/// Returns true if the instruction is an "output" (side-effecting write).
pub fn vm_is_output_instruction(instr: &VmInstruction) -> bool {
    matches!(
        instr,
        VmInstruction::WriteInternalPayload { .. }
            | VmInstruction::WriteWorldActionMeta { .. }
            | VmInstruction::WriteDirectionBid { .. }
            | VmInstruction::AddVote { .. }
            | VmInstruction::PushAction { .. }
            | VmInstruction::PopAction
            | VmInstruction::ExecuteActionQueue
            | VmInstruction::WriteRouteGate { .. }
            | VmInstruction::SetPriorityBid { .. }
            | VmInstruction::StoreSlot { .. }
            | VmInstruction::StoreSlotImm { .. }
            | VmInstruction::ClearSlot { .. }
    )
}

// ── VM slice functions ──────────────────────────────────────────────────────

/// Backward-slice from an anchor instruction (must be an output instruction).
///
/// Traces register dependencies backward through the program, collecting
/// all instructions that contribute to the anchor's inputs. Returns indices
/// in ascending order, capped at 32 instructions.
#[must_use]
pub fn vm_backward_slice(program: &[VmInstruction], anchor_idx: usize) -> Option<DetectedGene> {
    if anchor_idx >= program.len() || !vm_is_output_instruction(&program[anchor_idx]) {
        return None;
    }
    let mut needed: u32 = vm_register_read_mask(&program[anchor_idx]);
    let mut gene_indices = Vec::with_capacity(32);
    gene_indices.push(anchor_idx);
    for i in (0..anchor_idx).rev() {
        if let Some(dst) = vm_register_write(&program[i]) {
            if needed & vm_reg_bit(dst) != 0 {
                needed |= vm_register_read_mask(&program[i]);
                gene_indices.push(i);
                if gene_indices.len() >= 32 {
                    break;
                }
            }
        }
    }
    gene_indices.reverse();
    Some(DetectedGene {
        indices: gene_indices,
    })
}

/// Backward-slice from a randomly chosen output instruction.
#[must_use]
pub fn vm_backward_slice_random(
    program: &[VmInstruction],
    rng: &mut impl Rng,
) -> Option<DetectedGene> {
    let outputs: Vec<usize> = program
        .iter()
        .enumerate()
        .filter(|(_, instr)| vm_is_output_instruction(instr))
        .map(|(i, _)| i)
        .collect();
    if outputs.is_empty() {
        return None;
    }
    let anchor = outputs[rng.gen_range(0..outputs.len())];
    vm_backward_slice(program, anchor)
}

/// Forward-slice from a seed instruction (must write a register).
///
/// Traces register dependencies forward through the program, collecting
/// all instructions that read from registers produced by the growing set.
/// Returns indices in ascending order, capped at 32 instructions.
#[must_use]
pub fn vm_forward_slice(program: &[VmInstruction], seed_idx: usize) -> Option<DetectedGene> {
    if seed_idx >= program.len() {
        return None;
    }
    let seed_dst = vm_register_write(&program[seed_idx])?;
    let mut produced: u32 = vm_reg_bit(seed_dst);
    let mut gene_indices = Vec::with_capacity(32);
    gene_indices.push(seed_idx);
    for (i, instr) in program.iter().enumerate().skip(seed_idx + 1) {
        if vm_register_read_mask(instr) & produced != 0 {
            if let Some(dst) = vm_register_write(instr) {
                produced |= vm_reg_bit(dst);
            }
            gene_indices.push(i);
            if gene_indices.len() >= 32 {
                break;
            }
        }
    }
    Some(DetectedGene {
        indices: gene_indices,
    })
}

/// Forward-slice from a randomly chosen register-writing instruction.
#[must_use]
pub fn vm_forward_slice_random(
    program: &[VmInstruction],
    rng: &mut impl Rng,
) -> Option<DetectedGene> {
    let writers: Vec<usize> = program
        .iter()
        .enumerate()
        .filter(|(_, instr)| vm_register_write(instr).is_some())
        .map(|(i, _)| i)
        .collect();
    if writers.is_empty() {
        return None;
    }
    let seed_idx = writers[rng.gen_range(0..writers.len())];
    vm_forward_slice(program, seed_idx)
}

/// The reachable mesh nodes that lie on a cycle: every node the mesh can
/// route from `entry_node_id` that can route back to itself through one or
/// more targets (a self-target counts). Dangling targets and cycles the entry
/// never reaches are excluded (T19.F02).
#[must_use]
pub fn mesh_cycle_nodes(genome: &CreatureGenome) -> BTreeSet<NodeId> {
    let find_idx = |node_id| genome.nodes.iter().position(|n| n.node_id == node_id);
    let reachable_targets = |idx: usize| {
        genome.nodes[idx]
            .targets
            .iter()
            .filter_map(|target| find_idx(target.target_id))
    };
    mesh_reachable_nodes(genome)
        .into_iter()
        .filter(|&origin| {
            let mut seen = vec![false; genome.nodes.len()];
            let mut queue: VecDeque<usize> = reachable_targets(origin).collect();
            while let Some(idx) = queue.pop_front() {
                if idx == origin {
                    return true;
                }
                if !seen[idx] {
                    seen[idx] = true;
                    queue.extend(reachable_targets(idx));
                }
            }
            false
        })
        .map(|idx| genome.nodes[idx].node_id)
        .collect()
}

// ── Mesh reachability analysis ──────────────────────────────────────────────

/// BFS from `entry_node_id` following all targets, returning sorted
/// indices into `genome.nodes` for reachable nodes.
///
/// Identifies which mesh nodes are "live" vs unreachable "junk DNA".
#[must_use]
pub fn mesh_reachable_nodes(genome: &CreatureGenome) -> Vec<usize> {
    // Build index: node_id -> position in genome.nodes
    let find_idx = |node_id| genome.nodes.iter().position(|n| n.node_id == node_id);

    let entry_idx = match find_idx(genome.entry_node_id) {
        Some(idx) => idx,
        None => return vec![],
    };

    let mut visited = vec![false; genome.nodes.len()];
    visited[entry_idx] = true;
    let mut queue = VecDeque::new();
    queue.push_back(entry_idx);

    while let Some(idx) = queue.pop_front() {
        for target in &genome.nodes[idx].targets {
            if let Some(target_idx) = find_idx(target.target_id) {
                if !visited[target_idx] {
                    visited[target_idx] = true;
                    queue.push_back(target_idx);
                }
            }
        }
    }

    visited
        .iter()
        .enumerate()
        .filter(|(_, &v)| v)
        .map(|(i, _)| i)
        .collect()
}

/// Backward-slice from an anchor node at the mesh level.
///
/// Finds all nodes whose `targets` transitively reach the anchor node
/// (the "feeding pipeline"). Always includes the anchor itself. Returns
/// sorted indices capped at `max_size`.
#[must_use]
pub fn mesh_backward_slice(
    genome: &CreatureGenome,
    anchor_idx: usize,
    max_size: usize,
) -> Option<DetectedGene> {
    if anchor_idx >= genome.nodes.len() {
        return None;
    }

    // Pre-build node_id -> index map for O(1) lookups
    let node_id_to_idx: HashMap<NodeId, usize> = genome
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.node_id, i))
        .collect();

    let mut included = vec![false; genome.nodes.len()];
    included[anchor_idx] = true;
    let mut count = 1usize;

    // Fixpoint: find nodes whose targets contain any included node's node_id
    loop {
        let mut changed = false;
        for (i, node) in genome.nodes.iter().enumerate() {
            if included[i] || count >= max_size {
                continue;
            }
            let targets_included = node.targets.iter().any(|target| {
                node_id_to_idx
                    .get(&target.target_id)
                    .is_some_and(|&j| included[j])
            });
            if targets_included {
                included[i] = true;
                count += 1;
                changed = true;
                if count >= max_size {
                    break;
                }
            }
        }
        if !changed {
            break;
        }
    }

    let indices: Vec<usize> = included
        .iter()
        .enumerate()
        .filter(|(_, &inc)| inc)
        .map(|(i, _)| i)
        .collect();
    Some(DetectedGene { indices })
}

/// Forward-slice from a seed node at the mesh level.
///
/// BFS forward from the seed through `targets` edges, collecting the
/// downstream subtree. Always includes the seed itself. Returns sorted
/// indices capped at `max_size`.
#[must_use]
pub fn mesh_forward_slice(
    genome: &CreatureGenome,
    seed_idx: usize,
    max_size: usize,
) -> Option<DetectedGene> {
    if seed_idx >= genome.nodes.len() {
        return None;
    }

    let node_id_to_idx: HashMap<NodeId, usize> = genome
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.node_id, i))
        .collect();

    let mut visited = vec![false; genome.nodes.len()];
    visited[seed_idx] = true;
    let mut queue = VecDeque::new();
    queue.push_back(seed_idx);
    let mut count = 1usize;

    while let Some(idx) = queue.pop_front() {
        for target in &genome.nodes[idx].targets {
            if count >= max_size {
                break;
            }
            if let Some(&target_idx) = node_id_to_idx.get(&target.target_id) {
                if !visited[target_idx] {
                    visited[target_idx] = true;
                    queue.push_back(target_idx);
                    count += 1;
                }
            }
        }
        if count >= max_size {
            break;
        }
    }

    let indices: Vec<usize> = visited
        .iter()
        .enumerate()
        .filter(|(_, &v)| v)
        .map(|(i, _)| i)
        .collect();
    Some(DetectedGene { indices })
}

// ── Functional complexity ───────────────────────────────────────────────────

/// Reachability-aware functional complexity: only reachable mesh nodes
/// and live backend components contribute to the score.
///
/// For each reachable node, counts:
/// - The node itself (+1)
/// - Targets on the node
/// - Live backend components (instructions/nodes reached by backward slicing
///   from output instructions/nodes)
/// - Constants referenced by live instructions (VM only)
/// - Input refs consumed by live backend components
#[must_use]
pub fn functional_complexity(genome: &CreatureGenome) -> u32 {
    let node_id_to_idx: HashMap<NodeId, usize> = genome
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.node_id, i))
        .collect();

    let entry_idx = match node_id_to_idx.get(&genome.entry_node_id) {
        Some(&idx) => idx,
        None => return 0,
    };

    // BFS to find reachable mesh nodes
    let mut reachable = vec![false; genome.nodes.len()];
    reachable[entry_idx] = true;
    let mut queue = VecDeque::with_capacity(genome.nodes.len());
    queue.push_back(entry_idx);

    while let Some(idx) = queue.pop_front() {
        for target in &genome.nodes[idx].targets {
            if let Some(&target_idx) = node_id_to_idx.get(&target.target_id) {
                if !reachable[target_idx] {
                    reachable[target_idx] = true;
                    queue.push_back(target_idx);
                }
            }
        }
    }

    // Score each reachable node
    let mut score: u32 = 0;

    for (i, node) in genome.nodes.iter().enumerate() {
        if !reachable[i] {
            continue;
        }

        score += 1; // node itself
        score += node.targets.len() as u32;

        match &node.backend_def {
            BackendDef::Vm(vm) => {
                // Union backward slices from all output instructions
                let mut live = HashSet::new();
                for (idx, instr) in vm.program.iter().enumerate() {
                    if vm_is_output_instruction(instr) {
                        if let Some(gene) = vm_backward_slice(&vm.program, idx) {
                            live.extend(gene.indices);
                        }
                    }
                }

                score += live.len() as u32;

                // Count unique constants and consumed input_refs from live set
                let mut live_consts = HashSet::new();
                let mut consumed_refs = HashSet::new();
                for &idx in &live {
                    match &vm.program[idx] {
                        VmInstruction::LoadConst { const_idx, .. } => {
                            live_consts.insert(*const_idx);
                        }
                        VmInstruction::ReadInput { ref_idx, .. } => {
                            consumed_refs.insert(*ref_idx);
                        }
                        _ => {}
                    }
                }
                score += live_consts.len() as u32;
                score += consumed_refs.len() as u32;
            }
            BackendDef::Graph(graph) => {
                score += cgp_analysis::cgp_functional_complexity(graph);
            }
        }
    }

    score
}

#[cfg(test)]
#[path = "tests/analysis_tests.rs"]
mod tests;
