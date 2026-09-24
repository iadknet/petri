//! Versioned committed evidence; full observation payloads remain local.
use std::collections::BTreeMap;
use std::io::{BufWriter, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

use super::{ComparisonInputs, Indicator, Report};
use v3_core::neighborhood::recruitment_paths as recruitment;

pub const SUMMARY_KIND: &str = "petri-benchmark-summary";
pub const SUMMARY_VERSION: u32 = 1;
pub const MAX_CLAIMS: usize = 16;
pub const MAX_PERSISTENCE_CHECKPOINTS: usize = 21;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Invocation {
    pub executable: String,
    pub arguments: Vec<String>,
    pub working_directory: PathBuf,
}

impl Invocation {
    pub fn capture() -> Result<Self, String> {
        Ok(Self {
            executable: std::env::current_exe()
                .map_err(|e| format!("cannot identify executable: {e}"))?
                .into_os_string()
                .into_string()
                .map_err(|_| "executable path is not UTF-8")?,
            arguments: std::env::args_os()
                .skip(1)
                .map(|arg| {
                    arg.into_string()
                        .map_err(|_| "command argument is not UTF-8".to_string())
                })
                .collect::<Result<_, _>>()?,
            working_directory: std::env::current_dir()
                .map_err(|e| format!("cannot identify working directory: {e}"))?,
        })
    }
}

/// All converter identity and verification time are explicit inputs, making
/// conversion repeatable. Supplied evidence is attributed to its provider and
/// does not replace the original measured identity or raw measurements.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConversionProvenance {
    pub verified_at: String,
    pub converter: Invocation,
    pub supplied_evidence: Option<SuppliedEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuppliedEvidence {
    pub source: String,
    pub command: Option<Invocation>,
    pub cli_exit: Option<i32>,
    pub outer_exit: Option<i32>,
    pub thresholds: Option<Value>,
    pub wall_caps: Option<Value>,
    pub inheritance: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawProvenance {
    pub sha256: String,
    pub bytes: usize,
    pub path: PathBuf,
    pub availability: String,
    pub verified_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub pointer: String,
    pub value: Value,
}

/// The retained presentation JSON is projected from the original JSON, not a
/// reserialized Report: legacy Serde defaults must not invent observations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub kind: String,
    pub summary_version: u32,
    pub source_schema_version: u32,
    pub feature: String,
    pub deterministic: Value,
    pub environment: Value,
    pub comparison: Value,
    pub comparison_inputs: ComparisonInputs,
    pub measurement_evidence: Value,
    pub conversion: ConversionProvenance,
    pub raw: RawProvenance,
    pub claims: Vec<Claim>,
    pub omitted_details: Vec<String>,
}

pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn pick(source: &Value, fields: &[&str]) -> Value {
    Value::Object(
        fields
            .iter()
            .filter_map(|name| {
                source
                    .get(*name)
                    .map(|value| ((*name).to_string(), value.clone()))
            })
            .collect(),
    )
}

/// Copy only fields owned by an existing aggregate type, and only when the
/// source measured them. The aggregate must not contain raw trace payloads.
fn aggregate_fields(source: &Value, aggregate: &impl Serialize) -> Result<Value, String> {
    let model = serde_json::to_value(aggregate).map_err(|e| e.to_string())?;
    let keys: Vec<_> = model
        .as_object()
        .into_iter()
        .flat_map(|object| object.keys())
        .map(String::as_str)
        .collect();
    Ok(pick(source, &keys))
}

fn mesh_summary(source: &Value) -> Value {
    let rows = source
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|genome| {
            genome
                .get("mesh_execution")
                .filter(|value| value.is_object())
        });
    let mut out: BTreeMap<&str, u64> = BTreeMap::new();
    for row in rows {
        *out.entry("genomes").or_default() += 1;
        for (target, key) in [
            ("total", "total_node_count"),
            ("reachable", "reachable_node_count"),
            ("executed", "executed_node_count"),
            ("knockout", "knockout_count"),
            ("capHits", "pass_cap_hits"),
            ("passes", "passes"),
            ("execs", "executions_per_genome"),
        ] {
            if let Some(count) = row[key].as_u64() {
                *out.entry(target).or_default() += count;
            }
        }
        *out.entry("routeVaries").or_default() += u64::from(row["route_varies_with_input"] == true);
        *out.entry("routeDestinationVaries").or_default() +=
            u64::from(row["route_destination_varies"] == true);
        // State-driven flags (T19.F06): a row measured before them carries no
        // key, so the count is kept only over rows that do and is absent,
        // never zero, when none does.
        for (target, key) in [
            ("routeVariesWithinSnapshot", "route_varies_within_snapshot"),
            (
                "routeDestinationVariesWithinSnapshot",
                "route_destination_varies_within_snapshot",
            ),
        ] {
            if let Some(varies) = row.get(key).and_then(Value::as_bool) {
                *out.entry(target).or_default() += u64::from(varies);
            }
        }
    }
    if out.is_empty() {
        Value::Null
    } else {
        json!(out)
    }
}

fn neighborhood(source: &Value) -> Value {
    if !source.is_object() {
        return source.clone();
    }
    let mut out = pick(source, &["battery", "founder"]);
    if let Some(evolved) = source.get("evolved") {
        out["evolved"] = if let Some(rows) = evolved.get("per_seed").and_then(Value::as_array) {
            json!({"per_seed": rows.iter().map(|row| {
                let mut projected = pick(row, &["generation_distribution", "seed", "final_population_size", "pooled_operator_rows", "pooled_births", "steering_pooled"]);
                projected["mesh_summary"] = mesh_summary(&row["sampled_genomes"]);
                projected
            }).collect::<Vec<_>>()})
        } else {
            evolved.clone()
        };
    }
    out
}

fn persistence(source: &Value, report: &Report) -> Result<Value, String> {
    let mut out = Map::new();
    if let Some(rows) = source.get("per_seed").and_then(Value::as_array) {
        let rows = rows
            .iter()
            .zip(
                &report
                    .deterministic
                    .goal_indicators
                    .population_persistence
                    .per_seed,
            )
            .map(|(row, measured)| {
                let mut projected = pick(
                    row,
                    &[
                        "seed",
                        "extinction_tick",
                        "minimum_population",
                        "final_population",
                        "peak_population",
                        "peak_tick",
                        "plateau_population",
                        "mean_energy",
                    ],
                );
                if let Some(samples) = row.get("samples").and_then(Value::as_array) {
                    let count = samples.len().min(MAX_PERSISTENCE_CHECKPOINTS);
                    let mut retained = Vec::with_capacity(count);
                    for index in 0..count {
                        let source_index = if count <= 1 {
                            0
                        } else {
                            index * (samples.len() - 1) / (count - 1)
                        };
                        let sample = &samples[source_index];
                        let mut selected =
                            aggregate_fields(sample, &measured.samples[source_index].tracking)?;
                        for (key, value) in pick(
                            sample,
                            &[
                                "tick",
                                "population",
                                "mean_energy",
                                "births_total",
                                "mean_genome_size",
                                "mean_mesh_nodes",
                                "mean_generation",
                                "surviving_founder_clade_count",
                                "shannon_entropy_nats",
                                "sensor_census",
                                "occupancy_grid",
                            ],
                        )
                        .as_object()
                        .into_iter()
                        .flatten()
                        {
                            selected[key] = value.clone();
                        }
                        retained.push(selected);
                    }
                    projected["samples"] = json!(retained);
                }
                Ok(projected)
            })
            .collect::<Result<Vec<_>, String>>()?;
        out.insert("per_seed".into(), json!(rows));
    }
    Ok(Value::Object(out))
}

fn constructed_stage(stage: &recruitment::ConstructionStage) -> Value {
    json!({"name":stage.name,"edits":stage.edits,"seed":stage.seed,
        "task_summary":stage.task.summary(),"battery_class":stage.battery_class,
        "incumbent_actions_unchanged":stage.incumbent_actions_unchanged,"useful":stage.useful})
}

/// Counts come from observed proposal rows, not sizes.proposals(). No assay,
/// mutation, task evaluation or estimate is rerun during projection.
fn recruitment_summary(source: &Value, report: &recruitment::Report) -> Value {
    let mut out = pick(
        source,
        &[
            "version",
            "config",
            "config_digest",
            "sizes",
            "supply",
            "supply_rule",
            "task_definition",
            "mutation_context",
            "construction_resolution",
            "observation_resolution",
            "rng_control",
            "limitations",
            "total_proposals",
            "opportunities",
            "pairs",
        ],
    );
    out["constructed"] = json!(report.constructed.iter().map(|path| json!({
        "backend":path.backend,"stages":path.stages.iter().map(constructed_stage).collect::<Vec<_>>(),
        "copy_stages":path.copy_stages.iter().map(constructed_stage).collect::<Vec<_>>(),
        "split_stage":path.split_stage.as_ref().map(constructed_stage)
    })).collect::<Vec<_>>());
    out["starts"] =
        json!(report.starts.iter().map(|start| json!({
        "name":start.name,"task":start.task,"backend":start.backend,"scaffold":start.scaffold,
        "creation_operator":start.creation_operator,"mutable_sites":start.mutable_sites,
        "genome_size":start.genome_size,"task_summary":start.task_reading.summary(),
        "history":start.history.iter().map(constructed_stage).collect::<Vec<_>>()
    })).collect::<Vec<_>>());
    out["arms"] = json!(report.arms.iter().map(|arm| {
        let mut outcomes: BTreeMap<String, u64> = BTreeMap::new();
        let mut inapplicable: BTreeMap<_, BTreeMap<_, u64>> = BTreeMap::new();
        let mut unresolved = 0u64;
        let mut total = 0u64;
        let lineages = arm.lineages.iter().map(|lineage| {
            total += lineage.proposals.len() as u64;
            for proposal in &lineage.proposals {
                for (&backend, operators) in &proposal.selected_inapplicable_by_backend_operator {
                    for (&operator, &count) in operators {
                        *inapplicable.entry(backend).or_default().entry(operator).or_default() += count;
                    }
                }
                unresolved += proposal.selected_inapplicable_backend_unresolved;
                for (name, present) in [("chosen", proposal.chosen), ("discovery",proposal.discovery),
                    ("viable_path",proposal.viable_path), ("parent_live",proposal.parent_live),
                    ("all_scenes_survived",proposal.outcome.surviving_scenes == 8)] {
                    *outcomes.entry(name.into()).or_default() += u64::from(present);
                }
                *outcomes.entry(format!("correct_a_{}",proposal.outcome.correct_a)).or_default() += 1;
                *outcomes.entry(format!("correct_b_{}",proposal.outcome.correct_b)).or_default() += 1;
                for event in &proposal.events {
                    *outcomes.entry(format!("event_{}",event.outcome)).or_default() += 1;
                }
            }
            json!({"batch":lineage.batch,"lineage":lineage.lineage,"proposal_count":lineage.proposals.len(),
                "proposal_discovery":lineage.proposal_discovery,"retained_discovery":lineage.retained_discovery,
                "viable_retained_discovery":lineage.viable_retained_discovery,"retention":lineage.retention,
                "specialized_discovery":lineage.specialized_discovery,
                "horizons":lineage.horizons.iter().map(|horizon| json!({"offset":horizon.offset,
                    "at_generation":horizon.at_generation,"outcome":horizon.outcome,"score":horizon.score})).collect::<Vec<_>>(),
                "ladder":lineage.ladder,"classification":lineage.classification})
        }).collect::<Vec<_>>();
        json!({"start":arm.start,"task":arm.task,"policy":arm.policy,"summary":arm.summary,
            "batches":arm.batches,"opportunities":arm.opportunities,"proposal_count":total,
            "selected_inapplicable_by_backend_operator":inapplicable,
            "selected_inapplicable_backend_unresolved":unresolved,
            "outcome_counts":outcomes,"lineages":lineages})
    }).collect::<Vec<_>>());
    out
}

fn goal_indicators(source: &Value, report: &Report) -> Result<Value, String> {
    let measured = &report.deterministic.goal_indicators;
    let mut out = pick(
        source,
        &[
            "births_per_100_ticks",
            "reachable_structure_size_distribution",
            "lineage_diversity",
            "memory_sensitivity",
            "structural_companions",
            "temporal_memory_sensitivity",
            "drift_depth",
            "strategy_count",
            "strategy_causal_distinctness",
            "evolutionary_activity",
            "adaptive_novelty",
            "memory_dependence",
            "learning_dependence",
            "prediction_dependence",
            "information_integration",
            "reciprocal_interaction",
        ],
    );
    out["population_persistence"] = persistence(&source["population_persistence"], report)?;
    if let Some(value) = source.get("mutational_neighborhood") {
        out["mutational_neighborhood"] = neighborhood(value);
    }
    if let Some(value) = source.get("recruitment_paths") {
        out["recruitment_paths"] = match &measured.recruitment_paths {
            Indicator::Defined(experiment) => recruitment_summary(value, experiment),
            Indicator::Undefined(_) => value.clone(),
        };
    }
    if let Some(cases) = source.get("cases").and_then(Value::as_array) {
        out["cases"] = json!(cases
            .iter()
            .zip(&measured.cases)
            .map(|(case, measured)| {
                let mut projected = aggregate_fields(case, &measured.tracking)?;
                for (key, value) in aggregate_fields(case, &measured.fractions)?
                    .as_object()
                    .into_iter()
                    .flatten()
                {
                    projected[key] = value.clone();
                }
                for key in [
                    "case",
                    "reachable_structure_size_distribution",
                    "drift_depth",
                    "neighborhood_read",
                    "mutation_effects",
                ] {
                    if let Some(value) = case.get(key) {
                        projected[key] = value.clone();
                    }
                }
                if let Some(value) = case.get("mutational_neighborhood") {
                    projected["mutational_neighborhood"] = neighborhood(value);
                }
                Ok(projected)
            })
            .collect::<Result<Vec<_>, String>>()?);
    }
    Ok(out)
}

fn claims(source: &Value) -> Vec<Claim> {
    // Fixed source locations, never a traversal that grows with proposal count.
    ["/deterministic/totals/creature_ticks", "/deterministic/totals/births",
        "/deterministic/per_creature_tick/vm_steps", "/comparison/severe",
        "/deterministic/goal_indicators/population_persistence/per_seed/0/final_population",
        "/deterministic/goal_indicators/population_persistence/per_seed/0/minimum_population",
        "/deterministic/goal_indicators/cases/0/drift_depth/readings/0/lineages",
        "/deterministic/goal_indicators/recruitment_paths/total_proposals",
        "/deterministic/goal_indicators/recruitment_paths/opportunities/attempted",
        "/deterministic/goal_indicators/recruitment_paths/arms/0/summary/retained_discovery/numerator",
        "/deterministic/goal_indicators/recruitment_paths/arms/0/summary/retained_discovery/denominator",
        "/deterministic/goal_indicators/recruitment_paths/arms/0/lineages/0/proposals/0/outcome/correct_a",
        "/deterministic/goal_indicators/recruitment_paths/arms/0/lineages/0/proposals/0/chosen",
        "/deterministic/goal_indicators/recruitment_paths/pairs/0/lineages/0/matched_proposals_before_divergence",
        "/deterministic/goal_indicators/recruitment_paths/constructed/0/stages/0/useful",
        "/environment/wall_clock_ms_total"]
        .into_iter().filter_map(|pointer| source.pointer(pointer).filter(|value| !value.is_array() && !value.is_object())
            .map(|value| Claim { pointer:pointer.into(), value:value.clone() })).collect()
}

pub fn summarize(
    raw: &[u8],
    raw_path: &Path,
    provenance: &ConversionProvenance,
) -> Result<Summary, String> {
    if provenance.verified_at.trim().is_empty() {
        return Err("verification time must be supplied".into());
    }
    let source: Value =
        serde_json::from_slice(raw).map_err(|e| format!("invalid full report: {e}"))?;
    if source.get("kind").is_some() {
        return Err("conversion requires a full report, not a summary".into());
    }
    let report: Report =
        serde_json::from_slice(raw).map_err(|e| format!("invalid full report: {e}"))?;
    if report.schema_version != super::SCHEMA_VERSION {
        return Err(format!(
            "unsupported full report version {}",
            report.schema_version
        ));
    }
    let resolved = raw_path
        .canonicalize()
        .map_err(|e| format!("cannot verify raw {}: {e}", raw_path.display()))?;
    let mut stored = std::fs::File::open(&resolved)
        .map_err(|e| format!("cannot verify raw {}: {e}", resolved.display()))?;
    let mut hasher = Sha256::new();
    let mut offset = 0;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = stored
            .read(&mut buffer)
            .map_err(|e| format!("cannot verify raw {}: {e}", resolved.display()))?;
        if count == 0 {
            break;
        }
        if raw.get(offset..offset + count) != Some(&buffer[..count]) {
            return Err("raw bytes differ from the stored artifact".into());
        }
        hasher.update(&buffer[..count]);
        offset += count;
    }
    if offset != raw.len() {
        return Err("raw bytes differ from the stored artifact".into());
    }
    let mut deterministic = pick(
        &source["deterministic"],
        &[
            "graph_work_definition",
            "profile",
            "per_seed",
            "totals",
            "per_creature_tick",
        ],
    );
    deterministic["goal_indicators"] =
        goal_indicators(&source["deterministic"]["goal_indicators"], &report)?;
    Ok(Summary {
        kind: SUMMARY_KIND.into(), summary_version: SUMMARY_VERSION, source_schema_version: report.schema_version,
        feature: report.feature.clone(), deterministic,
        environment: aggregate_fields(&source["environment"], &report.environment)?,
        comparison: source["comparison"].clone(), comparison_inputs: (&report).into(),
        measurement_evidence: source.get("measurement_evidence").cloned().unwrap_or_else(|| json!({
            "command":null,"dirty":null,"cli_exit":null,"outer_exit":null,"thresholds":null,"wall_caps":null,"inheritance":null
        })),
        conversion: provenance.clone(),
        raw: RawProvenance { sha256:format!("{:x}",hasher.finalize()), bytes:offset, path:resolved,
            availability:"verified_local".into(), verified_at:provenance.verified_at.clone() },
        claims: claims(&source),
        omitted_details: ["full genomes and genome deltas", "per-proposal rows", "scene, signature, route and state traces",
            "replay paths", "per-genome neighborhood rows (pooled readings retained)", "persistence checkpoints beyond 21 evenly spaced source indices"]
            .into_iter().map(str::to_string).collect(),
    })
}

pub fn summary_bytes(summary: &Summary) -> Result<Vec<u8>, String> {
    let mut bytes =
        serde_json::to_vec_pretty(summary).map_err(|e| format!("cannot serialize summary: {e}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub(crate) fn comparison_inputs_from_bytes(bytes: &[u8]) -> Result<ComparisonInputs, String> {
    // Inspect only the explicit tag before selecting the historical branch.
    #[derive(Deserialize)]
    struct Header {
        kind: Option<String>,
        summary_version: Option<u32>,
    }
    let header: Header = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    if let Some(kind) = header.kind {
        if kind != SUMMARY_KIND {
            return Err(format!("unsupported artifact kind {kind}"));
        }
        if header.summary_version != Some(SUMMARY_VERSION) {
            return Err(format!(
                "unsupported summary version {:?}",
                header.summary_version
            ));
        }
        let summary: Summary =
            serde_json::from_slice(bytes).map_err(|e| format!("invalid summary: {e}"))?;
        if summary.source_schema_version != super::SCHEMA_VERSION {
            return Err(format!(
                "unsupported summary source version {}",
                summary.source_schema_version
            ));
        }
        summary.comparison_inputs.validate()?;
        let profile: super::ProfileBlock =
            serde_json::from_value(summary.deterministic["profile"].clone())
                .map_err(|e| format!("invalid summary profile: {e}"))?;
        if profile != summary.comparison_inputs.profile
            || summary.feature != summary.comparison_inputs.identity.feature
        {
            return Err("summary identity and comparison inputs disagree".into());
        }
        let environment: super::Environment = serde_json::from_value(summary.environment)
            .map_err(|e| format!("invalid summary environment: {e}"))?;
        let normalized: super::PerCreatureTick =
            serde_json::from_value(summary.deterministic["per_creature_tick"].clone())
                .map_err(|e| format!("invalid summary normalized counters: {e}"))?;
        let inputs = &summary.comparison_inputs;
        if environment.git_revision != inputs.identity.git_revision
            || environment.generated_at != inputs.identity.generated_at
            || environment.host != inputs.host
            || environment.wall_clock_ms_per_creature_tick.to_string()
                != inputs.wall_clock_ms_per_creature_tick
            || normalized != inputs.per_creature_tick
        {
            return Err("summary measured metadata and comparison inputs disagree".into());
        }
        Ok(summary.comparison_inputs)
    } else {
        let report: Report = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if report.schema_version != super::SCHEMA_VERSION {
            return Err(format!(
                "unsupported full report version {}",
                report.schema_version
            ));
        }
        Ok((&report).into())
    }
}

/// Resolve existing ancestors as well as the final file. Symlink aliases must
/// compare equal even before either new output has been created.
pub fn resolved_path(path: &Path) -> Result<PathBuf, String> {
    let absolute =
        std::path::absolute(path).map_err(|e| format!("cannot resolve {}: {e}", path.display()))?;
    let mut resolved = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                resolved.pop();
            }
            other => {
                resolved.push(other);
                match std::fs::symlink_metadata(&resolved) {
                    Ok(_) => {
                        resolved = resolved
                            .canonicalize()
                            .map_err(|e| format!("cannot resolve {}: {e}", resolved.display()))?;
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => return Err(format!("cannot resolve {}: {e}", resolved.display())),
                }
            }
        }
    }
    Ok(resolved)
}

pub fn same_path(a: &Path, b: &Path) -> Result<bool, String> {
    let a = resolved_path(a)?;
    let b = resolved_path(b)?;
    if a == b {
        return Ok(true);
    }
    match same_file::is_same_file(&a, &b) {
        Ok(same) => Ok(same),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(format!(
            "cannot compare file identities {} and {}: {e}",
            a.display(),
            b.display()
        )),
    }
}

fn git_output(cwd: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .current_dir(cwd)
        .args(args)
        .output()
        .map_err(|e| format!("cannot inspect Git context: {e}"))?;
    if !output.status.success() {
        return Err(
            "usable Git checkout required for default outputs; supply --out and --summary-out"
                .into(),
        );
    }
    String::from_utf8(output.stdout).map_err(|e| format!("Git path is not UTF-8: {e}"))
}

#[derive(Debug, Clone)]
pub struct OutputPaths {
    pub raw: PathBuf,
    pub summary: PathBuf,
}

pub fn output_paths(
    cwd: &Path,
    profile: &str,
    feature: Option<&str>,
    raw: Option<&Path>,
    summary: Option<&Path>,
) -> Result<OutputPaths, String> {
    let suffix = if profile == "gate" {
        String::new()
    } else {
        format!("-{profile}")
    };
    output_paths_with_suffix(cwd, profile, &suffix, feature, raw, summary)
}

/// [`output_paths`] with the summary file's suffix chosen by the caller:
/// the raw file is `<profile>.json`, the summary `<feature><suffix>.json`.
pub fn output_paths_with_suffix(
    cwd: &Path,
    profile: &str,
    suffix: &str,
    feature: Option<&str>,
    raw: Option<&Path>,
    summary: Option<&Path>,
) -> Result<OutputPaths, String> {
    if let Some(feature) = feature {
        if feature.is_empty()
            || !feature
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err("feature label must contain only letters, digits, '-' or '_'".into());
        }
    }
    let unlabelled_sweep = profile == "sweep" && feature.is_none();
    let raw_path = if let Some(raw) = raw {
        cwd.join(raw)
    } else {
        let feature = feature.ok_or("--feature or explicit --out is required for sweep")?;
        if git_output(cwd, &["rev-parse", "--is-inside-work-tree"])?.trim() != "true" {
            return Err("usable Git checkout required for default outputs".into());
        }
        let worktrees = git_output(cwd, &["worktree", "list", "--porcelain", "-z"])?;
        if worktrees
            .split('\0')
            .take_while(|field| !field.is_empty())
            .any(|field| field == "bare")
        {
            return Err(
                "main Git worktree is bare; supply explicit --out and --summary-out".into(),
            );
        }
        let main = worktrees
            .split('\0')
            .next()
            .and_then(|line| line.strip_prefix("worktree "))
            .ok_or("Git did not identify the main checkout")?;
        if !Path::new(main).is_dir() {
            return Err("main checkout is unavailable; supply explicit output paths".into());
        }
        Path::new(main)
            .join(".bench-artifacts")
            .join(feature)
            .join(format!("{profile}.json"))
    };
    let summary_path = if let Some(summary) = summary {
        cwd.join(summary)
    } else if unlabelled_sweep {
        let stem = raw_path
            .file_stem()
            .ok_or("raw output must name a file")?
            .to_string_lossy();
        raw_path.with_file_name(format!("{stem}.summary.json"))
    } else {
        let checkout = git_output(cwd, &["rev-parse", "--show-toplevel"])?;
        Path::new(checkout.trim_end())
            .join("docs/progress/features")
            .join(format!(
                "{}{suffix}.json",
                feature.ok_or("feature label required for default summary")?
            ))
    };
    let paths = OutputPaths {
        raw: resolved_path(&raw_path)?,
        summary: resolved_path(&summary_path)?,
    };
    if same_path(&paths.raw, &paths.summary)? {
        return Err("raw and summary outputs must be distinct".into());
    }
    Ok(paths)
}

/// Write directly from borrowed data and report buffered flush errors before
/// the caller can announce a completed artifact pair.
pub fn write_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    if let Some(parent) = path.parent().filter(|path| !path.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
    }
    let file = std::fs::File::create(path)
        .map_err(|e| format!("failed to write {}: {e}", path.display()))?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, value)
        .map_err(|e| format!("failed to serialize {}: {e}", path.display()))?;
    writer
        .write_all(b"\n")
        .and_then(|()| writer.flush())
        .map_err(|e| format!("failed to write {}: {e}", path.display()))
}

pub fn convert(
    input: &Path,
    output: &Path,
    provenance: &ConversionProvenance,
) -> Result<Summary, String> {
    if same_path(input, output)? {
        return Err("conversion input and summary output must be distinct".into());
    }
    let raw =
        std::fs::read(input).map_err(|e| format!("failed to read {}: {e}", input.display()))?;
    let summary = summarize(&raw, input, provenance)?;
    drop(raw);
    write_json(output, &summary)?;
    Ok(summary)
}

pub fn measurement_evidence(invocation: &Invocation, severe: bool) -> Value {
    let dirty = Command::new("git")
        .current_dir(&invocation.working_directory)
        .args(["status", "--porcelain"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| !out.stdout.is_empty());
    json!({"command":invocation,"dirty":dirty,
        "cli_exit":{"code":if severe {3} else {0},"source":"v3-cli status on successful artifact-pair completion; output errors instead exit 1"},
        "outer_exit":null,
        "thresholds":{"source":"v3-cli benchmark comparison constants", "work_flag_percent":super::FLAG_PERCENT,
            "work_severe_percent":super::SEVERE_PERCENT,"wall_flag_percent":super::WALL_CLOCK_FLAG_PERCENT,
            "wall_severe_percent":super::WALL_CLOCK_SEVERE_PERCENT,"wall_levels_fatal":false},
        "wall_caps":{"source":"docs/workflow.md, T13.F02 observation wall budget and T15.F01 predeclaration; closure checks, not CLI exit gates",
            "founder_seconds_per_world":10,"evolved_seconds_per_world":180,"drift_seconds_per_world":30,
            "recruitment_seconds_per_report":120,"goal_investigation_seconds":900},
        "inheritance":null})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mesh_summary_counts_every_measured_genome_field() {
        let source = json!([
            {"mesh_execution": {
                "total_node_count": 2,
                "reachable_node_count": 3,
                "executed_node_count": 5,
                "knockout_count": 7,
                "pass_cap_hits": 11,
                "passes": 41,
                "executions_per_genome": 13,
                "route_varies_with_input": true,
                "route_destination_varies": false,
                "route_varies_within_snapshot": true,
                "route_destination_varies_within_snapshot": false
            }},
            {"mesh_execution": {
                "total_node_count": 17,
                "reachable_node_count": 19,
                "executed_node_count": 23,
                "knockout_count": 29,
                "pass_cap_hits": 31,
                "passes": 43,
                "executions_per_genome": 37,
                "route_varies_with_input": false,
                "route_destination_varies": true,
                "route_varies_within_snapshot": true,
                "route_destination_varies_within_snapshot": true
            }},
            {"unmeasured": true}
        ]);

        assert_eq!(
            mesh_summary(&source),
            json!({
                "genomes": 2,
                "total": 19,
                "reachable": 22,
                "executed": 28,
                "knockout": 36,
                "capHits": 42,
                "passes": 84,
                "execs": 50,
                "routeVaries": 1,
                "routeDestinationVaries": 1,
                "routeVariesWithinSnapshot": 2,
                "routeDestinationVariesWithinSnapshot": 1
            })
        );
    }

    /// A row measured before T19.F06 lacks the within-snapshot flags; the
    /// projection omits their keys rather than reading them as false.
    #[test]
    fn mesh_summary_omits_within_snapshot_keys_no_row_carries() {
        let summary = mesh_summary(&json!([{"mesh_execution": {
            "route_varies_with_input": true,
            "route_destination_varies": true
        }}]));
        assert_eq!(summary["routeVaries"], 1);
        assert!(summary.get("routeVariesWithinSnapshot").is_none());
        assert!(summary
            .get("routeDestinationVariesWithinSnapshot")
            .is_none());
    }

    #[test]
    fn neighborhood_projects_evolved_rows_and_drops_raw_payloads() {
        let source = json!({
            "battery": {"name": "tiny"},
            "founder": {"measured": true},
            "raw_trace": [1, 2, 3],
            "evolved": {"per_seed": [{
                "generation_distribution": [1, 2],
                "seed": 7,
                "final_population_size": 3,
                "pooled_operator_rows": [{"operator": "copy"}],
                "pooled_births": 4,
                "steering_pooled": {"version": "steering-v1", "moves": 9},
                "sampled_genomes": [{"mesh_execution": {
                    "total_node_count": 2,
                    "reachable_node_count": 3,
                    "executed_node_count": 5,
                    "knockout_count": 7,
                    "pass_cap_hits": 11,
                    "passes": 41,
                    "executions_per_genome": 13,
                    "route_varies_with_input": true
                }}],
                "raw_genome_rows": ["omitted"]
            }]}
        });

        assert_eq!(
            neighborhood(&source),
            json!({
                "battery": {"name": "tiny"},
                "founder": {"measured": true},
                "evolved": {"per_seed": [{
                    "generation_distribution": [1, 2],
                    "seed": 7,
                    "final_population_size": 3,
                    "pooled_operator_rows": [{"operator": "copy"}],
                    "pooled_births": 4,
                    "steering_pooled": {"version": "steering-v1", "moves": 9},
                    "mesh_summary": {
                        "genomes": 1,
                        "total": 2,
                        "reachable": 3,
                        "executed": 5,
                        "knockout": 7,
                        "capHits": 11,
                        "passes": 41,
                        "execs": 13,
                        "routeVaries": 1,
                        "routeDestinationVaries": 0
                    }
                }]}
            })
        );
    }

    #[test]
    fn claims_keep_only_scalar_values_at_fixed_locations() {
        let source = json!({
            "deterministic": {
                "totals": {"creature_ticks": 12, "births": [3]},
                "per_creature_tick": {"vm_steps": {"raw": 4}}
            },
            "comparison": {"severe": false},
            "environment": {"wall_clock_ms_total": "9.5"}
        });

        assert_eq!(
            serde_json::to_value(claims(&source)).unwrap(),
            json!([
                {"pointer": "/deterministic/totals/creature_ticks", "value": 12},
                {"pointer": "/comparison/severe", "value": false},
                {"pointer": "/environment/wall_clock_ms_total", "value": "9.5"}
            ])
        );
    }
}
