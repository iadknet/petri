//! Area reducers and nearby-creature ranking for extended perception.
//!
//! Per v3-sensor-spec.md Sections 5–6: assembles `PerceptionSnapshot` fields
//! from the set of visible cells computed by the visibility module.

use crate::config::OrdinaryFoodTypeId;
use crate::contracts::{barrier_idx, food_idx, nearby_idx, occupancy_idx, CreatureId};
use crate::kernel::WorldState;
use crate::sensors::perception::{PerceptionConfig, PerceptionSnapshot};
use crate::sensors::static_inputs::{BodyView, BodyViews};
use crate::sensors::visibility::VisibleCell;

/// Assemble a complete `PerceptionSnapshot` from visible cells.
///
/// Per v3-sensor-spec.md Sections 5–6:
/// - Area summaries use full candidate-window maxima as denominators
/// - Self cell included for food, excluded for occupancy/creature ranking
/// - All identity similarity values clamped to `[0.0, 1.0]`
///
/// The observer is `views[observer_id]`; a visible creature without a view
/// is skipped.
pub fn assemble_perception(
    observer_id: CreatureId,
    visible: &[VisibleCell],
    world: &WorldState,
    views: &BodyViews,
    config: &PerceptionConfig,
) -> PerceptionSnapshot {
    let r = config.vision_radius as f32;
    let max_candidate_cells = (2.0 * r + 1.0) * (2.0 * r + 1.0);
    let max_other_candidate_cells = max_candidate_cells - 1.0;
    let max_dist = (2.0 * r * r).sqrt();
    let typed_area_food: Vec<_> = world
        .food()
        .food_types()
        .iter()
        .map(|food_type| {
            reduce_food(
                visible,
                world,
                config,
                food_type.id,
                r,
                max_candidate_cells,
                max_dist,
            )
        })
        .collect();
    let area_food = typed_area_food
        .get(usize::from(OrdinaryFoodTypeId::default().get()))
        .copied()
        .unwrap_or([0.0; 7]);
    let area_barrier = reduce_barrier(visible, world, r, max_candidate_cells, max_dist);
    let area_occupancy = reduce_occupancy(
        visible,
        world,
        observer_id,
        r,
        max_other_candidate_cells,
        max_dist,
    );

    let nearby = rank_nearby_creatures(
        visible,
        world,
        observer_id,
        &views[observer_id],
        views,
        config,
        r,
        max_dist,
    );

    PerceptionSnapshot {
        area_food,
        typed_area_food,
        area_barrier,
        area_occupancy,
        nearby,
    }
}

/// Reduce food area summary per v3-sensor-spec.md Section 6.1.
fn reduce_food(
    visible: &[VisibleCell],
    world: &WorldState,
    config: &PerceptionConfig,
    type_idx: OrdinaryFoodTypeId,
    r: f32,
    max_candidate_cells: f32,
    max_dist: f32,
) -> [f32; 7] {
    let mut result = [0.0f32; 7];
    let max_food = config.max_food_density;
    if max_food <= 0.0 {
        return result;
    }

    let mut food_sum = 0.0f32;
    let mut grad_x_sum = 0.0f32;
    let mut grad_y_sum = 0.0f32;
    let mut max_value = 0.0f32;
    let mut nearest_dist_sq = f32::MAX;
    let mut nearest_dx = 0.0f32;
    let mut nearest_dy = 0.0f32;

    for cell in visible {
        let food = world.food_at_type(cell.pos, type_idx);
        let food_ratio = (food / max_food).clamp(0.0, 1.0);

        food_sum += food_ratio;

        if r > 0.0 {
            let dx_norm = cell.dx as f32 / r;
            let dy_norm = cell.dy as f32 / r;
            grad_x_sum += dx_norm * food_ratio;
            grad_y_sum += dy_norm * food_ratio;
        }

        if food_ratio > max_value {
            max_value = food_ratio;
        }

        if food_ratio > 0.0 {
            let dist_sq = (cell.dx * cell.dx + cell.dy * cell.dy) as f32;
            if dist_sq < nearest_dist_sq {
                nearest_dist_sq = dist_sq;
                nearest_dx = cell.dx as f32;
                nearest_dy = cell.dy as f32;
            }
        }
    }

    result[food_idx::TOTAL_RATIO] = food_sum / max_candidate_cells;
    result[food_idx::GRADIENT_X] = grad_x_sum / max_candidate_cells;
    result[food_idx::GRADIENT_Y] = grad_y_sum / max_candidate_cells;
    result[food_idx::MAX_VALUE] = max_value;

    if nearest_dist_sq < f32::MAX {
        result[food_idx::NEAREST_DX] = nearest_dx / r;
        result[food_idx::NEAREST_DY] = nearest_dy / r;
        result[food_idx::NEAREST_DIST] = nearest_dist_sq.sqrt() / max_dist;
    }

    result
}

/// Reduce barrier area summary per v3-sensor-spec.md Section 6.2.
fn reduce_barrier(
    visible: &[VisibleCell],
    world: &WorldState,
    r: f32,
    max_candidate_cells: f32,
    max_dist: f32,
) -> [f32; 7] {
    let mut result = [0.0f32; 7];

    let mut barrier_count = 0.0f32;
    let mut grad_x_sum = 0.0f32;
    let mut grad_y_sum = 0.0f32;
    let mut nearest_dist_sq = f32::MAX;
    let mut nearest_dx = 0.0f32;
    let mut nearest_dy = 0.0f32;

    // Count blocked adjacent cells among 8 immediate neighbors.
    let mut adjacent_barrier_count = 0u32;
    for cell in visible {
        let is_barrier = world.is_barrier(cell.pos);
        let barrier_val = if is_barrier { 1.0f32 } else { 0.0f32 };

        if is_barrier {
            barrier_count += 1.0;
        }

        if r > 0.0 {
            let dx_norm = cell.dx as f32 / r;
            let dy_norm = cell.dy as f32 / r;
            grad_x_sum += dx_norm * barrier_val;
            grad_y_sum += dy_norm * barrier_val;
        }

        if is_barrier {
            let dist_sq = (cell.dx * cell.dx + cell.dy * cell.dy) as f32;
            if dist_sq < nearest_dist_sq {
                nearest_dist_sq = dist_sq;
                nearest_dx = cell.dx as f32;
                nearest_dy = cell.dy as f32;
            }

            // Check if this is an adjacent cell (Chebyshev distance 1).
            if cell.dx.abs() <= 1 && cell.dy.abs() <= 1 && !(cell.dx == 0 && cell.dy == 0) {
                adjacent_barrier_count += 1;
            }
        }
    }

    result[barrier_idx::DENSITY_RATIO] = barrier_count / max_candidate_cells;
    result[barrier_idx::BLOCKED_ADJACENT_RATIO] = adjacent_barrier_count as f32 / 8.0;
    result[barrier_idx::GRADIENT_X] = grad_x_sum / max_candidate_cells;
    result[barrier_idx::GRADIENT_Y] = grad_y_sum / max_candidate_cells;

    if nearest_dist_sq < f32::MAX {
        result[barrier_idx::NEAREST_DX] = nearest_dx / r;
        result[barrier_idx::NEAREST_DY] = nearest_dy / r;
        result[barrier_idx::NEAREST_DIST] = nearest_dist_sq.sqrt() / max_dist;
    }

    result
}

/// Reduce occupancy area summary per v3-sensor-spec.md Section 6.3.
fn reduce_occupancy(
    visible: &[VisibleCell],
    world: &WorldState,
    observer_id: CreatureId,
    r: f32,
    max_other_candidate_cells: f32,
    max_dist: f32,
) -> [f32; 7] {
    let mut result = [0.0f32; 7];
    if max_other_candidate_cells <= 0.0 {
        return result;
    }

    let mut count = 0.0f32;
    let mut center_x_sum = 0.0f32;
    let mut center_y_sum = 0.0f32;
    let mut crowding_sum = 0.0f32;
    let mut nearest_dist_sq = f32::MAX;
    let mut nearest_dx = 0.0f32;
    let mut nearest_dy = 0.0f32;

    for cell in visible {
        // Skip self cell
        if cell.dx == 0 && cell.dy == 0 {
            continue;
        }

        let Some(creature_id) = world.creature_at(cell.pos) else {
            continue;
        };

        // Skip self
        if creature_id == observer_id {
            continue;
        }

        count += 1.0;

        let dx_norm = if r > 0.0 { cell.dx as f32 / r } else { 0.0 };
        let dy_norm = if r > 0.0 { cell.dy as f32 / r } else { 0.0 };
        center_x_sum += dx_norm;
        center_y_sum += dy_norm;

        let dist_sq = (cell.dx * cell.dx + cell.dy * cell.dy) as f32;
        let euclidean = dist_sq.sqrt();
        let dist_norm = if max_dist > 0.0 {
            euclidean / max_dist
        } else {
            0.0
        };
        crowding_sum += 1.0 - dist_norm;

        if dist_sq < nearest_dist_sq {
            nearest_dist_sq = dist_sq;
            nearest_dx = cell.dx as f32;
            nearest_dy = cell.dy as f32;
        }
    }

    result[occupancy_idx::COUNT_RATIO] = count / max_other_candidate_cells;
    if count > 0.0 {
        result[occupancy_idx::CENTER_X] = center_x_sum / count;
        result[occupancy_idx::CENTER_Y] = center_y_sum / count;
    }
    result[occupancy_idx::CROWDING_RATIO] = crowding_sum / max_other_candidate_cells;

    if nearest_dist_sq < f32::MAX {
        result[occupancy_idx::NEAREST_DX] = nearest_dx / r;
        result[occupancy_idx::NEAREST_DY] = nearest_dy / r;
        result[occupancy_idx::NEAREST_DIST] = nearest_dist_sq.sqrt() / max_dist;
    }

    result
}

/// Candidate for nearby-creature ranking.
struct NearbyCandidate {
    creature_id: CreatureId,
    dx: i32,
    dy: i32,
    euclidean_dist: f32,
    chebyshev_dist: i32,
}

/// Rank visible non-self creatures and fill the `NearbyCreature` bank.
///
/// Per v3-sensor-spec.md Section 5.4–5.6:
/// - 4 ranked slots, ascending by euclidean → chebyshev → (dy,dx) → creature_id
/// - Empty slots are all zeros
#[allow(clippy::too_many_arguments)]
fn rank_nearby_creatures(
    visible: &[VisibleCell],
    world: &WorldState,
    observer_id: CreatureId,
    observer: &BodyView,
    views: &BodyViews,
    config: &PerceptionConfig,
    r: f32,
    max_dist: f32,
) -> [f32; nearby_idx::WIDTH] {
    let mut candidates: Vec<NearbyCandidate> = Vec::new();
    for cell in visible {
        if cell.dx == 0 && cell.dy == 0 {
            continue;
        }
        let Some(cid) = world.creature_at(cell.pos) else {
            continue;
        };
        if cid == observer_id {
            continue;
        }
        let euclidean = ((cell.dx * cell.dx + cell.dy * cell.dy) as f32).sqrt();
        let chebyshev = cell.dx.abs().max(cell.dy.abs());
        candidates.push(NearbyCandidate {
            creature_id: cid,
            dx: cell.dx,
            dy: cell.dy,
            euclidean_dist: euclidean,
            chebyshev_dist: chebyshev,
        });
    }

    // Sensor ordering is contract-visible: tie-break by euclidean distance,
    // then Chebyshev distance, then stable local offsets, then creature ID.
    candidates.sort_by(|a, b| {
        a.euclidean_dist
            .partial_cmp(&b.euclidean_dist)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.chebyshev_dist.cmp(&b.chebyshev_dist))
            .then_with(|| a.dy.cmp(&b.dy))
            .then_with(|| a.dx.cmp(&b.dx))
            .then_with(|| a.creature_id.cmp(&b.creature_id))
    });

    fill_nearby_banks(&candidates, observer, views, config, r, max_dist)
}

fn fill_nearby_banks(
    candidates: &[NearbyCandidate],
    observer: &BodyView,
    views: &BodyViews,
    config: &PerceptionConfig,
    r: f32,
    max_dist: f32,
) -> [f32; nearby_idx::WIDTH] {
    let mut nearby = [0.0f32; nearby_idx::WIDTH];

    for (slot, candidate) in candidates.iter().take(nearby_idx::SLOTS).enumerate() {
        let Some(target) = views.get(candidate.creature_id) else {
            continue;
        };
        let fields = &mut nearby
            [slot * nearby_idx::FIELDS_PER_SLOT..(slot + 1) * nearby_idx::FIELDS_PER_SLOT];

        // Position
        fields[nearby_idx::PRESENT] = 1.0;
        fields[nearby_idx::REL_X] = if r > 0.0 {
            candidate.dx as f32 / r
        } else {
            0.0
        };
        fields[nearby_idx::REL_Y] = if r > 0.0 {
            candidate.dy as f32 / r
        } else {
            0.0
        };
        fields[nearby_idx::DIST] = if max_dist > 0.0 {
            candidate.euclidean_dist / max_dist
        } else {
            0.0
        };

        // Vitals
        fields[nearby_idx::ENERGY_RATIO] = if target.energy_scale > 0.0 {
            (target.energy / target.energy_scale).clamp(0.0, 1.0)
        } else {
            0.0
        };
        fields[nearby_idx::REPRODUCE_READY] = if target.energy >= config.min_reproduce_energy
            && target.age >= config.min_reproduce_age
        {
            1.0
        } else {
            0.0
        };

        // Identity and colour
        fields[nearby_idx::KIN_AFFINITY] = compute_kin_affinity(observer.kin_tag, target.kin_tag);
        fields[nearby_idx::RED..=nearby_idx::BLUE].copy_from_slice(&target.colour);
    }

    nearby
}

/// Compute kin affinity: `1.0 - popcount(a ^ b) / 32.0`, clamped to `[0.0, 1.0]`.
fn compute_kin_affinity(a: u32, b: u32) -> f32 {
    let hamming = (a ^ b).count_ones();
    (1.0 - hamming as f32 / 32.0).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EnergyCapacityConfig;
    use crate::config::WorldEdgeMode;
    use crate::creature::body::HeritableBody;
    use crate::creature::founder::v3alpha1_founder_genome;
    use crate::creature::identity::CreatureIdentityState;
    use crate::creature::phenotype::Phenotype;
    use crate::sensors::visibility::{compute_visible_cells, get_visibility_table};
    use slotmap::SlotMap;

    fn make_world(w: u16, h: u16) -> WorldState {
        WorldState::new(w, h, WorldEdgeMode::Wrap)
    }

    fn make_creature(
        id: CreatureId,
        pos: crate::contracts::Position,
        energy: f32,
        identity: CreatureIdentityState,
        phenotype: [u8; 6],
    ) -> BodyView {
        crate::creature::birth::Zygote::founder(
            v3alpha1_founder_genome(),
            HeritableBody {
                phenotype: Phenotype {
                    channels: phenotype,
                    active_channel: 0,
                    polarity: [true; 6],
                },
                identity,
                capacity: 200.0,
                steal_share: 0.2,
                give_share: 0.2,
            },
        )
        .finalize(
            id,
            crate::creature::birth::BirthPlace {
                position: pos,
                heading: crate::contracts::Direction::N,
                energy,
            },
        )
        .creature
        .sensed_body(&EnergyCapacityConfig::default())
    }

    /// The views the scheduler would collect over these creatures.
    fn views(creatures: &SlotMap<CreatureId, BodyView>) -> BodyViews {
        creatures.iter().map(|(id, view)| (id, *view)).collect()
    }

    #[test]
    fn kin_affinity_identical_tags() {
        assert!((compute_kin_affinity(42, 42) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn kin_affinity_all_bits_different() {
        assert!((compute_kin_affinity(0, u32::MAX) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn kin_affinity_one_bit_different() {
        let affinity = compute_kin_affinity(0b0000, 0b0001);
        assert!((affinity - (1.0 - 1.0 / 32.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn food_summary_empty_world() {
        let world = make_world(20, 20);
        let table = get_visibility_table(2);
        let origin = crate::contracts::Position::new(10, 10);
        let visible = compute_visible_cells(origin, &world, table);
        let config = PerceptionConfig::default();

        let food = reduce_food(
            visible.cells(),
            &world,
            &config,
            OrdinaryFoodTypeId::default(),
            2.0,
            25.0,
            (8.0f32).sqrt(),
        );
        assert!((food[food_idx::TOTAL_RATIO] - 0.0).abs() < f32::EPSILON);
        assert!((food[food_idx::MAX_VALUE] - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn food_summary_with_food() {
        let mut world = make_world(20, 20);
        let origin = crate::contracts::Position::new(10, 10);
        // Place food at origin and one neighbor
        world.set_food(origin, 0.5);
        world.set_food(crate::contracts::Position::new(11, 10), 1.0);

        let table = get_visibility_table(2);
        let visible = compute_visible_cells(origin, &world, table);
        let config = PerceptionConfig::default();

        let food = reduce_food(
            visible.cells(),
            &world,
            &config,
            OrdinaryFoodTypeId::default(),
            2.0,
            25.0,
            (8.0f32).sqrt(),
        );
        // total_ratio = (0.5 + 1.0) / 25.0 = 0.06
        assert!((food[food_idx::TOTAL_RATIO] - 0.06).abs() < 1e-5);
        assert!((food[food_idx::MAX_VALUE] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn food_summary_nearest_is_closest() {
        let mut world = make_world(20, 20);
        let origin = crate::contracts::Position::new(10, 10);
        // Food at (11, 10) = dx=1, dist=1
        // Food at (12, 10) = dx=2, dist=2
        world.set_food(crate::contracts::Position::new(11, 10), 0.5);
        world.set_food(crate::contracts::Position::new(12, 10), 0.8);

        let table = get_visibility_table(3);
        let visible = compute_visible_cells(origin, &world, table);
        let config = PerceptionConfig::default();

        let food = reduce_food(
            visible.cells(),
            &world,
            &config,
            OrdinaryFoodTypeId::default(),
            3.0,
            49.0,
            (18.0f32).sqrt(),
        );
        // Nearest food is at dx=1, dist=1/sqrt(18)
        assert!((food[food_idx::NEAREST_DX] - 1.0 / 3.0).abs() < 1e-5);
        assert!((food[food_idx::NEAREST_DY] - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn barrier_summary_with_barriers() {
        let mut world = make_world(20, 20);
        let origin = crate::contracts::Position::new(10, 10);
        // Place barrier to the east
        world.set_barrier(crate::contracts::Position::new(11, 10), true);

        let table = get_visibility_table(2);
        let visible = compute_visible_cells(origin, &world, table);

        let barrier = reduce_barrier(visible.cells(), &world, 2.0, 25.0, (8.0f32).sqrt());
        // At least 1 barrier visible
        assert!(barrier[barrier_idx::DENSITY_RATIO] > 0.0);
        // Adjacent barrier count = 1/8
        assert!((barrier[barrier_idx::BLOCKED_ADJACENT_RATIO] - 1.0 / 8.0).abs() < 1e-5);
    }

    #[test]
    fn occupancy_summary_no_creatures() {
        let world = make_world(20, 20);
        let table = get_visibility_table(2);
        let origin = crate::contracts::Position::new(10, 10);
        let visible = compute_visible_cells(origin, &world, table);

        let mut creatures: SlotMap<CreatureId, BodyView> = SlotMap::with_key();
        let observer_id = creatures.insert_with_key(|id| {
            make_creature(id, origin, 50.0, CreatureIdentityState::default(), [0; 6])
        });

        let occ = reduce_occupancy(
            visible.cells(),
            &world,
            observer_id,
            2.0,
            24.0,
            (8.0f32).sqrt(),
        );
        assert!((occ[occupancy_idx::COUNT_RATIO] - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn occupancy_summary_with_neighbors() {
        let mut world = make_world(20, 20);
        let origin = crate::contracts::Position::new(10, 10);

        let mut creatures: SlotMap<CreatureId, BodyView> = SlotMap::with_key();
        let observer_id = creatures.insert_with_key(|id| {
            make_creature(id, origin, 50.0, CreatureIdentityState::default(), [0; 6])
        });
        world.place_creature(origin, observer_id);

        let neighbor_pos = crate::contracts::Position::new(11, 10);
        let neighbor_id = creatures.insert_with_key(|id| {
            make_creature(
                id,
                neighbor_pos,
                30.0,
                CreatureIdentityState::default(),
                [0; 6],
            )
        });
        world.place_creature(neighbor_pos, neighbor_id);

        let table = get_visibility_table(2);
        let visible = compute_visible_cells(origin, &world, table);

        let occ = reduce_occupancy(
            visible.cells(),
            &world,
            observer_id,
            2.0,
            24.0,
            (8.0f32).sqrt(),
        );
        // 1 non-self creature visible
        assert!((occ[occupancy_idx::COUNT_RATIO] - 1.0 / 24.0).abs() < 1e-5);
    }

    #[test]
    fn nearby_ranking_order_by_distance() {
        let mut world = make_world(20, 20);
        let origin = crate::contracts::Position::new(10, 10);

        let mut creatures: SlotMap<CreatureId, BodyView> = SlotMap::with_key();
        let observer_id = creatures.insert_with_key(|id| {
            make_creature(id, origin, 50.0, CreatureIdentityState::default(), [0; 6])
        });
        world.place_creature(origin, observer_id);

        // Near creature at distance 1
        let near_pos = crate::contracts::Position::new(11, 10);
        let near_id = creatures.insert_with_key(|id| {
            make_creature(
                id,
                near_pos,
                30.0,
                CreatureIdentityState::default(),
                [100, 100, 100, 100, 100, 100],
            )
        });
        world.place_creature(near_pos, near_id);

        // Far creature at distance 2
        let far_pos = crate::contracts::Position::new(12, 10);
        let far_id = creatures.insert_with_key(|id| {
            make_creature(
                id,
                far_pos,
                80.0,
                CreatureIdentityState::default(),
                [200, 200, 200, 200, 200, 200],
            )
        });
        world.place_creature(far_pos, far_id);

        let table = get_visibility_table(3);
        let visible = compute_visible_cells(origin, &world, table);
        let config = PerceptionConfig::default();
        let observer = &creatures[observer_id];

        let nearby = rank_nearby_creatures(
            visible.cells(),
            &world,
            observer_id,
            observer,
            &views(&creatures),
            &config,
            3.0,
            (18.0f32).sqrt(),
        );

        // Slot 0 should be the nearer creature
        assert!((nearby[nearby_idx::PRESENT] - 1.0).abs() < f32::EPSILON);
        assert!((nearby[nearby_idx::REL_X] - 1.0 / 3.0).abs() < 1e-5);

        // Slot 1 should be the farther creature
        let slot1_base = nearby_idx::FIELDS_PER_SLOT;
        assert!((nearby[slot1_base + nearby_idx::PRESENT] - 1.0).abs() < f32::EPSILON);
        assert!((nearby[slot1_base + nearby_idx::REL_X] - 2.0 / 3.0).abs() < 1e-5);

        // Slot 2+ should be empty
        let slot2_base = 2 * nearby_idx::FIELDS_PER_SLOT;
        assert!((nearby[slot2_base + nearby_idx::PRESENT] - 0.0).abs() < f32::EPSILON);

        // Vitals slot 0: near creature energy = 30/200 = 0.15
        assert!((nearby[nearby_idx::ENERGY_RATIO] - 30.0 / 200.0).abs() < 1e-5);
    }

    #[test]
    fn nearby_ranking_truncates_to_top_slots_with_stable_tiebreaks() {
        let mut world = make_world(20, 20);
        let origin = crate::contracts::Position::new(10, 10);

        let mut creatures: SlotMap<CreatureId, BodyView> = SlotMap::with_key();
        let observer_id = creatures.insert_with_key(|id| {
            make_creature(id, origin, 50.0, CreatureIdentityState::default(), [0; 6])
        });
        world.place_creature(origin, observer_id);

        let positions = [
            crate::contracts::Position::new(10, 9),
            crate::contracts::Position::new(9, 10),
            crate::contracts::Position::new(11, 10),
            crate::contracts::Position::new(10, 11),
            crate::contracts::Position::new(12, 10),
        ];

        for position in positions {
            let creature_id = creatures.insert_with_key(|id| {
                make_creature(id, position, 30.0, CreatureIdentityState::default(), [0; 6])
            });
            world.place_creature(position, creature_id);
        }

        let table = get_visibility_table(3);
        let visible = compute_visible_cells(origin, &world, table);
        let config = PerceptionConfig::default();
        let observer = &creatures[observer_id];

        let nearby = rank_nearby_creatures(
            visible.cells(),
            &world,
            observer_id,
            observer,
            &views(&creatures),
            &config,
            3.0,
            (18.0f32).sqrt(),
        );

        let slot = |index: usize, field: usize| nearby[index * nearby_idx::FIELDS_PER_SLOT + field];

        assert!((slot(0, nearby_idx::REL_X) - 0.0).abs() < 1e-5);
        assert!((slot(0, nearby_idx::REL_Y) - (-1.0 / 3.0)).abs() < 1e-5);

        assert!((slot(1, nearby_idx::REL_X) - (-1.0 / 3.0)).abs() < 1e-5);
        assert!((slot(1, nearby_idx::REL_Y) - 0.0).abs() < 1e-5);

        assert!((slot(2, nearby_idx::REL_X) - (1.0 / 3.0)).abs() < 1e-5);
        assert!((slot(2, nearby_idx::REL_Y) - 0.0).abs() < 1e-5);

        assert!((slot(3, nearby_idx::REL_X) - 0.0).abs() < 1e-5);
        assert!((slot(3, nearby_idx::REL_Y) - (1.0 / 3.0)).abs() < 1e-5);
    }

    #[test]
    fn full_perception_assembly() {
        let mut world = make_world(20, 20);
        let origin = crate::contracts::Position::new(10, 10);
        world.set_food(origin, 0.5);

        let mut creatures: SlotMap<CreatureId, BodyView> = SlotMap::with_key();
        let observer_id = creatures.insert_with_key(|id| {
            make_creature(id, origin, 50.0, CreatureIdentityState::default(), [0; 6])
        });
        world.place_creature(origin, observer_id);

        let table = get_visibility_table(5);
        let visible = compute_visible_cells(origin, &world, table);
        let config = PerceptionConfig::default();

        let snap = assemble_perception(
            observer_id,
            visible.cells(),
            &world,
            &views(&creatures),
            &config,
        );

        // Food at origin should contribute to total_ratio
        assert!(snap.area_food[food_idx::TOTAL_RATIO] > 0.0);
        // No barriers, no neighbors
        assert!((snap.area_barrier[barrier_idx::DENSITY_RATIO] - 0.0).abs() < f32::EPSILON);
        assert!((snap.area_occupancy[occupancy_idx::COUNT_RATIO] - 0.0).abs() < f32::EPSILON);
        assert!((snap.nearby[nearby_idx::PRESENT] - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn assemble_perception_matches_reference_reducers() {
        let mut world = make_world(20, 20);
        let origin = crate::contracts::Position::new(10, 10);
        world.set_food(origin, 0.5);
        world.set_food(crate::contracts::Position::new(11, 10), 1.0);
        world.set_barrier(crate::contracts::Position::new(10, 9), true);

        let mut creatures: SlotMap<CreatureId, BodyView> = SlotMap::with_key();
        let observer_id = creatures.insert_with_key(|id| {
            make_creature(
                id,
                origin,
                50.0,
                CreatureIdentityState {
                    lineage_id: 7,
                    kin_tag: 0xAAAA_AAAA,
                },
                [10, 20, 30, 40, 50, 60],
            )
        });
        world.place_creature(origin, observer_id);

        for (position, energy, lineage_id, kin_tag, phenotype) in [
            (
                crate::contracts::Position::new(11, 10),
                30.0,
                7,
                0xAAAA_AAAA,
                [10, 20, 30, 40, 50, 60],
            ),
            (
                crate::contracts::Position::new(9, 10),
                80.0,
                9,
                0x5555_5555,
                [200, 180, 160, 140, 120, 100],
            ),
            (
                crate::contracts::Position::new(10, 11),
                120.0,
                11,
                0xAAAA_5555,
                [30, 60, 90, 120, 150, 180],
            ),
        ] {
            let creature_id = creatures.insert_with_key(|id| {
                make_creature(
                    id,
                    position,
                    energy,
                    CreatureIdentityState {
                        lineage_id,
                        kin_tag,
                    },
                    phenotype,
                )
            });
            world.place_creature(position, creature_id);
        }

        let table = get_visibility_table(5);
        let visible = compute_visible_cells(origin, &world, table);
        let config = PerceptionConfig::default();
        let observer = &creatures[observer_id];
        let r = config.vision_radius as f32;
        let max_candidate_cells = (2.0 * r + 1.0) * (2.0 * r + 1.0);
        let max_other_candidate_cells = max_candidate_cells - 1.0;
        let max_dist = (2.0 * r * r).sqrt();
        let nearby = rank_nearby_creatures(
            visible.cells(),
            &world,
            observer_id,
            observer,
            &views(&creatures),
            &config,
            r,
            max_dist,
        );

        let expected = PerceptionSnapshot {
            area_food: reduce_food(
                visible.cells(),
                &world,
                &config,
                OrdinaryFoodTypeId::default(),
                r,
                max_candidate_cells,
                max_dist,
            ),
            typed_area_food: vec![reduce_food(
                visible.cells(),
                &world,
                &config,
                OrdinaryFoodTypeId::default(),
                r,
                max_candidate_cells,
                max_dist,
            )],
            area_barrier: reduce_barrier(visible.cells(), &world, r, max_candidate_cells, max_dist),
            area_occupancy: reduce_occupancy(
                visible.cells(),
                &world,
                observer_id,
                r,
                max_other_candidate_cells,
                max_dist,
            ),
            nearby,
        };

        let actual = assemble_perception(
            observer_id,
            visible.cells(),
            &world,
            &views(&creatures),
            &config,
        );

        assert_eq!(actual, expected);
    }

    /// Slot `k`'s ten fields, in `nearby_idx` order.
    fn merged_slot(snap: &PerceptionSnapshot, k: usize) -> Vec<f32> {
        let width = nearby_idx::FIELDS_PER_SLOT;
        snap.nearby[k * width..(k + 1) * width].to_vec()
    }

    /// The layout pin (T27.F01 invariants 1 and 2): slot `k` at channels
    /// `10k..10k+10`, fields 0–6 at their T25.F05 values (the identity
    /// comparisons retired), fields 7–9 the seen creature's RGB over 255.
    /// The two creatures' colours differ in every channel, so a swapped
    /// colour field cannot pass.
    #[test]
    fn merged_nearby_creature_equals_core_vitals_identity_per_slot() {
        let mut world = make_world(20, 20);
        let origin = crate::contracts::Position::new(10, 10);
        let mut creatures: SlotMap<CreatureId, BodyView> = SlotMap::with_key();
        let observer_identity = CreatureIdentityState {
            lineage_id: 42,
            kin_tag: 0xAAAA_AAAA,
        };
        let observer_id = creatures
            .insert_with_key(|id| make_creature(id, origin, 50.0, observer_identity, [0; 6]));
        world.place_creature(origin, observer_id);
        // Slot 0: one cell east, one kin bit apart, of age; the founder's
        // colour, RGB (204, 61, 61).
        let near = crate::contracts::Position::new(11, 10);
        let near_identity = CreatureIdentityState {
            lineage_id: 42,
            kin_tag: 0xAAAA_AAAB,
        };
        let near_id = creatures.insert_with_key(|id| {
            make_creature(id, near, 30.0, near_identity, [0, 0, 92, 92, 138, 138])
        });
        creatures[near_id].age = 25;
        world.place_creature(near, near_id);
        // Slot 1: two west and one south, every kin bit apart, under age;
        // RGB (13, 89, 174).
        let far = crate::contracts::Position::new(8, 11);
        let far_identity = CreatureIdentityState {
            lineage_id: 7,
            kin_tag: 0x5555_5555,
        };
        let far_id = creatures.insert_with_key(|id| {
            make_creature(id, far, 160.0, far_identity, [150, 0, 200, 200, 60, 60])
        });
        world.place_creature(far, far_id);

        let config = PerceptionConfig::default();
        let visible = compute_visible_cells(origin, &world, get_visibility_table(5));
        let snap = assemble_perception(
            observer_id,
            visible.cells(),
            &world,
            &views(&creatures),
            &config,
        );

        let max_dist = 50.0f32.sqrt();
        let want = [
            vec![
                1.0,
                0.2,
                0.0,
                1.0 / max_dist,
                30.0 / 200.0,
                1.0,
                31.0 / 32.0,
                204.0 / 255.0,
                61.0 / 255.0,
                61.0 / 255.0,
            ],
            vec![
                1.0,
                -0.4,
                0.2,
                5.0f32.sqrt() / max_dist,
                160.0 / 200.0,
                0.0,
                0.0,
                13.0 / 255.0,
                89.0 / 255.0,
                174.0 / 255.0,
            ],
            vec![0.0; 10],
            vec![0.0; 10],
        ];
        for (k, want) in want.iter().enumerate() {
            let got = merged_slot(&snap, k);
            assert_eq!(got.len(), 10);
            for (field, (got, want)) in got.iter().zip(want).enumerate() {
                assert!(
                    (got - want).abs() < 1e-6,
                    "slot {k} field {field}: {got} vs {want}"
                );
            }
        }
    }

    /// Symmetry (T27.F01 invariant 3): what a creature reads as its own
    /// colour is what a neighbour's slot reads of it in the same tick, over
    /// the real reducer and `assemble_static_inputs`.
    #[test]
    fn own_colour_equals_what_a_neighbour_sees() {
        use crate::config::EnergyConfig;
        use crate::contracts::StaticIntrospectionKey;
        use crate::sensors::static_inputs::assemble_static_inputs;

        let mut world = make_world(20, 20);
        let mut creatures: SlotMap<CreatureId, BodyView> = SlotMap::with_key();
        let a_pos = crate::contracts::Position::new(10, 10);
        let b_pos = crate::contracts::Position::new(12, 9);
        let a_id = creatures.insert_with_key(|id| {
            make_creature(id, a_pos, 50.0, CreatureIdentityState::default(), [0; 6])
        });
        let b_id = creatures.insert_with_key(|id| {
            make_creature(
                id,
                b_pos,
                50.0,
                CreatureIdentityState::default(),
                [150, 0, 200, 200, 60, 60],
            )
        });
        world.place_creature(a_pos, a_id);
        world.place_creature(b_pos, b_id);

        let visible = compute_visible_cells(a_pos, &world, get_visibility_table(5));
        let seen = assemble_perception(
            a_id,
            visible.cells(),
            &world,
            &views(&creatures),
            &PerceptionConfig::default(),
        );
        let own = assemble_static_inputs(&world, &creatures[b_id], &EnergyConfig::default());

        let slot0 = &seen.nearby[nearby_idx::RED..=nearby_idx::BLUE];
        assert_eq!(own.own_colour.as_slice(), slot0);
        assert_eq!(own.own_colour, [13.0 / 255.0, 89.0 / 255.0, 174.0 / 255.0]);
        for (c, key) in [
            StaticIntrospectionKey::OwnRed,
            StaticIntrospectionKey::OwnGreen,
            StaticIntrospectionKey::OwnBlue,
        ]
        .iter()
        .enumerate()
        {
            assert_eq!(own.resolve_static(key).to_bits(), slot0[c].to_bits());
        }
    }

    #[test]
    fn nearby_reproduce_ready_requires_energy_and_age() {
        let mut world = make_world(20, 20);
        let origin = crate::contracts::Position::new(10, 10);

        let mut creatures: SlotMap<CreatureId, BodyView> = SlotMap::with_key();
        let observer_id = creatures.insert_with_key(|id| {
            make_creature(id, origin, 50.0, CreatureIdentityState::default(), [0; 6])
        });
        world.place_creature(origin, observer_id);

        let neighbor_pos = crate::contracts::Position::new(11, 10);
        let neighbor_id = creatures.insert_with_key(|id| {
            make_creature(
                id,
                neighbor_pos,
                100.0,
                CreatureIdentityState::default(),
                [0; 6],
            )
        });
        creatures[neighbor_id].age = 5;
        world.place_creature(neighbor_pos, neighbor_id);

        let table = get_visibility_table(2);
        let visible = compute_visible_cells(origin, &world, table);
        let config = PerceptionConfig {
            min_reproduce_energy: 30.0,
            min_reproduce_age: 20,
            ..PerceptionConfig::default()
        };

        let snap_underage = assemble_perception(
            observer_id,
            visible.cells(),
            &world,
            &views(&creatures),
            &config,
        );
        assert_eq!(
            snap_underage.nearby[nearby_idx::REPRODUCE_READY],
            0.0,
            "underage creature should not be marked reproduce_ready"
        );

        creatures[neighbor_id].age = 20;
        let snap_of_age = assemble_perception(
            observer_id,
            visible.cells(),
            &world,
            &views(&creatures),
            &config,
        );
        assert_eq!(
            snap_of_age.nearby[nearby_idx::REPRODUCE_READY],
            1.0,
            "age+energy eligible creature should be reproduce_ready"
        );
    }

    #[test]
    fn nearby_slot_reads_zero_for_a_non_positive_normalizer() {
        let origin = crate::contracts::Position::new(10, 10);
        let mut creatures: SlotMap<CreatureId, BodyView> = SlotMap::with_key();
        let observer_id = creatures.insert_with_key(|id| {
            make_creature(id, origin, 50.0, CreatureIdentityState::default(), [0; 6])
        });
        let neighbor_id = creatures.insert_with_key(|id| {
            make_creature(
                id,
                crate::contracts::Position::new(11, 11),
                40.0,
                CreatureIdentityState::default(),
                [0; 6],
            )
        });
        let candidates = [NearbyCandidate {
            creature_id: neighbor_id,
            dx: 1,
            dy: 1,
            euclidean_dist: 2.0f32.sqrt(),
            chebyshev_dist: 1,
        }];
        creatures[neighbor_id].energy_scale = 0.0;
        let config = PerceptionConfig::default();

        let nearby = fill_nearby_banks(
            &candidates,
            &creatures[observer_id],
            &views(&creatures),
            &config,
            0.0,
            0.0,
        );

        assert_eq!(nearby[nearby_idx::PRESENT], 1.0);
        for field in [
            nearby_idx::REL_X,
            nearby_idx::REL_Y,
            nearby_idx::DIST,
            nearby_idx::ENERGY_RATIO,
        ] {
            assert_eq!(nearby[field], 0.0, "field {field}");
        }
    }

    /// The nearby energy read divides by the target's viewed energy scale
    /// (T26.F02), which the creature record resolves.
    #[test]
    fn nearby_energy_ratio_divides_by_the_targets_energy_scale() {
        let origin = crate::contracts::Position::new(10, 10);
        let mut creatures: SlotMap<CreatureId, BodyView> = SlotMap::with_key();
        let observer_id = creatures.insert_with_key(|id| {
            make_creature(id, origin, 50.0, CreatureIdentityState::default(), [0; 6])
        });
        let neighbor_id = creatures.insert_with_key(|id| {
            make_creature(
                id,
                crate::contracts::Position::new(11, 11),
                40.0,
                CreatureIdentityState::default(),
                [0; 6],
            )
        });
        let candidates = [NearbyCandidate {
            creature_id: neighbor_id,
            dx: 1,
            dy: 1,
            euclidean_dist: 2.0f32.sqrt(),
            chebyshev_dist: 1,
        }];
        let config = PerceptionConfig::default();
        for (scale, expected) in [(200.0, 0.2), (50.0, 0.8), (20.0, 1.0)] {
            creatures[neighbor_id].energy_scale = scale;
            let nearby = fill_nearby_banks(
                &candidates,
                &creatures[observer_id],
                &views(&creatures),
                &config,
                2.0,
                2.0,
            );
            assert_eq!(nearby[nearby_idx::ENERGY_RATIO], expected, "{scale}");
        }
    }

    /// A visible creature without a view (not alive this tick) leaves its
    /// slot empty, as a missing creature did.
    #[test]
    fn a_visible_creature_without_a_view_is_skipped() {
        let mut world = make_world(20, 20);
        let origin = crate::contracts::Position::new(10, 10);
        let mut creatures: SlotMap<CreatureId, BodyView> = SlotMap::with_key();
        let observer_id = creatures.insert_with_key(|id| {
            make_creature(id, origin, 50.0, CreatureIdentityState::default(), [0; 6])
        });
        world.place_creature(origin, observer_id);
        let ghost = crate::contracts::Position::new(11, 10);
        let ghost_id = creatures.insert_with_key(|id| {
            make_creature(id, ghost, 50.0, CreatureIdentityState::default(), [0; 6])
        });
        world.place_creature(ghost, ghost_id);
        let mut views = views(&creatures);
        views.remove(ghost_id);

        let visible = compute_visible_cells(origin, &world, get_visibility_table(3));
        let snap = assemble_perception(
            observer_id,
            visible.cells(),
            &world,
            &views,
            &PerceptionConfig::default(),
        );
        assert_eq!(snap.nearby[nearby_idx::PRESENT], 0.0);
    }
}
