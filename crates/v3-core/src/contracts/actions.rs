use crate::contracts::Direction;
use crate::contracts::OrdinaryFoodTypeId;

/// The action a creature emits at the end of mesh execution for one tick.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum WorldAction {
    /// Do nothing this tick.
    NoOp,
    /// Consume food on the current cell for the configured food type.
    Eat { type_idx: OrdinaryFoodTypeId },
    /// Move one step in the given direction.
    Move(Direction),
    /// Attempt to spawn an offspring in the given direction, transferring energy.
    Reproduce {
        direction: Direction,
        energy_transfer: f32,
    },
    /// Attempt to steal energy from a neighbor in the given direction.
    StealEnergy { direction: Direction, amount: f32 },
}

impl WorldAction {
    /// Construct a typed Eat action for the given ordinary food type.
    #[must_use]
    pub const fn eat(type_idx: OrdinaryFoodTypeId) -> Self {
        Self::Eat { type_idx }
    }

    pub fn is_noop(&self) -> bool {
        matches!(self, WorldAction::NoOp)
    }

    /// Return the action type discriminant matching the decode_world_action encoding.
    /// 0=NoOp, 1=Eat, 2=Move, 3=Reproduce, 4=StealEnergy.
    #[inline]
    pub fn action_type(&self) -> u8 {
        match self {
            WorldAction::NoOp => 0,
            WorldAction::Eat { .. } => 1,
            WorldAction::Move(_) => 2,
            WorldAction::Reproduce { .. } => 3,
            WorldAction::StealEnergy { .. } => 4,
        }
    }

    /// The direction a movement action commits; `None` for `NoOp` and `Eat`.
    #[inline]
    #[must_use]
    pub fn direction(&self) -> Option<Direction> {
        match self {
            WorldAction::Move(direction)
            | WorldAction::Reproduce { direction, .. }
            | WorldAction::StealEnergy { direction, .. } => Some(*direction),
            WorldAction::NoOp | WorldAction::Eat { .. } => None,
        }
    }

    /// Read back a parameter slot, mirroring the meta buffer layout used during encoding.
    /// Slot 0 = direction index (as f32), slot 1 = amount/energy_transfer. Returns 0.0 for
    /// unknown slots or variants without that parameter.
    #[inline]
    pub fn param(&self, slot: usize) -> f32 {
        match (self, slot) {
            (WorldAction::Eat { type_idx }, 0) => type_idx.get() as f32,
            (WorldAction::Move(dir), 0) => dir.to_index() as f32,
            (WorldAction::Reproduce { direction, .. }, 0) => direction.to_index() as f32,
            (
                WorldAction::Reproduce {
                    energy_transfer, ..
                },
                1,
            ) => *energy_transfer,
            (WorldAction::StealEnergy { direction, .. }, 0) => direction.to_index() as f32,
            (WorldAction::StealEnergy { amount, .. }, 1) => *amount,
            _ => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::OrdinaryFoodTypeId;

    #[test]
    fn noop_is_noop() {
        assert!(WorldAction::NoOp.is_noop());
    }

    #[test]
    fn other_actions_not_noop() {
        assert!(!WorldAction::Eat {
            type_idx: OrdinaryFoodTypeId::default()
        }
        .is_noop());
        assert!(!WorldAction::Move(Direction::N).is_noop());
        assert!(!WorldAction::Reproduce {
            direction: Direction::S,
            energy_transfer: 5.0
        }
        .is_noop());
        assert!(!WorldAction::StealEnergy {
            direction: Direction::E,
            amount: 10.0
        }
        .is_noop());
    }

    #[test]
    fn direction_is_the_committed_direction_of_movement_variants_only() {
        assert_eq!(
            WorldAction::Move(Direction::N).direction(),
            Some(Direction::N)
        );
        assert_eq!(
            WorldAction::Reproduce {
                direction: Direction::SE,
                energy_transfer: 1.0
            }
            .direction(),
            Some(Direction::SE)
        );
        assert_eq!(
            WorldAction::StealEnergy {
                direction: Direction::W,
                amount: 1.0
            }
            .direction(),
            Some(Direction::W)
        );
        assert_eq!(WorldAction::NoOp.direction(), None);
        assert_eq!(
            WorldAction::Eat {
                type_idx: OrdinaryFoodTypeId::default()
            }
            .direction(),
            None
        );
    }

    #[test]
    fn reproduce_fields_accessible() {
        let action = WorldAction::Reproduce {
            direction: Direction::SE,
            energy_transfer: 10.0,
        };
        if let WorldAction::Reproduce {
            direction,
            energy_transfer,
        } = action
        {
            assert_eq!(direction, Direction::SE);
            assert!((energy_transfer - 10.0).abs() < f32::EPSILON);
        } else {
            panic!("expected Reproduce variant");
        }
    }

    #[test]
    fn steal_energy_fields_accessible() {
        let action = WorldAction::StealEnergy {
            direction: Direction::NW,
            amount: 7.5,
        };
        if let WorldAction::StealEnergy { direction, amount } = action {
            assert_eq!(direction, Direction::NW);
            assert!((amount - 7.5).abs() < f32::EPSILON);
        } else {
            panic!("expected StealEnergy variant");
        }
    }

    #[test]
    fn world_action_serde_roundtrip() {
        let actions = vec![
            WorldAction::NoOp,
            WorldAction::Eat {
                type_idx: OrdinaryFoodTypeId::default(),
            },
            WorldAction::Move(Direction::W),
            WorldAction::Reproduce {
                direction: Direction::NE,
                energy_transfer: 15.5,
            },
            WorldAction::StealEnergy {
                direction: Direction::S,
                amount: 8.0,
            },
        ];
        for action in actions {
            let json = serde_json::to_string(&action).unwrap();
            let a2: WorldAction = serde_json::from_str(&json).unwrap();
            assert_eq!(action, a2);
        }
    }
}
