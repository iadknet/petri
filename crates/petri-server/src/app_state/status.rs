use super::{SimulationState, SimulationStatus};

pub(super) fn simulation_status_from_locked(sim: &SimulationState) -> SimulationStatus {
    let (run_id, seed, tick, population, average_energy) = if let Some(run) = sim.run.as_ref() {
        (
            Some(run.run_id),
            Some(run.seed),
            run.world.tick_count(),
            run.world.creature_count(),
            run.world.average_energy(),
        )
    } else {
        (None, None, 0, 0, 0.0)
    };

    SimulationStatus {
        phase: sim.phase,
        initialization_stage: sim.initialization_stage,
        viability_probe_enabled: sim.viability_probe_enabled,
        run_id,
        seed,
        tick,
        population,
        average_energy,
        pending_restart: sim.pending_restart,
        startup_viable: sim.startup_viability.is_viable,
        startup_viability_code: sim.startup_viability.code,
        startup_viability_message: sim.startup_viability.message.clone(),
        startup_draft: sim.startup_draft.clone(),
    }
}
