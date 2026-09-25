//! Offline stored-artifact comparison parity harness (large file cleanup A3,
//! docs/specs/large-file-cleanup-2026-09-24.md). It loads stored artifacts
//! through the lossless `ComparisonInputs` path, keeps the profile
//! compatibility check, and writes every comparison the series index implies
//! as JSON. It runs no simulation. The same file runs at the base revision
//! over the v1 oracle and at the feature revision over v2; the outputs must be
//! byte-identical.
//!
//! ```sh
//! PETRI_PARITY_ROOT=<artifact root> PETRI_PARITY_SERIES=<frozen series index> \
//! PETRI_PARITY_OUT=<output json> \
//!   cargo test -p v3-cli --lib stored_artifact_comparison_parity -- --ignored
//! ```
use super::{artifacts, comparable_profile, compare_inputs, ComparisonInputs, SeriesIndex};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn env_path(name: &str) -> PathBuf {
    std::env::var_os(name)
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("{name} must be set"))
}

/// Every stored benchmark summary under `dir`, repo-relative.
fn stored_artifacts(root: &Path, dir: &Path, out: &mut Vec<String>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            stored_artifacts(root, &path, out);
        } else if path.extension().is_some_and(|ext| ext == "json") {
            let Ok(value) = serde_json::from_slice::<Value>(&std::fs::read(&path).unwrap()) else {
                continue;
            };
            if value["kind"] == artifacts::SUMMARY_KIND {
                let relative = path.strip_prefix(root).unwrap();
                out.push(relative.to_string_lossy().into_owned());
            }
        }
    }
}

fn load(root: &Path, logical: &str) -> Result<ComparisonInputs, String> {
    let bytes = std::fs::read(root.join(logical)).map_err(|e| format!("read {logical}: {e}"))?;
    artifacts::comparison_inputs_from_bytes(&bytes)
}

/// Compare two stored artifacts as a closure run compares a current report
/// against a stored reference, with the reference named by its logical path.
fn compare_pair(root: &Path, current: &str, reference: &str) -> Value {
    let (current_inputs, reference_inputs) = match (load(root, current), load(root, reference)) {
        (Ok(current), Ok(reference)) => (current, reference),
        (current, reference) => {
            return json!({ "load_error": [current.err(), reference.err()] });
        }
    };
    if comparable_profile(&reference_inputs.profile) != comparable_profile(&current_inputs.profile)
    {
        return json!({
            "profile_mismatch": format!("{:?} vs {:?}", reference_inputs.profile, current_inputs.profile)
        });
    }
    let compared = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        compare_inputs(&current_inputs, Path::new(reference), &reference_inputs)
    }));
    match compared {
        Ok(comparison) => json!({ "comparison": comparison }),
        Err(payload) => json!({
            "panic": payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
        }),
    }
}

#[test]
#[ignore = "offline parity harness; set PETRI_PARITY_ROOT, PETRI_PARITY_SERIES and PETRI_PARITY_OUT"]
fn stored_artifact_comparison_parity() {
    let root = env_path("PETRI_PARITY_ROOT");
    let index: BTreeMap<String, Value> =
        serde_json::from_slice(&std::fs::read(env_path("PETRI_PARITY_SERIES")).unwrap()).unwrap();

    let mut pairs = Vec::new();
    for name in ["gate", "goal", "goal_worlds"] {
        let Some(series) = index.get(name) else {
            continue;
        };
        let series: SeriesIndex = serde_json::from_value(series.clone()).unwrap();
        for (position, entry) in series.closed.iter().enumerate() {
            if *entry != series.epoch_baseline {
                pairs.push(json!({
                    "series": series.series, "current": entry, "against": "epoch_baseline",
                    "reference": series.epoch_baseline,
                    "result": compare_pair(&root, entry, &series.epoch_baseline),
                }));
            }
            if let Some(previous) = position.checked_sub(1).map(|p| &series.closed[p]) {
                pairs.push(json!({
                    "series": series.series, "current": entry, "against": "predecessor",
                    "reference": previous,
                    "result": compare_pair(&root, entry, previous),
                }));
            }
        }
    }

    let mut files = Vec::new();
    stored_artifacts(&root, &root.join("docs/progress"), &mut files);
    let inputs: BTreeMap<_, _> = files
        .iter()
        .map(|file| {
            let loaded = load(&root, file).map(|inputs| serde_json::to_value(inputs).unwrap());
            (
                file.clone(),
                loaded.unwrap_or_else(|e| json!({ "load_error": e })),
            )
        })
        .collect();

    let count = |key: &str| {
        pairs
            .iter()
            .filter(|pair| pair["result"].get(key).is_some())
            .count()
    };
    let summary = json!({
        "pairs": pairs.len(),
        "compared": count("comparison"),
        "profile_mismatch": count("profile_mismatch"),
        "panic": count("panic"),
        "load_error": count("load_error"),
        "files": files.len(),
        "file_load_errors": inputs.values().filter(|v| v.get("load_error").is_some()).count(),
    });
    println!("{summary}");
    let output = json!({ "summary": summary, "pairs": pairs, "inputs": inputs });
    std::fs::write(
        env_path("PETRI_PARITY_OUT"),
        serde_json::to_vec_pretty(&output).unwrap(),
    )
    .unwrap();
    // Diagnostics are written first so a failing run still leaves evidence.
    assert!(!pairs.is_empty(), "no series pairs to compare");
    assert_eq!(summary["compared"], summary["pairs"], "{summary}");
    assert_eq!(summary["file_load_errors"], 0, "{summary}");
}
