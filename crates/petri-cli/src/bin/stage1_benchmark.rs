use std::process::ExitCode;
use std::time::Instant;

use clap::Parser;
use petri_core::{World, WorldConfig};

#[derive(Debug, Parser)]
#[command(
    name = "stage1-benchmark",
    about = "Deterministic Stage 1 throughput benchmark"
)]
struct Args {
    #[arg(long, default_value_t = 200)]
    ticks: u64,
    #[arg(long, default_value_t = 200)]
    width: u32,
    #[arg(long, default_value_t = 200)]
    height: u32,
    #[arg(long, default_value_t = 5_000)]
    initial_creatures: usize,
    #[arg(long, default_value_t = 5_000)]
    max_creatures: usize,
    #[arg(long, default_value_t = 42)]
    seed: u64,
    #[arg(long, default_value_t = 30.0)]
    min_ticks_per_second: f64,
    #[arg(long, default_value_t = false)]
    assert_min: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();
    let config = WorldConfig {
        width: args.width,
        height: args.height,
        initial_creatures: args.initial_creatures,
        max_creatures: args.max_creatures,
        ..WorldConfig::default()
    };

    let mut world = World::new(config.clone(), args.seed);
    let start = Instant::now();
    for _ in 0..args.ticks {
        world.tick();
    }
    let elapsed = start.elapsed();

    let elapsed_seconds = elapsed.as_secs_f64().max(f64::EPSILON);
    let ticks_per_second = args.ticks as f64 / elapsed_seconds;
    println!("stage1_benchmark");
    println!("seed={}", args.seed);
    println!(
        "world={}x{} initial_creatures={} max_creatures={}",
        config.width, config.height, config.initial_creatures, config.max_creatures
    );
    println!("ticks={}", args.ticks);
    println!("elapsed_ms={:.2}", elapsed_seconds * 1000.0);
    println!("ticks_per_second={:.2}", ticks_per_second);
    println!("final_population={}", world.creature_count());
    println!("average_energy={:.4}", world.average_energy());

    if args.assert_min && ticks_per_second < args.min_ticks_per_second {
        eprintln!(
            "benchmark threshold failed: {:.2} < {:.2} ticks/s",
            ticks_per_second, args.min_ticks_per_second
        );
        return ExitCode::from(1);
    }

    ExitCode::SUCCESS
}
