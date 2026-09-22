//! Report comparison, reference selection, and the series index.

use super::artifacts;
use super::profiles::{ADDITIVE_COUNTER_NAMES, COUNTER_NAMES, GOAL_WORLD_SET};
use super::schema::{
    ByReaderState, CaseComparison, CaseReadingComparison, ComparisonLevel, CounterComparison,
    GoalIndicators, Host, MemorySensitivitySeed, MovesBlockedByCause, PerCreatureTick, PerSeed,
    ProfileBlock, ReferenceComparison, Report, TemporalMemorySensitivitySeed, WallClockComparison,
    VALUE_ONLY_CASE_READINGS,
};
use crate::six;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Deterministic work-counter regression thresholds against each reference,
/// per the T10.F10 spec.
pub(super) const FLAG_PERCENT: f64 = 10.0;
pub(super) const SEVERE_PERCENT: f64 = 50.0;

/// Wall-clock regression thresholds against a host-matching reference. They
/// are looser than the work-counter ones because wall-clock is a noisy
/// secondary signal, and a wall-clock level never fails a comparison.
pub(super) const WALL_CLOCK_FLAG_PERCENT: f64 = 25.0;
pub(super) const WALL_CLOCK_SEVERE_PERCENT: f64 = 100.0;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Comparison {
    pub references: Vec<ReferenceComparison>,
    /// Why `references` is empty, when it is: one of the fixed cause strings
    /// built by [`apply_comparisons`] and the series-index resolvers. Absent
    /// whenever at least one reference was compared, and in historical reports.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_absence: Option<String>,
    pub severe: bool,
}

pub(super) fn percent_delta(current: f64, reference: f64) -> Option<f64> {
    if reference == 0.0 {
        return None;
    }
    Some((current - reference) / reference * 100.0)
}

fn counter_level(delta_percent: Option<f64>) -> ComparisonLevel {
    match delta_percent {
        None => ComparisonLevel::Ok,
        Some(d) if d > SEVERE_PERCENT => ComparisonLevel::Severe,
        Some(d) if d > FLAG_PERCENT => ComparisonLevel::Flag,
        _ => ComparisonLevel::Ok,
    }
}

/// Look up one counter's `per_creature_tick` value. Returns `None` when the
/// counter is absent from this report (an older schema), which the caller
/// must surface as `level: "new"` rather than treating as zero.
fn per_creature_tick_value(pct: &PerCreatureTick, name: &str) -> Option<f64> {
    let raw = match name {
        "mesh_hops" => &pct.mesh_hops,
        "vm_steps" => &pct.vm_steps,
        "graph_relax_iters" => &pct.graph_relax_iters,
        "plasticity_updates" => &pct.plasticity_updates,
        "actions_applied" => &pct.actions_applied,
        "births" => &pct.births,
        "pass_cap_hits" => &pct.pass_cap_hits,
        _ => unreachable!("unknown counter name: {name}"),
    };
    raw.as_deref().and_then(|s| s.parse::<f64>().ok())
}

/// Parse a six-decimal report string; `Undefined` reads as unmeasured.
fn parse_reading(value: &str) -> Option<f64> {
    value.parse::<f64>().ok()
}

/// A per-food-type reading is followed as one reading per type, named
/// `{name}_type_{index}`, so each type is compared against its own history.
fn per_food_type_readings<'a>(
    name: &'a str,
    values: &'a [String],
) -> impl Iterator<Item = (String, Option<f64>)> + 'a {
    values
        .iter()
        .enumerate()
        .map(move |(index, value)| (format!("{name}_type_{index}"), parse_reading(value)))
}

/// A reader-state fraction is followed as one reading per state, named for the
/// state, so each is compared against its own history.
fn by_reader_state_readings(
    name: &str,
    fractions: &ByReaderState<String>,
) -> [(String, Option<f64>); 2] {
    [
        (
            format!("{name}_has_barrier_reader"),
            parse_reading(&fractions.has_barrier_reader),
        ),
        (
            format!("{name}_no_barrier_reader"),
            parse_reading(&fractions.no_barrier_reader),
        ),
    ]
}

/// Blocked moves are followed as one total per cause, named for the cause, so
/// a world that trades barrier blocks for crowding is readable as such.
fn by_cause_readings(blocked: Option<&MovesBlockedByCause>) -> [(String, Option<f64>); 3] {
    [
        ("moves_blocked_barrier_total", blocked.map(|by| by.barrier)),
        (
            "moves_blocked_occupied_total",
            blocked.map(|by| by.occupied),
        ),
        (
            "moves_blocked_out_of_bounds_total",
            blocked.map(|by| by.out_of_bounds),
        ),
    ]
    .map(|(name, count)| (name.to_string(), count.map(|count| count as f64)))
}

/// Every reading a world-set comparison follows for one case, in report order.
/// The per-seed goal indicators that live beside the cases rather than inside
/// them, read from the row whose seed is the case's run seed. Every one is
/// `None` when its indicator is `Undefined` or the row is absent.
fn per_seed_indicator_readings(
    indicators: &GoalIndicators,
    seed: u64,
) -> [(String, Option<f64>); 6] {
    let lineage = indicators
        .lineage_diversity
        .defined()
        .and_then(|reading| reading.per_seed.iter().find(|row| row.seed == seed));
    let memory = indicators
        .memory_sensitivity
        .defined()
        .and_then(|reading| reading.per_seed.iter().find(|row| row.seed == seed));
    // Matched on the outer row's seed; the component rows carry their own.
    let temporal = indicators
        .temporal_memory_sensitivity
        .defined()
        .and_then(|reading| reading.per_seed.iter().find(|row| row.seed == seed));
    let temporal_fraction =
        |component: fn(&TemporalMemorySensitivitySeed) -> &MemorySensitivitySeed| {
            temporal.and_then(|row| parse_reading(&component(row).different_from_either_fraction))
        };
    [
        (
            "lineage_shannon_entropy_nats".to_string(),
            lineage.and_then(|row| parse_reading(&row.shannon_entropy_nats)),
        ),
        (
            "surviving_founder_clade_count".to_string(),
            lineage.map(|row| row.surviving_founder_clade_count as f64),
        ),
        (
            "memory_different_from_either_fraction".to_string(),
            memory.and_then(|row| parse_reading(&row.different_from_either_fraction)),
        ),
        (
            "temporal_memory_previous_slots_different_from_either_fraction".to_string(),
            temporal_fraction(|row| &row.previous_slots),
        ),
        (
            "temporal_memory_persisted_outputs_different_from_either_fraction".to_string(),
            temporal_fraction(|row| &row.persisted_outputs),
        ),
        (
            "temporal_memory_operator_state_different_from_either_fraction".to_string(),
            temporal_fraction(|row| &row.operator_state),
        ),
    ]
}

/// An absent case, an unmeasured indicator, and a zero denominator all read as
/// `None` rather than as a zero the delta would then compare against.
fn case_readings(report: &Report, case_name: &str) -> Vec<(String, Option<f64>)> {
    let indicators = &report.deterministic.goal_indicators;
    let Some(observation) = indicators
        .cases
        .iter()
        .find(|entry| entry.case.name == case_name)
    else {
        return Vec::new();
    };
    let seed = observation.case.seed;
    let persistence = indicators
        .population_persistence
        .per_seed
        .iter()
        .find(|row| row.seed == seed);
    let run = report
        .deterministic
        .per_seed
        .iter()
        .find(|row| row.seed == seed);
    let neighborhood = observation.mutational_neighborhood.defined();
    let evolved = neighborhood.and_then(|reading| reading.evolved.defined()?.per_seed.first());
    let neighborhood_read = observation.neighborhood_read.defined();
    let per_creature_tick = |count: fn(&PerSeed) -> u64| {
        run.and_then(|row| {
            (row.creature_ticks > 0).then(|| count(row) as f64 / row.creature_ticks as f64)
        })
    };

    let mut readings: Vec<(String, Option<f64>)> = vec![
        (
            "final_population".into(),
            persistence.map(|row| row.final_population as f64),
        ),
        (
            "minimum_population".into(),
            persistence.map(|row| row.minimum_population as f64),
        ),
        (
            "peak_population".into(),
            persistence.map(|row| row.peak_population as f64),
        ),
        (
            "plateau_population".into(),
            persistence
                .and_then(|row| row.plateau_population.as_deref())
                .and_then(parse_reading),
        ),
        ("births".into(), run.map(|row| row.births as f64)),
        (
            "mean_energy".into(),
            persistence
                .and_then(|row| row.mean_energy.as_deref())
                .and_then(parse_reading),
        ),
        (
            "extinction_tick".into(),
            persistence
                .and_then(|row| row.extinction_tick)
                .map(|tick| tick as f64),
        ),
    ];
    // The same counters the profile totals normalize, in the same order, so
    // a per-case row can never drift from the profile-level counter list.
    let counts: [fn(&PerSeed) -> u64; COUNTER_NAMES.len()] = [
        |row| row.mesh_hops,
        |row| row.vm_steps,
        |row| row.graph_relax_iters,
        |row| row.plasticity_updates,
        |row| row.actions_applied,
        |row| row.births,
        |row| row.pass_cap_hits,
    ];
    for (name, count) in COUNTER_NAMES.iter().zip(counts) {
        readings.push((
            format!("{name}_per_creature_tick"),
            per_creature_tick(count),
        ));
    }
    readings.extend(per_food_type_readings(
        "typed_eat_share",
        &observation.fractions.typed_eat_share,
    ));
    readings.extend(per_food_type_readings(
        "grazing_modifier_mean",
        &observation.tracking.grazing_modifier_mean,
    ));
    readings.extend(per_food_type_readings(
        "grazed_cell_share",
        &observation.tracking.grazed_cell_share,
    ));
    readings.extend(by_reader_state_readings(
        "barrier_blocked_fraction",
        &observation
            .fractions
            .barrier_blocked_fraction_by_reader_state,
    ));
    readings.extend(by_reader_state_readings(
        "avoidable_blocked_share_of_all_moves",
        &observation
            .fractions
            .avoidable_blocked_share_of_all_moves_by_reader_state,
    ));
    readings.extend(by_cause_readings(
        observation.tracking.moves_blocked_total_by_cause.as_ref(),
    ));
    readings.push((
        "blocked_move_fraction".to_string(),
        parse_reading(&observation.fractions.blocked_move_fraction),
    ));
    readings.extend(per_seed_indicator_readings(indicators, seed));
    readings.extend([
        (
            "drift_changed_per_all_births_at_2000".to_string(),
            observation.drift_depth.defined().and_then(|drift| {
                drift
                    .readings
                    .iter()
                    .find(|row| row.depth == 2_000)
                    .and_then(|row| parse_reading(&row.changed_per_all_births))
            }),
        ),
        (
            "founder_changed_per_all_births".to_string(),
            neighborhood.and_then(|reading| {
                parse_reading(&reading.founder.births.any_events.changed_fraction)
            }),
        ),
        (
            "founder_dead_per_all_births".to_string(),
            neighborhood.and_then(|reading| {
                parse_reading(&reading.founder.births.any_events.dead_fraction)
            }),
        ),
        // `any_events.*_fraction` divides by mutated births (`applied`), so the
        // key names that denominator (T14.F12); the values are unchanged.
        (
            "evolved_changed_per_mutated_births".to_string(),
            evolved.and_then(|row| parse_reading(&row.pooled_births.any_events.changed_fraction)),
        ),
        (
            "evolved_dead_per_mutated_births".to_string(),
            evolved.and_then(|row| parse_reading(&row.pooled_births.any_events.dead_fraction)),
        ),
        (
            "neighborhood_read_changed_per_all_births".to_string(),
            neighborhood_read.and_then(|read| parse_reading(&read.changed_per_all_births)),
        ),
        (
            "neighborhood_read_dead_per_all_births".to_string(),
            neighborhood_read.and_then(|read| parse_reading(&read.dead_per_all_births)),
        ),
        (
            "neighborhood_read_silent_per_all_births".to_string(),
            neighborhood_read.and_then(|read| parse_reading(&read.silent_per_all_births)),
        ),
        (
            "reachable_structure_size_median".to_string(),
            observation
                .reachable_structure_size_distribution
                .as_ref()
                .map(|distribution| f64::from(distribution.median)),
        ),
    ]);
    readings
}

/// Compare every world in `current` against the same-named world in
/// `reference`. Empty outside the world-set profile.
pub(super) fn compare_cases(
    current: &ComparisonInputs,
    reference: &ComparisonInputs,
) -> Vec<CaseComparison> {
    if current.profile.name != GOAL_WORLD_SET {
        return Vec::new();
    }
    current
        .profile
        .cases
        .iter()
        .map(|case| {
            let reference_case = reference
                .profile
                .cases
                .iter()
                .find(|other| other.name == case.name);
            let reference_readings = reference.readings(&case.name);
            let readings = current
                .readings(&case.name)
                .into_iter()
                .map(|(name, current_value)| {
                    let reference_value = reference_readings
                        .iter()
                        .find(|(other, _)| *other == name)
                        .and_then(|(_, value)| *value);
                    let percent_delta = (!VALUE_ONLY_CASE_READINGS.contains(&name.as_str()))
                        .then(|| current_value.zip(reference_value))
                        .flatten()
                        .and_then(|(current, reference)| percent_delta(current, reference));
                    CaseReadingComparison {
                        name,
                        current: current_value.map(six),
                        reference: reference_value.map(six),
                        percent_delta: percent_delta.map(six),
                    }
                })
                .collect();
            CaseComparison {
                case: case.name.clone(),
                inputs_changed: reference_case.is_some_and(|other| {
                    other.config_digest != case.config_digest || other.seed != case.seed
                }),
                absent_in_reference: reference_case.is_none(),
                current_digest: case.config_digest.clone(),
                reference_digest: reference_case.map(|other| other.config_digest.clone()),
                readings,
            }
        })
        .collect()
}

/// Compare `current` against one stored reference report, returning a
/// `ReferenceComparison`. Only integer work counters (never wall-clock) can
/// mark a comparison `severe`.
pub fn compare_against(
    current: &Report,
    reference_path: &Path,
    reference: &Report,
) -> ReferenceComparison {
    compare_inputs(&current.into(), reference_path, &reference.into())
}

/// Minimal measured inputs shared by full reports and summaries. Case readings
/// and wall time use round-trip decimal strings, not six-decimal display values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonInputs {
    pub identity: MeasuredIdentity,
    pub profile: ProfileBlock,
    pub per_creature_tick: PerCreatureTick,
    pub host: Host,
    pub wall_clock_ms_per_creature_tick: String,
    pub case_readings: BTreeMap<String, Vec<(String, Option<String>)>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeasuredIdentity {
    pub feature: String,
    pub git_revision: String,
    pub generated_at: String,
}

impl From<&Report> for ComparisonInputs {
    fn from(report: &Report) -> Self {
        Self {
            identity: MeasuredIdentity {
                feature: report.feature.clone(),
                git_revision: report.environment.git_revision.clone(),
                generated_at: report.environment.generated_at.clone(),
            },
            profile: report.deterministic.profile.clone(),
            per_creature_tick: report.deterministic.per_creature_tick.clone(),
            host: report.environment.host.clone(),
            wall_clock_ms_per_creature_tick: report
                .environment
                .wall_clock_ms_per_creature_tick
                .to_string(),
            case_readings: report
                .deterministic
                .profile
                .cases
                .iter()
                .map(|case| {
                    (
                        case.name.clone(),
                        case_readings(report, &case.name)
                            .into_iter()
                            .map(|(name, value)| (name, value.map(|v| v.to_string())))
                            .collect(),
                    )
                })
                .collect(),
        }
    }
}

impl ComparisonInputs {
    fn readings(&self, case: &str) -> Vec<(String, Option<f64>)> {
        self.case_readings
            .get(case)
            .into_iter()
            .flatten()
            .map(|(name, value)| (name.clone(), value.as_deref().and_then(parse_reading)))
            .collect()
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        let normalized = &self.per_creature_tick;
        for value in std::iter::once(self.wall_clock_ms_per_creature_tick.as_str())
            .chain(
                [
                    &normalized.mesh_hops,
                    &normalized.vm_steps,
                    &normalized.graph_relax_iters,
                    &normalized.plasticity_updates,
                    &normalized.actions_applied,
                    &normalized.births,
                ]
                .into_iter()
                .filter_map(|value| value.as_deref()),
            )
            .chain(
                self.case_readings
                    .values()
                    .flatten()
                    .filter_map(|(_, value)| value.as_deref()),
            )
        {
            if !value.parse::<f64>().is_ok_and(f64::is_finite) {
                return Err(format!("invalid lossless comparison reading: {value}"));
            }
        }
        if self
            .profile
            .cases
            .iter()
            .any(|case| !self.case_readings.contains_key(&case.name))
        {
            return Err("summary is missing comparison readings for a declared case".into());
        }
        Ok(())
    }
}

fn compare_inputs(
    current: &ComparisonInputs,
    reference_path: &Path,
    reference: &ComparisonInputs,
) -> ReferenceComparison {
    let mut counters = Vec::with_capacity(COUNTER_NAMES.len());
    let mut any_severe = false;

    for &name in &COUNTER_NAMES {
        let current_value = match per_creature_tick_value(&current.per_creature_tick, name) {
            Some(value) => value,
            None if ADDITIVE_COUNTER_NAMES.contains(&name) => 0.0,
            None => panic!("a freshly built report always populates the {name} counter"),
        };
        let reference_value = per_creature_tick_value(&reference.per_creature_tick, name);

        let (level, reference_str, delta_str) = match reference_value {
            None => (ComparisonLevel::New, None, None),
            // The reference recorded no work for this counter but the
            // current run does: the ratio is unbounded (division by zero),
            // so treat it as severe rather than silently reporting "ok"
            // with a null delta. This is the only way a feature that
            // introduces the first nonzero reading of a counter (e.g. the
            // first plastic founder genome) can trip a regression at all.
            Some(reference_value) if reference_value == 0.0 && current_value > 0.0 => {
                any_severe = true;
                (ComparisonLevel::Severe, Some(six(reference_value)), None)
            }
            Some(reference_value) => {
                let delta = percent_delta(current_value, reference_value);
                let level = counter_level(delta);
                if level == ComparisonLevel::Severe {
                    any_severe = true;
                }
                (level, Some(six(reference_value)), delta.map(six))
            }
        };

        counters.push(CounterComparison {
            name: name.to_string(),
            current: six(current_value),
            reference: reference_str,
            percent_delta: delta_str,
            level,
        });
    }

    let wall_clock = if current.host == reference.host {
        let current_ms = current
            .wall_clock_ms_per_creature_tick
            .parse()
            .expect("validated wall reading");
        let reference_ms = reference
            .wall_clock_ms_per_creature_tick
            .parse()
            .expect("validated wall reading");
        let delta = percent_delta(current_ms, reference_ms).unwrap_or(0.0);
        let level = if delta > WALL_CLOCK_SEVERE_PERCENT {
            ComparisonLevel::Severe
        } else if delta > WALL_CLOCK_FLAG_PERCENT {
            ComparisonLevel::Flag
        } else {
            ComparisonLevel::Ok
        };
        Some(WallClockComparison {
            current_ms_per_creature_tick: current_ms,
            reference_ms_per_creature_tick: reference_ms,
            percent_delta: delta,
            level,
        })
    } else {
        None
    };

    ReferenceComparison {
        path: reference_path.display().to_string(),
        measured_identity: Some(reference.identity.clone()),
        counters,
        cases: compare_cases(current, reference),
        wall_clock,
        severe: any_severe,
    }
}

/// The profile block two reports must agree on to be comparable at all.
///
/// For the world set the per-case block is excluded: every closure that edits a
/// recipe changes that case's `config_digest`, and the standard-baseline
/// contract expects exactly that. Hard-failing on it would discard a ten-minute
/// run, so a digest change is reported per case as `inputs_changed` and a case
/// the reference never ran is reported as `absent_in_reference`. World size,
/// founder count, seeds, ticks, and food coverage still have to match.
fn comparable_profile(profile: &ProfileBlock) -> ProfileBlock {
    let mut profile = profile.clone();
    if profile.name == GOAL_WORLD_SET {
        profile.cases.clear();
    }
    profile
}

/// Load a reference report from disk and compare `current` against it.
/// Returns `Err` when the reference file cannot be read or parsed — a
/// missing declared reference is a hard error, not a silent skip.
pub fn compare_against_path(
    current: &Report,
    reference_path: &Path,
) -> Result<ReferenceComparison, String> {
    let content = std::fs::read_to_string(reference_path)
        .map_err(|e| format!("failed to read reference {}: {e}", reference_path.display()))?;
    let reference = artifacts::comparison_inputs_from_bytes(content.as_bytes()).map_err(|e| {
        format!(
            "failed to parse reference {}: {e}",
            reference_path.display()
        )
    })?;
    if comparable_profile(&reference.profile) != comparable_profile(&current.deterministic.profile)
    {
        return Err(format!(
            "reference {} was generated with a different profile ({:?}) than the \
             current run ({:?}); a work-counter comparison across different world \
             size, founder count, seeds, ticks, or food coverage is meaningless. \
             Re-pin the reference or exclude it.",
            reference_path.display(),
            reference.profile,
            current.deterministic.profile
        ));
    }
    Ok(compare_inputs(&current.into(), reference_path, &reference))
}

/// The reference paths a run compares against, with the cause when the
/// selection is empty, so the reason travels with the (lack of) paths.
/// Explicit `--baseline`/`--compare` paths carry no absence.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReferenceSelection {
    pub paths: Vec<PathBuf>,
    pub absence: Option<String>,
}

impl ReferenceSelection {
    fn absent(cause: String) -> Self {
        Self {
            paths: Vec::new(),
            absence: Some(cause),
        }
    }
}

/// Compare `current` against every selected reference that is not the report's
/// own output path, folding the results into `current.comparison`, and return
/// whether any reference was severe. A skipped self-reference is never an
/// error and never severe; when nothing remains to compare, the comparison
/// records why.
pub fn apply_comparisons(
    current: &mut Report,
    selection: &ReferenceSelection,
    out_path: &Path,
) -> Result<bool, String> {
    apply_comparisons_for_outputs(current, selection, &[out_path])
}

/// Both artifacts identify this run and are excluded from its references.
pub fn apply_comparisons_for_outputs(
    current: &mut Report,
    selection: &ReferenceSelection,
    outputs: &[&Path],
) -> Result<bool, String> {
    let mut references = Vec::with_capacity(selection.paths.len());
    let mut overall_severe = false;
    let mut skipped_self = false;
    for path in &selection.paths {
        let mut is_output = false;
        for output in outputs {
            is_output |= artifacts::same_path(path, output)?;
        }
        if is_output {
            skipped_self = true;
            continue;
        }
        let entry = compare_against_path(current, path)?;
        if entry.severe {
            overall_severe = true;
        }
        references.push(entry);
    }
    let reference_absence = if references.is_empty() {
        Some(selection.absence.clone().unwrap_or_else(|| {
            if skipped_self {
                format!(
                    "the only candidate reference is this report's own output path {}",
                    outputs
                        .iter()
                        .map(|path| path.display().to_string())
                        .collect::<Vec<_>>()
                        .join(" or ")
                )
            } else {
                "no reference paths were given".to_string()
            }
        }))
    } else {
        None
    };
    current.comparison = Comparison {
        references,
        reference_absence,
        severe: overall_severe,
    };
    Ok(overall_severe)
}

// ── Series index (docs/progress/benchmark-series.json) ──────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesIndex {
    pub series: String,
    pub epoch_baseline: String,
    pub closed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSeriesIndex {
    pub gate: SeriesIndex,
    pub goal: SeriesIndex,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal_worlds: Option<SeriesIndex>,
}

/// Resolve the gate profile's default comparison references from the series
/// index: the epoch baseline and the last closed report (if different).
/// An absent series index (this feature's own first report, which becomes the
/// epoch baseline) is an empty selection carrying that cause.
pub fn default_gate_references(series_index_path: &Path) -> Result<ReferenceSelection, String> {
    let Some(index) = read_series_index(series_index_path)? else {
        return Ok(ReferenceSelection::absent(no_series_index(
            series_index_path,
        )));
    };
    Ok(references_from_series(&index.gate))
}

/// Resolve the goal profile's comparison references from its distinct series.
/// Its initial baseline does not exist until that first goal report is written,
/// so the first invocation deliberately has no comparison and says so.
pub fn default_goal_references(series_index_path: &Path) -> Result<ReferenceSelection, String> {
    let Some(index) = read_series_index(series_index_path)? else {
        return Ok(ReferenceSelection::absent(no_series_index(
            series_index_path,
        )));
    };
    let Some(series) = index.goal_worlds.as_ref() else {
        return Ok(ReferenceSelection::absent(no_epoch_baseline(
            GOAL_WORLD_SET,
        )));
    };
    if !Path::new(&series.epoch_baseline).exists() {
        return Ok(ReferenceSelection::absent(no_epoch_baseline(
            &series.series,
        )));
    }
    Ok(references_from_series(series))
}

fn read_series_index(series_index_path: &Path) -> Result<Option<BenchmarkSeriesIndex>, String> {
    if !series_index_path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(series_index_path)
        .map_err(|e| format!("failed to read {}: {e}", series_index_path.display()))?;
    serde_json::from_str(&content)
        .map(Some)
        .map_err(|e| format!("failed to parse {}: {e}", series_index_path.display()))
}

fn no_series_index(series_index_path: &Path) -> String {
    format!("no series index at {}", series_index_path.display())
}

fn no_epoch_baseline(series: &str) -> String {
    format!("series {series} has no stored epoch baseline yet")
}

fn references_from_series(index: &SeriesIndex) -> ReferenceSelection {
    let mut paths = vec![PathBuf::from(&index.epoch_baseline)];
    if let Some(last) = index.closed.last() {
        if last != &index.epoch_baseline {
            paths.push(PathBuf::from(last));
        }
    }
    ReferenceSelection {
        paths,
        absence: None,
    }
}

/// Read the simulation crate's dependency identity, not lockfile package order.
pub(super) fn locked_rand_version() -> &'static str {
    rand_version_from_lock(include_str!("../../../../Cargo.lock"))
        .expect("workspace lockfile identifies v3-core's rand version")
}

fn rand_version_from_lock(lockfile: &str) -> Option<&str> {
    let packages = lockfile.split("[[package]]");
    let core = packages
        .clone()
        .find(|package| package.lines().any(|line| line == "name = \"v3-core\""))?;
    if let Some(version) = core.lines().find_map(|line| {
        line.trim()
            .strip_prefix("\"rand ")
            .and_then(|dependency| dependency.strip_suffix("\","))
    }) {
        return Some(version);
    }
    // Cargo omits the version in dependency identities when the name is unique.
    if !core.lines().any(|line| line.trim() == "\"rand\",") {
        return None;
    }
    let mut versions = packages
        .filter(|package| package.lines().any(|line| line == "name = \"rand\""))
        .filter_map(|package| {
            package.lines().find_map(|line| {
                line.strip_prefix("version = \"")
                    .and_then(|version| version.strip_suffix('"'))
            })
        });
    let version = versions.next()?;
    versions.next().is_none().then_some(version)
}

// ── Persistence accumulator unit tests ──────────────────────────────────────

#[cfg(test)]
mod tests;
