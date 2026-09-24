use crate::config::OrdinaryFoodTypeId;
use crate::contracts::{Direction, WorldAction};
use crate::creature::genome::vote::{ActionParamField, VoteSink, ACTION_PARAM_FIELD_COUNT};

/// The parameter surface a commit reads, indexed by `ActionParamField::index`.
pub type ActionParams = [f32; ACTION_PARAM_FIELD_COUNT];

/// Decode the action a pass commits (T19.F04): the direction is the winning
/// sink's, and the kind's parameter field supplies the rest (T11.F27): `Eat`
/// reads `EatFoodType`, `Reproduce` `ReproduceTransferFraction` and
/// `StealEnergy` `StealEnergyAmount`; `Move` reads nothing. `Terminate` and
/// `Decide` never commit and decode to `NoOp`.
#[must_use]
pub fn decode_commit(sink: VoteSink, params: &ActionParams) -> WorldAction {
    match sink {
        VoteSink::Eat => WorldAction::Eat {
            type_idx: decode_food_type_idx(params[ActionParamField::EatFoodType.index()]),
        },
        VoteSink::Move(d) => WorldAction::Move(sink_direction(d)),
        VoteSink::Reproduce(d) => WorldAction::Reproduce {
            direction: sink_direction(d),
            energy_transfer_fraction: clamp_unit_interval(
                params[ActionParamField::ReproduceTransferFraction.index()],
            ),
        },
        VoteSink::StealEnergy(d) => WorldAction::StealEnergy {
            direction: sink_direction(d),
            amount: clamp_non_negative_finite(params[ActionParamField::StealEnergyAmount.index()]),
        },
        VoteSink::Terminate | VoteSink::Decide => WorldAction::NoOp,
    }
}

/// The direction of a directed sink; the catalog never builds one outside
/// `0..8`, and an out-of-range index clamps to the last direction.
#[inline]
fn sink_direction(d: u8) -> Direction {
    Direction::ALL[usize::from(d).min(Direction::ALL.len() - 1)]
}

#[inline]
fn decode_food_type_idx(raw: f32) -> OrdinaryFoodTypeId {
    if raw.is_finite() && raw >= 0.0 {
        OrdinaryFoodTypeId::new(raw.round().clamp(0.0, u16::MAX as f32) as u16)
    } else {
        OrdinaryFoodTypeId::default()
    }
}

/// Clamp to the unit interval (T17.F01): NaN, ±∞, and negatives become 0.0;
/// values above 1.0 become 1.0. The reproduce action carries its parameter
/// as a fraction of the parent's post-cost energy, so this is the only shape
/// `apply_reproduce` accepts.
#[inline]
#[must_use]
pub(crate) fn clamp_unit_interval(v: f32) -> f32 {
    if v.is_finite() {
        v.clamp(0.0, 1.0)
    } else {
        0.0
    }
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
    use proptest::prelude::*;

    #[test]
    fn directed_sinks_commit_their_own_direction() {
        for (d, direction) in Direction::ALL.into_iter().enumerate() {
            let d = d as u8;
            assert_eq!(
                decode_commit(VoteSink::Move(d), &[9.0, 9.0, 9.0]),
                WorldAction::Move(direction)
            );
            assert_eq!(
                decode_commit(VoteSink::Reproduce(d), &[0.0, 0.25, 0.0]),
                WorldAction::Reproduce {
                    direction,
                    energy_transfer_fraction: 0.25,
                }
            );
            assert_eq!(
                decode_commit(VoteSink::StealEnergy(d), &[0.0, 0.0, 3.0]),
                WorldAction::StealEnergy {
                    direction,
                    amount: 3.0,
                }
            );
        }
    }

    /// A directed sink index outside `0..8` clamps to the last direction
    /// rather than panicking.
    #[test]
    fn out_of_range_sink_direction_clamps_to_the_last_direction() {
        let last = Direction::ALL[Direction::ALL.len() - 1];
        for d in [8, 9, u8::MAX] {
            assert_eq!(
                decode_commit(VoteSink::Move(d), &[0.0; 3]),
                WorldAction::Move(last)
            );
        }
    }

    #[test]
    fn eat_reads_its_food_type_field() {
        assert_eq!(
            decode_commit(VoteSink::Eat, &[3.0, 8.0, 8.0]),
            WorldAction::eat(OrdinaryFoodTypeId::new(3))
        );
        for raw in [f32::NAN, f32::INFINITY, -1.0] {
            assert_eq!(
                decode_commit(VoteSink::Eat, &[raw, 0.0, 0.0]),
                WorldAction::eat(OrdinaryFoodTypeId::default())
            );
        }
    }

    #[test]
    fn pass_control_sinks_decode_to_noop() {
        assert_eq!(
            decode_commit(VoteSink::Terminate, &[1.0; 3]),
            WorldAction::NoOp
        );
        assert_eq!(
            decode_commit(VoteSink::Decide, &[1.0; 3]),
            WorldAction::NoOp
        );
    }

    #[test]
    fn steal_amount_is_non_negative_finite() {
        for raw in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -2.0] {
            assert_eq!(
                decode_commit(VoteSink::StealEnergy(0), &[0.0, 0.0, raw]),
                WorldAction::StealEnergy {
                    direction: Direction::N,
                    amount: 0.0,
                }
            );
        }
    }

    /// T17.F01 invariant 1: the reproduce parameter leaves decode as a
    /// fraction in [0, 1].
    #[test]
    fn reproduce_fraction_is_sanitized_to_unit_interval() {
        for (raw, expected) in [
            (f32::NAN, 0.0_f32),
            (f32::INFINITY, 0.0),
            (f32::NEG_INFINITY, 0.0),
            (-0.5, 0.0),
            (0.0, 0.0),
            (0.42, 0.42),
            (1.0, 1.0),
            (7.0, 1.0),
        ] {
            let WorldAction::Reproduce {
                energy_transfer_fraction,
                ..
            } = decode_commit(VoteSink::Reproduce(0), &[0.0, raw, 0.0])
            else {
                panic!("expected Reproduce");
            };
            assert_eq!(
                energy_transfer_fraction.to_bits(),
                expected.to_bits(),
                "{raw}"
            );
        }
    }

    /// `decode_commit` for every sink `field` is read by, with `value` in
    /// `field` and every other field zero.
    fn decode_field(field: ActionParamField, value: f32) -> Vec<WorldAction> {
        let mut params = [0.0; ACTION_PARAM_FIELD_COUNT];
        params[field.index()] = value;
        VoteSink::all()
            .filter(|sink| sink.kind() == Some(field.kind()))
            .map(|sink| decode_commit(sink, &params))
            .collect()
    }

    /// T11.F27 decoder agreement, sensitivity: every catalogued field has a
    /// pair of finite values, the other fields fixed, that changes the action
    /// its kind's sinks commit.
    #[test]
    fn every_action_param_field_changes_its_kinds_commit() {
        const PROBES: [f32; 5] = [0.0, 0.25, 0.5, 1.0, 3.0];
        for field in ActionParamField::ALL {
            let changes = PROBES.iter().any(|&a| {
                PROBES
                    .iter()
                    .any(|&b| decode_field(field, a) != decode_field(field, b))
            });
            assert!(changes, "{field:?}");
        }
    }

    proptest! {
        /// T11.F27 decoder agreement, independence: a field never changes
        /// the action of a sink of another kind, including `Move`,
        /// `Terminate` and `Decide`, whatever finite values the surface holds.
        #[test]
        fn a_field_never_changes_another_kinds_commit(
            field_index in 0..ActionParamField::ALL.len(),
            base in prop::array::uniform3(prop::num::f32::NORMAL | prop::num::f32::SUBNORMAL | prop::num::f32::ZERO),
            value in prop::num::f32::NORMAL | prop::num::f32::SUBNORMAL | prop::num::f32::ZERO,
        ) {
            let field = ActionParamField::ALL[field_index];
            let mut changed = base;
            changed[field.index()] = value;
            for sink in VoteSink::all().filter(|sink| sink.kind() != Some(field.kind())) {
                prop_assert_eq!(decode_commit(sink, &base), decode_commit(sink, &changed), "{:?}", sink);
            }
        }

        #[test]
        fn reproduce_fraction_always_lands_in_unit_interval(raw in prop::num::f32::ANY) {
            let WorldAction::Reproduce {
                energy_transfer_fraction,
                ..
            } = decode_commit(VoteSink::Reproduce(2), &[0.0, raw, 0.0])
            else {
                panic!("expected Reproduce");
            };
            prop_assert!((0.0..=1.0).contains(&energy_transfer_fraction));
            if raw.is_finite() {
                prop_assert_eq!(energy_transfer_fraction, raw.clamp(0.0, 1.0));
            }
        }
    }
}
