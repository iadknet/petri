use std::collections::BTreeSet;

use super::*;
use crate::config::FounderProfile;
use crate::contracts::NodeId;
use crate::creature::founder::founder_genome;
use crate::creature::genome::analysis::mesh_reachable_nodes;
use crate::creature::genome::cgp::test_support::plastic_node;
use crate::creature::genome::cgp::{
    validate_graph, CgpGraphBackendDef, ComputeNode, ComputeNodeKind, EdgeOccurrence, GraphEdge,
    GraphInvariantViolation, GraphSource, OutputSinkKind,
};
use crate::creature::genome::vote::VoteSink;
use crate::creature::genome::{
    BackendDef, CreatureGenome, NodeGenome, OutcomeChannel, PlasticityConfig,
    RewardModulationConfig,
};
use crate::creature::state::GraphRuntimeState;
use crate::runtime::plasticity::PlasticityKind;

fn plastic(rule: HebbianRule, reward_modulated: bool, lamarckian: bool) -> PlasticityConfig {
    PlasticityConfig {
        rule,
        learning_rate: 0.1,
        weight_clamp: 2.0,
        lamarckian,
        modulation: reward_modulated.then_some(RewardModulationConfig {
            reward_source: OutcomeChannel::EnergyDelta,
            trace_decay: 0.5,
        }),
        retention: 1.0,
    }
}

fn node(plasticity: Option<PlasticityConfig>, weights: &[f32]) -> ComputeNode {
    plastic_node(
        ComputeNodeKind::Add,
        weights
            .iter()
            .enumerate()
            .map(|(slot, &weight)| {
                GraphEdge::new(
                    GraphSource::SharedMemory {
                        slot: slot as u8,
                        previous: false,
                    },
                    weight,
                )
            })
            .collect(),
        plasticity,
    )
}

fn genome_of(def: CgpGraphBackendDef) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Graph(def),
            targets: vec![],
        }],
    }
}

fn graph_genome(compute_nodes: Vec<ComputeNode>) -> CreatureGenome {
    genome_of(CgpGraphBackendDef::new_with_fixed_outputs().with_compute_nodes(compute_nodes))
}

/// The learned state of `genome` with its plastic slots of mesh node 0, in
/// canonical order, set to `learned` (the backend visited), or unvisited
/// when `learned` is `None`.
fn learned_state(genome: &CreatureGenome, learned: Option<&[f32]>) -> GraphRuntimeState {
    let mut state = GraphRuntimeState::for_genome(genome, Vec::new());
    for (slot, &value) in learned.unwrap_or(&[]).iter().enumerate() {
        state.learned.set_slot(0, slot, value, 0.0);
    }
    state
}

fn census_of(
    genome: &CreatureGenome,
    reachable: &[usize],
    learned: Option<&[f32]>,
) -> PlasticityCensus {
    let state = learned_state(genome, learned);
    PlasticityCensus::observe([(reachable, state.learned())])
}

const HEBBIAN: usize = 0;
const REWARD: usize = 1;
const COMPUTE: usize = 0;
const SINK: usize = 1;

#[test]
fn every_attribute_key_is_distinct() {
    let rules: BTreeSet<_> = RULES.iter().map(|&rule| rule_key(rule)).collect();
    assert_eq!(rules.len(), RULES.len());
    for keys in [LEARNING_KEYS, SURFACE_KEYS, INHERITANCE_KEYS] {
        assert_ne!(keys[0], keys[1]);
    }
    for (index, &rule) in RULES.iter().enumerate() {
        let kind = PlasticityKind::of(&plastic(rule, false, false));
        assert_eq!(kind.rule_index(), index);
    }
}

#[test]
fn the_founder_carries_no_plasticity() {
    let genome = founder_genome(FounderProfile::default());
    let reachable = mesh_reachable_nodes(&genome);
    let census = census_of(&genome, &reachable, None);
    assert_eq!(census.creatures, 1);
    assert_eq!(census.carriers, 0);
    assert_eq!(
        census,
        PlasticityCensus {
            creatures: 1,
            ..PlasticityCensus::default()
        }
    );
}

#[test]
fn drift_and_clamp_read_the_learned_weights() {
    let config = plastic(HebbianRule::Classic, false, false);
    let genome = graph_genome(vec![node(Some(config), &[0.5, -0.25, 1.0])]);
    let census = census_of(&genome, &[0], Some(&[0.75, -2.0, 1.0]));

    let row = census.edges[HEBBIAN][COMPUTE];
    assert_eq!(census.carriers, 1);
    assert_eq!(row.edges, 3);
    // 0.25 + 1.75 + 0; every term is exact in binary.
    assert_eq!(row.drift_sum, 2.0);
    assert_eq!(row.at_clamp, 1);
}

/// An uninitialized edge (its backend never visited, no seed) is counted
/// but exposes no learned value: no drift and never at its clamp, even
/// when its genomic weight is beyond the clamp.
#[test]
fn an_uninitialized_edge_has_no_drift_and_is_never_at_its_clamp() {
    let config = plastic(HebbianRule::Classic, false, false);
    let genome = graph_genome(vec![node(Some(config), &[5.0])]);
    let census = census_of(&genome, &[0], None);
    let row = census.edges[HEBBIAN][COMPUTE];
    assert_eq!((census.carriers, row.edges, row.at_clamp), (1, 1, 0));
    assert_eq!(row.drift_sum, 0.0);
}

/// The census reads the stored clamp, which I1 keeps in `[0.01, 10]`: a
/// clamp beyond it is rejected at the load boundary instead of being
/// normalized by the reader.
#[test]
fn an_out_of_range_clamp_is_rejected_at_the_load_boundary() {
    let json = r#"{"compute_nodes":[{"kind":"Add","inputs":[{"source":{"ComputeNode":0},"own_weight":0.0,"plasticity":{"rule":"Oja","learning_rate":0.1,"weight_clamp":50.0,"lamarckian":false,"retention":1.0}}]}],"output_sinks":[]}"#;
    let def: CgpGraphBackendDef = serde_json::from_str(json).unwrap();
    let node = genome_of(def).nodes.remove(0);
    assert_eq!(
        validate_graph(&node),
        Err(GraphInvariantViolation::PlasticityOutOfRange)
    );
}

#[test]
fn edges_land_in_their_pathway_surface_rule_and_inheritance() {
    let mut genome = graph_genome(vec![
        node(
            Some(plastic(HebbianRule::Covariance, true, true)),
            &[0.1, 0.2],
        ),
        node(None, &[0.3, 0.4, 0.5]),
        node(Some(plastic(HebbianRule::Oja, false, false)), &[0.6]),
    ]);
    let BackendDef::Graph(def) = &mut genome.nodes[0].backend_def else {
        unreachable!()
    };
    let vote = def
        .sink_surface(OutputSinkKind::ActionVote(VoteSink::Eat))
        .unwrap();
    def.push_edge(vote, GraphEdge::new(GraphSource::ComputeNode(0), 1.0));
    def.edge_mut(EdgeOccurrence::new(vote, 0))
        .set_plasticity(Some(plastic(HebbianRule::Oja, true, false)));
    let census = census_of(&genome, &[0], None);
    assert_eq!(census.carriers, 1);
    assert_eq!(census.edges[REWARD][COMPUTE].edges, 2);
    assert_eq!(census.edges[HEBBIAN][COMPUTE].edges, 1);
    assert_eq!(census.edges[REWARD][SINK].edges, 1);
    assert_eq!(census.edges[HEBBIAN][SINK].edges, 0);
    assert_eq!(census.edges_by_rule, [0, 2, 0, 2]);
    assert_eq!(census.edges_by_inheritance, [2, 2]);
}

#[test]
fn unreachable_nodes_are_not_read() {
    let config = plastic(HebbianRule::Classic, false, false);
    let genome = graph_genome(vec![node(Some(config), &[0.5])]);
    let census = census_of(&genome, &[], Some(&[2.0]));
    assert_eq!(
        census,
        PlasticityCensus {
            creatures: 1,
            ..PlasticityCensus::default()
        }
    );
}

#[test]
fn carriers_count_creatures_not_edges() {
    let config = plastic(HebbianRule::AntiHebb, false, true);
    let carrier = graph_genome(vec![node(Some(config), &[0.1, 0.2])]);
    let plain = graph_genome(vec![node(None, &[0.1])]);
    let carrier_state = learned_state(&carrier, None);
    let plain_state = learned_state(&plain, None);
    let census = PlasticityCensus::observe([
        ([0].as_slice(), carrier_state.learned()),
        ([0].as_slice(), carrier_state.learned()),
        ([0].as_slice(), plain_state.learned()),
    ]);
    assert_eq!((census.creatures, census.carriers), (3, 2));
    assert_eq!(census.edges[HEBBIAN][COMPUTE].edges, 4);
    assert_eq!(census.edges_by_inheritance, [0, 4]);
}

/// The keyed readers the telemetry export walks name every cell, rule and
/// inheritance in key order with its own count, and the surface totals sum
/// both pathways of each surface.
#[test]
fn the_keyed_readers_carry_every_count_under_its_key() {
    let cell = |edges, drift_sum, at_clamp| PlasticEdges {
        edges,
        drift_sum,
        at_clamp,
    };
    let census = PlasticityCensus {
        creatures: 4,
        carriers: 3,
        edges: [
            [cell(1, 0.5, 2), cell(2, 0.25, 3)],
            [cell(3, 0.125, 5), cell(4, 2.0, 7)],
        ],
        edges_by_rule: [11, 12, 13, 14],
        edges_by_inheritance: [21, 22],
        edges_by_scope: [31, 32, 33],
        scales: [
            LearnedScales {
                scales: 41,
                drift_sum: 0.5,
                at_bound: 1,
            },
            LearnedScales {
                scales: 42,
                drift_sum: 1.5,
                at_bound: 2,
            },
        ],
        edges_by_retention: std::array::from_fn(|bin| RetentionCell {
            count: 50 + bin as u64,
            ..RetentionCell::default()
        }),
        scales_by_retention: std::array::from_fn(|bin| RetentionCell {
            count: 60 + bin as u64,
            ..RetentionCell::default()
        }),
    };

    let cells: Vec<_> = census.cells().collect();
    assert_eq!(
        cells,
        vec![
            ("hebbian", "compute", cell(1, 0.5, 2)),
            ("hebbian", "sink", cell(2, 0.25, 3)),
            ("reward_modulated", "compute", cell(3, 0.125, 5)),
            ("reward_modulated", "sink", cell(4, 2.0, 7)),
        ]
    );
    let rules: Vec<_> = census.rule_counts().collect();
    let expected: Vec<_> = RULES.iter().map(|&rule| rule_key(rule)).zip(11..).collect();
    assert_eq!(rules, expected);
    assert_eq!(rules.len(), 4);
    assert_eq!(
        census.scope_counts().collect::<Vec<_>>(),
        vec![("ungrouped", 31), ("edge", 32), ("group", 33)]
    );
    let scales: Vec<_> = census.scale_cells().collect();
    assert_eq!(
        scales,
        vec![
            ("darwinian", census.scales[0]),
            ("lamarckian", census.scales[1])
        ]
    );
    let inheritance: Vec<_> = census.inheritance_counts().collect();
    assert_eq!(inheritance, vec![("darwinian", 21), ("lamarckian", 22)]);
    let retention = |cells: Vec<(&'static str, RetentionCell)>| -> Vec<(&str, u64)> {
        cells
            .into_iter()
            .map(|(key, cell)| (key, cell.count))
            .collect()
    };
    assert_eq!(
        retention(census.edge_retention_cells().collect()),
        RETENTION_KEYS.into_iter().zip(50..).collect::<Vec<_>>()
    );
    assert_eq!(
        retention(census.scale_retention_cells().collect()),
        RETENTION_KEYS.into_iter().zip(60..).collect::<Vec<_>>()
    );
    // Every term is exact in binary.
    let totals: Vec<_> = census.surface_totals().collect();
    assert_eq!(totals, vec![("compute", 0.625, 7), ("sink", 2.25, 10)]);
}

/// T25.F07 invariant 10: a group learner adds no edge drift, an unchanged
/// correction under a nonzero genomic scale drifts 0, and a learned scale
/// counts once per group by its `inherit_scale`, with its drift and bound.
#[test]
fn group_scope_learners_and_learned_scales_are_censused_by_scope_and_scale() {
    use crate::contracts::WorldInputKey;
    use crate::creature::genome::cgp::{BirthSeeds, EdgeSurface, Scope, Shape};
    use crate::creature::genome::vote::DirectedVoteKind;
    use crate::creature::genome::HebbianRule;

    let lamarckian = PlasticityConfig {
        rule: HebbianRule::Classic,
        learning_rate: 0.1,
        weight_clamp: 1.5,
        lamarckian: true,
        modulation: None,
        retention: 1.0,
    };
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let id = def.add_group(Shape::Ring { ring_ref: 0 }, DirectedVoteKind::Move, 0);
    def.group_mut(id).set_scale(0.5);
    def.group_mut(id).set_inherit_scale(true);
    let members: Vec<EdgeOccurrence> = def.members(id).collect();
    def.edge_mut(members[0]).set_weight(0.25);
    for (k, scope) in [Scope::Edge, Scope::Group, Scope::Group]
        .into_iter()
        .enumerate()
    {
        let mut edge = def.edge_mut(members[k]);
        edge.set_plasticity(Some(lamarckian));
        edge.set_scope(scope);
    }
    let mut sinks: Vec<Vec<Option<f32>>> = def
        .output_sinks()
        .iter()
        .map(|sink| vec![None; sink.inputs.len()])
        .collect();
    let EdgeSurface::SinkInput(sink) = members[0].surface else {
        unreachable!()
    };
    // The edge learner's seed is its genomic correction; the scale is
    // seeded past its bound.
    sinks[sink][members[0].index] = Some(0.25);
    let seeds = BirthSeeds::Rows {
        compute: Vec::new(),
        sinks,
        groups: vec![Some(4.0)],
    };
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![crate::contracts::InputReference::World(
                WorldInputKey::NeighborBarrierRing,
            )],
            backend_def: BackendDef::Graph(def),
            targets: vec![],
        }],
    };
    let state = GraphRuntimeState::for_genome(&genome, vec![Some(seeds)]);
    let census = PlasticityCensus::observe([(&[0usize][..], state.learned())]);

    assert_eq!(census.edges_by_scope, [0, 1, 2]);
    let sink_cell = census.edges[0][1];
    assert_eq!(sink_cell.edges, 3);
    assert_eq!(sink_cell.drift_sum, 0.0, "the unchanged correction");
    assert_eq!(sink_cell.at_clamp, 0);
    let [darwinian, lamarckian_scales] = census.scales;
    assert_eq!(darwinian, LearnedScales::default());
    assert_eq!(lamarckian_scales.scales, 1, "once per group");
    // Seeded at 4, clamped to the bound 1.5 at construction: its birth
    // value, so no lifetime drift (T25.F09).
    assert_eq!(lamarckian_scales.drift_sum, 0.0);
    assert_eq!(lamarckian_scales.at_bound, 1);
}

/// The retention bins (T25.F09 invariant 7) are exhaustive over `[0, 1]`:
/// exactly 1 is `none`, the largest f32 below 1 is in `1e-3`, 0 in `1`, and
/// each decade boundary's two f32 neighbors fall on either side by their
/// exact deficit.
#[test]
fn retention_bins_cover_every_admitted_retention() {
    assert_eq!(RETENTION_KEYS, ["none", "1e-3", "1e-2", "1e-1", "1"]);
    assert_eq!(retention_index(1.0), 0);
    assert_eq!(retention_index(f32::from_bits(1.0f32.to_bits() - 1)), 1);
    assert_eq!(retention_index(0.0), 4);
    for (decade, below, at_or_above) in [(1e-3f64, 1, 2), (1e-2, 2, 3), (1e-1, 3, 4)] {
        // The f32 nearest `1 − 10⁻ᵏ` and its neighbors bracket the boundary.
        let near = (1.0 - decade) as f32;
        let candidates = [
            f32::from_bits(near.to_bits() - 1),
            near,
            f32::from_bits(near.to_bits() + 1),
        ];
        let mut sides = [false, false];
        for retention in candidates {
            let deficit = 1.0 - f64::from(retention);
            let expected = if deficit < decade { below } else { at_or_above };
            sides[usize::from(deficit >= decade)] = true;
            assert_eq!(retention_index(retention), expected, "{retention}");
        }
        assert_eq!(sides, [true, true], "both sides of {decade} are tested");
    }
}

/// An edge and a scale slot land in their retention bin: `count` every
/// entry, `initialized`, drift from birth and at-bound over initialized
/// ones only.
#[test]
fn edges_and_scales_are_censused_by_retention_bin_with_birth_drift() {
    use crate::contracts::{InputReference, WorldInputKey};
    use crate::creature::genome::cgp::{Scope, Shape};
    use crate::creature::genome::vote::DirectedVoteKind;

    for (retention, bin) in [
        (1.0, 0),
        (f32::from_bits(1.0f32.to_bits() - 1), 1),
        (0.995, 2),
        (0.95, 3),
        (0.0, 4),
    ] {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        let id = def.add_group(Shape::Ring { ring_ref: 0 }, DirectedVoteKind::Move, 0);
        def.group_mut(id).set_scale(0.5);
        def.group_mut(id).set_retention(retention);
        let members: Vec<EdgeOccurrence> = def.members(id).collect();
        let config = PlasticityConfig {
            retention,
            ..plastic(HebbianRule::Classic, false, false)
        };
        for (k, scope) in [Scope::Edge, Scope::Group].into_iter().enumerate() {
            let mut edge = def.edge_mut(members[k]);
            edge.set_plasticity(Some(config));
            edge.set_scope(scope);
        }
        let genome = CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![InputReference::World(WorldInputKey::NeighborBarrierRing)],
                backend_def: BackendDef::Graph(def),
                targets: vec![],
            }],
        };
        let unvisited = GraphRuntimeState::for_genome(&genome, Vec::new());
        let census = PlasticityCensus::observe([(&[0usize][..], unvisited.learned())]);
        let mut expected = [RetentionCell::default(); 5];
        expected[bin] = RetentionCell {
            count: 2,
            ..RetentionCell::default()
        };
        assert_eq!(census.edges_by_retention, expected, "{retention}");
        expected[bin].count = 1;
        assert_eq!(census.scales_by_retention, expected, "{retention}");

        let mut visited = GraphRuntimeState::for_genome(&genome, Vec::new());
        visited.learned.set_slot(0, 0, -2.0, 0.0);
        visited.learned.set_scale(0, 0, 1.25);
        let census = PlasticityCensus::observe([(&[0usize][..], visited.learned())]);
        let edges = census.edges_by_retention[bin];
        // The group learner counts but has no learned own quantity.
        assert_eq!((edges.count, edges.initialized, edges.at_bound), (2, 1, 1));
        assert_eq!(edges.drift_sum, 2.0, "|-2 - 0| from the birth correction");
        let scales = census.scales_by_retention[bin];
        assert_eq!(
            (scales.count, scales.initialized, scales.at_bound),
            (1, 1, 0)
        );
        assert_eq!(scales.drift_sum, 0.75);
        assert_eq!(census.scales[0].drift_sum, 0.75);
    }
}
