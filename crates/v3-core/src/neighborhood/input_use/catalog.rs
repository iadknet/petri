//! The `input-use-v1` catalog: the 19 named input families the mutation
//! engine draws, `UpstreamSlot` (one channel per slot), and the separate
//! shared-memory inventory, with the channel each read actually addresses
//! under `runtime::inputs::resolve_input`.

use crate::contracts::{
    DynamicIntrospectionKey, InputReference, StaticIntrospectionKey, WorldInputKey,
};
use crate::mutation::compound::sub_value_count;

/// One input family. Food families carry their food type. Declaration order
/// follows the buckets of `mutation::sampling::random_input_reference_for_food_types`,
/// then `UpstreamSlot`, then the two shared-memory banks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Family {
    FoodHere(u16),
    NeighborFoodRing(u16),
    NeighborBarrierRing,
    NeighborOccupiedRing,
    AgeTicks,
    EnergyCurrent,
    EnergyConsumedThisTick,
    ActionQueue,
    AreaFoodSummary(u16),
    AreaBarrierSummary,
    NearbyCreatureCore,
    NearbyCreatureVitals,
    NearbyCreatureIdentity,
    AreaOccupancySummary,
    ActionVotes,
    PreviousPassVotes,
    CommitCounts,
    HopsThisTick,
    PreviousOutcome,
    /// Channel = slot.
    UpstreamSlot,
    /// Shared memory, current tick (channel = slot); no declaration or causal stage.
    SharedMemory,
    /// Shared memory, previous tick (channel = slot); no declaration or causal stage.
    SharedMemoryPrevious,
}

/// The mutation draw width of `ActionQueue`: channels at or past it are
/// labelled `beyond_draw_width` (audit S4).
pub const ACTION_QUEUE_DRAW_WIDTH: u16 = 12;

impl Family {
    /// The family of a reference.
    #[must_use]
    pub fn of(reference: &InputReference) -> Self {
        match reference {
            InputReference::World(key) => match *key {
                WorldInputKey::FoodHere { type_idx } => Self::FoodHere(type_idx.get()),
                WorldInputKey::NeighborFoodRing { type_idx } => {
                    Self::NeighborFoodRing(type_idx.get())
                }
                WorldInputKey::NeighborBarrierRing => Self::NeighborBarrierRing,
                WorldInputKey::NeighborOccupiedRing => Self::NeighborOccupiedRing,
                WorldInputKey::AreaFoodSummary { type_idx } => {
                    Self::AreaFoodSummary(type_idx.get())
                }
                WorldInputKey::AreaBarrierSummary => Self::AreaBarrierSummary,
                WorldInputKey::AreaOccupancySummary => Self::AreaOccupancySummary,
                WorldInputKey::NearbyCreatureCore => Self::NearbyCreatureCore,
                WorldInputKey::NearbyCreatureVitals => Self::NearbyCreatureVitals,
                WorldInputKey::NearbyCreatureIdentity => Self::NearbyCreatureIdentity,
            },
            InputReference::StaticIntrospection(StaticIntrospectionKey::AgeTicks) => Self::AgeTicks,
            InputReference::DynamicIntrospection(key) => match key {
                DynamicIntrospectionKey::EnergyCurrent => Self::EnergyCurrent,
                DynamicIntrospectionKey::EnergyConsumedThisTick => Self::EnergyConsumedThisTick,
                DynamicIntrospectionKey::HopsThisTick => Self::HopsThisTick,
            },
            InputReference::UpstreamSlot(_) => Self::UpstreamSlot,
            InputReference::ActionQueue => Self::ActionQueue,
            InputReference::ActionVotes => Self::ActionVotes,
            InputReference::PreviousPassVotes => Self::PreviousPassVotes,
            InputReference::CommitCounts => Self::CommitCounts,
            InputReference::PreviousOutcome => Self::PreviousOutcome,
        }
    }

    /// Whether the family is a shared-memory bank (no declaration or causal stage).
    #[must_use]
    pub const fn is_shared_memory(self) -> bool {
        matches!(self, Self::SharedMemory | Self::SharedMemoryPrevious)
    }

    /// Report label: the variant name, with the food type appended.
    #[must_use]
    pub fn label(self) -> String {
        let food = |name: &str, type_idx: u16| format!("{name}:{type_idx}");
        match self {
            Self::FoodHere(t) => food("FoodHere", t),
            Self::NeighborFoodRing(t) => food("NeighborFoodRing", t),
            Self::AreaFoodSummary(t) => food("AreaFoodSummary", t),
            other => format!("{other:?}"),
        }
    }
}

/// One family channel: the value the resolver actually addresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Channel {
    pub family: Family,
    pub channel: u16,
}

impl Channel {
    /// An `ActionQueue` channel at or past the mutation draw width.
    #[must_use]
    pub fn beyond_draw_width(self) -> bool {
        self.family == Family::ActionQueue && self.channel >= ACTION_QUEUE_DRAW_WIDTH
    }
}

/// What one reference read addresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Addressed {
    Channel(Channel),
    /// A decision-compound read at or past its width: a constant 0.0.
    OutOfWidth(Family),
}

/// The channel a read of `reference` at `sub_idx` resolves, mirroring
/// `resolve_input`: scalars read channel 0, world compounds wrap `sub_idx`
/// modulo their width, `ActionQueue` addresses the raw `sub_idx`, decision
/// compounds address `sub_idx` below their width and nothing at or past it,
/// and `UpstreamSlot(k)` is channel `k`.
#[must_use]
pub fn addressed(reference: &InputReference, sub_idx: u16) -> Addressed {
    let family = Family::of(reference);
    let channel = match reference {
        InputReference::World(key) if key.compound_width() > 1 => sub_idx % key.compound_width(),
        InputReference::ActionQueue => sub_idx,
        InputReference::ActionVotes
        | InputReference::PreviousPassVotes
        | InputReference::CommitCounts
        | InputReference::PreviousOutcome => {
            if sub_idx >= sub_value_count(reference) {
                return Addressed::OutOfWidth(family);
            }
            sub_idx
        }
        InputReference::UpstreamSlot(slot) => u16::try_from(*slot).unwrap_or(u16::MAX),
        _ => 0,
    };
    Addressed::Channel(Channel { family, channel })
}

/// A shared-memory channel.
#[must_use]
pub fn shared_memory(slot: usize, previous: bool) -> Channel {
    Channel {
        family: if previous {
            Family::SharedMemoryPrevious
        } else {
            Family::SharedMemory
        },
        channel: (slot % 16) as u16,
    }
}

/// What a node's `input_refs` entry declares: its family, and for
/// `UpstreamSlot` its slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Declaration {
    pub family: Family,
    pub slot: Option<u16>,
}

impl Declaration {
    #[must_use]
    pub fn of(reference: &InputReference) -> Self {
        let family = Family::of(reference);
        let slot = match reference {
            InputReference::UpstreamSlot(slot) => Some(u16::try_from(*slot).unwrap_or(u16::MAX)),
            _ => None,
        };
        Self { family, slot }
    }

    /// Whether this declaration declares `channel`: the same family, and the
    /// same slot for `UpstreamSlot`.
    #[must_use]
    pub fn covers(self, channel: Channel) -> bool {
        self.family == channel.family && self.slot.is_none_or(|slot| slot == channel.channel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OrdinaryFoodTypeId;

    fn world(key: WorldInputKey, sub_idx: u16) -> Addressed {
        addressed(&InputReference::World(key), sub_idx)
    }

    #[test]
    fn world_compounds_wrap_and_scalars_read_channel_zero() {
        let ring = WorldInputKey::NeighborBarrierRing;
        assert_eq!(world(ring, 10), world(ring, 2));
        let area = WorldInputKey::area_food_summary(OrdinaryFoodTypeId::new(1));
        let Addressed::Channel(channel) = world(area, 9) else {
            panic!("a world compound always addresses a channel");
        };
        assert_eq!(channel.family, Family::AreaFoodSummary(1));
        assert_eq!(channel.channel, 2);
        let here = WorldInputKey::food_here(OrdinaryFoodTypeId::new(0));
        assert_eq!(world(here, 5), world(here, 0));
    }

    #[test]
    fn action_queue_keeps_the_raw_index_and_labels_past_the_draw_width() {
        let Addressed::Channel(inside) = addressed(&InputReference::ActionQueue, 11) else {
            panic!("ActionQueue always addresses a channel");
        };
        assert!(!inside.beyond_draw_width());
        let Addressed::Channel(beyond) = addressed(&InputReference::ActionQueue, 14) else {
            panic!("ActionQueue always addresses a channel");
        };
        assert_eq!(beyond.channel, 14);
        assert!(beyond.beyond_draw_width());
    }

    #[test]
    fn decision_compounds_read_nothing_at_or_past_their_width() {
        let width = sub_value_count(&InputReference::CommitCounts);
        assert!(matches!(
            addressed(&InputReference::CommitCounts, width - 1),
            Addressed::Channel(_)
        ));
        assert_eq!(
            addressed(&InputReference::CommitCounts, width),
            Addressed::OutOfWidth(Family::CommitCounts)
        );
    }

    #[test]
    fn upstream_declarations_cover_only_their_slot() {
        let declaration = Declaration::of(&InputReference::UpstreamSlot(3));
        let Addressed::Channel(three) = addressed(&InputReference::UpstreamSlot(3), 0) else {
            panic!("upstream slots address a channel");
        };
        let Addressed::Channel(four) = addressed(&InputReference::UpstreamSlot(4), 0) else {
            panic!("upstream slots address a channel");
        };
        assert!(declaration.covers(three));
        assert!(!declaration.covers(four));
        let ring = Declaration::of(&InputReference::World(WorldInputKey::NeighborOccupiedRing));
        let Addressed::Channel(occupied) = world(WorldInputKey::NeighborOccupiedRing, 7) else {
            panic!("world compounds address a channel");
        };
        assert!(ring.covers(occupied));
    }
}
