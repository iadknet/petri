//! Profiling harness: runs N simulation ticks with a realistic config.
//!
//! Usage:
//!   # CPU profiling with samply:
//!   cargo build --profile profiling -p v3-core --bin profile_ticks
//!   samply record ./target/profiling/profile_ticks [--ticks 500] [--creatures 500] [--world-size 200]
//!
//!   # Heap allocation profiling with dhat:
//!   cargo run -p v3-core --bin profile_ticks --features dhat-heap -- [--ticks 500]

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use v3_core::config::SimulationConfig;
use v3_core::simulation::seeding::seed_simulation;
use v3_core::simulation::tick::run_tick;

fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    let args: Vec<String> = std::env::args().collect();

    let mut ticks: u64 = 200;
    let mut creatures: u32 = 200;
    let mut world_size: u16 = 200;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--ticks" => {
                i += 1;
                ticks = args[i].parse().expect("invalid --ticks");
            }
            "--creatures" => {
                i += 1;
                creatures = args[i].parse().expect("invalid --creatures");
            }
            "--world-size" => {
                i += 1;
                world_size = args[i].parse().expect("invalid --world-size");
            }
            other => {
                eprintln!("unknown arg: {other}");
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let mut cfg = SimulationConfig::default();
    cfg.world.width = world_size;
    cfg.world.height = world_size;
    cfg.population.initial_creatures = creatures;
    cfg.normalize();

    eprintln!(
        "Profiling: {} ticks, {} creatures, {}x{} world",
        ticks, creatures, world_size, world_size
    );

    let mut sim = seed_simulation(cfg, 42);

    let start = std::time::Instant::now();
    for _ in 0..ticks {
        run_tick(&mut sim, &mut None);
    }
    let elapsed = start.elapsed();

    let pop = sim.creatures.len();
    let tps = ticks as f64 / elapsed.as_secs_f64();
    eprintln!(
        "Done: {:.2?} elapsed, {:.1} ticks/sec, {} creatures remaining",
        elapsed, tps, pop
    );
}
