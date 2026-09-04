use clap::Parser;
use v3_cli::bench::{self, ProfileParams};
use v3_cli::RunError;
use v3_core::config::SimulationConfig;

#[derive(Parser)]
#[command(name = "v3-cli")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    Run(RunArgs),
    Bench(BenchArgs),
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum BenchProfile {
    Gate,
    Sweep,
}

#[derive(clap::Args)]
struct BenchArgs {
    #[arg(long, value_enum)]
    profile: BenchProfile,
    #[arg(long)]
    out: Option<std::path::PathBuf>,
    #[arg(long)]
    feature: Option<String>,
    #[arg(long)]
    compare: Vec<std::path::PathBuf>,
    #[arg(long)]
    baseline: Vec<std::path::PathBuf>,
    // Sweep-only parameters.
    #[arg(long)]
    width: Option<u16>,
    #[arg(long)]
    height: Option<u16>,
    #[arg(long)]
    founders: Option<u32>,
    #[arg(long)]
    seeds: Option<String>,
    #[arg(long)]
    ticks: Option<u64>,
    #[arg(long, default_value = "1.0")]
    food_coverage: f32,
}

#[derive(clap::Args)]
struct RunArgs {
    #[arg(long)]
    ticks: u64,
    #[arg(long, default_value = "1")]
    sample_every: u16,
    #[arg(long)]
    seed: u64,
    #[arg(long)]
    config: Option<std::path::PathBuf>,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run(args) => {
            if args.ticks < 1 {
                eprintln!("error: --ticks must be >= 1");
                std::process::exit(1);
            }
            if args.sample_every < 1 {
                eprintln!("error: --sample-every must be >= 1");
                std::process::exit(1);
            }

            let mut config = if let Some(path) = args.config {
                let content = match std::fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("error: failed to read config file: {e}");
                        std::process::exit(1);
                    }
                };
                match serde_json::from_str::<SimulationConfig>(&content) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("error: invalid config: {e}");
                        std::process::exit(1);
                    }
                }
            } else {
                SimulationConfig::default()
            };
            config.normalize();

            let mut out = std::io::stdout();
            match v3_cli::run_simulation(config, args.seed, args.ticks, args.sample_every, &mut out)
            {
                Ok(()) => {}
                Err(RunError::ValidationError(msg)) => {
                    eprintln!("validation error: {msg}");
                    std::process::exit(1);
                }
                Err(RunError::RuntimeError(msg)) => {
                    eprintln!("runtime error: {msg}");
                    std::process::exit(1);
                }
            }
        }
        Commands::Bench(args) => run_bench(args),
    }
}

fn run_bench(args: BenchArgs) {
    let (params, feature) = match args.profile {
        BenchProfile::Gate => {
            let feature = match args.feature {
                Some(f) => f,
                None => {
                    eprintln!("error: --feature is required for --profile gate");
                    std::process::exit(1);
                }
            };
            (bench::gate_profile_params(), feature)
        }
        BenchProfile::Sweep => {
            let width = args.width.unwrap_or_else(|| {
                eprintln!("error: --width is required for --profile sweep");
                std::process::exit(1);
            });
            let height = args.height.unwrap_or_else(|| {
                eprintln!("error: --height is required for --profile sweep");
                std::process::exit(1);
            });
            let founders = args.founders.unwrap_or_else(|| {
                eprintln!("error: --founders is required for --profile sweep");
                std::process::exit(1);
            });
            let ticks = args.ticks.unwrap_or_else(|| {
                eprintln!("error: --ticks is required for --profile sweep");
                std::process::exit(1);
            });
            let seeds_arg = args.seeds.clone().unwrap_or_else(|| {
                eprintln!("error: --seeds is required for --profile sweep");
                std::process::exit(1);
            });
            let seeds: Vec<u64> = match seeds_arg
                .split(',')
                .map(|s| s.trim().parse::<u64>())
                .collect::<Result<Vec<_>, _>>()
            {
                Ok(s) if !s.is_empty() => s,
                _ => {
                    eprintln!("error: --seeds must be a non-empty comma-separated list of u64");
                    std::process::exit(1);
                }
            };
            let params = ProfileParams {
                name: "sweep".to_string(),
                width,
                height,
                founders,
                seeds,
                ticks,
                food_coverage: args.food_coverage,
            };
            let feature = args.feature.unwrap_or_else(|| "sweep".to_string());
            (params, feature)
        }
    };

    let out_path = args.out.clone().unwrap_or_else(|| {
        if args.profile == BenchProfile::Gate {
            std::path::PathBuf::from(format!("docs/progress/features/{feature}.json"))
        } else {
            eprintln!("error: --out is required for --profile sweep");
            std::process::exit(1);
        }
    });

    let mut report = bench::build_report(&params, &feature);

    let mut reference_paths = args.baseline.clone();
    reference_paths.extend(args.compare.clone());
    if reference_paths.is_empty() && args.profile == BenchProfile::Gate {
        let series_path = std::path::PathBuf::from("docs/progress/benchmark-series.json");
        match bench::default_gate_references(&series_path) {
            Ok(paths) => reference_paths = paths,
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
    }

    let severe = match bench::apply_comparisons(&mut report, &reference_paths) {
        Ok(severe) => severe,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("error: failed to create {}: {e}", parent.display());
                std::process::exit(1);
            }
        }
    }
    let json = bench::report_json_pretty(&report);
    if let Err(e) = std::fs::write(&out_path, format!("{json}\n")) {
        eprintln!("error: failed to write {}: {e}", out_path.display());
        std::process::exit(1);
    }

    println!("wrote {}", out_path.display());
    for reference in &report.comparison.references {
        println!(
            "compared against {}: severe={}",
            reference.path, reference.severe
        );
        for counter in &reference.counters {
            println!(
                "  {}: current={} reference={:?} delta%={:?} level={}",
                counter.name,
                counter.current,
                counter.reference,
                counter.percent_delta,
                counter.level
            );
        }
    }

    if severe {
        eprintln!("error: severe work-counter regression against a stored reference");
        std::process::exit(3);
    }
}
