//! The summary keep-list (`kind: petri-lab-summary`, `summary_version: 1`)
//! and the report rendered from it alone.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::arena::{Policy, Role};

pub const SUMMARY_KIND: &str = "petri-lab-summary";
pub const SUMMARY_VERSION: u32 = 1;
pub const GENOME_FORMAT: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Summary {
    pub kind: String,
    pub summary_version: u32,
    pub assay: String,
    pub provenance: Provenance,
    pub calibration: Calibration,
    pub arms: Vec<ArmSummary>,
    /// Reference arm only; null when no campaign ran.
    pub fidelity: Option<Fidelity>,
    /// Null, `"byte_cap"` or `"uncalibrated"`.
    pub incomplete: Option<String>,
    pub exit_code: u8,
    /// The only non-deterministic block.
    pub timing: Timing,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    pub git_revision: Option<String>,
    pub dirty: Option<bool>,
    /// SHA-256 of the resolved reference config JSON.
    pub config_digest: String,
    pub overlays: Vec<OverlayRecord>,
    pub genomes: Vec<GenomeRecord>,
    pub arena: ArenaRecord,
    pub seeds: Seeds,
    pub threads: usize,
    pub sizes: Sizes,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OverlayRecord {
    pub name: String,
    pub content: serde_json::Value,
    /// Index in the run's overlay list: the built-in `mutation-off` overlay
    /// is 0, user `--arm` overlays follow in command-line order. Each is
    /// applied alone over the arena config.
    pub order: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenomeRecord {
    pub name: String,
    pub sha256: String,
    pub genome_format: u32,
    pub v3_core_version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArenaRecord {
    pub spec: serde_json::Value,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Seeds {
    pub seed: u64,
    pub replicates: Vec<u64>,
    pub calibration: u64,
    pub validation: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sizes {
    pub replicates: u32,
    pub generations: u32,
    pub population: u32,
    pub elite_fraction: f64,
    pub scenes: u32,
    pub validation_scenes: u32,
    pub arena_size: u16,
    pub start_energy: f32,
    pub food_fraction: Option<f64>,
    pub lifetime: Option<u32>,
    pub quick: bool,
}

/// Mean scores of the calibrated actors at one grid point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PointMeans {
    pub founder: f64,
    pub floor: f64,
    pub half: f64,
    pub oracle: f64,
    pub comparator: f64,
    pub floor_progress: f64,
    pub comparator_progress: f64,
    /// Scenes on which the comparator scored above the floor.
    pub comparator_wins: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationPoint {
    pub food_fraction: f64,
    pub lifetime: u32,
    pub scenes: u32,
    /// Redraws per scene (up to the failing draw when exposure failed).
    pub redraws: Vec<u32>,
    pub exposure: bool,
    /// Null when exposure failed and nothing was scored.
    pub means: Option<PointMeans>,
    pub competence: bool,
    pub sensitivity: bool,
    /// Competence on the validation scenes; null when not evaluated.
    pub validation_competence: Option<bool>,
    pub validation: Option<PointMeans>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Calibrated,
    Uncalibrated,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Selected {
    pub food_fraction: f64,
    pub lifetime: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Calibration {
    pub margin: f64,
    pub points: Vec<CalibrationPoint>,
    pub selected: Option<Selected>,
    pub verdict: Verdict,
    pub reach_threshold: Option<f64>,
    /// `"calibrated"` or `"override"`.
    pub reach_threshold_source: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StoppedBy {
    Reached,
    Horizon,
    ByteCap,
    NotStarted,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplicateResult {
    pub replicate: u32,
    /// Null for arms whose reachability is not reported.
    pub reached: Option<bool>,
    pub generation_to_threshold: Option<u32>,
    pub censored: Option<bool>,
    pub incomplete: bool,
    pub stopped_by: StoppedBy,
    pub generations_run: u32,
    pub final_best: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArmSummary {
    pub name: String,
    pub role: Role,
    pub policy: Policy,
    /// Whether reachability is reported (a `native`, non-instrument arm).
    pub reach_reported: bool,
    pub replicates: Vec<ReplicateResult>,
    /// Over completed replicates; null when any replicate is incomplete or
    /// reachability is not reported.
    pub reached_fraction: Option<f64>,
    pub wilson_95: Option<[f64; 2]>,
    pub incomplete_replicates: u32,
}

/// Mutation events by kind: totals and by operator (keys sorted), plus
/// offspring identity.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Events {
    pub requested: u64,
    pub applied: u64,
    pub skipped: u64,
    pub requested_by_operator: BTreeMap<String, u64>,
    pub applied_by_operator: BTreeMap<String, u64>,
    pub skipped_by_operator: BTreeMap<String, u64>,
    pub offspring: u64,
    /// Offspring whose genome `==` the parent's (distinct from zero applied
    /// events).
    pub identical_offspring: u64,
}

impl Events {
    /// Fold `other` into `self`.
    pub fn add(&mut self, other: &Self) {
        self.requested += other.requested;
        self.applied += other.applied;
        self.skipped += other.skipped;
        for (into, from) in [
            (
                &mut self.requested_by_operator,
                &other.requested_by_operator,
            ),
            (&mut self.applied_by_operator, &other.applied_by_operator),
            (&mut self.skipped_by_operator, &other.skipped_by_operator),
        ] {
            for (key, count) in from {
                *into.entry(key.clone()).or_default() += count;
            }
        }
        self.offspring += other.offspring;
        self.identical_offspring += other.identical_offspring;
    }

    /// Identical offspring over offspring; null with no offspring.
    #[must_use]
    pub fn identical_offspring_fraction(&self) -> Option<f64> {
        (self.offspring > 0).then(|| self.identical_offspring as f64 / self.offspring as f64)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fidelity {
    pub per_unit_rate: f64,
    pub executed_bias: f64,
    pub executed_window_ticks: u64,
    #[serde(flatten)]
    pub events: Events,
    pub identical_offspring_fraction: Option<f64>,
    pub elite_carry_overs: u64,
    pub phenotype_mutation: bool,
    pub learned_weight_capture: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Timing {
    pub wall_seconds: f64,
    /// Production-tick creature-ticks (genome evaluations; scripted
    /// instruments excluded).
    pub creature_ticks: u64,
    pub per_creature_tick_ms: Option<f64>,
}

fn fmt_opt(value: Option<f64>) -> String {
    value.map_or_else(|| "-".to_owned(), |v| format!("{v:.3}"))
}

/// Render the assay report from a summary alone.
#[must_use]
pub fn render_report(summary: &Summary) -> String {
    let mut out = String::new();
    let calibration = &summary.calibration;
    let _ = writeln!(out, "# {} assay report", summary.assay);
    let _ = writeln!(
        out,
        "\nrevision {} (dirty: {}), seed {}, exit {}{}",
        summary
            .provenance
            .git_revision
            .as_deref()
            .unwrap_or("unknown"),
        summary
            .provenance
            .dirty
            .map_or("unknown".into(), |d| d.to_string()),
        summary.provenance.seeds.seed,
        summary.exit_code,
        summary
            .incomplete
            .as_deref()
            .map_or(String::new(), |why| format!(", incomplete: {why}")),
    );
    let _ = writeln!(out, "\n## Calibration: {:?}\n", calibration.verdict);
    let _ = writeln!(
        out,
        "| fraction | lifetime | exposure | competence | sensitivity | validation | founder | floor | half | oracle | comparator |"
    );
    let _ = writeln!(
        out,
        "| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |"
    );
    for point in &calibration.points {
        let means = point.means.as_ref();
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            point.food_fraction,
            point.lifetime,
            point.exposure,
            point.competence,
            point.sensitivity,
            point
                .validation_competence
                .map_or("-".into(), |v| v.to_string()),
            fmt_opt(means.map(|m| m.founder)),
            fmt_opt(means.map(|m| m.floor)),
            fmt_opt(means.map(|m| m.half)),
            fmt_opt(means.map(|m| m.oracle)),
            fmt_opt(means.map(|m| m.comparator)),
        );
    }
    if let Some(selected) = &calibration.selected {
        let _ = writeln!(
            out,
            "\nSelected fraction {} lifetime {}; reach threshold {} ({}).",
            selected.food_fraction,
            selected.lifetime,
            fmt_opt(calibration.reach_threshold),
            calibration.reach_threshold_source.as_deref().unwrap_or("-"),
        );
    }
    if !summary.arms.is_empty() {
        let _ = writeln!(out, "\n## Arms\n");
        let _ = writeln!(
            out,
            "| arm | role | policy | reached k/n | Wilson 95% | median gen to threshold | censored | incomplete |"
        );
        let _ = writeln!(out, "| --- | --- | --- | --- | --- | --- | --- | --- |");
        for arm in &summary.arms {
            let completed: Vec<&ReplicateResult> =
                arm.replicates.iter().filter(|r| !r.incomplete).collect();
            let reached: Vec<f64> = completed
                .iter()
                .filter_map(|r| r.generation_to_threshold.map(f64::from))
                .collect();
            let censored = completed
                .iter()
                .filter(|r| r.censored == Some(true))
                .count();
            let (kn, interval, generation, censored) = if arm.reach_reported {
                (
                    format!("{}/{}", reached.len(), completed.len()),
                    arm.wilson_95
                        .map_or("-".into(), |[lo, hi]| format!("[{lo:.3}, {hi:.3}]")),
                    fmt_opt(crate::stats::median(&reached)),
                    censored.to_string(),
                )
            } else {
                ("n/a".into(), "n/a".into(), "n/a".into(), "n/a".into())
            };
            let _ = writeln!(
                out,
                "| {} | {} | {} | {kn} | {interval} | {generation} | {censored} | {} |",
                arm.name,
                serde_json::to_value(arm.role)
                    .unwrap_or_default()
                    .as_str()
                    .unwrap_or("-"),
                serde_json::to_value(arm.policy)
                    .unwrap_or_default()
                    .as_str()
                    .unwrap_or("-"),
                arm.incomplete_replicates,
            );
        }
    }
    if let Some(fidelity) = &summary.fidelity {
        let _ = writeln!(
            out,
            "\nFidelity (reference): per_unit_rate {}, executed_bias {}, window {} ticks; events requested {}, applied {}, skipped {}; identical offspring {}; elite carry-overs {}; phenotype mutation {}, learned-weight capture {}.",
            fidelity.per_unit_rate,
            fidelity.executed_bias,
            fidelity.executed_window_ticks,
            fidelity.events.requested,
            fidelity.events.applied,
            fidelity.events.skipped,
            fmt_opt(fidelity.identical_offspring_fraction),
            fidelity.elite_carry_overs,
            fidelity.phenotype_mutation,
            fidelity.learned_weight_capture,
        );
    }
    let _ = writeln!(
        out,
        "\nTiming: {:.1} s wall, {} creature-ticks, {} ms per creature-tick.",
        summary.timing.wall_seconds,
        summary.timing.creature_ticks,
        summary
            .timing
            .per_creature_tick_ms
            .map_or("-".into(), |ms| format!("{ms:.5}")),
    );
    out
}
