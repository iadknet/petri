use crate::creature::genome::{
    BackendDef, CreatureGenome, InputReference, IntrospectionInputKey, NodeGenome, NodeId,
    OutputDefinition, VmBackendDef, VmInstruction, WorldInputKey,
};

/// Return the named founder genome. Panics with a clear message on unknown names.
///
/// # Founders
///
/// - `"simple"`: Minimal viable VM creature. Eats food at its cell, reproduces
///   toward N when energy ≥ 300, moves toward N if food is there, otherwise
///   moves E. Sustains a viable population under the non-collapse contract.
pub fn get(name: &str) -> CreatureGenome {
    match name {
        "simple" => simple_founder(),
        other => panic!("unknown founder genome name: '{other}'"),
    }
}

/// 21-instruction VM program: eat → reproduce N → move N (if food) → move E.
///
/// Constants:
///   [0] = 0.0  (zero threshold)
///   [1] = 50.0 (reproduce energy gate — achievable under both default max=100 and
///               stage3-test max=150; the action layer enforces min_reproduce_energy)
///   [2] = 100.0 (offspring energy bid; action layer caps it at default_offspring_energy)
///   [3] = 2.0   (direction E = Direction::ALL index 2)
///
/// input_refs:
///   slot 0 → World(FoodHere)         → food_density / 255.0
///   slot 1 → Introspection(Energy)   → energy as f32
///
/// Jump offset rule: JumpIfZero at index N with offset O → if falsy, land at (N+1)+O.
fn simple_founder() -> CreatureGenome {
    use VmInstruction::*;

    let program = vec![
        // ── Eat phase ────────────────────────────────────────────────────────
        ReadInput {
            dst: 0,
            input_idx: 0,
        }, // r0 = food_here_norm
        LoadConst {
            dst: 6,
            const_idx: 0,
        }, // r6 = 0.0
        CmpGt { dst: 1, a: 0, b: 6 },       // r1 = food > 0?
        JumpIfZero { cond: 1, offset: 1 },  // no food → skip to instr 5 (PC=4+1=5)
        EmitWorldAction { action_type: 1 }, // Eat, halt
        // ── Reproduce phase ───────────────────────────────────────────────
        ReadInput {
            dst: 2,
            input_idx: 1,
        }, // r2 = energy_current
        LoadConst {
            dst: 3,
            const_idx: 1,
        }, // r3 = 50.0
        CmpGt { dst: 4, a: 2, b: 3 },      // r4 = energy > 50?
        JumpIfZero { cond: 4, offset: 4 }, // not enough → skip to instr 13 (PC=9+4=13)
        WriteWorldActionMeta {
            slot_idx: 0,
            src: 6,
        }, // meta[0] = 0.0 (dir N = 0)
        LoadConst {
            dst: 5,
            const_idx: 2,
        }, // r5 = 100.0 (offspring energy)
        WriteWorldActionMeta {
            slot_idx: 1,
            src: 5,
        }, // meta[1] = 100.0
        EmitWorldAction { action_type: 3 }, // Reproduce N, halt
        // ── Move phase ────────────────────────────────────────────────────
        ReadNeighborCell {
            dst: 0,
            neighbor_idx: 0,
            field_idx: 0,
        }, // r0 = N food norm
        CmpGt { dst: 1, a: 0, b: 6 },      // food at N?
        JumpIfZero { cond: 1, offset: 2 }, // no food → skip to instr 18 (PC=16+2=18)
        WriteWorldActionMeta {
            slot_idx: 0,
            src: 6,
        }, // meta[0] = 0.0 (dir N)
        EmitWorldAction { action_type: 2 }, // Move N, halt
        LoadConst {
            dst: 0,
            const_idx: 3,
        }, // r0 = 2.0 (dir E)
        WriteWorldActionMeta {
            slot_idx: 0,
            src: 0,
        }, // meta[0] = 2.0 (dir E)
        EmitWorldAction { action_type: 2 }, // Move E, halt
    ];

    let node_id = NodeId(0);
    let node = NodeGenome {
        node_id,
        input_refs: vec![
            InputReference::World(WorldInputKey::FoodHere),
            InputReference::Introspection(IntrospectionInputKey::EnergyCurrent),
        ],
        backend_def: BackendDef::Vm(VmBackendDef {
            register_count: 8,
            program,
            constants: vec![0.0, 50.0, 100.0, 2.0],
        }),
        output_definitions: Vec::<OutputDefinition>::new(),
    };

    CreatureGenome {
        entry_node_id: node_id,
        nodes: vec![node],
    }
}
