//! Food growth pass timings (T24.F03): which grid passes `grow` records for
//! the tick-trace seam, in what order, and that they nest in time.

use super::FoodPass::{
    self, DensityTotal, DepletionRecovery, GrazingRecovery, GrowthSpread, InhibitedCount,
    InhibitionSums, RecoverySpawns, WriteBack,
};
use crate::config::{FoodConfig, FoodTypeConfig, WorldEdgeMode};
use crate::kernel::ordinary_food::{FoodGrowthSummary, FoodResource, FoodStreamKey};
use crate::kernel::Grid;

fn grow_once(width: u16, height: u16, config: FoodConfig) -> FoodGrowthSummary {
    let mut food = FoodResource::new(width, height, config, WorldEdgeMode::Wrap);
    let barriers = Grid::new(width, height, false);
    let occupancy: Grid<Option<()>> = Grid::new(width, height, None);
    food.grow(&barriers, &occupancy, 0, FoodStreamKey::from_run_seed(1))
}

fn passes(summary: &FoodGrowthSummary) -> Vec<(FoodPass, Option<u16>)> {
    summary
        .passes
        .iter()
        .map(|pass| (pass.pass, pass.food_type.map(|id| id.get())))
        .collect()
}

/// Two food types on an empty grid: the density ratio is 0, so the recovery
/// spawns run when `below_floor` sets a positive floor and not when the
/// floor is 0 (the ratio is then not below it).
fn two_types(below_floor: bool) -> FoodConfig {
    let mut config = FoodConfig {
        types: (0..2)
            .map(|index| FoodTypeConfig {
                name: format!("type-{index}"),
                recovery_spawn_rate: None,
                ..FoodTypeConfig::default()
            })
            .collect(),
        ..FoodConfig::default()
    };
    config.shared.recovery_spawn_rate = 0.1;
    config.shared.recovery_floor_ratio = if below_floor { 0.5 } else { 0.0 };
    config
}

fn expected(recovery: bool) -> Vec<(FoodPass, Option<u16>)> {
    let mut expected = vec![
        (DepletionRecovery, None),
        (GrazingRecovery, None),
        (InhibitionSums, None),
    ];
    for food_type in 0..2 {
        expected.push((DensityTotal, Some(food_type)));
        expected.push((GrowthSpread, Some(food_type)));
        if recovery {
            expected.push((RecoverySpawns, Some(food_type)));
        }
        expected.push((WriteBack, Some(food_type)));
    }
    expected.push((InhibitedCount, None));
    expected
}

#[test]
fn phase_seam_passes_run_in_order_once_per_type_with_recovery_spawns_below_the_floor() {
    let summary = grow_once(8, 6, two_types(true));
    assert_eq!(passes(&summary), expected(true));
}

#[test]
fn phase_seam_recovery_spawns_are_absent_when_the_ratio_is_not_below_the_floor() {
    let summary = grow_once(8, 6, two_types(false));
    assert_eq!(passes(&summary), expected(false));
}

#[test]
fn phase_seam_passes_nest_in_time_in_execution_order() {
    let summary = grow_once(8, 6, two_types(true));
    for pair in summary.passes.windows(2) {
        assert!(
            pair[0].started + pair[0].elapsed <= pair[1].started,
            "{:?} overlaps {:?}",
            pair[0].pass,
            pair[1].pass
        );
    }
}

#[test]
fn phase_seam_depletion_and_grazing_spans_are_present_enabled_or_not() {
    for enabled in [false, true] {
        let mut config = two_types(true);
        config.shared.occupancy_depletion.enabled = enabled;
        config.shared.grazing.enabled = enabled;
        let summary = grow_once(8, 6, config);
        assert_eq!(passes(&summary), expected(true), "enabled = {enabled}");
    }
}

#[test]
fn phase_seam_a_zero_cell_grid_records_no_pass() {
    let summary = grow_once(0, 0, two_types(true));
    assert!(summary.passes.is_empty(), "{:?}", passes(&summary));
}

/// `grow` returns after the two recovery calls when `max_density` is zero.
/// Its empty-catalog return is the same branch but cannot be reached here:
/// `OrdinaryFoodCatalog::new` substitutes one default type for an empty
/// `types` list.
#[test]
fn phase_seam_only_the_recovery_spans_without_density() {
    let mut config = two_types(true);
    config.shared.max_density = 0.0;
    let summary = grow_once(8, 6, config);
    assert_eq!(
        passes(&summary),
        [(DepletionRecovery, None), (GrazingRecovery, None)]
    );
}

#[test]
fn phase_seam_pass_names_are_the_span_names() {
    let names: Vec<&str> = [
        DepletionRecovery,
        GrazingRecovery,
        InhibitionSums,
        DensityTotal,
        GrowthSpread,
        RecoverySpawns,
        WriteBack,
        InhibitedCount,
    ]
    .iter()
    .map(|pass| pass.name())
    .collect();
    assert_eq!(
        names,
        [
            "depletion_recovery",
            "grazing_recovery",
            "inhibition_sums",
            "density_total",
            "growth_spread",
            "recovery_spawns",
            "write_back",
            "inhibited_count"
        ]
    );
}
