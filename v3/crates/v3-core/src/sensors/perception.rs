use crate::contracts::WorldInputKey;
use crate::sensors::static_inputs::StaticInputs;

/// Number of ranked nearby-creature slots.
pub const NEARBY_SLOTS: usize = 4;

// ── Compound sub-field index constants ──────────────────────────────────────
// Per v3-sensor-spec.md Section 5. Prevents magic-number indexing in assembly.

/// `AreaFoodSummary` sub-field indices (width 7).
pub mod food_idx {
    pub const TOTAL_RATIO: usize = 0;
    pub const GRADIENT_X: usize = 1;
    pub const GRADIENT_Y: usize = 2;
    pub const NEAREST_DX: usize = 3;
    pub const NEAREST_DY: usize = 4;
    pub const NEAREST_DIST: usize = 5;
    pub const MAX_VALUE: usize = 6;
}

/// `AreaBarrierSummary` sub-field indices (width 7).
pub mod barrier_idx {
    pub const DENSITY_RATIO: usize = 0;
    pub const BLOCKED_ADJACENT_RATIO: usize = 1;
    pub const GRADIENT_X: usize = 2;
    pub const GRADIENT_Y: usize = 3;
    pub const NEAREST_DX: usize = 4;
    pub const NEAREST_DY: usize = 5;
    pub const NEAREST_DIST: usize = 6;
}

/// `AreaOccupancySummary` sub-field indices (width 7).
pub mod occupancy_idx {
    pub const COUNT_RATIO: usize = 0;
    pub const CENTER_X: usize = 1;
    pub const CENTER_Y: usize = 2;
    pub const NEAREST_DX: usize = 3;
    pub const NEAREST_DY: usize = 4;
    pub const NEAREST_DIST: usize = 5;
    pub const CROWDING_RATIO: usize = 6;
}

/// `NearbyCreatureCore` per-slot field count and offsets.
pub mod core_idx {
    pub const FIELDS_PER_SLOT: usize = 4;
    pub const PRESENT: usize = 0;
    pub const REL_X: usize = 1;
    pub const REL_Y: usize = 2;
    pub const DIST: usize = 3;
}

/// `NearbyCreatureVitals` per-slot field count and offsets.
pub mod vitals_idx {
    pub const FIELDS_PER_SLOT: usize = 2;
    pub const ENERGY_RATIO: usize = 0;
    pub const REPRODUCE_READY: usize = 1;
}

/// `NearbyCreatureIdentity` per-slot field count and offsets.
pub mod identity_idx {
    pub const FIELDS_PER_SLOT: usize = 3;
    pub const KIN_AFFINITY: usize = 0;
    pub const LINEAGE_MATCH: usize = 1;
    pub const PHENOTYPE_SIMILARITY: usize = 2;
}

/// Fixed-width extended perception snapshot assembled from the frozen world state.
///
/// Contains area summaries and nearby-creature detail banks per
/// `v3-sensor-spec.md` Section 5. Stack-allocated, no heap.
///
/// Use `PerceptionSnapshot::zero()` for non-perception genomes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerceptionSnapshot {
    /// Area food summary: [total_ratio, gradient_x, gradient_y, nearest_dx,
    ///   nearest_dy, nearest_dist, max_value]
    pub area_food: [f32; 7],
    /// Area barrier summary: [density_ratio, blocked_adjacent_ratio, gradient_x,
    ///   gradient_y, nearest_dx, nearest_dy, nearest_dist]
    pub area_barrier: [f32; 7],
    /// Area occupancy summary: [count_ratio, center_x, center_y, nearest_dx,
    ///   nearest_dy, nearest_dist, crowding_ratio]
    pub area_occupancy: [f32; 7],
    /// Nearby creature core: 4 slots × [present, rel_x, rel_y, dist]
    pub nearby_core: [f32; 16],
    /// Nearby creature vitals: 4 slots × [energy_ratio, reproduce_ready]
    pub nearby_vitals: [f32; 8],
    /// Nearby creature identity: 4 slots × [kin_affinity, lineage_match, phenotype_similarity]
    pub nearby_identity: [f32; 12],
}

impl PerceptionSnapshot {
    /// All-zeros fallback for non-perception genomes. By value, no allocation.
    #[must_use]
    pub const fn zero() -> Self {
        Self {
            area_food: [0.0; 7],
            area_barrier: [0.0; 7],
            area_occupancy: [0.0; 7],
            nearby_core: [0.0; 16],
            nearby_vitals: [0.0; 8],
            nearby_identity: [0.0; 12],
        }
    }

    /// Resolve an extended perception compound key at the given sub_idx.
    ///
    /// Returns 0.0 for out-of-range sub_idx or non-compound keys.
    ///
    /// NOTE: No longer used in the production resolution path. All compound
    /// resolution goes through `SensorSnapshot::resolve_compound()`, which
    /// wraps out-of-range sub_idx via modular arithmetic. This method is
    /// retained for direct-access use cases.
    #[inline]
    pub fn resolve(&self, key: &WorldInputKey, sub_idx: u16) -> f32 {
        let idx = sub_idx as usize;
        match key {
            WorldInputKey::AreaFoodSummary => *self.area_food.get(idx).unwrap_or(&0.0),
            WorldInputKey::AreaBarrierSummary => *self.area_barrier.get(idx).unwrap_or(&0.0),
            WorldInputKey::AreaOccupancySummary => *self.area_occupancy.get(idx).unwrap_or(&0.0),
            WorldInputKey::NearbyCreatureCore => *self.nearby_core.get(idx).unwrap_or(&0.0),
            WorldInputKey::NearbyCreatureVitals => *self.nearby_vitals.get(idx).unwrap_or(&0.0),
            WorldInputKey::NearbyCreatureIdentity => *self.nearby_identity.get(idx).unwrap_or(&0.0),
            // Local scalar keys are not resolved here.
            _ => 0.0,
        }
    }

    /// Returns true if this key is an extended perception compound key.
    #[inline]
    pub fn is_extended_key(key: &WorldInputKey) -> bool {
        matches!(
            key,
            WorldInputKey::AreaFoodSummary
                | WorldInputKey::AreaBarrierSummary
                | WorldInputKey::AreaOccupancySummary
                | WorldInputKey::NearbyCreatureCore
                | WorldInputKey::NearbyCreatureVitals
                | WorldInputKey::NearbyCreatureIdentity
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
    /// Extended perception summaries and nearby-creature banks.
    pub perception: PerceptionSnapshot,
}

impl SensorSnapshot {
    /// Resolve a compound WorldInputKey at the given sub_idx.
    ///
    /// Sub_idx is wrapped via `compound_width()` — out-of-range values wrap to
    /// valid indices. The `compound_width_matches_array_sizes` test ensures
    /// width constants stay in sync with actual array lengths.
    ///
    /// [`opt-inline-small`] Hot path — called per compound input per tick.
    #[inline]
    pub fn resolve_compound(&self, key: &WorldInputKey, sub_idx: u16) -> f32 {
        debug_assert!(
            key.compound_width() > 1,
            "scalar key routed to resolve_compound"
        );
        let idx = (sub_idx % key.compound_width()) as usize;
        match key {
            WorldInputKey::AreaFoodSummary => self.perception.area_food[idx],
            WorldInputKey::AreaBarrierSummary => self.perception.area_barrier[idx],
            WorldInputKey::AreaOccupancySummary => self.perception.area_occupancy[idx],
            WorldInputKey::NearbyCreatureCore => self.perception.nearby_core[idx],
            WorldInputKey::NearbyCreatureVitals => self.perception.nearby_vitals[idx],
            WorldInputKey::NearbyCreatureIdentity => self.perception.nearby_identity[idx],
            WorldInputKey::NeighborFoodRing => self.local.neighbor_food[idx],
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
}

impl Default for PerceptionConfig {
    fn default() -> Self {
        Self {
            vision_radius: 5,
            max_food_density: 1.0,
            max_energy: 200.0,
            min_reproduce_energy: 1.0,
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
    use crate::sensors::static_inputs::StaticInputs;

    #[test]
    fn perception_snapshot_size_within_budget() {
        assert!(
            std::mem::size_of::<PerceptionSnapshot>() <= 256,
            "PerceptionSnapshot is {} bytes, must be <= 256",
            std::mem::size_of::<PerceptionSnapshot>()
        );
    }

    #[test]
    fn perception_snapshot_zero_is_all_zeros() {
        let snap = PerceptionSnapshot::zero();
        assert_eq!(snap.area_food, [0.0; 7]);
        assert_eq!(snap.area_barrier, [0.0; 7]);
        assert_eq!(snap.area_occupancy, [0.0; 7]);
        assert_eq!(snap.nearby_core, [0.0; 16]);
        assert_eq!(snap.nearby_vitals, [0.0; 8]);
        assert_eq!(snap.nearby_identity, [0.0; 12]);
    }

    #[test]
    fn resolve_area_food_in_range() {
        let mut snap = PerceptionSnapshot::zero();
        snap.area_food[0] = 0.5;
        snap.area_food[6] = 0.9;
        assert!((snap.resolve(&WorldInputKey::AreaFoodSummary, 0) - 0.5).abs() < f32::EPSILON);
        assert!((snap.resolve(&WorldInputKey::AreaFoodSummary, 6) - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn resolve_area_food_out_of_range_returns_zero() {
        let snap = PerceptionSnapshot::zero();
        assert_eq!(snap.resolve(&WorldInputKey::AreaFoodSummary, 7), 0.0);
        assert_eq!(snap.resolve(&WorldInputKey::AreaFoodSummary, 100), 0.0);
    }

    #[test]
    fn resolve_nearby_core_in_range() {
        let mut snap = PerceptionSnapshot::zero();
        snap.nearby_core[0] = 1.0; // present
        snap.nearby_core[1] = 0.3; // rel_x
        assert!((snap.resolve(&WorldInputKey::NearbyCreatureCore, 0) - 1.0).abs() < f32::EPSILON);
        assert!((snap.resolve(&WorldInputKey::NearbyCreatureCore, 1) - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn resolve_nearby_core_out_of_range_returns_zero() {
        let snap = PerceptionSnapshot::zero();
        assert_eq!(snap.resolve(&WorldInputKey::NearbyCreatureCore, 16), 0.0);
    }

    #[test]
    fn resolve_nearby_identity_in_range() {
        let mut snap = PerceptionSnapshot::zero();
        snap.nearby_identity[0] = 0.75;
        assert!(
            (snap.resolve(&WorldInputKey::NearbyCreatureIdentity, 0) - 0.75).abs() < f32::EPSILON
        );
    }

    #[test]
    fn resolve_local_key_returns_zero() {
        let snap = PerceptionSnapshot::zero();
        assert_eq!(snap.resolve(&WorldInputKey::FoodHere, 0), 0.0);
    }

    #[test]
    fn is_extended_key_classification() {
        assert!(PerceptionSnapshot::is_extended_key(
            &WorldInputKey::AreaFoodSummary
        ));
        assert!(PerceptionSnapshot::is_extended_key(
            &WorldInputKey::NearbyCreatureIdentity
        ));
        assert!(!PerceptionSnapshot::is_extended_key(
            &WorldInputKey::FoodHere
        ));
        assert!(!PerceptionSnapshot::is_extended_key(
            &WorldInputKey::NeighborFoodRing
        ));
    }

    #[test]
    fn compound_width_matches_array_sizes() {
        // Extended perception arrays
        let p = PerceptionSnapshot::zero();
        assert_eq!(
            WorldInputKey::AreaFoodSummary.compound_width() as usize,
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
            WorldInputKey::NearbyCreatureCore.compound_width() as usize,
            p.nearby_core.len()
        );
        assert_eq!(
            WorldInputKey::NearbyCreatureVitals.compound_width() as usize,
            p.nearby_vitals.len()
        );
        assert_eq!(
            WorldInputKey::NearbyCreatureIdentity.compound_width() as usize,
            p.nearby_identity.len()
        );

        // Ring sensor arrays on StaticInputs
        let local = StaticInputs {
            food_here: 0.0,
            neighbor_food: [0.0; 8],
            neighbor_barrier: [0.0; 8],
            neighbor_occupied: [0.0; 8],
            generation: 0.0,
            age_ticks: 0.0,
        };
        assert_eq!(
            WorldInputKey::NeighborFoodRing.compound_width() as usize,
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
                input_refs: vec![InputReference::World(WorldInputKey::FoodHere)],
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
                    InputReference::World(WorldInputKey::FoodHere),
                    InputReference::World(WorldInputKey::AreaFoodSummary),
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
