//! One birth transaction (T29.F01): every creature, founder or child, is
//! built from a [`Zygote`] (its genome, body chromosome included, its birth
//! payload, its provenance and the birth's inheritance mapping) by one finalization,
//! [`Zygote::finalize`], the only caller of the creature constructor.
//!
//! An egg is a nucleus and its cytoplasm, assembled once: conception clones
//! the parent's genome, captures its learned values and memory, mutates both
//! chromosomes, clamps the body to the ranges in force and records the
//! provenance; finalization carries the
//! learned values across the birth's edits, builds the derived state and the
//! runtime, zeroes per-life state, sets the heading and provisions energy
//! within the body's capacity. World placement, the action log, the parent's
//! counters and every flow booking stay in the simulation.

use rand::Rng;

use crate::config::SimulationConfig;
use crate::contracts::{CreatureId, Direction, Position, SHARED_MEMORY_SLOTS};
use crate::creature::body::body_ranges;
use crate::creature::state::CreatureState;
use crate::genome::learning::{InheritanceMapping, Seeds};
use crate::genome::CreatureGenome;
use crate::mutation::reachability::ParentExecuted;
use crate::mutation::{BirthEdgeClass, MutationEngine, MutationOperator, MutationSummary};

/// What a zygote carries beside its genome (T29.F01 invariant 2): the
/// parent's learned values, keyed by learnable identity, and its shared
/// memory at delivery. A founder's is empty and zeroed.
#[derive(Debug, Clone, PartialEq)]
pub struct BirthPayload {
    seeds: Seeds,
    shared_memory: [f32; SHARED_MEMORY_SLOTS],
}

impl Default for BirthPayload {
    fn default() -> Self {
        Self::new(Seeds::default(), [0.0; SHARED_MEMORY_SLOTS])
    }
}

impl BirthPayload {
    /// A payload of `seeds` and `shared_memory`.
    #[must_use]
    pub const fn new(seeds: Seeds, shared_memory: [f32; SHARED_MEMORY_SLOTS]) -> Self {
        Self {
            seeds,
            shared_memory,
        }
    }

    /// The learned values carried, keyed by the parent's learnable
    /// identities.
    #[must_use]
    pub const fn seeds(&self) -> &Seeds {
        &self.seeds
    }

    /// The shared memory carried.
    #[must_use]
    pub const fn shared_memory(&self) -> &[f32; SHARED_MEMORY_SLOTS] {
        &self.shared_memory
    }
}

/// Where a creature comes from (T29.F01 invariant 3): its lineage, its
/// generation and its birth record, derived from the birth's mutation
/// summary; a founder's is its own lineage at generation 0 with an empty
/// record.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Provenance {
    /// The founder this creature descends from (T29.F02 invariant 6): a
    /// founder's index, a child's parent's. Observation only: no rule,
    /// sensor or draw reads it.
    pub lineage_id: u32,
    pub generation: u64,
    /// The operators applied at this birth, sorted by key.
    pub birth_mutation_operators: Box<[MutationOperator]>,
    /// The class of the vote sink this birth's sole edge push landed on
    /// (T11.F30).
    pub birth_edge_class: Option<BirthEdgeClass>,
}

impl Provenance {
    /// A child of `parent` whose birth `summary` reports.
    #[must_use]
    fn of_child(parent: &CreatureState, summary: &MutationSummary) -> Self {
        let mut operators: Vec<_> = summary.applied_by_operator.keys().copied().collect();
        operators.sort_by_key(|operator| operator.as_key());
        Self {
            lineage_id: parent.lineage_id,
            generation: parent.generation + 1,
            birth_mutation_operators: operators.into_boxed_slice(),
            birth_edge_class: summary.birth_edge_class(),
        }
    }
}

/// Where and how a zygote is born: its cell, its heading and the energy it
/// is given.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BirthPlace {
    pub position: Position,
    pub heading: Direction,
    pub energy: f32,
}

/// A creature before its birth (T29.F01 invariant 1).
#[derive(Debug, Clone)]
pub struct Zygote {
    genome: CreatureGenome,
    payload: BirthPayload,
    provenance: Provenance,
    /// Where every learnable of `genome` inherits from among the payload's.
    inheritance: InheritanceMapping,
}

/// A finalized birth: the creature and the energy its body could not store.
#[must_use = "a birth's capacity overflow is booked or deliberately dropped"]
pub struct Newborn {
    pub creature: CreatureState,
    /// The given energy above the body's capacity, `applied_debit`-exact.
    pub overflow: f64,
}

impl Zygote {
    /// A founder of lineage `lineage_id` (its founder index): `genome`, an
    /// empty payload, generation 0.
    #[must_use]
    pub fn founder(genome: CreatureGenome, lineage_id: u32) -> Self {
        Self {
            genome,
            payload: BirthPayload::default(),
            provenance: Provenance {
                lineage_id,
                ..Provenance::default()
            },
            inheritance: InheritanceMapping::Identity,
        }
    }

    /// Conceive a child of `parent` under `config` (T29.F01 invariant 9):
    /// clone the parent's genome, capture the payload, mutate both
    /// chromosomes under the config's body ranges (the parent's reachable
    /// nodes and dispatch record as its `ParentExecuted`), clamp the body's
    /// scalars to the same ranges, so a live patch of a bound reaches the
    /// next generation without a draw (T29.F02 invariant 5), derive the
    /// provenance. Returns the zygote beside the
    /// birth's mutation summary, which the simulation folds into its stats.
    pub(crate) fn conceive(
        parent: &CreatureState,
        config: &SimulationConfig,
        rng: &mut impl Rng,
    ) -> (Self, MutationSummary) {
        let mut genome = parent.genome().clone();
        let payload = BirthPayload::new(
            parent.graph_runtime().learned().capture(),
            parent.shared_memory,
        );
        // T11.F17: the parent's own recent dispatches bias the offspring's
        // targets. Passed unresolved, so a zero-event birth derives nothing.
        let executed_within = |window| {
            parent
                .graph_runtime()
                .dispatch_record
                .executed_indices(parent.age, window)
        };
        let ranges = body_ranges(config);
        let (summary, inheritance) = MutationEngine::apply_mutations_in(
            &mut genome,
            &config.mutation,
            &ranges,
            parent.reachable_nodes(),
            ParentExecuted::Query(&executed_within),
            rng,
            &config.sensor_context(config.world.food.types.len()),
        );
        genome.body_mut().clamp(&ranges);
        let provenance = Provenance::of_child(parent, &summary);
        (
            Self {
                genome,
                payload,
                provenance,
                inheritance,
            },
            summary,
        )
    }

    /// This zygote carrying `payload` instead of its own: a fixture's
    /// seeded learners and memory. Seeding and delivery never call it.
    #[must_use]
    pub fn with_payload(self, payload: BirthPayload) -> Self {
        Self { payload, ..self }
    }

    /// The genome the creature will be built from.
    #[must_use]
    pub const fn genome(&self) -> &CreatureGenome {
        &self.genome
    }

    /// The one finalization (T29.F01 invariant 10): the payload's seeds
    /// carried across the birth's edits by its mapping, the derived state,
    /// the learned state and the runtime built through the one constructor,
    /// the payload's memory taken, every per-life state zeroed, the heading
    /// set and the given energy provisioned up to the body's capacity, the
    /// overflow returned.
    pub fn finalize(self, id: CreatureId, place: BirthPlace) -> Newborn {
        let Self {
            genome,
            payload,
            provenance,
            inheritance,
        } = self;
        let seeds = inheritance.apply(&payload.seeds);
        let mut creature = CreatureState::new(
            id,
            genome,
            place.position,
            place.energy,
            payload.shared_memory,
            &seeds,
        );
        creature.lineage_id = provenance.lineage_id;
        creature.generation = provenance.generation;
        creature.birth_mutation_operators = provenance.birth_mutation_operators;
        creature.birth_edge_class = provenance.birth_edge_class;
        creature.heading = place.heading;
        let overflow = creature.cap_energy();
        Newborn { creature, overflow }
    }
}

#[cfg(test)]
impl Zygote {
    /// A fixture's creature: this zygote finalized at `position`, facing
    /// north, given `energy`, its overflow dropped.
    pub(crate) fn born_at(self, id: CreatureId, position: Position, energy: f32) -> CreatureState {
        self.finalize(
            id,
            BirthPlace {
                position,
                heading: Direction::N,
                energy,
            },
        )
        .creature
    }

    /// A fixture's `creature` born again from its genome carrying the body
    /// `edit` makes of its own (T29.F02 invariant 10: a body changes only by
    /// a birth): the same id, lineage, cell and heading, holding the same
    /// energy, uncapped.
    pub(crate) fn reborn(
        creature: &CreatureState,
        edit: impl FnOnce(crate::genome::body::BodyGenome) -> crate::genome::body::BodyGenome,
    ) -> CreatureState {
        let genome = creature.genome().clone();
        let body = edit(*genome.body());
        let mut reborn = Self::founder(genome.with_body(body), creature.lineage_id)
            .finalize(
                creature.id,
                BirthPlace {
                    position: creature.position,
                    heading: creature.heading,
                    energy: creature.energy,
                },
            )
            .creature;
        reborn.energy = creature.energy;
        reborn
    }
}

#[cfg(test)]
#[path = "birth_tests.rs"]
mod tests;
