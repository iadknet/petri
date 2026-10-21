use crate::contracts::WorldInputKey;
use crate::sensors::reads::WorldReads;
use crate::sensors::static_inputs::StaticInputs;
use crate::sensors::typed_food::TypedFoodLocalSnapshot;

use crate::contracts::nearby_idx;

/// Fixed-width extended perception snapshot assembled from the frozen world state.
///
/// Contains area summaries and nearby-creature detail banks per
/// `v3-sensor-spec.md` Section 5. Stack-allocated, no heap.
///
/// Use `PerceptionSnapshot::zero()` for non-perception genomes.
#[derive(Debug, Clone, PartialEq)]
pub struct PerceptionSnapshot {
    /// Area food summary: [total_ratio, gradient_x, gradient_y, nearest_dx,
    ///   nearest_dy, nearest_dist, max_value]
    pub area_food: [f32; 7],
    /// Typed area-food summaries keyed by configured food type.
    pub typed_area_food: Vec<[f32; 7]>,
    /// Area barrier summary: [density_ratio, blocked_adjacent_ratio, gradient_x,
    ///   gradient_y, nearest_dx, nearest_dy, nearest_dist]
    pub area_barrier: [f32; 7],
    /// Area occupancy summary: [count_ratio, center_x, center_y, nearest_dx,
    ///   nearest_dy, nearest_dist, crowding_ratio]
    pub area_occupancy: [f32; 7],
    /// Nearby creatures: 4 slots × [present, rel_x, rel_y, dist,
    ///   energy_ratio, reproduce_ready, kin_affinity, red, green, blue],
    ///   laid out by [`nearby_idx`].
    pub nearby: [f32; nearby_idx::WIDTH],
}

impl PerceptionSnapshot {
    /// All-zeros fallback for non-perception genomes. By value, no allocation.
    #[must_use]
    pub const fn zero() -> Self {
        Self {
            area_food: [0.0; 7],
            typed_area_food: Vec::new(),
            area_barrier: [0.0; 7],
            area_occupancy: [0.0; 7],
            nearby: [0.0; nearby_idx::WIDTH],
        }
    }

    #[must_use]
    pub fn zeroed(type_count: usize) -> Self {
        Self {
            typed_area_food: vec![[0.0; 7]; type_count],
            ..Self::zero()
        }
    }

    /// Returns true if this key is an extended perception compound key.
    #[inline]
    pub fn is_extended_key(key: &WorldInputKey) -> bool {
        matches!(
            key,
            WorldInputKey::AreaFoodSummary { .. }
                | WorldInputKey::AreaBarrierSummary
                | WorldInputKey::AreaOccupancySummary
                | WorldInputKey::NearbyCreature
        )
    }
}

/// Unified runtime-facing sensor bundle per v3-sensor-spec.md Section 2.
///
/// Contains both the local radius-1 snapshot and the extended perception snapshot.
/// Runtime resolves all frozen inputs through this type.
#[derive(Debug, Clone, PartialEq)]
pub struct SensorSnapshot {
    /// Local/radius-1 world and static introspection values.
    pub local: StaticInputs,
    /// Typed local food snapshot keyed by configured food type.
    pub typed_local_food: TypedFoodLocalSnapshot,
    /// Extended perception summaries and nearby-creature banks.
    pub perception: PerceptionSnapshot,
    /// The world keys this creature's mesh run has resolved so far
    /// (observation only; `resolve_input` marks it).
    pub world_reads: WorldReads,
}

impl SensorSnapshot {
    /// Resolve a scalar world key from the frozen sensor snapshot.
    #[inline]
    pub fn resolve_world(&self, key: &WorldInputKey) -> f32 {
        match key {
            WorldInputKey::FoodHere { type_idx } => self.typed_local_food.food_here(*type_idx),
            _ => self.local.resolve_world(key),
        }
    }

    /// Resolve a compound WorldInputKey at the given sub_idx.
    ///
    /// Sub_idx is wrapped via `compound_width()` — out-of-range values wrap to
    /// valid indices. The `compound_width_matches_array_sizes` test ensures
    /// width constants stay in sync with actual array lengths.
    ///
    /// `opt-inline-small`: Hot path — called per compound input per tick.
    #[inline]
    pub fn resolve_compound(&self, key: &WorldInputKey, sub_idx: u16) -> f32 {
        debug_assert!(
            key.compound_width() > 1,
            "scalar key routed to resolve_compound"
        );
        let idx = (sub_idx % key.compound_width()) as usize;
        match key {
            WorldInputKey::AreaFoodSummary { type_idx } => self
                .perception
                .typed_area_food
                .get(usize::from(type_idx.get()))
                .and_then(|summary| summary.get(idx))
                .copied()
                .unwrap_or(0.0),
            WorldInputKey::AreaBarrierSummary => self.perception.area_barrier[idx],
            WorldInputKey::AreaOccupancySummary => self.perception.area_occupancy[idx],
            WorldInputKey::NearbyCreature => self.perception.nearby[idx],
            WorldInputKey::NeighborFoodRing { type_idx } => {
                self.typed_local_food.neighbor_food(*type_idx, idx)
            }
            WorldInputKey::NeighborBarrierRing => self.local.neighbor_barrier[idx],
            WorldInputKey::NeighborOccupiedRing => self.local.neighbor_occupied[idx],
            _ => 0.0,
        }
    }
}

/// Narrow config view consumed by perception reducers.
///
/// Per v3-runtime-config-spec.md: sole owner of `vision_radius`.
/// Production code should use `PerceptionConfig::from_sim_config()`;
/// `Default` is provided for tests and fallback contexts only.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerceptionConfig {
    /// Vision radius for area summaries. Default 5, valid 1..=8.
    pub vision_radius: u8,
    /// Maximum food density for normalization.
    pub max_food_density: f32,
    /// Maximum creature energy for normalization.
    pub max_energy: f32,
    /// Minimum energy to reproduce (for reproduce_ready signal).
    pub min_reproduce_energy: f32,
    /// Minimum age in ticks to reproduce (for reproduce_ready signal).
    pub min_reproduce_age: u64,
}

impl Default for PerceptionConfig {
    fn default() -> Self {
        Self {
            vision_radius: 5,
            max_food_density: 1.0,
            max_energy: 200.0,
            min_reproduce_energy: 1.0,
            min_reproduce_age: 20,
        }
    }
}

impl PerceptionConfig {
    /// Build from a `SimulationConfig` reference.
    #[must_use]
    pub fn from_sim_config(config: &crate::config::SimulationConfig) -> Self {
        Self {
            vision_radius: config.runtime.perception.vision_radius,
            max_food_density: config.world.food.max_density,
            max_energy: config.energy.lifecycle.max_energy,
            min_reproduce_energy: config.energy.lifecycle.min_reproduce_energy,
            min_reproduce_age: config.energy.lifecycle.min_reproduce_age,
        }
    }
}

/// Check if a creature genome references any extended perception world keys.
///
/// Used for conditional assembly: only genomes that actually read extended
/// perception inputs need full visibility computation and area reduction.
pub fn genome_uses_extended_perception(genome: &crate::creature::genome::CreatureGenome) -> bool {
    genome.nodes.iter().any(|node| {
        node.input_refs
            .iter()
            .any(|r| matches!(r, crate::contracts::InputReference::World(key) if PerceptionSnapshot::is_extended_key(key)))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OrdinaryFoodTypeId;
    use crate::sensors::static_inputs::StaticInputs;

    #[test]
    fn perception_snapshot_zero_is_all_zeros() {
        let snap = PerceptionSnapshot::zero();
        assert_eq!(snap.area_food, [0.0; 7]);
        assert_eq!(snap.area_barrier, [0.0; 7]);
        assert_eq!(snap.area_occupancy, [0.0; 7]);
        assert_eq!(snap.nearby, [0.0; 40]);
    }

    #[test]
    fn is_extended_key_classification() {
        assert!(PerceptionSnapshot::is_extended_key(
            &WorldInputKey::AreaFoodSummary {
                type_idx: OrdinaryFoodTypeId::default(),
            }
        ));
        assert!(PerceptionSnapshot::is_extended_key(
            &WorldInputKey::NearbyCreature
        ));
        assert!(!PerceptionSnapshot::is_extended_key(
            &WorldInputKey::FoodHere {
                type_idx: OrdinaryFoodTypeId::default(),
            }
        ));
        assert!(!PerceptionSnapshot::is_extended_key(
            &WorldInputKey::NeighborFoodRing {
                type_idx: OrdinaryFoodTypeId::default(),
            }
        ));
    }

    #[test]
    fn compound_width_matches_array_sizes() {
        // Extended perception arrays
        let p = PerceptionSnapshot::zero();
        assert_eq!(
            WorldInputKey::AreaFoodSummary {
                type_idx: OrdinaryFoodTypeId::default(),
            }
            .compound_width() as usize,
            p.area_food.len()
        );
        assert_eq!(
            WorldInputKey::AreaBarrierSummary.compound_width() as usize,
            p.area_barrier.len()
        );
        assert_eq!(
            WorldInputKey::AreaOccupancySummary.compound_width() as usize,
            p.area_occupancy.len()
        );
        assert_eq!(
            WorldInputKey::NearbyCreature.compound_width() as usize,
            p.nearby.len()
        );

        // Ring sensor arrays on StaticInputs
        let local = StaticInputs {
            food_here: 0.0,
            neighbor_food: [0.0; 8],
            neighbor_barrier: [0.0; 8],
            neighbor_occupied: [0.0; 8],
            max_energy: 200.0,
            age_ticks: 0.0,
            previous_outcome: [0.0; 4],
        };
        assert_eq!(
            WorldInputKey::NeighborFoodRing {
                type_idx: OrdinaryFoodTypeId::default(),
            }
            .compound_width() as usize,
            local.neighbor_food.len()
        );
        assert_eq!(
            WorldInputKey::NeighborBarrierRing.compound_width() as usize,
            local.neighbor_barrier.len()
        );
        assert_eq!(
            WorldInputKey::NeighborOccupiedRing.compound_width() as usize,
            local.neighbor_occupied.len()
        );
    }

    #[test]
    fn perception_config_default() {
        let cfg = PerceptionConfig::default();
        assert_eq!(cfg.vision_radius, 5);
        assert!((cfg.max_food_density - 1.0).abs() < f32::EPSILON);
        assert!((cfg.max_energy - 200.0).abs() < f32::EPSILON);
        assert!((cfg.min_reproduce_energy - 1.0).abs() < f32::EPSILON);
        assert_eq!(cfg.min_reproduce_age, 20);
    }

    #[test]
    fn perception_config_from_sim_config_matches_defaults() {
        let sim_cfg = crate::config::SimulationConfig::default();
        let p = PerceptionConfig::from_sim_config(&sim_cfg);
        assert_eq!(p.vision_radius, 5);
        assert!((p.max_food_density - sim_cfg.world.food.max_density).abs() < f32::EPSILON);
        assert!((p.max_energy - sim_cfg.energy.lifecycle.max_energy).abs() < f32::EPSILON);
        assert!(
            (p.min_reproduce_energy - sim_cfg.energy.lifecycle.min_reproduce_energy).abs()
                < f32::EPSILON
        );
        assert_eq!(
            p.min_reproduce_age,
            sim_cfg.energy.lifecycle.min_reproduce_age
        );
    }

    #[test]
    fn genome_uses_extended_perception_false_for_local_only() {
        use crate::contracts::{InputReference, NodeId};
        use crate::creature::genome::{
            BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
        };
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![InputReference::World(WorldInputKey::FoodHere {
                    type_idx: OrdinaryFoodTypeId::default(),
                })],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 1,
                    constants: vec![],
                    program: vec![VmInstruction::Halt],
                }),
                targets: vec![],
            }],
        };
        assert!(!genome_uses_extended_perception(&genome));
    }

    #[test]
    fn genome_uses_extended_perception_true_for_area_food() {
        use crate::contracts::{InputReference, NodeId};
        use crate::creature::genome::{
            BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
        };
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![
                    InputReference::World(WorldInputKey::FoodHere {
                        type_idx: OrdinaryFoodTypeId::default(),
                    }),
                    InputReference::World(WorldInputKey::AreaFoodSummary {
                        type_idx: OrdinaryFoodTypeId::default(),
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
        assert!(genome_uses_extended_perception(&genome));
    }
}
