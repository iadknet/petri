use crate::config::OrdinaryFoodTypeId;
use crate::contracts::{Direction, WorldAction};

/// The largest `action_type` that decodes to an action; every value above
/// it is the soft `NoOp` default, so mutation draws new pushes in
/// `0..=MAX_DECODED_ACTION_TYPE` (T13.F05).
pub(crate) const MAX_DECODED_ACTION_TYPE: u8 = 4;

/// Number of direction-bank slots: one per [`Direction::ALL`] index.
pub const DIRECTION_BANK_SLOTS: usize = Direction::ALL.len();

/// Eight direction bids, one per [`Direction::ALL`] index (T11.F21). A bank
/// exists only once something wrote it; `None` means the scalar decode
/// applies unchanged.
pub type DirectionBank = [f32; DIRECTION_BANK_SLOTS];

/// Decode a WorldAction from the raw action_type discriminant, metadata
/// buffer, and the direction bank written for this action, if any.
///
/// Per v3-vm-isa-spec.md Section 7 action encoding table.
/// Unknown action_type values decode to NoOp (soft default). `Eat`, `Pop`,
/// and `NoOp` never consult the bank.
#[must_use]
pub fn decode_world_action(
    action_type: u8,
    meta: &[f32; 8],
    bank: Option<&DirectionBank>,
) -> WorldAction {
    match action_type {
        0 => WorldAction::NoOp,
        1 => WorldAction::Eat {
            type_idx: decode_food_type_idx(meta[0]),
        },
        2 => WorldAction::Move(select_direction(meta[0], bank)),
        3 => WorldAction::Reproduce {
            direction: select_direction(meta[0], bank),
            energy_transfer: clamp_non_negative_finite(meta[1]),
        },
        4 => WorldAction::StealEnergy {
            direction: select_direction(meta[0], bank),
            amount: clamp_non_negative_finite(meta[1]),
        },
        _ => WorldAction::NoOp,
    }
}

/// The direction a movement action commits (T11.F21 decode rule): with no
/// bank, the scalar decode of `raw`; otherwise the index of the maximum
/// sanitized bid, ties going to the scalar-decoded direction when it is among
/// them and to the lowest tied index otherwise.
#[must_use]
pub fn select_direction(raw: f32, bank: Option<&DirectionBank>) -> Direction {
    let scalar = decode_direction(raw);
    let Some(bank) = bank else {
        return scalar;
    };
    let bids = bank.map(sanitize_bid);
    let max = bids.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let tied = |index: usize| bids[index] == max;
    if tied(scalar.to_index()) {
        return scalar;
    }
    let lowest = (0..DIRECTION_BANK_SLOTS)
        .find(|&index| tied(index))
        .expect("the maximum of a finite array is attained");
    Direction::ALL[lowest]
}

/// Non-finite bids compare as 0.0 so a NaN or infinite bid can neither win nor
/// poison the maximum.
#[inline]
fn sanitize_bid(bid: f32) -> f32 {
    if bid.is_finite() {
        bid
    } else {
        0.0
    }
}

#[inline]
fn decode_food_type_idx(raw: f32) -> OrdinaryFoodTypeId {
    if raw.is_finite() && raw >= 0.0 {
        OrdinaryFoodTypeId::new(raw.round().clamp(0.0, u16::MAX as f32) as u16)
    } else {
        OrdinaryFoodTypeId::default()
    }
}

/// Decode a direction from a raw f32 meta value.
/// Rounds to nearest integer, clamps to [0, 7], indexes into Direction::ALL.
#[inline]
fn decode_direction(raw: f32) -> Direction {
    // NaN rounds to itself and would cause issues; sanitize first
    let clamped = if raw.is_nan() {
        0.0
    } else {
        raw.round().clamp(0.0, 7.0)
    };
    Direction::ALL[clamped as usize]
}

/// Clamp to non-negative finite: NaN/Inf/negative → 0.0.
#[inline]
fn clamp_non_negative_finite(v: f32) -> f32 {
    if v.is_finite() && v >= 0.0 {
        v
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::Direction;
    use proptest::prelude::*;

    fn zero_meta() -> [f32; 8] {
        [0.0; 8]
    }

    fn any_bid() -> impl Strategy<Value = f32> {
        prop_oneof![
            8 => -10.0f32..10.0,
            1 => Just(f32::NAN),
            1 => Just(f32::INFINITY),
            1 => Just(f32::NEG_INFINITY),
        ]
    }

    fn any_bank() -> impl Strategy<Value = DirectionBank> {
        prop::array::uniform8(any_bid())
    }

    fn any_raw_direction() -> impl Strategy<Value = f32> {
        prop_oneof![
            8 => -3.0f32..11.0,
            1 => Just(f32::NAN),
            1 => Just(f32::INFINITY),
        ]
    }

    proptest! {
        /// No bank: the selection is exactly the scalar decode for every raw value.
        #[test]
        fn select_direction_without_a_bank_is_the_scalar_decode(raw in any_raw_direction()) {
            prop_assert_eq!(select_direction(raw, None), decode_direction(raw));
        }

        /// An all-tie bank (every bid equal) yields the scalar direction.
        #[test]
        fn select_direction_all_tie_yields_the_scalar_direction(
            raw in any_raw_direction(),
            level in -5.0f32..5.0,
        ) {
            let bank = [level; DIRECTION_BANK_SLOTS];
            prop_assert_eq!(select_direction(raw, Some(&bank)), decode_direction(raw));
        }

        /// A unique finite maximum wins regardless of the scalar.
        #[test]
        fn select_direction_unique_maximum_wins(
            raw in any_raw_direction(),
            mut bank in any_bank(),
            winner in 0usize..DIRECTION_BANK_SLOTS,
        ) {
            bank[winner] = 100.0;
            prop_assert_eq!(select_direction(raw, Some(&bank)), Direction::ALL[winner]);
        }

        /// A tie at the maximum containing the scalar direction yields the scalar;
        /// otherwise the lowest tied index.
        #[test]
        fn select_direction_tie_prefers_the_scalar_then_the_lowest_index(
            raw in any_raw_direction(),
            mut bank in any_bank(),
            tied in prop::collection::btree_set(0usize..DIRECTION_BANK_SLOTS, 1..=DIRECTION_BANK_SLOTS),
        ) {
            for index in &tied {
                bank[*index] = 100.0;
            }
            let scalar = decode_direction(raw);
            let expected = if tied.contains(&scalar.to_index()) {
                scalar
            } else {
                Direction::ALL[*tied.iter().next().unwrap()]
            };
            prop_assert_eq!(select_direction(raw, Some(&bank)), expected);
        }

        /// Non-finite bids behave exactly as 0.0 bids.
        #[test]
        fn select_direction_sanitizes_non_finite_bids(
            raw in any_raw_direction(),
            bank in any_bank(),
        ) {
            let sanitized = bank.map(|bid| if bid.is_finite() { bid } else { 0.0 });
            prop_assert_eq!(
                select_direction(raw, Some(&bank)),
                select_direction(raw, Some(&sanitized))
            );
            prop_assert!(sanitized.iter().all(|bid| bid.is_finite()));
        }

        /// One positive edge `food[d] -> bid d` on an otherwise-zero bank selects d;
        /// one negative bid alone never selects its slot and leaves the scalar in force.
        #[test]
        fn select_direction_single_bid_one_edge_property(
            raw in any_raw_direction(),
            slot in 0usize..DIRECTION_BANK_SLOTS,
            magnitude in 0.001f32..10.0,
        ) {
            let mut bank = [0.0; DIRECTION_BANK_SLOTS];
            bank[slot] = magnitude;
            prop_assert_eq!(select_direction(raw, Some(&bank)), Direction::ALL[slot]);
            bank[slot] = -magnitude;
            let scalar = decode_direction(raw);
            let expected = if scalar.to_index() == slot {
                Direction::ALL[(0..DIRECTION_BANK_SLOTS).find(|&i| i != slot).unwrap()]
            } else {
                scalar
            };
            prop_assert_eq!(select_direction(raw, Some(&bank)), expected);
        }
    }

    #[test]
    fn eat_and_noop_ignore_the_bank() {
        let bank = [0.0, 0.0, 0.0, 5.0, 0.0, 0.0, 0.0, 0.0];
        assert_eq!(
            decode_world_action(1, &zero_meta(), Some(&bank)),
            decode_world_action(1, &zero_meta(), None)
        );
        assert_eq!(
            decode_world_action(0, &zero_meta(), Some(&bank)),
            WorldAction::NoOp
        );
        assert_eq!(
            decode_world_action(9, &zero_meta(), Some(&bank)),
            WorldAction::NoOp
        );
    }

    #[test]
    fn movement_actions_consult_the_bank() {
        let bank = [0.0, 0.0, 0.0, 5.0, 0.0, 0.0, 0.0, 0.0];
        let meta = zero_meta();
        assert_eq!(
            decode_world_action(2, &meta, Some(&bank)),
            WorldAction::Move(Direction::ALL[3])
        );
        assert!(matches!(
            decode_world_action(3, &meta, Some(&bank)),
            WorldAction::Reproduce { direction, .. } if direction == Direction::ALL[3]
        ));
        assert!(matches!(
            decode_world_action(4, &meta, Some(&bank)),
            WorldAction::StealEnergy { direction, .. } if direction == Direction::ALL[3]
        ));
    }

    #[test]
    fn max_decoded_action_type_is_the_last_non_noop_discriminant() {
        let meta = [1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        assert_ne!(
            decode_world_action(MAX_DECODED_ACTION_TYPE, &meta, None),
            WorldAction::NoOp
        );
        assert_eq!(
            decode_world_action(MAX_DECODED_ACTION_TYPE + 1, &meta, None),
            WorldAction::NoOp
        );
    }

    #[test]
    fn action_type_0_is_noop() {
        assert_eq!(
            decode_world_action(0, &zero_meta(), None),
            WorldAction::NoOp
        );
    }

    #[test]
    fn action_type_1_is_eat() {
        assert_eq!(
            decode_world_action(1, &zero_meta(), None),
            WorldAction::Eat {
                type_idx: OrdinaryFoodTypeId::default(),
            }
        );
    }

    #[test]
    fn action_type_1_reads_food_type_from_meta_slot_0() {
        let mut meta = zero_meta();
        meta[0] = 4.0;
        meta[1] = 9.0;
        assert_eq!(
            decode_world_action(1, &meta, None),
            WorldAction::Eat {
                type_idx: OrdinaryFoodTypeId::new(4),
            }
        );
    }

    #[test]
    fn action_type_2_is_move_with_direction() {
        let mut meta = zero_meta();
        meta[0] = 2.0; // E = index 2
        assert_eq!(
            decode_world_action(2, &meta, None),
            WorldAction::Move(Direction::E)
        );
    }

    #[test]
    fn action_type_3_is_reproduce() {
        let mut meta = zero_meta();
        meta[0] = 4.0; // S = index 4
        meta[1] = 15.0;
        let action = decode_world_action(3, &meta, None);
        if let WorldAction::Reproduce {
            direction,
            energy_transfer,
        } = action
        {
            assert_eq!(direction, Direction::S);
            assert!((energy_transfer - 15.0).abs() < 1e-6);
        } else {
            panic!("expected Reproduce, got {action:?}");
        }
    }

    #[test]
    fn action_type_4_is_steal_energy() {
        let mut meta = zero_meta();
        meta[0] = 4.0; // S = index 4
        meta[1] = 12.0;
        let action = decode_world_action(4, &meta, None);
        if let WorldAction::StealEnergy { direction, amount } = action {
            assert_eq!(direction, Direction::S);
            assert!((amount - 12.0).abs() < 1e-6);
        } else {
            panic!("expected StealEnergy, got {action:?}");
        }
    }

    #[test]
    fn steal_energy_negative_amount_becomes_zero() {
        let mut meta = zero_meta();
        meta[1] = -5.0;
        if let WorldAction::StealEnergy { amount, .. } = decode_world_action(4, &meta, None) {
            assert_eq!(amount, 0.0);
        } else {
            panic!("expected StealEnergy");
        }
    }

    #[test]
    fn steal_energy_nan_amount_becomes_zero() {
        let mut meta = zero_meta();
        meta[1] = f32::NAN;
        if let WorldAction::StealEnergy { amount, .. } = decode_world_action(4, &meta, None) {
            assert_eq!(amount, 0.0);
        } else {
            panic!("expected StealEnergy");
        }
    }

    #[test]
    fn steal_energy_inf_amount_becomes_zero() {
        let mut meta = zero_meta();
        meta[1] = f32::INFINITY;
        if let WorldAction::StealEnergy { amount, .. } = decode_world_action(4, &meta, None) {
            assert_eq!(amount, 0.0);
        } else {
            panic!("expected StealEnergy");
        }
    }

    #[test]
    fn unknown_action_type_is_noop() {
        assert_eq!(
            decode_world_action(5, &zero_meta(), None),
            WorldAction::NoOp
        );
        assert_eq!(
            decode_world_action(255, &zero_meta(), None),
            WorldAction::NoOp
        );
    }

    #[test]
    fn direction_decode_all_8_valid_indices() {
        for (expected_idx, expected_dir) in Direction::ALL.iter().enumerate() {
            let mut meta = zero_meta();
            meta[0] = expected_idx as f32;
            if let WorldAction::Move(dir) = decode_world_action(2, &meta, None) {
                assert_eq!(&dir, expected_dir, "index {expected_idx}");
            } else {
                panic!("expected Move");
            }
        }
    }

    #[test]
    fn direction_decode_rounds_half_up() {
        let mut meta = zero_meta();
        meta[0] = 0.6; // rounds to 1 = NE
        if let WorldAction::Move(dir) = decode_world_action(2, &meta, None) {
            assert_eq!(dir, Direction::NE);
        } else {
            panic!("expected Move");
        }
    }

    #[test]
    fn direction_decode_clamps_above_7() {
        let mut meta = zero_meta();
        meta[0] = 10.0; // clamped to 7 = NW
        if let WorldAction::Move(dir) = decode_world_action(2, &meta, None) {
            assert_eq!(dir, Direction::NW);
        } else {
            panic!("expected Move");
        }
    }

    #[test]
    fn direction_decode_clamps_negative_to_zero() {
        let mut meta = zero_meta();
        meta[0] = -3.0; // clamped to 0 = N
        if let WorldAction::Move(dir) = decode_world_action(2, &meta, None) {
            assert_eq!(dir, Direction::N);
        } else {
            panic!("expected Move");
        }
    }

    #[test]
    fn direction_decode_nan_becomes_zero() {
        let mut meta = zero_meta();
        meta[0] = f32::NAN;
        if let WorldAction::Move(dir) = decode_world_action(2, &meta, None) {
            assert_eq!(dir, Direction::N); // index 0
        } else {
            panic!("expected Move");
        }
    }

    #[test]
    fn reproduce_energy_negative_becomes_zero() {
        let mut meta = zero_meta();
        meta[1] = -5.0;
        if let WorldAction::Reproduce {
            energy_transfer, ..
        } = decode_world_action(3, &meta, None)
        {
            assert_eq!(energy_transfer, 0.0);
        } else {
            panic!("expected Reproduce");
        }
    }

    #[test]
    fn reproduce_energy_nan_becomes_zero() {
        let mut meta = zero_meta();
        meta[1] = f32::NAN;
        if let WorldAction::Reproduce {
            energy_transfer, ..
        } = decode_world_action(3, &meta, None)
        {
            assert_eq!(energy_transfer, 0.0);
        } else {
            panic!("expected Reproduce");
        }
    }

    #[test]
    fn reproduce_energy_inf_becomes_zero() {
        let mut meta = zero_meta();
        meta[1] = f32::INFINITY;
        if let WorldAction::Reproduce {
            energy_transfer, ..
        } = decode_world_action(3, &meta, None)
        {
            assert_eq!(energy_transfer, 0.0);
        } else {
            panic!("expected Reproduce");
        }
    }
}
