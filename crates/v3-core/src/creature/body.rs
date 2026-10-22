//! The heritable body: what a creature inherits beside its brain (T26.F01,
//! T26.F02).
//!
//! [`HeritableBody`] carries the six-channel colour [`Phenotype`], the kin
//! [`CreatureIdentityState`], the energy `capacity` and the two exchange
//! shares (T27.F03). Seeding builds it with
//! [`HeritableBody::founder`], reproduction with [`HeritableBody::inherit`],
//! and the creature constructors take it as one argument. The record owns the
//! order of the inheritance draws and the capacity's body rules
//! ([`HeritableBody::move_factor`], [`HeritableBody::upkeep`],
//! [`HeritableBody::energy_scale`]); `mutation::phenotype` and
//! `creature::identity` own how the other two parts mutate.

use rand::Rng;

use crate::config::{
    EnergyCapacityConfig, EnergyConfig, ExchangeConfig, LogStepMutationConfig, MutationConfig,
    SensorScale,
};
use crate::creature::identity::CreatureIdentityState;
use crate::creature::phenotype::Phenotype;
use crate::mutation::phenotype::mutate_phenotype;

/// Everything a creature inherits beside its genome and shared memory.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeritableBody {
    pub phenotype: Phenotype,
    pub identity: CreatureIdentityState,
    /// The most energy this creature can store, in energy units; in
    /// `[energy.capacity.min, energy.lifecycle.max_energy]` once inherited.
    pub capacity: f32,
    /// The share of a victim's store a `StealEnergy` at modifier 1 takes, in
    /// `[exchange.share_min, 1]` once inherited (T27.F03).
    pub steal_share: f32,
    /// The share of its own store a `GiveEnergy` at modifier 1 passes, in
    /// `[exchange.share_min, 1]` once inherited (T27.F03).
    pub give_share: f32,
}

impl HeritableBody {
    /// The body of founder `founder_index` of a run seeded with
    /// `startup_seed`: [`Phenotype::FOUNDER`],
    /// [`CreatureIdentityState::founder`], `capacity`
    /// (`energy.capacity.founder`) and both shares at `share`
    /// (`exchange.founder_share`).
    #[must_use]
    pub fn founder(founder_index: usize, startup_seed: u64, capacity: f32, share: f32) -> Self {
        Self {
            phenotype: Phenotype::FOUNDER,
            identity: CreatureIdentityState::founder(founder_index, startup_seed),
            capacity,
            steal_share: share,
            give_share: share,
        }
    }

    /// The child's body: a copy of `parent`'s with the capacity clamped to
    /// the range in force at this birth, `[capacity.min,
    /// lifecycle.max_energy]`, so a live patch of the bounds reaches the next
    /// generation without a draw; when the birth `mutated` (at least one
    /// genome mutation event applied), a phenotype mutation followed by a
    /// kin-tag mutation; then, on every birth, the capacity, `steal_share`
    /// and `give_share` in that order by [`inherit_scalar`], the capacity
    /// under `mutation.capacity` over `[capacity.min, lifecycle.max_energy]`
    /// and the shares under `mutation.share` over `[exchange.share_min, 1]`.
    #[must_use]
    pub fn inherit(
        parent: &Self,
        mutated: bool,
        mutation: &MutationConfig,
        energy: &EnergyConfig,
        exchange: &ExchangeConfig,
        rng: &mut impl Rng,
    ) -> Self {
        let mut child = Self {
            identity: CreatureIdentityState::inherit(&parent.identity),
            ..*parent
        };
        if mutated {
            child.phenotype = mutate_phenotype(child.phenotype, &mutation.phenotype, rng);
            child.identity.mutate_kin_tag(rng);
        }
        let (min, max) = (energy.capacity.min, energy.lifecycle.max_energy);
        child.capacity = inherit_scalar(parent.capacity, &mutation.capacity, min, max, rng);
        let share_min = exchange.share_min;
        child.steal_share =
            inherit_scalar(parent.steal_share, &mutation.share, share_min, 1.0, rng);
        child.give_share = inherit_scalar(parent.give_share, &mutation.share, share_min, 1.0, rng);
        child
    }

    /// The move base's multiplier, `(1 − β) + β·(capacity / reference)`,
    /// computed in `f64`: exactly 1.0 at `capacity = reference` for every
    /// `f32` β, and positive over the normalized ranges.
    #[must_use]
    pub fn move_factor(&self, config: &EnergyCapacityConfig) -> f32 {
        let slope = f64::from(config.move_cost_slope);
        let ratio = f64::from(self.capacity) / f64::from(config.reference);
        ((1.0 - slope) + slope * ratio) as f32
    }

    /// The signed per-tick upkeep, `u·(capacity − reference)`: negative for a
    /// body below the reference, `0.0` at it.
    #[must_use]
    pub fn upkeep(&self, config: &EnergyCapacityConfig) -> f32 {
        config.upkeep_per_unit * (self.capacity - config.reference)
    }

    /// The denominator of this creature's energy sensor reads: the reference
    /// under [`SensorScale::Reference`], its own capacity under
    /// [`SensorScale::Own`].
    #[must_use]
    pub fn energy_scale(&self, config: &EnergyCapacityConfig) -> f32 {
        match config.sensor_scale {
            SensorScale::Reference => config.reference,
            SensorScale::Own => self.capacity,
        }
    }
}

/// One heritable scalar trait's inheritance (T26.F02, T27.F03): the parent's
/// value clamped to `[min, max]`, the range in force at this birth, so a
/// live patch of the bounds reaches the next generation without a draw;
/// then one `f32` draw against `config.chance` and, when it fires, a uniform
/// log-space step `δ ∈ [-log_step, log_step]` with the child
/// `(parent × e^δ)` clamped to the same range.
fn inherit_scalar(
    parent: f32,
    config: &LogStepMutationConfig,
    min: f32,
    max: f32,
    rng: &mut impl Rng,
) -> f32 {
    if rng.gen::<f32>() < config.chance {
        let step = config.log_step;
        let delta = rng.gen_range(-step..=step);
        (parent * delta.exp()).clamp(min, max)
    } else {
        parent.clamp(min, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{LogStepMutationConfig, PhenotypeConfig};
    use proptest::prelude::*;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    /// A mutation config whose capacity and share mutations both draw at
    /// `chance` with step `log_step`.
    fn mutation(chance: f32, log_step: f32) -> MutationConfig {
        MutationConfig {
            phenotype: PhenotypeConfig {
                channel_step: 3,
                channel_change_chance: 0.5,
                polarity_flip_chance: 0.5,
            },
            capacity: LogStepMutationConfig { chance, log_step },
            share: LogStepMutationConfig { chance, log_step },
            ..MutationConfig::default()
        }
    }

    const EXCHANGE: ExchangeConfig = ExchangeConfig {
        effort_rate: 0.05,
        share_min: 0.01,
        founder_share: 0.2,
    };

    fn phenotype() -> impl Strategy<Value = Phenotype> {
        (any::<[u8; 6]>(), 0usize..6, any::<[bool; 6]>()).prop_map(
            |(channels, active_channel, polarity)| Phenotype {
                channels,
                active_channel,
                polarity,
            },
        )
    }

    fn body() -> impl Strategy<Value = HeritableBody> {
        (
            phenotype(),
            any::<u32>(),
            any::<u32>(),
            50.0f32..=800.0,
            0.01f32..=1.0,
            0.01f32..=1.0,
        )
            .prop_map(
                |(phenotype, lineage_id, kin_tag, capacity, steal_share, give_share)| {
                    HeritableBody {
                        phenotype,
                        identity: CreatureIdentityState {
                            lineage_id,
                            kin_tag,
                        },
                        capacity,
                        steal_share,
                        give_share,
                    }
                },
            )
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
    fn founder_pairs_the_founder_phenotype_identity_capacity_and_shares() {
        let body = HeritableBody::founder(4, 99, 321.0, 0.2);
        assert_eq!(body.phenotype, Phenotype::FOUNDER);
        assert_eq!(body.identity, CreatureIdentityState::founder(4, 99));
        assert_eq!(body.capacity, 321.0);
        assert_eq!((body.steal_share, body.give_share), (0.2, 0.2));
    }

    #[test]
    fn move_factor_is_positive_at_the_extreme_normalized_range() {
        let config = EnergyCapacityConfig {
            reference: 16_777_216.0,
            min: 1.0,
            move_cost_slope: 1.0,
            ..EnergyCapacityConfig::default()
        };
        let body = HeritableBody::founder(0, 0, 1.0, 0.2);
        assert!(body.move_factor(&config) > 0.0);
    }

    #[test]
    fn energy_scale_reads_the_reference_or_the_own_capacity() {
        let body = HeritableBody::founder(0, 0, 50.0, 0.2);
        let mut config = EnergyCapacityConfig::default();
        assert_eq!(body.energy_scale(&config), 200.0);
        config.sensor_scale = SensorScale::Own;
        assert_eq!(body.energy_scale(&config), 50.0);
    }

    /// An RNG whose every word is zero, so `gen::<f32>()` draws exactly 0.0.
    struct ZeroRng;

    impl rand::RngCore for ZeroRng {
        fn next_u32(&mut self) -> u32 {
            0
        }
        fn next_u64(&mut self) -> u64 {
            0
        }
        fn fill_bytes(&mut self, dest: &mut [u8]) {
            dest.fill(0);
        }
        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
            dest.fill(0);
            Ok(())
        }
    }

    /// A zero capacity chance never fires, even on the smallest draw (0.0),
    /// and a chance just above it does.
    #[test]
    fn a_zero_capacity_chance_never_fires_on_a_zero_draw() {
        let parent = HeritableBody::founder(0, 0, 200.0, 0.2);
        let energy = EnergyConfig::default();
        let child = HeritableBody::inherit(
            &parent,
            false,
            &mutation(0.0, 0.25),
            &energy,
            &EXCHANGE,
            &mut ZeroRng,
        );
        assert_eq!(child, parent);
        let fired = HeritableBody::inherit(
            &parent,
            false,
            &mutation(f32::MIN_POSITIVE, 0.25),
            &energy,
            &EXCHANGE,
            &mut ZeroRng,
        );
        assert_eq!(fired.capacity, 200.0 * (-0.25f32).exp());
        assert_eq!(fired.steal_share, 0.2 * (-0.25f32).exp());
        assert_eq!(fired.give_share, 0.2 * (-0.25f32).exp());
    }

    /// The shares clamp at both bounds of the range in force: a step below
    /// `share_min` lands on it, one above 1 lands on 1, and at `share_min`
    /// = 1 both shares pin at 1 with a zero step.
    #[test]
    fn a_child_share_clamps_at_share_min_at_one_and_pins_at_a_unit_range() {
        let energy = EnergyConfig::default();
        let mut parent = HeritableBody::founder(0, 0, 200.0, 0.011);
        parent.give_share = 0.99;
        // ZeroRng steps by -log_step: 0.011 × e^-1 is below 0.01.
        let low = HeritableBody::inherit(
            &parent,
            false,
            &mutation(1.0, 1.0),
            &energy,
            &EXCHANGE,
            &mut ZeroRng,
        );
        assert_eq!(low.steal_share, 0.01);
        // A step up from 0.99 by up to e^1 clamps at 1 for any draw above
        // ln(1 / 0.99); a parent above the range clamps without a draw.
        parent.give_share = 1.0;
        let mut rng = SmallRng::seed_from_u64(3);
        let mut hit_one = false;
        for _ in 0..64 {
            let child = HeritableBody::inherit(
                &parent,
                false,
                &mutation(1.0, 1.0),
                &energy,
                &EXCHANGE,
                &mut rng,
            );
            assert!((0.01..=1.0).contains(&child.give_share));
            hit_one |= child.give_share == 1.0;
        }
        assert!(hit_one, "a step up from 1 clamps at 1");
        let unit = ExchangeConfig {
            share_min: 1.0,
            ..EXCHANGE
        };
        let child = HeritableBody::inherit(
            &parent,
            false,
            &mutation(1.0, 0.0),
            &energy,
            &unit,
            &mut rng,
        );
        assert_eq!((child.steal_share, child.give_share), (1.0, 1.0));
    }

    /// A `log_step` at its normalized bound, `ln(max_energy / min)` over the
    /// widest range, samples without panic.
    #[test]
    fn a_log_step_at_its_bound_samples() {
        let mut cfg = crate::config::SimulationConfig::default();
        cfg.energy.lifecycle.max_energy = f32::MAX;
        cfg.energy.capacity.min = 1.0;
        let bound = f32::MAX.ln();
        cfg.mutation.capacity.chance = 1.0;
        cfg.mutation.capacity.log_step = bound;
        cfg.normalize();
        assert_eq!(cfg.mutation.capacity.log_step, bound);
        let parent = HeritableBody::founder(0, 0, 200.0, 0.2);
        let mut rng = SmallRng::seed_from_u64(7);
        for _ in 0..64 {
            let child = HeritableBody::inherit(
                &parent,
                false,
                &cfg.mutation,
                &cfg.energy,
                &cfg.exchange,
                &mut rng,
            );
            assert!((1.0..=f32::MAX).contains(&child.capacity));
        }
    }

    proptest! {
        /// An inheritance draws, in order: the phenotype mutation and the
        /// kin-tag mutation when `mutated`; then the capacity, `steal_share`
        /// and `give_share` chances on every birth, each followed by its
        /// log-space step only when it fires. It leaves the stream where
        /// those calls leave it.
        #[test]
        fn an_inheritance_draws_phenotype_kin_tag_capacity_then_shares(
            parent in body(),
            mutated in any::<bool>(),
            chance in 0.0f32..=1.0,
            seed in any::<u64>(),
        ) {
            let mutation = mutation(chance, 0.25);
            let energy = EnergyConfig::default();
            let mut rng = SmallRng::seed_from_u64(seed);
            let child =
                HeritableBody::inherit(&parent, mutated, &mutation, &energy, &EXCHANGE, &mut rng);

            let mut expected_rng = SmallRng::seed_from_u64(seed);
            let mut expected = HeritableBody {
                identity: CreatureIdentityState::inherit(&parent.identity),
                ..parent
            };
            if mutated {
                expected.phenotype =
                    mutate_phenotype(parent.phenotype, &mutation.phenotype, &mut expected_rng);
                expected.identity.mutate_kin_tag(&mut expected_rng);
            }
            let mut draw = |value: f32, min: f32, max: f32| {
                if expected_rng.gen::<f32>() < chance {
                    let delta: f32 = expected_rng.gen_range(-0.25f32..=0.25);
                    (value * delta.exp()).clamp(min, max)
                } else {
                    value.clamp(min, max)
                }
            };
            expected.capacity = draw(parent.capacity, 50.0, 800.0);
            expected.steal_share = draw(parent.steal_share, 0.01, 1.0);
            expected.give_share = draw(parent.give_share, 0.01, 1.0);

            prop_assert_eq!(child, expected);
            prop_assert_eq!(rng.gen::<u64>(), expected_rng.gen::<u64>());
        }

        /// An unmutated birth whose three chances miss copies the parent
        /// and draws only the three chances.
        #[test]
        fn an_unmutated_missed_inheritance_copies_the_parent(parent in body(), seed in any::<u64>()) {
            let mut rng = SmallRng::seed_from_u64(seed);
            let child = HeritableBody::inherit(
                &parent, false, &mutation(0.0, 0.25), &EnergyConfig::default(), &EXCHANGE, &mut rng,
            );
            prop_assert_eq!(child, parent);
            let mut expected_rng = SmallRng::seed_from_u64(seed);
            let _: [f32; 3] = expected_rng.gen();
            prop_assert_eq!(rng.gen::<u64>(), expected_rng.gen::<u64>());
        }

        /// An unmutated newborn's capacity is its parent's clamped to the
        /// range in force at its birth, whatever range the parent was born in.
        #[test]
        fn an_unmutated_newborn_is_clamped_to_the_range_at_its_birth(
            parent in body(),
            min in 1.0f32..=400.0,
            span in 0.0f32..=800.0,
            seed in any::<u64>(),
        ) {
            let mut energy = EnergyConfig::default();
            energy.capacity.min = min;
            energy.lifecycle.max_energy = min + span;
            let mut rng = SmallRng::seed_from_u64(seed);
            let child = HeritableBody::inherit(
                &parent, false, &mutation(0.0, 0.25), &energy, &EXCHANGE, &mut rng,
            );
            prop_assert_eq!(child.capacity, parent.capacity.clamp(min, min + span));
        }

        /// A child's shares stay in `[share_min, 1]` for any parent shares
        /// (in range or not), any share range and any share mutation.
        #[test]
        fn a_child_share_stays_in_range_for_any_parent_and_config(
            parent in body(),
            steal_share in proptest::num::f32::POSITIVE | proptest::num::f32::ZERO,
            give_share in proptest::num::f32::POSITIVE | proptest::num::f32::ZERO,
            share_min in 1.0e-6f32..=1.0,
            chance in 0.0f32..=1.0,
            log_step in 0.0f32..=14.0,
            seed in any::<u64>(),
        ) {
            let parent = HeritableBody { steal_share, give_share, ..parent };
            let exchange = ExchangeConfig { share_min, ..EXCHANGE };
            let log_step = log_step.min((1.0 / share_min).ln());
            let mut rng = SmallRng::seed_from_u64(seed);
            let child = HeritableBody::inherit(
                &parent, false, &mutation(chance, log_step), &EnergyConfig::default(), &exchange, &mut rng,
            );
            prop_assert!((share_min..=1.0).contains(&child.steal_share));
            prop_assert!((share_min..=1.0).contains(&child.give_share));
        }

        /// A mutated inheritance keeps the lineage and flips one kin-tag bit.
        #[test]
        fn a_mutated_inheritance_keeps_lineage_and_flips_one_kin_bit(parent in body(), seed in any::<u64>()) {
            let mut rng = SmallRng::seed_from_u64(seed);
            let child = HeritableBody::inherit(
                &parent, true, &mutation(0.1, 0.25), &EnergyConfig::default(), &EXCHANGE, &mut rng,
            );
            prop_assert_eq!(child.identity.lineage_id, parent.identity.lineage_id);
            prop_assert_eq!((child.identity.kin_tag ^ parent.identity.kin_tag).count_ones(), 1);
        }

        /// A fired capacity mutation lands in `[min, max_energy]` and within
        /// one log step of the parent's capacity, before the clamp.
        #[test]
        fn an_inherited_capacity_stays_in_its_bounds(
            parent in body(),
            log_step in 0.0f32..=4.0,
            seed in any::<u64>(),
        ) {
            let energy = EnergyConfig::default();
            let mut rng = SmallRng::seed_from_u64(seed);
            let child = HeritableBody::inherit(
                &parent, false, &mutation(1.0, log_step), &energy, &EXCHANGE, &mut rng,
            );
            prop_assert!(child.capacity >= energy.capacity.min);
            prop_assert!(child.capacity <= energy.lifecycle.max_energy);
            let lo = (parent.capacity * (-log_step).exp()).max(energy.capacity.min);
            let hi = (parent.capacity * log_step.exp()).min(energy.lifecycle.max_energy);
            prop_assert!(child.capacity >= lo * (1.0 - 1e-6) && child.capacity <= hi * (1.0 + 1e-6));
        }

        /// At the reference both cost rules are neutral: the move factor is
        /// exactly 1.0 and the upkeep exactly 0.0, for every config.
        #[test]
        fn the_reference_body_pays_neutral_costs(config in capacity_config()) {
            let body = HeritableBody::founder(0, 0, config.reference, 0.2);
            prop_assert_eq!(body.move_factor(&config).to_bits(), 1.0f32.to_bits());
            prop_assert_eq!(body.upkeep(&config).to_bits(), 0.0f32.to_bits());
        }

        /// Over the normalized ranges the move factor is positive and the
        /// upkeep's sign follows the capacity's side of the reference.
        #[test]
        fn move_factor_is_positive_and_upkeep_is_signed(
            config in capacity_config(),
            share in 0.0f32..=1.0,
        ) {
            let capacity = config.min + share * (config.reference * 4.0 - config.min);
            let body = HeritableBody::founder(0, 0, capacity, 0.2);
            prop_assert!(body.move_factor(&config) > 0.0);
            let upkeep = body.upkeep(&config);
            if capacity < config.reference {
                prop_assert!(upkeep <= 0.0);
            } else {
                prop_assert!(upkeep >= 0.0);
            }
        }
    }
}
