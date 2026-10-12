//! Kind drives and the heading tie at the mesh level (T11.F30): every case
//! runs in the plain, observed and traced modes, which agree on the queue,
//! the termination reason and the heading.

use crate::config::RuntimeConfig;
use crate::contracts::{Direction, WorldAction};
use crate::creature::genome::vote::{DirectedVoteKind, VoteKind, VoteSink};
use crate::runtime::trace::domain::TerminationReason;
use crate::runtime::vote_test_support::{genome, run_tick_heading, vm_voter, GraphBuilder, Senses};

fn config() -> RuntimeConfig {
    RuntimeConfig {
        max_actions_per_turn: 4,
        ..RuntimeConfig::default()
    }
}

const DRIVE_MOVE: VoteSink = VoteSink::Drive(DirectedVoteKind::Move);
const DRIVE_REPRODUCE: VoteSink = VoteSink::Drive(DirectedVoteKind::Reproduce);

#[test]
fn a_flat_channel_with_a_positive_drive_moves_along_the_heading() {
    let genome = genome(vec![vm_voter(0, &[(DRIVE_MOVE, 0.5)], &[])]);
    for heading in Direction::ALL {
        let tick = run_tick_heading(
            &genome,
            Senses::default(),
            &config(),
            100.0,
            [0.0; 16],
            heading,
        );
        assert_eq!(tick.actions(), [WorldAction::Move(heading)]);
        assert_eq!(tick.output.heading, heading);
        assert_eq!(
            tick.passes[0].best[VoteKind::Move.index()],
            VoteSink::Move(heading.to_index() as u8)
        );
        assert_eq!(tick.passes[0].effective_votes[VoteKind::Move.index()], 0.5);
    }
}

#[test]
fn a_move_commit_turns_the_heading_for_the_later_passes_of_the_tick() {
    // Pass 1 commits `Move(E)`; pass 2's flat `Reproduce` channel ties and
    // the drive commits it toward the new heading, not the starting `N`.
    let genome = genome(vec![vm_voter(
        0,
        &[(VoteSink::Move(2), 1.0), (DRIVE_REPRODUCE, 0.5)],
        &[],
    )]);
    let tick = run_tick_heading(
        &genome,
        Senses::default(),
        &config(),
        100.0,
        [0.0; 16],
        Direction::N,
    );
    assert_eq!(
        tick.actions()[..2],
        [
            WorldAction::Move(Direction::E),
            WorldAction::Reproduce {
                direction: Direction::E,
                energy_transfer_fraction: 0.0,
            },
        ]
    );
    assert_eq!(tick.output.heading, Direction::E);
    assert_eq!(tick.passes[1].bars, [0, 1, 0, 0]);
    assert_eq!(
        tick.passes[1].best[VoteKind::Reproduce.index()],
        VoteSink::Reproduce(2)
    );
}

#[test]
fn only_a_move_commit_changes_the_heading() {
    let voters = [
        vec![(VoteSink::Eat, 1.0)],
        vec![(VoteSink::Reproduce(3), 1.0)],
        vec![(VoteSink::StealEnergy(5), 1.0)],
        vec![],
    ];
    for votes in voters {
        let genome = genome(vec![vm_voter(0, &votes, &[])]);
        let tick = run_tick_heading(
            &genome,
            Senses::default(),
            &config(),
            100.0,
            [0.0; 16],
            Direction::SW,
        );
        assert!(!tick
            .actions()
            .iter()
            .any(|a| matches!(a, WorldAction::Move(_))));
        assert_eq!(tick.output.heading, Direction::SW, "{votes:?}");
    }
    let genome = genome(vec![vm_voter(0, &[(VoteSink::Move(6), 1.0)], &[])]);
    let tick = run_tick_heading(
        &genome,
        Senses::default(),
        &config(),
        100.0,
        [0.0; 16],
        Direction::SW,
    );
    assert_eq!(tick.output.heading, Direction::W);
}

#[test]
fn a_graph_drive_edge_and_a_negative_drive_gate_the_kind() {
    // One Graph edge into the drive adds to the best direction vote; a
    // negative drive suppresses a kind whose direction vote alone would win.
    let mut graph = GraphBuilder::new();
    let one = graph.constant(1.0);
    graph
        .vote(VoteSink::Move(4), &[(one, 0.3)])
        .vote(DRIVE_MOVE, &[(one, 0.4)]);
    let genome_on = genome(vec![graph.build(0, vec![], &[])]);
    let tick = run_tick_heading(
        &genome_on,
        Senses::default(),
        &config(),
        100.0,
        [0.0; 16],
        Direction::N,
    );
    assert_eq!(tick.actions(), [WorldAction::Move(Direction::S)]);
    assert_eq!(tick.passes[0].votes[DRIVE_MOVE.index()], 0.4);
    assert_eq!(
        tick.passes[0].effective_votes[VoteKind::Move.index()],
        0.4 + 0.3
    );

    let mut graph = GraphBuilder::new();
    let one = graph.constant(1.0);
    graph
        .vote(VoteSink::Move(4), &[(one, 0.3)])
        .vote(DRIVE_MOVE, &[(one, -0.4)]);
    let genome_off = genome(vec![graph.build(0, vec![], &[])]);
    let tick = run_tick_heading(
        &genome_off,
        Senses::default(),
        &config(),
        100.0,
        [0.0; 16],
        Direction::N,
    );
    assert_eq!(tick.actions(), [WorldAction::NoOp]);
    assert_eq!(
        tick.output.termination_reason,
        TerminationReason::NoDecision
    );
}

#[test]
fn a_drive_alone_never_commits_without_its_kind_and_decodes_nothing() {
    // `Drive(Reproduce)` raises only `Reproduce`; the drive sink itself is
    // never the committed sink.
    let mut graph = GraphBuilder::new();
    let one = graph.constant(1.0);
    graph.vote(DRIVE_REPRODUCE, &[(one, 0.7)]);
    let genome = genome(vec![graph.build(0, vec![], &[])]);
    let tick = run_tick_heading(
        &genome,
        Senses::default(),
        &config(),
        100.0,
        [0.0; 16],
        Direction::NE,
    );
    assert_eq!(
        tick.actions(),
        [WorldAction::Reproduce {
            direction: Direction::NE,
            energy_transfer_fraction: 0.0,
        }]
    );
    assert!(tick.passes.iter().all(|pass| pass
        .best
        .iter()
        .all(|sink| !matches!(sink, VoteSink::Drive(_)))));
}
