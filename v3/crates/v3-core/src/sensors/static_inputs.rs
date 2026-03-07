use crate::contracts::{Direction, InputReference, StaticIntrospectionKey, WorldInputKey};
use crate::creature::state::CreatureState;
use crate::kernel::WorldState;

/// Snapshot of world and static-introspection sensor values for one creature's turn.
///
/// Assembled once at turn start; dynamic introspection (energy, consumed) is
/// resolved live during mesh evaluation.
///
/// All food values are normalized to [0.0, 1.0] by clamping raw food density.
#[derive(Debug, Clone, PartialEq)]
pub struct StaticInputs {
    /// Food density at the creature's current cell, normalized to [0.0, 1.0].
    pub food_here: f32,
    /// Food density of each neighboring cell, indexed by Direction::to_index().
    /// 0.0 for out-of-bounds (bounded edge mode) neighbors.
    pub neighbor_food: [f32; 8],
    /// Whether each neighboring cell has a barrier (1.0) or not (0.0).
    /// 0.0 for out-of-bounds neighbors.
    pub neighbor_barrier: [f32; 8],
    /// Whether each neighboring cell is occupied by a creature (1.0) or not (0.0).
    /// 0.0 for out-of-bounds neighbors.
    pub neighbor_occupied: [f32; 8],
    /// Creature's generation number cast to f32. Unbounded.
    pub generation: f32,
    /// Creature's age in ticks cast to f32. Unbounded.
    pub age_ticks: f32,
}

impl StaticInputs {
    /// Look up a resolved f32 value by WorldInputKey.
    ///
    /// Extended perception compound keys return 0.0 here — they are resolved
    /// through `PerceptionSnapshot::resolve()` in the `SensorSnapshot` path.
    pub fn resolve_world(&self, key: &WorldInputKey) -> f32 {
        match key {
            WorldInputKey::FoodHere => self.food_here,
            WorldInputKey::NeighborCellFood(dir) => self.neighbor_food[dir.to_index()],
            WorldInputKey::NeighborCellBarrier(dir) => self.neighbor_barrier[dir.to_index()],
            WorldInputKey::NeighborCellOccupied(dir) => self.neighbor_occupied[dir.to_index()],
            // Extended perception compounds — resolved via PerceptionSnapshot, not StaticInputs.
            WorldInputKey::AreaFoodSummary
            | WorldInputKey::AreaBarrierSummary
            | WorldInputKey::AreaOccupancySummary
            | WorldInputKey::NearbyCreatureCore
            | WorldInputKey::NearbyCreatureVitals
            | WorldInputKey::NearbyCreatureIdentity => 0.0,
        }
    }

    /// Look up a resolved f32 value by StaticIntrospectionKey.
    pub fn resolve_static(&self, key: &StaticIntrospectionKey) -> f32 {
        match key {
            StaticIntrospectionKey::Generation => self.generation,
            StaticIntrospectionKey::AgeTicks => self.age_ticks,
        }
    }
}

/// Assemble all static sensor values for one creature at turn start.
///
/// Edge-mode behaviour for neighbor cells:
/// - `WorldEdgeMode::Wrap`: neighbor coordinates are wrapped toroidally, so every
///   direction always resolves to a valid cell and is sampled normally.
/// - `WorldEdgeMode::Bounded`: a neighbor that falls outside the grid bounds resolves
///   to `None`. All sensor slots for such out-of-bounds neighbors are set to 0.0
///   (no food, no barrier signal, not occupied).
pub fn assemble_static_inputs(world: &WorldState, creature: &CreatureState) -> StaticInputs {
    let pos = creature.position;
    let food_here = world.food_at(pos).clamp(0.0, 1.0);

    let mut neighbor_food = [0.0f32; 8];
    let mut neighbor_barrier = [0.0f32; 8];
    let mut neighbor_occupied = [0.0f32; 8];

    for dir in Direction::ALL {
        let idx = dir.to_index();
        match world.resolve_neighbor(pos, dir) {
            Some(npos) => {
                neighbor_food[idx] = world.food_at(npos).clamp(0.0, 1.0);
                neighbor_barrier[idx] = if world.is_barrier(npos) { 1.0 } else { 0.0 };
                neighbor_occupied[idx] = if world.creature_at(npos).is_some() {
                    1.0
                } else {
                    0.0
                };
            }
            None => {
                // Bounded-edge out-of-bounds: soft default 0.0 for all fields.
            }
        }
    }

    StaticInputs {
        food_here,
        neighbor_food,
        neighbor_barrier,
        neighbor_occupied,
        generation: creature.generation as f32,
        age_ticks: creature.age as f32,
    }
}

/// Resolve an InputReference using only static and upstream data.
/// Dynamic introspection keys return 0.0 (resolved live by runtime).
pub fn resolve_static_ref(
    reference: &InputReference,
    static_inputs: &StaticInputs,
    upstream_slots: &[f32; 12],
) -> f32 {
    match reference {
        InputReference::World(key) => static_inputs.resolve_world(key),
        InputReference::StaticIntrospection(key) => static_inputs.resolve_static(key),
        InputReference::DynamicIntrospection(_) => 0.0,
        InputReference::UpstreamSlot(idx) => {
            if *idx < 12 {
                upstream_slots[*idx]
            } else {
                0.0
            }
        }
        InputReference::ActionQueue => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{SimulationConfig, WorldEdgeMode};
    use crate::contracts::{CreatureId, DynamicIntrospectionKey, NodeId, Position};
    use crate::creature::genome::{
        BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
    };
    use crate::creature::identity::CreatureIdentityState;
    use crate::creature::state::CreatureState;
    use crate::kernel::WorldState;
    use slotmap::SlotMap;

    fn make_world(w: u16, h: u16) -> WorldState {
        WorldState::new(w, h, WorldEdgeMode::Wrap)
    }

    fn make_creature(id: CreatureId, pos: Position) -> CreatureState {
        let genome = CreatureGenome {
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
        };
        CreatureState::new(
            id,
            genome,
            pos,
            20.0,
            0,
            [128, 64, 32, 10, 20, 30],
            0,
            [true; 6],
            CreatureIdentityState::default(),
            [0.0; 16],
        )
    }

    fn get_id() -> CreatureId {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        sm.insert(())
    }

    #[test]
    fn food_here_zero_when_no_food() {
        let world = make_world(4, 4);
        let id = get_id();
        let creature = make_creature(id, Position::new(2, 2));
        let si = assemble_static_inputs(&world, &creature);
        assert_eq!(si.food_here, 0.0);
    }

    #[test]
    fn food_here_normalized_to_one_when_max() {
        let mut world = make_world(4, 4);
        use rand::rngs::SmallRng;
        use rand::SeedableRng;
        let mut cfg = SimulationConfig::default();
        cfg.world.food.initial_coverage = 1.0;
        cfg.world.food.initial_density = 1.0;
        let mut rng = SmallRng::seed_from_u64(42);
        world.seed_food(&mut rng, &cfg);
        let id = get_id();
        let creature = make_creature(id, Position::new(1, 1));
        let si = assemble_static_inputs(&world, &creature);
        assert!((si.food_here - 1.0).abs() < 1e-6);
    }

    #[test]
    fn neighbor_food_all_one_when_all_cells_max_food() {
        let mut world = make_world(4, 4);
        use rand::rngs::SmallRng;
        use rand::SeedableRng;
        let mut cfg = SimulationConfig::default();
        cfg.world.food.initial_coverage = 1.0;
        cfg.world.food.initial_density = 1.0;
        let mut rng = SmallRng::seed_from_u64(0);
        world.seed_food(&mut rng, &cfg);
        let id = get_id();
        let creature = make_creature(id, Position::new(2, 2));
        let si = assemble_static_inputs(&world, &creature);
        for i in 0..8 {
            assert!(
                (si.neighbor_food[i] - 1.0).abs() < 1e-6,
                "neighbor_food[{i}]"
            );
        }
    }

    #[test]
    fn neighbor_barrier_detected_north() {
        let mut world = make_world(5, 5);
        world.set_barrier(Position::new(2, 1), true);
        let id = get_id();
        let creature = make_creature(id, Position::new(2, 2));
        let si = assemble_static_inputs(&world, &creature);
        assert_eq!(si.neighbor_barrier[Direction::N.to_index()], 1.0);
        assert_eq!(si.neighbor_barrier[Direction::S.to_index()], 0.0);
    }

    #[test]
    fn neighbor_occupied_detected_east() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id1 = sm.insert(());
        let id2 = sm.insert(());
        let mut world = make_world(5, 5);
        world.place_creature(Position::new(3, 2), id2);
        let creature = make_creature(id1, Position::new(2, 2));
        let si = assemble_static_inputs(&world, &creature);
        assert_eq!(si.neighbor_occupied[Direction::E.to_index()], 1.0);
        assert_eq!(si.neighbor_occupied[Direction::W.to_index()], 0.0);
    }

    #[test]
    fn static_introspection_generation_and_age() {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        let id = sm.insert(());
        let world = make_world(4, 4);
        let genome = CreatureGenome {
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
        };
        let mut creature = CreatureState::new(
            id,
            genome,
            Position::new(1, 1),
            20.0,
            5,
            [0; 6],
            0,
            [true; 6],
            CreatureIdentityState::default(),
            [0.0; 16],
        );
        creature.age = 42;
        let si = assemble_static_inputs(&world, &creature);
        assert!((si.generation - 5.0).abs() < 1e-6);
        assert!((si.age_ticks - 42.0).abs() < 1e-6);
    }

    #[test]
    fn resolve_world_food_here_zero() {
        let world = make_world(4, 4);
        let id = get_id();
        let creature = make_creature(id, Position::new(1, 1));
        let si = assemble_static_inputs(&world, &creature);
        let result = si.resolve_world(&WorldInputKey::FoodHere);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn resolve_static_ref_upstream_slot_in_range() {
        let world = make_world(4, 4);
        let id = get_id();
        let creature = make_creature(id, Position::new(0, 0));
        let si = assemble_static_inputs(&world, &creature);
        let mut upstream = [0.0f32; 12];
        upstream[3] = 7.5;
        let val = resolve_static_ref(&InputReference::UpstreamSlot(3), &si, &upstream);
        assert!((val - 7.5).abs() < 1e-6);
    }

    #[test]
    fn resolve_static_ref_upstream_slot_out_of_range_zero() {
        let world = make_world(4, 4);
        let id = get_id();
        let creature = make_creature(id, Position::new(0, 0));
        let si = assemble_static_inputs(&world, &creature);
        let upstream = [0.0f32; 12];
        let val = resolve_static_ref(&InputReference::UpstreamSlot(12), &si, &upstream);
        assert_eq!(val, 0.0);
    }

    #[test]
    fn resolve_static_ref_dynamic_introspection_yields_zero() {
        let world = make_world(4, 4);
        let id = get_id();
        let creature = make_creature(id, Position::new(0, 0));
        let si = assemble_static_inputs(&world, &creature);
        let upstream = [0.0f32; 12];
        let val = resolve_static_ref(
            &InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
            &si,
            &upstream,
        );
        assert_eq!(val, 0.0);
    }
}
