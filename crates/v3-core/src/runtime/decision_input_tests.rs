//! T19.F05 worked cases V1 to V4 and V6: the in-tick decision-state inputs
//! on hand-built vote genomes, each run in all three execution modes and
//! against a control whose read is replaced by the constant 0. V5, the
//! frozen previous outcome, runs on the simulation (`simulation/tick/tests`).

use crate::config::{OrdinaryFoodTypeId, RuntimeConfig};
use crate::contracts::{Direction, DynamicIntrospectionKey, InputReference, WorldAction};
use crate::creature::genome::cgp::GraphSource;
use crate::creature::genome::vote::{VoteKind, VoteSink};
use crate::creature::genome::VmInstruction;
use crate::runtime::trace::domain::{PassEndReason, TerminationReason};
use crate::runtime::vote_test_support::{
    genome, leaf, run_tick, vm_node, vm_voter, GraphBuilder, Senses,
};

const N: u8 = 0;

fn eat() -> WorldAction {
    WorldAction::eat(OrdinaryFoodTypeId::default())
}

fn move_n() -> WorldAction {
    WorldAction::Move(Direction::N)
}

fn config() -> RuntimeConfig {
    RuntimeConfig::default()
}

/// `dst = input_refs[0][sub_idx]`, or `dst = 0` from constant `zero_const`
/// in the control.
fn read_or_zero(read: bool, dst: u8, sub_idx: u16, zero_const: u8) -> VmInstruction {
    if read {
        VmInstruction::ReadInput {
            dst,
            ref_idx: 0,
            sub_idx,
        }
    } else {
        VmInstruction::LoadConst {
            dst,
            const_idx: zero_const,
        }
    }
}

/// Graph leaf on `input_refs[0][sub_idx]`, or a constant-0 node in the
/// control.
fn leaf_or_zero(graph: &mut GraphBuilder, read: bool, sub_idx: u16) -> GraphSource {
    if read {
        leaf(0, sub_idx)
    } else {
        graph.constant(0.0)
    }
}

fn eat_sub() -> u16 {
    VoteSink::Eat.index() as u16
}

/// V1: A votes `Eat 1` and routes to B; B votes `Move N = 1 -
/// ActionVotes[Eat]`, so A's committed vote inhibits B's.
fn v1_lateral_inhibition(read: bool) -> crate::creature::genome::CreatureGenome {
    let a = vm_voter(0, &[(VoteSink::Eat, 1.0)], &[1]);
    let b = vm_node(
        1,
        3,
        vec![1.0, 0.0],
        vec![
            read_or_zero(read, 0, eat_sub(), 1),
            VmInstruction::LoadConst {
                dst: 1,
                const_idx: 0,
            },
            VmInstruction::Sub { dst: 2, a: 1, b: 0 },
            VmInstruction::AddVote {
                sink: VoteSink::Move(N).index() as u8,
                src: 2,
            },
            VmInstruction::Halt,
        ],
        vec![InputReference::ActionVotes],
        &[],
    );
    genome(vec![a, b])
}

#[test]
fn v1_action_votes_let_a_committed_vote_inhibit_a_later_node() {
    let control = run_tick(
        &v1_lateral_inhibition(false),
        Senses::default(),
        &config(),
        20.0,
        [0.0; 16],
    );
    assert_eq!(control.actions(), &[eat(), move_n()]);
    assert_eq!(control.committed(), vec![Some(eat()), Some(move_n()), None]);

    let tick = run_tick(
        &v1_lateral_inhibition(true),
        Senses::default(),
        &config(),
        20.0,
        [0.0; 16],
    );
    assert_eq!(tick.actions(), &[eat()]);
    assert_eq!(tick.committed(), vec![Some(eat()), None]);
    assert_eq!(
        tick.output.termination_reason,
        TerminationReason::NoDecision
    );
    // B read A's committed Eat vote: its Move N vote is 1 - 1 = 0.
    assert_eq!(tick.passes[0].votes[VoteSink::Move(N).index()], 0.0);
}

/// V2: one graph node votes `Eat 1` and `Move N = PreviousPassVotes[Eat]`.
fn v2_decision_history(read: bool) -> crate::creature::genome::CreatureGenome {
    let mut graph = GraphBuilder::new();
    let one = graph.constant(1.0);
    let previous_eat = leaf_or_zero(&mut graph, read, eat_sub());
    graph.vote(VoteSink::Eat, &[(one, 1.0)]);
    graph.vote(VoteSink::Move(N), &[(previous_eat, 1.0)]);
    genome(vec![graph.build(
        0,
        vec![InputReference::PreviousPassVotes],
        &[],
    )])
}

#[test]
fn v2_previous_pass_votes_carry_the_last_pass_decision_into_the_next() {
    let control = run_tick(
        &v2_decision_history(false),
        Senses::default(),
        &config(),
        20.0,
        [0.0; 16],
    );
    assert_eq!(control.actions(), &[eat()]);

    let tick = run_tick(
        &v2_decision_history(true),
        Senses::default(),
        &config(),
        20.0,
        [0.0; 16],
    );
    assert_eq!(tick.actions(), &[eat(), move_n()]);
    assert_eq!(tick.committed(), vec![Some(eat()), Some(move_n()), None]);
    // Pass one reads zeros; pass two reads pass one's Eat vote.
    assert_eq!(tick.passes[0].votes[VoteSink::Move(N).index()], 0.0);
    assert_eq!(tick.passes[1].votes[VoteSink::Move(N).index()], 1.0);
    assert_eq!(
        tick.output.termination_reason,
        TerminationReason::NoDecision
    );
}

/// V3: one graph node votes `Eat = 2 - CommitCounts[Eat]`.
fn v3_bar_aware_plan(read: bool) -> crate::creature::genome::CreatureGenome {
    let mut graph = GraphBuilder::new();
    let one = graph.constant(1.0);
    let eat_bar = leaf_or_zero(&mut graph, read, VoteKind::Eat.index() as u16);
    graph.vote(VoteSink::Eat, &[(one, 2.0), (eat_bar, -1.0)]);
    genome(vec![graph.build(
        0,
        vec![InputReference::CommitCounts],
        &[],
    )])
}

#[test]
fn v3_commit_counts_let_a_plan_stop_after_one_commit() {
    let control = run_tick(
        &v3_bar_aware_plan(false),
        Senses::default(),
        &config(),
        20.0,
        [0.0; 16],
    );
    assert_eq!(control.actions(), &[eat(), eat()]);

    let tick = run_tick(
        &v3_bar_aware_plan(true),
        Senses::default(),
        &config(),
        20.0,
        [0.0; 16],
    );
    assert_eq!(tick.actions(), &[eat()]);
    // Pass two reads the bar 1: Eat votes 1, effective 0.
    assert_eq!(tick.passes[1].votes[VoteSink::Eat.index()], 1.0);
    assert_eq!(tick.passes[1].effective_votes[VoteKind::Eat.index()], 0.0);
    assert_eq!(tick.output.commit_counts, [1, 0, 0, 0]);
}

/// V4: a self-routing VM node, no `Decide`, votes `Move N 2` and
/// `Terminate = HopsThisTick - 64`.
fn v4_deliberation_cost(read: bool) -> crate::creature::genome::CreatureGenome {
    genome(vec![vm_node(
        0,
        3,
        vec![2.0, 64.0, 0.0],
        vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            VmInstruction::AddVote {
                sink: VoteSink::Move(N).index() as u8,
                src: 0,
            },
            read_or_zero(read, 1, 0, 2),
            VmInstruction::LoadConst {
                dst: 2,
                const_idx: 1,
            },
            VmInstruction::Sub { dst: 1, a: 1, b: 2 },
            VmInstruction::AddVote {
                sink: VoteSink::Terminate.index() as u8,
                src: 1,
            },
            VmInstruction::Halt,
        ],
        vec![InputReference::DynamicIntrospection(
            DynamicIntrospectionKey::HopsThisTick,
        )],
        &[0],
    )])
}

#[test]
fn v4_hops_this_tick_lets_deliberation_end_the_tick() {
    let config = RuntimeConfig {
        max_mesh_hops: 64,
        max_actions_per_turn: 10,
        ..config()
    };
    let control = run_tick(
        &v4_deliberation_cost(false),
        Senses::default(),
        &config,
        100.0,
        [0.0; 16],
    );
    assert_eq!(control.actions(), &[move_n(), move_n()]);
    assert_eq!(control.output.work_counters.mesh_hops, 192);
    assert_eq!(
        control.output.termination_reason,
        TerminationReason::NoDecision
    );
    assert_eq!(control.passes[2].votes[VoteSink::Terminate.index()], -64.0);

    let tick = run_tick(
        &v4_deliberation_cost(true),
        Senses::default(),
        &config,
        100.0,
        [0.0; 16],
    );
    assert_eq!(tick.actions(), &[move_n()]);
    assert_eq!(tick.output.work_counters.mesh_hops, 128);
    assert_eq!(
        tick.output.termination_reason,
        TerminationReason::TerminateVoted
    );
    // Pass one's last dispatch is hop 64: Terminate 0. Pass two's is hop
    // 128: Terminate 64 against Move's effective 2 - 1 = 1.
    assert_eq!(tick.passes[0].votes[VoteSink::Terminate.index()], 0.0);
    assert_eq!(tick.passes[1].end_reason, PassEndReason::PassCapReached);
    assert_eq!(tick.passes[1].votes[VoteSink::Terminate.index()], 64.0);
    assert_eq!(tick.passes[1].effective_votes[VoteKind::Move.index()], 1.0);
}

/// V6: a self-targeting VM node, no `Decide`, reads `ActionVotes[Eat]`
/// before and after its own `AddVote Eat 1`, writes the two reads to bus
/// slots 0 and 1, and sums them into shared-memory slots 0 and 1 so every
/// execution mode's reads are compared.
fn v6_staging_boundary() -> crate::creature::genome::CreatureGenome {
    let mut program = Vec::new();
    for (step, slot) in [(0u8, 0u8), (1, 1)] {
        if step == 1 {
            program.extend([
                VmInstruction::LoadConst {
                    dst: 0,
                    const_idx: 0,
                },
                VmInstruction::AddVote {
                    sink: VoteSink::Eat.index() as u8,
                    src: 0,
                },
            ]);
        }
        program.extend([
            VmInstruction::ReadInput {
                dst: 1,
                ref_idx: 0,
                sub_idx: eat_sub(),
            },
            VmInstruction::WriteInternalPayload {
                slot_idx: slot,
                src: 1,
            },
            VmInstruction::LoadSlotImm {
                dst: 2,
                slot_idx: slot,
            },
            VmInstruction::Add { dst: 2, a: 2, b: 1 },
            VmInstruction::StoreSlotImm {
                slot_idx: slot,
                src: 2,
            },
        ]);
    }
    program.push(VmInstruction::Halt);
    genome(vec![vm_node(
        0,
        3,
        vec![1.0],
        program,
        vec![InputReference::ActionVotes],
        &[0],
    )])
}

#[test]
fn v6_a_dispatch_never_reads_the_vote_it_is_staging() {
    let tick = run_tick(
        &v6_staging_boundary(),
        Senses::default(),
        &config(),
        20.0,
        [0.0; 16],
    );
    assert_eq!(tick.actions(), &[eat()]);
    assert_eq!(tick.committed(), vec![Some(eat()), None]);
    assert_eq!(
        tick.output.termination_reason,
        TerminationReason::NoDecision
    );
    assert!(tick
        .passes
        .iter()
        .all(|pass| pass.hops == 64 && pass.end_reason == PassEndReason::PassCapReached));
    let mut first_of_pass = 0;
    for hop in &tick.hops {
        let first = hop.hop_index % 64 == 0;
        first_of_pass += usize::from(first);
        let expected = if first { 0.0 } else { 1.0 };
        assert_eq!(
            [hop.output_slots[0], hop.output_slots[1]],
            [expected, expected],
            "hop {}",
            hop.hop_index
        );
    }
    assert_eq!(first_of_pass, 2);
    // `run_tick` asserted every mode ended with the same memory, so the
    // untraced reads summed the same 126 ones.
    assert_eq!([tick.memory[0], tick.memory[1]], [126.0, 126.0]);
}
