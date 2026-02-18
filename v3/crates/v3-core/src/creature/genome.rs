/// Unique identifier for a node within a creature's genome.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);

// ── Input references ──────────────────────────────────────────────────────────

/// A world-observable input value the VM can read via ReadInput.
#[derive(Clone, Debug, PartialEq)]
pub enum WorldInputKey {
    /// Normalized food density at the creature's current cell (food_u8 / 255.0).
    FoodHere,
    // Stage 4+: NearestFoodDistance, NearestFoodDirection, etc.
}

/// An introspection input value the VM can read via ReadInput.
#[derive(Clone, Debug, PartialEq)]
pub enum IntrospectionInputKey {
    /// Raw creature energy as f32.
    EnergyCurrent,
    /// Raw generation count as f32.
    Generation,
    /// Raw age in ticks as f32.
    AgeTicks,
}

/// A neighbor cell field the VM can read via ReadNeighborCell.
/// field_idx values: 0=FoodDensityNorm, 1=BarrierFlag, 2=OccupiedFlag.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NeighborCellField {
    /// Normalized food density at neighbor cell (food_u8 / 255.0).
    FoodDensityNorm = 0,
    /// 1.0 if the neighbor cell is a barrier, else 0.0.
    BarrierFlag = 1,
    /// 1.0 if the neighbor cell is occupied by a creature, else 0.0.
    OccupiedFlag = 2,
}

/// A neighbor creature field the VM can read via ReadNeighborCreature.
/// field_idx values: 0=PresentFlag, 1=EnergyNorm (Stage 4+ only).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NeighborCreatureField {
    /// 1.0 if a creature is present at the neighbor cell, else 0.0.
    PresentFlag = 0,
    // Stage 4+: EnergyNorm, PhenotypeRNorm, etc.
}

/// A typed input reference for a VM node's ReadInput slot mapping.
/// Each entry in a node's `input_refs` list corresponds to a zero-indexed slot
/// (per genome-sensor-spec §9). `ReadInput(idx)` reads slot `idx`.
#[derive(Clone, Debug, PartialEq)]
pub enum InputReference {
    /// A world-observable input (food, distances, etc.).
    World(WorldInputKey),
    /// A creature introspection input (energy, age, generation).
    Introspection(IntrospectionInputKey),
    /// A neighbor cell field at a given direction index (0–7, Direction::ALL order).
    NeighborCell {
        direction_idx: u8,
        field: NeighborCellField,
    },
    /// A neighbor creature field at a given direction index (0–7).
    NeighborCreature {
        direction_idx: u8,
        field: NeighborCreatureField,
    },
    // Stage 4+: SensorCell, SensorCreature, SensorSummary, Packet variants.
}

// ── VM instruction set (38 opcodes) ──────────────────────────────────────────

/// All 38 VM opcodes as defined in `docs/reference/v3-vm-isa-spec.md`.
/// Operand naming:
///   - `dst`, `src`, `a`, `b`: register indices (u8).
///   - `const_idx`: constant pool index (u8).
///   - `offset`: signed PC jump offset (i16; applied to post-increment PC).
///   - `input_idx`, `slot_idx`: buffer slot index (u8).
///   - `action_type`: world-action type encoding (u8; 0=NoOp,1=Eat,2=Move,3=Reproduce).
///   - `sensor_idx`, `field_idx`, `neighbor_idx`, `summary_idx`: query indices (u8).
///   - `imm_addr`: immediate memory address (u16; resolved mod 1024 at runtime).
///   - `addr_reg`: register holding memory address.
///   - `eps`: epsilon register index for CmpEq.
#[derive(Clone, Debug, PartialEq)]
pub enum VmInstruction {
    // Arithmetic and data movement
    Noop,
    LoadConst {
        dst: u8,
        const_idx: u8,
    },
    Move {
        dst: u8,
        src: u8,
    },
    Add {
        dst: u8,
        a: u8,
        b: u8,
    },
    Sub {
        dst: u8,
        a: u8,
        b: u8,
    },
    Mul {
        dst: u8,
        a: u8,
        b: u8,
    },
    Div {
        dst: u8,
        a: u8,
        b: u8,
    },
    Min {
        dst: u8,
        a: u8,
        b: u8,
    },
    Max {
        dst: u8,
        a: u8,
        b: u8,
    },
    Abs {
        dst: u8,
        src: u8,
    },
    Neg {
        dst: u8,
        src: u8,
    },
    Clamp01 {
        dst: u8,
        src: u8,
    },
    // Comparison and logic
    CmpGt {
        dst: u8,
        a: u8,
        b: u8,
    },
    CmpLt {
        dst: u8,
        a: u8,
        b: u8,
    },
    CmpEq {
        dst: u8,
        a: u8,
        b: u8,
        eps: u8,
    },
    And {
        dst: u8,
        a: u8,
        b: u8,
    },
    Or {
        dst: u8,
        a: u8,
        b: u8,
    },
    Not {
        dst: u8,
        src: u8,
    },
    // Type conversion
    ToI32 {
        dst: u8,
        src: u8,
    },
    ToU8 {
        dst: u8,
        src: u8,
    },
    ToBool {
        dst: u8,
        src: u8,
    },
    // Control flow
    JumpIfZero {
        cond: u8,
        offset: i16,
    },
    Jump {
        offset: i16,
    },
    // Input / sensor reads
    ReadInput {
        dst: u8,
        input_idx: u8,
    },
    ReadSensorCell {
        dst: u8,
        sensor_idx: u8,
        field_idx: u8,
    },
    ReadSensorCreature {
        dst: u8,
        sensor_idx: u8,
        field_idx: u8,
    },
    ReadSensorSummary {
        dst: u8,
        summary_idx: u8,
    },
    ReadNeighborCell {
        dst: u8,
        neighbor_idx: u8,
        field_idx: u8,
    },
    ReadNeighborCreature {
        dst: u8,
        neighbor_idx: u8,
        field_idx: u8,
    },
    // Output / action writes
    WriteInternalPayload {
        slot_idx: u8,
        src: u8,
    },
    WriteWorldActionMeta {
        slot_idx: u8,
        src: u8,
    },
    EmitInternal {
        action_type: u8,
    },
    EmitWorldAction {
        action_type: u8,
    },
    // Halt
    Halt,
    // Memory access
    LoadMem8 {
        dst: u8,
        addr_reg: u8,
    },
    StoreMem8 {
        addr_reg: u8,
        src: u8,
    },
    LoadMem8Imm {
        dst: u8,
        imm_addr: u16,
    },
    StoreMem8Imm {
        imm_addr: u16,
        src: u8,
    },
}

// ── Backend definition ────────────────────────────────────────────────────────

/// VM-specific backend definition for a node.
#[derive(Clone, Debug, PartialEq)]
pub struct VmBackendDef {
    /// Number of registers available to this node's VM (1–64).
    pub register_count: u8,
    /// VM instruction sequence. Empty program halts immediately.
    pub program: Vec<VmInstruction>,
    /// Constant pool referenced by `LoadConst`. Up to 64 entries.
    pub constants: Vec<f32>,
}

/// Backend definition for a node. Stage 3C: Vm only. Graph added in Stage 4+.
#[derive(Clone, Debug, PartialEq)]
pub enum BackendDef {
    Vm(VmBackendDef),
    // Stage 4+: Graph(GraphBackendDef)
}

// ── Output routing (Stage 4+ expansion point) ─────────────────────────────────

/// Output routing definition. Always empty in Stage 3C; extended in Stage 4+
/// with WorldAction and InternalTarget variants for multi-node mesh routing.
#[derive(Clone, Debug, PartialEq)]
pub enum OutputDefinition {
    // Stage 4+: WorldAction { ... }, InternalTarget { ... }
}

// ── Node and genome ───────────────────────────────────────────────────────────

/// A single node in the creature's genome mesh.
#[derive(Clone, Debug, PartialEq)]
pub struct NodeGenome {
    /// Unique identifier for this node within the genome.
    pub node_id: NodeId,
    /// Typed input slot mapping for `ReadInput` opcode (spec §9).
    /// `ReadInput(idx)` resolves against `input_refs[idx]`.
    pub input_refs: Vec<InputReference>,
    /// Backend-specific definition (VM program, graph operators, etc.).
    pub backend_def: BackendDef,
    /// Output routing definitions. Always empty in Stage 3C.
    pub output_definitions: Vec<OutputDefinition>,
}

/// A creature's heritable genome: the entry node and all nodes in the mesh.
#[derive(Clone, Debug, PartialEq)]
pub struct CreatureGenome {
    /// First node evaluated each tick.
    pub entry_node_id: NodeId,
    /// All nodes in the genome mesh.
    pub nodes: Vec<NodeGenome>,
}

impl CreatureGenome {
    /// Validate structural invariants.
    ///
    /// Returns `Ok(())` if valid:
    /// - `nodes` is non-empty.
    /// - All `node_id` values are unique.
    /// - `entry_node_id` matches an existing node.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.nodes.is_empty() {
            return Err("genome must have at least one node");
        }

        // Check unique node IDs
        let mut seen = std::collections::HashSet::new();
        for node in &self.nodes {
            if !seen.insert(node.node_id) {
                return Err("genome has duplicate node IDs");
            }
        }

        // Check entry node exists
        if !self.nodes.iter().any(|n| n.node_id == self.entry_node_id) {
            return Err("entry_node_id does not match any node in the genome");
        }

        Ok(())
    }
}
