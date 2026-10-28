//! Census pins (T30.F13): the population values the status and health
//! payloads carry, pinned on a living and an empty population against the
//! code before the censuses moved into core's readers. Only the call
//! expressions may change, never the literals. T30.F15 re-pinned none: the
//! readers' mean energy and effective lineage count cast to `f32` render
//! these populations' bits unchanged.

use super::*;
use crate::query::cache::build_food_fertility_u8;
use v3_core::simulation::run_tick;

fn frame_of(sim: Simulation) -> WsFrame {
    let cached_fertility_u8 = build_food_fertility_u8(sim.world.food());
    let handle = SimHandle {
        sim,
        status: SimulationStatus::Paused,
        active_trace: None,
        #[cfg(feature = "telemetry")]
        window_trace: None,
        cached_fertility_u8,
    };
    build_ws_frame(&handle)
}

/// `(status mean energy, health mean energy, complexity mean)` as `f32`
/// bits, the integer census fields, the `f32` means as bits, and the live VM
/// world-input reads in key order.
fn census_fields(frame: &WsFrame) -> String {
    let health = &frame.health;
    let reads: std::collections::BTreeMap<_, _> =
        health.vm_live_read_world_inputs_current.iter().collect();
    format!(
        "status.mean_energy={:#x} health.mean_energy={:#x} complexity={:#x}/{}/{} \
         generation={:#x}/{} genome_size={:#x} lineages={}/{:#x} world_inputs_read={:#x} \
         population={}/{} vm_reads={reads:?}",
        frame.status.mean_energy.to_bits(),
        health.mean_energy.to_bits(),
        health.genome_complexity_mean.to_bits(),
        health.genome_complexity_min,
        health.genome_complexity_max,
        health.generation_mean.to_bits(),
        health.generation_max,
        health.genome_size_mean.to_bits(),
        health.lineages_alive,
        health.lineages_effective.to_bits(),
        health.world_inputs_read_mean.to_bits(),
        frame.status.population,
        health.population,
    )
}

fn living() -> Simulation {
    let mut config = SimulationConfig::default();
    config.world.width = 40;
    config.world.height = 40;
    config.population.initial_creatures = 48;
    let mut sim = seed_simulation(config, 29);
    for _ in 0..40 {
        run_tick(&mut sim, &mut None);
    }
    sim
}

#[test]
fn the_status_and_health_census_keeps_its_pre_move_values() {
    let measured = census_fields(&frame_of(living()));
    println!("PIN {measured}");
    assert_eq!(measured, LIVING_PIN);
}

#[test]
fn an_empty_population_keeps_its_pre_move_census_values() {
    let mut config = SimulationConfig::default();
    config.world.width = 16;
    config.world.height = 16;
    config.population.initial_creatures = 0;
    let measured = census_fields(&frame_of(seed_simulation(config, 29)));
    println!("PIN {measured}");
    assert_eq!(measured, EMPTY_PIN);
}

/// Each payload's `f32` mean energy and effective lineage count is the
/// reader's value cast, on energies whose `f32` sum would round them away,
/// and `0` on an empty population. Exact equality: the payload only casts.
#[test]
fn the_payloads_cast_the_readers_mean_energy_and_effective_lineages() {
    let mut sim = living();
    for (creature, energy) in sim
        .creatures
        .values_mut()
        .zip([16_777_216.0_f32].into_iter().chain(std::iter::repeat(1.0)))
    {
        creature.energy = energy;
    }
    let census = PopulationCensus::of(&sim);
    let mean_energy = census.mean_energy().expect("a living population") as f32;
    let effective = CladeCensus::of(census.lineage_ids())
        .effective_lineages()
        .expect("a living population") as f32;
    let frame = frame_of(sim);
    assert_eq!(frame.status.mean_energy.to_bits(), mean_energy.to_bits());
    assert_eq!(frame.health.mean_energy.to_bits(), mean_energy.to_bits());
    assert_eq!(
        frame.health.lineages_effective.to_bits(),
        effective.to_bits()
    );

    let mut config = SimulationConfig::default();
    config.population.initial_creatures = 0;
    let empty = frame_of(seed_simulation(config, 29));
    assert_eq!(empty.status.mean_energy.to_bits(), 0.0_f32.to_bits());
    assert_eq!(empty.health.mean_energy.to_bits(), 0.0_f32.to_bits());
    assert_eq!(empty.health.lineages_effective.to_bits(), 0.0_f32.to_bits());
}

const LIVING_PIN: &str = "status.mean_energy=0x41f60aa1 health.mean_energy=0x41f60aa1 complexity=0x42745816/61/63 generation=0x3f22689a/1 genome_size=0x42a0268a lineages=45/0x42272ab5 world_inputs_read=0x40000000 population=93/93 vm_reads={}";
const EMPTY_PIN: &str = "status.mean_energy=0x0 health.mean_energy=0x0 complexity=0x0/0/0 generation=0x0/0 genome_size=0x0 lineages=0/0x0 world_inputs_read=0x0 population=0/0 vm_reads={}";
