//! Genome analysis utilities: gene detection via backward/forward slicing.
//!
//! Pure analysis functions that identify functional gene boundaries within
//! VM programs and graph backends. Used by mutation operators, and available
//! for sexual reproduction, visualization, and selective reproduction.

use rand::Rng;

use std::collections::HashMap;
use std::collections::VecDeque;

use crate::contracts::NodeId;

use super::{CreatureGenome, GraphInternalNode, GraphNodeKind, VmInstruction};

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
        | VmInstruction::LoadMem8 { dst, .. }
        | VmInstruction::LoadMem8Imm { dst, .. } => Some(*dst),
        VmInstruction::Noop
        | VmInstruction::Halt
        | VmInstruction::Jump { .. }
        | VmInstruction::JumpIfZero { .. }
        | VmInstruction::WriteInternalPayload { .. }
        | VmInstruction::WriteWorldActionMeta { .. }
        | VmInstruction::EmitWorldAction { .. }
        | VmInstruction::WriteRouteTarget { .. }
        | VmInstruction::StoreMem8 { .. }
        | VmInstruction::StoreMem8Imm { .. } => None,
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
        VmInstruction::Noop | VmInstruction::Halt | VmInstruction::Jump { .. } => 0,
        VmInstruction::LoadConst { .. } | VmInstruction::LoadMem8Imm { .. } => 0,
        VmInstruction::EmitWorldAction { .. } => 0,
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
        | VmInstruction::WriteRouteTarget { src } => vm_reg_bit(*src),
        VmInstruction::StoreMem8Imm { src, .. } => vm_reg_bit(*src),
        VmInstruction::LoadMem8 { addr_reg, .. } => vm_reg_bit(*addr_reg),
        VmInstruction::StoreMem8 { addr_reg, src } => vm_reg_bit(*addr_reg) | vm_reg_bit(*src),
    }
}

/// Returns true if the instruction is an "output" (side-effecting write).
pub fn vm_is_output_instruction(instr: &VmInstruction) -> bool {
    matches!(
        instr,
        VmInstruction::WriteInternalPayload { .. }
            | VmInstruction::WriteWorldActionMeta { .. }
            | VmInstruction::EmitWorldAction { .. }
            | VmInstruction::WriteRouteTarget { .. }
            | VmInstruction::StoreMem8 { .. }
            | VmInstruction::StoreMem8Imm { .. }
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

// ── Graph analysis functions ────────────────────────────────────────────────

/// Returns true if the graph node kind is an output writer.
pub fn graph_is_output_node(kind: &GraphNodeKind) -> bool {
    matches!(
        kind,
        GraphNodeKind::CustomOutput(_) | GraphNodeKind::RouterOutput
    )
}

/// Backward-slice from an anchor node (must be an output node).
///
/// BFS backward from the anchor following `source_idx` edges. Returns
/// sorted indices capped at `max_size` nodes.
#[must_use]
pub fn graph_backward_slice(
    nodes: &[GraphInternalNode],
    anchor_idx: usize,
    max_size: usize,
) -> Option<DetectedGene> {
    if anchor_idx >= nodes.len() || !graph_is_output_node(&nodes[anchor_idx].kind) {
        return None;
    }
    let mut visited = vec![false; nodes.len()];
    visited[anchor_idx] = true;
    let mut queue = VecDeque::new();
    let mut gene_indices = Vec::with_capacity(max_size.min(nodes.len()));
    gene_indices.push(anchor_idx);

    // Enqueue sources of the anchor
    for input in &nodes[anchor_idx].inputs {
        let src = input.source_idx as usize;
        if src < nodes.len() && !visited[src] {
            visited[src] = true;
            queue.push_back(src);
        }
    }

    while let Some(idx) = queue.pop_front() {
        gene_indices.push(idx);
        if gene_indices.len() >= max_size {
            break;
        }
        for input in &nodes[idx].inputs {
            let src = input.source_idx as usize;
            if src < nodes.len() && !visited[src] {
                visited[src] = true;
                queue.push_back(src);
            }
        }
    }

    gene_indices.sort_unstable();
    Some(DetectedGene {
        indices: gene_indices,
    })
}

/// Backward-slice from a randomly chosen output node.
#[must_use]
pub fn graph_backward_slice_random(
    nodes: &[GraphInternalNode],
    rng: &mut impl Rng,
    max_size: usize,
) -> Option<DetectedGene> {
    let outputs: Vec<usize> = nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| graph_is_output_node(&n.kind))
        .map(|(i, _)| i)
        .collect();
    if outputs.is_empty() {
        return None;
    }
    let anchor = outputs[rng.gen_range(0..outputs.len())];
    graph_backward_slice(nodes, anchor, max_size)
}

/// Forward-slice from a seed node.
///
/// Fixpoint expansion: starts from seed, iteratively includes any node
/// whose inputs reference an already-included node. Returns sorted indices
/// capped at `max_size`.
#[must_use]
pub fn graph_forward_slice(
    nodes: &[GraphInternalNode],
    seed_idx: usize,
    max_size: usize,
) -> Option<DetectedGene> {
    if seed_idx >= nodes.len() {
        return None;
    }
    let mut included = vec![false; nodes.len()];
    included[seed_idx] = true;
    let mut count = 1usize;

    // Fixpoint iteration
    loop {
        let mut changed = false;
        for i in 0..nodes.len() {
            if included[i] || count >= max_size {
                continue;
            }
            let refs_included = nodes[i].inputs.iter().any(|inp| {
                (inp.source_idx as usize) < nodes.len() && included[inp.source_idx as usize]
            });
            if refs_included {
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

/// Forward-slice from a randomly chosen node.
#[must_use]
pub fn graph_forward_slice_random(
    nodes: &[GraphInternalNode],
    rng: &mut impl Rng,
    max_size: usize,
) -> Option<DetectedGene> {
    if nodes.is_empty() {
        return None;
    }
    let seed_idx = rng.gen_range(0..nodes.len());
    graph_forward_slice(nodes, seed_idx, max_size)
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
        for target_id in &genome.nodes[idx].targets {
            if let Some(target_idx) = find_idx(*target_id) {
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
            let targets_included = node.targets.iter().any(|target_id| {
                node_id_to_idx
                    .get(target_id)
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

/// Backward-slice from a randomly chosen mesh node.
#[inline]
#[must_use]
pub fn mesh_backward_slice_random(
    genome: &CreatureGenome,
    rng: &mut impl Rng,
    max_size: usize,
) -> Option<DetectedGene> {
    if genome.nodes.is_empty() {
        return None;
    }
    let anchor_idx = rng.gen_range(0..genome.nodes.len());
    mesh_backward_slice(genome, anchor_idx, max_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::NodeId;
    use crate::creature::genome::{
        BackendDef, GraphInput, GraphNodeKind, NodeGenome, VmBackendDef,
    };
    use rand::SeedableRng;

    // ── VM helper tests ─────────────────────────────────────────────────

    #[test]
    fn register_write_returns_dst_for_alu() {
        let instr = VmInstruction::Add { dst: 3, a: 1, b: 2 };
        assert_eq!(vm_register_write(&instr), Some(3));
    }

    #[test]
    fn register_write_returns_none_for_output() {
        let instr = VmInstruction::EmitWorldAction { action_type: 0 };
        assert_eq!(vm_register_write(&instr), None);
    }

    #[test]
    fn register_write_returns_none_for_halt() {
        assert_eq!(vm_register_write(&VmInstruction::Halt), None);
    }

    #[test]
    fn reg_bit_normal_range() {
        assert_eq!(vm_reg_bit(0), 1);
        assert_eq!(vm_reg_bit(1), 2);
        assert_eq!(vm_reg_bit(31), 1u32 << 31);
    }

    #[test]
    fn reg_bit_out_of_range_returns_zero() {
        assert_eq!(vm_reg_bit(32), 0);
        assert_eq!(vm_reg_bit(255), 0);
    }

    #[test]
    fn read_mask_alu_two_sources() {
        let instr = VmInstruction::Add { dst: 0, a: 1, b: 2 };
        assert_eq!(vm_register_read_mask(&instr), vm_reg_bit(1) | vm_reg_bit(2));
    }

    #[test]
    fn read_mask_noop_is_zero() {
        assert_eq!(vm_register_read_mask(&VmInstruction::Noop), 0);
    }

    #[test]
    fn read_mask_write_route_target() {
        let instr = VmInstruction::WriteRouteTarget { src: 5 };
        assert_eq!(vm_register_read_mask(&instr), vm_reg_bit(5));
    }

    #[test]
    fn is_output_matches_side_effecting_writes() {
        assert!(vm_is_output_instruction(&VmInstruction::EmitWorldAction {
            action_type: 0
        }));
        assert!(vm_is_output_instruction(&VmInstruction::WriteRouteTarget {
            src: 0
        }));
        assert!(vm_is_output_instruction(
            &VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 1
            }
        ));
        assert!(!vm_is_output_instruction(&VmInstruction::Add {
            dst: 0,
            a: 1,
            b: 2
        }));
        assert!(!vm_is_output_instruction(&VmInstruction::Halt));
    }

    // ── VM backward slice tests ─────────────────────────────────────────

    #[test]
    fn backward_slice_traces_register_deps() {
        // r0 = LoadConst     (idx 0)
        // r1 = Add r0, r0    (idx 1)
        // WriteRoute r1      (idx 2, anchor)
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            VmInstruction::Add { dst: 1, a: 0, b: 0 },
            VmInstruction::WriteRouteTarget { src: 1 },
        ];
        let gene = vm_backward_slice(&program, 2).unwrap();
        assert_eq!(gene.indices, vec![0, 1, 2]);
    }

    #[test]
    fn backward_slice_skips_unrelated_instructions() {
        // r0 = LoadConst     (idx 0)
        // r1 = LoadConst     (idx 1) — unrelated
        // r2 = Add r0, r0    (idx 2)
        // WriteRoute r2      (idx 3, anchor)
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            VmInstruction::LoadConst {
                dst: 1,
                const_idx: 1,
            },
            VmInstruction::Add { dst: 2, a: 0, b: 0 },
            VmInstruction::WriteRouteTarget { src: 2 },
        ];
        let gene = vm_backward_slice(&program, 3).unwrap();
        assert_eq!(gene.indices, vec![0, 2, 3]);
    }

    #[test]
    fn backward_slice_returns_none_for_non_output_anchor() {
        let program = vec![VmInstruction::Add { dst: 0, a: 1, b: 2 }];
        assert_eq!(vm_backward_slice(&program, 0), None);
    }

    #[test]
    fn backward_slice_returns_none_for_out_of_bounds() {
        let program = vec![VmInstruction::Halt];
        assert_eq!(vm_backward_slice(&program, 5), None);
    }

    #[test]
    fn backward_slice_anchor_only_when_no_deps() {
        // EmitWorldAction reads no registers
        let program = vec![VmInstruction::EmitWorldAction { action_type: 1 }];
        let gene = vm_backward_slice(&program, 0).unwrap();
        assert_eq!(gene.indices, vec![0]);
    }

    // ── VM forward slice tests ──────────────────────────────────────────

    #[test]
    fn forward_slice_traces_downstream() {
        // r0 = LoadConst     (idx 0, seed)
        // r1 = Add r0, r0    (idx 1)
        // WriteRoute r1      (idx 2)
        // Halt               (idx 3)
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            VmInstruction::Add { dst: 1, a: 0, b: 0 },
            VmInstruction::WriteRouteTarget { src: 1 },
            VmInstruction::Halt,
        ];
        let gene = vm_forward_slice(&program, 0).unwrap();
        assert_eq!(gene.indices, vec![0, 1, 2]);
    }

    #[test]
    fn forward_slice_skips_unrelated() {
        // r0 = LoadConst     (idx 0, seed)
        // r1 = LoadConst     (idx 1) — unrelated
        // r2 = Add r0, r0    (idx 2)
        // WriteRoute r1      (idx 3) — reads r1, not r0/r2
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            VmInstruction::LoadConst {
                dst: 1,
                const_idx: 1,
            },
            VmInstruction::Add { dst: 2, a: 0, b: 0 },
            VmInstruction::WriteRouteTarget { src: 1 },
        ];
        let gene = vm_forward_slice(&program, 0).unwrap();
        // Starts at 0, picks up 2 (reads r0), does NOT pick up 3 (reads r1, not in produced set)
        assert_eq!(gene.indices, vec![0, 2]);
    }

    #[test]
    fn forward_slice_returns_none_for_non_writer() {
        let program = vec![VmInstruction::EmitWorldAction { action_type: 0 }];
        assert_eq!(vm_forward_slice(&program, 0), None);
    }

    #[test]
    fn forward_slice_returns_none_for_out_of_bounds() {
        let program = vec![VmInstruction::Halt];
        assert_eq!(vm_forward_slice(&program, 5), None);
    }

    #[test]
    fn forward_slice_seed_only_when_no_consumers() {
        // r0 = LoadConst — nothing reads r0 afterward
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            VmInstruction::Halt,
        ];
        let gene = vm_forward_slice(&program, 0).unwrap();
        assert_eq!(gene.indices, vec![0]);
    }

    // ── VM random slice tests ───────────────────────────────────────────

    #[test]
    fn backward_slice_random_returns_none_for_no_outputs() {
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            VmInstruction::Halt,
        ];
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
        assert_eq!(vm_backward_slice_random(&program, &mut rng), None);
    }

    #[test]
    fn forward_slice_random_returns_none_for_no_writers() {
        let program = vec![VmInstruction::Halt, VmInstruction::Noop];
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
        assert_eq!(vm_forward_slice_random(&program, &mut rng), None);
    }

    #[test]
    fn backward_slice_random_returns_some_for_valid_program() {
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            VmInstruction::WriteRouteTarget { src: 0 },
        ];
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
        let gene = vm_backward_slice_random(&program, &mut rng).unwrap();
        assert_eq!(gene.indices, vec![0, 1]);
    }

    #[test]
    fn forward_slice_random_returns_some_for_valid_program() {
        let program = vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            VmInstruction::WriteRouteTarget { src: 0 },
        ];
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
        let gene = vm_forward_slice_random(&program, &mut rng).unwrap();
        assert_eq!(gene.indices, vec![0, 1]);
    }

    // ── Graph analysis tests ────────────────────────────────────────────

    fn make_graph_nodes() -> Vec<GraphInternalNode> {
        // Node 0: InputRef(0) — no inputs (reads from external input_refs)
        // Node 1: Add — inputs from node 0
        // Node 2: CustomOutput(0) — inputs from node 1 (output node)
        vec![
            GraphInternalNode {
                kind: GraphNodeKind::InputRef(0),
                inputs: vec![],
            },
            GraphInternalNode {
                kind: GraphNodeKind::Add,
                inputs: vec![GraphInput {
                    source_idx: 0,
                    weight: 1.0,
                }],
            },
            GraphInternalNode {
                kind: GraphNodeKind::CustomOutput(0),
                inputs: vec![GraphInput {
                    source_idx: 1,
                    weight: 1.0,
                }],
            },
        ]
    }

    #[test]
    fn graph_is_output_node_matches_outputs() {
        assert!(graph_is_output_node(&GraphNodeKind::CustomOutput(0)));
        assert!(graph_is_output_node(&GraphNodeKind::RouterOutput));
        assert!(!graph_is_output_node(&GraphNodeKind::Add));
        assert!(!graph_is_output_node(&GraphNodeKind::InputRef(0)));
    }

    #[test]
    fn graph_backward_slice_traces_deps() {
        let nodes = make_graph_nodes();
        // Anchor at node 2 (CustomOutput) -> node 1 (Add) -> node 0 (InputRef)
        let gene = graph_backward_slice(&nodes, 2, 32).unwrap();
        assert_eq!(gene.indices, vec![0, 1, 2]);
    }

    #[test]
    fn graph_backward_slice_returns_none_for_non_output() {
        let nodes = make_graph_nodes();
        assert_eq!(graph_backward_slice(&nodes, 0, 32), None);
    }

    #[test]
    fn graph_backward_slice_excludes_disconnected() {
        // Node 0: InputRef — no inputs
        // Node 1: InputRef — no inputs (disconnected from output)
        // Node 2: CustomOutput — inputs from node 0 only
        let nodes = vec![
            GraphInternalNode {
                kind: GraphNodeKind::InputRef(0),
                inputs: vec![],
            },
            GraphInternalNode {
                kind: GraphNodeKind::InputRef(1),
                inputs: vec![],
            },
            GraphInternalNode {
                kind: GraphNodeKind::CustomOutput(0),
                inputs: vec![GraphInput {
                    source_idx: 0,
                    weight: 1.0,
                }],
            },
        ];
        let gene = graph_backward_slice(&nodes, 2, 32).unwrap();
        assert_eq!(gene.indices, vec![0, 2]); // node 1 excluded
    }

    #[test]
    fn graph_backward_slice_respects_max_size() {
        let nodes = make_graph_nodes();
        let gene = graph_backward_slice(&nodes, 2, 2).unwrap();
        assert_eq!(gene.indices.len(), 2);
    }

    #[test]
    fn graph_backward_slice_empty_returns_none() {
        let nodes: Vec<GraphInternalNode> = vec![];
        assert_eq!(graph_backward_slice(&nodes, 0, 32), None);
    }

    #[test]
    fn graph_forward_slice_traces_downstream() {
        let nodes = make_graph_nodes();
        // Seed at node 0: node 1 reads from 0, node 2 reads from 1
        let gene = graph_forward_slice(&nodes, 0, 32).unwrap();
        assert_eq!(gene.indices, vec![0, 1, 2]);
    }

    #[test]
    fn graph_forward_slice_excludes_disconnected() {
        // Node 0: InputRef — seed
        // Node 1: InputRef — disconnected
        // Node 2: Add — reads from node 0
        let nodes = vec![
            GraphInternalNode {
                kind: GraphNodeKind::InputRef(0),
                inputs: vec![],
            },
            GraphInternalNode {
                kind: GraphNodeKind::InputRef(1),
                inputs: vec![],
            },
            GraphInternalNode {
                kind: GraphNodeKind::Add,
                inputs: vec![GraphInput {
                    source_idx: 0,
                    weight: 1.0,
                }],
            },
        ];
        let gene = graph_forward_slice(&nodes, 0, 32).unwrap();
        assert_eq!(gene.indices, vec![0, 2]); // node 1 excluded
    }

    #[test]
    fn graph_forward_slice_respects_max_size() {
        let nodes = make_graph_nodes();
        let gene = graph_forward_slice(&nodes, 0, 2).unwrap();
        assert_eq!(gene.indices.len(), 2);
    }

    #[test]
    fn graph_forward_slice_out_of_bounds_returns_none() {
        let nodes = make_graph_nodes();
        assert_eq!(graph_forward_slice(&nodes, 10, 32), None);
    }

    #[test]
    fn graph_backward_slice_random_returns_none_for_no_outputs() {
        let nodes = vec![GraphInternalNode {
            kind: GraphNodeKind::Add,
            inputs: vec![],
        }];
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
        assert_eq!(graph_backward_slice_random(&nodes, &mut rng, 32), None);
    }

    #[test]
    fn graph_forward_slice_random_returns_none_for_empty() {
        let nodes: Vec<GraphInternalNode> = vec![];
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
        assert_eq!(graph_forward_slice_random(&nodes, &mut rng, 32), None);
    }

    // ── Mesh reachability tests ─────────────────────────────────────────

    fn simple_vm_node(id: u32, targets: Vec<u32>) -> NodeGenome {
        NodeGenome {
            node_id: NodeId::new(id),
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![VmInstruction::Halt],
            }),
            targets: targets.into_iter().map(NodeId::new).collect(),
        }
    }

    #[test]
    fn mesh_reachable_both_nodes_in_founder_style() {
        // Node 0 -> Node 1, entry at node 0
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![simple_vm_node(0, vec![1]), simple_vm_node(1, vec![])],
        };
        assert_eq!(mesh_reachable_nodes(&genome), vec![0, 1]);
    }

    #[test]
    fn mesh_reachable_excludes_unreachable_node() {
        // Node 0 -> Node 1, Node 2 is unreachable
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![
                simple_vm_node(0, vec![1]),
                simple_vm_node(1, vec![]),
                simple_vm_node(2, vec![]),
            ],
        };
        assert_eq!(mesh_reachable_nodes(&genome), vec![0, 1]);
    }

    #[test]
    fn mesh_reachable_entry_only_when_no_targets() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![simple_vm_node(0, vec![])],
        };
        assert_eq!(mesh_reachable_nodes(&genome), vec![0]);
    }

    #[test]
    fn mesh_reachable_empty_genome() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![],
        };
        assert_eq!(mesh_reachable_nodes(&genome), Vec::<usize>::new());
    }

    #[test]
    fn mesh_reachable_handles_dangling_target() {
        // Node 0 targets node 99 which doesn't exist
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![simple_vm_node(0, vec![99])],
        };
        assert_eq!(mesh_reachable_nodes(&genome), vec![0]);
    }

    #[test]
    fn mesh_reachable_handles_cycle() {
        // Node 0 -> Node 1 -> Node 0 (cycle)
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![simple_vm_node(0, vec![1]), simple_vm_node(1, vec![0])],
        };
        assert_eq!(mesh_reachable_nodes(&genome), vec![0, 1]);
    }

    // ── Mesh backward slice tests ───────────────────────────────────────

    #[test]
    fn mesh_backward_slice_finds_feeding_pipeline() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![
                simple_vm_node(0, vec![1]),
                simple_vm_node(1, vec![2]),
                simple_vm_node(2, vec![]),
            ],
        };
        let gene = mesh_backward_slice(&genome, 2, 8).unwrap();
        assert_eq!(gene.indices, vec![0, 1, 2]);
    }

    #[test]
    fn mesh_backward_slice_excludes_unconnected_nodes() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![
                simple_vm_node(0, vec![1]),
                simple_vm_node(1, vec![]),
                simple_vm_node(2, vec![1]),
                simple_vm_node(3, vec![]),
            ],
        };
        let gene = mesh_backward_slice(&genome, 1, 8).unwrap();
        assert_eq!(gene.indices, vec![0, 1, 2]);
    }

    #[test]
    fn mesh_backward_slice_anchor_only_when_nothing_targets_it() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![
                simple_vm_node(0, vec![1]),
                simple_vm_node(1, vec![]),
            ],
        };
        let gene = mesh_backward_slice(&genome, 0, 8).unwrap();
        assert_eq!(gene.indices, vec![0]);
    }

    #[test]
    fn mesh_backward_slice_respects_max_size() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![
                simple_vm_node(0, vec![1]),
                simple_vm_node(1, vec![2]),
                simple_vm_node(2, vec![]),
            ],
        };
        let gene = mesh_backward_slice(&genome, 2, 2).unwrap();
        assert_eq!(gene.indices.len(), 2);
    }

    #[test]
    fn mesh_backward_slice_out_of_bounds_returns_none() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![simple_vm_node(0, vec![])],
        };
        assert_eq!(mesh_backward_slice(&genome, 5, 8), None);
    }

    #[test]
    fn mesh_backward_slice_empty_genome_returns_none() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![],
        };
        assert_eq!(mesh_backward_slice(&genome, 0, 8), None);
    }

    #[test]
    fn mesh_backward_slice_handles_cycle() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![
                simple_vm_node(0, vec![1]),
                simple_vm_node(1, vec![0]),
            ],
        };
        let gene = mesh_backward_slice(&genome, 0, 8).unwrap();
        assert_eq!(gene.indices, vec![0, 1]);
    }
}
