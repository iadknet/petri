use crate::contracts::Direction;
use crate::contracts::OrdinaryFoodTypeId;

/// Action type discriminant: the action-queue code `ReadActionQueueType`
/// reads, the action-log type, and the index of the per-type charge ledger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[non_exhaustive]
#[repr(u8)]
pub enum ActionType {
    NoOp = 0,
    Eat = 1,
    Move = 2,
    Reproduce = 3,
    StealEnergy = 4,
}

/// Number of meaningful action type discriminants (NoOp..StealEnergy).
pub const ACTION_TYPE_COUNT: u8 = 5;
const _: () = assert!(ActionType::StealEnergy as u8 + 1 == ACTION_TYPE_COUNT);

impl ActionType {
    /// Every action type in discriminant order, for fixed-order reports.
    pub const ALL: [Self; ACTION_TYPE_COUNT as usize] = [
        Self::NoOp,
        Self::Eat,
        Self::Move,
        Self::Reproduce,
        Self::StealEnergy,
    ];

    /// Stable diagnostics key for this action type.
    ///
    /// Matches the variant name emitted by the derived `Serialize` impl.
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::NoOp => "NoOp",
            Self::Eat => "Eat",
            Self::Move => "Move",
            Self::Reproduce => "Reproduce",
            Self::StealEnergy => "StealEnergy",
        }
    }

    /// The energy-ledger key of this type's charge and failed-effort rows.
    #[must_use]
    pub const fn flow_key(self) -> &'static str {
        match self {
            Self::NoOp => "noop",
            Self::Eat => "eat",
            Self::Move => "move",
            Self::Reproduce => "reproduce",
            Self::StealEnergy => "steal_energy",
        }
    }

    /// Position in [`Self::ALL`]: the discriminant as an index.
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// The action a creature emits at the end of mesh execution for one tick.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum WorldAction {
    /// Do nothing this tick.
    NoOp,
    /// Consume food of `type_idx` on the current cell, leaving the share
    /// `leave` (in [0, 1]) of the cell's density in place (T02.F08).
    Eat {
        type_idx: OrdinaryFoodTypeId,
        leave: f32,
    },
    /// Move one step in the given direction.
    Move(Direction),
    /// Attempt to spawn an offspring in the given direction. The child starts
    /// with `energy_transfer_fraction` (in [0, 1]) of the parent's post-cost
    /// energy, capped at `default_offspring_energy` (T17.F01).
    Reproduce {
        direction: Direction,
        energy_transfer_fraction: f32,
    },
    /// Attempt to steal energy from the neighbour in the given direction;
    /// `effort` is the decoded modifier in `[0.25, 4]` that scales the
    /// attacker's `steal_share` (T27.F03).
    StealEnergy { direction: Direction, effort: f32 },
}

impl WorldAction {
    /// Construct a typed Eat action for the given ordinary food type that
    /// eats the whole cell (`leave` 0.0).
    #[must_use]
    pub const fn eat(type_idx: OrdinaryFoodTypeId) -> Self {
        Self::Eat {
            type_idx,
            leave: 0.0,
        }
    }

    pub fn is_noop(&self) -> bool {
        matches!(self, WorldAction::NoOp)
    }

    /// The action's type; `as u8` is the code `ReadActionQueueType` reads.
    #[inline]
    #[must_use]
    pub const fn action_type(&self) -> ActionType {
        match self {
            WorldAction::NoOp => ActionType::NoOp,
            WorldAction::Eat { .. } => ActionType::Eat,
            WorldAction::Move(_) => ActionType::Move,
            WorldAction::Reproduce { .. } => ActionType::Reproduce,
            WorldAction::StealEnergy { .. } => ActionType::StealEnergy,
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

    /// Read back a parameter slot for `ReadActionQueueParam`.
    /// Slot 0 = direction index (as f32), or the food type index (Eat);
    /// slot 1 = the effort modifier (StealEnergy), energy_transfer_fraction (Reproduce) or
    /// the leave share (Eat). Returns 0.0 for unknown slots or
    /// variants without that parameter.
    #[inline]
    pub fn param(&self, slot: usize) -> f32 {
        match (self, slot) {
            (WorldAction::Eat { type_idx, .. }, 0) => type_idx.get() as f32,
            (WorldAction::Eat { leave, .. }, 1) => *leave,
            (WorldAction::Move(dir), 0) => dir.to_index() as f32,
            (WorldAction::Reproduce { direction, .. }, 0) => direction.to_index() as f32,
            (
                WorldAction::Reproduce {
                    energy_transfer_fraction,
                    ..
                },
                1,
            ) => *energy_transfer_fraction,
            (WorldAction::StealEnergy { direction, .. }, 0) => direction.to_index() as f32,
            (WorldAction::StealEnergy { effort, .. }, 1) => *effort,
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
        assert!(!WorldAction::eat(OrdinaryFoodTypeId::default()).is_noop());
        assert!(!WorldAction::Move(Direction::N).is_noop());
        assert!(!WorldAction::Reproduce {
            direction: Direction::S,
            energy_transfer_fraction: 5.0
        }
        .is_noop());
        assert!(!WorldAction::StealEnergy {
            direction: Direction::E,
            effort: 1.0
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
                energy_transfer_fraction: 1.0
            }
            .direction(),
            Some(Direction::SE)
        );
        assert_eq!(
            WorldAction::StealEnergy {
                direction: Direction::W,
                effort: 1.0
            }
            .direction(),
            Some(Direction::W)
        );
        assert_eq!(WorldAction::NoOp.direction(), None);
        assert_eq!(
            WorldAction::eat(OrdinaryFoodTypeId::default()).direction(),
            None
        );
    }

    #[test]
    fn eat_builds_a_whole_cell_eat_and_param_reads_type_then_leave() {
        let whole = WorldAction::eat(OrdinaryFoodTypeId::new(2));
        assert_eq!(
            whole,
            WorldAction::Eat {
                type_idx: OrdinaryFoodTypeId::new(2),
                leave: 0.0
            }
        );
        let partial = WorldAction::Eat {
            type_idx: OrdinaryFoodTypeId::new(3),
            leave: 0.25,
        };
        assert_eq!(partial.param(0).to_bits(), 3.0f32.to_bits());
        assert_eq!(partial.param(1).to_bits(), 0.25f32.to_bits());
        assert_eq!(whole.param(1).to_bits(), 0.0f32.to_bits());
        assert_eq!(partial.param(2).to_bits(), 0.0f32.to_bits());
    }

    #[test]
    fn reproduce_fields_accessible() {
        let action = WorldAction::Reproduce {
            direction: Direction::SE,
            energy_transfer_fraction: 10.0,
        };
        if let WorldAction::Reproduce {
            direction,
            energy_transfer_fraction,
        } = action
        {
            assert_eq!(direction, Direction::SE);
            assert!((energy_transfer_fraction - 10.0).abs() < f32::EPSILON);
        } else {
            panic!("expected Reproduce variant");
        }
    }

    #[test]
    fn steal_energy_fields_accessible() {
        let action = WorldAction::StealEnergy {
            direction: Direction::NW,
            effort: 0.5,
        };
        assert_eq!(action.param(1), 0.5, "slot 1 reads the effort modifier");
        if let WorldAction::StealEnergy { direction, effort } = action {
            assert_eq!(direction, Direction::NW);
            assert_eq!(effort, 0.5);
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
                leave: 0.25,
            },
            WorldAction::Move(Direction::W),
            WorldAction::Reproduce {
                direction: Direction::NE,
                energy_transfer_fraction: 15.5,
            },
            WorldAction::StealEnergy {
                direction: Direction::S,
                effort: 4.0,
            },
        ];
        for action in actions {
            let json = serde_json::to_string(&action).unwrap();
            let a2: WorldAction = serde_json::from_str(&json).unwrap();
            assert_eq!(action, a2);
        }
    }
}
