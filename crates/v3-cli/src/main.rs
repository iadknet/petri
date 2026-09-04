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
    /// Force this initial coverage onto every food type. Omit it to keep the
    /// production per-food-type defaults. Sweep-only.
    #[arg(long)]
    food_coverage: Option<f32>,
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

/// Resolve the benchmark profile and feature name from the parsed arguments.
/// Returns the error message `run_bench` prints, so the argument rules are
/// unit-testable without spawning the binary.
fn resolve_bench_profile(args: &BenchArgs) -> Result<(ProfileParams, String), String> {
    match args.profile {
        BenchProfile::Gate => {
            // clap cannot express "forbidden when --profile gate", and
            // silently ignoring the flag would misreport the profile the
            // gate report was generated with.
            if args.food_coverage.is_some() {
                return Err(
                    "--food-coverage is not accepted for --profile gate; the gate profile's \
                     food coverage is predeclared"
                        .to_string(),
                );
            }
            let feature = args
                .feature
                .clone()
                .ok_or("--feature is required for --profile gate")?;
            Ok((bench::gate_profile_params(), feature))
        }
        BenchProfile::Sweep => {
            let width = args
                .width
                .ok_or("--width is required for --profile sweep")?;
            let height = args
                .height
                .ok_or("--height is required for --profile sweep")?;
            let founders = args
                .founders
                .ok_or("--founders is required for --profile sweep")?;
            let ticks = args
                .ticks
                .ok_or("--ticks is required for --profile sweep")?;
            let seeds_arg = args
                .seeds
                .as_deref()
                .ok_or("--seeds is required for --profile sweep")?;
            let seeds: Vec<u64> = seeds_arg
                .split(',')
                .map(|s| s.trim().parse::<u64>())
                .collect::<Result<Vec<_>, _>>()
                .ok()
                .filter(|s| !s.is_empty())
                .ok_or("--seeds must be a non-empty comma-separated list of u64")?;
            let params = ProfileParams {
                name: "sweep".to_string(),
                width,
                height,
                founders,
                seeds,
                ticks,
                food_coverage: args.food_coverage,
            };
            let feature = args.feature.clone().unwrap_or_else(|| "sweep".to_string());
            Ok((params, feature))
        }
    }
}

fn run_bench(args: BenchArgs) {
    let (params, feature) = match resolve_bench_profile(&args) {
        Ok(resolved) => resolved,
        Err(message) => {
            eprintln!("error: {message}");
            std::process::exit(1);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn bench_args(profile: BenchProfile) -> BenchArgs {
        BenchArgs {
            profile,
            out: None,
            feature: Some("t01-f11-baseline-persistence-characterization".to_string()),
            compare: Vec::new(),
            baseline: Vec::new(),
            width: None,
            height: None,
            founders: None,
            seeds: None,
            ticks: None,
            food_coverage: None,
        }
    }

    fn parse_bench(argv: &[&str]) -> BenchArgs {
        match Cli::try_parse_from(argv)
            .expect("arguments must parse")
            .command
        {
            Commands::Bench(args) => args,
            Commands::Run(_) => panic!("expected the bench subcommand"),
        }
    }

    #[test]
    fn omitted_food_coverage_parses_as_none() {
        let args = parse_bench(&[
            "v3-cli",
            "bench",
            "--profile",
            "sweep",
            "--width",
            "128",
            "--height",
            "128",
            "--founders",
            "64",
            "--seeds",
            "11,22,33",
            "--ticks",
            "2000",
        ]);
        assert_eq!(args.food_coverage, None);

        let (params, _) = resolve_bench_profile(&args).expect("sweep arguments are complete");
        assert_eq!(params.food_coverage, None);
        assert_eq!(params.seeds, vec![11, 22, 33]);
    }

    #[test]
    fn explicit_food_coverage_is_forwarded_to_the_sweep_profile() {
        let args = BenchArgs {
            width: Some(32),
            height: Some(32),
            founders: Some(8),
            seeds: Some("1".to_string()),
            ticks: Some(30),
            food_coverage: Some(0.5),
            ..bench_args(BenchProfile::Sweep)
        };

        let (params, feature) = resolve_bench_profile(&args).expect("sweep arguments are complete");
        assert_eq!(params.food_coverage, Some(0.5));
        assert_eq!(feature, "t01-f11-baseline-persistence-characterization");
    }

    #[test]
    fn food_coverage_is_rejected_for_the_gate_profile() {
        let args = BenchArgs {
            food_coverage: Some(0.5),
            ..bench_args(BenchProfile::Gate)
        };

        let error = resolve_bench_profile(&args)
            .expect_err("the gate profile's food coverage is predeclared");
        assert!(
            error.contains("--food-coverage is not accepted for --profile gate"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn the_gate_profile_still_resolves_without_food_coverage() {
        let (params, feature) = resolve_bench_profile(&bench_args(BenchProfile::Gate))
            .expect("the gate profile needs only --feature");

        assert_eq!(params, bench::gate_profile_params());
        assert_eq!(feature, "t01-f11-baseline-persistence-characterization");
    }

    #[test]
    fn a_missing_sweep_argument_is_an_error_rather_than_a_default() {
        let args = bench_args(BenchProfile::Sweep);

        let error = resolve_bench_profile(&args).expect_err("--width is required");
        assert!(error.contains("--width"), "unexpected error: {error}");
    }

    #[test]
    fn an_unparsable_seed_list_is_rejected() {
        let args = BenchArgs {
            width: Some(32),
            height: Some(32),
            founders: Some(8),
            seeds: Some("11,not-a-seed".to_string()),
            ticks: Some(30),
            ..bench_args(BenchProfile::Sweep)
        };

        let error = resolve_bench_profile(&args).expect_err("seeds must parse as u64");
        assert!(error.contains("--seeds"), "unexpected error: {error}");
    }
}
