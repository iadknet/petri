//! The T20.F01 ecological opportunity assay: `v3-cli input-opportunity`.
//!
//! Per goal world: one incumbent-source run of the goal config at its goal
//! seed, then replicate competitions of the eight arms at native costs with
//! mutation off, each replicate's compact record streamed to the raw file as
//! it completes. The wall-clock and byte caps stop the run between
//! replicates and mark the record `incomplete`; a completed replicate is
//! never dropped. The summary carries every replicate row, the per-world and
//! per-family verdicts, the F02–F05 gate, the controllers' competence and
//! Graph feasibility, the discovery baseline and the VM concerns.

use std::io::Write;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use v3_core::config::SimulationConfig;
use v3_core::creature::founder::founder_genome_with_age_gate;
use v3_core::creature::genome::CreatureGenome;
use v3_core::neighborhood::opportunity::assay::{draw_incumbents, run_replicate};
use v3_core::neighborhood::opportunity::controllers::{self, controller, AuthoredStructure};
use v3_core::neighborhood::opportunity::discovery::{
    self, discovery, feasibility, vm_concern, Discovery, Feasibility, VmConcern,
};
use v3_core::neighborhood::opportunity::fixtures::{competence, Competence};
use v3_core::neighborhood::opportunity::verdict::{family_verdict, gate_favorable, ratio};
use v3_core::neighborhood::opportunity::{
    self as assay, applicable, replicate_config, run_seed, world_verdicts, Arm, ArmGenomes, Family,
    ReplicateRun, Verdict, WorldVerdict,
};
use v3_core::neighborhood::Battery;
use v3_core::simulation::{run_tick, seed_simulation};

use crate::bench::artifacts::{output_paths_with_suffix, write_json, OutputPaths};
use crate::bench::GoalCase;
use crate::recruitment::{io_error, thread_pool, HashingWriter};

/// The record kind the summary carries.
pub const SUMMARY_KIND: &str = "petri-input-opportunity-summary";
/// Default wall-clock cap, seconds.
pub const DEFAULT_WALL_CAP_SECS: u64 = 7_200;
/// The committed summary's budget (spec: summary cap 300 KB); a larger
/// summary is still written, marked incomplete with `summary_cap`.
pub const SUMMARY_CAP_BYTES: u64 = 300 * 1024;
/// Default on-disk byte cap of the raw record: 64 MiB.
pub const DEFAULT_BYTE_CAP: u64 = 64 * 1024 * 1024;

/// Sizes other than the assay's fixed ones (tests only).
#[derive(Debug, Clone, Copy)]
pub struct Sizes {
    pub replicates: u32,
    pub horizon: u64,
    /// World dimensions and founders replacing each recipe's.
    pub world: Option<(u16, u16, u32)>,
    pub discovery_proposals: u32,
}

impl Sizes {
    pub const FULL: Self = Self {
        replicates: assay::REPLICATES,
        horizon: assay::HORIZON,
        world: None,
        discovery_proposals: discovery::DISCOVERY_PROPOSALS,
    };
    /// One replicate per world; everything else as the full run.
    pub const PILOT: Self = Self {
        replicates: 1,
        ..Self::FULL
    };
}

#[derive(Debug, Clone)]
pub struct Options {
    pub feature: String,
    pub pilot: bool,
    pub threads: Option<NonZeroUsize>,
    pub wall_cap: Duration,
    pub byte_cap: u64,
    pub sizes: Option<Sizes>,
    pub raw: Option<PathBuf>,
    pub summary: Option<PathBuf>,
    pub cwd: PathBuf,
    pub source_revision: String,
}

impl Options {
    #[must_use]
    pub fn new(feature: &str, cwd: PathBuf) -> Self {
        Self {
            feature: feature.into(),
            pilot: false,
            threads: None,
            wall_cap: Duration::from_secs(DEFAULT_WALL_CAP_SECS),
            byte_cap: DEFAULT_BYTE_CAP,
            sizes: None,
            raw: None,
            summary: None,
            cwd,
            source_revision: "unknown".into(),
        }
    }

    fn sizes(&self) -> Sizes {
        self.sizes.unwrap_or(if self.pilot {
            Sizes::PILOT
        } else {
            Sizes::FULL
        })
    }
}

/// A ratio as text: six decimals, `inf`, or `null` when both counts are zero.
fn ratio_text(value: Option<f64>) -> Option<String> {
    value.map(|value| {
        if value.is_infinite() {
            "inf".to_string()
        } else {
            crate::six(value)
        }
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArmRow {
    pub arm: String,
    pub founders: u32,
    pub births: u64,
    pub living: Vec<u32>,
    pub extinction_tick: Option<u64>,
    pub sampled: u64,
    pub exposed: u64,
    pub applied: u64,
}

/// One replicate's paired ratios; the `I` ratios are `null` under the
/// founder fallback.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PairedRatios {
    pub family: String,
    pub a_over_z: Option<String>,
    pub a_over_f: Option<String>,
    pub a_over_i: Option<String>,
    pub z_over_f: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplicateRow {
    pub replicate: u32,
    pub run_seed: u64,
    pub ticks: u64,
    /// Every birth of the replicate: each arm's births plus
    /// `unattributed_births`, newborns dying within their birth tick.
    pub births_total: u64,
    pub unattributed_births: u64,
    pub arms: Vec<ArmRow>,
    pub i_over_f: Option<String>,
    pub ratios: Vec<PairedRatios>,
}

fn replicate_row(replicate: u32, run: &ReplicateRun, fallback: bool) -> ReplicateRow {
    let births = |arm: Arm| run.arms[arm.index()].births;
    let incumbent = |value: Option<f64>| if fallback { None } else { ratio_text(value) };
    ReplicateRow {
        replicate,
        run_seed: run.run_seed,
        ticks: run.ticks,
        births_total: run.births_total,
        unattributed_births: run.unattributed_births,
        arms: Arm::ALL
            .into_iter()
            .map(|arm| {
                let reading = &run.arms[arm.index()];
                ArmRow {
                    arm: if arm == Arm::Incumbent && fallback {
                        "I (founder fallback)".to_string()
                    } else {
                        arm.label()
                    },
                    founders: reading.founders,
                    births: reading.births,
                    living: reading.living.clone(),
                    extinction_tick: reading.extinction_tick,
                    sampled: reading.exposure.sampled,
                    exposed: reading.exposure.exposed,
                    applied: reading.exposure.applied,
                }
            })
            .collect(),
        i_over_f: incumbent(ratio(births(Arm::Incumbent), births(Arm::Founder))),
        ratios: Family::ALL
            .into_iter()
            .map(|family| {
                let (a, z) = (births(Arm::Authored(family)), births(Arm::Inert(family)));
                PairedRatios {
                    family: family.as_key().to_string(),
                    a_over_z: ratio_text(ratio(a, z)),
                    a_over_f: ratio_text(ratio(a, births(Arm::Founder))),
                    a_over_i: incumbent(ratio(a, births(Arm::Incumbent))),
                    z_over_f: ratio_text(ratio(z, births(Arm::Founder))),
                }
            })
            .collect(),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerdictRow {
    pub family: String,
    pub world: String,
    pub verdict: Verdict,
    pub sampled: u64,
    pub exposed: u64,
    pub applied: u64,
    pub informative: u32,
    pub above_one: u32,
    pub below_line: u32,
    pub ratios: Vec<Option<String>>,
    pub pooled: Option<String>,
    pub p_above_one: String,
    pub p_below_line: String,
}

fn verdict_row(family: Family, world: &str, reading: &WorldVerdict) -> VerdictRow {
    VerdictRow {
        family: family.as_key().to_string(),
        world: world.to_string(),
        verdict: reading.verdict,
        sampled: reading.exposure.sampled,
        exposed: reading.exposure.exposed,
        applied: reading.exposure.applied,
        informative: reading.informative,
        above_one: reading.above_one,
        below_line: reading.below_line,
        ratios: reading.ratios.iter().map(|&r| ratio_text(r)).collect(),
        pooled: ratio_text(reading.pooled),
        p_above_one: crate::six(reading.p_above_one),
        p_below_line: crate::six(reading.p_below_line),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldSummary {
    pub case: GoalCase,
    pub replicate_config_digest: String,
    pub incumbent_source_ticks: u64,
    pub incumbents: u32,
    pub founder_fallback: bool,
    pub replicates_requested: u32,
    pub replicates: Vec<ReplicateRow>,
    pub verdicts: Vec<VerdictRow>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FamilySummary {
    pub family: String,
    pub reference: String,
    pub authored_sub_indices: Vec<u16>,
    pub structure: AuthoredStructure,
    pub verdict: Verdict,
    pub world_verdicts: Vec<Verdict>,
    pub competence: Competence,
    pub feasibility: Feasibility,
    pub discovery: Discovery,
    pub vm: VmConcern,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawIdentity {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}

/// The committed summary of one assay or pilot run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunSummary {
    pub kind: String,
    pub version: String,
    pub feature: String,
    pub pilot: bool,
    pub source_revision: String,
    pub threads: usize,
    pub wall_secs: f64,
    pub incomplete: bool,
    pub stop_reason: Option<String>,
    pub horizon: u64,
    pub rules: Vec<String>,
    pub controller_constants: Vec<(String, f32)>,
    pub raw: RawIdentity,
    pub worlds: Vec<WorldSummary>,
    pub families: Vec<FamilySummary>,
    pub gate_favorable: bool,
    pub vm_notes: String,
}

/// What a run produced.
#[derive(Debug, Clone)]
pub struct Outcome {
    pub raw: PathBuf,
    pub summary: PathBuf,
    pub incomplete: bool,
    pub replicates: u64,
    pub bytes: u64,
}

fn rules() -> Vec<String> {
    [
        "arms: F founder; I incumbents; A_k authored controller and Z_k its zero-weight twin for \
         k in ring, vector, scalar; the k-th seeded creature carries arm k mod 8 and every \
         descendant its founder's arm by inherited lineage",
        "births: creatures alive at a tick's end and not at the previous tick's end, by their \
         lineage's arm; newborns dying within their birth tick are unattributed_births",
        "run seed 26_000_000 + 1_000 world_index + replicate; mutation per_unit_rate 0.0; every \
         other setting at recipe and production values",
        "incumbents: up to 20 living genomes of the goal config at its goal seed at the horizon, \
         drawn from the id-sorted population with SmallRng::seed_from_u64(27_000_000 + goal_seed)",
        "exposure: every 100th tick, up to 32 living creatures per A/Z arm drawn with \
         SmallRng::seed_from_u64(29_000_000 + 16 (run_seed - 26_000_000) + tick / 100); exposed \
         when an authored channel reads nonzero; applied when ablating the authored channels \
         changes the committed actions of one execution from fresh cognition state",
        "verdict: gate exposed >= 5% and applied >= 1% of sampled A_k; informative a + z >= 20; \
         positive when >= 7 of the 8 replicates are both informative and a/z > 1, and sum a / \
         sum z >= 1.05; negative when >= 7 of the 8 are both informative and a/z < 1.05, and the \
         pooled ratio < 1.05; an uninformative replicate never counts; otherwise inconclusive; \
         sign-test tails with n = 8",
        "family: positive when any applicable world is, negative when all are; F02-F05 gate \
         favorable with two positive Graph-feasible families, one non-ring",
    ]
    .map(str::to_string)
    .to_vec()
}

fn controller_constants() -> Vec<(String, f32)> {
    [
        ("ring_inhibition", controllers::RING_INHIBITION),
        ("vector_far_threshold", controllers::VECTOR_FAR_THRESHOLD),
        ("vector_gate_threshold", controllers::VECTOR_GATE_THRESHOLD),
        ("vector_weight", controllers::VECTOR_WEIGHT),
        ("scalar_only_threshold", controllers::SCALAR_ONLY_THRESHOLD),
        ("scalar_eat_weight", controllers::SCALAR_EAT_WEIGHT),
        ("scalar_type_weight", controllers::SCALAR_TYPE_WEIGHT),
    ]
    .map(|(name, value)| (name.to_string(), value))
    .to_vec()
}

/// The worlds the assay runs: the goal worlds, resized under test sizes.
fn worlds(sizes: Sizes) -> Vec<(GoalCase, SimulationConfig)> {
    crate::bench::goal_world_configs()
        .into_iter()
        .map(|(case, mut config)| {
            if let Some((width, height, founders)) = sizes.world {
                config.world.width = width;
                config.world.height = height;
                config.population.initial_creatures = founders;
            }
            (case, config)
        })
        .collect()
}

/// The goal config run at its goal seed to the horizon, and its incumbents.
fn incumbents(config: &SimulationConfig, goal_seed: u64, horizon: u64) -> Vec<CreatureGenome> {
    let mut sim = seed_simulation(config.clone(), goal_seed);
    for _ in 0..horizon {
        if sim.creatures.is_empty() {
            break;
        }
        run_tick(&mut sim, &mut None);
    }
    draw_incumbents(&sim, goal_seed)
}

/// The raw and summary paths: explicit paths win; otherwise both default
/// names follow the run, so a pilot and a full run never share a file.
fn output_paths(options: &Options) -> Result<OutputPaths, String> {
    let (profile, suffix) = if options.pilot {
        ("input-opportunity-pilot", "-opportunity-pilot")
    } else {
        ("input-opportunity", "-opportunity")
    };
    output_paths_with_suffix(
        &options.cwd,
        profile,
        suffix,
        Some(&options.feature),
        options.raw.as_deref(),
        options.summary.as_deref(),
    )
}

/// Run the assay, write both artifacts, and report what was produced.
///
/// # Errors
/// When an output path cannot be resolved or written.
pub fn run(options: &Options) -> Result<Outcome, String> {
    let sizes = options.sizes();
    let paths = output_paths(options)?;
    let pool = thread_pool(options.threads)?;
    let work = || execute(options, sizes, &paths.raw);
    let (execution, threads) = match &pool {
        Some(pool) => pool.install(|| (work(), rayon::current_num_threads())),
        None => (work(), rayon::current_num_threads()),
    };
    let execution = execution?;
    let mut summary = RunSummary {
        kind: SUMMARY_KIND.into(),
        version: assay::VERSION.into(),
        feature: options.feature.clone(),
        pilot: options.pilot,
        source_revision: options.source_revision.clone(),
        threads,
        wall_secs: execution.wall_secs,
        incomplete: execution.incomplete,
        stop_reason: execution.stop_reason.clone(),
        horizon: sizes.horizon,
        rules: rules(),
        controller_constants: controller_constants(),
        raw: RawIdentity {
            path: paths.raw.display().to_string(),
            bytes: execution.bytes,
            sha256: execution.sha256.clone(),
        },
        worlds: execution.worlds,
        families: execution.families,
        gate_favorable: execution.gate_favorable,
        vm_notes: "counted, not run: each hand translation follows the founder vote node's \
                   straight-line VM translation; fresh VM input reads and payload draws \
                   (audit S1, M2) are F03's"
            .to_string(),
    };
    let summary_bytes = serde_json::to_vec_pretty(&summary)
        .map_err(|e| format!("failed to serialize the summary: {e}"))?
        .len() as u64;
    if summary_bytes > SUMMARY_CAP_BYTES {
        summary.incomplete = true;
        summary
            .stop_reason
            .get_or_insert_with(|| "summary_cap".to_string());
    }
    write_json(&paths.summary, &summary)?;
    Ok(Outcome {
        raw: paths.raw,
        summary: paths.summary,
        incomplete: summary.incomplete,
        replicates: execution.replicates,
        bytes: execution.bytes,
    })
}

struct Execution {
    worlds: Vec<WorldSummary>,
    families: Vec<FamilySummary>,
    gate_favorable: bool,
    incomplete: bool,
    stop_reason: Option<String>,
    replicates: u64,
    bytes: u64,
    sha256: String,
    wall_secs: f64,
}

/// Write `lines` to `raw`, one per line, and return its byte count and SHA-256.
fn write_raw<'a>(
    raw: &Path,
    lines: impl Iterator<Item = &'a str>,
) -> Result<(u64, String), String> {
    let mut out = HashingWriter::create(raw)?;
    for line in lines {
        out.write_all(line.as_bytes())
            .and_then(|()| out.write_all(b"\n"))
            .map_err(|e| io_error(raw, &e))?;
    }
    out.flush().map_err(|e| io_error(raw, &e))?;
    Ok((out.bytes, out.sha256()))
}

fn execute(options: &Options, sizes: Sizes, raw: &Path) -> Result<Execution, String> {
    let started = Instant::now();
    let worlds = worlds(sizes);
    let first = &worlds[0].1;
    let founder =
        founder_genome_with_age_gate(first.population.founder_profile, &first.energy.lifecycle);
    let genomes: Vec<ArmGenomes> = worlds
        .par_iter()
        .map(|(case, config)| {
            ArmGenomes::new(
                founder.clone(),
                incumbents(config, case.seed, sizes.horizon),
            )
        })
        .collect();

    let tasks: Vec<(usize, u32)> = (0..worlds.len())
        .flat_map(|world| (0..sizes.replicates).map(move |replicate| (world, replicate)))
        .collect();
    let stop = OnceLock::<String>::new();
    let recorded = AtomicU64::new(0);
    // Replicates finish in any order; their lines are counted against the
    // byte cap as they finish and written in world/replicate order after.
    let mut runs: Vec<(usize, u32, ReplicateRun, String)> = tasks
        .par_iter()
        .filter_map(|&(world, replicate)| {
            if stop.get().is_some() {
                return None;
            }
            if started.elapsed() >= options.wall_cap {
                let _ = stop.set("wall_cap".into());
                return None;
            }
            let run = run_replicate(
                &replicate_config(&worlds[world].1),
                run_seed(world, replicate),
                &genomes[world],
                sizes.horizon,
            );
            let line =
                serde_json::json!({"world": world, "replicate": replicate, "run": run}).to_string();
            let total =
                recorded.fetch_add(line.len() as u64 + 1, Ordering::SeqCst) + line.len() as u64 + 1;
            if total >= options.byte_cap {
                let _ = stop.set("byte_cap".into());
            }
            Some((world, replicate, run, line))
        })
        .collect();
    runs.sort_by_key(|(world, replicate, _, _)| (*world, *replicate));
    let (bytes, sha256) = write_raw(raw, runs.iter().map(|(_, _, _, line)| line.as_str()))?;
    let runs: Vec<(usize, u32, ReplicateRun)> = runs
        .into_iter()
        .map(|(world, replicate, run, _)| (world, replicate, run))
        .collect();
    let replicates = runs.len() as u64;
    let incomplete = runs.len() < tasks.len();

    let battery = Battery::generate(discovery::DISCOVERY_FOOD_TYPES);
    let mut world_summaries = Vec::with_capacity(worlds.len());
    let mut verdicts_by_family: Vec<Vec<Verdict>> = vec![Vec::new(); Family::ALL.len()];
    for (index, ((case, config), genomes)) in worlds.iter().zip(&genomes).enumerate() {
        let (replicate_indices, world_runs): (Vec<u32>, Vec<ReplicateRun>) = runs
            .iter()
            .filter(|(world, _, _)| *world == index)
            .map(|(_, replicate, run)| (*replicate, run.clone()))
            .unzip();
        let verdicts = world_verdicts(index, &world_runs);
        for (family, reading) in Family::ALL.into_iter().zip(&verdicts) {
            verdicts_by_family[family.index() as usize].push(reading.verdict);
        }
        let fallback = genomes.founder_fallback();
        world_summaries.push(WorldSummary {
            case: case.clone(),
            replicate_config_digest: v3_core::config::config_digest(&replicate_config(config)),
            incumbent_source_ticks: sizes.horizon,
            incumbents: genomes.incumbents.len() as u32,
            founder_fallback: fallback,
            replicates_requested: sizes.replicates,
            replicates: replicate_indices
                .into_iter()
                .zip(&world_runs)
                .map(|(replicate, run)| replicate_row(replicate, run, fallback))
                .collect(),
            verdicts: Family::ALL
                .into_iter()
                .zip(&verdicts)
                .map(|(family, reading)| verdict_row(family, &case.name, reading))
                .collect(),
        });
    }
    let families: Vec<FamilySummary> = Family::ALL
        .into_par_iter()
        .map(|family| {
            let world_verdicts = verdicts_by_family[family.index() as usize].clone();
            let (_, structure) = controller(&founder, family, false);
            FamilySummary {
                family: family.as_key().to_string(),
                reference: format!("{:?}", family.reference()),
                authored_sub_indices: family.authored_sub_indices().to_vec(),
                structure,
                verdict: family_verdict(&world_verdicts),
                world_verdicts,
                competence: competence(
                    &founder,
                    family,
                    &battery,
                    &first.runtime,
                    first.shared_memory.decay_rate,
                ),
                feasibility: feasibility(&founder, family, first),
                discovery: discovery(&founder, family, first, sizes.discovery_proposals),
                vm: vm_concern(family, first, discovery::FOUNDER_VM_TRANSLATION_STEPS),
            }
        })
        .collect();
    let gate = gate_favorable(
        &families
            .iter()
            .zip(Family::ALL)
            .map(|(summary, family)| {
                (
                    family == Family::Ring,
                    summary.verdict,
                    summary.feasibility.feasible,
                )
            })
            .collect::<Vec<_>>(),
    );
    debug_assert!(Family::ALL
        .into_iter()
        .all(|family| (0..3).any(|world| applicable(family, world))));
    let wall_secs = started.elapsed().as_secs_f64();
    let stop_reason = stop
        .into_inner()
        .or_else(|| (wall_secs > options.wall_cap.as_secs_f64()).then(|| "wall_cap".to_string()));
    Ok(Execution {
        worlds: world_summaries,
        families,
        gate_favorable: gate,
        incomplete: incomplete || stop_reason.is_some(),
        stop_reason,
        replicates,
        bytes,
        sha256,
        wall_secs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "petri-opportunity-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::SeqCst)
            ));
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// A throwaway Git checkout for exercising default artifact paths without
    /// relying on test execution from the source checkout.
    fn git_checkout(dir: &Temp) -> &Path {
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .current_dir(&dir.0)
                .args(args)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        };
        git(&["init", "--quiet"]);
        git(&[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "-c",
            "core.hooksPath=/dev/null",
            "commit",
            "--quiet",
            "--allow-empty",
            "-m",
            "fixture",
        ]);
        &dir.0
    }

    const REDUCED: Sizes = Sizes {
        replicates: 2,
        horizon: 110,
        world: Some((20, 20, 32)),
        discovery_proposals: 50,
    };

    fn reduced(dir: &Path, name: &str, threads: usize) -> Options {
        Options {
            threads: NonZeroUsize::new(threads),
            sizes: Some(REDUCED),
            raw: Some(dir.join(format!("{name}-raw.jsonl"))),
            summary: Some(dir.join(format!("{name}.json"))),
            ..Options::new("t20-f01-test", dir.to_path_buf())
        }
    }

    fn deterministic(summary: &Path) -> serde_json::Value {
        let mut value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(summary).unwrap()).unwrap();
        let object = value.as_object_mut().unwrap();
        for key in ["wall_secs", "threads"] {
            object.remove(key);
        }
        object["raw"].as_object_mut().unwrap().remove("path");
        value
    }

    /// Two reduced runs, at different thread counts, write byte-identical
    /// raw records and summaries identical apart from wall time, thread
    /// count and the raw path (the raw hash and size included).
    #[test]
    fn reduced_runs_are_byte_identical() {
        let dir = Temp::new();
        let one = run(&reduced(&dir.0, "one", 1)).unwrap();
        let two = run(&reduced(&dir.0, "two", 3)).unwrap();
        assert!(!one.incomplete);
        assert_eq!(one.replicates, 6);
        assert_eq!(
            std::fs::read(&one.raw).unwrap(),
            std::fs::read(&two.raw).unwrap()
        );
        assert_eq!(deterministic(&one.summary), deterministic(&two.summary));
        let summary: RunSummary =
            serde_json::from_slice(&std::fs::read(&one.summary).unwrap()).unwrap();
        assert_eq!(summary.worlds.len(), 3);
        assert_eq!(summary.families.len(), 3);
        for world in &summary.worlds {
            assert_eq!(world.replicates.len(), 2);
            for replicate in &world.replicates {
                let attributed: u64 = replicate.arms.iter().map(|arm| arm.births).sum();
                assert_eq!(
                    attributed + replicate.unattributed_births,
                    replicate.births_total
                );
            }
            for row in &world.verdicts {
                assert_ne!(
                    row.verdict,
                    Verdict::Positive,
                    "two replicates never decide"
                );
            }
        }
        // The null references are reported, never judged.
        assert_eq!(
            summary.worlds[0].verdicts[0].verdict,
            Verdict::NotApplicable
        );
        assert_eq!(
            summary.worlds[1].verdicts[2].verdict,
            Verdict::NotApplicable
        );
        assert!(summary
            .families
            .iter()
            .all(|family| family.competence.violations == 0));
    }

    /// A pilot must never overwrite the full run's raw record (or the
    /// reverse): each default raw path is named after its own run.
    #[test]
    fn pilot_and_full_runs_resolve_to_distinct_raw_paths() {
        let dir = Temp::new();
        let checkout = git_checkout(&dir).to_path_buf();
        let full = Options::new("t20-f01-x", checkout);
        let pilot = Options {
            pilot: true,
            ..full.clone()
        };
        let full = output_paths(&full).unwrap();
        let pilot = output_paths(&pilot).unwrap();
        assert!(full
            .raw
            .ends_with(".bench-artifacts/t20-f01-x/input-opportunity.json"));
        assert!(pilot
            .raw
            .ends_with(".bench-artifacts/t20-f01-x/input-opportunity-pilot.json"));
        assert!(full
            .summary
            .ends_with("docs/progress/features/t20-f01-x-opportunity.json"));
        assert!(pilot
            .summary
            .ends_with("docs/progress/features/t20-f01-x-opportunity-pilot.json"));
    }

    #[test]
    fn an_explicit_raw_path_is_kept_for_a_pilot() {
        let dir = Temp::new();
        let options = Options {
            pilot: true,
            ..reduced(&dir.0, "explicit", 1)
        };
        let paths = output_paths(&options).unwrap();
        assert!(paths.raw.ends_with("explicit-raw.jsonl"));
        assert!(paths.summary.ends_with("explicit.json"));
    }

    #[test]
    fn a_wall_cap_of_zero_stops_before_any_replicate() {
        let dir = Temp::new();
        let options = Options {
            wall_cap: Duration::ZERO,
            ..reduced(&dir.0, "capped", 2)
        };
        let outcome = run(&options).unwrap();
        assert!(outcome.incomplete);
        assert_eq!(outcome.replicates, 0);
        let summary: RunSummary =
            serde_json::from_slice(&std::fs::read(&outcome.summary).unwrap()).unwrap();
        assert_eq!(summary.stop_reason.as_deref(), Some("wall_cap"));
        assert!(summary
            .worlds
            .iter()
            .all(|world| world.verdicts.iter().all(|row| matches!(
                row.verdict,
                Verdict::InconclusiveExposure | Verdict::NotApplicable
            ))));
    }
}
