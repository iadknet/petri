//! One `v3-lab run`: resolve arms, run the calibration gate, run the
//! campaign, write the summary. Exit codes: 0 complete, 2 uncalibrated,
//! 3 stopped by the byte cap (1, any error, is the binary's).

use std::path::PathBuf;
use std::time::Instant;

use serde_json::{json, Value};
use v3_core::config::SimulationConfig;
use v3_core::creature::founder::founder_genome_with_age_gate;
use v3_core::creature::genome::CreatureGenome;
use v3_core::neighborhood::opportunity::controllers::controller;
use v3_core::neighborhood::opportunity::Family;

use crate::arena::{arena_config, classify, resolve_arm, Role, ARENA_ID};
use crate::calibration::{run_gate, GridInput};
use crate::campaign::{fidelity, run_campaign, Arm, ArmKind, Plan};
use crate::eval::{Scripted, Setup};
use crate::output::{
    checkout_root, git_state, resolve_out, utc_stamp, Budget, RunDir, SUMMARY_RESERVE,
};
use crate::rng::{replicate_seed, tagged};
use crate::summary::{
    ArenaRecord, GenomeRecord, OverlayRecord, Provenance, Seeds, Sizes, Summary, Timing, Verdict,
    GENOME_FORMAT, SUMMARY_KIND, SUMMARY_VERSION,
};
use crate::{sha256_hex, GenomeFile, LabError};

/// The only assay in F01.
pub const ASSAY: &str = "food-seeking";
/// Built-in arm names; a user arm may not reuse one.
pub const BUILT_IN_ARMS: [&str; 8] = [
    "native",
    "founder-only",
    "mutation-off",
    "shuffled-score",
    "comparator",
    "random-walk",
    "half-seeker",
    "oracle-seeker",
];

/// Exit code of an `uncalibrated` gate.
pub const EXIT_UNCALIBRATED: u8 = 2;
/// Exit code of a run stopped by the byte cap.
pub const EXIT_BYTE_CAP: u8 = 3;

/// `--arm name=overlay.json[:genome.json]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserArm {
    pub name: String,
    pub overlay: PathBuf,
    pub genome: Option<PathBuf>,
}

/// Everything a run needs, after CLI defaults are applied.
#[derive(Debug, Clone)]
pub struct RunParams {
    pub seed: u64,
    pub replicates: u32,
    pub generations: u32,
    pub population: u32,
    pub elite_fraction: f64,
    pub scenes: u32,
    pub validation_scenes: u32,
    pub lifetime: Option<u32>,
    pub arena_size: u16,
    pub food_fraction: Option<f64>,
    pub start_energy: f32,
    pub calibration_fractions: Vec<f64>,
    pub calibration_lifetimes: Vec<u32>,
    pub calibration_scenes: u32,
    pub calibration_margin: f64,
    pub reach_threshold: Option<f64>,
    pub arms: Vec<UserArm>,
    pub genome: Option<PathBuf>,
    pub comparator: Option<PathBuf>,
    pub threads: usize,
    pub byte_cap: u64,
    pub out: Option<PathBuf>,
    pub calibrate_only: bool,
    pub quick: bool,
}

/// A finished run.
#[derive(Debug, Clone)]
pub struct RunOutcome {
    pub exit_code: u8,
    pub dir: PathBuf,
    pub summary: Summary,
}

fn check(ok: bool, message: &str) -> Result<(), LabError> {
    if ok {
        Ok(())
    } else {
        Err(LabError::Config(message.to_owned()))
    }
}

fn validate_params(params: &RunParams) -> Result<(), LabError> {
    check(params.population >= 2, "--population must be at least 2")?;
    check(
        params.replicates >= 1 && params.generations >= 1,
        "--replicates and --generations must be at least 1",
    )?;
    check(
        params.scenes >= 1 && params.validation_scenes >= 1 && params.calibration_scenes >= 1,
        "scene counts must be at least 1",
    )?;
    check(
        params.elite_fraction > 0.0 && params.elite_fraction <= 1.0,
        "--elite-fraction must be in (0, 1]",
    )?;
    check(
        (48..=64).contains(&params.arena_size),
        "--arena-size must be in 48..=64",
    )?;
    let fractions = params
        .food_fraction
        .map_or(params.calibration_fractions.clone(), |f| vec![f]);
    check(
        !fractions.is_empty() && fractions.iter().all(|f| *f > 0.0 && *f < 1.0),
        "food fractions must be in (0, 1)",
    )?;
    let lifetimes = params
        .lifetime
        .map_or(params.calibration_lifetimes.clone(), |l| vec![l]);
    check(
        !lifetimes.is_empty() && lifetimes.iter().all(|l| *l >= 1),
        "lifetimes must be at least 1",
    )?;
    check(params.threads >= 1, "--threads must be at least 1")?;
    check(
        params.calibration_margin.is_finite(),
        "--calibration-margin must be finite",
    )?;
    for (index, arm) in params.arms.iter().enumerate() {
        check(
            !arm.name.is_empty()
                && arm
                    .name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
            "--arm names use [A-Za-z0-9_-]",
        )?;
        check(
            !BUILT_IN_ARMS.contains(&arm.name.as_str())
                && params.arms[..index].iter().all(|a| a.name != arm.name),
            "--arm names must be unique and not a built-in arm",
        )?;
    }
    Ok(())
}

/// Run one assay end to end.
///
/// # Errors
///
/// Invalid parameters, overlay or genome files, output path, or I/O.
pub fn run(params: &RunParams) -> Result<RunOutcome, LabError> {
    validate_params(params)?;
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(params.threads)
        .build()
        .map_err(|error| LabError::Config(format!("thread pool: {error}")))?;
    pool.install(|| run_in_pool(params))
}

struct Genomes {
    production_founder: CreatureGenome,
    start: CreatureGenome,
    comparator: CreatureGenome,
    records: Vec<GenomeRecord>,
}

fn genome_record(name: &str, genome: &CreatureGenome) -> GenomeRecord {
    GenomeRecord {
        name: name.to_owned(),
        sha256: sha256_hex(&serde_json::to_vec(genome).expect("genome serializes")),
        genome_format: GENOME_FORMAT,
        v3_core_version: env!("CARGO_PKG_VERSION").to_owned(),
    }
}

fn load_genomes(params: &RunParams, reference: &SimulationConfig) -> Result<Genomes, LabError> {
    let production_founder = founder_genome_with_age_gate(
        reference.population.founder_profile,
        &reference.energy.lifecycle,
    );
    let start = match &params.genome {
        Some(path) => GenomeFile::load(path)?,
        None => production_founder.clone(),
    };
    let comparator = match &params.comparator {
        Some(path) => GenomeFile::load(path)?,
        None => controller(&production_founder, Family::Vector, false).0,
    };
    let records = vec![
        genome_record("start", &start),
        genome_record("comparator", &comparator),
    ];
    Ok(Genomes {
        production_founder,
        start,
        comparator,
        records,
    })
}

struct Overlay {
    name: String,
    content: Value,
    genome: Option<CreatureGenome>,
    bytes: u64,
}

fn load_overlays(params: &RunParams) -> Result<Vec<Overlay>, LabError> {
    let mut overlays = vec![Overlay {
        name: "mutation-off".into(),
        content: json!({"mutation": {"per_unit_rate": 0.0}}),
        genome: None,
        bytes: 0,
    }];
    for arm in &params.arms {
        let text = std::fs::read_to_string(&arm.overlay)
            .map_err(|error| LabError::Io(format!("{}: {error}", arm.overlay.display())))?;
        let content: Value = serde_json::from_str(&text)
            .map_err(|error| LabError::Config(format!("{}: {error}", arm.overlay.display())))?;
        overlays.push(Overlay {
            name: arm.name.clone(),
            content,
            genome: arm.genome.as_deref().map(GenomeFile::load).transpose()?,
            bytes: text.len() as u64,
        });
    }
    Ok(overlays)
}

#[allow(clippy::too_many_lines)]
fn run_in_pool(params: &RunParams) -> Result<RunOutcome, LabError> {
    let started = Instant::now();
    let size = params.arena_size;
    let reference = resolve_arm(None, size, params.start_energy)?;
    let genomes = load_genomes(params, &reference)?;
    let overlays = load_overlays(params)?;
    let overlay_configs: Vec<SimulationConfig> = overlays
        .iter()
        .map(|o| resolve_arm(Some(&o.content), size, params.start_energy))
        .collect::<Result<_, _>>()?;

    let root = checkout_root()?;
    let default_name = format!("{ASSAY}-{}-{}", params.seed, utc_stamp());
    let path = resolve_out(&root, params.out.as_deref(), &default_name)?;
    let reserve = SUMMARY_RESERVE + overlays.iter().map(|o| o.bytes).sum::<u64>();
    let mut dir = RunDir::create(path, Budget::new(params.byte_cap, reserve)?)?;

    let calibration_seed = tagged(params.seed, "calibration");
    let validation_seed = tagged(params.seed, "validation");
    let gate = run_gate(
        &reference,
        &genomes.start,
        &genomes.comparator,
        &GridInput {
            fractions: params
                .food_fraction
                .map_or_else(|| params.calibration_fractions.clone(), |f| vec![f]),
            lifetimes: params
                .lifetime
                .map_or_else(|| params.calibration_lifetimes.clone(), |l| vec![l]),
            scenes: params.calibration_scenes,
            validation_scenes: params.validation_scenes,
            margin: params.calibration_margin,
            calibration_seed,
            validation_seed,
            start_energy: params.start_energy,
            reach_threshold: params.reach_threshold,
        },
    );
    let mut creature_ticks = gate.creature_ticks;

    let (arms_summary, fidelity_block, byte_cap_hit, selected) =
        match (&gate.calibration.selected, gate.calibration.reach_threshold) {
            (Some(selected), Some(threshold)) if !params.calibrate_only => {
                let setup = |config: &SimulationConfig| {
                    Setup::new(config.clone(), params.start_energy, selected.lifetime)
                };
                let arms = build_arms(&reference, &genomes, &overlays, &overlay_configs, &setup);
                let plan = Plan {
                    seed: params.seed,
                    replicates: params.replicates,
                    generations: params.generations,
                    population: params.population,
                    elite_fraction: params.elite_fraction,
                    scenes: params.scenes,
                    food_fraction: selected.food_fraction,
                    threshold,
                };
                let (summaries, totals) = run_campaign(&arms, &plan, &gate.validation, &mut dir)?;
                creature_ticks += totals.creature_ticks;
                (
                    summaries,
                    Some(fidelity(&arms[0], &totals)),
                    totals.byte_cap_hit,
                    Some(selected.clone()),
                )
            }
            (selected, _) => (Vec::new(), None, false, selected.clone()),
        };

    let (incomplete, exit_code) = if gate.calibration.verdict == Verdict::Uncalibrated {
        (Some("uncalibrated".to_owned()), EXIT_UNCALIBRATED)
    } else if byte_cap_hit {
        (Some("byte_cap".to_owned()), EXIT_BYTE_CAP)
    } else {
        (None, 0)
    };
    let (git_revision, dirty) = git_state();
    let arena = arena_config(size);
    let wall_seconds = started.elapsed().as_secs_f64();
    let summary = Summary {
        kind: SUMMARY_KIND.to_owned(),
        summary_version: SUMMARY_VERSION,
        assay: ASSAY.to_owned(),
        provenance: Provenance {
            git_revision,
            dirty,
            config_digest: sha256_hex(&serde_json::to_vec(&reference).expect("config serializes")),
            overlays: overlays
                .iter()
                .enumerate()
                .map(|(order, o)| OverlayRecord {
                    name: o.name.clone(),
                    content: o.content.clone(),
                    order,
                })
                .collect(),
            genomes: genomes.records.clone(),
            arena: ArenaRecord {
                spec: json!({"id": ARENA_ID, "size": size, "start": "centre", "food_type": 0}),
                sha256: sha256_hex(&serde_json::to_vec(&arena).expect("config serializes")),
            },
            seeds: Seeds {
                seed: params.seed,
                replicates: (0..params.replicates)
                    .map(|i| replicate_seed(params.seed, i))
                    .collect(),
                calibration: calibration_seed,
                validation: validation_seed,
            },
            threads: params.threads,
            sizes: Sizes {
                replicates: params.replicates,
                generations: params.generations,
                population: params.population,
                elite_fraction: params.elite_fraction,
                scenes: params.scenes,
                validation_scenes: params.validation_scenes,
                arena_size: size,
                start_energy: params.start_energy,
                food_fraction: selected.as_ref().map(|s| s.food_fraction),
                lifetime: selected.as_ref().map(|s| s.lifetime),
                quick: params.quick,
            },
        },
        calibration: gate.calibration,
        arms: arms_summary,
        fidelity: fidelity_block,
        incomplete,
        exit_code,
        timing: Timing {
            wall_seconds,
            creature_ticks,
            per_creature_tick_ms: (creature_ticks > 0)
                .then(|| wall_seconds * 1_000.0 / creature_ticks as f64),
        },
    };
    let bytes = serde_json::to_vec_pretty(&summary).expect("summary serializes");
    dir.write_summary(&bytes)?;
    Ok(RunOutcome {
        exit_code,
        dir: dir.path.clone(),
        summary,
    })
}

fn build_arms(
    reference: &SimulationConfig,
    genomes: &Genomes,
    overlays: &[Overlay],
    overlay_configs: &[SimulationConfig],
    setup: &dyn Fn(&SimulationConfig) -> Setup,
) -> Vec<Arm> {
    let arm = |name: &str, role: Role, config: &SimulationConfig, kind: ArmKind| Arm {
        name: name.to_owned(),
        role,
        policy: classify(config, reference),
        setup: setup(config),
        kind,
    };
    let start = &genomes.start;
    let evolving = |shuffled| ArmKind::Evolving {
        start: start.clone(),
        shuffled,
    };
    let mut arms = vec![
        arm("native", Role::Reference, reference, evolving(false)),
        arm(
            "founder-only",
            Role::Control,
            reference,
            ArmKind::Fixed(start.clone()),
        ),
        arm(
            "mutation-off",
            Role::Control,
            &overlay_configs[0],
            evolving(false),
        ),
        arm("shuffled-score", Role::Control, reference, evolving(true)),
        arm(
            "comparator",
            Role::Instrument,
            reference,
            ArmKind::Fixed(genomes.comparator.clone()),
        ),
        arm(
            Scripted::RandomWalk.name(),
            Role::Instrument,
            reference,
            ArmKind::Scripted {
                policy: Scripted::RandomWalk,
                body: genomes.production_founder.clone(),
            },
        ),
    ];
    for (overlay, config) in overlays.iter().zip(overlay_configs).skip(1) {
        let user_start = overlay.genome.clone().unwrap_or_else(|| start.clone());
        arms.push(arm(
            &overlay.name,
            Role::User,
            config,
            ArmKind::Evolving {
                start: user_start,
                shuffled: false,
            },
        ));
    }
    arms
}
