//! Live barrier-use probe (`docs/strategy/barrier-awareness-probe-2026-10-12.md`).
//! Does a live creature's genome use barrier input to navigate? Each genome
//! dumped from a running server (`GET /v3/simulation/creature/:id`, one JSON
//! body per file in `PETRI_DUMP_DIR`) runs on the 16 calibration scenes of the
//! T22.F02 calibrated point (wall-v1, scale 2, lifetime 200, seed 1) against a
//! copy with every barrier-family input ablated: graph input edges and VM
//! `ReadInput` through `readings::ablate`, plus ring projections. The probe
//! was written when `ablate` left projections live; `ablate` now masks them
//! too, so the `*_edges_only` fields equal the full ones and the recorded
//! 98 -> 110 result is the pre-fix reading. Observation only. Ignored by default; run with
//! `PETRI_DUMP_DIR=<dir> cargo test --release -p v3-lab --test live_barrier_use -- --ignored --nocapture`.
//! Prints one `PROBE_SETUP` line, one `PROBE {json}` line per creature and a
//! `PROBE_META` summary. Dump each tile's sample right after its snapshot:
//! live lifespans are short and late dumps 404.
#![allow(
    clippy::pedantic,
    clippy::nursery,
    clippy::cast_precision_loss,
    missing_docs
)]

use std::collections::BTreeMap;
use std::io::Write;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::Instant;

use rayon::prelude::*;
use serde::Deserialize;
use serde_json::{json, Value};
use v3_core::creature::founder::founder_genome_with_age_gate;
use v3_core::creature::genome::analysis::mesh_reachable_nodes;
use v3_core::creature::genome::mesh_annotations::{derive_mesh_annotations, MeshReadClass};
use v3_core::creature::genome::{BackendDef, CreatureGenome};
use v3_core::neighborhood::input_use::catalog::Family;
use v3_lab::arena::resolve_arm;
use v3_lab::calibration::draw_scenes;
use v3_lab::eval::{evaluate_genome_observed, Boundary, SceneScore, Scoring, Setup};
use v3_lab::readings::ablate;
use v3_lab::rng::tagged;
use v3_lab::scene::{Assay, Geometry, Scene, SceneSpec};
use v3_lab::sha256_hex;

const SEED: u64 = 1;
const ARENA_SIZE: u16 = 64;
const START_ENERGY: f32 = 100.0;
const LIFETIME: u32 = 200;
const SCALE: u8 = 2;
const CALIBRATION_SCENES: u32 = 16;
/// `seeds.calibration` in docs/progress/lab/t22-f02-wall.json.
const RECORDED_CALIBRATION_SEED: u64 = 13_916_701_290_424_016_740;
/// `provenance.config_digest` in the same summary (git 809a7b48).
const RECORDED_CONFIG_DIGEST: &str =
    "2d997590cf6632743d6b604ba611256dc13aaa7455002e37bc56efe9a9dd580f";
const BARRIER_FAMILIES: [Family; 2] = [Family::NeighborBarrierRing, Family::AreaBarrierSummary];

#[derive(Deserialize)]
struct LiveCircuit {
    barrier_reader_reachable_node_count: u64,
    barrier_decision_writer_reachable_node_count: u64,
    barrier_reader_without_decision_writer_reachable_node_count: u64,
}

#[derive(Deserialize)]
struct Diagnostics {
    live_circuit: LiveCircuit,
}

#[derive(Deserialize)]
struct Dump {
    id: u64,
    generation: u64,
    genome_size: u64,
    heading: Option<Value>,
    genome: CreatureGenome,
    diagnostics: Diagnostics,
    action_log: Vec<Value>,
}

type Run = (SceneScore, Vec<Boundary>);

fn run_all(setup: &Setup, genome: &CreatureGenome, scenes: &[Scene]) -> Vec<Run> {
    scenes
        .par_iter()
        .map(|scene| {
            let (score, _, sequence) = evaluate_genome_observed(setup, genome, scene);
            (score, sequence)
        })
        .collect()
}

fn mean(values: impl Iterator<Item = f64>) -> f64 {
    let v: Vec<f64> = values.collect();
    if v.is_empty() {
        0.0
    } else {
        v.iter().sum::<f64>() / v.len() as f64
    }
}

fn totals(runs: &[Run]) -> Value {
    json!({
        "moves_attempted": runs.iter().map(|r| r.0.moves_attempted).sum::<u64>(),
        "moves_blocked": runs.iter().map(|r| r.0.moves_blocked).sum::<u64>(),
        "mean_blocked_fraction": mean(runs.iter().map(|r| r.0.blocked_fraction())),
        "food_eaten": runs.iter().map(|r| u64::from(r.0.food_eaten)).sum::<u64>(),
        "deaths": runs.iter().filter(|r| r.0.death_tick.is_some()).count(),
        "mean_score": mean(runs.iter().map(|r| r.0.score)),
        "mean_ticks": mean(runs.iter().map(|r| f64::from(r.0.ticks))),
    })
}

fn differing(a: &[Run], b: &[Run]) -> usize {
    a.iter()
        .zip(b)
        .filter(|(x, y)| x.0 != y.0 || x.1 != y.1)
        .count()
}

/// `genome` with every T11.F31 ring projection on a barrier-family reference
/// fully channel-ablated (`ablated_channels = 0xFF`, the production
/// input-use ablation state; reads 0.0, ref_idx and charges unchanged).
/// `v3_lab::readings::ablate` now applies the same mask, so on its output
/// this only re-masks and counts. Returns the copy and per-reachability
/// counts: (projections, nonzero-weight projections, reachable, reachable nonzero).
fn ablate_barrier_projections(genome: &CreatureGenome) -> (CreatureGenome, [u64; 4]) {
    let reachable = mesh_reachable_nodes(genome);
    let mut copy = genome.clone();
    let mut counts = [0u64; 4];
    for (index, node) in copy.nodes.iter_mut().enumerate() {
        let refs = &node.input_refs;
        let BackendDef::Graph(graph) = &mut node.backend_def else {
            continue;
        };
        for projection in &mut graph.projections {
            let barrier = refs
                .get(usize::from(projection.ref_idx))
                .is_some_and(|r| BARRIER_FAMILIES.contains(&Family::of(r)));
            if !barrier {
                continue;
            }
            projection.ablated_channels = 0xFF;
            let nonzero = u64::from(projection.weight != 0.0);
            let live = u64::from(reachable.contains(&index));
            counts[0] += 1;
            counts[1] += nonzero;
            counts[2] += live;
            counts[3] += live * nonzero;
        }
    }
    (copy, counts)
}

/// One creature's line, or an error string.
fn probe(setup: &Setup, scenes: &[Scene], dump: &Dump) -> Value {
    let genome = &dump.genome;
    let has_reader = dump
        .diagnostics
        .live_circuit
        .barrier_reader_reachable_node_count
        > 0;
    // Cross-check: the reader count recomputed from the deserialized genome.
    let recomputed_readers = derive_mesh_annotations(genome)
        .iter()
        .filter(|a| a.reachable && a.read_classes.contains(&MeshReadClass::Barrier))
        .count() as u64;

    let mut moves = 0u64;
    let mut move_results: BTreeMap<String, u64> = BTreeMap::new();
    for entry in &dump.action_log {
        if entry.get("action_type").and_then(Value::as_str) == Some("Move") {
            moves += 1;
            let result = match entry.get("result") {
                Some(Value::String(s)) => s.clone(),
                Some(other) => other.to_string(),
                None => "<none>".into(),
            };
            *move_results.entry(result).or_insert(0) += 1;
        }
    }

    let edge_ablated = BARRIER_FAMILIES
        .iter()
        .try_fold(genome.clone(), |g, &family| ablate(&g, family));
    let original = run_all(setup, genome, scenes);
    let original_again = run_all(setup, genome, scenes);
    let deterministic = differing(&original, &original_again) == 0;

    let mut line = json!({
        "id": dump.id,
        "generation": dump.generation,
        "genome_size": dump.genome_size,
        "heading": dump.heading,
        "barrier_reader_reachable_node_count": dump.diagnostics.live_circuit.barrier_reader_reachable_node_count,
        "barrier_decision_writer_reachable_node_count": dump.diagnostics.live_circuit.barrier_decision_writer_reachable_node_count,
        "barrier_reader_without_decision_writer_reachable_node_count": dump.diagnostics.live_circuit.barrier_reader_without_decision_writer_reachable_node_count,
        "recomputed_reader_count": recomputed_readers,
        "has_reader": has_reader,
        "deterministic": deterministic,
        "original": totals(&original),
        "invivo_moves": moves,
        "invivo_move_results": move_results,
        "invivo_actions_logged": dump.action_log.len(),
    });
    let map = line.as_object_mut().expect("object");
    let Some(edge_ablated) = edge_ablated else {
        map.insert("unsupported".into(), json!(true));
        return line;
    };
    let (full_ablated, projection_counts) = ablate_barrier_projections(&edge_ablated);
    let compare = |runs: &[Run]| {
        let scenes_differ = differing(&original, runs);
        let delta = mean(
            original
                .iter()
                .zip(runs)
                .map(|(o, a)| a.0.blocked_fraction() - o.0.blocked_fraction()),
        );
        (scenes_differ, delta)
    };
    // Full ablation: lab `ablate` (edges, ReadInput, projections) plus the
    // re-masked barrier projections.
    let full_runs = run_all(setup, &full_ablated, scenes);
    let (scenes_differ, delta) = compare(&full_runs);
    // Lab `ablate` alone: edges and ReadInput only when the probe ran, the
    // same as the full ablation since `ablate` masks projections.
    let edge_runs = run_all(setup, &edge_ablated, scenes);
    let (edge_scenes_differ, edge_delta) = compare(&edge_runs);
    map.insert("unsupported".into(), json!(false));
    map.insert(
        "ablated_identical".into(),
        json!(&edge_ablated == genome && projection_counts[0] == 0),
    );
    map.insert("barrier_projections".into(), json!(projection_counts[0]));
    map.insert(
        "barrier_projections_nonzero".into(),
        json!(projection_counts[1]),
    );
    map.insert(
        "barrier_projections_reachable".into(),
        json!(projection_counts[2]),
    );
    map.insert(
        "barrier_projections_reachable_nonzero".into(),
        json!(projection_counts[3]),
    );
    map.insert("uses".into(), json!(scenes_differ > 0));
    map.insert("scenes_differ".into(), json!(scenes_differ));
    map.insert("ablated".into(), totals(&full_runs));
    map.insert("delta_blocked".into(), json!(delta));
    map.insert("uses_edges_only".into(), json!(edge_scenes_differ > 0));
    map.insert("scenes_differ_edges_only".into(), json!(edge_scenes_differ));
    map.insert("ablated_edges_only".into(), totals(&edge_runs));
    map.insert("delta_blocked_edges_only".into(), json!(edge_delta));
    line
}

#[test]
#[ignore = "exploration probe; about a minute in release over ~400 dumps"]
fn probe_barrier_use() {
    let dir = std::env::var("PETRI_DUMP_DIR").expect("PETRI_DUMP_DIR");
    let started = Instant::now();

    let reference = resolve_arm(None, ARENA_SIZE, START_ENERGY).expect("reference arm");
    let config_digest = sha256_hex(&serde_json::to_vec(&reference).expect("config serializes"));
    let setup = Setup::new(reference.clone(), START_ENERGY, LIFETIME)
        .with_scoring(Assay::BarrierNavigation.scoring());
    assert_eq!(setup.scoring, Scoring::BARRIER_NAVIGATION);
    let calibration_seed = tagged(SEED, "calibration");
    assert_eq!(calibration_seed, RECORDED_CALIBRATION_SEED);
    let spec = SceneSpec {
        geometry: Geometry::Wall { scale: SCALE },
        size: reference.world.width,
        vision_radius: reference.runtime.perception.vision_radius,
        assay: Assay::BarrierNavigation,
    };
    let scenes = draw_scenes(calibration_seed, CALIBRATION_SCENES, &spec).expect("feasible draw");
    let redraws: Vec<u32> = scenes.iter().map(|s| s.redraws).collect();
    assert_eq!(
        redraws,
        vec![0; 16],
        "recorded calibration redraws are all 0"
    );

    // Reference genomes on the same scenes, to compare with the recorded means
    // (founder 0.6135, comparator 10.4171 at git 809a7b48).
    let founder = founder_genome_with_age_gate(
        reference.population.founder_profile,
        &reference.energy.lifecycle,
    );
    let comparator = v3_lab::comparator::built_in(Assay::BarrierNavigation, &founder);
    let founder_runs = run_all(&setup, &founder, &scenes);
    let comparator_runs = run_all(&setup, &comparator, &scenes);
    let scene_desc: Vec<Value> = scenes
        .iter()
        .map(|s| json!({"seed": s.seed, "barriers": s.barriers.iter().map(|p| [p.x, p.y]).collect::<Vec<_>>(), "food0": s.food.first().map(|p| [p.x, p.y])}))
        .collect();

    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .expect("dump dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    paths.sort();
    let mut unparsed = Vec::new();
    let mut dumps = Vec::new();
    for path in &paths {
        let parsed = std::fs::read_to_string(path)
            .map_err(|e| e.to_string())
            .and_then(|t| serde_json::from_str::<Dump>(&t).map_err(|e| e.to_string()));
        match parsed {
            Ok(dump) => dumps.push(dump),
            Err(error) => unparsed.push(json!({"file": path.file_name().map(|n| n.to_string_lossy().into_owned()), "error": error})),
        }
    }

    {
        let mut out = std::io::stdout().lock();
        writeln!(
            out,
            "PROBE_SETUP {}",
            json!({
                "seed": SEED, "calibration_seed": calibration_seed, "scale": SCALE,
                "lifetime": LIFETIME, "arena_size": ARENA_SIZE, "start_energy": START_ENERGY,
                "scenes": CALIBRATION_SCENES, "redraws": redraws,
                "vision_radius": reference.runtime.perception.vision_radius,
                "config_digest_now": config_digest, "config_digest_recorded": RECORDED_CONFIG_DIGEST,
                "families": BARRIER_FAMILIES.iter().map(|f| format!("{f:?}")).collect::<Vec<_>>(),
                "founder": totals(&founder_runs), "comparator": totals(&comparator_runs),
                "scene_geometry": scene_desc,
                "files": paths.len(), "parsed": dumps.len(), "unparsed": unparsed,
            })
        )
        .expect("stdout");
    }

    let errors = std::sync::atomic::AtomicUsize::new(0);
    let lines: Vec<Value> = dumps
        .par_iter()
        .map(|dump| {
            let line = catch_unwind(AssertUnwindSafe(|| probe(&setup, &scenes, dump)))
                .unwrap_or_else(|panic| {
                    errors.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let msg = panic
                        .downcast_ref::<String>()
                        .cloned()
                        .or_else(|| panic.downcast_ref::<&str>().map(|s| (*s).to_owned()))
                        .unwrap_or_default();
                    json!({"id": dump.id, "error": msg})
                });
            let tag = if line.get("error").is_some() {
                "PROBE_ERR"
            } else {
                "PROBE"
            };
            writeln!(std::io::stdout().lock(), "{tag} {line}").expect("stdout");
            line
        })
        .collect();

    // Summary tallies (controls only; the caller aggregates the rest).
    let ok: Vec<&Value> = lines.iter().filter(|l| l.get("error").is_none()).collect();
    let flag = |l: &Value, k: &str| l.get(k).and_then(Value::as_bool).unwrap_or(false);
    let nondeterministic = ok.iter().filter(|l| !flag(l, "deterministic")).count();
    let non_readers: Vec<&&Value> = ok.iter().filter(|l| !flag(l, "has_reader")).collect();
    let non_reader_uses = non_readers.iter().filter(|l| flag(l, "uses")).count();
    let readers = ok.len() - non_readers.len();
    let reader_uses = ok
        .iter()
        .filter(|l| flag(l, "has_reader") && flag(l, "uses"))
        .count();
    let reader_uses_edges_only = ok
        .iter()
        .filter(|l| flag(l, "has_reader") && flag(l, "uses_edges_only"))
        .count();
    let non_reader_uses_edges_only = non_readers
        .iter()
        .filter(|l| flag(l, "uses_edges_only"))
        .count();
    let with_projection = ok
        .iter()
        .filter(|l| l["barrier_projections_reachable"].as_u64().unwrap_or(0) > 0)
        .count();
    let unsupported = ok.iter().filter(|l| flag(l, "unsupported")).count();
    let reader_mismatch = ok
        .iter()
        .filter(|l| l["recomputed_reader_count"] != l["barrier_reader_reachable_node_count"])
        .count();
    let mut results: BTreeMap<String, u64> = BTreeMap::new();
    for l in &ok {
        if let Some(m) = l["invivo_move_results"].as_object() {
            for (k, v) in m {
                *results.entry(k.clone()).or_insert(0) += v.as_u64().unwrap_or(0);
            }
        }
    }
    writeln!(
        std::io::stdout().lock(),
        "PROBE_META {}",
        json!({
            "creatures": ok.len(), "errors": errors.into_inner(), "unsupported": unsupported,
            "nondeterministic": nondeterministic,
            "readers": readers, "reader_uses": reader_uses,
            "non_readers": non_readers.len(), "non_reader_uses": non_reader_uses,
            "reader_uses_edges_only": reader_uses_edges_only,
            "non_reader_uses_edges_only": non_reader_uses_edges_only,
            "creatures_with_reachable_barrier_projection": with_projection,
            "reader_count_mismatch_vs_recomputed": reader_mismatch,
            "invivo_move_results_total": results,
            "wall_seconds": started.elapsed().as_secs_f64(),
        })
    )
    .expect("stdout");
}
