use super::super::run_tick;
use super::support::*;
use crate::config::SimulationConfig;
use crate::creature::genome::vote::DirectedVoteKind;
use crate::runtime::trace::recording::ActiveTrace;
use crate::simulation::seeding::seed_simulation;

#[test]
fn trace_populated_after_tick() {
    let (mut sim, id) = make_sim_with_one_creature(50.0);
    let mut trace = Some(ActiveTrace::new(id, 1));

    run_tick(&mut sim, &mut trace);

    let active = trace.as_ref().expect("trace should still be Some");
    assert_eq!(active.ticks.len(), 1, "one tick should be recorded");
    assert!(
        active.is_complete(),
        "trace should be complete after 1 tick"
    );

    let tick_trace = &active.ticks[0];
    assert_eq!(
        tick_trace.tick_number, 0,
        "tick_number should be 0 (pre-increment)"
    );
    assert!(
        tick_trace.energy_before > 0.0,
        "energy_before should be positive"
    );
    assert!(!tick_trace.hops.is_empty(), "hops should not be empty");
}

#[test]
fn trace_completes_after_n_ticks() {
    let (mut sim, id) = make_sim_with_one_creature(50.0);
    let mut trace = Some(ActiveTrace::new(id, 3));

    for i in 0..3 {
        assert!(!trace.as_ref().unwrap().is_complete());
        run_tick(&mut sim, &mut trace);
        assert_eq!(trace.as_ref().unwrap().ticks.len(), i + 1);
    }
    assert!(trace.as_ref().unwrap().is_complete());
    assert_eq!(trace.as_ref().unwrap().ticks_remaining, 0);
}

#[test]
fn trace_creature_death_finalizes_partial() {
    let decay = SimulationConfig::default()
        .energy
        .lifecycle
        .energy_decay_per_tick;
    let (mut sim, id) = make_sim_with_one_creature(decay * 0.5);
    let mut trace = Some(ActiveTrace::new(id, 5));

    run_tick(&mut sim, &mut trace);

    let active = trace.as_ref().expect("trace should still be Some");
    assert!(
        active.is_complete(),
        "trace should be complete after creature death"
    );
    assert_eq!(
        active.ticks.len(),
        0,
        "no tick traces since creature died before cognition"
    );
}

#[test]
fn non_traced_creatures_unaffected() {
    let cfg = small_config();
    let mut sim_a = seed_simulation(cfg.clone(), 42);
    let mut sim_b = seed_simulation(cfg, 42);

    run_tick(&mut sim_a, &mut None);

    let first_id = {
        let mut keys: Vec<_> = sim_b.creatures.keys().collect();
        keys.sort();
        keys[0]
    };

    let mut trace = Some(ActiveTrace::new(first_id, 1));
    run_tick(&mut sim_b, &mut trace);

    assert_eq!(sim_a.tick, sim_b.tick);
    assert_eq!(sim_a.creatures.len(), sim_b.creatures.len());

    let ids_a: Vec<_> = {
        let mut k: Vec<_> = sim_a.creatures.keys().collect();
        k.sort();
        k
    };
    let ids_b: Vec<_> = {
        let mut k: Vec<_> = sim_b.creatures.keys().collect();
        k.sort();
        k
    };
    assert_eq!(ids_a, ids_b);
}

#[test]
fn run_tick_with_none_trace_identical_behavior() {
    let mut sim = seed_simulation(small_config(), 42);
    let pop_before = sim.creatures.len();
    run_tick(&mut sim, &mut None);
    assert_eq!(sim.tick, 1);
    assert!(!sim.creatures.is_empty() || pop_before == 0);
}

/// A committed `Move` sets the stored heading after the tick, the world's
/// block notwithstanding (T11.F30); the traced tick carries the heading it
/// started with, and the traced and untraced paths store the same heading.
#[test]
fn a_committed_move_sets_the_stored_heading_and_the_trace_keeps_the_start() {
    use crate::contracts::{Direction, Position};
    use crate::creature::genome::vote::VoteSink;
    use crate::creature::genome::VmInstruction;

    let genome = vm_program_genome(vec![VmInstruction::AddVote {
        sink: VoteSink::Directed(DirectedVoteKind::Move, 6).index() as u8,
        src: 0,
    }]);
    for traced in [false, true] {
        let (mut sim, id) = make_sim_with_custom_genome(100.0, genome.clone());
        sim.creatures[id].heading = Direction::SW;
        // The move west is blocked: the creature still turns.
        sim.world.set_barrier(Position::new(4, 5), true);
        let mut trace = traced.then(|| ActiveTrace::new(id, 1));
        run_tick(&mut sim, &mut trace);
        assert_eq!(sim.creatures[id].position, Position::new(5, 5));
        assert_eq!(sim.creatures[id].heading, Direction::W);
        if let Some(active) = trace {
            assert_eq!(active.ticks[0].heading, Direction::SW);
        }
    }
}
