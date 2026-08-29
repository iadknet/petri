use crate::config::OrdinaryFoodTypeId;
use crate::contracts::{Direction, InputReference, Position, WorldInputKey};
use crate::creature::genome::CreatureGenome;
use crate::kernel::WorldState;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TypedFoodLocalSnapshot {
    pub food_here_by_type: Vec<f32>,
    pub neighbor_food_by_type: Vec<[f32; 8]>,
}

impl TypedFoodLocalSnapshot {
    #[must_use]
    pub const fn zero() -> Self {
        Self {
            food_here_by_type: Vec::new(),
            neighbor_food_by_type: Vec::new(),
        }
    }

    #[must_use]
    pub fn zeroed(type_count: usize) -> Self {
        Self {
            food_here_by_type: vec![0.0; type_count],
            neighbor_food_by_type: vec![[0.0; 8]; type_count],
        }
    }

    #[must_use]
    pub fn food_here(&self, type_idx: OrdinaryFoodTypeId) -> f32 {
        self.food_here_by_type
            .get(usize::from(type_idx.get()))
            .copied()
            .unwrap_or(0.0)
    }

    #[must_use]
    pub fn neighbor_food(&self, type_idx: OrdinaryFoodTypeId, sub_idx: usize) -> f32 {
        self.neighbor_food_by_type
            .get(usize::from(type_idx.get()))
            .and_then(|ring| ring.get(sub_idx))
            .copied()
            .unwrap_or(0.0)
    }
}

#[must_use]
pub fn assemble_typed_food_local_snapshot(
    world: &WorldState,
    pos: Position,
) -> TypedFoodLocalSnapshot {
    let mut snapshot = TypedFoodLocalSnapshot::zeroed(world.food().food_type_count());
    for food_type in world.food().food_types() {
        let idx = usize::from(food_type.id.get());
        snapshot.food_here_by_type[idx] = world.food_at_type(pos, food_type.id).clamp(0.0, 1.0);

        let mut ring = [0.0; 8];
        for dir in Direction::ALL {
            let sub_idx = dir.to_index();
            if let Some(neighbor) = world.resolve_neighbor(pos, dir) {
                ring[sub_idx] = world.food_at_type(neighbor, food_type.id).clamp(0.0, 1.0);
            }
        }
        snapshot.neighbor_food_by_type[idx] = ring;
    }
    snapshot
}

/// Returns true when a genome references typed local-food keys that require
/// per-creature local food snapshot assembly.
#[must_use]
pub fn genome_uses_typed_local_food(genome: &CreatureGenome) -> bool {
    genome.nodes.iter().any(|node| {
        node.input_refs.iter().any(|input_ref| {
            matches!(
                input_ref,
                InputReference::World(
                    WorldInputKey::FoodHere { .. } | WorldInputKey::NeighborFoodRing { .. }
                )
            )
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::NodeId;
    use crate::creature::genome::{BackendDef, NodeGenome, VmBackendDef, VmInstruction};

    #[test]
    fn genome_uses_typed_local_food_true_for_food_here_or_neighbor_ring() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![
                    InputReference::World(WorldInputKey::FoodHere {
                        type_idx: OrdinaryFoodTypeId::default(),
                    }),
                    InputReference::World(WorldInputKey::NeighborFoodRing {
                        type_idx: OrdinaryFoodTypeId::new(2),
                    }),
                ],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![],
            }],
        };
        assert!(genome_uses_typed_local_food(&genome));
    }

    #[test]
    fn genome_uses_typed_local_food_false_for_non_food_inputs() {
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![InputReference::World(WorldInputKey::AreaBarrierSummary)],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![],
            }],
        };
        assert!(!genome_uses_typed_local_food(&genome));
    }

    #[test]
    fn zero_snapshot_soft_defaults_all_reads() {
        let snapshot = TypedFoodLocalSnapshot::zero();
        assert_eq!(
            snapshot.food_here(OrdinaryFoodTypeId::new(9)),
            0.0,
            "out-of-range food_here must soft-default to 0"
        );
        assert_eq!(
            snapshot.neighbor_food(OrdinaryFoodTypeId::new(9), 3),
            0.0,
            "out-of-range neighbor_food must soft-default to 0"
        );
    }
}
