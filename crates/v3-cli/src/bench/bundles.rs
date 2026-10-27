//! The per-sample `bundles` block (T25.F11, `bundles-v1`): the bundles the
//! living population carries, by wiring (`plain`, `ring`, `pair`), with
//! their genomic and learned scales off zero, their members and members
//! with a nonzero own correction, the bundles on unreachable nodes, rows by
//! wiring and target (a geometric wiring's kind, a `Plain` wiring's sink),
//! and the run's creations by operator, wiring and form.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use v3_core::genome::cgp::{BundleId, Wiring};
use v3_core::genome::{BackendDef, CreatureGenome};
use v3_core::mutation::types::{wiring_index, MutationOperator, FORM_KEYS, WIRING_KEYS};
use v3_core::runtime::plasticity::LearnedState;

/// The block's definition version.
pub const BUNDLES_VERSION: &str = "bundles-v1";

/// A scale or own correction off zero: `|w| > 1e-3`, the census weight
/// floor.
const NONZERO_FLOOR: f32 = v3_core::neighborhood::drives::PATH_WEIGHT_FLOOR;

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

/// One sample's `bundles` block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bundles {
    pub version: String,
    /// Living creatures.
    pub creatures: u64,
    /// Creatures carrying a bundle on a reachable node.
    pub carriers: u64,
    /// Totals by wiring key.
    pub wirings: BTreeMap<String, WiringTotals>,
    /// Cumulative committed creations by operator key, wiring and form.
    pub created: BTreeMap<String, BTreeMap<String, BTreeMap<String, u64>>>,
    pub rows: Vec<BundleRow>,
}

/// The target key of a wiring's row.
fn target(wiring: &Wiring) -> String {
    match wiring {
        Wiring::Plain { sink } => sink.as_key(),
        Wiring::Ring { kind, .. } | Wiring::DirectionPair { kind, .. } => kind.as_key(),
    }
    .to_owned()
}

impl Bundles {
    /// Census `creatures`, each a genome, its reachable node indices and,
    /// when sampled, its runtime's learned state, with the run's creations.
    pub fn census<'a>(
        creatures: impl IntoIterator<Item = (&'a CreatureGenome, &'a [usize], Option<&'a LearnedState>)>,
        created_total: &std::collections::HashMap<MutationOperator, [[u64; 2]; 3]>,
    ) -> Self {
        let mut totals: [WiringTotals; 3] = Default::default();
        let mut learned_sampled = false;
        let mut creature_count = 0;
        let mut carriers = 0;
        let mut rows: BTreeMap<(usize, String), BundleRow> = BTreeMap::new();
        for (genome, reachable, learned) in creatures {
            creature_count += 1;
            learned_sampled |= learned.is_some();
            let mut carried = false;
            for (index, node) in genome.nodes.iter().enumerate() {
                let BackendDef::Graph(def) = &node.backend_def else {
                    continue;
                };
                let scales: BTreeMap<BundleId, Option<f32>> = learned
                    .map(|learned| {
                        learned
                            .scales(index)
                            .map(|scale| (scale.bundle, scale.learned))
                            .collect()
                    })
                    .unwrap_or_default();
                for (id, bundle) in def.bundles().iter().enumerate() {
                    let wiring = bundle.wiring();
                    let row = wiring_index(&wiring);
                    if !reachable.contains(&index) {
                        totals[row].unreachable += 1;
                        continue;
                    }
                    carried = true;
                    let id = BundleId::new(u16::try_from(id).expect("bundles below u16::MAX"));
                    let members = def.members(id).count() as u64;
                    let own_nonzero = def
                        .members(id)
                        .filter(|&member| def.edge_at(member).own_weight().abs() > NONZERO_FLOOR)
                        .count() as u64;
                    let genomic = bundle.scale().abs() > NONZERO_FLOOR;
                    let learned_nonzero = scales
                        .get(&id)
                        .copied()
                        .flatten()
                        .is_some_and(|value| value.abs() > NONZERO_FLOOR);
                    let total = &mut totals[row];
                    total.bundles += 1;
                    total.genomic_nonzero += u64::from(genomic);
                    *total.learned_nonzero.get_or_insert(0) += u64::from(learned_nonzero);
                    total.members += members;
                    total.own_nonzero += own_nonzero;
                    let entry = rows
                        .entry((row, target(&wiring)))
                        .or_insert_with(|| BundleRow {
                            wiring: WIRING_KEYS[row].to_string(),
                            target: target(&wiring),
                            count: 0,
                            genomic_nonzero: 0,
                            members: 0,
                        });
                    entry.count += 1;
                    entry.genomic_nonzero += u64::from(genomic);
                    entry.members += members;
                }
            }
            carriers += u64::from(carried);
        }
        let wirings = WIRING_KEYS
            .iter()
            .zip(totals)
            .map(|(key, mut total)| {
                total.learned_nonzero = learned_sampled.then(|| total.learned_nonzero.unwrap_or(0));
                (key.to_string(), total)
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
            creatures: creature_count,
            carriers,
            wirings,
            created,
            rows: rows.into_values().collect(),
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
    use v3_core::genome::cgp::{BundleBirth, BundleSource};

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
            MutationOperator::GraphAddBundleLive,
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
