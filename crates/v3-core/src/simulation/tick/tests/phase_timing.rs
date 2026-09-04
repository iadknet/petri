//! Phase wall-clock accumulation (T10.F09).
//!
//! Wall-clock is host-dependent, so these tests assert only the structural
//! properties the throughput baseline depends on — every timed phase records
//! time, the totals never shrink, and they survive the per-tick counter reset
//! — never a magnitude.

use std::time::Duration;

use super::super::run_tick;
use super::support::*;
use crate::simulation::stats::PhaseWallClock;

/// The five timed phases, so each assertion below names the field it failed on.
fn phases(clock: &PhaseWallClock) -> [(&'static str, Duration); 5] {
    [
        ("world_update", clock.world_update),
        ("sensor_assembly", clock.sensor_assembly),
        ("cognition", clock.cognition),
        ("actions", clock.actions),
        ("reward_learning", clock.reward_learning),
    ]
}

/// Running ticks accumulates a nonzero, non-decreasing duration in every one
/// of the five timed phases.
#[test]
fn run_tick_accumulates_every_phase_wall_clock_field() {
    let (mut sim, _id) = make_sim_with_one_creature(10_000.0);

    for _ in 0..32 {
        run_tick(&mut sim, &mut None);
    }
    let after_first = sim.stats.phase_wall_clock;

    for (name, elapsed) in phases(&after_first) {
        assert!(
            elapsed > Duration::ZERO,
            "phase {name} recorded no wall-clock across 32 ticks"
        );
    }

    for _ in 0..32 {
        run_tick(&mut sim, &mut None);
    }
    let after_second = sim.stats.phase_wall_clock;

    for ((name, later), (_, earlier)) in phases(&after_second).into_iter().zip(phases(&after_first))
    {
        assert!(
            later >= earlier,
            "phase {name} wall-clock decreased: {earlier:?} then {later:?}"
        );
    }
}

/// `reset_tick_counters` clears the per-tick counters at the start of every
/// tick; the cumulative phase wall-clock must not be cleared with them.
#[test]
fn reset_tick_counters_clears_per_tick_counters_but_not_phase_wall_clock() {
    let (mut sim, _id) = make_sim_with_one_creature(10_000.0);

    run_tick(&mut sim, &mut None);
    let before = sim.stats.phase_wall_clock;
    sim.stats.last_tick_noop = 7;

    sim.stats.reset_tick_counters();

    assert_eq!(sim.stats.last_tick_noop, 0, "per-tick counters must reset");
    for ((name, after), (_, before)) in phases(&sim.stats.phase_wall_clock)
        .into_iter()
        .zip(phases(&before))
    {
        assert_eq!(
            after, before,
            "phase {name} wall-clock must survive reset_tick_counters"
        );
        assert!(
            after > Duration::ZERO,
            "phase {name} must have recorded time before the reset for this test to mean anything"
        );
    }
}
