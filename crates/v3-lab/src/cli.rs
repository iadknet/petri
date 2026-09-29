//! Command-line interface: `v3-lab run` and `v3-lab report`.

use std::path::{Path, PathBuf};

use clap::{Args, Parser, Subcommand};

use crate::output::{GitProvenance, LabRoot, DEFAULT_BYTE_CAP};
use crate::run::{run, RunParams, UserArm};
use crate::scene::{ArenaId, Assay};
use crate::summary::{render_report, Summary, SUMMARY_KIND, SUMMARY_VERSION};
use crate::LabError;

/// `(replicates, generations, population)` for a campaign run.
pub const CAMPAIGN_SIZES: (u32, u32, u32) = (8, 100, 64);
/// `(replicates, generations, population)` for `--quick`.
pub const QUICK_SIZES: (u32, u32, u32) = (4, 40, 16);

#[derive(Parser, Debug)]
#[command(name = "v3-lab", about = "Petri capability-assay lab (T22)")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Calibrate and run an assay; output under `.bench-artifacts/lab/`.
    Run(Box<RunArgs>),
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
        let (replicates, generations, population) = if self.quick {
            QUICK_SIZES
        } else {
            CAMPAIGN_SIZES
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
        }
    }
}

/// Read and check a summary file.
///
/// # Errors
///
/// I/O, malformed JSON, or a foreign `kind` / `summary_version`.
pub fn read_summary(path: &std::path::Path) -> Result<Summary, LabError> {
    let summary: Summary = crate::read_json(path)?;
    if summary.kind != SUMMARY_KIND || summary.summary_version != SUMMARY_VERSION {
        return Err(LabError::Config(format!(
            "{}: not a {SUMMARY_KIND} v{SUMMARY_VERSION}",
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

/// Run the parsed command and return the process exit code.
///
/// # Errors
///
/// Any [`LabError`].
pub fn execute(cli: Cli) -> Result<u8, LabError> {
    match cli.command {
        Command::Run(args) => {
            let lab_root = resolve_checkout(&std::env::current_dir()?)?;
            let outcome = run(&args.params(), &lab_root)?;
            print!("{}", render_report(&outcome.summary));
            println!("\nrun directory: {}", outcome.dir.display());
            Ok(outcome.exit_code)
        }
        Command::Report { summary } => {
            print!("{}", render_report(&read_summary(&summary)?));
            Ok(0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let plan = crate::run::resolve_arena(&params).unwrap();
        assert_eq!(plan.size, 64);
        assert_eq!(plan.points.len(), 3);
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
