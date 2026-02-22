# V3 Stage 2: Creature Schema Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task. Invoke `rust-skills` before writing any Rust code.

**Goal:** Implement all creature genome types, the creature state struct, the canonical v3alpha1 founder genome constant, and the parseability gate.

**Goal IDs:** `GP-01`, `GP-02`, `GP-03`

**Scope:** `v3/crates/v3-core/src/creature/` module only. No runtime execution, sensor resolution, or mutation logic in this stage.

**Docs Impact:**
- No canonical docs changed.
- New `creature/` module added under v3-core (not yet public from lib.rs until wired in Task 6).

**Supersedes:** none

**Superseded-By:** none

**Architecture:** `creature/` module has no dependencies within v3-core except on `contracts/` and `config/`. Dependency direction is: `contracts` <- `creature`. The module is split into: `genome.rs` (type definitions), `state.rs` (creature runtime state), `founder.rs` (v3alpha1 constant), `parseability.rs` (validation gate).

**Tech Stack:** Rust 1.93.0, serde, slotmap (CreatureId already defined in contracts), std::collections::HashMap.

---

## Goal Alignment

- **GP-01**: Genome types define the representation for richer creature cognition — VM and Graph backends with full instruction/operator sets.
- **GP-02**: Clean `creature/` module with no circular deps; wired into lib.rs as `pub mod creature`.
- **GP-03**: Founder genome is tested to pass the parseability gate; invalid genomes tested to fail.

## Boundary Impact

- New `v3/crates/v3-core/src/creature/` module created.
- No changes to contracts, config, or kernel modules.

## Existing Boundary Recheck

| area | decision | rationale |
|------|----------|-----------|
| `v3/crates/v3-core/src/contracts/` | keep | Already defined NodeId, CreatureId, InputReference — creature module imports these |
| `v3/crates/v3-core/src/config/` | keep | SimulationConfig already defined; creature state uses energy/config values via runtime, not config directly |
| `v3/crates/v3-core/src/kernel/` | keep | WorldState defined; creature module has no dependency on kernel |

## Open Questions

| question | decision | owner | status |
|----------|----------|-------|--------|
| Should VmInstruction use struct-like variants or tuple variants? | Struct variants — more readable and resistant to operand reordering bugs | agent | resolved |
| How to represent the founder genome opcodes that reference action_type as a literal vs register? | `EmitWorldAction { action_type: u8 }` uses direct value, not register — per spec "action_type" is the WorldAction discriminant | agent | resolved |
| Should CreatureState own the genome or hold a reference? | Owned — creatures own their genome; copies made on reproduction | agent | resolved |
| Does parseability check entry_node_id resolution? | No — per spec Section 4: entry_node_id missing is handled by runtime soft default, not validation | agent | resolved |

---

## Tasks

### Task 1: Genome types (`creature/genome.rs`)

**Files:**
- Create: `v3/crates/v3-core/src/creature/genome.rs`

**Step 1: Implement all genome types**

VM instruction set (33 opcodes per v3-vm-isa-spec.md Section 2):

```rust
use crate::contracts::{InputReference, NodeId};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum VmInstruction {
    // Arithmetic and Data Movement
    Noop,
    LoadConst { dst: u8, const_idx: u8 },
    Move { dst: u8, src: u8 },
    Add { dst: u8, a: u8, b: u8 },
    Sub { dst: u8, a: u8, b: u8 },
    Mul { dst: u8, a: u8, b: u8 },
    Div { dst: u8, a: u8, b: u8 },
    Min { dst: u8, a: u8, b: u8 },
    Max { dst: u8, a: u8, b: u8 },
    Abs { dst: u8, src: u8 },
    Neg { dst: u8, src: u8 },
    Clamp01 { dst: u8, src: u8 },
    // Comparison and Logic
    CmpGt { dst: u8, a: u8, b: u8 },
    CmpLt { dst: u8, a: u8, b: u8 },
    CmpEq { dst: u8, a: u8, b: u8, eps: u8 },
    And { dst: u8, a: u8, b: u8 },
    Or { dst: u8, a: u8, b: u8 },
    Not { dst: u8, src: u8 },
    // Type Conversion
    ToI32 { dst: u8, src: u8 },
    ToU8 { dst: u8, src: u8 },
    ToBool { dst: u8, src: u8 },
    // Control Flow
    JumpIfZero { cond: u8, offset: i32 },
    Jump { offset: i32 },
    // Input Reads
    ReadInput { dst: u8, input_idx: u8 },
    // Output and Routing
    WriteInternalPayload { slot_idx: u8, src: u8 },
    WriteWorldActionMeta { slot_idx: u8, src: u8 },
    EmitWorldAction { action_type: u8 },
    WriteRouteTarget { src: u8 },
    // Halt and Memory
    Halt,
    LoadMem8 { dst: u8, addr_reg: u8 },
    StoreMem8 { addr_reg: u8, src: u8 },
    LoadMem8Imm { dst: u8, imm_addr: u16 },
    StoreMem8Imm { imm_addr: u16, src: u8 },
}
```

VM backend definition:

```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct VmBackendDef {
    pub register_count: u8,
    pub constants: Vec<f32>,
    pub program: Vec<VmInstruction>,
}
```

Graph types (per v3-graph-backend-spec.md):

```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GraphInput {
    pub source_idx: u16,
    pub weight: f32,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum GraphNodeKind {
    InputRef(u8),
    Constant(f32),
    Add,
    Multiply,
    Negate,
    Abs,
    Min,
    Max,
    Threshold(f32),
    GreaterThan,
    Sigmoid,
    Tanh,
    Relu,
    Select,
    Clamp01,
    WeightedSum,
    DecayIntegrator(f32),
    Momentum(f32),
    Oscillator(f32),
    AdaptiveGain,
    CustomOutput(u8),
    RouterOutput,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GraphInternalNode {
    pub kind: GraphNodeKind,
    pub inputs: Vec<GraphInput>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GraphBackendDef {
    pub internal_nodes: Vec<GraphInternalNode>,
}
```

Top-level genome types:

```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum BackendDef {
    Vm(VmBackendDef),
    Graph(GraphBackendDef),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NodeGenome {
    pub node_id: NodeId,
    pub input_refs: Vec<InputReference>,
    pub backend_def: BackendDef,
    pub targets: Vec<NodeId>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CreatureGenome {
    pub entry_node_id: NodeId,
    pub nodes: Vec<NodeGenome>,
}
```

**Step 2: Write tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vm_instruction_count_is_33() {
        // Compile-time guard: all 33 opcodes must be constructible
        let instructions: Vec<VmInstruction> = vec![
            VmInstruction::Noop,
            VmInstruction::LoadConst { dst: 0, const_idx: 0 },
            VmInstruction::Move { dst: 0, src: 1 },
            VmInstruction::Add { dst: 0, a: 1, b: 2 },
            VmInstruction::Sub { dst: 0, a: 1, b: 2 },
            VmInstruction::Mul { dst: 0, a: 1, b: 2 },
            VmInstruction::Div { dst: 0, a: 1, b: 2 },
            VmInstruction::Min { dst: 0, a: 1, b: 2 },
            VmInstruction::Max { dst: 0, a: 1, b: 2 },
            VmInstruction::Abs { dst: 0, src: 1 },
            VmInstruction::Neg { dst: 0, src: 1 },
            VmInstruction::Clamp01 { dst: 0, src: 1 },
            VmInstruction::CmpGt { dst: 0, a: 1, b: 2 },
            VmInstruction::CmpLt { dst: 0, a: 1, b: 2 },
            VmInstruction::CmpEq { dst: 0, a: 1, b: 2, eps: 3 },
            VmInstruction::And { dst: 0, a: 1, b: 2 },
            VmInstruction::Or { dst: 0, a: 1, b: 2 },
            VmInstruction::Not { dst: 0, src: 1 },
            VmInstruction::ToI32 { dst: 0, src: 1 },
            VmInstruction::ToU8 { dst: 0, src: 1 },
            VmInstruction::ToBool { dst: 0, src: 1 },
            VmInstruction::JumpIfZero { cond: 0, offset: 2 },
            VmInstruction::Jump { offset: -1 },
            VmInstruction::ReadInput { dst: 0, input_idx: 0 },
            VmInstruction::WriteInternalPayload { slot_idx: 0, src: 1 },
            VmInstruction::WriteWorldActionMeta { slot_idx: 0, src: 1 },
            VmInstruction::EmitWorldAction { action_type: 1 },
            VmInstruction::WriteRouteTarget { src: 0 },
            VmInstruction::Halt,
            VmInstruction::LoadMem8 { dst: 0, addr_reg: 1 },
            VmInstruction::StoreMem8 { addr_reg: 0, src: 1 },
            VmInstruction::LoadMem8Imm { dst: 0, imm_addr: 100 },
            VmInstruction::StoreMem8Imm { imm_addr: 200, src: 1 },
        ];
        assert_eq!(instructions.len(), 33);
    }

    #[test]
    fn vm_instruction_serde_roundtrip() {
        let instr = VmInstruction::CmpEq { dst: 0, a: 1, b: 2, eps: 3 };
        let json = serde_json::to_string(&instr).unwrap();
        let instr2: VmInstruction = serde_json::from_str(&json).unwrap();
        assert_eq!(instr, instr2);
    }

    #[test]
    fn graph_node_kinds_constructible() {
        let kinds = vec![
            GraphNodeKind::InputRef(0),
            GraphNodeKind::Constant(1.0),
            GraphNodeKind::Add,
            GraphNodeKind::Multiply,
            GraphNodeKind::Negate,
            GraphNodeKind::Abs,
            GraphNodeKind::Min,
            GraphNodeKind::Max,
            GraphNodeKind::Threshold(0.5),
            GraphNodeKind::GreaterThan,
            GraphNodeKind::Sigmoid,
            GraphNodeKind::Tanh,
            GraphNodeKind::Relu,
            GraphNodeKind::Select,
            GraphNodeKind::Clamp01,
            GraphNodeKind::WeightedSum,
            GraphNodeKind::DecayIntegrator(0.9),
            GraphNodeKind::Momentum(0.1),
            GraphNodeKind::Oscillator(1.0),
            GraphNodeKind::AdaptiveGain,
            GraphNodeKind::CustomOutput(0),
            GraphNodeKind::RouterOutput,
        ];
        assert_eq!(kinds.len(), 22);
    }

    #[test]
    fn creature_genome_minimal() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::EmitWorldAction { action_type: 0 }],
                }),
                targets: vec![],
            }],
        };
        assert_eq!(genome.entry_node_id, NodeId::new(0));
        assert_eq!(genome.nodes.len(), 1);
    }

    #[test]
    fn creature_genome_serde_roundtrip() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 2,
                    constants: vec![1.0, 2.0],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![],
            }],
        };
        let json = serde_json::to_string(&genome).unwrap();
        let genome2: CreatureGenome = serde_json::from_str(&json).unwrap();
        assert_eq!(genome, genome2);
    }
}
```

**Step 3: Run tests**

```bash
cd v3 && cargo test creature::genome -- --nocapture 2>&1 | tail -15
```

Expected: PASS (4 tests).

**Step 4: Commit**

```bash
git add v3/crates/v3-core/src/creature/genome.rs
git commit -m "feat(v3-core): add genome types (VmInstruction x33, GraphNodeKind x22, CreatureGenome)"
```

---

### Task 2: Creature state (`creature/state.rs`)

**Files:**
- Create: `v3/crates/v3-core/src/creature/state.rs`

**Spec reference:** `docs/reference/v3-genome-spec.md` Section 5.

**Step 1: Implement**

```rust
use std::collections::HashMap;

use crate::contracts::{CreatureId, NodeId, Position};
use crate::creature::genome::CreatureGenome;

/// Full runtime state of a creature in the simulation.
pub struct CreatureState {
    pub id: CreatureId,
    pub genome: CreatureGenome,
    pub position: Position,
    pub energy: f32,
    pub age: u64,
    pub generation: u64,
    /// 1024 bytes persistent memory, copied on reproduction.
    pub memory: [u8; 1024],
    /// Per-node stateful operator state for the Graph backend.
    /// Keyed by NodeId; lazily initialized on first access.
    pub graph_state: HashMap<NodeId, Vec<f32>>,
    /// RGB phenotype color. Channel values in [0, 255].
    pub phenotype_rgb: [u8; 3],
}

impl CreatureState {
    /// Create a new creature with zeroed memory and empty graph state.
    pub fn new(
        id: CreatureId,
        genome: CreatureGenome,
        position: Position,
        energy: f32,
        generation: u64,
        phenotype_rgb: [u8; 3],
    ) -> Self {
        Self {
            id,
            genome,
            position,
            energy,
            age: 0,
            generation,
            memory: [0u8; 1024],
            graph_state: HashMap::new(),
            phenotype_rgb,
        }
    }
}
```

**Step 2: Write tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::NodeId;
    use crate::creature::genome::{BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction};
    use slotmap::SlotMap;

    fn minimal_genome() -> CreatureGenome {
        CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
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
    fn new_creature_has_zero_age_and_empty_memory() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let state = CreatureState::new(
            id,
            minimal_genome(),
            Position::new(5, 5),
            20.0,
            0,
            [128, 64, 32],
        );
        assert_eq!(state.age, 0);
        assert_eq!(state.memory, [0u8; 1024]);
        assert!(state.graph_state.is_empty());
        assert_eq!(state.generation, 0);
        assert!((state.energy - 20.0).abs() < f32::EPSILON);
        assert_eq!(state.phenotype_rgb, [128, 64, 32]);
    }

    #[test]
    fn creature_state_position_stored() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let state = CreatureState::new(
            id,
            minimal_genome(),
            Position::new(3, 7),
            50.0,
            2,
            [0, 0, 0],
        );
        assert_eq!(state.position, Position::new(3, 7));
        assert_eq!(state.generation, 2);
    }
}
```

**Step 3: Run tests**

```bash
cd v3 && cargo test creature::state -- --nocapture 2>&1 | tail -10
```

Expected: PASS (2 tests).

**Step 4: Commit**

```bash
git add v3/crates/v3-core/src/creature/state.rs
git commit -m "feat(v3-core): add CreatureState with position, energy, memory, graph_state, phenotype"
```

---

### Task 3: Founder genome constant (`creature/founder.rs`)

**Files:**
- Create: `v3/crates/v3-core/src/creature/founder.rs`

**Spec reference:** `docs/reference/v3-startup-seeding-spec.md` Section 5.1.

Note: The founder genome is the canonical v3alpha1 layout. The VM program in the spec uses
pseudo-code register names (r0-r7); in our encoding these are indices 0-7.

**Step 1: Implement**

The founder uses two nodes:
- Node 0: Graph backend (10 inputs, 18 internal nodes, targets=[1])
- Node 1: VM backend (6 inputs, register_count=8, program with 33+ instructions, targets=[])

Implement the founder genome following the spec exactly. See full implementation in code below.

Key spec-to-code mapping for the VM program (Node 1):
- `ReadInput(r0, 0)` = `ReadInput { dst: 0, input_idx: 0 }`
- `CmpGt(r6, r0, r7)` = `CmpGt { dst: 6, a: 0, b: 7 }` (r7 = 0.0 initially)
- `JumpIfZero(r6, +2)` = `JumpIfZero { cond: 6, offset: 2 }`
- `EmitWorldAction(1)` = `EmitWorldAction { action_type: 1 }`
- `WriteWorldActionMeta(0, r7)` = `WriteWorldActionMeta { slot_idx: 0, src: 7 }`
- `LoadConst(r6, 4)` = `LoadConst { dst: 6, const_idx: 4 }` (constants[4]=20.0)
- `WriteWorldActionMeta(1, r6)` = `WriteWorldActionMeta { slot_idx: 1, src: 6 }`

**Step 2: Write test**

```rust
#[test]
fn founder_genome_structure() {
    let g = v3alpha1_founder_genome();
    assert_eq!(g.entry_node_id, NodeId::new(0));
    assert_eq!(g.nodes.len(), 2);

    // Node 0: Graph backend
    let node0 = &g.nodes[0];
    assert_eq!(node0.node_id, NodeId::new(0));
    assert_eq!(node0.input_refs.len(), 10);
    assert_eq!(node0.targets, vec![NodeId::new(1)]);

    if let BackendDef::Graph(ref gdef) = node0.backend_def {
        assert_eq!(gdef.internal_nodes.len(), 18);
    } else {
        panic!("Node 0 must be Graph backend");
    }

    // Node 1: VM backend
    let node1 = &g.nodes[1];
    assert_eq!(node1.node_id, NodeId::new(1));
    assert_eq!(node1.input_refs.len(), 6);
    assert!(node1.targets.is_empty());

    if let BackendDef::Vm(ref vdef) = node1.backend_def {
        assert_eq!(vdef.register_count, 8);
        assert_eq!(vdef.constants, vec![0.5, 1.0, 2.0, 3.0, 20.0]);
        assert!(!vdef.program.is_empty());
    } else {
        panic!("Node 1 must be VM backend");
    }
}
```

**Step 3: Run tests**

```bash
cd v3 && cargo test creature::founder -- --nocapture 2>&1 | tail -10
```

Expected: PASS.

**Step 4: Commit**

```bash
git add v3/crates/v3-core/src/creature/founder.rs
git commit -m "feat(v3-core): add canonical v3alpha1 founder genome constant"
```

---

### Task 4: Parseability gate (`creature/parseability.rs`)

**Files:**
- Create: `v3/crates/v3-core/src/creature/parseability.rs`

**Spec reference:** `docs/reference/v3-genome-spec.md` Section 4.

Required invariants:
- `nodes` is non-empty.
- `node_id` values are unique.
- backend payloads remain decodable (all types are well-typed, so this is satisfied by construction).

NOT required at parse time: entry_node_id resolves, targets resolve, non-empty targets.

**Step 1: Implement**

```rust
use std::collections::HashSet;
use crate::creature::genome::CreatureGenome;

#[derive(Debug, PartialEq)]
pub enum ParseabilityError {
    EmptyNodeList,
    DuplicateNodeId(u32),
}

pub struct ParseabilityGate;

impl ParseabilityGate {
    pub fn validate(genome: &CreatureGenome) -> Result<(), ParseabilityError> {
        if genome.nodes.is_empty() {
            return Err(ParseabilityError::EmptyNodeList);
        }
        let mut seen = HashSet::new();
        for node in &genome.nodes {
            if !seen.insert(node.node_id.0) {
                return Err(ParseabilityError::DuplicateNodeId(node.node_id.0));
            }
        }
        Ok(())
    }
}
```

**Step 2: Write tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn founder_genome_passes_parseability() {
        let genome = crate::creature::founder::v3alpha1_founder_genome();
        assert!(ParseabilityGate::validate(&genome).is_ok());
    }

    #[test]
    fn empty_node_list_fails() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![],
        };
        assert_eq!(
            ParseabilityGate::validate(&genome),
            Err(ParseabilityError::EmptyNodeList)
        );
    }

    #[test]
    fn duplicate_node_ids_fail() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![
                make_minimal_node(0),
                make_minimal_node(0), // duplicate!
            ],
        };
        assert_eq!(
            ParseabilityGate::validate(&genome),
            Err(ParseabilityError::DuplicateNodeId(0))
        );
    }

    #[test]
    fn unresolved_entry_node_passes_parseability() {
        // entry_node_id resolution is NOT a parseability requirement
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(99),
            nodes: vec![make_minimal_node(0)],
        };
        assert!(ParseabilityGate::validate(&genome).is_ok());
    }

    #[test]
    fn dangling_route_targets_pass_parseability() {
        // targets may reference non-existent nodes (junk DNA)
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1, constants: vec![], program: vec![VmInstruction::Halt],
                }),
                targets: vec![NodeId::new(999)], // dangling
            }],
        };
        assert!(ParseabilityGate::validate(&genome).is_ok());
    }
}
```

**Step 3: Run tests**

```bash
cd v3 && cargo test creature::parseability -- --nocapture 2>&1 | tail -10
```

Expected: PASS (5 tests including founder_genome_passes_parseability).

**Step 4: Commit**

```bash
git add v3/crates/v3-core/src/creature/parseability.rs
git commit -m "feat(v3-core): add ParseabilityGate (non-empty nodes, unique IDs)"
```

---

### Task 5: Wire creature module into lib.rs and run quality checks

**Files:**
- Create: `v3/crates/v3-core/src/creature/mod.rs`
- Modify: `v3/crates/v3-core/src/lib.rs`

**Step 1: Create `creature/mod.rs`**

```rust
pub mod founder;
pub mod genome;
pub mod parseability;
pub mod state;

pub use genome::{
    BackendDef, CreatureGenome, GraphBackendDef, GraphInput, GraphInternalNode, GraphNodeKind,
    NodeGenome, VmBackendDef, VmInstruction,
};
pub use parseability::{ParseabilityError, ParseabilityGate};
pub use state::CreatureState;
```

**Step 2: Add `pub mod creature;` to lib.rs**

**Step 3: Run fmt, clippy, tests**

```bash
cd v3 && cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace 2>&1 | tail -15
```

Expected: all tests pass, 0 warnings.

**Step 4: Run plan harness**

```bash
scripts/check-plan-harness.sh --mode strict 2>&1 | tail -3
```

Expected: violations=0.

**Step 5: Final commit**

```bash
git add v3/ && git commit -m "feat(v3-core): wire creature module into lib.rs; Stage 2 complete"
```

---

## Verification Checklist

- [x] `v3/crates/v3-core/src/creature/` contains `genome.rs`, `state.rs`, `founder.rs`, `parseability.rs`, `mod.rs`
- [x] `VmInstruction` has exactly 33 variants (tested by `vm_instruction_count_is_33`)
- [x] `GraphNodeKind` has 22 variants (tested by `graph_node_kinds_constructible`)
- [x] `v3alpha1_founder_genome()` returns a 2-node genome matching spec Section 5.1
- [x] Founder genome passes parseability gate
- [x] Empty node list fails parseability
- [x] Duplicate IDs fail parseability
- [x] Unresolved entry_node_id passes (spec intent: runtime soft default handles this)
- [x] `cd v3 && cargo test --workspace` → all green
- [x] `cd v3 && cargo clippy --workspace --all-targets -- -D warnings` → 0 warnings
- [x] `scripts/check-plan-harness.sh --mode strict` → violations=0

## Reconciliation Snapshot (2026-02-22)

- verified: 11
- partial: 0
- missing: 0
- conflict: 0
- evidence matrix: `docs/plans/reconciliation/2026-02-22-v3-stage-2-evidence-matrix.md`

---

## Risks

- **Risk:** The founder VM program in the spec uses pseudo-code. The translation to `VmInstruction` structs requires careful mapping. Mitigation: each instruction is mapped explicitly in `founder.rs` with comments.
- **Risk:** `VmInstruction` enum may be large (~100-200 bytes for large variants). Mitigation: not a hot path concern in Stage 2; if needed, `Box<VmInstruction>` variants or the `mem-box-large-variant` rule can be applied later.
