use std::collections::BTreeSet;

use crate::contracts::OrdinaryFoodTypeId;

// ── Compound sub-field index constants ──────────────────────────────────────
// Per v3-sensor-spec.md Section 5. Prevents magic-number indexing in assembly.

/// `AreaFoodSummary` sub-field indices (width 7).
pub mod food_idx {
    pub const TOTAL_RATIO: usize = 0;
    pub const GRADIENT_X: usize = 1;
    pub const GRADIENT_Y: usize = 2;
    pub const NEAREST_DX: usize = 3;
    pub const NEAREST_DY: usize = 4;
    pub const NEAREST_DIST: usize = 5;
    pub const MAX_VALUE: usize = 6;
}

/// `AreaBarrierSummary` sub-field indices (width 7).
pub mod barrier_idx {
    pub const DENSITY_RATIO: usize = 0;
    pub const BLOCKED_ADJACENT_RATIO: usize = 1;
    pub const GRADIENT_X: usize = 2;
    pub const GRADIENT_Y: usize = 3;
    pub const NEAREST_DX: usize = 4;
    pub const NEAREST_DY: usize = 5;
    pub const NEAREST_DIST: usize = 6;
}

/// `AreaOccupancySummary` sub-field indices (width 7).
pub mod occupancy_idx {
    pub const COUNT_RATIO: usize = 0;
    pub const CENTER_X: usize = 1;
    pub const CENTER_Y: usize = 2;
    pub const NEAREST_DX: usize = 3;
    pub const NEAREST_DY: usize = 4;
    pub const NEAREST_DIST: usize = 5;
    pub const CROWDING_RATIO: usize = 6;
}

/// One `NearbyCreature` field's display name and description: the
/// declaration the frontend's `nearbyFields.json` fixture is pinned to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct NearbyField {
    pub name: &'static str,
    pub description: &'static str,
}

/// `NearbyCreature` layout (T25.F05, T27.F01): `SLOTS` ranked slots of
/// `FIELDS_PER_SLOT` fields; slot `k` field `f` is channel `k * FIELDS_PER_SLOT + f`.
pub mod nearby_idx {
    use super::NearbyField;

    pub const SLOTS: usize = 4;
    pub const FIELDS_PER_SLOT: usize = 10;
    /// The family's channel count.
    pub const WIDTH: usize = SLOTS * FIELDS_PER_SLOT;
    pub const PRESENT: usize = 0;
    pub const REL_X: usize = 1;
    pub const REL_Y: usize = 2;
    pub const DIST: usize = 3;
    pub const ENERGY_RATIO: usize = 4;
    pub const REPRODUCE_READY: usize = 5;
    pub const KIN_AFFINITY: usize = 6;
    /// The seen creature's colour, each channel over 255 (T27.F01).
    pub const RED: usize = 7;
    pub const GREEN: usize = 8;
    pub const BLUE: usize = 9;

    const fn field(name: &'static str, description: &'static str) -> NearbyField {
        NearbyField { name, description }
    }

    /// Each field's name and description, in layout order.
    pub const FIELDS: [NearbyField; FIELDS_PER_SLOT] = {
        let mut fields = [field("", ""); FIELDS_PER_SLOT];
        fields[PRESENT] = field(
            "present",
            "1 when this slot holds a creature, 0 when empty.",
        );
        fields[REL_X] = field("dx", "x offset, as a fraction of vision radius (+ = east).");
        fields[REL_Y] = field(
            "dy",
            "y offset, as a fraction of vision radius (+ = south).",
        );
        fields[DIST] = field("dist", "Distance (1 = edge of view).");
        fields[ENERGY_RATIO] = field("energy", "Energy as a fraction of max energy.");
        fields[REPRODUCE_READY] = field("ready", "1 when it has the energy and age to reproduce.");
        fields[KIN_AFFINITY] = field("kin", "Kin-tag similarity (1 = identical tags).");
        fields[RED] = field("red", "Red channel of its colour (0 to 1).");
        fields[GREEN] = field("green", "Green channel of its colour (0 to 1).");
        fields[BLUE] = field("blue", "Blue channel of its colour (0 to 1).");
        fields
    };
}

/// One `(x, y)` offset a world family senses toward a thing, and the thing's
/// plain facts (amount, distance, presence): what a `DirectionPair` edge
/// group reads and recruitment's pair case wires (T25.F04). Field indices of
/// the family's compound layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct DirectionPair {
    /// The `x` offset field (right positive).
    pub x: u16,
    /// The `y` offset field (down positive).
    pub y: u16,
    /// The same thing's plain facts, in wiring order.
    pub facts: &'static [u16],
}

// Field indices are `usize` for array indexing; the largest
// (`nearby_idx::WIDTH - 1`, 39) fits `u16`, so the casts are lossless.
const fn pair(x: usize, y: usize, facts: &'static [u16]) -> DirectionPair {
    DirectionPair {
        x: x as u16,
        y: y as u16,
        facts,
    }
}

const FOOD_FACTS: &[u16] = &[
    food_idx::TOTAL_RATIO as u16,
    food_idx::NEAREST_DIST as u16,
    food_idx::MAX_VALUE as u16,
];
const FOOD_PAIRS: &[DirectionPair] = &[
    pair(food_idx::GRADIENT_X, food_idx::GRADIENT_Y, FOOD_FACTS),
    pair(food_idx::NEAREST_DX, food_idx::NEAREST_DY, FOOD_FACTS),
];
const BARRIER_FACTS: &[u16] = &[
    barrier_idx::DENSITY_RATIO as u16,
    barrier_idx::BLOCKED_ADJACENT_RATIO as u16,
    barrier_idx::NEAREST_DIST as u16,
];
const BARRIER_PAIRS: &[DirectionPair] = &[
    pair(
        barrier_idx::GRADIENT_X,
        barrier_idx::GRADIENT_Y,
        BARRIER_FACTS,
    ),
    pair(
        barrier_idx::NEAREST_DX,
        barrier_idx::NEAREST_DY,
        BARRIER_FACTS,
    ),
];
const OCCUPANCY_FACTS: &[u16] = &[
    occupancy_idx::COUNT_RATIO as u16,
    occupancy_idx::NEAREST_DIST as u16,
    occupancy_idx::CROWDING_RATIO as u16,
];
const OCCUPANCY_PAIRS: &[DirectionPair] = &[
    pair(
        occupancy_idx::CENTER_X,
        occupancy_idx::CENTER_Y,
        OCCUPANCY_FACTS,
    ),
    pair(
        occupancy_idx::NEAREST_DX,
        occupancy_idx::NEAREST_DY,
        OCCUPANCY_FACTS,
    ),
];

/// A slot's facts: every field but the offset pair, in layout order.
const NEARBY_FACT_FIELDS: [usize; nearby_idx::FIELDS_PER_SLOT - 2] = [
    nearby_idx::PRESENT,
    nearby_idx::DIST,
    nearby_idx::ENERGY_RATIO,
    nearby_idx::REPRODUCE_READY,
    nearby_idx::KIN_AFFINITY,
    nearby_idx::RED,
    nearby_idx::GREEN,
    nearby_idx::BLUE,
];
const NEARBY_FACTS: [[u16; NEARBY_FACT_FIELDS.len()]; nearby_idx::SLOTS] = {
    let mut facts = [[0; NEARBY_FACT_FIELDS.len()]; nearby_idx::SLOTS];
    let mut k = 0;
    while k < nearby_idx::SLOTS {
        let mut f = 0;
        while f < NEARBY_FACT_FIELDS.len() {
            facts[k][f] = (k * nearby_idx::FIELDS_PER_SLOT + NEARBY_FACT_FIELDS[f]) as u16;
            f += 1;
        }
        k += 1;
    }
    facts
};
/// Slot `k`: `(rel_x, rel_y)`, facts the slot's eight other fields; the
/// four pairs partition the layout.
const NEARBY_PAIRS: [DirectionPair; nearby_idx::SLOTS] = {
    let mut pairs = [pair(0, 0, &[]); nearby_idx::SLOTS];
    let mut k = 0;
    while k < nearby_idx::SLOTS {
        let base = k * nearby_idx::FIELDS_PER_SLOT;
        pairs[k] = pair(
            base + nearby_idx::REL_X,
            base + nearby_idx::REL_Y,
            &NEARBY_FACTS[k],
        );
        k += 1;
    }
    pairs
};

/// Identifies a world-state spatial sensor input.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum WorldInputKey {
    /// Food density on current cell, normalized to [0.0, 1.0].
    FoodHere { type_idx: OrdinaryFoodTypeId },
    /// Compound: neighbor food ring (8 sub-values, indexed by Direction::to_index()).
    NeighborFoodRing { type_idx: OrdinaryFoodTypeId },
    /// Compound: neighbor barrier ring (8 sub-values, indexed by Direction::to_index()).
    NeighborBarrierRing,
    /// Compound: neighbor occupied ring (8 sub-values, indexed by Direction::to_index()).
    NeighborOccupiedRing,
    /// Compound: area food summary (7 sub-values). See v3-sensor-spec.md Section 5.1.
    AreaFoodSummary { type_idx: OrdinaryFoodTypeId },
    /// Compound: area barrier summary (7 sub-values). See v3-sensor-spec.md Section 5.2.
    AreaBarrierSummary,
    /// Compound: area occupancy summary (7 sub-values). See v3-sensor-spec.md Section 5.3.
    AreaOccupancySummary,
    /// Compound: the nearby creatures (4 slots × 10 fields = 40 sub-values,
    /// laid out by [`nearby_idx`]). See v3-sensor-spec.md Section 5.4.
    NearbyCreature,
}

impl WorldInputKey {
    /// Every world input key a world with these ordinary food types can
    /// present: the five unparameterized keys plus the three
    /// food-parameterized families, once per food type.
    #[must_use]
    pub fn universe(food_types: impl IntoIterator<Item = OrdinaryFoodTypeId>) -> BTreeSet<Self> {
        let mut keys = BTreeSet::from([
            Self::NeighborBarrierRing,
            Self::NeighborOccupiedRing,
            Self::AreaBarrierSummary,
            Self::AreaOccupancySummary,
            Self::NearbyCreature,
        ]);
        for type_idx in food_types {
            keys.insert(Self::food_here(type_idx));
            keys.insert(Self::neighbor_food_ring(type_idx));
            keys.insert(Self::area_food_summary(type_idx));
        }
        keys
    }

    /// Construct a typed food-here key for the given food type.
    #[must_use]
    pub const fn food_here(type_idx: OrdinaryFoodTypeId) -> Self {
        Self::FoodHere { type_idx }
    }

    /// Construct a typed neighbor-food-ring key for the given food type.
    #[must_use]
    pub const fn neighbor_food_ring(type_idx: OrdinaryFoodTypeId) -> Self {
        Self::NeighborFoodRing { type_idx }
    }

    /// Construct a typed area-food-summary key for the given food type.
    #[must_use]
    pub const fn area_food_summary(type_idx: OrdinaryFoodTypeId) -> Self {
        Self::AreaFoodSummary { type_idx }
    }

    /// Stable string key used by transport/API boundaries.
    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::FoodHere { .. } => "FoodHere",
            Self::NeighborFoodRing { .. } => "NeighborFoodRing",
            Self::NeighborBarrierRing => "NeighborBarrierRing",
            Self::NeighborOccupiedRing => "NeighborOccupiedRing",
            Self::AreaFoodSummary { .. } => "AreaFoodSummary",
            Self::AreaBarrierSummary => "AreaBarrierSummary",
            Self::AreaOccupancySummary => "AreaOccupancySummary",
            Self::NearbyCreature => "NearbyCreature",
        }
    }

    /// Whether this is one of the three neighbor rings: width 8, sub-value `c`
    /// the neighbor in `Direction::ALL[c]` (the shape `Ring` edge groups read).
    #[must_use]
    pub const fn is_neighbor_ring(self) -> bool {
        matches!(
            self,
            Self::NeighborFoodRing { .. } | Self::NeighborBarrierRing | Self::NeighborOccupiedRing
        )
    }

    /// The `(x, y)` offset pairs this family declares (T25.F04): two for each
    /// area summary, one per ranked slot of `NearbyCreature`, none for every
    /// other key. Depends on the variant only, never on a food type.
    #[must_use]
    pub const fn direction_pairs(self) -> &'static [DirectionPair] {
        match self {
            Self::AreaFoodSummary { .. } => FOOD_PAIRS,
            Self::AreaBarrierSummary => BARRIER_PAIRS,
            Self::AreaOccupancySummary => OCCUPANCY_PAIRS,
            Self::NearbyCreature => &NEARBY_PAIRS,
            _ => &[],
        }
    }

    /// Whether this family's pairs partition its layout, one pair per slot
    /// (T25.F05): every channel in exactly one pair's `{x, y} ∪ facts`, so
    /// a drawn pair names a whole slot. `NearbyCreature` only; pinned
    /// against [`Self::direction_pairs`] and [`Self::compound_width`].
    #[must_use]
    pub const fn slot_pairs(self) -> bool {
        matches!(self, Self::NearbyCreature)
    }

    /// Returns the typed food index for food-specific world keys, or `None` otherwise.
    #[must_use]
    pub const fn food_type_idx(self) -> Option<OrdinaryFoodTypeId> {
        match self {
            Self::FoodHere { type_idx }
            | Self::NeighborFoodRing { type_idx }
            | Self::AreaFoodSummary { type_idx } => Some(type_idx),
            _ => None,
        }
    }

    /// Number of sub-values for compound access. Returns 1 for scalar keys.
    ///
    /// `opt-inline-small`: Hot path — called per instruction per tick.
    /// `api-must-use`: Pure getter.
    #[inline]
    #[must_use]
    pub fn compound_width(&self) -> u16 {
        match self {
            Self::AreaFoodSummary { .. }
            | Self::AreaBarrierSummary
            | Self::AreaOccupancySummary => 7,
            Self::NeighborFoodRing { .. }
            | Self::NeighborBarrierRing
            | Self::NeighborOccupiedRing => 8,
            Self::NearbyCreature => nearby_idx::WIDTH as u16,
            _ => 1,
        }
    }
}

/// What a brain needs assembled before its turn: the typed local-food
/// snapshot, the extended perception snapshot (visibility, area reducers and
/// nearby banks), both or neither. A genome's value is the union of
/// [`Self::of_key`] over the world keys it references.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct PerceptionRequirements {
    /// `FoodHere` or `NeighborFoodRing` is referenced.
    pub typed_local_food: bool,
    /// An area summary or `NearbyCreature` is referenced.
    pub extended_perception: bool,
}

impl PerceptionRequirements {
    /// Nothing beyond the local snapshot.
    pub const NONE: Self = Self {
        typed_local_food: false,
        extended_perception: false,
    };

    /// Every snapshot, whatever a genome reads.
    pub const ALL: Self = Self {
        typed_local_food: true,
        extended_perception: true,
    };

    /// The key family classification: the snapshot a world key resolves
    /// through. The barrier and occupied rings read the local snapshot, so
    /// they require neither.
    #[must_use]
    pub const fn of_key(key: &WorldInputKey) -> Self {
        match key {
            WorldInputKey::FoodHere { .. } | WorldInputKey::NeighborFoodRing { .. } => Self {
                typed_local_food: true,
                extended_perception: false,
            },
            WorldInputKey::AreaFoodSummary { .. }
            | WorldInputKey::AreaBarrierSummary
            | WorldInputKey::AreaOccupancySummary
            | WorldInputKey::NearbyCreature => Self {
                typed_local_food: false,
                extended_perception: true,
            },
            WorldInputKey::NeighborBarrierRing | WorldInputKey::NeighborOccupiedRing => Self::NONE,
        }
    }

    /// Both requirement sets: a snapshot either needs.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self {
            typed_local_food: self.typed_local_food || other.typed_local_food,
            extended_perception: self.extended_perception || other.extended_perception,
        }
    }
}

/// Static introspection values assembled at tick start (snapshot).
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum StaticIntrospectionKey {
    /// Age as a saturating fraction of `age_reference_ticks`, in [0, 1].
    AgeTicks,
    /// This creature's own colour, red channel over 255, in [0, 1]: what a
    /// neighbour's `NearbyCreature` slot reads of it (T27.F01).
    OwnRed,
    /// Own colour, green channel over 255.
    OwnGreen,
    /// Own colour, blue channel over 255.
    OwnBlue,
}

/// Dynamic introspection values resolved live at read time (not snapshot).
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DynamicIntrospectionKey {
    /// Current energy level.
    EnergyCurrent,
    /// Energy spent since this tick's mesh run began (earlier dispatches and
    /// hop ramp charges), as a fraction of max energy. A VM read also counts
    /// its own in-flight step costs; a graph node reads the pre-dispatch value.
    EnergyConsumedThisTick,
    /// Mesh hops dispatched this tick, the one in flight included; a raw
    /// count (T19.F05).
    HopsThisTick,
}

/// A reference to a specific input source for a node input slot.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum InputReference {
    /// World state spatial sensor input.
    World(WorldInputKey),
    /// Static introspection value (snapshot at tick start).
    StaticIntrospection(StaticIntrospectionKey),
    /// Dynamic introspection value (resolved live).
    DynamicIntrospection(DynamicIntrospectionKey),
    /// Output slot from the upstream node in the mesh chain.
    UpstreamSlot(usize),
    /// Compound (T25.F11): every upstream output slot, sub-value `k` the
    /// slot `k`; width `OUTPUT_SLOT_COUNT`.
    UpstreamBus,
    /// Compound: the creature's action queue. sub_idx maps to queue slot values:
    /// `sub_idx / 3` = queue slot index, `sub_idx % 3`: 0 = action_type, 1 = param0, 2 = param1.
    ActionQueue,
    /// Compound (T19.F05): the current pass's vote vector, one sub-value per
    /// `VoteSink` index, resolved live.
    ActionVotes,
    /// Compound (T19.F05): the vote vector at the previous pass's end; zeros
    /// in the tick's first pass.
    PreviousPassVotes,
    /// Compound (T19.F05): the per-kind bars, one raw commit count per
    /// `VoteKind` index, resolved live.
    CommitCounts,
    /// Compound (T19.F05): the previous tick's outcome channels, one per
    /// `OutcomeChannel` discriminant, frozen at tick start.
    PreviousOutcome,
}

impl InputReference {
    /// Whether this reference is a neighbor ring a `Ring` edge group can read
    /// (T25.F03): `World` of one of [`WorldInputKey::is_neighbor_ring`].
    #[must_use]
    pub const fn is_ring(&self) -> bool {
        matches!(self, Self::World(key) if key.is_neighbor_ring())
    }

    /// The `(x, y)` pairs this reference declares (T25.F04): its world
    /// key's [`WorldInputKey::direction_pairs`], none for a non-world
    /// reference.
    #[must_use]
    pub const fn direction_pairs(&self) -> &'static [DirectionPair] {
        match self {
            Self::World(key) => key.direction_pairs(),
            _ => &[],
        }
    }

    /// Whether this reference's pairs are its slots (T25.F05): its world
    /// key's [`WorldInputKey::slot_pairs`], false for a non-world reference.
    #[must_use]
    pub const fn slot_pairs(&self) -> bool {
        matches!(self, Self::World(key) if key.slot_pairs())
    }
}

// ── Sensor descriptor (T25.F11) ─────────────────────────────────────────────

/// What a sensor's descriptor reads beyond the reference itself: the run's
/// ordinary food types and the action-queue slots a turn can hold. Built
/// once per run from the configuration (`SimulationConfig::sensor_context`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SensorContext {
    /// Ordinary food types in the world (at least 1 in a valid config).
    pub food_type_count: usize,
    /// The normalized `max_actions_per_turn`.
    pub action_queue_slots: u32,
}

impl SensorContext {
    #[must_use]
    pub const fn new(food_type_count: usize, action_queue_slots: u32) -> Self {
        Self {
            food_type_count,
            action_queue_slots,
        }
    }
}

/// `ActionQueue`'s declared width: the first four queued actions, three
/// sub-values each, whatever the configured queue.
const ACTION_QUEUE_WIDTH: u32 = 12;

/// One field set of a sensor or a shared-memory bank: the fields one bundle
/// wires and the geometry they carry. Field indices are the sensor's
/// sub-values (a memory bank's slots).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldSet {
    /// Fields `0..width`, no geometry.
    Plain { width: u32 },
    /// A neighbor ring: field `c` the neighbor in `Direction::ALL[c]`, eight.
    Ring,
    /// An `(x, y)` offset pair and the same thing's facts.
    Pair(DirectionPair),
}

/// A shared-memory bank's one field set: every slot, no geometry. A bank is
/// a bundle source without a declaration.
pub const MEMORY_BANK: FieldSet = FieldSet::Plain {
    width: crate::contracts::SHARED_MEMORY_SLOTS as u32,
};

/// The fields of a `Ring` set.
const RING_FIELDS: u16 = 8;

impl FieldSet {
    /// The set's fields, ascending.
    #[must_use]
    pub fn fields(self) -> Vec<u16> {
        match self {
            Self::Plain { width } => (0..width)
                .map(|field| u16::try_from(field).expect("a sub-value index fits u16"))
                .collect(),
            Self::Ring => (0..RING_FIELDS).collect(),
            Self::Pair(pair) => {
                let mut fields: Vec<u16> = [pair.x, pair.y]
                    .into_iter()
                    .chain(pair.facts.iter().copied())
                    .collect();
                fields.sort_unstable();
                fields
            }
        }
    }

    /// Whether `field` belongs to the set.
    #[must_use]
    pub fn contains(self, field: u16) -> bool {
        match self {
            Self::Plain { width } => u32::from(field) < width,
            Self::Ring => field < RING_FIELDS,
            Self::Pair(pair) => field == pair.x || field == pair.y || pair.facts.contains(&field),
        }
    }
}

/// How a sensor's fields split into field sets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Layout {
    /// One set of every field, no geometry.
    Plain,
    /// One `Ring` set.
    Ring,
    /// One `Pair` set per declared pair.
    Pairs(&'static [DirectionPair]),
}

/// A sensor's width and field sets (T25.F11): the one declaration every
/// width reader and every wiring reads. Never names an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SensorDescriptor {
    width: u32,
    layout: Layout,
}

impl SensorDescriptor {
    /// The descriptor of `reference` under `context`.
    #[must_use]
    pub fn of(reference: &InputReference, context: &SensorContext) -> Self {
        let _ = context;
        let width = match reference {
            InputReference::World(key) => u32::from(key.compound_width()),
            InputReference::ActionQueue => ACTION_QUEUE_WIDTH,
            InputReference::ActionVotes | InputReference::PreviousPassVotes => {
                crate::contracts::votes::VOTE_SINK_COUNT as u32
            }
            InputReference::CommitCounts => crate::contracts::votes::VOTE_KIND_COUNT as u32,
            InputReference::PreviousOutcome => crate::contracts::OUTCOME_CHANNEL_COUNT as u32,
            InputReference::UpstreamBus => crate::contracts::OUTPUT_SLOT_COUNT as u32,
            InputReference::StaticIntrospection(_)
            | InputReference::DynamicIntrospection(_)
            | InputReference::UpstreamSlot(_) => 1,
        };
        let layout = match reference {
            InputReference::World(key) if key.is_neighbor_ring() => Layout::Ring,
            InputReference::World(key) if !key.direction_pairs().is_empty() => {
                Layout::Pairs(key.direction_pairs())
            }
            _ => Layout::Plain,
        };
        Self { width, layout }
    }

    /// The number of sub-values the sensor exposes.
    #[must_use]
    pub const fn width(self) -> u32 {
        self.width
    }

    /// How many field sets the sensor declares (at least one).
    #[must_use]
    pub const fn field_set_count(self) -> usize {
        match self.layout {
            Layout::Plain | Layout::Ring => 1,
            Layout::Pairs(pairs) => pairs.len(),
        }
    }

    /// Field set `index`, when declared.
    #[must_use]
    pub fn field_set(self, index: u8) -> Option<FieldSet> {
        let index = usize::from(index);
        match self.layout {
            Layout::Plain => (index == 0).then_some(FieldSet::Plain { width: self.width }),
            Layout::Ring => (index == 0).then_some(FieldSet::Ring),
            Layout::Pairs(pairs) => pairs.get(index).copied().map(FieldSet::Pair),
        }
    }

    /// Every field set, in index order.
    pub fn field_sets(self) -> impl Iterator<Item = FieldSet> {
        (0..self.field_set_count()).map(move |index| {
            self.field_set(u8::try_from(index).expect("few field sets"))
                .expect("an index below the count")
        })
    }
}

/// Draws a world input key from [`WorldInputKey::universe`] over food types
/// 0..3, so property tests track the key families without a hand-kept list.
#[cfg(test)]
pub(crate) fn world_key_strategy() -> impl proptest::strategy::Strategy<Value = WorldInputKey> {
    let keys: Vec<_> = WorldInputKey::universe((0..3).map(OrdinaryFoodTypeId::new))
        .into_iter()
        .collect();
    proptest::sample::select(keys)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::OrdinaryFoodTypeId;

    fn every_world_key() -> Vec<WorldInputKey> {
        let food = (0..3).map(OrdinaryFoodTypeId::new);
        WorldInputKey::universe(food).into_iter().collect()
    }

    const CONTEXT: SensorContext = SensorContext::new(1, 10);

    /// Every field set's fields lie below the sensor's width, and the sets
    /// cover every field: a slot family's sets partition it (each slot of
    /// `NearbyCreature` one set), an area summary's two pairs share the
    /// family's facts, a ring is one `Ring` set and every other sensor one
    /// `Plain` set of all its fields (T25.F11 invariant 1).
    #[test]
    fn the_descriptor_field_sets_cover_each_sensor() {
        let world = every_world_key().into_iter().map(InputReference::World);
        let others = [
            InputReference::ActionQueue,
            InputReference::ActionVotes,
            InputReference::PreviousPassVotes,
            InputReference::CommitCounts,
            InputReference::PreviousOutcome,
            InputReference::UpstreamBus,
            InputReference::UpstreamSlot(3),
            InputReference::StaticIntrospection(StaticIntrospectionKey::AgeTicks),
        ];
        for reference in world.chain(others) {
            let descriptor = SensorDescriptor::of(&reference, &CONTEXT);
            let mut covered = vec![0u32; descriptor.width() as usize];
            for set in descriptor.field_sets() {
                for field in set.fields() {
                    assert!(set.contains(field));
                    covered[usize::from(field)] += 1;
                }
            }
            assert!(
                covered.iter().all(|&n| n >= 1),
                "{reference:?} covers every field"
            );
            let shared = matches!(
                reference,
                InputReference::World(
                    WorldInputKey::AreaFoodSummary { .. }
                        | WorldInputKey::AreaBarrierSummary
                        | WorldInputKey::AreaOccupancySummary
                )
            );
            if !shared {
                assert!(covered.iter().all(|&n| n == 1), "{reference:?} partitions");
            }
            let expected_sets = match &reference {
                InputReference::World(WorldInputKey::NearbyCreature) => 4,
                InputReference::World(key) if !key.direction_pairs().is_empty() => 2,
                _ => 1,
            };
            assert_eq!(descriptor.field_set_count(), expected_sets, "{reference:?}");
            assert_eq!(
                descriptor.field_set(u8::try_from(expected_sets).unwrap()),
                None
            );
        }
    }

    /// The non-world widths keep their values; `UpstreamBus` is every
    /// upstream slot; a ring's one set is `Ring` (T25.F11 invariant 1).
    #[test]
    fn the_descriptor_widths_follow_the_brain_io_catalog() {
        let width = |reference: InputReference| SensorDescriptor::of(&reference, &CONTEXT).width();
        assert_eq!(width(InputReference::ActionVotes), 39);
        assert_eq!(width(InputReference::PreviousPassVotes), 39);
        assert_eq!(width(InputReference::CommitCounts), 5);
        assert_eq!(width(InputReference::PreviousOutcome), 4);
        assert_eq!(width(InputReference::UpstreamBus), 24);
        assert_eq!(width(InputReference::UpstreamSlot(9)), 1);
        assert_eq!(width(InputReference::ActionQueue), 12);
        let ring = SensorDescriptor::of(
            &InputReference::World(WorldInputKey::NeighborBarrierRing),
            &CONTEXT,
        );
        assert_eq!(ring.field_set(0), Some(FieldSet::Ring));
        assert_eq!(MEMORY_BANK.fields().len(), 16);
    }

    #[test]
    fn every_declared_pair_has_distinct_fields_below_the_width() {
        for key in every_world_key() {
            for pair in key.direction_pairs() {
                let mut fields = vec![pair.x, pair.y];
                fields.extend_from_slice(pair.facts);
                assert!(fields.iter().all(|&f| f < key.compound_width()), "{key:?}");
                let mut sorted = fields.clone();
                sorted.sort_unstable();
                sorted.dedup();
                assert_eq!(sorted.len(), fields.len(), "{key:?}: {pair:?}");
            }
        }
    }

    #[test]
    fn the_four_layouts_declare_their_pairs_and_facts() {
        let food = WorldInputKey::area_food_summary(OrdinaryFoodTypeId::new(2));
        let layouts = [
            (food, vec![(1, 2, vec![0, 5, 6]), (3, 4, vec![0, 5, 6])]),
            (
                WorldInputKey::AreaBarrierSummary,
                vec![(2, 3, vec![0, 1, 6]), (4, 5, vec![0, 1, 6])],
            ),
            (
                WorldInputKey::AreaOccupancySummary,
                vec![(1, 2, vec![0, 5, 6]), (3, 4, vec![0, 5, 6])],
            ),
            (
                WorldInputKey::NearbyCreature,
                (0..4u16)
                    .map(|k| {
                        let base = 10 * k;
                        let facts = [0, 3, 4, 5, 6, 7, 8, 9].map(|f| base + f).to_vec();
                        (base + 1, base + 2, facts)
                    })
                    .collect(),
            ),
        ];
        for (key, want) in layouts {
            let got: Vec<_> = key
                .direction_pairs()
                .iter()
                .map(|pair| (pair.x, pair.y, pair.facts.to_vec()))
                .collect();
            assert_eq!(got, want, "{key:?}");
        }
        let declaring = every_world_key()
            .into_iter()
            .filter(|key| !key.direction_pairs().is_empty())
            .map(WorldInputKey::as_key)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            declaring,
            [
                "AreaBarrierSummary",
                "AreaFoodSummary",
                "AreaOccupancySummary",
                "NearbyCreature"
            ]
            .into()
        );
        assert!(InputReference::ActionVotes.direction_pairs().is_empty());
        assert_eq!(
            InputReference::World(food).direction_pairs(),
            food.direction_pairs()
        );
    }

    /// `slot_pairs` is true exactly where the pairs partition the layout:
    /// every channel in exactly one pair's `{x, y} ∪ facts` (T25.F05).
    #[test]
    fn slot_pairs_holds_exactly_where_the_pairs_partition_the_layout() {
        let mut slotted = Vec::new();
        for key in every_world_key() {
            let mut covered = vec![0u32; usize::from(key.compound_width())];
            for pair in key.direction_pairs() {
                for &field in [pair.x, pair.y].iter().chain(pair.facts) {
                    covered[usize::from(field)] += 1;
                }
            }
            let partitions =
                !key.direction_pairs().is_empty() && covered.iter().all(|&count| count == 1);
            assert_eq!(key.slot_pairs(), partitions, "{key:?}");
            if key.slot_pairs() {
                slotted.push(key.as_key());
            }
        }
        assert_eq!(slotted, ["NearbyCreature"]);
        assert_eq!(
            WorldInputKey::NearbyCreature.direction_pairs().len(),
            nearby_idx::SLOTS
        );
    }

    #[test]
    fn the_moved_index_modules_match_the_compound_widths() {
        let area = [
            food_idx::MAX_VALUE,
            barrier_idx::NEAREST_DIST,
            occupancy_idx::CROWDING_RATIO,
        ];
        assert!(area.iter().all(|&last| last + 1 == 7));
        assert_eq!(nearby_idx::BLUE + 1, nearby_idx::FIELDS_PER_SLOT);
        assert_eq!(WorldInputKey::NearbyCreature.compound_width(), 40);
    }

    #[test]
    fn every_swap_alternative_keeps_the_pairs() {
        for food_types in 1..=3 {
            for key in every_world_key() {
                let reference = InputReference::World(key);
                for alternative in crate::mutation::input_ref::swap_alternatives(
                    &reference,
                    &crate::config::test_sensor_context(food_types),
                ) {
                    assert_eq!(
                        alternative.direction_pairs(),
                        reference.direction_pairs(),
                        "{reference:?} -> {alternative:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_frontend_fixture_equals_the_declared_pairs() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../frontend/src/components/inspector/directionPairs.json"
        ))
        .expect("fixture parses");
        let mut declared = serde_json::Map::new();
        for key in every_world_key() {
            if !key.direction_pairs().is_empty() {
                declared.insert(
                    key.as_key().to_owned(),
                    serde_json::to_value(key.direction_pairs()).expect("serializes"),
                );
            }
        }
        assert_eq!(fixture, serde_json::Value::Object(declared));
    }

    /// The frontend's `nearbyFields.json` is the declared field table
    /// (T27.F01 invariant 6), as `directionPairs.json` is the pairs.
    #[test]
    fn the_frontend_fixture_equals_the_declared_nearby_fields() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../frontend/src/components/inspector/nearbyFields.json"
        ))
        .expect("fixture parses");
        let declared = serde_json::json!({
            "slots": nearby_idx::SLOTS,
            "fields": nearby_idx::FIELDS,
        });
        assert_eq!(fixture, declared);
        let names: Vec<_> = nearby_idx::FIELDS.iter().map(|f| f.name).collect();
        assert_eq!(
            names,
            ["present", "dx", "dy", "dist", "energy", "ready", "kin", "red", "green", "blue"]
        );
    }

    #[test]
    fn world_input_keys_constructible() {
        let _food = WorldInputKey::FoodHere {
            type_idx: OrdinaryFoodTypeId::default(),
        };
        let _nfood = WorldInputKey::NeighborFoodRing {
            type_idx: OrdinaryFoodTypeId::default(),
        };
        let _barrier = WorldInputKey::NeighborBarrierRing;
        let _occ = WorldInputKey::NeighborOccupiedRing;
    }

    #[test]
    fn input_reference_upstream_slot() {
        let r = InputReference::UpstreamSlot(3);
        if let InputReference::UpstreamSlot(idx) = r {
            assert_eq!(idx, 3);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn input_reference_serde_roundtrip() {
        let refs = vec![
            InputReference::World(WorldInputKey::FoodHere {
                type_idx: OrdinaryFoodTypeId::default(),
            }),
            InputReference::World(WorldInputKey::AreaFoodSummary {
                type_idx: OrdinaryFoodTypeId::default(),
            }),
            InputReference::World(WorldInputKey::AreaBarrierSummary),
            InputReference::World(WorldInputKey::AreaOccupancySummary),
            InputReference::World(WorldInputKey::NearbyCreature),
            InputReference::World(WorldInputKey::NeighborFoodRing {
                type_idx: OrdinaryFoodTypeId::default(),
            }),
            InputReference::World(WorldInputKey::NeighborBarrierRing),
            InputReference::World(WorldInputKey::NeighborOccupiedRing),
            InputReference::StaticIntrospection(StaticIntrospectionKey::AgeTicks),
            InputReference::StaticIntrospection(StaticIntrospectionKey::OwnRed),
            InputReference::StaticIntrospection(StaticIntrospectionKey::OwnGreen),
            InputReference::StaticIntrospection(StaticIntrospectionKey::OwnBlue),
            InputReference::DynamicIntrospection(DynamicIntrospectionKey::EnergyCurrent),
            InputReference::DynamicIntrospection(DynamicIntrospectionKey::HopsThisTick),
            InputReference::UpstreamSlot(0),
            InputReference::ActionQueue,
            InputReference::ActionVotes,
            InputReference::PreviousPassVotes,
            InputReference::CommitCounts,
            InputReference::PreviousOutcome,
        ];
        for r in refs {
            let json = serde_json::to_string(&r).unwrap();
            let r2: InputReference = serde_json::from_str(&json).unwrap();
            assert_eq!(r, r2);
        }
    }

    #[test]
    fn ring_sensor_compound_widths() {
        assert_eq!(
            WorldInputKey::NeighborFoodRing {
                type_idx: OrdinaryFoodTypeId::default(),
            }
            .compound_width(),
            8
        );
        assert_eq!(WorldInputKey::NeighborBarrierRing.compound_width(), 8);
        assert_eq!(WorldInputKey::NeighborOccupiedRing.compound_width(), 8);
        assert_eq!(
            WorldInputKey::FoodHere {
                type_idx: OrdinaryFoodTypeId::default(),
            }
            .compound_width(),
            1
        );
    }

    #[test]
    fn typed_food_keys_have_stable_keys_and_widths() {
        let here = WorldInputKey::FoodHere {
            type_idx: OrdinaryFoodTypeId::default(),
        };
        let ring = WorldInputKey::NeighborFoodRing {
            type_idx: OrdinaryFoodTypeId::default(),
        };
        let area = WorldInputKey::AreaFoodSummary {
            type_idx: OrdinaryFoodTypeId::default(),
        };

        assert_eq!(here.as_key(), "FoodHere");
        assert_eq!(ring.as_key(), "NeighborFoodRing");
        assert_eq!(area.as_key(), "AreaFoodSummary");
        assert_eq!(here.compound_width(), 1);
        assert_eq!(ring.compound_width(), 8);
        assert_eq!(area.compound_width(), 7);
    }

    #[test]
    fn each_key_family_requires_the_snapshot_it_resolves_through() {
        let food = OrdinaryFoodTypeId::default();
        let typed = PerceptionRequirements {
            typed_local_food: true,
            extended_perception: false,
        };
        let extended = PerceptionRequirements {
            typed_local_food: false,
            extended_perception: true,
        };
        let cases = [
            (WorldInputKey::food_here(food), typed),
            (WorldInputKey::neighbor_food_ring(food), typed),
            (WorldInputKey::area_food_summary(food), extended),
            (WorldInputKey::AreaBarrierSummary, extended),
            (WorldInputKey::AreaOccupancySummary, extended),
            (WorldInputKey::NearbyCreature, extended),
            (
                WorldInputKey::NeighborBarrierRing,
                PerceptionRequirements::NONE,
            ),
            (
                WorldInputKey::NeighborOccupiedRing,
                PerceptionRequirements::NONE,
            ),
        ];
        for (key, expected) in cases {
            assert_eq!(PerceptionRequirements::of_key(&key), expected, "{key:?}");
        }
        assert_eq!(typed.union(extended), PerceptionRequirements::ALL);
    }

    fn requirements() -> impl proptest::strategy::Strategy<Value = PerceptionRequirements> {
        use proptest::prelude::*;
        (any::<bool>(), any::<bool>()).prop_map(|(typed_local_food, extended_perception)| {
            PerceptionRequirements {
                typed_local_food,
                extended_perception,
            }
        })
    }

    proptest::proptest! {
        /// Union is a join: commutative, idempotent, `NONE` its identity and
        /// `ALL` absorbing; no key requires both snapshots.
        #[test]
        fn requirements_union_is_a_join(a in requirements(), b in requirements()) {
            use PerceptionRequirements as R;
            proptest::prop_assert_eq!(a.union(b), b.union(a));
            proptest::prop_assert_eq!(a.union(a), a);
            proptest::prop_assert_eq!(a.union(R::NONE), a);
            proptest::prop_assert_eq!(a.union(R::ALL), R::ALL);
            for key in every_world_key() {
                let r = R::of_key(&key);
                proptest::prop_assert!(!(r.typed_local_food && r.extended_perception));
            }
        }
    }
}
