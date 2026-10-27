use crate::contracts::Direction;
use crate::contracts::OrdinaryFoodTypeId;

/// Action type discriminant: the action-queue code `ReadActionQueueType`
/// reads, the action-log type, and the index of the per-type charge ledger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[repr(u8)]
pub enum ActionType {
    NoOp = 0,
    Eat = 1,
    Move = 2,
    Reproduce = 3,
    StealEnergy = 4,
    GiveEnergy = 5,
}

/// Number of meaningful action type discriminants (NoOp..GiveEnergy).
pub const ACTION_TYPE_COUNT: u8 = 6;
const _: () = assert!(ActionType::GiveEnergy as u8 + 1 == ACTION_TYPE_COUNT);

impl ActionType {
    /// Every action type in discriminant order, for fixed-order reports.
    pub const ALL: [Self; ACTION_TYPE_COUNT as usize] = [
        Self::NoOp,
        Self::Eat,
        Self::Move,
        Self::Reproduce,
        Self::StealEnergy,
        Self::GiveEnergy,
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
            Self::GiveEnergy => "GiveEnergy",
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
            Self::GiveEnergy => "give_energy",
        }
    }

    /// Position in [`Self::ALL`]: the discriminant as an index.
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// How an applied action ended: the kind its creature's action log stores
/// and the reward count reads (T30.F06).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[repr(u8)]
pub enum ActionOutcome {
    Success = 0,
    /// An eat found no food of its type on the cell.
    NoFood = 1,
    /// A move's target cell was off-world, a barrier or occupied.
    Blocked = 2,
    /// A reproduce's target cell was off-world, a barrier or occupied.
    InvalidTarget = 3,
    /// A reproduce would leave the parent under `min_reproduce_energy`.
    EnergyConstraints = 4,
    PopulationCap = 5,
    /// A steal drained its victim, which the commit step removes.
    TransferredAndKilled = 6,
    /// A steal found no creature on its target cell.
    NoVictim = 7,
    AgeConstraints = 8,
    /// A give found no creature on its target cell (T27.F03).
    NoRecipient = 9,
    /// A steal's victim lost no stored energy.
    NothingToTake = 10,
    /// A give moved none of the giver's stored energy.
    NothingToGive = 11,
    /// A reproduce's litter fell under `initial_energy` (T17.F01).
    LitterTooSmall = 12,
}

/// Number of action outcomes (Success..LitterTooSmall).
const ACTION_OUTCOME_COUNT: usize = 13;
const _: () = {
    assert!(ActionOutcome::LitterTooSmall as usize + 1 == ACTION_OUTCOME_COUNT);
    // `ALL[i]` has discriminant `i`, so the list is complete and ordered.
    let mut i = 0;
    while i < ACTION_OUTCOME_COUNT {
        assert!(ActionOutcome::ALL[i] as usize == i);
        i += 1;
    }
};

impl ActionOutcome {
    /// Every outcome in discriminant order, for fixed-order reports.
    pub const ALL: [Self; ACTION_OUTCOME_COUNT] = [
        Self::Success,
        Self::NoFood,
        Self::Blocked,
        Self::InvalidTarget,
        Self::EnergyConstraints,
        Self::PopulationCap,
        Self::TransferredAndKilled,
        Self::NoVictim,
        Self::AgeConstraints,
        Self::NoRecipient,
        Self::NothingToTake,
        Self::NothingToGive,
        Self::LitterTooSmall,
    ];

    /// Stable diagnostics key: the variant name the derived `Serialize`
    /// emits.
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Success => "Success",
            Self::NoFood => "NoFood",
            Self::Blocked => "Blocked",
            Self::InvalidTarget => "InvalidTarget",
            Self::EnergyConstraints => "EnergyConstraints",
            Self::PopulationCap => "PopulationCap",
            Self::TransferredAndKilled => "TransferredAndKilled",
            Self::NoVictim => "NoVictim",
            Self::AgeConstraints => "AgeConstraints",
            Self::NoRecipient => "NoRecipient",
            Self::NothingToTake => "NothingToTake",
            Self::NothingToGive => "NothingToGive",
            Self::LitterTooSmall => "LitterTooSmall",
        }
    }

    /// Whether the action applied: `Success`, or a steal that killed. Every
    /// other outcome is a failed attempt whose effort is failed effort.
    #[must_use]
    pub const fn succeeded(self) -> bool {
        matches!(self, Self::Success | Self::TransferredAndKilled)
    }
}

/// One applied action (T30.F06): its outcome, the attempt's applied effort
/// debit, and the amount the action moved.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActionResult {
    pub outcome: ActionOutcome,
    /// The applied debit of the attempt: the noop, eat or move effort, a
    /// steal's or give's exchange effort, or a reproduce's Step 0 effort.
    pub attempt_effort: f64,
    /// Food eaten, energy taken or given, a reproduce's requested transfer
    /// fraction; `0.0` otherwise.
    pub amount: f32,
}

impl ActionResult {
    /// An outcome with its effort and no amount.
    #[must_use]
    pub const fn new(outcome: ActionOutcome, attempt_effort: f64) -> Self {
        Self {
            outcome,
            attempt_effort,
            amount: 0.0,
        }
    }

    /// `self` with `amount`.
    #[must_use]
    pub const fn with_amount(self, amount: f32) -> Self {
        Self { amount, ..self }
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
    /// Pass part of the giver's own store to the neighbour in the given
    /// direction; `effort` is the decoded modifier in `[0.25, 4]` that
    /// scales the giver's `give_share` (T27.F03).
    GiveEnergy { direction: Direction, effort: f32 },
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
            WorldAction::GiveEnergy { .. } => ActionType::GiveEnergy,
        }
    }

    /// The direction a movement action commits; `None` for `NoOp` and `Eat`.
    #[inline]
    #[must_use]
    pub fn direction(&self) -> Option<Direction> {
        match self {
            WorldAction::Move(direction)
            | WorldAction::Reproduce { direction, .. }
            | WorldAction::StealEnergy { direction, .. }
            | WorldAction::GiveEnergy { direction, .. } => Some(*direction),
            WorldAction::NoOp | WorldAction::Eat { .. } => None,
        }
    }

    /// Read back a parameter slot for `ReadActionQueueParam`.
    /// Slot 0 = direction index (as f32), or the food type index (Eat);
    /// slot 1 = the effort modifier (StealEnergy, GiveEnergy), energy_transfer_fraction (Reproduce) or
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
            (WorldAction::StealEnergy { direction, .. }, 0)
            | (WorldAction::GiveEnergy { direction, .. }, 0) => direction.to_index() as f32,
            (WorldAction::StealEnergy { effort, .. }, 1)
            | (WorldAction::GiveEnergy { effort, .. }, 1) => *effort,
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
    fn a_give_is_type_five_and_reads_its_direction_and_effort() {
        let action = WorldAction::GiveEnergy {
            direction: Direction::SE,
            effort: 2.0,
        };
        assert_eq!(action.action_type(), ActionType::GiveEnergy);
        assert_eq!(action.action_type() as u8, 5);
        assert_eq!(action.direction(), Some(Direction::SE));
        assert_eq!(action.param(0), Direction::SE.to_index() as f32);
        assert_eq!(action.param(1), 2.0);
        assert_eq!(ActionType::GiveEnergy.flow_key(), "give_energy");
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
            WorldAction::GiveEnergy {
                direction: Direction::SW,
                effort: 0.25,
            },
        ];
        for action in actions {
            let json = serde_json::to_string(&action).unwrap();
            let a2: WorldAction = serde_json::from_str(&json).unwrap();
            assert_eq!(action, a2);
        }
    }

    /// Every outcome's key is its serialized variant name, its discriminant
    /// is its index in `ALL`, and only an applied outcome succeeds; `ALL`
    /// is in the contract order.
    #[test]
    fn outcome_keys_order_and_success_agree() {
        for (index, outcome) in ActionOutcome::ALL.into_iter().enumerate() {
            assert_eq!(outcome as usize, index);
            assert_eq!(
                serde_json::to_value(outcome).unwrap(),
                serde_json::Value::String(outcome.as_key().to_string())
            );
            assert_eq!(
                outcome.succeeded(),
                matches!(
                    outcome,
                    ActionOutcome::Success | ActionOutcome::TransferredAndKilled
                )
            );
        }
        let keys = ActionOutcome::ALL.map(ActionOutcome::as_key);
        assert_eq!(
            keys,
            [
                "Success",
                "NoFood",
                "Blocked",
                "InvalidTarget",
                "EnergyConstraints",
                "PopulationCap",
                "TransferredAndKilled",
                "NoVictim",
                "AgeConstraints",
                "NoRecipient",
                "NothingToTake",
                "NothingToGive",
                "LitterTooSmall",
            ]
        );
    }
}
