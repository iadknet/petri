use crate::config::OrdinaryFoodTypeId;
use crate::contracts::{Direction, WorldAction};
use crate::creature::genome::vote::{
    ActionParamField, DirectedVoteKind, VoteSink, ACTION_PARAM_FIELD_COUNT,
};

/// The parameter surface a commit reads, indexed by `ActionParamField::index`.
pub type ActionParams = [f32; ACTION_PARAM_FIELD_COUNT];

/// Decode the action a pass commits (T19.F04): the direction is the winning
/// sink's, and the kind's parameter fields supply the rest (T11.F27): `Eat`
/// reads `EatFoodType` and `EatLeaveShare` (T02.F08, clamped to `[0, 1]`, so
/// an unwired field eats the whole cell), `Reproduce` `ReproduceTransferFraction` and
/// `StealEnergy` `StealEffort` as its effort modifier ([`effort_modifier`],
/// T27.F03); `Move` reads nothing. A drive,
/// `Terminate` and `Decide` never commit and decode to `NoOp`.
#[must_use]
pub fn decode_commit(sink: VoteSink, params: &ActionParams) -> WorldAction {
    match sink {
        VoteSink::Eat => WorldAction::Eat {
            type_idx: decode_food_type_idx(params[ActionParamField::EatFoodType.index()]),
            leave: clamp_unit_interval(params[ActionParamField::EatLeaveShare.index()]),
        },
        VoteSink::Directed(kind, d) => {
            let direction = sink_direction(d);
            match kind {
                DirectedVoteKind::Move => WorldAction::Move(direction),
                DirectedVoteKind::Reproduce => WorldAction::Reproduce {
                    direction,
                    energy_transfer_fraction: clamp_unit_interval(
                        params[ActionParamField::ReproduceTransferFraction.index()],
                    ),
                },
                DirectedVoteKind::StealEnergy => WorldAction::StealEnergy {
                    direction,
                    effort: effort_modifier(params[ActionParamField::StealEffort.index()]),
                },
            }
        }
        VoteSink::Drive(_) | VoteSink::Terminate | VoteSink::Decide => WorldAction::NoOp,
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

/// An exchange act's effort modifier (T27.F03): `2^clamp(x, −2, 2)`, in
/// `[0.25, 4]`; the unwired 0 and a non-finite `x` give exactly 1.0, so an
/// act no field drives moves its actor's heritable share.
#[inline]
#[must_use]
pub fn effort_modifier(x: f32) -> f32 {
    if x.is_finite() {
        x.clamp(-2.0, 2.0).exp2()
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use ActionParamField::{EatFoodType, EatLeaveShare, ReproduceTransferFraction, StealEffort};

    /// A parameter surface holding `fields` and zero elsewhere.
    fn surface(fields: &[(ActionParamField, f32)]) -> ActionParams {
        let mut params = [0.0; ACTION_PARAM_FIELD_COUNT];
        for &(field, value) in fields {
            params[field.index()] = value;
        }
        params
    }

    #[test]
    fn directed_sinks_commit_their_own_direction() {
        for (d, direction) in Direction::ALL.into_iter().enumerate() {
            let d = d as u8;
            assert_eq!(
                decode_commit(
                    VoteSink::Directed(DirectedVoteKind::Move, d),
                    &[9.0; ACTION_PARAM_FIELD_COUNT]
                ),
                WorldAction::Move(direction)
            );
            assert_eq!(
                decode_commit(
                    VoteSink::Directed(DirectedVoteKind::Reproduce, d),
                    &surface(&[(ReproduceTransferFraction, 0.25)])
                ),
                WorldAction::Reproduce {
                    direction,
                    energy_transfer_fraction: 0.25,
                }
            );
            assert_eq!(
                decode_commit(
                    VoteSink::Directed(DirectedVoteKind::StealEnergy, d),
                    &surface(&[(StealEffort, 1.0)])
                ),
                WorldAction::StealEnergy {
                    direction,
                    effort: 2.0,
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
                decode_commit(
                    VoteSink::Directed(DirectedVoteKind::Move, d),
                    &[0.0; ACTION_PARAM_FIELD_COUNT]
                ),
                WorldAction::Move(last)
            );
        }
    }

    #[test]
    fn eat_reads_its_food_type_field() {
        assert_eq!(
            decode_commit(
                VoteSink::Eat,
                &surface(&[
                    (EatFoodType, 3.0),
                    (ReproduceTransferFraction, 8.0),
                    (StealEffort, 8.0)
                ])
            ),
            WorldAction::eat(OrdinaryFoodTypeId::new(3))
        );
        for raw in [f32::NAN, f32::INFINITY, -1.0] {
            assert_eq!(
                decode_commit(VoteSink::Eat, &surface(&[(EatFoodType, raw)])),
                WorldAction::eat(OrdinaryFoodTypeId::default())
            );
        }
    }

    /// T02.F08: `Eat` reads its leave share clamped to `[0, 1]`; a zero
    /// (unwired) surface eats the whole cell.
    #[test]
    fn eat_reads_its_leave_share_clamped_to_unit_interval() {
        assert_eq!(
            decode_commit(VoteSink::Eat, &[0.0; ACTION_PARAM_FIELD_COUNT]),
            WorldAction::eat(OrdinaryFoodTypeId::default())
        );
        for (raw, expected) in [
            (f32::NAN, 0.0_f32),
            (f32::INFINITY, 0.0),
            (f32::NEG_INFINITY, 0.0),
            (-0.5, 0.0),
            (0.0, 0.0),
            (0.3, 0.3),
            (1.0, 1.0),
            (4.0, 1.0),
        ] {
            let WorldAction::Eat { type_idx, leave } = decode_commit(
                VoteSink::Eat,
                &surface(&[(EatFoodType, 2.0), (EatLeaveShare, raw)]),
            ) else {
                panic!("expected Eat");
            };
            assert_eq!(type_idx, OrdinaryFoodTypeId::new(2));
            assert_eq!(leave.to_bits(), expected.to_bits(), "{raw}");
        }
    }

    #[test]
    fn pass_control_sinks_decode_to_noop() {
        assert_eq!(
            decode_commit(VoteSink::Terminate, &[1.0; ACTION_PARAM_FIELD_COUNT]),
            WorldAction::NoOp
        );
        assert_eq!(
            decode_commit(VoteSink::Decide, &[1.0; ACTION_PARAM_FIELD_COUNT]),
            WorldAction::NoOp
        );
    }

    /// T27.F03 invariant 5: the effort modifier is `2^clamp(x, −2, 2)`, and
    /// the unwired 0 and a non-finite value give exactly 1.0.
    #[test]
    fn steal_effort_is_a_bounded_power_of_two_and_one_when_unwired() {
        for (raw, expected) in [
            (0.0_f32, 1.0_f32),
            (-0.0, 1.0),
            (f32::NAN, 1.0),
            (f32::INFINITY, 1.0),
            (f32::NEG_INFINITY, 1.0),
            (1.0, 2.0),
            (-1.0, 0.5),
            (2.0, 4.0),
            (-2.0, 0.25),
            (9.0, 4.0),
            (-9.0, 0.25),
        ] {
            assert_eq!(
                decode_commit(
                    VoteSink::Directed(DirectedVoteKind::StealEnergy, 0),
                    &surface(&[(StealEffort, raw)])
                ),
                WorldAction::StealEnergy {
                    direction: Direction::N,
                    effort: expected,
                },
                "{raw}"
            );
        }
    }

    proptest! {
        /// Every field value decodes to a modifier in `[0.25, 4]`.
        #[test]
        fn the_effort_modifier_stays_in_its_bounds(bits in any::<u32>()) {
            let effort = effort_modifier(f32::from_bits(bits));
            prop_assert!((0.25..=4.0).contains(&effort));
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
            } = decode_commit(
                VoteSink::Directed(DirectedVoteKind::Reproduce, 0),
                &surface(&[(ReproduceTransferFraction, raw)]),
            )
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
            base in prop::array::uniform4(prop::num::f32::NORMAL | prop::num::f32::SUBNORMAL | prop::num::f32::ZERO),
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
            } = decode_commit(VoteSink::Directed(DirectedVoteKind::Reproduce, 2), &surface(&[(ReproduceTransferFraction, raw)]))
            else {
                panic!("expected Reproduce");
            };
            prop_assert!((0.0..=1.0).contains(&energy_transfer_fraction));
            if raw.is_finite() {
                prop_assert_eq!(energy_transfer_fraction, raw.clamp(0.0, 1.0));
            }
        }

        /// T02.F08: the leave share lands in `[0, 1]` for any `f32` and is
        /// `clamp(0, 1)` of a finite input.
        #[test]
        fn eat_leave_share_always_lands_in_unit_interval(raw in prop::num::f32::ANY) {
            let WorldAction::Eat { leave, .. } =
                decode_commit(VoteSink::Eat, &surface(&[(EatLeaveShare, raw)]))
            else {
                panic!("expected Eat");
            };
            prop_assert!((0.0..=1.0).contains(&leave));
            if raw.is_finite() {
                prop_assert_eq!(leave.to_bits(), raw.clamp(0.0, 1.0).to_bits());
            }
        }
    }
}
