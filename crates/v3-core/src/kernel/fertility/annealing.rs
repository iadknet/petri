use crate::config::AnnealingConfig;

/// Compute the effective fertility range at a given tick.
///
/// When annealing is disabled or `ramp_ticks == 0`, returns `(target_min, target_max)`
/// immediately. Otherwise, linearly interpolates from the initial range to the
/// target range over `ramp_ticks`.
pub fn effective_fertility_range(
    annealing: &AnnealingConfig,
    target_min: f32,
    target_max: f32,
    tick: u64,
) -> (f32, f32) {
    if !annealing.enabled || annealing.ramp_ticks == 0 {
        return (target_min, target_max);
    }

    let progress = (tick as f64 / annealing.ramp_ticks as f64).clamp(0.0, 1.0) as f32;
    let effective_min = lerp(annealing.initial_min_fertility, target_min, progress);
    let effective_max = lerp(annealing.initial_max_fertility, target_max, progress);
    (effective_min, effective_max)
}

/// Map a raw value in [-1, 1] to the effective [min, max] range.
#[inline]
pub fn map_fertility(raw: f32, effective_min: f32, effective_max: f32) -> f32 {
    let t = (raw + 1.0) / 2.0; // [-1, 1] -> [0, 1]
    effective_min + t * (effective_max - effective_min)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn annealing_enabled() -> AnnealingConfig {
        AnnealingConfig {
            enabled: true,
            ramp_ticks: 1000,
            initial_min_fertility: 0.3,
            initial_max_fertility: 1.5,
        }
    }

    fn annealing_disabled() -> AnnealingConfig {
        AnnealingConfig {
            enabled: false,
            ..annealing_enabled()
        }
    }

    #[test]
    fn tick_zero_uses_initial_range() {
        let cfg = annealing_enabled();
        let (lo, hi) = effective_fertility_range(&cfg, 0.0, 2.0, 0);
        assert!((lo - 0.3).abs() < 1e-5, "expected 0.3, got {lo}");
        assert!((hi - 1.5).abs() < 1e-5, "expected 1.5, got {hi}");
    }

    #[test]
    fn midpoint_interpolates_correctly() {
        let cfg = annealing_enabled();
        let (lo, hi) = effective_fertility_range(&cfg, 0.0, 2.0, 500);
        // At 50%: lo = 0.3 + 0.5 * (0.0 - 0.3) = 0.15
        // At 50%: hi = 1.5 + 0.5 * (2.0 - 1.5) = 1.75
        assert!((lo - 0.15).abs() < 1e-5, "expected 0.15, got {lo}");
        assert!((hi - 1.75).abs() < 1e-5, "expected 1.75, got {hi}");
    }

    #[test]
    fn at_ramp_ticks_uses_target_range() {
        let cfg = annealing_enabled();
        let (lo, hi) = effective_fertility_range(&cfg, 0.0, 2.0, 1000);
        assert!((lo - 0.0).abs() < 1e-5, "expected 0.0, got {lo}");
        assert!((hi - 2.0).abs() < 1e-5, "expected 2.0, got {hi}");
    }

    #[test]
    fn beyond_ramp_ticks_uses_target_range() {
        let cfg = annealing_enabled();
        let (lo, hi) = effective_fertility_range(&cfg, 0.0, 2.0, 5000);
        assert!((lo - 0.0).abs() < 1e-5, "expected 0.0, got {lo}");
        assert!((hi - 2.0).abs() < 1e-5, "expected 2.0, got {hi}");
    }

    #[test]
    fn disabled_uses_target_range_at_all_ticks() {
        let cfg = annealing_disabled();
        for tick in [0, 100, 500, 1000, 5000] {
            let (lo, hi) = effective_fertility_range(&cfg, 0.0, 2.0, tick);
            assert!(
                (lo - 0.0).abs() < 1e-5,
                "tick {tick}: expected min 0.0, got {lo}"
            );
            assert!(
                (hi - 2.0).abs() < 1e-5,
                "tick {tick}: expected max 2.0, got {hi}"
            );
        }
    }

    #[test]
    fn ramp_ticks_zero_uses_target_range() {
        let cfg = AnnealingConfig {
            enabled: true,
            ramp_ticks: 0,
            initial_min_fertility: 0.3,
            initial_max_fertility: 1.5,
        };
        let (lo, hi) = effective_fertility_range(&cfg, 0.0, 2.0, 0);
        assert!((lo - 0.0).abs() < 1e-5, "expected 0.0, got {lo}");
        assert!((hi - 2.0).abs() < 1e-5, "expected 2.0, got {hi}");
    }

    #[test]
    fn map_fertility_raw_neg_one_maps_to_min() {
        let val = map_fertility(-1.0, 0.0, 2.0);
        assert!((val - 0.0).abs() < 1e-5, "expected 0.0, got {val}");
    }

    #[test]
    fn map_fertility_raw_zero_maps_to_midpoint() {
        let val = map_fertility(0.0, 0.0, 2.0);
        assert!((val - 1.0).abs() < 1e-5, "expected 1.0, got {val}");
    }

    #[test]
    fn map_fertility_raw_one_maps_to_max() {
        let val = map_fertility(1.0, 0.0, 2.0);
        assert!((val - 2.0).abs() < 1e-5, "expected 2.0, got {val}");
    }
}
