//! The keyed per-cell random stream of food growth (T24.F02).
//!
//! Every random choice `grow` makes is a pure function of the run's food
//! stream key, the tick, the food type, a purpose tag and the cell or attempt
//! index, so a draw does not depend on which thread visits the cell or in
//! which order. The hoisted `(key, tick, type, purpose)` word costs two
//! finalizer rounds once per pass; each draw is one more round.

use crate::config::OrdinaryFoodTypeId;
use crate::contracts::splitmix64;

/// Separates the food stream from other uses of the run seed ("FOODSTRM").
const FOOD_STREAM_TAG: u64 = 0x464f_4f44_5354_524d;

/// The run's food stream key, derived from the run seed (never the world seed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FoodStreamKey(u64);

impl FoodStreamKey {
    /// `splitmix64(seed ^ FOOD_STREAM_TAG)`.
    #[must_use]
    pub fn from_run_seed(seed: u64) -> Self {
        Self(splitmix64(seed ^ FOOD_STREAM_TAG))
    }
}

/// What a draw chooses; part of the stream word so the purposes never share draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(super) enum DrawPurpose {
    /// The candidate-list entry a spreading cell sends its contribution to.
    SpreadTarget = 1,
    /// The cell a recovery-spawn attempt lands on.
    RecoveryCell = 2,
}

/// The stream of one `(key, tick, type, purpose)`.
#[derive(Debug, Clone, Copy)]
pub(super) struct FoodStream(u64);

impl FoodStream {
    /// Each input enters through a bijection or into its own bits, so two
    /// streams differing in exactly one argument have different words.
    #[must_use]
    pub(super) fn new(
        key: FoodStreamKey,
        tick: u64,
        type_idx: OrdinaryFoodTypeId,
        purpose: DrawPurpose,
    ) -> Self {
        let type_and_purpose = (u64::from(type_idx.get()) << 8) | purpose as u64;
        Self(splitmix64(splitmix64(key.0 ^ tick) ^ type_and_purpose))
    }

    /// The draw for cell or attempt `index`.
    #[inline]
    #[must_use]
    pub(super) fn draw(self, index: u64) -> u64 {
        splitmix64(self.0 ^ index)
    }

    /// The draw for `index` mapped to `0..n`.
    #[inline]
    #[must_use]
    pub(super) fn below(self, index: u64, n: usize) -> usize {
        in_range(self.draw(index), n)
    }
}

/// The reference form of a draw: `FoodStream::new(..).draw(index)`.
#[cfg(test)]
#[must_use]
pub(super) fn food_draw(
    key: FoodStreamKey,
    tick: u64,
    type_idx: OrdinaryFoodTypeId,
    purpose: DrawPurpose,
    index: u64,
) -> u64 {
    FoodStream::new(key, tick, type_idx, purpose).draw(index)
}

/// Multiply-shift of a 64-bit draw to `0..n`: no rejection loop, bucket bias
/// below `n · 2^-64`.
#[inline]
#[must_use]
pub(super) fn in_range(draw: u64, n: usize) -> usize {
    ((u128::from(draw) * n as u128) >> 64) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ty(raw: u16) -> OrdinaryFoodTypeId {
        OrdinaryFoodTypeId::new(raw)
    }

    #[test]
    fn same_arguments_give_the_same_draw() {
        let key = FoodStreamKey::from_run_seed(11);
        for index in [0, 1, 2_559_999] {
            assert_eq!(
                food_draw(key, 7, ty(1), DrawPurpose::SpreadTarget, index),
                food_draw(
                    FoodStreamKey::from_run_seed(11),
                    7,
                    ty(1),
                    DrawPurpose::SpreadTarget,
                    index
                )
            );
        }
    }

    #[test]
    fn draws_differing_in_any_one_argument_differ() {
        let key = FoodStreamKey::from_run_seed(11);
        let base = food_draw(key, 7, ty(1), DrawPurpose::SpreadTarget, 42);
        let variants = [
            food_draw(
                FoodStreamKey::from_run_seed(12),
                7,
                ty(1),
                DrawPurpose::SpreadTarget,
                42,
            ),
            food_draw(key, 8, ty(1), DrawPurpose::SpreadTarget, 42),
            food_draw(key, 7, ty(0), DrawPurpose::SpreadTarget, 42),
            food_draw(key, 7, ty(1), DrawPurpose::RecoveryCell, 42),
            food_draw(key, 7, ty(1), DrawPurpose::SpreadTarget, 43),
        ];
        for (argument, variant) in variants.into_iter().enumerate() {
            assert_ne!(variant, base, "argument {argument}");
        }
        // Small tick and type values do not alias each other.
        assert_ne!(
            food_draw(key, 1, ty(0), DrawPurpose::SpreadTarget, 0),
            food_draw(key, 0, ty(1), DrawPurpose::SpreadTarget, 0)
        );
    }

    #[test]
    fn the_run_key_depends_on_the_seed_and_the_tag() {
        assert_ne!(
            FoodStreamKey::from_run_seed(0),
            FoodStreamKey::from_run_seed(1)
        );
        assert_ne!(FoodStreamKey::from_run_seed(0).0, splitmix64(0));
    }

    #[test]
    fn in_range_maps_the_ends_of_the_draw_range_to_the_ends_of_the_target_range() {
        for n in [1, 2, 3, 4, 2_560_000] {
            assert_eq!(in_range(0, n), 0);
            assert_eq!(in_range(u64::MAX, n), n - 1);
        }
        assert_eq!(in_range(1 << 63, 4), 2);
    }
}
