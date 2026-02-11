use clap::Parser;
use petri_core::{World, WorldConfig};

#[derive(Debug, Parser)]
#[command(name = "petri-cli", about = "Headless simulation runner")]
struct Args {
    #[arg(long, default_value_t = 500)]
    ticks: u64,
    #[arg(long, default_value_t = 25)]
    sample_every: u64,
    #[arg(long, default_value_t = 42)]
    seed: u64,
}

fn main() {
    let args = Args::parse();
    let config = WorldConfig::default();
    let mut world = World::new(config, args.seed);
    let lines = collect_stats_lines(&mut world, args.ticks, args.sample_every);
    for line in lines {
        println!("{line}");
    }
}

fn collect_stats_lines(world: &mut World, ticks: u64, sample_every: u64) -> Vec<String> {
    let interval = sample_every.max(1);
    let mut lines = Vec::new();

    for step in 1..=ticks {
        world.tick();
        if step % interval == 0 || step == ticks {
            let frame = world.frame();
            lines.push(format!(
                "tick={} population={} avg_energy={:.4}",
                frame.tick, frame.population, frame.average_energy
            ));
        }
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_lines_include_tick_population_and_energy() {
        let cfg = WorldConfig {
            width: 20,
            height: 20,
            initial_creatures: 10,
            max_creatures: 100,
            ..WorldConfig::default()
        };
        let mut world = World::new(cfg, 5);
        let lines = collect_stats_lines(&mut world, 5, 1);
        assert!(!lines.is_empty());
        assert!(lines[0].contains("tick="));
        assert!(lines[0].contains("population="));
        assert!(lines[0].contains("avg_energy="));
    }
}
