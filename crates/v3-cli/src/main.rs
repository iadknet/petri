use std::num::NonZeroUsize;

use clap::Parser;
use v3_cli::bench::artifacts;
use v3_cli::bench::{self, NeighborhoodSizes, ProfileParams};
use v3_cli::RunError;
use v3_core::config::SimulationConfig;

#[derive(Parser)]
#[command(name = "v3-cli")]
struct Cli {
    /// Export run telemetry over OTLP (`on` or `off`); beats PETRI_TELEMETRY,
    /// default off. `run` exports its run; `bench`, `recruitment` and
    /// `input-opportunity` export one measurement record pair (and `bench`
    /// one run per seed) after their work ends.
    #[cfg(feature = "telemetry")]
    #[arg(long, global = true, value_name = "on|off")]
    telemetry: Option<v3_telemetry::Switch>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    Run(RunArgs),
    Bench(BenchArgs),
    /// Convert a full benchmark artifact without rerunning observations.
    BenchSummarize(SummarizeArgs),
    /// Saved-world tooling.
    World(WorldArgs),
    /// The T13.F07 S0 recruitment panel under production supply: a compact
    /// raw record under the artifact root and a summary under
    /// docs/progress/features. Exit status 3 when a cap stopped the run and
    /// the record is marked incomplete.
    Recruitment(RecruitmentArgs),
    /// The T20.F01 ecological opportunity assay: a raw replicate record
    /// under the artifact root and a summary under docs/progress/features.
    /// Exit status 3 when a cap stopped the run and the record is marked
    /// incomplete.
    InputOpportunity(InputOpportunityArgs),
}

#[derive(clap::Args)]
struct InputOpportunityArgs {
    #[arg(long)]
    feature: String,
    /// One replicate per world: the pilot that projects the full run.
    #[arg(long)]
    pilot: bool,
    /// Private rayon pool of this many threads (>= 1). Omit it to use the
    /// rayon global pool.
    #[arg(long)]
    threads: Option<NonZeroUsize>,
    /// Stop between replicates once this many seconds have elapsed.
    #[arg(long, default_value_t = v3_cli::opportunity::DEFAULT_WALL_CAP_SECS)]
    wall_cap_secs: u64,
    /// Stop between replicates once the raw record reaches this many bytes.
    #[arg(long, default_value_t = v3_cli::opportunity::DEFAULT_BYTE_CAP)]
    byte_cap: u64,
    /// Raw record destination; the summary path is unaffected.
    #[arg(long)]
    out: Option<std::path::PathBuf>,
    /// Summary destination; the raw path is unaffected.
    #[arg(long)]
    summary_out: Option<std::path::PathBuf>,
}

#[derive(clap::Args)]
struct RecruitmentArgs {
    #[arg(long)]
    feature: String,
    /// Batch 0, lineages 0-1, every arm: the feasibility pilot whose
    /// lineages are a prefix of the panel.
    #[arg(long)]
    pilot: bool,
    /// Private rayon pool of this many threads (>= 1); arms and lineages run
    /// in parallel. Omit it to use the rayon global pool.
    #[arg(long)]
    threads: Option<NonZeroUsize>,
    /// Stop between lineages once this many seconds have elapsed.
    #[arg(long, default_value_t = v3_cli::recruitment::DEFAULT_WALL_CAP_SECS)]
    wall_cap_secs: u64,
    /// Stop between lineages once the raw record reaches this many bytes.
    #[arg(long, default_value_t = v3_cli::recruitment::DEFAULT_BYTE_CAP)]
    byte_cap: u64,
    /// Rebuild every proposal from the written record and compare
    /// fingerprints; the result goes into the summary.
    #[arg(long)]
    replay_check: bool,
    /// Raw record destination; the summary path is unaffected.
    #[arg(long)]
    out: Option<std::path::PathBuf>,
    /// Summary destination; the raw path is unaffected.
    #[arg(long)]
    summary_out: Option<std::path::PathBuf>,
}

#[derive(clap::Args)]
struct WorldArgs {
    #[command(subcommand)]
    command: WorldCommands,
}

#[derive(clap::Subcommand)]
enum WorldCommands {
    /// Print one seeded world's tick-zero readings as JSON, and optionally
    /// write a downsampled two-panel PNG preview of it.
    Inspect(InspectArgs),
}

#[derive(clap::Args)]
struct InspectArgs {
    /// Partial world recipe, resolved over the defaults exactly as the goal
    /// profile resolves its baseline worlds.
    #[arg(long)]
    config: std::path::PathBuf,
    /// Run seed: it places food and founders, and supplies the map seed when
    /// the recipe pins none.
    #[arg(long)]
    seed: u64,
    /// Write the preview PNG here instead of only printing the readings.
    #[arg(long)]
    png: Option<std::path::PathBuf>,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum BenchProfile {
    Gate,
    Goal,
    Sweep,
}

#[derive(clap::Args)]
struct BenchArgs {
    #[arg(long, value_enum)]
    profile: BenchProfile,
    #[arg(long)]
    out: Option<std::path::PathBuf>,
    /// Summary destination; --out controls the local full report.
    #[arg(long)]
    summary_out: Option<std::path::PathBuf>,
    #[arg(long)]
    feature: Option<String>,
    #[arg(long)]
    compare: Vec<std::path::PathBuf>,
    #[arg(long)]
    baseline: Vec<std::path::PathBuf>,
    // Sweep-only parameters.
    #[arg(long)]
    config: Option<std::path::PathBuf>,
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
    /// Run the profile on a private rayon pool of this many threads (>= 1),
    /// which scopes the parallel cognition phase to it. Omit it to use the
    /// rayon global pool. Accepted for both profiles.
    #[arg(long)]
    threads: Option<usize>,
}

#[derive(clap::Args)]
struct SummarizeArgs {
    /// A full benchmark report to project to a v2 summary.
    #[arg(
        long,
        requires = "provenance",
        required_unless_present = "from_summary_v1"
    )]
    input: Option<std::path::PathBuf>,
    /// A committed v1 summary to convert to v2; its own provenance is kept.
    #[arg(long, conflicts_with_all = ["input", "provenance"])]
    from_summary_v1: Option<std::path::PathBuf>,
    #[arg(long)]
    out: std::path::PathBuf,
    /// Fixed converter identity and verification time; see docs/benchmark-artifacts.md.
    #[arg(long)]
    provenance: Option<std::path::PathBuf>,
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
    /// Save the full applied recipe before running. Run seed is supplied separately.
    #[arg(long)]
    save_config: Option<std::path::PathBuf>,
}

fn main() {
    let cli = Cli::parse();
    #[cfg(feature = "telemetry")]
    let switch = v3_telemetry::switch_from_env(cli.telemetry).unwrap_or_else(|message| {
        eprintln!("error: {message}");
        std::process::exit(1);
    });
    #[cfg(not(feature = "telemetry"))]
    let switch = Switch;
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

            let config = match load_config(args.config.as_deref()) {
                Ok(config) => config,
                Err(message) => {
                    eprintln!("error: {message}");
                    std::process::exit(1);
                }
            };
            if let Some(path) = args.save_config {
                if let Err(message) = save_config(&config, &path) {
                    eprintln!("error: {message}");
                    std::process::exit(1);
                }
            }

            #[cfg(feature = "telemetry")]
            v3_cli::telemetry::install(
                v3_telemetry::Telemetry::start(v3_telemetry::Service::Cli, switch).unwrap_or_else(
                    |message| {
                        eprintln!("error: {message}");
                        std::process::exit(1);
                    },
                ),
                args.config.as_ref().map(|path| path.display().to_string()),
            );

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
        Commands::Bench(args) => run_bench(&args, switch),
        Commands::BenchSummarize(args) => {
            let result = match (&args.from_summary_v1, &args.input, &args.provenance) {
                (Some(v1), _, _) => artifacts::convert_summary_v1(v1, &args.out),
                (None, Some(input), Some(provenance)) => std::fs::read(provenance)
                    .map_err(|e| format!("failed to read {}: {e}", provenance.display()))
                    .and_then(|bytes| {
                        serde_json::from_slice(&bytes)
                            .map_err(|e| format!("invalid provenance: {e}"))
                    })
                    .and_then(|provenance| artifacts::convert(input, &args.out, &provenance)),
                _ => Err("--input with --provenance, or --from-summary-v1, is required".into()),
            };
            match result {
                Ok(summary) => println!(
                    "wrote {} ({} raw bytes; sha256 {})",
                    args.out.display(),
                    summary.raw.bytes,
                    summary.raw.sha256
                ),
                Err(e) => {
                    eprintln!("error: {e}");
                    std::process::exit(1);
                }
            }
        }
        Commands::Recruitment(args) => run_recruitment(args, switch),
        Commands::InputOpportunity(args) => run_input_opportunity(args, switch),
        Commands::World(args) => match args.command {
            WorldCommands::Inspect(args) => {
                if let Err(message) = run_world_inspect(&args, &mut std::io::stdout()) {
                    eprintln!("validation error: {message}");
                    std::process::exit(1);
                }
            }
        },
    }
}

/// Resolve, seed, and read one world, printing its readings as JSON and
/// writing the preview when `--png` asks for one. Every failure here is an
/// input problem the caller can fix, so they all read as validation errors.
fn run_world_inspect(args: &InspectArgs, out: &mut impl std::io::Write) -> Result<(), String> {
    let recipe_path = args.config.display().to_string();
    let config = v3_cli::inspect::resolve_baseline_world(read_recipe(&args.config)?, &recipe_path)?;
    let sim = v3_core::simulation::seed_simulation(config, args.seed);
    if let Some(path) = &args.png {
        std::fs::write(path, v3_cli::inspect::render_preview(&sim))
            .map_err(|error| format!("failed to write preview {}: {error}", path.display()))?;
    }
    let reading = v3_cli::inspect::read_world(&sim, &recipe_path, args.seed);
    let json = serde_json::to_string_pretty(&reading).expect("readings must serialize");
    writeln!(out, "{json}").map_err(|error| format!("failed to write readings: {error}"))
}

fn read_recipe(path: &std::path::Path) -> Result<serde_json::Value, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read config {}: {error}", path.display()))?;
    serde_json::from_str(&content)
        .map_err(|error| format!("invalid config {}: {error}", path.display()))
}

fn save_config(config: &SimulationConfig, path: &std::path::Path) -> Result<(), String> {
    let json = serde_json::to_string_pretty(config).expect("config must be serializable");
    std::fs::write(path, format!("{json}\n"))
        .map_err(|error| format!("failed to save config {}: {error}", path.display()))
}

fn load_config(path: Option<&std::path::Path>) -> Result<SimulationConfig, String> {
    let patch = match path {
        Some(path) => read_recipe(path)?,
        None => serde_json::json!({}),
    };
    v3_core::config::resolve_config(&SimulationConfig::default(), patch)
        .map_err(|error| format!("invalid config: {error}"))
}

/// The benchmark profile, feature name, and thread count a `bench` run needs.
#[derive(Debug)]
struct ResolvedBench {
    params: ProfileParams,
    feature: String,
    /// `None` means the rayon global pool.
    threads: Option<NonZeroUsize>,
}

/// Resolve the benchmark profile, feature name, and thread count from the
/// parsed arguments. Returns the error message `run_bench` prints, so the
/// argument rules are unit-testable without spawning the binary.
fn resolve_bench_profile(args: &BenchArgs) -> Result<ResolvedBench, String> {
    // A zero-thread pool is not a pool. Rejected for both profiles, before
    // any profile-specific rule, so the message never depends on --profile.
    let threads = args
        .threads
        .map(|threads| NonZeroUsize::new(threads).ok_or("--threads must be >= 1"))
        .transpose()?;
    let (params, feature) = resolve_profile_params(args)?;
    Ok(ResolvedBench {
        params,
        feature,
        threads,
    })
}

fn resolve_profile_params(args: &BenchArgs) -> Result<(ProfileParams, String), String> {
    if args.config.is_some() && args.profile != BenchProfile::Sweep {
        return Err("--config is only accepted for --profile sweep".into());
    }
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
        BenchProfile::Goal => {
            for (flag, supplied) in [
                ("--width", args.width.is_some()),
                ("--height", args.height.is_some()),
                ("--founders", args.founders.is_some()),
                ("--seeds", args.seeds.is_some()),
                ("--ticks", args.ticks.is_some()),
                ("--food-coverage", args.food_coverage.is_some()),
            ] {
                if supplied {
                    return Err(format!(
                        "{flag} is not accepted for --profile goal; the goal profile is predeclared"
                    ));
                }
            }
            let feature = args
                .feature
                .clone()
                .ok_or("--feature is required for --profile goal")?;
            Ok((bench::goal_profile_params(), feature))
        }
        BenchProfile::Sweep => {
            let recipe = args
                .config
                .as_ref()
                .map(|path| {
                    load_config(Some(path)).map(|config| bench::Recipe {
                        path: path.display().to_string(),
                        config,
                    })
                })
                .transpose()?;
            let width = args
                .width
                .or_else(|| recipe.as_ref().map(|r| r.config.world.width))
                .ok_or("--width is required for --profile sweep")?;
            let height = args
                .height
                .or_else(|| recipe.as_ref().map(|r| r.config.world.height))
                .ok_or("--height is required for --profile sweep")?;
            let founders = args
                .founders
                .or_else(|| {
                    recipe
                        .as_ref()
                        .map(|r| r.config.population.initial_creatures)
                })
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
            let mut params = ProfileParams {
                recipe,
                name: "sweep".to_string(),
                width,
                height,
                founders,
                seeds,
                ticks,
                food_coverage: args.food_coverage,
                neighborhood: NeighborhoodSizes::default(),
                drift: Default::default(),
                recruitment: v3_core::neighborhood::recruitment_paths::Sizes::PRODUCTION,
                mutation_effects: Default::default(),
            };
            if params.recipe.is_some() {
                let config = bench::build_config(&params);
                params.width = config.world.width;
                params.height = config.world.height;
                params.founders = config.population.initial_creatures;
                if let Some(recipe) = &mut params.recipe {
                    recipe.config = config;
                }
            }
            let feature = args.feature.clone().unwrap_or_else(|| "sweep".to_string());
            Ok((params, feature))
        }
    }
}

/// Exits with a measurement command's `code` once its telemetry has closed;
/// `0` returns.
fn exit_with(code: u8) {
    if code != 0 {
        std::process::exit(i32::from(code));
    }
}

fn working_directory() -> std::path::PathBuf {
    std::env::current_dir().unwrap_or_else(|e| {
        eprintln!("error: cannot identify the working directory: {e}");
        std::process::exit(1);
    })
}

fn run_input_opportunity(args: InputOpportunityArgs, switch: Switch) {
    let options = v3_cli::opportunity::Options {
        pilot: args.pilot,
        threads: args.threads,
        wall_cap: std::time::Duration::from_secs(args.wall_cap_secs),
        byte_cap: args.byte_cap,
        raw: args.out,
        summary: args.summary_out,
        source_revision: bench::detect_git_revision(),
        ..v3_cli::opportunity::Options::new(&args.feature, working_directory())
    };
    let measurement = measure::opportunity(switch, &options);
    let result = v3_cli::opportunity::run(&options);
    let code = match &result {
        Ok(outcome) => {
            println!(
                "wrote {} ({} bytes, {} replicates) and {}",
                outcome.raw.display(),
                outcome.bytes,
                outcome.replicates,
                outcome.summary.display()
            );
            if outcome.incomplete {
                eprintln!("incomplete: a cap stopped the run between replicates");
                3
            } else {
                0
            }
        }
        Err(e) => {
            eprintln!("error: {e}");
            1
        }
    };
    measure::opportunity_end(
        measurement,
        code,
        result
            .as_ref()
            .ok()
            .map(|outcome| outcome.summary.as_path()),
    );
    exit_with(code);
}

fn run_recruitment(args: RecruitmentArgs, switch: Switch) {
    let options = v3_cli::recruitment::Options {
        pilot: args.pilot,
        threads: args.threads,
        wall_cap: std::time::Duration::from_secs(args.wall_cap_secs),
        byte_cap: args.byte_cap,
        replay_check: args.replay_check,
        raw: args.out,
        summary: args.summary_out,
        source_revision: bench::detect_git_revision(),
        ..v3_cli::recruitment::Options::new(&args.feature, working_directory())
    };
    let measurement = measure::recruitment(switch, &options);
    let result = v3_cli::recruitment::run(&options);
    let code = match &result {
        Ok(outcome) => {
            println!(
                "wrote {} ({} bytes, {} lineages, {} proposals) and {}",
                outcome.raw.display(),
                outcome.bytes,
                outcome.lineage_count,
                outcome.proposal_count,
                outcome.summary.display()
            );
            if let Some(check) = outcome.replay_check {
                println!(
                    "replay check: {}/{} proposals matched",
                    check.matched, check.proposals
                );
            }
            if outcome.incomplete {
                eprintln!("incomplete: a cap stopped the run between lineages");
                3
            } else {
                0
            }
        }
        Err(e) => {
            eprintln!("error: {e}");
            1
        }
    };
    measure::recruitment_end(
        measurement,
        code,
        result
            .as_ref()
            .ok()
            .map(|outcome| outcome.summary.as_path()),
    );
    exit_with(code);
}

fn run_bench(args: &BenchArgs, switch: Switch) {
    let prepared = prepare_bench(args).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        std::process::exit(1);
    });
    let measurement = measure::bench(switch, &prepared);
    let result = execute_bench(prepared);
    let code = match &result {
        Ok(done) if done.severe => {
            eprintln!("error: severe work-counter regression against a stored reference");
            3
        }
        Ok(_) => 0,
        Err(e) => {
            eprintln!("error: {e}");
            1
        }
    };
    measure::bench_end(measurement, code, &result);
    exit_with(code);
}

/// A benchmark whose arguments, outputs and references are resolved: what
/// runs after `measurement.started`.
struct PreparedBench {
    params: ProfileParams,
    feature: String,
    /// `--feature`, when given.
    #[cfg_attr(not(feature = "telemetry"), allow(dead_code))]
    label: Option<String>,
    threads: Option<NonZeroUsize>,
    invocation: artifacts::Invocation,
    #[cfg_attr(not(feature = "telemetry"), allow(dead_code))]
    profile: &'static str,
    paths: artifacts::OutputPaths,
    selection: bench::ReferenceSelection,
}

/// What a completed benchmark wrote.
struct BenchDone {
    severe: bool,
    #[cfg(feature = "telemetry")]
    summary_path: String,
    #[cfg(feature = "telemetry")]
    raw: (String, u64),
    #[cfg(feature = "telemetry")]
    totals: bench::Totals,
}

fn prepare_bench(args: &BenchArgs) -> Result<PreparedBench, String> {
    let ResolvedBench {
        params,
        feature,
        threads,
    } = resolve_bench_profile(args)?;
    let invocation = artifacts::Invocation::capture()?;
    let profile = match args.profile {
        BenchProfile::Gate => "gate",
        BenchProfile::Goal => "goal",
        BenchProfile::Sweep => "sweep",
    };
    let paths = artifacts::output_paths(
        &invocation.working_directory,
        profile,
        args.feature.as_deref(),
        args.out.as_deref(),
        args.summary_out.as_deref(),
    )?;

    let mut explicit_paths = args.baseline.clone();
    explicit_paths.extend(args.compare.clone());
    let series_path = std::path::Path::new("docs/progress/benchmark-series.json");
    let selection = if !explicit_paths.is_empty() {
        Ok(bench::ReferenceSelection {
            paths: explicit_paths,
            absence: None,
        })
    } else {
        match args.profile {
            BenchProfile::Gate => bench::default_gate_references(series_path),
            BenchProfile::Goal => bench::default_goal_references(series_path),
            BenchProfile::Sweep => Ok(bench::ReferenceSelection::default()),
        }
    };
    let selection = selection?;
    if args.out.is_some() {
        for reference in &selection.paths {
            if artifacts::same_path(&paths.raw, reference)? {
                return Err(format!(
                    "explicit raw output would overwrite comparison reference {}",
                    reference.display()
                ));
            }
        }
    }
    Ok(PreparedBench {
        params,
        feature,
        label: args.feature.clone(),
        threads,
        invocation,
        profile,
        paths,
        selection,
    })
}

fn execute_bench(prepared: PreparedBench) -> Result<BenchDone, String> {
    let PreparedBench {
        params,
        feature,
        threads,
        invocation,
        paths,
        selection,
        ..
    } = prepared;
    let mut report = bench::build_report_with_threads(&params, &feature, threads)?;
    let severe = bench::apply_comparisons_for_outputs(
        &mut report,
        &selection,
        &[&paths.raw, &paths.summary],
    )?;
    #[derive(serde::Serialize)]
    struct MeasuredReport<'a> {
        #[serde(flatten)]
        report: &'a bench::Report,
        measurement_evidence: serde_json::Value,
    }
    let mut measurement_evidence = artifacts::measurement_evidence(&invocation, severe);
    // World-set identities are already captured per case. Other profiles also
    // retain the effective config identity without changing comparison inputs.
    if report.deterministic.profile.cases.is_empty() {
        measurement_evidence["effective_config_digest"] = serde_json::json!(
            v3_core::config::config_digest(&bench::build_config(&params))
        );
    }
    artifacts::write_json(
        &paths.raw,
        &MeasuredReport {
            report: &report,
            measurement_evidence,
        },
    )?;
    let comparison = std::mem::take(&mut report.comparison);
    #[cfg(feature = "telemetry")]
    let totals = report.deterministic.totals;
    drop(report);
    let provenance = artifacts::ConversionProvenance {
        verified_at: bench::rfc3339_now(),
        converter: invocation,
        supplied_evidence: None,
        from_summary_v1: None,
    };
    #[cfg_attr(not(feature = "telemetry"), allow(unused_variables))]
    let summary = artifacts::convert(&paths.raw, &paths.summary, &provenance)?;

    println!(
        "wrote raw {} and summary {}",
        paths.raw.display(),
        paths.summary.display()
    );
    if let Some(cause) = &comparison.reference_absence {
        println!("no comparison reference: {cause}");
    }
    for reference in &comparison.references {
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
    Ok(BenchDone {
        severe,
        #[cfg(feature = "telemetry")]
        summary_path: paths.summary.display().to_string(),
        #[cfg(feature = "telemetry")]
        raw: (summary.raw.sha256, summary.raw.bytes as u64),
        #[cfg(feature = "telemetry")]
        totals,
    })
}

/// The `--telemetry` switch; the reference build has none.
#[cfg(feature = "telemetry")]
type Switch = v3_telemetry::Switch;

/// The reference build's stand-in for the `--telemetry` switch.
#[cfg(not(feature = "telemetry"))]
#[derive(Clone, Copy)]
struct Switch;

/// Measurement records (T21.F06): `measurement.started` after argument
/// resolution and before the command's work, `measurement.ended` after its
/// output, then release and the bounded flush. Summary readings are taken
/// from the files the command already wrote.
#[cfg(feature = "telemetry")]
mod measure {
    use std::path::Path;

    use serde_json::{json, Value};
    use v3_cli::telemetry::Measurement;
    use v3_telemetry::{MeasurementEnd, MeasurementStart};

    use super::{bench, BenchDone, PreparedBench, Switch};

    /// Starts the measurement, or exits 1 before any record when a telemetry
    /// setting is invalid.
    fn begin(switch: Switch, start: MeasurementStart<'_>) -> Measurement {
        Measurement::begin(switch, start).unwrap_or_else(|message| {
            eprintln!("error: {message}");
            std::process::exit(1);
        })
    }

    fn threads(threads: Option<std::num::NonZeroUsize>) -> Option<u64> {
        threads.map(|threads| threads.get() as u64)
    }

    pub(super) fn bench(switch: Switch, prepared: &PreparedBench) -> Measurement {
        // The goal world set has no one effective config: its cases carry
        // their own digests.
        let digest = (prepared.profile != "goal")
            .then(|| v3_core::config::config_digest(&bench::build_config(&prepared.params)));
        begin(
            switch,
            MeasurementStart {
                command: "bench",
                feature: prepared.label.as_deref(),
                profile: Some(prepared.profile),
                seeds: Some(&prepared.params.seeds),
                config_digest: digest.as_deref(),
                threads: threads(prepared.threads),
                ..MeasurementStart::default()
            },
        )
    }

    pub(super) fn bench_end(
        measurement: Measurement,
        code: u8,
        result: &Result<BenchDone, String>,
    ) {
        let mut end = MeasurementEnd {
            exit_code: code,
            ..MeasurementEnd::default()
        };
        if let Ok(done) = result {
            end.severe = Some(done.severe);
            end.summary_path = Some(&done.summary_path);
            end.raw_sha256 = Some(&done.raw.0);
            end.raw_bytes = Some(done.raw.1);
            end.body = serde_json::to_value(done.totals)
                .ok()
                .map(|totals| totals.to_string());
        }
        measurement.finish(end);
    }

    pub(super) fn recruitment(
        switch: Switch,
        options: &v3_cli::recruitment::Options,
    ) -> Measurement {
        let digest = v3_core::config::config_digest(
            &v3_core::neighborhood::recruitment_paths::task_config(),
        );
        begin(
            switch,
            MeasurementStart {
                command: "recruitment",
                feature: Some(&options.feature),
                pilot: Some(options.pilot),
                config_digest: Some(&digest),
                threads: threads(options.threads),
                ..MeasurementStart::default()
            },
        )
    }

    pub(super) fn opportunity(
        switch: Switch,
        options: &v3_cli::opportunity::Options,
    ) -> Measurement {
        begin(
            switch,
            MeasurementStart {
                command: "input-opportunity",
                feature: Some(&options.feature),
                pilot: Some(options.pilot),
                threads: threads(options.threads),
                ..MeasurementStart::default()
            },
        )
    }

    /// An assay's `measurement.ended`: the written summary's readings, and
    /// `body` of it as the totals; no summary (an error exit) records only
    /// the code.
    fn assay_end(
        measurement: Measurement,
        code: u8,
        summary_path: Option<&Path>,
        body: fn(&Value) -> Value,
    ) {
        let summary = summary_path.and_then(|path| {
            let bytes = std::fs::read(path).ok()?;
            Some((path, serde_json::from_slice::<Value>(&bytes).ok()?))
        });
        let mut end = MeasurementEnd {
            exit_code: code,
            ..MeasurementEnd::default()
        };
        let path_text;
        if let Some((path, summary)) = &summary {
            path_text = path.display().to_string();
            end.summary_path = Some(&path_text);
            end.incomplete = summary["incomplete"].as_bool();
            end.stop_reason = summary["stop_reason"].as_str();
            end.horizon = summary["horizon"].as_u64();
            end.gate_favorable = summary["gate_favorable"].as_bool();
            end.raw_sha256 = summary["raw"]["sha256"].as_str();
            end.raw_bytes = summary["raw"]["bytes"].as_u64();
            end.body = Some(body(summary).to_string());
        }
        measurement.finish(end);
    }

    pub(super) fn recruitment_end(measurement: Measurement, code: u8, summary: Option<&Path>) {
        assay_end(measurement, code, summary, |summary| {
            json!({
                "expected_proposals": summary["expected_proposals"],
                "lineage_count": summary["lineage_count"],
                "proposal_count": summary["proposal_count"],
            })
        });
    }

    /// Per world, its case, requested replicates, and the count and sums of
    /// its replicate rows; per family and world, the verdict row's counts.
    pub(super) fn opportunity_end(measurement: Measurement, code: u8, summary: Option<&Path>) {
        assay_end(measurement, code, summary, |summary| {
            let worlds = summary["worlds"]
                .as_array()
                .map(Vec::as_slice)
                .unwrap_or_default();
            let sum = |rows: &[Value], field: &str| -> u64 {
                rows.iter().filter_map(|row| row[field].as_u64()).sum()
            };
            let world_rows: Vec<Value> = worlds
                .iter()
                .map(|world| {
                    let rows = world["replicates"]
                        .as_array()
                        .map(Vec::as_slice)
                        .unwrap_or_default();
                    json!({
                        "case": world["case"]["name"],
                        "replicates_requested": world["replicates_requested"],
                        "replicates": rows.len(),
                        "ticks": sum(rows, "ticks"),
                        "births_total": sum(rows, "births_total"),
                    })
                })
                .collect();
            let verdicts: Vec<Value> = worlds
                .iter()
                .flat_map(|world| world["verdicts"].as_array().cloned().unwrap_or_default())
                .map(|row| {
                    json!({
                        "family": row["family"],
                        "world": row["world"],
                        "verdict": row["verdict"],
                        "sampled": row["sampled"],
                        "exposed": row["exposed"],
                        "applied": row["applied"],
                    })
                })
                .collect();
            json!({ "worlds": world_rows, "verdicts": verdicts })
        });
    }
}

/// The reference build records nothing.
#[cfg(not(feature = "telemetry"))]
mod measure {
    use std::path::Path;

    use super::{BenchDone, PreparedBench, Switch};

    pub(super) struct Measurement;

    pub(super) fn bench(_: Switch, _: &PreparedBench) -> Measurement {
        Measurement
    }

    pub(super) fn bench_end(_: Measurement, _: u8, _: &Result<BenchDone, String>) {}

    pub(super) fn recruitment(_: Switch, _: &v3_cli::recruitment::Options) -> Measurement {
        Measurement
    }

    pub(super) fn opportunity(_: Switch, _: &v3_cli::opportunity::Options) -> Measurement {
        Measurement
    }

    pub(super) fn recruitment_end(_: Measurement, _: u8, _: Option<&Path>) {}

    pub(super) fn opportunity_end(_: Measurement, _: u8, _: Option<&Path>) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "telemetry")]
    fn switch_for(argv: &[&str], env: Option<&str>) -> v3_telemetry::Switch {
        let cli = Cli::try_parse_from(argv).expect("arguments must parse");
        v3_telemetry::resolve_switch(cli.telemetry, env).expect("the switch must resolve")
    }

    #[cfg(feature = "telemetry")]
    #[test]
    fn telemetry_flag_beats_environment_which_beats_default_off() {
        use v3_telemetry::Switch;
        let run = ["v3-cli", "run", "--ticks", "1", "--seed", "1"];
        assert_eq!(switch_for(&run, None), Switch::Off);
        assert_eq!(switch_for(&run, Some("on")), Switch::On);
        let off = [
            "v3-cli",
            "--telemetry",
            "off",
            "run",
            "--ticks",
            "1",
            "--seed",
            "1",
        ];
        assert_eq!(switch_for(&off, Some("on")), Switch::Off);
        let on = [
            "v3-cli",
            "--telemetry",
            "on",
            "run",
            "--ticks",
            "1",
            "--seed",
            "1",
        ];
        assert_eq!(switch_for(&on, None), Switch::On);
        // Global: accepted after the subcommand and by every subcommand.
        let after = [
            "v3-cli",
            "run",
            "--ticks",
            "1",
            "--seed",
            "1",
            "--telemetry",
            "on",
        ];
        assert_eq!(switch_for(&after, None), Switch::On);
        let bench = ["v3-cli", "--telemetry", "on", "bench", "--profile", "gate"];
        assert_eq!(switch_for(&bench, None), Switch::On);
        assert!(Cli::try_parse_from(["v3-cli", "--telemetry", "maybe", "run"]).is_err());
    }

    fn bench_args(profile: BenchProfile) -> BenchArgs {
        BenchArgs {
            profile,
            config: None,
            out: None,
            summary_out: None,
            feature: Some("t01-f11-baseline-persistence-characterization".to_string()),
            compare: Vec::new(),
            baseline: Vec::new(),
            width: None,
            height: None,
            founders: None,
            seeds: None,
            ticks: None,
            food_coverage: None,
            threads: None,
        }
    }

    fn parse_bench(argv: &[&str]) -> BenchArgs {
        match Cli::try_parse_from(argv)
            .expect("arguments must parse")
            .command
        {
            Commands::Bench(args) => args,
            Commands::Run(_)
            | Commands::World(_)
            | Commands::BenchSummarize(_)
            | Commands::Recruitment(_)
            | Commands::InputOpportunity(_) => {
                panic!("expected the bench subcommand")
            }
        }
    }

    #[test]
    fn recruitment_arguments_parse_with_their_defaults_and_flags() {
        let Commands::Recruitment(args) = Cli::try_parse_from([
            "v3-cli",
            "recruitment",
            "--feature",
            "t13-f07-current-policy-recruitment-transitions",
        ])
        .expect("arguments must parse")
        .command
        else {
            panic!("expected the recruitment subcommand");
        };
        assert!(!args.pilot);
        assert_eq!(args.threads, None);
        assert_eq!(
            args.wall_cap_secs,
            v3_cli::recruitment::DEFAULT_WALL_CAP_SECS
        );
        assert_eq!(args.byte_cap, v3_cli::recruitment::DEFAULT_BYTE_CAP);
        assert!(!args.replay_check);
        let Commands::Recruitment(args) = Cli::try_parse_from([
            "v3-cli",
            "recruitment",
            "--feature",
            "x",
            "--pilot",
            "--threads",
            "4",
            "--wall-cap-secs",
            "10",
            "--byte-cap",
            "1000",
            "--replay-check",
        ])
        .expect("arguments must parse")
        .command
        else {
            panic!("expected the recruitment subcommand");
        };
        assert!(args.pilot && args.replay_check);
        assert_eq!(
            (args.threads, args.wall_cap_secs, args.byte_cap),
            (NonZeroUsize::new(4), 10, 1000)
        );
        assert!(Cli::try_parse_from(["v3-cli", "recruitment"]).is_err());
        assert!(
            Cli::try_parse_from(["v3-cli", "recruitment", "--feature", "x", "--threads", "0"])
                .is_err()
        );
    }

    #[test]
    fn input_opportunity_arguments_parse_with_their_defaults_and_flags() {
        let parse = |args: &[&str]| match Cli::try_parse_from(args)
            .expect("arguments must parse")
            .command
        {
            Commands::InputOpportunity(args) => args,
            _ => panic!("expected the input-opportunity subcommand"),
        };
        let args = parse(&["v3-cli", "input-opportunity", "--feature", "t20-f01"]);
        assert!(!args.pilot);
        assert_eq!(args.threads, None);
        assert_eq!(
            (args.wall_cap_secs, args.byte_cap),
            (
                v3_cli::opportunity::DEFAULT_WALL_CAP_SECS,
                v3_cli::opportunity::DEFAULT_BYTE_CAP
            )
        );
        let args = parse(&[
            "v3-cli",
            "input-opportunity",
            "--feature",
            "x",
            "--pilot",
            "--threads",
            "2",
            "--wall-cap-secs",
            "5",
            "--byte-cap",
            "99",
        ]);
        assert!(args.pilot);
        assert_eq!(
            (args.threads, args.wall_cap_secs, args.byte_cap),
            (NonZeroUsize::new(2), 5, 99)
        );
        assert!(Cli::try_parse_from(["v3-cli", "input-opportunity"]).is_err());
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

        let resolved = resolve_bench_profile(&args).expect("sweep arguments are complete");
        assert_eq!(resolved.params.food_coverage, None);
        assert_eq!(resolved.params.seeds, vec![11, 22, 33]);
        assert_eq!(
            resolved.threads, None,
            "omitting --threads uses the rayon global pool"
        );
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

        let resolved = resolve_bench_profile(&args).expect("sweep arguments are complete");
        assert_eq!(resolved.params.food_coverage, Some(0.5));
        assert_eq!(
            resolved.feature,
            "t01-f11-baseline-persistence-characterization"
        );
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
        let resolved = resolve_bench_profile(&bench_args(BenchProfile::Gate))
            .expect("the gate profile needs only --feature");

        assert_eq!(resolved.params, bench::gate_profile_params());
        assert_eq!(
            resolved.feature,
            "t01-f11-baseline-persistence-characterization"
        );
    }

    #[test]
    fn goal_profile_is_fixed_and_rejects_parameter_overrides() {
        let resolved = resolve_bench_profile(&bench_args(BenchProfile::Goal))
            .expect("the goal profile needs only --feature and optional --threads");
        assert_eq!(resolved.params, bench::goal_profile_params());

        let error = resolve_bench_profile(&BenchArgs {
            ticks: Some(1),
            ..bench_args(BenchProfile::Goal)
        })
        .expect_err("goal parameters must be predeclared rather than silently ignored");
        assert!(error.contains("--ticks is not accepted for --profile goal"));
    }

    #[test]
    fn a_zero_thread_count_is_rejected_for_either_profile() {
        for profile in [BenchProfile::Gate, BenchProfile::Sweep] {
            let args = BenchArgs {
                threads: Some(0),
                ..bench_args(profile)
            };

            let error = resolve_bench_profile(&args).expect_err("a zero-thread pool is not a pool");
            assert!(
                error.contains("--threads must be >= 1"),
                "unexpected error: {error}"
            );
        }
    }

    #[test]
    fn a_positive_thread_count_is_resolved_for_the_gate_profile() {
        let args = BenchArgs {
            threads: Some(1),
            ..bench_args(BenchProfile::Gate)
        };

        let resolved = resolve_bench_profile(&args).expect("--threads 1 is valid");
        assert_eq!(resolved.threads, NonZeroUsize::new(1));
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
    #[test]
    fn recipe_sweep_resolves_defaults_and_explicit_overrides() {
        let path =
            std::env::temp_dir().join(format!("petri-sweep-recipe-{}.json", std::process::id()));
        std::fs::write(&path, r#"{"world":{"width":12,"height":9,"world_seed":18446744073709551615},"population":{"initial_creatures":3,"founder_profile":"forage_first_sparse"},"energy":{"costs":{"move_cost":0.25}}}"#).unwrap();
        let mut args = BenchArgs {
            config: Some(path.clone()),
            seeds: Some("1,2".into()),
            ticks: Some(1),
            ..bench_args(BenchProfile::Sweep)
        };
        let resolved = resolve_bench_profile(&args).unwrap();
        let config = bench::build_config(&resolved.params);
        assert_eq!(
            (
                config.world.width,
                config.world.height,
                config.population.initial_creatures
            ),
            (12, 9, 3)
        );
        assert_eq!(config.world.world_seed, Some(u64::MAX));
        assert_eq!(config.energy.costs.move_cost, 0.25);
        args.width = Some(5);
        args.founders = Some(2);
        args.food_coverage = Some(0.25);
        let overridden = bench::build_config(&resolve_bench_profile(&args).unwrap().params);
        assert_eq!(
            (
                overridden.world.width,
                overridden.world.height,
                overridden.population.initial_creatures
            ),
            (5, 9, 2)
        );
        assert!(overridden
            .world
            .food
            .types
            .iter()
            .all(|food| food.initial_coverage == 0.25));
        for profile in [BenchProfile::Gate, BenchProfile::Goal] {
            args.profile = profile;
            assert!(resolve_bench_profile(&args)
                .unwrap_err()
                .contains("--config"));
        }
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn default_recipe_can_be_saved_without_running_a_world() {
        let config = load_config(None).unwrap();
        let path =
            std::env::temp_dir().join(format!("petri-default-recipe-{}.json", std::process::id()));
        save_config(&config, &path).unwrap();
        let saved = std::fs::read_to_string(&path).unwrap();
        assert!(saved.contains("\n  "));
        let reloaded = load_config(Some(&path)).unwrap();
        assert_eq!(
            serde_json::to_value(config).unwrap(),
            serde_json::to_value(reloaded).unwrap()
        );
        std::fs::remove_file(path).unwrap();
    }
}
