use crate::contracts::Direction;

/// The action a creature emits at the end of mesh execution for one tick.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum WorldAction {
    /// Do nothing this tick.
    NoOp,
    /// Consume food on the current cell.
    Eat,
    /// Move one step in the given direction.
    Move(Direction),
    /// Attempt to spawn an offspring in the given direction, transferring energy.
    Reproduce {
        direction: Direction,
        energy_transfer: f32,
    },
}

impl WorldAction {
    pub fn is_noop(&self) -> bool {
        matches!(self, WorldAction::NoOp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_is_noop() {
        assert!(WorldAction::NoOp.is_noop());
    }

    #[test]
    fn other_actions_not_noop() {
        assert!(!WorldAction::Eat.is_noop());
        assert!(!WorldAction::Move(Direction::N).is_noop());
        assert!(!WorldAction::Reproduce {
            direction: Direction::S,
            energy_transfer: 5.0
        }
        .is_noop());
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
    fn world_action_serde_roundtrip() {
        let actions = vec![
            WorldAction::NoOp,
            WorldAction::Eat,
            WorldAction::Move(Direction::W),
            WorldAction::Reproduce {
                direction: Direction::NE,
                energy_transfer: 15.5,
            },
        ];
        for action in actions {
            let json = serde_json::to_string(&action).unwrap();
            let a2: WorldAction = serde_json::from_str(&json).unwrap();
            assert_eq!(action, a2);
        }
    }
}
