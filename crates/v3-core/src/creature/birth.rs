//! One birth transaction (T29.F01): every creature, founder or child, is
//! built from a [`Zygote`] (its genome, its body, its birth payload, its
//! provenance and the birth's inheritance mapping) by one finalization,
//! [`Zygote::finalize`], the only caller of the creature constructor.
//!
//! An egg is a nucleus and its cytoplasm, assembled once: conception clones
//! the parent's genome, captures its learned values and memory, mutates,
//! draws the body and records the provenance; finalization carries the
//! learned values across the birth's edits, builds the derived state and the
//! runtime, zeroes per-life state, sets the heading and provisions energy
//! within the body's capacity. World placement, the action log, the parent's
//! counters and every flow booking stay in the simulation.

use rand::Rng;

use crate::config::SimulationConfig;
use crate::contracts::{CreatureId, Direction, Position, SHARED_MEMORY_SLOTS};
use crate::creature::body::HeritableBody;
use crate::creature::state::CreatureState;
use crate::genome::learning::{InheritanceMapping, Seeds};
use crate::genome::CreatureGenome;
use crate::mutation::body::sample_body_draws;
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

/// Where a creature comes from (T29.F01 invariant 3): its generation and its
/// birth record, derived from the birth's mutation summary; a founder's is
/// generation 0 with an empty record.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Provenance {
    pub generation: u64,
    /// The operators applied at this birth, sorted by key.
    pub birth_mutation_operators: Box<[MutationOperator]>,
    /// The class of the vote sink this birth's sole edge push landed on
    /// (T11.F30).
    pub birth_edge_class: Option<BirthEdgeClass>,
}

impl Provenance {
    /// A child of a parent of `parent_generation` whose birth `summary`
    /// reports.
    #[must_use]
    fn of_child(parent_generation: u64, summary: &MutationSummary) -> Self {
        let mut operators: Vec<_> = summary.applied_by_operator.keys().copied().collect();
        operators.sort_by_key(|operator| operator.as_key());
        Self {
            generation: parent_generation + 1,
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
    /// The body until T29.F02 moves it into the genome.
    body: HeritableBody,
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
    /// A founder: `genome` and `body`, an empty payload, generation 0.
    #[must_use]
    pub fn founder(genome: CreatureGenome, body: HeritableBody) -> Self {
        Self {
            genome,
            body,
            payload: BirthPayload::default(),
            provenance: Provenance::default(),
            inheritance: InheritanceMapping::Identity,
        }
    }

    /// Conceive a child of `parent` under `config` (T29.F01 invariant 9):
    /// clone the parent's genome, capture the payload, mutate (the parent's
    /// reachable nodes and dispatch record as its `ParentExecuted`), draw and
    /// apply the body, derive the provenance. Returns the zygote beside the
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
        let (summary, inheritance) = MutationEngine::apply_mutations_in(
            &mut genome,
            &config.mutation,
            parent.reachable_nodes(),
            ParentExecuted::Query(&executed_within),
            rng,
            &config.sensor_context(config.world.food.types.len()),
        );
        let draws = sample_body_draws(summary.applied_events > 0, &config.mutation, rng);
        let body = HeritableBody::inherit(&parent.body, &draws, &config.energy, &config.exchange);
        let provenance = Provenance::of_child(parent.generation, &summary);
        (
            Self {
                genome,
                body,
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
            body,
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
            body,
            payload.shared_memory,
            &seeds,
        );
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
}

#[cfg(test)]
#[path = "birth_tests.rs"]
mod tests;
