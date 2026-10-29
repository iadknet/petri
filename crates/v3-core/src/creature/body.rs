//! The body rules (T26.F02, T27.F03, T29.F02): what a body's genes cost and
//! how they are read, over the body chromosome [`BodyGenome`] its genome
//! carries, and the ranges its scalar genes are held in, resolved from
//! config once by [`body_ranges`].

use crate::config::{EnergyCapacityConfig, SensorScale, SimulationConfig};
use crate::genome::body::{BodyGenome, BodyRanges};

/// The ranges in force under `config`: the capacity in
/// `[energy.capacity.min, energy.lifecycle.max_energy]`, each share in
/// `[exchange.share_min, 1]`. Every mutation of a body and conception's clamp
/// apply them.
#[must_use]
pub fn body_ranges(config: &SimulationConfig) -> BodyRanges {
    BodyRanges {
        capacity: (
            config.energy.capacity.min,
            config.energy.lifecycle.max_energy,
        ),
        share: (config.exchange.share_min, 1.0),
    }
}

/// The default config's ranges: the fixtures' and engine tests' ranges.
#[cfg(test)]
pub(crate) fn default_body_ranges() -> BodyRanges {
    body_ranges(&SimulationConfig::default())
}

/// The move base's multiplier, `(1 − β) + β·(capacity / reference)`,
/// computed in `f64`: exactly 1.0 at `capacity = reference` for every `f32`
/// β, and positive over the normalized ranges.
#[must_use]
pub fn move_factor(body: &BodyGenome, config: &EnergyCapacityConfig) -> f32 {
    let slope = f64::from(config.move_cost_slope);
    let ratio = f64::from(body.capacity()) / f64::from(config.reference);
    ((1.0 - slope) + slope * ratio) as f32
}

/// The signed per-tick upkeep, `u·(capacity − reference)`: negative for a
/// body below the reference, `0.0` at it.
#[must_use]
pub fn upkeep(body: &BodyGenome, config: &EnergyCapacityConfig) -> f32 {
    config.upkeep_per_unit * (body.capacity() - config.reference)
}

/// The denominator of a creature's energy sensor reads: the reference under
/// [`SensorScale::Reference`], its own capacity under [`SensorScale::Own`].
#[must_use]
pub fn energy_scale(body: &BodyGenome, config: &EnergyCapacityConfig) -> f32 {
    match config.sensor_scale {
        SensorScale::Reference => config.reference,
        SensorScale::Own => body.capacity(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genome::body::Phenotype;
    use proptest::prelude::*;

    /// A body of capacity `capacity`.
    fn body(capacity: f32) -> BodyGenome {
        BodyGenome::new(Phenotype::FOUNDER, 0, capacity, 0.2, 0.2)
    }

    /// A capacity config over the normalized ranges: `reference` in
    /// `[1, 2^24]`, `min` in `[1, reference]`, β in `[0, 1]`.
    fn capacity_config() -> impl Strategy<Value = EnergyCapacityConfig> {
        (
            1.0f32..=16_777_216.0,
            0.0f32..=1.0,
            0.0f32..=1.0,
            0.0f32..=1.0,
        )
            .prop_map(
                |(reference, min_share, slope, upkeep)| EnergyCapacityConfig {
                    reference,
                    min: (reference * min_share).max(1.0),
                    founder: reference,
                    move_cost_slope: slope,
                    upkeep_per_unit: upkeep,
                    sensor_scale: SensorScale::Reference,
                },
            )
    }

    #[test]
    fn the_ranges_are_the_capacity_and_share_bounds_in_force() {
        let mut config = SimulationConfig::default();
        config.energy.capacity.min = 40.0;
        config.energy.lifecycle.max_energy = 700.0;
        config.exchange.share_min = 0.03;
        assert_eq!(
            body_ranges(&config),
            BodyRanges {
                capacity: (40.0, 700.0),
                share: (0.03, 1.0),
            }
        );
    }

    #[test]
    fn move_factor_is_positive_at_the_extreme_normalized_range() {
        let config = EnergyCapacityConfig {
            reference: 16_777_216.0,
            min: 1.0,
            move_cost_slope: 1.0,
            ..EnergyCapacityConfig::default()
        };
        assert!(move_factor(&body(1.0), &config) > 0.0);
    }

    #[test]
    fn energy_scale_reads_the_reference_or_the_own_capacity() {
        let mut config = EnergyCapacityConfig::default();
        assert_eq!(energy_scale(&body(50.0), &config), 200.0);
        config.sensor_scale = SensorScale::Own;
        assert_eq!(energy_scale(&body(50.0), &config), 50.0);
    }

    proptest! {
        /// At the reference both cost rules are neutral: the move factor is
        /// exactly 1.0 and the upkeep exactly 0.0, for every config.
        #[test]
        fn the_reference_body_pays_neutral_costs(config in capacity_config()) {
            let body = body(config.reference);
            prop_assert_eq!(move_factor(&body, &config).to_bits(), 1.0f32.to_bits());
            prop_assert_eq!(upkeep(&body, &config).to_bits(), 0.0f32.to_bits());
        }

        /// Over the normalized ranges the move factor is positive and the
        /// upkeep's sign follows the capacity's side of the reference.
        #[test]
        fn move_factor_is_positive_and_upkeep_is_signed(
            config in capacity_config(),
            share in 0.0f32..=1.0,
        ) {
            let capacity = config.min + share * (config.reference * 4.0 - config.min);
            let body = body(capacity);
            prop_assert!(move_factor(&body, &config) > 0.0);
            let upkeep = upkeep(&body, &config);
            if capacity < config.reference {
                prop_assert!(upkeep <= 0.0);
            } else {
                prop_assert!(upkeep >= 0.0);
            }
        }
    }
}
