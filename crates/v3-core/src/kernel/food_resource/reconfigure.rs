use crate::config::FoodResourceConfig;

use super::depletion::OccupancyDepletionLayer;

pub(super) fn apply_config_transition(
    current: &mut FoodResourceConfig,
    next: FoodResourceConfig,
    occupancy_depletion: &mut OccupancyDepletionLayer,
) {
    let reset_depletion = current.occupancy_depletion.enabled != next.occupancy_depletion.enabled;
    *current = next;
    if reset_depletion {
        occupancy_depletion.reset();
    }
}
