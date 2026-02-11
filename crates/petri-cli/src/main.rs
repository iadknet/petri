use clap::{Parser, Subcommand};
use petri_core::{ControllerPalette, World, WorldConfig};

#[derive(Debug, Parser)]
#[command(name = "petri-cli", about = "Headless simulation runner")]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
    #[arg(long, default_value_t = 500, global = true)]
    ticks: u64,
    #[arg(long, default_value_t = 25, global = true)]
    sample_every: u64,
    #[arg(long, default_value_t = 42, global = true)]
    seed: u64,
}

#[derive(Debug, Subcommand, Clone, Copy)]
enum Command {
    Run,
    Ablation,
}

#[derive(Debug, Clone)]
struct AblationResult {
    palette: ControllerPalette,
    final_population: usize,
    average_energy: f32,
    moves: u64,
    eats: u64,
    reproductions: u64,
    deaths: u64,
}

fn main() {
    let args = Args::parse();
    let mode = args.command.unwrap_or(Command::Run);

    match mode {
        Command::Run => {
            let config = WorldConfig::default();
            let mut world = World::new(config, args.seed);
            let lines = collect_stats_lines(&mut world, args.ticks, args.sample_every);
            for line in lines {
                println!("{line}");
            }
        }
        Command::Ablation => {
            let config = WorldConfig::default();
            let results = run_ablation(config, args.ticks, args.seed);
            println!("{}", format_ablation_results(&results));
        }
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

fn run_ablation(config: WorldConfig, ticks: u64, seed: u64) -> Vec<AblationResult> {
    let palettes = [
        ControllerPalette::NeuralOnly,
        ControllerPalette::LogicOnly,
        ControllerPalette::Hybrid,
    ];

    palettes
        .into_iter()
        .enumerate()
        .map(|(i, palette)| {
            let mut world = World::new_with_palette(config.clone(), seed + i as u64, palette);
            for _ in 0..ticks {
                world.tick();
            }
            let frame = world.frame();
            let diag = world.diagnostics();
            AblationResult {
                palette,
                final_population: frame.population,
                average_energy: frame.average_energy,
                moves: diag.moves,
                eats: diag.eats,
                reproductions: diag.reproductions,
                deaths: diag.deaths,
            }
        })
        .collect()
}

fn format_ablation_results(results: &[AblationResult]) -> String {
    let mut lines =
        vec!["palette,population,avg_energy,moves,eats,reproductions,deaths".to_string()];

    for result in results {
        lines.push(format!(
            "{:?},{},{:.4},{},{},{},{}",
            result.palette,
            result.final_population,
            result.average_energy,
            result.moves,
            result.eats,
            result.reproductions,
            result.deaths
        ));
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use petri_core::ControllerPalette;

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

    #[test]
    fn ablation_returns_three_palette_results() {
        let cfg = WorldConfig {
            width: 20,
            height: 20,
            initial_creatures: 20,
            max_creatures: 200,
            ..WorldConfig::default()
        };

        let results = run_ablation(cfg, 40, 9);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].palette, ControllerPalette::NeuralOnly);
        assert_eq!(results[1].palette, ControllerPalette::LogicOnly);
        assert_eq!(results[2].palette, ControllerPalette::Hybrid);
    }

    #[test]
    fn ablation_output_contains_palette_names() {
        let results = vec![
            AblationResult {
                palette: ControllerPalette::NeuralOnly,
                final_population: 10,
                average_energy: 0.2,
                moves: 20,
                eats: 5,
                reproductions: 3,
                deaths: 8,
            },
            AblationResult {
                palette: ControllerPalette::LogicOnly,
                final_population: 11,
                average_energy: 0.3,
                moves: 21,
                eats: 6,
                reproductions: 4,
                deaths: 9,
            },
            AblationResult {
                palette: ControllerPalette::Hybrid,
                final_population: 12,
                average_energy: 0.4,
                moves: 22,
                eats: 7,
                reproductions: 5,
                deaths: 10,
            },
        ];

        let output = format_ablation_results(&results);
        assert!(output.contains("NeuralOnly"));
        assert!(output.contains("LogicOnly"));
        assert!(output.contains("Hybrid"));
    }
}
