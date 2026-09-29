//! The summary keep-list (`kind: petri-lab-summary`, `summary_version: 3`)
//! and the report rendered from it alone. v2 summaries (before T22.F03's
//! readings) still render, without the readings section; v1 is refused.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::arena::{Policy, Role};
use crate::readings::{ReplicateReadings, SignatureArms};
use crate::scene::Assay;

pub const SUMMARY_KIND: &str = "petri-lab-summary";
pub const SUMMARY_VERSION: u32 = 3;
/// The oldest summary version `report` still renders.
pub const SUMMARY_VERSION_MIN: u32 = 2;
pub const GENOME_FORMAT: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Summary {
    pub kind: String,
    pub summary_version: u32,
    pub assay: Assay,
    pub provenance: Provenance,
    pub calibration: Calibration,
    pub arms: Vec<ArmSummary>,
    /// Reference arm only; null when no campaign ran.
    pub fidelity: Option<Fidelity>,
    /// Null for a complete run.
    pub incomplete: Option<Incomplete>,
    pub exit_code: u8,
    /// The only non-deterministic block.
    pub timing: Timing,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    pub git_revision: Option<String>,
    pub dirty: Option<bool>,
    /// Present only when a library caller injected a root that is not a
    /// checkout; `git_revision` and `dirty` are then null.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git: Option<GitMarker>,
    /// SHA-256 of the resolved reference config JSON.
    pub config_digest: String,
    pub overlays: Vec<OverlayRecord>,
    pub genomes: Vec<GenomeRecord>,
    pub arena: ArenaRecord,
    pub seeds: Seeds,
    pub threads: usize,
    pub sizes: Sizes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GitMarker {
    Unavailable,
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
    /// Canonical descriptor: `arena_version`, id, size, start, food type,
    /// axis value, geometry and sampling rules (a layout's rows).
    pub spec: serde_json::Value,
    /// SHA-256 of `spec` serialized with sorted keys.
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
    pub scale: Option<u8>,
    pub lifetime: Option<u32>,
    pub blocked_weight: f64,
    pub efficiency_weight: f64,
    pub quick: bool,
    /// Resolved `--mutants`; absent from v2 summaries.
    #[serde(default)]
    pub mutants: Option<u32>,
    /// Resolved `--signature-arms`; absent from v2 summaries.
    #[serde(default)]
    pub signature_arms: Option<SignatureArms>,
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
    pub floor_blocked_fraction: f64,
    pub comparator_blocked_fraction: f64,
    pub floor_efficiency: f64,
    pub half_efficiency: f64,
    pub oracle_efficiency: f64,
    pub comparator_efficiency: f64,
    /// Scenes on which the comparator scored above the floor.
    pub comparator_wins: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationPoint {
    /// The axis value: `food_fraction` or `scale`, the other null (both
    /// null on a layout).
    pub food_fraction: Option<f64>,
    pub scale: Option<u8>,
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
    pub food_fraction: Option<f64>,
    pub scale: Option<u8>,
    pub lifetime: u32,
    /// The selected point's validation means.
    pub means: PointMeans,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Calibration {
    pub margin: f64,
    pub points: Vec<CalibrationPoint>,
    pub selected: Option<Selected>,
    pub verdict: Verdict,
    pub reach_threshold: Option<f64>,
    pub reach_threshold_source: Option<ThresholdSource>,
}

/// Where the reach threshold came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThresholdSource {
    Calibrated,
    Override,
}

/// Why a run stopped short.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Incomplete {
    Uncalibrated,
    ByteCap,
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
    /// Null for instrument arms, whose reach is not tested.
    pub reached: Option<bool>,
    pub generation_to_threshold: Option<u32>,
    pub censored: Option<bool>,
    pub incomplete: bool,
    pub stopped_by: StoppedBy,
    pub generations_run: u32,
    pub final_best: Option<f64>,
    /// Projections of the first and last written rows' readings; absent
    /// from v2 summaries.
    #[serde(default)]
    pub readings: ReplicateReadings,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArmSummary {
    pub name: String,
    pub role: Role,
    pub policy: Policy,
    /// Whether the reach fields are native reachability (a `native`,
    /// non-instrument arm); a tested `policy-deviation` arm's are a
    /// diagnostic.
    pub reach_reported: bool,
    pub replicates: Vec<ReplicateResult>,
    /// Over completed replicates; null when any replicate is incomplete or
    /// the arm is an instrument.
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
    pub lifetime_learning: LifetimeLearning,
}

/// Lifetime learning during evaluation: masked in every arm (T22.F02).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifetimeLearning {
    Masked,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Timing {
    pub wall_seconds: f64,
    /// Production-tick creature-ticks (genome evaluations; scripted
    /// instruments excluded).
    pub creature_ticks: u64,
    pub per_creature_tick_ms: Option<f64>,
    /// Production creature-ticks on ablated copies and mutants (also in
    /// `creature_ticks`); absent from v2 summaries.
    #[serde(default)]
    pub readings_creature_ticks: u64,
}

fn fmt_opt(value: Option<f64>) -> String {
    value.map_or_else(|| "-".to_owned(), |v| format!("{v:.3}"))
}

fn label<T: Serialize>(value: T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_else(|| "-".to_owned())
}

/// One reach table; instruments, whose reach is not tested, print `n/a`.
fn arm_table(out: &mut String, arms: &[&ArmSummary]) {
    let _ = writeln!(
        out,
        "| arm | role | policy | reached k/n | Wilson 95% | median gen to threshold | censored | incomplete |"
    );
    let _ = writeln!(out, "| --- | --- | --- | --- | --- | --- | --- | --- |");
    for arm in arms {
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
        let (kn, interval, generation, censored) = if arm.role == Role::Instrument {
            ("n/a".into(), "n/a".into(), "n/a".into(), "n/a".into())
        } else {
            (
                format!("{}/{}", reached.len(), completed.len()),
                arm.wilson_95
                    .map_or("-".into(), |[lo, hi]| format!("[{lo:.3}, {hi:.3}]")),
                fmt_opt(crate::stats::median(&reached)),
                censored.to_string(),
            )
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | {kn} | {interval} | {generation} | {censored} | {} |",
            arm.name,
            label(arm.role),
            label(arm.policy),
            arm.incomplete_replicates,
        );
    }
}

/// A point's axis cell: its fraction or scale, `layout` for a layout.
fn axis_cell(food_fraction: Option<f64>, scale: Option<u8>) -> String {
    match (food_fraction, scale) {
        (Some(fraction), _) => fraction.to_string(),
        (_, Some(scale)) => scale.to_string(),
        _ => "layout".to_owned(),
    }
}

/// Render the assay report from a summary alone. `policy-deviation` arms'
/// reach prints in a separate diagnostic table.
#[must_use]
pub fn render_report(summary: &Summary) -> String {
    let mut out = String::new();
    let calibration = &summary.calibration;
    let _ = writeln!(out, "# {} assay report", summary.assay.name());
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
            .map_or(String::new(), |why| format!(", incomplete: {}", label(why))),
    );
    let sizes = &summary.provenance.sizes;
    let _ = writeln!(
        out,
        "arena {}, size {}, blocked weight {}, efficiency weight {}",
        summary.provenance.arena.spec["id"]
            .as_str()
            .unwrap_or("unknown"),
        sizes.arena_size,
        sizes.blocked_weight,
        sizes.efficiency_weight,
    );
    let _ = writeln!(out, "\n## Calibration: {:?}\n", calibration.verdict);
    let axis = match calibration.points.first() {
        Some(point) if point.scale.is_some() => "scale",
        Some(point) if point.food_fraction.is_none() => "point",
        _ => "fraction",
    };
    let _ = writeln!(
        out,
        "| {axis} | lifetime | exposure | competence | sensitivity | validation | founder | floor | half | oracle | comparator | floor blocked | comparator blocked | floor efficiency | half efficiency | oracle efficiency | comparator efficiency |"
    );
    let _ = writeln!(
        out,
        "| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |"
    );
    for point in &calibration.points {
        let means = point.means.as_ref();
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            axis_cell(point.food_fraction, point.scale),
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
            fmt_opt(means.map(|m| m.floor_blocked_fraction)),
            fmt_opt(means.map(|m| m.comparator_blocked_fraction)),
            fmt_opt(means.map(|m| m.floor_efficiency)),
            fmt_opt(means.map(|m| m.half_efficiency)),
            fmt_opt(means.map(|m| m.oracle_efficiency)),
            fmt_opt(means.map(|m| m.comparator_efficiency)),
        );
    }
    if let Some(selected) = &calibration.selected {
        let _ = writeln!(
            out,
            "\nSelected {axis} {} lifetime {}; reach threshold {} ({}).",
            axis_cell(selected.food_fraction, selected.scale),
            selected.lifetime,
            fmt_opt(calibration.reach_threshold),
            calibration.reach_threshold_source.map_or("-".into(), label),
        );
    }
    let (native, diagnostic): (Vec<&ArmSummary>, Vec<&ArmSummary>) = summary
        .arms
        .iter()
        .partition(|arm| arm.policy == Policy::Native);
    if !native.is_empty() {
        let _ = writeln!(out, "\n## Arms\n");
        arm_table(&mut out, &native);
    }
    if !diagnostic.is_empty() {
        let _ = writeln!(
            out,
            "\n## Diagnostic reach (policy-deviation, not native reachability)\n"
        );
        arm_table(&mut out, &diagnostic);
    }
    if let Some(fidelity) = &summary.fidelity {
        let _ = writeln!(
            out,
            "\nFidelity (reference): per_unit_rate {}, executed_bias {}, window {} ticks; events requested {}, applied {}, skipped {}; identical offspring {}; elite carry-overs {}; phenotype mutation {}, learned-weight capture {}, lifetime learning {}.",
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
            label(fidelity.lifetime_learning),
        );
    }
    if summary.summary_version >= 3 && !summary.arms.is_empty() {
        readings_section(&mut out, &summary.arms);
    }
    let _ = writeln!(
        out,
        "\nTiming: {:.1} s wall, {} creature-ticks ({} on readings), {} ms per creature-tick.",
        summary.timing.wall_seconds,
        summary.timing.creature_ticks,
        summary.timing.readings_creature_ticks,
        summary
            .timing
            .per_creature_tick_ms
            .map_or("-".into(), |ms| format!("{ms:.5}")),
    );
    out
}

/// `last (±delta)`.
fn with_delta<T: Into<i64> + Copy>(last: T, first: T) -> String {
    let (last, first) = (last.into(), first.into());
    format!("{last} ({:+})", last - first)
}

/// `num/den = ratio`, `null` at a zero denominator.
fn ratio(num: u64, den: u64) -> String {
    if den == 0 {
        "null".to_owned()
    } else {
        #[allow(clippy::cast_precision_loss)]
        let value = num as f64 / den as f64;
        format!("{num}/{den} = {value:.3}")
    }
}

fn opt_bool(value: Option<bool>) -> String {
    value.map_or_else(|| "null".to_owned(), |v| v.to_string())
}

fn usize_i64(value: usize) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

/// The readings section (v3): one row per arm and replicate with the last
/// written row's elite and its change since the first, then each one's
/// family table and signature aggregates.
fn readings_section(out: &mut String, arms: &[ArmSummary]) {
    let _ = writeln!(
        out,
        "\n## Readings (elite: last written row, change since the first)\n"
    );
    let _ = writeln!(
        out,
        "| arm | replicate | genome size | functional complexity | nodes | reachable | executed | deaths | births | applied | steering exact/moves | avoided/trials |"
    );
    let _ = writeln!(
        out,
        "| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |"
    );
    for arm in arms {
        for replicate in &arm.replicates {
            let readings = &replicate.readings;
            let (Some(first), Some(last)) = (&readings.first, &readings.last) else {
                let _ = writeln!(
                    out,
                    "| {} | {} | - | - | - | - | - | - | - | - | - | - |",
                    arm.name, replicate.replicate
                );
                continue;
            };
            let shape = &last.shape;
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                arm.name,
                replicate.replicate,
                with_delta(shape.genome_size, first.genome_size),
                with_delta(shape.functional_complexity, first.functional_complexity),
                with_delta(usize_i64(shape.nodes), usize_i64(first.nodes)),
                with_delta(usize_i64(shape.reachable), usize_i64(first.reachable)),
                with_delta(usize_i64(shape.executed), usize_i64(first.executed)),
                shape.deaths,
                shape.ancestry.births,
                shape.ancestry.applied,
                ratio(shape.steering.exact_hits, shape.steering.moves),
                ratio(shape.steering.avoided, shape.steering.avoidance_trials),
            );
        }
    }
    for arm in arms {
        for replicate in &arm.replicates {
            let Some(last) = &replicate.readings.last else {
                continue;
            };
            let _ = writeln!(
                out,
                "\n### {} replicate {}\n",
                arm.name, replicate.replicate
            );
            let _ = writeln!(
                out,
                "| family | structural | executed_node | live | causal | score_delta |"
            );
            let _ = writeln!(out, "| --- | --- | --- | --- | --- | --- |");
            let causal = last
                .signature
                .as_ref()
                .and_then(|signature| signature.causal.as_ref());
            for family in &last.shape.families {
                let reading =
                    causal.and_then(|causal| causal.iter().find(|c| c.family == family.family));
                let _ = writeln!(
                    out,
                    "| {} | {} | {} | {} | {} | {} |",
                    family.family,
                    family.structural,
                    family.executed_node,
                    opt_bool(family.live),
                    reading.map_or("-".into(), |c| c.causal.to_string()),
                    reading.map_or("-".into(), |c| format!("{:.3}", c.score_delta)),
                );
            }
            let _ = match &last.signature {
                None => writeln!(out, "\nsignature: not computed"),
                Some(s) => writeln!(
                    out,
                    "\nsignature: {} mutants, {} identical; silent {}, changed {}, dead {}; improved {}, equal {}, worse {}; scores min {:.3} median {:.3} max {:.3}; mean delta {:.3}{}",
                    s.n,
                    s.identical,
                    s.silent,
                    s.changed,
                    s.dead,
                    s.improved,
                    s.equal,
                    s.worse,
                    s.score_min,
                    s.score_median,
                    s.score_max,
                    s.mean_delta,
                    s.unsupported
                        .as_ref()
                        .map_or(String::new(), |why| format!("; causal unsupported: {why}")),
                ),
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn events(scale: u64, operator: &str) -> Events {
        let map = |n: u64| BTreeMap::from([(operator.to_owned(), n)]);
        Events {
            requested: 3 * scale,
            applied: 2 * scale,
            skipped: scale,
            requested_by_operator: map(3 * scale),
            applied_by_operator: map(2 * scale),
            skipped_by_operator: map(scale),
            offspring: 4 * scale,
            identical_offspring: scale,
        }
    }

    #[test]
    fn events_add_sums_every_count_and_merges_operator_keys() {
        let mut total = events(1, "a");
        total.add(&events(2, "a"));
        total.add(&events(5, "b"));
        let map = |a: u64, b: u64| BTreeMap::from([("a".to_owned(), a), ("b".to_owned(), b)]);
        assert_eq!(
            total,
            Events {
                requested: 24,
                applied: 16,
                skipped: 8,
                requested_by_operator: map(9, 15),
                applied_by_operator: map(6, 10),
                skipped_by_operator: map(3, 5),
                offspring: 32,
                identical_offspring: 8,
            }
        );
    }

    #[test]
    fn identical_offspring_fraction_is_a_share_of_offspring_or_null() {
        assert_eq!(Events::default().identical_offspring_fraction(), None);
        let mut some = Events {
            offspring: 4,
            identical_offspring: 1,
            ..Events::default()
        };
        assert_eq!(some.identical_offspring_fraction(), Some(0.25));
        some.identical_offspring = 0;
        assert_eq!(some.identical_offspring_fraction(), Some(0.0));
    }

    #[test]
    fn report_cells_format_numbers_and_labels() {
        assert_eq!(fmt_opt(Some(1.23456)), "1.235");
        assert_eq!(fmt_opt(None), "-");
        assert_eq!(label(Role::Reference), "reference");
        assert_eq!(label(Policy::PolicyDeviation), "policy-deviation");
    }

    #[test]
    fn axis_cell_prints_the_fraction_else_the_scale_else_layout() {
        assert_eq!(axis_cell(Some(0.04), None), "0.04");
        assert_eq!(axis_cell(None, Some(2)), "2");
        assert_eq!(axis_cell(None, None), "layout");
    }

    fn replicate(generation: Option<u32>, incomplete: bool) -> ReplicateResult {
        ReplicateResult {
            replicate: 0,
            reached: (!incomplete).then_some(generation.is_some()),
            generation_to_threshold: generation,
            censored: (!incomplete).then_some(generation.is_none()),
            incomplete,
            stopped_by: if incomplete {
                StoppedBy::ByteCap
            } else {
                StoppedBy::Horizon
            },
            generations_run: 3,
            final_best: None,
            readings: ReplicateReadings::default(),
        }
    }

    fn arm(name: &str, role: Role) -> ArmSummary {
        ArmSummary {
            name: name.into(),
            role,
            policy: Policy::Native,
            reach_reported: role != Role::Instrument,
            replicates: vec![
                replicate(Some(2), false),
                replicate(Some(4), false),
                replicate(None, false),
                replicate(Some(9), true),
            ],
            reached_fraction: None,
            wilson_95: None,
            incomplete_replicates: 1,
        }
    }

    #[test]
    fn arm_table_counts_completed_replicates_and_prints_n_a_for_instruments() {
        let tested = arm("native", Role::Reference);
        let instrument = arm("comparator", Role::Instrument);
        let mut out = String::new();
        arm_table(&mut out, &[&tested, &instrument]);
        let rows: Vec<&str> = out.lines().skip(2).collect();
        assert_eq!(
            rows,
            [
                "| native | reference | native | 2/3 | - | 3.000 | 1 | 1 |",
                "| comparator | instrument | native | n/a | n/a | n/a | n/a | 1 |",
            ]
        );
    }
}
