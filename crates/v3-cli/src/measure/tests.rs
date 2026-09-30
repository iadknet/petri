use v3_cli::bench::GoalCase;
use v3_cli::opportunity::{RawIdentity as OpportunityRaw, ReplicateRow, VerdictRow, WorldSummary};
use v3_cli::recruitment::RawIdentity as RecruitmentRaw;
use v3_core::neighborhood::opportunity::Verdict;
use v3_core::neighborhood::recruitment::Opportunities;
use v3_core::neighborhood::recruitment_paths::{Panel, Sizes};

use super::*;

fn recruitment_summary() -> RecruitmentSummary {
    RecruitmentSummary {
        kind: "kind".into(),
        version: "version".into(),
        feature: "feature".into(),
        pilot: true,
        panel: Panel::legacy(Sizes::PRODUCTION),
        supply_rule: "rule".into(),
        source_revision: "revision".into(),
        config_digest: "digest".into(),
        threads: 1,
        wall_secs: 1.5,
        incomplete: true,
        stop_reason: Some("wall_cap".into()),
        lineage_count: 7,
        proposal_count: 11,
        expected_proposals: 13,
        raw: RecruitmentRaw {
            path: "raw.jsonl".into(),
            bytes: 17,
            sha256: "abc".into(),
        },
        replay_check: None,
        arms: Vec::new(),
        opportunities: Opportunities::default(),
    }
}

fn replicate(replicate: u32, ticks: u64, births_total: u64) -> ReplicateRow {
    ReplicateRow {
        replicate,
        run_seed: 1,
        ticks,
        births_total,
        unattributed_births: 0,
        arms: Vec::new(),
        i_over_f: None,
        ratios: Vec::new(),
    }
}

fn verdict_row(family: &str, world: &str, verdict: Verdict, base: u64) -> VerdictRow {
    VerdictRow {
        family: family.into(),
        world: world.into(),
        verdict,
        sampled: base,
        exposed: base + 1,
        applied: base + 2,
        informative: 99,
        above_one: 99,
        below_line: 99,
        ratios: vec![Some("9.0".into())],
        pooled: None,
        p_above_one: "0.5".into(),
        p_below_line: "0.5".into(),
    }
}

fn world(name: &str, replicates: Vec<ReplicateRow>, verdicts: Vec<VerdictRow>) -> WorldSummary {
    WorldSummary {
        case: GoalCase {
            name: name.into(),
            seed: 1,
            recipe_path: "recipe.json".into(),
            config_digest: "digest".into(),
            food_type_count: 1,
        },
        replicate_config_digest: "digest".into(),
        incumbent_source_ticks: 0,
        incumbents: 0,
        founder_fallback: false,
        replicates_requested: 4,
        replicates,
        verdicts,
    }
}

fn opportunity_summary() -> OpportunitySummary {
    OpportunitySummary {
        kind: "kind".into(),
        version: "version".into(),
        feature: "feature".into(),
        pilot: true,
        source_revision: "revision".into(),
        threads: 1,
        wall_secs: 1.5,
        incomplete: false,
        stop_reason: None,
        horizon: 500,
        rules: Vec::new(),
        controller_constants: Vec::new(),
        raw: OpportunityRaw {
            path: "raw.jsonl".into(),
            bytes: 23,
            sha256: "def".into(),
        },
        worlds: vec![
            world(
                "plains",
                vec![replicate(0, 10, 3), replicate(1, 20, 5)],
                vec![
                    verdict_row("ring", "plains", Verdict::Positive, 100),
                    verdict_row("vector", "plains", Verdict::InconclusiveExposure, 200),
                ],
            ),
            world("canyon", Vec::new(), Vec::new()),
        ],
        families: Vec::new(),
        gate_favorable: true,
        vm_notes: "notes".into(),
    }
}

#[test]
fn recruitment_readings_carry_the_summary_fields_and_its_three_counts() {
    let summary = recruitment_summary();
    assert_eq!(
        recruitment_readings(&summary),
        AssayReadings {
            incomplete: true,
            stop_reason: Some("wall_cap"),
            horizon: None,
            gate_favorable: None,
            raw_sha256: "abc",
            raw_bytes: 17,
            body: json!({
                "expected_proposals": 13,
                "lineage_count": 7,
                "proposal_count": 11,
            }),
        }
    );
}

#[test]
fn opportunity_readings_sum_each_worlds_replicates_and_cut_its_verdict_rows() {
    let summary = opportunity_summary();
    assert_eq!(
        opportunity_readings(&summary),
        AssayReadings {
            incomplete: false,
            stop_reason: None,
            horizon: Some(500),
            gate_favorable: Some(true),
            raw_sha256: "def",
            raw_bytes: 23,
            body: json!({
                "worlds": [
                    {
                        "case": "plains",
                        "replicates_requested": 4,
                        "replicates": 2,
                        "ticks": 30,
                        "births_total": 8,
                    },
                    {
                        "case": "canyon",
                        "replicates_requested": 4,
                        "replicates": 0,
                        "ticks": 0,
                        "births_total": 0,
                    },
                ],
                "verdicts": [
                    {
                        "family": "ring",
                        "world": "plains",
                        "verdict": "positive",
                        "sampled": 100,
                        "exposed": 101,
                        "applied": 102,
                    },
                    {
                        "family": "vector",
                        "world": "plains",
                        "verdict": "inconclusive_exposure",
                        "sampled": 200,
                        "exposed": 201,
                        "applied": 202,
                    },
                ],
            }),
        }
    );
}

#[test]
fn a_summary_missing_a_required_field_is_an_error_naming_the_file() {
    let dir = std::env::temp_dir().join(format!("petri-measure-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("summary.json");
    let mut written = serde_json::to_value(recruitment_summary()).unwrap();
    std::fs::write(&path, written.to_string()).unwrap();
    assert!(read_summary::<RecruitmentSummary>(&path).is_ok());

    written.as_object_mut().unwrap().remove("proposal_count");
    std::fs::write(&path, written.to_string()).unwrap();
    let error = read_summary::<RecruitmentSummary>(&path).unwrap_err();
    std::fs::remove_dir_all(&dir).unwrap();
    assert!(error.contains("proposal_count"), "{error}");
    assert!(error.contains(&path.display().to_string()), "{error}");
}
