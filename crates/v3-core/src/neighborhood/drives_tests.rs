use super::*;
use crate::config::FounderProfile;
use crate::contracts::NodeId;
use crate::creature::founder::founder_genome;
use crate::genome::analysis::mesh_reachable_nodes;
use crate::genome::cgp::GraphEdge;
use crate::genome::{NodeGenome, VmBackendDef};

fn census_of(genomes: &[CreatureGenome]) -> DriveCensus {
    let reachable: Vec<Vec<usize>> = genomes.iter().map(mesh_reachable_nodes).collect();
    DriveCensus::observe(
        genomes
            .iter()
            .zip(&reachable)
            .map(|(genome, reachable)| (genome, reachable.as_slice())),
    )
}

fn memory(slot: u8) -> GraphSource {
    GraphSource::SharedMemory {
        slot,
        previous: false,
    }
}

fn wire(def: &mut CgpGraphBackendDef, sink: VoteSink, source: GraphSource, weight: f32) {
    let surface = def.sink_surface(OutputSinkKind::ActionVote(sink));
    def.push_edge(surface, GraphEdge::new(source, weight));
}

fn graph_genome(def: CgpGraphBackendDef) -> CreatureGenome {
    CreatureGenome::new(
        NodeId::new(0),
        vec![NodeGenome::new(
            NodeId::new(0),
            vec![],
            BackendDef::Graph(def),
            vec![],
        )],
        crate::creature::founder::default_founder_body(),
    )
}

#[test]
fn the_founder_carries_four_fanouts_of_sixteen_edges_and_no_drive() {
    for profile in [FounderProfile::default()] {
        let census = census_of(&[founder_genome(
            profile,
            crate::creature::founder::default_founder_body(),
        )]);
        assert_eq!(census.fanout_totals(), (4, 16), "{profile:?}");
        let [moves, reproduce, steal, give] = census.kinds;
        assert_eq!((moves.fanouts, moves.fanout_edges), (3, 12));
        assert_eq!((reproduce.fanouts, reproduce.fanout_edges), (1, 4));
        assert_eq!((steal.fanouts, steal.fanout_edges), (0, 0));
        assert_eq!((give.fanouts, give.fanout_edges), (0, 0));
        for row in census.kinds {
            assert_eq!(row.drive_carriers, 0);
            assert_eq!(row.drive_edges + row.drive_vm_paths, 0);
        }
    }
}

#[test]
fn drive_edges_are_signed_and_floored_and_vm_paths_unsigned() {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let drive = DirectedVoteKind::Move.drive();
    wire(&mut def, drive, memory(0), 0.5);
    wire(&mut def, drive, memory(1), -0.25);
    wire(&mut def, drive, memory(2), 1e-4);
    // A second positive edge makes the sign split asymmetric.
    wire(&mut def, drive, memory(3), 0.75);
    let graph = graph_genome(def);
    let vm = CreatureGenome::new(
        NodeId::new(0),
        vec![NodeGenome::new(
            NodeId::new(0),
            vec![],
            BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![
                    VmInstruction::AddVote {
                        sink: DirectedVoteKind::StealEnergy.drive().index() as u8,
                        src: 0,
                    },
                    VmInstruction::AddVote {
                        sink: VoteSink::Directed(DirectedVoteKind::StealEnergy, 3).index() as u8,
                        src: 0,
                    },
                ],
            }),
            vec![],
        )],
        crate::creature::founder::default_founder_body(),
    );
    let empty = CreatureGenome::new(
        NodeId::new(0),
        vec![],
        crate::creature::founder::default_founder_body(),
    );
    let census = census_of(&[graph, vm, empty]);
    assert_eq!(census.creatures, 3);
    let moves = census.kinds[DirectedVoteKind::Move.index()];
    assert_eq!(
        (
            moves.drive_carriers,
            moves.drive_edges,
            moves.drive_edges_positive,
            moves.drive_edges_negative,
            moves.drive_vm_paths
        ),
        (1, 3, 2, 1, 0)
    );
    let steal = census.kinds[DirectedVoteKind::StealEnergy.index()];
    assert_eq!((steal.drive_carriers, steal.drive_vm_paths), (1, 1));
    assert_eq!(
        census.kinds[DirectedVoteKind::Reproduce.index()],
        KindDrives::default()
    );
}

#[test]
fn a_fanout_needs_one_source_and_sign_into_two_direction_sinks_on_one_node() {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    // Fan-out: memory(0) positive into Move(0) twice and Move(2): 3 edges.
    wire(
        &mut def,
        VoteSink::Directed(DirectedVoteKind::Move, 0),
        memory(0),
        0.5,
    );
    wire(
        &mut def,
        VoteSink::Directed(DirectedVoteKind::Move, 0),
        memory(0),
        0.7,
    );
    wire(
        &mut def,
        VoteSink::Directed(DirectedVoteKind::Move, 2),
        memory(0),
        0.5,
    );
    // Not one: the opposite sign into a third direction, a lone edge, the
    // drive, another kind, and a floored weight.
    wire(
        &mut def,
        VoteSink::Directed(DirectedVoteKind::Move, 4),
        memory(0),
        -0.5,
    );
    wire(
        &mut def,
        VoteSink::Directed(DirectedVoteKind::Move, 5),
        memory(1),
        0.5,
    );
    wire(&mut def, DirectedVoteKind::Move.drive(), memory(1), 0.5);
    wire(
        &mut def,
        VoteSink::Directed(DirectedVoteKind::Reproduce, 5),
        memory(1),
        0.5,
    );
    wire(
        &mut def,
        VoteSink::Directed(DirectedVoteKind::Move, 6),
        memory(1),
        1e-4,
    );
    let census = census_of(&[graph_genome(def)]);
    let moves = census.kinds[DirectedVoteKind::Move.index()];
    assert_eq!(
        (moves.fanout_creatures, moves.fanouts, moves.fanout_edges),
        (1, 1, 3)
    );
    assert_eq!(census.kinds[DirectedVoteKind::Reproduce.index()].fanouts, 0);
}

#[test]
fn unreachable_nodes_are_not_read() {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    wire(&mut def, DirectedVoteKind::Move.drive(), memory(0), 0.5);
    let genome = graph_genome(def);
    let census = DriveCensus::observe([(&genome, &[][..])]);
    assert_eq!(census.creatures, 1);
    assert_eq!(census.kinds[0], KindDrives::default());
}

#[test]
fn a_weight_exactly_at_the_floor_carries_no_path() {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let drive = DirectedVoteKind::Move.drive();
    wire(&mut def, drive, memory(0), PATH_WEIGHT_FLOOR);
    wire(&mut def, drive, memory(1), -PATH_WEIGHT_FLOOR);
    wire(
        &mut def,
        VoteSink::Directed(DirectedVoteKind::Move, 0),
        memory(2),
        PATH_WEIGHT_FLOOR,
    );
    wire(
        &mut def,
        VoteSink::Directed(DirectedVoteKind::Move, 1),
        memory(2),
        PATH_WEIGHT_FLOOR,
    );
    let census = census_of(&[graph_genome(def)]);
    assert_eq!(
        census.kinds[DirectedVoteKind::Move.index()],
        KindDrives::default(),
        "the floor is exclusive: |w| must exceed it"
    );
}

/// A bundle is not a fan-out (T25.F03): two `Move` bundles reading the same
/// ring channels at different offsets send each channel to two directions,
/// which as independent edges is eight fan-outs, and as members is none.
#[test]
fn a_bundle_is_not_a_fanout() {
    use crate::contracts::{InputReference, WorldInputKey};
    let mut grouped = CgpGraphBackendDef::new_with_fixed_outputs();
    for offset in [0, 3] {
        let id = grouped.add_ring_bundle(0, DirectedVoteKind::Move, offset);
        grouped.bundle_mut(id).set_scale(1.0);
    }
    let mut independent = CgpGraphBackendDef::new_with_fixed_outputs();
    for offset in [0u8, 3] {
        for channel in 0..8u8 {
            wire(
                &mut independent,
                VoteSink::Directed(DirectedVoteKind::Move, (channel + offset) % 8),
                GraphSource::InputLeaf {
                    ref_idx: 0,
                    sub_idx: u16::from(channel),
                },
                1.0,
            );
        }
    }
    let ring = vec![InputReference::World(WorldInputKey::NeighborBarrierRing)];
    let with_refs = |def| {
        let mut genome = graph_genome(def);
        *genome.nodes[0].input_refs_mut() = ring.clone();
        genome
    };
    let moves = |def| census_of(&[with_refs(def)]).kinds[DirectedVoteKind::Move.index()];
    let edges = moves(independent);
    assert_eq!((edges.fanouts, edges.fanout_edges), (8, 16));
    let members = moves(grouped);
    assert_eq!((members.fanouts, members.fanout_edges), (0, 0));
}

/// A `DirectionPair` bundle's members (T25.F04) send one source to six
/// directions each, yet are no fan-out; its facts edges into a drive are
/// drive edges once they carry.
#[test]
fn pair_members_are_not_a_fanout() {
    use crate::contracts::{InputReference, WorldInputKey};
    let area = InputReference::World(WorldInputKey::AreaOccupancySummary);
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    let id = def.add_pair_bundle(
        0,
        0,
        crate::contracts::pair_layout(&area, 0),
        DirectedVoteKind::Move,
        2,
    );
    def.bundle_mut(id).set_scale(1.0);
    let mut genome = graph_genome(def);
    *genome.nodes[0].input_refs_mut() = vec![area];
    let moves = census_of(&[genome]).kinds[DirectedVoteKind::Move.index()];
    assert_eq!((moves.fanouts, moves.fanout_edges), (0, 0));
}
