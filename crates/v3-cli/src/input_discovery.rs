//! Fixed T20.F09 campaign, with a streaming bounded artifact and no retries.
mod evidence;
use crate::{
    bench::artifacts::{output_paths_with_suffix, write_json},
    recruitment::HashingWriter,
};
use serde::Serialize;
use serde_json::json;
use std::{
    io::{BufWriter, Write},
    path::PathBuf,
    time::{Duration, Instant},
};
use v3_core::{
    creature::founder::founder_genome_with_age_gate,
    neighborhood::input_discovery::{
        self as assay,
        verdict::{compare, overall, Comparison, Verdict},
        Arm, Identity, Lineage, Panel, Record, FAMILIES,
    },
};

pub const WALL_CAP_SECS: u64 = 7200;
pub const BYTE_CAP: u64 = 512 * 1024 * 1024;
const SUMMARY_CAP: usize = 4 * 1024 * 1024;
const FINAL_RECORD_RESERVE: u64 = 8192;

pub struct Options {
    pub feature: String,
    pub cwd: PathBuf,
    pub source_revision: String,
    pub raw: Option<PathBuf>,
    pub summary: Option<PathBuf>,
    pub wall_cap: Duration,
    pub byte_cap: u64,
}

#[derive(Debug, Serialize)]
pub struct Summary {
    pub kind: String,
    pub version: String,
    pub evidence_schema: String,
    pub feature: String,
    pub source_revision: String,
    pub manifest_digest: String,
    pub valid_instrument: bool,
    pub complete: bool,
    pub stop_reason: Option<String>,
    pub wall_secs: f64,
    pub prefix_secs: Option<f64>,
    pub projected_secs: Option<f64>,
    pub raw_path: PathBuf,
    pub raw_bytes: u64,
    pub raw_sha256: String,
    pub lineages_requested: usize,
    pub lineages: Vec<Lineage>,
    pub comparisons: Vec<Comparison>,
    pub general_access_s: Verdict,
    pub general_access_w: Verdict,
    pub ring_coordination: Verdict,
    pub qualified_scope: Vec<String>,
    pub candidate_source_revision: String,
}

pub struct Outcome {
    pub raw: PathBuf,
    pub summary: PathBuf,
    pub complete: bool,
    pub bytes: u64,
}

struct Stream {
    writer: HashingWriter<BufWriter<std::fs::File>>,
    cap: u64,
    error: Option<String>,
    capped: bool,
}
impl Stream {
    fn emit(&mut self, record: &impl Serialize, reserve: bool) -> bool {
        if self.error.is_some() {
            return false;
        }
        let mut bytes = match serde_json::to_vec(record) {
            Ok(bytes) => bytes,
            Err(error) => {
                self.error = Some(error.to_string());
                return false;
            }
        };
        bytes.push(b'\n');
        let limit = self
            .cap
            .saturating_sub(if reserve { FINAL_RECORD_RESERVE } else { 0 });
        if self.writer.bytes.saturating_add(bytes.len() as u64) > limit {
            self.capped = true;
            return false;
        }
        if let Err(error) = self.writer.write_all(&bytes) {
            self.error = Some(error.to_string());
            return false;
        }
        true
    }
}

#[derive(Clone, Copy)]
struct Sizes {
    discovery: u32,
    validation: u32,
    generations: u32,
    prefix: u32,
}
const PRODUCTION: Sizes = Sizes {
    discovery: 32,
    validation: 16,
    generations: 256,
    prefix: 2,
};

pub fn run(options: &Options) -> Result<Outcome, String> {
    execute(options, PRODUCTION)
}

fn execute(options: &Options, sizes: Sizes) -> Result<Outcome, String> {
    let paths = output_paths_with_suffix(
        &options.cwd,
        "input-discovery",
        "-discovery",
        Some(&options.feature),
        options.raw.as_deref(),
        options.summary.as_deref(),
    )?;
    let wall_cap = options.wall_cap.min(Duration::from_secs(WALL_CAP_SECS));
    let mut stream = Stream {
        writer: HashingWriter::create(&paths.raw)?,
        cap: options.byte_cap.min(BYTE_CAP),
        error: None,
        capped: false,
    };
    let start = Instant::now();
    let config = assay::scene_config();
    let founder =
        founder_genome_with_age_gate(config.population.founder_profile, &config.energy.lifecycle);
    let starts: Vec<_> = FAMILIES
        .iter()
        .map(|family| assay::family_start(&founder, *family))
        .collect();
    if FAMILIES.iter().zip(&starts).any(|(family, start)| {
        start
            .nodes
            .iter()
            .any(|node| node.input_refs.contains(&family.reference()))
    }) {
        return Err("invalid instrument: focal family present in frozen family start".into());
    }
    let panels: Vec<_> = FAMILIES
        .iter()
        .zip(&starts)
        .map(|(family, start)| {
            (
                Panel::new(*family, false, start),
                Panel::new(*family, true, start),
            )
        })
        .collect();
    let manifest = manifest(
        options,
        sizes,
        &config,
        &founder,
        &starts,
        &panels,
        (wall_cap, stream.cap),
    );
    let manifest_digest = assay::digest(&manifest);
    let mut reason = (!stream.emit(&manifest, true)).then(|| "raw_cap_manifest".to_string());
    let instrument = if reason.is_none() {
        reason = validate_instrument(&starts, &panels, &mut stream, start, wall_cap);
        reason.is_none()
    } else {
        false
    };
    let (schedule, prefix_count) = schedule(sizes);
    let requested = schedule.len();
    let mut rows = Vec::new();
    let mut representatives = evidence::Representatives::default();
    let mut prefix_secs = None;
    let mut projected_secs = None;
    for identity in schedule {
        if reason.is_some() {
            break;
        }
        if start.elapsed() >= wall_cap {
            reason = Some("wall_cap_between_lineages".into());
            break;
        }
        let (training, held_out) = &panels[identity.family];
        let mut facts = evidence::LineageEvidence::default();
        let row = assay::lineage(
            &starts[identity.family],
            training,
            held_out,
            identity,
            sizes.generations,
            &mut |record| match record {
                Record::Proposal(proposal) => facts
                    .observe(&proposal)
                    .is_none_or(|witness| stream.emit(&witness, true)),
                Record::Frozen(frozen) => {
                    facts.freeze(&frozen);
                    representatives.observe(&frozen);
                    true
                }
                Record::Lineage(_) => unreachable!("lineage result is returned separately"),
            },
            &|| start.elapsed() >= wall_cap,
        );
        if !row.complete {
            reason = Some(
                if stream.capped {
                    "raw_cap_partial_lineage"
                } else {
                    "wall_cap_partial_lineage"
                }
                .into(),
            );
        }
        if !stream.emit(&facts.record(&row), reason.is_none()) {
            reason = Some("raw_cap_lineage_record".into());
        }
        rows.push(row);
        if rows.len() == prefix_count && reason.is_none() {
            let elapsed = start.elapsed().as_secs_f64();
            let projection = elapsed * requested as f64 / prefix_count as f64;
            prefix_secs = Some(elapsed);
            projected_secs = Some(projection);
            if projection > wall_cap.as_secs_f64() {
                reason = Some("feasibility_projection_exceeds_wall_cap".into());
            }
        }
    }
    if let Some(error) = stream.error.take() {
        return Err(error);
    }
    let complete =
        reason.is_none() && rows.len() == requested && rows.iter().all(|row| row.complete);
    let (complete, scope) =
        write_representatives(&representatives, &mut stream, &mut reason, &rows, complete);
    stream.emit(&json!({"kind":"termination", "complete":complete,"reason":reason,"lineages":rows.len(),"lineages_requested":requested}), false);
    if let Some(error) = stream.error.take() {
        return Err(error);
    }
    stream.writer.flush().map_err(|error| error.to_string())?;
    let bytes = stream.writer.bytes;
    let sha256 = stream.writer.sha256();
    let summary = Summary {
        kind: "petri-input-discovery-summary".into(),
        version: assay::VERSION.into(),
        evidence_schema: evidence::SCHEMA.into(),
        feature: options.feature.clone(),
        source_revision: options.source_revision.clone(),
        manifest_digest,
        valid_instrument: instrument,
        complete,
        stop_reason: reason,
        wall_secs: start.elapsed().as_secs_f64(),
        prefix_secs,
        projected_secs,
        raw_path: paths.raw.clone(),
        raw_bytes: bytes,
        raw_sha256: sha256,
        lineages_requested: requested,
        lineages: rows,
        comparisons: scope.comparisons,
        general_access_s: scope.general_access_s,
        general_access_w: scope.general_access_w,
        ring_coordination: scope.ring_coordination,
        qualified_scope: scope.qualified_scope,
        candidate_source_revision: options.source_revision.clone(),
    };
    write_summary(&paths.summary, &summary)?;
    Ok(Outcome {
        raw: paths.raw,
        summary: paths.summary,
        complete: summary.complete,
        bytes,
    })
}

fn write_summary(path: &std::path::Path, summary: &Summary) -> Result<(), String> {
    if serde_json::to_vec_pretty(summary)
        .map_err(|error| error.to_string())?
        .len()
        > SUMMARY_CAP
    {
        return Err(
            "summary cap exceeded; raw record preserved, no positive verdict authorized".into(),
        );
    }
    write_json(path, summary)
}

fn manifest(
    options: &Options,
    sizes: Sizes,
    config: &v3_core::config::SimulationConfig,
    founder: &v3_core::creature::genome::CreatureGenome,
    starts: &[v3_core::creature::genome::CreatureGenome],
    panels: &[(Panel, Panel)],
    caps: (Duration, u64),
) -> serde_json::Value {
    let controls: Vec<_> = FAMILIES.iter().zip(starts).map(|(family, start)| {
        let positive = assay::instrument_control(start, *family, false);
        let zero = assay::instrument_control(start, *family, true);
        json!({"family": family.as_key(), "role": "authored_instrument_only", "positive_digest": assay::digest(&positive), "zero_digest": assay::digest(&zero), "scalar_repair": "EatFoodType reads existing fruit_only; F01 fixture unchanged"})
    }).collect();
    let panel_manifest: Vec<_> = panels
        .iter()
        .map(|(training, held_out)| (evidence::panel(training), evidence::panel(held_out)))
        .collect();
    let scene_manifest: Vec<_> = panels.iter().flat_map(|(training, held_out)| [training, held_out])
        .map(|panel| json!({"family":panel.family,"held_out":panel.held_out,"scenes":panel.scenes,"focal":panel.focal})).collect();
    json!({"kind":"manifest", "version":assay::VERSION, "evidence_schema":evidence::SCHEMA, "source_revision": options.source_revision, "config": config, "config_digest": v3_core::config::config_digest(config), "mutation_configs": Arm::ALL.map(|arm| (arm, arm.config())), "mutation_configs_digest": assay::digest(&Arm::ALL.map(|arm| (arm, arm.config()))), "founder_digest":assay::digest(founder), "family_starts": FAMILIES.iter().zip(starts).map(|(family, genome)| json!({"family":family.as_key(),"digest":assay::digest(genome),"genome_size":genome.genome_size(),"genotype":genome})).collect::<Vec<_>>(), "ring_start_version":"f09-ring-area-incumbent-v1", "ring_control_version":"f09-ring-area-incumbent-inhibition-control-v1", "panels":panel_manifest, "scene_digest":assay::digest(&scene_manifest), "controls":controls, "learning_mask":"remove Graph compute plasticity on fresh expression clones only", "seed_rule":"20090000000 + panel*1000000000 + family*100000000 + arm*1000000 + lineage*10000 + generation*2 + sibling", "sizes":{"discovery":sizes.discovery,"validation":sizes.validation,"generations":sizes.generations,"siblings":2,"prefix":sizes.prefix}, "caps":{"wall_secs":caps.0.as_secs_f64(),"raw_bytes":caps.1,"summary_bytes":SUMMARY_CAP}, "physiology":"native charges; direct Graph vote edges and perception assembly have no separate energy debit"})
}

fn write_representatives(
    representatives: &evidence::Representatives,
    stream: &mut Stream,
    reason: &mut Option<String>,
    rows: &[Lineage],
    mut complete: bool,
) -> (bool, Scope) {
    let mut scope = scopes(rows, complete);
    for candidate in representatives.qualified(
        scope.general_access_s == Verdict::Positive,
        scope.general_access_w == Verdict::Positive,
        scope.ring_coordination == Verdict::Positive,
    ) {
        if !stream.emit(
            &json!({"kind":"qualified_representative","candidate":candidate}),
            true,
        ) {
            *reason = Some("raw_cap_qualified_representative".into());
            complete = false;
            scope = scopes(rows, false);
            break;
        }
    }
    (complete, scope)
}

fn validate_instrument(
    starts: &[v3_core::creature::genome::CreatureGenome],
    panels: &[(Panel, Panel)],
    stream: &mut Stream,
    start: Instant,
    wall_cap: Duration,
) -> Option<String> {
    for ((family, founder), (training, held_out)) in FAMILIES.iter().zip(starts).zip(panels) {
        let positive = assay::instrument_control(founder, *family, false);
        let zero = assay::instrument_control(founder, *family, true);
        for panel in [training, held_out] {
            if start.elapsed() >= wall_cap {
                return Some("wall_cap_instrument".into());
            }
            let positive = assay::checkpoint(&positive, panel, panel.evaluate(&positive));
            let zero_reading = panel.evaluate(&zero);
            let valid =
                positive.graph_discovery && !assay::qualifies_score(&zero_reading, &panel.baseline);
            let record = json!({"kind":"instrument_validation", "family":family.as_key(), "held_out":panel.held_out, "valid":valid, "positive":evidence::qualification(&positive),"zero":evidence::reading(&zero_reading)});
            if !stream.emit(&record, true) {
                return Some("raw_cap_instrument".into());
            }
            if !valid {
                return Some("invalid_instrument_control".into());
            }
        }
    }
    None
}

fn schedule(sizes: Sizes) -> (Vec<Identity>, usize) {
    let mut schedule = Vec::new();
    for family in 0..3 {
        for arm in Arm::ALL {
            for lineage in 0..sizes.prefix {
                schedule.push(Identity {
                    panel: 0,
                    family,
                    arm,
                    lineage,
                });
            }
        }
    }
    let prefix_count = schedule.len();
    for family in 0..3 {
        for arm in Arm::ALL {
            for lineage in sizes.prefix..sizes.discovery {
                schedule.push(Identity {
                    panel: 0,
                    family,
                    arm,
                    lineage,
                });
            }
        }
    }
    for family in 0..3 {
        for arm in Arm::ALL {
            for lineage in 0..sizes.validation {
                schedule.push(Identity {
                    panel: 1,
                    family,
                    arm,
                    lineage,
                });
            }
        }
    }
    (schedule, prefix_count)
}

struct Scope {
    comparisons: Vec<Comparison>,
    general_access_s: Verdict,
    general_access_w: Verdict,
    ring_coordination: Verdict,
    qualified_scope: Vec<String>,
}

fn scopes(rows: &[Lineage], complete: bool) -> Scope {
    let mut comparisons = Vec::new();
    for family in 0..3 {
        for arm in [Arm::S, Arm::W, Arm::C, Arm::M] {
            for panel in 0..2 {
                comparisons.push(compare(rows, panel, family, arm, Arm::B));
            }
        }
    }
    for control in [Arm::W, Arm::M] {
        for panel in 0..2 {
            comparisons.push(compare(rows, panel, 2, Arm::C, control));
        }
    }
    let family_verdict = |arm, family, control| {
        overall(
            &comparisons
                .iter()
                .filter(|row| row.arm == arm && row.family == family && row.control == control)
                .cloned()
                .collect::<Vec<_>>(),
        )
    };
    let access = |arm| {
        let values = [
            family_verdict(arm, 0, Arm::B),
            family_verdict(arm, 1, Arm::B),
        ];
        if complete && values.iter().all(|value| *value == Verdict::Positive) {
            Verdict::Positive
        } else if values.contains(&Verdict::Negative) {
            Verdict::Negative
        } else {
            Verdict::Inconclusive
        }
    };
    let general_access_s = access(Arm::S);
    let general_access_w = access(Arm::W);
    let coordination = [
        family_verdict(Arm::C, 2, Arm::W),
        family_verdict(Arm::C, 2, Arm::M),
    ];
    let ring_coordination =
        if complete && coordination.iter().all(|value| *value == Verdict::Positive) {
            Verdict::Positive
        } else if coordination.contains(&Verdict::Negative) {
            Verdict::Negative
        } else {
            Verdict::Inconclusive
        };
    let mut qualified_scope = Vec::new();
    if general_access_s == Verdict::Positive {
        qualified_scope.push(
            "SingleChannel access: scalar FoodHere(1) channel0 and AreaFoodSummary(0) dx/dy".into(),
        );
    }
    if general_access_w == Verdict::Positive {
        qualified_scope.push(
            "WholeFamily access: scalar FoodHere(1) channel0 and AreaFoodSummary(0) dx/dy".into(),
        );
    }
    if general_access_w == Verdict::Positive && ring_coordination == Verdict::Positive {
        qualified_scope.push(
            "ring-only structured refinement: NeighborBarrierRing cardinal directions".into(),
        );
    }
    Scope {
        comparisons,
        general_access_s,
        general_access_w,
        ring_coordination,
        qualified_scope,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduced_writer_emits_compact_lineages_and_only_fixed_audit_examples() {
        let directory =
            std::env::temp_dir().join(format!("petri-f09-compact-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let options = Options {
            feature: "t20-f09-compact-fixture".into(),
            cwd: directory.clone(),
            source_revision: "test".into(),
            raw: Some(directory.join("raw.jsonl")),
            summary: Some(directory.join("summary.json")),
            wall_cap: Duration::from_secs(120),
            byte_cap: 2 * 1024 * 1024,
        };
        let outcome = execute(
            &options,
            Sizes {
                discovery: 1,
                validation: 1,
                generations: 1,
                prefix: 1,
            },
        )
        .unwrap();
        assert!(outcome.complete);
        assert!(outcome.bytes < 2 * 1024 * 1024);
        let raw = std::fs::read_to_string(&outcome.raw).unwrap();
        let mut lineage_count = 0;
        let mut audit_count = 0;
        let mut audit_bytes = 0;
        let mut shared_bytes = 0;
        let mut largest = [0usize; 3];
        let mut primary_checkpoint = [0usize; 3];
        for line in raw.lines() {
            let row: serde_json::Value = serde_json::from_str(line).unwrap();
            match row["kind"].as_str().unwrap() {
                "audit_proposal" => {
                    audit_count += 1;
                    audit_bytes += line.len() + 1;
                    assert!(line.len() < evidence::AUDIT_BYTES);
                    assert_eq!(row["proposal"]["generation"], 0);
                    assert_eq!(row["proposal"]["identity"]["lineage"], 0);
                }
                "lineage" => {
                    lineage_count += 1;
                    let family = row["result"]["identity"]["family"].as_u64().unwrap() as usize;
                    largest[family] = largest[family].max(line.len() + 1);
                    primary_checkpoint[family] = primary_checkpoint[family]
                        .max(serde_json::to_vec(&row["endpoint"]).unwrap().len());
                    assert_eq!(row["transcript"]["proposals"], 2);
                    assert!(row["endpoint"].get("genotype").is_none());
                    assert!(row["endpoint"]["training"]["intact"]
                        .get("scenes")
                        .is_none());
                    assert!(row["endpoint"]["held_out"]["intact"]
                        .get("scenes")
                        .is_none());
                }
                "manifest" | "instrument_validation" | "termination" => {
                    shared_bytes += line.len() + 1
                }
                kind => panic!("unexpected record {kind}"),
            }
        }
        assert_eq!(lineage_count, 30);
        assert_eq!(audit_count, 60);
        assert!(audit_bytes <= 960 * 1024);
        let summary: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&outcome.summary).unwrap()).unwrap();
        assert_eq!(summary["qualified_scope"], json!([]));
        // Size projection only: both primary and endpoint at each lineage, with
        // production operator histories/representatives covered by the offline audit.
        let projected = shared_bytes
            + (largest.iter().sum::<usize>() + primary_checkpoint.iter().sum::<usize>()) * 240
            + 960 * 1024;
        println!("COMPACT_WRITER_FIXTURE raw_bytes={} shared_bytes={shared_bytes} audit_count={audit_count} audit_bytes={audit_bytes} max_lineage_bytes={largest:?} full_panel_two_checkpoint_projection={projected}",outcome.bytes);
        assert!(projected < 32 * 1024 * 1024);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn byte_cap_never_writes_partial_json_and_records_explicit_stop() {
        let directory = std::env::temp_dir().join(format!("petri-f09-cap-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let options = Options {
            feature: "t20-f09-test".into(),
            cwd: directory.clone(),
            source_revision: "test".into(),
            raw: Some(directory.join("raw.jsonl")),
            summary: Some(directory.join("summary.json")),
            wall_cap: Duration::ZERO,
            byte_cap: 4096,
        };
        let outcome = execute(
            &options,
            Sizes {
                discovery: 1,
                validation: 1,
                generations: 1,
                prefix: 1,
            },
        )
        .unwrap();
        assert!(!outcome.complete);
        assert!(outcome.bytes <= 4096);
        let raw = std::fs::read_to_string(&outcome.raw).unwrap();
        for line in raw.lines() {
            serde_json::from_str::<serde_json::Value>(line).unwrap();
        }
        assert!(raw.contains("termination"));
        let summary: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&outcome.summary).unwrap()).unwrap();
        assert_eq!(summary["general_access_s"], "inconclusive");
        assert_eq!(summary["qualified_scope"], json!([]));
        std::fs::remove_dir_all(directory).unwrap();
    }
}
