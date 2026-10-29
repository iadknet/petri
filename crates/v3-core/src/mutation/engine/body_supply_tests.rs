//! The body chromosome's supply (T29.F02 invariants 3 and 4): its own
//! per-unit draw after the brain's, each event's uniform gene and draws, and
//! how a body event is reported.

use proptest::prelude::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use super::*;
use crate::config::SimulationConfig;
use crate::creature::body::default_body_ranges;
use crate::creature::founder::v3alpha1_founder_genome;
use crate::genome::body::{BodyEdit, BodyGene, PhenotypeDraw};
use crate::mutation::body::{sample_body_edit, BodyOperator};
use crate::mutation::phenotype::sample_phenotype_draw;

/// `units` under `config` from `seed` on the founder.
fn birth(
    config: &MutationConfig,
    units: ChromosomeUnits,
    seed: u64,
) -> (
    CreatureGenome,
    MutationSummary,
    InheritanceMapping,
    SmallRng,
) {
    let mut genome = v3alpha1_founder_genome();
    let mut rng = SmallRng::seed_from_u64(seed);
    let (summary, inheritance) = MutationEngine::apply_mutations_on_units(
        &mut genome,
        units,
        config,
        &default_body_ranges(),
        &[0, 1],
        ParentExecuted::NONE,
        &mut rng,
        &crate::config::test_sensor_context(1),
    );
    (genome, summary, inheritance, rng)
}

fn rate(per_unit_rate: f64) -> MutationConfig {
    MutationConfig {
        per_unit_rate,
        ..SimulationConfig::default().mutation
    }
}

const BODY_ONLY: ChromosomeUnits = ChromosomeUnits { brain: 0, body: 5 };

/// The body-supply draw stream pinned: over seeds, the founder's body after
/// the body's own supply at rate 0.5 (colour switch and flip chances 0.5),
/// its event records and the next draw. Any change to a body draw, its
/// order or its arithmetic moves it.
#[test]
fn the_body_supply_draw_stream_is_pinned() {
    let mut config = rate(0.5);
    config.phenotype.channel_change_chance = 0.5;
    config.phenotype.polarity_flip_chance = 0.5;
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for seed in 0u64..512 {
        let (genome, summary, _, mut rng) = birth(&config, BODY_ONLY, seed);
        let bytes = format!(
            "{:?}{:?}{}",
            genome.body(),
            summary.events,
            rng.gen::<u64>()
        );
        for byte in bytes.bytes() {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3);
        }
    }
    assert_eq!(
        hash, 6_204_822_344_146_714_388,
        "the body supply draw stream"
    );
}

/// A birth with no brain event still draws the body's supply: at rate 1.0
/// on the body's five units, five body events, each attempted and applied
/// with no target, no discards and `NotApplicable`, and the brain and the
/// inheritance mapping unchanged.
#[test]
fn a_zero_brain_event_birth_still_draws_and_reports_the_body_supply() {
    let (genome, summary, inheritance, _) = birth(&rate(1.0), BODY_ONLY, 11);
    let founder = v3alpha1_founder_genome();
    assert_eq!(genome.clone().with_body(*founder.body()), founder);
    assert_ne!(genome.body(), founder.body());
    assert_eq!(inheritance, InheritanceMapping::Identity);
    assert_eq!(
        (
            summary.attempted_events,
            summary.applied_events,
            summary.skipped_events
        ),
        (5, 5, 0)
    );
    assert_eq!(summary.attempted_by_domain[&MutationDomain::Body], 5);
    assert_eq!(summary.applied_by_domain[&MutationDomain::Body], 5);
    assert_eq!(summary.not_applicable_events, 5);
    assert_eq!(summary.birth_edge_class(), None);
    for event in &summary.events {
        assert_eq!(event.domain, MutationDomain::Body);
        assert!(matches!(event.operator, Some(MutationOperator::Body(_))));
        assert_eq!(event.target, None);
        assert!(event.discarded.is_empty());
        assert_eq!(
            event.outcome,
            MutationEventOutcome::Applied(TargetReachability::NotApplicable)
        );
    }
    let funnels: u64 = summary
        .operator_funnel_by_operator
        .values()
        .map(|funnel| funnel.applied)
        .sum();
    assert_eq!(funnels, 5);
}

/// The body's events follow the brain's: at rate 1.0 on three brain units
/// and the body's five, the first three records are brain events and the
/// last five body events.
#[test]
fn the_body_events_follow_the_brain_events() {
    let units = ChromosomeUnits { brain: 3, body: 5 };
    let (_, summary, _, _) = birth(&rate(1.0), units, 5);
    assert_eq!(summary.events.len(), 8);
    assert!(summary.events[..3]
        .iter()
        .all(|event| event.domain != MutationDomain::Body));
    assert!(summary.events[3..]
        .iter()
        .all(|event| event.domain == MutationDomain::Body));
}

/// Size pressure restricts brain operators to decreasing ones but leaves
/// the body's events: a body edit changes no genome size.
#[test]
fn size_pressure_leaves_the_body_events() {
    let mut config = rate(1.0);
    config.genome_size_pressure_enabled = true;
    config.genome_size_cap = 1;
    let units = ChromosomeUnits { brain: 2, body: 5 };
    let (_, summary, _, _) = birth(&config, units, 3);
    assert_eq!(summary.applied_by_domain[&MutationDomain::Body], 5);
}

/// The production entry's units are the genome's own: its brain units and
/// the body's five.
#[test]
fn a_genomes_own_units_are_its_brain_units_and_the_five_body_genes() {
    let founder = v3alpha1_founder_genome();
    assert_eq!(
        ChromosomeUnits::of(&founder),
        ChromosomeUnits { brain: 80, body: 5 }
    );
}

/// Every body operator is complexity-neutral and has no brain weight.
#[test]
fn a_body_operator_is_neutral_and_unweighted() {
    let config = MutationConfig::default();
    for operator in BodyOperator::ALL {
        let operator = MutationOperator::Body(operator);
        assert_eq!(operator.domain(), MutationDomain::Body);
        assert_eq!(operator.weight(&config), None);
        assert!(!operator.complexity_effect().is_decreasing());
        assert!(operator.as_key().starts_with("Body."));
        assert_eq!(MutationDomain::Body.brain(), None);
    }
}

proptest! {
    /// One body event draws its gene uniformly over the five, then that
    /// gene's draws only: the colour step's, a bit in `0..32`, or a step
    /// uniform in `[-log_step, log_step]` of the gene's own config.
    #[test]
    fn a_body_event_draws_its_gene_then_the_genes_draws(seed in any::<u64>()) {
        let mut config = MutationConfig::default();
        config.capacity.log_step = 0.3;
        config.share.log_step = 0.7;
        let mut rng = SmallRng::seed_from_u64(seed);
        let edit = sample_body_edit(&config, &mut rng);
        let mut replay = SmallRng::seed_from_u64(seed);
        let gene = BodyGene::ALL[replay.gen_range(0..5)];
        let expected = match gene {
            BodyGene::Colour => BodyEdit::Colour(sample_phenotype_draw(&config.phenotype, &mut replay)),
            BodyGene::KinTag => BodyEdit::KinTag(replay.gen_range(0..32)),
            BodyGene::Capacity => BodyEdit::Capacity(replay.gen_range(-0.3f32..=0.3)),
            BodyGene::StealShare => BodyEdit::StealShare(replay.gen_range(-0.7f32..=0.7)),
            BodyGene::GiveShare => BodyEdit::GiveShare(replay.gen_range(-0.7f32..=0.7)),
        };
        prop_assert_eq!(edit, expected);
        prop_assert_eq!(edit.gene(), gene);
        if let BodyEdit::Colour(PhenotypeDraw { step, .. }) = edit {
            prop_assert_eq!(step, config.phenotype.channel_step.max(1));
        }
        prop_assert_eq!(rng.gen::<u64>(), replay.gen::<u64>());
    }

    /// The accounting invariant holds with body events, every body event
    /// applies, and the body's count never exceeds its five units.
    #[test]
    fn body_events_keep_the_accounting_invariant(
        seed in any::<u64>(),
        per_unit_rate in 0.0f64..=1.0,
        brain in 0u32..6,
    ) {
        let units = ChromosomeUnits { brain, body: 5 };
        let (_, summary, _, _) = birth(&rate(per_unit_rate), units, seed);
        let body = summary.attempted_by_domain.get(&MutationDomain::Body).copied().unwrap_or(0);
        prop_assert!(body <= 5);
        prop_assert_eq!(summary.applied_by_domain.get(&MutationDomain::Body).copied().unwrap_or(0), body);
        prop_assert_eq!(summary.attempted_events, summary.applied_events + summary.skipped_events);
        prop_assert_eq!(summary.events.len() as u32, summary.attempted_events);
    }
}
