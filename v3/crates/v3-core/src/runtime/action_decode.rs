use crate::config::OrdinaryFoodTypeId;
use crate::contracts::{Direction, WorldAction};

/// Decode a WorldAction from the raw action_type discriminant and metadata buffer.
///
/// Per v3-vm-isa-spec.md Section 7 action encoding table.
/// Unknown action_type values decode to NoOp (soft default).
#[must_use]
pub fn decode_world_action(action_type: u8, meta: &[f32; 8]) -> WorldAction {
    match action_type {
        0 => WorldAction::NoOp,
        1 => WorldAction::Eat {
            type_idx: decode_food_type_idx(meta[0]),
        },
        2 => WorldAction::Move(decode_direction(meta[0])),
        3 => WorldAction::Reproduce {
            direction: decode_direction(meta[0]),
            energy_transfer: clamp_non_negative_finite(meta[1]),
        },
        4 => WorldAction::StealEnergy {
            direction: decode_direction(meta[0]),
            amount: clamp_non_negative_finite(meta[1]),
        },
        _ => WorldAction::NoOp,
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

    fn zero_meta() -> [f32; 8] {
        [0.0; 8]
    }

    #[test]
    fn action_type_0_is_noop() {
        assert_eq!(decode_world_action(0, &zero_meta()), WorldAction::NoOp);
    }

    #[test]
    fn action_type_1_is_eat() {
        assert_eq!(
            decode_world_action(1, &zero_meta()),
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
            decode_world_action(1, &meta),
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
            decode_world_action(2, &meta),
            WorldAction::Move(Direction::E)
        );
    }

    #[test]
    fn action_type_3_is_reproduce() {
        let mut meta = zero_meta();
        meta[0] = 4.0; // S = index 4
        meta[1] = 15.0;
        let action = decode_world_action(3, &meta);
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
        let action = decode_world_action(4, &meta);
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
        if let WorldAction::StealEnergy { amount, .. } = decode_world_action(4, &meta) {
            assert_eq!(amount, 0.0);
        } else {
            panic!("expected StealEnergy");
        }
    }

    #[test]
    fn steal_energy_nan_amount_becomes_zero() {
        let mut meta = zero_meta();
        meta[1] = f32::NAN;
        if let WorldAction::StealEnergy { amount, .. } = decode_world_action(4, &meta) {
            assert_eq!(amount, 0.0);
        } else {
            panic!("expected StealEnergy");
        }
    }

    #[test]
    fn steal_energy_inf_amount_becomes_zero() {
        let mut meta = zero_meta();
        meta[1] = f32::INFINITY;
        if let WorldAction::StealEnergy { amount, .. } = decode_world_action(4, &meta) {
            assert_eq!(amount, 0.0);
        } else {
            panic!("expected StealEnergy");
        }
    }

    #[test]
    fn unknown_action_type_is_noop() {
        assert_eq!(decode_world_action(5, &zero_meta()), WorldAction::NoOp);
        assert_eq!(decode_world_action(255, &zero_meta()), WorldAction::NoOp);
    }

    #[test]
    fn direction_decode_all_8_valid_indices() {
        for (expected_idx, expected_dir) in Direction::ALL.iter().enumerate() {
            let mut meta = zero_meta();
            meta[0] = expected_idx as f32;
            if let WorldAction::Move(dir) = decode_world_action(2, &meta) {
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
        if let WorldAction::Move(dir) = decode_world_action(2, &meta) {
            assert_eq!(dir, Direction::NE);
        } else {
            panic!("expected Move");
        }
    }

    #[test]
    fn direction_decode_clamps_above_7() {
        let mut meta = zero_meta();
        meta[0] = 10.0; // clamped to 7 = NW
        if let WorldAction::Move(dir) = decode_world_action(2, &meta) {
            assert_eq!(dir, Direction::NW);
        } else {
            panic!("expected Move");
        }
    }

    #[test]
    fn direction_decode_clamps_negative_to_zero() {
        let mut meta = zero_meta();
        meta[0] = -3.0; // clamped to 0 = N
        if let WorldAction::Move(dir) = decode_world_action(2, &meta) {
            assert_eq!(dir, Direction::N);
        } else {
            panic!("expected Move");
        }
    }

    #[test]
    fn direction_decode_nan_becomes_zero() {
        let mut meta = zero_meta();
        meta[0] = f32::NAN;
        if let WorldAction::Move(dir) = decode_world_action(2, &meta) {
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
        } = decode_world_action(3, &meta)
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
        } = decode_world_action(3, &meta)
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
        } = decode_world_action(3, &meta)
        {
            assert_eq!(energy_transfer, 0.0);
        } else {
            panic!("expected Reproduce");
        }
    }
}
