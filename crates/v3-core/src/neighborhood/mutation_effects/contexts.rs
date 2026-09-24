//! The `neighborhood-coverage-v1` contexts (T11.F26): recorded terminal-world
//! snapshots, authored contexts that set one channel group `neighborhood-v1`
//! leaves at zero, and long sequences over them. Any change to these rules
//! bumps [`super::COVERAGE_VERSION`].

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::sensors::perception::SensorSnapshot;
use crate::simulation::{assemble_full_sensor_inputs, Simulation};

use super::super::battery::Scenario;
use super::super::Battery;

/// Seed base of the recorded-context draw: `RECORDED_SEED_BASE + world_seed`.
pub const RECORDED_SEED_BASE: u64 = 23_000_000;
/// Seed base of authored context `i`: `AUTHORED_SEED_BASE + i`.
pub const AUTHORED_SEED_BASE: u64 = 25_000_000;
pub const AUTHORED_CONTEXTS: usize = 24;
pub const SEQUENCES: usize = 4;
pub const SEQUENCE_TICKS: usize = 32;
/// Sequence ticks `1..=EARLY_TICKS` report apart from the later ticks.
pub const EARLY_TICKS: usize = 4;
/// Authored context `i` sets group `i mod 8`; every group is zero in
/// `neighborhood-v1`.
pub const CHANNEL_GROUPS: [&str; 8] = [
    "neighbor barriers",
    "previous outcomes",
    "area food and typed area food",
    "area barrier",
    "area occupancy",
    "nearby core",
    "nearby vitals",
    "nearby identity",
];

/// A field's production range, drawn uniformly (binary fields from {0, 1}).
#[derive(Clone, Copy)]
enum Range {
    Unit,
    Signed,
    NonPositive,
    Binary,
}
use Range::{Binary, NonPositive, Signed, Unit};

const AREA_FOOD: [Range; 7] = [Unit, Signed, Signed, Signed, Signed, Unit, Unit];
const AREA_BARRIER: [Range; 7] = [Unit, Unit, Signed, Signed, Signed, Signed, Unit];
const AREA_OCCUPANCY: [Range; 7] = [Unit, Signed, Signed, Signed, Signed, Unit, Unit];
const CORE_SLOT: [Range; 4] = [Binary, Signed, Signed, Unit];
const VITALS_SLOT: [Range; 2] = [Unit, Binary];
const IDENTITY_SLOT: [Range; 3] = [Unit, Binary, Unit];
/// `PreviousOutcome` read ranges by `OutcomeChannel`: EnergyDelta [-1, 1],
/// ActionSuccess [0, 1], DamageDelta [-1, 0], OffspringSuccess {0, 1}.
const OUTCOMES: [Range; 4] = [Signed, Unit, NonPositive, Binary];

fn fill(values: &mut [f32], ranges: &[Range], rng: &mut SmallRng) {
    for (value, range) in values.iter_mut().zip(ranges.iter().cycle()) {
        *value = match range {
            Unit => rng.gen_range(0.0f32..=1.0),
            Signed => rng.gen_range(-1.0f32..=1.0),
            NonPositive => rng.gen_range(-1.0f32..=0.0),
            Binary => f32::from(u8::from(rng.gen_bool(0.5))),
        };
    }
}

fn set_group(sensors: &mut SensorSnapshot, group: usize, rng: &mut SmallRng) {
    let perception = &mut sensors.perception;
    match group {
        0 => fill(&mut sensors.local.neighbor_barrier, &[Binary], rng),
        1 => fill(&mut sensors.local.previous_outcome, &OUTCOMES, rng),
        2 => {
            fill(&mut perception.area_food, &AREA_FOOD, rng);
            for typed in &mut perception.typed_area_food {
                fill(typed, &AREA_FOOD, rng);
            }
        }
        3 => fill(&mut perception.area_barrier, &AREA_BARRIER, rng),
        4 => fill(&mut perception.area_occupancy, &AREA_OCCUPANCY, rng),
        5 => fill(&mut perception.nearby_core, &CORE_SLOT, rng),
        6 => fill(&mut perception.nearby_vitals, &VITALS_SLOT, rng),
        _ => fill(&mut perception.nearby_identity, &IDENTITY_SLOT, rng),
    }
}

/// The 24 authored contexts: context `i` copies `neighborhood-v1` snapshot
/// `i` (energy included) and sets channel group `i mod 8` from
/// `SmallRng::seed_from_u64(AUTHORED_SEED_BASE + i)`. Not guaranteed
/// realizable in any world.
pub(in crate::neighborhood) fn authored_contexts(battery: &Battery) -> Vec<Scenario> {
    battery
        .snapshots()
        .iter()
        .take(AUTHORED_CONTEXTS)
        .enumerate()
        .map(|(index, snapshot)| {
            let mut context = snapshot.clone();
            let mut rng = SmallRng::seed_from_u64(AUTHORED_SEED_BASE + index as u64);
            set_group(&mut context.sensors, index % CHANNEL_GROUPS.len(), &mut rng);
            context
        })
        .collect()
}

/// Positions into the id-sorted population the recorded group reads, in
/// draw order: a uniform draw without replacement of
/// `min(requested, population)` from
/// `SmallRng::seed_from_u64(RECORDED_SEED_BASE + world_seed)`.
#[must_use]
pub fn recorded_sample(population: usize, requested: usize, world_seed: u64) -> Vec<usize> {
    let amount = requested.min(population);
    if amount == 0 {
        return Vec::new();
    }
    let mut rng = SmallRng::seed_from_u64(RECORDED_SEED_BASE.wrapping_add(world_seed));
    rand::seq::index::sample(&mut rng, population, amount).into_vec()
}

/// The recorded contexts of a world's terminal population: each sampled
/// creature's complete production snapshot (typed local food and extended
/// perception assembled whatever the creature's genome reads) and its own
/// energy, in sample order.
pub(in crate::neighborhood) fn recorded_contexts(
    sim: &Simulation,
    world_seed: u64,
    requested: usize,
) -> Vec<Scenario> {
    let mut ids: Vec<_> = sim.creatures.keys().collect();
    ids.sort();
    let sampled: Vec<_> = recorded_sample(ids.len(), requested, world_seed)
        .into_iter()
        .map(|position| ids[position])
        .collect();
    assemble_full_sensor_inputs(sim, &sampled)
        .into_iter()
        .map(|(id, sensors)| Scenario {
            sensors,
            energy: sim.creatures[id].energy,
        })
        .collect()
}

/// The long sequences: sequence `s` tick `t` uses context `(8s + t) mod r`
/// of the recorded contexts, or of the authored contexts when fewer than
/// [`SEQUENCES`] contexts were recorded.
pub(in crate::neighborhood) fn sequences(
    recorded: &[Scenario],
    authored: &[Scenario],
) -> Vec<Vec<Scenario>> {
    let source = if recorded.len() < SEQUENCES {
        authored
    } else {
        recorded
    };
    (0..SEQUENCES)
        .map(|sequence| {
            (0..SEQUENCE_TICKS)
                .map(|tick| source[(8 * sequence + tick) % source.len()].clone())
                .collect()
        })
        .collect()
}

/// A reported group of coverage executions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Recorded,
    Authored,
    SequenceEarly,
    SequenceLate,
}

impl Group {
    pub const ALL: [Self; 4] = [
        Self::Recorded,
        Self::Authored,
        Self::SequenceEarly,
        Self::SequenceLate,
    ];

    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }

    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Recorded => "recorded",
            Self::Authored => "authored",
            Self::SequenceEarly => "sequence_ticks_1_4",
            Self::SequenceLate => "sequence_ticks_5_32",
        }
    }
}

/// One world's complete extension panel.
#[derive(Debug, Clone)]
pub(in crate::neighborhood) struct Extension {
    pub(in crate::neighborhood) recorded: Vec<Scenario>,
    pub(in crate::neighborhood) authored: Vec<Scenario>,
    pub(in crate::neighborhood) sequences: Vec<Vec<Scenario>>,
}

impl Extension {
    pub(in crate::neighborhood) fn new(recorded: Vec<Scenario>, battery: &Battery) -> Self {
        let authored = authored_contexts(battery);
        let sequences = sequences(&recorded, &authored);
        Self {
            recorded,
            authored,
            sequences,
        }
    }

    /// The single-tick contexts, recorded then authored.
    pub(in crate::neighborhood) fn singles(&self) -> Vec<Scenario> {
        self.recorded
            .iter()
            .chain(&self.authored)
            .cloned()
            .collect()
    }

    /// The group of execution `index` in `singles()`-then-sequences order.
    pub(in crate::neighborhood) fn group(&self, index: usize) -> Group {
        let recorded = self.recorded.len();
        let singles = recorded + self.authored.len();
        if index < recorded {
            Group::Recorded
        } else if index < singles {
            Group::Authored
        } else if (index - singles) % SEQUENCE_TICKS < EARLY_TICKS {
            Group::SequenceEarly
        } else {
            Group::SequenceLate
        }
    }
}
