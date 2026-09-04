use super::super::run_tick;
use super::support::*;
use crate::creature::genome::VmInstruction;

/// A single creature running a known 3-opcode program (Noop, PushAction,
/// ExecuteActionQueue) for one tick exercises exactly one mesh hop, three VM
/// steps, no graph work, one creature-tick, and one applied action.
#[test]
fn run_tick_accumulates_known_vm_work_counters_for_one_creature() {
    let genome = vm_program_genome(vec![
        VmInstruction::Noop,
        VmInstruction::PushAction { action_type: 0 },
        VmInstruction::ExecuteActionQueue,
    ]);
    let (mut sim, _id) = make_sim_with_custom_genome(100.0, genome);

    run_tick(&mut sim, &mut None);

    assert_eq!(sim.stats.mesh_hops_total, 1);
    assert_eq!(sim.stats.vm_steps_total, 3);
    assert_eq!(sim.stats.graph_relax_iters_total, 0);
    assert_eq!(sim.stats.plasticity_updates_total, 0);
    assert_eq!(sim.stats.creature_ticks_total, 1);
    assert_eq!(sim.stats.actions_applied_total, 1);
    assert_eq!(sim.stats.last_tick_noop, 1);
}

/// Work counters accumulate cumulatively across ticks rather than resetting.
#[test]
fn run_tick_work_counters_accumulate_across_ticks() {
    let genome = vm_program_genome(vec![
        VmInstruction::Noop,
        VmInstruction::PushAction { action_type: 0 },
        VmInstruction::ExecuteActionQueue,
    ]);
    let (mut sim, _id) = make_sim_with_custom_genome(1000.0, genome);

    run_tick(&mut sim, &mut None);
    run_tick(&mut sim, &mut None);

    assert_eq!(sim.stats.mesh_hops_total, 2);
    assert_eq!(sim.stats.vm_steps_total, 6);
    assert_eq!(sim.stats.creature_ticks_total, 2);
    assert_eq!(sim.stats.actions_applied_total, 2);
}

/// `creature_ticks_total` and `actions_applied_total` sum across every
/// founder in a multi-creature population, not just the first.
#[test]
fn run_tick_sums_creature_ticks_and_actions_across_population() {
    let mut sim = seed_simulation_default();
    let population = sim.creatures.len() as u64;

    run_tick(&mut sim, &mut None);

    assert_eq!(sim.stats.creature_ticks_total, population);
    assert_eq!(
        sim.stats.actions_applied_total,
        u64::from(
            sim.stats.last_tick_move
                + sim.stats.last_tick_eat
                + sim.stats.last_tick_noop
                + sim.stats.last_tick_reproduce
                + sim.stats.last_tick_steal
        )
    );
}

fn seed_simulation_default() -> crate::simulation::Simulation {
    crate::simulation::seeding::seed_simulation(small_config(), 7)
}
