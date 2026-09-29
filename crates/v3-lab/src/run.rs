//! One `v3-lab run`: resolve arms, run the calibration gate, run the
//! campaign, write the summary. Exit codes: 0 complete, 2 uncalibrated,
//! 3 stopped by the byte cap (1, any error, is the binary's).

use std::path::PathBuf;
use std::time::Instant;

use serde_json::{json, Value};
use v3_core::config::SimulationConfig;
use v3_core::creature::founder::founder_genome_with_age_gate;
use v3_core::creature::genome::CreatureGenome;

use crate::arena::{classify, resolve_arm, Role};
use crate::calibration::{run_gate, GridInput};
use crate::campaign::{fidelity, run_campaign, Arm, ArmKind, Plan};
use crate::eval::{Scoring, Scripted, Setup};
use crate::layout::Layout;
use crate::output::{resolve_out, utc_stamp, Budget, LabRoot, RunDir, SUMMARY_RESERVE};
use crate::rng::{replicate_seed, tagged};
use crate::scene::{ArenaId, Assay, Geometry, SCALES};
use crate::summary::{
    ArenaRecord, GenomeRecord, Incomplete, OverlayRecord, Provenance, Seeds, Sizes, Summary,
    Timing, Verdict, GENOME_FORMAT, SUMMARY_KIND, SUMMARY_VERSION,
};
use crate::{sha256_hex, GenomeFile, LabError};

/// Built-in arena size when `--arena-size` is not given.
pub const DEFAULT_ARENA_SIZE: u16 = 64;
/// Built-in arena sizes.
pub const ARENA_SIZES: std::ops::RangeInclusive<u16> = 48..=64;
/// `--calibration-fractions` default.
pub const CALIBRATION_FRACTIONS: [f64; 3] = [0.02, 0.04, 0.08];
/// `--calibration-scales` default.
pub const CALIBRATION_SCALES: [u8; 3] = [1, 2, 3];
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
    pub assay: Assay,
    /// Default: the assay's built-in arena.
    pub arena: Option<ArenaId>,
    /// A JSON layout file; replaces `arena`.
    pub layout: Option<PathBuf>,
    pub seed: u64,
    pub replicates: u32,
    pub generations: u32,
    pub population: u32,
    pub elite_fraction: f64,
    pub scenes: u32,
    pub validation_scenes: u32,
    pub lifetime: Option<u32>,
    /// Built-in arenas only; default [`DEFAULT_ARENA_SIZE`].
    pub arena_size: Option<u16>,
    pub food_fraction: Option<f64>,
    pub scale: Option<u8>,
    pub start_energy: f32,
    /// Default [`CALIBRATION_FRACTIONS`].
    pub calibration_fractions: Option<Vec<f64>>,
    /// Default [`CALIBRATION_SCALES`].
    pub calibration_scales: Option<Vec<u8>>,
    pub calibration_lifetimes: Vec<u32>,
    /// Default: the assay's.
    pub blocked_weight: Option<f64>,
    /// Default: the assay's.
    pub efficiency_weight: Option<f64>,
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

impl RunParams {
    /// The lifetime axis: an explicit `--lifetime` replaces it.
    fn lifetimes(&self) -> Vec<u32> {
        self.lifetime
            .map_or_else(|| self.calibration_lifetimes.clone(), |l| vec![l])
    }

    /// The assay's scoring with the explicit weights applied.
    fn scoring(&self) -> Scoring {
        let default = self.assay.scoring();
        Scoring {
            blocked_weight: self.blocked_weight.unwrap_or(default.blocked_weight),
            efficiency_weight: self.efficiency_weight.unwrap_or(default.efficiency_weight),
            exhausted: default.exhausted,
        }
    }
}

/// The resolved arena: its size and its axis points.
#[derive(Debug, Clone)]
pub struct ArenaPlan {
    pub size: u16,
    pub points: Vec<Geometry>,
}

impl ArenaPlan {
    /// The canonical descriptor, with the axis value of `selected` (or of
    /// the only point).
    fn descriptor(&self, selected: Option<&Geometry>) -> Value {
        let only = (self.points.len() == 1).then(|| &self.points[0]);
        match selected.or(only) {
            Some(point) => point.descriptor(self.size, true),
            None => self.points[0].descriptor(self.size, false),
        }
    }
}

/// Resolve the arena: `--layout` or `--arena` (the assay's default), its
/// size and its axis. An axis flag the arena does not use is an error; an
/// explicit `--food-fraction` or `--scale` replaces its axis with that
/// value.
///
/// # Errors
///
/// [`LabError::Config`] for a conflicting or out-of-range flag or an
/// invalid layout file; I/O failures reading it.
pub fn resolve_arena(params: &RunParams) -> Result<ArenaPlan, LabError> {
    let fraction_flags = params.food_fraction.is_some() || params.calibration_fractions.is_some();
    let scale_flags = params.scale.is_some() || params.calibration_scales.is_some();
    if let Some(path) = &params.layout {
        check(params.arena.is_none(), "--layout replaces --arena")?;
        check(
            params.arena_size.is_none(),
            "a layout's size is its row count: --arena-size is refused with --layout",
        )?;
        check(
            !fraction_flags && !scale_flags,
            "a layout has no axis: fraction and scale flags are refused",
        )?;
        let layout = Layout::load(path)?;
        return Ok(ArenaPlan {
            size: layout.size(),
            points: vec![Geometry::Layout(layout)],
        });
    }
    let size = params.arena_size.unwrap_or(DEFAULT_ARENA_SIZE);
    check(
        ARENA_SIZES.contains(&size),
        "--arena-size must be in 48..=64",
    )?;
    let arena = params.arena.unwrap_or_else(|| params.assay.default_arena());
    let points = match arena {
        ArenaId::SparseFoodV1 => {
            check(!scale_flags, "sparse-food-v1 has no scale axis")?;
            let fractions = params.food_fraction.map_or_else(
                || {
                    params
                        .calibration_fractions
                        .clone()
                        .unwrap_or_else(|| CALIBRATION_FRACTIONS.to_vec())
                },
                |f| vec![f],
            );
            check(
                !fractions.is_empty() && fractions.iter().all(|f| *f > 0.0 && *f < 1.0),
                "food fractions must be in (0, 1)",
            )?;
            fractions
                .into_iter()
                .map(|fraction| Geometry::SparseFood { fraction })
                .collect()
        }
        ArenaId::WallV1 | ArenaId::RingV1 => {
            check(
                !fraction_flags,
                "wall-v1 and ring-v1 have no food-fraction axis",
            )?;
            let scales = params.scale.map_or_else(
                || {
                    params
                        .calibration_scales
                        .clone()
                        .unwrap_or_else(|| CALIBRATION_SCALES.to_vec())
                },
                |k| vec![k],
            );
            check(
                !scales.is_empty() && scales.iter().all(|k| SCALES.contains(k)),
                "scales must be in 1..=3",
            )?;
            scales
                .into_iter()
                .map(|scale| {
                    if arena == ArenaId::WallV1 {
                        Geometry::Wall { scale }
                    } else {
                        Geometry::Ring { scale }
                    }
                })
                .collect()
        }
    };
    Ok(ArenaPlan { size, points })
}

fn check(ok: bool, message: &str) -> Result<(), LabError> {
    if ok {
        Ok(())
    } else {
        Err(LabError::Config(message.to_owned()))
    }
}

fn validate_params(params: &RunParams) -> Result<ArenaPlan, LabError> {
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
    let lifetimes = params.lifetimes();
    check(
        !lifetimes.is_empty() && lifetimes.iter().all(|l| *l >= 1),
        "lifetimes must be at least 1",
    )?;
    let weight = |weight: f64| weight.is_finite() && weight >= 0.0;
    let scoring = params.scoring();
    check(
        weight(scoring.blocked_weight),
        "--blocked-weight must be finite and at least 0",
    )?;
    check(
        weight(scoring.efficiency_weight),
        "--efficiency-weight must be finite and at least 0",
    )?;
    check(params.threads >= 1, "--threads must be at least 1")?;
    check(
        params.calibration_margin.is_finite(),
        "--calibration-margin must be finite",
    )?;
    check(
        params.reach_threshold.is_none_or(f64::is_finite),
        "--reach-threshold must be finite",
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
    resolve_arena(params)
}

/// Run one assay end to end, writing under `lab_root.path`'s
/// `.bench-artifacts/` and recording `lab_root.git` as provenance.
///
/// # Errors
///
/// Invalid parameters, overlay or genome files, output path, or I/O.
pub fn run(params: &RunParams, lab_root: &LabRoot) -> Result<RunOutcome, LabError> {
    let arena = validate_params(params)?;
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(params.threads)
        .build()
        .map_err(|error| LabError::Config(format!("thread pool: {error}")))?;
    pool.install(|| run_in_pool(params, &arena, lab_root))
}

struct Genomes {
    production_founder: CreatureGenome,
    start: CreatureGenome,
    comparator: CreatureGenome,
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
        None => crate::comparator::built_in(params.assay, &production_founder),
    };
    Ok(Genomes {
        production_founder,
        start,
        comparator,
    })
}

/// Provenance for every genome a run evaluates: `start`, `comparator`, and
/// `arm:<name>` for each genome supplied through `--arm` (`:` cannot occur
/// in an arm name, so the names never collide).
fn genome_records(genomes: &Genomes, overlays: &[Overlay]) -> Vec<GenomeRecord> {
    let arms = overlays.iter().filter_map(|overlay| {
        let genome = overlay.genome.as_ref()?;
        Some(genome_record(&format!("arm:{}", overlay.name), genome))
    });
    [
        genome_record("start", &genomes.start),
        genome_record("comparator", &genomes.comparator),
    ]
    .into_iter()
    .chain(arms)
    .collect()
}

struct Overlay {
    name: String,
    content: Value,
    /// The arm config resolved from `content`.
    config: SimulationConfig,
    genome: Option<CreatureGenome>,
    bytes: u64,
}

fn load_overlays(params: &RunParams, size: u16) -> Result<Vec<Overlay>, LabError> {
    let resolve = |content: &Value| resolve_arm(Some(content), size, params.start_energy);
    let content = json!({"mutation": {"per_unit_rate": 0.0}});
    let mut overlays = vec![Overlay {
        name: "mutation-off".into(),
        config: resolve(&content)?,
        content,
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
            config: resolve(&content)?,
            content,
            genome: arm.genome.as_deref().map(GenomeFile::load).transpose()?,
            bytes: text.len() as u64,
        });
    }
    Ok(overlays)
}

#[allow(clippy::too_many_lines)]
fn run_in_pool(
    params: &RunParams,
    arena: &ArenaPlan,
    lab_root: &LabRoot,
) -> Result<RunOutcome, LabError> {
    let started = Instant::now();
    let size = arena.size;
    let scoring = params.scoring();
    let reference = resolve_arm(None, size, params.start_energy)?;
    let genomes = load_genomes(params, &reference)?;
    let overlays = load_overlays(params, size)?;

    let default_name = format!("{}-{}-{}", params.assay.name(), params.seed, utc_stamp());
    let path = resolve_out(&lab_root.path, params.out.as_deref(), &default_name)?;
    let reserve = SUMMARY_RESERVE + overlays.iter().map(|o| o.bytes).sum::<u64>();
    let mut dir = RunDir::create(path, Budget::new(params.byte_cap, reserve)?)?;

    let calibration_seed = tagged(params.seed, "calibration");
    let validation_seed = tagged(params.seed, "validation");
    let gate = run_gate(
        &reference,
        &genomes.start,
        &genomes.comparator,
        &GridInput {
            points: arena.points.clone(),
            lifetimes: params.lifetimes(),
            assay: params.assay,
            scoring,
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

    let (arms_summary, fidelity_block, byte_cap_hit, selected) = match (
        &gate.calibration.selected,
        &gate.spec,
        gate.calibration.reach_threshold,
    ) {
        (Some(selected), Some(spec), Some(threshold)) if !params.calibrate_only => {
            let setup = |config: &SimulationConfig| {
                Setup::new(config.clone(), params.start_energy, selected.lifetime)
                    .with_scoring(scoring)
            };
            let arms = build_arms(&reference, &genomes, &overlays, &setup);
            let plan = Plan {
                seed: params.seed,
                replicates: params.replicates,
                generations: params.generations,
                population: params.population,
                elite_fraction: params.elite_fraction,
                scenes: params.scenes,
                spec: spec.clone(),
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
        (selected, _, _) => (Vec::new(), None, false, selected.clone()),
    };

    let (incomplete, exit_code) = if gate.calibration.verdict == Verdict::Uncalibrated {
        (Some(Incomplete::Uncalibrated), EXIT_UNCALIBRATED)
    } else if byte_cap_hit {
        (Some(Incomplete::ByteCap), EXIT_BYTE_CAP)
    } else {
        (None, 0)
    };
    let (git_revision, dirty, git) = lab_root.git.fields();
    let arena_spec = arena.descriptor(gate.spec.as_ref().map(|spec| &spec.geometry));
    let wall_seconds = started.elapsed().as_secs_f64();
    let summary = Summary {
        kind: SUMMARY_KIND.to_owned(),
        summary_version: SUMMARY_VERSION,
        assay: params.assay,
        provenance: Provenance {
            git_revision,
            dirty,
            git,
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
            genomes: genome_records(&genomes, &overlays),
            arena: ArenaRecord {
                // `serde_json` maps keep keys sorted: the bytes are canonical.
                sha256: sha256_hex(
                    &serde_json::to_vec(&arena_spec).expect("descriptor serializes"),
                ),
                spec: arena_spec,
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
                food_fraction: selected.as_ref().and_then(|s| s.food_fraction),
                scale: selected.as_ref().and_then(|s| s.scale),
                lifetime: selected.as_ref().map(|s| s.lifetime),
                blocked_weight: scoring.blocked_weight,
                efficiency_weight: scoring.efficiency_weight,
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
            &overlays[0].config,
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
    for overlay in &overlays[1..] {
        let user_start = overlay.genome.clone().unwrap_or_else(|| start.clone());
        arms.push(arm(
            &overlay.name,
            Role::User,
            &overlay.config,
            ArmKind::Evolving {
                start: user_start,
                shuffled: false,
            },
        ));
    }
    arms
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::Exhausted;

    fn params() -> RunParams {
        RunParams {
            assay: Assay::FoodSeeking,
            arena: None,
            layout: None,
            seed: 1,
            replicates: 1,
            generations: 1,
            population: 2,
            elite_fraction: 0.5,
            scenes: 1,
            validation_scenes: 1,
            lifetime: None,
            arena_size: Some(48),
            food_fraction: None,
            scale: None,
            start_energy: 100.0,
            calibration_fractions: Some(vec![0.04]),
            calibration_scales: None,
            calibration_lifetimes: vec![100],
            blocked_weight: None,
            efficiency_weight: None,
            calibration_scenes: 1,
            calibration_margin: 1.0,
            reach_threshold: None,
            arms: Vec::new(),
            genome: None,
            comparator: None,
            threads: 1,
            byte_cap: 64 << 20,
            out: None,
            calibrate_only: false,
            quick: false,
        }
    }

    fn arm(name: &str) -> UserArm {
        UserArm {
            name: name.into(),
            overlay: PathBuf::from("o.json"),
            genome: None,
        }
    }

    fn refused(change: impl Fn(&mut RunParams)) -> bool {
        let mut p = params();
        change(&mut p);
        validate_params(&p).is_err()
    }

    #[test]
    fn valid_params_and_arm_names_pass() {
        assert!(validate_params(&params()).is_ok());
        let mut p = params();
        p.elite_fraction = 1.0;
        p.arms = vec![arm("a-b_C9"), arm("hot")];
        assert!(validate_params(&p).is_ok());
    }

    #[test]
    fn each_count_of_zero_is_refused_alone() {
        assert!(refused(|p| p.replicates = 0));
        assert!(refused(|p| p.generations = 0));
        assert!(refused(|p| p.scenes = 0));
        assert!(refused(|p| p.validation_scenes = 0));
        assert!(refused(|p| p.calibration_scenes = 0));
    }

    #[test]
    fn elite_fraction_and_grid_bounds_are_open_where_documented() {
        assert!(refused(|p| p.elite_fraction = 0.0));
        assert!(refused(|p| p.elite_fraction = 1.5));
        assert!(refused(|p| p.calibration_fractions = Some(Vec::new())));
        for fraction in [0.0, 1.0, 1.5, -0.5] {
            assert!(
                refused(|p| p.calibration_fractions = Some(vec![fraction])),
                "{fraction}"
            );
        }
        assert!(refused(|p| p.calibration_lifetimes = Vec::new()));
    }

    /// `params` on a built-in barrier arena with no axis flag.
    fn barrier(arena: ArenaId) -> impl Fn(&mut RunParams) {
        move |p| {
            p.assay = Assay::BarrierNavigation;
            p.arena = Some(arena);
            p.calibration_fractions = None;
        }
    }

    #[test]
    fn an_axis_flag_the_arena_does_not_use_is_refused() {
        let wall = barrier(ArenaId::WallV1);
        let with = |base: &dyn Fn(&mut RunParams), change: &dyn Fn(&mut RunParams)| {
            refused(|p| {
                base(p);
                change(p);
            })
        };
        assert!(!with(&wall, &|_| {}));
        assert!(with(&wall, &|p| p.food_fraction = Some(0.04)));
        assert!(with(&wall, &|p| p.calibration_fractions = Some(vec![0.04])));
        assert!(
            refused(|p| p.scale = Some(1)),
            "sparse-food-v1 has no scale"
        );
        assert!(refused(|p| p.calibration_scales = Some(vec![1])));
        for scale in [0, 4] {
            assert!(with(&wall, &|p| p.scale = Some(scale)), "{scale}");
            assert!(with(&wall, &|p| p.calibration_scales = Some(vec![scale])));
        }
        assert!(with(&wall, &|p| p.calibration_scales = Some(Vec::new())));
        assert!(refused(|p| p.arena_size = Some(47)));
        assert!(refused(|p| p.arena_size = Some(65)));
        // The ring shares the wall's axis.
        let ring = barrier(ArenaId::RingV1);
        assert!(!with(&ring, &|p| p.scale = Some(3)));
        assert!(with(&ring, &|p| p.food_fraction = Some(0.04)));
    }

    #[test]
    fn a_layout_refuses_arena_size_and_axis_flags() {
        let path =
            std::env::temp_dir().join(format!("petri-lab-layout-{}.json", std::process::id()));
        let rows = crate::layout::tests::rows(16, &[(1, 1, 'S'), (5, 5, 'F')]);
        std::fs::write(
            &path,
            serde_json::to_vec(&json!({"arena_format": 1, "rows": rows})).unwrap(),
        )
        .unwrap();
        let layout = |p: &mut RunParams| {
            p.layout = Some(path.clone());
            p.arena_size = None;
            p.calibration_fractions = None;
        };
        let with = |change: &dyn Fn(&mut RunParams)| {
            refused(|p| {
                layout(p);
                change(p);
            })
        };
        assert!(!with(&|_| {}));
        let plan = {
            let mut p = params();
            layout(&mut p);
            resolve_arena(&p).unwrap()
        };
        assert_eq!(plan.size, 16);
        assert!(with(&|p| p.arena_size = Some(48)));
        assert!(with(&|p| p.arena = Some(ArenaId::WallV1)));
        assert!(with(&|p| p.scale = Some(1)));
        assert!(with(&|p| p.food_fraction = Some(0.04)));
        assert!(with(&|p| p.calibration_fractions = Some(vec![0.04])));
        assert!(with(&|p| p.calibration_scales = Some(vec![1])));
        std::fs::remove_file(&path).unwrap();
        assert!(with(&|_| {}), "a missing layout file");
    }

    #[test]
    fn the_weights_are_finite_and_non_negative_with_assay_defaults() {
        assert_eq!(params().scoring(), Scoring::BITES_AND_PROGRESS);
        let mut p = params();
        barrier(ArenaId::WallV1)(&mut p);
        assert_eq!(p.scoring(), Assay::BarrierNavigation.scoring());
        assert_eq!(
            (p.scoring().blocked_weight, p.scoring().efficiency_weight),
            (1.0, 1.0)
        );
        p.blocked_weight = Some(0.25);
        p.efficiency_weight = Some(0.5);
        assert_eq!(
            p.scoring(),
            Scoring {
                blocked_weight: 0.25,
                efficiency_weight: 0.5,
                exhausted: Exhausted::Complete,
            }
        );
        for weight in [-0.5, f64::NAN, f64::INFINITY] {
            assert!(refused(|p| p.blocked_weight = Some(weight)), "{weight}");
            assert!(refused(|p| p.efficiency_weight = Some(weight)), "{weight}");
        }
        assert!(!refused(|p| p.blocked_weight = Some(0.0)));
        assert!(!refused(|p| p.efficiency_weight = Some(0.0)));
    }

    #[test]
    fn arm_names_must_be_well_formed_unique_and_not_built_in() {
        assert!(refused(|p| p.arms = vec![arm("")]));
        assert!(refused(|p| p.arms = vec![arm("bad name")]));
        assert!(refused(|p| p.arms = vec![arm("native")]));
        assert!(refused(|p| p.arms = vec![arm("hot"), arm("hot")]));
    }
}
