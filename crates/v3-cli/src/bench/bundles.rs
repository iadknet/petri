//! The per-sample `bundles` block (T25.F11, T25.F12, `bundles-v2`): the
//! bundles the living population carries, by wiring (`plain`, `ring`,
//! `pair`), with their genomic and learned scales off zero, their members
//! and members with a nonzero own modifier, the bundles on unreachable
//! nodes, their learning by wiring (the bundles with each slot, the members
//! by policy and the crediting members), rows by wiring and target (a
//! geometric wiring's kind, a `Plain` wiring's sink), and the run's
//! creations by operator, wiring and form.

use std::collections::BTreeMap;

use super::retention::{retained_struct, CheckpointRetention};
use serde::{Deserialize, Serialize};
use v3_core::genome::CreatureGenome;
use v3_core::mutation::types::{MutationOperator, FORM_KEYS, WIRING_KEYS};
use v3_core::neighborhood::bundles::{BundleCensus, BundleTarget, POLICY_KEYS};
use v3_core::runtime::plasticity::LearnedState;

/// The block's definition version.
pub const BUNDLES_VERSION: &str = "bundles-v2";

/// One wiring's totals over the reachable bundles.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WiringTotals {
    pub bundles: u64,
    /// Bundles with `|scale| > 1e-3`.
    pub genomic_nonzero: u64,
    /// Bundles whose runtime learned scale is initialized and off zero;
    /// absent where no runtime is sampled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub learned_nonzero: Option<u64>,
    /// Members present.
    pub members: u64,
    /// Members with `|own_weight| > 1e-3`.
    pub own_nonzero: u64,
    /// Bundles of this wiring on unreachable nodes.
    pub unreachable: u64,
}

/// One wiring's learning over the reachable bundles (T25.F12).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WiringLearning {
    /// Bundles with `scale_learning`.
    pub scale_learning: u64,
    /// Bundles with `member_default`.
    pub member_default: u64,
    /// Members by policy key (`inherit`, `static`, `own`).
    pub members_by_policy: BTreeMap<String, u64>,
    /// Members with `credits`.
    pub crediting: u64,
}

/// One `(wiring, target)` row: a geometric wiring's kind or a `Plain`
/// wiring's vote sink.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleRow {
    pub wiring: String,
    pub target: String,
    pub count: u64,
    pub genomic_nonzero: u64,
    pub members: u64,
}

retained_struct! {
    retention: CheckpointRetention;
    /// One sample's `bundles` block. The summary keeps the aggregates on
    /// every checkpoint and the rows on each world's last checkpoint only.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Bundles {
        pub version: String => CheckpointRetention::Every,
        /// Living creatures.
        pub creatures: u64 => CheckpointRetention::Every,
        /// Creatures carrying a bundle on a reachable node.
        pub carriers: u64 => CheckpointRetention::Every,
        /// Totals by wiring key.
        pub wirings: BTreeMap<String, WiringTotals> => CheckpointRetention::Every,
        /// Cumulative committed creations by operator key, wiring and form.
        pub created: BTreeMap<String, BTreeMap<String, BTreeMap<String, u64>>>
            => CheckpointRetention::Every,
        pub rows: Vec<BundleRow> => CheckpointRetention::Last,
        /// Learning by wiring key (T25.F12).
        pub learning: BTreeMap<String, WiringLearning> => CheckpointRetention::Last,
    }
}

/// The target key of a row.
fn target(target: BundleTarget) -> String {
    match target {
        BundleTarget::Sink(sink) => sink.as_key(),
        BundleTarget::Kind(kind) => kind.as_key().to_owned(),
    }
}

impl Bundles {
    /// The block of `creatures`, each a genome, its reachable node indices
    /// and, when sampled, its runtime's learned state, with the run's
    /// creations: the readers' bundle census, labelled.
    pub fn census<'a>(
        creatures: impl IntoIterator<Item = (&'a CreatureGenome, &'a [usize], Option<&'a LearnedState>)>,
        created_total: &std::collections::HashMap<MutationOperator, [[u64; 2]; 3]>,
    ) -> Self {
        let census = BundleCensus::observe(creatures);
        let wirings = WIRING_KEYS
            .iter()
            .zip(census.wirings)
            .map(|(key, total)| {
                (
                    key.to_string(),
                    WiringTotals {
                        bundles: total.bundles,
                        genomic_nonzero: total.genomic_nonzero,
                        learned_nonzero: census.learned_sampled.then_some(total.learned_nonzero),
                        members: total.members,
                        own_nonzero: total.own_nonzero,
                        unreachable: total.unreachable,
                    },
                )
            })
            .collect();
        let learning = WIRING_KEYS
            .iter()
            .zip(&census.wirings)
            .map(|(key, total)| {
                (
                    key.to_string(),
                    WiringLearning {
                        scale_learning: total.scale_learning,
                        member_default: total.member_default,
                        members_by_policy: POLICY_KEYS
                            .iter()
                            .zip(total.members_by_policy)
                            .map(|(policy, count)| (policy.to_string(), count))
                            .collect(),
                        crediting: total.crediting,
                    },
                )
            })
            .collect();
        let rows: BTreeMap<(usize, String), BundleRow> = census
            .rows
            .iter()
            .map(|row| {
                let label = target(row.target);
                (
                    (row.wiring, label.clone()),
                    BundleRow {
                        wiring: WIRING_KEYS[row.wiring].to_string(),
                        target: label,
                        count: row.count,
                        genomic_nonzero: row.genomic_nonzero,
                        members: row.members,
                    },
                )
            })
            .collect();
        let created = created_total
            .iter()
            .map(|(operator, by_wiring)| {
                let by_wiring = WIRING_KEYS
                    .iter()
                    .zip(by_wiring)
                    .map(|(wiring, by_form)| {
                        let by_form = FORM_KEYS
                            .iter()
                            .zip(by_form)
                            .map(|(form, count)| (form.to_string(), *count))
                            .collect();
                        (wiring.to_string(), by_form)
                    })
                    .collect();
                (operator.as_key().to_string(), by_wiring)
            })
            .collect();
        Self {
            version: BUNDLES_VERSION.to_string(),
            creatures: census.creatures,
            carriers: census.carriers,
            wirings,
            created,
            rows: rows.into_values().collect(),
            learning,
        }
    }

    /// The living population's block from post-tick state; `None` at
    /// extinction.
    pub(super) fn observe(sim: &v3_core::simulation::Simulation) -> Option<Self> {
        if sim.creatures.is_empty() {
            return None;
        }
        Some(Self::census(
            sim.creatures.values().map(|creature| {
                (
                    creature.genome(),
                    creature.reachable_nodes(),
                    Some(creature.graph_runtime().learned()),
                )
            }),
            &sim.stats.bundles_created_total,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use v3_core::config::SimulationConfig;
    use v3_core::contracts::votes::DirectedVoteKind;
    use v3_core::contracts::{InputReference, WorldInputKey};
    use v3_core::creature::founder::v3alpha1_founder_genome;
    use v3_core::genome::cgp::{BundleBirth, BundleSource, Wiring};
    use v3_core::genome::BackendDef;
    use v3_core::mutation::graph::GraphOperator;

    fn context() -> v3_core::contracts::SensorContext {
        SimulationConfig::default().sensor_context(1)
    }

    /// The founder with one `Ring` and one `Plain` bundle on its Graph
    /// node, the ring at `scale`.
    fn carrier(scale: f32) -> CreatureGenome {
        let mut genome = v3alpha1_founder_genome();
        let node = genome
            .nodes
            .iter_mut()
            .find(|node| matches!(node.backend_def, BackendDef::Graph(_)))
            .expect("the founder carries a Graph node");
        let ring = node
            .add_sensor(
                InputReference::World(WorldInputKey::NeighborBarrierRing),
                &[BundleBirth {
                    field_set: 0,
                    wiring: Wiring::Ring {
                        kind: DirectedVoteKind::Move,
                        offset: 1,
                    },
                    scale,
                }],
                &context(),
            )
            .unwrap();
        let _ = ring;
        node.add_bundle(
            BundleSource::Memory { previous: false },
            BundleBirth {
                field_set: 0,
                wiring: Wiring::Plain {
                    sink: v3_core::contracts::votes::VoteSink::Eat,
                },
                scale: 0.0,
            },
            &context(),
        )
        .unwrap();
        genome
    }

    #[test]
    fn the_block_counts_bundles_by_wiring_and_target() {
        let genome = carrier(0.5);
        let reachable: Vec<usize> = (0..genome.nodes.len()).collect();
        let mut created = std::collections::HashMap::new();
        created.insert(
            MutationOperator::Graph(GraphOperator::AddBundleLive),
            [[1, 0], [0, 2], [0, 0]],
        );
        let block = Bundles::census([(&genome, reachable.as_slice(), None)], &created);
        assert_eq!(block.version, BUNDLES_VERSION);
        assert_eq!((block.creatures, block.carriers), (1, 1));
        let ring = &block.wirings["ring"];
        assert_eq!(
            (ring.bundles, ring.genomic_nonzero, ring.members),
            (1, 1, 8)
        );
        assert_eq!(ring.learned_nonzero, None, "no runtime sampled");
        let plain = &block.wirings["plain"];
        assert_eq!((plain.bundles, plain.members), (1, 16));
        assert_eq!(block.wirings["pair"], WiringTotals::default());
        assert_eq!(
            block.created["Graph.AddBundleLive"]["ring"]["randomized"],
            2
        );
        assert_eq!(block.created["Graph.AddBundleLive"]["plain"]["uniform"], 1);
        assert_eq!(block.rows.len(), 2);
        assert_eq!(block.rows[0].target, "Eat");
        assert_eq!(block.rows[1].target, "Move");
    }

    #[test]
    fn an_unreachable_bundle_is_counted_apart() {
        let genome = carrier(0.0);
        let block = Bundles::census(
            [(&genome, [].as_slice(), None)],
            &std::collections::HashMap::new(),
        );
        assert_eq!(block.carriers, 0);
        assert_eq!(block.wirings["ring"].unreachable, 1);
        assert_eq!(block.wirings["plain"].unreachable, 1);
        assert_eq!(block.wirings["ring"].bundles, 0);
    }

    #[test]
    fn a_sampled_runtime_reads_learned_scales() {
        let config = SimulationConfig::default();
        let sim = v3_core::simulation::seeding::seed_simulation_with_founder_genomes(
            config,
            3,
            |founder, genome| {
                if founder == 0 {
                    *genome = carrier(0.5);
                }
            },
        );
        let block = Bundles::observe(&sim).expect("a living population");
        assert_eq!(block.wirings["ring"].learned_nonzero, Some(0));
        assert!(block.wirings["ring"].bundles > 0);
    }
}
