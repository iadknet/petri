//! T19.F04 acceptance fixtures W1 to W19 (mesh action-selection review,
//! Section 2.3): the pass loop on hand-built vote genomes, every case run in
//! all three execution modes.

use crate::config::{OrdinaryFoodTypeId, RuntimeConfig};
use crate::contracts::{
    Direction, DynamicIntrospectionKey, InputReference, WorldAction, WorldInputKey,
};
use crate::creature::genome::cgp::{ComputeNodeKind, GraphSource};
use crate::creature::genome::vote::{VoteKind, VoteSink};
use crate::creature::genome::{CreatureGenome, VmInstruction};
use crate::runtime::trace::domain::{PassEndReason, TerminationReason};
use crate::runtime::vote_test_support::{
    genome, leaf, run_tick, vm_node, vm_voter, GraphBuilder, Senses,
};
use crate::simulation::energy_accounting::DeathCause;

const N: u8 = 0;
const E: u8 = 2;
const W: u8 = 6;

fn food_here() -> InputReference {
    InputReference::World(WorldInputKey::food_here(OrdinaryFoodTypeId::default()))
}

fn food_ring() -> InputReference {
    InputReference::World(WorldInputKey::neighbor_food_ring(
        OrdinaryFoodTypeId::default(),
    ))
}

fn occupied_ring() -> InputReference {
    InputReference::World(WorldInputKey::NeighborOccupiedRing)
}

fn eat() -> WorldAction {
    WorldAction::eat(OrdinaryFoodTypeId::default())
}

fn mv(d: u8) -> WorldAction {
    WorldAction::Move(Direction::ALL[usize::from(d)])
}

fn config() -> RuntimeConfig {
    RuntimeConfig::default()
}

/// The W1 ring: N 0.2, E 0.9, S 0.1, W 0.4.
fn w1_senses(food: f32) -> Senses {
    let mut senses = Senses {
        food_here: food,
        ..Senses::default()
    };
    for (d, value) in [(0, 0.2), (2, 0.9), (4, 0.1), (6, 0.4)] {
        senses.food[d] = value;
    }
    senses
}

/// The founder forage votes on one graph node reading `FoodHere` (ref 0) and
/// the food ring (ref 1): `Eat = [food_here > 0]`, `Move[d] = 0.5 +
/// 0.4·ring[d]` on the cardinals. Extra input references follow.
fn forage_graph(extra_refs: Vec<InputReference>) -> (GraphBuilder, Vec<InputReference>) {
    let mut graph = GraphBuilder::new();
    let bias = graph.constant(1.0);
    let food = graph.node(ComputeNodeKind::Threshold(0.0), &[(leaf(0, 0), 1.0)]);
    graph.vote(VoteSink::Eat, &[(food, 1.0)]);
    for d in [0u8, 2, 4, 6] {
        graph.vote(
            VoteSink::Move(d),
            &[(bias, 0.5), (leaf(1, u16::from(d)), 0.4)],
        );
    }
    let mut refs = vec![food_here(), food_ring()];
    refs.extend(extra_refs);
    (graph, refs)
}

#[test]
fn w1_founder_forage_with_food_is_eat_then_the_best_move() {
    let (graph, refs) = forage_graph(vec![]);
    let g = genome(vec![graph.build(0, refs, &[])]);
    let tick = run_tick(&g, w1_senses(1.0), &config(), 20.0, [0.0; 16]);
    assert_eq!(tick.actions(), &[eat(), mv(E)]);
    assert_eq!(tick.committed(), vec![Some(eat()), Some(mv(E)), None]);
    assert_eq!(
        tick.output.termination_reason,
        TerminationReason::NoDecision
    );
    assert!(tick
        .passes
        .iter()
        .all(|pass| pass.end_reason == PassEndReason::NoTargets));
    // Pass 2: Eat's bar is 1, Move E 0.86 wins; pass 3: Move −0.14.
    assert_eq!(tick.passes[1].effective_votes[VoteKind::Eat.index()], 0.0);
    assert!((tick.passes[2].effective_votes[VoteKind::Move.index()] + 0.14).abs() < 1e-6);
    assert_eq!(tick.output.commit_counts, [1, 1, 0, 0]);
}

#[test]
fn w2_founder_forage_without_food_is_the_best_move() {
    let (graph, refs) = forage_graph(vec![]);
    let g = genome(vec![graph.build(0, refs, &[])]);
    let tick = run_tick(&g, w1_senses(0.0), &config(), 20.0, [0.0; 16]);
    assert_eq!(tick.actions(), &[mv(E)]);
    assert_eq!(tick.output.work_counters.passes, 2);
}

#[test]
fn w3_founder_reproduce_tick_commits_once_whatever_the_live_gate_reads() {
    use crate::config::FounderProfile;
    use crate::creature::founder::founder_genome;
    let g = founder_genome(FounderProfile::V3Alpha1);
    let mut senses = w1_senses(1.0);
    senses.age = 1.0;
    // Energy just above the 0.16 gate at 200 max: each graph visit costs
    // about 6.5e-5, so node 0 reads the gate open in pass 1 and closed in
    // pass 2. Well above the gate it stays open. Either way the queue-reading
    // `q` closes the reproduce branch after one commit.
    for (energy, gate_open_in_pass_2) in [(32.0001, false), (60.0, true)] {
        let tick = run_tick(&g, senses, &config(), energy, [0.0; 16]);
        assert_eq!(
            tick.actions(),
            &[WorldAction::Reproduce {
                direction: Direction::E,
                energy_transfer_fraction: 2.0 / 3.0,
            }],
            "energy {energy}"
        );
        assert_eq!(tick.output.work_counters.passes, 2);
        let second = &tick.passes[1];
        assert!(second.effective_votes[VoteKind::Eat.index()] <= 0.0);
        assert!(second.effective_votes[VoteKind::Move.index()] <= 0.0);
        assert_eq!(second.committed, None);
        // Reproduce E in pass 2: g·0.86 − 1, so −1 once the gate has closed.
        let reproduce = second.effective_votes[VoteKind::Reproduce.index()];
        assert_eq!(reproduce == -1.0, !gate_open_in_pass_2, "energy {energy}");
    }
}

#[test]
fn w4_one_edge_into_an_unused_kind_adds_exactly_one_action() {
    let (mut graph, refs) = forage_graph(vec![occupied_ring()]);
    graph.vote(VoteSink::StealEnergy(E), &[(leaf(2, u16::from(E)), 1.0)]);
    let g = genome(vec![graph.build(0, refs, &[])]);
    let mut senses = w1_senses(0.0);
    senses.occupied[usize::from(E)] = 1.0;
    let tick = run_tick(&g, senses, &config(), 20.0, [0.0; 16]);
    assert_eq!(
        tick.actions(),
        &[
            WorldAction::StealEnergy {
                direction: Direction::E,
                amount: 0.0,
            },
            mv(E),
        ]
    );
}

/// `Eat 1.0, Move W 0.9` plus one edge from the food ring at `edge_d` into
/// `Move(edge_d)`, with food at `edge_d`.
fn static_eat_and_west_plus_food_edge(edge_d: u8) -> Vec<WorldAction> {
    let mut graph = GraphBuilder::new();
    let one = graph.constant(1.0);
    graph.vote(VoteSink::Eat, &[(one, 1.0)]);
    graph.vote(VoteSink::Move(W), &[(one, 0.9)]);
    graph.vote(VoteSink::Move(edge_d), &[(leaf(0, u16::from(edge_d)), 1.0)]);
    let g = genome(vec![graph.build(0, vec![food_ring()], &[])]);
    let mut senses = Senses::default();
    senses.food[usize::from(edge_d)] = 1.0;
    run_tick(&g, senses, &config(), 20.0, [0.0; 16])
        .actions()
        .to_vec()
}

#[test]
fn w5_one_edge_into_a_used_kind_on_another_sink_steers() {
    assert_eq!(static_eat_and_west_plus_food_edge(E), vec![eat(), mv(E)]);
}

#[test]
fn w6_one_edge_into_a_voted_sink_recounts_and_reorders() {
    assert_eq!(
        static_eat_and_west_plus_food_edge(W),
        vec![mv(W), eat(), mv(W)]
    );
}

#[test]
fn w7_a_burst_commits_the_ceiling_of_the_vote() {
    let mut graph = GraphBuilder::new();
    let one = graph.constant(1.0);
    graph.vote(
        VoteSink::Move(W),
        &[(one, 0.9), (leaf(0, u16::from(E)), 3.0)],
    );
    let g = genome(vec![graph.build(0, vec![occupied_ring()], &[])]);
    let mut senses = Senses::default();
    senses.occupied[usize::from(E)] = 1.0;
    let tick = run_tick(&g, senses, &config(), 20.0, [0.0; 16]);
    assert_eq!(tick.actions(), &[mv(W); 4]);
    assert_eq!(tick.output.work_counters.passes, 5);
}

#[test]
fn w8_a_plan_reads_its_own_queue_move_then_eat() {
    let mut graph = GraphBuilder::new();
    // [queue slot 0 is a movement], [its direction index above 5.5], ring W.
    let moved = graph.node(ComputeNodeKind::Threshold(1.5), &[(leaf(1, 0), 1.0)]);
    let west = graph.node(ComputeNodeKind::Threshold(5.5), &[(leaf(1, 1), 1.0)]);
    let after = graph.node(
        ComputeNodeKind::Multiply,
        &[(moved, 1.0), (west, 1.0), (leaf(0, u16::from(W)), 1.0)],
    );
    graph.vote(VoteSink::Move(W), &[(leaf(0, u16::from(W)), 0.9)]);
    graph.vote(VoteSink::Eat, &[(after, 1.0)]);
    let g = genome(vec![graph.build(
        0,
        vec![food_ring(), InputReference::ActionQueue],
        &[],
    )]);
    let mut senses = Senses::default();
    senses.food[usize::from(W)] = 1.0;
    let tick = run_tick(&g, senses, &config(), 20.0, [0.0; 16]);
    assert_eq!(tick.actions(), &[mv(W), eat()]);
}

#[test]
fn w9_a_route_with_a_turn_west_west_north() {
    // r0 = queue length, r1 = 2, r2 = [len < 2], r3 = 1 - r2.
    let program = vec![
        VmInstruction::ReadActionQueueLength { dst: 0 },
        VmInstruction::LoadConst {
            dst: 1,
            const_idx: 0,
        },
        VmInstruction::CmpLt { dst: 2, a: 0, b: 1 },
        VmInstruction::LoadConst {
            dst: 3,
            const_idx: 1,
        },
        VmInstruction::Sub { dst: 3, a: 3, b: 2 },
        VmInstruction::LoadConst {
            dst: 4,
            const_idx: 2,
        },
        VmInstruction::Mul { dst: 4, a: 4, b: 2 },
        VmInstruction::AddVote {
            sink: VoteSink::Move(W).index() as u8,
            src: 4,
        },
        VmInstruction::LoadConst {
            dst: 4,
            const_idx: 3,
        },
        VmInstruction::Mul { dst: 4, a: 4, b: 3 },
        VmInstruction::AddVote {
            sink: VoteSink::Move(N).index() as u8,
            src: 4,
        },
        VmInstruction::Halt,
    ];
    let g = genome(vec![vm_node(
        0,
        5,
        vec![2.0, 1.0, 3.5, 2.5],
        program,
        vec![],
        &[],
    )]);
    let tick = run_tick(&g, Senses::default(), &config(), 20.0, [0.0; 16]);
    assert_eq!(tick.actions(), &[mv(W), mv(W), mv(N)]);
    assert_eq!(tick.output.work_counters.passes, 4);
}

#[test]
fn w10_a_clock_ends_a_fixed_length_plan_with_terminate() {
    let mut graph = GraphBuilder::new();
    let one = graph.constant(1.0);
    let clock = graph.node(ComputeNodeKind::DecayIntegrator(0.5), &[(one, 1.0)]);
    let done = graph.node(ComputeNodeKind::Threshold(0.8), &[(clock, 1.0)]);
    graph.vote(VoteSink::Move(W), &[(one, 3.0)]);
    graph.vote(VoteSink::Terminate, &[(done, 1.0)]);
    let g = genome(vec![graph.build(0, vec![], &[])]);
    let tick = run_tick(&g, Senses::default(), &config(), 20.0, [0.0; 16]);
    assert_eq!(tick.actions(), &[mv(W), mv(W)]);
    assert_eq!(
        tick.output.termination_reason,
        TerminationReason::TerminateVoted
    );
}

#[test]
fn w11_a_voteless_self_loop_runs_one_capped_pass_to_noop() {
    let g = genome(vec![vm_voter(0, &[], &[0])]);
    let tick = run_tick(&g, Senses::default(), &config(), 10.0, [0.0; 16]);
    assert_eq!(tick.actions(), &[WorldAction::NoOp]);
    assert_eq!(
        tick.output.termination_reason,
        TerminationReason::NoDecision
    );
    assert_eq!(tick.passes[0].end_reason, PassEndReason::PassCapReached);
    assert_eq!(tick.output.work_counters.mesh_hops, 64);
    assert_eq!(tick.output.work_counters.pass_cap_hits, 1);
    // The ramp: 1e-4 · 32 · 33 / 2 over the 32 hops past the allowance.
    assert!((tick.output.cost_report.mesh_ramp_cost - 0.0528).abs() < 1e-5);
}

#[test]
fn w12_a_static_self_loop_commits_once_over_two_capped_passes() {
    let g = genome(vec![vm_voter(0, &[(VoteSink::Move(W), 1.0)], &[0])]);
    let tick = run_tick(&g, Senses::default(), &config(), 10.0, [0.0; 16]);
    assert_eq!(tick.actions(), &[mv(W)]);
    assert_eq!(tick.output.work_counters.pass_cap_hits, 2);
    assert_eq!(tick.output.work_counters.passes, 2);
    // A revisited node replaces its contribution: the vote stays 1.0.
    assert_eq!(tick.passes[0].votes[VoteSink::Move(W).index()], 1.0);
}

/// A self-loop that counts visits on bus slot 0 (carried across passes).
fn counting_program(extra: Vec<VmInstruction>) -> Vec<VmInstruction> {
    let mut program = vec![
        VmInstruction::ReadInput {
            dst: 0,
            ref_idx: 0,
            sub_idx: 0,
        },
        VmInstruction::LoadConst {
            dst: 1,
            const_idx: 0,
        },
        VmInstruction::Add { dst: 0, a: 0, b: 1 },
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 0,
        },
    ];
    program.extend(extra);
    program.push(VmInstruction::Halt);
    program
}

#[test]
fn w13_a_bus_counter_voted_into_move_fills_the_action_cap_and_pays_the_ramp() {
    let program = counting_program(vec![VmInstruction::AddVote {
        sink: VoteSink::Move(W).index() as u8,
        src: 0,
    }]);
    let g = genome(vec![vm_node(
        0,
        2,
        vec![1.0],
        program,
        vec![InputReference::UpstreamSlot(0)],
        &[0],
    )]);
    let tick = run_tick(&g, Senses::default(), &config(), 100.0, [0.0; 16]);
    assert_eq!(tick.actions(), &[mv(W); 10]);
    assert_eq!(
        tick.output.termination_reason,
        TerminationReason::ActionCapReached
    );
    assert_eq!(tick.output.work_counters.passes, 10);
    assert_eq!(tick.output.work_counters.pass_cap_hits, 10);
    assert_eq!(tick.output.work_counters.mesh_hops, 640);
    // 640 hops at allowance 32 and cost 1e-4: 1e-4 · 608 · 609 / 2.
    assert!((tick.output.cost_report.mesh_ramp_cost - 18.5136).abs() < 1e-2);
    // The count is visits, not passes: the last pass voted 640.
    assert_eq!(tick.passes[9].votes[VoteSink::Move(W).index()], 640.0);
}

#[test]
fn w14_a_cycle_with_a_route_exit_leaves_after_its_counted_visits() {
    // Entry: count on the bus; gate slot 0 (self) = [count < 3], slot 1
    // (exit) = 0.5. Exit: vote Move W.
    let entry = vm_node(
        0,
        3,
        vec![1.0, 3.0, 0.5],
        counting_program(vec![
            VmInstruction::LoadConst {
                dst: 1,
                const_idx: 1,
            },
            VmInstruction::CmpLt { dst: 2, a: 0, b: 1 },
            VmInstruction::WriteRouteGate { slot: 0, src: 2 },
            VmInstruction::LoadConst {
                dst: 2,
                const_idx: 2,
            },
            VmInstruction::WriteRouteGate { slot: 1, src: 2 },
        ]),
        vec![InputReference::UpstreamSlot(0)],
        &[0, 1],
    );
    let exit = vm_voter(1, &[(VoteSink::Move(W), 1.0)], &[]);
    let g = genome(vec![entry, exit]);
    let tick = run_tick(&g, Senses::default(), &config(), 20.0, [0.0; 16]);
    assert_eq!(tick.actions(), &[mv(W)]);
    // Pass 1: three entry visits, then the exit.
    assert_eq!(tick.passes[0].hops, 4);
    assert_eq!(tick.passes[0].end_reason, PassEndReason::NoTargets);
    assert_eq!(tick.output.work_counters.pass_cap_hits, 0);
}

#[test]
fn w15_terminate_cannot_end_an_empty_tick() {
    let g = genome(vec![vm_voter(
        0,
        &[(VoteSink::Eat, 0.5), (VoteSink::Terminate, 1.0)],
        &[],
    )]);
    let tick = run_tick(&g, Senses::default(), &config(), 20.0, [0.0; 16]);
    assert_eq!(tick.actions(), &[eat()]);
    // Pass 2 has no positive effective vote, so it ends `NoDecision`
    // whatever `Terminate` holds.
    assert_eq!(
        tick.output.termination_reason,
        TerminationReason::NoDecision
    );
}

#[test]
fn w16_every_kind_inhibited_is_noop_without_a_stall() {
    let g = genome(vec![vm_voter(
        0,
        &[
            (VoteSink::Eat, -1.0),
            (VoteSink::Move(W), -0.5),
            (VoteSink::Reproduce(E), 0.0),
            (VoteSink::StealEnergy(N), -2.0),
            (VoteSink::Terminate, 3.0),
        ],
        &[],
    )]);
    let tick = run_tick(&g, Senses::default(), &config(), 20.0, [0.0; 16]);
    assert_eq!(tick.actions(), &[WorldAction::NoOp]);
    assert_eq!(tick.output.work_counters.passes, 1);
    assert_eq!(
        tick.output.termination_reason,
        TerminationReason::NoDecision
    );
}

/// W17's self-loop after `prefix`: counting on the bus, `Move W 0.9`, and
/// `[count >= 3] → Decide` (count > 2.5).
fn w17_genome(prefix: Vec<VmInstruction>) -> CreatureGenome {
    let mut extra = prefix;
    extra.extend([
        VmInstruction::LoadConst {
            dst: 1,
            const_idx: 1,
        },
        VmInstruction::AddVote {
            sink: VoteSink::Move(W).index() as u8,
            src: 1,
        },
        VmInstruction::LoadConst {
            dst: 1,
            const_idx: 2,
        },
        VmInstruction::CmpGt { dst: 1, a: 0, b: 1 },
        VmInstruction::AddVote {
            sink: VoteSink::Decide.index() as u8,
            src: 1,
        },
    ]);
    genome(vec![vm_node(
        0,
        2,
        vec![1.0, 0.9, 2.5],
        counting_program(extra),
        vec![InputReference::UpstreamSlot(0)],
        &[0],
    )])
}

#[test]
fn w17_a_decide_vote_exits_a_cycle_that_changes_state() {
    let g = w17_genome(vec![]);
    let tick = run_tick(&g, Senses::default(), &config(), 20.0, [0.0; 16]);
    assert_eq!(tick.actions(), &[mv(W)]);
    assert_eq!(tick.passes[0].end_reason, PassEndReason::Decided);
    assert_eq!(tick.passes[0].hops, 3);
    // Pass 2: the bus carries the count, so `Decide` holds from the first
    // visit, but `Move W` is 0.9 − 1 and no `Terminate` is voted, so the guard
    // keeps the pass running to its cap; the pass then ends the tick
    // `NoDecision`.
    assert_eq!(tick.passes[1].end_reason, PassEndReason::PassCapReached);
    assert_eq!(tick.output.work_counters.decided_passes, 1);
    assert_eq!(tick.output.work_counters.pass_cap_hits, 1);
    assert_eq!(
        tick.output.termination_reason,
        TerminationReason::NoDecision
    );
}

#[test]
fn w17b_a_terminate_vote_lets_decide_end_a_pass_with_nothing_to_commit() {
    // W17 plus a static `Terminate 1.0`. Pass 1 is W17's: the queue is empty,
    // so `Terminate` does not satisfy the guard. Pass 2: `Move W` is 0.9 − 1,
    // but the queue holds `Move W` and `Terminate` is positive, so `Decide`
    // ends the pass at its first boundary; no kind's effective vote is
    // positive, so the pass end ends the tick `NoDecision`.
    let g = w17_genome(vec![
        VmInstruction::LoadConst {
            dst: 1,
            const_idx: 0,
        },
        VmInstruction::AddVote {
            sink: VoteSink::Terminate.index() as u8,
            src: 1,
        },
    ]);
    let tick = run_tick(&g, Senses::default(), &config(), 20.0, [0.0; 16]);
    assert_eq!(tick.actions(), &[mv(W)]);
    assert_eq!(tick.passes.len(), 2);
    assert_eq!(tick.passes[0].end_reason, PassEndReason::Decided);
    assert_eq!(tick.passes[0].hops, 3);
    assert_eq!(tick.passes[1].end_reason, PassEndReason::Decided);
    assert_eq!(tick.passes[1].hops, 1);
    assert_eq!(tick.output.work_counters.decided_passes, 2);
    assert_eq!(tick.output.work_counters.pass_cap_hits, 0);
    assert_eq!(
        tick.output.termination_reason,
        TerminationReason::NoDecision
    );
}

#[test]
fn w18_decide_before_any_vote_is_ignored_by_the_guard() {
    let entry = vm_voter(0, &[(VoteSink::Decide, 1.0)], &[1]);
    let (graph, refs) = forage_graph(vec![]);
    let g = genome(vec![entry, graph.build(1, refs, &[])]);
    let tick = run_tick(&g, w1_senses(1.0), &config(), 20.0, [0.0; 16]);
    assert_eq!(tick.actions(), &[eat(), mv(E)]);
    // Both committing passes run the whole chain and end `Decided` at node 1.
    assert_eq!(tick.passes[0].hops, 2);
    assert_eq!(tick.passes[0].end_reason, PassEndReason::Decided);
    assert_eq!(tick.output.work_counters.decided_passes, 2);
}

/// W19 chain: n0 votes `Decide` from `Constant(-0.5)` and memory slot 0 at
/// weight −1; n1 votes `Move W 1.0` and readiness `Decide 0.7`; n2 votes
/// `Move E 2.0`.
fn w19_queue(slot: f32) -> Vec<WorldAction> {
    let mut threshold = GraphBuilder::new();
    let offset = threshold.constant(-0.5);
    threshold.vote(
        VoteSink::Decide,
        &[
            (offset, 1.0),
            (
                GraphSource::SharedMemory {
                    slot: 0,
                    previous: false,
                },
                -1.0,
            ),
        ],
    );
    let mut sure = GraphBuilder::new();
    let one = sure.constant(1.0);
    sure.vote(VoteSink::Move(W), &[(one, 1.0)]);
    sure.vote(VoteSink::Decide, &[(one, 0.7)]);
    let mut later = GraphBuilder::new();
    let two = later.constant(2.0);
    later.vote(VoteSink::Move(E), &[(two, 1.0)]);
    let g = genome(vec![
        threshold.build(0, vec![], &[1]),
        sure.build(1, vec![], &[2]),
        later.build(2, vec![], &[]),
    ]);
    let mut memory = [0.0; 16];
    memory[0] = slot;
    run_tick(&g, Senses::default(), &config(), 20.0, memory)
        .actions()
        .to_vec()
}

#[test]
fn w19_a_persistent_slot_tunes_the_deliberation_threshold() {
    // Slot 0.1: one confident node ends the first pass before `Move E` is
    // voted; slot 0.9: the chain runs to its end every pass.
    assert_eq!(w19_queue(0.1), vec![mv(W), mv(E)]);
    assert_eq!(w19_queue(0.9), vec![mv(E), mv(E)]);
}

// ── Commit decode, exhaustion, and the bid (invariants 3 and 6) ────────────

#[test]
fn a_commit_decodes_the_parameter_surface_of_its_kind() {
    let mut graph = GraphBuilder::new();
    let one = graph.constant(1.0);
    let fraction = graph.constant(0.25);
    graph.vote(VoteSink::Reproduce(W), &[(one, 1.0)]);
    graph.param(VoteKind::Reproduce, 1, &[(fraction, 1.0)]);
    graph.param(VoteKind::StealEnergy, 1, &[(fraction, 8.0)]);
    let steal = vm_node(
        1,
        1,
        vec![3.0, 1.0],
        vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            // params[Eat][0] (slot 0) = 3: food type 3.
            VmInstruction::WriteActionParam {
                slot_idx: 0,
                src: 0,
            },
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 1,
            },
            VmInstruction::AddVote {
                sink: VoteSink::StealEnergy(N).index() as u8,
                src: 0,
            },
            VmInstruction::AddVote {
                sink: VoteSink::Eat.index() as u8,
                src: 0,
            },
            // An invalid slot is ignored.
            VmInstruction::WriteActionParam {
                slot_idx: 8,
                src: 0,
            },
            VmInstruction::Halt,
        ],
        vec![],
        &[],
    );
    let g = genome(vec![graph.build(0, vec![], &[1]), steal]);
    let tick = run_tick(&g, Senses::default(), &config(), 20.0, [0.0; 16]);
    assert_eq!(
        tick.actions(),
        &[
            WorldAction::eat(OrdinaryFoodTypeId::new(3)),
            WorldAction::Reproduce {
                direction: Direction::W,
                energy_transfer_fraction: 0.25,
            },
            WorldAction::StealEnergy {
                direction: Direction::N,
                amount: 2.0,
            },
        ]
    );
}

#[test]
fn a_missing_entry_node_is_one_missing_node_pass_and_noop() {
    let mut g = genome(vec![vm_voter(3, &[(VoteSink::Eat, 1.0)], &[])]);
    g.entry_node_id = crate::contracts::NodeId::new(9);
    let tick = run_tick(&g, Senses::default(), &config(), 20.0, [0.0; 16]);
    assert_eq!(tick.actions(), &[WorldAction::NoOp]);
    assert_eq!(tick.passes[0].end_reason, PassEndReason::MissingNode);
    assert_eq!(
        tick.output.termination_reason,
        TerminationReason::NoDecision
    );
    assert_eq!(tick.output.work_counters.mesh_hops, 0);
}

#[test]
fn a_route_to_a_missing_node_ends_the_pass_with_its_votes() {
    let g = genome(vec![vm_voter(0, &[(VoteSink::Eat, 1.0)], &[7])]);
    let tick = run_tick(&g, Senses::default(), &config(), 20.0, [0.0; 16]);
    assert_eq!(tick.actions(), &[eat()]);
    assert_eq!(tick.passes[0].end_reason, PassEndReason::MissingNode);
}

#[test]
fn the_action_cap_ends_the_tick_at_one() {
    let (graph, refs) = forage_graph(vec![]);
    let g = genome(vec![graph.build(0, refs, &[])]);
    let config = RuntimeConfig {
        max_actions_per_turn: 1,
        ..config()
    };
    let tick = run_tick(&g, w1_senses(1.0), &config, 20.0, [0.0; 16]);
    assert_eq!(tick.actions(), &[eat()]);
    assert_eq!(
        tick.output.termination_reason,
        TerminationReason::ActionCapReached
    );
}

/// Production opcode costs are tiny, so exhaustion cases price compute at
/// its nominal base cost with both ramps off.
fn costly() -> RuntimeConfig {
    RuntimeConfig {
        hop_ramp_cost: 0.0,
        vm: crate::config::VmRuntimeConfig {
            opcode_cost_multiplier: 1.0,
            step_ramp_cost: 0.0,
            ..crate::config::VmRuntimeConfig::default()
        },
        ..config()
    }
}

#[test]
fn exhaustion_keeps_the_actions_committed_before_it() {
    // `Eat 1.0` commits in pass 1 (LoadConst 0.08 + AddVote 0.14 + Halt
    // 0.05 = 0.27); pass 2 cannot afford its second instruction.
    let g = genome(vec![vm_voter(0, &[(VoteSink::Eat, 1.0)], &[])]);
    let tick = run_tick(&g, Senses::default(), &costly(), 0.4, [0.0; 16]);
    assert_eq!(tick.actions(), &[eat()]);
    assert_eq!(
        tick.output.termination_reason,
        TerminationReason::EnergyExhausted
    );
    assert_eq!(tick.passes[1].end_reason, PassEndReason::EnergyExhausted);
    assert_eq!(tick.passes[1].committed, None);
    assert_eq!(
        tick.output.energy_observation.pending_cause,
        Some(DeathCause::VmCompute)
    );
}

/// A VM voter that also bids `bid`.
fn bidding_voter(bid: f32) -> crate::creature::genome::NodeGenome {
    vm_node(
        0,
        1,
        vec![bid, 1.0],
        vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            VmInstruction::SetPriorityBid { src: 0 },
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 1,
            },
            VmInstruction::AddVote {
                sink: VoteSink::Eat.index() as u8,
                src: 0,
            },
            VmInstruction::Halt,
        ],
        vec![],
        &[],
    )
}

#[test]
fn the_bid_settles_once_after_every_pass() {
    let g = genome(vec![bidding_voter(3.0)]);
    let tick = run_tick(&g, Senses::default(), &config(), 100.0, [0.0; 16]);
    assert_eq!(tick.actions(), &[eat()]);
    assert_eq!(tick.output.priority_bid, 3.0);
    assert_eq!(tick.output.work_counters.passes, 2);
    assert!(
        (tick.energy - 97.0).abs() < 1e-3,
        "paid once: {}",
        tick.energy
    );
}

#[test]
fn an_all_in_bid_keeps_the_queue_and_reports_the_energy_paid() {
    let g = genome(vec![bidding_voter(5.0)]);
    let tick = run_tick(&g, Senses::default(), &config(), 2.0, [0.0; 16]);
    assert_eq!(tick.energy, 0.0);
    assert_eq!(tick.actions(), &[eat()]);
    assert_eq!(
        tick.output.termination_reason,
        TerminationReason::EnergyExhausted
    );
    let paid = tick.output.priority_bid;
    assert!(paid > 1.99 && paid <= 2.0, "paid {paid}");
    assert_eq!(
        tick.output.energy_observation.pending_cause,
        Some(DeathCause::PriorityBid)
    );
}

#[test]
fn a_creature_exhausted_on_compute_pays_no_bid() {
    let g = genome(vec![bidding_voter(0.5)]);
    // LoadConst and SetPriorityBid record the bid (0.28); the next
    // LoadConst exhausts the creature, which then pays nothing.
    let tick = run_tick(&g, Senses::default(), &costly(), 0.3, [0.0; 16]);
    assert_eq!(
        tick.output.termination_reason,
        TerminationReason::EnergyExhausted
    );
    assert!(tick.energy <= 0.0);
    assert_eq!(tick.output.priority_bid, 0.0);
    assert_eq!(tick.output.energy_observation.priority_bid, 0.0);
}

#[test]
fn the_bus_is_zero_at_tick_start_and_carries_between_passes() {
    // Vote `Move W` by the bus slot 0 value plus 1, then write 5 to it.
    let program = vec![
        VmInstruction::ReadInput {
            dst: 0,
            ref_idx: 0,
            sub_idx: 0,
        },
        VmInstruction::LoadConst {
            dst: 1,
            const_idx: 0,
        },
        VmInstruction::Add { dst: 1, a: 0, b: 1 },
        VmInstruction::AddVote {
            sink: VoteSink::Move(W).index() as u8,
            src: 1,
        },
        VmInstruction::LoadConst {
            dst: 1,
            const_idx: 1,
        },
        VmInstruction::WriteInternalPayload {
            slot_idx: 0,
            src: 1,
        },
        VmInstruction::Halt,
    ];
    let g = genome(vec![vm_node(
        0,
        2,
        vec![1.0, 5.0],
        program,
        vec![InputReference::UpstreamSlot(0)],
        &[],
    )]);
    let tick = run_tick(&g, Senses::default(), &config(), 20.0, [0.0; 16]);
    assert_eq!(tick.passes[0].votes[VoteSink::Move(W).index()], 1.0);
    assert_eq!(tick.passes[1].votes[VoteSink::Move(W).index()], 6.0);
    assert_eq!(tick.actions(), &[mv(W); 6]);
}

#[test]
fn hop_indices_run_across_passes_and_carry_their_pass() {
    let (graph, refs) = forage_graph(vec![]);
    let g = genome(vec![graph.build(0, refs, &[])]);
    let tick = run_tick(&g, w1_senses(1.0), &config(), 20.0, [0.0; 16]);
    let indices: Vec<(usize, u32)> = tick
        .hops
        .iter()
        .map(|hop| (hop.hop_index, hop.pass_index))
        .collect();
    assert_eq!(indices, vec![(0, 0), (1, 1), (2, 2)]);
}

#[test]
fn dynamic_energy_reads_are_live_across_passes() {
    // Vote `Eat` by 10 · EnergyCurrent: every pass pays for its visit, so
    // the second pass reads less energy than the first.
    let mut graph = GraphBuilder::new();
    graph.vote(VoteSink::Eat, &[(leaf(0, 0), 10.0)]);
    let g = genome(vec![graph.build(
        0,
        vec![InputReference::DynamicIntrospection(
            DynamicIntrospectionKey::EnergyCurrent,
        )],
        &[],
    )]);
    let tick = run_tick(&g, Senses::default(), &config(), 50.0, [0.0; 16]);
    let first = tick.passes[0].votes[VoteSink::Eat.index()];
    let second = tick.passes[1].votes[VoteSink::Eat.index()];
    // The visit's own charge lands before its read.
    assert!(first < 2.5 && first > 2.49, "{first}");
    assert!(second < first, "{second} < {first}");
}
