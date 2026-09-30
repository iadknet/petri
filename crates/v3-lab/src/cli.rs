//! Command-line interface: `v3-lab run`, `v3-lab why-not` and `v3-lab report`.

use std::path::{Path, PathBuf};

use clap::{Args, Parser, Subcommand};

use crate::ladder::{StallRates, RETENTION_DEPTH};
use crate::output::{GitProvenance, LabRoot, DEFAULT_BYTE_CAP, SUMMARY};
use crate::readings::SignatureArms;
use crate::run::{run, RunOutcome, RunParams, UserArm};
use crate::scene::{ArenaId, Assay};
use crate::summary::{
    render_report, Incomplete, Summary, Timing, SUMMARY_KIND, SUMMARY_VERSION, SUMMARY_VERSION_MIN,
};
use crate::LabError;

/// `(replicates, generations, population)` for a campaign run.
pub const CAMPAIGN_SIZES: (u32, u32, u32) = (8, 100, 64);
/// `(replicates, generations, population)` for `--quick`.
pub const QUICK_SIZES: (u32, u32, u32) = (4, 40, 16);
/// `--mutants` for a campaign run.
pub const CAMPAIGN_MUTANTS: u32 = 8;
/// `--mutants` for `--quick`.
pub const QUICK_MUTANTS: u32 = 8;

#[derive(Parser, Debug)]
#[command(name = "v3-lab", about = "Petri capability-assay lab (T22)")]
pub struct Cli {
    /// Run telemetry (`on` or `off`); beats PETRI_TELEMETRY, default off.
    /// `run` and `why-not` export one measurement record pair after the run.
    #[cfg(feature = "telemetry")]
    #[arg(long, global = true, value_name = "on|off")]
    pub telemetry: Option<v3_telemetry::Switch>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Calibrate and run an assay; output under `.bench-artifacts/lab/`.
    Run(Box<RunArgs>),
    /// Calibrate and run an assay as `run` does, then print the why-not
    /// ladder: the first rung that is not `pass` and the track owning it.
    WhyNot(Box<RunArgs>),
    /// Render the assay report from a `summary.json` alone.
    Report {
        /// Path to a run's `summary.json`.
        summary: PathBuf,
    },
}

#[derive(Args, Debug)]
pub struct RunArgs {
    #[arg(long, value_enum, default_value = "food-seeking")]
    pub assay: Assay,
    /// Built-in arena (default: sparse-food-v1 for food-seeking, wall-v1 for
    /// barrier-navigation).
    #[arg(long, value_enum)]
    pub arena: Option<ArenaId>,
    /// JSON layout file (`arena_format: 1`); replaces `--arena`.
    #[arg(long)]
    pub layout: Option<PathBuf>,
    #[arg(long, default_value_t = 1)]
    pub seed: u64,
    /// Quick sizes for a smoke run.
    #[arg(long)]
    pub quick: bool,
    #[arg(long)]
    pub replicates: Option<u32>,
    #[arg(long)]
    pub generations: Option<u32>,
    #[arg(long)]
    pub population: Option<u32>,
    #[arg(long, default_value_t = 0.25)]
    pub elite_fraction: f64,
    #[arg(long, default_value_t = 4)]
    pub scenes: u32,
    #[arg(long, default_value_t = 8)]
    pub validation_scenes: u32,
    /// Skips the lifetime grid (default: selected by calibration).
    #[arg(long)]
    pub lifetime: Option<u32>,
    /// Built-in arena size, 48..=64 (default 64); a layout's is its row count.
    #[arg(long)]
    pub arena_size: Option<u16>,
    /// Restricts the fraction axis (default: selected by calibration).
    #[arg(long)]
    pub food_fraction: Option<f64>,
    /// Restricts the scale axis of wall-v1 and ring-v1, 1..=3 (default:
    /// selected by calibration).
    #[arg(long)]
    pub scale: Option<u8>,
    #[arg(long, default_value_t = 100.0)]
    pub start_energy: f32,
    /// sparse-food-v1 only (default 0.02,0.04,0.08).
    #[arg(long, value_delimiter = ',')]
    pub calibration_fractions: Option<Vec<f64>>,
    /// wall-v1 and ring-v1 only (default 1,2,3).
    #[arg(long, value_delimiter = ',')]
    pub calibration_scales: Option<Vec<u8>>,
    #[arg(long, value_delimiter = ',', default_values_t = [200, 400])]
    pub calibration_lifetimes: Vec<u32>,
    #[arg(long, default_value_t = 16)]
    pub calibration_scenes: u32,
    #[arg(long, default_value_t = 1.0)]
    pub calibration_margin: f64,
    /// Overrides the calibrated threshold (recorded).
    #[arg(long)]
    pub reach_threshold: Option<f64>,
    /// Weight of the blocked-move fraction in the scene score (default 1.0
    /// for barrier-navigation, 0 for food-seeking).
    #[arg(long)]
    pub blocked_weight: Option<f64>,
    /// Weight of the first-bite efficiency in the scene score (default 1.0
    /// for barrier-navigation, 0 for food-seeking).
    #[arg(long)]
    pub efficiency_weight: Option<f64>,
    /// `name=overlay.json[:genome.json]`, repeatable.
    #[arg(long = "arm", value_parser = parse_arm)]
    pub arms: Vec<UserArm>,
    /// Start genome file (default: the production founder).
    #[arg(long)]
    pub genome: Option<PathBuf>,
    /// Comparator genome file (default: the built-in area-food controller).
    #[arg(long)]
    pub comparator: Option<PathBuf>,
    /// Default: available parallelism.
    #[arg(long)]
    pub threads: Option<usize>,
    #[arg(long, default_value_t = DEFAULT_BYTE_CAP)]
    pub byte_cap: u64,
    /// Run directory; must resolve inside the checkout root's `.bench-artifacts/`.
    #[arg(long)]
    pub out: Option<PathBuf>,
    /// Stop after the calibration gate.
    #[arg(long)]
    pub calibrate_only: bool,
    /// Fresh mutants per signature reading, 1..=64 (default 8; `--quick`
    /// [`QUICK_MUTANTS`]).
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..=64))]
    pub mutants: Option<u32>,
    /// Arms whose rows carry the signature block.
    #[arg(long, value_enum, default_value = "changing")]
    pub signature_arms: SignatureArms,
    /// Applied events after a selected improvement the retention rung reads,
    /// 1..=8.
    #[arg(long, default_value_t = RETENTION_DEPTH, value_parser = clap::value_parser!(u32).range(1..=8))]
    pub retention_depth: u32,
    /// `supply,viability,benefit,retention` stall rates, each in (0, 1)
    /// (default 0.01,0.05,0.05,0.20).
    #[arg(long, value_parser = parse_stall_rates)]
    pub stall_rates: Option<StallRates>,
}

fn parse_stall_rates(text: &str) -> Result<StallRates, String> {
    let rates: Vec<f64> = text
        .split(',')
        .map(|rate| rate.trim().parse::<f64>().map_err(|e| e.to_string()))
        .collect::<Result<_, _>>()?;
    StallRates::from_slice(&rates)
        .ok_or_else(|| "expected supply,viability,benefit,retention, each in (0, 1)".to_owned())
}

fn parse_arm(text: &str) -> Result<UserArm, String> {
    let (name, rest) = text
        .split_once('=')
        .ok_or_else(|| "expected name=overlay.json[:genome.json]".to_owned())?;
    let (overlay, genome) = match rest.split_once(':') {
        Some((overlay, genome)) => (overlay, Some(PathBuf::from(genome))),
        None => (rest, None),
    };
    if name.is_empty() || overlay.is_empty() {
        return Err("expected name=overlay.json[:genome.json]".into());
    }
    Ok(UserArm {
        name: name.to_owned(),
        overlay: PathBuf::from(overlay),
        genome,
    })
}

impl RunArgs {
    /// Apply the campaign or `--quick` size defaults.
    #[must_use]
    pub fn params(&self) -> RunParams {
        let ((replicates, generations, population), mutants) = if self.quick {
            (QUICK_SIZES, QUICK_MUTANTS)
        } else {
            (CAMPAIGN_SIZES, CAMPAIGN_MUTANTS)
        };
        RunParams {
            assay: self.assay,
            arena: self.arena,
            layout: self.layout.clone(),
            seed: self.seed,
            replicates: self.replicates.unwrap_or(replicates),
            generations: self.generations.unwrap_or(generations),
            population: self.population.unwrap_or(population),
            elite_fraction: self.elite_fraction,
            scenes: self.scenes,
            validation_scenes: self.validation_scenes,
            lifetime: self.lifetime,
            arena_size: self.arena_size,
            food_fraction: self.food_fraction,
            scale: self.scale,
            start_energy: self.start_energy,
            calibration_fractions: self.calibration_fractions.clone(),
            calibration_scales: self.calibration_scales.clone(),
            calibration_lifetimes: self.calibration_lifetimes.clone(),
            blocked_weight: self.blocked_weight,
            efficiency_weight: self.efficiency_weight,
            calibration_scenes: self.calibration_scenes,
            calibration_margin: self.calibration_margin,
            reach_threshold: self.reach_threshold,
            arms: self.arms.clone(),
            genome: self.genome.clone(),
            comparator: self.comparator.clone(),
            threads: self.threads.unwrap_or_else(|| {
                std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get)
            }),
            byte_cap: self.byte_cap,
            out: self.out.clone(),
            calibrate_only: self.calibrate_only,
            quick: self.quick,
            mutants: self.mutants.unwrap_or(mutants),
            signature_arms: self.signature_arms,
            retention_depth: self.retention_depth,
            stall_rates: self.stall_rates.unwrap_or_default(),
        }
    }
}

/// Read and check a summary file.
///
/// # Errors
///
/// I/O, malformed JSON, or a foreign `kind` / `summary_version` (v2 and
/// v3 are read).
pub fn read_summary(path: &std::path::Path) -> Result<Summary, LabError> {
    let summary: Summary = crate::read_json(path)?;
    if summary.kind != SUMMARY_KIND
        || !(SUMMARY_VERSION_MIN..=SUMMARY_VERSION).contains(&summary.summary_version)
    {
        return Err(LabError::Config(format!(
            "{}: not a {SUMMARY_KIND} v{SUMMARY_VERSION_MIN}..=v{SUMMARY_VERSION}",
            path.display()
        )));
    }
    Ok(summary)
}

fn git(cwd: &Path, args: &[&str]) -> Option<String> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// The checkout containing `cwd`: its root (`git rev-parse
/// --show-toplevel`), revision and dirty state. A CLI run never records
/// unknown provenance.
///
/// # Errors
///
/// [`LabError::Io`] outside a git checkout or when its state is unreadable.
pub fn resolve_checkout(cwd: &Path) -> Result<LabRoot, LabError> {
    let root = git(cwd, &["rev-parse", "--show-toplevel"])
        .ok_or_else(|| LabError::Io("not inside a git checkout".into()))?;
    let revision = git(cwd, &["rev-parse", "HEAD"])
        .ok_or_else(|| LabError::Io("checkout has no HEAD".into()))?;
    let status = git(cwd, &["status", "--porcelain"])
        .ok_or_else(|| LabError::Io("git status failed".into()))?;
    Ok(LabRoot {
        path: PathBuf::from(root),
        git: GitProvenance::Checkout {
            revision,
            dirty: !status.is_empty(),
        },
    })
}

/// What a `run` or `why-not` wrote, as plain data for the binary's run
/// record (T21.F06): the summary's path and its `timing`, `exit_code` and
/// `incomplete`.
#[derive(Debug, Clone, PartialEq)]
pub struct Measurement {
    /// `run` or `why-not`.
    pub command: &'static str,
    pub assay: &'static str,
    pub seed: u64,
    pub summary_path: PathBuf,
    pub timing: Timing,
    pub exit_code: u8,
    pub incomplete: Option<Incomplete>,
}

impl Measurement {
    fn of(command: &'static str, args: &RunArgs, outcome: &RunOutcome) -> Self {
        Self {
            command,
            assay: args.assay.name(),
            seed: args.seed,
            summary_path: outcome.dir.join(SUMMARY),
            timing: outcome.summary.timing.clone(),
            exit_code: outcome.summary.exit_code,
            incomplete: outcome.summary.incomplete,
        }
    }

    /// `exit_code`, `incomplete` and `timing` as compact JSON with sorted
    /// keys, as `summary.json` records them.
    #[must_use]
    pub fn body(&self) -> String {
        serde_json::json!({
            "exit_code": self.exit_code,
            "incomplete": self.incomplete,
            "timing": self.timing,
        })
        .to_string()
    }
}

/// A command's exit code and, for `run` and `why-not`, its measurement.
#[derive(Debug, Clone, PartialEq)]
pub struct Executed {
    pub exit_code: u8,
    pub measurement: Option<Measurement>,
}

/// Run the parsed command in the checkout containing the working directory.
///
/// # Errors
///
/// Any [`LabError`].
pub fn execute(cli: Cli) -> Result<Executed, LabError> {
    match cli.command {
        Command::Report { .. } => execute_command(cli.command, None),
        Command::Run(_) | Command::WhyNot(_) => {
            let lab_root = resolve_checkout(&std::env::current_dir()?)?;
            execute_command(cli.command, Some(&lab_root))
        }
    }
}

/// [`execute`] on an injected lab root, as library callers run it.
///
/// # Errors
///
/// Any [`LabError`].
pub fn execute_with_root(cli: Cli, lab_root: &LabRoot) -> Result<Executed, LabError> {
    execute_command(cli.command, Some(lab_root))
}

fn execute_command(command: Command, lab_root: Option<&LabRoot>) -> Result<Executed, LabError> {
    let (name, args, render): (_, _, fn(&Summary) -> String) = match command {
        Command::Report { summary } => {
            print!("{}", render_report(&read_summary(&summary)?));
            return Ok(Executed {
                exit_code: 0,
                measurement: None,
            });
        }
        Command::Run(args) => ("run", args, render_report),
        Command::WhyNot(args) => ("why-not", args, crate::ladder::render),
    };
    let lab_root = lab_root.expect("run and why-not have a lab root");
    let outcome = run(&args.params(), lab_root)?;
    print!("{}", render(&outcome.summary));
    println!("\nrun directory: {}", outcome.dir.display());
    Ok(Executed {
        exit_code: outcome.exit_code,
        measurement: Some(Measurement::of(name, &args, &outcome)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "telemetry")]
    #[test]
    fn telemetry_flag_beats_environment_which_beats_default_off() {
        use v3_telemetry::{resolve_switch, Switch};
        let flag = |argv: &[&str]| {
            Cli::try_parse_from(argv)
                .expect("arguments must parse")
                .telemetry
        };
        let plain = flag(&["v3-lab", "report", "x.json"]);
        assert_eq!(resolve_switch(plain, None), Ok(Switch::Off));
        assert_eq!(resolve_switch(plain, Some("on")), Ok(Switch::On));
        let off = flag(&["v3-lab", "--telemetry", "off", "report", "x.json"]);
        assert_eq!(resolve_switch(off, Some("on")), Ok(Switch::Off));
        let on = flag(&["v3-lab", "--telemetry", "on", "report", "x.json"]);
        assert_eq!(resolve_switch(on, None), Ok(Switch::On));
    }

    #[test]
    fn arm_specs_parse_with_and_without_a_genome() {
        let arm = parse_arm("hot=o.json:g.json").unwrap();
        assert_eq!(arm.name, "hot");
        assert_eq!(arm.overlay, PathBuf::from("o.json"));
        assert_eq!(arm.genome, Some(PathBuf::from("g.json")));
        assert_eq!(parse_arm("x=o.json").unwrap().genome, None);
        assert!(parse_arm("o.json").is_err());
        assert!(parse_arm("=o.json").is_err(), "empty name");
        assert!(parse_arm("x=").is_err(), "empty overlay");
    }

    /// A per-test directory under the system temp directory; removed on drop.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("petri-lab-cli-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// Run `git -C dir …` with a child-only identity; the test process's
    /// environment and working directory are untouched.
    fn git_in(dir: &Path, args: &[&str]) -> String {
        let output = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args([
                "-c",
                "user.name=lab",
                "-c",
                "user.email=lab@example.invalid",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .output()
            .expect("git runs");
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    }

    #[test]
    fn a_checkout_resolves_to_its_root_revision_and_dirty_state() {
        let repo = TempDir::new("checkout");
        git_in(&repo.0, &["init", "-q"]);
        git_in(&repo.0, &["commit", "-q", "--allow-empty", "-m", "init"]);
        let revision = git_in(&repo.0, &["rev-parse", "HEAD"]);
        let nested = repo.0.join("nested");
        std::fs::create_dir_all(&nested).unwrap();

        let clean = resolve_checkout(&nested).unwrap();
        assert_eq!(clean.path, repo.0.canonicalize().unwrap());
        assert_eq!(
            clean.git,
            GitProvenance::Checkout {
                revision: revision.clone(),
                dirty: false
            }
        );

        std::fs::write(repo.0.join("untracked.txt"), "x").unwrap();
        let dirty = resolve_checkout(&repo.0).unwrap();
        assert_eq!(
            dirty.git,
            GitProvenance::Checkout {
                revision,
                dirty: true
            }
        );
    }

    #[test]
    fn quick_and_campaign_defaults_differ_and_explicit_sizes_win() {
        let cli = Cli::parse_from(["v3-lab", "run", "--quick", "--population", "5"]);
        let Command::Run(args) = cli.command else {
            panic!("run");
        };
        let params = args.params();
        assert_eq!(params.population, 5);
        assert_eq!(params.replicates, QUICK_SIZES.0);
        assert_eq!(
            params.calibration_fractions, None,
            "the arena supplies the default"
        );
        assert_eq!(params.assay, Assay::FoodSeeking);
        assert_eq!(params.mutants, QUICK_MUTANTS);
        assert_eq!(params.signature_arms, SignatureArms::Changing);
        assert_eq!(params.retention_depth, RETENTION_DEPTH);
        assert_eq!(params.stall_rates, StallRates::default());
        let plan = crate::run::resolve_arena(&params).unwrap();
        assert_eq!(plan.size, 64);
        assert_eq!(plan.points.len(), 3);
    }

    #[test]
    fn why_not_shares_the_run_flags_and_parses_the_ladder_flags() {
        let cli = Cli::parse_from([
            "v3-lab",
            "why-not",
            "--quick",
            "--retention-depth",
            "3",
            "--stall-rates",
            "0.5,0.1,0.2,0.3",
        ]);
        let Command::WhyNot(args) = cli.command else {
            panic!("why-not");
        };
        let params = args.params();
        assert_eq!(params.retention_depth, 3);
        assert_eq!(
            params.stall_rates,
            StallRates {
                supply: 0.5,
                viability: 0.1,
                benefit: 0.2,
                retention: 0.3
            }
        );
        assert_eq!(params.replicates, QUICK_SIZES.0);
        for bad in [
            ["--retention-depth", "0"],
            ["--retention-depth", "9"],
            ["--stall-rates", "0.1,0.1,0.1"],
            ["--stall-rates", "0.1,0.1,0.1,1.0"],
            ["--stall-rates", "0,0.1,0.1,0.1"],
            ["--stall-rates", "a,0.1,0.1,0.1"],
        ] {
            let parsed = Cli::try_parse_from(["v3-lab", "run", bad[0], bad[1]]);
            assert!(parsed.is_err(), "{bad:?}");
        }
    }

    #[test]
    fn barrier_flags_parse_into_the_barrier_arena() {
        let cli = Cli::parse_from([
            "v3-lab",
            "run",
            "--assay",
            "barrier-navigation",
            "--arena",
            "ring-v1",
            "--calibration-scales",
            "1,3",
            "--blocked-weight",
            "0.5",
            "--efficiency-weight",
            "0.25",
        ]);
        let Command::Run(args) = cli.command else {
            panic!("run");
        };
        let params = args.params();
        assert_eq!(params.arena, Some(ArenaId::RingV1));
        assert_eq!(params.calibration_scales, Some(vec![1, 3]));
        assert_eq!(params.blocked_weight, Some(0.5));
        assert_eq!(params.efficiency_weight, Some(0.25));
        let plan = crate::run::resolve_arena(&params).unwrap();
        assert_eq!(
            plan.points,
            [
                crate::scene::Geometry::Ring { scale: 1 },
                crate::scene::Geometry::Ring { scale: 3 }
            ]
        );
    }
}
