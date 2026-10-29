//! T29.F01 A1 step 1: every event's inheritance mapping, applied to the
//! parent's seeds, gives exactly the seed rows the container kept, over the
//! Graph, InputRef and Topology catalogs and over whole births.

use proptest::prelude::*;
use rand::{rngs::SmallRng, SeedableRng};

use super::*;
use crate::config::test_sensor_context;
use crate::creature::founder::v3alpha1_founder_genome;
use crate::genome::cgp::BirthSeeds;
use crate::genome::learning::{LearnableId, LearnableKey, Seeds};
use crate::mutation::reachability::TargetSets;

/// The founder grown by `growth` rate-1.0 births of four events.
fn grown(seed: u64, growth: u8) -> CreatureGenome {
    let config = MutationConfig {
        per_unit_rate: 1.0,
        ..MutationConfig::default()
    };
    let mut genome = v3alpha1_founder_genome();
    let mut rng = SmallRng::seed_from_u64(seed);
    for _ in 0..growth {
        let _ = MutationEngine::apply_mutations_on_units(
            &mut genome,
            4,
            &config,
            &[],
            ParentExecuted::NONE,
            &mut rng,
            &test_sensor_context(2),
        );
    }
    genome
}

/// Seed every edge and bundle of every Graph node with a distinct value,
/// leaving `holes` (by position modulo 3) unseeded.
fn seed_all(genome: &mut CreatureGenome, holes: u8) {
    let mut next = 0.5f32;
    let mut value = || {
        next += 1.0;
        (next as u32 % 3 != u32::from(holes % 3)).then_some(next)
    };
    for node in &mut genome.nodes {
        let BackendDef::Graph(def) = node.backend_edit().1 else {
            continue;
        };
        let compute = def
            .compute_nodes()
            .iter()
            .map(|n| n.inputs.iter().map(|_| value()).collect())
            .collect();
        let sinks = def
            .output_sinks()
            .iter()
            .map(|s| s.inputs.iter().map(|_| value()).collect())
            .collect();
        let bundles = def.bundles().iter().map(|_| value()).collect();
        def.seed_birth(BirthSeeds::Rows {
            compute,
            sinks,
            bundles,
        });
    }
}

/// The seed rows `genome` carries, keyed by learnable identity.
fn kept(genome: &CreatureGenome) -> Seeds {
    let mut seeds = Seeds::default();
    for (node, genome_node) in genome.nodes.iter().enumerate() {
        let BackendDef::Graph(def) = genome_node.backend_def() else {
            continue;
        };
        let Some(rows) = def.birth_seeds() else {
            continue;
        };
        for site in def.edge_sites() {
            if let Some(value) = rows.value(site) {
                seeds.insert(
                    LearnableId {
                        node,
                        key: LearnableKey::Edge(site),
                    },
                    value,
                );
            }
        }
        for (bundle, _) in def.indexed_bundles() {
            if let Some(value) = rows.bundle_value(bundle) {
                seeds.insert(
                    LearnableId {
                        node,
                        key: LearnableKey::Scale(bundle),
                    },
                    value,
                );
            }
        }
    }
    seeds
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// One event of every operator of the Graph, InputRef and Topology
    /// catalogs on a grown, seeded genome: the event's mapping applied to
    /// the parent's seeds equals the rows the container kept.
    #[test]
    fn an_events_mapping_gives_the_rows_the_container_kept(
        seed in any::<u64>(),
        growth in 0u8..6,
        holes in 0u8..3,
    ) {
        let mut parent = grown(seed, growth);
        seed_all(&mut parent, holes);
        let parent_seeds = kept(&parent);
        let config = MutationConfig::default();
        let context = test_sensor_context(2);
        let reachable: Vec<usize> = (0..parent.nodes.len()).collect();
        let sets = TargetSets::new(&reachable, &[]);
        for (index, &operator) in MutationOperator::ALL.iter().enumerate() {
            if matches!(operator, MutationOperator::Vm(_)) {
                continue;
            }
            let mut child = parent.clone();
            let mut rng = SmallRng::seed_from_u64(seed ^ index as u64);
            let mut targets = sets.selector(0.5, 0.0);
            if let Ok((_, report)) =
                dispatch(&mut child, operator, &mut targets, &mut rng, &config, &context)
            {
                prop_assert_eq!(
                    report.inheritance.apply(&parent_seeds),
                    kept(&child),
                    "{:?}",
                    operator
                );
            }
        }
    }

    /// A whole birth: the composed mapping of every committed event,
    /// applied to the parent's seeds, equals the rows the draft kept.
    #[test]
    fn a_births_composed_mapping_gives_the_rows_the_draft_kept(
        seed in any::<u64>(),
        growth in 0u8..6,
        units in 1u32..12,
        holes in 0u8..3,
    ) {
        let mut parent = grown(seed, growth);
        seed_all(&mut parent, holes);
        let parent_seeds = kept(&parent);
        let config = MutationConfig {
            per_unit_rate: 1.0,
            ..MutationConfig::default()
        };
        let mut child = parent.clone();
        let (_, inheritance) = MutationEngine::apply_mutations_on_units(
            &mut child,
            units,
            &config,
            &[],
            ParentExecuted::NONE,
            &mut SmallRng::seed_from_u64(seed),
            &test_sensor_context(2),
        );
        prop_assert_eq!(inheritance.apply(&parent_seeds), kept(&child));
    }
}
